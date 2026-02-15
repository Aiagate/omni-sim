# 提案 11: シナリオ・MOD システム

## 概要

現在の初期世界は `world_init.rs` にハードコードされている。**外部ファイル（TOML）** から世界設定を読み込み可能にし、カスタムシナリオの作成・共有を実現する。

---

## 設計

### 1. シナリオファイル形式

```toml
# scenarios/default.toml — デフォルトシナリオ

[simulation]
name = "Sol近傍3星系"
description = "太陽系近傍の3つの星系による文明シミュレーション"
max_ticks = 100
seed = 42

# === 星系定義 ===

[[star_systems]]
name = "Sol 星系"
position = [0.0, 0.0, 0.0]

[[star_systems.planets]]
name = "地球"
population = 10_000_000
base_growth_rate = 0.002
resources = { food = 1000, minerals = 500, energy = 800, goods = 300 }
production = { food = 120, minerals = 40, energy = 80, manufacturing = 30 }
military = { ships = 10 }

[star_systems.planets.environment]   # 省略可能（デフォルト値あり）
gravity = 1.0
atmosphere = "earth_like"
temperature = 15.0
water = 0.71
radiation = 0.05

[[star_systems.planets.nations]]
name = "地球連邦"
character = { aggression = 0.3, trade = 0.6, research = 0.8, expansion = 0.4, flexibility = 0.7 }
government = "democracy"

# --- Alpha Centauri ---

[[star_systems]]
name = "Alpha Centauri 星系"
position = [1.5, -3.5, -2.0]

[[star_systems.planets]]
name = "プロキシマb"
population = 2_000_000
base_growth_rate = 0.003
resources = { food = 400, minerals = 800, energy = 300, goods = 150 }
production = { food = 30, minerals = 80, energy = 25, manufacturing = 20 }
military = { ships = 5 }

[[star_systems.planets.nations]]
name = "ケンタウリ共和国"
character = { aggression = 0.2, trade = 0.9, research = 0.5, expansion = 0.3, flexibility = 0.8 }
government = "oligarchy"

# === 外交関係 ===

[[diplomacy]]
from = "地球連邦"
to = "ケンタウリ共和国"
score = 30.0

[[diplomacy]]
from = "地球連邦"
to = "シリウス帝国"
score = -10.0

# === 貿易ルート ===

[[trade_routes]]
from = "地球"
to = "プロキシマb"
capacity = 50.0

# === 初期条約 ===

[[treaties]]
type = "trade_agreement"
parties = ["地球連邦", "ケンタウリ共和国"]
terms = { trade_bonus = 1.5 }

# === 未発見星系（探索で発見される） ===

[[hidden_systems]]
name = "バーナード星系"
position = [3.0, 2.0, -4.5]
discovery_difficulty = 0.3    # 低い → 発見されやすい

[[hidden_systems.planets]]
name = "バーナードIII"
population = 0
resources = { food = 0, minerals = 1200, energy = 200, goods = 0 }
production = { food = 10, minerals = 100, energy = 40, manufacturing = 10 }
environment = { gravity = 0.6, atmosphere = "thin", temperature = -30.0, radiation = 0.2 }
```

### 2. 大規模シナリオ例

```toml
# scenarios/galaxy_war.toml — 銀河大戦シナリオ

[simulation]
name = "銀河大戦"
description = "10星系・5国家による大規模戦争シナリオ"
max_ticks = 500
seed = 777

# 5国家が異なる地域に分散
# 初期外交: 2つの勢力圏に分かれている
# 中央の星系は資源が豊富だが未開拓
# → このシナリオの核心は中央の資源をめぐる争奪戦
```

### 3. ランダム銀河生成

シナリオファイルの代わりに、パラメトリック生成も可能にする。

```toml
# scenarios/random.toml

[simulation]
name = "ランダム銀河"
max_ticks = 300
seed = 12345

[generation]
mode = "random"
num_star_systems = 20
num_nations = 5
galaxy_radius = 50.0        # 光年
planet_density = 1.5         # 星系あたりの惑星数
resource_variance = 0.5      # 資源のばらつき
hostile_environment_ratio = 0.3  # 過酷な環境の惑星の割合
```

### 4. データ構造

```rust
/// シナリオファイルのデシリアライズ構造
#[derive(Debug, Deserialize)]
pub struct ScenarioFile {
    pub simulation: SimulationSettings,
    #[serde(default)]
    pub star_systems: Vec<StarSystemDef>,
    #[serde(default)]
    pub diplomacy: Vec<DiplomacyDef>,
    #[serde(default)]
    pub trade_routes: Vec<TradeRouteDef>,
    #[serde(default)]
    pub treaties: Vec<TreatyDef>,
    #[serde(default)]
    pub hidden_systems: Vec<StarSystemDef>,
    #[serde(default)]
    pub generation: Option<GenerationSettings>,
}

/// world_init.rs を置換する初期化関数
fn load_scenario(
    path: &Path,
    commands: &mut Commands,
    events: &mut EventWriter<SimulationEvent>,
) -> Result<SimulationConfig, ScenarioError> {
    let content = std::fs::read_to_string(path)?;
    let scenario: ScenarioFile = toml::from_str(&content)?;
    
    // エンティティのスポーン
    for system_def in &scenario.star_systems {
        let system_entity = commands.spawn(/* ... */).id();
        for planet_def in &system_def.planets {
            // ...
        }
    }
    // ...
}
```

### 5. CLI 統合

```bash
# デフォルトシナリオ（内蔵）
cargo run

# カスタムシナリオ
cargo run -- --scenario scenarios/galaxy_war.toml

# ランダム生成
cargo run -- --scenario scenarios/random.toml --seed 42

# パラメトリック生成（CLIのみ）
cargo run -- --random --systems 10 --nations 4 --radius 30
```

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `scenario.rs` | 【NEW】シナリオ読み込み・パース |
| `plugins/world_init.rs` | シナリオからの初期化に対応 |
| `main.rs` | `--scenario` フラグの追加 |
| `Cargo.toml` | `toml`, `serde` 依存追加 |

---

## 期待される効果

1. **無限の再プレイ性**: シナリオファイルで無限のバリエーション
2. **バランステスト**: パラメータを微調整して結果を比較
3. **コミュニティ**: シナリオの共有・配布
4. **教育**: 歴史的なシナリオの再現
