use bevy::prelude::*;

use crate::components::common::BelongsToPlanet;
use crate::components::diplomacy::{DiplomaticRelation, AtWar, Truce};
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
    mut relations: Query<(Entity, &mut DiplomaticRelation, Option<&Truce>)>,
    nations: Query<(&crate::components::common::SimName, &BelongsToPlanet)>,
    mut memory_query: Query<&mut NationalMemory>,
    existing_wars: Query<&AtWar>,
    mut events: EventWriter<SimulationEvent>,
) {
    let mut relations_to_war = Vec::new();

    for (rel_entity, rel, truce) in &relations {
        // 停戦期間中は宣戦布告できない
        if let Some(t) = truce {
             if tick.value < t.expiration_tick {
                 continue;
             }
        }

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

        relations_to_war.push((rel_entity, rel.owner_nation, rel.target_nation, rel.score));
    }

    for (_rel_ent, owner, target, trigger_score) in relations_to_war {
        // オーナー国家と対象国家の惑星を取得
        let owner_planet = nations.get(owner).ok().map(|(_, bp)| bp.0);
        let target_planet = nations.get(target).ok().map(|(_, bp)| bp.0);

        if let (Some(own_planet), Some(enemy_planet)) = (owner_planet, target_planet) {
            let owner_name = nations.get(owner).map(|(n, _)| n.0.clone()).unwrap_or_default();
            let target_name = nations.get(target).map(|(n, _)| n.0.clone()).unwrap_or_default();

            // 表示イベントを送出
            events.send(SimulationEvent::WarDeclared {
                tick: tick.value,
                aggressor: owner_name.clone(),
                defender: target_name,
                score: trigger_score,
            });

            // 両国に AtWar コンポーネントを追加
            commands.entity(owner).insert(
                AtWar::new(target, enemy_planet, own_planet, tick.value)
            );
            commands.entity(target).insert(
                AtWar::new(owner, own_planet, enemy_planet, tick.value)
            );

            // 外交スコアを強制的に最低値に設定（相互）
            for (_, mut rel, _) in &mut relations {
                if (rel.owner_nation == owner && rel.target_nation == target) ||
                   (rel.owner_nation == target && rel.target_nation == owner) {
                    rel.score = -100.0;
                    rel.trend = -1.0;
                }
            }

            // 歴史的イベントを記録（信頼度の低下）
            if let Ok(mut mem) = memory_query.get_mut(target) {
                mem.record_event(owner, AIEvent {
                    tick: tick.value,
                    description: format!("{} が宣戦布告を行いました", owner_name),
                    trust_impact: -20.0,
                });
            }
        }
    }
}

