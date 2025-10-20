# サンプルシミュレーション

Omni Simulation Projectのデモプログラム集

---

## 実行環境

### 必要な環境

- Python 3.12以上
- uv（パッケージマネージャー）
- プロジェクト依存関係のインストール済み

### 環境セットアップ

```bash
# プロジェクトルートディレクトリで実行
cd /path/to/init-project-by-claude

# 依存関係のインストール
uv sync
```

---

## サンプルプログラム一覧

### Phase 1: basic_combat.py

**Phase 1基本戦闘システムのデモ**

#### 実行方法

```bash
uv run python examples/basic_combat.py
```

#### 実行内容

- **シナリオ**: 帝国艦隊（巡洋艦5隻）vs 反乱軍艦隊（駆逐艦6隻）
- **シミュレーション期間**: 最大1000tick
- **乱数シード**: 42（決定論的・再現可能）

**主要な機能デモ**:
1. **イベントスケジューラー**: 時系列順のイベント実行
2. **索敵システム**: 艦隊間距離判定による敵艦隊発見
3. **戦闘解決**: ターン制戦闘、ダメージ計算、艦船撃破判定
4. **決定論的乱数**: シード固定による再現可能な結果

#### サンプル出力

```
=== Omni Simulation - Phase 1: Basic Combat ===
Seed: 42
Max ticks: 1000

[Tick 1] DETECTION: Imperial-Fleet detected Rebel-Fleet at distance 10.00
[Tick 1] COMBAT START: Imperial-Fleet vs Rebel-Fleet

[Tick 2] COMBAT: Imperial-Fleet attacks Rebel-Fleet
  Imperial-Ship-0 (cruiser) -> Target: Rebel-Ship-3 (destroyer) | Damage: 30 | HP: 70/100
  Imperial-Ship-1 (cruiser) -> Target: Rebel-Ship-1 (destroyer) | Damage: 30 | HP: 70/100
  ...

[Tick 2] COMBAT: Rebel-Fleet counter-attacks Imperial-Fleet
  Rebel-Ship-0 (destroyer) -> Target: Imperial-Ship-2 (cruiser) | Damage: 10 | HP: 190/200
  ...

[Tick 15] DESTROYED: Rebel-Ship-3 (destroyer) destroyed!
[Tick 20] DESTROYED: Imperial-Ship-1 (cruiser) destroyed!

...

=== SIMULATION COMPLETE ===
Final tick: 45

Imperial-Fleet:
  Status: DESTROYED
  Ships remaining: 0/5
  Total fuel: 0.00 kg
  Total ammo: 0 rounds

Rebel-Fleet:
  Status: SEARCHING
  Ships remaining: 2/6
  Total fuel: 10000.00 kg
  Total ammo: 320 rounds
  Alive ships:
    - Rebel-Ship-0 (destroyer): HP=85/100, Fuel=5000.00kg, Ammo=160
    - Rebel-Ship-4 (destroyer): HP=42/100, Fuel=5000.00kg, Ammo=160

Winner: Rebel-Fleet
```

#### コード構造

```python
# examples/basic_combat.py
from src.core.scheduler import EventScheduler
from src.core.simulation import SimulationController
from src.entities.ship import Ship
from src.entities.fleet import Fleet
from src.events.detection import DetectionEvent
from src.utils.rng import set_global_seed

# 決定論的乱数の初期化
set_global_seed(42)

# シミュレーションコントローラー作成
sim = SimulationController(seed=42, max_ticks=1000)

# 艦隊の作成
imperial_fleet = Fleet(1, "Imperial-Fleet", (0.0, 0.0, 0.0), ships=[])
for i in range(5):
    imperial_fleet.ships.append(Ship(i, "cruiser", f"Imperial-Ship-{i}"))

rebel_fleet = Fleet(2, "Rebel-Fleet", (10.0, 0.0, 0.0), ships=[])
for i in range(6):
    rebel_fleet.ships.append(Ship(i+10, "destroyer", f"Rebel-Ship-{i}"))

# 索敵イベントをスケジュール
detection = DetectionEvent(tick=1, fleet_a=imperial_fleet, fleet_b=rebel_fleet)
sim.scheduler.schedule(detection)

# シミュレーション実行
sim.run()
```

---

### Phase 2: resource_combat.py

**Phase 2資源管理システムのデモ**

#### 実行方法

```bash
uv run python examples/resource_combat.py
```

#### 実行内容

- **シナリオ**: 資源産出惑星を持つ星系での艦隊戦闘
- **シミュレーション期間**: 最大500tick
- **乱数シード**: 123（決定論的・再現可能）

**Phase 2追加機能デモ**:
1. **惑星システム**: 資源産出（燃料・弾薬）と備蓄管理
2. **資源消費**:
   - 戦闘時の弾薬消費（攻撃ごとに消費）
   - 毎tickの燃料消費（固定レート）
