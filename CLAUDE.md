# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

**Omni Simulation Project** は、銀河規模の複雑系シミュレーションシステムです。艦隊戦闘、国家経済、惑星開発、輸送網、人口動態を統合的にモデル化します。

**開発言語**: Python 3.12以上

**パッケージマネージャー**: uv

**ライセンス**: MIT (Copyright 2025 Shimae)

**プロジェクト状態**: **Phase 2実装完了 + テストフレームワーク導入完了** ✅

**重要**: 開発チームは日本人で構成されているため、**すべてのドキュメント・コメント・会話は日本語で行うこと**。これは認知負荷軽減のための最重要指示です。

---

## 現在の実装状況（Phase 2完了）

### Phase 1: 基本戦闘システム ✅ 完了

- **イベント駆動型シミュレーション**: 優先度付きキューによる時系列イベント処理
- **艦隊・艦船システム**: 駆逐艦・巡洋艦・戦艦の3クラス
- **索敵システム**: 艦隊間の距離判定による敵艦隊発見
- **戦闘解決システム**: ターン制戦闘、ダメージ計算、艦船撃破判定
- **決定論的乱数生成**: シード固定による再現可能なシミュレーション（NumPy PCG64）
- **コンソール出力**: 戦闘経過と最終結果の詳細表示

**実装ファイル**:
- `src/core/scheduler.py`: イベントスケジューラー（heapq優先度キュー）
- `src/core/simulation.py`: シミュレーションコントローラー
- `src/entities/ship.py`: 艦船エンティティ（HP・攻撃力・防御力）
- `src/entities/fleet.py`: 艦隊エンティティ（複数艦船の管理）
- `src/events/base.py`: イベント基底クラス
- `src/events/detection.py`: 索敵イベント
- `src/events/combat.py`: 戦闘イベント
- `src/combat/resolver.py`: 戦闘解決ロジック（ダメージ計算）
- `src/utils/rng.py`: 決定論的乱数生成（PCG64）
- `src/utils/geometry.py`: 座標計算（距離判定）
- `src/config/constants.py`: 艦船パラメータ・定数定義

### Phase 2: 資源管理システム ✅ 完了

- **燃料・弾薬パラメータ**: 各艦船が燃料（kg）と弾薬（rounds）を保持
- **資源消費システム**:
  - 戦闘時の弾薬消費（攻撃ごとに消費）
  - 毎tickの燃料消費（固定レート）
- **惑星システム**: 資源産出と備蓄管理（簡易版・固定値産出）
- **補給システム**: 惑星から艦隊への即座補給（ResupplyEvent）
- **戦闘継続判定**: 弾薬枯渇時の戦闘不能判定
- **System層導入**: データとロジックの分離（ECS準備）
  - ResourceSystem: 惑星の資源産出管理
  - FleetSystem: 艦隊の燃料消費管理

**追加実装ファイル**:
- `src/entities/planet.py`: 惑星エンティティ（資源産出・備蓄）
- `src/entities/star_system.py`: 星系エンティティ（惑星・艦隊の管理）
- `src/systems/resource_system.py`: 資源産出System
- `src/systems/fleet_system.py`: 艦隊燃料消費System
- `src/events/resupply.py`: 補給イベント

**Phase 2での主要な設計判断**:
1. **即座補給**: 輸送時間なし（Phase 3でSimPy輸送システム追加予定）
2. **固定値産出**: 惑星資源は枯渇しない（Phase 3で動的資源管理追加予定）
3. **System層パターン**: OOPエンティティ + Systemロジックのハイブリッド（段階的ECS移行）

---

## アーキテクチャ哲学

### 階層型ハイブリッドシミュレーション

本プロジェクトは**単一の処理モデルではなく、階層ごとに最適な技術を組み合わせる**設計です：

```
┌────────────────────────────┐
│ 銀河層 (静的Graph)          │ ← NetworkX (年単位更新) [Phase 4]
├────────────────────────────┤
│ 国家層 (並列Actor)          │ ← Ray.remote (週単位更新) [Phase 3]
├────────────────────────────┤
│ 艦隊層 (ECS更新)           │ ← NumPy/Numba (毎tick更新) [Phase 1-2実装済]
├────────────────────────────┤
│ 輸送層 (離散イベント)       │ ← SimPy (イベント駆動) [Phase 3]
├────────────────────────────┤
│ 人口層 (統計近似)          │ ← NumPy配列 (月単位更新) [Phase 4]
└────────────────────────────┘
```

