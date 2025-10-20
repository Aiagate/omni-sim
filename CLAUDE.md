# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

**Omni Simulation Project** は、銀河規模の複雑系シミュレーションシステムです。艦隊戦闘、国家経済、惑星開発、輸送網、人口動態を統合的にモデル化します。

**開発言語**: Python（大規模シミュレーション向けの階層型並列処理に最適）

**ライセンス**: MIT (Copyright 2025 Shimae)

**プロジェクト状態**: 設計完了・実装準備段階

**重要**: 開発チームは日本人で構成されているため、**すべてのドキュメント・コメント・会話は日本語で行うこと**。これは認知負荷軽減のための最重要指示です。

---

## アーキテクチャ哲学

### 階層型ハイブリッドシミュレーション

本プロジェクトは**単一の処理モデルではなく、階層ごとに最適な技術を組み合わせる**設計です：

```
┌────────────────────────────┐
│ 銀河層 (静的Graph)          │ ← NetworkX (年単位更新)
├────────────────────────────┤
│ 国家層 (並列Actor)          │ ← Ray.remote (週単位更新)
├────────────────────────────┤
│ 艦隊層 (ECS更新)           │ ← NumPy/Numba (毎tick更新)
├────────────────────────────┤
│ 輸送層 (離散イベント)       │ ← SimPy (イベント駆動)
├────────────────────────────┤
│ 人口層 (統計近似)          │ ← NumPy配列 (月単位更新)
└────────────────────────────┘
```

**設計原則**: 「粗粒度はActor、細粒度はECS、さらに細かい層は統計近似」

### 技術スタック決定根拠

| 階層   | 技術選定                | 理由                                     |
| ------ | ----------------------- | ---------------------------------------- |
| 銀河層 | NetworkX, NumPy         | 航路と星系間距離をキャッシュ保持         |
| 国家層 | Ray (`@ray.remote`)     | 国家ごとにAIを独立スレッド化で並列処理   |
| 艦隊層 | NumPy + Numba           | 大量の艦隊を一括ベクトル更新（SIMD対応） |
| 輸送層 | SimPy                   | 到着時のみ更新でCPU削減                  |
| 人口層 | NumPy配列 or pandas     | 統計的集約更新で計算量削減               |

### 時間解像度の分離（重要）

**全層を毎tick更新しない** - 更新頻度を分離することで劇的な高速化を実現：

| 層     | 時間解像度   | 更新間隔      | 例                     |
| ------ | ------------ | ------------- | ---------------------- |
| 銀河層 | 年単位       | 1000tickに1回 | 航路変動や星系滅亡など |
| 国家層 | 週単位       | 7tickに1回    | 政策変更、外交         |
| 艦隊層 | 日単位       | 毎tick        | 航行・戦闘             |
| 輸送層 | イベント単位 | イベント時    | 輸送完了時のみ         |
| 人口層 | 月単位       | 30tickに1回   | 成長率・労働力変化     |

→ この設計により、**1日で数十年分**のシミュレーションが可能

---

## ドメインモデル（階層別）

### 銀河層（Galaxy Layer）
- **データ構造**: Adjacency matrix / CSR graph
- **主要パラメータ**: 星系座標(x,y,z), 接続星系リスト, 環境パラメータ(密度/放射線/不安定性)
- **更新頻度**: 年単位（1000tickに1回）
- **実装**: NetworkX + NumPyでキャッシュ化

### 国家層（Nation Layer）
- **データ構造**: Dataclass or Pydantic model
- **主要パラメータ**:
  - 政治: 政体、安定度、正統性
  - 経済: GDP、産業指数、技術水準、資源在庫
  - 軍事: 艦隊数、動員率、ドクトリン
  - 外交: 関係値、同盟、条約
- **更新頻度**: 週単位（7tickに1回）
- **実装**: Ray Actorで国家ごとに並列AI実行

### 艦隊層（Fleet Layer）
- **データ構造**: NumPy structured array（SIMD対応）
- **主要パラメータ**:
  - 戦闘: 攻撃力、防御力、耐久、機動性
  - 状態: 燃料、士気、練度、損傷度
  - 位置: 星系ID、座標(x,y,z)
- **更新頻度**: 毎tick（日単位）
- **実装**: NumPy/Numbaで一括ベクトル更新、ECSパターン

