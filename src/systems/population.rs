use bevy::prelude::*;

use crate::components::common::{BelongsToNation, SimName};
use crate::components::economy::Resources;
use crate::components::population::Population;
use crate::components::simulation_event::SimulationEvent;
use crate::components::technology::{TechId, TechnologyState};
use crate::tick::CurrentTick;

use crate::components::environment::PlanetaryEnvironment;

/// 人口増減システム
/// 食料供給と惑星環境に依存した成長率を計算し、人口を更新する
pub fn population_growth_system(
    tick: Res<CurrentTick>,
    config: Res<crate::config::SimulationConfig>,
    mut query: Query<(
        &SimName,
        &mut Population,
        &Resources,
        &PlanetaryEnvironment,
        &crate::components::environment::EnvironmentalHealth,
        &BelongsToNation,
    )>,
    tech_query: Query<&TechnologyState>,
    mut events: EventWriter<SimulationEvent>,
) {
    for (sim_name, mut pop, resources, env, health, belongs_to) in &mut query {
        let food_per_capita = resources.food / pop.count.max(1.0);
        let food_need_per_capita = config.balance.food_consumption_per_pop;

        // 食料充足率 (0.0 〜 1.0+)
        let food_satisfaction = (food_per_capita / food_need_per_capita).min(2.0);

        // 1. 食料によるベース成長率
        let mut growth_rate = if food_satisfaction >= 1.0 {
            pop.base_growth_rate * (0.5 + food_satisfaction * 0.5)
        } else if food_satisfaction >= 0.5 {
            pop.base_growth_rate * food_satisfaction
        } else {
            -0.001 * (1.0 - food_satisfaction)
        };

        // 2. 環境によるペナルティ
        // 放射線ペナルティ: 0.0 ~ 1.0 の radiation が、最大で成長率を 50% 低下させる
        if growth_rate > 0.0 {
            growth_rate *= 1.0 - (env.radiation * 0.5);
        }
        
        // 汚染ペナルティ: 直接成長率から減算
        growth_rate -= health.health_penalty * config.balance.health_penalty_growth_coeff;

        // 3. 人口キャパシティによる制限 (シグモイド的な抑制)
        // キャパシティの 90% を超えると急激に鈍化
        let capacity_usage = pop.count / env.population_capacity.max(1.0);
        if capacity_usage > 0.9 && growth_rate > 0.0 {
            let buffer = 1.0 - capacity_usage;
            growth_rate *= (buffer * 10.0).max(0.0);
        }

        // 4. 技術ボーナス (Biotech)
        let tech = tech_query.get(belongs_to.0).ok();
        let biotech_level = tech.map_or(0, |t| t.level(TechId::Biotech));
        let biotech_bonus = 1.0 + (biotech_level as f64 * 0.05);
        growth_rate *= biotech_bonus;

        pop.effective_growth_rate = growth_rate;

        // 飢餓カウント
        if food_satisfaction < 0.5 {
            pop.starvation_ticks += 1;
            if pop.starvation_ticks > 5 {
                pop.effective_growth_rate *= 1.0 + (pop.starvation_ticks as f64 * 0.1);
            }
        } else {
            pop.starvation_ticks = pop.starvation_ticks.saturating_sub(1);
        }

        // 人口更新
        let change = pop.count * pop.effective_growth_rate;
        pop.count = (pop.count + change).max(0.0);
        
        // キャパシティによるハードキャップ
        if pop.count > env.population_capacity {
            pop.count = env.population_capacity;
        }

        pop.last_change = change;

        // 表示イベントを送出
        events.send(SimulationEvent::PopulationReport {
            tick: tick.value,
            planet: sim_name.0.clone(),
            count: pop.count,
            change: pop.last_change,
            growth_rate: pop.effective_growth_rate,
            starvation_ticks: pop.starvation_ticks,
            habitability: env.habitability,
            capacity: env.population_capacity,
        });
    }
}