**現在の状態**: 艦隊層（Phase 1-2）は実装完了。イベント駆動システムとSystem層パターンが導入済み。

### イベント駆動アーキテクチャ（Phase 1-2実装済）

シミュレーションはイベントスケジューラーによって時系列順に実行されます：

```python
# src/core/scheduler.py
class EventScheduler:
    def __init__(self):
        self.event_queue = []  # heapq優先度キュー

    def schedule(self, event: Event):
        heapq.heappush(self.event_queue, (event.tick, event.priority, event))

    def run_until(self, max_tick: int):
        while self.event_queue and current_tick < max_tick:
            tick, priority, event = heapq.heappop(self.event_queue)
            event.execute(self)
```

**イベント優先度**:
1. **DetectionEvent** (priority=0): 索敵判定
2. **CombatEvent** (priority=1): 戦闘ラウンド
3. **ResupplyEvent** (priority=2): 補給処理

### System層パターン（Phase 2導入）

データ（Entities）とロジック（Systems）を分離するパターンを導入。完全なECSへの段階的移行を目指します。

```python
# src/systems/resource_system.py
class ResourceSystem:
    """惑星の資源産出を一括管理"""
    def update(self, planets: List[Planet], tick: int):
        for planet in planets:
            planet.produce_resources()

# src/systems/fleet_system.py
class FleetSystem:
    """艦隊の燃料消費を一括管理"""
    def update_consumption(self, fleets: List[Fleet], tick: int):
        for fleet in fleets:
            for ship in fleet.get_alive_ships():
                ship.consume_fuel(ship.fuel_consumption_rate)
```

**System統合箇所**: `src/core/simulation.py`のメインループで毎tick呼び出し

---

## ディレクトリ構造

```
init-project-by-claude/
├── src/
│   ├── core/              # シミュレーションコア
│   │   ├── scheduler.py   # イベントスケジューラー（heapq優先度キュー）
│   │   └── simulation.py  # シミュレーションコントローラー
│   ├── entities/          # エンティティ（OOPデータクラス）
│   │   ├── ship.py        # 艦船（HP・攻撃力・防御力・燃料・弾薬）
│   │   ├── fleet.py       # 艦隊（艦船リスト・位置・状態）
│   │   ├── planet.py      # 惑星（資源産出・備蓄）[Phase 2]
│   │   └── star_system.py # 星系（惑星・艦隊の管理）[Phase 2]
│   ├── systems/           # System層（ロジック分離）[Phase 2]
│   │   ├── resource_system.py  # 資源産出管理
│   │   └── fleet_system.py     # 艦隊燃料消費管理
│   ├── events/            # イベント定義
│   │   ├── base.py        # イベント基底クラス
│   │   ├── detection.py   # 索敵イベント
│   │   ├── combat.py      # 戦闘イベント
│   │   └── resupply.py    # 補給イベント [Phase 2]
│   ├── combat/            # 戦闘解決ロジック
│   │   └── resolver.py    # ダメージ計算・命中判定
│   ├── utils/             # ユーティリティ
│   │   ├── rng.py         # 決定論的乱数生成（NumPy PCG64）
│   │   └── geometry.py    # 座標計算（距離判定）
│   └── config/            # 定数・設定
│       └── constants.py   # 艦船パラメータ・定数定義
├── examples/              # サンプルシミュレーション
│   ├── basic_combat.py    # Phase 1: 基本戦闘デモ
│   ├── resource_combat.py # Phase 2: 資源管理デモ
│   └── README.md          # デモプログラム説明書
├── tests/                 # テストコード（pytest）
│   ├── unit/              # 単体テスト（77テスト・100%合格）
│   ├── integration/       # 統合テスト
│   ├── fixtures/          # テストフィクスチャ
│   └── README.md          # テスト実行ガイド
├── docs/                  # 設計ドキュメント（日本語）
│   ├── 001-project-plan/  # プロジェクト計画
│   ├── 002-world-concept/ # 世界設定
│   └── README.md          # ドキュメント構成ガイド
├── pytest.ini             # pytest設定
├── pyproject.toml         # 依存関係管理（uv）
└── README.md              # プロジェクト概要
```

