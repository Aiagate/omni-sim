# Omni Simulation Project

宇宙艦隊戦闘シミュレーションプロジェクト - **Phase 2実装完了** ✅

## プロジェクト概要

本プロジェクトは、大規模な宇宙文明シミュレーションを階層的ハイブリッドアーキテクチャで実装することを目指しています。Phase 2では、燃料・弾薬の資源管理システムと補給システムを実装しました。

## 現在の実装状況

### Phase 1: 基本戦闘システム ✅ 完了

- **イベント駆動型シミュレーション**: 優先度付きキューによる時系列イベント処理
- **艦隊・艦船システム**: 駆逐艦・巡洋艦・戦艦の3クラス、HP・攻撃力・防御力パラメータ
- **索敵システム**: 艦隊間の距離判定による敵艦隊発見
- **戦闘解決システム**: ターン制戦闘、ダメージ計算、艦船撃破判定
- **決定論的乱数生成**: シード固定による再現可能なシミュレーション
- **コンソール出力**: 戦闘経過と最終結果の詳細表示

### Phase 2: 資源管理システム ✅ 完了

- **燃料・弾薬パラメータ**: 各艦船が燃料（kg）と弾薬（発）を保持
- **資源消費システム**:
  - 戦闘時の弾薬消費（攻撃ごとに消費）
  - 毎tickの燃料消費（固定レート）
- **惑星システム**: 資源産出と備蓄管理（簡易版）
- **補給システム**: 惑星から艦隊への即座補給
- **戦闘継続判定**: 弾薬枯渇時の戦闘不能判定
- **System層導入**: データとロジックの分離（ECS準備）
  - ResourceSystem: 惑星の資源産出管理
  - FleetSystem: 艦隊の燃料消費管理

### 今後の実装予定

- **Phase 3**: SimPy輸送システム、星系間移動、国家AI（Ray）、経済システム
- **Phase 4**: 人口動態、Numba最適化、大規模シミュレーション

## セットアップ

### 必要環境

- Python 3.12以上
- [uv](https://github.com/astral-sh/uv) パッケージマネージャー

### インストール

1. uvのインストール（未インストールの場合）:

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

2. プロジェクトのセットアップ:

```bash
cd init-project-by-claude
uv sync
```

## 実行方法

### Phase 1: 基本的な戦闘シミュレーション

```bash
uv run python examples/basic_combat.py
```

5隻 vs 6隻の艦隊戦闘を実行します。

### Phase 2: 資源管理を含む戦闘シミュレーション

```bash
uv run python examples/resource_combat.py
```

燃料・弾薬の消費と補給システムを実証します：
- 弾薬25%の状態で開始
- 戦闘前後に補給イベントが発生
- 燃料・弾薬の消費を確認しながら戦闘継続

### 実行例

```
============================================================
Basic Combat Simulation - Phase 1 Demo
============================================================

============================================================
Simulation Start (seed=42)
============================================================

Fleet(id=1, name='Imperial Fleet', ships=5, state=SEARCHING)
  - Ship(id=1, class=battleship, 'Imperial-Sovereign', HP=500/500)
  - Ship(id=2, class=battleship, 'Imperial-Dominance', HP=500/500)
  - Ship(id=3, class=cruiser, 'Imperial-Guardian', HP=200/200)
  - Ship(id=4, class=destroyer, 'Imperial-Swift', HP=100/100)
  - Ship(id=5, class=destroyer, 'Imperial-Falcon', HP=100/100)

Fleet(id=2, name='Rebel Fleet', ships=6, state=SEARCHING)
  - Ship(id=101, class=cruiser, 'Rebel-Liberty', HP=200/200)
  - Ship(id=102, class=cruiser, 'Rebel-Freedom', HP=200/200)
  - Ship(id=103, class=cruiser, 'Rebel-Hope', HP=200/200)
  - Ship(id=104, class=destroyer, 'Rebel-Phoenix', HP=100/100)
  - Ship(id=105, class=destroyer, 'Rebel-Aurora', HP=100/100)
  - Ship(id=106, class=destroyer, 'Rebel-Comet', HP=100/100)

  Imperial Fleet detected Rebel Fleet! Initiating combat...

=== COMBAT: Imperial Fleet vs Rebel Fleet ===
  Total damage: Imperial Fleet dealt 200
  Total damage: Rebel Fleet dealt 40
  ...

*** Imperial Fleet VICTORY! ***

============================================================
Simulation End (tick=8)
============================================================
```

## アーキテクチャ

### イベント駆動システム

シミュレーションはイベントスケジューラーによって時系列順に実行されます：

1. **DetectionEvent** (優先度: 0, 高): 艦隊間の索敵判定
2. **CombatEvent** (優先度: 1, 中): 戦闘ラウンドの解決

### 戦闘メカニクス

#### ダメージ計算式

```python
hit_roll = random(0.0, 1.0)
if hit_roll > BASE_HIT_RATE:  # 命中率75%
    damage = 0
else:
    damage = max(attacker.attack - defender.defense, MIN_DAMAGE)
```

#### 艦船クラスパラメータ

| クラス | HP  | 攻撃力 | 防御力 | 速度 |
| ------ | --- | ------ | ------ | ---- |
| 駆逐艦 | 100 | 20     | 10     | 5.0  |
| 巡洋艦 | 200 | 40     | 20     | 3.0  |
| 戦艦   | 500 | 100    | 50     | 2.0  |

### ディレクトリ構造

```
init-project-by-claude/
├── src/
│   ├── core/           # シミュレーションコア（スケジューラー、コントローラー）
│   ├── entities/       # エンティティ（艦船、艦隊）
│   ├── events/         # イベント（索敵、戦闘）
│   ├── combat/         # 戦闘解決ロジック
│   ├── utils/          # ユーティリティ（乱数生成、幾何計算）
│   └── config/         # 定数・設定
├── examples/           # サンプルシミュレーション
└── docs/               # 設計資料
    ├── 001-project-plan/      # プロジェクト計画
    └── 002-world-concept/     # 世界設定
```

## 設計ドキュメント

詳細な設計資料は以下を参照してください：

- **CLAUDE.md**: プロジェクト全体方針、アーキテクチャ概要
- **docs/001-project-plan/003-solution.md**: 階層的ハイブリッドアーキテクチャ詳細
- **docs/002-world-concept/**: 世界設定（政治、経済、資源、輸送など）

## 技術スタック

### Phase 1（現在）
- **Python 3.12**: メイン言語
- **NumPy**: 決定論的乱数生成（PCG64）
- **uv**: パッケージ管理

### Phase 2以降（予定）
- **Ray**: 分散処理・国家AIアクター
- **SimPy**: 輸送イベントシミュレーション
- **NetworkX**: 銀河ネットワークグラフ
- **Numba**: 艦隊戦闘最適化

## 開発ガイドライン

### コーディング規約

- **日本語コメント**: すべてのdocstring、コメントは日本語で記述
- **型ヒント**: すべての関数に型アノテーションを付与
- **決定論性**: 乱数は必ずSeededRNGクラスを使用
- **イベント駆動**: 状態変更はすべてEventを通じて実行

### テスト方針

現在はPhase 1のため手動テストのみ。Phase 2以降でpytestベースの自動テストを導入予定。

## ライセンス

未定

## 貢献

現在は個人プロジェクトとして開発中です。

---

**Last Updated**: 2025-10-19
**Status**: Phase 1 Complete ✅
