use crate::mgv_lib::Market;
use crate::strats_lib::Strategy;
use crate::chain_lib::User;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io::Write;
use std::fs::OpenOptions;


#[derive(Debug, Clone, Copy)]
pub struct PricePoint {
    pub block: u64,
    pub price: f64,
}

impl PricePoint {
    pub fn new(block: u64, price: f64) -> Self {
        Self { block, price }
    }

    pub fn price_equals(&self, other: &PricePoint) -> bool {
        (self.price - other.price).abs() < f64::EPSILON
    }
}

impl std::fmt::Display for PricePoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block: {} Price: {:.2}", self.block, self.price)
    }
}

pub struct Simulator {
    pub market: Market,
    pub price_feed: Vec<PricePoint>,
    pub current_block: u64,
    pub users: HashMap<String, Arc<Mutex<User>>>,
    pub performance_metrics: HashMap<String, PerformanceMetrics>,
    pub strategies: HashMap<String, Box<dyn Strategy>>,              // Added
    pub user_strategies: HashMap<String, Vec<String>>,              // Added
}

#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub total_trades: u64,
    pub total_volume: f64,
    pub total_profit_loss: f64,
    pub initial_balance: f64,
    pub current_balance: f64,
}


impl Simulator {

    pub fn new(market: Market, price_feed: Vec<PricePoint>) -> Self {
        Self {
            market,
            price_feed,
            current_block: 0,
            users: HashMap::new(),
            performance_metrics: HashMap::new(),
            strategies: HashMap::new(),              // Added
            user_strategies: HashMap::new(),         // Added
        }
    }

    pub fn add_user(&mut self, user_id: String, initial_balance: f64) -> Arc<Mutex<User>> {
        let user = crate::new_user!(&user_id, initial_balance);
        self.users.insert(user_id.clone(), Arc::clone(&user));
        self.performance_metrics.insert(user_id, PerformanceMetrics::default());
        user
    }

    pub fn step(&mut self) -> Option<&PricePoint> {
        if self.current_block >= self.price_feed.len() as u64 {
            return None;
        }
        
        let price_point = &self.price_feed[self.current_block as usize];
        self.current_block += 1;
        
        Some(price_point)
    }

    pub fn update_metrics(&mut self, user_id: &str, trade_volume: f64, profit_loss: f64) {
        if let Some(metrics) = self.performance_metrics.get_mut(user_id) {
            metrics.total_trades += 1;
            metrics.total_volume += trade_volume;
            metrics.total_profit_loss += profit_loss;
            // Update current balance from user
            if let Some(user) = self.users.get(user_id) {
                if let Ok(user) = user.lock() {
                    metrics.current_balance = user.get_native_balance() as f64;
                }
            }
        }
    }

    pub fn print_metrics(&self) {
        println!("\n=== Performance Metrics ===");
        for (user_id, metrics) in &self.performance_metrics {
            println!("\nUser: {}", user_id);
            println!("Total Trades: {}", metrics.total_trades);
            println!("Total Volume: {:.2}", metrics.total_volume);
            println!("Total P&L: {:.2}", metrics.total_profit_loss);
            println!("Current Balance: {:.2}", metrics.current_balance);
        }
    }


    pub fn add_strategy(&mut self, strategy_id: String, strategy: Box<dyn Strategy>) {
        self.strategies.insert(strategy_id, strategy);
    }

    pub fn assign_strategy(&mut self, user_id: &str, strategy_id: &str) -> Result<(), &'static str> {
        if !self.users.contains_key(user_id) || !self.strategies.contains_key(strategy_id) {
            return Err("User or strategy not found");
        }
        
        self.user_strategies
            .entry(user_id.to_string())
            .or_default()
            .push(strategy_id.to_string());
        
