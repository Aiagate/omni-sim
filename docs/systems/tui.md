# TUI Display — インタラクティブ表示

**ファイル**: `src/plugins/tui_output.rs`, `src/plugins/tui_state.rs`  

## 概要
ターミナル上にリアルタイムなダッシュボードを描画し、シミュレーションの進行をインタラクティブに制御します。

## 主要なコンポーネント

- **TuiAppState (Resource)**:
  ECS からの状態（惑星の資源、イベントログ、外交、戦争）を TUI 用に保持するスナップショットキャッシュ。

- **SimControl (Resource)**:
  シミュレーションのポーズ、速度、ステップ実行などの制御フラグを管理。

## 構成システム

1. **入力処理 (tui_input_system)**:
   - キーボード入力を監視し、`SimControl` や選択中の惑星（選択ターゲット）を切り替える。

2. **状態更新 (tui_snapshot_system, tui_event_collector_system)**:
   - 毎 Tick 進行後に、ECS 内の最新情報を `TuiAppState` に反映。

3. **描画 (tui_render_system)**:
   - `TuiAppState` のデータを元に、Ratatui を使用して画面をレイアウト・描画。

## 主な操作
- **Space**: ポーズ / 再開。
- **N**: ステップ実行（ポーズ中のみ 1 Tick 進行）。
- **+/-**: シミュレーションの視覚的な速度調整（将来拡張用）。

## 設計メモ
- ドメインシステムと表示システムが完全に分離されており、TUI は単に `SimulationEvent` を受信し、リソースの値を描画しているだけである。
- これにより、表示を CLI に戻したり、将来的に別の UI ライブラリに差し替えることが容易になっている。
