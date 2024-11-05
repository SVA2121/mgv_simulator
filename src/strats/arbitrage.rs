use crate::strats_lib::Strategy;
use crate::simu_lib::PricePoint;
use crate::mgv_lib::{Market, OrderSide, Offer};
use crate::chain_lib::User;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
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
        let reference_price = price_point.price;
        
        loop {
            // Get market state and immediately drop the references
            let best_bid = market.best_bid();
            let best_ask = market.best_ask();
            
            // Exit if no orders in the book
            if best_bid.is_none() && best_ask.is_none() {
                break;
            }
    
            let mut traded = false;
    
            // Process bid side first, without holding any references
            if let Some(bid) = best_bid {
                if (bid.price as f64) - reference_price > self.min_profit_threshold {
                    let volume = bid.volume.min(self.max_volume_per_trade);
                    user.lock().unwrap().add_token_balance(&market.base, volume as f64);
                    market.market_order(&user, OrderSide::Sell, volume)?;
                    user.lock().unwrap().spend_token_balance(&market.quote, reference_price * volume as f64)?;
                    traded = true;
                    // Continue to next iteration immediately if we traded
                    continue;
                }
            }
    
            // Process ask side if we didn't trade on bid side
            if let Some(ask) = best_ask {
                if reference_price - (ask.price as f64) > self.min_profit_threshold {
                    let volume = ask.volume.min(self.max_volume_per_trade);
                    user.lock().unwrap().add_token_balance(&market.quote, reference_price * volume as f64);
                    market.market_order(&user, OrderSide::Buy, volume)?;
                    user.lock().unwrap().spend_token_balance(&market.base, volume as f64)?;
                    traded = true;
                    continue;
                }
            }
    
            // Exit if no profitable trades were made this iteration
            if !traded {
                break;
            }
        }
    
        Ok(())
    }

    fn post_hook(
        &mut self,
        market: &mut Market,
        maker: Arc<Mutex<User>>,
        filled_offer: &Offer,
    ) -> Result<(), &'static str> {
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
