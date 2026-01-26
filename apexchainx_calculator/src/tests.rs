#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Env};

#[test]
fn test_admin_can_set_and_get_config() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SLACalculatorContract);
    let client = SLACalculatorContractClient::new(&env, &contract_id);

    let admin = soroban_sdk::Address::generate(&env);
    let attacker = soroban_sdk::Address::generate(&env);

    client.initialize(&admin);

    
    client.set_config(
        &admin,
        &symbol_short!("critical"),
        &15,
        &100,
        &750,
    );

    let cfg = client.get_config(&symbol_short!("critical"));

    assert_eq!(cfg.threshold_minutes, 15);
    assert_eq!(cfg.penalty_per_minute, 100);
    assert_eq!(cfg.reward_base, 750);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_set_config() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SLACalculatorContract);
    let client = SLACalculatorContractClient::new(&env, &contract_id);

    let admin = soroban_sdk::Address::generate(&env);
    let attacker = soroban_sdk::Address::generate(&env);

    client.initialize(&admin);

    
    client.set_config(
        &attacker,
        &symbol_short!("critical"),
        &15,
        &100,
        &750,
    );
}

#[test]
fn test_defaults_exist_after_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SLACalculatorContract);
    let client = SLACalculatorContractClient::new(&env, &contract_id);

    let admin = soroban_sdk::Address::generate(&env);
    client.initialize(&admin);

    let critical = client.get_config(&symbol_short!("critical"));
    assert_eq!(critical.threshold_minutes, 15);

    let high = client.get_config(&symbol_short!("high"));
    assert_eq!(high.threshold_minutes, 30);

    let medium = client.get_config(&symbol_short!("medium"));
    assert_eq!(medium.threshold_minutes, 60);

    let low = client.get_config(&symbol_short!("low"));
    assert_eq!(low.threshold_minutes, 120);
}