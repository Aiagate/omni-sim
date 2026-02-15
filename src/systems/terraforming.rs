use bevy::prelude::*;
use crate::components::common::{SimName, BelongsToNation};
use crate::components::economy::Resources;
use crate::components::environment::{PlanetaryEnvironment, AtmosphereType};
use crate::components::terraforming::{TerraformingProject, TerraformingPhase};
use crate::components::technology::{TechnologyState, TechId};
use crate::components::simulation_event::SimulationEvent;
use crate::tick::CurrentTick;

/// テラフォーミング進行システム
pub fn terraforming_system(
    tick: Res<CurrentTick>,
    _config: Res<crate::config::SimulationConfig>,
    mut query: Query<(
        Entity,
        &SimName,
        &mut Resources,
        &mut PlanetaryEnvironment,
        &mut TerraformingProject,
        &BelongsToNation,
    )>,
    tech_query: Query<&TechnologyState>,
    mut events: EventWriter<SimulationEvent>,
) {
    for (_entity, name, mut res, mut env, mut project, belongs_to) in &mut query {
        if project.current_phase == TerraformingPhase::Complete {
            continue;
        }

        // 国家の技術レベルをチェック（テラフォーミングには環境技術が必要）
        let tech = match tech_query.get(belongs_to.0) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let env_tech_level = tech.level(TechId::Environmental);
        if env_tech_level < _config.balance.terraforming_min_env_tech {
            // 環境技術レベル不足
            continue;
        }

        // プロジェクトが未開始ならフェーズ1に設定
        if project.current_phase == TerraformingPhase::None {
            project.current_phase = TerraformingPhase::AtmosphericDensity;
            project.progress = 0.0;
        }

        // 各フェーズのコストと進捗速度を BalanceConfig から取得
        let (energy_cost, goods_cost, food_cost, progress_per_tick) = match project.current_phase {
            TerraformingPhase::AtmosphericDensity => _config.balance.tf_phase_atmospheric_costs,
            TerraformingPhase::TemperatureControl => _config.balance.tf_phase_temp_costs,
            TerraformingPhase::WaterManagement => _config.balance.tf_phase_water_costs,
            TerraformingPhase::BiologicalSeeding => _config.balance.tf_phase_bio_costs,
            _ => (0.0, 0.0, 0.0, 0.0),
        };

        // 資源チェックと消費
        if res.energy >= energy_cost && res.manufactured_goods >= goods_cost && res.food >= food_cost {
            res.energy -= energy_cost;
            res.manufactured_goods -= goods_cost;
            res.food -= food_cost;
            
            let mut progress_per_tick = progress_per_tick;
            
            // Tier 3 Terraforming 技術による加速 (機能アンロックが必要)
            let terraforming_level = tech.level(TechId::Terraforming);
            if terraforming_level > 0 && tech.is_feature_unlocked(crate::components::technology::UnlockableFeature::Terraforming) {
                progress_per_tick *= 1.0 + (terraforming_level as f64 * 1.0); // Lv1で倍速
            }

            project.progress += progress_per_tick;

            // 進捗イベント（10% 刻みくらいで出すといいが、重要度 Low なので毎 Tick でも可）
            if tick.value % 5 == 0 {
                events.send(SimulationEvent::TerraformingProgress {
                    tick: tick.value,
                    planet: name.0.clone(),
                    phase: project.current_phase.description().to_string(),
                    progress: project.progress,
                });
            }

            // フェーズ完了判定
            if project.progress >= 1.0 {
                let old_phase = project.current_phase;
                apply_phase_completion_effects(&mut env, old_phase);
                
                project.current_phase = old_phase.next();
                project.progress = 0.0;
                
                events.send(SimulationEvent::TerraformingPhaseComplete {
                    tick: tick.value,
                    planet: name.0.clone(),
                    phase: old_phase.description().to_string(),
                });
            }
        }
    }
}

/// フェーズ完了時の環境変化を適用
fn apply_phase_completion_effects(env: &mut PlanetaryEnvironment, phase: TerraformingPhase) {
    match phase {
        TerraformingPhase::AtmosphericDensity => {
            // 大気圧を 1.0 に近づけ、大気タイプを改善
            env.atmospheric_pressure = (env.atmospheric_pressure + 0.5 * (1.0 - env.atmospheric_pressure)).clamp(0.1, 2.0);
            if env.atmosphere == AtmosphereType::None || env.atmosphere == AtmosphereType::Thin {
                env.atmosphere = AtmosphereType::Thin; // 仮の段階
            }
        }
        TerraformingPhase::TemperatureControl => {
            // 気温を 20度に近づける
            env.temperature += (20.0 - env.temperature) * 0.5;
            if env.atmosphere == AtmosphereType::CarbonDioxide {
                env.atmosphere = AtmosphereType::EarthLike; // CO2を固定 or 排出
            }
        }
        TerraformingPhase::WaterManagement => {
            // 水の存在率を 0.5 に近づける
            env.water_coverage = (env.water_coverage + 0.5 * (0.5 - env.water_coverage)).clamp(0.0, 1.0);
        }
        TerraformingPhase::BiologicalSeeding => {
            // 最終的に EarthLike に
            env.atmosphere = AtmosphereType::EarthLike;
            env.radiation = (env.radiation * 0.5).max(0.05); // オゾン層形成的な
        }
        _ => {}
    }
    // 最後に居住性を再計算
    env.update_habitability();
}