3. **補給システム**: 惑星から艦隊への資源補給（ResupplyEvent）
4. **戦闘継続判定**: 弾薬枯渇時の戦闘不能判定
5. **System層**: ResourceSystem、FleetSystemによる一括更新

#### サンプル出力

```
=== Omni Simulation - Phase 2: Resource Management ===
Seed: 123
Max ticks: 500

=== Initial Setup ===
Star System: Alpha-System
  Planet: Alpha-Prime
    Fuel production: 2000.00 kg/tick
    Ammo production: 200 rounds/tick
    Initial fuel stock: 200000.00 kg
    Initial ammo stock: 20000 rounds

Fleet-A (3 destroyers):
  Total fuel: 15000.00 kg
  Total ammo: 600 rounds

Fleet-B (2 cruisers):
  Total fuel: 40000.00 kg
  Total ammo: 800 rounds

[Tick 1] RESOURCE: Alpha-Prime produces 2000.00kg fuel, 200 ammo
[Tick 1] FLEET CONSUMPTION: Fleet-A consumes 30.00kg fuel (3 destroyers × 10kg)
[Tick 1] FLEET CONSUMPTION: Fleet-B consumes 40.00kg fuel (2 cruisers × 20kg)

[Tick 1] DETECTION: Fleet-A detected Fleet-B at distance 15.00
[Tick 1] COMBAT START: Fleet-A vs Fleet-B

[Tick 2] COMBAT: Fleet-A attacks Fleet-B (ammo consumed: 3)
  Destroyer-A-0 -> Cruiser-B-1 | Damage: 10 | HP: 190/200 | Ammo: 199/200
  Destroyer-A-1 -> Cruiser-B-0 | Damage: 10 | HP: 190/200 | Ammo: 199/200
  Destroyer-A-2 -> Cruiser-B-1 | Damage: 10 | HP: 180/200 | Ammo: 199/200

[Tick 2] COMBAT: Fleet-B counter-attacks Fleet-A (ammo consumed: 4)
  Cruiser-B-0 -> Destroyer-A-1 | Damage: 20 | HP: 80/100 | Ammo: 398/400
  Cruiser-B-1 -> Destroyer-A-0 | Damage: 20 | HP: 80/100 | Ammo: 398/400

[Tick 10] RESUPPLY: Fleet-A resupplied from Alpha-Prime
  Fuel transferred: 10000.00 kg
  Ammo transferred: 500 rounds
  Planet remaining: Fuel=180000.00kg, Ammo=18500

[Tick 25] LOW AMMO WARNING: Fleet-A ammo below 20%
  Current ammo: 120 rounds

[Tick 30] COMBAT END: Fleet-A out of ammo (cannot fight)

=== SIMULATION COMPLETE ===
Final tick: 30

Planet: Alpha-Prime
  Final fuel stock: 240000.00 kg (produced 60000.00 kg over 30 ticks)
  Final ammo stock: 24500 rounds (produced 6000, consumed 500 for resupply)

Fleet-A:
  Status: SEARCHING (out of ammo)
  Ships remaining: 2/3
  Total fuel: 12700.00 kg
  Total ammo: 0 rounds
  Can fight: False

Fleet-B:
  Status: SEARCHING
  Ships remaining: 2/2
  Total fuel: 38800.00 kg
  Total ammo: 720 rounds
  Can fight: True

Winner: Fleet-B (Fleet-A ran out of ammunition)
```

#### コード構造

```python
# examples/resource_combat.py
from src.core.scheduler import EventScheduler
from src.core.simulation import SimulationController
from src.entities.ship import Ship
from src.entities.fleet import Fleet
from src.entities.planet import Planet
from src.entities.star_system import StarSystem
from src.systems.resource_system import ResourceSystem
from src.systems.fleet_system import FleetSystem
from src.events.detection import DetectionEvent
from src.events.resupply import ResupplyEvent
from src.utils.rng import set_global_seed

# 決定論的乱数の初期化
set_global_seed(123)

# 惑星の作成
planet = Planet(
    planet_id=1,
    name="Alpha-Prime",
    position=(0.0, 0.0, 0.0),
    fuel_production=2000.0,  # 2000kg/tick
    ammo_production=200      # 200発/tick
)

# 星系の作成
star_system = StarSystem(1, "Alpha-System", (0.0, 0.0, 0.0))
star_system.add_planet(planet)

# 艦隊の作成
fleet_a = Fleet(1, "Fleet-A", (0.0, 0.0, 0.0), ships=[], current_system=star_system)
for i in range(3):
    fleet_a.ships.append(Ship(i, "destroyer", f"Destroyer-A-{i}"))

fleet_b = Fleet(2, "Fleet-B", (15.0, 0.0, 0.0), ships=[], current_system=star_system)
for i in range(2):
    fleet_b.ships.append(Ship(i+10, "cruiser", f"Cruiser-B-{i}"))

# System層の作成
resource_system = ResourceSystem()
fleet_system = FleetSystem()

# シミュレーション実行（メインループで毎tick以下を実行）
# 1. resource_system.update([planet], tick)  # 惑星の資源産出
# 2. fleet_system.update_consumption([fleet_a, fleet_b], tick)  # 艦隊の燃料消費
# 3. イベント処理（DetectionEvent, CombatEvent, ResupplyEvent）
```

