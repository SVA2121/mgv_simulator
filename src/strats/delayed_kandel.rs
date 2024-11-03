use crate::strats_lib::Strategy;
use crate::simu_lib::PricePoint;
use crate::mgv_lib::{Market, OfferSide};
use crate::chain_lib::User;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub struct DelayedKandelStrategy {
    window_size: usize,
    recalibration_interval: u64,
    price_history: VecDeque<f64>,
    last_calibration: u64,
    kandel_params: KandelParams,
    initialized: bool,
}

struct KandelParams {
    grid_width: f64,      // Percentage around reference price
    num_levels: usize,    // Number of price levels on each side
    base_amount: f64,     // Total base token amount to distribute
    quote_amount: f64,    // Total quote token amount to distribute
}

impl DelayedKandelStrategy {
    pub fn new(window_size: usize, recalibration_interval: u64) -> Self {
        Self {
            window_size,
            recalibration_interval,
            price_history: VecDeque::with_capacity(window_size),
            last_calibration: 0,
            kandel_params: KandelParams {
                grid_width: 0.001,    // 0.1% spread (changed from 5%)
                num_levels: 1,        // Only 1 level on each side (changed from 5)
                base_amount: 1.0,     // 1 base token total
                quote_amount: 1000.0, // 1000 quote tokens total
            },
            initialized: false,
        }
    }

    fn geom_price_grid(&self, price_history: &VecDeque<f64>, spot_price: f64) -> Vec<f64> {
        // Calculate volatility from log returns
        let mut log_returns = Vec::new();
        let mut prev_price = price_history[0];
        
        for &price in price_history.iter().skip(1) {
            log_returns.push((price / prev_price).ln());
            prev_price = price;
        }

        // Calculate volatility (standard deviation of log returns)
        let mean = log_returns.iter().sum::<f64>() / log_returns.len() as f64;
        let variance = log_returns.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (log_returns.len() - 1) as f64;
        let sig = (variance.sqrt() / (365_f64).sqrt()); // Annualized volatility

        // Calculate price grid
        let vol_mult = 1.645; // 90% confidence interval
        let range_multiplier = (vol_mult * sig).exp();
        let grid_step = range_multiplier.powf(1.0 / self.kandel_params.num_levels as f64);

        let mut grid = Vec::with_capacity(2 * self.kandel_params.num_levels + 1);

        // Calculate bids (lower prices)
        for i in (1..=self.kandel_params.num_levels).rev() {
            let price = spot_price / grid_step.powi(i as i32);
            grid.push(price);
        }

        // Add spot price
        grid.push(spot_price);

        // Calculate asks (higher prices)
        for i in 1..=self.kandel_params.num_levels {
            let price = spot_price * grid_step.powi(i as i32);
            grid.push(price);
        }

        grid
    }
     
    

    fn deploy_kandel(&mut self, market: &mut Market, user: Arc<Mutex<User>>) -> Result<(), &'static str> {
        let avg_price = self.price_history.iter().sum::<f64>() / self.price_history.len() as f64;
        let price_grid = self.calculate_grid(&self.price_history, avg_price);
        
        // Create and configure a new Kandel strategy
        let mut kandel = crate::strats::kandel::KandelStrategy::new();
        kandel.set_parameters(
            price_grid,
            avg_price,
            self.kandel_params.base_amount,
            self.kandel_params.quote_amount
        );
        
        // Execute the Kandel strategy
        kandel.execute(&PricePoint::new(0, avg_price), market, user)?;
        
        Ok(())
    }
}

impl Strategy for DelayedKandelStrategy {
    fn name(&self) -> &str {
        "Delayed Kandel Strategy"
    }

    fn description(&self) -> &str {
        "Deploys Kandel strategy after collecting price data and recalibrates periodically"
    }

    fn execute(
        &mut self,
        price_point: &PricePoint,
        market: &mut Market,
        user: Arc<Mutex<User>>,
    ) -> Result<(), &'static str> {
        // Add current price to history
        self.price_history.push_back(price_point.price);
        if self.price_history.len() > self.window_size {
            self.price_history.pop_front();
        }

        // Check if we should deploy/recalibrate
        if self.price_history.len() == self.window_size {
            if !self.initialized || 
               (price_point.block - self.last_calibration >= self.recalibration_interval) {
                // Clear existing orders before recalibrating
                market.bids.clear();
                market.asks.clear();

                // Deploy new Kandel grid
                self.deploy_kandel(market, user)?;
                self.last_calibration = price_point.block;
                self.initialized = true;
            }
        }

        Ok(())
    }
}