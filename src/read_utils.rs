use std::fs;
use std::error::Error;
use crate::simu_lib::PricePoint;


pub fn read_price_feed(file_path: &str) -> Result<Vec<PricePoint>, Box<dyn Error>> {
    let content = fs::read_to_string(file_path)?;
    
    let mut price_points = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Split by semicolon
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid format at line {}: expected 'block_number;price', got '{}'",
                line_num + 1, line
            ).into());
        }

        // Parse block number - remove any "block_number" prefix if it exists
        let block_str = parts[0].trim().replace("block_number", "").trim().to_string();
        let block = block_str.parse::<u64>()
            .map_err(|_| format!("Invalid block number at line {}: {}", line_num + 1, parts[0]))?;

        // Parse price
        let price = parts[1].trim().parse::<f64>()
            .map_err(|_| format!("Invalid price at line {}: {}", line_num + 1, parts[1]))?;

        price_points.push(PricePoint { block, price });
    }

    Ok(price_points)
}
