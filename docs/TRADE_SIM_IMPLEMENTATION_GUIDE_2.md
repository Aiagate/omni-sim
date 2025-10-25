# 🚀 拡張実装手順書：Bevy 0.17.2 版 貿易シミュレーション

## 🧭 Phase 5 以降の実装計画

本章では、基礎的な貿易シミュレーション（Phase 1〜4）に以下の要素を拡張する。

| フェーズ    | 名称                   | 主な目的                                     |
| :---------- | :--------------------- | :------------------------------------------- |
| **Phase 5** | 外交・リスク・市場拡張 | 各貿易結果に確率や関係値を導入               |
| **Phase 6** | イベントシステム実装   | 事故・外交変動・市場衝撃をイベント駆動で処理 |
| **Phase 7** | 国家AI導入             | 自律的な貿易・政策判断を行うAIロジック       |
| **Phase 8** | 並列・最適化           | マルチスレッド化・大量国家対応の最適化       |
| **Phase 9** | 可視化と統合           | Bevy UI / Bevy Egui によるUI統合             |

---

## ⚙️ Phase 5: 外交・リスク・市場拡張

### 5.1 外交関係コンポーネントの追加

```rust
#[derive(Component)]
pub struct Diplomacy {
    pub relation: f32, // -1.0（敵対）〜 +1.0（友好）
}
```

* 各 `Nation` に関連付け。
* 初期値は0.5程度（中立〜やや友好）。
* 貿易成功で上昇、失敗や事故で減少。

### 5.2 リスク導入（輸送失敗確率）

```rust
use rand::Rng;

fn apply_risk(fleet: &TransportFleet, route: &TradeRoute) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen::<f32>() >= route.risk // 成功確率 = 1 - risk
}
```

* 成功時：通常通り貿易成立
* 失敗時：貨物喪失、外交関係悪化、GDP変化なし

### 5.3 市場価格モデル

```rust
#[derive(Resource)]
pub struct Market {
    pub price_per_unit: f32, // 現在の市場価格
    pub volatility: f32,     // 価格変動率
}

fn update_market(mut market: ResMut<Market>, time: Res<Time>) {
    let delta = (time.elapsed_seconds().sin() * market.volatility).clamp(-0.05, 0.05);
    market.price_per_unit = (market.price_per_unit * (1.0 + delta)).clamp(0.5, 2.0);
}
```

* 貿易成功時のGDP増加を `cargo * market.price_per_unit * 0.05` に変更。
* 定期的に市場が変動することで経済的ダイナミズムを再現。

---

## ⚡ Phase 6: イベントシステム実装

### 6.1 イベント構造

```rust
#[derive(Event)]
pub enum TradeEvent {
    TradeSuccess { origin: Entity, dest: Entity, cargo: f32 },
    TradeFailure { origin: Entity, dest: Entity, reason: String },
}
```

### 6.2 イベント発行（貿易実行システムから）

```rust
fn transport_execution(
    mut events: EventWriter<TradeEvent>,
    ...
) {
    if apply_risk(&fleet, &route) {
        events.send(TradeEvent::TradeSuccess {
            origin: route.origin,
            dest: route.destination,
            cargo: fleet.cargo,
        });
    } else {
        events.send(TradeEvent::TradeFailure {
            origin: route.origin,
            dest: route.destination,
            reason: "Pirate Attack".into(),
        });
    }
}
```

### 6.3 イベント処理システム

```rust
fn handle_trade_events(
    mut reader: EventReader<TradeEvent>,
    mut nations: Query<(&mut Nation, &mut Diplomacy)>,
) {
    for event in reader.read() {
        match event {
            TradeEvent::TradeSuccess { origin, dest, cargo } => {
                if let Ok([mut a, mut b]) = nations.get_many_mut([*origin, *dest]) {
                    a.gdp += cargo * 0.05;
                    b.gdp += cargo * 0.05;
                    a.relation = (a.relation + 0.05).clamp(-1.0, 1.0);
                    b.relation = (b.relation + 0.05).clamp(-1.0, 1.0);
                }
            }
            TradeEvent::TradeFailure { origin, dest, .. } => {
                if let Ok([mut a, mut b]) = nations.get_many_mut([*origin, *dest]) {
                    a.relation = (a.relation - 0.1).clamp(-1.0, 1.0);
                    b.relation = (b.relation - 0.1).clamp(-1.0, 1.0);
                }
            }
        }
    }
}
```

