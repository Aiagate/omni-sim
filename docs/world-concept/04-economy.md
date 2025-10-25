# 経済システム

## 基本的な経済モデル

### 現在の実装

シンプルな資源輸送と GDP 増加のモデルを採用：

1. 輸出国から資源を減算
2. 輸入国に資源を加算
3. 輸入国の GDP を増加（`貿易量 × 0.05`）

### 経済モデルの哲学

[要確認: 経済システムの現実性と複雑さ]

- 現実の経済理論に基づくか？
- ゲーム的な簡略化されたモデルか？
- どの程度の複雑さを目指すか？

## 資源システム

### 資源の定義

現在の実装では、単一の抽象的な「資源（resource）」を扱っています。

[要確認: 資源の詳細設定]

#### 資源の種類

**オプションA: 単一資源**
- 現在の実装: すべての資源を統一的に扱う
- メリット: シンプル、実装が容易
- デメリット: 現実性が低い

**オプションB: 複数資源タイプ**
- 例: 鉱物、エネルギー、食料、工業製品など
- メリット: 複雑な経済相互作用
- デメリット: 実装が複雑

#### 資源の生産

[要確認: 資源はどのように生まれるか]

- 資源は初期値のみか？
- 時間経過で自動的に生産されるか？
- 生産設備や技術が必要か？
- 無限か、有限か？

#### 資源の消費

[要確認: 資源の消費メカニズム]

- 資源は自然に減少するか？
- 人口や経済規模に応じて消費されるか？
- それとも貿易でのみ移動するか？

### 資源の制約

現在の実装では：
- `resource_stock >= 0.0`（負の値にならない）
- 貿易前に在庫を確認

[要確認: その他の制約]
- 資源の保管容量制限はあるか？
- 劣化や損失はあるか？

## GDP（国内総生産）システム

### 現在の実装の問題点

```rust
dest.gdp += fleet.cargo * 0.05;
```

- 輸入国のみGDP増加（輸出国は？）
- `× 0.05` という係数に根拠がない
- 星系ベースの経済モデルと整合性がない

### 新しいGDP計算モデル（提案）

**「ゲーム的だが合理的」な経済モデル**

#### GDP の定義

GDP = 全星系の経済活動の総和

```rust
nation.gdp =
    全星系の生産価値の合計 +
    貿易による付加価値 +
    技術・インフラ補正
```

#### 詳細な計算式

```rust
/// 国家のGDP計算
pub fn calculate_nation_gdp(
    nation: &Nation,
    systems: &Query<&StarSystem, With<Ownership>>,
    territory: &Territory,
) -> f32 {
    let mut total_gdp = 0.0;

    // 1. 各星系の基本生産価値
    for system_entity in &territory.owned_systems {
        if let Ok(system) = systems.get(*system_entity) {
            // 資源生産価値（仮の価格設定）
            let production_value =
                system.resource_production.minerals * 1.0 +
                system.resource_production.energy * 1.5 +
                system.resource_production.food * 1.2 +
                system.resource_production.industrial_goods * 2.0;

            // インフラ補正
            let infrastructure_bonus = system.infrastructure;

            total_gdp += production_value * (1.0 + infrastructure_bonus);
        }
    }

    // 2. 人口による経済規模
    total_gdp += territory.total_population * 10.0;

    // 3. 技術補正
    total_gdp *= 1.0 + (nation.tech_level * 0.1);

    total_gdp
}
```

#### 貿易による経済効果

貿易は両国に利益をもたらす：

```rust
/// 貿易実行時の経済効果
pub fn trade_economic_impact(
    exporter: &mut Nation,
    importer: &mut Nation,
    resource_type: ResourceType,
    volume: f32,
    distance: f32,
) {
    // 基本価格（資源タイプによる）
    let base_price = match resource_type {
        ResourceType::Minerals => 1.0,
        ResourceType::Energy => 1.5,
        ResourceType::Food => 1.2,
        ResourceType::Industrial_Goods => 2.0,
    };

    // 取引価値
    let trade_value = volume * base_price;

    // 輸出国の利益（販売収入）
    exporter.treasury.add_currency(trade_value * 0.8);

    // 輸入国の利益（資源獲得による生産性向上）
    // 資源を得ることで経済活動が活発化
    importer.economic_activity += trade_value * 0.3;

    // 輸送コスト（距離に応じて）
    let transport_cost = volume * distance * 0.01;
    exporter.treasury.subtract_currency(transport_cost * 0.5);
    importer.treasury.subtract_currency(transport_cost * 0.5);
}
```

### GDP の意味と影響

#### GDP は何を表すか

- **経済規模の総合指標**
- 国力のバロメーター
- 複数の要因から構成される複合指標

#### GDP が高いことの効果

1. **より多くの投資が可能**
   - 新規星系の植民
   - インフラ整備
   - 軍備拡張

2. **技術開発の加速**
   - GDP の一部を研究開発に投資
   - tech_level の向上

3. **外交的影響力**
   - 大国としての発言力
   - 小国からの協力要請

4. **軍事力の基盤**
   - GDP が高いほど大規模な艦隊を維持可能
   - military_power の上限に影響

### GDP の成長モデル

[要確認: 経済成長のメカニズム]

- 貿易のみで成長するか？
- 内需による成長はあるか？
- 技術進歩による生産性向上はあるか？

## 貿易のメカニズム

### 貿易の決定

[要確認: 貿易の意思決定プロセス]

#### 現在の実装
- 貿易ルートは固定
- 自動的に繰り返し実行

#### 将来の可能性
- 国家AIが貿易相手を選択？
- 価格メカニズムによる需給調整？
- プレイヤーが貿易協定を設定？

### 貿易の利益

[要確認: 貿易による利益配分]

- 現在: 輸入国のみ GDP 増加
- 疑問:
  - 輸出国の利益は？
  - 相互利益（Win-Win）のモデルにするか？
  - 取引価格の概念はあるか？

### 貿易コスト

[要確認: 貿易にかかるコスト]

- 輸送コストはあるか？
- 関税や手数料はあるか？
- 距離によるコスト増加はあるか？

## 市場と価格（Phase 5で予定）

### 価格メカニズム

[要確認: 価格システムの設計]

- 資源に価格はあるか？
- 需要と供給で価格が変動するか？
- 固定価格か、市場価格か？

### 市場の種類

**オプションA: グローバル市場**
- すべての国家が同じ価格で取引

**オプションB: ローカル市場**
- 各国家で異なる価格
- 裁定取引の可能性

**オプションC: 相対価格**
- 国家間の交渉で価格決定

## 経済危機とリスク

[要確認: 負のイベントの扱い]

### リスク要因
- 輸送の失敗（Phase 5で予定）
- 資源枯渇
- 経済バブルと崩壊
- 貿易封鎖

### 危機への対応
- 自動的に回復するか？
- プレイヤーや国家AIの介入が必要か？

## 経済の勝利条件

[要確認: シミュレーションの終了条件]

- GDP目標の達成？
- 資源の独占？
- 単に観察するだけ？
- 時間制限？
