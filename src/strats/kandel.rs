use crate::strats_lib::Strategy;
use crate::simu_lib::PricePoint;
use crate::mgv_lib::{Market, Offer, OfferSide};
use crate::chain_lib::User;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct KandelStrategy {
    price_grid: Vec<f64>,
    reference_price: f64,
    initial_quote: f64,
    initial_base: f64,
    offers: Vec<Offer>,
    initialized: bool,
    n_points: usize,
    range_multiplier: f64,
    gridstep: f64,
}

impl KandelStrategy {

    fn calculate_parameters(
        n_points: Option<usize>,
        range_multiplier: Option<f64>,
        gridstep: Option<f64>,
    ) -> Result<(usize, f64, f64), &'static str> {
        // Ensure exactly 2 parameters are provided
        let params_count = [n_points.is_some(), range_multiplier.is_some(), gridstep.is_some()]
            .iter()
            .filter(|&&x| x)
            .count();
        if params_count != 2 {
            return Err("Exactly 2 out of 3 parameters must be provided");
        }

        // Calculate the missing parameter
        match (n_points, range_multiplier, gridstep) {
            (None, Some(r), Some(g)) => {
                if r <= 1.0 {
                    return Err("Range multiplier must be greater than 1");
                }
                if g <= 0.0 {
                    return Err("Grid step must be positive");
                }
                // n = vol_multiplier / c in the Python code
                let n = ((r.ln() / g.ln()).ceil() as usize) / 2;
                Ok((n, r, g))
            },
            (Some(n), None, Some(g)) => {
                if n == 0 {
                    return Err("Number of points must be positive");
                }
                if g <= 0.0 {
                    return Err("Grid step must be positive");
                }
                let r = g.powf((n * 2) as f64);
                Ok((n, r, g))
            },
            (Some(n), Some(r), None) => {
                if n == 0 {
                    return Err("Number of points must be positive");
                }
                if r <= 0.0 || r >= 1.0 {
                    return Err("Range multiplier must be between 0 and 1");
                }
                let g = (2.0 * r) / (2.0 * n as f64);
                Ok((n, r, g))
            },
            _ => unreachable!(),
        }
    }

    fn calculate_grid(
        reference_price: f64,
        n_points: usize,
        range_multiplier: f64,
        gridstep: f64,
    ) -> Vec<f64> {
        println!("n_points: {}", n_points);
        println!("range_multiplier: {}", range_multiplier);
        println!("gridstep: {}", gridstep);

        let max_price = reference_price * range_multiplier;
        let min_price = reference_price / range_multiplier;
        let mut lower_prices = Vec::with_capacity(n_points);
        let mut higher_prices = Vec::with_capacity(n_points);
        
        // Calculate prices above reference price
        let mut current_price = reference_price;
        for _ in 0..n_points {
            current_price *= gridstep;
            if current_price > max_price {
                break;
            }
            higher_prices.push(current_price);
        }
        
        // Calculate prices below reference price
        let mut current_price = reference_price;
        for _ in 0..n_points {
            current_price /= gridstep;
            if current_price < min_price {
                break;
            }
            lower_prices.push(current_price);
        }

        // Combine all prices in correct order
        let mut price_grid = Vec::with_capacity(2 * n_points + 1);
        price_grid.extend(lower_prices.iter().rev()); // Add lower prices in ascending order
        price_grid.push(reference_price);             // Add reference price in the middle
        price_grid.extend(higher_prices);             // Add higher prices
    
        price_grid
    }


    pub fn new(
        reference_price: f64,
        initial_quote: f64,
        initial_base: f64,
        n_points: Option<usize>,
        range_multiplier: Option<f64>,
        gridstep: Option<f64>,
    ) -> Result<Self, &'static str> {
        if reference_price <= 0.0 {
            return Err("Reference price must be positive");
        }
        if initial_quote <= 0.0 {
            return Err("Initial quote must be positive");
        }

        let (n_points, range_multiplier, gridstep) = 
            Self::calculate_parameters(n_points, range_multiplier, gridstep)?;
        
        let price_grid = Self::calculate_grid(reference_price, n_points, range_multiplier, gridstep);

        Ok(Self {
            price_grid,
            reference_price,
            initial_quote,
            initial_base,
            offers: Vec::new(),
            initialized: false,
            n_points,
            range_multiplier,
            gridstep,
        })
    }

    

    pub fn set_parameters(
        &mut self, 
        reference_price: f64, 
        initial_base: f64, 
        initial_quote: f64, 
        n_points: Option<usize>, 
        range_multiplier: Option<f64>, 
        gridstep: Option<f64>
    ) -> Result<(), &'static str> {
        if reference_price <= 0.0 {
            return Err("Reference price must be positive");
        }
        if initial_quote <= 0.0 {
            return Err("Initial quote must be positive");
        }

        let (n_points, range_multiplier, gridstep) = 
            Self::calculate_parameters(n_points, range_multiplier, gridstep)?;
        
        self.price_grid = Self::calculate_grid(reference_price, n_points, range_multiplier, gridstep);
        self.reference_price = reference_price;
        self.initial_base = initial_base;
        self.initial_quote = initial_quote;
        Ok(())
    }

    // For testing purposes just not be used in production
    pub fn set_price_grid(&mut self, price_grid: Vec<f64>) {
        self.price_grid = price_grid;
    }

    fn calculate_volumes(&self) -> (f64, f64) {
        let bids_count = self.price_grid.iter().filter(|&&p| p < self.reference_price).count();
        let asks_count = self.price_grid.iter().filter(|&&p| p > self.reference_price).count();
        
        let volume_per_bid = self.initial_base / bids_count as f64;
        let volume_per_ask = self.initial_quote / asks_count as f64;
        
        (volume_per_bid, volume_per_ask)
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
        let strategy: Arc<Mutex<Box<dyn Strategy>>> = Arc::new(Mutex::new(Box::new(self.clone())));
        
        // Place initial offers
        for i in 0..self.price_grid.len() {
            let price = self.price_grid[i];
            
            if price < self.reference_price {
                // Place bid
                let volume = volume_per_bid / self.reference_price;
                let offer = Offer::new(
                    user.clone(),
                    OfferSide::Bid,
                    price,
                    volume,
                    100_000,
                    Arc::clone(&strategy),
                );
                market.place_offer(offer.clone())?;
                self.offers.push(offer);
            } else if price > self.reference_price {
                // Place ask
                let volume = volume_per_ask;
                let offer = Offer::new(
                    user.clone(),
                    OfferSide::Ask,
                    price,
                    volume,
                    100_000,
                    Arc::clone(&strategy),
                );
                market.place_offer(offer.clone())?;
                self.offers.push(offer);
            }
        }

        self.initialized = true;
        Ok(())
    }

    fn post_hook(
        &mut self,
        market: &mut Market,
        maker: Arc<Mutex<User>>,
        filled_offer: &Offer,
    ) -> Result<(), &'static str> {
        let flipped_side = filled_offer.side.flipped();
        
        // Find the next price in the grid
        let next_price = if filled_offer.side == OfferSide::Bid {
            self.price_grid.iter()
                .find(|&&p| p > filled_offer.price)
                .copied()
                .unwrap_or(filled_offer.price)
        } else {
            self.price_grid.iter()
                .rev()
                .find(|&&p| p < filled_offer.price)
                .copied()
                .unwrap_or(filled_offer.price)
        };
        let quote_amount = filled_offer.price * filled_offer.volume;
        let new_volume = match flipped_side {
            OfferSide::Bid => quote_amount / next_price, // For bids, convert quote to base at new price
            OfferSide::Ask => quote_amount / filled_offer.price, // For asks, use original quote amount
        };
        // Create new offer on the opposite side
        let new_offer = Offer::new(
            maker,
            flipped_side,
            next_price,
            new_volume,
            100_000,
            Arc::clone(&filled_offer.strategy), // Reuse the same strategy reference
        );

        market.place_offer(new_offer)
    }
}