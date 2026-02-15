# 提案 6: 文化・幸福度・内政システム

## 概要

現在のシミュレーションは経済と軍事の「ハードパワー」のみ。**文化・幸福度・内政安定度**という「ソフトパワー」の次元を追加し、国家の多面的な強さをモデル化する。

---

## 現状の課題

| 課題 | 現行の挙動 |
|------|-----------|
| 反乱が唐突 | ランダムイベントとしてのみ発生、原因がない |
| 国家の一枚岩性 | 内部力学（不満・派閥）の概念がない |
| 勝利条件の単調さ | 軍事力・資源量だけが「強さ」 |
| 文化的影響力なし | 隣接文明への文化浸透がない |

---

## 設計

### 1. 幸福度システム

```rust
/// 人口の幸福度と内政安定度（惑星に付与）
#[derive(Component, Debug, Clone)]
pub struct SocialState {
    /// 総合幸福度（0.0 ~ 100.0）
    pub happiness: f64,
    /// 内政安定度（0.0 ~ 100.0）
    pub stability: f64,
    /// 文化力（生産される文化ポイント/Tick）
    pub culture_output: f64,
    /// 蓄積された文化ポイント
    pub culture_points: f64,
    /// 政治体制
    pub government: GovernmentType,
    /// 自由度（0.0 ~ 1.0）
    pub freedom_index: f64,
}

/// 政治体制
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GovernmentType {
    /// 民主制: 幸福度→生産性ボーナス大、政策変更が遅い
    Democracy,
    /// 権威制: 安定で軍事力ボーナス、幸福度は低位安定
    Authoritarian,  
    /// 寡頭制: バランス型、交易ボーナス
    Oligarchy,
    /// 技術官僚制: 研究ボーナス大、幸福度への感度が低い
    Technocracy,
}
```

### 2. 幸福度の計算

```
毎 Tick:
  happiness_change = 0.0

  // === ポジティブ要因 ===
  // 食料充足
  if food_ratio >= 1.5:
    happiness_change += 0.3  (食料が十分で満足)
  elif food_ratio >= 1.0:
    happiness_change += 0.1

  // 経済繁栄（工業品が人口に対して十分）
  goods_per_capita = manufactured_goods / (population / 1_000_000)
  happiness_change += min(goods_per_capita × 0.1, 0.5)

  // 平和ボーナス
  if not at_war:
    happiness_change += 0.1
  
  // 技術進歩
  if tech_level_up_this_tick:
    happiness_change += 2.0  (一時的な喜び)

  // === ネガティブ要因 ===
  // 飢餓
  if food_ratio < 1.0:
    happiness_change -= (1.0 - food_ratio) × 5.0

  // 戦争
  if at_war:
    happiness_change -= 0.5
    happiness_change -= casualties_this_tick / population × 1000  (戦死者)

  // 過密 (人口が惑星キャパシティに近い)
  overcrowding = population / population_capacity
  if overcrowding > 0.8:
    happiness_change -= (overcrowding - 0.8) × 10.0

  // 汚染
  happiness_change -= pollution × 2.0

  // 自由度と体制の相互作用
  match government:
    Democracy     → happiness += freedom_index × 0.3
    Authoritarian → happiness -= freedom_index × 0.1 (高自由度は不安定)
    // ...

  happiness = clamp(happiness + happiness_change, 0.0, 100.0)
```

### 3. 幸福度の効果

| 幸福度 | 状態 | 効果 |
|--------|------|------|
| 80~100 | 繁栄 | 生産性 +20%, 人口成長 +10%, 反乱確率 0% |
| 60~80 | 満足 | 生産性 +10%, 反乱確率 0.1% |
| 40~60 | 普通 | ボーナス/ペナルティなし |
| 20~40 | 不満 | 生産性 -15%, 反乱確率 1%/Tick |
| 0~20 | 危機 | 生産性 -30%, 反乱確率 5%/Tick, 革命リスク |

### 4. 反乱・革命メカニクス

現在のランダムイベント「反乱」を、幸福度ベースの原因ある反乱に昇格。

```
反乱判定:
  rebellion_chance = base_chance × (1 - stability/100)
                   + unhappiness_factor
                   + freedom_suppression_factor

  // 自由度が高い社会で抑圧すると反乱リスク増加
  freedom_suppression = max(0, freedom_index - government.tolerance)
  
反乱の規模:
  mild:    人口の 1% が生産停止、幸福度 -5
  severe:  人口の 5% が反乱、軍事力の一部が離反
  revolution: 政体変更、新指導者（性格パラメータがリセット）

反乱の鎮圧:
  military_strength で鎮圧可能だが、さらに幸福度が低下
  → 鎮圧の悪循環 or 改革による根本解決の選択
```

### 5. 文化力システム

```
文化産出:
  culture_output = base_culture × population_factor × happiness_factor
  
  population_factor = log10(population) / 7.0  (人口が多いほど文化的)
  happiness_factor = happiness / 50.0           (幸福な社会は文化的)

文化力の効果:
  // 1. 外交ボーナス
  文化力の差 × 0.1 を外交スコアに加算（文化的に魅力的な国は好かれる）
  
  // 2. 文化浸透
  if 自国の文化力 > 隣接国の文化力 × 2.0:
    隣接国の幸福度が微減（自国文化への憧れ→不満）
    外交スコアが改善（文化的親近感）
  
  // 3. 文化勝利条件
  if culture_points > 10,000:
    「銀河の文化的覇者」としての勝利条件を達成
```

### 6. 政治体制と Government コンポーネント

| 体制 | 研究 | 軍事 | 貿易 | 幸福度 | 安定度 | 自由度の適正 |
|------|------|------|------|--------|--------|-------------|
| 民主制 | +10% | -10% | +20% | 高感度 | 変動大 | 0.7~1.0 |
| 権威制 | -10% | +20% | -10% | 低感度 | 安定 | 0.0~0.3 |
| 寡頭制 | ±0% | ±0% | +15% | 中感度 | やや安定 | 0.3~0.6 |
| 技術官僚制 | +30% | ±0% | ±0% | 低感度 | 安定 | 0.4~0.7 |

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `components/social.rs` | 【NEW】`SocialState`, `GovernmentType` 定義 |
| `systems/social.rs` | 【NEW】幸福度計算、文化力計算、反乱判定 |
| `systems/economy.rs` | 幸福度による生産性補正 |
| `systems/population.rs` | 幸福度による成長率補正 |
| `systems/events.rs` | ランダム反乱を幸福度ベースの反乱に置換 |
| `systems/diplomacy.rs` | 文化力による外交スコア補正 |
| `plugins/world_init.rs` | 各惑星に `SocialState` を付与 |

---

## 期待される効果

1. **多面的な国家運営**: 軍事力だけでなく文化・幸福度のバランスが重要に
2. **物語性**: 「繁栄する民主国 vs 安定した帝国」のドラマ
3. **原因ある反乱**: 飢餓→不幸→反乱の因果連鎖が理解できる
4. **勝利条件の多様化**: 軍事覇権・技術覇権・文化覇権の3ルート
