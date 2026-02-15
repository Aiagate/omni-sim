# 提案 7: 諜報・スパイシステム

## 概要

全ての情報がオープンである現在のシミュレーションに**情報の不完全性と諜報活動**を導入する。「見えないもの」があることで意思決定の不確実性が増し、スパイ活動という新たな戦略次元が加わる。

---

## 現状の課題

| 課題 | 現行の挙動 |
|------|-----------|
| 完全情報ゲーム | 全国家の全データが可視 |
| 間接的手段の不在 | 戦争以外に相手を弱体化させる手段がない |
| 情報の価値がゼロ | 情報を得ることにコストも価値もない |

---

## 設計

### 1. 情報の可視性レベル

```rust
/// 他国に対する情報の可視性
#[derive(Component, Debug, Clone)]
pub struct IntelligenceState {
    /// 対象国家
    pub target_nation: Entity,
    /// 情報精度レベル (0 ~ 5)
    pub intel_level: u8,
    /// 配置中のスパイ数
    pub deployed_spies: u8,
    /// 防諜スコア（自国の防衛力）
    pub counter_intel: f64,
}
```

| Level | 情報の精度 | 見えるもの |
|-------|-----------|-----------|
| 0 | 不明 | 国家名のみ |
| 1 | 推定 | 人口規模（±30%）、軍事力（±50%） |
| 2 | 概算 | 人口（±15%）、軍事力（±25%）、資源の大まかな量 |
| 3 | 正確 | 全資源の正確な値、技術レベル |
| 4 | 詳細 | 研究中の技術、外交スコア、条約内容 |
| 5 | 完全 | AI性格パラメータ、意思決定ロジックの予測 |

### 2. スパイアクション

```rust
/// スパイが実行できるミッション
#[derive(Debug, Clone)]
pub enum SpyMission {
    /// 偵察: 情報レベルを上げる
    Reconnaissance,
    /// 技術窃取: 相手の研究ポイントの一部を獲得
    TechTheft { amount: f64 },
    /// サボタージュ: 相手の生産力を一時的に低下
    Sabotage { target: SabotageTarget, duration: u64 },
    /// 扇動: 相手国の幸福度を低下させる
    Incitement { unrest_amount: f64 },
    /// 外交工作: 第三国との関係を操作
    DiplomaticManipulation { third_party: Entity, score_change: f64 },
    /// 暗殺: 相手国の指導者を排除（政策混乱）
    Assassination,
}

/// サボタージュの対象
#[derive(Debug, Clone)]
pub enum SabotageTarget {
    Factory,         // 工業品産出 -30% for N Ticks
    Mine,            // 鉱物産出 -30% for N Ticks
    PowerGrid,       // エネルギー産出 -40% for N Ticks
    MilitaryBase,    // 艦船の一部を破壊
    ResearchLab,     // 研究進捗をリセット
}
```

### 3. スパイの運用コスト

```
スパイの維持:
  cost_per_spy = 5 工業品/Tick + 3 エネルギー/Tick
  
スパイの配置:
  新規配置: 50 工業品 + 20 エネルギー
  到着までの遅延: 距離 × 2 Tick

ミッションのコスト:
  Reconnaissance:          5 工業品
  TechTheft:              20 工業品 + 10 エネルギー
  Sabotage:               30 工業品 + 15 エネルギー
  Incitement:             15 工業品 + 10 エネルギー
  DiplomaticManipulation: 25 工業品
  Assassination:          50 工業品 + 30 エネルギー
```

### 4. 成功/失敗/発覚の判定

```
成功確率:
  base_success = 0.5 + spy_skill × 0.1
  modifier = own_intel_level × 0.05     // 情報が多いほど有利
           - target_counter_intel × 0.1  // 防諜が高いと失敗
           - mission_difficulty × 0.1    // 高難度ミッションほど失敗

  success_chance = clamp(base_success + modifier, 0.1, 0.95)

発覚判定（失敗時に別途判定）:
  detection_chance = 0.3 + target_counter_intel × 0.15
  
  if 発覚:
    → 外交スコアに大ペナルティ (-20 ~ -40)
    → スパイが捕獲（失われる）
    → 「○○国のスパイを拘束！」イベント発生
```

### 5. 防諜システム

```
counter_intel = base_counter_intel
              + military_tech_level × 0.1
              + internal_security_spending × 0.05

防諜の効果:
  - 敵スパイの成功率を低下
  - スパイの発覚率を上昇
  - 技術窃取の被害を軽減

コスト:
  防諜レベル 1 ポイントあたり 2 工業品/Tick
```

### 6. AIのスパイ運用原則

```
スパイ配置の優先度:
  1. 外交スコアが低い国 → 軍事情報の偵察
  2. 技術レベルが高い国 → 技術窃取
  3. 戦争中の国 → サボタージュ

国家性格による運用差:
  好戦国:   暗殺, サボタージュを好む
  交易国:   外交工作を好む
  技術国:   技術窃取, 偵察を好む
  全体:     情報レベルが低い相手への偵察を最優先
```

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `components/intelligence.rs` | 【NEW】`IntelligenceState`, `SpyMission` 定義 |
| `systems/espionage.rs` | 【NEW】スパイ配置・ミッション判定・防諜 |
| `systems/diplomacy.rs` | スパイ発覚による外交ペナルティ |
| `systems/economy.rs` | サボタージュ効果の適用 |
| `components/simulation_event.rs` | スパイ関連イベント追加 |

---

## 期待される効果

1. **不確実性**: 「相手の軍事力が不明なので攻めるべきか迷う」
2. **間接戦術**: 戦争せずにスパイで相手を弱体化させる選択肢
3. **スパイ劇**: 「技術をスパイされた！防諜を強化しなければ」
4. **リソースのトレードオフ**: スパイに投資するか軍事に投資するかの選択
