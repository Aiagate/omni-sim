# 星系と国家の関係モデル設計

## 概要

QA.mdの回答に基づく星系（StarSystem）と国家（Nation）の関係モデル。

### スケール目標
- **星系数**: 1億を目標
- **国家**: 複数の星系を統治
- **動的変化**: 国家の滅亡や勃興で数が変動

## エンティティ構成

### StarSystem（星系）

銀河内の個別の恒星系を表現する基本単位。

```rust
/// 星系エンティティ
///
/// 銀河内の1つの恒星系を表現します。
/// 資源の生産拠点であり、国家の領土の基本単位です。
#[derive(Component)]
pub struct StarSystem {
    /// 星系名（自動生成または固有名）
    pub name: String,

    /// 3D銀河空間内の座標（光年単位）
    pub position: Vec3,

    /// 所有国家への参照（Option::None = 無主の星系）
    pub owner: Option<Entity>,

    /// 星系タイプ（資源や特性に影響）
    pub system_type: StarSystemType,

    /// 人口（百万人単位）
    pub population: f32,

    /// 資源生産レート（単位時間あたり）
    pub resource_production: ResourceProduction,

    /// インフラレベル（0.0-1.0）
    /// 生産効率や防衛力に影響
    pub infrastructure: f32,
}

/// 星系タイプ
#[derive(Clone, Copy, PartialEq)]
pub enum StarSystemType {
    /// 中心星系（国家の首都）
    Capital,

    /// 工業星系（工業製品生産に特化）
    Industrial,

    /// 採掘星系（鉱物資源に特化）
    Mining,

    /// 農業星系（食料生産に特化）
    Agricultural,

    /// 研究星系（技術開発）
    Research,

    /// 辺境星系（未開発、低生産）
    Frontier,

    /// 無人星系（資源のみ、人口ゼロ）
    Uninhabited,
}

/// 資源生産レート
#[derive(Clone, Copy)]
pub struct ResourceProduction {
    /// 鉱物資源（採掘による）
    pub minerals: f32,

    /// エネルギー（核融合炉などから）
    pub energy: f32,

    /// 食料（農業生産）
    pub food: f32,

    /// 工業製品（工場での生産）
    pub industrial_goods: f32,
}
```

### Nation（国家）- 拡張版

複数の星系を統治する星間国家。

```rust
/// 国家エンティティ（拡張版）
///
/// 複数の星系を統治する星間国家を表現します。
#[derive(Component)]
pub struct Nation {
    /// 国家名
    pub name: String,

    /// 国家の中心星系（首都）
    pub capital: Entity,

    /// GDP（全領土の経済規模合計）
    pub gdp: f32,

    /// 国庫（貿易や税収で蓄積される資源）
    pub treasury: Resources,

    /// 国家タイプ/政体
    pub government_type: GovernmentType,

    /// 技術レベル（0.0-10.0）
    pub tech_level: f32,

    /// 軍事力（艦隊数や戦力の総計）
    pub military_power: f32,
}

/// 国家の政体タイプ
#[derive(Clone, Copy, PartialEq)]
pub enum GovernmentType {
    /// 帝国（中央集権、拡張志向）
    Empire,

    /// 連邦（民主的、貿易重視）
    Federation,

    /// 企業国家（経済効率重視）
    Corporate,

    /// 軍事独裁（軍事力重視）
    MilitaryJunta,

    /// 共和国（バランス型）
    Republic,
}

/// 資源ストック
#[derive(Clone, Copy, Default)]
pub struct Resources {
    pub minerals: f32,
    pub energy: f32,
    pub food: f32,
    pub industrial_goods: f32,
}
```

### Territory（領土関係）

国家と星系の所有関係を管理するコンポーネント。

**オプションA: StarSystemのownerフィールドで管理**
```rust
// StarSystem構造体内
pub owner: Option<Entity>,  // 国家への参照
```

**オプションB: 別コンポーネントで管理**
```rust
/// 領土関係コンポーネント
///
/// 国家に付与し、所有する星系のリストを保持
#[derive(Component)]
pub struct Territory {
    /// 所有する全星系のエンティティリスト
    pub owned_systems: Vec<Entity>,

    /// 総人口（全星系の合計）
    pub total_population: f32,

    /// 総星系数
    pub system_count: usize,
}
```

**推奨**: オプションAとBの併用
- StarSystem.owner で個別の所有関係を管理
- Territory で国家側から効率的にクエリ

## 国家と星系の関係性

### 1対多の関係

```
Nation (1) ──< owns >── (N) StarSystem
    ↑                         ↓
    └────── capital ──────────┘
```

