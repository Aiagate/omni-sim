"""
艦隊エンティティ

複数の艦船をまとめた艦隊を表すクラス。
Phase 2: 星系関連パラメータ追加
"""

from enum import Enum
from typing import List, TYPE_CHECKING, Optional
from src.entities.ship import Ship
from src.utils.geometry import Position
from src.config.constants import BASE_DETECTION_RANGE

if TYPE_CHECKING:
    from src.entities.star_system import StarSystem


class FleetState(Enum):
    """艦隊の状態"""

    SEARCHING = "searching"  # 索敵中
    ENGAGED = "engaged"  # 交戦中
    DESTROYED = "destroyed"  # 全滅


class Fleet:
    """
    艦隊クラス

    複数の艦船を束ねた戦術単位。
    位置・艦船リスト・状態を管理する。
    """

    def __init__(
        self,
        fleet_id: int,
        name: str,
        position: Position,
        ships: List[Ship],
        current_system: Optional["StarSystem"] = None,
    ):
        """
        初期化

        Args:
            fleet_id: 艦隊ID（一意）
            name: 艦隊名
            position: 初期位置 (x, y, z)
            ships: 所属艦船のリスト
            current_system: 現在いる星系（Phase 2追加）
        """
        self.fleet_id = fleet_id
        self.name = name
        self.position = position
        self.ships = ships
        self.state = FleetState.SEARCHING
        self.detection_range = BASE_DETECTION_RANGE

        # Phase 2: 星系関連パラメータ
        self.current_system: Optional["StarSystem"] = current_system
        self.destination_system: Optional["StarSystem"] = None
        self.course: List["StarSystem"] = []  # 航路（星系リスト）
        self.travel_progress: float = 0.0  # 移動進捗（0.0～1.0）

    def get_alive_ships(self) -> List[Ship]:
        """
        生存している艦船のリストを取得

        Returns:
            生存艦船のリスト
        """
        return [ship for ship in self.ships if ship.is_alive()]

    def is_destroyed(self) -> bool:
        """
        艦隊全滅判定

        Returns:
            全艦船が撃沈されていればTrue
        """
        return len(self.get_alive_ships()) == 0

    def total_attack(self) -> int:
        """
        艦隊の総攻撃力を計算

        Returns:
            生存艦船の攻撃力合計
        """
        return sum(ship.attack for ship in self.get_alive_ships())

    def total_defense(self) -> int:
        """
        艦隊の総防御力を計算

        Returns:
            生存艦船の防御力合計
        """
        return sum(ship.defense for ship in self.get_alive_ships())

    def update_state(self):
        """
        艦隊の状態を更新

        全滅していればDESTROYED状態に遷移
        """
        if self.is_destroyed():
            self.state = FleetState.DESTROYED

    def move_to(self, new_position: Position):
        """
        艦隊を移動

        Args:
            new_position: 新しい位置 (x, y, z)
        """
        self.position = new_position

    def can_fight(self) -> bool:
        """
        戦闘継続可能か判定（Phase 2追加）

        Returns:
            生存艦船がいて、少なくとも1隻が弾薬を持っていればTrue
        """
        if self.is_destroyed():
            return False

        # 少なくとも1隻が攻撃可能であれば戦闘継続可
        return any(ship.can_shoot() for ship in self.get_alive_ships())

    def get_ships_with_ammo(self) -> List[Ship]:
        """
        弾薬を持つ艦船のリストを取得（Phase 2追加）

        Returns:
            弾薬があり攻撃可能な艦船のリスト
        """
        return [ship for ship in self.get_alive_ships() if ship.can_shoot()]

    def total_fuel(self) -> float:
        """
        艦隊の総燃料残量を計算（Phase 2追加）

        Returns:
            生存艦船の燃料合計（kg）
        """
        return sum(ship.fuel for ship in self.get_alive_ships())

    def total_ammo(self) -> int:
        """
        艦隊の総弾薬残量を計算（Phase 2追加）

        Returns:
            生存艦船の弾薬合計（発）
        """
        return sum(ship.ammo for ship in self.get_alive_ships())

    def __repr__(self) -> str:
        """文字列表現"""
        alive_count = len(self.get_alive_ships())
        total_count = len(self.ships)

        system_name = self.current_system.name if self.current_system else "Deep Space"

        return (
            f"Fleet({self.name}, "
            f"System:{system_name}, "
            f"Ships:{alive_count}/{total_count}, "
            f"State:{self.state.value})"
        )
