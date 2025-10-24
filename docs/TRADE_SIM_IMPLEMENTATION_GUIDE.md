# 🧭 Bevy 0.17.2 貿易シミュレーション実装手順書

## 📌 概要

本ドキュメントは、**Bevy 0.17.2** の ECS（Entity Component System）を利用して
「2国家間の貿易シミュレーション」を実装するための段階的な手順を示す。

対象読者は、Bevy と Rust の基本的な構文を理解している開発者を想定。

---

## 🏗️ フェーズ構成

| フェーズ        | 目的        | 内容                          |
| :---------- | :-------- | :-------------------------- |
| **Phase 1** | プロジェクト初期化 | Bevy 0.17.2 環境構築と基本エンティティ定義 |
| **Phase 2** | モデル実装     | 国家・貿易ルート・輸送艦隊の ECS 化        |
| **Phase 3** | システム実装    | 貿易ロジック（決定・実行・精算）            |
| **Phase 4** | 可視化と検証    | ログ出力・結果可視化・デバッグ             |
| **Phase 5** | 拡張        | イベント・外交関係・価格変動などの拡張設計       |

---

## ⚙️ Phase 1: プロジェクト初期化

### 1.1 Bevy プロジェクト作成

```bash
cargo init --name omni-sim
# または新規ディレクトリに作成する場合:
# cargo new omni-sim --bin
# cd omni-sim
```

### 1.2 依存関係の追加

**重要:** Cargo.tomlを手動で編集せず、必ず`cargo add`コマンドを使用してください。

```bash
cargo add bevy --features dynamic_linking
```

> 💡 `dynamic_linking` はホットリロードや開発速度向上に便利（リリース時は外しても可）

### 1.3 WSL2環境での注意事項

WSL2環境では、デフォルトのBevyグラフィックス機能に必要なWaylandライブラリが不足している場合があります。
その場合は、以下のように最小限の機能セットとScheduleRunnerを使用してください：

```bash
# 最小限の依存関係に変更
cargo add bevy --no-default-features --features bevy_core_pipeline,bevy_state
```

main.rsでScheduleRunnerPluginを使用：

```rust
use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                Duration::from_secs_f64(1.0 / 60.0),
            ))
        )
        // ... システム登録
        .run();
}
```

---

## 🧩 Phase 2: モデル実装（ECS構造）

### 2.1 コンポーネント定義

```rust
use bevy::prelude::*;

/// 国家エンティティ
///
/// シミュレーション内の経済主体を表現します。
#[derive(Component)]
pub struct Nation {
    /// 国家名（表示用）
    pub name: String,

    /// GDP（Gross Domestic Product）
    /// 経済規模を示す指標。貿易によって増加します。
    pub gdp: f32,

    /// 資源ストック量
    /// 貿易で輸出可能な資源の総量。負の値にならないよう管理が必要。
    pub resource_stock: f32,
}

/// 貿易ルート
///
/// 2つの国家間の恒常的な貿易経路を定義します。
#[derive(Component)]
pub struct TradeRoute {
    /// 輸出元国家のエンティティ
    pub origin: Entity,

    /// 輸入先国家のエンティティ
    pub destination: Entity,

    /// 1回の輸送で運搬する資源量
    /// 注: 現在の実装では TransportFleet.cargo を使用
    pub volume: f32,
}

/// 輸送艦隊
///
/// 貿易ルートに沿って資源を運搬する艦隊を表現します。
#[derive(Component)]
pub struct TransportFleet {
    /// 現在積載している貨物量
    pub cargo: f32,

    /// 到着までの残り時間（秒）
    /// 0以下になると貿易が実行され、リセットされます。
    pub eta: f32,

    /// 使用している貿易ルートのエンティティ
    pub route: Entity,
}
```

> 📝 **プロトタイプ優先のアプローチ**:
> - 最小限のフィールドで動作する実装を優先
> - 各フィールドの意味と単位を明記
> - 値の範囲や制約を記載（例: resource_stock は 0.0 以上）
> - 拡張機能（diplomacy, riskなど）はPhase 5で追加予定

