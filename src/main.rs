use mgv_simulator::mgv_lib::Market;
use mgv_simulator::simu_lib::Simulator;
use mgv_simulator::simu_lib::PricePoint;
use mgv_simulator::strats_lib::StrategyFactory;
use mgv_simulator::strats::delayed_kandel::DelayedKandelStrategy;
use mgv_simulator::strats::arbitrage::ArbitrageStrategy;
use mgv_simulator::read_utils;
use mgv_simulator::utils::inventory::initial_inventory_allocation;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

fn main() -> Result<(), &'static str> {
    let start_time = Instant::now();
    // Read price feed from input file
    let price_feed = read_utils::read_price_feed("data/input/price_feed.txt")
        .map_err(|_| "Failed to read price feed")?;

    // Initialize market and simulator
    let market = Market::new("WETH".to_string(), "USDC".to_string());
    let mut simulator = Simulator::new(market, price_feed.clone());

    // Create users
    let kandel_user = simulator.add_user("kandel_user".to_string(), 10000000000000000000.0);
    // Calculate initial inventory allocation
    let initial_capital = 75000.0;  // Your original USDC amount
    let (base_quantity, quote_quantity) = initial_inventory_allocation(
        price_feed[0].price,  // Use first price as current price
        price_feed[0].price * 0.999,  // 0.1% below current price
        price_feed[0].price * 1.001,  // 0.1% above current price
        initial_capital
    );
    println!("Debug: Base quantity: {}, Quote quantity: {}", base_quantity, quote_quantity);
    kandel_user.lock().unwrap().add_token_balance("WETH", base_quantity);
    kandel_user.lock().unwrap().add_token_balance("USDC", quote_quantity);

    let arb_user = simulator.add_user("arb_user".to_string(), 10000000000000000000.0);
    arb_user.lock().unwrap().add_token_balance("WETH", 100000000.0);
    arb_user.lock().unwrap().add_token_balance("USDC", 100000000.0);

    // Create strategies
    let delayed_kandel = DelayedKandelStrategy::new(3600, 3600, quote_quantity, base_quantity);
    let arbitrage = ArbitrageStrategy::new(0.001, 1.0);

    // Add strategies to simulator
    simulator.add_strategy("delayed_kandel".to_string(), Box::new(delayed_kandel));
    simulator.add_strategy("arbitrage".to_string(), Box::new(arbitrage));

    // Assign strategies to users
    simulator.assign_strategy("kandel_user", "delayed_kandel")?;
    simulator.assign_strategy("arb_user", "arbitrage")?;

    // Run simulation
    let show_progress = true;
    let verbose = false;
    simulator.run_simulation(show_progress, verbose)?;
    
    let duration = start_time.elapsed();
    println!("Simulation completed in: {:?}", duration);

    Ok(())
}