---

## 実行時のヒント

### 1. 詳細ログを見る

```bash
# 標準出力をファイルに保存
uv run python examples/basic_combat.py > output.log

# lessコマンドで確認
less output.log
```

### 2. 異なるシナリオを試す

```python
# examples/basic_combat.py を編集

# シード値を変更（異なる戦闘結果）
set_global_seed(12345)

# 艦隊構成を変更（圧倒的戦力差）
for i in range(10):  # 巡洋艦10隻に増加
    imperial_fleet.ships.append(Ship(i, "cruiser", f"Imperial-Ship-{i}"))

# 戦艦を投入
imperial_fleet.ships.append(Ship(100, "battleship", "Flagship"))
```

### 3. シミュレーション時間を変更

```python
# 短時間実行（デバッグ用）
sim = SimulationController(seed=42, max_ticks=50)

# 長時間実行（消耗戦）
sim = SimulationController(seed=42, max_ticks=5000)
```

### 4. 補給タイミングを調整

```python
# examples/resource_combat.py を編集

# 早期補給（tick=5で補給）
resupply_event = ResupplyEvent(
    tick=5,
    fleet=fleet_a,
    planet=planet,
    fuel_amount=5000.0,
    ammo_amount=300
)
sim.scheduler.schedule(resupply_event)

# 複数回補給（tick=10, 20, 30で補給）
for tick in [10, 20, 30]:
    sim.scheduler.schedule(ResupplyEvent(
        tick=tick,
        fleet=fleet_a,
        planet=planet,
        fuel_amount=3000.0,
        ammo_amount=200
    ))
```

---

## カスタムシナリオの作成

### 基本テンプレート

```python
# examples/custom_scenario.py
from src.core.scheduler import EventScheduler
from src.core.simulation import SimulationController
from src.entities.ship import Ship
from src.entities.fleet import Fleet
from src.events.detection import DetectionEvent
from src.utils.rng import set_global_seed

def main():
    # 1. 乱数シード設定
    set_global_seed(999)

    # 2. シミュレーションコントローラー作成
    sim = SimulationController(seed=999, max_ticks=1000)

    # 3. 艦隊作成
    fleet_a = Fleet(1, "Custom-Fleet-A", (0.0, 0.0, 0.0), ships=[])
    # 艦船追加
    fleet_a.ships.append(Ship(1, "battleship", "Flagship"))
    fleet_a.ships.append(Ship(2, "cruiser", "Escort-1"))

    fleet_b = Fleet(2, "Custom-Fleet-B", (20.0, 0.0, 0.0), ships=[])
    fleet_b.ships.append(Ship(10, "cruiser", "Enemy-1"))

    # 4. 初期イベントスケジュール
    detection = DetectionEvent(tick=1, fleet_a=fleet_a, fleet_b=fleet_b)
    sim.scheduler.schedule(detection)

    # 5. シミュレーション実行
    sim.run()

if __name__ == "__main__":
    main()
```

### 大規模艦隊戦

```python
# examples/large_fleet_battle.py
def create_large_fleet(fleet_id: int, name: str, position, ship_count: int, ship_class: str):
    """大規模艦隊の生成"""
    fleet = Fleet(fleet_id, name, position, ships=[])
    for i in range(ship_count):
        fleet.ships.append(Ship(fleet_id * 1000 + i, ship_class, f"{name}-Ship-{i}"))
    return fleet

# 駆逐艦50隻 vs 巡洋艦30隻
fleet_a = create_large_fleet(1, "Destroyer-Fleet", (0.0, 0.0, 0.0), 50, "destroyer")
fleet_b = create_large_fleet(2, "Cruiser-Fleet", (50.0, 0.0, 0.0), 30, "cruiser")
```

### 混成艦隊戦

