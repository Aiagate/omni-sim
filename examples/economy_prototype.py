"""
経済・交易・物価変動システムのプロトタイプ実装

SimPyを使わず、NumPy行列演算とNumba最適化による高速シミュレーション
"""

import numpy as np
import networkx as nx
from typing import Dict, List, Tuple
import time


class TradeNetwork:
    """
    交易ネットワークグラフ（NetworkX）
    """
    def __init__(self):
        self.graph = nx.DiGraph()
        self.system_ids = {}  # 星系名 -> インデックス
        self.system_names = []  # インデックス -> 星系名

    def add_system(self, system_name: str):
        """星系を追加"""
        if system_name not in self.system_ids:
            idx = len(self.system_names)
            self.system_ids[system_name] = idx
            self.system_names.append(system_name)
            self.graph.add_node(system_name)

    def add_trade_route(self, from_system: str, to_system: str,
                       distance: float, tariff: float = 0.0):
        """
        交易路を追加

        Args:
            from_system: 出発星系
            to_system: 到着星系
            distance: 距離（ly）
            tariff: 関税率（0.0〜1.0）
        """
        # 輸送コスト = 距離 × 10 + 関税 × 100
        transport_cost = distance * 10.0 + tariff * 100.0

        self.graph.add_edge(from_system, to_system,
                           distance=distance,
                           tariff=tariff,
                           cost=transport_cost)

    def get_cost_matrix(self) -> np.ndarray:
        """
        輸送コスト行列を取得（NumPy配列）

        Returns:
            (星系数, 星系数) の行列
        """
        num_systems = len(self.system_names)
        cost_matrix = np.full((num_systems, num_systems), np.inf)

        for i in range(num_systems):
            cost_matrix[i, i] = 0.0

        for from_sys, to_sys, data in self.graph.edges(data=True):
            i = self.system_ids[from_sys]
            j = self.system_ids[to_sys]
            cost_matrix[i, j] = data['cost']

        return cost_matrix


class MarketPriceSystem:
    """
    物価計算システム（NumPy行列演算）
    """
    def __init__(self, num_systems: int, num_resources: int):
        self.num_systems = num_systems
        self.num_resources = num_resources

        # 生産量行列 (星系数 × 資源数)
        self.production = np.zeros((num_systems, num_resources))

        # 消費量行列 (星系数 × 資源数)
        self.consumption = np.zeros((num_systems, num_resources))

        # 在庫行列 (星系数 × 資源数)
        self.stocks = np.zeros((num_systems, num_resources))

        # 価格行列 (星系数 × 資源数)
        self.prices = np.ones((num_systems, num_resources)) * 100.0

        # 基準価格（銀河平均価格）
        self.base_prices = np.ones(num_resources) * 100.0

    def set_production(self, system_id: int, resource_id: int, amount: float):
        """生産量を設定"""
        self.production[system_id, resource_id] = amount

    def set_consumption(self, system_id: int, resource_id: int, amount: float):
        """消費量を設定"""
        self.consumption[system_id, resource_id] = amount

    def update_stocks(self):
        """
        在庫を更新（生産 - 消費）
        """
        self.stocks += self.production - self.consumption
        # 在庫は0未満にならない
        self.stocks = np.maximum(self.stocks, 0.0)

    def update_prices(self, elasticity: float = 0.1):
        """
        需給バランスから価格を更新

        Args:
            elasticity: 価格弾力性（0.0〜1.0）
        """
        # 需給差分を計算（生産 - 消費）
        supply_demand_gap = self.production - self.consumption

        # 価格変動率 = -弾力性 × (需給差分 / 消費量)
        # 供給過剰（+）なら価格下落、需要過多（-）なら価格上昇
        consumption_safe = np.where(self.consumption > 0,
                                    self.consumption, 1.0)
        price_change_rate = -elasticity * (supply_demand_gap / consumption_safe)

        # 価格を更新（下限は基準価格の10%、上限は1000%）
        self.prices *= (1.0 + price_change_rate)
        self.prices = np.clip(self.prices,
                             self.base_prices * 0.1,
                             self.base_prices * 10.0)

    def propagate_prices(self, trade_volume_matrix: np.ndarray,
                        diffusion_rate: float = 0.05):
        """
        交易による価格伝播を計算（拡散方程式モデル）

        Args:
            trade_volume_matrix: 交易量行列 (星系数 × 星系数)
            diffusion_rate: 拡散率（0.0〜1.0）
        """
        # 交易量で正規化
        trade_sum = trade_volume_matrix.sum(axis=1, keepdims=True)
        trade_normalized = np.where(trade_sum > 0,
                                    trade_volume_matrix / trade_sum,
                                    0.0)

        for resource_id in range(self.num_resources):
            price_vector = self.prices[:, resource_id]

            # 隣接星系の価格差を計算
            price_diff = price_vector[:, np.newaxis] - price_vector[np.newaxis, :]

            # 交易量に比例して価格が収束
            price_adjustment = np.sum(trade_normalized * price_diff, axis=1)

            # 価格を更新
            self.prices[:, resource_id] += diffusion_rate * price_adjustment

    def get_price(self, system_id: int, resource_id: int) -> float:
        """特定星系・資源の価格を取得"""
        return self.prices[system_id, resource_id]