### 初期設定例

```rust
fn setup_galaxy(mut commands: Commands) {
    // 国家A（帝国）を作成
    let empire_capital = commands.spawn(StarSystem {
        name: "Terra Prime".into(),
        position: Vec3::new(0.0, 0.0, 0.0),
        owner: None,  // 後で設定
        system_type: StarSystemType::Capital,
        population: 10.0,  // 10億人
        resource_production: ResourceProduction {
            minerals: 50.0,
            energy: 100.0,
            food: 80.0,
            industrial_goods: 120.0,
        },
        infrastructure: 0.9,
    }).id();

    let nation_a = commands.spawn(Nation {
        name: "Terran Empire".into(),
        capital: empire_capital,
        gdp: 10000.0,
        treasury: Resources {
            minerals: 1000.0,
            energy: 2000.0,
            food: 1500.0,
            industrial_goods: 3000.0,
        },
        government_type: GovernmentType::Empire,
        tech_level: 5.0,
        military_power: 1000.0,
    }).insert(Territory {
        owned_systems: vec![empire_capital],
        total_population: 10.0,
        system_count: 1,
    }).id();

    // 首都の所有者を設定
    commands.entity(empire_capital)
        .insert(Ownership { nation: nation_a });

    // 追加の星系を生成
    for i in 0..10 {
        let system = commands.spawn(StarSystem {
            name: format!("System {}", i),
            position: Vec3::new(
                (i as f32) * 10.0,
                random_float() * 5.0,
                random_float() * 5.0,
            ),
            owner: Some(nation_a),
            system_type: StarSystemType::Industrial,
            population: 2.0,
            resource_production: ResourceProduction {
                minerals: 30.0,
                energy: 40.0,
                food: 20.0,
                industrial_goods: 60.0,
            },
            infrastructure: 0.5,
        }).id();

        // Territoryに追加
        // （実際はシステムで動的に管理）
    }
}
```

### 所有権コンポーネント（オプション）

双方向の関連を効率化するため、星系側にも所有情報を持たせる：

```rust
/// 星系の所有権
#[derive(Component)]
pub struct Ownership {
    /// 所有国家への参照
    pub nation: Entity,

    /// 占領された時刻（シミュレーション時間）
    pub acquired_at: f32,

    /// 占領方法（征服、植民、購入など）
    pub acquisition_type: AcquisitionType,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AcquisitionType {
    /// 初期配置
    Initial,

    /// 植民地化（無主の星系）
    Colonization,

    /// 軍事征服
    Conquest,

    /// 外交的譲渡
    Cession,

    /// 購入
    Purchase,
}
```

## システム設計

### 資源生産システム

各星系が時間経過で資源を生産し、国家の国庫に蓄積：

```rust
/// 星系の資源生産システム
pub fn starsystem_production(
    time: Res<Time>,
    systems: Query<(&StarSystem, &Ownership)>,
    mut nations: Query<&mut Nation>,
) {
    for (system, ownership) in systems.iter() {
        let Ok(mut nation) = nations.get_mut(ownership.nation) else {
            continue;
        };

        let dt = time.delta_secs();
        let production = &system.resource_production;
        let efficiency = system.infrastructure;

        // 生産量 = 基本生産レート × インフラ効率 × 時間
        nation.treasury.minerals += production.minerals * efficiency * dt;
        nation.treasury.energy += production.energy * efficiency * dt;
        nation.treasury.food += production.food * efficiency * dt;
        nation.treasury.industrial_goods +=
            production.industrial_goods * efficiency * dt;
    }
}
```

### 領土管理システム

国家の領土情報を自動更新：

```rust
/// 領土情報更新システム
pub fn update_territory(
    mut nations: Query<(Entity, &mut Territory)>,
    systems: Query<(&StarSystem, &Ownership)>,
) {
    for (nation_entity, mut territory) in nations.iter_mut() {
        // 所有星系をリセット
        territory.owned_systems.clear();
        territory.total_population = 0.0;

        // 全星系をスキャンして所有を確認
        for (system, ownership) in systems.iter() {
            if ownership.nation == nation_entity {
                territory.owned_systems.push(ownership.nation);
                territory.total_population += system.population;
            }
        }

        territory.system_count = territory.owned_systems.len();
    }
}
```

## 貿易の再設計

### 星系間貿易

国家レベルではなく、星系間の直接貿易に変更：

