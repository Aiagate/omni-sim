# 提案 12: モンテカルロ統計分析

## 概要

同じ設定で**シードだけ変えてN回シミュレーション**を実行し、統計的な傾向を分析する「モンテカルロモード」。ゲームバランスの科学的な検証ツール。

---

## 設計

### 1. CLI インターフェース

```bash
# 基本的な使い方
cargo run -- --monte-carlo 1000

# 出力形式を指定
cargo run -- --monte-carlo 1000 --output stats.json

# 基本設定と組み合わせ
cargo run -- --monte-carlo 500 --max-ticks 200 --scenario scenarios/default.toml
```

### 2. 統計出力

```
=== Deep Juno モンテカルロ分析 (1000 回実行) ===

■ 最終人口ランキング（1位の確率）
  地球連邦:         43.2%  (平均: 15.3M, σ=2.1M)
  シリウス帝国:     38.5%  (平均: 12.8M, σ=3.4M)
  ケンタウリ共和国:  18.3%  (平均:  4.2M, σ=1.8M)

■ 戦争統計
  戦争が1回以上発生:  91.3%
  平均戦争回数:       2.4回
  最初の戦争の平均 Tick: 22.3 (σ=8.1)
  最も頻繁に宣戦する国: シリウス帝国 (78.4%)

■ 技術統計
  最初にLv10到達する国:
    地球連邦: 52.1%  (平均 Tick: 187)
    シリウス: 31.2%  (平均 Tick: 215)
    ケンタウリ: 16.7% (平均 Tick: 243)

■ 生存率
  100 Tick 時点の生存率:
    地球連邦: 100%
    シリウス帝国: 97.2%
    ケンタウリ共和国: 84.5%

■ 結論
  このシナリオは地球連邦がやや有利（43%で1位）。
  ケンタウリは最初の戦争で敗北しやすい（15.5%で滅亡）。
```

### 3. 実装

```rust
fn run_monte_carlo(config: &MonteCarloConfig) {
    let results: Vec<SimulationResult> = (0..config.iterations)
        .into_par_iter()  // rayon で並列化
        .map(|i| {
            let seed = config.base_seed.wrapping_add(i as u64);
            run_single_simulation(seed, config.max_ticks, &config.scenario)
        })
        .collect();

    let analysis = analyze_results(&results);
    output_analysis(&analysis, &config.output_format);
}
```

### 4. 追加クレート

| クレート | 用途 |
|---------|------|
| `rayon` | 並列化による高速実行 |

---

## 期待される効果

1. **バランス検証**: 「この設定は公平か？」をデータで回答
2. **パラメータ調整**: 定数を変更→再実行→結果比較のサイクル
3. **シナリオ評価**: 「面白い」シナリオの定量的な指標
