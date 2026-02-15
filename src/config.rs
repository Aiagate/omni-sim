use bevy::prelude::*;

/// シミュレーションのバランス調整用定数
#[derive(Debug, Clone)]
pub struct BalanceConfig {
    /// 人口あたりの食料消費率
    pub food_consumption_per_pop: f64,
    /// 人口あたりのエネルギー消費率のベース
    pub energy_consumption_per_pop_base: f64,
    /// 温度によるエネルギー消費増の係数
    pub energy_temp_penalty_coeff: f64,
    /// 工業生産に必要な鉱物の比率 (1.0 の工業品生産に 0.5 の鉱物が必要など)
    pub mineral_consumption_mf_ratio: f64,
    /// 工業生産に必要なレアアースの比率
    pub rare_earth_consumption_mf_ratio: f64,
    
    /// 距離あたりの貿易コスト係数
    pub trade_cost_per_ly: f64,
    /// 貿易コストの最大値 (0.0 〜 1.0)
    pub max_trade_cost: f64,
    
    /// 研究コストのベース値
    pub research_cost_base: f64,
    /// 研究レベル上昇に伴うコスト倍率
    pub research_cost_multiplier: f64,
    /// 技術レベルあたりのボーナス倍率
    pub tech_bonus_per_level: f64,

    /// 工業生産による大気汚染係数
    pub air_pollution_mf_coeff: f64,
    /// 化石燃料消費による大気汚染係数
    pub air_pollution_fossil_coeff: f64,
    /// 鉱物採掘による水質汚染係数
    pub water_pollution_mining_coeff: f64,
    /// 環境浄化のベース値
    pub base_cleanup_rate: f64,
    /// 環境技術による浄化ボーナス係数
    pub env_tech_cleanup_bonus_coeff: f64,
    /// 汚染による健康（成長率）ペナルティ係数
    pub health_penalty_growth_coeff: f64,
    
    /// テラフォーミング開始に必要な環境技術レベル
    pub terraforming_min_env_tech: u32,
    /// 各フェーズのコストと進捗率
    pub tf_phase_atmospheric_costs: (f64, f64, f64, f64), // (energy, goods, food, progress)
    pub tf_phase_temp_costs: (f64, f64, f64, f64),
    pub tf_phase_water_costs: (f64, f64, f64, f64),
    pub tf_phase_bio_costs: (f64, f64, f64, f64),

    /// 軍事技術による建造コスト削減係数 (1レベルあたり)
    pub military_tech_cost_reduction: f64,
    /// 宇宙航行技術による貿易容量ボーナス係数 (1レベルあたり)
    pub navigation_tech_capacity_bonus: f64,
    /// FTL技術による貿易コスト削減係数 (1レベルあたり)
    pub ftl_tech_cost_reduction: f64,
    /// ナノテクによる工業品産出ボーナス係数 (1レベルあたり)
    pub nanotech_mf_bonus: f64,
    /// バイオテクによる人口成長ボーナス係数 (1レベルあたり)
    pub biotech_growth_bonus: f64,
}

impl Default for BalanceConfig {
    fn default() -> Self {
        Self {
            food_consumption_per_pop: 0.00001,
            energy_consumption_per_pop_base: 0.000005,
            energy_temp_penalty_coeff: 0.02,
            mineral_consumption_mf_ratio: 0.5,
            rare_earth_consumption_mf_ratio: 0.1,
            trade_cost_per_ly: 0.05,
            max_trade_cost: 0.5,
            research_cost_base: 100.0,
            research_cost_multiplier: 1.5,
            tech_bonus_per_level: 0.10,
            air_pollution_mf_coeff: 0.00005,
            air_pollution_fossil_coeff: 0.00002,
            water_pollution_mining_coeff: 0.00002,
            base_cleanup_rate: 0.0005,
            env_tech_cleanup_bonus_coeff: 0.0005,
            health_penalty_growth_coeff: 0.005,
            terraforming_min_env_tech: 5,
            tf_phase_atmospheric_costs: (50.0, 0.0, 0.0, 0.02),
            tf_phase_temp_costs: (30.0, 0.0, 0.0, 0.033),
            tf_phase_water_costs: (20.0, 10.0, 0.0, 0.025),
            tf_phase_bio_costs: (10.0, 5.0, 10.0, 0.016),
            military_tech_cost_reduction: 0.05,
            navigation_tech_capacity_bonus: 0.15,
            ftl_tech_cost_reduction: 0.30,
            nanotech_mf_bonus: 0.30,
            biotech_growth_bonus: 0.05,
        }
    }
}

/// シミュレーション全体の設定を保持するリソース
#[derive(Resource, Debug, Clone)]
pub struct SimulationConfig {
    /// シミュレーションの最大 Tick 数
    pub max_ticks: u64,
    /// 乱数シード（決定論的再現性のため）
    pub seed: u64,
    /// TUI モードで起動するかどうか
    pub tui_mode: bool,
    /// シミュレーションの最大 TPS (Ticks Per Second)
    pub max_tps: Option<u64>,
    /// 表示する最低重要度
    pub min_importance: crate::components::simulation_event::EventImportance,
    /// バランス調整用定数
    pub balance: BalanceConfig,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            max_ticks: 50,
            seed: 42,
            tui_mode: false,
            max_tps: None,
            min_importance: crate::components::simulation_event::EventImportance::Low,
            balance: BalanceConfig::default(),
        }
    }
}
