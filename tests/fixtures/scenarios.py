"""
テストシナリオとフィクスチャ

共通のテストデータ、シナリオ、ヘルパー関数を定義。
"""

import pytest
from src.entities.ship import Ship
from src.entities.fleet import Fleet
from src.entities.planet import Planet
from src.entities.star_system import StarSystem
from src.utils.rng import set_global_seed


@pytest.fixture
def reset_rng():
    """
    各テスト前に乱数生成器をリセット
    
    決定論的なテストを保証するため、シード42で初期化。
    """
    set_global_seed(42)
    yield
    # テスト後のクリーンアップは不要（次のテストで再初期化）


@pytest.fixture
def sample_destroyer():
    """駆逐艦のサンプルインスタンスを返す"""
    return Ship(
        ship_id=1,
        ship_class="destroyer",
        name="Test-Destroyer"
    )


@pytest.fixture
def sample_cruiser():
    """巡洋艦のサンプルインスタンスを返す"""
    return Ship(
        ship_id=2,
        ship_class="cruiser",
        name="Test-Cruiser"
    )


@pytest.fixture
def sample_battleship():
    """戦艦のサンプルインスタンスを返す"""
    return Ship(
        ship_id=3,
        ship_class="battleship",
        name="Test-Battleship"
    )


@pytest.fixture
def sample_fleet():
    """
    標準的な艦隊（駆逐艦2隻）を返す
    """
    ships = [
        Ship(1, "destroyer", "Destroyer-1"),
        Ship(2, "destroyer", "Destroyer-2")
    ]
    fleet = Fleet(
        fleet_id=1,
        name="Test-Fleet",
        position=(0.0, 0.0, 0.0),
        ships=ships
    )
    return fleet


@pytest.fixture
def sample_mixed_fleet():
    """
    混成艦隊（戦艦1・巡洋艦1・駆逐艦1）を返す
    """
    ships = [
        Ship(1, "battleship", "Battleship-1"),
        Ship(2, "cruiser", "Cruiser-1"),
        Ship(3, "destroyer", "Destroyer-1")
    ]
    fleet = Fleet(
        fleet_id=2,
        name="Mixed-Fleet",
        position=(0.0, 0.0, 0.0),
        ships=ships
    )
    return fleet


@pytest.fixture
def sample_planet():
    """標準的な惑星を返す"""
    return Planet(
        planet_id=1,
        name="Test-Planet",
        position=(0.0, 0.0, 0.0),
        fuel_production=1000.0,
        ammo_production=100
    )


@pytest.fixture
def sample_star_system():
    """
    標準的な星系（惑星1・艦隊1）を返す
    """
    system = StarSystem(
        system_id=1,
        name="Test-System",
        position=(0.0, 0.0, 0.0)
    )
    
    # 惑星を追加
    planet = Planet(
        planet_id=1,
        name="Homeworld",
        position=(0.0, 0.0, 0.0),
        fuel_production=1000.0,
        ammo_production=100
    )
    system.add_planet(planet)
    
    # 艦隊を追加
    ships = [Ship(1, "destroyer", "Destroyer-1")]
    fleet = Fleet(
        fleet_id=1,
        name="Home-Fleet",
        position=(0.0, 0.0, 0.0),
        ships=ships
    )
    system.add_fleet(fleet)

    return system


def create_damaged_ship(ship_class: str = "destroyer", hp_percent: float = 0.5):
    """
    ダメージを受けた艦船を作成

    Args:
        ship_class: 艦種（destroyer/cruiser/battleship）
        hp_percent: 残HP比率（0.0～1.0）

    Returns:
        ダメージを受けた艦船インスタンス
    """
    ship = Ship(
        ship_id=999,
        ship_class=ship_class,
        name=f"Damaged-{ship_class}"
    )
    ship.hp = int(ship.max_hp * hp_percent)
    return ship


def create_low_ammo_ship(ship_class: str = "destroyer", ammo_percent: float = 0.1):
    """
    弾薬が少ない艦船を作成

    Args:
        ship_class: 艦種
        ammo_percent: 残弾薬比率（0.0～1.0）

    Returns:
        弾薬が少ない艦船インスタンス
    """
    ship = Ship(
        ship_id=999,
        ship_class=ship_class,
        name=f"LowAmmo-{ship_class}"
    )
    ship.ammo = int(ship.max_ammo * ammo_percent)
    return ship


def create_low_fuel_ship(ship_class: str = "destroyer", fuel_percent: float = 0.1):
    """
    燃料が少ない艦船を作成

    Args:
        ship_class: 艦種
        fuel_percent: 残燃料比率（0.0～1.0）

    Returns:
        燃料が少ない艦船インスタンス
    """
    ship = Ship(
        ship_id=999,
        ship_class=ship_class,
        name=f"LowFuel-{ship_class}"
    )
    ship.fuel = ship.max_fuel * fuel_percent
    return ship
