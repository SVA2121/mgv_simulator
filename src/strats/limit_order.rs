use crate::strats_lib::Strategy;
use crate::simu_lib::PricePoint;
use crate::mgv_lib::{Market, Offer, OfferSide};
use crate::chain_lib::User;
use std::sync::{Arc, Mutex};

// Example implementation of a simple limit order strategy
#[derive(Clone)]
pub struct LimitOrderStrategy {
    trigger_price: f64,
    volume: f64,
    side: OfferSide,
    executed: bool,
}


impl LimitOrderStrategy {
    pub fn new(trigger_price: f64, volume: f64, side: OfferSide) -> Self {
        Self {
            trigger_price,
            volume,
            side,
            executed: false,
        }
    }
}

impl Strategy for LimitOrderStrategy {
    fn name(&self) -> &str {
        "Limit Order Strategy"
    }

    fn description(&self) -> &str {
        "Places a limit order when price reaches trigger level"
    }

    fn execute(
        &mut self,
        price_point: &PricePoint,
        market: &mut Market,
        user: Arc<Mutex<User>>,
    ) -> Result<(), &'static str> {
        println!("Executing strategy: Limit Order Strategy");
        if !self.executed && 
           ((self.side == OfferSide::Bid && price_point.price <= self.trigger_price) ||
            (self.side == OfferSide::Ask && price_point.price >= self.trigger_price)) {
                let strategy = Arc::new(Mutex::new(Box::new(self.clone()) as Box<dyn Strategy>));
                let offer = crate::new_offer!(user, self.side, self.trigger_price, self.volume, 100_000, strategy);
                market.place_offer(offer)?;
                println!("Market state: {:?}", market);
                self.executed = true;
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
            "trigger_price" => {
                self.trigger_price = value;
                Ok(())
            }
            "volume" => {
                self.volume = value;
                Ok(())
            }
            _ => Err("Unknown parameter"),
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f64> {
        match name {
            "trigger_price" => Some(self.trigger_price),
            "volume" => Some(self.volume as f64),
            _ => None,
        }
    }
}