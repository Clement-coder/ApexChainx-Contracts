#[cfg(test)]
mod outage_id_tests {
    use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};
    use crate::{SLACalculatorContract, SLACalculatorContractClient};

    fn setup(env: &Env) -> (Address, Address, SLACalculatorContractClient) {
        env.mock_all_auths();
        let contract_id = env.register_contract(None, SLACalculatorContract);
        let client = SLACalculatorContractClient::new(env, &contract_id);
        let admin = Address::generate(env);
        let operator = Address::generate(env);
        client.initialize(&admin, &operator);
        (admin, operator, client)
    }

    #[test]
    fn test_repeated_outage_id_each_recorded() {
        let env = Env::default();
        let (_admin, operator, client) = setup(&env);
        let outage_id = symbol_short!("INC01");

        client.calculate_sla(&operator, &outage_id, &symbol_short!("high"), &10);
        client.calculate_sla(&operator, &outage_id, &symbol_short!("high"), &10);

        let stats = client.get_stats();
        assert_eq!(stats.total_calculations, 2);
    }

    #[test]
    fn test_repeated_outage_id_results_are_consistent() {
        let env = Env::default();
        let (_admin, operator, client) = setup(&env);
        let outage_id = symbol_short!("INC02");

        let r1 = client.calculate_sla(&operator, &outage_id, &symbol_short!("high"), &10);
        let r2 = client.calculate_sla(&operator, &outage_id, &symbol_short!("high"), &10);

        assert_eq!(r1.status, r2.status);
        assert_eq!(r1.amount, r2.amount);
    }

    #[test]
    fn test_different_outage_ids_tracked_independently() {
        let env = Env::default();
        let (_admin, operator, client) = setup(&env);

        client.calculate_sla(&operator, &symbol_short!("A"), &symbol_short!("high"), &5);
        client.calculate_sla(&operator, &symbol_short!("B"), &symbol_short!("high"), &50);

        let stats = client.get_stats();
        assert_eq!(stats.total_calculations, 2);
    }
}
