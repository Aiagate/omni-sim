use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// 各技術の一意な識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TechId {
    // Tier 1: 基礎技術
    Agriculture,
    Mining,
    Energy,
    Manufacturing,
    Military,
    Navigation,
    Environmental,
    
    // Tier 2: 応用技術
    Biotech,
    DeepMining,
    Fusion,
    Nanotech,
    Shields,
    FTL,

    // Tier 3: 高等技術
    Terraforming,
    DysonSphere,
    ColonyShip,
    Megastructure,
    PsiTech,
}

/// 技術によってアンロックされる機能
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum UnlockableFeature {
    Terraforming,
    AstralMining,
    ColonyShip,
    DysonSphere,
}

/// 技術効果の種類
#[derive(Debug, Clone)]
pub enum TechEffect {
    ProductionBonus {
        resource_pct: f64, // 0.1 = +10%
    },
    MilitaryPowerBonus(#[allow(dead_code)] f64),
    MilitaryCostReduction(#[allow(dead_code)] f64),
    TradeCapacityBonus(#[allow(dead_code)] f64),
    TradeCostReduction(#[allow(dead_code)] f64),
    CleanupBonus(#[allow(dead_code)] f64),
    PopulationGrowthBonus(#[allow(dead_code)] f64),
    CombatDamageReduction(#[allow(dead_code)] f64),
    MiningStabilityBonus, // 資源量低下の影響を固定値で相殺
    Unlock(UnlockableFeature),
}

/// 技術の定義（静的データ）
#[derive(Debug, Clone)]
pub struct TechDefinition {
    pub id: TechId,
    pub name: String,
    pub tier: u8,
    pub max_level: u32,
    pub prerequisites: Vec<(TechId, u32)>,
    pub base_cost: f64,
    pub effects: Vec<TechEffect>,
}

impl TechDefinition {
    pub fn all() -> Vec<Self> {
        vec![
            // Tier 1
            Self {
                id: TechId::Agriculture,
                name: "Agriculture".to_string(),
                tier: 1,
                max_level: 10,
                prerequisites: vec![],
                base_cost: 100.0,
                effects: vec![TechEffect::ProductionBonus { resource_pct: 0.1 }],
            },
            Self {
                id: TechId::Mining,
                name: "Mining".to_string(),
                tier: 1,
                max_level: 10,
                prerequisites: vec![],
                base_cost: 100.0,
                effects: vec![TechEffect::ProductionBonus { resource_pct: 0.1 }],
            },
            Self {
                id: TechId::Energy,
                name: "Energy".to_string(),
                tier: 1,
                max_level: 10,
                prerequisites: vec![],
                base_cost: 100.0,
                effects: vec![TechEffect::ProductionBonus { resource_pct: 0.1 }],
            },
            Self {
                id: TechId::Manufacturing,
                name: "Manufacturing".to_string(),
                tier: 1,
                max_level: 10,
                prerequisites: vec![],
                base_cost: 100.0,
                effects: vec![TechEffect::ProductionBonus { resource_pct: 0.1 }],
            },
            Self {
                id: TechId::Military,
                name: "Military".to_string(),
                tier: 1,
                max_level: 10,
                prerequisites: vec![],
                base_cost: 100.0,
                effects: vec![TechEffect::MilitaryPowerBonus(0.1), TechEffect::MilitaryCostReduction(0.05)],
            },
            Self {
                id: TechId::Navigation,
                name: "Navigation".to_string(),
                tier: 1,
                max_level: 10,
                prerequisites: vec![],
                base_cost: 100.0,
                effects: vec![TechEffect::TradeCapacityBonus(0.15), TechEffect::TradeCostReduction(0.03)],
            },
            Self {
                id: TechId::Environmental,
                name: "Environmental".to_string(),
                tier: 1,
                max_level: 10,
                prerequisites: vec![],
                base_cost: 100.0,
                effects: vec![TechEffect::CleanupBonus(1.0)],
            },
            // Tier 2
            Self {
                id: TechId::Biotech,
                name: "Biotechnology".to_string(),
                tier: 2,
                max_level: 5,
                prerequisites: vec![(TechId::Agriculture, 3)],
                base_cost: 300.0,
                effects: vec![TechEffect::PopulationGrowthBonus(0.05)],
            },
            Self {
                id: TechId::DeepMining,
                name: "Deep Mining".to_string(),
                tier: 2,
                max_level: 5,
                prerequisites: vec![(TechId::Mining, 3)],
                base_cost: 300.0,
                effects: vec![TechEffect::MiningStabilityBonus],
            },
            Self {
                id: TechId::Fusion,
                name: "Nuclear Fusion".to_string(),
                tier: 2,
                max_level: 3,
                prerequisites: vec![(TechId::Energy, 4)],
                base_cost: 500.0,
                effects: vec![TechEffect::ProductionBonus { resource_pct: 0.5 }],
            },
            Self {
                id: TechId::Nanotech,
                name: "Nanotechnology".to_string(),
                tier: 2,
                max_level: 3,
                prerequisites: vec![(TechId::Manufacturing, 4)],
                base_cost: 500.0,
                effects: vec![TechEffect::ProductionBonus { resource_pct: 0.3 }],
            },
            Self {
                id: TechId::Shields,
                name: "Shield Technology".to_string(),
                tier: 2,
                max_level: 3,
                prerequisites: vec![(TechId::Military, 3), (TechId::Energy, 2)],
                base_cost: 400.0,
                effects: vec![TechEffect::CombatDamageReduction(0.15)],
            },
            Self {
                id: TechId::FTL,
                name: "FTL Navigation".to_string(),
                tier: 2,
                max_level: 3,
                prerequisites: vec![(TechId::Navigation, 4), (TechId::Energy, 3)],
                base_cost: 600.0,
                effects: vec![TechEffect::TradeCostReduction(0.30)],
            },
            // Tier 3
            Self {
                id: TechId::Terraforming,
                name: "Advanced Terraforming".to_string(),
                tier: 3,
                max_level: 1,
                prerequisites: vec![(TechId::Environmental, 5), (TechId::Biotech, 3)],
                base_cost: 2000.0,
                effects: vec![TechEffect::Unlock(UnlockableFeature::Terraforming)],
            },
            Self {
                id: TechId::DysonSphere,
                name: "Dyson Sphere".to_string(),
                tier: 3,
                max_level: 1,
                prerequisites: vec![(TechId::Fusion, 3), (TechId::Megastructure, 1)],
                base_cost: 5000.0,
                effects: vec![TechEffect::Unlock(UnlockableFeature::DysonSphere)],
            },
            Self {
                id: TechId::ColonyShip,
                name: "Interstellar Colony Ship".to_string(),
                tier: 3,
                max_level: 1,
                prerequisites: vec![(TechId::FTL, 3), (TechId::Biotech, 3)],
                base_cost: 3000.0,
                effects: vec![TechEffect::Unlock(UnlockableFeature::ColonyShip)],
            },
            Self {
                id: TechId::Megastructure,
                name: "Megastructure Engineering".to_string(),
                tier: 3,
                max_level: 5,
                prerequisites: vec![(TechId::Nanotech, 3), (TechId::DeepMining, 3)],
                base_cost: 1500.0,
                effects: vec![TechEffect::ProductionBonus { resource_pct: 0.5 }],
            },
            Self {
                id: TechId::PsiTech,
                name: "Psionic Technology".to_string(),
                tier: 3,
                max_level: 5,
                prerequisites: vec![(TechId::Biotech, 5), (TechId::Energy, 5)],
                base_cost: 2000.0,
                effects: vec![TechEffect::MilitaryPowerBonus(0.5)],
            },
        ]
    }
}

/// 技術レベルと研究進捗を管理するコンポーネント（国家に付与）
#[derive(Component, Debug, Clone)]
pub struct TechnologyState {
    /// 各技術のレベル (TechId → Level)
    pub levels: HashMap<TechId, u32>,
    /// アンロック済み機能
    pub unlocked_features: HashSet<UnlockableFeature>,

    /// 現在研究中の技術
    pub current_research: TechId,
    /// 現在の研究ポイント蓄積
    pub research_points: f64,
    /// 1 Tick あたりの研究ポイント産出量（人口・リソース依存）
    pub research_rate: f64,
}

impl TechnologyState {
    pub fn new() -> Self {
        let mut levels = HashMap::new();
        // 初期状態では基礎技術のみLv0で登録
        levels.insert(TechId::Agriculture, 0);
        levels.insert(TechId::Mining, 0);
        levels.insert(TechId::Energy, 0);
        levels.insert(TechId::Manufacturing, 0);
        levels.insert(TechId::Military, 0);
        levels.insert(TechId::Navigation, 0);
        levels.insert(TechId::Environmental, 0);
        
        levels.insert(TechId::Biotech, 0);
        levels.insert(TechId::DeepMining, 0);
        levels.insert(TechId::Fusion, 0);
        levels.insert(TechId::Nanotech, 0);
        levels.insert(TechId::Shields, 0);
        levels.insert(TechId::FTL, 0);

        levels.insert(TechId::Terraforming, 0);
        levels.insert(TechId::DysonSphere, 0);
        levels.insert(TechId::ColonyShip, 0);
        levels.insert(TechId::Megastructure, 0);
        levels.insert(TechId::PsiTech, 0);

        Self {
            levels,
            unlocked_features: HashSet::new(),
            current_research: TechId::Agriculture,
            research_points: 0.0,
            research_rate: 1.0,
        }
    }

    /// 指定された技術のレベルを取得
    pub fn level(&self, id: TechId) -> u32 {
        *self.levels.get(&id).unwrap_or(&0)
    }

    /// レベルアップに必要な研究ポイント
    pub fn cost_for_next_level(&self, id: TechId, balance: &crate::config::BalanceConfig) -> f64 {
        let current = self.level(id);
        balance.research_cost_base * (balance.research_cost_multiplier.powi(current as i32))
    }

    /// 技術ボーナス倍率 (1.0 + level * bonus_per_level)
    pub fn bonus_multiplier(&self, id: TechId, balance: &crate::config::BalanceConfig) -> f64 {
        1.0 + self.level(id) as f64 * balance.tech_bonus_per_level
    }

    /// レベルアップ処理
    pub fn level_up(&mut self, id: TechId) {
        let entry = self.levels.entry(id).or_insert(0);
        *entry += 1;
    }

    /// 総合技術レベル
    pub fn total_level(&self) -> u32 {
        self.levels.values().sum()
    }

    /// 技術の前提条件を満たしているかチェック
    pub fn can_research(&self, def: &TechDefinition) -> bool {
        if self.level(def.id) >= def.max_level {
            return false;
        }
        for (pre_id, pre_lv) in &def.prerequisites {
            if self.level(*pre_id) < *pre_lv {
                return false;
            }
        }
        true
    }

    /// 特定の技術の組み合わせによるシナジーボーナスを取得
    pub fn synergy_bonus(&self, id: TechId) -> f64 {
        match id {
            TechId::Mining => {
                // Nanotech + DeepMining シナジー: 資源抽出効率向上
                if self.level(TechId::Nanotech) >= 1 && self.level(TechId::DeepMining) >= 1 {
                    return 0.2; // +20%
                }
            }
            TechId::Military => {
                // Shields + PsiTech シナジー: 鉄壁の防衛
                if self.level(TechId::Shields) >= 2 && self.level(TechId::PsiTech) >= 1 {
                    return 0.3; // +30%
                }
            }
            TechId::Navigation => {
                // FTL + Megastructure シナジー: ゲートウェイ網
                if self.level(TechId::FTL) >= 2 && self.level(TechId::Megastructure) >= 1 {
                    return 0.25; // +25% cost reduction
                }
            }
            _ => {}
        }
        0.0
    }

    /// 指定された機能がアンロックされているかチェック
    pub fn is_feature_unlocked(&self, feature: UnlockableFeature) -> bool {
        self.unlocked_features.contains(&feature)
    }

    /// 技術効果からアンロック機能を登録
    pub fn add_unlocks(&mut self, def: &TechDefinition) {
        if self.level(def.id) > 0 {
            for effect in &def.effects {
                if let TechEffect::Unlock(feature) = effect {
                    self.unlocked_features.insert(*feature);
                }
            }
        }
    }
}
