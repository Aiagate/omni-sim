# ゲームバランス定数・数式

Deep Juno のシミュレーションで使用される全ての定数と数式の定義です。これらの値の多くは `src/config.rs` の `BalanceConfig` で調整可能です。

---

## 1. 共通定数 (Common)

| パラメータ | 変数名 | デフォルト値 | 説明 |
|-----------|-------|:-----------:|------|
| **労働力ボーナス** | - | `log10(pop) / 6.0` | 範囲: 0.5 〜 2.0 |
| **技術レベルボーナス** | `tech_bonus_per_level` | `0.10` (+10%) | 該当分野の産出に乗算 |

---

## 2. 人口 (Population)

### 成長率計算
```
growth_rate = base_growth_rate(food_ratio)
growth_rate *= 1.0 - (radiation * 0.5)
growth_rate -= health_penalty * health_penalty_growth_coeff
if usage > 0.9: growth_rate *= (1.0 - usage) * 10.0
```

| パラメータ | 変数名 | デフォルト値 | 説明 |
|-----------|-------|:-----------:|------|
| **基本成長率** | - | `0.002` (0.2%) | 食料充足時のベース |
| **食料消費率** | `food_consumption_per_pop` | `0.00001` | 1人あたりの消費量 |
| **健康被害係数** | `health_penalty_growth_coeff` | `0.005` | 汚染 1.0 ごとの成長率低下 |

---

## 3. 経済と環境 (Economy & Environment)

### 産出・消費
```
food_prod = base * labour * tech * soil_fertility
mineral_prod = min(base * labour * tech * (1.0 - mining_depth), reserves)
energy_prod = base * labour * tech * (1.0 + fusion) * renewable_output
goods_prod = base * labour * tech * gravity_penalty * rare_earth_efficiency
```

| パラメータ | 変数名 | デフォルト値 | 説明 |
|-----------|-------|:-----------:|------|
| **エネルギー消費(ベース)** | `energy_consumption_per_pop_base` | `0.000005` | 1人あたりの消費量 |
| **気温消費ペナルティ** | `energy_temp_penalty_coeff` | `0.02` | 温度差によるエネルギー増 |
| **工業鉱物消費比** | `mineral_consumption_mf_ratio` | `0.5` | 工業品 1.0 に対して必要量 |
| **工業レアアース消費比**| `rare_earth_consumption_mf_ratio`| `0.1` | - |

### 汚染と浄化
| パラメータ | 変数名 | デフォルト値 | 説明 |
|-----------|-------|:-----------:|------|
| **工業大気汚染係数** | `air_pollution_mf_coeff` | `0.00005` | 工業品産出量当たり |
| **燃料大気汚染係数** | `air_pollution_fossil_coeff`| `0.00002` | 化石燃料消費当たり |
| **採掘水質汚染係数** | `water_pollution_mining_coeff`| `0.00002` | 鉱物採掘量当たり |
| **自然浄化率** | `base_cleanup_rate` | `0.0005` | 毎 Tick のベース |
| **技術浄化ボーナス** | `env_tech_cleanup_bonus_coeff`| `0.0005` | 環境技術 Lv1 ごとに加算 |

---

## 4. 貿易 (Trade)

| パラメータ | 変数名 | デフォルト値 | 説明 |
|-----------|-------|:-----------:|------|
| **距離コスト係数** | `trade_cost_per_ly` | `0.05` (5%) | 1光年あたりの資源消失 |
| **最大貿易コスト** | `max_trade_cost` | `0.5` (50%) | 消失上限 |
| **余剰判定閾値** | - | `1.5` 倍 | from が to のこの倍数以上 |

---

## 5. 軍事と戦争 (Military & War)

| パラメータ | 値 | 説明 |
|-----------|:---:|------|
| **艦船建造コスト** | 鉱物 50 + 工業品 30 | 1 隻あたり |
| **艦船維持費** | エネルギー 1.0 | 1 隻あたり / Tick |
| **戦力評価値** | 10.0 | 1 隻あたり |
| **開戦トリガー** | スコア $\le -50$ | 外交関係の下限 |
| **強制停戦期間** | 15 Tick | - |

---

## 6. 技術研究 (Research)

| パラメータ | 変数名 | デフォルト値 | 説明 |
|-----------|-------|:-----------:|------|
| **基礎研究コスト** | `research_cost_base` | `100.0` | Lv 0 -> 1 の必要量 |
| **コスト累進倍率** | `research_cost_multiplier` | `1.5` | 次レベルへのコスト倍率 |
| **技術ボーナス** | `tech_bonus_per_level` | `0.10` | 産出量への補正 (+10%) |

---

## 7. テラフォーミング (Terraforming)

| パラメータ | 変数名 | デフォルト値 |
|-----------|-------|:-----------:|
| **開始必要技術** | `terraforming_min_env_tech` | `5` |

### フェーズ別コストと進捗 (`(energy, goods, food, progress)`)
- **Atmospheric**: `(50.0, 0.0, 0.0, 0.02)`
- **Temperature**: `(30.0, 0.0, 0.0, 0.033)`
- **Water**: `(20.0, 10.0, 0.0, 0.025)`
- **Biological**: `(10.0, 5.0, 10.0, 0.016)`

---

## 8. ランダムイベント (Events)

- **基本発生率**: 5% / Tick / 惑星
- **強度範囲**: $0.5 〜 1.5$

| イベント | 基本効果 (強度 1.0) |
|----------|-------------------|
| **Earthquake** | 人口、工業品減少（地殻活動依存） |
| **RadiationStorm** | 人口、エネルギー減少（放射線依存） |
| **MeteoriteImpact**| 人口、食料減少 + 鉱物増加 |
| **EnvironmentalDisaster** | 人口激減（汚染 0.5 以上で発生） |
| **Plague / BabyBoom** | 人口 -1% / +0.5% |
| **TradeBoom / Harvest** | 全資源 +50 / 食料 +200 |
