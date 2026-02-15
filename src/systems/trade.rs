use bevy::prelude::*;
use crate::components::common::{SimName, Nation, BelongsToNation, BelongsToStarSystem};
use crate::components::economy::{Resources, ResourceType, TradeRoute, CargoFleet};
use crate::components::space::Position;
use crate::components::national_ai::{AIEvent, NationalMemory};
use crate::components::simulation_event::SimulationEvent;
use crate::components::nation_index::NationPlanetIndex;
use crate::components::technology::{TechnologyState, TechId};
use crate::tick::CurrentTick;

/// 星間貿易ディスパッチシステム
///
/// 貿易ルートに沿って、余剰資源を不足惑星へ輸送するための船団（CargoFleet）を派遣する。
/// 実際の資源移動は船団が目的地に到着した際に行われる。
pub fn trade_dispatch_system(
    mut commands: Commands,
    tick: Res<CurrentTick>,
    config: Res<crate::config::SimulationConfig>,
    trade_routes: Query<&TradeRoute>,
    mut planets: Query<(&SimName, &mut Resources, &BelongsToNation, &BelongsToStarSystem)>,
    star_systems: Query<&crate::components::space::Position, With<crate::components::common::StarSystem>>,
    nation_query: Query<&TechnologyState, With<Nation>>,
    // 貿易頻度調整用: 以前と同様 10 tick に 1 回
) {
    if tick.value % 10 != 0 {
        return;
    }

    for route in &trade_routes {
        if !route.active || route.from_planet == route.to_planet {
            continue;
        }

        // 1. 必要なデータをコピーして取得 (借用を保持しない)
        // Note: We use `get` (returns &mut because Query is mutable) but we don't need mutation yet.
        // Copying values to avoid borrowing issues.
        let (from_name, from_res_clone, belongs_to, from_sys) = {
             if let Ok((name, res, belongs, sys)) = planets.get(route.from_planet) {
                 (name.0.clone(), res.clone(), belongs.0, sys.0)
             } else { continue; }
        };
        
        // Destination check
        let (to_name, to_res_clone, to_sys_ent) = {
             if let Ok((name, res, _, sys)) = planets.get(route.to_planet) {
                 (name.0.clone(), res.clone(), sys.0)
             } else { continue; }
        };

        // 星系座標の取得
        let start_pos = if let Ok(p) = star_systems.get(from_sys) { *p } else { continue; };
        let end_pos = if let Ok(p) = star_systems.get(to_sys_ent) { *p } else { continue; };

        // 技術レベル計算
        let tech = nation_query.get(belongs_to).ok();
        let nav_tech_level = tech.map(|t| t.level(TechId::Navigation)).unwrap_or(0);
        let ftl_tech_level = tech.map(|t| t.level(TechId::FTL)).unwrap_or(0);
        let speed = 0.2 + (nav_tech_level as f64 * 0.05) + (ftl_tech_level as f64 * 0.2);
        let route_capacity = route.capacity * (1.0 + nav_tech_level as f64 * config.balance.navigation_tech_capacity_bonus) * 10.0;

        // 輸送量の計算
        let mut transfers = Vec::new();
        let mut total_transferred = 0.0;
        let resource_types = [ResourceType::Food, ResourceType::Minerals, ResourceType::Energy, ResourceType::ManufacturedGoods];

        for &rt in &resource_types {
            let from_amt = from_res_clone.get(rt);
            let to_amt = to_res_clone.get(rt);
            
            if from_amt > to_amt * 1.5 && from_amt > 50.0 {
                let transfer = ((from_amt - to_amt) * 0.1).min(route_capacity - total_transferred).min(from_amt * 0.2);
                if transfer > 0.1 {
                    transfers.push((rt, transfer));
                    total_transferred += transfer;
                }
            }
            if total_transferred >= route_capacity { break; }
        }

        if transfers.is_empty() { continue; }

        // 資源の引き落とし
        // Now using get_mut to modify the source planet
        if let Ok((_, mut res, _, _)) = planets.get_mut(route.from_planet) {
            for &(rt, amt) in &transfers {
                let current = res.get(rt);
                res.set(rt, current - amt);
            }
        }

        // CargoFleet の生成
        commands.spawn((
            CargoFleet {
                origin: route.from_planet,
                origin_name: from_name.clone(),
                destination: route.to_planet,
                target_pos: Vec3::new(end_pos.x as f32, end_pos.y as f32, end_pos.z as f32),
                resources: transfers,
                speed,
                total_distance: start_pos.distance_to(&end_pos),
                traveled_distance: 0.0,
                launch_tick: tick.value,
            },
            // 現在位置（初期位置）
            start_pos, 
            SimName(format!("Cargo Fleet: {} -> {}", from_name, to_name)),
        ));
    }
}


/// 貨物船団移動・到着処理システム
pub fn cargo_movement_system(
    mut commands: Commands,
    mut fleets: Query<(Entity, &mut CargoFleet, &mut crate::components::space::Position)>,
    mut planets: Query<(&SimName, &mut Resources)>,
    mut events: EventWriter<SimulationEvent>,
    tick: Res<CurrentTick>,
    _memory_query: Query<&mut NationalMemory>,
    _index: Res<NationPlanetIndex>,
) {
    for (entity, mut fleet, mut pos) in &mut fleets {
        // 移動処理
        let current_vec = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
        let target_vec = fleet.target_pos;
        let direction = target_vec - current_vec;
        let dist_sq = direction.length_squared();
        let move_dist = fleet.speed as f32; // speed は f64 だが Vec3 演算のため f32 に

        if dist_sq <= move_dist * move_dist {
            // 到着処理
            if let Ok((to_name, mut to_res)) = planets.get_mut(fleet.destination) {
                // 技術等による損失計算（今回は簡易的に距離ベースの固定損失）
                // 距離 100 光年で 50% 損失など
                // コストレート計算
                let cost_rate = (fleet.total_distance * 0.005).min(0.8); // 1lyあたり0.5%ロス

                let mut applied_transfers = Vec::new();
                let mut total_arrived = 0.0;

                for (rt, amt) in &fleet.resources {
                    let arrived_amt = amt * (1.0 - cost_rate);
                    let current = to_res.get(*rt);
                    to_res.set(*rt, current + arrived_amt);
                    
                    applied_transfers.push((*rt, arrived_amt));
                    total_arrived += arrived_amt;
                }

                // イベント送出（到着時）
                events.send(SimulationEvent::TradeSummary {
                    tick: tick.value,
                    from: fleet.origin_name.clone(),
                    to: to_name.0.clone(),
                    total_amount: total_arrived,
                    resources: applied_transfers,
                    cost_pct: cost_rate * 100.0,
                });

                // 信頼度更新もここで行うべきだが、今回は省略または Origin の所有者取得が必要
                // 必要なら planets.get(fleet.origin) で取れる
            }

            // フリート消滅
            commands.entity(entity).despawn();
        } else {
            // 移動継続
            let dir = direction.normalize();
            let new_pos = current_vec + dir * move_dist;
            
            pos.x = new_pos.x as f64;
            pos.y = new_pos.y as f64;
            pos.z = new_pos.z as f64;
            
            fleet.traveled_distance += move_dist as f64;
        }
    }
}
