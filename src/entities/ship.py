"""
艦船エンティティ

個々の艦船を表すクラス。
Phase 1: HP・攻撃力・防御力
Phase 2: 燃料・弾薬パラメータ追加
"""

from typing import Literal
from src.config.constants import SHIP_CLASSES, FUEL_UNIT, AMMO_UNIT


ShipClass = Literal["destroyer", "cruiser", "battleship"]


class Ship:
    """
    艦船クラス

    Phase 1: 基本的な戦闘パラメータ
    Phase 2: 燃料・弾薬パラメータ追加
    """

    def __init__(
        self,
        ship_id: int,
        ship_class: ShipClass,
        name: str = "",
    ):
        """
        初期化

        Args:
            ship_id: 艦船ID（一意）
            ship_class: 艦種（destroyer/cruiser/battleship）
            name: 艦船名（オプション）
        """
        self.ship_id = ship_id
        self.ship_class = ship_class
        self.name = name or f"{ship_class.capitalize()}-{ship_id}"

        # 艦種別パラメータを取得
        params = SHIP_CLASSES[ship_class]
        self.max_hp = params["hp"]
        self.hp = self.max_hp  # 現在HP
        self.attack = params["attack"]
        self.defense = params["defense"]
        self.speed = params["speed"]

        # Phase 2: 燃料・弾薬パラメータ
        self.max_fuel = params["max_fuel"]
        self.fuel = self.max_fuel  # 現在燃料（kg）
        self.max_ammo = params["max_ammo"]
        self.ammo = self.max_ammo  # 現在弾薬（発）
        self.fuel_consumption_rate = params["fuel_consumption_rate"]  # kg/tick
        self.ammo_per_shot = params["ammo_per_shot"]  # 発/攻撃

    def is_alive(self) -> bool:
        """
        生存判定

        Returns:
            HPが1以上ならTrue
        """
        return self.hp > 0

    def take_damage(self, damage: int) -> int:
        """
        ダメージを受ける

        Args:
            damage: ダメージ量

        Returns:
            実際に受けたダメージ
        """
        before_hp = self.hp
        self.hp = max(0, self.hp - damage)
        actual_damage = before_hp - self.hp
        return actual_damage

    def consume_fuel(self, amount: float) -> float:
        """
        燃料を消費

        Args:
            amount: 消費量（kg）

        Returns:
            実際に消費した量（kg）
        """
        consumed = min(self.fuel, amount)
        self.fuel -= consumed
        return consumed

    def consume_ammo(self, amount: int) -> int:
        """
        弾薬を消費

        Args:
            amount: 消費量（発）

        Returns:
            実際に消費した量（発）
        """
        consumed = min(self.ammo, amount)
        self.ammo -= consumed
        return consumed

    def refuel(self, amount: float) -> float:
        """
        燃料を補給

        Args:
            amount: 補給量（kg）

        Returns:
            実際に補給した量（kg）
        """
        refueled = min(self.max_fuel - self.fuel, amount)
        self.fuel += refueled
        return refueled

    def rearm(self, amount: int) -> int:
        """
        弾薬を補給

        Args:
            amount: 補給量（発）

        Returns:
            実際に補給した量（発）
        """
        rearmed = min(self.max_ammo - self.ammo, amount)
        self.ammo += rearmed
        return rearmed

    def can_move(self) -> bool:
        """
        移動可能か判定

        Returns:
            燃料があればTrue
        """
        return self.fuel > 0

    def can_shoot(self) -> bool:
        """
        攻撃可能か判定

        Returns:
            弾薬があればTrue
        """
        return self.ammo >= self.ammo_per_shot

    def can_fight(self) -> bool:
        """
        戦闘継続可能か判定

        Returns:
            生存していて弾薬があればTrue
        """
        return self.is_alive() and self.can_shoot()

    def __repr__(self) -> str:
        """文字列表現"""
        status = "ALIVE" if self.is_alive() else "DESTROYED"
        return (
            f"Ship({self.name}, {self.ship_class}, "
            f"HP:{self.hp}/{self.max_hp}, "
            f"Fuel:{self.fuel:.0f}/{self.max_fuel:.0f}{FUEL_UNIT}, "
            f"Ammo:{self.ammo}/{self.max_ammo} {AMMO_UNIT}, "
            f"{status})"
        )
