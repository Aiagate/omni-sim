use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// テラフォーミングの進行フェーズ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerraformingPhase {
    /// フェーズ 0: 未開始
    None,
    /// フェーズ 1: 大気密度の調整
    AtmosphericDensity,
    /// フェーズ 2: 温室効果ガス制御
    TemperatureControl,
    /// フェーズ 3: 水の液化・配布
    WaterManagement,
    /// フェーズ 4: 生態系の植え付け
    BiologicalSeeding,
    /// 完成
    Complete,
}

impl TerraformingPhase {
    pub fn next(&self) -> Self {
        match self {
            TerraformingPhase::None => TerraformingPhase::AtmosphericDensity,
            TerraformingPhase::AtmosphericDensity => TerraformingPhase::TemperatureControl,
            TerraformingPhase::TemperatureControl => TerraformingPhase::WaterManagement,
            TerraformingPhase::WaterManagement => TerraformingPhase::BiologicalSeeding,
            TerraformingPhase::BiologicalSeeding => TerraformingPhase::Complete,
            TerraformingPhase::Complete => TerraformingPhase::Complete,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TerraformingPhase::None => "未開始",
            TerraformingPhase::AtmosphericDensity => "大気密度の調整",
            TerraformingPhase::TemperatureControl => "温室効果ガス制御",
            TerraformingPhase::WaterManagement => "水の液化・配布",
            TerraformingPhase::BiologicalSeeding => "生態系の植え付け",
            TerraformingPhase::Complete => "完了",
        }
    }
}

/// テラフォーミングプロジェクト
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct TerraformingProject {
    pub current_phase: TerraformingPhase,
    pub progress: f64, // 0.0 to 1.0
    pub target_progress: f64, // 1.0
}

impl Default for TerraformingProject {
    fn default() -> Self {
        Self {
            current_phase: TerraformingPhase::None,
            progress: 0.0,
            target_progress: 1.0,
        }
    }
}
