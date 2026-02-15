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
use crate::components::space::{Position, GalacticCatalog, PotentialSystem, Orbit};
use crate::config::SimulationConfig;
use crate::world_snapshot::*;
use crate::world_snapshot::AtmosphereTypeSnapshot;
use rand::prelude::*;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use std::f64::consts::PI;

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
        let pos = Position::new(ss_snap.position.x, ss_snap.position.y, ss_snap.position.z);
        let star_system_id = commands.spawn((StarSystem, SimName(ss_snap.name), pos)).id();

        // 惑星の生成
        for (idx, p_snap) in ss_snap.planets.into_iter().enumerate() {
            // シナリオからの生成時は、インデックスに基づいて適当な軌道を割り当てる
            // 0.5 AU + idx * 1.5 AU
            let orbit = Orbit {
                distance: 0.5 + idx as f32 * 1.5,
                angle: rand::thread_rng().gen_range(0.0..2.0 * std::f32::consts::PI),
                speed: 0.02 / (1.0 + idx as f32).sqrt(), // 遠いほど遅く
            };

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
                orbit,
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

    // 4. 銀河カタログの生成
    let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
    let mut catalog = GalacticCatalog::default();
    
    for i in 0..50 { // とりあえず50星系
        let distance_from_center = rng.gen_range(10.0..100.0);
        let angle = rng.gen_range(0.0..2.0 * PI);
        let x = distance_from_center * angle.cos();
        let y = distance_from_center * angle.sin();
        let z = rng.gen_range(-5.0..5.0);
        
        let position = Position::new(x, y, z);
        let planets = generate_random_planets(&mut rng, i, position);

        catalog.undiscovered_systems.push(PotentialSystem {
            name: format!("Sector-{:03}", i),
            position,
            planets,
        });
    }
    commands.insert_resource(catalog);

    // 初期化完了イベントを送出
    events.send(SimulationEvent::WorldInitialized {
        max_ticks: config.max_ticks,
        seed: config.seed,
    });
}

/// ランダムな惑星を生成するヘルパー関数
fn generate_random_planets(rng: &mut ChaCha20Rng, system_index: usize, _system_pos: Position) -> Vec<crate::components::space::PotentialPlanet> {
    let num_planets = rng.gen_range(1..=5);
    let mut planets = Vec::new();

    for j in 0..num_planets {
        let name = format!("Sector-{:03}-{:?}", system_index, ["I", "II", "III", "IV", "V"].get(j).unwrap_or(&"X"));
        
        // 恒星からの距離（0.5 ~ 10.0 AU 相当）- これ環境パラメータへの影響用
        let orbit_dist = rng.gen_range(0.5f64..10.0f64);

        // 環境の生成
        let mass = rng.gen_range(0.1f64..3.0f64);
        let gravity = mass.powf(0.8); // 簡易計算
        let radius = mass.powf(0.33);

        // 距離による温度決定 (簡易モデル)
        // 1.0 AU (Earth) = 15.0 C と仮定, 距離の2乗に反比例してエネルギーが減る
        let base_temp = 288.0 / orbit_dist.sqrt() - 273.15; 
        let temperature = base_temp + rng.gen_range(-20.0f64..20.0f64);

        // 大気と水
        let (atmosphere, water_coverage) = if temperature > 100.0 || temperature < -100.0 {
            (crate::components::environment::AtmosphereType::None, 0.0)
        } else if gravity < 0.5 {
             (crate::components::environment::AtmosphereType::Thin, rng.gen_range(0.0..0.1))
        } else {
             let roll = rng.gen_range(0.0..1.0);
             if roll < 0.2 { (crate::components::environment::AtmosphereType::Toxic, rng.gen_range(0.0..0.5)) }
             else if roll < 0.5 { (crate::components::environment::AtmosphereType::CarbonDioxide, rng.gen_range(0.0..0.3)) }
             else if roll < 0.8 { (crate::components::environment::AtmosphereType::Thin, rng.gen_range(0.0..0.2)) }
             else { (crate::components::environment::AtmosphereType::EarthLike, rng.gen_range(0.3..0.9)) }
        };

        let mut env = PlanetaryEnvironment {
            mass,
            gravity,
            radius,
            atmosphere,
            atmospheric_pressure: if atmosphere == crate::components::environment::AtmosphereType::None { 0.0 } else { rng.gen_range(0.1..5.0) },
            temperature,
            water_coverage,
            habitability: 0.0, // update_habitability で計算
            population_capacity: 0.0,
            radiation: if orbit_dist < 1.0 { rng.gen_range(0.1..0.5) } else { rng.gen_range(0.0..0.2) },
            tectonic_activity: rng.gen_range(0.0..0.5),
            meteorite_risk: rng.gen_range(0.0..0.1),
            mineral_deposits: rng.gen_range(100.0..10000.0),
            mineral_accessibility: rng.gen_range(0.1..1.0),
            renewable_energy_potential: if orbit_dist < 2.0 { rng.gen_range(0.5..1.5) } else { rng.gen_range(0.1..0.5) },
        };
        env.update_habitability();

        // 資源の分布 (環境に依存させるのが理想だが、一旦ランダム)
        let resources = Resources {
            food: rng.gen_range(0.0..1000.0), // 自然食品？
            minerals: rng.gen_range(100.0..5000.0),
            energy: rng.gen_range(0.0..1000.0),
            manufactured_goods: 0.0,
        };

        let production = Production {
            food_rate: if env.habitability > 0.5 { rng.gen_range(1.0..10.0) } else { 0.0 },
            mineral_rate: rng.gen_range(5.0..20.0) * env.mineral_accessibility,
            energy_rate: rng.gen_range(5.0..15.0),
            manufacturing_rate: 0.0, // 無人なので工場なし
        };

        let depletable_resources = DepletableResources {
            mineral_reserves: env.mineral_deposits * 1000.0,
            initial_mineral_reserves: env.mineral_deposits * 1000.0,
            fossil_fuel_reserves: if env.habitability > 0.3 { rng.gen_range(1000.0..50000.0) } else { 0.0 },
            initial_fossil_fuel_reserves: if env.habitability > 0.3 { rng.gen_range(1000.0..50000.0) } else { 0.0 },
            rare_earth_reserves: rng.gen_range(100.0..5000.0),
            initial_rare_earth_reserves: rng.gen_range(100.0..5000.0),
            mining_depth: 0.0,
        };
        
        // 森林などは環境次第
        let forest_coverage = if env.water_coverage > 0.1 && env.temperature > -10.0 && env.temperature < 40.0 {
            rng.gen_range(0.1..0.8)
        } else {
            0.0
        };

        let renewable_resources = RenewableResources {
            soil_fertility: if env.water_coverage > 0.0 { rng.gen_range(0.0..1.0) } else { 0.0 },
            forest_coverage,
            renewable_energy_output: env.renewable_energy_potential * 10.0,
        };

        let orbit = Orbit {
            distance: orbit_dist as f32,
            angle: rng.gen_range(0.0..2.0 * std::f32::consts::PI),
            speed: 0.02 / orbit_dist.sqrt() as f32,
        };

        planets.push(crate::components::space::PotentialPlanet {
            name,
            environment: env,
            resources,
            production,
            depletable_resources,
            renewable_resources,
            orbit,
        });
    }

    planets
}
