# 提案 1: 国家AI性格システム

## 概要

現在の外交システム（`diplomacy.rs`）は `sin(tick)` ベースの機械的な変動のみで、国家間の関係はすべて同じパターンで推移する。**国家ごとに異なる「性格」パラメータ**を導入し、意思決定ロジックを構築することで、毎回異なるドラマが自然発生するシミュレーションへ進化させる。

---

## 現状の課題

| 課題 | 現行の挙動 |
|------|-----------|
| 外交の単調さ | 全関係が一律 `-0.3/Tick + sin(tick×0.07)×0.4` で変動 |
| 戦争の予測可能性 | 常に同じTickタイミングで戦争が発生する |
| 国家の「個性」不在 | 帝国・共和国・連邦の名前だけで行動に差がない |
| プレイヤーの感情移入 | 国家が機械的すぎて「ストーリー」を感じにくい |

---

## 設計

### 1. 性格パラメータ（NationalCharacter コンポーネント）

```rust
/// 国家の性格・方針を定義するコンポーネント
#[derive(Component, Debug, Clone)]
pub struct NationalCharacter {
    // === 外交性格 (0.0 ~ 1.0) ===
    /// 好戦性: 高い → 先制攻撃を好む、軍備増強を優先
    pub aggression: f64,
    /// 交易志向: 高い → 貿易ルート拡大、通商条約を優先
    pub trade_affinity: f64,
    /// 技術志向: 高い → 研究投資を優先、技術共有に積極的
    pub research_focus: f64,
    /// 拡張主義: 高い → 植民・領土拡大を優先
    pub expansionism: f64,
    /// 外交柔軟性: 高い → 同盟形成しやすいが裏切りもある
    pub diplomacy_flexibility: f64,

    // === 意思決定の閾値 ===
    /// 戦争を仕掛ける軍事力優位倍率
    pub war_power_threshold: f64,       // default: 1.5
    /// 同盟を提案する好感度閾値
    pub alliance_score_threshold: f64,  // default: 40.0
    /// 研究に振り向ける工業品の割合
    pub research_investment_ratio: f64, // default: 0.3
}
```

#### 初期設定案

| 国家 | 好戦性 | 交易 | 技術 | 拡張 | 柔軟性 | イメージ |
|------|--------|------|------|------|--------|---------|
| 地球連邦 | 0.3 | 0.6 | 0.8 | 0.4 | 0.7 | 技術立国・外交重視 |
| ケンタウリ共和国 | 0.2 | 0.9 | 0.5 | 0.3 | 0.8 | 通商国家・平和主義 |
| シリウス帝国 | 0.8 | 0.3 | 0.4 | 0.7 | 0.3 | 軍事大国・攻撃的 |

### 2. 意思決定システム（NationalAISystem）

外交スコア変動のロジックを性格パラメータに基づいて個別化。

```
/// 毎 Tick の外交スコア変動アルゴリズム
fn calculate_diplomacy_drift(
    character: &NationalCharacter,
    own: &NationState,        // 自国の状態
    target: &NationState,     // 相手国の状態
    tick: u64,
) -> f64 {
    let mut drift = 0.0;

    // 1. 基本トレンド: 好戦性が高いほど関係が悪化しやすい
    drift -= character.aggression * 0.5;

    // 2. 軍事力格差: 自国が強いほど攻撃的な国家は関係を悪化させる
    let power_ratio = own.military_power / target.military_power.max(1.0);
    if power_ratio > character.war_power_threshold {
        drift -= character.aggression * 0.3;  // 攻撃チャンス
    } else {
        drift += (1.0 - character.aggression) * 0.2;  // 弱い時は友好的
    }

    // 3. 貿易関係: 貿易量が多いほど関係改善
    let trade_bonus = own.trade_volume_with_target * character.trade_affinity * 0.01;
    drift += trade_bonus;

    // 4. 技術格差: 技術志向が高い国は先進国と友好的になる
    let tech_gap = target.tech_level as f64 - own.tech_level as f64;
    if tech_gap > 0.0 {
        drift += character.research_focus * tech_gap * 0.1;  // 先進国には友好的
    }

    // 5. 周期的変動（ただし性格で振幅が変わる）
    let periodic = (tick as f64 * 0.07).sin() * character.diplomacy_flexibility * 0.5;
    drift += periodic;

    drift
}
```

### 3. 戦略的意思決定（StrategyDecisionSystem）

性格パラメータに基づいて、技術研究の優先順位や軍備方針を動的に決定。

```
/// 技術研究の優先分野を性格に基づいて決定
fn decide_research_priority(character: &NationalCharacter) -> TechField {
    // 重み付き選択
    let weights = [
        (TechField::Agriculture,     character.trade_affinity * 0.5),
        (TechField::Mining,          character.expansionism * 0.5),
        (TechField::EnergyTech,      0.3),  // 常に一定の需要
        (TechField::Manufacturing,   character.trade_affinity * 0.3 + character.research_focus * 0.3),
        (TechField::MilitaryTech,    character.aggression * 0.8),
        (TechField::SpaceNavigation, character.expansionism * 0.5 + character.trade_affinity * 0.3),
    ];
    // → 最低レベルかつ最大重みの分野を選択
}

/// 軍備方針: 好戦的な国は軍備に資源を多く振り向ける
fn decide_military_buildup(character: &NationalCharacter, resources: &Resources) -> u32 {
    let buildup_priority = character.aggression * 0.7 + character.expansionism * 0.3;
    let max_ships = if buildup_priority > 0.6 { 3 } else { 2 };  // 好戦国はより多く建造
    // ...
}
```

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `components/mod.rs` | `national_character` モジュール追加 |
| `components/national_character.rs` | 【NEW】`NationalCharacter` コンポーネント定義 |
| `systems/diplomacy.rs` | 固定ドリフトを `NationalCharacter` ベースの計算に置換 |
| `systems/research.rs` | 自動ローテーションを性格ベースの優先度に置換 |
| `systems/military.rs` | 建造上限を性格に基づいて動的に決定 |
| `plugins/world_init.rs` | 各国家に `NationalCharacter` を付与 |
| `components/simulation_event.rs` | AI意思決定イベントを追加 |

---

## 期待される効果

1. **再現性のあるドラマ**: 同じシードでも国家性格の初期値を変えると異なる展開
2. **emergent behavior**: 「ケンタウリが技術共有で地球と同盟 → シリウスを挟み撃ち」等が自然発生
3. **シナリオの多様性**: 性格パラメータをシナリオファイルから読むことで無限の組み合わせ
4. **プレイヤーの感情移入**: 「シリウス帝国がまた宣戦布告してきた…」等のストーリー感

---

## 実装の段階的アプローチ

| フェーズ | 内容 | 工数目安 |
|---------|------|---------|
| Phase 1 | `NationalCharacter` コンポーネント追加 + 外交ドリフトの個別化 | 小 |
| Phase 2 | 研究優先度の性格ベース決定 | 小 |
| Phase 3 | 軍備方針の動的決定 | 小 |
| Phase 4 | 貿易・技術格差のフィードバックループ | 中 |
