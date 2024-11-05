use crate::strats_lib::Strategy;
use crate::simu_lib::PricePoint;
use crate::mgv_lib::{Market, Offer};
use crate::chain_lib::User;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub struct ActiveKandelStrategy {
    window_size: usize,
    recalibration_interval: u64,
    price_history: VecDeque<f64>,
    last_calibration: u64,
    kandel_params: KandelParams,
    initialized: bool,
}

struct KandelParams {
    reference_price: f64,
    base_amount: f64,
    quote_amount: f64,
    n_points: usize,
    range_multiplier: f64,
    gridstep: f64,
}

impl ActiveKandelStrategy {
    pub fn new(window_size: usize, recalibration_interval: u64, quote_amount: f64, base_amount: f64) -> Self {
        Self {
            window_size,
            recalibration_interval,
            price_history: VecDeque::with_capacity(window_size),
            last_calibration: 0,

            kandel_params: KandelParams {
                reference_price: 0.0,
                n_points: 0,
                range_multiplier: 0.0,
                gridstep: 0.0,
                base_amount: base_amount,
                quote_amount: quote_amount,
            },
            initialized: false,
        }
    }

    fn set_parameters(
        &mut self, 
        reference_price: f64, 
        base_amount: f64, 
        quote_amount: f64, 
        n_points: Option<usize>, 
        range_multiplier: Option<f64>, 
        gridstep: Option<f64>
    ) -> Result<(), &'static str> {
        self.kandel_params.reference_price = reference_price;
        self.kandel_params.base_amount = base_amount;
        self.kandel_params.quote_amount = quote_amount;
        self.kandel_params.n_points = n_points.unwrap();
        self.kandel_params.range_multiplier = range_multiplier.unwrap();
        self.kandel_params.gridstep = gridstep.unwrap();
        Ok(())
    }
    

    fn deploy_kandel(&mut self, market: &mut Market, user: Arc<Mutex<User>>) -> Result<(), &'static str> {
        // Create and configure a new Kandel strategy
        let mut kandel = crate::strats::kandel::KandelStrategy::new(
            self.kandel_params.reference_price,
            self.kandel_params.base_amount,
            self.kandel_params.quote_amount,
            Some(self.kandel_params.n_points),
            Some(self.kandel_params.range_multiplier),
            Some(self.kandel_params.gridstep)
        )?; // Add ? here to propagate the error
        
        // Execute the Kandel strategy
        kandel.execute(&PricePoint::new(0, 0.0), market, user)?;
        
        Ok(())
    }
}

impl Strategy for ActiveKandelStrategy {
    fn name(&self) -> &str {
        "Active Kandel Strategy"
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

    fn post_hook(
        &mut self,
        market: &mut Market,
        maker: Arc<Mutex<User>>,
        filled_offer: &Offer,
    ) -> Result<(), &'static str> {
        Ok(())
    }
}
