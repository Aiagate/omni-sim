use bevy::prelude::*;

use crate::components::common::{SimName, BelongsToNation};
use crate::components::economy::{Resources, DepletableResources};
use crate::components::population::Population;
use crate::components::technology::TechnologyState;
use crate::components::environment::{PlanetaryEnvironment, EnvironmentalHealth};
use crate::components::events::{EventKind, EventEffect, EventLog, EventRecord};
use crate::components::simulation_event::SimulationEvent;
use crate::config::SimulationConfig;
use crate::tick::CurrentTick;

/// イベントシステム
///
/// シード + Tick + 惑星インデックスから決定論的にイベントを生成
/// 毎 Tick 各惑星に対して、確率的にイベントが発生するかを判定
pub fn event_system(
    tick: Res<CurrentTick>,
    config: Res<SimulationConfig>,
    mut event_log: ResMut<EventLog>,
    mut planets: Query<(
        Entity,
        &SimName,
        &mut Population,
        &mut Resources,
        &PlanetaryEnvironment,
        &mut DepletableResources,
        &EnvironmentalHealth,
        Option<&BelongsToNation>,
    )>,
    mut nations: Query<&mut TechnologyState>,
    mut sim_events: EventWriter<SimulationEvent>,
) {
    for (idx, (_entity, name, mut pop, mut res, env, mut deplet, health, belongs_to)) in planets.iter_mut().enumerate() {
        // 決定論的ハッシュ: seed × tick × planet_index
        let hash = deterministic_hash(config.seed, tick.value, idx as u64);

        // イベント発生確率: 5% / Tick
        if hash % 20 != 0 {
            continue;
        }

        // --- ウェイトベースのイベント選択 ---
        let mut weights = Vec::new();
        for kind in EventKind::all() {
            let base_weight = 1.0;
            let weight = match kind {
                EventKind::Earthquake => base_weight * env.tectonic_activity * 10.0,
                EventKind::RadiationStorm => base_weight * env.radiation * 5.0,
                EventKind::MeteoriteImpact => base_weight * env.meteorite_risk * 3.0,
                EventKind::EnergyCrisis => {
                    let fossil_ratio = deplet.fossil_fuel_reserves / deplet.initial_fossil_fuel_reserves.max(1.0);
                    base_weight * (1.0 + (1.0 - fossil_ratio) * 10.0)
                },
                EventKind::EnvironmentalDisaster => {
                    let pollution = health.air_pollution + health.water_pollution;
                    if pollution > 0.5 { base_weight * (pollution - 0.5) * 50.0 } else { 0.0 }
                },
                _ => base_weight,
            };
            weights.push((*kind, weight));
        }

        let total_weight: f64 = weights.iter().map(|(_, w)| w).sum();
        let mut n = (hash >> 8) as f64 % total_weight;
        let mut selected_kind = EventKind::Plague; // Default
        for (kind, w) in weights {
            if n < w {
                selected_kind = kind;
                break;
            }
            n -= w;
        }
        // ----------------------------------

        // イベント効果の強度（hash ベース、0.5 〜 1.5）
        let intensity = 0.5 + (((hash >> 16) % 100) as f64 / 100.0);

        // 所属国家の技術ステートを取得
        let tech: Option<Mut<TechnologyState>> = belongs_to.and_then(|b: &BelongsToNation| nations.get_mut(b.0).ok());
        let effect = apply_event(selected_kind, intensity, &mut pop, &mut res, &mut deplet, tech);

        // 表示イベントを送出
        sim_events.send(SimulationEvent::RandomEvent {
            tick: tick.value,
            planet: name.0.clone(),
            kind: selected_kind,
            intensity,
            effect: effect.clone(),
        });

        event_log.events.push(EventRecord {
            tick: tick.value,
            planet_name: name.0.clone(),
            kind: selected_kind,
            effect,
        });
    }
}

