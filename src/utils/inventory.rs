use std::f64;

/// Implementation of concentrator Univ3 concept / see cash_ratio.pdf
/// 
/// # Arguments
/// * `price` - Current price
/// * `lower_price_bound` - Lower price bound
/// * `upper_price_bound` - Upper price bound
/// 
/// # Panics
/// Panics if price is not within bounds
fn concentrator(price: f64, lower_price_bound: f64, upper_price_bound: f64) -> f64 {
    if price < lower_price_bound || price > upper_price_bound {
        panic!("Price is not in range");
    }
    
    let inverse_concentrator = 2.0 * f64::sqrt(price) 
        - price / f64::sqrt(upper_price_bound) 
        - f64::sqrt(lower_price_bound);
    
    1.0 / inverse_concentrator
}

/// Computes base_quantity(A) and quote_quantity(B) at initialization
/// depending on current price and capital provided.
///
/// # Arguments
/// * `curr_price` - Current price
/// * `p_min` - Price min of the range
/// * `p_max` - Price max of the range
/// * `capital` - Capital provided in stable
///
/// # Returns
/// Returns a tuple of (base_quantity, quote_quantity)
pub fn initial_inventory_allocation(
    curr_price: f64,
    p_min: f64,
    p_max: f64,
    capital: f64,
) -> (f64, f64) {
    // price in range
    if curr_price >= p_min && curr_price <= p_max {
        let concentrator_val = capital * concentrator(curr_price, p_min, p_max);
        let base_quantity = concentrator_val * (1.0 / f64::sqrt(curr_price) - 1.0 / f64::sqrt(p_max));
        let quote_quantity = concentrator_val * (f64::sqrt(curr_price) - f64::sqrt(p_min));
        return (base_quantity, quote_quantity);
    }
    
    // only asset, all asks on indices > 0
    if curr_price < p_min {
        let base_quantity = capital / curr_price;
        let quote_quantity = 0.0;
        return (base_quantity, quote_quantity);
    }
    
    // only cash, all bids on indices < nb_slots - 1
    let base_quantity = 0.0;
    let quote_quantity = capital;
    (base_quantity, quote_quantity)
}