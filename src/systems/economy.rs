use bevy::prelude::*;
use crate::components::common::{SimName, BelongsToNation};
use crate::components::economy::{Production, Resources, DepletableResources};
use crate::components::population::Population;
use crate::components::technology::{TechnologyState, TechId, UnlockableFeature};
use crate::components::simulation_event::SimulationEvent;
use crate::tick::CurrentTick;

use crate::components::environment::{PlanetaryEnvironment, RenewableResources, EnvironmentalHealth};

/// 資源産出・消費システム
pub fn resource_production_system(
    tick: Res<CurrentTick>,
    config: Res<crate::config::SimulationConfig>,
    mut query: Query<(
        &SimName,
        &mut Resources,
        &Production,
        &Population,
        Option<&BelongsToNation>,
        &PlanetaryEnvironment,
        &mut DepletableResources,
        &RenewableResources,
        &mut EnvironmentalHealth,
    )>,
    tech_query: Query<&TechnologyState>,
    mut events: EventWriter<SimulationEvent>,
) {
    for (sim_name, mut res, prod, pop, belongs_to, env, mut deplet, renew, mut health) in &mut query {
        // 1. 技術ボーナスの取得 (国家エンティティから取得)
        let tech = belongs_to.and_then(|b| tech_query.get(b.0).ok());

        // 労働力ボーナス: 人口が多いほど産出に微小ボーナス
        let labor_bonus = (pop.count.max(1.0).log10() / 6.0).clamp(0.5, 2.0);

        // 2. 技術ボーナス
        let (ag_bonus, mn_bonus, en_bonus, mf_bonus, _env_tech_bonus, fusion_tech_bonus, _nano_tech_bonus) = if let Some(t) = tech {
            (
                t.bonus_multiplier(TechId::Agriculture, &config.balance),
                t.bonus_multiplier(TechId::Mining, &config.balance),
                t.bonus_multiplier(TechId::Energy, &config.balance),
                t.bonus_multiplier(TechId::Manufacturing, &config.balance),
                t.bonus_multiplier(TechId::Environmental, &config.balance),
                t.bonus_multiplier(TechId::Fusion, &config.balance),
                t.bonus_multiplier(TechId::Nanotech, &config.balance),
            )
        } else {
            (1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0)
        };

        let _fusion_bonus = fusion_tech_bonus - 1.0;

        // 3. 環境・資源補正
        // 重力補正: 1.3G超で工業効率低下
        let gravity_mf_penalty = if env.gravity > 1.3 { 0.8 } else { 1.0 };
        // 土壌肥沃度補正
        let fertility_ag_mod = renew.soil_fertility;
        
        // 採掘深度ペナルティ
        let mining_depth_penalty = 1.0 - (deplet.mining_depth * 0.5);

        // === 産出 ===
        
        // 農業
        res.food += prod.food_rate * labor_bonus * ag_bonus * fertility_ag_mod;

        // 鉱物: 埋蔵量による減衰を DeepMining で緩和
        let deep_mining_level = tech.map_or(0, |t| t.level(TechId::DeepMining));
        let reserve_factor = (deplet.mineral_reserves / deplet.initial_mineral_reserves.max(1.0)).clamp(0.1, 1.0);
        let effective_reserve_factor = if deep_mining_level > 0 {
            // Level 1ごとにデバフを20%軽減（最大100%軽減 = 0.1 が 1.0 に近づく）
            // ここは固定値で良いが、調整しやすくするため 0.2 としている
            let cleanup = (deep_mining_level as f64 * 0.2).min(1.0);
            reserve_factor + (1.0 - reserve_factor) * cleanup
        } else {
            reserve_factor
        };
        
        let mut mineral_extracted = prod.mineral_rate * labor_bonus * mn_bonus * mining_depth_penalty * effective_reserve_factor;
        
        // シナジーボーナス: Mining (Nanotech + DeepMining)
        if let Some(t) = tech {
            mineral_extracted *= 1.0 + t.synergy_bonus(TechId::Mining);
        }
        mineral_extracted = mineral_extracted.min(deplet.mineral_reserves); // Still limited by actual reserves
        res.minerals += mineral_extracted;
        deplet.mineral_reserves -= mineral_extracted;
        
        // 採掘深度の進行
        if deplet.initial_mineral_reserves > 0.0 {
            deplet.mining_depth = (deplet.mining_depth + (mineral_extracted / deplet.initial_mineral_reserves * 0.05)).min(1.0);
        }

        // レアアース採掘 (鉱物採掘の 10% 程度を副産物 + 技術ボーナス)
        let mut re_extracted = mineral_extracted * 0.1 * mn_bonus;
        re_extracted = re_extracted.min(deplet.rare_earth_reserves);
        deplet.rare_earth_reserves -= re_extracted;
        // 簡略化のため、レアアースは抽出後すぐに工業生産に「割り当て」られるものとし、
        // 貯蔵はせず、このターンの工業生産にボーナスを与える。

        // エネルギー (化石燃料消費)
        // 技術レベルが上がると化石燃料依存度と産出量が変わる
        let fusion_level = tech.map_or(0, |t| t.level(TechId::Fusion));
        let energy_tech_level = tech.map_or(0, |t| t.level(TechId::Energy));
        let nano_tech_level = tech.map_or(0, |t| t.level(TechId::Nanotech));
        let dyson_level = tech.map_or(0, |t| t.level(TechId::DysonSphere));
        let mega_level = tech.map_or(0, |t| t.level(TechId::Megastructure));

        // 基礎依存度0.7. EnergyTech Lv1ごとに-0.05, Fusion Lv1ごとに-0.15
        let fossil_dependency = (0.7 - (energy_tech_level as f64 * 0.05) - (fusion_level as f64 * 0.15)).clamp(0.0, 1.0);
        
        // エネルギー (核融合ボーナス適用)
        let mut energy_produced = prod.energy_rate * labor_bonus * en_bonus * (1.0 + fusion_level as f64 * config.balance.tech_bonus_per_level) * renew.renewable_energy_output;
        
        // ダイソンスフィア: 100倍の出力 (機能アンロックが必要)
        if dyson_level > 0 && tech.map_or(false, |t| t.is_feature_unlocked(UnlockableFeature::DysonSphere)) {
            energy_produced *= 100.0;
        }

        let fossil_needed = (energy_produced * fossil_dependency * 0.1).min(deplet.fossil_fuel_reserves);
        
        if fossil_needed < (energy_produced * fossil_dependency * 0.1) && fossil_dependency > 0.0 {
            // 燃料不足による出力低下 (依存している割合分だけ低下)
            let shortage_ratio = 1.0 - (fossil_needed / (energy_produced * fossil_dependency * 0.1).max(0.01));
            energy_produced *= 1.0 - (fossil_dependency * shortage_ratio);
        }
        
        res.energy += energy_produced;
        deplet.fossil_fuel_reserves -= fossil_needed;

        // 工業品 (ナノテクボーナス適用)
        let mut mf_produced = prod.manufacturing_rate * labor_bonus * mf_bonus * gravity_mf_penalty * (1.0 + nano_tech_level as f64 * config.balance.nanotech_mf_bonus);
        
        // メガストラクチャー: +50%/Lv
        if mega_level > 0 {
            mf_produced *= 1.0 + (mega_level as f64 * 0.5);
        }

        // レアアースによる補正 (このターン採掘した分を使用)
        let re_needed = mf_produced * config.balance.rare_earth_consumption_mf_ratio;
        let re_efficiency = if re_needed > 0.0 { (re_extracted / re_needed).min(1.0) } else { 1.0 };
        // レアアースが足りないと 20% 出力低下
        mf_produced *= 0.8 + (0.2 * re_efficiency);

        res.manufactured_goods += mf_produced;

        // === 汚染の発生 ===
        health.air_pollution = (health.air_pollution + (mf_produced * config.balance.air_pollution_mf_coeff) + (fossil_needed * config.balance.air_pollution_fossil_coeff)).min(1.0);
        health.water_pollution = (health.water_pollution + (mineral_extracted * config.balance.water_pollution_mining_coeff)).min(1.0);

        // === 消費 ===
        let food_consumption = pop.count * config.balance.food_consumption_per_pop;
        res.food = (res.food - food_consumption).max(0.0);

        // 温度によるエネルギー消費増
        let temp_diff = (env.temperature - 15.0).abs();
        let energy_temp_penalty = 1.0 + (temp_diff * config.balance.energy_temp_penalty_coeff);
        let energy_consumption = pop.count * config.balance.energy_consumption_per_pop_base * energy_temp_penalty;
        res.energy = (res.energy - energy_consumption).max(0.0);

        let mineral_consumption = prod.manufacturing_rate * config.balance.mineral_consumption_mf_ratio;
        res.minerals = (res.minerals - mineral_consumption).max(0.0);

        // 表示イベントを送出
        events.send(SimulationEvent::ResourceReport {
            tick: tick.value,
            planet: sim_name.0.clone(),
            food: res.food,
            minerals: res.minerals,
            energy: res.energy,
            goods: res.manufactured_goods,
            labor_bonus,
            tech_level: tech.map(|t| t.total_level()),
        });
    }
}