```rust
/// 星系間貿易ルート
#[derive(Component)]
pub struct InterstellarTradeRoute {
    /// 輸出元星系
    pub origin_system: Entity,

    /// 輸入先星系
    pub destination_system: Entity,

    /// 貿易する資源タイプ
    pub resource_type: ResourceType,

    /// 取引量
    pub volume: f32,

    /// 輸送距離（光年）
    pub distance: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ResourceType {
    Minerals,
    Energy,
    Food,
    IndustrialGoods,
}
```

### 国家間貿易（上位レベル）

国家が保有する資源の総量での取引：

```rust
/// 国家間貿易協定
#[derive(Component)]
pub struct TradeAgreement {
    /// 輸出国
    pub exporter: Entity,

    /// 輸入国
    pub importer: Entity,

    /// 取引内容（複数資源を同時に）
    pub terms: Vec<TradeTerm>,

    /// 協定期間（残り時間）
    pub duration: f32,
}

pub struct TradeTerm {
    pub resource_type: ResourceType,
    pub amount_per_cycle: f32,
    pub price: f32,  // 単位あたりの価格
}
```

## スケーラビリティ考慮事項

### 1億星系への対応

#### 段階的な実装
1. **Phase 1**: 10-100星系（プロトタイプ）
2. **Phase 2**: 1000-10000星系（最適化テスト）
3. **Phase 3**: 100000星系（大規模テスト）
4. **Phase 4**: 1億星系（最終目標）

#### パフォーマンス最適化
- 空間パーティショニング（Octree）
- 非活性星系のスリープ化
- LOD（詳細度レベル）による処理の簡略化
- 並列処理の活用

#### メモリ管理
- 遠方の星系はメモリから解放
- 必要時にオンデマンドで再生成
- プロシージャル生成の活用

## 国家の勃興と滅亡

### 滅亡条件

```rust
/// 国家の滅亡判定システム
pub fn nation_collapse(
    mut commands: Commands,
    nations: Query<(Entity, &Territory, &Nation)>,
    mut systems: Query<&mut Ownership>,
) {
    for (nation_entity, territory, nation) in nations.iter() {
        // 滅亡条件チェック
        if territory.system_count == 0 {
            println!("{} has collapsed (no systems)", nation.name);
            commands.entity(nation_entity).despawn();
            continue;
        }

        if nation.treasury.energy < 0.0 && nation.treasury.food < 0.0 {
            println!("{} has collapsed (resource depletion)", nation.name);

            // 全星系を無主にする
            for system_entity in &territory.owned_systems {
                if let Ok(mut ownership) = systems.get_mut(*system_entity) {
                    commands.entity(*system_entity).remove::<Ownership>();
                }
            }

            commands.entity(nation_entity).despawn();
        }
    }
}
```

### 勃興（新国家の誕生）

```rust
/// 新国家の勃興システム
pub fn nation_emergence(
    mut commands: Commands,
    uninhabited: Query<(Entity, &StarSystem), Without<Ownership>>,
) {
    // 無主の星系が一定数集まったら新国家を建国
    let frontier_systems: Vec<_> = uninhabited.iter()
        .filter(|(_, sys)| sys.population > 1.0)
        .take(5)
        .collect();

    if frontier_systems.len() >= 5 {
        // 新国家を作成
        let capital = frontier_systems[0].0;
        let new_nation = commands.spawn(Nation {
            name: generate_nation_name(),
            capital,
            gdp: 1000.0,
            treasury: Resources::default(),
            government_type: GovernmentType::Republic,
            tech_level: 1.0,
            military_power: 100.0,
        }).id();

        // 星系に所有権を設定
        for (system_entity, _) in frontier_systems {
            commands.entity(system_entity).insert(Ownership {
                nation: new_nation,
                acquired_at: 0.0,
                acquisition_type: AcquisitionType::Initial,
            });
        }

        println!("New nation emerged: {}", "New Republic");
    }
}
```

## 次のステップ

この設計を基に：
1. 世界観ドキュメントを更新
2. GDP計算式を星系ベースで再設計
3. 3D銀河マップの実装方針を策定
4. 実装ガイドのPhase更新

## 未解決の設計判断

1. **ResourcesとGDPの関係**
   - GDPは資源総量から計算するか？
   - それとも独立した指標か？

2. **星系の自動生成**
   - プロシージャル生成のアルゴリズム
   - 名前、位置、タイプの決定方法

3. **国家の初期配置**
   - ランダム配置か？
   - クラスター配置か？
   - プレイヤーが設定するか？

4. **輸送時間の計算**
   - 距離に比例？
   - ワープゲート網があれば短縮？
   - 技術レベルで変化？
