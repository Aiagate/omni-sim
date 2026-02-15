# 提案 4: 資源枯渇と持続可能性

## 概要

現在のリソースは毎Tickの産出による無限蓄積モデル。**有限の埋蔵量と枯渇メカニクス**を導入することで、短期的な搾取 vs 長期的な持続可能性のジレンマを生み、技術進歩の必然性を演出する。

---

## 現状の課題

| 課題 | 現行の挙動 |
|------|-----------|
| 資源が無限 | `Production.mineral_rate` が毎Tick一定量を無条件に産出 |
| 時間軸の意味薄 | 1000 Tick でも 100 Tick でも基本的に同じ構造 |
| 技術進歩の動機薄 | 深層採掘や核融合の「必要性」がない |
| 環境破壊の概念なし | 採掘・工業化の副作用がゼロ |

---

## 設計

### 1. 資源タイプの再分類

```rust
/// 枯渇性資源（惑星ごとに固有の埋蔵量）
#[derive(Component, Debug, Clone)]
pub struct DepletableResources {
    /// 鉱物の総埋蔵量
    pub mineral_reserves: f64,
    /// 化石燃料の総埋蔵量（エネルギー源）
    pub fossil_fuel_reserves: f64,
    /// レアアース埋蔵量（高等技術に必要）
    pub rare_earth_reserves: f64,

    /// 採掘深度（0.0 ~ 1.0、進むほどコスト増）
    pub mining_depth: f64,
}

/// 再生可能資源（枯渇しないが産出量に上限あり）
#[derive(Component, Debug, Clone)]
pub struct RenewableResources {
    /// 土壌肥沃度（0.0 ~ 1.5、過耕作で低下、休耕で回復）
    pub soil_fertility: f64,
    /// 再エネ発電力（太陽光・風力・地熱、惑星環境に依存）
    pub renewable_energy_output: f64,
    /// 森林被覆率（0.0 ~ 1.0、食料と環境の両方に影響）
    pub forest_coverage: f64,
}
```

### 2. 枯渇メカニクス

#### 鉱物の枯渇

```
毎 Tick:
  // 実際の採掘量は埋蔵量と採掘深度に依存
  depth_penalty = 1.0 - mining_depth × 0.5   // 深いほど効率低下
  tech_bonus = deep_mining_level × 0.15       // 深層採掘技術による緩和

  actual_extraction = base_mineral_rate × depth_penalty × (1 + tech_bonus)
  actual_extraction = min(actual_extraction, mineral_reserves)

  mineral_reserves -= actual_extraction
  resources.minerals += actual_extraction

  // 採掘が進むほど深度が増加
  mining_depth += actual_extraction / initial_reserves × 0.01
  mining_depth = min(mining_depth, 1.0)

枯渇時:
  mineral_reserves == 0 の場合、鉱物産出はゼロになる
  → 貿易 or 小惑星採掘技術が必要
```

#### 化石燃料の枯渇

```
毎 Tick:
  if energy_source == FossilFuel:
    fuel_consumed = energy_production × 0.1
    fossil_fuel_reserves -= fuel_consumed

  if fossil_fuel_reserves <= 0:
    energy_production の化石燃料分がゼロに
    → 核融合 or 再エネへの移行が必須

// エネルギー源の構成比
energy_output = renewable_energy × renewable_ratio
              + fossil_fuel_energy × fossil_ratio
              + fusion_energy × fusion_ratio  // 技術アンロック後
```

#### 土壌の劣化と回復

```
毎 Tick:
  // 農業による土壌消耗
  exploitation = food_production / (soil_fertility × 1000)
  soil_fertility -= exploitation × 0.001

  // 森林被覆による自然回復
  natural_recovery = forest_coverage × 0.0005
  soil_fertility = min(soil_fertility + natural_recovery, 1.5)

  // 実効食料産出
  effective_food = base_food_rate × soil_fertility × labor_bonus × tech_bonus

影響:
  土壌肥沃度が 0.3 以下 → 「砂漠化」警告
  土壌肥沃度が回復しない場合 → 長期的な食料危機
```

### 3. 環境汚染と産業の副作用