### 輸送層（Transport Layer）
- **データ構造**: PriorityQueue (SimPyイベントキュー)
- **主要パラメータ**: 出発地、目的地、積載物種別、到着予定時刻、リスク値
- **更新頻度**: イベント発生時のみ
- **実装**: SimPyの離散イベントシミュレーション

### 人口層（Population Layer）
- **データ構造**: Dense array (NumPy)
- **主要パラメータ**: 総人口、職業分布(労働者/科学者/軍人)、幸福度、教育水準、出生率/死亡率
- **更新頻度**: 月単位（30tickに1回）
- **実装**: 統計的集約更新、都市単位で代表値

---

## 開発フェーズ

### Phase 1: 艦隊戦闘コア実装
- **目標**: 2艦隊の索敵・移動・交戦シミュレーション
- **実装範囲**:
  - NumPy配列ベースの艦隊エンティティ
  - 離散イベント駆動の戦闘解決
  - 疑似乱数シード固定による再現性確保
- **成功基準**: 同一シードで同一結果が得られる2艦隊戦闘

### Phase 2: 資源・輸送レイヤー統合
- **目標**: 艦隊の継戦能力を資源で制約
- **実装範囲**:
  - SimPyによる輸送イベントシステム
  - 燃料・弾薬消費と補給線
  - 惑星-艦隊間の資源フロー
- **成功基準**: 補給線が切れた艦隊が作戦継続不能になる

### Phase 3: 国家AIと経済システム
- **目標**: 国家レベルの意思決定と経済循環
- **実装範囲**:
  - Ray Actorによる並列国家AI
  - GDP・産業・技術の動的変化
  - 外交関係と同盟システム
- **成功基準**: 複数国家が並列で自律行動し、経済が循環する

### Phase 4: 人口動態と最適化
- **目標**: 大規模シミュレーションの実現
- **実装範囲**:
  - 人口増減の統計モデル
  - Numba JITによる高速化
  - 並列実行の最適化
- **成功基準**: 1000星系・100国家・10000艦隊を1日でシミュレート可能

---

## プロジェクト構造

```
omni-sim/
├── docs/                         # 設計ドキュメント（日本語）
│   └── 001-project-plan/
│       ├── 001-project-plan.md   # 艦隊戦闘設計
│       ├── 002-domain-plan.md    # ドメインパラメータ設計
│       └── 003-solution.md       # 階層別技術選定
├── src/
│   ├── layers/                   # 階層別実装
│   │   ├── galaxy/               # 銀河層（NetworkX）
│   │   ├── nation/               # 国家層（Ray Actor）
│   │   ├── fleet/                # 艦隊層（NumPy ECS）
│   │   ├── transport/            # 輸送層（SimPy）
│   │   └── population/           # 人口層（統計モデル）
│   ├── core/                     # シミュレーションエンジン
│   │   ├── scheduler.py          # 時間ステップ管理
│   │   ├── coordinator.py        # 階層間調整
│   │   └── state_manager.py     # 状態保存/復元
│   ├── ai/                       # 国家AI
│   │   ├── decision_engine.py
│   │   └── strategies/
│   ├── utils/
│   │   ├── rng.py               # シード固定RNG
│   │   ├── geometry.py          # 座標計算
│   │   └── logger.py            # ログシステム
│   └── config/                  # 設定ファイル
├── tests/
│   ├── unit/
│   ├── integration/
│   └── fixtures/
├── examples/                    # サンプルシミュレーション
│   └── basic_combat.py
├── pyproject.toml              # 依存関係管理
├── requirements.txt
└── README.md
```

---

## コーディング規約とパターン

### 1. NumPy配列による一括更新（艦隊層）

```python
# 悪い例: ループで個別更新
for fleet in fleets:
    fleet.position += fleet.velocity * dt

# 良い例: ベクトル化
positions += velocities * dt  # NumPy配列演算
```

### 2. Ray Actorによる国家並列化

```python
@ray.remote
class NationAI:
    def decide_policy(self, state):
        # 重い思考処理を並列実行
        return policy

# 使用例
nations = [NationAI.remote() for _ in range(100)]
policies = ray.get([n.decide_policy.remote(state) for n in nations])
```

### 3. SimPyによるイベント駆動輸送

```python
def transport_process(env, cargo, origin, dest):
    yield env.timeout(travel_time)  # 到着まで待機
    dest.receive(cargo)  # 到着時のみ処理

env = simpy.Environment()
env.process(transport_process(env, cargo, A, B))
```

### 4. 決定論的乱数生成（再現性確保）

