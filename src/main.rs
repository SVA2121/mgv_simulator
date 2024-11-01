use mgv_simulator::mgv_lib::Market;
use mgv_simulator::simu_lib::Simulator;
use mgv_simulator::simu_lib::PricePoint;
use mgv_simulator::strats_lib::StrategyFactory;


fn main() -> Result<(), &'static str> {
    // Initialize market and simulator
    let market = Market::new("WETH".to_string(), "USDC".to_string());
    let quote_token = market.quote.clone();
    let base_token = market.base.clone();
    let price_feed = vec![PricePoint::new(0, 1000.0), PricePoint::new(1, 1052.0), PricePoint::new(2, 1002.0)];
    let mut simulator = Simulator::new(market, price_feed);

    // Create user
    let maker = simulator.add_user("maker".to_string(), 1000000.0);
    maker.lock().unwrap().add_token_balance(&quote_token, 1500.0);
    maker.lock().unwrap().add_token_balance(&base_token, 1.0);
    let taker = simulator.add_user("taker".to_string(), 1000000.0);
    
    // Create strategy factory
    let factory = StrategyFactory::new();
    
    // Create and configure strategy
    let mut limit_strategy = factory.create_strategy("limit_order")
        .ok_or("Strategy not found")?;
    limit_strategy.set_parameter("trigger_price", 1500.0)?;
    limit_strategy.set_parameter("volume", 1.0)?;
    
    // Add strategy to simulator
    simulator.add_strategy("limit_1".to_string(), limit_strategy);
    simulator.assign_strategy("maker", "limit_1")?;
    let mut arbitrage_strategy = factory.create_strategy("arbitrage")
        .ok_or("Strategy not found")?;
    arbitrage_strategy.set_parameter("min_profit_threshold", 0.0)?;
    arbitrage_strategy.set_parameter("max_volume_per_trade", 1000.0)?;
    simulator.add_strategy("arbitrage_1".to_string(), arbitrage_strategy);
    simulator.assign_strategy("taker", "arbitrage_1")?;
    
    // Run simulation
    simulator.run_simulation(true)?;
    simulator.print_metrics();
    
    Ok(())
}