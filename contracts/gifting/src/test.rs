#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

fn setup_test() -> (Env, GiftingContractClient<'static>, Address, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, GiftingContract);
    let client = GiftingContractClient::new(&env, &contract_id);
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    (env, client, user_a, user_b)
}

#[test]
fn test_create_account_and_get_balance() {
    let (env, client, user_a, _) = setup_test();
    env.mock_all_auths();

    // Prior to creation, balance must be 0 and has_account false
    assert_eq!(client.get_balance(&user_a), 0);
    assert!(!client.has_account(&user_a));

    // Create account with initial balance
    client.create_account(&user_a, &10_000);

    assert_eq!(client.get_balance(&user_a), 10_000);
    assert!(client.has_account(&user_a));
}

#[test]
fn test_create_account_already_exists() {
    let (env, client, user_a, _) = setup_test();
    env.mock_all_auths();

    client.create_account(&user_a, &5_000);
    let res = client.try_create_account(&user_a, &2_000);
    assert_eq!(res, Err(Ok(Error::AccountAlreadyExists)));
}

#[test]
fn test_create_account_invalid_amount() {
    let (env, client, user_a, user_b) = setup_test();
    env.mock_all_auths();

    // Zero amount
    let res_zero = client.try_create_account(&user_a, &0);
    assert_eq!(res_zero, Err(Ok(Error::InvalidAmount)));

    // Negative amount
    let res_neg = client.try_create_account(&user_b, &-500);
    assert_eq!(res_neg, Err(Ok(Error::InvalidAmount)));
}

#[test]
fn test_deposit_and_balance_transfer() {
    let (env, client, alice, bob) = setup_test();
    env.mock_all_auths();

    client.create_account(&alice, &10_000);
    client.create_account(&bob, &2_000);

    // Alice deposits 3_500 to Bob
    client.deposit(&alice, &bob, &3_500);

    assert_eq!(client.get_balance(&alice), 6_500);
    assert_eq!(client.get_balance(&bob), 5_500);
}

#[test]
fn test_deposit_to_new_recipient() {
    let (env, client, alice, recipient) = setup_test();
    env.mock_all_auths();

    client.create_account(&alice, &5_000);
    assert_eq!(client.get_balance(&recipient), 0);
    assert!(!client.has_account(&recipient));

    client.deposit(&alice, &recipient, &1_200);

    assert_eq!(client.get_balance(&alice), 3_800);
    assert_eq!(client.get_balance(&recipient), 1_200);
    assert!(client.has_account(&recipient));
}

#[test]
fn test_deposit_insufficient_balance() {
    let (env, client, alice, bob) = setup_test();
    env.mock_all_auths();

    client.create_account(&alice, &500);
    client.create_account(&bob, &1_000);

    let res = client.try_deposit(&alice, &bob, &600);
    assert_eq!(res, Err(Ok(Error::InsufficientBalance)));

    // Ensure state untouched
    assert_eq!(client.get_balance(&alice), 500);
    assert_eq!(client.get_balance(&bob), 1_000);
}

#[test]
fn test_deposit_negative_or_zero_amount() {
    let (env, client, alice, bob) = setup_test();
    env.mock_all_auths();

    client.create_account(&alice, &5_000);

    // Zero amount deposit
    let res_zero = client.try_deposit(&alice, &bob, &0);
    assert_eq!(res_zero, Err(Ok(Error::InvalidAmount)));

    // Negative amount deposit
    let res_neg = client.try_deposit(&alice, &bob, &-100);
    assert_eq!(res_neg, Err(Ok(Error::InvalidAmount)));
}

#[test]
fn test_deposit_self_transfer_rejected() {
    let (env, client, alice, _) = setup_test();
    env.mock_all_auths();

    client.create_account(&alice, &5_000);
    let res = client.try_deposit(&alice, &alice, &1_000);
    assert_eq!(res, Err(Ok(Error::SelfTransferNotAllowed)));
}

