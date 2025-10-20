"""
シミュレーションコントローラ

シミュレーション全体を統括する。
Phase 2: System層統合、Planet/StarSystemサポート追加
"""

from typing import List
from src.core.scheduler import EventScheduler
from src.entities.fleet import Fleet
from src.entities.planet import Planet
from src.entities.star_system import StarSystem
from src.events.detection import DetectionEvent
from src.utils.rng import set_global_seed
from src.config.constants import MAX_TICKS
from src.systems.resource_system import ResourceSystem
from src.systems.fleet_system import FleetSystem


class SimulationController:
    """
    シミュレーションコントローラ

    艦隊を登録し、イベントスケジューラを使って
    シミュレーションを実行する。
    Phase 2: System層統合、Planet/StarSystemサポート
    """

    def __init__(self, seed: int = 42):
        """
        初期化

        Args:
            seed: 乱数シード
        """
        self.seed = seed
        set_global_seed(seed)
        self.scheduler = EventScheduler()
        self.fleets: List[Fleet] = []

        # Phase 2: System層
        self.resource_system = ResourceSystem()
        self.fleet_system = FleetSystem()

        # Phase 2: 惑星・星系リスト
        self.star_systems: List[StarSystem] = []
        self.planets: List[Planet] = []

    def add_fleet(self, fleet: Fleet):
        """
        艦隊を追加

        Args:
            fleet: 追加する艦隊
        """
        self.fleets.append(fleet)

    def add_star_system(self, star_system: StarSystem):
        """
        星系を追加（Phase 2追加）

        Args:
            star_system: 追加する星系
        """
        self.star_systems.append(star_system)
        # 星系内の惑星もリストに追加
        for planet in star_system.planets:
            if planet not in self.planets:
                self.planets.append(planet)

    def add_planet(self, planet: Planet):
        """
        惑星を追加（Phase 2追加）

        Args:
            planet: 追加する惑星
        """
        if planet not in self.planets:
            self.planets.append(planet)

    def run(self, max_ticks: int = MAX_TICKS):
        """
        シミュレーションを実行

        Args:
            max_ticks: 最大実行tick数
        """
        print(f"\n{'='*60}")
        print(f"Simulation Start (seed={self.seed})")
        print(f"{'='*60}\n")

        # 初期状態表示
        for fleet in self.fleets:
            print(fleet)
            for ship in fleet.ships:
                print(f"  - {ship}")
        print()

        # 初期イベント登録（Phase 1では索敵イベントのみ）
        if len(self.fleets) >= 2:
            # 簡略化: 最初の2艦隊のみ戦闘
            fleet_a, fleet_b = self.fleets[0], self.fleets[1]
            detection_event = DetectionEvent(tick=1, fleet_a=fleet_a, fleet_b=fleet_b)
            self.scheduler.schedule(detection_event)

        # シミュレーション実行
        tick = 0
        while self.scheduler.has_events() and tick < max_ticks:
            next_tick = self.scheduler.peek_next_tick()
            if next_tick is None:
                break

            # 次のイベントまでジャンプ
            tick = next_tick

            # Phase 2: System更新（毎tick）
            self.resource_system.update(self.planets, tick)
            self.fleet_system.update_consumption(self.fleets, tick)

            # イベント実行
            executed = self.scheduler.execute_tick(tick)

            # 終了判定（Phase 2: 燃料・弾薬枯渇も考慮）
            if self._check_battle_end():
                break

        print(f"\n{'='*60}")
        print(f"Simulation End (tick={tick})")
        print(f"{'='*60}\n")

        # 最終状態表示
        self._print_final_state()

    def _check_battle_end(self) -> bool:
        """
        戦闘終了判定

        Phase 2: 燃料・弾薬枯渇も考慮

        Returns:
            どちらかの艦隊が全滅または戦闘不能ならTrue
        """
        if len(self.fleets) < 2:
            return False

        fleet_a, fleet_b = self.fleets[0], self.fleets[1]

        # 全滅判定
        if fleet_a.is_destroyed() or fleet_b.is_destroyed():
            return True

        # Phase 2: 燃料・弾薬枯渇による戦闘不能
        if not fleet_a.can_fight() or not fleet_b.can_fight():
            return True

        return False

    def _print_final_state(self):
        """最終状態を表示"""
        print("Final State:")
        for fleet in self.fleets:
            print(f"\n{fleet}")
            alive_ships = fleet.get_alive_ships()
            destroyed_ships = [ship for ship in fleet.ships if not ship.is_alive()]

            if alive_ships:
                print("  Surviving ships:")
                for ship in alive_ships:
                    print(f"    - {ship}")

            if destroyed_ships:
                print("  Destroyed ships:")
                for ship in destroyed_ships:
                    print(f"    - {ship}")