class SimpleTradeExecutor:
    """
    簡易交易実行システム（Numba最適化版は省略）
    """
    def __init__(self, price_system: MarketPriceSystem,
                 cost_matrix: np.ndarray):
        self.price_system = price_system
        self.cost_matrix = cost_matrix

    def execute_trades(self) -> np.ndarray:
        """
        価格差と輸送コストから交易を実行

        Returns:
            交易量行列 (星系数 × 星系数)
        """
        num_systems = self.price_system.num_systems
        trade_volume = np.zeros((num_systems, num_systems))

        # 各資源について交易を計算
        for res_id in range(self.price_system.num_resources):
            prices = self.price_system.prices[:, res_id]

            # 価格差行列（買い手 - 売り手）
            price_diff = prices[:, np.newaxis] - prices[np.newaxis, :]

            # 利益行列（価格差 - 輸送コスト）
            profit_matrix = price_diff - self.cost_matrix

            # 利益がある交易のみ実行
            profitable_trades = profit_matrix > 0

            # 交易量を計算（簡易版：在庫の一定割合）
            for i in range(num_systems):
                for j in range(num_systems):
                    if profitable_trades[i, j]:
                        # 売り手の在庫の10%を交易
                        available = self.price_system.stocks[i, res_id]
                        trade_amount = available * 0.1

                        # 資源を移動
                        self.price_system.stocks[i, res_id] -= trade_amount
                        self.price_system.stocks[j, res_id] += trade_amount

                        # 交易量を記録
                        trade_volume[i, j] += trade_amount

        return trade_volume


# ===============================
# デモシミュレーション
# ===============================

