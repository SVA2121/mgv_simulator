use std::sync::{Arc, Mutex};
use crate::simu_lib::PricePoint;
use crate::mgv_lib::{Market, OfferSide};
use crate::chain_lib::User;
use crate::strats::limit_order::LimitOrderStrategy;
use crate::strats::arbitrage::ArbitrageStrategy;
use crate::strats::kandel::KandelStrategy;
use std::collections::HashMap;



pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    
    // Main strategy execution method
    fn execute(
        &mut self,
        price_point: &PricePoint,
        market: &mut Market,
        user: Arc<Mutex<User>>,
    ) -> Result<(), &'static str>;
    
    // Optional methods for strategy parameters
    fn set_parameter(&mut self, name: &str, value: f64) -> Result<(), &'static str> {
        Err("Parameter not supported")
    }
    
    fn get_parameter(&self, name: &str) -> Option<f64> {
        None
    }
}



// Strategy Factory for dynamic strategy creation
pub struct StrategyFactory {
    builders: HashMap<String, Box<dyn Fn() -> Box<dyn Strategy> + Send + Sync>>,
}

impl StrategyFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            builders: HashMap::new(),
        };
        
        // Register default strategies
        factory.register_strategy("limit_order", || {
            Box::new(LimitOrderStrategy::new(0.0, 0.0, OfferSide::Bid))
        });
        factory.register_strategy("arbitrage", || {
            Box::new(ArbitrageStrategy::new(0.1, 1000.0))
        });
        factory.register_strategy("kandel", || {
            Box::new(KandelStrategy::new())
        });
        
        factory
    }

    pub fn register_strategy<F>(&mut self, name: &str, builder: F)
    where
        F: Fn() -> Box<dyn Strategy> + Send + Sync + 'static,
    {
        self.builders.insert(name.to_string(), Box::new(builder));
    }

    pub fn create_strategy(&self, name: &str) -> Option<Box<dyn Strategy>> {
        self.builders.get(name).map(|builder| builder())
    }

    pub fn list_strategies(&self) -> Vec<String> {
        self.builders.keys().cloned().collect()
    }
}