```rust
#[derive(Component, Debug, Clone)]
pub struct EnvironmentalHealth {
    /// 大気汚染レベル（0.0 ~ 1.0）
    pub air_pollution: f64,
    /// 水質汚染レベル（0.0 ~ 1.0）
    pub water_pollution: f64,
    /// 汚染による人口成長ペナルティ
    pub health_penalty: f64,
}
```

```
毎 Tick:
  // 工業生産による汚染増加
  air_pollution += manufacturing_output × 0.0001
  water_pollution += mining_output × 0.00005

  // 技術による汚染低減（クリーンテクノロジー）
  pollution_reduction = cleantech_level × 0.05
  air_pollution = max(0, air_pollution - pollution_reduction)

  // 汚染による影響
  health_penalty = (air_pollution + water_pollution) × 0.5
  population_growth_rate -= health_penalty × 0.002
  food_production_modifier -= water_pollution × 0.1
```

### 4. 資源依存段階モデル（文明の進化フェーズ）

歴史的な文明の進化と対応する資源利用の変遷をモデル化。

```
Phase 1: 農業文明（Tick 0~）
  主資源: 土壌（再生可能）
  主エネルギー: 化石燃料
  リスク: 土壌枯渇、化石燃料の有限性

Phase 2: 産業文明（鉱物枯渇の兆候が出始める）
  主資源: 鉱物 + 工業品
  主エネルギー: 化石燃料 → 再エネ移行
  リスク: 環境汚染、資源争奪戦争

Phase 3: 知識文明（核融合 or 再エネへの完全移行後）
  主資源: エネルギー（事実上無限）→ 工業品自動生産
  リスク: 技術格差による覇権争い

Phase 4: 宇宙文明（小惑星採掘、ダイソンスフィア後）
  主資源: 宇宙由来資源（事実上無限）
  リスク: 拡張主義 vs 孤立主義の対立
```

### 5. 新イベントの追加

| イベント | トリガー | 効果 |
|---------|---------|------|
| 資源枯渇危機 | mineral_reserves < 10% | 国家の外交姿勢が攻撃的に（資源確保のため） |
| 環境災害 | air_pollution > 0.7 | 人口に大ダメージ、食料産出 -50% |
| 資源発見 | 探査イベント（確率） | 埋蔵量が部分的に回復 |
| 技術革新 | 研究ブレイクスルー | 採掘効率の一時的な大幅改善 |
| エネルギー危機（深刻版） | 化石燃料 < 5% AND 核融合なし | 全生産 -30%, 戦争リスク増大 |

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `components/economy.rs` | `DepletableResources`, `RenewableResources`, `EnvironmentalHealth` 追加 |
| `systems/economy.rs` | 産出計算に枯渇・肥沃度・汚染の影響を統合 |
| `systems/population.rs` | 汚染ペナルティ、人口キャップの導入 |
| `systems/events.rs` | 資源枯渇関連イベントの追加 |
| `systems/diplomacy.rs` | 資源不足による外交スコア変動 |
| `plugins/world_init.rs` | 各惑星に初期埋蔵量を設定 |
| `plugins/cli_output.rs` | 埋蔵量・汚染レベルのサマリー表示 |

---

## バランス設計のポイント

| パラメータ | 設計意図 |
|-----------|---------|
| 鉱物埋蔵量 | 50~200 Tick で枯渇兆候、技術なしでは 300 Tick で完全枯渇 |
| 化石燃料 | 100~150 Tick で半減、核融合研究への動機 |
| 土壌 | 過耕作なら 80 Tick で劣化、持続可能農業なら永続 |
| 汚染 | 技術なしでは 150 Tick で深刻化、クリーンテクで抑制可能 |

> 核心: 「今は豊かだが、何もしなければ衰退する」というプレッシャーが技術研究と外交にドラマを生む

---

## 実装の段階的アプローチ

| フェーズ | 内容 | 工数目安 |
|---------|------|---------|
| Phase 1 | 鉱物埋蔵量と枯渇メカニクス | 小 |
| Phase 2 | 化石燃料 → 再エネ移行 | 小 |
| Phase 3 | 土壌肥沃度と農業持続性 | 小 |
| Phase 4 | 環境汚染システム | 中 |
| Phase 5 | 枯渇関連イベントと外交フィードバック | 中 |
