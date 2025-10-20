"""
戦闘イベント

2艦隊間の戦闘を実行する。
"""

from src.events.base import Event
from src.entities.fleet import Fleet
from src.combat.resolver import CombatResolver
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from src.core.scheduler import EventScheduler


class CombatEvent(Event):
    """
    戦闘イベント

    2艦隊が交戦し、ダメージを与え合う。
    戦闘が継続する場合は次のtickに新たな戦闘イベントを登録。
    """

    def __init__(self, tick: int, fleet_a: Fleet, fleet_b: Fleet):
        """
        初期化

        Args:
            tick: イベント発火時刻
            fleet_a: 艦隊A
            fleet_b: 艦隊B
        """
        super().__init__(tick, priority=1)  # 戦闘は中優先度
        self.fleet_a = fleet_a
        self.fleet_b = fleet_b
        self.resolver = CombatResolver()

    def execute(self, scheduler: "EventScheduler"):
        """
        戦闘を実行

        戦闘解決を行い、どちらかの艦隊が全滅するまで継続。
        """
        print(f"\n=== COMBAT: {self.fleet_a.name} vs {self.fleet_b.name} ===")

        # 両艦隊が生存しているか確認
        if self.fleet_a.is_destroyed():
            print(f"  {self.fleet_a.name} is destroyed. Combat ends.")
            return

        if self.fleet_b.is_destroyed():
            print(f"  {self.fleet_b.name} is destroyed. Combat ends.")
            return

        # 戦闘解決
        damage_to_b, damage_to_a = self.resolver.resolve_combat(
            self.fleet_a, self.fleet_b
        )

        print(f"  Total damage: {self.fleet_a.name} dealt {damage_to_b}")
        print(f"  Total damage: {self.fleet_b.name} dealt {damage_to_a}")

        # 戦闘後の状態表示
        print(f"  {self.fleet_a}")
        print(f"  {self.fleet_b}")

        # どちらかが全滅していれば戦闘終了
        if self.fleet_a.is_destroyed():
            print(f"\n*** {self.fleet_b.name} VICTORY! ***")
            return

        if self.fleet_b.is_destroyed():
            print(f"\n*** {self.fleet_a.name} VICTORY! ***")
            return

        # 戦闘継続 - 次のtickで再度戦闘
        next_combat = CombatEvent(
            tick=self.tick + 1, fleet_a=self.fleet_a, fleet_b=self.fleet_b
        )
        scheduler.schedule(next_combat)

    def __repr__(self) -> str:
        return (
            f"CombatEvent(tick={self.tick}, "
            f"{self.fleet_a.name} vs {self.fleet_b.name})"
        )