/// 戦闘解決システム（簡略化版：星系単位）
///
/// 1. 星系ごとに戦力を集計 (惑星 + 艦隊)
/// 2. 敵対勢力が存在する星系で戦闘発生
/// 3. ダメージ適用と占領判定
/// 4. 戦場 (Battlefront) エンティティの更新
pub fn combat_resolution_system(
    mut commands: Commands,
    tick: Res<CurrentTick>,
    mut war_nations: Query<(Entity, &crate::components::common::SimName, &mut AtWar)>,
    mut diplomatic_relations: Query<(Entity, &DiplomaticRelation)>,
    mut planets: Query<(
        Entity,
        &crate::components::common::BelongsToStarSystem,
        &crate::components::common::BelongsToNation,
        &mut MilitaryStrength,
        &mut Population,
        &mut Resources,
        Option<&crate::components::warfare::Occupied>
    ), (With<crate::components::common::Planet>, Without<crate::components::military::Fleet>)>,
    mut fleets: Query<(
        Entity,
        &crate::components::common::BelongsToStarSystem,
        &crate::components::common::BelongsToNation,
        &mut MilitaryStrength,
        Option<&crate::components::military::FleetStance>,
    ), (With<crate::components::military::Fleet>, Without<crate::components::common::Planet>)>,
    mut battlefronts: Query<(Entity, &mut crate::components::warfare::Battlefront)>,
    mut events: EventWriter<SimulationEvent>,
) {
    // 1. 各星系の勢力別戦力を集計
    // SystemEntity -> NationEntity -> TotalPower
    let mut system_powers: std::collections::HashMap<Entity, std::collections::HashMap<Entity, f64>> = std::collections::HashMap::new();
    
    // 惑星の戦力
    for (_, system, nation, mil, _, _, _) in &planets {
        let entry = system_powers.entry(system.0).or_default();
        *entry.entry(nation.0).or_default() += mil.power;
    }
    
    // 艦隊の戦力
    for (_, system, nation, mil, stance) in &fleets {
        let entry = system_powers.entry(system.0).or_default();
        let mut power = mil.power;
        
        // Stanceによる補正
        if let Some(s) = stance {
            match s {
                crate::components::military::FleetStance::Aggressive => power *= 1.2,
                crate::components::military::FleetStance::Defensive => power *= 0.8,
                crate::components::military::FleetStance::Passive => power *= 0.1, // 戦闘に参加しないがゼロではない
                crate::components::military::FleetStance::Active => {},
            }
        }
        
        *entry.entry(nation.0).or_default() += power;
    }

    // 既存の Battlefront をマッピング (System -> BattlefrontEntity)
    let mut system_battlefronts: std::collections::HashMap<Entity, Entity> = std::collections::HashMap::new();
    for (entity, bf) in &battlefronts {
        system_battlefronts.insert(bf.system, entity);
    }

    // 2. 星系ごとに戦闘処理
    for (system_entity, nations_in_system) in system_powers {
        if nations_in_system.len() < 2 {
            // 敵対勢力がいない場合、Battlefront があれば解決済みにする
            if let Some(&bf_entity) = system_battlefronts.get(&system_entity) {
                 commands.entity(bf_entity).despawn();
            }
            continue; 
        }

        // 戦争状態にあるペアを探す
        let wars: Vec<_> = war_nations.iter().map(|(e, _, w)| (e, w.enemy_nation)).collect();
        let mut combat_occurred = false;
        let mut total_attacker_str = 0.0;
        let mut total_defender_str = 0.0;

        for (nation_entity, enemy_nation_entity) in &wars {
             // 0. 和平判定 (15 tick)
             if let Ok((_, _, war_comp)) = war_nations.get(*nation_entity) {
                 let duration = tick.value.saturating_sub(war_comp.started_at);
                 if duration >= 15 {
                     // 停戦イベントは一度だけ送りたいが、現状はループ内で送ると重複する可能性がある
                     // 簡易的に、duration == 15 の時だけ送る、あるいは既存の AtWar を削除する
                     // ここでは entity を削除するコマンドを発行
                     commands.entity(*nation_entity).remove::<AtWar>();
                     events.send(SimulationEvent::Ceasefire {
                         tick: tick.value,
                         nation: format!("Nation_{:?}", nation_entity), // TODO: Name
                         duration: duration,
                     });
                     
                     // 停戦条約 (Truce) を締結: 双方の外交関係に Truce コンポーネントを追加
                     // 期間はとりあえず 50 Ticks
                     let truce_duration = 50;
                     let expiration = tick.value + truce_duration;
                     
                     for (rel_entity, rel) in diplomatic_relations.iter_mut() {
                         if (rel.owner_nation == *nation_entity && rel.target_nation == *enemy_nation_entity) ||
                            (rel.owner_nation == *enemy_nation_entity && rel.target_nation == *nation_entity) {
                                commands.entity(rel_entity).insert(Truce {
                                    with_nation: rel.target_nation,
                                    expiration_tick: expiration,
                                });
                         }
                     }

                     continue;
                 }
                 // 降伏判定 (自国惑星の戦力がゼロかつ艦隊もゼロなら)
                 // これはコストが高いので、簡易的に「自国惑星の戦力」だけ見る
                 if let Ok((_, _, _, mil, _, _, _)) = planets.get(war_comp.own_planet) {
                     if mil.ships == 0 && mil.power <= 0.0 {
                         // 艦隊も一応チェック
                         let has_fleet = fleets.iter().any(|(_, _, f_nation, _, _)| f_nation.0 == *nation_entity);
                         if !has_fleet {
                             commands.entity(*nation_entity).remove::<AtWar>();
                             events.send(SimulationEvent::Surrender {
                                 tick: tick.value,
                                 nation: format!("Nation_{:?}", nation_entity),
                             });
                             continue;
                         }
                     }
                 }
             }

            let power_a = *nations_in_system.get(nation_entity).unwrap_or(&0.0);
            let power_b = *nations_in_system.get(enemy_nation_entity).unwrap_or(&0.0);

            if power_a <= 0.0 || power_b <= 0.0 {
                continue;
            }
            
            combat_occurred = true;
            total_attacker_str += power_a; // 簡易集計
            total_defender_str += power_b;

            // ダメージ計算 (双方にダメージ)
            let damage_to_b = (power_a * 0.05).ceil(); 
            let damage_to_a = (power_b * 0.05).ceil();
            
            // 3. ダメージ適用
            apply_damage_to_system(
                &mut commands,
                &tick,
                &mut events,
                system_entity,
                *enemy_nation_entity,
                damage_to_b,
                *nation_entity, 
                &mut planets,
                &mut fleets
            );

            apply_damage_to_system(
                &mut commands,
                &tick,
                &mut events,
                system_entity,
                *nation_entity,
                damage_to_a,
                *enemy_nation_entity,
                &mut planets,
                &mut fleets
            );
        }

        // 4. Battlefront エンティティの管理
        if combat_occurred {
            // Determine status based on occupation
            // Use nation_entity (one of the combatants) to check if the system is occupied by them
            // Note: system_is_fully_occupied_by checks if "occupier" controls everything.
            // If the battle occurred, it means there are two factions. 
            // We just need to check if the battle is resolved.
            // A battle is resolved if one side is gone OR one side fully occupies the system.
            // In the loop above, we check if nations_in_system.len() < 2, which covers "one side gone".
            // Here we are inside the loop, so combat might have happened.
            
            // Logic fix: The previous code tried to use *nation_entity outside the loop.
            // We need to check if ANY of the involved nations has fully occupied the system.
            
            // Check if any nation present in the system has fully occupied it
            let mut is_resolved = false;
            for (nation_id, _) in &nations_in_system {
                if system_is_fully_occupied_by(&planets, system_entity, *nation_id) {
                     is_resolved = true;
                     break;
                }
            }

            let status = if is_resolved {
                crate::components::warfare::BattleStatus::Resolved
            } else {
                crate::components::warfare::BattleStatus::Active
            };

            if let Some(&bf_entity) = system_battlefronts.get(&system_entity) {
                // 更新
                if let Ok((_, mut bf)) = battlefronts.get_mut(bf_entity) {
                    bf.attacker_strength = total_attacker_str;
                    bf.defender_strength = total_defender_str;
                    bf.status = status;
                }
            } else {
                // 新規作成
                let bf_entity = commands.spawn(crate::components::warfare::Battlefront {
                    system: system_entity,
                    attacker_strength: total_attacker_str,
                    defender_strength: total_defender_str,
                    status: crate::components::warfare::BattleStatus::Active,
                }).id();
                
                // AtWar に追加
                for (nation_entity, _, mut war) in war_nations.iter_mut() {
                    if nations_in_system.contains_key(&nation_entity) {
                        if !war.battlefronts.contains(&bf_entity) {
                            war.battlefronts.push(bf_entity);
                        }
                    }
                }
            }
        } else {
             // 戦闘が発生しなかった場合（片方が完全に制圧された、または撤退した）
             if let Some(&bf_entity) = system_battlefronts.get(&system_entity) {
                 commands.entity(bf_entity).despawn();
             }
        }
    }
}

