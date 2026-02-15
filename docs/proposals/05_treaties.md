# 提案 5: 条約・同盟システム

## 概要

現在の外交は数値スコアの変動と「スコア -50 以下で自動戦争」のみ。**明示的な条約・協定・同盟**を導入し、国家間の関係に構造を持たせることで、複雑な政治ゲームを実現する。

---

## 現状の課題

| 課題 | 現行の挙動 |
|------|-----------|
| 外交アクションが存在しない | 国家は一切の外交行動を取れない |
| 関係の二値性 | 戦争か非戦争かの二択のみ |
| 同盟・通商の概念なし | 複数国間の協力関係がモデル化されていない |
| 戦争の巻き込みなし | 第三国は常に傍観者 |

---

## 設計

### 1. 条約の種類

```rust
/// 条約の種類
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TreatyType {
    /// 不可侵条約: 戦争トリガー閾値を下げる
    NonAggression,
    /// 通商条約: 貿易ボーナス
    TradeAgreement,
    /// 技術共有条約: 研究ポイント共有
    TechSharing,
    /// 防衛同盟: 一方が攻撃されたら他方も参戦
    DefensiveAlliance,
    /// 攻守同盟: 攻撃・防衛の両方で参戦
    MilitaryAlliance,
    /// 属国条約: 資源上納の代わりに保護
    Vassalage,
    /// 停戦条約: 一定期間の戦争禁止
    Ceasefire { expires_at: u64 },
}
```

### 2. 条約コンポーネント

```rust
/// 二国間条約（エンティティとしてスポーン）
#[derive(Component, Debug, Clone)]
pub struct Treaty {
    pub treaty_type: TreatyType,
    pub party_a: Entity,       // 提案国
    pub party_b: Entity,       // 受諾国
    pub signed_at: u64,        // 締結 Tick
    pub duration: Option<u64>, // Some(n) → n Tick 後に期限切れ
    pub terms: TreatyTerms,    // 条約の具体的条件
}

/// 条約の具体的条件
#[derive(Debug, Clone)]
pub struct TreatyTerms {
    /// 貿易ボーナス倍率（通商条約用）
    pub trade_bonus: f64,               // default: 1.0 (ボーナスなし)
    /// 研究ポイント共有率（技術共有用）
    pub tech_sharing_rate: f64,          // default: 0.0
    /// 戦争トリガー閾値の変更（不可侵用）
    pub war_threshold_modifier: f64,     // default: 0.0 (-30 なら -80 になる)
    /// 資源上納率（属国用）
    pub tribute_rate: f64,               // default: 0.0
}
```

### 3. 条約の提案・受諾ロジック

```
条約提案の判断基準（AI）:

不可侵条約:
  条件: 外交スコア > -20 AND 自国が弱い（軍事力で劣位）
  効果: war_threshold を -50 → -80 に変更（攻められにくくなる）

通商条約:
  条件: 外交スコア > 0 AND trade_affinity > 0.5
  効果: 相互の貿易ルートで容量 +50%, コスト -20%

技術共有条約:
  条件: 外交スコア > 30 AND research_focus > 0.6
  効果: 相手の研究ポイントの 10% を毎Tick獲得

防衛同盟:
  条件: 外交スコア > 50 AND 共通の敵が存在
  効果: 一方が宣戦された場合、他方も自動参戦

受諾の判断基準:
  相手の提案を受けるかどうかは以下で判定:
  acceptance_score = 外交スコア × 0.3
                   + 条約の自国メリット × 0.5
                   + diplomacy_flexibility × 20
                   - 相手の好戦性の認識 × 15
  
  if acceptance_score > 30: 受諾
```

### 4. 条約の破棄と裏切り

```
条約破棄の条件:
  - 期限切れ
  - 外交スコアが一定値以下に低下
  - 戦争が勃発（不可侵条約は直接破棄される）

裏切りのペナルティ:
  - 条約破棄国の全外交スコアに -20 のペナルティ（信頼性の低下）
  - 同盟破棄 → -40 のペナルティ
  - 他国が「裏切り者」として認識 → 条約提案が受けられにくくなる

裏切りの確率:
  betrayal_chance = (1.0 - diplomacy_flexibility) × 0.01
                  + aggression × 0.02 (好戦的な国ほど裏切る)
                  - 外交スコア × 0.001
```

### 5. 多国間戦争のメカニクス

```
同盟による連鎖参戦:
  A が B に宣戦 →
    if B と C に DefensiveAlliance が存在:
      C が自動的に A に宣戦（B 側で参戦）

    if A と D に MilitaryAlliance が存在:
      D が自動的に B に宣戦（A 側で参戦）

  結果: A+D vs B+C の多国間戦争

戦闘解決:
  同盟軍の合計戦力で計算
  ただし異なる星系からの援軍は距離に応じた遅延あり
```

### 6. SimulationEvent の追加

```rust
// 条約関連イベント
SimulationEvent::TreatyProposed {
    tick: u64,
    from: String,
    to: String,
    treaty_type: TreatyType,
}
SimulationEvent::TreatySigned {
    tick: u64,
    parties: (String, String),
    treaty_type: TreatyType,
}
SimulationEvent::TreatyBroken {
    tick: u64,
    breaker: String,
    treaty_type: TreatyType,
    penalty: f64,
}
SimulationEvent::AllianceWar {
    tick: u64,
    defender: String,
    ally_joining: String,
    against: String,
}
```

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `components/diplomacy.rs` | `Treaty`, `TreatyTerms` 追加 |
| `systems/diplomacy.rs` | 条約提案・受諾ロジック追加 |
| `systems/war.rs` | 同盟連鎖参戦ロジック追加、戦争トリガー閾値の条約修正 |
| `systems/trade.rs` | 通商条約ボーナスの適用 |
| `systems/research.rs` | 技術共有条約によるポイント共有 |
| `plugins/world_init.rs` | 初期条約の設定（任意） |
| `plugins/cli_output.rs` | 条約一覧のサマリー表示 |

---

## 期待される効果

1. **政治ドラマ**: 「ケンタウリがシリウスと不可侵条約→地球と同盟→シリウスに宣戦」
2. **戦略的深み**: 条約による間接的なパワーバランス操作
3. **裏切りのスリル**: 同盟破棄のタイミングが重要な戦略要素に
4. **多国間ダイナミクス**: 2国間だけでない複雑な関係構造

---

## 実装の段階的アプローチ

| フェーズ | 内容 | 工数目安 |
|---------|------|---------|
| Phase 1 | `Treaty` コンポーネントと不可侵条約 | 小 |
| Phase 2 | 通商条約と貿易ボーナス | 小 |
| Phase 3 | 防衛同盟と連鎖参戦 | 中 |
| Phase 4 | 条約AI（提案・受諾・裏切り判定） | 中 |
| Phase 5 | 属国条約と技術共有 | 中 |
