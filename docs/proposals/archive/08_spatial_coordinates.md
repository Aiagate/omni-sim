# 提案 8: 空間座標と宇宙地理

## 概要

現在は星系間距離がTradeRouteにハードコードされているだけで、空間的な位置関係が存在しない。**3D座標系**を導入し、距離に基づく交易・軍事・探索のダイナミクスと、新星系の動的発見を実現する。

---

## 現状の課題

| 課題 | 現行の挙動 |
|------|-----------|
| 空間の不在 | 星系に座標がない |
| 距離が静的 | TradeRoute に固定値として埋め込み |
| 新星系の発見不可 | 初期配置の3星系で固定 |
| 地理的戦略なし | 「中央に位置する」「辺境にいる」等の概念がない |

---

## 設計

### 1. 空間座標コンポーネント

```rust
/// 星系の空間座標（光年単位）
#[derive(Component, Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    /// 2点間の距離（光年）
    pub fn distance_to(&self, other: &Position) -> f64 {
        ((self.x - other.x).powi(2)
       + (self.y - other.y).powi(2)
       + (self.z - other.z).powi(2)).sqrt()
    }
}
```

#### 初期配置

| 星系 | x | y | z | 備考 |
|------|---|---|---|------|
| Sol | 0.0 | 0.0 | 0.0 | 原点（地球） |
| Alpha Centauri | 1.5 | -3.5 | -2.0 | 距離 4.37 光年 |
| Sirius | 2.5 | -6.0 | -5.5 | 距離 8.6 光年 |

### 2. 距離ベースの力学

TradeRoute の距離をハードコードではなく座標から動的に計算すべき対象は以下のとおり。

#### 2-1. 貿易

```
// 現行: distance がハードコード
TradeRoute::new(earth, proxima, 4.37, 50.0)

// 改善: 座標から自動計算
fn trade_system(...) {
    let distance = from_pos.distance_to(&to_pos);
    let transport_cost = (distance * 0.05).min(0.5);
    let travel_time = (distance / warp_speed).ceil() as u64;  // 到着遅延
    // ...
}
```

#### 2-2. 軍事（遠征ペナルティ）

```
// 遠征距離に応じた戦闘ペナルティ
fn calculate_expedition_penalty(distance: f64) -> f64 {
    let base_penalty = (distance / 10.0).min(0.5);  // 最大 50% ペナルティ
    let nav_bonus = navigation_level as f64 * 0.05;   // 航行技術で軽減
    (base_penalty - nav_bonus).max(0.0)
}

// 攻撃力 = 本来の戦力 × (1.0 - 遠征ペナルティ)
```

#### 2-3. 文化浸透（距離減衰）

```
// 文化力の影響は距離の二乗に反比例
cultural_influence = culture_power / distance.powi(2)
```

### 3. 探索と新星系の発見

宇宙航行技術のレベルに応じて**探索範囲（ = 発見できる星系の最大距離）**が広がる。

```
exploration_range = 5.0 + navigation_level × 3.0  // 光年

// 毎 Tick（低確率で探索判定）
fn exploration_system(...) {
    let hash = deterministic_hash(seed, tick, nation_index);
    if hash % 50 != 0 { return; }  // 2% の確率

    let range = exploration_range(tech.navigation_level);
    
    // 未発見の星系カタログから範囲内のものを選択
    for candidate in &undiscovered_systems {
        if own_position.distance_to(&candidate.position) <= range {
            discover_system(candidate);
            break;
        }
    }
}
```

#### 未発見星系のカタログ

```rust
/// シミュレーション開始時に生成される未発見星系のプール
#[derive(Resource, Debug)]
pub struct GalacticCatalog {
    pub undiscovered_systems: Vec<PotentialSystem>,
}

#[derive(Debug, Clone)]
pub struct PotentialSystem {
    pub name: String,
    pub position: Position,
    pub planets: Vec<PotentialPlanet>,
    pub discovered: bool,
    pub discovered_by: Option<Entity>,
}

#[derive(Debug, Clone)]
pub struct PotentialPlanet {
    pub name: String,
    pub environment: PlanetaryEnvironment,
    pub resources: Resources,
    pub production: Production,
}
```

### 4. 銀河マップの動的生成

初期化時にシードから決定論的に銀河全体を生成。

```
fn generate_galaxy(seed: u64, num_systems: usize) -> GalacticCatalog {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // 銀河の腕構造（螺旋銀河を模倣）
    for i in 0..num_systems {
        let arm = i % 4;  // 4本の銀河腕
        let distance_from_center = rng.gen_range(5.0..100.0);
        let angle = arm as f64 * PI / 2.0 + distance_from_center * 0.1;
        
        let x = distance_from_center * angle.cos() + rng.gen_range(-2.0..2.0);
        let y = distance_from_center * angle.sin() + rng.gen_range(-2.0..2.0);
        let z = rng.gen_range(-3.0..3.0);  // 銀河面からのズレ
        
        // 星系の特性も距離から決定
        let planet_count = 1 + rng.gen_range(0..3);
        // ...
    }
}
```

### 5. 植民と新惑星の開拓

`colony_ship` 技術をアンロックした国家は、発見した星系に植民可能。

```
植民プロセス:
  1. 植民船を建造: 鉱物 500 + 工業品 300 + エネルギー 200
  2. 植民船を発進: 到着まで distance × 2 Tick
  3. 植民地を設立: 人口 100,000（母惑星から移住）
  4. 新惑星が ECS エンティティとしてスポーン

植民地の特性:
  - 初期段階は母国に強く依存（資源・技術の輸送が必要）
  - 人口が 100万 を超えると自立的な成長が可能
  - 将来的に独立する可能性も（幸福度次第）
```

### 6. 宇宙の障害物

```rust
/// 空間上の障害物や特殊地形
#[derive(Component, Debug, Clone)]
pub enum SpaceFeature {
    /// 星間ガス雲: 通過する貿易ルートにペナルティ
    Nebula { radius: f64, density: f64 },
    /// 重力異常: 航行コスト増大
    GravityAnomaly { radius: f64, strength: f64 },
    /// ワームホール: 2点間の距離をゼロにする近道
    Wormhole { exit_position: Position },
    /// 危険宙域: 艦船にダメージ
    DangerZone { radius: f64, damage_per_tick: f64 },
}
```

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `components/common.rs` | `Position` コンポーネント追加 |
| `components/economy.rs` | `TradeRoute` から `distance` フィールドを削除（動的計算に） |
| `systems/trade.rs` | 距離を座標から算出 |
| `systems/war.rs` | 遠征ペナルティの導入 |
| `systems/exploration.rs` | 【NEW】探索・発見システム |
| `systems/colonization.rs` | 【NEW】植民システム |
| `plugins/world_init.rs` | 座標設定 + `GalacticCatalog` の生成 |

---

## 期待される効果

1. **地理的戦略**: 「Sol は中心部で交易有利、辺境は安全だが孤立」
2. **探索のワクワク**: 新星系の発見が技術投資のモチベーション
3. **拡張の代償**: 植民は大きな投資。防衛が手薄になるリスク
4. **スケール感**: 3星系 → 数十星系に銀河が拡大していく壮大さ
