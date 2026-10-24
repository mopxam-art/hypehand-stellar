#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

/// Error codes returned by the Gifting smart contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GiftingError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
}

/// Storage keys for the smart contract instance data
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Initialized,
}

/// Gifting smart contract managing off-chain and on-chain gifting logic
#[contract]
pub struct GiftingContract;

#[contractimpl]
impl GiftingContract {
    /// Returns the semantic contract version
    pub fn version(_env: Env) -> u32 {
        1
    }

    /// Initializes the smart contract with an administrator address
    pub fn init(env: Env, admin: Address) -> Result<(), GiftingError> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(GiftingError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);

        Ok(())
    }

    /// Checks if the smart contract has been initialized
    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    /// Returns the current administrator address, if initialized
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }
}

mod test;