/// イベント効果を適用し、構造化データを返す
fn apply_event(
    kind: EventKind,
    intensity: f64,
    pop: &mut Population,
    res: &mut Resources,
    deplet: &mut DepletableResources,
    tech: Option<Mut<TechnologyState>>,
) -> EventEffect {
    let mut effect = EventEffect::default();

    match kind {
        EventKind::Plague => {
            let casualties = pop.count * 0.01 * intensity;
            pop.count = (pop.count - casualties).max(0.0);
            effect.population_change = -casualties;
        }
        EventKind::BountifulHarvest => {
            let bonus = 200.0 * intensity;
            res.food += bonus;
            effect.food_change = bonus;
        }
        EventKind::MineralDiscovery => {
            let bonus = 150.0 * intensity;
            let reserve_bonus = 1000.0 * intensity;
            res.minerals += bonus;
            deplet.mineral_reserves += reserve_bonus;
            effect.minerals_change = bonus;
        }
        EventKind::BabyBoom => {
            let pop_bonus = pop.count * 0.005 * intensity;
            pop.count += pop_bonus;
            effect.population_change = pop_bonus;
        }
        EventKind::EnergyCrisis => {
            let loss = res.energy * 0.3 * intensity;
            res.energy = (res.energy - loss).max(0.0);
            effect.energy_change = -loss;
        }
        EventKind::TechBreakthrough => {
            if let Some(mut t) = tech {
                let bonus = 50.0 * intensity;
                t.research_points += bonus;
                effect.research_change = bonus;
            }
        }
        EventKind::Rebellion => {
            let pop_loss = pop.count * 0.005 * intensity;
            let food_loss = res.food * 0.1 * intensity;
            pop.count = (pop.count - pop_loss).max(0.0);
            res.food = (res.food - food_loss).max(0.0);
            effect.population_change = -pop_loss;
            effect.food_change = -food_loss;
        }
        EventKind::TradeBoom => {
            let bonus = 50.0 * intensity;
            res.food += bonus;
            res.minerals += bonus;
            res.energy += bonus;
            res.manufactured_goods += bonus;
            effect.food_change = bonus;
            effect.minerals_change = bonus;
            effect.energy_change = bonus;
            effect.goods_change = bonus;
        }
        EventKind::Earthquake => {
            let pop_loss = pop.count * 0.002 * intensity;
            let goods_loss = res.manufactured_goods * 0.2 * intensity;
            pop.count = (pop.count - pop_loss).max(0.0);
            res.manufactured_goods = (res.manufactured_goods - goods_loss).max(0.0);
            effect.population_change = -pop_loss;
            effect.goods_change = -goods_loss;
        }
        EventKind::RadiationStorm => {
            let pop_loss = pop.count * 0.001 * intensity;
            let energy_loss = res.energy * 0.2 * intensity;
            pop.count = (pop.count - pop_loss).max(0.0);
            res.energy = (res.energy - energy_loss).max(0.0);
            effect.population_change = -pop_loss;
            effect.energy_change = -energy_loss;
        }
        EventKind::MeteoriteImpact => {
            let pop_loss = pop.count * 0.05 * intensity;
            let food_loss = res.food * 0.5 * intensity;
            let minerals_bonus = 300.0 * intensity; // 隕石からの資源
            pop.count = (pop.count - pop_loss).max(0.0);
            res.food = (res.food - food_loss).max(0.0);
            res.minerals += minerals_bonus;
            effect.population_change = -pop_loss;
            effect.food_change = -food_loss;
            effect.minerals_change = minerals_bonus;
        }
        EventKind::EnvironmentalDisaster => {
            let casualties = pop.count * 0.05 * intensity;
            pop.count = (pop.count - casualties).max(0.0);
            effect.population_change = -casualties;
        }
    }

    effect
}

/// 決定論的ハッシュ関数（seed × tick × index → u64）
fn deterministic_hash(seed: u64, tick: u64, index: u64) -> u64 {
    let mut h = seed.wrapping_mul(2654435761);
    h = h.wrapping_add(tick.wrapping_mul(1099511628211));
    h = h.wrapping_add(index.wrapping_mul(6364136223846793005));
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;
    h
}
