# ドキュメント構成

Omni Simulation Projectの設計資料・世界設定ドキュメント集

---

## ドキュメント構成

```
docs/
├── 001-project-plan/       # プロジェクト計画・設計資料
│   ├── 001-project-plan.md # 艦隊戦闘システム設計
│   ├── 002-domain-plan.md  # 全ドメインパラメータ設計
│   └── 003-solution.md     # 階層別技術選定とスケーリング戦略
├── 002-world-concept/      # 世界設定・背景設定
│   ├── 001-world.md        # 世界観・銀河構造
│   ├── 002-political-structure.md  # 政治構造・国家体制
│   ├── 003-economy.md      # 経済システム
│   ├── 004-resources.md    # 資源システム
│   ├── 005-transportation.md  # 輸送システム
│   └── QA.md               # Q&A集
└── README.md               # このファイル
```

---

## 001-project-plan: プロジェクト計画

### 001-project-plan.md

**艦隊戦闘システム設計**

**主な内容**:
- シミュレーション目的と要件整理
- ソリューション選定（実装言語・アーキテクチャ）
- システム構造設計（SimulationController、FleetManager、EventScheduler）
- データ構造定義（Ship、Fleet、ShipGroup）
- イベント駆動アーキテクチャ（DetectionEvent、CombatEvent）
- 状態遷移モデル（SEARCHING → ENGAGED → DESTROYED）
- 戦闘解決ロジック（ダメージ計算・命中判定）

**対象Phase**: Phase 1（基本戦闘システム）

**Mermaid図**:
- システム構造フローチャート
- 状態遷移図
- イベントシーケンス図

**関連実装ファイル**:
- `src/core/scheduler.py`: EventScheduler実装
- `src/core/simulation.py`: SimulationController実装
- `src/entities/ship.py`, `src/entities/fleet.py`: エンティティ定義
- `src/events/detection.py`, `src/events/combat.py`: イベント定義

---

### 002-domain-plan.md

**全ドメインパラメータ設計**

**主な内容**:
- 艦隊層パラメータ（艦船クラス・HP・攻撃力・防御力）
- 資源層パラメータ（燃料・弾薬・産出量・備蓄）
- 惑星層パラメータ（資源産出・人口・開発度）
- 輸送層パラメータ（輸送時間・航路・コスト）
- 国家層パラメータ（GDP・産業・技術レベル）
- 経済層パラメータ（貿易・税収・支出）

**対象Phase**: Phase 1〜Phase 4（全フェーズ）

**パラメータ表**:
- 艦船クラス別パラメータ（駆逐艦・巡洋艦・戦艦）
- 資源産出レート（惑星タイプ別）
- 経済成長モデル（GDP・産業指数）

**関連実装ファイル**:
- `src/config/constants.py`: 定数・パラメータ定義
- `src/entities/planet.py`: 惑星パラメータ実装
- `src/entities/ship.py`: 艦船パラメータ実装

---

### 003-solution.md

**階層別技術選定とスケーリング戦略**

**主な内容**:
- 階層型ハイブリッドシミュレーションアーキテクチャ
- 層別技術選定（艦隊層・輸送層・国家層・銀河層・人口層）
- 艦隊層: イベント駆動 + ECS（NumPy/Numba）
- 輸送層: 離散イベントシミュレーション（SimPy）
- 国家層: 並列Actorモデル（Ray）
- 銀河層: 静的ネットワークグラフ（NetworkX）
- 人口層: 統計近似モデル（NumPy配列）
- スケーリング戦略（1万艦隊 vs 10万艦隊対応）

**対象Phase**: Phase 1〜Phase 4（全フェーズ）

**Mermaid図**:
- 階層型アーキテクチャ図
- 技術スタック構成図
- データフロー図

**関連設計判断**:
- Phase 1-2: イベント駆動 + OOPエンティティ（実装完了）
- Phase 3: SimPy輸送システム導入予定
- Phase 4: Ray並列化、NetworkX銀河グラフ導入予定

---

## 002-world-concept: 世界設定

### 001-world.md

**世界観・銀河構造**

**主な内容**:
- 銀河の概要（スケール・星系数・人口）
- FTL（超光速）航行システム
  - 通常空間航行（短距離移動）
  - ハイパースペース航法（長距離ジャンプ）
- 星系構造（恒星・惑星・資源分布）
- 初期設定（Phase 1: 1-10星系 → Phase 3以降: 数百星系）

