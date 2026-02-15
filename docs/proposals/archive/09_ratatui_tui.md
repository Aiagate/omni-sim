# 提案 9: Ratatui リッチ TUI

## 概要

現在の `cli_output.rs` は `println!` / `info!` による逐次テキスト出力。[Ratatui](https://ratatui.rs/) を導入してリアルタイム更新ダッシュボードを構築し、シミュレーション観察の体験を劇的に向上させる。

---

## 現状の課題

| 課題 | 現行の挙動 |
|------|-----------|
| 一方向出力 | ログが流れるだけでインタラクションなし |
| 情報の埋没 | 重要イベントがログの山に埋もれる |
| 全体把握の困難 | 3惑星の状態を同時に比較できない |
| 操作不可 | Tick速度の変更やフォーカス切替ができない |

---

## 設計

### 1. 画面レイアウト

```
┌─ Deep Juno ─────────────────── Tick: 47/100 ── [████████░░] 47% ── Speed: x10 ─┐
│                                                                                   │
│ ┌─── 銀河マップ ──────────────────┐ ┌─── 惑星詳細 ────────────────────────────┐ │
│ │                                  │ │ 📍 地球 (Sol 星系)                      │ │
│ │   Sol ●━━━━━━━━● Alpha C.       │ │                                          │ │
│ │     \   (4.37ly)        /        │ │ 人口: 11,615,734  (+0.30%/Tick)          │ │
│ │      \            (5.5ly)        │ │ 食料: ████████████░░  2558 (+120/Tick)   │ │
│ │  (8.6ly)                         │ │ 鉱物: ████░░░░░░░░░░   389 (+40/Tick)   │ │
│ │        ● Sirius                  │ │ ｴﾈﾙｷﾞｰ: ████████░░░░   936 (+80/Tick)  │ │
│ │                                  │ │ 工業品: █░░░░░░░░░░░░    17 (+30/Tick)  │ │
│ │  ○ 未発見星系 (探索範囲:8ly)      │ │                                          │ │
│ │                                  │ │ ⚔️ 艦船: 74  戦力: 740                   │ │
│ │  ● = 発見済み  ○ = 探索可能      │ │ 🔬 技術: Lv6  現在: 農業 (42/100pt)      │ │
│ └──────────────────────────────────┘ │ 💰 貿易: 3件 (入 +45/Tick, 出 -32/Tick) │ │
│                                       └──────────────────────────────────────────┘ │
│ ┌─── イベントフィード ────────────────────────────────────────────────────────────┐ │
│ │ [47] 🦠 プロキシマb — 疫病の流行 (強度 1.2x) — 人口 -24,864                    │ │
│ │ [45] ⚔️ シリウス→地球 — 戦闘: 艦船 3 撃破, 民間人 3700 犠牲                    │ │
│ │ [42] 💎 地球 — 鉱脈の発見 — 鉱物 +187                                          │ │
│ │ [41] 📈 ケンタウリ→地球 — 交易ブーム — 全資源 +62                              │ │
│ │ [38] 💡 シリウスIII — 技術ブレイクスルー — 研究pt +73                           │ │
│ └────────────────────────────────────────────────────────────────────────────────┘ │
│ ┌─── 外交 ─────────┐ ┌─── 統計グラフ ──────────────────────────────────────────┐ │
│ │ 地球─ケンタウリ    │ │ 人口推移:                                              │ │
│ │  🟢 +30.2 友好    │ │ 地球     ▁▂▃▃▄▅▅▆▆▇▇█████████  11.6M                  │ │
│ │ 地球─シリウス      │ │ ﾌﾟﾛｷｼﾏb  ▁▁▂▂▃▃▃▃▄▄▄▄▅▅▅▅▅▅▅   2.5M                  │ │
│ │  🔴 -65.3 戦争中  │ │ ｼﾘｳｽIII ▃▃▃▄▄▄▅▅▅▅▅▆▆▆▇▇▇▇▇▇   5.4M                  │ │
│ │ ケンタウリ─シリウス │ │                                                        │ │
│ │  🟡 -28.5 緊張    │ │ [Tab] 切替: 人口/資源/軍事/外交                         │ │
│ └──────────────────┘ └────────────────────────────────────────────────────────┘ │
│                                                                                   │
│ [Space] 一時停止  [←→] 速度調整  [1-3] 惑星切替  [q] 終了  [s] スクショ保存       │
└───────────────────────────────────────────────────────────────────────────────────┘
```

### 2. アーキテクチャ

```
Bevy ECS App
  │
  ├── Domain Systems (不変)
  │     economy → population → trade → ...
  │     └── EventWriter<SimulationEvent>
  │
  ├── TUI Bridge System (NEW)
  │     └── EventReader<SimulationEvent>
  │         → TuiState (Resource) に書き込み
  │
  └── TUI Render Thread (NEW, 別スレッド)
        └── TuiState を参照して Ratatui で描画
            → crossterm でターミナルを制御
            → キー入力を SimulationControl に反映
```

```rust
/// TUI と Bevy の間でデータを共有するリソース
#[derive(Resource)]
pub struct TuiState {
    /// 各惑星のスナップショット（毎Tick更新）
    pub planets: Vec<PlanetSnapshot>,
    /// 外交関係のスナップショット
    pub relations: Vec<RelationSnapshot>,
    /// イベントログ（最新N件）
    pub event_feed: VecDeque<EventEntry>,
    /// 現在のTick
    pub current_tick: u64,
    /// 選択中の惑星インデックス
    pub selected_planet: usize,
    /// 表示中のグラフタブ
    pub graph_tab: GraphTab,
    /// 時系列データ（グラフ用）
    pub history: SimulationHistory,
}

/// シミュレーション制御（キー入力から設定）
#[derive(Resource)]
pub struct SimulationControl {
    pub paused: bool,
    pub speed_multiplier: u32,  // 1, 2, 5, 10, 50
    pub should_quit: bool,
}
```

### 3. 必要なクレート

| クレート | バージョン | 用途 |
|---------|-----------|------|
| `ratatui` | 0.28+ | TUI フレームワーク |
| `crossterm` | 0.28+ | ターミナル制御バックエンド |

### 4. キーバインド

| キー | アクション |
|------|-----------|
| `Space` | 一時停止 / 再開 |
| `←` / `→` | Tick 速度を調整 (x1, x2, x5, x10, x50) |
| `1` ~ `9` | 惑星をフォーカス（詳細パネルに表示） |
| `Tab` | 統計グラフの切替 (人口/資源/軍事/外交/技術) |
| `d` | 外交詳細パネルの表示/非表示 |
| `e` | イベントフィードの拡大 |
| `s` | 現在の状態をスクリーンショットとしてファイル保存 |
| `q` | 終了 |

### 5. グラフの描画

Ratatui の `Chart` ウィジェットを使ったスパークライン/折れ線グラフ。

```rust
fn render_population_chart(f: &mut Frame, area: Rect, history: &SimulationHistory) {
    let datasets = history.planets.iter().map(|p| {
        Dataset::default()
            .name(&p.name)
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .data(&p.population_history)  // Vec<(f64, f64)> → (tick, population)
    }).collect();

    let chart = Chart::new(datasets)
        .block(Block::default().title("人口推移").borders(Borders::ALL))
        .x_axis(Axis::default().title("Tick").bounds([0.0, max_tick]))
        .y_axis(Axis::default().title("人口").bounds([0.0, max_pop]));

    f.render_widget(chart, area);
}
```

### 6. CLI モードとの共存

既存の `CliOutputPlugin` との切り替えを容易にする。

```rust
// main.rs
if args.contains("--tui") {
    app.add_plugins(TuiPlugin);
} else {
    app.add_plugins(CliOutputPlugin);
}
```

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `plugins/tui_output.rs` | 【NEW】TUI描画・キー入力処理 |
| `plugins/mod.rs` | `tui_output` モジュール追加 |
| `main.rs` | `--tui` フラグによる切り替え |
| `Cargo.toml` | `ratatui`, `crossterm` 依存追加 |
| 既存システム | **変更なし**（SimulationEvent 経由のため） |

> 重要: `SimulationEvent` アーキテクチャのおかげで、ドメインシステムは一切変更不要。出力プラグインの差し替えだけで実現可能。

---

## 期待される効果

1. **没入感**: リアルタイムで銀河が動くダッシュボードを眺める体験
2. **即時理解**: 3惑星の状態を一目で比較、グラフで傾向を把握
3. **インタラクション**: 速度調整、惑星切替で能動的な観察
4. **スクリーンショット**: 面白い瞬間を保存・共有
