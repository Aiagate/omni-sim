# 提案 13: Web フロントエンド (Tactical FUI Edition)

## 概要

Bevy ECS のシミュレーション結果を WebSocket 経由でブラウザに配信し、**「Tactical FUI (Fantasy User Interface)」コンセプトに基づいた没入感のある銀河マップとダッシュボード**をリアルタイム表示する Web フロントエンド。

---

## デザインコンセプト: "Tactical FUI"

**コアコンセプト**: 軍事用戦術モニター、コックピット HUD。
**キーワード**: 高コントラスト、情報密度、グリッチノイズ、幾何学的、ネオン発光。

### カラーパレット & スタイル
*   **Background**: `#020205` (Deep Navy/Black)
*   **Primary Accent**: `#FF8C00` (Dark Orange) or `#00F0FF` (Cyber Cyan)
*   **Alert**: `#FF2A2A` (High Saturation Red)
*   **Typography**: `Share Tech Mono`, `Rajdhani`, `Fira Code` (All Caps, Wide Spacing)

### 視覚効果 (Post-Processing)
*   **Glow / Bloom**: 全ての発光要素にブラー処理
*   **Chromatic Aberration**: 画面端の色収差（版ズレ）
*   **Scanline / Grid**: 走査線とグリッドオーバーレイ

---

## 設計

### 1. アーキテクチャ

```
┌──────────────────┐     WebSocket      ┌────────────────────────┐
│  Bevy ECS Server │ ◄──────────────── │   Browser Frontend     │
│                  │ ──────────────── ► │ (React + Three.js)     │
│ SimulationEvent  │   JSON messages    │  - Tactical Map Layer  │
│ → WebSocket送信  │                    │  - FUI HUD Overlay     │
│                  │                    │  - Post-Processing     │
└──────────────────┘                    └────────────────────────┘
```

### 2. フロントエンド構成案

| コンポーネント | 技術スタック | 内容 |
|--------|------|------|
| **Core Framework** | React + Vite | UI構築の基盤 |
| **Tactical Map** | **Three.js / R3F** | 3D空間での星系・艦隊・貿易ルートの描画。ポストプロセス効果の適用 |
| **HUD / Overlay** | CSS Modules / Tailwind | HTMLによる情報オーバーレイ（ヘッダー、ログ、詳細パネル）。FUIスタイル（枠線、装飾）の適用 |
| **Motion** | Framer Motion | 数値カウントアップ、点滅、スライドイン等の演出 |
| **Data Viz** | Recharts / Custom | 時系列グラフ等のデータ表示 |

### 3. 機能要件

#### 3.1 マップ表示 (Tactical Map Layer)
*   **Visual**: 暗視モード風のベクターライン描画。
*   **Markers**: 幾何学図形（▲, ◆）による艦隊/星系表示。
*   **Trails**: 移動体へのトラッキングライン（軌跡）。
*   **Selection**: リーダーラインによる詳細情報の引き出し表示。

#### 3.2 HUD / オーバーレイ
*   **Global Header**: 作戦時間、ステータス表示。
*   **Decorations**: 画面四隅のブラケット、無意味な装飾数値、回転するレティクル。
*   **Log Console**: 高速で流れるシステムログ。

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `plugins/web_server.rs` | 【NEW】WebSocket サーバー・状態配信 |
| `web/` | 【NEW】Vite + React プロジェクト |
| `main.rs` | `--web` フラグによる Web モード起動 |

以上
