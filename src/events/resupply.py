"""
補給イベント

惑星から艦隊へ燃料・弾薬を補給する。
Phase 2: 即座補給（輸送時間なし）
"""

from src.events.base import Event
from src.entities.fleet import Fleet
from src.entities.planet import Planet
from src.config.constants import FUEL_UNIT, AMMO_UNIT
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from src.core.scheduler import EventScheduler


class ResupplyEvent(Event):
    """
    補給イベント

    惑星から艦隊へ燃料・弾薬を転送する。
    Phase 2では同一星系内であれば即座に補給。
    Phase 3で輸送船団を使った補給を実装予定。
    """

    def __init__(
        self,
        tick: int,
        fleet: Fleet,
        planet: Planet,
        fuel_amount: float = 0.0,
        ammo_amount: int = 0,
    ):
        """
        初期化

        Args:
            tick: イベント発火時刻
            fleet: 補給を受ける艦隊
            planet: 補給元の惑星
            fuel_amount: 補給する燃料量（kg）、0なら満タンまで
            ammo_amount: 補給する弾薬量（発）、0なら満タンまで
        """
        super().__init__(tick, priority=2)  # 補給は低優先度
        self.fleet = fleet
        self.planet = planet
        self.fuel_amount = fuel_amount
        self.ammo_amount = ammo_amount

    def execute(self, scheduler: "EventScheduler"):
        """
        補給を実行

        惑星の備蓄から艦隊へ燃料・弾薬を転送。
        """
        # 艦隊が全滅していれば補給不要
        if self.fleet.is_destroyed():
            print(f"  {self.fleet.name} is destroyed. Resupply cancelled.")
            return

        # Phase 2: 同一星系判定は省略（簡易版）
        # Phase 3で星系判定を追加予定

        total_fuel_resupplied = 0.0
        total_ammo_resupplied = 0
        remaining_fuel_quota = self.fuel_amount if self.fuel_amount > 0 else None
        remaining_ammo_quota = self.ammo_amount if self.ammo_amount > 0 else None

        print(f"\n=== RESUPPLY: {self.planet.name} -> {self.fleet.name} ===")

        # 各艦船に補給
        for ship in self.fleet.get_alive_ships():
            # 燃料補給
            if ship.fuel < ship.max_fuel:
                fuel_to_supply = 0.0
                fuel_quota_exhausted = (
                    remaining_fuel_quota is not None and remaining_fuel_quota <= 0
                )

                if not fuel_quota_exhausted:
                    fuel_needed = ship.max_fuel - ship.fuel

                    if remaining_fuel_quota is not None:
                        # 指定量を補給（イベント単位の残量で制限）
                        fuel_to_supply = min(
                            fuel_needed, remaining_fuel_quota, self.planet.fuel_stock
                        )
                    else:
                        # 満タンまで補給
                        fuel_to_supply = min(fuel_needed, self.planet.fuel_stock)

                if fuel_to_supply > 0:
                    # 惑星から消費
                    if self.planet.consume_fuel(fuel_to_supply):
                        # 艦船に補給
                        actual_refueled = ship.refuel(fuel_to_supply)
                        total_fuel_resupplied += actual_refueled
                        if remaining_fuel_quota is not None:
                            remaining_fuel_quota = max(
                                0.0, remaining_fuel_quota - actual_refueled
                            )

            # 弾薬補給
            if ship.ammo < ship.max_ammo:
                ammo_to_supply = 0
                ammo_quota_exhausted = (
                    remaining_ammo_quota is not None and remaining_ammo_quota <= 0
                )

                if not ammo_quota_exhausted:
                    ammo_needed = ship.max_ammo - ship.ammo

                    if remaining_ammo_quota is not None:
                        # 指定量を補給（イベント単位の残量で制限）
                        ammo_to_supply = min(
                            ammo_needed, remaining_ammo_quota, self.planet.ammo_stock
                        )
                    else:
                        # 満タンまで補給
                        ammo_to_supply = min(ammo_needed, self.planet.ammo_stock)

                if ammo_to_supply > 0:
                    # 惑星から消費
                    if self.planet.consume_ammo(ammo_to_supply):
                        # 艦船に補給
                        actual_rearmed = ship.rearm(ammo_to_supply)
                        total_ammo_resupplied += actual_rearmed
                        if remaining_ammo_quota is not None:
                            remaining_ammo_quota = max(
                                0, remaining_ammo_quota - actual_rearmed
                            )

        print(f"  Fuel resupplied: {total_fuel_resupplied:.0f}{FUEL_UNIT}")
        print(f"  Ammo resupplied: {total_ammo_resupplied} {AMMO_UNIT}")
        print(f"  {self.planet}")
        print(f"  Fleet total fuel: {self.fleet.total_fuel():.0f}{FUEL_UNIT}")
        print(f"  Fleet total ammo: {self.fleet.total_ammo()} {AMMO_UNIT}")

    def __repr__(self) -> str:
        return (
            f"ResupplyEvent(tick={self.tick}, "
            f"{self.planet.name} -> {self.fleet.name})"
        )
