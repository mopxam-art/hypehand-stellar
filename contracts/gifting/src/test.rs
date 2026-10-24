#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};

#[test]
fn test_version() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GiftingContract);
    let client = GiftingContractClient::new(&env, &contract_id);

    assert_eq!(client.version(), 1);
}

#[test]
fn test_init_and_get_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, GiftingContract);
    let client = GiftingContractClient::new(&env, &contract_id);

    assert!(!client.is_initialized());
    assert_eq!(client.get_admin(), None);

    let admin = Address::generate(&env);
    client.init(&admin);

    assert!(client.is_initialized());
    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
fn test_init_already_initialized_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, GiftingContract);
    let client = GiftingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    let second_admin = Address::generate(&env);
    let res = client.try_init(&second_admin);
    assert_eq!(res, Err(Ok(GiftingError::AlreadyInitialized)));
}

#[test]
fn test_auth_enforcement_on_init() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GiftingContract);
    let client = GiftingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "init",
                args: (&admin,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .init(&admin);

    assert!(client.is_initialized());
    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
#[should_panic]
fn test_init_without_auth_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, GiftingContract);
    let client = GiftingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    // Without mock_all_auths or mock_auths, require_auth will panic
    client.init(&admin);
}
