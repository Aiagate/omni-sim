# テストドキュメント

Omni Simulation Projectのテストフレームワーク説明書

---

## テスト構成

```
tests/
├── unit/                    # 単体テスト（各クラス・関数の独立テスト）
│   ├── test_ship.py         # 艦船クラステスト（28テスト）
│   ├── test_fleet.py        # 艦隊クラステスト（22テスト）
│   ├── test_combat.py       # 戦闘解決ロジックテスト（8テスト）
│   └── test_resources.py    # 資源管理システムテスト（19テスト）
├── integration/             # 統合テスト（複数システムの連携テスト）
│   └── test_basic_combat.py # 基本戦闘統合テスト
├── fixtures/                # テストフィクスチャ（共通テストデータ）
│   └── scenarios.py         # シナリオ・サンプルデータ定義
└── README.md                # このファイル
```

---

## テスト実行方法

### 基本コマンド

```bash
# 全テスト実行
uv run pytest tests/

# 単体テストのみ実行
uv run pytest tests/unit/

# 統合テストのみ実行
uv run pytest tests/integration/

# 詳細出力（-v: verbose）
uv run pytest tests/unit/ -v

# 特定のテストファイルを実行
uv run pytest tests/unit/test_ship.py

# 特定のテストクラスを実行
uv run pytest tests/unit/test_ship.py::TestShipCreation

# 特定のテストメソッドを実行
uv run pytest tests/unit/test_ship.py::TestShipCreation::test_destroyer_creation
```

### マーカーを使用したテスト実行

```bash
# @pytest.mark.unit が付与されたテストのみ実行
uv run pytest -m unit

# @pytest.mark.integration が付与されたテストのみ実行
uv run pytest -m integration

# @pytest.mark.slow が付与されたテストを除外
uv run pytest -m "not slow"
```

### カバレッジレポート生成

```bash
# カバレッジ付きでテスト実行
uv run pytest --cov=src tests/

# HTMLレポート生成（htmlcov/index.htmlに出力）
uv run pytest --cov=src --cov-report=html tests/

# ターミナルで詳細表示
uv run pytest --cov=src --cov-report=term-missing tests/
```

---

## テストカテゴリ詳細

### 単体テスト（Unit Tests）

各クラス・関数の独立した動作を検証します。外部依存を最小限にし、決定論的な結果を保証します。

#### test_ship.py（28テスト）

**テスト対象**: `src/entities/ship.py`

**テストクラス構成**:
- `TestShipCreation`: 艦船生成テスト（駆逐艦・巡洋艦・戦艦）
- `TestShipCombat`: 戦闘関連テスト（ダメージ受け取り・撃破判定）
- `TestShipFuel`: 燃料管理テスト（消費・残量判定）
- `TestShipAmmo`: 弾薬管理テスト（消費・射撃可能判定）
- `TestShipRefuelRearm`: 補給テスト（燃料・弾薬の最大値制限）

**主要なテストケース**:
```python
# 艦船パラメータ検証
def test_destroyer_creation(self, sample_destroyer):
    assert sample_destroyer.ship_class == "destroyer"
    assert sample_destroyer.hp == 100
    assert sample_destroyer.attack == 20

# ダメージ処理
def test_take_damage(self, sample_destroyer):
    sample_destroyer.take_damage(30)
    assert sample_destroyer.hp == 70

# 燃料消費
def test_consume_fuel_success(self, sample_destroyer):
    success = sample_destroyer.consume_fuel(1000.0)
    assert success is True
    assert sample_destroyer.fuel == 4000.0

# 補給処理（最大値制限）
def test_refuel_max(self):
    ship = Ship(1, "destroyer", "Test")
    ship.fuel = 4500.0
    refueled = ship.refuel(1000.0)
    assert refueled == 500.0  # 4500 + 500 = 5000（最大）
```

#### test_fleet.py（22テスト）

**テスト対象**: `src/entities/fleet.py`

