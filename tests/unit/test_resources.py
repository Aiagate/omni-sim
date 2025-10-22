"""
資源管理システムの単体テスト（Phase 2）

惑星の資源産出、備蓄管理、補給システムをテスト。
"""

import pytest
from src.entities.planet import Planet
from src.entities.star_system import StarSystem
from src.entities.ship import Ship
from src.entities.fleet import Fleet
from src.systems.resource_system import ResourceSystem
from src.systems.fleet_system import FleetSystem
from tests.fixtures.scenarios import sample_planet, sample_star_system


@pytest.mark.unit
class TestPlanet:
    """惑星の資源管理テスト"""

    def test_planet_creation(self, sample_planet):
        """惑星の生成"""
        assert sample_planet.name == "Test-Planet"
        assert sample_planet.fuel_production == 1000.0
        assert sample_planet.ammo_production == 100

    def test_initial_stock(self, sample_planet):
        """初期備蓄は産出量の100倍"""
        assert sample_planet.fuel_stock == 100000.0  # 1000 * 100
        assert sample_planet.ammo_stock == 10000  # 100 * 100

    def test_produce_resources(self, sample_planet):
        """資源産出"""
        initial_fuel = sample_planet.fuel_stock
        initial_ammo = sample_planet.ammo_stock
        
        sample_planet.produce_resources()
        
        assert sample_planet.fuel_stock == initial_fuel + 1000.0
        assert sample_planet.ammo_stock == initial_ammo + 100

    def test_consume_fuel_success(self, sample_planet):
        """燃料消費（成功）"""
        initial = sample_planet.fuel_stock
        success = sample_planet.consume_fuel(5000.0)
        
        assert success is True
        assert sample_planet.fuel_stock == initial - 5000.0

    def test_consume_fuel_failure(self, sample_planet):
        """燃料消費（備蓄不足）"""
        initial = sample_planet.fuel_stock
        success = sample_planet.consume_fuel(1000000.0)  # 備蓄以上
        
        assert success is False
        assert sample_planet.fuel_stock == initial  # 変化なし

    def test_consume_ammo_success(self, sample_planet):
        """弾薬消費（成功）"""
        initial = sample_planet.ammo_stock
        success = sample_planet.consume_ammo(500)
        
        assert success is True
        assert sample_planet.ammo_stock == initial - 500

    def test_consume_ammo_failure(self, sample_planet):
        """弾薬消費（備蓄不足）"""
        initial = sample_planet.ammo_stock
        success = sample_planet.consume_ammo(100000)  # 備蓄以上
        
        assert success is False
        assert sample_planet.ammo_stock == initial  # 変化なし


@pytest.mark.unit
class TestResourceSystem:
    """ResourceSystemのテスト"""

    def test_resource_system_update(self, sample_planet):
        """ResourceSystemによる資源産出"""
        system = ResourceSystem()
        initial_fuel = sample_planet.fuel_stock
        initial_ammo = sample_planet.ammo_stock
        
        system.update([sample_planet], tick=1)
        
        assert sample_planet.fuel_stock == initial_fuel + sample_planet.fuel_production
        assert sample_planet.ammo_stock == initial_ammo + sample_planet.ammo_production

    def test_resource_system_multiple_planets(self):
        """複数惑星の一括更新"""
        planet1 = Planet(1, "P1", (0.0, 0.0, 0.0), 500.0, 50)
        planet2 = Planet(2, "P2", (0.0, 0.0, 0.0), 1500.0, 150)
        
        system = ResourceSystem()
        system.update([planet1, planet2], tick=1)
        
        # 各惑星が産出
        assert planet1.fuel_stock == 50000.0 + 500.0
        assert planet2.fuel_stock == 150000.0 + 1500.0


@pytest.mark.unit
class TestFleetSystem:
    """FleetSystemのテスト"""

    def test_fleet_fuel_consumption(self):
        """艦隊の燃料消費"""
        ship = Ship(1, "destroyer", "Destroyer-1")
        fleet = Fleet(1, "Test-Fleet", (0.0, 0.0, 0.0), ships=[ship])

        system = FleetSystem()
        initial_fuel = ship.fuel

        system.update_consumption([fleet], tick=1)

        # 駆逐艦の燃料消費率: 10kg/tick
        assert ship.fuel == initial_fuel - 10.0

    def test_multiple_fleet_consumption(self):
        """複数艦隊の一括燃料消費"""
        fleet1 = Fleet(1, "Fleet-1", (0.0, 0.0, 0.0), ships=[Ship(1, "destroyer", "DD-1")])
        
        fleet2 = Fleet(2, "Fleet-2", (0.0, 0.0, 0.0), ships=[Ship(2, "cruiser", "CA-1")])
        
        system = FleetSystem()
        system.update_consumption([fleet1, fleet2], tick=1)
        
        # 駆逐艦: -10kg, 巡洋艦: -20kg
        assert fleet1.ships[0].fuel == 5000.0 - 10.0
        assert fleet2.ships[0].fuel == 20000.0 - 20.0


@pytest.mark.unit
class TestStarSystem:
    """星系の管理テスト"""

    def test_star_system_creation(self, sample_star_system):
        """星系の生成"""
        assert sample_star_system.name == "Test-System"
        assert len(sample_star_system.planets) == 1
        assert len(sample_star_system.fleets_present) == 1

    def test_add_planet(self):
        """惑星の追加"""
        system = StarSystem(1, "System", (0.0, 0.0, 0.0))
        planet = Planet(1, "Planet", (0.0, 0.0, 0.0))
        
        system.add_planet(planet)
        assert len(system.planets) == 1
        assert system.planets[0] == planet

    def test_add_fleet(self):
        """艦隊の追加"""
        system = StarSystem(1, "System", (0.0, 0.0, 0.0))
        fleet = Fleet(1, "Fleet", (0.0, 0.0, 0.0), ships=[])
        
        system.add_fleet(fleet)
        assert len(system.fleets_present) == 1
        assert system.fleets_present[0] == fleet

    def test_remove_fleet(self, sample_star_system):
        """艦隊の削除"""
        initial_count = len(sample_star_system.fleets_present)
        fleet = sample_star_system.fleets_present[0]
        
        sample_star_system.remove_fleet(fleet)
        assert len(sample_star_system.fleets_present) == initial_count - 1


@pytest.mark.unit
class TestShipRefuelRearm:
    """艦船の補給テスト"""

    def test_ship_refuel(self):
        """燃料補給"""
        ship = Ship(1, "destroyer", "Test")
        ship.fuel = 1000.0
        
        refueled = ship.refuel(2000.0)
        
        assert refueled == 2000.0
        assert ship.fuel == 3000.0

    def test_ship_refuel_max(self):
        """燃料補給（最大値制限）"""
        ship = Ship(1, "destroyer", "Test")
        ship.fuel = 4500.0
        
        refueled = ship.refuel(1000.0)
        
        # 4500 + 500 = 5000（最大）
        assert refueled == 500.0
        assert ship.fuel == 5000.0

    def test_ship_rearm(self):
        """弾薬補給"""
        ship = Ship(1, "destroyer", "Test")
        ship.ammo = 50
        
        rearmed = ship.rearm(100)
        
        assert rearmed == 100
        assert ship.ammo == 150

    def test_ship_rearm_max(self):
        """弾薬補給（最大値制限）"""
        ship = Ship(1, "destroyer", "Test")
        ship.ammo = 180
        
        rearmed = ship.rearm(50)
        
        # 180 + 20 = 200（最大）
        assert rearmed == 20
        assert ship.ammo == 200
