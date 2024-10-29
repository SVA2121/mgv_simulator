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



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offer {
    pub side: Side,
    pub price: u128,
    pub volume: u128,
    pub gasreq: u128,
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



#[derive(Debug, Clone)]
pub struct Market {
    pub bids: Vec<Offer>,
    pub asks: Vec<Offer>,
}

impl Market {
    pub fn new() -> Self {
        Self {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    pub fn insert(&mut self, offer: Offer) {
        match offer.side {
            Side::Bid => {
                self.bids.push(offer);
                // Sort bids in descending order (highest price first)
                self.bids.sort_by(|a, b| b.price.cmp(&a.price));
            }
            Side::Ask => {
                self.asks.push(offer);
                // Sort asks in ascending order (lowest price first)
                self.asks.sort_by(|a, b| a.price.cmp(&b.price));
            }
        }
    }

    pub fn best_bid(&self) -> Option<&Offer> {
        self.bids.first()
    }

    pub fn best_ask(&self) -> Option<&Offer> {
        self.asks.first()
    }
}
