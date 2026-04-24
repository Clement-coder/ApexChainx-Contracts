use soroban_sdk::{Vec, Symbol};
use crate::SLAResult;

pub struct NormalizedSnapshot {
    pub count: u32,
    pub has_violations: bool,
    pub has_rewards: bool,
}

pub fn normalize_history(history: &Vec<SLAResult>) -> NormalizedSnapshot {
    let mut has_violations = false;
    let mut has_rewards = false;

    for i in 0..history.len() {
        let entry = history.get(i).unwrap();
        if entry.status == Symbol::new(history.env(), "viol") {
            has_violations = true;
        }
        if entry.payment_type == Symbol::new(history.env(), "rew") {
            has_rewards = true;
        }
    }

    NormalizedSnapshot {
        count: history.len(),
        has_violations,
        has_rewards,
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};
    use crate::{SLACalculatorContract, SLACalculatorContractClient};

    #[test]
    fn test_history_snapshot_is_deterministic() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, SLACalculatorContract);
        let client = SLACalculatorContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        client.initialize(&admin, &operator);
        client.calculate_sla(&operator, &symbol_short!("OUT1"), &symbol_short!("high"), &10);
        client.calculate_sla(&operator, &symbol_short!("OUT2"), &symbol_short!("high"), &10);
        let stats = client.get_stats();
        assert_eq!(stats.total_calculations, 2);
    }
}
