# コンポーネントリファレンス

全 ECS コンポーネントのフィールド定義、型、初期値、用途を記述する。

---

## 共通 (`common.rs`)

### `Name`
| フィールド | 型 | 説明 |
|-----------|-----|------|
| `0` | `String` | エンティティの表示名 |

### マーカーコンポーネント

| コンポーネント | 用途 |
|---------------|------|
| `StarSystem` | 星系エンティティ |
| `Planet` | 惑星エンティティ |
| `Nation` | 国家エンティティ |

### 関係コンポーネント

| コンポーネント | フィールド | 型 | 説明 |
|---------------|-----------|-----|------|
| `BelongsToStarSystem` | `0` | `Entity` | 所属星系 |
| `BelongsToPlanet` | `0` | `Entity` | 所属惑星 |
| `BelongsToNation` | `0` | `Entity` | 所属国家 |

---

## 経済 (`economy.rs`)

### `Resources`
惑星の資源ストック。

| フィールド | 型 | デフォルト | 説明 |
|-----------|-----|----------|------|
| `food` | `f64` | 100.0 | 食料 |
| `minerals` | `f64` | 100.0 | 鉱物 |
| `energy` | `f64` | 100.0 | エネルギー |
| `manufactured_goods` | `f64` | 50.0 | 工業品 |

**メソッド**: `get(ResourceType)`, `set(ResourceType, f64)`

### `Production`
惑星の毎 Tick 産出レート（技術・労働力ボーナス適用前のベース値）。

| フィールド | 型 | デフォルト | 説明 |
|-----------|-----|----------|------|
| `food_rate` | `f64` | 10.0 | 食料産出/Tick |
| `mineral_rate` | `f64` | 5.0 | 鉱物産出/Tick |
| `energy_rate` | `f64` | 8.0 | エネルギー産出/Tick |
| `manufacturing_rate` | `f64` | 3.0 | 工業品産出/Tick |

### `TradeRoute`
二つの惑星間の貿易接続。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `from_planet` | `Entity` | 輸出元惑星 |
| `to_planet` | `Entity` | 輸入先惑星 |
| `distance` | `f64` | 距離（光年） |
| `capacity` | `f64` | 1 Tick あたりの輸送容量 |
| `active` | `bool` | アクティブフラグ |

**メソッド**: `transport_cost() -> f64` — `min(distance × 0.05, 0.5)`

### `ResourceType` (enum)
`Food`, `Minerals`, `Energy`, `ManufacturedGoods`

### `DepletableResources`
惑星の有限資源と採掘の状態。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `mineral_reserves` | `f64` | 現在の鉱物埋蔵量 |
| `initial_mineral_reserves` | `f64` | 初期鉱物埋蔵量 |
| `fossil_fuel_reserves` | `f64` | 現在の化石燃料埋蔵量 |
| `initial_fossil_fuel_reserves` | `f64` | 初期化石燃料埋蔵量 |
| `rare_earth_reserves` | `f64` | レアアース埋蔵量 |
| `initial_rare_earth_reserves` | `f64` | 初期レアアース埋蔵量 |
| `mining_depth` | `f64` | 採掘深度 (0.0 〜 1.0) |
| `available_rare_earth` | `bool` | レアアース採掘可能性フラグ |

---

## 人口 (`population.rs`)

### `Population`

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `count` | `f64` | 現在の人口数 |
| `base_growth_rate` | `f64` | 基本成長率/Tick（例: 0.002 = 0.2%） |
| `effective_growth_rate` | `f64` | 食料等の影響を加味した実効成長率 |
| `last_change` | `f64` | 前 Tick との人口差分 |
| `starvation_ticks` | `u32` | 連続飢餓 Tick 数 |

---

## 環境 (`environment.rs`)

### `PlanetaryEnvironment`
惑星の物理特性とマクロな環境状態。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `mass`, `gravity`, `radius` | `f64` | 物理特性 |
| `atmosphere` | `AtmosphereType` | 大気組成タイプ |
| `atmospheric_pressure` | `f64` | 大気圧 (atm) |
| `temperature` | `f64` | 平均気温 (℃) |
| `water_coverage` | `f64` | 水の存在率 (0.0 〜 1.0) |
| `habitability` | `f64` | 総合居住性 (0.0 〜 1.0) |
| `population_capacity`| `f64` | 最大人口収容力 |
| `radiation`, `tectonic_activity`, `meteorite_risk` | `f64` | 環境リスク係数 |
| `mineral_deposits` | `f64` | 鉱物資源ポテンシャル |
| `mineral_accessibility` | `f64` | 資源採掘の容易さ |
| `renewable_energy_potential` | `f64` | 再生可能エネルギーポテンシャル |

