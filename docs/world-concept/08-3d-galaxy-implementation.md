# 3D銀河マップの実装方針

## 概要

1億の星系を3D空間に配置し、銀河規模のシミュレーションを実現するための技術設計。

## スケールの課題

### 目標仕様

- **星系数**: 1億（100,000,000）
- **国家数**: 数百〜数千（動的に変化）
- **貿易ルート**: 数万〜数十万
- **輸送艦隊**: 数万〜数十万

### パフォーマンス要件

- 60 FPS の維持（Phase 5以降でGUI実装時）
- メモリ使用量: 合理的な範囲内（16GB未満推奨）
- スケーラビリティ: 段階的に星系数を増やせる設計

## 3D空間構造

### 座標系

```rust
/// 3D銀河座標
#[derive(Component, Clone, Copy)]
pub struct GalacticPosition {
    /// X座標（光年）
    pub x: f32,

    /// Y座標（光年、銀河円盤の高さ）
    pub y: f32,

    /// Z座標（光年）
    pub z: f32,
}

impl GalacticPosition {
    /// 2点間の距離を計算（光年）
    pub fn distance_to(&self, other: &GalacticPosition) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 銀河中心からの距離
    pub fn distance_from_core(&self) -> f32 {
        (self.x * self.x + self.z * self.z).sqrt()
    }
}
```

### 銀河の形状

**渦巻き銀河（天の川銀河型）**

```rust
/// 銀河の形状パラメータ
pub struct GalaxyShape {
    /// 銀河の半径（光年）
    pub radius: f32,  // 50,000 光年

    /// 渦巻きアームの数
    pub arm_count: u32,  // 4-6本

    /// 中心核の半径
    pub core_radius: f32,  // 5,000 光年

    /// 円盤の厚さ
    pub disk_thickness: f32,  // 1,000 光年

    /// 渦巻きの巻き具合（ラジアン/光年）
    pub spiral_tightness: f32,
}

/// プロシージャルな星系生成
pub fn generate_galaxy_positions(
    galaxy: &GalaxyShape,
    star_count: usize,
) -> Vec<GalacticPosition> {
    let mut positions = Vec::with_capacity(star_count);
    let mut rng = thread_rng();

    for _ in 0..star_count {
        // 渦巻きアームに沿った分布
        let arm_index = rng.gen_range(0..galaxy.arm_count);
        let arm_angle = (arm_index as f32) * (2.0 * PI / galaxy.arm_count as f32);

        // 中心からの距離（指数分布で中心部に多く）
        let radius = rng.gen::<f32>().powf(0.5) * galaxy.radius;

        // 渦巻きアームに沿った角度
        let spiral_offset = radius * galaxy.spiral_tightness;
        let angle = arm_angle + spiral_offset + rng.gen_range(-0.5..0.5);

        // X, Z座標（銀河平面）
        let x = radius * angle.cos();
        let z = radius * angle.sin();

        // Y座標（円盤の厚さ）
        let y = rng.gen_range(
            -galaxy.disk_thickness * 0.5,
            galaxy.disk_thickness * 0.5
        ) * (1.0 - radius / galaxy.radius);  // 外縁ほど薄い

        positions.push(GalacticPosition { x, y, z });
    }

    positions
}
```

## 空間パーティショニング（Octree）

1億の星系を効率的に管理するため、Octree構造を使用。

### Octree の設計

```rust
/// Octree ノード
pub struct OctreeNode {
    /// ノードの境界ボックス
    pub bounds: AABB,

    /// このノードに含まれる星系（葉ノードのみ）
    pub star_systems: Vec<Entity>,

    /// 子ノード（8分割）
    pub children: Option<Box<[OctreeNode; 8]>>,

    /// このノードの星系数
    pub system_count: usize,
}

/// 軸平行境界ボックス
#[derive(Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    /// 点が境界ボックス内にあるか
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    /// 2つの境界ボックスが交差するか
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    /// 8分割した子境界ボックスを生成
    pub fn subdivide(&self) -> [AABB; 8] {
        let mid = (self.min + self.max) * 0.5;
        [
            AABB { min: self.min, max: mid },
            AABB { min: Vec3::new(mid.x, self.min.y, self.min.z),
                   max: Vec3::new(self.max.x, mid.y, mid.z) },
            // ... 残り6つ
        ]
    }
}
```

### Octree の構築

