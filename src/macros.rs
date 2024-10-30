use std::sync::{Arc, Mutex};
use crate::chain_lib::User;
use crate::mgv_lib::{Offer};

#[macro_export]
macro_rules! new_user {
    ($id:expr, $initial_native:expr) => {
        Arc::new(Mutex::new(User::new($id.to_string(), $initial_native)))
    };
}

#[macro_export]
macro_rules! new_offer {
    ($user:expr, $side:expr, $price:expr, $volume:expr, $gasreq:expr) => {
        Offer::new(Arc::clone(&$user), $side, $price, $volume, $gasreq)
    };
}