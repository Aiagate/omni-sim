"""
惑星エンティティ

Phase 2: 簡易版（固定値）
資源産出と備蓄を管理する。
"""

from typing import Tuple
from src.config.constants import FUEL_UNIT, AMMO_UNIT

Position = Tuple[float, float, float]


class Planet:
    """
    惑星エンティティ（簡易版）

    Phase 2では固定値の資源産出のみ。
    Phase 3で人口・技術レベル・エネルギー収支を追加予定。
    """

    def __init__(
        self,
        planet_id: int,
        name: str,
        position: Position,
        fuel_production: float = 1000.0,
        ammo_production: int = 100,
    ):
        """
        初期化

        Args:
            planet_id: 惑星ID
            name: 惑星名
            position: 惑星座標（星系内ローカル座標）
            fuel_production: 燃料産出量（kg/tick）
            ammo_production: 弾薬産出量（発/tick）
        """
        self.planet_id = planet_id
        self.name = name
        self.position = position

        # 資源産出（固定値）
        self.fuel_production = fuel_production  # kg/tick
        self.ammo_production = ammo_production  # 発/tick

        # 備蓄（初期値は産出量の100倍）
        self.fuel_stock = fuel_production * 100.0
        self.ammo_stock = ammo_production * 100

    def produce_resources(self):
        """
        資源を産出して備蓄に追加

        毎tick呼び出されることを想定。
        """
        self.fuel_stock += self.fuel_production
        self.ammo_stock += self.ammo_production

    def consume_fuel(self, amount: float) -> bool:
        """
        燃料を消費

        Args:
            amount: 消費量（kg）

        Returns:
            消費に成功したらTrue、備蓄不足ならFalse
        """
        if self.fuel_stock >= amount:
            self.fuel_stock -= amount
            return True
        else:
            return False

    def consume_ammo(self, amount: int) -> bool:
        """
        弾薬を消費

        Args:
            amount: 消費量（発）

        Returns:
            消費に成功したらTrue、備蓄不足ならFalse
        """
        if self.ammo_stock >= amount:
            self.ammo_stock -= amount
            return True
        else:
            return False

    def __repr__(self) -> str:
        return (
            f"Planet({self.name}, "
            f"Fuel:{self.fuel_stock:.0f}{FUEL_UNIT}, "
            f"Ammo:{self.ammo_stock} {AMMO_UNIT})"
        )
