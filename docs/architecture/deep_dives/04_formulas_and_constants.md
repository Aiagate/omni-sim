# 深掘り分析: 数式・定数の管理戦略

## 1. マジックナンバーの現状

現在のシミュレーションロジックには、バランス調整に直結する係数がコード内に直接記述されています。

- **人口消費率**: `pop.count * 0.00001` ([economy.rs](file:///home/dorothy/repos/omni-sim/src/systems/economy.rs))
- **貿易コスト**: `distance * 0.05` ([trade.rs](file:///home/dorothy/repos/omni-sim/src/components/economy.rs))
- **研究コスト上昇率**: `100.0 * 1.5.powi(level)` ([research.rs](file:///home/dorothy/repos/omni-sim/src/systems/research.rs))

## 2. 課題: 静的数値による柔軟性の欠如

これらの数値がハードコードされているため、以下の拡張が困難です。

- **技術による改善**: 「技術レベルが上がると人口あたりの食料消費が 10% 減る」といった効果を実装する場合、定数ではなく計算式に国家/惑星の状態を反映させる必要があります。
- **難易度設定**: シミュレーション全体の産出量を一括で変更するような調整が分散しています。

## 3. 改善済みの管理構造: BalanceConfig への集約

[src/config.rs](file:///home/dorothy/repos/omni-sim/src/config.rs) に `BalanceConfig` 構造体を導入し、物理定数やゲームバランスに関わる全ての係数を集約しました。

### 構成例
```rust
pub struct BalanceConfig {
    pub food_consumption_per_pop: f64,
    pub energy_consumption_per_pop_base: f64,
    pub trade_cost_per_ly: f64,
    pub research_cost_base: f64,
    pub tech_bonus_per_level: f64,
    // ...
}
```

### 利用方法
各システムは `Res<SimulationConfig>` を通じてこれらの値を参照します。また、コンポーネントのメソッド（例: `cost_for_next_level`）も `BalanceConfig` を受け取るように設計されています。

## 4. 将来の展望: 動的補正（Effective Values）への移行

現在の構成により、集約管理は達成されました。次は、値を単なる定数として扱うのではなく、国家の状態や環境に応じた「実効値」として計算する仕組みへと拡張します。

### 実裝イメージ: Formula トレイト
将来的に特定の係数が複雑な計算を必要とする場合、以下のような抽象化を検討します。

```rust
pub trait SimulationFormula {
    fn calculate(&self, context: &FormulaContext) -> f64;
}
```

これにより、「技術レベルが上がると人口あたりの食料消費が 10% 減る」といった効果を、システムコードを汚さずに `BalanceConfig` 側のロジックとして統合できるようになります。
