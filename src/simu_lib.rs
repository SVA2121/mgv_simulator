// use std::collections::HashMap;
// use crate::chain_lib::User;
// use std::fs;
// use std::io;
// use std::io::Write;
// //use crate::strats_lib::StrategyLibrary;
// use crate::read_utils::read_price_feed;
// use crate::mgv_lib::Market;
// // Represents a price point at a specific block
#[derive(Debug, Clone)]
pub struct PricePoint {
    pub block: u64,
    pub price: f64,
}

// // Main simulator struct
// pub struct Simulator {
//     input_data: Vec<PricePoint>,
//     users: HashMap<String, User>,
//     strategies: HashMap<String, Box<dyn Fn(&PricePoint, &mut User)>>,
//     user_strategies: HashMap<String, Vec<String>>
// }

// impl Simulator {
//     pub fn new(input_data: Vec<PricePoint>) -> Self {
//         Simulator {
//             input_data,
//             users: HashMap::new(),
//             strategies: HashMap::new(),
//             user_strategies: HashMap::new(),
//         }
//     }

//     // Add a user to the simulation
//     pub fn add_user(&mut self, user_id: String, initial_balance: u128) {
//         self.users.insert(user_id.clone(), User::new(user_id, initial_balance));
//     }

//     pub fn add_strategy(&mut self, strategy_id: String, strategy: Box<dyn Fn(&PricePoint, &mut User)>) {
//         self.strategies.insert(strategy_id, strategy);
//     }

//     pub fn subscribe_user_to_strategy(&mut self, user_id: &str, strategy_id: &str) {
//         self.user_strategies
//             .entry(user_id.to_string())
//             .or_insert_with(Vec::new)
//             .push(strategy_id.to_string());
//     }

//     // Run the simulation with a given strategy
//     pub fn run(&mut self) {
//         for strategy in self.strategies.values_mut() {
//             // TO DO : add multiple price feeds
//             for price_point in &self.input_data {
//                 println!("Price point: {:?}", price_point);
//                 for (_user_id, user) in self.users.iter_mut() {
//                     strategy(price_point, user);
//                     println!("User: {:?}", user);
//                 }
//             }
//         }
//     }
// }

// pub fn launch_simul_environment() {
//     // 1. List available price feeds
//     println!("Available price feeds:");
//     let paths = fs::read_dir("data/input/").unwrap();
//     let input_data_files: Vec<String> = paths
//         .filter_map(|entry| {
//             entry.ok().and_then(|e| 
//                 e.path()
//                     .file_name()
//                     .and_then(|n| n.to_str().map(String::from))
//             )
//         })
//         .collect();

//     for (i, file) in input_data_files.iter().enumerate() {
//         println!("{}: {}", i + 1, file);
//     }

//     // 2. Get user selection for price feed
//     print!("Select price feed (enter number): ");
//     io::stdout().flush().unwrap();
//     let mut input = String::new();
//     io::stdin().read_line(&mut input).unwrap();
//     let selection: usize = input.trim().parse().unwrap();
//     let selected_file = &input_data_files[selection - 1];

//     // 3. Load price feed
//     let input_data = read_input_data(&format!("data/input/{}", selected_file));

//     let mut market = Market::new(
//         "WETH".to_string(),
//         "USDC".to_string(),
//     );
//     // 4. Initialize strategy library and show options
//     let strategy_lib = StrategyLibrary::new();
//     let limit_order = LimitOrder::new(market, 1000.0, 100);
//     strategy_lib.add_strategy("limit_order", Box::new(limit_order));
//     println!("\nAvailable strategies:");
//     for (i, strategy_name) in strategy_lib.list_strategies().iter().enumerate() {
//         println!("{}: {}", i + 1, strategy_name);
//     }

//     // 5. Get user selection for strategy
//     print!("Select strategy (enter number): ");
//     io::stdout().flush().unwrap();
//     let mut input = String::new();
//     io::stdin().read_line(&mut input).unwrap();
//     let strategy_selection: usize = input.trim().parse().unwrap();
//     let selected_strategy = &strategy_lib.list_strategies()[strategy_selection - 1];

//     // 6. Set up simulation environment
//     let mut simulator = Simulator::new(input_data.expect("Failed to read price feed"));
    
//     // Add a test user
//     simulator.add_user("test_user".to_string(), 1000);
    
//     // Add selected strategy
//     // Add selected strategy
//     if let Some(strategy) = strategy_lib.get_strategy(selected_strategy) {
//         simulator.add_strategy(
//             selected_strategy.to_string(),
//             Box::new(strategy)  // This works because fn is 'static
//         );
//     }

//     // 7. Run simulation
//     simulator.run();
    
//     println!("Simulation complete!");
// }