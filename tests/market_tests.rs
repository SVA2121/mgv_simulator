use std::sync::{Arc, Mutex};

use mgv_simulator::mgv_lib::{Market, Offer, OfferSide, OrderSide};
use mgv_simulator::chain_lib::{User};
use mgv_simulator::{new_user, new_offer};

const GASREQ: u128 = 100_000;

#[test]
fn test_place_offer() {
    let maker = new_user!("maker", 100000000000000000);
    maker.lock().unwrap().add_token_balance("USDC", 2000);
    
    let offer = new_offer!(maker, OfferSide::Bid, 2000, 1, GASREQ); 
    let mut market = Market::new("WETH".to_string(), "USDC".to_string());
    market.place_offer(offer).unwrap(); 
    assert_eq!(market.best_bid().unwrap().price, 2000);
}

#[test]
fn test_market_order() {
    let maker = new_user!("maker", 100000000000000000);
    maker.lock().unwrap().add_token_balance("USDC", 2000);
    let taker = new_user!("taker", 100000000000000000);
    taker.lock().unwrap().add_token_balance("WETH", 1);

    let mut market = Market::new("WETH".to_string(), "USDC".to_string());

    let offer = new_offer!(maker, OfferSide::Bid, 2000, 1, GASREQ); 
    market.place_offer(offer).unwrap();  


    
    market.market_order(&taker, OrderSide::Sell, 1).unwrap();
    assert_eq!(*maker.lock().unwrap().balances.get("WETH").unwrap(), 1);
    assert_eq!(*taker.lock().unwrap().balances.get("USDC").unwrap(), 2000);

}

#[test]
fn test_post_hook_alternating_offers() {
    let maker = new_user!("maker", 100000000000000000);
    maker.lock().unwrap().add_token_balance("USDC", 2000);
    maker.lock().unwrap().add_token_balance("WETH", 1);
    
    let taker = new_user!("taker", 100000000000000000);
    taker.lock().unwrap().add_token_balance("WETH", 1);
    taker.lock().unwrap().add_token_balance("USDC", 2000);

    let mut market = Market::new("WETH".to_string(), "USDC".to_string());

    // Create initial Ask offer with a post-hook that places a Bid
    let initial_offer = new_offer!(maker, OfferSide::Ask, 2000, 1, GASREQ)
        .with_post_hook(|market, maker| {
            let new_offer = new_offer!(Arc::new(Mutex::new(maker.clone())), OfferSide::Bid, 1900, 1, GASREQ);
            market.place_offer(new_offer).unwrap();
        });
    println!("Market Before: {}", market);
    market.place_offer(initial_offer).unwrap();
    println!("Market After: {}", market);
    // Execute the Ask offer
    market.market_order(&taker, OrderSide::Buy, 1).unwrap();
    println!("Market After Market Order: {}", market);
    // Verify that a new Bid offer was created via post-hook
    assert!(market.best_ask().is_none());
    assert!(market.best_bid().is_some());
    assert_eq!(market.best_bid().unwrap().price, 1900);
}