### `RenewableResources`
再生可能な表面資源の状態。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `soil_fertility` | `f64` | 土壌肥沃度 (0.0 〜 1.5) |
| `forest_coverage` | `f64` | 森林被覆率 |
| `renewable_energy_output` | `f64` | 再生可能エネルギー出力係数 |

### `EnvironmentalHealth`
人為的な環境負荷の状態。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `air_pollution` | `f64` | 大気汚染レベル |
| `water_pollution` | `f64` | 水質汚染レベル |
| `health_penalty` | `f64` | 健康被害（成長率へのペナルティ） |

---

## 軍事 (`military.rs`)

### `MilitaryStrength`

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `ships` | `u32` | 艦船数 |
| `power` | `f64` | 総合戦力（`ships × 10.0`） |
| `defense_bonus` | `f64` | 防衛ボーナス（予約、現在 1.2 固定） |
| `in_combat` | `bool` | 戦闘中フラグ |

**メソッド**: `recalculate()` — `power = ships × 10.0`

---

## 外交 (`diplomacy.rs`)

### `DiplomaticRelation`

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `owner_nation` | `Entity` | この関係のオーナー国家 |
| `target_nation` | `Entity` | 相手国 |
| `score` | `f64` | 関係スコア（-100 〜 +100） |
| `trend` | `f64` | スコアの変動トレンド |

### `AtWar`
戦争状態。国家エンティティに付与される。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `enemy_nation` | `Entity` | 敵国 |
| `enemy_planet` | `Entity` | 敵の惑星 |
| `own_planet` | `Entity` | 自分の惑星 |
| `started_at` | `u64` | 戦争開始 Tick |
| `total_ships_lost` | `u32` | 累積撃破艦船数 |
| `total_casualties` | `f64` | 累積民間犠牲者数 |

---

## 国家 AI (`national_ai.rs`)

### `NationalCharacter`
国家の基礎的な性格パラメータ (0.0 〜 1.0)。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `aggression` | `f64` | 好戦性 |
| `trade_affinity` | `f64` | 交易志向 |
| `research_focus` | `f64` | 技術志向 |
| `expansionism` | `f64` | 拡張主義 |
| `diplomacy_flexibility` | `f64` | 外交の揺らぎ |
| `ideology` | `enum` | 国家の政治思想 (Statist, Liberal, etc.) |

### `NationalMemory`
他国に対しての歴史的な出来事を記録。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `events` | `Vec<(Entity, AIEvent)>` | 相手国ごとの出来事履歴 |

### `InternalFactions`
国内派閥の勢力均衡。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `power` | `HashMap<FactionType, f64>` | 軍部、商人、技術者の影響力 |

---

## 技術 (`technology.rs`)

### `TechnologyState`

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `agriculture_level` | `u32` | 農業技術レベル |
| `mining_level` | `u32` | 採掘技術レベル |
| `energy_level` | `u32` | エネルギー技術レベル |
| `manufacturing_level` | `u32` | 工業技術レベル |
| `military_level` | `u32` | 軍事技術レベル |
| `navigation_level` | `u32` | 宇宙航行技術レベル |
| `current_research` | `TechField` | 現在研究中の分野 |
| `research_points` | `f64` | 現在の研究ポイント蓄積 |
| `research_rate` | `f64` | 直近の研究レート |

**メソッド**:
- `level(TechField) -> u32`
- `cost_for_next_level(TechField) -> f64` — `100.0 × 1.5^level`
- `bonus_multiplier(TechField) -> f64` — `1.0 + level × 0.10`
- `add_research(f64) -> bool` — ポイント加算、レベルアップ判定
- `total_level() -> u32` — 全分野の合計

### `TechField` (enum)
`Agriculture`, `Mining`, `EnergyTech`, `Manufacturing`, `MilitaryTech`, `SpaceNavigation`, `EnvironmentalTech`, `NuclearFusion`

---

## テラフォーミング (`terraforming.rs`)

### `TerraformingProject`
惑星ごとの改造進捗。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `current_phase` | `TerraformingPhase` | 現在のフェーズ |
| `progress` | `f64` | 現在のフェーズの進捗 (0.0 〜 1.0) |

