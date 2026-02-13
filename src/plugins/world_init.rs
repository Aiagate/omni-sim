use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use bevy::prelude::*;

use crate::components::common::{SimName, Nation, Planet, StarSystem, BelongsToStarSystem, BelongsToPlanet, BelongsToNation};
use crate::components::economy::{Production, Resources, TradeRoute, TradeRouteMarker, DepletableResources};
use crate::components::population::Population;
use crate::components::military::MilitaryStrength;
use crate::components::diplomacy::DiplomaticRelation;
use crate::components::environment::{PlanetaryEnvironment, RenewableResources, EnvironmentalHealth};
use crate::components::technology::TechnologyState;
use crate::components::terraforming::TerraformingProject;
use crate::components::simulation_event::SimulationEvent;
use crate::config::SimulationConfig;
use crate::world_snapshot::*;

/// 初期世界を生成するプラグイン
pub struct WorldInitPlugin;

impl Plugin for WorldInitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_world_from_scenario);
    }
}

/// シナリオファイルから初期世界を生成する
fn spawn_world_from_scenario(
    mut commands: Commands,
    config: Res<SimulationConfig>,
    mut events: EventWriter<SimulationEvent>,
) {
    let scenario_path = "scenarios/default.yaml";
    
    let mut file = match File::open(scenario_path) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open scenario file {}: {}", scenario_path, e);
            return;
        }
    };

    let mut content = String::new();
    if let Err(e) = file.read_to_string(&mut content) {
        error!("Failed to read scenario file: {}", e);
        return;
    }

    let snapshot: WorldSnapshot = match serde_yaml::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to parse scenario YAML: {}", e);
            return;
        }
    };

    let mut planet_map = HashMap::new();
    let mut nation_map = HashMap::new();

    // 1. 星系と惑星・国家の生成
    for ss_snap in snapshot.star_systems {
        let star_system_id = commands.spawn((StarSystem, SimName(ss_snap.name))).id();

        // 惑星の生成
        for p_snap in ss_snap.planets {
            let atmosphere = match p_snap.environment.atmosphere {
                AtmosphereTypeSnapshot::EarthLike => crate::components::environment::AtmosphereType::EarthLike,
                AtmosphereTypeSnapshot::CarbonDioxide => crate::components::environment::AtmosphereType::CarbonDioxide,
                AtmosphereTypeSnapshot::Thin => crate::components::environment::AtmosphereType::Thin,
                AtmosphereTypeSnapshot::Toxic => crate::components::environment::AtmosphereType::Toxic,
                AtmosphereTypeSnapshot::None => crate::components::environment::AtmosphereType::None,
            };

            let mut env = PlanetaryEnvironment {
                mass: p_snap.environment.mass,
                gravity: p_snap.environment.gravity,
                radius: p_snap.environment.radius,
                atmosphere,
                atmospheric_pressure: p_snap.environment.atmospheric_pressure,
                temperature: p_snap.environment.temperature,
                water_coverage: p_snap.environment.water_coverage,
                habitability: 0.0,
                population_capacity: 0.0,
                radiation: p_snap.environment.radiation,
                tectonic_activity: p_snap.environment.tectonic_activity,
                meteorite_risk: p_snap.environment.meteorite_risk,
                mineral_deposits: p_snap.environment.mineral_deposits,
                mineral_accessibility: p_snap.environment.mineral_accessibility,
                renewable_energy_potential: p_snap.environment.renewable_energy_potential,
            };
            env.update_habitability();

            let depletable = DepletableResources {
                mineral_reserves: p_snap.depletable_resources.mineral_reserves,
                initial_mineral_reserves: p_snap.depletable_resources.mineral_reserves,
                fossil_fuel_reserves: p_snap.depletable_resources.fossil_fuel_reserves,
                initial_fossil_fuel_reserves: p_snap.depletable_resources.fossil_fuel_reserves,
                rare_earth_reserves: p_snap.depletable_resources.rare_earth_reserves,
                initial_rare_earth_reserves: p_snap.depletable_resources.rare_earth_reserves,
                mining_depth: 0.0,
            };

            let renewable = RenewableResources {
                soil_fertility: p_snap.renewable_resources.soil_fertility,
                forest_coverage: p_snap.renewable_resources.forest_coverage,
                renewable_energy_output: p_snap.renewable_resources.renewable_energy_output,
            };

            let health = EnvironmentalHealth {
                air_pollution: p_snap.environmental_health.air_pollution,
                water_pollution: p_snap.environmental_health.water_pollution,
                health_penalty: 0.0,
            };

            let planet_id = commands.spawn((
                Planet,
                SimName(p_snap.name.clone()),
                BelongsToStarSystem(star_system_id),
                Population::new(p_snap.population.count, p_snap.population.growth_rate),
                Resources {
                    food: p_snap.resources.food,
                    minerals: p_snap.resources.minerals,
                    energy: p_snap.resources.energy,
                    manufactured_goods: p_snap.resources.manufactured_goods,
                },
                Production {
                    food_rate: p_snap.production.food_rate,
                    mineral_rate: p_snap.production.mineral_rate,
                    energy_rate: p_snap.production.energy_rate,
                    manufacturing_rate: p_snap.production.manufacturing_rate,
                },
                MilitaryStrength::new(p_snap.military_strength),
                env,
                depletable,
                renewable,
                health,
                TerraformingProject::default(),
            )).id();
            planet_map.insert(p_snap.name, planet_id);
        }

        // 国家の生成
        for n_snap in ss_snap.nations {
            let planet_id = match planet_map.get(&n_snap.planet_name) {
                Some(id) => *id,
                None => {
                    warn!("Nation {} belongs to unknown planet {}", n_snap.name, n_snap.planet_name);
                    continue;
                }
            };

            let traits = n_snap.identity.traits.into_iter().map(|t| match t {
                NationalTraitSnapshot::CollectiveMind => crate::components::national_ai::NationalTrait::CollectiveMind,
                NationalTraitSnapshot::WarriorCulture => crate::components::national_ai::NationalTrait::WarriorCulture,
                NationalTraitSnapshot::Pacifist => crate::components::national_ai::NationalTrait::Pacifist,
                NationalTraitSnapshot::MerchantGuild => crate::components::national_ai::NationalTrait::MerchantGuild,
                NationalTraitSnapshot::Technocracy => crate::components::national_ai::NationalTrait::Technocracy,
            }).collect();

            let nation_id = commands.spawn((
                Nation,
                SimName(n_snap.name.clone()),
                BelongsToStarSystem(star_system_id),
                BelongsToPlanet(planet_id),
                crate::components::national_ai::NationalCharacter {
                    aggression: n_snap.character.aggression,
                    trade_affinity: n_snap.character.trade_affinity,
                    research_focus: n_snap.character.research_focus,
                    expansionism: n_snap.character.expansionism,
                    diplomacy_flexibility: n_snap.character.diplomacy_flexibility,
                },
                crate::components::national_ai::NationalIdentity { traits },
                crate::components::national_ai::NationalMemory::new(),
                crate::components::national_ai::InternalFactions::new(
                    n_snap.factions.militarist,
                    n_snap.factions.merchant,
                    n_snap.factions.technocrat,
                ),
                TechnologyState::new(),
            )).id();
            nation_map.insert(n_snap.name, nation_id);

            // 惑星に所属国家を記録
            commands.entity(planet_id).insert(BelongsToNation(nation_id));
        }
    }

    // 2. 外交関係の設定
    for dr_snap in snapshot.diplomatic_relations {
        let from_id = match nation_map.get(&dr_snap.from_nation) {
            Some(id) => *id,
            None => continue,
        };
        let to_id = match nation_map.get(&dr_snap.to_nation) {
            Some(id) => *id,
            None => continue,
        };

        commands.spawn((
            SimName(format!("{}→{}", dr_snap.from_nation, dr_snap.to_nation)),
            DiplomaticRelation::new(from_id, to_id, dr_snap.score),
        ));
    }

    // 3. 貿易ルートの設定
    for tr_snap in snapshot.trade_routes {
        let from_id = match planet_map.get(&tr_snap.from_planet) {
            Some(id) => *id,
            None => continue,
        };
        let to_id = match planet_map.get(&tr_snap.to_planet) {
            Some(id) => *id,
            None => continue,
        };

        commands.spawn((TradeRouteMarker, TradeRoute::new(from_id, to_id, tr_snap.distance, tr_snap.capacity)));
    }

    // 初期化完了イベントを送出
    events.send(SimulationEvent::WorldInitialized {
        max_ticks: config.max_ticks,
        seed: config.seed,
    });
}
