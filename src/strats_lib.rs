use std::collections::HashMap;
use crate::simu_lib::PricePoint;
use crate::chain_lib::User;
use crate::mgv_lib::{Market, Offer};



pub trait Strategy {
    // A strategy is simply a function that modifies the user's balance
    // according to the strategy's logic.
    // It has a setup phase and an execution phase.
    fn setup(&mut self);
    fn execute(&mut self, user: &mut User, price_points: &Vec<PricePoint>, markets: &Vec<Market>);
    fn get_user(&self) -> &User;
    fn set_user(&mut self, user: User);
}




