use soroban_sdk::Env;
use crate::{SLAConfigSnapshot, SLAResultSchema, CONFIG_KEY, RESULT_SCHEMA_VERSION};

pub struct ConfigBundle {
    pub snapshot: SLAConfigSnapshot,
    pub schema: SLAResultSchema,
}

pub fn read_config_bundle(env: &Env) -> Option<ConfigBundle> {
    let snapshot: SLAConfigSnapshot = env.storage().instance().get(&CONFIG_KEY)?;
    let schema = build_result_schema(env);
    Some(ConfigBundle { snapshot, schema })
}

fn build_result_schema(env: &Env) -> SLAResultSchema {
    use soroban_sdk::symbol_short;
    SLAResultSchema {
        version: symbol_short!("v1"),
        schema_version: RESULT_SCHEMA_VERSION,
        status_met: symbol_short!("met"),
        status_violated: symbol_short!("viol"),
        payment_reward: symbol_short!("rew"),
        payment_penalty: symbol_short!("pen"),
        rating_exceptional: symbol_short!("top"),
        rating_excellent: symbol_short!("excel"),
        rating_good: symbol_short!("good"),
        rating_poor: symbol_short!("poor"),
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env};
    use crate::{SLACalculatorContract, SLACalculatorContractClient};

    #[test]
    fn test_config_bundle_available_after_init() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, SLACalculatorContract);
        let client = SLACalculatorContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        client.initialize(&admin, &operator);
        let bundle = client.get_config_bundle();
        assert!(bundle.is_some());
    }
}
