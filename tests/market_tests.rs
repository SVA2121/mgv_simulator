use std::sync::{Arc, Mutex};

use mgv_simulator::mgv_lib::{Market, Offer, OfferSide, OrderSide};
use mgv_simulator::chain_lib::User;
use mgv_simulator::{new_user, new_offer};
use mgv_simulator::strats::{arbitrage::ArbitrageStrategy, kandel::KandelStrategy};
use mgv_simulator::simu_lib::PricePoint;
use mgv_simulator::simu_lib::Simulator;
const GASREQ: u128 = 100_000;

#[test]
fn test_place_offer() {
    let maker = new_user!("maker", 100000000000000000.0);
    maker.lock().unwrap().add_token_balance("USDC", 2000.0);
    
    let offer = new_offer!(maker, OfferSide::Bid, 2000.0, 1.0, GASREQ); 
    let mut market = Market::new("WETH".to_string(), "USDC".to_string());
    market.place_offer(offer).unwrap(); 
    assert_eq!(market.best_bid().unwrap().price, 2000.0);
}

#[test]
fn test_market_order() {
    let maker = new_user!("maker", 100000000000000000.0);
    maker.lock().unwrap().add_token_balance("USDC", 2000.0);
    let taker = new_user!("taker", 100000000000000000.0);
    taker.lock().unwrap().add_token_balance("WETH", 1.0);

    let mut market = Market::new("WETH".to_string(), "USDC".to_string());

    let offer = new_offer!(maker, OfferSide::Bid, 2000.0, 1.0, GASREQ); 
    market.place_offer(offer).unwrap();  


    
    market.market_order(&taker, OrderSide::Sell, 1.0).unwrap();
    assert_eq!(*maker.lock().unwrap().balances.get("WETH").unwrap(), 1.0);
    assert_eq!(*taker.lock().unwrap().balances.get("USDC").unwrap(), 2000.0);

}

#[test]
fn test_post_hook_alternating_offers() {
    let maker = new_user!("maker", 100000000000000000.0);
    maker.lock().unwrap().add_token_balance("USDC", 2000.0);
    maker.lock().unwrap().add_token_balance("WETH", 1.0);
    
    let taker = new_user!("taker", 100000000000000000.0);
    taker.lock().unwrap().add_token_balance("WETH", 1.0);
    taker.lock().unwrap().add_token_balance("USDC", 2000.0);

    let mut market = Market::new("WETH".to_string(), "USDC".to_string());

    // Create initial Ask offer with a post-hook that places a Bid
    let initial_offer = new_offer!(maker, OfferSide::Ask, 2000.0, 1.0, GASREQ)
        .with_post_hook(|market, maker| {
            let new_offer = new_offer!(Arc::new(Mutex::new(maker.clone())), OfferSide::Bid, 1900.0, 1.0, GASREQ);
            market.place_offer(new_offer).unwrap();
        });
    println!("Market Before: {}", market);
    market.place_offer(initial_offer).unwrap();
    println!("Market After: {}", market);
    // Execute the Ask offer
    market.market_order(&taker, OrderSide::Buy, 1.0).unwrap();
    println!("Market After Market Order: {}", market);
    // Verify that a new Bid offer was created via post-hook
    assert!(market.best_ask().is_none());
    assert!(market.best_bid().is_some());
    assert_eq!(market.best_bid().unwrap().price, 1900.0);
}


#[test]
fn test_kandel_with_arb() {
    // Initialize simulator with market and price feed
    let mut market = Market::new("WETH".to_string(), "USDC".to_string());
    let price_feed = vec![
        PricePoint::new(0, 100.0),  // Initial price
        PricePoint::new(1, 102.0),  // Price moves up
        PricePoint::new(2, 105.0),  // Price moves up more
        PricePoint::new(3, 99.0),   // Price drops
        PricePoint::new(4, 95.0),   // Price drops more
        PricePoint::new(5, 100.0),  // Price returns to initial
    ];
    let mut simulator = Simulator::new(market, price_feed);

    // Create and register users
    let kandel_user = simulator.add_user("kandel".to_string(), 100000000000000000.0);
    kandel_user.lock().unwrap().add_token_balance("WETH", 10.0);
    kandel_user.lock().unwrap().add_token_balance("USDC", 20000.0);

    let arb_user = simulator.add_user("arb".to_string(), 100000000000000000.0);
    arb_user.lock().unwrap().add_token_balance("WETH", 10.0);
    arb_user.lock().unwrap().add_token_balance("USDC", 20000.0);

    // Create and configure strategies
    let mut kandel_strat = KandelStrategy::new();
    let price_grid = vec![95.0, 97.0, 99.0, 101.0, 103.0, 105.0];
    kandel_strat.set_parameters(price_grid, 100.0, 10.0, 1000.0);

    let arb_strat = ArbitrageStrategy::new(0.5, 100000000.0);

    // Register strategies with simulator
    simulator.add_strategy("kandel_strat".to_string(), Box::new(kandel_strat));
    simulator.add_strategy("arb_strat".to_string(), Box::new(arb_strat));

    // Assign strategies to users
    simulator.assign_strategy("kandel", "kandel_strat").unwrap();
    simulator.assign_strategy("arb", "arb_strat").unwrap();

    // Run simulation
    simulator.run_simulation(true).unwrap();

    // Verify final state
    let kandel_final = kandel_user.lock().unwrap();
    let arb_final = arb_user.lock().unwrap();

    println!("Kandel final WETH: {}", kandel_final.balances.get("WETH").unwrap());
    println!("Kandel final USDC: {}", kandel_final.balances.get("USDC").unwrap());
    println!("Arb final WETH: {}", arb_final.balances.get("WETH").unwrap());
    println!("Arb final USDC: {}", arb_final.balances.get("USDC").unwrap());

    // Print metrics
    simulator.print_metrics();
}