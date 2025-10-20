# Omni Simulation Project

銀河規模の複雑系シミュレーションシステム - **Phase 2実装完了** ✅

[![Python](https://img.shields.io/badge/Python-3.12+-blue.svg)](https://www.python.org/downloads/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-77%2F77_passing-brightgreen.svg)](#テスト)

## 📋 目次

- [プロジェクト概要](#プロジェクト概要)
- [実装状況](#実装状況)
- [セットアップ](#セットアップ)
- [使い方](#使い方)
- [テスト](#テスト)
- [アーキテクチャ](#アーキテクチャ)
- [開発ガイド](#開発ガイド)

---

## プロジェクト概要

**Omni Simulation Project** は、銀河規模の複雑系シミュレーションシステムです。艦隊戦闘、国家経済、惑星開発、輸送網、人口動態を統合的にモデル化します。

### 特徴

- 🎯 **イベント駆動型シミュレーション**: 優先度付きキューによる高効率な時系列処理
- 🔬 **決定論的実行**: 同一シードで完全に再現可能な結果
- 🏗️ **階層型ハイブリッドアーキテクチャ**: 各層に最適な技術を組み合わせ
- ⚡ **System層パターン**: データとロジックの分離（段階的ECS移行）
- 🧪 **100%テストカバレッジ**: 単体テスト77件すべてパス

### プロジェクト情報

- **開発言語**: Python 3.12以上
- **パッケージマネージャー**: [uv](https://github.com/astral-sh/uv)
- **ライセンス**: MIT (Copyright 2025 Shimae)
- **開発言語（ドキュメント）**: 日本語

---

## 実装状況

### ✅ Phase 1: 基本戦闘システム（完了）

<details>
<summary>実装内容を表示</summary>

#### コアシステム
- **イベントスケジューラー**: heapq優先度キューによる時系列イベント管理
- **決定論的乱数生成**: NumPy PCG64による再現可能なシミュレーション

#### 戦闘システム
- **艦船システム**: 駆逐艦・巡洋艦・戦艦の3クラス
  - HP・攻撃力・防御力パラメータ
  - 速度パラメータ
- **艦隊システム**: 複数艦船の管理、状態遷移
- **索敵システム**: 距離判定による敵艦隊発見（DetectionEvent）
- **戦闘解決**: ターン制戦闘、ダメージ計算、艦船撃破判定（CombatEvent）

#### 実装ファイル
```
src/
├── core/scheduler.py       # イベントスケジューラー
├── core/simulation.py      # シミュレーションコントローラー
├── entities/ship.py        # 艦船エンティティ
├── entities/fleet.py       # 艦隊エンティティ
├── events/base.py          # イベント基底クラス
├── events/detection.py     # 索敵イベント
├── events/combat.py        # 戦闘イベント
├── combat/resolver.py      # 戦闘解決ロジック
├── utils/rng.py            # 決定論的乱数生成
├── utils/geometry.py       # 座標計算
└── config/constants.py     # 定数定義
```

</details>

### ✅ Phase 2: 資源管理システム（完了）

<details>
<summary>実装内容を表示</summary>

#### 資源システム
- **燃料・弾薬パラメータ**: 各艦船が燃料（kg）と弾薬（rounds）を保持
- **資源消費**:
  - 戦闘時の弾薬消費（攻撃ごとに艦種別消費量）
  - 毎tickの燃料消費（艦種別消費率）
- **戦闘継続判定**: 弾薬枯渇時の戦闘不能判定（can_shoot）

#### 惑星・補給システム
- **惑星システム**: 資源産出と備蓄管理（簡易版・固定値産出）
- **補給システム**: 惑星から艦隊への即座補給（ResupplyEvent）
- **星系管理**: 惑星・艦隊の所属管理（StarSystem）

#### System層導入
- **ResourceSystem**: 惑星の資源産出を一括管理
- **FleetSystem**: 艦隊の燃料消費を一括管理
- データ（Entities）とロジック（Systems）の分離パターン

#### 追加実装ファイル
```
src/
├── entities/planet.py      # 惑星エンティティ
├── entities/star_system.py # 星系エンティティ
├── systems/resource_system.py  # 資源産出System
├── systems/fleet_system.py     # 艦隊燃料消費System
└── events/resupply.py      # 補給イベント
```

</details>

### ✅ テストフレームワーク導入（完了）

<details>
<summary>テスト詳細を表示</summary>

#### テストカバレッジ
- **単体テスト**: 77/77 パス（100% ✅）
  - 艦船テスト: 28件（生成・資源・戦闘）
  - 艦隊テスト: 22件（管理・状態・資源集計）
  - 戦闘テスト: 8件（解決ロジック・ダメージ計算）
  - 資源テスト: 19件（惑星・System層・補給）

#### テスト構成
```
tests/
├── unit/                   # 単体テスト（77件）
│   ├── test_ship.py        # 艦船テスト
│   ├── test_fleet.py       # 艦隊テスト
│   ├── test_combat.py      # 戦闘テスト
│   └── test_resources.py  # 資源テスト
├── integration/            # 統合テスト（Phase 3対応予定）
│   ├── test_basic_combat.py
│   └── test_resource_flow.py
└── fixtures/
    └── scenarios.py        # 共通フィクスチャ
```

#### テスト実行
```bash
# 全単体テスト実行
uv run pytest tests/unit/ -v

# カバレッジレポート生成
uv run pytest tests/unit/ --cov=src --cov-report=html
```

</details>

### 🚧 Phase 3: 輸送・移動・国家AI（計画中）

<details>
<summary>実装予定内容</summary>

- **SimPy輸送システム**: 惑星間の資源輸送、輸送時間計算
- **星系間移動**: 艦隊の航路設定、移動中の燃料消費
- **国家AIシステム**: Ray Actorによる並列AI、艦隊建造・資源配分
- **経済システム**: GDP・産業指数の動的変化

</details>

### 📅 Phase 4: 最適化・大規模化（計画中）

<details>
<summary>実装予定内容</summary>

- **人口動態システム**: 惑星人口の増減、労働力管理
- **Numba最適化**: 艦隊戦闘の高速化（JITコンパイル）
- **NetworkX銀河グラフ**: 星系間ネットワークの管理
- **大規模シミュレーション**: 数百艦隊・数千艦船規模の同時処理

</details>

---

## セットアップ

### 必要環境

- **Python 3.12以上**
- **uv パッケージマネージャー**

### インストール手順

1. **uvのインストール**（未インストールの場合）

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

2. **プロジェクトのセットアップ**

```bash
cd init-project-by-claude
uv sync
```

3. **動作確認**

```bash
# Phase 1: 基本戦闘デモ
uv run python examples/basic_combat.py

# Phase 2: 資源管理デモ
uv run python examples/resource_combat.py
```

---

## 使い方

### デモシミュレーションの実行

#### Phase 1: 基本戦闘シミュレーション

```bash
uv run python examples/basic_combat.py
```

**シナリオ**: 5隻（戦艦2・巡洋艦1・駆逐艦2）vs 6隻（巡洋艦3・駆逐艦3）の艦隊戦闘

<details>
<summary>実行結果例</summary>

```
============================================================
Basic Combat Simulation - Phase 1 Demo
============================================================

Fleet(Imperial Fleet, System:Deep Space, Ships:5/5, State:searching)
  - Ship(Imperial-Sovereign, battleship, HP:500/500, Fuel:100000/100000kg, Ammo:1000/1000 rounds, ALIVE)
  - Ship(Imperial-Dominance, battleship, HP:500/500, Fuel:100000/100000kg, Ammo:1000/1000 rounds, ALIVE)
  ...

[Tick 1] Executing: DetectionEvent(tick=1, Imperial Fleet vs Rebel Fleet)
  Imperial Fleet detected Rebel Fleet! Initiating combat...

[Tick 2] Executing: CombatEvent(tick=2, Imperial Fleet vs Rebel Fleet)

=== COMBAT: Imperial Fleet vs Rebel Fleet ===
  Imperial-Sovereign -> Rebel-Liberty: 80 damage (HP: 200 -> 120)
  ...
  Total damage: Imperial Fleet dealt 100
  Total damage: Rebel Fleet dealt 75

*** Imperial Fleet VICTORY! ***

Simulation End (tick=8)
```

</details>

#### Phase 2: 資源管理を含む戦闘シミュレーション

```bash
uv run python examples/resource_combat.py
```

**シナリオ**: 弾薬25%で開始 → 補給 → 戦闘 → 追加補給

<details>
<summary>実行結果例</summary>

```
============================================================
Resource Combat Simulation - Phase 2 Demo
============================================================

[Tick 0] Executing: ResupplyEvent(tick=0, Homeworld -> Alliance Fleet)

=== RESUPPLY: Homeworld -> Alliance Fleet ===
  Fuel resupplied: 20kg
  Ammo resupplied: 300 rounds
  Planet(Homeworld, Fuel:201980kg, Ammo:19900 rounds)
  Fleet total fuel: 10000kg
  Fleet total ammo: 400 rounds

[Tick 2] Executing: CombatEvent(tick=2, Alliance Fleet vs Pirate Fleet)

=== COMBAT: Alliance Fleet vs Pirate Fleet ===
  Alliance-Alpha -> Pirate-Raider: 0 damage (HP: 200 -> 200, Ammo: 199/200)
  Alliance-Beta -> Pirate-Raider: 5 damage (HP: 200 -> 195, Ammo: 199/200)
  ...

*** Pirate Fleet VICTORY! ***
```

</details>

### デモファイルの詳細

| ファイル | 説明 | Phase |
|---------|------|-------|
| `examples/basic_combat.py` | 基本的な艦隊戦闘デモ | Phase 1 |
| `examples/resource_combat.py` | 資源管理・補給システムデモ | Phase 2 |

詳細は [examples/README.md](examples/README.md) を参照してください。

---

## テスト

### テスト実行コマンド

```bash
# 全単体テスト実行
uv run pytest tests/unit/ -v

# 特定ファイルのみ
uv run pytest tests/unit/test_ship.py -v

# カバレッジレポート生成
uv run pytest tests/unit/ --cov=src --cov-report=html

# マーカー指定
uv run pytest -m unit -v
```

### テスト結果

```
================================ test session starts =================================
tests/unit/test_ship.py::TestShipCreation::test_destroyer_creation PASSED      [  1%]
tests/unit/test_ship.py::TestShipCreation::test_cruiser_creation PASSED        [  2%]
...
tests/unit/test_resources.py::TestShipRefuelRearm::test_ship_rearm_max PASSED  [100%]

================================= 77 passed in 0.13s =================================
```

### テストカバレッジ

| カテゴリ | テスト数 | 成功率 |
|---------|---------|--------|
| 艦船テスト | 28 | 100% ✅ |
| 艦隊テスト | 22 | 100% ✅ |
| 戦闘テスト | 8 | 100% ✅ |
| 資源テスト | 19 | 100% ✅ |
| **総合** | **77** | **100% ✅** |

詳細は [tests/README.md](tests/README.md) を参照してください。

---

## アーキテクチャ

### 階層型ハイブリッドシミュレーション

本プロジェクトは**階層ごとに最適な技術を組み合わせる**設計です：

```
┌────────────────────────────┐
│ 銀河層 (静的Graph)          │ ← NetworkX (年単位更新) [Phase 4]
├────────────────────────────┤
│ 国家層 (並列Actor)          │ ← Ray.remote (週単位更新) [Phase 3]
├────────────────────────────┤
│ 艦隊層 (ECS更新)           │ ← NumPy/Numba (毎tick更新) [Phase 1-2]
├────────────────────────────┤
│ 輸送層 (離散イベント)       │ ← SimPy (イベント駆動) [Phase 3]
├────────────────────────────┤
│ 人口層 (統計近似)          │ ← NumPy配列 (月単位更新) [Phase 4]
└────────────────────────────┘
```

### イベント駆動アーキテクチャ

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
1. DetectionEvent (priority=0): 索敵判定
2. CombatEvent (priority=1): 戦闘ラウンド
3. ResupplyEvent (priority=2): 補給処理

### 戦闘メカニクス

#### ダメージ計算式

```python
hit_roll = rng.random()  # 0.0～1.0
if hit_roll > BASE_HIT_RATE:  # BASE_HIT_RATE = 0.7 (命中率75%)
    damage = 0  # 外れ
else:
    damage = max(attacker.attack - defender.defense, MIN_DAMAGE)  # MIN_DAMAGE = 5
```

#### 艦船クラスパラメータ

| クラス | HP  | 攻撃力 | 防御力 | 速度 | 最大燃料 | 最大弾薬 | 燃料消費率 | 弾薬消費 |
| ------ | --- | ------ | ------ | ---- | -------- | -------- | ---------- | -------- |
| 駆逐艦 | 100 | 20     | 10     | 5.0  | 5000kg   | 200発    | 10kg/tick  | 1発/攻撃 |
| 巡洋艦 | 200 | 40     | 20     | 3.0  | 20000kg  | 400発    | 20kg/tick  | 2発/攻撃 |
| 戦艦   | 500 | 100    | 50     | 2.0  | 100000kg | 1000発   | 50kg/tick  | 5発/攻撃 |

### ディレクトリ構造

```
init-project-by-claude/
├── src/                    # ソースコード
│   ├── core/              # シミュレーションコア
│   │   ├── scheduler.py   # イベントスケジューラー
│   │   └── simulation.py  # シミュレーションコントローラー
│   ├── entities/          # エンティティ
│   │   ├── ship.py        # 艦船
│   │   ├── fleet.py       # 艦隊
│   │   ├── planet.py      # 惑星
│   │   └── star_system.py # 星系
│   ├── systems/           # System層（ロジック分離）
│   │   ├── resource_system.py  # 資源産出管理
│   │   └── fleet_system.py     # 艦隊燃料消費管理
│   ├── events/            # イベント定義
│   │   ├── base.py        # イベント基底クラス
│   │   ├── detection.py   # 索敵イベント
│   │   ├── combat.py      # 戦闘イベント
│   │   └── resupply.py    # 補給イベント
│   ├── combat/            # 戦闘解決ロジック
│   │   └── resolver.py    # ダメージ計算・命中判定
│   ├── utils/             # ユーティリティ
│   │   ├── rng.py         # 決定論的乱数生成
│   │   └── geometry.py    # 座標計算
│   └── config/            # 定数・設定
│       └── constants.py   # 艦船パラメータ・定数定義
├── examples/              # サンプルシミュレーション
│   ├── basic_combat.py    # Phase 1デモ
│   └── resource_combat.py # Phase 2デモ
├── tests/                 # テストコード
│   ├── unit/             # 単体テスト（77件）
│   ├── integration/      # 統合テスト
│   └── fixtures/         # テストフィクスチャ
├── docs/                  # 設計ドキュメント
│   ├── 001-project-plan/ # プロジェクト計画
│   └── 002-world-concept/ # 世界設定
├── pyproject.toml         # 依存関係管理（uv）
├── pytest.ini             # pytest設定
├── CLAUDE.md              # Claude Code向けプロジェクト指示
└── README.md              # このファイル
```

---

## 開発ガイド

### コーディング規約

#### 1. 日本語コメント（必須）

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

#### 2. 型ヒント（必須）

すべての関数に型アノテーションを付与します。

```python
from typing import List, Tuple, Optional

def calculate_damage(attacker: Ship, defender: Ship) -> int:
    damage = max(attacker.attack - defender.defense, MIN_DAMAGE)
    return damage

Position = Tuple[float, float, float]  # 型エイリアス定義
```

#### 3. 決定論的乱数生成（必須）

すべての乱数生成は必ずSeededRNGを使用します。

```python
from src.utils.rng import get_rng

rng = get_rng()
hit_roll = rng.random()  # 0.0～1.0の乱数
target_index = rng.choice(targets)  # リストからランダム選択
```

**禁止**: `random.random()`, `random.choice()` など標準ライブラリの使用

#### 4. イベント駆動パターン

すべての状態変更はEventを通じて実行します。

```python
# 悪い例: 直接状態変更
fleet.state = FleetState.ENGAGED

# 良い例: イベント経由で変更
combat_event = CombatEvent(tick=current_tick, fleet_a=fleet_a, fleet_b=fleet_b)
scheduler.schedule(combat_event)
```

### 設計ドキュメント

詳細な設計資料は以下を参照してください：

- **[CLAUDE.md](CLAUDE.md)**: プロジェクト全体方針、Claude Code向け指示
- **[docs/001-project-plan/003-solution.md](docs/001-project-plan/003-solution.md)**: 階層的ハイブリッドアーキテクチャ詳細
- **[docs/002-world-concept/](docs/002-world-concept/)**: 世界設定（政治、経済、資源、輸送など）
- **[tests/README.md](tests/README.md)**: テスト構成とガイドライン
- **[examples/README.md](examples/README.md)**: デモシナリオの詳細

---

## 技術スタック

### Phase 1-2（現在使用中）

- **Python 3.12+**: メイン言語
- **NumPy 2.3.4+**: 決定論的乱数生成（PCG64）
- **uv**: パッケージ管理・仮想環境管理
- **pytest 8.4.2**: テストフレームワーク
- **pytest-cov 7.0.0**: カバレッジレポート生成

### Phase 3以降（導入予定）

- **Ray**: 分散処理・国家AIアクター
- **SimPy**: 輸送イベントシミュレーション
- **NetworkX**: 銀河ネットワークグラフ
- **Numba**: 艦隊戦闘最適化（JITコンパイル）
- **Pydantic**: データバリデーション

---

## ライセンス

MIT License

Copyright (c) 2025 Shimae

---

## 貢献

現在は個人プロジェクトとして開発中です。

---

## 更新履歴

- **2025-01-20**: Phase 2完了、テストフレームワーク導入（単体テスト100%達成）
- **2025-01-19**: Phase 2実装開始（資源管理システム）
- **2025-01-18**: Phase 1完了（基本戦闘システム）

---

**Last Updated**: 2025-01-20
**Status**: Phase 2 Complete ✅ | Tests: 77/77 Passing ✅
**Next Milestone**: Phase 3 - SimPy輸送システム、星系間移動、国家AI（Ray）
