"""
艦隊（Fleet）クラスの単体テスト

Phase 1: 艦隊管理、状態遷移
Phase 2: 資源集計、戦闘可能判定
"""

import pytest
from src.entities.ship import Ship
from src.entities.fleet import Fleet, FleetState
from tests.fixtures.scenarios import reset_rng, sample_fleet, sample_mixed_fleet


@pytest.mark.unit
class TestFleetCreation:
    """艦隊の生成テスト"""

    def test_empty_fleet_creation(self):
        """空の艦隊を生成"""
        fleet = Fleet(1, "Test-Fleet", (0.0, 0.0, 0.0), ships=[])
        assert fleet.name == "Test-Fleet"
        assert fleet.state == FleetState.SEARCHING
        assert len(fleet.ships) == 0

    def test_add_ship(self):
        """艦船を追加（ships配列への直接追加）"""
        ship = Ship(1, "destroyer", "Destroyer-1")
        fleet = Fleet(1, "Test-Fleet", (0.0, 0.0, 0.0), ships=[ship])
        assert len(fleet.ships) == 1
        assert fleet.ships[0] == ship

    def test_add_multiple_ships(self, sample_mixed_fleet):
        """複数艦船を追加"""
        assert len(sample_mixed_fleet.ships) == 3


@pytest.mark.unit
class TestFleetState:
    """艦隊の状態管理テスト"""

    def test_initial_state(self, sample_fleet):
        """初期状態はSEARCHING"""
        assert sample_fleet.state == FleetState.SEARCHING

    def test_state_transition_to_engaged(self, sample_fleet):
        """ENGAGED状態への遷移"""
        sample_fleet.state = FleetState.ENGAGED
        assert sample_fleet.state == FleetState.ENGAGED

    def test_state_transition_to_destroyed(self, sample_fleet):
        """DESTROYED状態への遷移"""
        sample_fleet.state = FleetState.DESTROYED
        assert sample_fleet.state == FleetState.DESTROYED


@pytest.mark.unit
class TestFleetShipManagement:
    """艦隊の艦船管理テスト"""

    def test_get_alive_ships(self, sample_fleet):
        """生存艦船の取得"""
        alive = sample_fleet.get_alive_ships()
        assert len(alive) == 2

    def test_get_alive_ships_after_destruction(self, sample_fleet):
        """一部撃破後の生存艦船"""
        sample_fleet.ships[0].take_damage(1000)  # 1隻撃破
        alive = sample_fleet.get_alive_ships()
        assert len(alive) == 1

    def test_is_destroyed_false(self, sample_fleet):
        """全滅していない"""
        assert sample_fleet.is_destroyed() is False

    def test_is_destroyed_true(self, sample_fleet):
        """全滅判定"""
        for ship in sample_fleet.ships:
            ship.take_damage(1000)
        assert sample_fleet.is_destroyed() is True

    def test_count_alive(self, sample_fleet):
        """生存艦船数のカウント（len(get_alive_ships())で代替）"""
        assert len(sample_fleet.get_alive_ships()) == 2
        sample_fleet.ships[0].take_damage(1000)
        assert len(sample_fleet.get_alive_ships()) == 1


@pytest.mark.unit
class TestFleetResources:
    """艦隊の資源管理テスト（Phase 2）"""

    def test_total_fuel(self, sample_fleet):
        """艦隊の総燃料"""
        # 駆逐艦2隻 × 5000kg = 10000kg
        assert sample_fleet.total_fuel() == 10000.0

    def test_total_ammo(self, sample_fleet):
        """艦隊の総弾薬"""
        # 駆逐艦2隻 × 200発 = 400発
        assert sample_fleet.total_ammo() == 400

    def test_total_fuel_after_consumption(self, sample_fleet):
        """燃料消費後の総燃料"""
        sample_fleet.ships[0].consume_fuel(1000.0)
        assert sample_fleet.total_fuel() == 9000.0

    def test_total_ammo_after_consumption(self, sample_fleet):
        """弾薬消費後の総弾薬"""
        sample_fleet.ships[0].consume_ammo(50)
        assert sample_fleet.total_ammo() == 350

    def test_can_fight_with_ammo(self, sample_fleet):
        """弾薬があれば戦闘可能"""
        assert sample_fleet.can_fight() is True

    def test_cannot_fight_without_ammo(self, sample_fleet):
        """弾薬がなければ戦闘不可"""
        for ship in sample_fleet.ships:
            ship.ammo = 0
        assert sample_fleet.can_fight() is False

    def test_can_fight_with_partial_ammo(self, sample_fleet):
        """一部の艦船に弾薬があれば戦闘可能"""
        sample_fleet.ships[0].ammo = 0
        sample_fleet.ships[1].ammo = 100
        assert sample_fleet.can_fight() is True

    def test_cannot_fight_when_destroyed(self, sample_fleet):
        """全滅していれば戦闘不可"""
        for ship in sample_fleet.ships:
            ship.take_damage(1000)
        assert sample_fleet.can_fight() is False


@pytest.mark.unit
class TestFleetMixedComposition:
    """混成艦隊のテスト"""

    def test_mixed_fleet_total_fuel(self, sample_mixed_fleet):
        """混成艦隊の総燃料"""
        # 戦艦: 100000kg + 巡洋艦: 20000kg + 駆逐艦: 5000kg = 125000kg
        assert sample_mixed_fleet.total_fuel() == 125000.0

    def test_mixed_fleet_total_ammo(self, sample_mixed_fleet):
        """混成艦隊の総弾薬"""
        # 戦艦: 1000発 + 巡洋艦: 400発 + 駆逐艦: 200発 = 1600発
        assert sample_mixed_fleet.total_ammo() == 1600

    def test_mixed_fleet_partial_destruction(self, sample_mixed_fleet):
        """混成艦隊の一部撃破"""
        # 駆逐艦のみ撃破
        sample_mixed_fleet.ships[2].take_damage(1000)
        alive = sample_mixed_fleet.get_alive_ships()
        assert len(alive) == 2
        assert sample_mixed_fleet.is_destroyed() is False