### 2.2 初期データ生成システム

```rust
use crate::components::{Nation, TradeRoute, TransportFleet};
use bevy::prelude::*;

fn setup(mut commands: Commands) {
    // 国家A
    let nation_a = commands
        .spawn(Nation {
            name: "Nation A".into(),
            gdp: 1000.0,
            resource_stock: 500.0,
        })
        .id();

    // 国家B
    let nation_b = commands
        .spawn(Nation {
            name: "Nation B".into(),
            gdp: 800.0,
            resource_stock: 300.0,
        })
        .id();

    // 貿易ルート
    let route = commands
        .spawn(TradeRoute {
            origin: nation_a,
            destination: nation_b,
            volume: 50.0,
        })
        .id();

    // 輸送艦隊
    commands.spawn(TransportFleet {
        cargo: 50.0,
        eta: 10.0,
        route,
    });

    println!("Trade simulation initialized:");
    println!("  - Nation A: GDP={}, Resources={}", 1000.0, 500.0);
    println!("  - Nation B: GDP={}, Resources={}", 800.0, 300.0);
    println!("  - Trade Route: A -> B ({} units per cycle)", 50.0);
}
```

---

## 🔁 Phase 3: システム実装

### 3.1 輸送進行システム

```rust
use crate::components::{Nation, TradeRoute, TransportFleet};
use bevy::prelude::*;

pub fn transport_execution(
    time: Res<Time>,
    mut fleets: Query<&mut TransportFleet>,
    routes: Query<&TradeRoute>,
    mut nations: Query<&mut Nation>,
) {
    for mut fleet in fleets.iter_mut() {
        fleet.eta -= time.delta_secs();

        if fleet.eta <= 0.0 {
            // ルート情報を取得
            let Ok(route) = routes.get(fleet.route) else {
                continue;
            };

            // 同じエンティティへの重複した可変借用を防ぐ
            if route.origin == route.destination {
                println!("Warning: Origin and destination are the same entity");
                fleet.eta = 10.0;
                continue;
            }

            // 安全に両方の国家を可変借用
            let Ok([mut origin, mut dest]) =
                nations.get_many_mut([route.origin, route.destination]) else {
                println!("Warning: Invalid nation entities in trade route");
                fleet.eta = 10.0;
                continue;
            };

            // 資源が十分にあるか確認
            if origin.resource_stock < fleet.cargo {
                println!(
                    "Warning: Insufficient resources - {} has {:.1} but needs {:.1}",
                    origin.name, origin.resource_stock, fleet.cargo
                );
                fleet.eta = 10.0;
                continue;
            }

            // 貿易実行
            origin.resource_stock -= fleet.cargo;
            dest.resource_stock += fleet.cargo;
            dest.gdp += fleet.cargo * 0.05;

            println!(
                "Trade completed: {:.1} units from {} to {}",
                fleet.cargo, origin.name, dest.name
            );

            fleet.eta = 10.0; // 繰り返し運行
        }
    }
}
```

### 3.2 エラーハンドリング

貿易シミュレーションで発生しうるエラーと対処方法：

| エラーケース              | 原因                       | 対処方法                                |
| ------------------- | ------------------------ | ----------------------------------- |
| **ルートエンティティが存在しない** | `fleet.route` が無効         | `routes.get()` で確認し、失敗時は `continue` |
| **国家エンティティが存在しない** | 削除された国家への参照              | `get_many_mut()` で確認                |
| **可変借用の競合**         | origin と destination が同一 | 事前チェックでスキップ                         |
| **資源不足**            | 輸出側の資源が貨物量より少ない          | 実行前に `resource_stock` を確認           |
| **負の資源量**           | 計算ミスや競合による不正な減算          | 貿易前に十分な資源があることを保証                   |

#### エラーログの活用

