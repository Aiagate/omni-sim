#!/usr/bin/env python3
"""
基本的な2艦隊戦闘シミュレーション

Phase 1のデモンストレーション。
2つの艦隊が戦闘し、どちらかが全滅するまで続行する。
"""

import sys
import os

# src/ をパスに追加
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from src.core.simulation import SimulationController
from src.entities.fleet import Fleet
from src.entities.ship import Ship


def create_imperial_fleet() -> Fleet:
    """
    帝国艦隊を作成

    戦艦中心の重厚な編成
    """
    ships = [
        Ship(ship_id=1, ship_class="battleship", name="Imperial-Sovereign"),
        Ship(ship_id=2, ship_class="battleship", name="Imperial-Dominance"),
        Ship(ship_id=3, ship_class="cruiser", name="Imperial-Guardian"),
        Ship(ship_id=4, ship_class="destroyer", name="Imperial-Swift"),
        Ship(ship_id=5, ship_class="destroyer", name="Imperial-Falcon"),
    ]

    fleet = Fleet(
        fleet_id=1, name="Imperial Fleet", position=(0.0, 0.0, 0.0), ships=ships
    )

    return fleet


def create_rebel_fleet() -> Fleet:
    """
    反乱軍艦隊を作成

    駆逐艦・巡洋艦中心の機動力重視編成
    """
    ships = [
        Ship(ship_id=101, ship_class="cruiser", name="Rebel-Liberty"),
        Ship(ship_id=102, ship_class="cruiser", name="Rebel-Freedom"),
        Ship(ship_id=103, ship_class="cruiser", name="Rebel-Hope"),
        Ship(ship_id=104, ship_class="destroyer", name="Rebel-Phoenix"),
        Ship(ship_id=105, ship_class="destroyer", name="Rebel-Aurora"),
        Ship(ship_id=106, ship_class="destroyer", name="Rebel-Comet"),
    ]

    fleet = Fleet(
        fleet_id=2, name="Rebel Fleet", position=(50.0, 0.0, 0.0), ships=ships
    )

    return fleet


def main():
    """メイン関数"""
    print("\n" + "=" * 60)
    print("Basic Combat Simulation - Phase 1 Demo")
    print("=" * 60)

    # シミュレーション初期化（シード固定で再現性確保）
    sim = SimulationController(seed=42)

    # 艦隊作成
    imperial_fleet = create_imperial_fleet()
    rebel_fleet = create_rebel_fleet()

    # 艦隊登録
    sim.add_fleet(imperial_fleet)
    sim.add_fleet(rebel_fleet)

    # シミュレーション実行
    sim.run(max_ticks=100)

    print("\nSimulation completed successfully!")


if __name__ == "__main__":
    main()
