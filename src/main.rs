use std::sync::{Arc, Mutex};
use mgv_simulator::{
    chain_lib::{User},
    mgv_lib::{Market, Offer, OfferSide, OrderSide}
};

#[macro_use]
extern crate mgv_simulator;

const GASREQ: u128 = 100_000;
fn main() {
    let maker = new_user!("maker", 100000000000000000);
    maker.lock().unwrap().add_token_balance("WETH", 1);
    
    let taker = new_user!("taker", 100000000000000000);
    taker.lock().unwrap().add_token_balance("WETH", 1000);
    
    let offer = new_offer!(maker, OfferSide::Bid, 2000, 1, GASREQ);
    
    let mut market = Market::new("WETH".to_string(), "USDC".to_string());
    println!("{}", market);
    market.place_offer(offer).unwrap();
    println!("{}", market);
    
    println!("Taker: {:?}", taker.lock().unwrap());
    market.market_order(&taker, OrderSide::Sell, 100).unwrap();
    //launch_simul_environment();
    //let mut U

}