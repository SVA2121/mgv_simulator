use std::fmt;
use std::collections::HashMap;

/// Represents a user/wallet in the blockchain with an ID and token native
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub native: u128,  // Using u128 for large token amounts
    pub balances: HashMap<String, u128>
}

impl User {
    /// Creates a new user with given ID and initial native
    pub fn new(id: String, initial_native: u128) -> Self {
        let mut balances = HashMap::new();
        balances.insert("Native".to_string(), initial_native);
        User {
            id,
            native: initial_native,
            balances: HashMap::new(),
        }
    }

    /// Returns the user's current native
    pub fn get_native_balance(&self) -> u128 {
        self.native
    }

    pub fn get_token_balance(&self, token: &str) -> u128 {
        *self.balances.get(token).unwrap_or(&0)
    }



    pub fn add_token_balance(&mut self, token: &str, amount: u128) {
        let balance = self.balances.entry(token.to_string()).or_insert(0);
        *balance = balance.checked_add(amount).unwrap_or_else(|| {
            panic!("Token balance overflow when adding {} {}", amount, token)
        });
    }

    /// Adds tokens to the user's native
    pub fn add_native(&mut self, amount: u128) {
        self.native = self.native.checked_add(amount).unwrap_or_else(|| {
            panic!("Budget overflow when adding {} tokens", amount)
        });
    }

    /// Removes tokens from the user's native if sufficient funds exist
    pub fn spend_native(&mut self, amount: u128) -> Result<(), &'static str> {
        if self.native >= amount {
            self.native = self.native.checked_sub(amount).unwrap();
            Ok(())
        } else {
            Err("Insufficient Gas Funds")
        }
    }

    pub fn spend_token_balance(&mut self, token: &str, amount: u128) -> Result<(), &'static str> {
        let balance = self.balances.get(token).unwrap_or(&0);

        if *balance >= amount {
            let new_balance = balance.checked_sub(amount).unwrap();
            self.balances.insert(token.to_string(), new_balance);
            Ok(())
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
