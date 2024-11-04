
use crate::chain_lib::User;
use std::sync::{Arc, Mutex};

const OFFER_WRITE_COST: u128 = 200_000; // TO CHECK
//const OFFER_DELETE_COST: u128 = 100_000; // TO CHECK

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfferSide {
    Ask,
    Bid,
}

impl OfferSide {
    pub fn flipped(&self) -> Self {
        match self {
            Self::Ask => Self::Bid,
            Self::Bid => Self::Ask,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

////////////////////////
// Offer
///////////////////////

pub struct Offer {
    pub maker:  Arc<Mutex<User>>,
    pub side: OfferSide,
    pub price: f64,
    pub volume: f64,
    pub gasreq: u128,
    pub post_hook: Option<Arc<Mutex<Box<dyn FnMut(&mut Market, &mut User) + Send + 'static>>>>,

}

impl std::fmt::Debug for Offer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Offer")
            .field("maker", &self.maker)
            .field("side", &self.side)
            .field("price", &self.price)
            .field("volume", &self.volume)
            .field("gasreq", &self.gasreq)
            .field("post_hook", &if self.post_hook.is_some() { "Some(FnMut)" } else { "None" })
            .finish()
    }
}

impl Clone for Offer {
    fn clone(&self) -> Self {
        Self {
            maker: Arc::clone(&self.maker),
            side: self.side,
            price: self.price,
            volume: self.volume,
            gasreq: self.gasreq,
            post_hook: None, // Note: we can't clone the post_hook closure
        }
    }
}

impl Offer {
    pub fn new(maker: Arc<Mutex<User>>, side: OfferSide, price: f64, volume: f64, gasreq: u128) -> Self {
        Self {
            maker,
            side,
            price,
            volume,
            gasreq,
            post_hook: None,
        }
    }

    pub fn with_post_hook<F>(mut self, hook: F) -> Self 
    where 
        F: FnMut(&mut Market, &mut User) + Send + 'static
    {
        self.post_hook = Some(Arc::new(Mutex::new(Box::new(hook))));
        self
    }

    pub fn execute_post_hook(&mut self, market: &mut Market, maker: &mut User) {
        if let Some(hook) = &self.post_hook {
            if let Ok(mut hook) = hook.lock() {
                hook(market, maker);
            }
        }
    }
}


impl PartialOrd for Offer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.side {
            OfferSide::Ask => self.price.partial_cmp(&other.price),
            OfferSide::Bid => other.price.partial_cmp(&self.price),
        }
    }
}

impl Ord for Offer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Offer {
    fn eq(&self, other: &Self) -> bool {
        self.side == other.side 
            && self.price == other.price 
            && self.volume == other.volume 
            && self.gasreq == other.gasreq
    }
}

impl Eq for Offer {}



////////////////////////
// Market
///////////////////////
pub struct Market {
    pub base: String,
    pub quote: String,
    pub bids: Vec<Offer>,
    pub asks: Vec<Offer>,
    pub offer_write_cost: u128,
}

impl Market {
    pub fn new(base: String, quote: String) -> Self {
        Self {
            base,
            quote,
            bids: Vec::new(),
            asks: Vec::new(),
            offer_write_cost: OFFER_WRITE_COST,
        }
    }

    fn insert(&mut self, offer: Offer) {
        match offer.side {
            OfferSide::Bid => {
                self.bids.push(offer);
                self.bids.sort_by(|a, b| b.price.partial_cmp(&a.price).expect("price compare error"));
            }
            OfferSide::Ask => {
                self.asks.push(offer);
                self.asks.sort_by(|a, b| a.price.partial_cmp(&b.price).expect("price compare error"));
            }
        }
    }

    // Add a new method that requires a User to insert an offer
    pub fn place_offer(&mut self, offer: Offer) -> Result<(), &'static str> {
        // Calculate required gas cost
        let gas_cost = self.offer_write_cost;
        
        // Check if user can pay for gas
        offer.maker.lock().unwrap().spend_native(gas_cost as f64)?;
        
