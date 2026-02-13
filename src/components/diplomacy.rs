use bevy::prelude::*;

/// 二国間の外交関係を表すコンポーネント
#[derive(Component, Debug, Clone)]
pub struct DiplomaticRelation {
    /// この関係のオーナー国家
    pub owner_nation: Entity,
    /// 相手国のエンティティ
    pub target_nation: Entity,
    /// 関係スコア（-100.0 〜 100.0）
    /// 負: 敵対的、正: 友好的
    pub score: f64,
    /// スコアの変動トレンド（正: 改善中、負: 悪化中）
    pub trend: f64,
}

impl DiplomaticRelation {
    pub fn new(owner: Entity, target: Entity, score: f64) -> Self {
        Self {
            owner_nation: owner,
            target_nation: target,
            score: score.clamp(-100.0, 100.0),
            trend: 0.0,
        }
    }
}

/// 戦争状態を表すコンポーネント（国家に付与）
#[derive(Component, Debug, Clone)]
pub struct AtWar {
    /// 敵国
    pub enemy_nation: Entity,
    /// 敵の惑星
    pub enemy_planet: Entity,
    /// 自分の惑星
    pub own_planet: Entity,
    /// 戦争が始まった Tick
    pub started_at: u64,
    /// 戦争による累積損害（艦船）
    pub total_ships_lost: u32,
    /// 戦争による累積人口損害
    pub total_casualties: f64,
}

impl AtWar {
    pub fn new(enemy_nation: Entity, enemy_planet: Entity, own_planet: Entity, started_at: u64) -> Self {
        Self {
            enemy_nation,
            enemy_planet,
            own_planet,
            started_at,
            total_ships_lost: 0,
            total_casualties: 0.0,
        }
    }
}
