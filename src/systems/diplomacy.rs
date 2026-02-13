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
    planet_query: Query<(&MilitaryStrength, &DepletableResources)>,
    mut events: EventWriter<SimulationEvent>,
) {
    for (mut rel, rel_name) in &mut rel_query {
        // 自国の性格、状態、記憶、派閥、技術を取得
        let Ok((character, belongs, memory, factions, own_tech)) = nation_query.get(rel.owner_nation) else {
            continue;
        };
        let memory: &NationalMemory = memory;
        let factions: &InternalFactions = factions;
        let Ok((own_mil, deplet)) = planet_query.get(belongs.0) else {
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
        let Ok((target_mil, _)) = planet_query.get(target_belongs.0) else {
            continue;
        };

        let mut drift = 0.0;

        // 1. 信頼度 (Trust) の影響
        let trust = memory.calculate_trust(rel.target_nation);
        drift += trust * 0.01;

        // 2. 基本トレンド: 好戦性が高いほど、あるいは自然に微減する
        drift -= 0.1;
        drift -= effective_aggression * 0.2;

        // 3. 軍事力格差の影響
        let power_ratio = own_mil.power / target_mil.power.max(1.0);
        if effective_aggression > 0.6 && power_ratio > 1.5 {
            drift -= 0.5 * effective_aggression;
        } else if power_ratio < 0.5 {
            drift += (1.0 - effective_aggression) * 0.3;
        }

        // 4. 資源不足の影響 (鉱物が 20% を切ると攻撃性が増す)
        let mineral_ratio = deplet.mineral_reserves / deplet.initial_mineral_reserves.max(1.0);
        if mineral_ratio < 0.2 {
            // 資源不足による焦り
            let desperation = (0.2 - mineral_ratio) * 5.0; // 0.0 ~ 1.0
            drift -= desperation * effective_aggression * 0.5;
        }

        // 4. 技術格差
        let tech_gap = target_tech.total_level() as f64 - own_tech.total_level() as f64;
        if tech_gap > 0.0 {
            drift += effective_research * tech_gap * 0.05;
        }

        // 4. 周期的・ランダムな変動（柔軟性）
        let periodic = (tick.value as f64 * 0.07).sin() * character.diplomacy_flexibility * 0.5;
        drift += periodic;

        // ドリフトの適用
        rel.trend = drift;
        rel.score = (rel.score + drift).clamp(-100.0, 100.0);

        // 表示イベントを送出
        events.send(SimulationEvent::DiplomacyReport {
            tick: tick.value,
            relation: rel_name.0.clone(),
            score: rel.score,
            trend: rel.trend,
        });
    }
}
