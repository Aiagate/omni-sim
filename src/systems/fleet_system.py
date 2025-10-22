"""
艦隊管理システム

艦隊の燃料・弾薬消費を一括処理する。
Phase 2: 固定消費率
"""

from typing import List
from src.entities.fleet import Fleet


class FleetSystem:
    """
    艦隊管理システム

    全艦隊の燃料消費を一括処理。
    Phase 3で移動距離に応じた燃料消費を追加予定。
    """

    def __init__(self):
        """初期化"""
        pass

    def update_consumption(self, fleets: List[Fleet], tick: int):
        """
        全艦隊の燃料を消費

        Args:
            fleets: 更新対象の艦隊リスト
            tick: 現在のtick
        """
        for fleet in fleets:
            self.update_fleet_consumption(fleet, tick)

    def update_fleet_consumption(self, fleet: Fleet, tick: int):
        """
        単一艦隊の燃料を消費

        Args:
            fleet: 更新対象の艦隊
            tick: 現在のtick
        """
        for ship in fleet.get_alive_ships():
            # 燃料消費（Phase 2では固定レート）
            ship.consume_fuel(ship.fuel_consumption_rate)
