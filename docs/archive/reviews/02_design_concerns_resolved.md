# 懸念事項と設計課題 (Design Concerns)

本ドキュメントでは、現在のアーキテクチャおよび実装において特定された、将来的な拡張性や整合性に影響を与える可能性のある懸念事項をまとめます。
各項目の詳細は、それぞれの Deep Dive ドキュメントを参照してください。

## 1. 艦隊 (Fleet) の粒度と戦闘システム

*   **概要**: `Fleet` エンティティが存在するものの、現在の戦闘システムは星系ごとの総戦力で計算を行っており、艦隊ごとの個別の挙動（移動、撤退、特定地点の防衛など）が反映されにくい構造になっています。
*   **詳細**: [Deep Dive: Fleet Granularity](deep_dives/05_fleet_granularity.md)

## 2. 占領状態 (Occupied) のデータ管理

*   **概要**: 惑星の占領状態 (`Occupied` コンポーネント) と、星系の戦況 (`Battlefront` エンティティのステータス) が二重管理に近い状態になっており、整合性を保つためのロジックが複雑化するリスクがあります。
*   **詳細**: [Deep Dive: Occupation State](deep_dives/06_occupation_state.md)

## 3. 外交と戦争の循環 (Diplomacy & War Cycle)

*   **概要**: 停戦処理と外交スコアの回復ロジックが疎結合であるため、停戦直後に再度宣戦布告される「無限戦争」ループに陥る可能性があります。外交状態と戦争状態の遷移をより堅牢にする必要があります。
*   **詳細**: [Deep Dive: Diplomacy & War Cycle](deep_dives/07_diplomacy_war_cycle.md)