**テストクラス構成**:
- `TestFleetCreation`: 艦隊生成テスト
- `TestFleetState`: 状態遷移テスト（SEARCHING/ENGAGED/DESTROYED）
- `TestFleetShipManagement`: 艦船管理テスト（生存艦船取得・全滅判定）
- `TestFleetResources`: 資源集計テスト（総燃料・総弾薬）
- `TestFleetMixedComposition`: 混成艦隊テスト

**主要なテストケース**:
```python
# 艦隊状態遷移
def test_state_transition_to_engaged(self, sample_fleet):
    sample_fleet.state = FleetState.ENGAGED
    assert sample_fleet.state == FleetState.ENGAGED

# 生存艦船取得
def test_get_alive_ships_after_destruction(self, sample_fleet):
    sample_fleet.ships[0].take_damage(1000)  # 1隻撃破
    alive = sample_fleet.get_alive_ships()
    assert len(alive) == 1

# 総燃料・総弾薬
def test_total_fuel(self, sample_fleet):
    # 駆逐艦2隻 × 5000kg = 10000kg
    assert sample_fleet.total_fuel() == 10000.0

# 戦闘可能判定
def test_cannot_fight_without_ammo(self, sample_fleet):
    for ship in sample_fleet.ships:
        ship.ammo = 0
    assert sample_fleet.can_fight() is False
```

#### test_combat.py（8テスト）

**テスト対象**: `src/combat/resolver.py`

**テストクラス構成**:
- `TestCombatResolver`: 戦闘解決テスト（ダメージ計算・命中判定）
- `TestFleetCombat`: 艦隊戦闘テスト
- `TestAmmoConsumption`: 弾薬消費テスト

**主要なテストケース**:
```python
# ダメージ計算（命中時）
def test_damage_calculation_hit(self, reset_rng):
    attacker = Ship(1, "cruiser", "Attacker")  # 攻撃40
    defender = Ship(2, "destroyer", "Defender")  # 防御10
    resolver = CombatResolver()

    damages = []
    for _ in range(10):
        attacker.ammo = attacker.max_ammo
        damage = resolver._calculate_damage(attacker, defender)
        damages.append(damage)

    # 命中時: 40 - 10 = 30ダメージ、外れ時: 0
    assert 30 in damages or 0 in damages

# 最低ダメージ保証
def test_minimum_damage(self, reset_rng):
    attacker = Ship(1, "destroyer", "Weak")  # 攻撃20
    defender = Ship(2, "battleship", "Strong")  # 防御50
    # 攻撃力 < 防御力でも MIN_DAMAGE = 5 保証

# 弾薬消費
def test_ammo_consumed_on_attack(self, reset_rng):
    initial_ammo = attacker_ship.ammo
    damage = resolver._attack_fleet(attacker_fleet, defender_fleet)
    assert attacker_ship.ammo == initial_ammo - attacker_ship.ammo_per_shot
```

#### test_resources.py（19テスト）

**テスト対象**: `src/entities/planet.py`, `src/systems/resource_system.py`, `src/systems/fleet_system.py`

**テストクラス構成**:
- `TestPlanet`: 惑星の資源管理テスト
- `TestResourceSystem`: ResourceSystemのテスト
- `TestFleetSystem`: FleetSystemのテスト
- `TestStarSystem`: 星系管理テスト
- `TestShipRefuelRearm`: 艦船補給テスト

