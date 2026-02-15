# 深掘り分析: データ駆動型アーキテクチャへの移行

## 1. 現状の課題

現在の世界初期化処理 ([src/plugins/world_init.rs](file:///home/dorothy/repos/omni-sim/src/plugins/world_init.rs)) は、Rustコード内に具体的なエンティティ（地球、プロキシマbなど）のパラメータがハードコードされています。

- **保守性**: 惑星の初期資源や産出レートを調整するたびにコンパイルが必要です。
- **拡張性**: 提案03で検討されている「複雑な惑星環境パラメータ」を全ての惑星に対してコードで定義するのは非常に煩雑です。
- **リプレイ性**: 異なるシナリオ（開始時の惑星配置や国家構成）を試すことが困難です。

## 2. 移行案: 設定ファイルによる定義

初期状態を JSON または YAML（Rustでは `serde` を使用）で定義し、起動時にロードする仕組みへの移行を推奨します。

### 設定ファイルのイメージ (scenarios/default.yaml)
```yaml
star_systems:
  - name: "Sol"
    planets:
      - name: "Earth"
        population: 10_000_000
        environment:
          mass: 1.0
          atmosphere: EarthLike
          temperature: 15.0
        production:
          food: 120.0
          minerals: 40.0
```

### 実装ステップ
1. **スキーマ定義**: `WorldSnapshot` や `PlanetInitialState` といった `Serialize / Deserialize` 可能な構造体を定義します。
2. **ローダーの実装**: `WorldInitPlugin` が指定されたパスの設定ファイルを読み込み、ループでエンティティを生成するように変更します。
3. **既存データの移行**: 現在 `world_init.rs` にある値を `scenarios/default.yaml` に書き出します。

## 3. 期待される効果
- **デザイナー/ユーザーフレンドリー**: コードを触らずにゲームバランスを調整可能。
- **テストの容易化**: 特定の状況（例: 資源が極端に少ない、戦争直前など）を再現するテスト用シナリオを簡単に作成可能。
- **MOD対応**: ユーザーが独自の星系や惑星を追加するのが容易になります。