```rust
impl OctreeNode {
    /// 星系リストからOctreeを構築
    pub fn build(
        bounds: AABB,
        systems: Vec<(Entity, GalacticPosition)>,
        max_depth: u32,
        max_systems_per_node: usize,
    ) -> Self {
        // 分割が不要な場合（葉ノード）
        if systems.len() <= max_systems_per_node || max_depth == 0 {
            return OctreeNode {
                bounds,
                star_systems: systems.into_iter().map(|(e, _)| e).collect(),
                children: None,
                system_count: systems.len(),
            };
        }

        // 8分割
        let child_bounds = bounds.subdivide();
        let mut child_systems: [Vec<(Entity, GalacticPosition)>; 8] =
            Default::default();

        // 各星系を適切な子ノードに振り分け
        for (entity, pos) in systems {
            let point = Vec3::new(pos.x, pos.y, pos.z);
            for (i, bounds) in child_bounds.iter().enumerate() {
                if bounds.contains(point) {
                    child_systems[i].push((entity, pos));
                    break;
                }
            }
        }

        // 子ノードを再帰的に構築
        let children = child_bounds
            .into_iter()
            .zip(child_systems.into_iter())
            .map(|(bounds, systems)| {
                Self::build(bounds, systems, max_depth - 1, max_systems_per_node)
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        OctreeNode {
            bounds,
            star_systems: Vec::new(),
            children: Some(Box::new(children)),
            system_count: child_systems.iter().map(|v| v.len()).sum(),
        }
    }

    /// 範囲内の星系を検索
    pub fn query_range(&self, range: &AABB) -> Vec<Entity> {
        if !self.bounds.intersects(range) {
            return Vec::new();
        }

        let mut result = Vec::new();

        // 葉ノードの場合
        if let None = self.children {
            result.extend(self.star_systems.iter().copied());
            return result;
        }

        // 子ノードを再帰的に検索
        if let Some(children) = &self.children {
            for child in children.iter() {
                result.extend(child.query_range(range));
            }
        }

        result
    }

    /// 最近傍の星系を検索
    pub fn nearest_neighbor(
        &self,
        point: GalacticPosition,
        k: usize,
    ) -> Vec<Entity> {
        // k-NN検索の実装
        // 優先度付きキューを使用して効率的に検索
        // （実装詳細は省略）
        todo!()
    }
}
```

## LOD（Level of Detail）システム

### 詳細度レベル

```rust
/// 星系の詳細度レベル
#[derive(Clone, Copy, PartialEq)]
pub enum DetailLevel {
    /// 完全な詳細（カメラに近い）
    Full,

    /// 中程度の詳細（経済計算のみ）
    Medium,

    /// 低詳細（統計のみ）
    Low,

    /// スリープ（計算スキップ）
    Sleep,
}

/// カメラからの距離に基づいてLODを決定
pub fn calculate_lod(
    camera_pos: Vec3,
    system_pos: GalacticPosition,
) -> DetailLevel {
    let distance = camera_pos.distance(Vec3::new(
        system_pos.x,
        system_pos.y,
        system_pos.z,
    ));

    match distance {
        d if d < 1000.0 => DetailLevel::Full,
        d if d < 10000.0 => DetailLevel::Medium,
        d if d < 50000.0 => DetailLevel::Low,
        _ => DetailLevel::Sleep,
    }
}
```

### LODベースのシステム実行

```rust
/// LODに応じて処理を制御
pub fn lod_based_production(
    camera: Res<Camera>,
    mut systems: Query<(&GalacticPosition, &mut StarSystem)>,
) {
    for (pos, mut system) in systems.iter_mut() {
        let lod = calculate_lod(camera.position, *pos);

        match lod {
            DetailLevel::Full => {
                // 完全な生産計算
                system.produce_resources(1.0);
            }
            DetailLevel::Medium => {
                // 簡略化した計算
                system.produce_resources_simplified();
            }
            DetailLevel::Low => {
                // 統計的な推定
                system.estimate_production();
            }
            DetailLevel::Sleep => {
                // スキップ
            }
        }
    }
}
```

## メモリ最適化

### 遠方星系のアンロード

```rust
/// メモリ最適化用のリソース
#[derive(Resource)]
pub struct GalaxyMemoryManager {
    /// ロード済み星系のキャッシュ
    pub loaded_systems: HashMap<Entity, CachedStarSystem>,

    /// アンロード済み星系のコンパクト表現
    pub unloaded_systems: HashMap<Entity, CompactStarSystem>,
}

/// コンパクトな星系表現（メモリ節約）
#[derive(Clone, Copy)]
pub struct CompactStarSystem {
    /// 位置（f32 × 3 = 12バイト）
    pub position: GalacticPosition,

    /// 所有国（4バイト）
    pub owner: Option<u32>,  // Entity の ID のみ

    /// 星系タイプ（1バイト）
    pub system_type: u8,

    /// 合計: 約20バイト
}

impl GalaxyMemoryManager {
    /// カメラ位置に基づいてロード/アンロード
    pub fn update_loaded_region(
        &mut self,
        camera_pos: Vec3,
        load_radius: f32,
    ) {
        // 範囲外の星系をアンロード
        // 範囲内の星系をロード
        // （実装詳細は省略）
    }
}
```

