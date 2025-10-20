#!/usr/bin/env python3
"""
資源管理を含む戦闘シミュレーション

Phase 2のデモンストレーション。
燃料・弾薬の消費と補給システムを実証する。
"""

import sys
import os

# src/ をパスに追加
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from src.core.simulation import SimulationController
from src.entities.fleet import Fleet
from src.entities.ship import Ship
from src.entities.planet import Planet
from src.entities.star_system import StarSystem
from src.events.resupply import ResupplyEvent


def create_small_fleet() -> Fleet:
    """
    小規模艦隊を作成（弾薬が少なめ）

    少数精鋭で弾薬が早く尽きる設定。
    """
    ships = [
        Ship(ship_id=1, ship_class="destroyer", name="Alliance-Alpha"),
        Ship(ship_id=2, ship_class="destroyer", name="Alliance-Beta"),
    ]

    # 弾薬を減らしておく（テスト用）
    for ship in ships:
        ship.ammo = ship.max_ammo // 4  # 25%まで減らす

    fleet = Fleet(
        fleet_id=1, name="Alliance Fleet", position=(0.0, 0.0, 0.0), ships=ships
    )

    return fleet


def create_enemy_fleet() -> Fleet:
    """
    敵艦隊を作成

    巡洋艦1隻のみ。
    """
    ships = [
        Ship(ship_id=101, ship_class="cruiser", name="Pirate-Raider"),
    ]

    fleet = Fleet(
        fleet_id=2, name="Pirate Fleet", position=(50.0, 0.0, 0.0), ships=ships
    )

    return fleet


def create_home_base() -> tuple[StarSystem, Planet]:
    """
    母港星系と惑星を作成

    Returns:
        (星系, 惑星) のタプル
    """
    # 惑星（補給基地）
    planet = Planet(
        planet_id=1,
        name="Homeworld",
        position=(0.0, 0.0, 0.0),
        fuel_production=2000.0,  # kg/tick
        ammo_production=200,  # 発/tick
    )

    # 星系
    star_system = StarSystem(
        system_id=1,
        name="Alpha Centauri",
        position=(0.0, 0.0, 0.0),
        planets=[planet],
    )

    return star_system, planet


def main():
    """メイン関数"""
    print("\n" + "=" * 60)
    print("Resource Combat Simulation - Phase 2 Demo")
    print("=" * 60)

    # シミュレーション初期化
    sim = SimulationController(seed=42)

    # 星系・惑星作成
    home_system, home_planet = create_home_base()
    sim.add_star_system(home_system)

    # 艦隊作成
    alliance_fleet = create_small_fleet()
    enemy_fleet = create_enemy_fleet()

    # 艦隊を星系に配置
    alliance_fleet.current_system = home_system
    enemy_fleet.current_system = home_system
    home_system.add_fleet(alliance_fleet)
    home_system.add_fleet(enemy_fleet)

    # 艦隊登録
    sim.add_fleet(alliance_fleet)
    sim.add_fleet(enemy_fleet)

    # 初期補給イベント（tick=0で補給）
    initial_resupply = ResupplyEvent(
        tick=0, fleet=alliance_fleet, planet=home_planet
    )
    sim.scheduler.schedule(initial_resupply)

    # 戦闘中の補給イベント（tick=5で補給）
    mid_combat_resupply = ResupplyEvent(
        tick=5, fleet=alliance_fleet, planet=home_planet
    )
    sim.scheduler.schedule(mid_combat_resupply)

    # シミュレーション実行
    print("\nシナリオ:")
    print("- Alliance Fleetは弾薬25%の状態で開始")
    print("- Tick 0で初期補給を受ける")
    print("- 敵艦隊と交戦")
    print("- Tick 5で追加補給を受ける")
    print("- 燃料・弾薬消費を確認しながら戦闘継続\n")

    sim.run(max_ticks=50)

    print("\nSimulation completed successfully!")


if __name__ == "__main__":
    main()
