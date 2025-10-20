"""
イベント基底クラス

すべてのイベントはこの抽象クラスを継承する。
"""

from abc import ABC, abstractmethod
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from src.core.scheduler import EventScheduler


class Event(ABC):
    """
    イベント抽象クラス

    すべてのイベントはexecute()メソッドを実装する必要がある。
    """

    def __init__(self, tick: int, priority: int = 0):
        """
        初期化

        Args:
            tick: イベント発火時刻（tick数）
            priority: 優先度（同一tickで複数イベントがある場合の実行順、小さいほど優先）
        """
        self.tick = tick
        self.priority = priority

    @abstractmethod
    def execute(self, scheduler: "EventScheduler") -> None:
        """
        イベントを実行

        Args:
            scheduler: イベントスケジューラ（新規イベント登録用）
        """
        pass

    def __lt__(self, other: "Event") -> bool:
        """
        イベントの比較演算子（優先度付きキュー用）

        同一tick内では優先度で、tickが異なる場合はtickで比較
        """
        if self.tick == other.tick:
            return self.priority < other.priority
        return self.tick < other.tick

    def __repr__(self) -> str:
        """文字列表現"""
        return f"{self.__class__.__name__}(tick={self.tick}, priority={self.priority})"
