
use crate::chain_lib::User;

const OFFER_WRITE_COST: u128 = 200_000; // TO CHECK
//const OFFER_DELETE_COST: u128 = 100_000; // TO CHECK

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Ask,
    Bid,
}

impl Side {
    pub fn flipped(&self) -> Self {
        match self {
            Self::Ask => Self::Bid,
            Self::Bid => Self::Ask,
        }
    }
}

////////////////////////
// Offer
///////////////////////

pub struct Offer {
    pub user_id: String,
    pub side: Side,
    pub price: u128,
    pub volume: u128,
    pub gasreq: u128,
    pub post_hook: Option<Box<dyn FnMut(&mut Market, &mut User) + Send + 'static>>,
}

impl std::fmt::Debug for Offer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Offer")
            .field("user_id", &self.user_id)
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
            user_id: self.user_id.clone(),
            side: self.side,
            price: self.price,
            volume: self.volume,
            gasreq: self.gasreq,
            post_hook: None, // Note: we can't clone the post_hook closure
        }
    }
}

impl Offer {
    pub fn new(user_id: String, side: Side, price: u128, volume: u128, gasreq: u128) -> Self {
        Self {
            user_id,
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
        self.post_hook = Some(Box::new(hook));
        self
    }

    pub fn execute_post_hook(&mut self, market: &mut Market, user: &mut User) {
        if let Some(hook) = &mut self.post_hook {
            hook(market, user);
        }
    }
}


impl PartialOrd for Offer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.side {
            Side::Ask => self.price.partial_cmp(&other.price),
            Side::Bid => other.price.partial_cmp(&self.price),
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
    pub fn new() -> Self {
        Self {
            bids: Vec::new(),
            asks: Vec::new(),
            offer_write_cost: OFFER_WRITE_COST,
        }
    }

    fn insert(&mut self, offer: Offer) {
        match offer.side {
            Side::Bid => {
                self.bids.push(offer);
                self.bids.sort_by(|a, b| b.price.cmp(&a.price));
            }
            Side::Ask => {
                self.asks.push(offer);
                self.asks.sort_by(|a, b| a.price.cmp(&b.price));
            }
        }
    }

    // Add a new method that requires a User to insert an offer
    pub fn place_offer(&mut self, user: &mut User, offer: Offer) -> Result<(), &'static str> {
        // Calculate required gas cost
        let gas_cost = self.offer_write_cost;
        
        // Check if user can pay for gas
        user.spend_native(gas_cost)?;
        
        self.insert(offer);
        Ok(())
    }


    pub fn best_bid(&self) -> Option<&Offer> {
        self.bids.first()
    }

    pub fn best_ask(&self) -> Option<&Offer> {
        self.asks.first()
    }

    

}

impl std::fmt::Display for Market {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Market:")?;
        
        writeln!(f, "  Bids:")?;
        for bid in &self.bids {
            writeln!(f, "    {} @ {} - {}", bid.volume, bid.price, bid.post_hook.is_some())?;
        }
        
        writeln!(f, "  Asks:")?;
        for ask in &self.asks {
            writeln!(f, "    {} @ {} - {}", ask.volume, ask.price, ask.post_hook.is_some())?;
        }
        
        Ok(())
    }
}

impl std::fmt::Debug for Market {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self) // This will use our Display implementation
    }
}