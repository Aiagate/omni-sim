use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::components::economy::ResourceType;
use crate::components::events::{EventEffect, EventKind};
use crate::components::technology::TechId;

// ============================================================
// 惑星スナップショット
// ============================================================

/// 惑星の状態を TUI 表示用にスナップショットとして保持する
#[derive(Clone, Debug)]
pub struct PlanetSnapshot {
    pub name: String,
    pub population: f64,
    pub growth_rate: f64,
    pub food: f64,
    pub minerals: f64,
    pub energy: f64,
    pub manufactured_goods: f64,
    pub ships: u32,
    pub power: f64,
    pub in_combat: bool,
    pub starvation_ticks: u32,
    pub habitability: f64,
    pub population_capacity: f64,
    pub mineral_reserves_pct: f64,
    pub pollution: f64,
    pub soil_fertility: f64,
}

#[derive(Clone, Debug)]
pub struct StarSystemSnapshot {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub is_discovered: bool,
}

// ============================================================
// 国家スナップショット
// ============================================================

/// 国家の状態を TUI 表示用にスナップショットとして保持する
#[derive(Clone, Debug)]
pub struct NationSnapshot {
    pub name: String,
    pub total_population: f64,
    pub total_ships: u32,
    pub total_power: f64,
    pub tech_total: u32,
    pub tech_levels: [u32; 8], // 農業, 採掘, ｴﾈﾙｷﾞｰ, 工業, 軍事, 航行, 環境, 核融合
    pub aggression: f64,
    pub research_focus: f64,
    pub trade_affinity: f64,
    pub expansionism: f64,
    pub character_label: String,
    pub planets: Vec<String>, // 保有する惑星名のリスト
}

// ============================================================
// 国家履歴
// ============================================================

/// 国家の資源・人口推移を保持する
#[derive(Clone, Debug, Default)]
pub struct NationHistory {
    pub population: Vec<u64>,
    pub power: Vec<u64>,
    pub food: Vec<u64>,
    pub minerals: Vec<u64>,
    pub energy: Vec<u64>,
    pub goods: Vec<u64>,
}

impl NationHistory {
    pub fn push(&mut self, pop: f64, pwr: f64, f: f64, m: f64, e: f64, g: f64) {
        self.population.push(pop as u64);
        self.power.push(pwr as u64);
        self.food.push(f as u64);
        self.minerals.push(m as u64);
        self.energy.push(e as u64);
        self.goods.push(g as u64);

        if self.population.len() > 100 { self.population.remove(0); }
        if self.power.len() > 100 { self.power.remove(0); }
        if self.food.len() > 100 { self.food.remove(0); }
        if self.minerals.len() > 100 { self.minerals.remove(0); }
        if self.energy.len() > 100 { self.energy.remove(0); }
        if self.goods.len() > 100 { self.goods.remove(0); }
    }
}

// ============================================================
// イベントログエントリ
// ============================================================

/// TUI に表示するイベントログ1行分
#[derive(Clone, Debug)]
pub struct EventLogEntry {
    pub tick: u64,
    pub message: String,
    pub severity: EventSeverity,
}

/// イベントの深刻度（色分け用）
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventSeverity {
    Info,
    Warning,
    Critical,
}

// ============================================================
// 外交・戦争エントリ
// ============================================================

/// 外交関係の表示用エントリ
#[derive(Clone, Debug)]
pub struct DiplomacyEntry {
    pub relation_name: String,
    pub score: f64,
    pub status: String,
}

/// 進行中の戦争の表示用エントリ
#[derive(Clone, Debug)]
pub struct WarEntry {
    pub nation: String,
    pub started_at: u64,
    pub ships_lost: u32,
    pub casualties: f64,
}

// ============================================================
// シミュレーション速度
// ============================================================

/// シミュレーション速度設定
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimSpeed {
    X1,
    X5,
    X10,
}

impl SimSpeed {
    /// 次の速度に切り替え
    pub fn faster(self) -> Self {
        match self {
            SimSpeed::X1 => SimSpeed::X5,
            SimSpeed::X5 => SimSpeed::X10,
            SimSpeed::X10 => SimSpeed::X10,
        }
    }

    /// 前の速度に切り替え
    pub fn slower(self) -> Self {
        match self {
            SimSpeed::X1 => SimSpeed::X1,
            SimSpeed::X5 => SimSpeed::X1,
            SimSpeed::X10 => SimSpeed::X5,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SimSpeed::X1 => "x1",
            SimSpeed::X5 => "x5",
            SimSpeed::X10 => "x10",
        }
    }
}

// ============================================================
// メイン状態リソース
// ============================================================

/// TUI の全表示データを保持する Bevy Resource
#[derive(Resource)]
pub struct TuiAppState {
    pub planets: Vec<PlanetSnapshot>,
    pub systems: Vec<StarSystemSnapshot>,
    pub nations: Vec<NationSnapshot>,
    pub nation_histories: HashMap<String, NationHistory>,
    pub event_log: Vec<EventLogEntry>,
    pub diplomacy: Vec<DiplomacyEntry>,
    pub wars: Vec<WarEntry>,
    pub current_tick: u64,
    pub max_ticks: u64,
    pub seed: u64,
    pub selected_planet: usize,
    pub selected_nation: usize,
    pub log_scroll_offset: usize,
    pub view: TuiView,
    pub population_history: Vec<u64>,
    pub resource_history: Vec<u64>,
    pub tps: f64,
}

