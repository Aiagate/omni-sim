use bevy::prelude::*;
use crate::components::common::{SimName, Nation, BelongsToNation};
use crate::components::economy::Resources;
use crate::components::national_ai::NationalCharacter;
use crate::components::population::Population;
use crate::components::simulation_event::SimulationEvent;
use crate::components::technology::{TechnologyState, TechId, TechDefinition};
use crate::tick::CurrentTick;

/// 研究システム
pub fn research_system(
    tick: Res<CurrentTick>,
    config: Res<crate::config::SimulationConfig>,
    mut nation_query: Query<(Entity, &SimName, &mut TechnologyState, &NationalCharacter), With<Nation>>,
    mut planet_query: Query<(Entity, &SimName, &Population, &mut Resources, &BelongsToNation, &crate::components::economy::DepletableResources, &crate::components::military::MilitaryStrength)>,
    mut events: EventWriter<SimulationEvent>,
) {
    // 1. 各国家の状態集計用バッファ (Entity -> Stats)
    use std::collections::HashMap;
    #[derive(Default, Clone)]
    struct NationStats {
        research_rate: f64,
        food_shortage: bool,
        energy_shortage: bool,
        mining_need: f64, // 0.0 (充満) 〜 1.0 (枯渇)
        total_military_power: f64,
        planet_count: u32,
    }
    let mut stats_map: HashMap<Entity, NationStats> = HashMap::new();

    // 2. 惑星を走査し、所属国家の状態を集計
    for (_, _, pop, mut res, belongs_to, deplet, mil) in &mut planet_query {
        // 各惑星での研究レート計算
        let pop_factor = pop.count / 1_000_000.0;
        let research_cost = 2.0_f64.min(res.manufactured_goods);
        res.manufactured_goods -= research_cost;
        let goods_bonus = research_cost * 0.5;
        let research_rate = pop_factor + goods_bonus;

        let entry = stats_map.entry(belongs_to.0).or_default();
        entry.research_rate += research_rate;
        entry.planet_count += 1;
        
        // 資源不足判定: 50 ターン分以下のストックなら不足とみなす
        let food_need = pop.count * config.balance.food_consumption_per_pop * 50.0;
        if res.food < food_need {
            entry.food_shortage = true;
        }
        let energy_need = pop.count * config.balance.energy_consumption_per_pop_base * 50.0;
        if res.energy < energy_need {
            entry.energy_shortage = true;
        }
        
        // 採掘の必要性: 埋蔵量が 50% を切ると DeepMining の必要性が高まる
        let reserve_ratio = deplet.mineral_reserves / deplet.initial_mineral_reserves.max(1.0);
        entry.mining_need = entry.mining_need.max((1.0 - reserve_ratio * 2.0).clamp(0.0, 1.0));
        
        // 軍事力
        entry.total_military_power += mil.power;
    }

    // 3. 国家ごとの研究進行処理
    for (nation_entity, nation_name, mut tech, character) in &mut nation_query {
        let stats = stats_map.get(&nation_entity).cloned().unwrap_or_default();
        tech.research_rate = stats.research_rate;
        let total_research_rate = tech.research_rate;

        // 研究ポイントの加算
        tech.research_points += total_research_rate;
        let cost = tech.cost_for_next_level(tech.current_research, &config.balance);

        if tech.research_points >= cost {
            tech.research_points -= cost;
            let current_field = tech.current_research;
            
            // レベルアップ前に研究可能だったリストを保存
            use std::collections::HashSet;
            let pre_can_research: HashSet<_> = TechDefinition::all().iter()
                .filter(|d| tech.can_research(d))
                .map(|d| d.id)
                .collect();

            // レベルアップの適用
            tech.level_up(current_field);
            if let Some(def) = TechDefinition::all().into_iter().find(|d| d.id == current_field) {
                tech.add_unlocks(&def);
            }

            // レベルアップ後に新しくアンロックされた技術をチェック
            let post_can_research: HashSet<_> = TechDefinition::all().iter()
                .filter(|d| tech.can_research(d))
                .map(|d| d.id)
                .collect();

            for unlocked_id in post_can_research.difference(&pre_can_research) {
                // Tier 2 以上の新規アンロックのみ通知
                if tech.level(*unlocked_id) == 0 {
                    events.send(SimulationEvent::TechUnlocked {
                        tick: tick.value,
                        nation: nation_name.0.clone(),
                        tech: *unlocked_id,
                    });
                }
            }

            // 次の研究分野を性格と状況に基づいて決定
            let tech_defs = TechDefinition::all();
            let mut candidates: Vec<(TechId, f64)> = Vec::new();

            for def in tech_defs {
                if tech.can_research(&def) {
                    let mut weight = 1.0;
                    
                    // 性格による補正
                    match def.id {
                        TechId::Agriculture => weight *= 0.5 + character.trade_affinity * 0.5,
                        TechId::Mining => weight *= 0.5 + character.expansionism * 0.5,
                        TechId::Energy => weight *= 1.0,
                        TechId::Manufacturing => weight *= 0.8 + character.research_focus * 0.4,
                        TechId::Military => weight *= 0.2 + character.aggression * 1.5,
                        TechId::Navigation => weight *= 0.5 + character.expansionism * 1.0,
                        TechId::Environmental => weight *= 1.0 - character.aggression * 0.5,
                        
                        TechId::Biotech => weight *= 0.6 + character.trade_affinity * 0.6,
                        TechId::DeepMining => weight *= 0.6 + character.expansionism * 0.6,
                        TechId::Fusion => weight *= 1.2,
                        TechId::Nanotech => weight *= 1.0 + character.research_focus * 0.5,
                        TechId::Shields => weight *= 0.4 + character.aggression * 1.2,
                        TechId::FTL => weight *= 0.8 + character.expansionism * 0.8,
                        _ => weight *= 1.0,
                    }

                    // 状況による補正 (Phase 4)
                    if stats.food_shortage && (def.id == TechId::Agriculture || def.id == TechId::Biotech) {
                        weight *= 3.0;
                    }
                    if stats.energy_shortage && (def.id == TechId::Energy || def.id == TechId::Fusion) {
                        weight *= 2.5;
                    }
                    if stats.mining_need > 0.5 && (def.id == TechId::Mining || def.id == TechId::DeepMining) {
                        weight *= 1.5 + stats.mining_need;
                    }
                    // 軍事力不足 (目安: 1惑星あたり平均船舶数 5隻パワー以下なら不足)
                    let avg_mil = stats.total_military_power / stats.planet_count.max(1) as f64;
                    if avg_mil < 50.0 && (def.id == TechId::Military || def.id == TechId::Shields) {
                        weight *= 2.0;
                    }

                    // Tier ボーナス
                    weight *= match def.tier {
                        1 => 1.0,
                        2 => 2.0,
                        3 => 4.0,
                        _ => 1.0,
                    };

                    candidates.push((def.id, weight));
                }
            }

            if !candidates.is_empty() {
                candidates.sort_by(|(id1, w1), (id2, w2)| {
                    let score1 = w1 / (tech.level(*id1) as f64 + 1.0);
                    let score2 = w2 / (tech.level(*id2) as f64 + 1.0);
                    score2.partial_cmp(&score1).unwrap()
                });
                tech.current_research = candidates[0].0;
            }

            events.send(SimulationEvent::TechLevelUp {
                tick: tick.value,
                planet: nation_name.0.clone(),
                total_level: tech.total_level(),
                next_field: tech.current_research,
                rate: total_research_rate,
            });
        }

        // レポート送出
        events.send(SimulationEvent::ResearchReport {
            tick: tick.value,
            planet: nation_name.0.clone(),
            levels: [
                tech.level(TechId::Agriculture),
                tech.level(TechId::Mining),
                tech.level(TechId::Energy),
                tech.level(TechId::Manufacturing),
                tech.level(TechId::Military),
                tech.level(TechId::Navigation),
                tech.level(TechId::Environmental),
                tech.level(TechId::Biotech),
                tech.level(TechId::DeepMining),
                tech.level(TechId::Fusion),
                tech.level(TechId::Nanotech),
                tech.level(TechId::Shields),
                tech.level(TechId::FTL),
                tech.level(TechId::Terraforming),
                tech.level(TechId::DysonSphere),
                tech.level(TechId::ColonyShip),
                tech.level(TechId::Megastructure),
                tech.level(TechId::PsiTech),
            ],
            rate: total_research_rate,
            progress: tech.research_points,
            cost: tech.cost_for_next_level(tech.current_research, &config.balance),
        });
    }
}
