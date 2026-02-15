use bevy::prelude::*;

/// 星系ごとの戦況
#[derive(Component, Debug, Clone)]
pub struct Battlefront {
    /// 戦闘が行われている星系
    pub system: Entity,
    /// 攻撃側の総戦力（概算）
    pub attacker_strength: f64,
    /// 防衛側の総戦力（概算）
    pub defender_strength: f64,
    /// 戦況ステータス
    pub status: BattleStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleStatus {
    /// 戦闘中（両軍が存在）
    Active,
    /// 戦闘終了（次の Tick で削除）
    Resolved,
}

/// 惑星の占領状態
/// 惑星エンティティに付与される
#[derive(Component, Debug, Clone)]
pub struct Occupied {
    /// もともとの所有国
    #[allow(dead_code)]
    pub original_owner: Entity,
    /// もともとの所有国名（表示用）
    #[allow(dead_code)]
    pub original_owner_name: String,
    /// 現在の実効支配国
    pub occupied_by: Entity,
    /// 占領された Tick
    #[allow(dead_code)]
    pub occupied_at: u64,
}
