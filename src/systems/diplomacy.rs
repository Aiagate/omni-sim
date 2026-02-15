use bevy::prelude::*;

use crate::components::common::{BelongsToPlanet, SimName};
use crate::components::diplomacy::DiplomaticRelation;
use crate::components::economy::DepletableResources;
use crate::components::military::MilitaryStrength;
use crate::components::national_ai::{InternalFactions, NationalCharacter, NationalMemory};
use crate::components::simulation_event::SimulationEvent;
use crate::components::technology::TechnologyState;
use crate::tick::CurrentTick;

/// 外交システム
///
/// 毎 Tick、各国家の性格パラメータに基づいて外交関係スコアを更新する。
pub fn diplomacy_system(
    tick: Res<CurrentTick>,
    mut rel_query: Query<(&mut DiplomaticRelation, &SimName)>,
    nation_query: Query<(
        &NationalCharacter,
        &BelongsToPlanet,
        &NationalMemory,
        &InternalFactions,
        &TechnologyState,
    )>,
    planet_query: Query<(Entity, &MilitaryStrength, &DepletableResources, Option<&crate::components::common::BelongsToNation>)>,
    trade_routes: Query<&crate::components::economy::TradeRoute>,
    mut events: EventWriter<SimulationEvent>,
) {
    // 貿易ルートのキャッシュを作成 (Nation -> Nation -> count)
    // 双方向のルートがあるので、(Entity, Entity) -> count で管理
    // ルートは FromPlanet -> ToPlanet だが、Nation に変換する必要がある
    let mut trade_counts: std::collections::HashMap<(Entity, Entity), usize> = std::collections::HashMap::new();
    
    // Planet -> Nation マップを作成
    let mut planet_to_nation: std::collections::HashMap<Entity, Entity> = std::collections::HashMap::new();
    for (entity, _, _, nation_opt) in planet_query.iter() {
         if let Some(n) = nation_opt {
             planet_to_nation.insert(entity, n.0);
         }
    }

    for route in &trade_routes {
        if let (Some(&nation_a), Some(&nation_b)) = (planet_to_nation.get(&route.from_planet), planet_to_nation.get(&route.to_planet)) {
            if nation_a != nation_b {
                *trade_counts.entry((nation_a, nation_b)).or_default() += 1;
                *trade_counts.entry((nation_b, nation_a)).or_default() += 1;
            }
        }
    }


    for (mut rel, rel_name) in &mut rel_query {
        // 自国の性格、状態、記憶、派閥、技術を取得
        let Ok((character, belongs, memory, factions, own_tech)) = nation_query.get(rel.owner_nation) else {
            continue;
        };
        let memory: &NationalMemory = memory;
        let factions: &InternalFactions = factions;
        let Ok((_, own_mil, deplet, _)) = planet_query.get(belongs.0) else {
            continue;
        };

        // 派閥による性格の補正を計算
        let (agg_off, _trait_off, res_off) = factions.get_character_offset();
        let effective_aggression = (character.aggression + agg_off).clamp(0.0, 1.0);
        let effective_research = (character.research_focus + res_off).clamp(0.0, 1.0);

        // 相手国の状態を取得
        let Ok((_, target_belongs, _, _, target_tech)) = nation_query.get(rel.target_nation) else {
            continue;
        };
        let Ok((_, target_mil, _, _)) = planet_query.get(target_belongs.0) else {
            continue;
        };

        let mut drift = 0.0;

        // 1. 信頼度 (Trust) の影響
        let trust = memory.calculate_trust(rel.target_nation);
        drift += trust * 0.01;

        // 2. 自然回復（中立への回帰）
        // スコアが悪いときは回復し、良いときは減少する（自然な鎮静化）
        // ただし、非常に遅く
        drift += (0.0 - rel.score) * 0.005;

        // 3. 攻撃性の影響 (無差別な減衰ではなく、係数で調整)
        // 攻撃性が高い = 関係改善が遅く、悪化は早い
        if drift > 0.0 {
            drift *= (1.0 - effective_aggression * 0.5);
        } else {
            drift *= (1.0 + effective_aggression * 0.5);
        }
        
        // 基本的な不信感（定数減衰）は削除し、攻撃性が高い場合のみ微減させる
        if effective_aggression > 0.7 {
             drift -= 0.05 * (effective_aggression - 0.7);
        }

        // 4. 軍事力格差の影響
        let power_ratio = own_mil.power / target_mil.power.max(1.0);
        if effective_aggression > 0.6 && power_ratio > 1.5 {
            // 強気に出る（関係悪化）
            drift -= 0.1 * effective_aggression;
        } else if power_ratio < 0.5 {
            // 脅威を感じて警戒（関係悪化）または媚びる（関係改善）？
            // ここでは警戒として悪化させるが、平和主義なら改善するかも
            if effective_aggression < 0.3 {
                 drift += 0.05; // 友好関係を求める
            } else {
                 drift -= 0.05; // 警戒
            }
        }

        // 5. 資源不足の影響 (鉱物が 20% を切ると攻撃性が増す)
        let mineral_ratio = deplet.mineral_reserves / deplet.initial_mineral_reserves.max(1.0);
        if mineral_ratio < 0.2 {
            // 資源不足による焦り
            let desperation = (0.2 - mineral_ratio) * 5.0; // 0.0 ~ 1.0
            drift -= desperation * effective_aggression * 0.5;
        }

        // 6. 技術格差
        let tech_gap = target_tech.total_level() as f64 - own_tech.total_level() as f64;
        if tech_gap > 0.0 {
            drift += effective_research * tech_gap * 0.05;
        }

        // 7. 貿易関係のボーナス
        let trade_count = trade_counts.get(&(rel.owner_nation, rel.target_nation)).unwrap_or(&0);
        if *trade_count > 0 {
            drift += *trade_count as f64 * 0.05 * character.trade_affinity;
        }

        // 8. 周期的・ランダムな変動（柔軟性）
        let periodic = (tick.value as f64 * 0.07).sin() * character.diplomacy_flexibility * 0.2; // 係数を0.5 -> 0.2へ減少
        drift += periodic;

        // ドリフトの適用
        rel.trend = drift;
        rel.score = (rel.score + drift).clamp(-100.0, 100.0);

        // 表示イベントを送出
        // 頻繁すぎるので変更があった場合のみ、あるいは一定間隔のみにするのが良いが
        // 現状は毎Tick送る仕様のようなので維持
        events.send(SimulationEvent::DiplomacyReport {
            tick: tick.value,
            relation: rel_name.0.clone(),
            score: rel.score,
            trend: rel.trend,
        });
    }
}
