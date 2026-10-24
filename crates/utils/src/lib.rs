//! HypeHand Stellar Utilities Library
//!
//! Provides fundamental helper functions, cryptographic identifier validations,
//! and fixed-point currency conversions for the HypeHand Stellar platform.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Number of stroops in 1 XLM (10^7)
pub const STROOPS_PER_XLM: i128 = 10_000_000;

/// Errors returned by utility operations
#[derive(Error, Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum UtilError {
    #[error("Invalid Stellar account address: {0}")]
    InvalidAccountAddress(String),

    #[error("Invalid Stellar contract address: {0}")]
    InvalidContractAddress(String),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Arithmetic overflow in currency conversion")]
    ConversionOverflow,

    #[error("Expiration timestamp has already passed: current {0}, expiry {1}")]
    AlreadyExpired(u64, u64),
}

/// Stellar address classifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressType {
    /// Stellar Ed25519 public key account (starts with 'G', 56 characters)
    Account,
    /// Soroban Contract ID (starts with 'C', 56 characters)
    Contract,
}

/// Validates whether a string matches standard Stellar public key formatting (G...)
pub fn is_valid_account_address(addr: &str) -> bool {
    addr.len() == 56
        && addr.starts_with('G')
        && addr.chars().all(|c| {
            matches!(c, 'A'..='Z' | '2'..='7')
        })
}

/// Validates whether a string matches standard Soroban contract ID formatting (C...)
pub fn is_valid_contract_address(addr: &str) -> bool {
    addr.len() == 56
        && addr.starts_with('C')
        && addr.chars().all(|c| {
            matches!(c, 'A'..='Z' | '2'..='7')
        })
}

/// Validates and parses any Stellar address into its recognized type
pub fn parse_stellar_address(addr: &str) -> Result<AddressType, UtilError> {
    if is_valid_account_address(addr) {
        Ok(AddressType::Account)
    } else if is_valid_contract_address(addr) {
        Ok(AddressType::Contract)
    } else {
        Err(UtilError::InvalidAccountAddress(addr.to_string()))
    }
}

/// Converts an amount in stroops (10^-7 XLM) to a human-readable floating point string
pub fn stroops_to_xlm_string(stroops: i128) -> String {
    let sign = if stroops < 0 { "-" } else { "" };
    let abs = stroops.abs();
    let whole = abs / STROOPS_PER_XLM;
    let fraction = abs % STROOPS_PER_XLM;
    format!("{}{}.{:07}", sign, whole, fraction)
}

/// Converts an XLM decimal amount string into integer stroops (10^-7)
pub fn xlm_string_to_stroops(xlm: &str) -> Result<i128, UtilError> {
    let parts: Vec<&str> = xlm.trim().split('.').collect();
    if parts.is_empty() || parts.len() > 2 {
        return Err(UtilError::InvalidAmount(xlm.to_string()));
    }

    let whole: i128 = parts[0]
        .parse()
        .map_err(|_| UtilError::InvalidAmount(xlm.to_string()))?;

    let frac_part = if parts.len() == 2 { parts[1] } else { "" };
    if frac_part.len() > 7 {
        return Err(UtilError::InvalidAmount(format!(
            "Excess precision: maximum 7 decimal places allowed, got {}",
            frac_part.len()
        )));
    }

    // Pad right with zeros up to 7 places
    let mut frac_str = frac_part.to_string();
    while frac_str.len() < 7 {
        frac_str.push('0');
    }

    let fraction: i128 = if frac_str.is_empty() {
        0
    } else {
        frac_str
            .parse()
            .map_err(|_| UtilError::InvalidAmount(xlm.to_string()))?
    };

    let total = if whole >= 0 {
        whole
            .checked_mul(STROOPS_PER_XLM)
            .and_then(|w| w.checked_add(fraction))
            .ok_or(UtilError::ConversionOverflow)?
    } else {
        whole
            .checked_mul(STROOPS_PER_XLM)
            .and_then(|w| w.checked_sub(fraction))
            .ok_or(UtilError::ConversionOverflow)?
    };

    Ok(total)
}

