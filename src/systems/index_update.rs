use bevy::prelude::*;
use crate::components::common::BelongsToPlanet;
use crate::components::nation_index::NationPlanetIndex;

/// 国家が所有する惑星の情報が変更されたときにインデックスを更新するシステム
pub fn update_nation_planet_index_system(
    mut index: ResMut<NationPlanetIndex>,
    query: Query<(Entity, &BelongsToPlanet), Changed<BelongsToPlanet>>,
    mut removed: RemovedComponents<BelongsToPlanet>,
) {
    // 削除されたコンポーネントに対応するインデックスを削除
    for nation_entity in removed.read() {
        if let Some(planet_entity) = index.nation_to_planet.remove(&nation_entity) {
            index.planet_to_nation.remove(&planet_entity);
        }
    }

    // 変更または新規追加されたマッピングを更新
    for (nation_entity, belongs_to) in &query {
        let planet_entity = belongs_to.0;
        
        // 古いマッピングがあれば削除（別の惑星に乗り換えた場合）
        if let Some(old_planet) = index.nation_to_planet.insert(nation_entity, planet_entity) {
            if old_planet != planet_entity {
                index.planet_to_nation.remove(&old_planet);
            }
        }
        index.planet_to_nation.insert(planet_entity, nation_entity);
    }
}

/// インデックスをゼロから構築するシステム（起動時など）
#[allow(dead_code)]
pub fn rebuild_nation_planet_index_system(
    mut index: ResMut<NationPlanetIndex>,
    query: Query<(Entity, &BelongsToPlanet)>,
) {
    index.planet_to_nation.clear();
    index.nation_to_planet.clear();
    for (nation_entity, belongs_to) in &query {
        index.planet_to_nation.insert(belongs_to.0, nation_entity);
        index.nation_to_planet.insert(nation_entity, belongs_to.0);
    }
}