```python
# examples/mixed_fleet_battle.py
def create_balanced_fleet(fleet_id: int, name: str, position):
    """バランス型混成艦隊"""
    fleet = Fleet(fleet_id, name, position, ships=[])

    # 戦艦1隻（旗艦）
    fleet.ships.append(Ship(fleet_id * 100, "battleship", f"{name}-Flagship"))

    # 巡洋艦3隻（主力）
    for i in range(3):
        fleet.ships.append(Ship(fleet_id * 100 + 10 + i, "cruiser", f"{name}-Cruiser-{i}"))

    # 駆逐艦5隻（護衛）
    for i in range(5):
        fleet.ships.append(Ship(fleet_id * 100 + 20 + i, "destroyer", f"{name}-Destroyer-{i}"))

    return fleet

fleet_a = create_balanced_fleet(1, "Imperial", (0.0, 0.0, 0.0))
fleet_b = create_balanced_fleet(2, "Rebel", (30.0, 0.0, 0.0))
```

---

## パラメータリファレンス

### 艦船クラスパラメータ

| クラス | HP  | 攻撃力 | 防御力 | 速度 | 最大燃料 | 最大弾薬 | 燃料消費率 | 弾薬消費 |
| ------ | --- | ------ | ------ | ---- | -------- | -------- | ---------- | -------- |
| 駆逐艦 | 100 | 20     | 10     | 5.0  | 5000kg   | 200発    | 10kg/tick  | 1発/攻撃 |
| 巡洋艦 | 200 | 40     | 20     | 3.0  | 20000kg  | 400発    | 20kg/tick  | 2発/攻撃 |
| 戦艦   | 500 | 100    | 50     | 2.0  | 100000kg | 1000発   | 50kg/tick  | 5発/攻撃 |

### 戦闘メカニクス

```python
# ダメージ計算式
damage = max(attacker.attack - defender.defense, MIN_DAMAGE)  # MIN_DAMAGE = 5

# 命中判定
hit_roll = rng.random()  # 0.0～1.0
if hit_roll > BASE_HIT_RATE:  # BASE_HIT_RATE = 0.7 (命中率75%)
    damage = 0  # 外れ

# 弾薬消費
ship.ammo -= ship.ammo_per_shot  # 駆逐艦: 1発、巡洋艦: 2発、戦艦: 5発

# 燃料消費（毎tick）
ship.fuel -= ship.fuel_consumption_rate  # 駆逐艦: 10kg、巡洋艦: 20kg、戦艦: 50kg
```

### 索敵距離

```python
# src/config/constants.py
DETECTION_RANGE = 100.0  # 索敵範囲（デフォルト）

# 距離計算（ユークリッド距離）
distance = sqrt((x1-x2)**2 + (y1-y2)**2 + (z1-z2)**2)
```

---

## トラブルシューティング

### 実行時エラー

#### ModuleNotFoundError

```bash
ModuleNotFoundError: No module named 'src'
```

**解決方法**: プロジェクトルートディレクトリで実行
```bash
cd /path/to/init-project-by-claude
uv run python examples/basic_combat.py
```

#### 乱数結果が再現しない

```python
# 原因: シード設定が複数回実行されている
# 解決方法: set_global_seed() を最初に1回だけ呼び出す

# 悪い例
set_global_seed(42)
# ... 処理 ...
set_global_seed(42)  # 再設定すると結果が変わる

# 良い例
set_global_seed(42)
# ... すべての処理 ...
```

#### 戦闘が終了しない

```python
# 原因: MAX_TICKSが大きすぎる、または戦闘終了条件が満たされない
# 解決方法: MAX_TICKSを調整、または戦闘ログを確認

# 短時間実行
sim = SimulationController(seed=42, max_ticks=100)
```

---

## 次のステップ

### Phase 3で追加予定のデモ

1. **輸送システムデモ** (`transport_demo.py`)
   - 輸送船団による惑星間資源輸送
   - 輸送時間と航路計算
   - 輸送リスク（海賊襲撃）のシミュレーション

2. **星系間移動デモ** (`interstellar_demo.py`)
   - 艦隊の星系間移動
   - 移動中の燃料消費
   - 複数星系での連続戦闘

3. **国家AIデモ** (`nation_ai_demo.py`)
   - 複数国家の並列動作（Ray Actor）
   - 艦隊建造・資源配分の自律的意思決定
   - 国家間の外交・戦争

4. **経済システムデモ** (`economy_demo.py`)
   - GDP・産業指数の動的変化
   - 資源循環システム
   - 貿易と経済成長

---

## 参考資料

### プロジェクトドキュメント

- **メインREADME**: `/README.md` - プロジェクト全体概要
- **CLAUDE.md**: `/CLAUDE.md` - 開発者向けガイド
- **テストドキュメント**: `/tests/README.md` - テストフレームワーク説明

### 設計資料

- **001-project-plan.md**: プロジェクト計画・Phase定義
- **002-domain-plan.md**: 全ドメインパラメータ設計
- **003-solution.md**: 階層別技術選定とスケーリング戦略

---

**Last Updated**: 2025-10-20
**Status**: Phase 2 Complete - Resource Management Demos Available ✅
