use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 大気組成の分類
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AtmosphereType {
    /// 地球型（O₂/N₂ — 呼吸可能）
    EarthLike,
    /// CO₂ 主体（金星型 — テラフォーミング必要）
    CarbonDioxide,
    /// 希薄大気（火星型）
    Thin,
    /// 有毒（H₂S, NH₃ 等）
    Toxic,
    /// なし（月型）
    None,
}

/// 惑星の物理的・環境的特性
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryEnvironment {
    // === 基本物理 ===
    /// 惑星質量（地球 = 1.0）
    pub mass: f64,
    /// 表面重力（地球 = 1.0G）
    pub gravity: f64,
    /// 惑星半径（地球 = 1.0）
    pub radius: f64,

    // === 大気・気候 ===
    /// 大気組成タイプ
    pub atmosphere: AtmosphereType,
    /// 大気圧（地球 = 1.0 atm）
    pub atmospheric_pressure: f64,
    /// 平均気温（℃）
    pub temperature: f64,
    /// 水の存在率（0.0 ~ 1.0）
    pub water_coverage: f64,

    // === 居住性 ===
    /// 総合居住性スコア（0.0 ~ 1.0）
    pub habitability: f64,
    /// 最大人口キャパシティ
    pub population_capacity: f64,

    // === 環境リスク ===
    pub radiation: f64,
    pub tectonic_activity: f64,
    pub meteorite_risk: f64,

    // === 資源ポテンシャル（初期値/最大値） ===
    pub mineral_deposits: f64,
    pub mineral_accessibility: f64,
    pub renewable_energy_potential: f64,
}

/// 再生可能資源の状態
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct RenewableResources {
    /// 土壌肥沃度（0.0 〜 1.5、1.0が標準）
    pub soil_fertility: f64,
    /// 森林被覆率（0.0 〜 1.0）
    pub forest_coverage: f64,
    /// 再エネ出力係数
    pub renewable_energy_output: f64,
}

impl Default for RenewableResources {
    fn default() -> Self {
        Self {
            soil_fertility: 1.0,
            forest_coverage: 0.5,
            renewable_energy_output: 1.0,
        }
    }
}

/// 環境健全性
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalHealth {
    /// 大気汚染レベル（0.0 〜 1.0）
    pub air_pollution: f64,
    /// 水質汚染レベル（0.0 〜 1.0）
    pub water_pollution: f64,
    /// 汚染による健康ペナルティ（成長率への直接影響）
    pub health_penalty: f64,
}

impl Default for EnvironmentalHealth {
    fn default() -> Self {
        Self {
            air_pollution: 0.0,
            water_pollution: 0.0,
            health_penalty: 0.0,
        }
    }
}

impl PlanetaryEnvironment {
    /// 居住性と人口キャパシティを再計算する
    pub fn update_habitability(&mut self) {
        let atmosphere_score = match self.atmosphere {
            AtmosphereType::EarthLike => 1.0,
            AtmosphereType::Thin => 0.4,
            AtmosphereType::CarbonDioxide => 0.2,
            AtmosphereType::Toxic => 0.05,
            AtmosphereType::None => 0.01,
        };

        let gravity_score = if self.gravity >= 0.8 && self.gravity <= 1.3 {
            1.0
        } else if self.gravity >= 0.5 && self.gravity < 0.8 {
            0.8
        } else if self.gravity > 1.3 && self.gravity <= 2.0 {
            0.7
        } else {
            0.3
        };

        let temp_score = if self.temperature >= -10.0 && self.temperature <= 40.0 {
            1.0
        } else if (self.temperature >= -40.0 && self.temperature < -10.0)
            || (self.temperature > 40.0 && self.temperature <= 60.0)
        {
            0.6
        } else {
            0.2
        };

        let water_score = if self.water_coverage >= 0.3 {
            1.0
        } else if self.water_coverage >= 0.1 {
            0.7
        } else {
            0.3
        };

        let radiation_score = if self.radiation < 0.2 {
            1.0
        } else if self.radiation <= 0.5 {
            0.7
        } else {
            0.3
        };

        self.habitability =
            atmosphere_score * gravity_score * temp_score * water_score * radiation_score;
        
        // population_capacity = habitability * radius^2 * 50,000,000
        self.population_capacity = self.habitability * self.radius * self.radius * 50_000_000.0;
    }
}