---

## 開発コマンド

### 環境セットアップ

```bash
# uvのインストール（未インストールの場合）
curl -LsSf https://astral.sh/uv/install.sh | sh

# プロジェクトのセットアップ
cd init-project-by-claude
uv sync
```

### シミュレーション実行

```bash
# Phase 1: 基本戦闘シミュレーション（5隻 vs 6隻）
uv run python examples/basic_combat.py

# Phase 2: 資源管理を含む戦闘シミュレーション
uv run python examples/resource_combat.py
```

### テスト実行

```bash
# 全単体テスト実行（77テスト）
uv run pytest tests/unit/

# 詳細出力付き実行
uv run pytest tests/unit/ -v

# 特定のテストファイルを実行
uv run pytest tests/unit/test_combat.py

# 特定のテストクラスを実行
uv run pytest tests/unit/test_ship.py::TestShipCreation

# カバレッジレポート生成
uv run pytest --cov=src tests/

# HTMLカバレッジレポート生成
uv run pytest --cov=src --cov-report=html tests/

# マーカー別実行
uv run pytest -m unit          # 単体テストのみ
uv run pytest -m integration   # 統合テストのみ
uv run pytest -m "not slow"    # 時間がかかるテストを除外
```

---

## コーディング規約とパターン

### 1. 決定論的乱数生成（必須）

**すべての乱数生成は必ずSeededRNGを使用すること**。同一シードで同一結果を保証します。

```python
# src/utils/rng.py
from src.utils.rng import get_rng

rng = get_rng()
hit_roll = rng.random()  # 0.0～1.0の乱数
target_index = rng.choice(targets)  # リストからランダム選択
```

**禁止**: `random.random()`, `random.choice()` など標準ライブラリの使用

### 2. イベント駆動パターン

すべての状態変更はEventを通じて実行します。直接エンティティの状態を変更しないこと。

```python
# 悪い例: 直接状態変更
fleet.state = FleetState.ENGAGED

# 良い例: イベント経由で変更
combat_event = CombatEvent(tick=current_tick, fleet_a=fleet_a, fleet_b=fleet_b)
scheduler.schedule(combat_event)
```

### 3. System層パターン（Phase 2導入）

ロジックはSystemクラスに集約し、Entityはデータ保持に専念させます。

```python
# Entity: データ保持
class Planet:
    def __init__(self, fuel_production: float):
        self.fuel_production = fuel_production
        self.fuel_stock = fuel_production * 100.0

    def produce_resources(self):
        self.fuel_stock += self.fuel_production

# System: ロジック実行
class ResourceSystem:
    def update(self, planets: List[Planet], tick: int):
        for planet in planets:
            planet.produce_resources()  # 全惑星を一括更新
```

### 4. 型ヒント（必須）

すべての関数に型アノテーションを付与します。

```python
from typing import List, Tuple, Optional

def calculate_damage(attacker: Ship, defender: Ship) -> int:
    damage = max(attacker.attack - defender.defense, MIN_DAMAGE)
    return damage

Position = Tuple[float, float, float]  # 型エイリアス定義
```

### 5. 日本語コメント（必須）

すべてのdocstring、コメントは日本語で記述します。

```python
def resolve_combat(attacker: Fleet, defender: Fleet) -> Tuple[int, int]:
    """
    戦闘を解決

    Args:
        attacker: 攻撃側艦隊
        defender: 防御側艦隊

    Returns:
        (攻撃側が与えたダメージ, 防御側が与えたダメージ)
    """
    # 攻撃側の攻撃処理
    damage_to_defender = self._attack_fleet(attacker, defender)
    return (damage_to_defender, damage_to_attacker)
```

### 6. 定数管理

すべての定数は`src/config/constants.py`で一元管理します。

```python
# src/config/constants.py
SHIP_CLASSES = {
    "destroyer": {
        "hp": 100,
        "attack": 20,
        "defense": 10,
        "speed": 5.0,
        "max_fuel": 5000.0,
        "max_ammo": 200,
        "fuel_consumption_rate": 10.0,
        "ammo_per_shot": 1,
    },
    # ...
}

BASE_HIT_RATE = 0.7  # 命中率75%
MIN_DAMAGE = 5       # 最低ダメージ保証
FUEL_UNIT = "kg"     # 燃料単位
AMMO_UNIT = "rounds" # 弾薬単位
```

