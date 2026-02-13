use bevy::prelude::*;

/// エンティティの名前
#[derive(Component, Debug, Clone)]
pub struct SimName(pub String);

/// 星系マーカー
#[derive(Component, Debug)]
pub struct StarSystem;

/// 惑星マーカー
#[derive(Component, Debug)]
pub struct Planet;

/// 国家マーカー
#[derive(Component, Debug)]
pub struct Nation;

/// 所属する星系を示すコンポーネント
#[derive(Component, Debug)]
pub struct BelongsToStarSystem(#[allow(dead_code)] pub Entity);

/// 所属する惑星を示すコンポーネント
#[derive(Component, Debug)]
pub struct BelongsToPlanet(pub Entity);

/// 所属する国家を示すコンポーネント
#[derive(Component, Debug)]
pub struct BelongsToNation(pub Entity);
