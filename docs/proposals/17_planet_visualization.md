# Proposal 17: Web UI Planet Visualization

## 1. 概要
現在のWeb UIでは星系（StarSystem）と艦隊（Fleet）のみが表示されており、惑星（Planet）の情報が欠落しています。
本プロポーザルでは、星系内の惑星を可視化し、各惑星の詳細情報（人口、資源、所有国など）を確認できるようにするための設計を提案します。

## 2. データ構造の拡張

### 2.1 バックエンド (Rust)
`src/plugins/web_integration.rs` の `StarSystemDto` を拡張し、惑星リストを含めます。

```rust
#[derive(Serialize)]
struct StarSystemDto {
    id: String,
    name: String,
    position: [f32; 3],  // 星系の銀河座標
    faction: String,     // 支配国（または支配的な国）
    population: f64,     // 総人口
    planets: Vec<PlanetDto>, // [NEW] 惑星リスト
}

#[derive(Serialize)] // [NEW]
struct PlanetDto {
    id: String,
    name: String,
    orbit_distance: f32, // 恒星からの距離 (描画用)
    angle: f32,          // 公転角 (描画用)
    size: f32,           // 惑星サイズ (描画用)
    color: String,       // 惑星タイプに基づく色
    faction: String,     // 支配国
    population: f64,
    type_name: String,   // "Earth-like", "Gas Giant" etc.
}
```

### 2.2 フロントエンド (TypeScript)
`web/src/api/SimulationClient.ts` の型定義を更新します。

```typescript
interface StarSystemDto {
    // ...既存フィールド
    planets: PlanetDto[];
}

interface PlanetDto {
    id: string;
    name: string;
    orbit_distance: number;
    angle: number;
    size: number;
    color: string;
    faction: string;
    population: number;
    type_name: string;
}
```

## 3. レンダリング設計

### 3.1 表示スケール
恒星間（数光年）と惑星系内（数天文単位）のスケール差が大きすぎるため、**デフォルメ表示**を採用します。

*   **銀河ビュー（Galaxy View）**:
    *   これまで通り星系を「点」として表示。
    *   惑星は表示しない（小さすぎるため）。

*   **星系詳細ビュー（System Detail View）**:
    *   ユーザーが星系をクリックしたときに、カメラを星系にズームするか、ポップオーバー/モーダルで詳細を表示。
    *   **提案**: 現在の `TacticalMap` 内で、星系ノードの周囲に「抽象的な軌道リング」と「惑星アイコン」を描画する。
    *   ズームレベルに応じて惑星の表示/非表示を切り替える（LOD: Level of Detail）。

### 3.2 描画ロジック (`TacticalMap.tsx`)
`StarNode` コンポーネント内で、`planets` 配列をループして子要素としてレンダリングします。

```tsx
const StarNode = ({ system }) => {
  return (
    <group position={system.position}>
      {/* 恒星本体 */}
      <mesh ... />
      
      {/* 惑星の軌道と本体 */}
      {system.planets.map(planet => (
        <group key={planet.id} rotation={[0, planet.angle, 0]}>
           {/* 軌道線 (Optional) */}
           <mesh rotation={[Math.PI/2, 0, 0]}>
             <ringGeometry args={[planet.orbit_distance, planet.orbit_distance + 0.1, 32]} />
             <meshBasicMaterial color="#333" opacity={0.2} transparent />
           </mesh>
           
           {/* 惑星メッシュ */}
           <mesh position={[planet.orbit_distance, 0, 0]}>
             <sphereGeometry args={[planet.size * 0.5, 16, 16]} />
             <meshStandardMaterial color={planet.color} />
           </mesh>
        </group>
      ))}
    </group>
  );
}
```

## 4. UI/UX
*   **惑星ホバー**: カーソルを合わせると惑星名、タイプ、人口、所有国を表示するツールチップを出します。
*   **惑星クリック**: 選択状態にし、サイドパネルに詳細情報を表示（将来的な拡張）。

## 5. 課題と対策
*   **通信量**: 惑星データが増えるとパケットサイズが大きくなる。
    *   対策: 初回は全データを送るが、以降は差分更新にするか、または現状の規模（数千星系程度）ならgzip圧縮が効くので一旦は毎回全送で進める。
*   **視認性**: 星系が密集していると惑星表示が重なって見づらくなる。
    *   対策: カメラが一定距離以上近づいたときだけ惑星を表示する（LOD）。

## 6. 実装ステップ
1.  **Backend**: `StarSystemDto` に `planets` を追加し、データ入力ロジックを実装。
2.  **Frontend**: 型定義を更新。
3.  **Frontend**: `TacticalMap` にLOD制御と惑星描画ロジックを追加。
4.  **Frontend**: ツールチップ情報の拡充。