/// Validates gift amount against configured minimum and maximum limits
pub fn validate_gift_amount(
    amount: i128,
    min_amount: i128,
    max_amount: i128,
) -> Result<(), UtilError> {
    if amount <= 0 {
        return Err(UtilError::InvalidAmount(
            "Amount must be strictly positive".to_string(),
        ));
    }
    if amount < min_amount {
        return Err(UtilError::InvalidAmount(format!(
            "Amount {} is below minimum {}",
            amount, min_amount
        )));
    }
    if amount > max_amount {
        return Err(UtilError::InvalidAmount(format!(
            "Amount {} exceeds maximum limit {}",
            amount, max_amount
        )));
    }
    Ok(())
}

/// Checks if a gift timelock has expired
pub fn check_expiration(current_time: u64, expiry_time: u64) -> Result<(), UtilError> {
    if current_time >= expiry_time {
        Err(UtilError::AlreadyExpired(current_time, expiry_time))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_ACCOUNT: &str = "GBBD47IF6LWK7P7MDEVG2DP463W7IGFLYTLEWIVPJYTXSTZDHZX6YPOO";
    const VALID_CONTRACT: &str = "CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPFUTDWNGPGZ4R7TFR7TXU2ST";

    #[test]
    fn test_valid_account_address() {
        assert!(is_valid_account_address(VALID_ACCOUNT));
        assert!(!is_valid_account_address(VALID_CONTRACT));
        assert!(!is_valid_account_address("GBBD47")); // too short
        assert!(!is_valid_account_address(&format!("{}1", VALID_ACCOUNT))); // invalid char '1'
    }

    #[test]
    fn test_valid_contract_address() {
        assert!(is_valid_contract_address(VALID_CONTRACT));
        assert!(!is_valid_contract_address(VALID_ACCOUNT));
        assert!(!is_valid_contract_address("CA3D5K")); // too short
    }

    #[test]
    fn test_parse_stellar_address() {
        assert_eq!(
            parse_stellar_address(VALID_ACCOUNT).unwrap(),
            AddressType::Account
        );
        assert_eq!(
            parse_stellar_address(VALID_CONTRACT).unwrap(),
            AddressType::Contract
        );
        assert!(parse_stellar_address("invalid").is_err());
    }

    #[test]
    fn test_stroops_to_xlm_string() {
        assert_eq!(stroops_to_xlm_string(10_000_000), "1.0000000");
        assert_eq!(stroops_to_xlm_string(1), "0.0000001");
        assert_eq!(stroops_to_xlm_string(50_500_000), "5.0500000");
        assert_eq!(stroops_to_xlm_string(0), "0.0000000");
        assert_eq!(stroops_to_xlm_string(-10_000_000), "-1.0000000");
    }

    #[test]
    fn test_xlm_string_to_stroops() {
        assert_eq!(xlm_string_to_stroops("1").unwrap(), 10_000_000);
        assert_eq!(xlm_string_to_stroops("1.0").unwrap(), 10_000_000);
        assert_eq!(xlm_string_to_stroops("1.5").unwrap(), 15_000_000);
        assert_eq!(xlm_string_to_stroops("0.0000001").unwrap(), 1);
        assert_eq!(xlm_string_to_stroops("123.4567890").unwrap(), 1234567890);
    }

    #[test]
    fn test_xlm_string_to_stroops_excess_precision() {
        let err = xlm_string_to_stroops("1.00000001");
        assert!(err.is_err());
    }

    #[test]
    fn test_validate_gift_amount() {
        assert!(validate_gift_amount(100, 10, 1000).is_ok());
        assert!(validate_gift_amount(0, 10, 1000).is_err());
        assert!(validate_gift_amount(-5, 10, 1000).is_err());
        assert!(validate_gift_amount(5, 10, 1000).is_err());
        assert!(validate_gift_amount(1500, 10, 1000).is_err());
    }

    #[test]
    fn test_check_expiration() {
        assert!(check_expiration(1000, 2000).is_ok());
        assert!(check_expiration(2000, 2000).is_err());
        assert!(check_expiration(2500, 2000).is_err());
    }
}
