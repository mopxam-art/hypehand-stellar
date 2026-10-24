use gifting_contract::{Error, GiftingContract, GiftingContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env,
};

/// Integration test harness fixture providing environment management,
/// account generation, ledger time simulation, and contract interactions.
pub struct GiftingTestHarness {
    pub env: Env,
    pub contract_id: Address,
}

impl GiftingTestHarness {
    /// Initialize a new isolated Soroban environment with mocked auths
    pub fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, GiftingContract);
        Self { env, contract_id }
    }

    /// Initialize harness with unmocked auth for testing authorization boundaries
    pub fn new_unauthed() -> Self {
        let env = Env::default();
        let contract_id = env.register_contract(None, GiftingContract);
        Self { env, contract_id }
    }

    /// Access typed contract client
    pub fn client(&self) -> GiftingContractClient<'static> {
        GiftingContractClient::new(&self.env, &self.contract_id)
    }

    /// Generate a fresh random address
    pub fn create_user(&self) -> Address {
        Address::generate(&self.env)
    }

    /// Generate a user and prime their account with an initial balance
    pub fn create_user_with_balance(&self, balance: i128) -> Address {
        let user = self.create_user();
        self.client().create_account(&user, &balance);
        user
    }

    /// Simulate ledger time progression (advancing timestamp and sequence number)
    pub fn advance_ledger_time(&self, delta_seconds: u64) {
        let mut ledger_info = self.env.ledger().get();
        ledger_info.timestamp = ledger_info.timestamp.saturating_add(delta_seconds);
        ledger_info.sequence_number = ledger_info.sequence_number.saturating_add(1);
        self.env.ledger().set(ledger_info);
    }

    /// Retrieve current simulated ledger timestamp
    pub fn current_timestamp(&self) -> u64 {
        self.env.ledger().timestamp()
    }
}

impl Default for GiftingTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test_e2e_multi_user_lifecycle_and_transfers() {
    let harness = GiftingTestHarness::new();
    let client = harness.client();

    // 1. Create accounts for Alice and Bob with initial balances
    let alice = harness.create_user_with_balance(10_000);
    let bob = harness.create_user_with_balance(5_000);
    let charlie = harness.create_user(); // Unregistered recipient initially

    assert_eq!(client.get_balance(&alice), 10_000);
    assert_eq!(client.get_balance(&bob), 5_000);
    assert_eq!(client.get_balance(&charlie), 0);
    assert!(!client.has_account(&charlie));

    // 2. Alice sends a gift of 3_000 to Bob
    client.deposit(&alice, &bob, &3_000);
    assert_eq!(client.get_balance(&alice), 7_000);
    assert_eq!(client.get_balance(&bob), 8_000);

    // 3. Advance ledger time by 1 hour (3600 seconds)
    harness.advance_ledger_time(3600);

    // 4. Bob sends a gift of 4_000 to Charlie (auto-registering Charlie)
    client.deposit(&bob, &charlie, &4_000);
    assert_eq!(client.get_balance(&bob), 4_000);
    assert_eq!(client.get_balance(&charlie), 4_000);
    assert!(client.has_account(&charlie));

    // 5. Advance ledger time by 2 hours (7200 seconds)
    harness.advance_ledger_time(7200);

    // 6. Charlie gifts 1_500 back to Alice
    client.deposit(&charlie, &alice, &1_500);
    assert_eq!(client.get_balance(&charlie), 2_500);
    assert_eq!(client.get_balance(&alice), 8_500);

    // 7. Verify conservation of total supply: 10_000 + 5_000 = 15_000
    let total = client.get_balance(&alice) + client.get_balance(&bob) + client.get_balance(&charlie);
    assert_eq!(total, 15_000);
}

#[test]
fn test_e2e_boundary_zero_and_negative_conditions() {
    let harness = GiftingTestHarness::new();
    let client = harness.client();

    let user_a = harness.create_user();
    let user_b = harness.create_user();

    // Boundary: Zero initial balance is rejected
    let res_zero = client.try_create_account(&user_a, &0);
    assert_eq!(res_zero, Err(Ok(Error::InvalidAmount)));

    // Boundary: Negative initial balance is rejected
    let res_neg = client.try_create_account(&user_a, &-100);
    assert_eq!(res_neg, Err(Ok(Error::InvalidAmount)));

    // Now properly prime user_a
    client.create_account(&user_a, &1_000);
    client.create_account(&user_b, &500);

    // Boundary: Zero deposit amount is rejected
    let res_dep_zero = client.try_deposit(&user_a, &user_b, &0);
    assert_eq!(res_dep_zero, Err(Ok(Error::InvalidAmount)));

    // Boundary: Negative deposit amount is rejected
    let res_dep_neg = client.try_deposit(&user_a, &user_b, &-50);
    assert_eq!(res_dep_neg, Err(Ok(Error::InvalidAmount)));

    // Boundary: Overdrawing account balance is rejected
    let res_overdraw = client.try_deposit(&user_a, &user_b, &1_500);
    assert_eq!(res_overdraw, Err(Ok(Error::InsufficientBalance)));

    // Ensure state remained untouched after all boundary failures
    assert_eq!(client.get_balance(&user_a), 1_000);
    assert_eq!(client.get_balance(&user_b), 500);
}

#[test]
fn test_e2e_rapid_circular_transfers_conservation() {
    let harness = GiftingTestHarness::new();
    let client = harness.client();

    // Create 5 users with 1,000 each (Total = 5,000)
    let users: [Address; 5] = core::array::from_fn(|_| harness.create_user_with_balance(1_000));

    // Circular transfer: user[i] -> user[(i+1)%5]
    for i in 0..5 {
        let next_idx = (i + 1) % 5;
        client.deposit(&users[i], &users[next_idx], &200);
        harness.advance_ledger_time(60);
    }

    // Since every user sent 200 and received 200, each should still have exactly 1,000
    for user in &users {
        assert_eq!(client.get_balance(user), 1_000);
    }
}

#[test]
fn test_e2e_ledger_time_and_sequence_advancement() {
    let harness = GiftingTestHarness::new();
    let client = harness.client();

    let initial_ts = harness.current_timestamp();
    let user = harness.create_user_with_balance(500);

    // Advance 7 days (604,800 seconds)
    harness.advance_ledger_time(604_800);
    assert_eq!(harness.current_timestamp(), initial_ts + 604_800);

    // Verify storage persists across simulated time
    assert_eq!(client.get_balance(&user), 500);
    assert!(client.has_account(&user));
}

#[test]
fn test_e2e_empty_account_queries() {
    let harness = GiftingTestHarness::new();
    let client = harness.client();

    let random_user = harness.create_user();
    assert_eq!(client.get_balance(&random_user), 0);
    assert!(!client.has_account(&random_user));
}

#[test]
#[should_panic]
fn test_e2e_unauthorized_create_account_rejected() {
    let harness = GiftingTestHarness::new_unauthed();
    let user = harness.create_user();
    // Must panic on missing auth signature
    harness.client().create_account(&user, &1_000);
}

#[test]
#[should_panic]
fn test_e2e_unauthorized_deposit_rejected() {
    let harness = GiftingTestHarness::new_unauthed();
    let user_a = harness.create_user();
    let user_b = harness.create_user();
    // Must panic on missing auth signature
    harness.client().deposit(&user_a, &user_b, &100);
}
