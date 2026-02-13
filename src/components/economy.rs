use bevy::prelude::*;

/// 資源の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Food,
    Minerals,
    Energy,
    ManufacturedGoods,
}

/// 惑星の資源ストックを管理するコンポーネント
#[derive(Component, Debug, Clone)]
pub struct Resources {
    pub food: f64,
    pub minerals: f64,
    pub energy: f64,
    pub manufactured_goods: f64,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            food: 100.0,
            minerals: 100.0,
            energy: 100.0,
            manufactured_goods: 50.0,
        }
    }
}

impl Resources {
    /// 指定した資源タイプの量を取得
    pub fn get(&self, rt: ResourceType) -> f64 {
        match rt {
            ResourceType::Food => self.food,
            ResourceType::Minerals => self.minerals,
            ResourceType::Energy => self.energy,
            ResourceType::ManufacturedGoods => self.manufactured_goods,
        }
    }

    /// 指定した資源タイプの量を設定
    pub fn set(&mut self, rt: ResourceType, value: f64) {
        match rt {
            ResourceType::Food => self.food = value,
            ResourceType::Minerals => self.minerals = value,
            ResourceType::Energy => self.energy = value,
            ResourceType::ManufacturedGoods => self.manufactured_goods = value,
        }
    }
}

/// 惑星の資源産出能力
#[derive(Component, Debug, Clone)]
pub struct Production {
    pub food_rate: f64,
    pub mineral_rate: f64,
    pub energy_rate: f64,
    pub manufacturing_rate: f64,
}

impl Default for Production {
    fn default() -> Self {
        Self {
            food_rate: 10.0,
            mineral_rate: 5.0,
            energy_rate: 8.0,
            manufacturing_rate: 3.0,
        }
    }
}

/// 貿易ルートコンポーネント
/// 二つの惑星間の貿易接続を表す
#[derive(Component, Debug, Clone)]
pub struct TradeRoute {
    /// 貿易元の惑星
    pub from_planet: Entity,
    /// 貿易先の惑星
    pub to_planet: Entity,
    /// 距離（光年）— コスト計算に使用
    pub distance: f64,
    /// 1 Tick あたりの輸送容量
    pub capacity: f64,
    /// アクティブかどうか
    pub active: bool,
}

impl TradeRoute {
    pub fn new(from: Entity, to: Entity, distance: f64, capacity: f64) -> Self {
        Self {
            from_planet: from,
            to_planet: to,
            distance,
            capacity,
            active: true,
        }
    }

    /// 距離に基づく輸送コスト（0.0〜1.0、1.0 に近いほど損失が大きい）
    pub fn transport_cost(&self, balance: &crate::config::BalanceConfig) -> f64 {
        (self.distance * balance.trade_cost_per_ly).min(balance.max_trade_cost)
    }
}

/// 貿易ルートマーカー
#[derive(Component, Debug)]
pub struct TradeRouteMarker;
/// 枯渇性資源（惑星ごとに固有の埋蔵量）
#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DepletableResources {
    /// 鉱物の総埋蔵量
    pub mineral_reserves: f64,
    /// 初期埋蔵量（%計算用）
    pub initial_mineral_reserves: f64,
    /// 化石燃料の総埋蔵量
    pub fossil_fuel_reserves: f64,
    /// 初期埋蔵量
    pub initial_fossil_fuel_reserves: f64,
    /// レアアース埋蔵量（高等技術用）
    pub rare_earth_reserves: f64,
    /// 初期埋蔵量
    pub initial_rare_earth_reserves: f64,
    /// 採掘深度（0.0 〜 1.0）
    pub mining_depth: f64,
}

impl Default for DepletableResources {
    fn default() -> Self {
        Self {
            mineral_reserves: 10000.0,
            initial_mineral_reserves: 10000.0,
            fossil_fuel_reserves: 5000.0,
            initial_fossil_fuel_reserves: 5000.0,
            rare_earth_reserves: 1000.0,
            initial_rare_earth_reserves: 1000.0,
            mining_depth: 0.0,
        }
    }
}