---

## 主要なデータフロー

### 戦闘フロー（Phase 1-2）

```
1. DetectionEvent (tick=1)
   └─> 艦隊間距離判定
       └─> 索敵成功 → CombatEvent をスケジュール

2. CombatEvent (tick=2)
   └─> CombatResolver.resolve_combat()
       ├─> 攻撃側: 各艦船が弾薬チェック → 弾薬消費 → ダメージ計算
       ├─> 防御側: 反撃処理
       └─> 艦船撃破判定
       └─> 戦闘継続判定（弾薬枯渇/全滅チェック）
           └─> 継続なら次のCombatEventをスケジュール

3. ResupplyEvent (tick=5)
   └─> 惑星から艦隊へ資源転送
       ├─> 惑星備蓄から消費
       └─> 艦船へ補給
```

### 資源フロー（Phase 2）

```
毎tick実行:
1. ResourceSystem.update()
   └─> 全惑星が資源産出
       └─> planet.fuel_stock += planet.fuel_production

2. FleetSystem.update_consumption()
   └─> 全艦隊の燃料消費
       └─> ship.fuel -= ship.fuel_consumption_rate

イベント時:
3. ResupplyEvent.execute()
   └─> 惑星 → 艦隊への資源転送
       ├─> planet.consume_fuel(amount)
       └─> ship.refuel(amount)
```

---

## 戦闘メカニクス詳細

### ダメージ計算式

```python
# 命中判定
hit_roll = rng.random()  # 0.0～1.0
if hit_roll > BASE_HIT_RATE:  # BASE_HIT_RATE = 0.7 (命中率75%)
    damage = 0  # 外れ
else:
    damage = max(attacker.attack - defender.defense, MIN_DAMAGE)  # MIN_DAMAGE = 5
```

### 艦船クラスパラメータ

| クラス | HP  | 攻撃力 | 防御力 | 速度 | 最大燃料 | 最大弾薬 | 燃料消費率 | 弾薬消費 |
| ------ | --- | ------ | ------ | ---- | -------- | -------- | ---------- | -------- |
| 駆逐艦 | 100 | 20     | 10     | 5.0  | 5000kg   | 200発    | 10kg/tick  | 1発/攻撃 |
| 巡洋艦 | 200 | 40     | 20     | 3.0  | 20000kg  | 400発    | 20kg/tick  | 2発/攻撃 |
| 戦艦   | 500 | 100    | 50     | 2.0  | 100000kg | 1000発   | 50kg/tick  | 5発/攻撃 |

### 戦闘終了条件

```python
# src/core/simulation.py
def _should_end_battle(fleet_a: Fleet, fleet_b: Fleet) -> bool:
    # 条件1: どちらかが全滅
    if fleet_a.is_destroyed() or fleet_b.is_destroyed():
        return True

    # 条件2: Phase 2追加 - どちらかが弾薬枯渇で戦闘不能
    if not fleet_a.can_fight() or not fleet_b.can_fight():
        return True

    return False
```

---

## 次のClaude Instanceへの指示

### 現在の状態（2025-10-20時点）

- **Phase 1完了**: 基本戦闘システム実装済み
- **Phase 2完了**: 資源管理システム実装済み
- **テストフレームワーク完了**: pytest導入、77/77単体テスト合格（100%）
- **ドキュメント完備**: README.md × 4（メイン・tests・examples・docs）
- **Branch**: `feature/init-project` → `main` へのマージ待ち
- **実行可能**: `examples/basic_combat.py`, `examples/resource_combat.py`
- **テスト実行可能**: `uv run pytest tests/unit/` で全77テスト実行

### Phase 3開始時の最初のタスク

#### 1. SimPy輸送システム実装

```python
# src/layers/transport/ 新規作成
# - transport_process.py: SimPyプロセス定義
# - convoy.py: 輸送船団エンティティ
# - routing.py: 航路計算

# 実装目標:
# - 惑星→惑星の資源輸送
# - 輸送時間の計算（距離ベース）
# - 輸送リスク（海賊襲撃など）
```