def run_economy_demo():
    """
    経済シミュレーションのデモ実行
    """
    print("=" * 60)
    print("経済・交易・物価変動システム - プロトタイプデモ")
    print("=" * 60)
    print()

    # 1. 交易ネットワークを構築
    print("Step 1: 交易ネットワーク構築")
    network = TradeNetwork()

    systems = ["Alpha", "Beta", "Gamma", "Delta", "Epsilon"]
    for sys_name in systems:
        network.add_system(sys_name)

    # 交易路を追加
    routes = [
        ("Alpha", "Beta", 10.0, 0.05),
        ("Beta", "Gamma", 15.0, 0.10),
        ("Gamma", "Delta", 20.0, 0.02),
        ("Delta", "Epsilon", 12.0, 0.08),
        ("Alpha", "Gamma", 25.0, 0.15),
    ]

    for from_sys, to_sys, distance, tariff in routes:
        network.add_trade_route(from_sys, to_sys, distance, tariff)

    cost_matrix = network.get_cost_matrix()
    print(f"  - 星系数: {len(systems)}")
    print(f"  - 交易路数: {len(routes)}")
    print()

    # 2. 物価システムを初期化
    print("Step 2: 物価システム初期化")
    num_systems = len(systems)
    num_resources = 3  # 鉄、銅、チタン

    price_system = MarketPriceSystem(num_systems, num_resources)

    # 資源名
    resources = ["Iron", "Copper", "Titanium"]

    # 生産量を設定（星系ごとに特性が異なる）
    production_config = [
        [1000.0, 500.0, 100.0],   # Alpha: 鉄が豊富
        [200.0, 1000.0, 200.0],   # Beta: 銅が豊富
        [500.0, 300.0, 800.0],    # Gamma: チタンが豊富
        [800.0, 800.0, 300.0],    # Delta: バランス型
        [300.0, 200.0, 1000.0],   # Epsilon: チタンが豊富
    ]

    for sys_id, prod in enumerate(production_config):
        for res_id, amount in enumerate(prod):
            price_system.set_production(sys_id, res_id, amount)

    # 消費量を設定（需要が異なる）
    consumption_config = [
        [500.0, 800.0, 300.0],    # Alpha: 銅の需要が高い
        [900.0, 300.0, 400.0],    # Beta: 鉄の需要が高い
        [400.0, 600.0, 200.0],    # Gamma: バランス型
        [600.0, 500.0, 700.0],    # Delta: チタン需要が高い
        [700.0, 400.0, 300.0],    # Epsilon: 鉄の需要が高い
    ]

    for sys_id, cons in enumerate(consumption_config):
        for res_id, amount in enumerate(cons):
            price_system.set_consumption(sys_id, res_id, amount)

    # 初期在庫を設定
    price_system.stocks = price_system.production * 10.0

    print(f"  - 資源数: {num_resources}")
    print(f"  - 初期価格: {price_system.base_prices}")
    print()

    # 3. 交易実行システムを初期化
    print("Step 3: 交易実行システム初期化")
    trade_executor = SimpleTradeExecutor(price_system, cost_matrix)
    print()

    # 4. シミュレーション開始
    print("Step 4: シミュレーション開始")
    print("=" * 60)
    print()

    num_ticks = 20
    start_time = time.time()

    for tick in range(num_ticks):
        # 生産・消費を更新
        price_system.update_stocks()

        # 価格を更新（需給バランス）
        price_system.update_prices(elasticity=0.15)

        # 交易を実行
        trade_volume = trade_executor.execute_trades()

        # 価格伝播を計算
        price_system.propagate_prices(trade_volume, diffusion_rate=0.05)

        # 結果を表示（5tickごと）
        if tick % 5 == 0:
            print(f"[Tick {tick}]")
            for sys_id, sys_name in enumerate(systems):
                print(f"  {sys_name}:")
                for res_id, res_name in enumerate(resources):
                    price = price_system.get_price(sys_id, res_id)
                    stock = price_system.stocks[sys_id, res_id]
                    print(f"    - {res_name}: Price={price:.2f}, Stock={stock:.0f}kg")
            print()

    elapsed_time = time.time() - start_time

    print("=" * 60)
    print("シミュレーション完了")
    print(f"  - 実行時間: {elapsed_time*1000:.2f}ms")
    print(f"  - 平均tick時間: {elapsed_time/num_ticks*1000:.2f}ms/tick")
    print()

    # 5. 最終結果サマリー
    print("最終価格サマリー:")
    print()
    print(f"{'星系':<10} {'Iron':>10} {'Copper':>10} {'Titanium':>10}")
    print("-" * 45)
    for sys_id, sys_name in enumerate(systems):
        prices_str = "  ".join([
            f"{price_system.get_price(sys_id, res_id):>8.2f}"
            for res_id in range(num_resources)
        ])
        print(f"{sys_name:<10} {prices_str}")

    print()
    print("価格変動率（初期価格100から）:")
    print()
    print(f"{'星系':<10} {'Iron':>10} {'Copper':>10} {'Titanium':>10}")
    print("-" * 45)
    for sys_id, sys_name in enumerate(systems):
        changes_str = "  ".join([
            f"{(price_system.get_price(sys_id, res_id) - 100.0):>+8.2f}%"
            for res_id in range(num_resources)
        ])
        print(f"{sys_name:<10} {changes_str}")


if __name__ == "__main__":
    run_economy_demo()
