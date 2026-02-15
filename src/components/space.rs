use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::economy::{Resources, Production, DepletableResources};
use crate::components::environment::{PlanetaryEnvironment, RenewableResources};

/// 星系の空間座標（光年単位）
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// 2点間の距離（光年）
    pub fn distance_to(&self, other: &Position) -> f64 {
        ((self.x - other.x).powi(2)
       + (self.y - other.y).powi(2)
       + (self.z - other.z).powi(2)).sqrt()
    }
}

/// 軌道パラメータ（描画用）
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Orbit {
    pub distance: f32, // AU
    pub angle: f32,    // Radians
    pub speed: f32,    // Radians per tick
}

impl Default for Orbit {
    fn default() -> Self {
        Self {
            distance: 1.0,
            angle: 0.0,
            speed: 0.01,
        }
    }
}

/// 銀河全体の未発見星系カタログ
#[derive(Resource, Debug, Default)]
pub struct GalacticCatalog {
    pub undiscovered_systems: Vec<PotentialSystem>,
}

#[derive(Debug, Clone)]
pub struct PotentialSystem {
    #[allow(dead_code)]
    pub name: String,
    pub position: Position,
    #[allow(dead_code)]
    pub planets: Vec<PotentialPlanet>,
}

#[derive(Debug, Clone)]
pub struct PotentialPlanet {
    pub name: String,
    pub environment: PlanetaryEnvironment,
    pub resources: Resources,
    pub production: Production,
    pub depletable_resources: DepletableResources,
    pub renewable_resources: RenewableResources,
    pub orbit: Orbit,
}

/// 銀河生成の設定
#[derive(Resource, Debug, Clone)]
pub struct GalaxyConfig {
    pub seed: u64,
    pub num_systems: usize,
    pub galaxy_radius: f64,
}

impl Default for GalaxyConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            num_systems: 50,
            galaxy_radius: 100.0,
        }
    }
}
