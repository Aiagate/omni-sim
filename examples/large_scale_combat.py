#!/usr/bin/env python3
"""
大規模戦闘シミュレーション

1万隻 vs 2万隻の艦隊戦闘のパフォーマンステスト。
処理時間を計測する。
"""

import sys
import os
import time

# src/ をパスに追加
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from src.core.scheduler import EventScheduler
from src.entities.fleet import Fleet
from src.entities.ship import Ship
from src.events.detection import DetectionEvent
from src.utils.rng import set_global_seed
from src.config.constants import MAX_TICKS


def create_large_fleet(
    fleet_id: int, name: str, num_ships: int, position: tuple[float, float, float]
) -> Fleet:
    """
    大規模艦隊を作成

    Args:
        fleet_id: 艦隊ID
        name: 艦隊名
        num_ships: 艦船数
        position: 初期位置

    Returns:
        作成された艦隊
    """
    ships = []

    # 艦船クラスの比率: 駆逐艦60%, 巡洋艦30%, 戦艦10%
    num_destroyers = int(num_ships * 0.6)
    num_cruisers = int(num_ships * 0.3)
    num_battleships = num_ships - num_destroyers - num_cruisers

    ship_id_base = fleet_id * 100000  # IDの衝突を避ける

    # 戦艦
    for i in range(num_battleships):
        ships.append(
            Ship(
                ship_id=ship_id_base + i,
                ship_class="battleship",
                name=f"{name}-BB-{i+1}",
            )
        )

    # 巡洋艦
    for i in range(num_cruisers):
        ships.append(
            Ship(
                ship_id=ship_id_base + num_battleships + i,
                ship_class="cruiser",
                name=f"{name}-CA-{i+1}",
            )
        )

    # 駆逐艦
    for i in range(num_destroyers):
        ships.append(
            Ship(
                ship_id=ship_id_base + num_battleships + num_cruisers + i,
                ship_class="destroyer",
                name=f"{name}-DD-{i+1}",
            )
        )

    fleet = Fleet(fleet_id=fleet_id, name=name, position=position, ships=ships)

    return fleet


def run_silent_simulation(fleet_a: Fleet, fleet_b: Fleet, seed: int = 42, max_ticks: int = MAX_TICKS) -> int:
    """
    サイレントモードでシミュレーションを実行（ログ出力なし）

    Args:
        fleet_a: 艦隊A
        fleet_b: 艦隊B
        seed: 乱数シード
        max_ticks: 最大tick数

    Returns:
        終了時のtick数
    """
    set_global_seed(seed)
    scheduler = EventScheduler()

    # 初期イベント登録
    detection_event = DetectionEvent(tick=1, fleet_a=fleet_a, fleet_b=fleet_b)
    scheduler.schedule(detection_event)

    # シミュレーション実行（ログ出力抑制）
    tick = 0
    while scheduler.has_events() and tick < max_ticks:
        next_tick = scheduler.peek_next_tick()
        if next_tick is None:
            break

        tick = next_tick
        # イベント実行（標準出力を抑制）
        import io
        import contextlib

        with contextlib.redirect_stdout(io.StringIO()):
            scheduler.execute_tick(tick)

        # 終了判定
        if fleet_a.is_destroyed() or fleet_b.is_destroyed():
            break

    return tick


def main():
    """メイン関数"""
    print("\n" + "=" * 60)
    print("Large Scale Combat Simulation - Performance Test")
    print("=" * 60)

    # 艦隊規模
    fleet_a_size = 10000
    fleet_b_size = 20000

    print(f"\nFleet A: {fleet_a_size} ships")
    print(f"Fleet B: {fleet_b_size} ships")
    print(f"Total: {fleet_a_size + fleet_b_size} ships")

    # 艦隊作成開始
    print("\n--- Creating fleets ---")
    start_time = time.time()

    fleet_a = create_large_fleet(
        fleet_id=1, name="Alliance Fleet", num_ships=fleet_a_size, position=(0.0, 0.0, 0.0)
    )
    fleet_b = create_large_fleet(
        fleet_id=2, name="Empire Fleet", num_ships=fleet_b_size, position=(50.0, 0.0, 0.0)
    )

    creation_time = time.time() - start_time
    print(f"Fleet creation time: {creation_time:.2f} seconds")

    # シミュレーション実行（サイレントモード）
    print("\n--- Running simulation (silent mode) ---")
    start_time = time.time()

    final_tick = run_silent_simulation(fleet_a, fleet_b, seed=42, max_ticks=1000)

    simulation_time = time.time() - start_time
    print(f"Simulation completed at tick {final_tick}")
    print(f"Simulation time: {simulation_time:.2f} seconds")

    # 最終結果表示
    print(f"\n--- Final State ---")
    fleet_a_alive = len(fleet_a.get_alive_ships())
    fleet_b_alive = len(fleet_b.get_alive_ships())

    print(f"{fleet_a.name}: {fleet_a_alive}/{fleet_a_size} ships alive")
    print(f"{fleet_b.name}: {fleet_b_alive}/{fleet_b_size} ships alive")

    if fleet_a.is_destroyed():
        print(f"\n*** {fleet_b.name} VICTORY! ***")
    elif fleet_b.is_destroyed():
        print(f"\n*** {fleet_a.name} VICTORY! ***")
    else:
        print(f"\n*** BATTLE CONTINUES (max ticks reached) ***")

    # 合計時間
    total_time = creation_time + simulation_time
    print(f"\n{'='*60}")
    print(f"Total execution time: {total_time:.2f} seconds")
    print(f"{'='*60}")

    # パフォーマンスサマリー
    print("\n--- Performance Summary ---")
    print(f"Fleet creation: {creation_time:.2f}s ({creation_time/total_time*100:.1f}%)")
    print(f"Simulation: {simulation_time:.2f}s ({simulation_time/total_time*100:.1f}%)")
    print(f"Ticks per second: {final_tick/simulation_time:.0f} ticks/s")
    print(f"Ships per second: {(fleet_a_size + fleet_b_size)/simulation_time:.0f} ships processed/s")


if __name__ == "__main__":
    main()
