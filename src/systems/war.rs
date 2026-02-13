use bevy::prelude::*;

use crate::components::common::BelongsToPlanet;
use crate::components::diplomacy::{DiplomaticRelation, AtWar};
use crate::components::military::MilitaryStrength;
use crate::components::population::Population;
use crate::components::economy::Resources;
use crate::components::simulation_event::SimulationEvent;
use crate::components::national_ai::{AIEvent, NationalMemory};
use crate::tick::CurrentTick;

/// 戦争システム
///
/// 1. 戦争トリガー: 外交スコアが -50 以下になると戦争開始
/// 2. 戦闘解決: 毎 Tick 艦船を消耗（攻撃力に基づく）
/// 3. 戦争影響: 人口減少、資源破壊
/// 4. 和平: 片方の艦船が 0 になるか、15 Tick 経過で停戦
pub fn war_trigger_system(
    mut commands: Commands,
    tick: Res<CurrentTick>,
    relations: Query<(Entity, &DiplomaticRelation)>,
    nations: Query<(&crate::components::common::SimName, &BelongsToPlanet)>,
    mut memory_query: Query<&mut NationalMemory>,
    existing_wars: Query<&AtWar>,
    mut events: EventWriter<SimulationEvent>,
) {
    for (_, rel) in &relations {
        // 戦争トリガー: スコアが -50 以下
        if rel.score > -50.0 {
            continue;
        }

        // すでに戦争中かチェック
        let already_at_war = existing_wars.iter().any(|war| {
            (war.enemy_nation == rel.target_nation && war.own_planet != Entity::PLACEHOLDER)
                || (war.enemy_nation == rel.owner_nation)
        });

        if already_at_war {
            continue;
        }

        // オーナー国家と対象国家の惑星を取得
        let owner_planet = nations.get(rel.owner_nation).ok().map(|(_, bp)| bp.0);
        let target_planet = nations.get(rel.target_nation).ok().map(|(_, bp)| bp.0);

        if let (Some(own_planet), Some(enemy_planet)) = (owner_planet, target_planet) {
            let owner_name = nations.get(rel.owner_nation).map(|(n, _)| n.0.clone()).unwrap_or_default();
            let target_name = nations.get(rel.target_nation).map(|(n, _)| n.0.clone()).unwrap_or_default();

            // 表示イベントを送出
            events.send(SimulationEvent::WarDeclared {
                tick: tick.value,
                aggressor: owner_name.clone(),
                defender: target_name,
                score: rel.score,
            });

            // 両国に AtWar コンポーネントを追加
            commands.entity(rel.owner_nation).insert(
                AtWar::new(rel.target_nation, enemy_planet, own_planet, tick.value)
            );
            commands.entity(rel.target_nation).insert(
                AtWar::new(rel.owner_nation, own_planet, enemy_planet, tick.value)
            );

            // 歴史的イベントを記録（信頼度の低下）
            if let Ok(mut mem) = memory_query.get_mut(rel.target_nation) {
                mem.record_event(rel.owner_nation, AIEvent {
                    tick: tick.value,
                    description: format!("{} が宣戦布告を行いました", owner_name),
                    trust_impact: -20.0,
                });
            }
        }
    }
}

/// 戦闘解決システム
///
/// 戦争中の国家間で毎 Tick 戦闘を解決する
pub fn combat_resolution_system(
    mut commands: Commands,
    tick: Res<CurrentTick>,
    mut war_nations: Query<(Entity, &crate::components::common::SimName, &mut AtWar)>,
    mut planets: Query<(&crate::components::common::SimName, &mut MilitaryStrength, &mut Population, &mut Resources)>,
    mut memory_query: Query<&mut NationalMemory>,
    mut events: EventWriter<SimulationEvent>,
) {
    // 戦争中の国家ペアを収集（借用の衝突を避ける）
    let war_data: Vec<_> = war_nations
        .iter()
        .map(|(entity, name, war)| {
            (entity, name.0.clone(), war.own_planet, war.enemy_planet, war.started_at, war.enemy_nation)
        })
        .collect();

    for (nation_entity, nation_name, own_planet, enemy_planet, war_start, enemy_nation) in &war_data {
        // 和平条件: 15 Tick 経過
        let war_duration = tick.value.saturating_sub(*war_start);
        if war_duration >= 15 {
            events.send(SimulationEvent::Ceasefire {
                tick: tick.value,
                nation: nation_name.clone(),
                duration: war_duration,
            });
            commands.entity(*nation_entity).remove::<AtWar>();
            continue;
        }

        // own_planet の MilitaryStrength で攻撃力を計算
        let (own_ships, own_power) = planets
            .get(*own_planet)
            .map(|(_, mil, _, _)| (mil.ships, mil.power))
            .unwrap_or((0, 0.0));

        if own_ships == 0 {
            events.send(SimulationEvent::Surrender {
                tick: tick.value,
                nation: nation_name.clone(),
            });
            commands.entity(*nation_entity).remove::<AtWar>();
            continue;
        }

        // 敵惑星にダメージを与える
        if let Ok((enemy_name, mut enemy_mil, mut enemy_pop, mut enemy_res)) = planets.get_mut(*enemy_planet) {
            // 戦闘ダメージ: 攻撃力の 5% が艦船を破壊
            let damage_ships = (own_power * 0.05).ceil() as u32;
            let ships_destroyed = damage_ships.min(enemy_mil.ships);
            enemy_mil.ships = enemy_mil.ships.saturating_sub(ships_destroyed);
            enemy_mil.recalculate();
            enemy_mil.in_combat = true;

            // 人口被害: 戦力に比例した民間人犠牲
            let civilian_casualties = own_power * 5.0;
            enemy_pop.count = (enemy_pop.count - civilian_casualties).max(0.0);

            // 資源破壊: 爆撃による食料・エネルギー損失
            enemy_res.food = (enemy_res.food - own_power * 0.1).max(0.0);
            enemy_res.energy = (enemy_res.energy - own_power * 0.05).max(0.0);

            // 敵国の AtWar 統計を更新（敵国が受けた被害を記録）
            if let Ok((_, _, mut enemy_war)) = war_nations.get_mut(*enemy_nation) {
                enemy_war.total_ships_lost += ships_destroyed;
                enemy_war.total_casualties += civilian_casualties;
            }

            // 表示イベントを送出
            events.send(SimulationEvent::CombatReport {
                tick: tick.value,
                attacker: nation_name.clone(),
                target: enemy_name.0.clone(),
                ships_destroyed,
                casualties: civilian_casualties,
                remaining_ships: enemy_mil.ships,
            });

            // 歴史的イベントを記録（被害に応じた信頼度低下）
            if let Ok(mut mem) = memory_query.get_mut(*enemy_nation) {
                mem.record_event(*nation_entity, AIEvent {
                    tick: tick.value,
                    description: format!("{} による爆撃で甚大な被害を受けました", nation_name),
                    trust_impact: -2.0, // 毎Tickの被害は小さめに
                });
            }
        }

        // 自軍も消耗（攻撃側のアトリション）
        let mut own_attrition = 0;
        if let Ok((_, mut own_mil, _, _)) = planets.get_mut(*own_planet) {
            own_attrition = (own_mil.ships as f64 * 0.02).ceil() as u32;
            own_mil.ships = own_mil.ships.saturating_sub(own_attrition);
            own_mil.recalculate();
            own_mil.in_combat = true;
        }

        // 自国の AtWar 統計を更新（自軍のアトリション被害を記録）
        if let Ok((_, _, mut own_war)) = war_nations.get_mut(*nation_entity) {
            own_war.total_ships_lost += own_attrition;
        }
    }
}
