use bevy::prelude::*;

/// 国家の性格パラメータ（0.0 〜 1.0）
#[derive(Component, Debug, Clone)]
pub struct NationalCharacter {
    /// 好戦性: 高いほど先制攻撃や軍備拡張を好む
    pub aggression: f64,
    /// 交易志向: 高いほど貿易ルート維持や通商条約を好む
    pub trade_affinity: f64,
    /// 技術志向: 高いほど研究投資を優先し、技術共有に積極的
    pub research_focus: f64,
    /// 拡張主義: 高いほど未開惑星への入植を優先
    pub expansionism: f64,
    /// 外交柔軟性: 高いほど関係変動の振幅が大きく、同盟を組みやすい
    pub diplomacy_flexibility: f64,
}

impl Default for NationalCharacter {
    fn default() -> Self {
        Self {
            aggression: 0.5,
            trade_affinity: 0.5,
            research_focus: 0.5,
            expansionism: 0.5,
            diplomacy_flexibility: 0.5,
        }
    }
}

/// 国家の質的な特性（Trait）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NationalTrait {
    /// 集合意識: 内部派閥が存在せず、性格が極めて安定
    CollectiveMind,
    /// 戦士文化: 軍事力ボーナス、平和時の好感度低下
    WarriorCulture,
    /// 平和主義者: 戦争コスト増、交易ボーナス
    Pacifist,
    /// 商人ギルド: 貿易効率向上
    MerchantGuild,
    /// 技術至上主義: 研究速度向上、技術格差に敏感
    Technocracy,
}

#[derive(Component, Debug, Clone)]
pub struct NationalIdentity {
    #[allow(dead_code)]
    pub traits: Vec<NationalTrait>,
}

/// 外交上の重要な出来事
#[derive(Debug, Clone)]
pub struct AIEvent {
    #[allow(dead_code)]
    pub tick: u64,
    #[allow(dead_code)]
    pub description: String,
    pub trust_impact: f64,
}

/// 国家の記憶システム
#[derive(Component, Debug, Clone)]
pub struct NationalMemory {
    /// 相手国ごとの出来事履歴
    pub events: Vec<(Entity, AIEvent)>,
}

impl NationalMemory {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// 特定の相手国に対する現在の合計信頼度（Trust）を算出
    pub fn calculate_trust(&self, target: Entity) -> f64 {
        self.events
            .iter()
            .filter(|(e, _)| *e == target)
            .map(|(_, ev)| ev.trust_impact)
            .sum()
    }

    /// 新しい出来事を記録
    pub fn record_event(&mut self, target: Entity, event: AIEvent) {
        self.events.push((target, event));
    }
}

/// 国家内部の派閥
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FactionType {
    Militarist,
    Merchant,
    Technocrat,
}

/// 派閥のパワーバランス
#[derive(Component, Debug, Clone)]
pub struct InternalFactions {
    /// 各派閥の勢力（合計 1.0 に正規化される想定）
    pub power: std::collections::HashMap<FactionType, f64>,
}

impl InternalFactions {
    pub fn new(militarist: f64, merchant: f64, technocrat: f64) -> Self {
        let mut power = std::collections::HashMap::new();
        power.insert(FactionType::Militarist, militarist);
        power.insert(FactionType::Merchant, merchant);
        power.insert(FactionType::Technocrat, technocrat);
        Self { power }
    }

    /// 派閥による性格パラメータへのオフセットを算出
    pub fn get_character_offset(&self) -> (f64, f64, f64) {
        let mil = self.power.get(&FactionType::Militarist).unwrap_or(&0.0);
        let mer = self.power.get(&FactionType::Merchant).unwrap_or(&0.0);
        let tec = self.power.get(&FactionType::Technocrat).unwrap_or(&0.0);

        // (aggression_offset, trade_offset, research_offset)
        (
            mil * 0.3 - mer * 0.1, // 軍部は好戦性を上げ、商人は下げる
            mer * 0.4,             // 商人は交易志向を上げる
            tec * 0.4,             // 技術者は技術志向を上げる
        )
    }
}
