"""
索敵イベント

艦隊が敵を索敵し、発見した場合に戦闘イベントを発生させる。
"""

from src.events.base import Event
from src.entities.fleet import Fleet, FleetState
from src.utils.geometry import is_within_range
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from src.core.scheduler import EventScheduler


class DetectionEvent(Event):
    """
    索敵イベント

    艦隊が周囲を索敵し、敵艦隊を発見すると戦闘イベントを登録する。
    """

    def __init__(self, tick: int, fleet_a: Fleet, fleet_b: Fleet):
        """
        初期化

        Args:
            tick: イベント発火時刻
            fleet_a: 艦隊A
            fleet_b: 艦隊B
        """
        super().__init__(tick, priority=0)  # 索敵は高優先度
        self.fleet_a = fleet_a
        self.fleet_b = fleet_b

    def execute(self, scheduler: "EventScheduler"):
        """
        索敵を実行

        両艦隊が生存していて、索敵範囲内にいれば戦闘イベントを登録。
        """
        # 両艦隊が生存しているか確認
        if self.fleet_a.is_destroyed() or self.fleet_b.is_destroyed():
            print(f"  One or both fleets destroyed. No detection.")
            return

        # 距離計算
        in_range = is_within_range(
            self.fleet_a.position,
            self.fleet_b.position,
            self.fleet_a.detection_range,
        )

        if in_range:
            print(
                f"  {self.fleet_a.name} detected {self.fleet_b.name}! "
                f"Initiating combat..."
            )

            # 両艦隊を交戦状態に変更
            self.fleet_a.state = FleetState.ENGAGED
            self.fleet_b.state = FleetState.ENGAGED

            # 戦闘イベントを次のtickで登録
            from src.events.combat import CombatEvent

            combat_event = CombatEvent(
                tick=self.tick + 1, fleet_a=self.fleet_a, fleet_b=self.fleet_b
            )
            scheduler.schedule(combat_event)
        else:
            print(
                f"  {self.fleet_a.name} searching... "
                f"{self.fleet_b.name} not detected."
            )

    def __repr__(self) -> str:
        return (
            f"DetectionEvent(tick={self.tick}, "
            f"{self.fleet_a.name} vs {self.fleet_b.name})"
        )
