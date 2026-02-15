# 提案 2: テクノロジーツリー

## 概要

現在の技術システムは6分野が独立しており、最低レベル分野に自動ローテーションするだけの単純な構造。**技術間の前提条件・シナジー・分岐選択**を導入することで、国家ごとに異なる技術戦略が生まれ、戦略的深みが劇的に増す。

---

## 現状の課題

| 課題 | 現行の挙動 |
|------|-----------|
| 戦略性の欠如 | 全国家が同じ順序で均等にレベルアップ |
| 分岐がない | プレイヤー/AIの選択肢がゼロ |
| 技術の相互作用なし | 農業Lv5と工業Lv5の組み合わせでシナジーが発生しない |
| 将来用の死に分野 | `MilitaryTech` と `SpaceNavigation` が効果のない空き分野 |
| ゲーム後半の停滞 | 全分野がLv5超えると新しいことが何も起きない |

---

## 設計

### 1. 技術ツリー構造

技術を **基礎技術**（既存6分野）と **応用技術**（新規追加）の2層構造にする。

#### 基礎技術（Tier 1 — 前提条件なし）

| ID | 分野 | 効果 | 最大レベル |
|----|------|------|-----------|
| `agriculture` | 農業 | 食料産出 +10%/Lv | 10 |
| `mining` | 採掘 | 鉱物産出 +10%/Lv | 10 |
| `energy` | エネルギー | エネルギー産出 +10%/Lv | 10 |
| `manufacturing` | 工業 | 工業品産出 +10%/Lv | 10 |
| `military` | 軍事 | 艦船戦力 +10%/Lv, 建造コスト -5%/Lv | 10 |
| `navigation` | 宇宙航行 | 貿易容量 +15%/Lv, 輸送コスト -3%/Lv | 10 |

#### 応用技術（Tier 2 — 前提条件あり）

| ID | 名称 | 前提条件 | 効果 | 最大Lv |
|----|------|---------|------|--------|
| `biotech` | バイオテクノロジー | agriculture ≥ 3 | 人口成長率 +5%/Lv, 疫病耐性 | 5 |
| `deep_mining` | 深層採掘 | mining ≥ 3 | 埋蔵量に関係なく一定産出を保証 | 5 |
| `fusion` | 核融合 | energy ≥ 4 | エネルギー産出 2倍, 維持コスト -20% | 3 |
| `nanotech` | ナノ製造 | manufacturing ≥ 4 | 工業品産出 +30%/Lv, 建造速度 +50% | 3 |
| `shields` | シールド技術 | military ≥ 3, energy ≥ 2 | 戦闘被害 -15%/Lv | 3 |
| `ftl` | 超光速航行 | navigation ≥ 4, energy ≥ 3 | 輸送コスト実質ゼロ化 | 3 |

#### 高等技術（Tier 3 — 複数の応用技術が前提）

| ID | 名称 | 前提条件 | 効果 |
|----|------|---------|------|
| `terraforming` | テラフォーミング | biotech ≥ 3, energy ≥ 5 | 惑星の居住性を改善 |
| `dyson_sphere` | ダイソンスフィア | fusion ≥ 3, manufacturing ≥ 6 | エネルギー問題の完全解決 |
| `colony_ship` | 植民船 | ftl ≥ 2, nanotech ≥ 2 | 新惑星の開拓が可能に |
| `megastructure` | メガストラクチャー | nanotech ≥ 3, deep_mining ≥ 3 | 全資源産出 +100% |
| `psi_tech` | 超能力技術 | biotech ≥ 5, shields ≥ 3 | 諜報能力大幅強化 |

### 2. データ構造

