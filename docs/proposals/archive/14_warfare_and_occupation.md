# 提案 14: 戦争と占領メカニクス

## 概要

現在（または提案段階）の戦闘は艦隊決戦が主であり、惑星の支配権を巡る具体的なプロセスが欠落している。**軌道爆撃・地上侵攻・占領統治**のプロセスを導入し、惑星を「奪い合う」ための戦略的な深みを提供する。

---

## 現状の課題

| 課題 | 現行の挙動 |
|------|-----------|
| 艦隊戦のみ | 敵艦隊を撃破しても、敵惑星をどうにかする手段がない |
| 占領不可 | 敵の領土を奪うメカニクスが存在しない |
| 戦争の目的 | 相手の戦力を削ぐだけで、領土拡張につながらない |
| 地上軍の不在 | 惑星防衛隊や侵攻部隊といった概念がない |

---

## 設計

### 1. 侵攻のフェーズ

惑星攻略は3段階のプロセスで進行する。

1.  **制宙権の確保（Orbital Supremacy）**: 敵艦隊を排除し、惑星軌道を封鎖する。
2.  **軌道爆撃（Orbital Bombardment）**: 地上防衛施設を破壊し、防衛軍を弱体化させる。
3.  **地上侵攻（Ground Invasion）**: 輸送船から陸軍を降下させ、都市を制圧する。

### 2. コンポーネント設計

#### 2-1. 地上軍（Army）

```rust
/// 地上軍ユニット
#[derive(Component, Debug, Clone)]
pub struct Army {
    pub army_type: ArmyType,
    pub strength: f64,      // 戦闘力
    pub morale: f64,        // 士気（0になると壊滅/降伏）
    pub experience: f64,    // 経験値ランク
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArmyType {
    Infantry,       // 歩兵：占領・治安維持に有利
    Mechanized,     // 機甲：平地での戦闘力が高い
    SpecialForces,  // 特殊部隊：防御ボーナス無視
    Militia,        // 民兵：緊急徴兵、弱いが維持費が安い
    Garrison,       // 駐屯軍：防御専用、移動不可
}
```

#### 2-2. 惑星の防衛状態

```rust
/// 惑星の防衛パラメータ
#[derive(Component, Debug, Clone)]
pub struct PlanetaryDefense {
    pub fortification_level: f64,  // 要塞化レベル（爆撃耐性）
    pub shield_integrity: f64,     // 惑星シールド（0になるまで爆撃無効）
    pub garrison_strength: f64,    // 駐屯軍の総戦力
}
```

### 3. 戦闘ロジック

#### 3-1. 軌道爆撃

艦隊が惑星軌道上にある場合、毎 Tick 爆撃を実行できる。

*   **精密爆撃（Surgical Strike）**:
    *   目標: 軍事施設のみ
    *   効果: 防衛軍へのダメージ小
    *   副作用: インフラ損傷・民間人被害は最小限
*   **無差別爆撃（Indiscriminate）**:
    *   目標: 地表すべて
    *   効果: 防衛軍へのダメージ大、要塞レベル低下
    *   副作用: 人口減少、施設破壊、惑星環境（居住性）の悪化
*   **ハルマゲドン（Armageddon）**:
    *   目標: 惑星の完全破壊（特定技術が必要）
    *   効果: 惑星が Tomb World 化（居住不可）

#### 3-2. 地上戦

侵攻軍 vs 防衛軍の戦闘解決。

```rust
fn resolve_ground_combat(attacker: &mut Army, defender: &mut Army, context: &CombatContext) {
    // 攻撃側のダメージ
    let atk_dmg = attacker.strength * context.attacker_tactics_bonus;
    // 防衛側のダメージ（要塞ボーナスあり）
    let def_dmg = defender.strength * context.fortification_bonus;

    defender.take_damage(atk_dmg);
    attacker.take_damage(def_dmg);
    
    // 士気判定
    if defender.morale <= 0.0 {
        // 防衛側敗北 -> 占領
    }
}
```

### 4. 占領と統治

地上戦に勝利すると、惑星は**占領地（Occupied Territory）**となる。所有権は即座には移転しない。

#### 4-1. 占領政策（Occupation Policy）

占領国は統治方針を選択できる。

| 政策 | メリット | デメリット |
|------|----------|------------|
| **慈悲的統治** | レジスタンス発生率↓、外交評判への悪影響小 | 資源収奪なし、維持コストがかかる |
| **搾取** | 資源産出量の一部を即座に獲得 | 人口減少、レジスタンス激化、施設破壊リスク |
| **浄化** | 異種族/敵対人口を強制排除（住居確保） | 外交関係最悪（Genocidal）、全国家から敵視 |
| **傀儡化** | 以前の統治機構を利用 | 収入の一部のみ獲得、独立のリスク |

#### 4-2. レジスタンス活動

占領下では確率でレジスタンスイベントが発生。

*   **サボタージュ**: 建造物や資源貯蔵の破壊
*   **暴動**: 駐屯軍へのダメージ
*   **ストライキ**: 資源産出の停止

安定度（Stability）を高めるか、十分な駐屯軍（Garrison）を置くことで抑制可能。

### 5. 戦争の終結と講和

戦争状態を終了させる講和会議（Peace Conference）にて、占領地の扱いが確定する。

*   **割譲（Cession）**: 正式に領土として併合。
*   **解放（Liberation）**: 独立国として復活、または元の持ち主に返還。
*   **属国化（Vassalization）**: 敗戦国全体を属国にする。

---

## 既存コードへの影響

| ファイル | 変更内容 |
|---------|---------|
| `components/military.rs` | `Army`, `PlanetaryDefense` コンポーネント追加 |
| `components/planet.rs` | 占領状態フラグ、レジスタンス値の追加 |
| `systems/war.rs` | 軌道爆撃、地上戦処理の追加 |
| `systems/population.rs` | 戦災による人口減少、難民発生ロジック |
| `systems/diplomacy.rs` | 占領政策による外交ペナルティの計算 |

---

## 期待される効果

1.  **戦争の長期化と重み**: 艦隊で勝っても、地上軍がいなければ惑星を奪えないため、補給と輸送が重要になる。
2.  **倫理的ジレンマ**: 「早期決着のために無差別爆撃をして、手に入れた惑星が廃墟になってもいいか？」
3.  **内政とのリンク**: 強力な陸軍を維持するための経済的・人的コスト。
