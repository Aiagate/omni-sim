use bevy::prelude::*;
use crate::components::common::{StarSystem, SimName, Nation, BelongsToStarSystem, Planet, BelongsToPlanet, BelongsToNation};
use crate::components::economy::{Production, Resources, DepletableResources};
use crate::components::population::Population;
use crate::components::military::MilitaryStrength;
use crate::components::environment::{PlanetaryEnvironment, RenewableResources, EnvironmentalHealth};
use crate::components::terraforming::TerraformingProject;
use crate::components::technology::{TechnologyState, TechId};
use crate::components::space::{Position, GalacticCatalog};
use crate::components::simulation_event::SimulationEvent;
use crate::tick::CurrentTick;
use rand::prelude::*;
use rand::thread_rng;

/// 探索システム
///
/// 航行技術（Navigation）のレベルに応じて、近隣の未発見星系を探索・発見する。
pub fn exploration_system(
    mut commands: Commands,
    tick: Res<CurrentTick>,
    mut catalog: ResMut<GalacticCatalog>,
    nations: Query<(&SimName, &TechnologyState, &BelongsToStarSystem), With<Nation>>,
    systems: Query<&Position, With<StarSystem>>,
    mut events: EventWriter<SimulationEvent>,
) {
    // 毎Tick判定すると多すぎるので、50 Tick ごとに判定
    if tick.value % 50 != 0 {
        return;
    }

    let mut rng = thread_rng();

    for (nation_name, tech, belongs_ss) in &nations {
        let nav_level = tech.level(TechId::Navigation);
        // 探索範囲: 5.0 + Navigationレベル * 3.0 光年
        let range = 5.0 + nav_level as f64 * 3.0;

        if let Ok(pos) = systems.get(belongs_ss.0) {
            // カタログから範囲内の星系を探す
            let mut discovered_idx = None;
            for (idx, potential) in catalog.undiscovered_systems.iter().enumerate() {
                if pos.distance_to(&potential.position) <= range {
                    // 発見判定（10% の確率）
                    if rng.gen_bool(0.1) {
                        discovered_idx = Some(idx);
                        break;
                    }
                }
            }

            if let Some(idx) = discovered_idx {
                let system = catalog.undiscovered_systems.remove(idx);
                
                // 新しい星系をスポーン
                let star_system_id = commands.spawn((
                    StarSystem,
                    SimName(system.name.clone()),
                    system.position,
                )).id();

                // 惑星のスポーン
                for planet_data in system.planets {
                    commands.spawn((
                        Planet,
                        SimName(planet_data.name),
                        BelongsToStarSystem(star_system_id),
                        Population::new(0.0, 0.0), 
                        planet_data.resources,
                        planet_data.production,
                        MilitaryStrength::new(0),
                        planet_data.environment,
                        planet_data.depletable_resources,
                        planet_data.renewable_resources,
                        EnvironmentalHealth::default(),
                        TerraformingProject::default(),
                    ));
                }

                // 発見イベントを送出
                events.send(SimulationEvent::SystemDiscovered {
                    tick: tick.value,
                    nation: nation_name.0.clone(),
                    system: system.name,
                });
            }
        }
    }
}
