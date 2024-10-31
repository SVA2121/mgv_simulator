use mgv_simulator::chain_lib;  // Changed from 'use crate::chain_lib'

#[test]
fn test_native_token_operations() {
    let mut alice = chain_lib::User::new("alice".to_string(), 1000.0);
    
    // Test initial state
    assert_eq!(alice.get_native_balance(), 1000.0);
    
    // Test adding native tokens
    alice.add_native(500.0);
    assert_eq!(alice.get_native_balance(), 1500.0);
    
    // Test successful spending
    assert!(alice.spend_native(300.0).is_ok());
    assert_eq!(alice.get_native_balance(), 1200.0);
    
    // Test spending more than balance
    assert!(alice.spend_native(2000.0).is_err());
}

#[test]
fn test_token_operations() {
    let mut alice = chain_lib::User::new("alice".to_string(), 1000.0);
    
    // Test USDC operations
    alice.add_token_balance("USDC", 1000.0);
    assert_eq!(alice.get_token_balance("USDC"), 1000.0);
    
    assert!(alice.spend_token_balance("USDC", 500.0).is_ok());
    assert_eq!(alice.get_token_balance("USDC"), 500.0);
    
    // Test spending non-existent token
    assert!(alice.spend_token_balance("WETH", 10.0).is_err());
    
    // Test spending more than balance
    assert!(alice.spend_token_balance("USDC", 1000.0).is_err());
}