use mgv_simulator::chain_lib;  // Changed from 'use crate::chain_lib'

#[test]
fn test_native_token_operations() {
    let mut alice = chain_lib::User::new("alice".to_string(), 1000);
    
    // Test initial state
    assert_eq!(alice.get_native_balance(), 1000);
    
    // Test adding native tokens
    alice.add_native(500);
    assert_eq!(alice.get_native_balance(), 1500);
    
    // Test successful spending
    assert!(alice.spend_native(300).is_ok());
    assert_eq!(alice.get_native_balance(), 1200);
    
    // Test spending more than balance
    assert!(alice.spend_native(2000).is_err());
}

#[test]
fn test_token_operations() {
    let mut alice = chain_lib::User::new("alice".to_string(), 1000);
    
    // Test USDC operations
    alice.add_token_balance("USDC", 1000);
    assert_eq!(alice.get_token_balance("USDC"), 1000);
    
    assert!(alice.spend_token_balance("USDC", 500).is_ok());
    assert_eq!(alice.get_token_balance("USDC"), 500);
    
    // Test spending non-existent token
    assert!(alice.spend_token_balance("WETH", 10).is_err());
    
    // Test spending more than balance
    assert!(alice.spend_token_balance("USDC", 1000).is_err());
}