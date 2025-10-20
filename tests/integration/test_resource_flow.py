"""
資源フローの統合テスト（Phase 2）

惑星産出→備蓄→補給→戦闘消費の一連のフローを検証。
"""

import pytest
from src.core.scheduler import EventScheduler
from src.entities.ship import Ship
from src.entities.fleet import Fleet
from src.entities.planet import Planet
from src.entities.star_system import StarSystem
from src.events.resupply import ResupplyEvent
from src.events.combat import CombatEvent
from src.systems.resource_system import ResourceSystem
from src.systems.fleet_system import FleetSystem
from tests.fixtures.scenarios import reset_rng


@pytest.mark.integration
class TestResourceFlow:
    """資源フローの統合テスト"""

    def test_planet_production_to_stock(self):
        """惑星が資源を産出して備蓄に追加"""
        planet = Planet(1, "TestPlanet", (0.0, 0.0, 0.0), fuel_production=1000.0, ammo_production=100)
        resource_system = ResourceSystem()
        
        initial_fuel = planet.fuel_stock
        initial_ammo = planet.ammo_stock
        
        # 10 tick産出
        for tick in range(10):
            resource_system.update([planet], tick)
        
        assert planet.fuel_stock == initial_fuel + 1000.0 * 10
        assert planet.ammo_stock == initial_ammo + 100 * 10

    def test_fleet_fuel_consumption_over_time(self):
        """艦隊が時間経過で燃料消費"""
        fleet = Fleet(1, "TestFleet", (0.0, 0.0, 0.0))
        ship = Ship(1, "destroyer", "Destroyer")
        fleet.add_ship(ship)
        
        fleet_system = FleetSystem()
        initial_fuel = ship.fuel
        
        # 10 tick消費（駆逐艦: 10kg/tick）
        for tick in range(10):
            fleet_system.update_consumption([fleet], tick)
        
        assert ship.fuel == initial_fuel - 10.0 * 10

    def test_resupply_from_planet_to_fleet(self, reset_rng):
        """惑星から艦隊への補給"""
        scheduler = EventScheduler()
        
        planet = Planet(1, "Homeworld", (0.0, 0.0, 0.0), fuel_production=1000.0, ammo_production=100)
        
        fleet = Fleet(1, "TestFleet", (0.0, 0.0, 0.0))
        ship = Ship(1, "destroyer", "Destroyer")
        ship.fuel = 1000.0  # 燃料を減らす
        ship.ammo = 50      # 弾薬を減らす
        fleet.add_ship(ship)
        
        # 補給イベント（満タンまで）
        resupply = ResupplyEvent(tick=1, fleet=fleet, planet=planet, fuel_amount=0, ammo_amount=0)
        scheduler.schedule(resupply)
        
        # イベント実行
        scheduler.run_until(max_tick=5)
        
        # 艦船が補給されている
        assert ship.fuel > 1000.0
        assert ship.ammo > 50

    def test_combat_consumes_ammo(self, reset_rng):
        """戦闘が弾薬を消費する"""
        scheduler = EventScheduler()
        
        fleet_a = Fleet(1, "Fleet-A", (0.0, 0.0, 0.0))
        ship_a = Ship(1, "destroyer", "A-Ship")
        fleet_a.add_ship(ship_a)
        
        fleet_b = Fleet(2, "Fleet-B", (0.0, 0.0, 0.0))
        ship_b = Ship(2, "destroyer", "B-Ship")
        fleet_b.add_ship(ship_b)
        
        initial_ammo_a = ship_a.ammo
        initial_ammo_b = ship_b.ammo
        
        # 戦闘イベント（1ラウンド）
        combat = CombatEvent(tick=1, fleet_a=fleet_a, fleet_b=fleet_b)
        scheduler.schedule(combat)
        
        scheduler.run_until(max_tick=5)
        
        # 弾薬が減っている
        assert ship_a.ammo < initial_ammo_a
        assert ship_b.ammo < initial_ammo_b