**主要なテストケース**:
```python
# 惑星の初期備蓄（産出量の100倍）
def test_initial_stock(self, sample_planet):
    assert sample_planet.fuel_stock == 100000.0  # 1000 * 100
    assert sample_planet.ammo_stock == 10000  # 100 * 100

# 資源産出
def test_produce_resources(self, sample_planet):
    initial_fuel = sample_planet.fuel_stock
    sample_planet.produce_resources()
    assert sample_planet.fuel_stock == initial_fuel + 1000.0

# ResourceSystemによる一括更新
def test_resource_system_update(self, sample_planet):
    system = ResourceSystem()
    initial_fuel = sample_planet.fuel_stock
    system.update([sample_planet], tick=1)
    assert sample_planet.fuel_stock == initial_fuel + sample_planet.fuel_production

# FleetSystemによる燃料消費
def test_fleet_fuel_consumption(self):
    ship = Ship(1, "destroyer", "Destroyer-1")
    fleet = Fleet(1, "Test-Fleet", (0.0, 0.0, 0.0), ships=[ship])
    system = FleetSystem()
    initial_fuel = ship.fuel
    system.update_consumption([fleet], tick=1)
    # 駆逐艦の燃料消費率: 10kg/tick
    assert ship.fuel == initial_fuel - 10.0
```

### 統合テスト（Integration Tests）

複数のシステムを組み合わせた動作を検証します。イベントスケジューラーを含む実際のシミュレーションフローをテストします。

#### test_basic_combat.py

**テスト対象**: イベント駆動システム全体（`EventScheduler`, `DetectionEvent`, `CombatEvent`）

**テストクラス構成**:
- `TestBasicCombatIntegration`: 基本戦闘統合テスト
- `TestSimulationController`: SimulationController統合テスト
- `TestLongBattle`: 長時間戦闘テスト（@pytest.mark.slow）

**主要なテストケース**:
```python
# 索敵→戦闘の連鎖
def test_detection_triggers_combat(self, reset_rng):
    scheduler = EventScheduler()
    fleet_a = Fleet(1, "Fleet-A", (0.0, 0.0, 0.0), ships=[Ship(1, "destroyer", "A-Ship")])
    fleet_b = Fleet(2, "Fleet-B", (10.0, 0.0, 0.0), ships=[Ship(2, "destroyer", "B-Ship")])

    detection = DetectionEvent(tick=1, fleet_a=fleet_a, fleet_b=fleet_b)
    scheduler.schedule(detection)
    scheduler.run_until(max_tick=10)

    # 戦闘が開始されている
    assert fleet_a.state == FleetState.ENGAGED or fleet_b.state == FleetState.ENGAGED

# 戦闘が全滅まで継続する
def test_combat_until_destruction(self, reset_rng):
    # 圧倒的な戦力差（戦艦3隻 vs 駆逐艦1隻）
    scheduler.run_until(max_tick=100)
    assert fleet_a.is_destroyed() or fleet_b.is_destroyed()
```

---

## フィクスチャ（Fixtures）

`tests/fixtures/scenarios.py`で共通のテストデータを定義しています。

### 基本フィクスチャ

```python
@pytest.fixture
def sample_destroyer() -> Ship:
    """駆逐艦のサンプル"""
    return Ship(1, "destroyer", "Test-Destroyer")

@pytest.fixture
def sample_cruiser() -> Ship:
    """巡洋艦のサンプル"""
    return Ship(2, "cruiser", "Test-Cruiser")

@pytest.fixture
def sample_battleship() -> Ship:
    """戦艦のサンプル"""
    return Ship(3, "battleship", "Test-Battleship")

@pytest.fixture
def sample_fleet() -> Fleet:
    """駆逐艦2隻の艦隊"""
    ships = [
        Ship(1, "destroyer", "DD-1"),
        Ship(2, "destroyer", "DD-2")
    ]
    return Fleet(1, "Test-Fleet", (0.0, 0.0, 0.0), ships=ships)

@pytest.fixture
def sample_mixed_fleet() -> Fleet:
    """混成艦隊（戦艦・巡洋艦・駆逐艦）"""
    ships = [
        Ship(1, "battleship", "BB-1"),
        Ship(2, "cruiser", "CA-1"),
        Ship(3, "destroyer", "DD-1")
    ]
    return Fleet(1, "Mixed-Fleet", (0.0, 0.0, 0.0), ships=ships)

@pytest.fixture
def sample_planet() -> Planet:
    """資源産出惑星"""
    return Planet(
        planet_id=1,
        name="Test-Planet",
        position=(0.0, 0.0, 0.0),
        fuel_production=1000.0,  # 1000kg/tick
        ammo_production=100      # 100発/tick
    )

@pytest.fixture
def sample_star_system(sample_planet, sample_fleet) -> StarSystem:
    """星系（惑星1つ、艦隊1つ）"""
    system = StarSystem(1, "Test-System", (0.0, 0.0, 0.0))
    system.add_planet(sample_planet)
    system.add_fleet(sample_fleet)
    return system
```