```rust
// プロトタイプ：println!を使用した簡易ログ
println!("Warning: {}", message);

// 本番環境では、Bevyのログシステムを使用することを推奨
use bevy::log::*;
warn!("Trade failed: insufficient resources");
error!("Critical: Nation entity not found");
```

#### 堅牢性を高めるためのベストプラクティス

1. **プロダクションコードでunwrap()を使用しない**
   ```rust
   // ❌ 避けるべき
   let route = routes.get(fleet.route).unwrap();

   // ✅ 推奨
   let Ok(route) = routes.get(fleet.route) else {
       warn!("Invalid route entity");
       continue;
   };
   ```

2. **Result型を活用する**
   ```rust
   fn execute_trade(
       origin: &mut Nation,
       dest: &mut Nation,
       amount: f32,
   ) -> Result<(), String> {
       if origin.resource_stock < amount {
           return Err(format!(
               "Insufficient resources: {} < {}",
               origin.resource_stock, amount
           ));
       }

       origin.resource_stock -= amount;
       dest.resource_stock += amount;
       Ok(())
   }
   ```

3. **不変条件の保証**
   - `resource_stock` は常に 0.0 以上

### 3.3 システム登録

```rust
use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::Duration;
use systems::{setup, transport_execution, report};

fn main() {
    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                Duration::from_secs_f64(1.0 / 60.0),
            ))
        )
        .add_systems(Startup, setup)
        .add_systems(Update, (transport_execution, report))
        .run();
}
```

> 💡 **Bevy 0.17.2の変更点**:
> - `add_system` → `add_systems` に変更され、スケジュール（`Startup`, `Update`など）を明示的に指定
> - WSL2環境では`MinimalPlugins`と`ScheduleRunnerPlugin`を使用してヘッドレスモードで実行
> - 複数のシステムをタプルで登録可能: `(transport_execution, report)`

---

## 📊 Phase 4: ログ出力・検証

### 4.1 シンプルなコンソール出力

* 各ターンの GDP・資源量をログ表示。
* シミュレーション進行を確認。

```rust
use crate::components::Nation;
use bevy::prelude::*;

pub fn report(nations: Query<&Nation>) {
    for nation in nations.iter() {
        println!(
            "{}: GDP={:.1}, Resources={:.1}",
            nation.name, nation.gdp, nation.resource_stock
        );
    }
}
```

→ `.add_systems(Update, (transport_execution, report))` で併用。

### 4.2 テストコード

CLAUDE.mdの要件に従い、テストを追加します。

#### 単体テスト例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nation_creation() {
        let nation = Nation {
            name: "Test Nation".into(),
            gdp: 1000.0,
            resource_stock: 500.0,
        };

        assert_eq!(nation.name, "Test Nation");
        assert_eq!(nation.gdp, 1000.0);
        assert_eq!(nation.resource_stock, 500.0);
    }

    #[test]
    fn test_trade_route_creation() {
        use bevy::ecs::world::World;

        let mut world = World::new();
        let origin = world.spawn_empty().id();
        let destination = world.spawn_empty().id();

        let route = TradeRoute {
            origin,
            destination,
            volume: 50.0,
        };

        assert_eq!(route.origin, origin);
        assert_eq!(route.destination, destination);
        assert_eq!(route.volume, 50.0);
    }
}
```

#### システムテスト例

```rust
#[cfg(test)]
mod system_tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_transport_execution() {
        let mut app = App::new();

        // MinimalPluginsを追加してTimeリソースを有効化
        app.add_plugins(MinimalPlugins);

        // 初期設定
        let nation_a = app.world_mut().spawn(Nation {
            name: "Nation A".into(),
            gdp: 1000.0,
            resource_stock: 500.0,
        }).id();

        let nation_b = app.world_mut().spawn(Nation {
            name: "Nation B".into(),
            gdp: 800.0,
            resource_stock: 300.0,
        }).id();

        let route = app.world_mut().spawn(TradeRoute {
            origin: nation_a,
            destination: nation_b,
            volume: 50.0,
        }).id();

