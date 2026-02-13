use bevy::prelude::*;

/// 技術分野
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TechField {
    /// 農業技術 → 食料産出ボーナス
    Agriculture,
    /// 採掘技術 → 鉱物産出ボーナス
    Mining,
    /// エネルギー技術 → エネルギー産出ボーナス
    EnergyTech,
    /// 工業技術 → 工業品産出ボーナス
    Manufacturing,
    /// 軍事技術 → 艦船の戦力ボーナス
    MilitaryTech,
    /// 宇宙航行技術 → 貿易容量ボーナス
    SpaceNavigation,
    /// 環境技術 → 汚染浄化ボーナス
    EnvironmentalTech,
    /// 核融合技術 → エネルギー産出・化石燃料依存度低減
    NuclearFusion,
}

/// 技術レベルと研究進捗を管理するコンポーネント（国家に付与）
#[derive(Component, Debug, Clone)]
pub struct TechnologyState {
    /// 各分野の技術レベル (0 〜)
    pub agriculture_level: u32,
    pub mining_level: u32,
    pub energy_level: u32,
    pub manufacturing_level: u32,
    pub military_level: u32,
    pub navigation_level: u32,
    pub environmental_level: u32,
    pub nuclear_fusion_level: u32,

    /// 現在研究中の分野
    pub current_research: TechField,
    /// 現在の研究ポイント蓄積
    pub research_points: f64,

    /// 1 Tick あたりの研究ポイント産出量（人口・リソース依存）
    pub research_rate: f64,
}

impl TechnologyState {
    pub fn new() -> Self {
        Self {
            agriculture_level: 0,
            mining_level: 0,
            energy_level: 0,
            manufacturing_level: 0,
            military_level: 0,
            navigation_level: 0,
            environmental_level: 0,
            nuclear_fusion_level: 0,
            current_research: TechField::Agriculture,
            research_points: 0.0,
            research_rate: 1.0,
        }
    }

    /// 指定された分野の技術レベルを取得
    pub fn level(&self, field: TechField) -> u32 {
        match field {
            TechField::Agriculture => self.agriculture_level,
            TechField::Mining => self.mining_level,
            TechField::EnergyTech => self.energy_level,
            TechField::Manufacturing => self.manufacturing_level,
            TechField::MilitaryTech => self.military_level,
            TechField::SpaceNavigation => self.navigation_level,
            TechField::EnvironmentalTech => self.environmental_level,
            TechField::NuclearFusion => self.nuclear_fusion_level,
        }
    }

    /// レベルアップに必要な研究ポイント
    /// レベルが上がるほど必要ポイントが増加
    pub fn cost_for_next_level(&self, field: TechField, balance: &crate::config::BalanceConfig) -> f64 {
        let current = self.level(field);
        balance.research_cost_base * (balance.research_cost_multiplier.powi(current as i32))
    }

    /// 分野の産出ボーナス倍率 (1.0 + level * bonus_per_level)
    pub fn bonus_multiplier(&self, field: TechField, balance: &crate::config::BalanceConfig) -> f64 {
        1.0 + self.level(field) as f64 * balance.tech_bonus_per_level
    }

    /// レベルアップ処理
    pub fn level_up(&mut self, field: TechField) {
        match field {
            TechField::Agriculture => self.agriculture_level += 1,
            TechField::Mining => self.mining_level += 1,
            TechField::EnergyTech => self.energy_level += 1,
            TechField::Manufacturing => self.manufacturing_level += 1,
            TechField::MilitaryTech => self.military_level += 1,
            TechField::SpaceNavigation => self.navigation_level += 1,
            TechField::EnvironmentalTech => self.environmental_level += 1,
            TechField::NuclearFusion => self.nuclear_fusion_level += 1,
        }
    }

    /// 研究ポイントを加算し、レベルアップがあれば適用
    /// レベルアップした場合は true を返す
    #[allow(dead_code)]
    pub fn add_research(&mut self, points: f64, balance: &crate::config::BalanceConfig) -> bool {
        self.research_points += points;
        let cost = self.cost_for_next_level(self.current_research, balance);
        if self.research_points >= cost {
            self.research_points -= cost;
            self.level_up(self.current_research);
            // 次の研究分野を自動ローテーション
            self.current_research = self.next_research_field();
            true
        } else {
            false
        }
    }

    /// 研究分野のローテーション（最もレベルが低い分野を選択）
    #[allow(dead_code)]
    fn next_research_field(&self) -> TechField {
        let fields = [
            (TechField::Agriculture, self.agriculture_level),
            (TechField::Mining, self.mining_level),
            (TechField::EnergyTech, self.energy_level),
            (TechField::Manufacturing, self.manufacturing_level),
            (TechField::MilitaryTech, self.military_level),
            (TechField::SpaceNavigation, self.navigation_level),
            (TechField::EnvironmentalTech, self.environmental_level),
            (TechField::NuclearFusion, self.nuclear_fusion_level),
        ];
        fields.iter().min_by_key(|(_, level)| *level).unwrap().0
    }

    /// 総合技術レベル
    pub fn total_level(&self) -> u32 {
        self.agriculture_level
            + self.mining_level
            + self.energy_level
            + self.manufacturing_level
            + self.military_level
            + self.navigation_level
            + self.environmental_level
            + self.nuclear_fusion_level
    }
}