**対象Phase**: Phase 3以降（星系間移動システム導入時に参照）

**世界設定要素**:
- 銀河規模: 直径10万光年
- 星系数: 500〜10,000星系（Phase 3拡張時）
- 政治体制: 多極化（複数国家）
- 技術レベル: 惑星間10〜30%の技術格差

**関連実装予定**:
- `src/entities/galaxy.py`: 銀河ネットワークグラフ（Phase 4）
- `src/systems/navigation_system.py`: 星系間航行システム（Phase 3）

---

### 002-political-structure.md

**政治構造・国家体制**

**主な内容**:
- 国家の種類（帝国・共和国・連邦・独立星系）
- 政治システム（中央集権 vs 分権）
- 外交システム（同盟・戦争・貿易協定）
- 国家AI意思決定モデル

**対象Phase**: Phase 3以降（国家AIシステム導入時に参照）

**国家パラメータ**:
- 領土（星系数・惑星数）
- 軍事力（艦隊規模・技術レベル）
- 経済力（GDP・産業指数）
- 外交関係（友好度・同盟関係）

**関連実装予定**:
- `src/layers/nation/nation.py`: 国家エンティティ（Phase 3）
- `src/layers/nation/ai.py`: 国家AI（Ray Actor）（Phase 3）
- `src/layers/nation/diplomacy.py`: 外交システム（Phase 4）

---

### 003-economy.md

**経済システム**

**主な内容**:
- GDP成長モデル（産業・技術・人口）
- 税収・支出システム
- 貿易システム（惑星間・星系間・国家間）
- 艦隊建造コスト
- 資源価格変動モデル

**対象Phase**: Phase 3以降（経済システム導入時に参照）

**経済パラメータ**:
- GDP成長率（年率1〜5%）
- 税率（10〜40%）
- 艦隊維持費（GDP比1〜10%）
- 貿易収入（GDP比5〜20%）

**関連実装予定**:
- `src/layers/nation/economy.py`: 経済システム（Phase 3）
- `src/systems/trade_system.py`: 貿易システム（Phase 4）

---

### 004-resources.md

**資源システム**

**主な内容**:
- 資源の種類（燃料・弾薬・鉱物・食料）
- 資源産出メカニクス（惑星タイプ別産出量）
- 資源備蓄管理（惑星・星系・国家レベル）
- 資源枯渇モデル
- 資源輸送システム（輸送時間・コスト）

**対象Phase**: Phase 2〜Phase 4

**実装状況**:
- **Phase 2完了**: 燃料・弾薬の基本システム実装済み
  - 惑星の固定値産出（枯渇なし）
  - 艦隊への即座補給（輸送時間なし）
- **Phase 3拡張予定**: 輸送時間・航路コスト導入
- **Phase 4拡張予定**: 資源枯渇モデル・動的産出量

**資源パラメータ（Phase 2実装済み）**:
- 燃料産出量: 500〜5000kg/tick（惑星タイプ別）
- 弾薬産出量: 50〜500発/tick（惑星タイプ別）
- 初期備蓄量: 産出量 × 100（固定）

**関連実装ファイル**:
- `src/entities/planet.py`: 惑星資源システム（Phase 2実装済み）
- `src/systems/resource_system.py`: 資源産出管理（Phase 2実装済み）
- `src/events/resupply.py`: 補給イベント（Phase 2実装済み）

---

### 005-transportation.md

**輸送システム**

**主な内容**:
- 輸送船団システム（民間・軍事）
- 航路計算（最短経路・安全経路）
- 輸送時間モデル（距離ベース）
- 輸送リスク（海賊・事故・戦争）
- 輸送コスト（燃料・人件費・保険）

**対象Phase**: Phase 3以降（SimPy輸送システム導入時に参照）

**輸送パラメータ**:
- 輸送速度: 1〜10光年/日（船団タイプ別）
- 輸送容量: 1000〜100000トン（船団タイプ別）
- 輸送コスト: 距離・容量・リスクに依存
- 輸送時間: 距離 / 速度（最短数時間〜最長数ヶ月）

**関連実装予定**:
- `src/layers/transport/transport_process.py`: SimPyプロセス（Phase 3）
- `src/layers/transport/convoy.py`: 輸送船団エンティティ（Phase 3）
- `src/layers/transport/routing.py`: 航路計算（Phase 3）

---

### QA.md

**よくある質問集**

**主な内容**:
- プロジェクト全般に関する質問
- 技術選定に関する質問
- 実装に関する質問
- 世界設定に関する質問
- パフォーマンスに関する質問