        Ok(())
    }

    fn write_user_balance(&self, user_id: &str, block: u64, user: &User, truncate: bool) -> std::io::Result<()> {

        let file_path = format!("data/output/{}.txt", user_id);
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(truncate)
            .append(!truncate)
            .open(file_path)?;
        let balance_list: Vec<String> = user.get_balance_list()
            .iter()
            .map(|b| format!("{}", b))
            .collect();

        writeln!(file, "{},{}", block, balance_list.join(","))?;

        Ok(())
    }

    fn write_market_state(&self, block: u64, truncate: bool, price_point: &PricePoint) -> std::io::Result<()> {
        let file_path = "data/output/market_state.txt";
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(truncate)
            .append(!truncate)
            .open(file_path)?;

        // Write market state data for this block
        writeln!(
            file,
            "{},{},{}", 
            block,
            price_point.price,
            self.market
        )?;

        Ok(())
    }

    pub fn run_simulation(&mut self, show_progress: bool, verbose: bool) -> Result<(), &'static str> {
        if verbose {
            println!("Running simulation...");
            println!("Price feed length: {}", self.price_feed.len());
            println!("Users: {:?}", self.users);
            println!("Market: {:?}", self.market);
            println!("--------------------------------");
            println!("--------------------------------");
        }
        let total_steps = self.price_feed.len();
        let progress_interval = total_steps / 10;
        

        let mut last_price_point: Option<PricePoint> = None;
        let mut last_write_block = 0u64;

        // Write initial balance data
        for (user_id, user) in &self.users {
            if let Ok(user) = user.lock() {
                self.write_user_balance(user_id, 0, &user, true);
            }
        }
        if let Err(_) = self.write_market_state(0, true, &self.price_feed[0]) {
            return Err("Failed to write initial market state");
        }
        let mut price_point = &self.price_feed[0];
        while self.current_block < self.price_feed.len() as u64 {
            if show_progress && self.current_block as usize % progress_interval == 0 {
                println!("Simulation progress: {}%", (self.current_block as usize * 100) / total_steps);
            }

            price_point = &self.price_feed[self.current_block as usize];
            if let Some(last_pp) = last_price_point {
                if price_point.price_equals(&last_pp) {
                    // Write balance data for each user
                    for (user_id, user) in &self.users {
                        if let Ok(user) = user.lock() {
                            if let Err(_) = self.write_user_balance(user_id, self.current_block, &user, false) {
                                return Err("Failed to write balance data");
                            }
                        }
                    }
                    if let Err(_) = self.write_market_state(self.current_block, false, price_point) {
                        return Err("Failed to write market state");
                    }
                    self.current_block += 1;
                    continue;
                }
            }
            
            // Collect all the actions we need to take
            let mut actions = Vec::new();
            for (user_id, strategy_ids) in &self.user_strategies {
                if let Some(user) = self.users.get(user_id) {
                    for strategy_id in strategy_ids {
                        if let Some(strategy) = self.strategies.get(strategy_id) {
                            actions.push((strategy_id.clone(), Arc::clone(user)));
                        }
                    }
                }
            }
            
            // Execute the actions
            if verbose && !actions.is_empty() {
                println!("--------------------------------");
                println!("Price point: {}", price_point);
                println!("Market: {}", self.market);
                println!("Actions: {:?}", actions);
            }

            for (strategy_id, user) in actions {
                if let Some(strategy) = self.strategies.get_mut(&strategy_id) {
                    if verbose {
                        println!("Executing strategy: {}", strategy_id);
                        println!("User: {:?}", user);
                    }
                    strategy.execute(price_point, &mut self.market, user)?;
                }
            }

            // Write balance data for each user
            for (user_id, user) in &self.users {
                if let Ok(user) = user.lock() {
                    if let Err(_) = self.write_user_balance(user_id, self.current_block, &user, false) {
                        return Err("Failed to write balance data");
                    }
                }
            }
            if let Err(_) = self.write_market_state(self.current_block, false, price_point) {
                return Err("Failed to write market state");
            }

            self.current_block += 1;
        }

        if show_progress {
            println!("Simulation progress: 100%");
        }

        Ok(())
    }
}