### メモリ使用量の見積もり

**フルデータ（全星系ロード時）**
- StarSystem構造体: 約200バイト/星系
- 1億星系 × 200バイト = 20GB **（現実的でない）**

**コンパクト表現**
- CompactStarSystem: 約20バイト/星系
- 1億星系 × 20バイト = 2GB **（許容範囲）**

**ハイブリッドアプローチ**
- カメラ周辺の10万星系のみフルデータ: 20MB
- その他9990万星系はコンパクト: 2GB
- **合計: 約2GB（許容範囲）**

## プロシージャル生成

### 星系の動的生成

すべての星系を事前生成せず、必要に応じてプロシージャル生成：

```rust
/// シード値から星系を生成
pub fn generate_star_system(
    seed: u64,
    position: GalacticPosition,
) -> StarSystem {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // 中心からの距離で特性が変化
    let core_distance = position.distance_from_core();

    let system_type = if core_distance < 5000.0 {
        // 中心部: 古い文明、工業化
        StarSystemType::Industrial
    } else if core_distance > 40000.0 {
        // 辺境: 未開発
        StarSystemType::Frontier
    } else {
        // 中間部: ランダム
        pick_random_type(&mut rng)
    };

    StarSystem {
        name: generate_name(seed),
        position,
        owner: None,
        system_type,
        population: generate_population(&mut rng, system_type),
        resource_production: generate_resources(&mut rng, system_type),
        infrastructure: generate_infrastructure(&mut rng, core_distance),
    }
}
```

## 段階的実装計画

### Phase 1-4（現在）
- 2星系、固定位置
- 3D座標なし

### Phase 5
- 10-100星系
- 3D座標導入
- 単純な距離計算

### Phase 6-7
- 1000-10000星系
- Octree実装
- 基本的なLOD

### Phase 8-9
- 10万-100万星系
- 完全なLOD システム
- メモリ最適化

### Phase 10（最終目標）
- 1億星系
- プロシージャル生成
- 高度な最適化

## Bevy での実装

### 必要なクレート

```toml
[dependencies]
bevy = { version = "0.17.2", ... }
rand = "0.8"
rand_chacha = "0.3"  # 再現可能な乱数生成
noise = "0.9"  # プロシージャル生成用
```

### システム構成

```rust
fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(GalaxyMemoryManager::new())
        .add_systems(Startup, (
            initialize_galaxy,
            build_octree,
        ))
        .add_systems(Update, (
            update_lod_levels,
            lod_based_production,
            manage_memory,
        ))
        .run();
}
```

## パフォーマンス目標

### 計算量の見積もり

**最悪ケース（全星系を毎フレーム更新）**
- 1億星系 × 60 FPS = 60億操作/秒 **（不可能）**

**LOD適用後**
- Full詳細: 10万星系
- Medium詳細: 100万星系（簡略計算）
- Low詳細: 1000万星系（推定のみ）
- Sleep: 残り8890万星系（スキップ）

→ 実質的な計算量: 数百万操作/フレーム **（実現可能）**

### ベンチマーク目標

- Phase 5（100星系）: 60 FPS 安定
- Phase 7（10000星系）: 60 FPS 安定
- Phase 9（100万星系）: 30 FPS 以上
- Phase 10（1億星系）: 30 FPS 以上

## 技術的リスクと対策

### リスク1: メモリ不足

**対策**:
- コンパクト表現の徹底
- 動的ロード/アンロード
- プロシージャル生成

### リスク2: CPU負荷

**対策**:
- LODシステム
- 並列処理（Bevyの並列システム）
- 計算の簡略化

### リスク3: GPU描画負荷

**対策**:
- インスタンシング描画
- ポイントスプライトでの星系表現
- 遠方星系は描画スキップ

## 参考実装

類似のスケールを扱うゲーム：
- Stellaris（数千星系）
- Space Engine（プロシージャル宇宙生成）
- No Man's Sky（プロシージャル惑星生成）

技術的なアプローチを参考にしつつ、Bevyのエコシステムに適応。
