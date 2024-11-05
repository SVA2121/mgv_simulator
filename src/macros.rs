
#[macro_export]
macro_rules! new_user {
    ($id:expr, $initial_native:expr) => {
        Arc::new(Mutex::new(User::new($id.to_string(), $initial_native)))
    };
}

#[macro_export]
macro_rules! new_offer {
    ($maker:expr, $side:expr, $price:expr, $volume:expr, $gasreq:expr, $strategy:expr) => {
        Offer::new($maker, $side, $price, $volume, $gasreq, $strategy)
    };
}