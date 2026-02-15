use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorldSnapshot {
    pub star_systems: Vec<StarSystemSnapshot>,
    pub diplomatic_relations: Vec<DiplomaticRelationSnapshot>,
    pub trade_routes: Vec<TradeRouteSnapshot>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StarSystemSnapshot {
    pub name: String,
    pub position: PositionSnapshot,
    pub planets: Vec<PlanetSnapshot>,
    pub nations: Vec<NationSnapshot>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PositionSnapshot {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanetSnapshot {
    pub name: String,
    pub population: PopulationSnapshot,
    pub resources: ResourcesSnapshot,
    pub production: ProductionSnapshot,
    pub military_strength: u32,
    pub environment: PlanetaryEnvironmentSnapshot,
    pub depletable_resources: DepletableResourcesSnapshot,
    pub renewable_resources: RenewableResourcesSnapshot,
    pub environmental_health: EnvironmentalHealthSnapshot,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanetaryEnvironmentSnapshot {
    pub mass: f64,
    pub gravity: f64,
    pub radius: f64,
    pub atmosphere: AtmosphereTypeSnapshot,
    pub atmospheric_pressure: f64,
    pub temperature: f64,
    pub water_coverage: f64,
    pub radiation: f64,
    pub tectonic_activity: f64,
    pub meteorite_risk: f64,
    pub mineral_deposits: f64,
    pub mineral_accessibility: f64,
    pub renewable_energy_potential: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DepletableResourcesSnapshot {
    pub mineral_reserves: f64,
    pub fossil_fuel_reserves: f64,
    pub rare_earth_reserves: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenewableResourcesSnapshot {
    pub soil_fertility: f64,
    pub forest_coverage: f64,
    pub renewable_energy_output: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentalHealthSnapshot {
    pub air_pollution: f64,
    pub water_pollution: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AtmosphereTypeSnapshot {
    EarthLike,
    CarbonDioxide,
    Thin,
    Toxic,
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PopulationSnapshot {
    pub count: f64,
    pub growth_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourcesSnapshot {
    pub food: f64,
    pub minerals: f64,
    pub energy: f64,
    pub manufactured_goods: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductionSnapshot {
    pub food_rate: f64,
    pub mineral_rate: f64,
    pub energy_rate: f64,
    pub manufacturing_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NationSnapshot {
    pub name: String,
    pub planet_name: String, // Which planet this nation is based on
    pub character: NationalCharacterSnapshot,
    pub identity: NationalIdentitySnapshot,
    pub factions: InternalFactionsSnapshot,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NationalCharacterSnapshot {
    pub aggression: f64,
    pub trade_affinity: f64,
    pub research_focus: f64,
    pub expansionism: f64,
    pub diplomacy_flexibility: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NationalTraitSnapshot {
    CollectiveMind,
    WarriorCulture,
    Pacifist,
    MerchantGuild,
    Technocracy,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NationalIdentitySnapshot {
    pub traits: Vec<NationalTraitSnapshot>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InternalFactionsSnapshot {
    pub militarist: f64,
    pub merchant: f64,
    pub technocrat: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiplomaticRelationSnapshot {
    pub from_nation: String,
    pub to_nation: String,
    pub score: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeRouteSnapshot {
    pub from_planet: String,
    pub to_planet: String,
    pub distance: f64,
    pub capacity: f64,
}