---

## 🧠 Phase 7: 国家AIの導入

### 7.1 NationAI コンポーネント

```rust
#[derive(Component)]
pub struct NationAI {
    pub export_preference: f32, // 輸出志向
    pub risk_tolerance: f32,    // リスク許容度
}
```

### 7.2 自律行動システム

```rust
fn ai_trade_decision(
    mut commands: Commands,
    nations: Query<(Entity, &Nation, &NationAI)>,
    routes: Query<&TradeRoute>,
) {
    for (entity, nation, ai) in nations.iter() {
        if nation.resource_stock > 100.0 && ai.export_preference > 0.5 {
            // 他国を探索
            // 条件に合う貿易ルートがなければ生成
            // commands.spawn(TradeRoute { ... });
        }
    }
}
```

* 自律的に貿易を開始・停止。
* 将来的に強化学習や重み付き確率判断にも対応可能。

---

## 🧮 Phase 8: 並列実行・最適化

### 8.1 並列システム登録

```rust
.add_systems(Update, (
    ai_trade_decision,
    transport_execution,
    handle_trade_events,
    report,
).chain())
```

または `.add_systems(Update, (ai_trade_decision, transport_execution).in_schedule(Parallel))`

### 8.2 パフォーマンス改善

* ECSの並列実行を活かし、国家単位でジョブ分割。
* `Query<&mut Nation>` のアクセスを最小限に。
* `Res<Market>` を読み取り専用にすることでロックを回避。

---

## 🖥️ Phase 9: 可視化・UI統合

### 9.1 `bevy_egui` 導入

```bash
cargo add bevy_egui
```

### 9.2 シンプルな統計UI

```rust
use bevy_egui::{EguiContexts, egui};

fn ui_system(mut contexts: EguiContexts, nations: Query<&Nation>) {
    egui::Window::new("Trade Status").show(contexts.ctx_mut(), |ui| {
        for nation in nations.iter() {
            ui.label(format!("{} - GDP: {:.1} | Resources: {:.1}", nation.name, nation.gdp, nation.resource_stock));
        }
    });
}
```

### 9.3 統合

```rust
.add_plugins(DefaultPlugins)
.add_plugins(bevy_egui::EguiPlugin)
.add_systems(Update, ui_system)
```

---

## ✅ 拡張後の最終ディレクトリ構成

```
omni-sim/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── components/
│   │   ├── nation.rs
│   │   ├── diplomacy.rs
│   │   ├── market.rs
│   │   └── ai.rs
│   ├── systems/
│   │   ├── setup.rs
│   │   ├── trade.rs
│   │   ├── event.rs
│   │   ├── ai.rs
│   │   ├── market.rs
│   │   ├── report.rs
│   │   └── ui.rs
│   └── lib.rs
└── docs/
    └── TRADE_SIM_IMPLEMENTATION_GUIDE_v2.md
```

---

## 📈 今後の展開案（Phase 10+）

| フェーズ     | 拡張方向         | 概要                                |
| :----------- | :--------------- | :---------------------------------- |
| **Phase 10** | 銀河規模化       | 複数国家・惑星・交易ネットワーク化  |
| **Phase 11** | 経済モデル強化   | 資源価格・需要供給モデル            |
| **Phase 12** | 歴史イベント生成 | 長期的変動・紛争・技術進化          |
| **Phase 13** | 永続化・リプレイ | JSON/SQLiteへのスナップショット保存 |
| **Phase 14** | AI評価・分析     | 国家AIの成果を評価・ランキング化    |