#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Account(Address),
    Balance(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AccountAlreadyExists = 1,
    AccountNotFound = 2,
    InvalidAmount = 3,
    InsufficientBalance = 4,
    ArithmeticOverflow = 5,
    SelfTransferNotAllowed = 6,
}

#[contract]
pub struct GiftingContract;

#[contractimpl]
impl GiftingContract {
    /// Creates a new user account with an initial gift balance.
    /// Requires authentication from the creator.
    pub fn create_account(env: Env, creator: Address, initial_balance: i128) -> Result<(), Error> {
        creator.require_auth();

        if initial_balance <= 0 {
            return Err(Error::InvalidAmount);
        }

        let account_key = DataKey::Account(creator.clone());
        if env.storage().persistent().has(&account_key) {
            return Err(Error::AccountAlreadyExists);
        }

        let balance_key = DataKey::Balance(creator.clone());
        env.storage().persistent().set(&account_key, &true);
        env.storage().persistent().set(&balance_key, &initial_balance);

        env.events().publish(
            (symbol_short!("account"), symbol_short!("created")),
            (creator, initial_balance),
        );

        Ok(())
    }

    /// Returns the balance of the specified creator or account.
    /// Returns 0 if the account has not been created or has no balance.
    pub fn get_balance(env: Env, creator: Address) -> i128 {
        let balance_key = DataKey::Balance(creator);
        env.storage().persistent().get(&balance_key).unwrap_or(0)
    }

    /// Deposits/transfers gifting balance from `from` to `to`.
    /// Requires authorization from `from`.
    pub fn deposit(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        if from == to {
            return Err(Error::SelfTransferNotAllowed);
        }

        let from_account_key = DataKey::Account(from.clone());
        if !env.storage().persistent().has(&from_account_key) {
            return Err(Error::AccountNotFound);
        }

        let from_key = DataKey::Balance(from.clone());
        let from_balance: i128 = env.storage().persistent().get(&from_key).unwrap_or(0);

        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let to_key = DataKey::Balance(to.clone());
        let to_balance: i128 = env.storage().persistent().get(&to_key).unwrap_or(0);

        let new_to_balance = to_balance
            .checked_add(amount)
            .ok_or(Error::ArithmeticOverflow)?;
        let new_from_balance = from_balance
            .checked_sub(amount)
            .ok_or(Error::InsufficientBalance)?;

        env.storage().persistent().set(&from_key, &new_from_balance);
        env.storage().persistent().set(&to_key, &new_to_balance);

        let to_account_key = DataKey::Account(to.clone());
        if !env.storage().persistent().has(&to_account_key) {
            env.storage().persistent().set(&to_account_key, &true);
        }

        env.events().publish(
            (symbol_short!("deposit"), symbol_short!("transfer")),
            (from, to, amount),
        );

        Ok(())
    }

    /// Checks if an account has been registered.
    pub fn has_account(env: Env, account: Address) -> bool {
        let account_key = DataKey::Account(account);
        env.storage().persistent().has(&account_key)
    }
}

#[cfg(test)]
mod test;