```rust
/// 技術の定義（静的データ、ハードコードまたはTOMLから読み込み）
#[derive(Debug, Clone)]
pub struct TechDefinition {
    pub id: TechId,
    pub name: String,
    pub tier: u8,                          // 1, 2, 3
    pub max_level: u32,
    pub prerequisites: Vec<(TechId, u32)>, // (技術ID, 必要レベル) のリスト
    pub base_cost: f64,                    // Level 0→1 のコスト
    pub cost_scaling: f64,                 // レベルごとのコスト倍率
    pub effects: Vec<TechEffect>,
}

/// 技術効果の種類
#[derive(Debug, Clone)]
pub enum TechEffect {
    ProductionBonus {
        resource: ResourceType,
        bonus_per_level: f64,   // +10% = 0.10
    },
    MilitaryBonus {
        power_per_level: f64,
        cost_reduction_per_level: f64,
    },
    TradeBonus {
        capacity_per_level: f64,
        cost_reduction_per_level: f64,
    },
    PopulationBonus {
        growth_per_level: f64,
    },
    CombatDefense {
        damage_reduction_per_level: f64,
    },
    Unlock {
        feature: UnlockableFeature,
    },
}

/// 技術によってアンロックされる機能
#[derive(Debug, Clone)]
pub enum UnlockableFeature {
    Terraforming,       // 惑星環境改善
    ColonyShip,         // 新惑星開拓
    DysonSphere,        // 無限エネルギー構造物
    Megastructure,      // 巨大建造物
    AsteroidMining,     // 小惑星採掘
    EspionageNetwork,   // 諜報網
}

/// 惑星の技術状態（コンポーネント、現行の TechnologyState を拡張）
#[derive(Component, Debug, Clone)]
pub struct TechnologyState {
    /// 各技術のレベル（TechId → Level）
    pub levels: HashMap<TechId, u32>,
    /// 現在研究中の技術
    pub current_research: TechId,
    /// 蓄積された研究ポイント
    pub research_points: f64,
    /// 研究レート（/Tick）
    pub research_rate: f64,
    /// アンロック済み機能
    pub unlocked_features: HashSet<UnlockableFeature>,
}
```

### 3. 技術シナジーシステム

特定の技術の組み合わせで追加ボーナスが発生する。

```
シナジー定義:
  agriculture ≥ 3 AND manufacturing ≥ 3 → "食品加工産業" → 食料産出 +20%
  military ≥ 3 AND shields ≥ 2 → "防衛ドクトリン" → 戦闘被害 -30%
  navigation ≥ 3 AND trade の貿易ルート数 ≥ 3 → "通商連合" → 全貿易容量 +50%
  fusion ≥ 2 AND manufacturing ≥ 5 → "産業革命" → 全生産 +25%
```

### 4. 研究優先度の意思決定

自動ローテーション（現行）を廃止し、性格ベース or 状況ベースの選択に置換。

```
fn choose_next_research(
    state: &TechnologyState,
    character: &NationalCharacter,
    situation: &NationSituation,
    tech_defs: &[TechDefinition],
) -> TechId {
    // 1. 前提条件を満たし、最大レベルに達していない技術を列挙
    let available: Vec<_> = tech_defs.iter()
        .filter(|def| state.can_research(def))
        .collect();

    // 2. 各技術にスコアを付ける
    for tech in &available {
        let mut score = 0.0;

        // 国家性格による重み付け
        score += match tech.primary_category() {
            Military => character.aggression * 10.0,
            Economic => character.trade_affinity * 10.0,
            Science  => character.research_focus * 10.0,
            // ...
        };

        // 状況による補正
        if situation.at_war && tech.has_military_effect() {
            score += 20.0;  // 戦争中は軍事技術を急ぐ
        }
        if situation.food_shortage && tech.id == "agriculture" {
            score += 15.0;  // 飢餓時は農業を優先
        }

        // Tier の低い（＝研究が早い）技術にボーナス
        score += (4 - tech.tier) as f64 * 5.0;
    }

    // 3. 最高スコアの技術を選択
}
```

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `components/technology.rs` | `TechnologyState` を HashMap ベースに全面改修 |
| `systems/research.rs` | 自動ローテーションを意思決定ロジックに置換 |
| `systems/economy.rs` | 技術ボーナス計算を `TechEffect` から動的に取得 |
| `systems/military.rs` | 軍事技術ボーナスを適用 |
| `systems/trade.rs` | 航行技術ボーナスを適用 |
| `plugins/world_init.rs` | 技術定義データの初期化 |
| `components/simulation_event.rs` | `TechUnlocked` イベントを追加 |

---

## 期待される効果

1. **戦略的分岐**: 「核融合に全力投資 vs 軍事技術で即座に制圧」の選択
2. **ゲーム後半の目標**: Tier 3 技術が「勝利条件」に近い位置づけになる
3. **国家の個性化**: AI性格×技術ツリーで固有の発展パターンが生まれる
4. **リプレイ性**: 異なる技術ルートを試す動機が生まれる

---

## 実装の段階的アプローチ

| フェーズ | 内容 | 工数目安 |
|---------|------|---------|
| Phase 1 | 既存6分野に `MilitaryTech`/`SpaceNavigation` 効果を実装 | 小 |
| Phase 2 | `TechDefinition` + 前提条件システムの導入 | 中 |
| Phase 3 | Tier 2 応用技術6種の追加 | 中 |
| Phase 4 | シナジーシステムとTier 3高等技術 | 中 |
| Phase 5 | 性格ベース研究優先度の意思決定 | 小 |
