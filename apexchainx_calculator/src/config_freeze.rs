use soroban_sdk::{symbol_short, Env, Symbol};

const FREEZE_KEY: Symbol = symbol_short!("FREEZE");

pub fn freeze_config(env: &Env) {
    env.storage().instance().set(&FREEZE_KEY, &true);
}

pub fn unfreeze_config(env: &Env) {
    env.storage().instance().set(&FREEZE_KEY, &false);
}

pub fn is_config_frozen(env: &Env) -> bool {
    env.storage()
        .instance()
        .get::<Symbol, bool>(&FREEZE_KEY)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};
    use crate::{SLACalculatorContract, SLACalculatorContractClient};

    #[test]
    fn test_config_unfrozen_by_default() {
        let env = Env::default();
        assert!(!is_config_frozen(&env));
    }

    #[test]
    fn test_freeze_and_query() {
        let env = Env::default();
        freeze_config(&env);
        assert!(is_config_frozen(&env));
    }

    #[test]
    fn test_unfreeze_restores_mutable_state() {
        let env = Env::default();
        freeze_config(&env);
        unfreeze_config(&env);
        assert!(!is_config_frozen(&env));
    }

    #[test]
    fn test_frozen_config_blocks_set_config() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, SLACalculatorContract);
        let client = SLACalculatorContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        client.initialize(&admin, &operator);
        freeze_config(&env);
        assert!(is_config_frozen(&env));
    }
}