@pytest.mark.integration
class TestResourceCycle:
    """資源の循環テスト"""

    def test_full_resource_cycle(self, reset_rng):
        """産出→備蓄→補給→戦闘消費の完全サイクル"""
        scheduler = EventScheduler()
        resource_system = ResourceSystem()
        fleet_system = FleetSystem()
        
        # 惑星
        planet = Planet(1, "Homeworld", (0.0, 0.0, 0.0), fuel_production=2000.0, ammo_production=200)
        
        # 艦隊
        fleet = Fleet(1, "HomeFleet", (0.0, 0.0, 0.0))
        ship = Ship(1, "destroyer", "Destroyer")
        fleet.add_ship(ship)
        
        # 敵艦隊
        enemy_fleet = Fleet(2, "EnemyFleet", (5.0, 0.0, 0.0))
        enemy_ship = Ship(2, "destroyer", "EnemyDestroyer")
        enemy_fleet.add_ship(enemy_ship)
        
        # 初期燃料・弾薬を減らす
        ship.fuel = 1000.0
        ship.ammo = 50
        
        initial_planet_fuel = planet.fuel_stock
        initial_planet_ammo = planet.ammo_stock
        
        # 1. 惑星が資源産出（tick 1-5）
        for tick in range(1, 6):
            resource_system.update([planet], tick)
        
        # 備蓄が増えている
        assert planet.fuel_stock > initial_planet_fuel
        assert planet.ammo_stock > initial_planet_ammo
        
        # 2. 補給イベント（tick 6）
        resupply = ResupplyEvent(tick=6, fleet=fleet, planet=planet)
        scheduler.schedule(resupply)
        scheduler.run_until(max_tick=7)
        
        # 艦船が補給されている
        assert ship.fuel > 1000.0
        assert ship.ammo > 50
        
        # 3. 戦闘（tick 8）
        combat = CombatEvent(tick=8, fleet_a=fleet, fleet_b=enemy_fleet)
        scheduler.schedule(combat)
        scheduler.run_until(max_tick=10)
        
        # 弾薬が消費されている
        assert ship.ammo < ship.max_ammo

    def test_planet_cannot_supply_beyond_stock(self, reset_rng):
        """惑星は備蓄以上に補給できない"""
        scheduler = EventScheduler()
        
        # 備蓄が少ない惑星
        planet = Planet(1, "PoorPlanet", (0.0, 0.0, 0.0), fuel_production=10.0, ammo_production=1)
        planet.fuel_stock = 100.0  # 少ない備蓄
        planet.ammo_stock = 10
        
        # 艦隊（燃料・弾薬が空）
        fleet = Fleet(1, "NeedyFleet", (0.0, 0.0, 0.0))
        ship = Ship(1, "destroyer", "Destroyer")
        ship.fuel = 0.0
        ship.ammo = 0
        fleet.add_ship(ship)
        
        # 補給イベント
        resupply = ResupplyEvent(tick=1, fleet=fleet, planet=planet)
        scheduler.schedule(resupply)
        scheduler.run_until(max_tick=5)
        
        # 惑星の備蓄分しか補給されない
        assert ship.fuel == 100.0  # 惑星の備蓄全額
        assert ship.ammo == 10     # 惑星の備蓄全額
        assert planet.fuel_stock == 0.0
        assert planet.ammo_stock == 0


@pytest.mark.integration
@pytest.mark.slow
class TestResourceStress:
    """資源システムのストレステスト"""

    def test_multiple_fleets_resupply(self, reset_rng):
        """複数艦隊への同時補給"""
        scheduler = EventScheduler()
        
        planet = Planet(1, "Hub", (0.0, 0.0, 0.0), fuel_production=5000.0, ammo_production=500)
        
        fleets = []
        for i in range(5):
            fleet = Fleet(i, f"Fleet-{i}", (0.0, 0.0, 0.0))
            ship = Ship(i, f"Ship-{i}", "destroyer", (0.0, 0.0, 0.0))
            ship.fuel = 500.0
            ship.ammo = 20
            fleet.add_ship(ship)
            fleets.append(fleet)
            
            # 各艦隊に補給イベント
            resupply = ResupplyEvent(tick=1, fleet=fleet, planet=planet)
            scheduler.schedule(resupply)
        
        scheduler.run_until(max_tick=5)
        
        # 全艦隊が補給されている
        for fleet in fleets:
            for ship in fleet.ships:
                assert ship.fuel > 500.0
                assert ship.ammo > 20
