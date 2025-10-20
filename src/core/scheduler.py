"""
イベントスケジューラ

時系列順にイベントを管理・実行する。
"""

import heapq
from typing import List, Optional
from src.events.base import Event


class EventScheduler:
    """
    イベントスケジューラ

    優先度付きキューを使用してイベントを時系列順に管理。
    同一tick内では優先度順に実行する。
    """

    def __init__(self):
        """初期化"""
        self.event_queue: List[Event] = []  # 優先度付きキュー
        self.current_tick = 0

    def schedule(self, event: Event):
        """
        イベントをスケジュールに追加

        Args:
            event: 追加するイベント
        """
        heapq.heappush(self.event_queue, event)

    def has_events(self) -> bool:
        """
        実行待ちイベントが存在するか確認

        Returns:
            イベントが1つ以上あればTrue
        """
        return len(self.event_queue) > 0

    def peek_next_tick(self) -> Optional[int]:
        """
        次のイベントのtickを確認（取り出さない）

        Returns:
            次のイベントのtick、キューが空ならNone
        """
        if self.has_events():
            return self.event_queue[0].tick
        return None

    def execute_tick(self, tick: int) -> int:
        """
        指定tickのイベントをすべて実行

        Args:
            tick: 実行するtick

        Returns:
            実行したイベント数
        """
        self.current_tick = tick
        executed_count = 0

        # 指定tickのイベントをすべて取り出して実行
        while self.has_events() and self.event_queue[0].tick == tick:
            event = heapq.heappop(self.event_queue)
            print(f"[Tick {tick}] Executing: {event}")
            event.execute(self)
            executed_count += 1

        return executed_count

    def run_until(self, max_tick: int):
        """
        指定tickまでシミュレーションを実行

        Args:
            max_tick: 実行する最大tick
        """
        while self.has_events():
            next_tick = self.peek_next_tick()
            if next_tick is None or next_tick > max_tick:
                break

            self.execute_tick(next_tick)

    def clear(self):
        """イベントキューをクリア"""
        self.event_queue.clear()
        self.current_tick = 0

    def __repr__(self) -> str:
        """文字列表現"""
        return f"EventScheduler(current_tick={self.current_tick}, pending_events={len(self.event_queue)})"
