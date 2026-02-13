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