fn system_is_fully_occupied_by(
    planets: &Query<(
        Entity,
        &crate::components::common::BelongsToStarSystem,
        &crate::components::common::BelongsToNation,
        &mut MilitaryStrength,
        &mut Population,
        &mut Resources,
        Option<&crate::components::warfare::Occupied>
    ), (With<crate::components::common::Planet>, Without<crate::components::military::Fleet>)>,
    system_entity: Entity,
    occupier: Entity,
) -> bool {
    // 星系内の全惑星について
    // 1. 敵対国の惑星であっても、Occupied by occupier なら OK
    // 2. 味方の惑星なら OK
    // 3. 中立惑星なら無視？
    
    // 簡略化: 星系内のすべての惑星が「occupierが所有または占領している」状態かチェック
    // ただし、マルチプレイヤー（3国以上）など複雑なケースは考慮が必要
    // ここでは「occupierにとっての敵対勢力の拠点」が残っていないかを確認する

    // TODO: 厳密な判定が必要だが、一旦「星系内の全惑星が自分のものであるか、自分が占領している」場合にTrueとする
    for (_, system, nation, _, _, _, occupied) in planets {
        if system.0 == system_entity {
            if nation.0 != occupier {
                // 自分の惑星ではない場合、自分が占領していなければ False
                 if let Some(occ) = occupied {
                    if occ.occupied_by != occupier {
                        return false;
                    }
                 } else {
                     return false;
                 }
            }
        }
    }
    true
}

