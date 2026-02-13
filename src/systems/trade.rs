use bevy::prelude::*;

use crate::components::common::SimName;
use crate::components::economy::{Resources, ResourceType, TradeRoute};
use crate::components::national_ai::{AIEvent, NationalMemory};
use crate::components::simulation_event::SimulationEvent;
use crate::components::nation_index::NationPlanetIndex;
use crate::tick::CurrentTick;

/// 星間貿易システム
///
/// 貿易ルートに沿って、余剰資源を不足惑星へ輸送する。
/// 各ルートについて以下を実行:
/// 1. 輸出元の余剰資源を特定
/// 2. 容量制限内で資源を輸送
/// 3. 距離に応じた輸送コスト（損失）を適用
pub fn trade_system(
    tick: Res<CurrentTick>,
    config: Res<crate::config::SimulationConfig>,
    trade_routes: Query<&TradeRoute>,
    mut planets: Query<(&SimName, &mut Resources)>,
    mut memory_query: Query<&mut NationalMemory>,
    index: Res<NationPlanetIndex>,
    mut events: EventWriter<SimulationEvent>,
) {
    // 各貿易ルートを処理
    for route in &trade_routes {
        if !route.active {
            continue;
        }

        let cost_rate = route.transport_cost(&config.balance);

        // 安全に二つのエンティティを同時に取得するため、
        // まず from の資源をスナップショットとして取得
        let (from_name, from_resources) = {
            if let Ok((sim_name_val, res)) = planets.get(route.from_planet) {
                (sim_name_val.0.clone(), res.clone())
            } else {
                continue;
            }
        };

        let (to_name, to_resources) = {
            if let Ok((sim_name_val, res)) = planets.get(route.to_planet) {
                (sim_name_val.0.clone(), res.clone())
            } else {
                continue;
            }
        };

        // 各資源タイプについて、余剰→不足の輸送を計算
        let resource_types = [
            ResourceType::Food,
            ResourceType::Minerals,
            ResourceType::Energy,
            ResourceType::ManufacturedGoods,
        ];

        let mut transfers: Vec<(ResourceType, f64)> = Vec::new();
        let mut total_transferred = 0.0;

        for &rt in &resource_types {
            let from_amount = from_resources.get(rt);
            let to_amount = to_resources.get(rt);

            // 余剰判定: from が to よりも十分に多い場合のみ輸送
            let surplus_threshold = 1.5; // 1.5 倍以上あれば余剰とみなす
            if from_amount > to_amount * surplus_threshold && from_amount > 50.0 {
                // 輸送量: 余剰分の 10%、ただし容量制限あり
                let surplus = from_amount - to_amount;
                let transfer = (surplus * 0.1)
                    .min(route.capacity - total_transferred)
                    .min(from_amount * 0.2); // from の 20% 以上は輸送しない

                if transfer > 0.1 {
                    transfers.push((rt, transfer));
                    total_transferred += transfer;
                }
            }

            if total_transferred >= route.capacity {
                break;
            }
        }

        // 実際の資源移動を適用
        if !transfers.is_empty() {
            if let Ok((_, mut from_res)) = planets.get_mut(route.from_planet) {
                for &(rt, amount) in &transfers {
                    let current = from_res.get(rt);
                    from_res.set(rt, current - amount);
                }
            }

            if let Ok((_, mut to_res)) = planets.get_mut(route.to_planet) {
                for &(rt, amount) in &transfers {
                    let received = amount * (1.0 - cost_rate); // 輸送コストを差し引く
                    let current = to_res.get(rt);
                    to_res.set(rt, current + received);
                }
            }

            // 表示イベントを送出（ルート単位でバッチ化）
            events.send(SimulationEvent::TradeSummary {
                tick: tick.value,
                from: from_name.clone(),
                to: to_name.clone(),
                total_amount: total_transferred,
                resources: transfers.clone(),
                cost_pct: cost_rate * 100.0,
            });

            // 歴史的イベントを記録: 貿易による相互信頼の向上
            // 惑星から国家を逆引き (インデックスを使用して高速化)
            let from_nation = index.planet_to_nation.get(&route.from_planet).copied();
            let to_nation = index.planet_to_nation.get(&route.to_planet).copied();

            if let (Some(fn_ent), Some(tn_ent)) = (from_nation, to_nation) {
                // 大規模な貿易（合計 10.0 以上）の場合のみ記録
                if total_transferred > 10.0 {
                    if let Ok(mut mem) = memory_query.get_mut(tn_ent) {
                        mem.record_event(fn_ent, AIEvent {
                            tick: tick.value,
                            description: format!("{} からの物資供給を受けました", from_name),
                            trust_impact: 0.5,
                        });
                    }
                    if let Ok(mut mem) = memory_query.get_mut(fn_ent) {
                        mem.record_event(tn_ent, AIEvent {
                            tick: tick.value,
                            description: format!("{} への輸出実績を上げました", to_name),
                            trust_impact: 0.2,
                        });
                    }
                }
            }
        }
    }
}
