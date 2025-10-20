"""
戦闘解決ロジック

艦隊間の戦闘を解決する。
Phase 2: 弾薬消費ロジック追加
"""

from typing import Tuple
from src.entities.fleet import Fleet
from src.entities.ship import Ship
from src.utils.rng import get_rng
from src.config.constants import BASE_HIT_RATE, MIN_DAMAGE


class CombatResolver:
    """
    戦闘解決クラス

    2艦隊間の戦闘を計算し、ダメージを適用する。
    Phase 2: 弾薬消費を追加
    """

    def __init__(self):
        """初期化"""
        self.rng = get_rng()

    def resolve_combat(self, attacker: Fleet, defender: Fleet) -> Tuple[int, int]:
        """
        戦闘を解決

        Args:
            attacker: 攻撃側艦隊
            defender: 防御側艦隊

        Returns:
            (攻撃側が与えたダメージ, 防御側が与えたダメージ)
        """
        # 攻撃側の攻撃
        damage_to_defender = self._attack_fleet(attacker, defender)

        # 防御側の反撃
        damage_to_attacker = self._attack_fleet(defender, attacker)

        # 艦隊状態を更新
        attacker.update_state()
        defender.update_state()

        return (damage_to_defender, damage_to_attacker)

    def _attack_fleet(self, attacker: Fleet, defender: Fleet) -> int:
        """
        艦隊が艦隊を攻撃

        Args:
            attacker: 攻撃側艦隊
            defender: 防御側艦隊

        Returns:
            与えた総ダメージ
        """
        total_damage = 0
        attacker_ships = attacker.get_alive_ships()
        defender_ships = defender.get_alive_ships()

        if not attacker_ships or not defender_ships:
            return 0

        # 各攻撃艦がランダムな防御艦を攻撃
        for attacker_ship in attacker_ships:
            if not defender_ships:
                break

            # Phase 2: 弾薬チェック
            if not attacker_ship.can_shoot():
                print(f"  {attacker_ship.name} out of ammo!")
                continue

            # ランダムにターゲット選択
            target_ship = self.rng.choice(defender_ships)

            # Phase 2: 弾薬消費
            attacker_ship.consume_ammo(attacker_ship.ammo_per_shot)

            # ダメージ計算
            damage = self._calculate_damage(attacker_ship, target_ship)

            # ダメージ適用
            actual_damage = target_ship.take_damage(damage)
            total_damage += actual_damage

            print(
                f"  {attacker_ship.name} -> {target_ship.name}: "
                f"{actual_damage} damage "
                f"(HP: {target_ship.hp + actual_damage} -> {target_ship.hp}, "
                f"Ammo: {attacker_ship.ammo}/{attacker_ship.max_ammo})"
            )

            # 撃沈されたらリストから削除
            if not target_ship.is_alive():
                defender_ships.remove(target_ship)
                print(f"  {target_ship.name} DESTROYED!")

        return total_damage

    def _calculate_damage(self, attacker: Ship, defender: Ship) -> int:
        """
        1艦船対1艦船のダメージ計算

        Args:
            attacker: 攻撃艦
            defender: 防御艦

        Returns:
            ダメージ量
        """
        # 命中判定
        hit_roll = self.rng.random()
        if hit_roll > BASE_HIT_RATE:
            # 外れ
            return 0

        # ダメージ = 攻撃力 - 防御力（最低ダメージ保証）
        damage = max(attacker.attack - defender.defense, MIN_DAMAGE)

        return damage