### ヘルパー関数

```python
def create_damaged_ship(ship_id: int, ship_class: str, damage: int) -> Ship:
    """ダメージを受けた艦船を生成"""
    ship = Ship(ship_id, ship_class, f"Damaged-{ship_class}")
    ship.take_damage(damage)
    return ship

def create_low_ammo_ship(ship_id: int, ship_class: str, ammo: int) -> Ship:
    """弾薬が少ない艦船を生成"""
    ship = Ship(ship_id, ship_class, f"LowAmmo-{ship_class}")
    ship.ammo = ammo
    return ship

def create_low_fuel_ship(ship_id: int, ship_class: str, fuel: float) -> Ship:
    """燃料が少ない艦船を生成"""
    ship = Ship(ship_id, ship_class, f"LowFuel-{ship_class}")
    ship.fuel = fuel
    return ship
```

### 乱数固定フィクスチャ

```python
@pytest.fixture
def reset_rng():
    """決定論的乱数生成のリセット"""
    from src.utils.rng import set_global_seed
    set_global_seed(42)
    yield
    set_global_seed(42)  # テスト後も再固定
```

**使用例**:
```python
def test_combat_with_fixed_seed(reset_rng):
    """シード固定により再現可能な戦闘テスト"""
    # 同じシードなら同じ結果が保証される
    pass
```

---

## 新しいテストの書き方

### 1. 単体テストの追加

```python
# tests/unit/test_new_feature.py
import pytest
from src.entities.new_entity import NewEntity

@pytest.mark.unit
class TestNewEntity:
    """新機能のテスト"""

    def test_creation(self):
        """生成テスト"""
        entity = NewEntity(1, "Test")
        assert entity.name == "Test"

    def test_behavior(self, sample_fixture):
        """動作テスト（フィクスチャ使用）"""
        result = sample_fixture.do_something()
        assert result is True
```

### 2. 統合テストの追加

```python
# tests/integration/test_new_integration.py
import pytest
from src.core.scheduler import EventScheduler
from src.events.new_event import NewEvent

@pytest.mark.integration
class TestNewIntegration:
    """新しいイベントの統合テスト"""

    def test_event_chain(self, reset_rng):
        """イベント連鎖のテスト"""
        scheduler = EventScheduler()
        event = NewEvent(tick=1)
        scheduler.schedule(event)
        scheduler.run_until(max_tick=10)
        # アサーション
```

### 3. フィクスチャの追加

```python
# tests/fixtures/scenarios.py
@pytest.fixture
def sample_new_entity() -> NewEntity:
    """新エンティティのサンプル"""
    return NewEntity(
        entity_id=1,
        name="Sample-Entity",
        param1=100.0,
        param2=50
    )
```

---

## テストマーカー説明

### @pytest.mark.unit

単体テスト用マーカー。各クラス・関数の独立した動作を検証。

```python
@pytest.mark.unit
class TestShip:
    def test_creation(self):
        pass
```

### @pytest.mark.integration

統合テスト用マーカー。複数システムの連携を検証。

```python
@pytest.mark.integration
class TestCombatIntegration:
    def test_full_battle(self):
        pass
```

### @pytest.mark.slow

時間がかかるテスト用マーカー。通常のテスト実行時は除外可能。

```python
@pytest.mark.integration
@pytest.mark.slow
class TestLongBattle:
    def test_large_fleet_battle(self):
        # 200tick実行する長時間テスト
        pass
```

**除外方法**:
```bash
pytest -m "not slow"
```