#### 2. 星系間移動システム

```python
# src/entities/fleet.py 拡張
# - set_course(destination: StarSystem): 航路設定
# - update_travel(dt: float): 移動進捗更新
# - 星系間ジャンプの実装

# 実装目標:
# - 艦隊が星系間を移動できる
# - 移動中の燃料消費
# - 航路上での遭遇イベント
```

#### 3. 国家AIと経済システム（Ray導入）

```python
# src/layers/nation/ 新規作成
# - nation.py: 国家エンティティ（GDP・産業・技術）
# - ai.py: Ray Actorによる国家AI
# - economy.py: 経済循環システム

# 実装目標:
# - 国家ごとに独立したAIスレッド
# - 艦隊建造・資源管理・外交の意思決定
# - Ray.remote での並列実行
```

### Phase 3成功基準

- [ ] 輸送船団が惑星間を移動し、資源を運搬できる
- [ ] 艦隊が星系間を移動し、移動中に燃料を消費する
- [ ] 国家AIが並列で動作し、艦隊建造・資源配分を自律的に決定する
- [ ] 経済システムが循環し、GDP・産業指数が動的に変化する
- [ ] 統合テストが全て合格する

### テスト戦略（既存テストの拡張）

**現在のテスト状況**:
- ✅ 単体テスト: 77/77合格（100%）
- ✅ テストフィクスチャ: 完備
- ✅ pytest設定: 完了
- ⏳ 統合テスト: 一部実装（Phase 3で拡張）

**Phase 3で追加するテスト**:
```bash
# 新規テストファイル作成予定
tests/unit/test_transport.py      # 輸送システムテスト
tests/unit/test_navigation.py     # 星系間移動テスト
tests/unit/test_nation_ai.py      # 国家AIテスト
tests/integration/test_transport_flow.py  # 輸送フロー統合テスト

# テスト実行（Phase 3）
uv run pytest tests/unit/test_transport.py -v
uv run pytest tests/integration/ -v
uv run pytest --cov=src --cov-report=html tests/
```

---

## 技術スタック

### 現在使用中（Phase 1-2）

- **Python 3.12+**: メイン言語
- **NumPy 2.3.4+**: 決定論的乱数生成（PCG64）
- **uv**: パッケージ管理・仮想環境管理
- **pytest 8.4.2**: テストフレームワーク
- **pytest-cov 7.0.0**: カバレッジ測定

### Phase 3以降で導入予定

- **Ray**: 分散処理・国家AIアクター
- **SimPy**: 輸送イベントシミュレーション
- **NetworkX**: 銀河ネットワークグラフ
- **Numba**: 艦隊戦闘最適化（JITコンパイル）
- **Pydantic**: データバリデーション

---

## 設計判断の根拠

### なぜイベント駆動？

- 時系列シミュレーションの標準パターン
- 状態変更の追跡が容易（デバッグ・リプレイ）
- 優先度制御による柔軟な実行順序

### なぜSystem層を導入？

- 完全なECSへの段階的移行
- データとロジックの分離により、将来的なNumba最適化が容易
- 複数エンティティの一括更新でパフォーマンス向上

### なぜ決定論的乱数？

- シミュレーション結果の再現性確保
- デバッグの容易性
- テストの信頼性向上

### なぜ段階的実装（Phase分割）？

- 各フェーズで動作検証しながら進める
- 設計ミスの早期発見
- チーム全体の理解促進

---

## 重要: 日本語ドキュメント運用

**開発チームは日本人で構成されているため、すべての会話・ドキュメント・コメントは日本語で記述すること。**

### コーディング時の言語使用

- **設計文書**: `docs/` 内はすべて日本語
- **コードコメント**: 日本語で記述（特に複雑なロジック）
- **docstring**: 日本語で記述
- **Commit message**: 日本語推奨
- **Pull Request**: 日本語で記述

### 主要な用語対応

| 日本語   | 英語（コード内）     |
| -------- | -------------------- |
| 艦隊     | Fleet                |
| 艦船     | Ship                 |
| 惑星     | Planet               |
| 星系     | StarSystem           |
| 戦闘     | Combat               |
| 補給     | Resupply             |
| 索敵     | Detection            |
| 燃料     | Fuel                 |
| 弾薬     | Ammo/Ammunition      |
| 資源     | Resource             |
| 国家     | Nation               |
| 輸送     | Transport            |
| 離散イベント | Discrete Event   |
| 並列処理 | Parallel Processing  |

