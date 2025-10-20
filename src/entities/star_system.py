"""
星系エンティティ

Phase 2: 簡易版
星系内の惑星・艦隊を管理する。
"""

from typing import List, Tuple, TYPE_CHECKING

if TYPE_CHECKING:
    from src.entities.planet import Planet
    from src.entities.fleet import Fleet

Position = Tuple[float, float, float]


class StarSystem:
    """
    星系エンティティ

    Phase 2では星系内の惑星・艦隊を管理。
    Phase 3でNetworkXによる銀河グラフに統合予定。
    """

    def __init__(
        self,
        system_id: int,
        name: str,
        position: Position,
        planets: List["Planet"] = None,
    ):
        """
        初期化

        Args:
            system_id: 星系ID
            name: 星系名
            position: 銀河座標（光年単位）
            planets: この星系に属する惑星リスト
        """
        self.system_id = system_id
        self.name = name
        self.position = position  # 銀河座標

        # 星系内の惑星
        self.planets: List["Planet"] = planets if planets else []

        # 星系内にいる艦隊（動的に更新される）
        self.fleets_present: List["Fleet"] = []

    def add_planet(self, planet: "Planet"):
        """
        惑星を追加

        Args:
            planet: 追加する惑星
        """
        self.planets.append(planet)

    def add_fleet(self, fleet: "Fleet"):
        """
        艦隊を星系内に追加

        Args:
            fleet: 追加する艦隊
        """
        if fleet not in self.fleets_present:
            self.fleets_present.append(fleet)

    def remove_fleet(self, fleet: "Fleet"):
        """
        艦隊を星系から除外

        Args:
            fleet: 除外する艦隊
        """
        if fleet in self.fleets_present:
            self.fleets_present.remove(fleet)

    def get_total_fuel_production(self) -> float:
        """
        星系全体の燃料産出量を取得

        Returns:
            燃料産出量の合計（kg/tick）
        """
        return sum(planet.fuel_production for planet in self.planets)

    def get_total_ammo_production(self) -> int:
        """
        星系全体の弾薬産出量を取得

        Returns:
            弾薬産出量の合計（発/tick）
        """
        return sum(planet.ammo_production for planet in self.planets)

    def __repr__(self) -> str:
        return (
            f"StarSystem({self.name}, "
            f"Planets:{len(self.planets)}, "
            f"Fleets:{len(self.fleets_present)})"
        )
