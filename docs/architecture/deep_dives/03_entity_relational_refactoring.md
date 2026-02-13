# 深掘り分析: エンティティ関係の再定義 (国家への集約)

## 1. 現状の不正規化

現在のコードベースでは、本来「国家（Nation）」に帰属すべきデータが「惑星（Planet）」に直接付与されている傾向があります。

- **技術 (TechnologyState)**: [src/components/technology.rs](file:///home/dorothy/repos/omni-sim/src/components/technology.rs) は Planet エンティティに付与されています。
- **歴史/メモリ (NationalMemory)**: [src/components/national_ai.rs](file:///home/dorothy/repos/omni-sim/src/components/national_ai.rs) は Nation にありますが、貿易システムなどでは Planet を起点に Nation を逆引きする処理が頻発しています。

## 2. 発生する問題

### 多惑星国家の表現
将来的に1つの国家が2つ以上の惑星を統治する場合、現在の「惑星ごとに技術レベルがある」構造では以下の不整合が生じます。
- 首都惑星で開発した新技術が、同じ国家の開拓惑星ですぐに使えない。
- 国家全体の研究リソースが分散し、管理が複雑になる。

### 逆引きコスト
前の分析でも触れた通り、惑星を起点としたシステムの多くで「この惑星を統治しているのは誰か？」という検索が発生し、パフォーマンスのボトルネックになっています。

## 3. 改善案: Nation Centric な構造への移行

### データの移動
- `TechnologyState` を `Planet` から `Nation` エンティティへ移動します。
- 経済システム ([src/systems/economy.rs](file:///home/dorothy/repos/omni-sim/src/systems/economy.rs)) は、惑星の `BelongsToNation` （または既存の Nation から Planet へのリンク）を辿って、その国家の技術ボーナスを参照するようにします。

### 高速なリンクの導入
Bevy の `Parent / Child` 関係をより積極的に活用するか、`BelongsToNation(Entity)` というコンポーネントを全惑星に付与し、検索なしに即座に対象の国家にアクセス可能にします。

## 4. 期待される効果
- **論理的な一貫性**: 国家として1つの技術ツリーを共有し、外交関係も国家単位で完結します。
- **計算効率**: $O(N)$ の検索処理が $O(1)$ のコンポーネント取得に置き換わります。