---

## インデックス (`nation_index.rs`)

### `NationPlanetIndex` (Resource)
国家 ID から関連惑星・星系を高速に検索するためのハッシュマップ。

---

## イベント (`events.rs`)

### `EventKind` (enum)

| バリアント | 効果 |
|-----------|------|
| `Plague` | 人口減少 |
| `BountifulHarvest` | 食料ボーナス |
| `MineralDiscovery` | 鉱物ボーナス |
| `BabyBoom` | 人口増加 |
| `EnergyCrisis` | エネルギー減少 |
| `TechBreakthrough` | 研究ポイントボーナス |
| `Rebellion` | 人口・資源減少 |
| `TradeBoom` | 全資源微増 |
| `Earthquake` | 地震被害 |
| `RadiationStorm` | 放射線嵐 |
| `MeteoriteImpact` | 隕石衝突 |
| `EnvironmentalDisaster` | 環境災害（汚染閾値で発生） |

> 絵文字や日本語名の変換は表示レイヤー（`cli_output.rs`）が担当する。

### `EventEffect`
イベントの効果を表す構造化データ。表示レイヤーが人間可読な文字列に変換する。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `population_change` | `f64` | 人口変動量 |
| `food_change` | `f64` | 食料変動量 |
| `minerals_change` | `f64` | 鉱物変動量 |
| `energy_change` | `f64` | エネルギー変動量 |
| `goods_change` | `f64` | 工業品変動量 |
| `research_change` | `f64` | 研究ポイント変動量 |

### `EventLog` (Resource)
| フィールド | 型 | 説明 |
|-----------|-----|------|
| `events` | `Vec<EventRecord>` | イベント履歴 |

### `EventRecord`
| フィールド | 型 | 説明 |
|-----------|-----|------|
| `tick` | `u64` | 発生 Tick |
| `planet_name` | `String` | 対象惑星名 |
| `kind` | `EventKind` | イベント種類 |
| `effect` | `EventEffect` | イベント効果（構造化データ） |

---

## シミュレーションイベント (`simulation_event.rs`)

### `SimulationEvent` (Bevy Event)
ドメインシステムが `EventWriter` で送出し、表示レイヤーが `EventReader` で受信する。

| バリアント | 送出元 | 頻度 |
|-----------|---------|------|
| `ResourceReport` | economy | 毎 Tick |
| `PopulationReport` | population | 毎 Tick |
| `TradeTransfer` | trade | 輸送発生時 |
| `MilitaryReport` | military | 毎 Tick |
| `DiplomacyReport` | diplomacy | 毎 Tick |
| `WarDeclared` | war_trigger | 戦争勃発時 |
| `Ceasefire` | combat | 停戦時 |
| `Surrender` | combat | 降伏時 |
| `CombatReport` | combat | 毎戦闘 Tick |
| `TechLevelUp` | research | レベルアップ時 |
| `ResearchReport` | research | 毎 Tick |
| `RandomEvent` | events | イベント発生時 |
| `TerraformingProgress` | terraforming | 進捗時 |
| `TerraformingPhaseComplete` | terraforming | フェーズ完了時 |
| `WorldInitialized` | world_init | 起動時 |
---

## TUI 表示・操作 (`tui_state.rs`)

TUI モード時にのみ使用される表示用キャッシュと制御用リソース。

### `TuiAppState` (Resource)
TUI の全表示データを保持するスナップショットリソース。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `planets` | `Vec<PlanetSnapshot>` | 惑星の表示用データ一覧 |
| `event_log` | `Vec<EventLogEntry>` | 最新のイベントログ履歴 |
| `diplomacy` | `Vec<DiplomacyEntry>` | 外交関係の表示データ |
| `wars` | `Vec<WarEntry>` | 進行中の戦争の表示データ |
| `current_tick` | `u64` | 現在の Tick |
| `max_ticks` | `u64` | シミュレーション最大 Tick |
| `selected_planet` | `usize` | ダッシュボードでフォーカスされている惑星の index |
| `log_scroll_offset` | `usize` | ログ表示のスクロール位置 |

### `SimControl` (Resource)
シミュレーションの進行を制御するリソース。

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `paused` | `bool` | 一時停止フラグ |
| `step_once` | `bool` | 1 Tick だけ進めるフラグ |
| `speed` | `SimSpeed` | シミュレーション速度 (x1, x5, x10) |
| `quit_requested` | `bool` | 終了リクエストフラグ |
