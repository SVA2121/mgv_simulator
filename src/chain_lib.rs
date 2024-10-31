use std::fmt;
use std::collections::HashMap;

/// Represents a user/wallet in the blockchain with an ID and token native
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub native: f64,  // Using u128 for large token amounts
    pub balances: HashMap<String, f64>
}

impl User {
    /// Creates a new user with given ID and initial native
    pub fn new(id: String, initial_native: f64) -> Self {
        let mut balances = HashMap::new();
        balances.insert("Native".to_string(), initial_native);
        User {
            id,
            native: initial_native,
            balances: HashMap::new(),
        }
    }

    /// Returns the user's current native
    pub fn get_native_balance(&self) -> f64 {
        self.native
    }

    pub fn get_token_balance(&self, token: &str) -> f64 {
        *self.balances.get(token).unwrap_or(&0.0)
    }

    pub fn get_balance_list(&self) -> Vec<f64> {
        let mut balance_list = vec![self.native];
        for (_, balance) in &self.balances {
            balance_list.push(*balance);
        }
        balance_list
    }


    pub fn add_token_balance(&mut self, token: &str, amount: f64) {
        let balance = self.balances.entry(token.to_string()).or_insert(0.0);
        let new_balance = *balance + amount;
        if new_balance.is_finite() {
            *balance = new_balance;
        } else {
            panic!("Token balance overflow when adding {} {}", amount, token);
        }
    }

    /// Adds tokens to the user's native
    pub fn add_native(&mut self, amount: f64) {
        let new_balance = self.native + amount;
        if new_balance.is_finite() {
            self.native = new_balance;
        } else {
            panic!("Native balance overflow when adding {}", amount);
        }
    }

    /// Removes tokens from the user's native if sufficient funds exist
    pub fn spend_native(&mut self, amount: f64) -> Result<(), &'static str> {
        if self.native >= amount {
            let new_balance = self.native - amount;
            if new_balance.is_finite() {
                self.native = new_balance;
                Ok(())
            } else {
                Err("Invalid native balance after subtraction")
            }
        } else {
            Err("Insufficient Gas Funds")
        }
    }

    pub fn spend_token_balance(&mut self, token: &str, amount: f64) -> Result<(), &'static str> {
        let balance = self.balances.get(token).unwrap_or(&0.0);

        if *balance >= amount {
            let new_balance = *balance - amount;
            if new_balance.is_finite() {
                self.balances.insert(token.to_string(), new_balance);
                Ok(())
            } else {
                Err("Invalid token balance after subtraction")
            }
        } else {
            Err("Insufficient token balance")
        }
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User(id: {}, native: {}, balances: {:?})", 
            self.id, self.native, self.balances)
    }
}
