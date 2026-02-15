# Deep Dive: 占領状態のデータ管理 (Occupation State)

## 現状の課題

現在、占領に関連するステータスが複数の場所に分散しています。

1.  **`Planet` エンティティ**: `Occupied` コンポーネントを持つ。
    *   `occupied_by`: 誰が支配しているか。
2.  **`Battlefront` エンティティ**: `status` フィールドを持つ。
    *   `BattleStatus::Occupied`: 攻撃側が制圧した状態。
    *   `BattleStatus::Secure`: 防衛側が確保した状態。

この二重管理は以下のシナリオで不整合を起こすリスクがあります。

*   星系内に惑星が2つあり、1つだけが占領された場合、`Battlefront` のステータスは何にすべきか？ (`Contested`? `PartiallyOccupied`?)
*   イベント等でスクリプト的に `Occupied` コンポーネントが削除された場合、`Battlefront` 側のステータスと矛盾が生じる。

## 改善案

### `Battlefront` を「戦況のキャッシュ」と定義する

恒久的な「誰のものか」という情報は **`Planet` (または `StarSystem`) に一元化** し、`Battlefront` はあくまで一時的な戦闘状態の追跡のみに利用する。

#### 具体的な変更

1.  `BattleStatus` から `Occupied` / `Secure` を削除する、あるいは意味を変える。
    *   `Active`: 戦闘中。
    *   `Resolved`: 戦闘終了（次の Tick で `Battlefront` エンティティ自体が削除される）。
2.  占領判定ロジックのフロー:
    *   戦闘システムの最後で、惑星の防御力が0になったら `Planet` に `Occupied` を付与。
    *   星系内の全ての惑星が `Occupied` (または所有者が攻撃側と同じ) になったら、その星系の戦闘は終了とみなす。
    *   `Battlefront` エンティティを削除する。

### 支配権の移譲 (Ownership Transfer)

`Occupied` コンポーネントはずっとつけておくのか、それとも `BelongsToNation` を書き換えるのか？

*   **現状**: `Occupied` を付与し、元の持ち主 (`BelongsToNation`) は維持している（国際法上の占領状態）。
*   **将来**: 講和会議（Diplomacy System）で割譲が決定したタイミングで、`Occupied` を外し、`BelongsToNation` を書き換えるのが適切。

## 推奨方針

`Battlefront` のステータス管理を簡素化し、**「戦闘中かどうか」** の判定のみに使用する。
占領の実態は `Planet` エンティティの `Occupied` コンポーネントを正とする。
