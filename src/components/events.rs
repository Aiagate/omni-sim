use bevy::prelude::*;

/// イベントの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// 疫病: 人口減少
    Plague,
    /// 豊作: 食料ボーナス
    BountifulHarvest,
    /// 鉱脈発見: 鉱物ボーナス
    MineralDiscovery,
    /// ベビーブーム: 成長率一時ボーナス
    BabyBoom,
    /// エネルギー危機: エネルギー減少
    EnergyCrisis,
    /// 技術ブレイクスルー: 研究ポイントボーナス
    TechBreakthrough,
    /// 反乱: 人口・資源減少
    Rebellion,
    /// 交易ブーム: 全資源微増
    TradeBoom,
    /// 地震: 資源・人口に中ダメージ（地殻活動依存）
    Earthquake,
    /// 放射線嵐: 人口減少と電力低下（放射線依存）
    RadiationStorm,
    /// 隕石衝突: 甚大な被害（隕石リスク依存）
    MeteoriteImpact,
    /// 環境災害: 汚染による人口へのダメージ（汚染レベル依存）
    EnvironmentalDisaster,
}

impl EventKind {
    /// 全イベント種類のリスト
    pub fn all() -> &'static [EventKind] {
        &[
            EventKind::Plague,
            EventKind::BountifulHarvest,
            EventKind::MineralDiscovery,
            EventKind::BabyBoom,
            EventKind::EnergyCrisis,
            EventKind::TechBreakthrough,
            EventKind::Rebellion,
            EventKind::TradeBoom,
            EventKind::Earthquake,
            EventKind::RadiationStorm,
            EventKind::MeteoriteImpact,
            EventKind::EnvironmentalDisaster,
        ]
    }
}

/// イベントの効果を表す構造化データ
///
/// ドメインシステムが実際の値の変動量を記録し、
/// 表示レイヤーがこれを人間可読な文字列に変換する
#[derive(Debug, Clone, Default)]
pub struct EventEffect {
    /// 人口変動量（正: 増加、負: 減少）
    pub population_change: f64,
    /// 食料変動量
    pub food_change: f64,
    /// 鉱物変動量
    pub minerals_change: f64,
    /// エネルギー変動量
    pub energy_change: f64,
    /// 工業品変動量
    pub goods_change: f64,
    /// 研究ポイント変動量
    pub research_change: f64,
}

/// イベントログ（発生したイベントの履歴）
#[derive(Resource, Debug, Default)]
pub struct EventLog {
    pub events: Vec<EventRecord>,
}

/// 個別のイベント記録
#[derive(Debug, Clone)]
pub struct EventRecord {
    pub tick: u64,
    pub planet_name: String,
    pub kind: EventKind,
    pub effect: EventEffect,
}