impl Default for TuiAppState {
    fn default() -> Self {
        Self {
            planets: Vec::new(),
            systems: Vec::new(),
            nations: Vec::new(),
            nation_histories: HashMap::new(),
            event_log: Vec::new(),
            diplomacy: Vec::new(),
            wars: Vec::new(),
            current_tick: 0,
            max_ticks: 0,
            seed: 0,
            selected_planet: 0,
            selected_nation: 0,
            log_scroll_offset: 0,
            view: TuiView::Main,
            population_history: Vec::new(),
            resource_history: Vec::new(),
            tps: 0.0,
        }
    }
}

impl TuiAppState {
    /// イベントログに新しいエントリを追加（最大 200 件保持）
    pub fn push_log(&mut self, tick: u64, message: String, severity: EventSeverity) {
        self.event_log.push(EventLogEntry {
            tick,
            message,
            severity,
        });
        if self.event_log.len() > 200 {
            self.event_log.remove(0);
        }
    }
}

// ============================================================
// シミュレーション制御リソース
// ============================================================

/// シミュレーションの実行制御（ポーズ、速度など）
#[derive(Resource)]
pub struct SimControl {
    pub paused: bool,
    pub stopped: bool,
    pub step_once: bool,
    pub speed: SimSpeed,
    pub quit_requested: bool,
}

impl Default for SimControl {
    fn default() -> Self {
        Self {
            paused: false,
            stopped: false,
            step_once: false,
            speed: SimSpeed::X1,
            quit_requested: false,
        }
    }
}

/// TUI の表示画面モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiView {
    Main,
    NationDetail,
}

/// シミュレーションが実行中かどうかを判定する Bevy Run Condition
pub fn simulation_running(sim_control: Option<Res<SimControl>>) -> bool {
    match sim_control {
        Some(control) => {
            if control.step_once {
                return true;
            }
            if control.stopped || control.paused {
                return false;
            }
            true
        }
        None => true, // CLI モード
    }
}

// ============================================================
// 表示ヘルパー関数（TUI 表示レイヤー専用）
// ============================================================

/// EventKind → 絵文字の変換（表示専用）
pub fn event_kind_emoji(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::Plague => "🦠",
        EventKind::BountifulHarvest => "🌾",
        EventKind::MineralDiscovery => "💎",
        EventKind::BabyBoom => "👶",
        EventKind::EnergyCrisis => "⚡",
        EventKind::TechBreakthrough => "💡",
        EventKind::Rebellion => "🔥",
        EventKind::TradeBoom => "📈",
        EventKind::Earthquake => "🌋",
        EventKind::RadiationStorm => "☢️",
        EventKind::MeteoriteImpact => "☄️",
        EventKind::EnvironmentalDisaster => "☣️",
    }
}

/// EventKind → 日本語名の変換（表示専用）
pub fn event_kind_name(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::Plague => "疫病の流行",
        EventKind::BountifulHarvest => "豊作",
        EventKind::MineralDiscovery => "鉱脈の発見",
        EventKind::BabyBoom => "ベビーブーム",
        EventKind::EnergyCrisis => "エネルギー危機",
        EventKind::TechBreakthrough => "技術ブレイクスルー",
        EventKind::Rebellion => "反乱",
        EventKind::TradeBoom => "交易ブーム",
        EventKind::Earthquake => "大地震",
        EventKind::RadiationStorm => "放射線嵐",
        EventKind::MeteoriteImpact => "隕石衝突",
        EventKind::EnvironmentalDisaster => "環境異常災害",
    }
}

/// ResourceType → 日本語名の変換（表示専用）
pub fn resource_type_name(rt: &ResourceType) -> &'static str {
    match rt {
        ResourceType::Food => "食料",
        ResourceType::Minerals => "鉱物",
        ResourceType::Energy => "ｴﾈﾙｷﾞｰ",
        ResourceType::ManufacturedGoods => "工業品",
    }
}

/// TechId → 日本語名の変換（表示専用）
pub fn tech_field_name(field: &TechId) -> &'static str {
    match field {
        TechId::Agriculture => "農業",
        TechId::Mining => "採掘",
        TechId::Energy => "ｴﾈﾙｷﾞｰ",
        TechId::Manufacturing => "工業",
        TechId::Military => "軍事",
        TechId::Navigation => "宇宙航行",
        TechId::Environmental => "環境技術",
        TechId::Biotech => "バイオ",
        TechId::DeepMining => "深部採掘",
        TechId::Fusion => "核融合",
        TechId::Nanotech => "ナノテク",
        TechId::Shields => "シールド",
        TechId::FTL => "FTL航法",
        TechId::Terraforming => "テラフォーミング",
        TechId::DysonSphere => "ダイソンスフィア",
        TechId::ColonyShip => "植民船",
        TechId::Megastructure => "メガ構造",
        TechId::PsiTech => "サイオニック",
    }
}

/// EventEffect → 人間可読な説明文の変換（表示専用）
pub fn format_event_effect(effect: &EventEffect) -> String {
    let mut parts: Vec<String> = Vec::new();

    if effect.population_change.abs() > 0.5 {
        parts.push(format!("人口 {:+.0}", effect.population_change));
    }
    if effect.food_change.abs() > 0.5 {
        parts.push(format!("食料 {:+.0}", effect.food_change));
    }
    if effect.minerals_change.abs() > 0.5 {
        parts.push(format!("鉱物 {:+.0}", effect.minerals_change));
    }
    if effect.energy_change.abs() > 0.5 {
        parts.push(format!("ｴﾈﾙｷﾞｰ {:+.0}", effect.energy_change));
    }
    if effect.goods_change.abs() > 0.5 {
        parts.push(format!("工業品 {:+.0}", effect.goods_change));
    }
    if effect.research_change.abs() > 0.5 {
        parts.push(format!("研究 {:+.0}", effect.research_change));
    }

    if parts.is_empty() {
        "効果なし".to_string()
    } else {
        parts.join(", ")
    }
}
