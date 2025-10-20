"""
艦船（Ship）クラスの単体テスト

Phase 1: 基本パラメータ、戦闘機能
Phase 2: 燃料・弾薬管理、補給機能
"""

import pytest
from src.entities.ship import Ship
from src.config.constants import SHIP_CLASSES
from tests.fixtures.scenarios import reset_rng, sample_destroyer, sample_cruiser, sample_battleship


@pytest.mark.unit
class TestShipCreation:
    """艦船の生成テスト"""

    def test_destroyer_creation(self, sample_destroyer):
        """駆逐艦の生成と初期パラメータ"""
        assert sample_destroyer.ship_class == "destroyer"
        assert sample_destroyer.hp == 100
        assert sample_destroyer.max_hp == 100
        assert sample_destroyer.attack == 20
        assert sample_destroyer.defense == 10
        assert sample_destroyer.is_alive() is True

    def test_cruiser_creation(self, sample_cruiser):
        """巡洋艦の生成と初期パラメータ"""
        assert sample_cruiser.ship_class == "cruiser"
        assert sample_cruiser.hp == 200
        assert sample_cruiser.max_hp == 200
        assert sample_cruiser.attack == 40
        assert sample_cruiser.defense == 20

    def test_battleship_creation(self, sample_battleship):
        """戦艦の生成と初期パラメータ"""
        assert sample_battleship.ship_class == "battleship"
        assert sample_battleship.hp == 500
        assert sample_battleship.max_hp == 500
        assert sample_battleship.attack == 100
        assert sample_battleship.defense == 50

    def test_invalid_ship_class(self):
        """無効な艦種でエラーが発生する"""
        with pytest.raises(KeyError):
            Ship(1, "carrier", "Invalid")


@pytest.mark.unit
class TestShipResources:
    """艦船の資源管理テスト（Phase 2）"""

    def test_initial_fuel(self, sample_destroyer):
        """初期燃料が満タン"""
        assert sample_destroyer.fuel == sample_destroyer.max_fuel
        assert sample_destroyer.fuel == 5000.0

    def test_initial_ammo(self, sample_destroyer):
        """初期弾薬が満タン"""
        assert sample_destroyer.ammo == sample_destroyer.max_ammo
        assert sample_destroyer.ammo == 200

    def test_fuel_consumption(self, sample_destroyer):
        """燃料消費"""
        initial_fuel = sample_destroyer.fuel
        sample_destroyer.consume_fuel(100.0)
        assert sample_destroyer.fuel == initial_fuel - 100.0

    def test_fuel_consumption_below_zero(self, sample_destroyer):
        """燃料が0未満にならない"""
        sample_destroyer.consume_fuel(10000.0)
        assert sample_destroyer.fuel == 0.0

    def test_ammo_consumption(self, sample_destroyer):
        """弾薬消費"""
        initial_ammo = sample_destroyer.ammo
        sample_destroyer.consume_ammo(10)
        assert sample_destroyer.ammo == initial_ammo - 10

    def test_ammo_consumption_below_zero(self, sample_destroyer):
        """弾薬が0未満にならない"""
        sample_destroyer.consume_ammo(1000)
        assert sample_destroyer.ammo == 0

    def test_can_fire_with_ammo(self, sample_destroyer):
        """弾薬があれば射撃可能"""
        assert sample_destroyer.can_shoot() is True

    def test_cannot_fire_without_ammo(self, sample_destroyer):
        """弾薬がなければ射撃不可"""
        sample_destroyer.ammo = 0
        assert sample_destroyer.can_shoot() is False

    def test_refuel(self, sample_destroyer):
        """燃料補給"""
        sample_destroyer.fuel = 1000.0
        refueled = sample_destroyer.refuel(2000.0)
        assert refueled == 2000.0
        assert sample_destroyer.fuel == 3000.0

    def test_refuel_overflow(self, sample_destroyer):
        """燃料補給が最大値を超えない"""
        sample_destroyer.fuel = 4000.0
        refueled = sample_destroyer.refuel(2000.0)
        assert refueled == 1000.0  # 4000 + 1000 = 5000（最大）
        assert sample_destroyer.fuel == 5000.0

    def test_rearm(self, sample_destroyer):
        """弾薬補給"""
        sample_destroyer.ammo = 50
        rearmed = sample_destroyer.rearm(100)
        assert rearmed == 100
        assert sample_destroyer.ammo == 150

    def test_rearm_overflow(self, sample_destroyer):
        """弾薬補給が最大値を超えない"""
        sample_destroyer.ammo = 150
        rearmed = sample_destroyer.rearm(100)
        assert rearmed == 50  # 150 + 50 = 200（最大）
        assert sample_destroyer.ammo == 200


@pytest.mark.unit
class TestShipCombat:
    """艦船の戦闘機能テスト"""

    def test_take_damage(self, sample_destroyer):
        """ダメージを受ける"""
        sample_destroyer.take_damage(30)
        assert sample_destroyer.hp == 70
        assert sample_destroyer.is_alive() is True

    def test_take_fatal_damage(self, sample_destroyer):
        """致命的ダメージで撃破"""
        sample_destroyer.take_damage(150)
        assert sample_destroyer.hp == 0
        assert sample_destroyer.is_alive() is False

    def test_cannot_go_negative_hp(self, sample_destroyer):
        """HPが負にならない"""
        sample_destroyer.take_damage(1000)
        assert sample_destroyer.hp == 0

    def test_ammo_per_shot_destroyer(self, sample_destroyer):
        """駆逐艦の1射撃あたり弾薬消費"""
        assert sample_destroyer.ammo_per_shot == 1

    def test_ammo_per_shot_cruiser(self, sample_cruiser):
        """巡洋艦の1射撃あたり弾薬消費"""
        assert sample_cruiser.ammo_per_shot == 2

    def test_ammo_per_shot_battleship(self, sample_battleship):
        """戦艦の1射撃あたり弾薬消費"""
        assert sample_battleship.ammo_per_shot == 5


@pytest.mark.unit
class TestShipClassParameters:
    """艦種ごとのパラメータ検証"""

    @pytest.mark.parametrize("ship_class,expected_hp,expected_attack,expected_defense", [
        ("destroyer", 100, 20, 10),
        ("cruiser", 200, 40, 20),
        ("battleship", 500, 100, 50),
    ])
    def test_ship_class_stats(self, ship_class, expected_hp, expected_attack, expected_defense):
        """艦種ごとの戦闘パラメータ"""
        ship = Ship(1, ship_class, "Test")
        assert ship.max_hp == expected_hp
        assert ship.attack == expected_attack
        assert ship.defense == expected_defense

    @pytest.mark.parametrize("ship_class,expected_fuel,expected_ammo", [
        ("destroyer", 5000.0, 200),
        ("cruiser", 20000.0, 400),
        ("battleship", 100000.0, 1000),
    ])
    def test_ship_class_resources(self, ship_class, expected_fuel, expected_ammo):
        """艦種ごとの資源パラメータ"""
        ship = Ship(1, ship_class, "Test")
        assert ship.max_fuel == expected_fuel
        assert ship.max_ammo == expected_ammo
