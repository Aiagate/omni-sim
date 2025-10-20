"""
資源管理システム

惑星の資源産出・備蓄更新を一括処理する。
Phase 2: 簡易版（固定産出量）
"""

from typing import List
from src.entities.planet import Planet


class ResourceSystem:
    """
    資源管理システム

    全惑星の資源産出を一括処理。
    Phase 3で技術レベルによる採掘効率を追加予定。
    """

    def __init__(self):
        """初期化"""
        pass

    def update(self, planets: List[Planet], tick: int):
        """
        全惑星の資源を産出

        Args:
            planets: 更新対象の惑星リスト
            tick: 現在のtick
        """
        for planet in planets:
            planet.produce_resources()

    def update_single(self, planet: Planet, tick: int):
        """
        単一惑星の資源を産出

        Args:
            planet: 更新対象の惑星
            tick: 現在のtick
        """
        planet.produce_resources()
