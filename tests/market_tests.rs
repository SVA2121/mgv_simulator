use std::sync::{Arc, Mutex};

use mgv_simulator::mgv_lib::{Market, Offer, OfferSide, OrderSide};
use mgv_simulator::chain_lib::User;
use mgv_simulator::{new_user, new_offer};
use mgv_simulator::strats::{arbitrage::ArbitrageStrategy, kandel::KandelStrategy};
use mgv_simulator::simu_lib::PricePoint;
use mgv_simulator::simu_lib::Simulator;
use mgv_simulator::strats_lib::Strategy;


const GASREQ: u128 = 100_000;



struct DummyStrategy;
impl Strategy for DummyStrategy {
    fn post_hook(&mut self, _market: &mut Market, _user: Arc<Mutex<User>>, _offer: &Offer) -> Result<(), &'static str> {
        Ok(())
    }
    fn name(&self) -> &str {
        "DummyStrategy"
    }
    fn description(&self) -> &str {
        "DummyStrategy"
    }   
    fn execute(&mut self, _price_point: &PricePoint, _market: &mut Market, _user: Arc<Mutex<User>>) -> Result<(), &'static str> {
        Ok(())
    }
}

#[test]
fn test_place_offer() {
    let maker = new_user!("maker", 100000000000000000.0);
    maker.lock().unwrap().add_token_balance("USDC", 2000.0);
    
    let offer = new_offer!(maker, OfferSide::Bid, 2000.0, 1.0, GASREQ, Arc::new(Mutex::new(Box::new(DummyStrategy)))); 
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

    let offer = new_offer!(maker.clone(), OfferSide::Bid, 2000.0, 1.0, GASREQ, Arc::new(Mutex::new(Box::new(DummyStrategy)))); 
    market.place_offer(offer).unwrap();  


    
    market.market_order(&taker, OrderSide::Sell, 1.0).unwrap();
    assert_eq!(*maker.lock().unwrap().balances.get("WETH").unwrap(), 1.0);
    assert_eq!(*taker.lock().unwrap().balances.get("USDC").unwrap(), 2000.0);

}


#[test]
fn test_kandel_with_arb() {
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
        PricePoint::new(10, 100.0), // Final recovery
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
    let reference_price = 100.0;
    let initial_base = 2.0;
    let initial_quote = 200.0; 
    let n_points = 2;
    //let range_multiplier = 0.0;
    let gridstep = 2.0;
    let mut kandel_strat = KandelStrategy::new(
                                            reference_price, 
                                            initial_base, 
                                            initial_quote, 
                                            Some(n_points), 
                                            None, 
                                            Some(gridstep)).unwrap();
    
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