---

## 参考ドキュメント

### 設計資料（日本語）

場所: `/home/dorothy/repos/omni-sim-project/init-project-by-claude/docs/`

- **001-project-plan/001-project-plan.md**: 艦隊戦闘システム設計
- **001-project-plan/002-domain-plan.md**: 全ドメインパラメータ設計
- **001-project-plan/003-solution.md**: 階層別技術選定とスケーリング戦略
- **002-world-concept/**: 世界設定（政治・経済・資源・輸送など）

すべてMermaid図付きで詳細に記述されています。

### README.md関連

- **メインREADME.md**: Phase 1-2の実装詳細、実行方法、アーキテクチャ解説、テストカバレッジ
- **tests/README.md**: テストフレームワーク完全ガイド、実行方法、フィクスチャ説明
- **examples/README.md**: サンプルプログラム説明、カスタムシナリオ作成方法
- **docs/README.md**: ドキュメント構成ガイド、開発者向け・プランナー向け使い方

---

## トラブルシューティング

### シミュレーションが停止しない

```python
# 原因: 戦闘終了条件が満たされない
# 解決: MAX_TICKSを確認、または戦闘終了条件をログ出力

# src/config/constants.py
MAX_TICKS = 1000  # 無限ループ防止
```

### 乱数結果が再現しない

```python
# 原因: グローバルRNGが初期化されていない
# 解決: SimulationController初期化時にシード設定を確認

from src.utils.rng import set_global_seed
set_global_seed(42)  # 必ず最初に呼び出す
```

### 弾薬が補給されない

```python
# 原因: 惑星備蓄が不足
# 解決: planet.fuel_stock / planet.ammo_stock を確認

print(f"Planet stock: Fuel={planet.fuel_stock}kg, Ammo={planet.ammo_stock} rounds")
```

---

---

## テストAPI仕様（重要）

**Phase 2でのテスト実装により判明した正確なAPI仕様**:

### Ship初期化
```python
# 正しい順序: ship_id, ship_class, name
Ship(1, "destroyer", "Destroyer-1")

# 間違い（古い仕様）: ship_id, name, ship_class, position
# Ship(1, "Destroyer-1", "destroyer", (0.0, 0.0, 0.0))  # エラー
```

### Fleet初期化
```python
# shipsパラメータは必須
Fleet(1, "Fleet-A", (0.0, 0.0, 0.0), ships=[])

# 間違い: shipsパラメータなし
# Fleet(1, "Fleet-A", (0.0, 0.0, 0.0))  # エラー
```

### Fleetメソッド
```python
# 利用可能なメソッド
fleet.get_alive_ships()  # 生存艦船リスト取得
fleet.is_destroyed()     # 全滅判定
fleet.total_fuel()       # 総燃料
fleet.total_ammo()       # 総弾薬
fleet.can_fight()        # 戦闘可能判定

# 利用不可（存在しないメソッド）
# fleet.add_ship(ship)   # エラー（直接ships.append()を使用）
# fleet.count_alive()    # エラー（len(get_alive_ships())を使用）
```

### StarSystem属性
```python
# 正しい属性名
star_system.fleets_present  # 存在する艦隊リスト

# 間違い（古い属性名）
# star_system.fleets  # エラー
```

### CombatResolver戻り値
```python
# _calculate_damage() は int を返す（tupleではない）
damage = resolver._calculate_damage(attacker, defender)  # int

# 間違い（古い仕様）
# damage, hit = resolver._calculate_damage(attacker, defender)  # エラー
```

### Ship能力判定
```python
# 正しいメソッド名
ship.can_shoot()  # 射撃可能判定（弾薬チェック）

# 間違い（存在しないメソッド）
# ship.can_fire()  # エラー
```

---

**Last Updated**: 2025-10-20
**Status**: Phase 2 Complete + Test Framework Complete ✅
**Test Status**: 77/77 Unit Tests Passing (100%)
**Next Milestone**: Phase 3 - SimPy輸送システム、星系間移動、国家AI（Ray）
