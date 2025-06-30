use std::str::FromStr;
use solana_sdk::pubkey::Pubkey;

/// Helper function to validate and parse a required string field from an Option.
pub fn validate_required_string(field: &Option<String>, field_name: &str) -> Result<String, String> {
    match field {
        Some(s) if !s.is_empty() => Ok(s.clone()),
        Some(_) => Err(format!("Field '{}' cannot be empty", field_name)),
        None => Err(format!("Missing required field: {}", field_name)),
    }
}

/// Helper function to validate and parse a required numeric field from an Option.
pub fn validate_required_numeric<T: Copy>(field: &Option<T>, field_name: &str) -> Result<T, String> {
    match field {
        Some(value) => Ok(*value),
        None => Err(format!("Missing required field: {}", field_name)),
    }
}

/// Helper function to validate and parse Pubkey from a string.
pub fn validate_pubkey(pubkey_str: &str, field_name: &str) -> Result<Pubkey, String> {
    Pubkey::from_str(pubkey_str)
        .map_err(|_| format!("Invalid '{}' address provided", field_name))
}

/// Helper function to validate amount (must be greater than 0).
pub fn validate_amount(amount: Option<u64>, field_name: &str) -> Result<u64, String> {
    match amount {
        Some(a) if a > 0 => Ok(a),
        Some(_) => Err("Amount must be greater than 0".to_string()),
        None => Err(format!("Missing required field: {}", field_name)),
    }
} 