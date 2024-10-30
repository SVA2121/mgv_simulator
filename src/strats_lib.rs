use std::collections::HashMap;
use crate::simu_lib::PricePoint;
use crate::chain_lib::User;
use crate::mgv_lib::Market;
use crate::mgv_lib::Offer;


pub trait Strategy {
    // A strategy is simply a function that modifies the user's balance
    // according to the strategy's logic.
    // It has a setup phase and an execution phase.
    fn setup(&mut self);
    fn execute(&mut self, user: &mut User, price_points: &Vec<PricePoint>, markets: &Vec<Market>);
}

pub struct StrategyLibrary {
    strategies: HashMap<String, Box<dyn Strategy>>
}

impl StrategyLibrary {
    pub fn new() -> Self {
        let mut strategies: HashMap<String, Box<dyn Strategy>> = HashMap::new();
        StrategyLibrary { strategies }
    }

    pub fn add_strategy(&mut self, name: &str, strategy: Box<dyn Strategy>) {
        self.strategies.insert(name.to_string(), strategy);
    }

    pub fn get_strategy(&self, name: &str) -> Option<&Strategy> {
        self.strategies.get(name)
    }

    pub fn list_strategies(&self) -> Vec<&str> {
        self.strategies.keys().map(|k| k.as_str()).collect()
    }
}

pub struct LimitOrder {
    market: Market,
    limit_price: f64,
    quantity: u128,
}

impl LimitOrder {
    pub fn new(market: Market, limit_price: f64, quantity: u128) -> Self {
        LimitOrder {
            market,
            limit_price,
            quantity,
        }
    }
}

impl Strategy for LimitOrder {
    fn setup(&mut self) {
        let offer = Offer::new(
            self.market.clone(),
            self.limit_price,
            self.quantity,
            user.address.clone(),
        );
        // Add the offer to the market
        self.market.add_offer(offer);
    }

    fn execute(&mut self, user: &mut User, _price_points: &Vec<PricePoint>, _markets: &Vec<Market>) {
        // No execution needed for limit order
    }
}




