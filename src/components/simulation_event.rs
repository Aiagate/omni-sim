use bevy::prelude::*;

use crate::components::economy::ResourceType;
use crate::components::events::{EventKind, EventEffect};
use crate::components::technology::TechField;

/// イベントの重要度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Reflect)]
pub enum EventImportance {
    #[default]
    Low,
    Medium,
    High,
}

/// シミュレーション中に発生するイベントを表す Bevy Event。
/// ドメインシステムが送出し、表示レイヤーが受信して描画する。
///
/// ドメインロジックと表示を分離するための Observer パターン。
#[derive(Event, Debug, Clone)]
pub enum SimulationEvent {
    // === 経済 ===
    /// 惑星の資源状況レポート（毎 Tick 送出）
    ResourceReport {
        tick: u64,
        planet: String,
        food: f64,
        minerals: f64,
        energy: f64,
        goods: f64,
        labor_bonus: f64,
        tech_level: Option<u32>,
    },

    // === 人口 ===
    /// 人口変動レポート（毎 Tick 送出）
    PopulationReport {
        tick: u64,
        planet: String,
        count: f64,
        change: f64,
        growth_rate: f64,
        starvation_ticks: u32,
        habitability: f64,
        capacity: f64,
    },

    // SimulationEvent::TradeTransfer はバッチ化に伴い削除されました

    /// 貿易の集計レポート（1ルート1回送出、バッチ版）
    TradeSummary {
        tick: u64,
        from: String,
        to: String,
        total_amount: f64,
        resources: Vec<(ResourceType, f64)>,
        #[allow(dead_code)]
        cost_pct: f64,
    },

    // === 軍事 ===
    /// 軍事力レポート（毎 Tick 送出）
    MilitaryReport {
        tick: u64,
        planet: String,
        ships: u32,
        power: f64,
    },

    // === 外交 ===
    /// 外交スコア変動レポート（毎 Tick 送出）
    DiplomacyReport {
        tick: u64,
        #[allow(dead_code)]
        relation: String,
        score: f64,
        trend: f64,
    },

    // === 戦争 ===
    /// 戦争勃発
    WarDeclared {
        tick: u64,
        aggressor: String,
        defender: String,
        score: f64,
    },
    /// 停戦
    Ceasefire {
        tick: u64,
        nation: String,
        duration: u64,
    },
    /// 降伏
    Surrender {
        tick: u64,
        nation: String,
    },
    /// 戦闘レポート
    CombatReport {
        tick: u64,
        attacker: String,
        target: String,
        ships_destroyed: u32,
        casualties: f64,
        remaining_ships: u32,
    },

    // === 研究 ===
    /// 技術レベルアップ
    TechLevelUp {
        tick: u64,
        planet: String,
        total_level: u32,
        next_field: TechField,
        rate: f64,
    },
    /// 研究進捗レポート（定期送出）
    ResearchReport {
        tick: u64,
        planet: String,
        levels: [u32; 8],
        rate: f64,
        progress: f64,
        cost: f64,
    },

    // === ランダムイベント ===
    /// ランダムイベント発生
    RandomEvent {
        tick: u64,
        planet: String,
        kind: EventKind,
        intensity: f64,
        effect: EventEffect,
    },

    // === テラフォーミング ===
    /// テラフォーミング進捗
    TerraformingProgress {
        tick: u64,
        planet: String,
        phase: String,
        progress: f64,
    },
    /// テラフォーミングフェーズ完了
    TerraformingPhaseComplete {
        tick: u64,
        planet: String,
        phase: String,
    },

    // === 初期化 ===
    /// ワールド初期化完了
    WorldInitialized {
        max_ticks: u64,
        seed: u64,
    },
}

impl SimulationEvent {
    /// イベントの重要度を取得する
    pub fn importance(&self) -> EventImportance {
        match self {
            SimulationEvent::ResourceReport { .. } => EventImportance::Low,
            SimulationEvent::PopulationReport { .. } => EventImportance::Low,
            SimulationEvent::MilitaryReport { .. } => EventImportance::Low,
            SimulationEvent::DiplomacyReport { .. } => EventImportance::Low,
            // SimulationEvent::TradeTransfer { .. } => EventImportance::Low,
            SimulationEvent::ResearchReport { .. } => EventImportance::Low,

            SimulationEvent::TradeSummary { .. } => EventImportance::Medium,
            SimulationEvent::TechLevelUp { .. } => EventImportance::Medium,
            SimulationEvent::CombatReport { .. } => EventImportance::Medium,
            SimulationEvent::Ceasefire { .. } => EventImportance::Medium,
            SimulationEvent::RandomEvent { .. } => EventImportance::Medium,
            SimulationEvent::TerraformingPhaseComplete { .. } => EventImportance::Medium,

            SimulationEvent::WarDeclared { .. } => EventImportance::High,
            SimulationEvent::Surrender { .. } => EventImportance::High,
            SimulationEvent::WorldInitialized { .. } => EventImportance::High,
            SimulationEvent::TerraformingProgress { .. } => EventImportance::Low,
        }
    }
}