---

## テストカバレッジ結果

### 現在のカバレッジ（Phase 2完了時点）

| カテゴリ | テストファイル | テスト数 | 成功率 |
|---------|--------------|---------|--------|
| 艦船 | test_ship.py | 28 | 100% ✅ |
| 艦隊 | test_fleet.py | 22 | 100% ✅ |
| 戦闘 | test_combat.py | 8 | 100% ✅ |
| 資源 | test_resources.py | 19 | 100% ✅ |
| **単体テスト合計** | - | **77** | **100% ✅** |

### カバレッジレポートの見方

```bash
# HTMLレポート生成
uv run pytest --cov=src --cov-report=html tests/

# htmlcov/index.html をブラウザで開く
# ファイルごとの行カバレッジが色分け表示される
# 緑: テストされた行
# 赤: テストされていない行
# 黄: 部分的にテストされた行
```

---

## トラブルシューティング

### テストが失敗する場合

#### 1. インポートエラー

```bash
ModuleNotFoundError: No module named 'src'
```

**解決方法**: プロジェクトルートディレクトリで実行していることを確認
```bash
cd /path/to/init-project-by-claude
uv run pytest tests/
```

#### 2. フィクスチャが見つからない

```bash
fixture 'sample_destroyer' not found
```

**解決方法**: `tests/fixtures/scenarios.py`が正しくインポートされているか確認
```python
# tests/conftest.py に以下を追加
pytest_plugins = ["tests.fixtures.scenarios"]
```

#### 3. 乱数結果が再現しない

```python
# reset_rng フィクスチャを使用
def test_random_behavior(reset_rng):
    # シード固定により再現可能
    pass
```

#### 4. テストが途中で止まる

```bash
# タイムアウト設定（pytest-timeout が必要）
uv add --dev pytest-timeout
pytest --timeout=10 tests/  # 各テスト10秒制限
```

---

## ベストプラクティス

### 1. テストは独立させる

各テストは他のテストに依存せず、独立して実行可能であるべきです。

```python
# 悪い例: グローバル変数に依存
global_fleet = Fleet(1, "Global", (0.0, 0.0, 0.0), ships=[])

def test_add_ship():
    global_fleet.ships.append(Ship(1, "destroyer", "DD-1"))  # 他のテストに影響

# 良い例: フィクスチャを使用
def test_add_ship(sample_fleet):
    initial_count = len(sample_fleet.ships)
    # テスト内で完結
```

### 2. 明確なアサーションメッセージ

```python
# 悪い例
assert result == 100

# 良い例
assert result == 100, f"Expected 100, but got {result}"
```

### 3. 決定論的乱数の使用

```python
def test_combat_result(reset_rng):
    """reset_rngで乱数をシード固定"""
    # 同じシードなら同じ結果
    pass
```

### 4. テストの命名規則

- `test_<対象>_<条件>` の形式
- 日本語docstringで詳細説明

```python
def test_ship_take_damage_when_hit(self, sample_destroyer):
    """艦船が命中時にダメージを受ける"""
    pass
```

---

## Phase 3以降のテスト計画

### 追加予定のテスト

1. **輸送システムテスト** (`test_transport.py`)
   - 輸送船団の移動テスト
   - 資源輸送時間の検証
   - 輸送リスク（海賊襲撃）のテスト

2. **星系間移動テスト** (`test_navigation.py`)
   - 艦隊の航路設定テスト
   - 移動中の燃料消費テスト
   - 複数星系間のジャンプテスト

3. **国家AIテスト** (`test_nation_ai.py`)
   - 国家の意思決定テスト
   - 艦隊建造・資源配分テスト
   - Ray Actorの並列実行テスト

4. **経済システムテスト** (`test_economy.py`)
   - GDP・産業指数の動的変化テスト
   - 資源循環システムテスト
   - 貿易システムテスト

---

**Last Updated**: 2025-10-20
**Status**: Phase 2 Complete - 77/77 Unit Tests Passing ✅
