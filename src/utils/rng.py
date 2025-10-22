"""
シード固定乱数生成器

再現性のあるシミュレーションを実現するため、
シードを固定した乱数生成器を提供する。
"""

import numpy as np
from typing import Optional


class SeededRNG:
    """
    シード固定乱数生成器

    同一シードで同一の乱数列を生成することで、
    シミュレーションの再現性を保証する。
    """

    def __init__(self, seed: int = 42):
        """
        初期化

        Args:
            seed: 乱数シード（デフォルト: 42）
        """
        self.seed = seed
        self.rng = np.random.Generator(np.random.PCG64(seed))

    def random(self) -> float:
        """
        0.0～1.0の一様乱数を生成

        Returns:
            0.0以上1.0未満の浮動小数点数
        """
        return self.rng.random()

    def randint(self, low: int, high: int) -> int:
        """
        low以上high未満の整数乱数を生成

        Args:
            low: 下限（含む）
            high: 上限（含まない）

        Returns:
            low以上high未満の整数
        """
        return self.rng.integers(low, high)

    def uniform(self, low: float, high: float) -> float:
        """
        low以上high未満の一様乱数を生成

        Args:
            low: 下限
            high: 上限

        Returns:
            low以上high未満の浮動小数点数
        """
        return self.rng.uniform(low, high)

    def choice(self, choices: list):
        """
        リストからランダムに1要素を選択

        Args:
            choices: 選択肢のリスト

        Returns:
            ランダムに選ばれた要素
        """
        return self.rng.choice(choices)

    def reset(self, seed: Optional[int] = None):
        """
        乱数生成器をリセット

        Args:
            seed: 新しいシード（Noneの場合は初期シードを使用）
        """
        if seed is not None:
            self.seed = seed
        self.rng = np.random.Generator(np.random.PCG64(self.seed))


# グローバル乱数生成器（シングルトンパターン）
_global_rng: Optional[SeededRNG] = None


def get_rng() -> SeededRNG:
    """
    グローバル乱数生成器を取得

    Returns:
        グローバルSeededRNGインスタンス
    """
    global _global_rng
    if _global_rng is None:
        _global_rng = SeededRNG()
    return _global_rng


def set_global_seed(seed: int):
    """
    グローバル乱数生成器のシードを設定

    Args:
        seed: 乱数シード
    """
    global _global_rng
    _global_rng = SeededRNG(seed)
