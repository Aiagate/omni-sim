use bevy::prelude::*;

/// 軍事力を表すコンポーネント（惑星に付与）
#[derive(Component, Debug, Clone)]
pub struct MilitaryStrength {
    /// 艦船数
    pub ships: u32,
    /// 総合戦力値
    pub power: f64,
    /// 戦闘中フラグ
    pub in_combat: bool,
}

impl MilitaryStrength {
    pub fn new(ships: u32) -> Self {
        Self {
            ships,
            power: ships as f64 * 10.0,
            in_combat: false,
        }
    }

    /// 軍事力を再計算
    pub fn recalculate(&mut self) {
        self.power = self.ships as f64 * 10.0;
    }
}

/// 艦隊マーカー（宇宙空間に存在する軍事力）
#[derive(Component, Debug, Clone)]
pub struct Fleet;

/// 艦隊の戦術態勢
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FleetStance {
    #[default]
    Active,    // 通常戦闘（バランス）
    Aggressive, // 攻撃優先（攻撃力+20%, 被弾率高）
    Defensive,  // 防御優先（攻撃力-20%, 惑星の身代わりになる）
    Passive,    // 戦闘回避（攻撃しない、被弾率極低）
}
