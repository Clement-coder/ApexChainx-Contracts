#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Env};

#[test]
fn test_initialize_and_get_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SLACalculatorContract);
    let client = SLACalculatorContractClient::new(&env, &contract_id);

    let admin = soroban_sdk::Address::generate(&env);

    client.initialize(&admin);

    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, admin);
}

#[test]
#[should_panic]
fn test_double_initialize_panics() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SLACalculatorContract);
    let client = SLACalculatorContractClient::new(&env, &contract_id);

    let admin = soroban_sdk::Address::generate(&env);

    client.initialize(&admin);
    client.initialize(&admin); // should panic
}