        app.world_mut().spawn(TransportFleet {
            cargo: 50.0,
            eta: 0.0, // 即座に到着
            route,
        });

        // システムを実行
        app.add_systems(Update, transport_execution);
        app.update();

        // 検証：Nation Aの資源が減少
        let nation_a_stock = app.world().get::<Nation>(nation_a)
            .map(|n| n.resource_stock);
        assert_eq!(nation_a_stock, Some(450.0));

        // 検証：Nation Bの資源が増加
        let nation_b_stock = app.world().get::<Nation>(nation_b)
            .map(|n| n.resource_stock);
        assert_eq!(nation_b_stock, Some(350.0));
    }

    #[test]
    fn test_insufficient_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let nation_a = app.world_mut().spawn(Nation {
            name: "Nation A".into(),
            gdp: 1000.0,
            resource_stock: 10.0, // 不十分
        }).id();

        let nation_b = app.world_mut().spawn(Nation {
            name: "Nation B".into(),
            gdp: 800.0,
            resource_stock: 300.0,
        }).id();

        let route = app.world_mut().spawn(TradeRoute {
            origin: nation_a,
            destination: nation_b,
            volume: 50.0,
        }).id();

        app.world_mut().spawn(TransportFleet {
            cargo: 50.0,
            eta: 0.0,
            route,
        });

        app.add_systems(Update, transport_execution);
        app.update();

        // 検証：貿易が実行されず、資源量が変わらない
        let nation_a_stock = app.world().get::<Nation>(nation_a)
            .map(|n| n.resource_stock);
        assert_eq!(nation_a_stock, Some(10.0));

        let nation_b_stock = app.world().get::<Nation>(nation_b)
            .map(|n| n.resource_stock);
        assert_eq!(nation_b_stock, Some(300.0));
    }
}
```

#### テスト実行

```bash
# すべてのテストを実行
cargo test

# 特定のテストのみ実行
cargo test test_transport_execution

# テスト出力を詳細表示
cargo test -- --nocapture
```

---

## 🚀 Phase 5: 拡張案

| 機能     | 追加内容                     | Bevy側対応                        |
| ------ | ------------------------ | ------------------------------ |
| 外交変化   | 貿易成功/失敗で `diplomacy` を変化 | `EventWriter<DiplomacyChange>` |
| リスク処理  | `risk` に基づき輸送失敗確率を算出     | `rand` クレート導入                  |
| リソース価格 | 市場価格を `Resource` として管理   | `Res<Market>`                  |
| 表示     | Bevy UI or Bevy_Egui     | `bevy_egui` 追加                 |
| 並列実行   | 複数国家・艦隊を同時処理             | Bevyの並列System活用                |

---

## 🧮 今後の展開

* 国家 AI 行動ロジック (`NationAI` コンポーネント)
* 輸送事故や外交イベント (`EventSystem`)
* 銀河規模スケールへの拡張（国家を多数生成）

---

## ✅ 最終構成（ディレクトリ例）

```
omni-sim/
├── Cargo.toml
├── CLAUDE.md
├── src/
│   ├── main.rs
│   ├── components.rs
│   └── systems/
│       ├── mod.rs
│       ├── setup.rs
│       ├── transport.rs
│       └── report.rs
├── tests/
│   └── integration_test.rs
└── docs/
    └── TRADE_SIM_IMPLEMENTATION_GUIDE.md
```

---

## 🧠 補足：シミュレーション実行

```bash
cargo run
```

出力例：

```
Trade simulation initialized:
  - Nation A: GDP=1000, Resources=500
  - Nation B: GDP=800, Resources=300
  - Trade Route: A -> B (50 units per cycle)
Nation A: GDP=1000.0, Resources=500.0
Nation B: GDP=800.0, Resources=300.0
...（約10秒後）
Trade completed: 50.0 units from Nation A to Nation B
Nation A: GDP=1000.0, Resources=450.0
Nation B: GDP=802.5, Resources=350.0
```
