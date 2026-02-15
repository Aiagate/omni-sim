use bevy::prelude::*;
use serde::Serialize;
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;
use futures_util::SinkExt;

use crate::components::economy::CargoFleet;
use crate::components::population::Population;
use crate::components::common::{SimName, Nation, BelongsToNation, Planet, StarSystem, BelongsToStarSystem};
use crate::components::space::{Position, Orbit};
use crate::components::environment::PlanetaryEnvironment;
use crate::tick::CurrentTick;

pub struct WebIntegrationPlugin;

impl Plugin for WebIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WebConfig::default())
           .add_systems(Startup, setup_web_server)
           .add_systems(Update, broadcast_state);
    }
}

#[derive(Resource)]
struct WebConfig {
    sender: Option<broadcast::Sender<String>>,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self { sender: None }
    }
}

// DTOs for JSON Serialization
#[derive(Serialize)]
struct GameStateDto {
    tick: u32,
    systems: Vec<StarSystemDto>,
    fleets: Vec<FleetDto>,
    events: Vec<EventDto>,
}

#[derive(Serialize)]
struct StarSystemDto {
    id: String,
    name: String,
    position: [f32; 3],
    faction: String,
    population: f64,
    planets: Vec<PlanetDto>,
}

#[derive(Serialize)]
struct PlanetDto {
    id: String,
    name: String,
    orbit_distance: f32,
    angle: f32,
    size: f32,
    color: String,
    faction: String,
    population: f64,
    type_name: String,
    owner_id: Option<String>,
}

#[derive(Serialize)]
struct FleetDto {
    id: String,
    name: String,
    faction: String,
    position: [f32; 3],
    destination_id: String,
    origin_id: String,
    progress: f32,
    status:String,
}

#[derive(Serialize)]
struct EventDto {
    id: String,
    timestamp: u128,
    #[serde(rename = "type")]
    event_type: String,
    message: String,
    severity: String,
}

fn setup_web_server(mut web_config: ResMut<WebConfig>) {
    let (tx, _rx) = broadcast::channel(100);
    web_config.sender = Some(tx.clone());

    let tx_clone = tx.clone();
    
    // Spawn Tokio runtime in a separate thread
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
            info!("WebSocket Integration Server listening on ws://0.0.0.0:8080");
            while let Ok((stream, addr)) = listener.accept().await {
                info!("New connection attempt from: {}", addr);
                let mut rx = tx_clone.subscribe();
                tokio::spawn(async move {
                    match accept_async(stream).await {
                        Ok(mut websocket) => {
                            info!("WebSocket handshake successful with {}", addr);
                            while let Ok(msg) = rx.recv().await {
                                if websocket.send(tokio_tungstenite::tungstenite::Message::Text(msg.into())).await.is_err() {
                                    info!("WebSocket send error for client {}", addr);
                                    break;
                                }
                            }
                            info!("Client {} disconnected", addr);
                        }
                        Err(e) => {
                            error!("Error during the websocket handshake for {}: {}", addr, e);
                        }
                    }
                });
            }
        });
    });
}

fn broadcast_state(
    web_config: Res<WebConfig>,
    tick: Res<CurrentTick>,
    query_systems: Query<(Entity, &SimName, &Position), With<StarSystem>>,
    query_planets: Query<(
        Entity, 
        &SimName, 
        &Orbit, 
        &BelongsToStarSystem, 
        &Population, 
        Option<&BelongsToNation>,
        &PlanetaryEnvironment
    ), With<Planet>>,
    query_fleets: Query<(Entity, &Position, &CargoFleet)>,
    query_nations: Query<&SimName, With<Nation>>,
    // events: EventReader<SimulationEvent>, // TODO: Event integration
) {
    if let Some(sender) = &web_config.sender {
        // Only broadcast if there are subscribers to save CPU
        if sender.receiver_count() == 0 {
            return;
        }

        // Group planets by star system
        let mut system_planets: std::collections::HashMap<Entity, Vec<PlanetDto>> = std::collections::HashMap::new();
        for (entity, name, orbit, belongs_to, pop, belongs_nation, env) in &query_planets {
            let faction = if let Some(b) = belongs_nation {
                if let Ok(n_name) = query_nations.get(b.0) {
                    n_name.0.clone()
                } else { "Neutral".to_string() }
            } else { "Neutral".to_string() };

            let owner_id = belongs_nation.map(|b| format!("{:?}", b.0));

            let planet_dto = PlanetDto {
                id: format!("{:?}", entity),
                name: name.0.clone(),
                orbit_distance: orbit.distance,
                angle: orbit.angle,
                size: (env.radius as f32).max(0.2), // Minimum size for visibility
                color: match env.atmosphere {
                    crate::components::environment::AtmosphereType::EarthLike => "#2288CC".to_string(), // Blue
                    crate::components::environment::AtmosphereType::CarbonDioxide => "#DDAA55".to_string(), // Venus-like
                    crate::components::environment::AtmosphereType::Thin => "#CC5544".to_string(), // Mars-like
                    crate::components::environment::AtmosphereType::Toxic => "#88CC44".to_string(), // Toxic green
                    crate::components::environment::AtmosphereType::None => "#888888".to_string(), // Grey
                },
                faction,
                population: pop.count,
                type_name: format!("{:?}", env.atmosphere),
                owner_id,
            };

            system_planets.entry(belongs_to.0).or_default().push(planet_dto);
        }

        let mut systems = Vec::new();
        for (entity, name, pos) in &query_systems {
            let planets = system_planets.remove(&entity).unwrap_or_default();
            
            // Calculate total population of the system
            let total_pop: f64 = planets.iter().map(|p| p.population).sum();

            // Determine dominant faction (simplified: first planet's faction or Neutral)
            let faction = planets.first().map(|p| p.faction.clone()).unwrap_or("Neutral".to_string());

            systems.push(StarSystemDto {
                id: format!("{:?}", entity),
                name: name.0.clone(),
                position: [pos.x as f32, pos.y as f32, pos.z as f32],
                faction,
                population: total_pop,
                planets,
            });
        }

        let mut fleets = Vec::new();
        for (entity, pos, fleet) in &query_fleets {
             let progress = if fleet.total_distance > 0.0 {
                 fleet.traveled_distance / fleet.total_distance
             } else {
                 0.0
             };

             // Note: Faction for fleets is also determined by their origin planet's nation,
             // but for now we'll just mark them from the origin nation if possible.
             let faction = if let Ok((_, _, _, _, _, belongs, _)) = query_planets.get(fleet.origin) {
                if let Some(b) = belongs {
                    if let Ok(n_name) = query_nations.get(b.0) {
                        n_name.0.clone()
                    } else { "Neutral".to_string() }
                } else { "Neutral".to_string() }
             } else { "Neutral".to_string() };

             fleets.push(FleetDto {
                 id: format!("{:?}", entity),
                 name: format!("Cargo Fleet from {}", fleet.origin_name),
                 faction,
                 position: [pos.x as f32, pos.y as f32, pos.z as f32],
                 destination_id: format!("{:?}", fleet.destination),
                 origin_id: format!("{:?}", fleet.origin),
                 progress: progress as f32,
                 status: "moving".to_string(),
             });
        }

        let state = GameStateDto {
            tick: tick.value as u32,
            systems,
            fleets,
            events: vec![], // TODO: Convert events
        };

        if let Ok(json) = serde_json::to_string(&state) {
            let _ = sender.send(json);
        }
    }
}