```python
# 必ずシード固定して初期化
rng = np.random.Generator(np.random.PCG64(seed=42))
hit_chance = rng.random()  # 同一シードで同一結果
```

### 5. 時間解像度の分離

```python
class Scheduler:
    def tick(self, current_tick):
        if current_tick % 1 == 0:      # 毎tick
            self.update_fleets()
        if current_tick % 7 == 0:      # 週単位
            self.update_nations()
        if current_tick % 30 == 0:     # 月単位
            self.update_population()
        if current_tick % 1000 == 0:   # 年単位
            self.update_galaxy()
```

### 6. データクラス定義（Pydantic推奨）

```python
from pydantic import BaseModel

class FleetState(BaseModel):
    fleet_id: int
    position: tuple[float, float, float]
    fuel: float
    morale: float
    # 型安全性と自動検証
```

---

## 開発コマンド

```bash
# 依存関係インストール
pip install -r requirements.txt
# または
poetry install

# テスト実行
pytest tests/

# 単一テスト実行
pytest tests/unit/test_fleet.py::test_combat_resolution -v

# カバレッジ付きテスト
pytest --cov=src tests/

# Numba JITコンパイル確認
python -c "from src.layers.fleet.combat import resolve_combat; resolve_combat.inspect_types()"

# Ray クラスタ起動（開発用）
ray start --head --port=6379

# シミュレーション実行例
python examples/basic_combat.py --seed 42 --duration 1000

# ログ解析
python scripts/analyze_log.py logs/simulation_20250119.log
```

---

## 重要: 日本語ドキュメント運用

**開発チームは日本人で構成されているため、すべての会話・ドキュメント・コメントは日本語で記述すること。**

- **設計文書**: `docs/001-project-plan/` 内はすべて日本語
- **コードコメント**: 日本語で記述（特に複雑なロジック）
- **Commit message**: 日本語推奨
- **Pull Request**: 日本語で記述

### 主要な用語対応
- 艦隊 = Fleet
- 国家 = Nation
- 惑星 = Planet
- 輸送 = Transport
- 人口 = Population
- 離散イベント = Discrete Event
- 並列処理 = Parallel Processing

---

## 次のClaude Instanceへの指示

### 現在の状態
- **Phase 0完了**: プロジェクト初期化・設計完了
- **Phase 1準備中**: 艦隊戦闘コアの実装待ち
- **Branch**: `feature/init-project`（ドキュメント整備）

### Phase 1開始時の最初のタスク

1. Python環境セットアップ（pyproject.toml作成）
2. `src/core/scheduler.py` 実装（時間ステップ管理）
3. `src/layers/fleet/` 実装（NumPy配列ベース）
4. `src/utils/rng.py` 実装（シード固定RNG）
5. 2艦隊戦闘の統合テスト作成
6. `examples/basic_combat.py` サンプル実装

### テスト戦略

- **単体テスト**: 各階層の更新ロジック（pytest）
- **統合テスト**: 2艦隊戦闘シナリオ（複数シードで再現性確認）
- **性能テスト**: 1000艦隊での実行時間計測
- **決定論テスト**: 同一シード→同一結果の保証

### Phase 1成功基準

- 2艦隊が索敵・移動・交戦できる
- 戦闘結果が艦隊パラメータに基づいて計算される
- 同一シードで同一結果が再現される
- NumPy配列演算で高速動作する

---

## 設計判断の根拠

### なぜPython？
- NumPy/Numbaによる高速配列演算
- Ray/SimPyなど強力なシミュレーションライブラリ
- 科学計算エコシステムが充実
- プロトタイピング速度

### なぜ階層型ハイブリッド？
- 単一モデルでは全階層を効率的に扱えない
- 国家AI（重い処理）と艦隊移動（軽量・大量）は異なる最適化が必要
- 更新頻度の分離で劇的な高速化

### なぜ時間解像度を分離？
- 銀河構造は毎tick変化しない
- 人口も毎日変化しない
- 必要な層だけを高頻度更新することでCPU使用量を1/10以下に削減

---

## 参考ドキュメント

場所: `/home/dorothy/repos/omni-sim-project/init-project-by-claude/docs/001-project-plan/`

- **001-project-plan.md**: 艦隊戦闘システム設計（日本語）
- **002-domain-plan.md**: 全ドメインパラメータ設計（日本語）
- **003-solution.md**: 階層別技術選定とスケーリング戦略（日本語）

すべてMermaid図付きで詳細に記述されています。
