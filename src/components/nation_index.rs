use bevy::prelude::*;
use bevy::utils::HashMap;

/// 惑星エンティティから国家エンティティを逆引きするためのインデックス
#[derive(Resource, Debug, Default)]
pub struct NationPlanetIndex {
    /// Key: Planet Entity, Value: Nation Entity
    pub planet_to_nation: HashMap<Entity, Entity>,
    /// Key: Nation Entity, Value: Planet Entity
    pub nation_to_planet: HashMap<Entity, Entity>,
}