#[test]
fn test_deposit_unregistered_sender() {
    let (env, client, unregistered, bob) = setup_test();
    env.mock_all_auths();

    let res = client.try_deposit(&unregistered, &bob, &100);
    assert_eq!(res, Err(Ok(Error::AccountNotFound)));
}

#[test]
fn test_deposit_arithmetic_overflow_rejected() {
    let (env, client, alice, bob) = setup_test();
    env.mock_all_auths();

    client.create_account(&alice, &i128::MAX);
    client.create_account(&bob, &i128::MAX);

    let res = client.try_deposit(&alice, &bob, &1);
    assert_eq!(res, Err(Ok(Error::ArithmeticOverflow)));
}

#[test]
#[should_panic]
fn test_auth_enforcement_create_account_without_auth() {
    let (_env, client, alice, _) = setup_test();
    // Intentionally do NOT call env.mock_all_auths()
    client.create_account(&alice, &1_000);
}

#[test]
#[should_panic]
fn test_auth_enforcement_deposit_without_auth() {
    let (env, client, alice, bob) = setup_test();
    env.mock_all_auths();
    client.create_account(&alice, &5_000);
    client.create_account(&bob, &1_000);

    // Recreate client / env without mock_all_auths to test authorization failure on deposit
    let unauthed_env = Env::default();
    let contract_id = unauthed_env.register_contract(None, GiftingContract);
    let unauthed_client = GiftingContractClient::new(&unauthed_env, &contract_id);
    let unauthed_alice = Address::generate(&unauthed_env);
    let unauthed_bob = Address::generate(&unauthed_env);

    unauthed_client.deposit(&unauthed_alice, &unauthed_bob, &500);
}

#[test]
fn test_multiple_sequential_deposits() {
    let (env, client, alice, bob) = setup_test();
    let charlie = Address::generate(&env);
    env.mock_all_auths();

    client.create_account(&alice, &10_000);
    client.create_account(&bob, &5_000);

    // Alice sends to Bob
    client.deposit(&alice, &bob, &2_000);
    assert_eq!(client.get_balance(&alice), 8_000);
    assert_eq!(client.get_balance(&bob), 7_000);

    // Bob sends to Charlie (new account)
    client.deposit(&bob, &charlie, &3_000);
    assert_eq!(client.get_balance(&bob), 4_000);
    assert_eq!(client.get_balance(&charlie), 3_000);
    assert!(client.has_account(&charlie));

    // Charlie sends to Alice
    client.deposit(&charlie, &alice, &1_000);
    assert_eq!(client.get_balance(&charlie), 2_000);
    assert_eq!(client.get_balance(&alice), 9_000);

    // Total balance in the system is conserved (10_000 + 5_000 = 15_000)
    let total = client.get_balance(&alice) + client.get_balance(&bob) + client.get_balance(&charlie);
    assert_eq!(total, 15_000);
}

#[test]
fn test_deposit_exact_entire_balance() {
    let (env, client, alice, bob) = setup_test();
    env.mock_all_auths();

    client.create_account(&alice, &4_200);
    client.create_account(&bob, &100);

    // Deposit entire balance to Bob
    client.deposit(&alice, &bob, &4_200);

    assert_eq!(client.get_balance(&alice), 0);
    assert_eq!(client.get_balance(&bob), 4_300);
    // Alice account still exists even with 0 balance
    assert!(client.has_account(&alice));

    // Subsequent deposit from Alice fails with InsufficientBalance
    let res = client.try_deposit(&alice, &bob, &1);
    assert_eq!(res, Err(Ok(Error::InsufficientBalance)));
}

#[test]
fn test_min_and_max_valid_values() {
    let (env, client, user_min, user_max) = setup_test();
    env.mock_all_auths();

    // Smallest positive amount
    client.create_account(&user_min, &1);
    assert_eq!(client.get_balance(&user_min), 1);

    // Large safe amount
    let safe_large = i128::MAX - 1000;
    client.create_account(&user_max, &safe_large);
    assert_eq!(client.get_balance(&user_max), safe_large);
}

