# 提案 15: 異星種族と宇宙生態系

## 概要

人類文明だけでなく、**宇宙そのものが生きている環境**として**異星種族・宇宙現象・未知のエンカウンター**を導入。シミュレーションに驚きと未知の要素を追加する。

---

## 設計

### 1. 異星種族

探索中に発見される非人類文明。友好的なものから敵対的なものまで。

```rust
/// 異星種族の分類
#[derive(Debug, Clone)]
pub enum AlienDisposition {
    /// 友好的: 技術共有・貿易が可能
    Friendly,
    /// 中立: 接触に慎重、条件付きで交流
    Neutral,
    /// 敵対的: 領域に近づくと攻撃
    Hostile,
    /// 超越的: 人類の理解を超えた存在
    Transcendent,
}

#[derive(Component, Debug, Clone)]
pub struct AlienCivilization {
    pub name: String,
    pub disposition: AlienDisposition,
    pub tech_level: u32,         // 人類の Tech level との比較
    pub military_power: f64,
    pub territory_radius: f64,   // 光年
    pub home_position: Position,
    pub traits: Vec<AlienTrait>,
}

#[derive(Debug, Clone)]
pub enum AlienTrait {
    /// テレパシー: 外交が容易、スパイが効かない
    Telepathic,
    /// 群体意識: 個体の概念がなく、反乱が起きない
    HiveMind,
    /// シリコン生物: 鉱物を「食料」として消費
    SiliconBased,
    /// エネルギー生命体: 物理的な船を持たない
    EnergyBeing,
    /// 古代種族: 超高技術だが衰退中
    AncientRace,
}
```

### 2. 遭遇イベント

```
探索 System で異星種族を発見した場合:

1. ファーストコンタクト
   → 異星種族の disposition による初期反応
   → Friendly: 即座に通商可能
   → Neutral: 5~10 Tick の観察期間
   → Hostile: 即座に攻撃してくる
   → Transcendent: 謎のメッセージを残して去る

2. 外交オプション（Friendly/Neutral の場合）
   → 技術交換: 相手の得意分野と自国の得意分野を交換
   → 資源貿易: 特殊な異星資源の入手
   → 軍事同盟: 他の人類文明に対する抑止力
   → 文化交流: 幸福度・文化力にボーナス

3. 戦闘（Hostile の場合）
   → 異星種族の軍事力は通常の国家より強い
   → 勝利すると、異星技術の残骸からテクノロジーボーナス
   → 敗北すると、惑星に甚大な被害
```

### 3. 宇宙現象イベント

異星種族以外の「宇宙が生きている」ことを表現するイベント。

```rust
pub enum CosmicEvent {
    /// 超新星爆発: 近傍星系に放射線ダメージ
    Supernova {
        position: Position,
        blast_radius: f64,    // 光年
        radiation_increase: f64,
    },
    /// ガンマ線バースト: 広範囲だが方向性あり
    GammaRayBurst {
        direction: (f64, f64, f64),
        cone_angle: f64,
        damage: f64,
    },
    /// ワームホールの出現: 2地点間の近道
    WormholeAppearance {
        entrance: Position,
        exit: Position,
        stability: f64,      // 0.0~1.0, 低い = すぐ消える
        duration: u64,        // 存続 Tick 数
    },
    /// 宇宙嵐: 通信・貿易の一時断絶
    CosmicStorm {
        center: Position,
        radius: f64,
        duration: u64,
    },
    /// 不思議な信号: 発信元を調査すると何かが見つかる
    MysteriousSignal {
        origin: Position,
        content: SignalContent,
    },
    /// 小惑星群の到来: 採掘チャンスだが衝突リスクも
    AsteroidSwarm {
        path: Vec<Position>,
        mineral_value: f64,
        collision_risk: f64,
    },
}

pub enum SignalContent {
    AncientMap,          // 隠された星系の位置が判明
    TechBlueprint,       // 高等技術の研究コスト -50%
    Warning,             // 何かの脅威の接近を示唆
    Noise,               // ただのノイズ（はずれ）
}
```

### 4. 先行者文明の遺跡

探索で発見される古代文明の遺跡。大きなリスクと報酬。

```
遺跡の種類:
  - 技術アーカイブ: ランダムな技術をLv+2
  - 資源キャッシュ: 大量の希少資源
  - 兵器庫: 強力な古代兵器（一時的な軍事力ブースト）
  - 禁断の知識: 研究スピードが永続 +20%、ただし謎の代償あり
  - ガーディアン: 遺跡を守る自動防衛システム（倒すと報酬）

遺跡の危険度:
  safe:      リスクなし、報酬は小さい
  moderate:  探索チームの損失リスク（人口 -0.1%）
  dangerous: 軍事力が必要（艦船を消費）
  lethal:    失敗すると壊滅的被害（人口 -5%, 資源 -30%）
```

### 5. 銀河の「生態系」

```
銀河の長期的な変化:

1. 恒星の寿命
   - 一部の星系は数百 Tick 後に恒星が膨張し居住不能に
   → 移住の必要性

2. 銀河中心ブラックホール
   - 活動期に入ると放射線が銀河全体に影響
   → 数十 Tick の周期で活動期/安定期を繰り返す

3. 暗黒物質の潮流
   - 超光速航行の効率に影響する「宇宙の天気」
   → ある方向は速く、別の方向は遅い
```

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `components/alien.rs` | 【NEW】`AlienCivilization`, `AlienTrait` |
| `components/cosmic.rs` | 【NEW】`CosmicEvent` |
| `systems/encounter.rs` | 【NEW】異星種族との遭遇処理 |
| `systems/cosmic_events.rs` | 【NEW】宇宙現象イベント |
| `systems/exploration.rs` | 遺跡・異星種族の発見を追加 |
| `systems/events.rs` | 宇宙現象をイベントシステムに統合 |
| `components/simulation_event.rs` | 宇宙関連イベント追加 |

---

## 期待される効果

1. **驚きと未知**: 「何が出るかわからない」探索のスリル
2. **外部圧力**: 異星種族という共通の脅威が人類間の外交を動かす
3. **物語の深み**: 古代遺跡や謎のシグナルがSF的ロマンを演出
4. **ダイナミックな宇宙**: 超新星やワームホールが戦略を根本的に変える
5. **スケール感**: 人類が「銀河の住人の一部」であるという壮大さ
