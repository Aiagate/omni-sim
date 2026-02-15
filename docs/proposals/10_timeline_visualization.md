# 提案 10: タイムライン可視化と事後分析

## 概要

シミュレーション全体の履歴データを記録し、終了後に**タイムライン分析・統計サマリー・グラフ出力**を提供する。「何が起きたか」を視覚的に理解するためのリプレイ・分析機能。

---

## 設計

### 1. 時系列データの記録

```rust
/// 全Tickのスナップショットを記録するリソース
#[derive(Resource, Debug)]
pub struct SimulationHistory {
    /// Tick ごとの惑星スナップショット
    pub planet_snapshots: Vec<TickSnapshot>,
    /// Tick ごとの外交スナップショット
    pub relation_snapshots: Vec<DiplomacySnapshot>,
    /// 全イベントのタイムスタンプ付きログ
    pub events: Vec<TimestampedEvent>,
}

#[derive(Debug, Clone)]
pub struct TickSnapshot {
    pub tick: u64,
    pub planets: Vec<PlanetState>,
}

#[derive(Debug, Clone)]
pub struct PlanetState {
    pub name: String,
    pub population: f64,
    pub growth_rate: f64,
    pub food: f64,
    pub minerals: f64,
    pub energy: f64,
    pub goods: f64,
    pub ships: u32,
    pub power: f64,
    pub tech_level: u32,
}
```

### 2. 終了時のサマリー強化

現行のテーブルに加え、以下を出力。

#### ターニングポイント分析

```
=== ターニングポイント ===

Tick 18: 📉 シリウス→ケンタウリ 外交スコア -50 突破 → 戦争勃発
  影響: ケンタウリの人口成長が -3.2% に転落

Tick 24: 💡 地球 技術ブレイクスルー → 工業Lv3到達
  影響: 工業品産出が +40% に急増、軍備増強が加速

Tick 33: ⚔️ シリウス帝国 降伏（艦船全滅）
  影響: ケンタウリが回復基調に
```

#### テキストベースグラフ

```
人口推移 (50 Tick):
 12M ┤
     │                                         ╭──── 地球
 10M ┤                              ╭──────────╯
     │                     ╭────────╯
  8M ┤              ╭──────╯
     │        ╭─────╯
  6M ┤  ╭─────╯                                ╭──── シリウスIII
     │  │            ╭─────────────────────────╯
  4M ┤──╯     ╭──────╯
     │        │                        ╭──────── プロキシマb
  2M ┤────────╯   ╭───╮ ╭─────────────╯
     │            ╰───╯ (戦争による人口減)
  0M ┼────┬────┬────┬────┬────┬────┬────┬────┬────┬────
     0    5   10   15   20   25   30   35   40   45   50
```

### 3. ファイル出力

#### CSV 出力

```bash
cargo run -- --output csv --output-dir ./results/

# 生成ファイル:
# results/population.csv
# results/resources.csv
# results/military.csv
# results/diplomacy.csv
# results/events.csv
```

```csv
# population.csv
tick,planet,population,growth_rate
0,地球,10000000,0.002
1,地球,10020000,0.002
...
```

#### JSON 出力

```bash
cargo run -- --output json --output-file ./results/simulation.json
```

```json
{
  "config": { "max_ticks": 50, "seed": 42 },
  "history": [
    {
      "tick": 0,
      "planets": [
        { "name": "地球", "population": 10000000, ... }
      ],
      "events": []
    }
  ],
  "summary": {
    "winner": "地球連邦",
    "turning_points": [...]
  }
}
```

### 4. ターニングポイントの自動検出

```
fn detect_turning_points(history: &SimulationHistory) -> Vec<TurningPoint> {
    let mut points = Vec::new();

    for i in 1..history.len() {
        let prev = &history[i-1];
        let curr = &history[i];

        // 戦争勃発/終結
        // 人口の急激な変動（±5% 以上）
        // 技術レベルアップ
        // 外交スコアの閾値突破
        // 資源の枯渇
    }

    // 影響度でソートして上位N件を返す
    points.sort_by(|a, b| b.impact.partial_cmp(&a.impact).unwrap());
    points.truncate(10);
    points
}
```

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `systems/history.rs` | 【NEW】毎Tickのスナップショット記録 |
| `plugins/cli_output.rs` | サマリーにグラフ・ターニングポイント追加 |
| `main.rs` | `--output` フラグの追加 |
| 既存システム | **変更なし** |

---

## 期待される効果

1. **事後分析**: 「あの戦争がなければどうなっていたか」の考察に
2. **外部ツール連携**: CSV/JSON → Python/gnuplot で高品質グラフ
3. **バランス調整**: 数値の時系列推移から不均衡を発見
4. **共有**: 面白い結果をファイルとして保存・共有
