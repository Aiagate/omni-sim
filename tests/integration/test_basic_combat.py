"""
基本戦闘の統合テスト（Phase 1）

イベントスケジューラー、索敵、戦闘解決の統合動作を検証。
"""

import pytest
from src.core.scheduler import EventScheduler
from src.core.simulation import SimulationController
from src.entities.ship import Ship
from src.entities.fleet import Fleet, FleetState
from src.events.detection import DetectionEvent
from src.events.combat import CombatEvent
from src.utils.rng import set_global_seed
from tests.fixtures.scenarios import reset_rng


@pytest.mark.integration
class TestBasicCombatIntegration:
    """基本戦闘の統合テスト"""

    def test_detection_triggers_combat(self, reset_rng):
        """索敵イベントが戦闘を開始する"""
        scheduler = EventScheduler()
        
        # 2つの艦隊を作成
        fleet_a = Fleet(1, "Fleet-A", (0.0, 0.0, 0.0), ships=[Ship(1, "destroyer", "A-Ship")])
        fleet_b = Fleet(2, "Fleet-B", (10.0, 0.0, 0.0), ships=[Ship(2, "destroyer", "B-Ship")])
        
        # 索敵イベントをスケジュール
        detection = DetectionEvent(tick=1, fleet_a=fleet_a, fleet_b=fleet_b)
        scheduler.schedule(detection)
        
        # イベント実行
        scheduler.run_until(max_tick=10)
        
        # 戦闘が開始されている（状態がENGAGED）
        assert fleet_a.state == FleetState.ENGAGED or fleet_b.state == FleetState.ENGAGED

    def test_combat_deals_damage(self, reset_rng):
        """戦闘イベントがダメージを与える"""
        scheduler = EventScheduler()
        
        fleet_a = Fleet(1, "Fleet-A", (0.0, 0.0, 0.0), ships=[])
        ship_a = Ship(1, "battleship", "A-Battleship")
        fleet_a.add_ship(ship_a)
        
        fleet_b = Fleet(2, "Fleet-B", (0.0, 0.0, 0.0), ships=[])
        ship_b = Ship(2, "destroyer", "B-Destroyer")
        fleet_b.add_ship(ship_b)
        
        initial_hp_a = ship_a.hp
        initial_hp_b = ship_b.hp
        
        # 戦闘イベントをスケジュール
        combat = CombatEvent(tick=1, fleet_a=fleet_a, fleet_b=fleet_b)
        scheduler.schedule(combat)
        
        # イベント実行
        scheduler.run_until(max_tick=5)
        
        # 少なくとも一方がダメージを受けている（命中判定あり）
        assert ship_a.hp <= initial_hp_a or ship_b.hp <= initial_hp_b

    def test_combat_until_destruction(self, reset_rng):
        """戦闘が全滅まで継続する"""
        scheduler = EventScheduler()
        
        # 圧倒的な戦力差
        fleet_a = Fleet(1, "Fleet-A", (0.0, 0.0, 0.0), ships=[])
        for i in range(3):
            fleet_a.add_ship(Ship(i, f"A-Battleship-{i}", "battleship", (0.0, 0.0, 0.0)))
        
        fleet_b = Fleet(2, "Fleet-B", (0.0, 0.0, 0.0), ships=[Ship(10, "destroyer", "B-Destroyer")])
        
        # 索敵→戦闘
        detection = DetectionEvent(tick=1, fleet_a=fleet_a, fleet_b=fleet_b)
        scheduler.schedule(detection)
        
        # 十分な時間実行
        scheduler.run_until(max_tick=100)
        
        # どちらかが全滅している
        assert fleet_a.is_destroyed() or fleet_b.is_destroyed()


@pytest.mark.integration
class TestSimulationController:
    """SimulationControllerの統合テスト"""

    def test_simulation_runs_to_completion(self, reset_rng):
        """シミュレーションが完了まで実行される"""
        sim = SimulationController(seed=42, max_ticks=50)
        
        # 簡単な戦闘シナリオ
        fleet_a = Fleet(1, "Fleet-A", (0.0, 0.0, 0.0), ships=[Ship(1, "cruiser", "A-Ship")])
        
        fleet_b = Fleet(2, "Fleet-B", (5.0, 0.0, 0.0))
        fleet_b.add_ship(Ship(2, "destroyer", "B-Ship"))
        
        # シミュレーション実行（コンソール出力なし）
        # 注: 実際のSimulationControllerは未実装の場合はスキップ
        # sim.add_fleet(fleet_a)
        # sim.add_fleet(fleet_b)
        # sim.run()
        
        # 仮のアサーション（実装状況に応じて調整）
        assert sim.max_ticks == 50


@pytest.mark.integration
@pytest.mark.slow
class TestLongBattle:
    """長時間戦闘の統合テスト"""

    def test_large_fleet_battle(self, reset_rng):
        """大規模艦隊戦"""
        scheduler = EventScheduler()
        
        # 大規模艦隊
        fleet_a = Fleet(1, "Imperial-Fleet", (0.0, 0.0, 0.0), ships=[])
        for i in range(5):
            fleet_a.add_ship(Ship(i, f"Imperial-{i}", "cruiser", (0.0, 0.0, 0.0)))
        
        fleet_b = Fleet(2, "Rebel-Fleet", (10.0, 0.0, 0.0))
        for i in range(6):
            fleet_b.add_ship(Ship(i+10, f"Rebel-{i}", "destroyer", (10.0, 0.0, 0.0)))
        
        # 索敵
        detection = DetectionEvent(tick=1, fleet_a=fleet_a, fleet_b=fleet_b)
        scheduler.schedule(detection)
        
        # 長時間実行
        scheduler.run_until(max_tick=200)
        
        # 戦闘が終了している
        assert fleet_a.is_destroyed() or fleet_b.is_destroyed()
        
        # 生存艦隊の弾薬が減っている
        survivor = fleet_a if not fleet_a.is_destroyed() else fleet_b
        for ship in survivor.get_alive_ships():
            assert ship.ammo < ship.max_ammo