/// 環境ダイナミクスシステム (劣化と回復)
pub fn environment_dynamics_system(
    config: Res<crate::config::SimulationConfig>,
    mut query: Query<(&Production, &mut RenewableResources, &mut EnvironmentalHealth, Option<&BelongsToNation>)>,
    tech_query: Query<&TechnologyState>,
) {
    for (prod, mut renew, mut health, belongs_to) in &mut query {
        let tech = belongs_to.and_then(|b| tech_query.get(b.0).ok());

        // 1. 土壌の劣化 (食料生産量に比例)
        let exploitation = prod.food_rate / 1000.0;
        renew.soil_fertility = (renew.soil_fertility - (exploitation * 0.001)).max(0.1);

        // 2. 森林による回復
        let recovery = renew.forest_coverage * 0.0005;
        renew.soil_fertility = (renew.soil_fertility + recovery).min(1.5);

        // 3. 汚染の自然浄化と技術的対策
        // 動的な浄化式: 基礎値 + (技術ボーナス) + (現在の汚染量による自然分解)
        let env_tech_bonus = tech.map_or(0.0, |t| t.level(TechId::Environmental) as f64 * config.balance.env_tech_cleanup_bonus_coeff);
        
        let base_cleanup = config.balance.base_cleanup_rate;
        let air_natural_cleanup = health.air_pollution * 0.002;
        let water_natural_cleanup = health.water_pollution * 0.002;
        
        health.air_pollution = (health.air_pollution - base_cleanup - env_tech_bonus - air_natural_cleanup).max(0.0);
        health.water_pollution = (health.water_pollution - base_cleanup - env_tech_bonus - water_natural_cleanup).max(0.0);

        // 4. 健康ペナルティの更新 (汚染レベルの合計)
        health.health_penalty = (health.air_pollution + health.water_pollution) * 0.5;
    }
}
