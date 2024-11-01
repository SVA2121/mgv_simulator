use crate::strats_lib::Strategy;
use crate::simu_lib::PricePoint;
use crate::mgv_lib::{Market, OrderSide};
use crate::chain_lib::User;
use std::sync::{Arc, Mutex};

pub struct ArbitrageStrategy {
    min_profit_threshold: f64,
    max_volume_per_trade: f64,
}

impl ArbitrageStrategy {
    pub fn new(min_profit_threshold: f64, max_volume_per_trade: f64) -> Self {
        Self {
            min_profit_threshold,
            max_volume_per_trade,
        }
    }
}

impl Strategy for ArbitrageStrategy {
    fn name(&self) -> &str {
        "Arbitrage Strategy"
    }

    fn description(&self) -> &str {
        "Executes trades when market prices deviate from reference price"
    }

    fn execute(
        &mut self,
        price_point: &PricePoint,
        market: &mut Market,
        user: Arc<Mutex<User>>,
    ) -> Result<(), &'static str> {
        
        // Get best bid and ask from the market
        let best_bid_price = market.best_bid().map(|o| o.price as f64);
        let best_ask_price = market.best_ask().map(|o| o.price as f64);
        let best_bid_volume = market.best_bid().map(|o| o.volume);
        let best_ask_volume = market.best_ask().map(|o| o.volume);
        let reference_price = price_point.price;
        

        // Check for arbitrage opportunities
        if let Some(best_bid_price) = best_bid_price {
            if best_bid_price > reference_price + self.min_profit_threshold {
                // Sell at the bid price (higher than reference)
                let volume = best_bid_volume
                    .map(|v| v.min(self.max_volume_per_trade))
                    .unwrap_or(self.max_volume_per_trade);
                user.lock().unwrap().add_token_balance(&market.base, volume as f64);
                market.market_order(&user, OrderSide::Sell, volume)?;
                user.lock().unwrap().spend_token_balance(&market.quote, reference_price * volume as f64)?;

            }
        }

        if let Some(best_ask_price) = best_ask_price {

            if best_ask_price < reference_price - self.min_profit_threshold {
                // Buy at the ask price (lower than reference)
                let volume = best_ask_volume
                    .map(|v| v.min(self.max_volume_per_trade))
                    .unwrap_or(self.max_volume_per_trade);
                user.lock().unwrap().add_token_balance(&market.quote, volume as f64);
                market.market_order(&user, OrderSide::Buy, volume)?;
                user.lock().unwrap().spend_token_balance(&market.base, reference_price * volume as f64)?;

            }
        }

        Ok(())
    }

    fn set_parameter(&mut self, name: &str, value: f64) -> Result<(), &'static str> {
        match name {
            "min_profit_threshold" => {
                self.min_profit_threshold = value;
                Ok(())
            }
            "max_volume_per_trade" => {
                self.max_volume_per_trade = value;
                Ok(())
            }
            _ => Err("Unknown parameter"),
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f64> {
        match name {
            "min_profit_threshold" => Some(self.min_profit_threshold),
            "max_volume_per_trade" => Some(self.max_volume_per_trade),
            _ => None,
        }
    }
}
