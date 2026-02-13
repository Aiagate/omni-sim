use bevy::prelude::*;

/// 人口を管理するコンポーネント
#[derive(Component, Debug, Clone)]
pub struct Population {
    /// 現在の人口数
    pub count: f64,
    /// 基本成長率（1 Tick あたりの割合、例: 0.002 = 0.2%）
    pub base_growth_rate: f64,
    /// 実効成長率（食料等の影響を加味した値）
    pub effective_growth_rate: f64,
    /// 前 Tick との人口差分
    pub last_change: f64,
    /// 飢餓 Tick カウント（連続して食料不足だった Tick 数）
    pub starvation_ticks: u32,
}

impl Population {
    pub fn new(count: f64, base_growth_rate: f64) -> Self {
        Self {
            count,
            base_growth_rate,
            effective_growth_rate: base_growth_rate,
            last_change: 0.0,
            starvation_ticks: 0,
        }
    }
}
