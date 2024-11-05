use mgv_simulator::mgv_lib::Market;
use mgv_simulator::simu_lib::Simulator;
use mgv_simulator::simu_lib::PricePoint;
use mgv_simulator::strats_lib::StrategyFactory;
use mgv_simulator::strats::kandel::KandelStrategy;
use mgv_simulator::strats::active_kandel::ActiveKandelStrategy;
use mgv_simulator::strats::arbitrage::ArbitrageStrategy;
use mgv_simulator::read_utils;
use mgv_simulator::utils::inventory::initial_inventory_allocation;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

fn main() -> Result<(), &'static str> {
    // Initialize simulator with market and price feed
    let mut market = Market::new("WETH".to_string(), "USDC".to_string());
    let price_feed = vec![
        PricePoint::new(0, 100.0),  // Initial price
        PricePoint::new(1, 101.0),  // Price moves up
        PricePoint::new(2, 103.0),  // Continues up
        PricePoint::new(3, 104.0),  // Peak
        PricePoint::new(4, 102.0),  // First drop
        PricePoint::new(5, 96.0),   // Sharp decline
        PricePoint::new(6, 94.0),   // Bottom
        PricePoint::new(7, 96.0),   // Recovery begins
        PricePoint::new(8, 98.0),   // Continues recovering
        PricePoint::new(9, 97.0),   // Small pullback
        PricePoint::new(10, 100.0),
        PricePoint::new(11, 101.0),  // Price moves up
        PricePoint::new(12, 103.0),  // Continues up
        PricePoint::new(13, 104.0),  // Peak
        PricePoint::new(14, 102.0),  // First drop
        PricePoint::new(15, 96.0),   // Sharp decline
        PricePoint::new(16, 94.0),   // Bottom
        PricePoint::new(17, 96.0),   // Recovery begins
        PricePoint::new(18, 98.0),   // Continues recovering
        PricePoint::new(19, 97.0),   // Final recovery
        PricePoint::new(20, 101.0),
        PricePoint::new(21, 100.0),
        PricePoint::new(22, 101.0),  // Price moves up
        PricePoint::new(23, 103.0),  // Continues up
        PricePoint::new(24, 104.0),  // Peak
        PricePoint::new(25, 102.0),  // First drop
        PricePoint::new(26, 96.0),   // Sharp decline
        PricePoint::new(27, 94.0),   // Bottom
        PricePoint::new(28, 96.0),   // Recovery begins
        PricePoint::new(29, 98.0),   // Continues recovering
        PricePoint::new(30, 97.0),   // Final recovery
        PricePoint::new(31, 101.0), // Final recovery
    ];
    let mut simulator = Simulator::new(market, price_feed);

    // Create and register users
    let kandel_user = simulator.add_user("kandel".to_string(), 100000000000000000.0);
    kandel_user.lock().unwrap().add_token_balance("WETH", 2.0);
    kandel_user.lock().unwrap().add_token_balance("USDC", 200.0);

    let arb_user = simulator.add_user("arb".to_string(), 100000000000000000.0);
    //arb_user.lock().unwrap().add_token_balance("WETH", 10.0);
    //arb_user.lock().unwrap().add_token_balance("USDC", 20000.0);

    // Create and configure strategies
    let reference_price = 100.0;
    let initial_base = 2.0;
    let initial_quote = 200.0; 
    let n_points = 2;
    //let range_multiplier = 0.0;
    let gridstep = 1.0202;
    let mut kandel_strat = KandelStrategy::new(
                                            reference_price, 
                                            initial_base, 
                                            initial_quote, 
                                            Some(n_points), 
                                            None, 
                                            Some(gridstep)).unwrap();
    kandel_strat.set_price_grid(vec![96.0, 98.0, 100.0, 102.0, 104.0]);
    
    let arb_strat = ArbitrageStrategy::new(0.0, 100000000.0);

    // Register strategies with simulator
    simulator.add_strategy("kandel_strat".to_string(), Box::new(kandel_strat));
    simulator.add_strategy("arb_strat".to_string(), Box::new(arb_strat));

    // Assign strategies to users
    simulator.assign_strategy("kandel", "kandel_strat").unwrap();
    simulator.assign_strategy("arb", "arb_strat").unwrap();

    // Run simulation 
    let show_progress = true;
    let verbose = true;
    simulator.run_simulation(show_progress, verbose).unwrap();
    println!("Simulation completed");
    println!("Market: {}", simulator.market);

    // Verify final state
    let kandel_final = kandel_user.lock().unwrap();
    let arb_final = arb_user.lock().unwrap();

    println!("Kandel final WETH: {}", kandel_final.balances.get("WETH").unwrap());
    println!("Kandel final USDC: {}", kandel_final.balances.get("USDC").unwrap());
    println!("Arb final WETH: {}", arb_final.balances.get("WETH").unwrap());
    println!("Arb final USDC: {}", arb_final.balances.get("USDC").unwrap());

    // Print metrics
    simulator.print_metrics();

    Ok(())
}

// fn main() -> Result<(), &'static str> {
//     let start_time = Instant::now();
//     // Read price feed from input file
//     let price_feed = read_utils::read_price_feed("data/input/price_feed.txt")
//         .map_err(|_| "Failed to read price feed")?;

//     // Initialize market and simulator
//     let market = Market::new("WETH".to_string(), "USDC".to_string());
//     let mut simulator = Simulator::new(market, price_feed.clone());

//     // Create users
//     let kandel_user = simulator.add_user("kandel_user".to_string(), 10000000000000000000.0);
//     // Calculate initial inventory allocation
//     let initial_capital = 75000.0;  // Your original USDC amount
//     let (base_quantity, quote_quantity) = initial_inventory_allocation(
//         price_feed[0].price,  // Use first price as current price
//         price_feed[0].price * 0.999,  // 0.1% below current price
//         price_feed[0].price * 1.001,  // 0.1% above current price
//         initial_capital
//     );
//     println!("Debug: Base quantity: {}, Quote quantity: {}", base_quantity, quote_quantity);
//     kandel_user.lock().unwrap().add_token_balance("WETH", base_quantity);
//     kandel_user.lock().unwrap().add_token_balance("USDC", quote_quantity);

//     let arb_user = simulator.add_user("arb_user".to_string(), 10000000000000000000.0);
//     arb_user.lock().unwrap().add_token_balance("WETH", 100000000.0);
//     arb_user.lock().unwrap().add_token_balance("USDC", 100000000.0);

//     // Create strategies
//     let active_kandel = ActiveKandelStrategy::new(3600, 3600, quote_quantity, base_quantity);
//     let arbitrage = ArbitrageStrategy::new(0.001, 1.0);

//     // Add strategies to simulator
//     simulator.add_strategy("active_kandel".to_string(), Box::new(active_kandel));
//     simulator.add_strategy("arbitrage".to_string(), Box::new(arbitrage));

//     // Assign strategies to users
//     simulator.assign_strategy("kandel_user", "delayed_kandel")?;
//     simulator.assign_strategy("arb_user", "arbitrage")?;

//     // Run simulation
//     let show_progress = true;
//     let verbose = false;
//     simulator.run_simulation(show_progress, verbose)?;
    
//     let duration = start_time.elapsed();
//     println!("Simulation completed in: {:?}", duration);

//     Ok(())
// }