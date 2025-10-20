"""
座標・距離計算ユーティリティ

3次元空間での座標計算と距離計算を提供する。
"""

import math
from typing import Tuple


# 型エイリアス
Position = Tuple[float, float, float]


def distance(pos1: Position, pos2: Position) -> float:
    """
    2点間のユークリッド距離を計算

    Args:
        pos1: 点1の座標 (x, y, z)
        pos2: 点2の座標 (x, y, z)

    Returns:
        2点間の距離
    """
    x1, y1, z1 = pos1
    x2, y2, z2 = pos2
    return math.sqrt((x2 - x1) ** 2 + (y2 - y1) ** 2 + (z2 - z1) ** 2)


def move_toward(
    current_pos: Position, target_pos: Position, speed: float
) -> Position:
    """
    目標地点に向かって移動

    Args:
        current_pos: 現在位置 (x, y, z)
        target_pos: 目標位置 (x, y, z)
        speed: 移動速度（単位距離/tick）

    Returns:
        移動後の位置 (x, y, z)
    """
    dist = distance(current_pos, target_pos)

    # すでに目標地点にいる、または1tickで到達可能
    if dist <= speed:
        return target_pos

    # 移動方向ベクトルを正規化して速度を掛ける
    x1, y1, z1 = current_pos
    x2, y2, z2 = target_pos

    ratio = speed / dist
    new_x = x1 + (x2 - x1) * ratio
    new_y = y1 + (y2 - y1) * ratio
    new_z = z1 + (z2 - z1) * ratio

    return (new_x, new_y, new_z)


def is_within_range(pos1: Position, pos2: Position, range_limit: float) -> bool:
    """
    2点が指定範囲内にあるか判定

    Args:
        pos1: 点1の座標 (x, y, z)
        pos2: 点2の座標 (x, y, z)
        range_limit: 判定範囲

    Returns:
        範囲内ならTrue、範囲外ならFalse
    """
    return distance(pos1, pos2) <= range_limit
