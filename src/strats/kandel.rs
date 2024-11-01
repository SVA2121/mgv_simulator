use crate::strats_lib::Strategy;
use crate::simu_lib::PricePoint;
use crate::mgv_lib::{Market, Offer, OfferSide};
use crate::chain_lib::User;
use std::sync::{Arc, Mutex};

pub struct KandelStrategy {
    price_grid: Vec<f64>,
    reference_price: f64,
    initial_base: f64,
    initial_quote: f64,
    offers: Vec<Offer>,
    initialized: bool,
}

impl KandelStrategy {
    pub fn new() -> Self {
        Self {
            price_grid: Vec::new(),
            reference_price: 0.0,
            initial_base: 0.0,
            initial_quote: 0.0,
            offers: Vec::new(),
            initialized: false,
        }
    }

    pub fn set_parameters(&mut self, price_grid: Vec<f64>, reference_price: f64, initial_base: f64, initial_quote: f64) {
        self.price_grid = price_grid;
        self.reference_price = reference_price;
        self.initial_base = initial_base;
        self.initial_quote = initial_quote;
    }

    fn calculate_volumes(&self) -> (f64, f64) {
        let bids_count = self.price_grid.iter().filter(|&&p| p < self.reference_price).count();
        let asks_count = self.price_grid.iter().filter(|&&p| p > self.reference_price).count();
        
        let volume_per_bid = self.initial_quote / bids_count as f64;
        let volume_per_ask = self.initial_base / asks_count as f64;
        
        (volume_per_bid, volume_per_ask)
    }

    fn create_offer_with_hook(
        &self,
        user: Arc<Mutex<User>>,
        side: OfferSide,
        price: f64,
        volume: f64,
        next_price: f64,
    ) -> Offer {
        let user_clone = Arc::clone(&user);
        let hook: Box<dyn FnMut(&mut Market, &mut User) + Send + Sync + 'static> = Box::new(move |market: &mut Market, _maker: &mut User| {
            let new_offer = crate::new_offer!(
                user_clone.clone(),
                side.flipped(),
                next_price,
                volume,
                100_000
            );
            let _ = market.place_offer(new_offer);
        });

        crate::new_offer!(user, side, price, volume, 100_000)
            .with_post_hook(hook)
    }

}

impl Strategy for KandelStrategy {
    fn name(&self) -> &str {
        "Kandel Strategy"
    }

    fn description(&self) -> &str {
        "Creates a grid of orders that repost on opposite side when filled"
    }


    fn execute(
        &mut self,
        _price_point: &PricePoint,
        market: &mut Market,
        user: Arc<Mutex<User>>,
    ) -> Result<(), &'static str> {
        if self.initialized {
            return Ok(());  // Post-hooks are now handled automatically by the market
        }

        // Initialize the grid
        let (volume_per_bid, volume_per_ask) = self.calculate_volumes();
        
        // Place initial offers
        for i in 0..self.price_grid.len() {
            let price = self.price_grid[i];
            
            if price < self.reference_price {
                // Place bid
                let volume = volume_per_bid / price;
                let next_price = self.price_grid.get(i + 1)
                    .map(|&p| p)
                    .unwrap_or(price);
                
                let offer = self.create_offer_with_hook(
                    user.clone(),
                    OfferSide::Bid,
                    price,
                    volume,
                    next_price,
                );
                market.place_offer(offer.clone())?;
                self.offers.push(offer);
            } else if price > self.reference_price {
                // Place ask
                let volume = volume_per_ask;
                let next_price = self.price_grid.get(i.saturating_sub(1))
                    .map(|&p| p)
                    .unwrap_or(price);
                
                let offer = self.create_offer_with_hook(
                    user.clone(),
                    OfferSide::Ask,
                    price,
                    volume,
                    next_price,
                );
                market.place_offer(offer.clone())?;
                self.offers.push(offer);
            }
        }

        self.initialized = true;
        Ok(())
    }
}