/// 星系内の特定国家の勢力にダメージを配分し、必要なら占領処理を行う
fn apply_damage_to_system(
    commands: &mut Commands,
    tick: &Res<CurrentTick>,
    events: &mut EventWriter<SimulationEvent>,
    system_entity: Entity,
    target_nation: Entity,
    total_damage: f64,
    attacker_nation: Entity,
    planets: &mut Query<(
        Entity,
        &crate::components::common::BelongsToStarSystem,
        &crate::components::common::BelongsToNation,
        &mut MilitaryStrength,
        &mut Population,
        &mut Resources,
        Option<&crate::components::warfare::Occupied>
    ), (With<crate::components::common::Planet>, Without<crate::components::military::Fleet>)>,
    fleets: &mut Query<(
        Entity,
        &crate::components::common::BelongsToStarSystem,
        &crate::components::common::BelongsToNation,
        &mut MilitaryStrength,
        Option<&crate::components::military::FleetStance>,
    ), (With<crate::components::military::Fleet>, Without<crate::components::common::Planet>)>,
) {
    let mut remaining_damage = total_damage as u32; // 艦船数ベースで簡易計算

    // Stanceに基づいて艦隊をグループ分け
    // 本当はソートしたいが、QueryIter はソートできないので、複数回回すか集める必要がある。
    // ここでは簡易的に、優先度順にループを回す（非効率だが可読性重視）

    // Priority 1: Defensive Fleets (Shields)
    for (entity, system, nation, mut mil, stance) in fleets.iter_mut() {
        if system.0 == system_entity && nation.0 == target_nation && mil.ships > 0 {
            if let Some(crate::components::military::FleetStance::Defensive) = stance {
                let dmg = remaining_damage.min(mil.ships);
                mil.ships -= dmg;
                mil.recalculate();
                remaining_damage -= dmg;
                
                if mil.ships == 0 {
                     commands.entity(entity).despawn();
                }
                if remaining_damage == 0 { return; }
            }
        }
    }

    // Priority 2: Aggressive & Active Fleets (and unknown stances)
    for (entity, system, nation, mut mil, stance) in fleets.iter_mut() {
        if system.0 == system_entity && nation.0 == target_nation && mil.ships > 0 {
            let s = stance.unwrap_or(&crate::components::military::FleetStance::Active);
            if matches!(s, crate::components::military::FleetStance::Active | crate::components::military::FleetStance::Aggressive) {
                let dmg = remaining_damage.min(mil.ships);
                mil.ships -= dmg;
                mil.recalculate();
                remaining_damage -= dmg;
                
                if mil.ships == 0 {
                     commands.entity(entity).despawn();
                }
                if remaining_damage == 0 { return; }
            }
        }
    }

    // 2. 惑星（駐留艦隊）へのダメージ
    // (Note: This logic was previously "2." but now comes after active fleets)
    let mut target_planets_defeated = true;
    for (_, system, nation, mut mil, mut pop, mut res, _) in planets.iter_mut() {
        if system.0 == system_entity && nation.0 == target_nation {
            if mil.ships > 0 {
                let dmg = remaining_damage.min(mil.ships);
                mil.ships -= dmg;
                mil.recalculate();
                remaining_damage -= dmg;
                
                // 副次被害
                pop.count = (pop.count - dmg as f64 * 0.1).max(0.0);
                res.energy = (res.energy - dmg as f64 * 0.05).max(0.0);
            }
            
            if mil.ships > 0 {
                target_planets_defeated = false;
            }
             if remaining_damage == 0 { 
                // ダメージ使い切っても、まだ艦船が残ってるなら制圧されていない
                 if mil.ships > 0 { target_planets_defeated = false; }
                 // ここで return すると占領判定に行かないので、breakするだけにする
                 break; 
             }
        }
    }

    // Priority 3: Passive Fleets (Last resort)
    if remaining_damage > 0 {
        for (entity, system, nation, mut mil, stance) in fleets.iter_mut() {
            if system.0 == system_entity && nation.0 == target_nation && mil.ships > 0 {
                if let Some(crate::components::military::FleetStance::Passive) = stance {
                    let dmg = remaining_damage.min(mil.ships);
                    mil.ships -= dmg;
                    mil.recalculate();
                    remaining_damage -= dmg;
                    
                    if mil.ships == 0 {
                         commands.entity(entity).despawn();
                    }
                    if remaining_damage == 0 { break; }
                }
            }
        }
    }


    // 3. 占領判定
    // ターゲットの艦隊が完全に無力化され、まだダメージ（＝攻撃力）が残っている場合
    if target_planets_defeated && remaining_damage > 0 {
        for (planet_entity, system, nation, _, _, _, occupied) in planets.iter_mut() {
            if system.0 == system_entity && nation.0 == target_nation {
                // すでに占領されていないか、あるいは別の国に占領されている
                if let Some(occ) = occupied {
                    if occ.occupied_by == attacker_nation {
                         continue; // 既に自分が占領済み
                    }
                }

                // 占領実行
                // Occupied コンポーネントを付与/更新
                commands.entity(planet_entity).insert(crate::components::warfare::Occupied {
                    original_owner: nation.0,
                    original_owner_name: format!("Nation_{:?}", nation.0), // TODO: Get name properly
                    occupied_by: attacker_nation,
                    occupied_at: tick.value,
                });

                events.send(SimulationEvent::CombatReport {
                    tick: tick.value,
                    attacker: format!("Nation_{:?}", attacker_nation),
                    target: format!("Planet_{:?}", planet_entity), // TODO: Get name
                    ships_destroyed: 0,
                    casualties: 0.0,
                    remaining_ships: 0,
                });
                
                // TODO: SimulationEvent に PlanetOccupied を追加すべきだが、今はCombatReportで代用
            }
        }
    }
}