        self.insert(offer);
        Ok(())
    }


    pub fn best_bid(&self) -> Option<&Offer> {
        self.bids.first()
    }

    pub fn best_ask(&self) -> Option<&Offer> {
        self.asks.first()
    }

 
    pub fn market_order(&mut self, taker: &Arc<Mutex<User>>, side: OrderSide, volume: f64) -> Result<(), &'static str> {
        let offers = match side {
            OrderSide::Buy => &self.asks,  // If user wants to buy (bid), look at asks
            OrderSide::Sell => &self.bids,  // If user wants to sell (ask), look at bids
        };
        
        // Calculate total volume and gas requirements
        let mut remaining_volume = volume;
        let mut total_gas = 0u128;
        let mut offers_to_execute = 0;
        
        for offer in offers {
            if remaining_volume == 0.0 {
                break;
            }
            total_gas += offer.gasreq;
            remaining_volume -= offer.volume;
            offers_to_execute += 1;
        }
        
        // Check if we can fill the order
        if remaining_volume > 0.0 {
            return Err("Insufficient liquidity");
        }

        taker.lock().unwrap().spend_native(total_gas as f64)?;

        let mut remaining_volume = volume;
        let offers_to_remove = offers_to_execute;  // Store how many offers we'll process
    
        for _ in 0..offers_to_remove {
            let offer = match side {
                OrderSide::Buy => self.asks.remove(0),  // Remove from front of asks
                OrderSide::Sell => self.bids.remove(0),  // Remove from front of bids
            };
            
            let trade_volume = remaining_volume.min(offer.volume);
            let trade_amount = trade_volume * offer.price;
            

            // Transfer tokens
            match side {
                OrderSide::Buy => {
                    // Taker sends quote tokens, receives base tokens
                    if let Err(e) = taker.lock().unwrap().spend_token_balance(&self.quote, trade_amount as f64) {
                        let user_id = taker.lock().unwrap().id.clone();
                        println!("Error: User {} failed to spend {} {}: {}", user_id, trade_amount, self.quote, e);
                        return Err("Insufficient token balance for taker");
                    }
                    if let Err(e) = offer.maker.lock().unwrap().add_token_balance(&self.quote, trade_amount as f64) {
                        let user_id = offer.maker.lock().unwrap().id.clone();
                        println!("Error: User {} failed to receive {} {}: {}", user_id, trade_amount, self.quote, e);
                        return Err("Failed to add token balance to maker");
                    }
                    if let Err(e) = taker.lock().unwrap().add_token_balance(&self.base, trade_volume as f64) {
                        let user_id = taker.lock().unwrap().id.clone();
                        println!("Error: User {} failed to receive {} {}: {}", user_id, trade_volume, self.base, e);
                        return Err("Failed to add token balance to taker");
                    }
                    if let Err(e) = offer.maker.lock().unwrap().spend_token_balance(&self.base, trade_volume as f64) {
                        let user_id = offer.maker.lock().unwrap().id.clone();
                        println!("Error: User {} failed to spend {} {}: {}", user_id, trade_volume, self.base, e);
                        return Err("Insufficient token balance for maker");
                    }
                }
                OrderSide::Sell => {
                    // Taker sends base tokens, receives quote tokens
                    if let Err(e) = taker.lock().unwrap().spend_token_balance(&self.base, trade_volume as f64) {
                        let user_id = taker.lock().unwrap().id.clone();
                        println!("Error: User {} failed to spend {} {}: {}", user_id, trade_volume, self.base, e);
                        return Err("Insufficient token balance for taker");
                    }
                    if let Err(e) = offer.maker.lock().unwrap().add_token_balance(&self.base, trade_volume as f64) {
                        let user_id = offer.maker.lock().unwrap().id.clone();
                        println!("Error: User {} failed to receive {} {}: {}", user_id, trade_volume, self.base, e);
                        return Err("Failed to add token balance to maker");
                    }
                    if let Err(e) = taker.lock().unwrap().add_token_balance(&self.quote, trade_amount as f64) {
                        let user_id = taker.lock().unwrap().id.clone();
                        println!("Error: User {} failed to receive {} {}: {}", user_id, trade_amount, self.quote, e);
                        return Err("Failed to add token balance to taker");
                    }
                    if let Err(e) = offer.maker.lock().unwrap().spend_token_balance(&self.quote, trade_amount as f64) {
                        let user_id = offer.maker.lock().unwrap().id.clone();
                        println!("Error: User {} failed to spend {} {}: {}", user_id, trade_amount, self.quote, e);
                        return Err("Insufficient token balance for maker");
                    }
                }
            }
            
            // Execute post-hook if any
            if let Some(hook) = &offer.post_hook {
                let mut maker = offer.maker.lock().unwrap();
                if let Ok(mut hook) = hook.lock() {
                    hook(self, &mut maker);
                }
            }
            
            remaining_volume -= trade_volume;
            if remaining_volume == 0.0 {
                break;
            }
        }

        Ok(())
    }
     
    

}

impl std::fmt::Display for Market {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Market:")?;

        writeln!(f, "  Asks:")?;
        for ask in self.asks.iter().rev() {
            let user_id = ask.maker.lock().map(|user| user.id.clone()).unwrap_or_else(|_| "locked".to_string()); 
            writeln!(f, "    {} @ {} - {},{}", ask.volume, ask.price, user_id, ask.post_hook.is_some())?;
        }
        
        writeln!(f, "  Bids:")?;
        for bid in &self.bids {
            let user_id = bid.maker.lock().map(|user| user.id.clone()).unwrap_or_else(|_| "locked".to_string());
            writeln!(f, "    {} @ {} - {},{}", bid.volume, bid.price, user_id, bid.post_hook.is_some())?;
        }
        
        Ok(())
    }
}

impl std::fmt::Debug for Market {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self) // This will use our Display implementation
    }
}