# システムリファレンス

Deep Juno のシミュレーションを構成する各システムの詳細設計へのポータルです。

## ドメインシステム (Domain Systems)

シミュレーションの中核ロジックを担うシステム群です。

- [**economy — 資源産出・消費**](systems/economy.md)
  - 資源の生産、維持コスト、環境汚染、資源の枯渇。
- [**population — 人口増減**](systems/population.md)
  - 食料、環境、収容力に基づいた人口動態。
- [**trade — 星間貿易**](systems/trade.md)
  - 惑星間の資源輸送と距離コスト。
- [**diplomacy — 外交と国家 AI**](systems/diplomacy.md)
  - 性格、記憶、派閥に基づいた動的な関係変動。
- [**war — 戦争処理**](systems/war.md)
  - 開戦トリガー、戦闘解決、和平条件。
- [**research — 技術研究**](systems/research.md)
  - 研究レート、自動化された技術進歩。
- [**events — ランダムイベント**](systems/events.md)
  - 決定論的な環境災害や社会イベント。
- [**terraforming — テラフォーミング**](systems/terraforming.md)
  - 段階的な惑星環境の改造。
- [**military — 軍備管理**](systems/military.md)
  - 艦船の建造と維持。

## 表示・制御システム (Display & Control)

シミュレーションデータの表示とインタラクションを担います。

- [**TUI Display — インタラクティブ表示**](systems/tui.md)
  - Ratatui によるダッシュボード描画と制御。

---

詳細な数式やバランス調整用パラメータについては、[**ゲームバランス定義 (balance.md)**](balance.md) を参照してください。