**例**:
- Q: なぜイベント駆動アーキテクチャを採用したのか？
- Q: なぜPythonを選択したのか？
- Q: 大規模化（10万艦隊）への対応方針は？
- Q: 決定論的乱数生成の仕組みは？
- Q: Phase 3以降のスケジュールは？

**関連ドキュメント**:
- 技術選定: `003-solution.md`
- 実装詳細: `/README.md`、`/CLAUDE.md`
- 世界設定: `002-world-concept/` 配下全ドキュメント

---

## ドキュメントの使い方

### 開発者向け

#### 新機能実装時

1. **要件確認**: `002-domain-plan.md`で該当ドメインのパラメータを確認
2. **技術選定**: `003-solution.md`で該当層の技術スタックを確認
3. **設計確認**: `001-project-plan.md`でアーキテクチャパターンを確認
4. **実装開始**: `/CLAUDE.md`のコーディング規約に従う

#### 世界設定参照時

1. **全体像**: `001-world.md`で銀河規模・星系構造を確認
2. **政治**: `002-political-structure.md`で国家体制・外交を確認
3. **経済**: `003-economy.md`でGDP・貿易システムを確認
4. **資源**: `004-resources.md`で資源産出・輸送を確認
5. **輸送**: `005-transportation.md`で航路・輸送時間を確認

### プランナー向け

#### シナリオ作成時

1. **世界設定**: `002-world-concept/`配下を順番に読む
2. **パラメータ**: `002-domain-plan.md`で各パラメータの範囲を確認
3. **実装状況**: `/README.md`でPhase別実装状況を確認
4. **サンプル**: `/examples/README.md`でデモプログラムを実行

#### バランス調整時

1. **パラメータ一覧**: `002-domain-plan.md`で全パラメータを確認
2. **定数定義**: `src/config/constants.py`で実装値を確認
3. **テスト実行**: `pytest tests/`で変更の影響を確認
4. **デモ実行**: `examples/`で実際の挙動を確認

---

## ドキュメントメンテナンス

### 更新方針

- **設計変更時**: 該当ドキュメントを即座に更新
- **新機能追加時**: `002-domain-plan.md`にパラメータを追加
- **Phase進行時**: 実装状況セクションを更新
- **質問追加時**: `QA.md`に追記

### ドキュメント間の整合性

- **プロジェクト計画**: `001-project-plan/` ↔ `/CLAUDE.md`
- **パラメータ設計**: `002-domain-plan.md` ↔ `src/config/constants.py`
- **技術選定**: `003-solution.md` ↔ `/README.md` アーキテクチャセクション
- **世界設定**: `002-world-concept/` ↔ `examples/`のシナリオ設定

### レビュー基準

- [ ] Mermaid図が正しく表示される
- [ ] 実装ファイルへのリンクが正しい
- [ ] パラメータ値が`src/config/constants.py`と一致
- [ ] Phase別実装状況が最新
- [ ] 日本語表記が統一されている

---

## 次のPhaseでの追加予定

### Phase 3ドキュメント

- **輸送システム詳細設計**: SimPyプロセス設計、航路計算アルゴリズム
- **星系間移動詳細設計**: 航行システム、燃料消費モデル
- **国家AI詳細設計**: Ray Actor設計、意思決定ツリー

### Phase 4ドキュメント

- **銀河ネットワーク詳細設計**: NetworkXグラフ構造、星系間関係
- **人口動態詳細設計**: 統計近似モデル、人口成長・移動
- **経済循環詳細設計**: GDP成長モデル、貿易フロー

---

## 参考資料

### プロジェクト関連

- **メインREADME**: `/README.md` - プロジェクト全体概要・実装状況
- **開発ガイド**: `/CLAUDE.md` - 開発者向けガイド・コーディング規約
- **テストドキュメント**: `/tests/README.md` - テストフレームワーク説明
- **デモドキュメント**: `/examples/README.md` - サンプルプログラム説明

### 実装関連

- **定数定義**: `src/config/constants.py` - 全パラメータ定義
- **エンティティ**: `src/entities/` - Ship、Fleet、Planet、StarSystemなど
- **イベント**: `src/events/` - DetectionEvent、CombatEvent、ResupplyEvent
- **System層**: `src/systems/` - ResourceSystem、FleetSystem

---

**Last Updated**: 2025-10-20
**Status**: Phase 2 Complete - Documentation Structure Established ✅
