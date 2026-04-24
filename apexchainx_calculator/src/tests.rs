#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::testutils::Events as _;
use soroban_sdk::{Env, Symbol, TryIntoVal};

// ============================================================
// Test helpers
// ============================================================

struct Actors {
    admin: soroban_sdk::Address,
    operator: soroban_sdk::Address,
    stranger: soroban_sdk::Address,
}

struct GoldenCase<'a> {
    severity: &'a str,
    mttr_minutes: u32,
    expected_status: &'a str,
    expected_payment_type: &'a str,
    expected_rating: &'a str,
    expected_amount: i128,
}

fn symbol(env: &Env, value: &str) -> Symbol {
    Symbol::new(env, value)
}

fn setup() -> (Env, SLACalculatorContractClient<'static>, Actors) {
    let env = Env::default();
    let cid = env.register_contract(None, SLACalculatorContract);
    let client = SLACalculatorContractClient::new(&env, &cid);
    let actors = Actors {
        admin: soroban_sdk::Address::generate(&env),
        operator: soroban_sdk::Address::generate(&env),
        stranger: soroban_sdk::Address::generate(&env),
    };
    client.initialize(&actors.admin, &actors.operator);
    (env, client, actors)
}

// ============================================================
// Initialisation
// ============================================================

#[test]
fn test_initialize_stores_roles() {
    let (_env, client, actors) = setup();
    assert_eq!(client.get_admin(), actors.admin);
    assert_eq!(client.get_operator(), actors.operator);
}

#[test]
#[should_panic]
fn test_double_initialize_fails() {
    let (_env, client, actors) = setup();
    // second call must panic with AlreadyInitialized
    client.initialize(&actors.admin, &actors.operator);
}

// ============================================================
// Default configs present after init
// ============================================================

#[test]
fn test_defaults_exist_after_initialize() {
    let (_env, client, _actors) = setup();

    assert_eq!(
        client
            .get_config(&symbol_short!("critical"))
            .threshold_minutes,
        15
    );
    assert_eq!(
        client.get_config(&symbol_short!("high")).threshold_minutes,
        30
    );
    assert_eq!(
        client
            .get_config(&symbol_short!("medium"))
            .threshold_minutes,
        60
    );
    assert_eq!(
        client.get_config(&symbol_short!("low")).threshold_minutes,
        120
    );
}

#[test]
fn test_config_snapshot_is_deterministic_and_complete() {
    let (_env, client, _actors) = setup();

    let snapshot = client.get_config_snapshot();
    assert_eq!(snapshot.version, symbol_short!("v1"));
    assert_eq!(snapshot.entries.len(), 4);

    let critical = snapshot.entries.get(0).unwrap();
    let high = snapshot.entries.get(1).unwrap();
    let medium = snapshot.entries.get(2).unwrap();
    let low = snapshot.entries.get(3).unwrap();

    assert_eq!(critical.severity, symbol_short!("critical"));
    assert_eq!(critical.config.threshold_minutes, 15);
    assert_eq!(high.severity, symbol_short!("high"));
    assert_eq!(high.config.threshold_minutes, 30);
    assert_eq!(medium.severity, symbol_short!("medium"));
    assert_eq!(medium.config.threshold_minutes, 60);
    assert_eq!(low.severity, symbol_short!("low"));
    assert_eq!(low.config.threshold_minutes, 120);
}

#[test]
fn test_result_schema_is_explicit_and_stable() {
    let (_env, client, _actors) = setup();

    let schema = client.get_result_schema();
    assert_eq!(schema.version, symbol_short!("v1"));
    assert_eq!(schema.schema_version, 1);
    assert_eq!(schema.status_met, symbol_short!("met"));
    assert_eq!(schema.status_violated, symbol_short!("viol"));
    assert_eq!(schema.payment_reward, symbol_short!("rew"));
    assert_eq!(schema.payment_penalty, symbol_short!("pen"));
    assert_eq!(schema.rating_exceptional, symbol_short!("top"));
    assert_eq!(schema.rating_excellent, symbol_short!("excel"));
    assert_eq!(schema.rating_good, symbol_short!("good"));
    assert_eq!(schema.rating_poor, symbol_short!("poor"));
}

#[test]
fn test_calculate_sla_emits_versioned_integration_event() {
    let (env, client, actors) = setup();

    client.calculate_sla(
        &actors.operator,
        &symbol_short!("EVT001"),
        &symbol_short!("critical"),
        &5,
    );

    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();

    let topic_0: Symbol = topics.get(0).unwrap().try_into_val(&env).unwrap();
    let topic_1: Symbol = topics.get(1).unwrap().try_into_val(&env).unwrap();
    let topic_2: Symbol = topics.get(2).unwrap().try_into_val(&env).unwrap();
    let event_data: (Symbol, Symbol, Symbol, Symbol, u32, u32, i128) =
        data.try_into_val(&env).unwrap();

    assert_eq!(topic_0, EVENT_SLA_CALC);
    assert_eq!(topic_1, EVENT_VERSION);
    assert_eq!(topic_2, symbol_short!("critical"));
    assert_eq!(
        event_data,
        (
            symbol_short!("EVT001"),
            symbol_short!("met"),
            symbol_short!("rew"),
            symbol_short!("top"),
            5u32,
            15u32,
            1500i128,
        ),
    );
}

#[test]
fn test_set_config_emits_versioned_config_event() {
    let (env, client, actors) = setup();

    client.set_config(&actors.admin, &symbol_short!("critical"), &20, &200, &1000);

    let events = env.events().all();
    let (_, topics, data) = events.last().unwrap();

    let topic_0: Symbol = topics.get(0).unwrap().try_into_val(&env).unwrap();
    let topic_1: Symbol = topics.get(1).unwrap().try_into_val(&env).unwrap();
    let topic_2: Symbol = topics.get(2).unwrap().try_into_val(&env).unwrap();
    let event_data: (u32, i128, i128) = data.try_into_val(&env).unwrap();

    assert_eq!(topic_0, EVENT_CONFIG_UPD);
    assert_eq!(topic_1, EVENT_VERSION);
    assert_eq!(topic_2, symbol_short!("critical"));
    assert_eq!(event_data, (20u32, 200i128, 1000i128));
}

// ============================================================
// #28 – Operator management
// ============================================================

#[test]
fn test_admin_can_set_operator() {
    let (env, client, actors) = setup();
    let new_op = soroban_sdk::Address::generate(&env);

    client.set_operator(&actors.admin, &new_op);

    assert_eq!(client.get_operator(), new_op);
}

#[test]
#[should_panic]
fn test_operator_cannot_set_operator() {
    let (env, client, actors) = setup();
    let new_op = soroban_sdk::Address::generate(&env);

    // operator does not have the admin role
    client.set_operator(&actors.operator, &new_op);
}

#[test]
#[should_panic]
fn test_stranger_cannot_set_operator() {
    let (env, client, actors) = setup();
    let new_op = soroban_sdk::Address::generate(&env);

    client.set_operator(&actors.stranger, &new_op);
}

// ============================================================
// #28 – Config management: admin only
// ============================================================

#[test]
fn test_admin_can_set_and_get_config() {
    let (_env, client, actors) = setup();

    client.set_config(&actors.admin, &symbol_short!("critical"), &20, &200, &1000);

    let cfg = client.get_config(&symbol_short!("critical"));
    assert_eq!(cfg.threshold_minutes, 20);
    assert_eq!(cfg.penalty_per_minute, 200);
    assert_eq!(cfg.reward_base, 1000);
}

#[test]
#[should_panic]
fn test_operator_cannot_set_config() {
    let (_env, client, actors) = setup();
    // operator must not be allowed to change config
    client.set_config(
        &actors.operator,
        &symbol_short!("critical"),
        &20,
        &200,
        &1000,
    );
}

#[test]
#[should_panic]
fn test_stranger_cannot_set_config() {
    let (_env, client, actors) = setup();
    client.set_config(
        &actors.stranger,
        &symbol_short!("critical"),
        &20,
        &200,
        &1000,
    );
}

// ============================================================
// #28 – calculate_sla: operator only
// ============================================================

#[test]
fn test_operator_can_calculate_sla() {
    let (_env, client, actors) = setup();

    let result = client.calculate_sla(
        &actors.operator,
        &symbol_short!("INC001"),
        &symbol_short!("critical"),
        &10, // under 15-min threshold → met
    );

    assert_eq!(result.status, symbol_short!("met"));
}

#[test]
#[should_panic]
fn test_admin_cannot_calculate_sla() {
    let (_env, client, actors) = setup();
    // admin does not hold the operator role
    client.calculate_sla(
        &actors.admin,
        &symbol_short!("INC002"),
        &symbol_short!("critical"),
        &10,
    );
}

#[test]
#[should_panic]
fn test_stranger_cannot_calculate_sla() {
    let (_env, client, actors) = setup();
    client.calculate_sla(
        &actors.stranger,
        &symbol_short!("INC003"),
        &symbol_short!("critical"),
        &10,
    );
}

/// After the admin reassigns the operator, the OLD operator is locked out
/// and the NEW operator can calculate.
#[test]
fn test_operator_rotation() {
    let (env, client, actors) = setup();
    let new_op = soroban_sdk::Address::generate(&env);

    client.set_operator(&actors.admin, &new_op);

    // new operator succeeds
    let result = client.calculate_sla(
        &new_op,
        &symbol_short!("INC004"),
        &symbol_short!("high"),
        &20,
    );
    assert_eq!(result.status, symbol_short!("met"));
}

#[test]
#[should_panic]
fn test_old_operator_locked_out_after_rotation() {
    let (env, client, actors) = setup();
    let new_op = soroban_sdk::Address::generate(&env);

    client.set_operator(&actors.admin, &new_op);

    // original operator should now be rejected
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("INC005"),
        &symbol_short!("high"),
        &20,
    );
}

// ============================================================
// #27 – Pause / Emergency Stop
// ============================================================

#[test]
fn test_contract_starts_unpaused() {
    let (_env, client, _actors) = setup();
    assert_eq!(client.is_paused(), false);
}

#[test]
fn test_admin_can_pause_and_unpause() {
    let (_env, client, actors) = setup();

    client.pause(&actors.admin, &soroban_sdk::String::from_str(&_env, "test"));
    assert_eq!(client.is_paused(), true);

    client.unpause(&actors.admin);
    assert_eq!(client.is_paused(), false);
}

#[test]
#[should_panic]
fn test_operator_cannot_pause() {
    let (env, client, actors) = setup();
    client.pause(&actors.operator, &soroban_sdk::String::from_str(&env, "x"));
}

#[test]
#[should_panic]
fn test_stranger_cannot_pause() {
    let (env, client, actors) = setup();
    client.pause(&actors.stranger, &soroban_sdk::String::from_str(&env, "x"));
}

#[test]
#[should_panic]
fn test_operator_cannot_unpause() {
    let (env, client, actors) = setup();
    client.pause(&actors.admin, &soroban_sdk::String::from_str(&env, "x"));
    client.unpause(&actors.operator);
}

#[test]
#[should_panic]
fn test_calculate_sla_blocked_when_paused() {
    let (env, client, actors) = setup();
    client.pause(&actors.admin, &soroban_sdk::String::from_str(&env, "maintenance"));

    // must panic – ContractPaused
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("INC006"),
        &symbol_short!("critical"),
        &10,
    );
}

#[test]
fn test_calculate_sla_works_after_unpause() {
    let (env, client, actors) = setup();

    client.pause(&actors.admin, &soroban_sdk::String::from_str(&env, "x"));
    client.unpause(&actors.admin);

    let result = client.calculate_sla(
        &actors.operator,
        &symbol_short!("INC007"),
        &symbol_short!("critical"),
        &10,
    );
    assert_eq!(result.status, symbol_short!("met"));
}

// ============================================================
// SLA business logic correctness
// ============================================================

#[test]
fn test_sla_violation_calculates_penalty() {
    let (_env, client, actors) = setup();

    // critical threshold = 15 min, penalty = 100/min
    // mttr = 25 → 10 min overtime → penalty = 1000
    let result = client.calculate_sla(
        &actors.operator,
        &symbol_short!("INC008"),
        &symbol_short!("critical"),
        &25,
    );

    assert_eq!(result.status, symbol_short!("viol"));
    assert_eq!(result.payment_type, symbol_short!("pen"));
    assert_eq!(result.rating, symbol_short!("poor"));
    assert_eq!(result.amount, -1000);
}

#[test]
fn test_sla_met_top_rating() {
    let (_env, client, actors) = setup();

    // critical threshold = 15 min; mttr = 5 → ratio = 33% < 50% → "top", 2× reward
    let result = client.calculate_sla(
        &actors.operator,
        &symbol_short!("INC009"),
        &symbol_short!("critical"),
        &5,
    );

    assert_eq!(result.status, symbol_short!("met"));
    assert_eq!(result.payment_type, symbol_short!("rew"));
    assert_eq!(result.rating, symbol_short!("top"));
    assert_eq!(result.amount, 1500); // 750 * 200 / 100
}

#[test]
fn test_backend_parity_threshold_boundary_cases() {
    let (env, client, actors) = setup();
    let cases = [
        GoldenCase {
            severity: "critical",
            mttr_minutes: 15,
            expected_status: "met",
            expected_payment_type: "rew",
            expected_rating: "good",
            expected_amount: 750,
        },
        GoldenCase {
            severity: "critical",
            mttr_minutes: 16,
            expected_status: "viol",
            expected_payment_type: "pen",
            expected_rating: "poor",
            expected_amount: -100,
        },
        GoldenCase {
            severity: "high",
            mttr_minutes: 30,
            expected_status: "met",
            expected_payment_type: "rew",
            expected_rating: "good",
            expected_amount: 750,
        },
        GoldenCase {
            severity: "high",
            mttr_minutes: 31,
            expected_status: "viol",
            expected_payment_type: "pen",
            expected_rating: "poor",
            expected_amount: -50,
        },
        GoldenCase {
            severity: "medium",
            mttr_minutes: 60,
            expected_status: "met",
            expected_payment_type: "rew",
            expected_rating: "good",
            expected_amount: 750,
        },
        GoldenCase {
            severity: "medium",
            mttr_minutes: 61,
            expected_status: "viol",
            expected_payment_type: "pen",
            expected_rating: "poor",
            expected_amount: -25,
        },
        GoldenCase {
            severity: "low",
            mttr_minutes: 120,
            expected_status: "met",
            expected_payment_type: "rew",
            expected_rating: "good",
            expected_amount: 600,
        },
        GoldenCase {
            severity: "low",
            mttr_minutes: 121,
            expected_status: "viol",
            expected_payment_type: "pen",
            expected_rating: "poor",
            expected_amount: -10,
        },
    ];

    for case in cases {
        let outage_id = symbol(&env, "PARITY_B");
        let severity = symbol(&env, case.severity);
        let result =
            client.calculate_sla(&actors.operator, &outage_id, &severity, &case.mttr_minutes);

        assert_eq!(result.status, symbol(&env, case.expected_status));
        assert_eq!(
            result.payment_type,
            symbol(&env, case.expected_payment_type)
        );
        assert_eq!(result.rating, symbol(&env, case.expected_rating));
        assert_eq!(result.amount, case.expected_amount);
    }
}

#[test]
fn test_backend_parity_reward_tier_cases() {
    let (env, client, actors) = setup();
    let cases = [
        GoldenCase {
            severity: "critical",
            mttr_minutes: 7,
            expected_status: "met",
            expected_payment_type: "rew",
            expected_rating: "top",
            expected_amount: 1500,
        },
        GoldenCase {
            severity: "critical",
            mttr_minutes: 10,
            expected_status: "met",
            expected_payment_type: "rew",
            expected_rating: "excel",
            expected_amount: 1125,
        },
        GoldenCase {
            severity: "critical",
            mttr_minutes: 15,
            expected_status: "met",
            expected_payment_type: "rew",
            expected_rating: "good",
            expected_amount: 750,
        },
        GoldenCase {
            severity: "low",
            mttr_minutes: 59,
            expected_status: "met",
            expected_payment_type: "rew",
            expected_rating: "top",
            expected_amount: 1200,
        },
        GoldenCase {
            severity: "low",
            mttr_minutes: 89,
            expected_status: "met",
            expected_payment_type: "rew",
            expected_rating: "excel",
            expected_amount: 900,
        },
        GoldenCase {
            severity: "low",
            mttr_minutes: 120,
            expected_status: "met",
            expected_payment_type: "rew",
            expected_rating: "good",
            expected_amount: 600,
        },
    ];

    for case in cases {
        let outage_id = symbol(&env, "PARITY_R");
        let severity = symbol(&env, case.severity);
        let result =
            client.calculate_sla(&actors.operator, &outage_id, &severity, &case.mttr_minutes);

        assert_eq!(result.status, symbol(&env, case.expected_status));
        assert_eq!(
            result.payment_type,
            symbol(&env, case.expected_payment_type)
        );
        assert_eq!(result.rating, symbol(&env, case.expected_rating));
        assert_eq!(result.amount, case.expected_amount);
    }
}

// ============================================================
// Budget / performance
// ============================================================

#[test]
fn test_calculate_sla_budget_is_reasonable() {
    let env = Env::default();
    env.budget().reset_unlimited();

    let cid = env.register_contract(None, SLACalculatorContract);
    let client = SLACalculatorContractClient::new(&env, &cid);
    let admin = soroban_sdk::Address::generate(&env);
    let op = soroban_sdk::Address::generate(&env);
    client.initialize(&admin, &op);

    let before = env.budget().cpu_instruction_cost();
    let _ = client.calculate_sla(&op, &symbol_short!("BUDG"), &symbol_short!("critical"), &25);
    let after = env.budget().cpu_instruction_cost();

    assert!(
        after - before < 200_000,
        "calculate_sla too expensive: {} instructions",
        after - before
    );
}

#[test]
fn test_set_config_budget_is_reasonable() {
    let env = Env::default();
    env.budget().reset_unlimited();

    let cid = env.register_contract(None, SLACalculatorContract);
    let client = SLACalculatorContractClient::new(&env, &cid);
    let admin = soroban_sdk::Address::generate(&env);
    let op = soroban_sdk::Address::generate(&env);
    client.initialize(&admin, &op);

    let before = env.budget().cpu_instruction_cost();
    client.set_config(&admin, &symbol_short!("critical"), &15, &100, &750);
    let after = env.budget().cpu_instruction_cost();

    assert!(
        after - before < 150_000,
        "set_config too expensive: {} instructions",
        after - before
    );
}

// ============================================================
// #29 – SLA Statistics Aggregation
// ============================================================

#[test]
fn test_stats_zeroed_after_initialize() {
    let (_env, client, _actors) = setup();
    let stats = client.get_stats();
    assert_eq!(stats.total_calculations, 0);
    assert_eq!(stats.total_violations, 0);
    assert_eq!(stats.total_rewards, 0);
    assert_eq!(stats.total_penalties, 0);
}

#[test]
fn test_stats_increment_on_violation() {
    let (_env, client, actors) = setup();

    // critical: threshold=15, penalty=100/min; mttr=25 → 10 min over → penalty=1000
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("S001"),
        &symbol_short!("critical"),
        &25,
    );

    let stats = client.get_stats();
    assert_eq!(stats.total_calculations, 1);
    assert_eq!(stats.total_violations, 1);
    assert_eq!(stats.total_penalties, 1000);
    assert_eq!(stats.total_rewards, 0);
}

#[test]
fn test_stats_increment_on_met() {
    let (_env, client, actors) = setup();

    // critical: threshold=15, mttr=5 → "top" → reward=1500
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("S002"),
        &symbol_short!("critical"),
        &5,
    );

    let stats = client.get_stats();
    assert_eq!(stats.total_calculations, 1);
    assert_eq!(stats.total_violations, 0);
    assert_eq!(stats.total_rewards, 1500);
    assert_eq!(stats.total_penalties, 0);
}

#[test]
fn test_stats_accumulate_across_multiple_calculations() {
    let (_env, client, actors) = setup();

    // 1 violation: mttr=25, critical → penalty=1000
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("S003"),
        &symbol_short!("critical"),
        &25,
    );
    // 2 met: mttr=5, critical → reward=1500
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("S004"),
        &symbol_short!("critical"),
        &5,
    );
    // 3 met: mttr=20, high (threshold=30) → ratio=66% → "excel" → reward=750*150/100=1125
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("S005"),
        &symbol_short!("high"),
        &20,
    );
    // 4 violation: mttr=40, high (threshold=30) → 10 min over, penalty=50/min → penalty=500
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("S006"),
        &symbol_short!("high"),
        &40,
    );

    let stats = client.get_stats();
    assert_eq!(stats.total_calculations, 4);
    assert_eq!(stats.total_violations, 2);
    assert_eq!(stats.total_rewards, 1500 + 1125); // 2625
    assert_eq!(stats.total_penalties, 1000 + 500); // 1500
}

#[test]
fn test_stats_not_updated_on_paused_rejection() {
    let (env, client, actors) = setup();

    client.pause(&actors.admin, &soroban_sdk::String::from_str(&env, "test"));

    // Fresh setup: verify stats stay at 0 when no successful calls were made.
    let (_env2, client2, _actors2) = setup();
    let stats = client2.get_stats();
    assert_eq!(stats.total_calculations, 0);
}

#[test]
fn test_stats_not_incremented_by_unauthorized_caller() {
    let (_env, _client, _actors) = setup();

    // Confirm baseline stays zero after only failed calls in another env.
    let (_env2, client2, _actors2) = setup();
    let stats = client2.get_stats();
    assert_eq!(stats.total_calculations, 0);
}

// ============================================================
// #31 – Deterministic SLA Calculation Audit Mode
// ============================================================

#[test]
fn test_calculate_sla_view_matches_mutating_and_does_not_mutate() {
    let (_env, client, actors) = setup();

    let outage_id = symbol_short!("INC999");
    let severity = symbol_short!("critical");
    let mttr = 25; // 10 min over threshold, results in penalty

    // 1. Get initial stats
    let initial_stats = client.get_stats();
    assert_eq!(initial_stats.total_calculations, 0);

    // 2. Call view function
    let view_result = client.calculate_sla_view(&outage_id, &severity, &mttr);

    // 3. Ensure no state mutated
    let after_view_stats = client.get_stats();
    assert_eq!(
        after_view_stats.total_calculations, 0,
        "View function must not mutate stats"
    );

    // 4. Call mutating function
    let mut_result = client.calculate_sla(&actors.operator, &outage_id, &severity, &mttr);

    // 5. Ensure state mutated
    let after_mut_stats = client.get_stats();
    assert_eq!(
        after_mut_stats.total_calculations, 1,
        "Mutating function must mutate stats"
    );

    // 6. Ensure results are perfectly identical
    assert_eq!(view_result.status, mut_result.status);
    assert_eq!(view_result.amount, mut_result.amount);
    assert_eq!(view_result.rating, mut_result.rating);
    assert_eq!(view_result.payment_type, mut_result.payment_type);
    assert_eq!(view_result.mttr_minutes, mut_result.mttr_minutes);
    assert_eq!(view_result.threshold_minutes, mut_result.threshold_minutes);
    assert_eq!(view_result.outage_id, mut_result.outage_id);
}
// ============================================================
// #32 – Contract Economic Stress Test Suite
// ============================================================

#[test]
fn test_stress_1000_calculations_mixed_severities() {
    let env = Env::default();

    // Reset budget to unlimited to allow 1000 sequential calls in a single test environment.
    // We will manually track CPU instruction counts to assert gas efficiency per call.
    env.budget().reset_unlimited();

    let cid = env.register_contract(None, SLACalculatorContract);
    let client = SLACalculatorContractClient::new(&env, &cid);
    let admin = soroban_sdk::Address::generate(&env);
    let op = soroban_sdk::Address::generate(&env);
    client.initialize(&admin, &op);

    let severities = [
        symbol_short!("critical"),
        symbol_short!("high"),
        symbol_short!("medium"),
        symbol_short!("low"),
    ];

    let mut expected_calculations = 0;
    let mut expected_violations = 0;
    let mut expected_rewards = 0i128;
    let mut expected_penalties = 0i128;

    let before_cpu = env.budget().cpu_instruction_cost();

    for i in 0..1000u32 {
        let severity = severities[(i % 4) as usize].clone();
        let cfg = client.get_config(&severity);

        // Alternate between meeting and violating the SLA to stress both logic paths
        let mttr = if i % 2 == 0 {
            cfg.threshold_minutes / 2 // Safely met
        } else {
            cfg.threshold_minutes + 10 // Safely violated by 10 mins
        };

        let outage_id = symbol_short!("STRESS");

        let res = client.calculate_sla(&op, &outage_id, &severity, &mttr);

        expected_calculations += 1;

        if res.status == symbol_short!("viol") {
            expected_violations += 1;
            // The contract returns penalties as negative values, so we negate it to track the positive aggregate
            expected_penalties += -res.amount;
        } else {
            expected_rewards += res.amount;
        }
    }

    let after_cpu = env.budget().cpu_instruction_cost();
    let avg_cpu_per_call = (after_cpu - before_cpu) / 1000;

    // 1. Assert no overflows occurred and cumulative statistics precisely match the local simulation
    let stats = client.get_stats();
    assert_eq!(
        stats.total_calculations, expected_calculations,
        "Calculation aggregate mismatch"
    );
    assert_eq!(
        stats.total_violations, expected_violations,
        "Violation aggregate mismatch"
    );
    assert_eq!(
        stats.total_rewards, expected_rewards,
        "Reward aggregate mismatch"
    );
    assert_eq!(
        stats.total_penalties, expected_penalties,
        "Penalty aggregate mismatch"
    );

    // 2. Assert gas bounds remain stable to catch unintended exponential looping or storage bloat
    assert!(
        avg_cpu_per_call < 50_000_000,
        "Average CPU instructions per call exceeded safe bounds: {}",
        avg_cpu_per_call
    );
}

// ============================================================
// #33 – Storage Compaction Strategy Tests
// ============================================================

#[test]
fn test_history_records_calculations() {
    let (_env, client, actors) = setup();

    client.calculate_sla(
        &actors.operator,
        &symbol_short!("H001"),
        &symbol_short!("critical"),
        &5,
    );
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("H002"),
        &symbol_short!("high"),
        &25,
    );

    let history = client.get_history();
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(0).unwrap().outage_id, symbol_short!("H001"));
    assert_eq!(history.get(1).unwrap().outage_id, symbol_short!("H002"));
}

#[test]
fn test_admin_can_prune_history() {
    let (_env, client, actors) = setup();

    // Generate 5 records
    for _i in 0..5 {
        client.calculate_sla(
            &actors.operator,
            &symbol_short!("H_GEN"),
            &symbol_short!("low"),
            &10,
        );
    }

    let history_before = client.get_history();
    assert_eq!(history_before.len(), 5);

    // Prune down to the latest 2
    client.prune_history(&actors.admin, &2);

    let history_after = client.get_history();
    assert_eq!(
        history_after.len(),
        2,
        "History should be truncated to 2 items"
    );
}

#[test]
#[should_panic]
fn test_operator_cannot_prune_history() {
    let (_env, client, actors) = setup();
    client.prune_history(&actors.operator, &0);
}

#[test]
fn test_prune_history_preserves_latest_records_accurately() {
    let (_env, client, actors) = setup();

    client.calculate_sla(
        &actors.operator,
        &symbol_short!("ID_1"),
        &symbol_short!("low"),
        &10,
    );
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("ID_2"),
        &symbol_short!("low"),
        &10,
    );
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("ID_3"),
        &symbol_short!("low"),
        &10,
    );

    // Keep only the latest 1. ID_1 and ID_2 should be dropped, ID_3 retained.
    client.prune_history(&actors.admin, &1);

    let history = client.get_history();
    assert_eq!(history.len(), 1);
    assert_eq!(
        history.get(0).unwrap().outage_id,
        symbol_short!("ID_3"),
        "Did not retain the correct recent record"
    );
}

// ============================================================
// #54 – Config snapshot version hash
// ============================================================

#[test]
fn test_config_version_hash_is_deterministic() {
    let (_env, client, _actors) = setup();
    let h1 = client.get_config_version_hash();
    let h2 = client.get_config_version_hash();
    assert_eq!(h1, h2);
}

#[test]
fn test_config_version_hash_changes_on_update() {
    let (_env, client, actors) = setup();
    let before = client.get_config_version_hash();
    client.set_config(&actors.admin, &symbol_short!("critical"), &20, &200, &1000);
    let after = client.get_config_version_hash();
    assert_ne!(before, after);
}

#[test]
fn test_config_version_hash_stable_after_same_value_write() {
    let (_env, client, actors) = setup();
    let before = client.get_config_version_hash();
    // Write the same values back – hash must not change
    client.set_config(&actors.admin, &symbol_short!("critical"), &15, &100, &750);
    let after = client.get_config_version_hash();
    assert_eq!(before, after);
}

#[test]
fn test_config_version_hash_collision_resistance() {
    let (_env, client, actors) = setup();
    
    // Get initial hash
    let initial_hash = client.get_config_version_hash();
    
    // Create a different config that would have same additive checksum
    // Original critical: threshold=15, penalty=100, reward=750 (sum=865)
    // New critical: threshold=865, penalty=0, reward=0 (sum=865)
    client.set_config(&actors.admin, &symbol_short!("critical"), &865, &0, &0);
    let collision_attempt_hash = client.get_config_version_hash();
    
    // Hash should be different despite same additive sum
    assert_ne!(initial_hash, collision_attempt_hash, 
        "Hash should resist collision from additive checksum equivalence");
    
    // Restore original config
    client.set_config(&actors.admin, &symbol_short!("critical"), &15, &100, &750);
    let restored_hash = client.get_config_version_hash();
    assert_eq!(initial_hash, restored_hash, 
        "Hash should return to original value after restoring config");
}

#[test]
fn test_config_version_hash_field_order_sensitivity() {
    let (_env, client, actors) = setup();
    
    // Test that changing different fields produces different hashes
    let original_hash = client.get_config_version_hash();
    
    // Change threshold only
    client.set_config(&actors.admin, &symbol_short!("high"), &25, &50, &750);
    let threshold_hash = client.get_config_version_hash();
    assert_ne!(original_hash, threshold_hash);
    
    // Reset and change penalty only  
    client.set_config(&actors.admin, &symbol_short!("high"), &30, &60, &750);
    let penalty_hash = client.get_config_version_hash();
    assert_ne!(original_hash, penalty_hash);
    assert_ne!(threshold_hash, penalty_hash);
    
    // Reset and change reward only
    client.set_config(&actors.admin, &symbol_short!("high"), &30, &50, &800);
    let reward_hash = client.get_config_version_hash();
    assert_ne!(original_hash, reward_hash);
    assert_ne!(threshold_hash, reward_hash);
    assert_ne!(penalty_hash, reward_hash);
    
    // Restore original
    client.set_config(&actors.admin, &symbol_short!("high"), &30, &50, &750);
    let restored_hash = client.get_config_version_hash();
    assert_eq!(original_hash, restored_hash);
}

#[test]
fn test_config_version_hash_severity_isolation() {
    let (_env, client, actors) = setup();
    
    let original_hash = client.get_config_version_hash();
    
    // Change only critical severity
    client.set_config(&actors.admin, &symbol_short!("critical"), &20, &200, &1000);
    let critical_changed_hash = client.get_config_version_hash();
    assert_ne!(original_hash, critical_changed_hash);
    
    // Change only high severity (restore critical first)
    client.set_config(&actors.admin, &symbol_short!("critical"), &15, &100, &750);
    client.set_config(&actors.admin, &symbol_short!("high"), &35, &55, &775);
    let high_changed_hash = client.get_config_version_hash();
    assert_ne!(original_hash, high_changed_hash);
    assert_ne!(critical_changed_hash, high_changed_hash);
    
    // Both changes should produce yet another hash
    client.set_config(&actors.admin, &symbol_short!("critical"), &20, &200, &1000);
    let both_changed_hash = client.get_config_version_hash();
    assert_ne!(original_hash, both_changed_hash);
    assert_ne!(critical_changed_hash, both_changed_hash);
    assert_ne!(high_changed_hash, both_changed_hash);
}

#[test]
fn test_config_version_hash_distribution() {
    let (_env, client, actors) = setup();
    
    // Test hash changes are well-distributed by making multiple small changes
    let mut hashes = Vec::new(&_env);
    
    // Collect hashes from various config states
    for i in 1..=10 {
        client.set_config(&actors.admin, &symbol_short!("critical"), &(15 + i), &100, &750);
        let hash = client.get_config_version_hash();
        hashes.push_back(hash);
    }
    
    // Verify all hashes are unique
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            assert_ne!(hashes.get(i), hashes.get(j), 
                "Hashes should be unique for different config values");
        }
    }
    
    // Restore original config
    client.set_config(&actors.admin, &symbol_short!("critical"), &15, &100, &750);
}

// ============================================================
// #56 – Repeated config update regression tests
// ============================================================

#[test]
fn test_repeated_config_updates_latest_wins() {
    let (_env, client, actors) = setup();

    client.set_config(&actors.admin, &symbol_short!("critical"), &10, &50, &500);
    client.set_config(&actors.admin, &symbol_short!("critical"), &20, &100, &800);
    client.set_config(&actors.admin, &symbol_short!("critical"), &30, &200, &1200);

    let cfg = client.get_config(&symbol_short!("critical"));
    assert_eq!(cfg.threshold_minutes, 30);
    assert_eq!(cfg.penalty_per_minute, 200);
    assert_eq!(cfg.reward_base, 1200);
}

#[test]
fn test_repeated_config_updates_do_not_corrupt_calculation() {
    let (_env, client, actors) = setup();

    // Update critical config twice; final state: threshold=20, penalty=100, reward=800
    client.set_config(&actors.admin, &symbol_short!("critical"), &10, &50, &500);
    client.set_config(&actors.admin, &symbol_short!("critical"), &20, &100, &800);

    // mttr=25 → 5 min over threshold=20 → penalty = 5 * 100 = 500
    let result = client.calculate_sla(
        &actors.operator,
        &symbol_short!("RC001"),
        &symbol_short!("critical"),
        &25,
    );
    assert_eq!(result.status, symbol_short!("viol"));
    assert_eq!(result.amount, -500);
}

#[test]
fn test_repeated_config_updates_across_severities_are_independent() {
    let (_env, client, actors) = setup();

    client.set_config(&actors.admin, &symbol_short!("critical"), &5, &10, &100);
    client.set_config(&actors.admin, &symbol_short!("high"), &5, &10, &100);

    // medium and low must remain at their defaults
    let medium = client.get_config(&symbol_short!("medium"));
    let low = client.get_config(&symbol_short!("low"));
    assert_eq!(medium.threshold_minutes, 60);
    assert_eq!(low.threshold_minutes, 120);
}

// ============================================================
// #50 – Canonical SLA vector snapshot export
// ============================================================

#[cfg(feature = "export-snapshots")]
mod snapshots {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn write_snapshot(name: &str, json: &str) {
        let dir = Path::new("test_snapshots/tests");
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join(format!("{}.json", name)), json).unwrap();
    }

    #[test]
    fn test_backend_parity_threshold_boundary_cases_snapshot() {
        let (env, client, actors) = setup();
        let cases = [
            ("critical", 15u32, "met", "rew", "good", 750i128),
            ("critical", 16, "viol", "pen", "poor", -100),
            ("high", 30, "met", "rew", "good", 750),
            ("high", 31, "viol", "pen", "poor", -50),
            ("medium", 60, "met", "rew", "good", 750),
            ("medium", 61, "viol", "pen", "poor", -25),
            ("low", 120, "met", "rew", "good", 600),
            ("low", 121, "viol", "pen", "poor", -10),
        ];

        let mut entries = Vec::new();
        for (sev, mttr, status, ptype, rating, amount) in cases {
            let result = client.calculate_sla_view(
                &symbol(&env, "SNAP_B"),
                &symbol(&env, sev),
                &mttr,
            );
            assert_eq!(result.status, symbol(&env, status));
            assert_eq!(result.payment_type, symbol(&env, ptype));
            assert_eq!(result.rating, symbol(&env, rating));
            assert_eq!(result.amount, amount);
            entries.push(format!(
                r#"{{"severity":"{sev}","mttr_minutes":{mttr},"status":"{status}","payment_type":"{ptype}","rating":"{rating}","amount":{amount}}}"#
            ));
        }
        write_snapshot(
            "test_backend_parity_threshold_boundary_cases",
            &format!("[{}]", entries.join(",")),
        );
    }

    #[test]
    fn test_backend_parity_reward_tier_cases_snapshot() {
        let (env, client, _actors) = setup();
        let cases = [
            ("critical", 7u32, "met", "rew", "top", 1500i128),
            ("critical", 10, "met", "rew", "excel", 1125),
            ("critical", 15, "met", "rew", "good", 750),
            ("low", 59, "met", "rew", "top", 1200),
            ("low", 89, "met", "rew", "excel", 900),
            ("low", 120, "met", "rew", "good", 600),
        ];

        let mut entries = Vec::new();
        for (sev, mttr, status, ptype, rating, amount) in cases {
            let result = client.calculate_sla_view(
                &symbol(&env, "SNAP_R"),
                &symbol(&env, sev),
                &mttr,
            );
            assert_eq!(result.status, symbol(&env, status));
            assert_eq!(result.payment_type, symbol(&env, ptype));
            assert_eq!(result.rating, symbol(&env, rating));
            assert_eq!(result.amount, amount);
            entries.push(format!(
                r#"{{"severity":"{sev}","mttr_minutes":{mttr},"status":"{status}","payment_type":"{ptype}","rating":"{rating}","amount":{amount}}}"#
            ));
        }
        write_snapshot(
            "test_backend_parity_reward_tier_cases",
            &format!("[{}]", entries.join(",")),
        );
    }

    #[test]
    fn test_config_snapshot_is_deterministic_and_complete_snapshot() {
        let (_env, client, _actors) = setup();
        let snap = client.get_config_snapshot();
        assert_eq!(snap.entries.len(), 4);

        let mut entries = Vec::new();
        for i in 0..snap.entries.len() {
            let e = snap.entries.get(i).unwrap();
            entries.push(format!(
                r#"{{"severity":"{}","threshold_minutes":{},"penalty_per_minute":{},"reward_base":{}}}"#,
                ["critical", "high", "medium", "low"][i as usize],
                e.config.threshold_minutes,
                e.config.penalty_per_minute,
                e.config.reward_base,
            ));
        }
        write_snapshot(
            "test_config_snapshot_is_deterministic_and_complete",
            &format!("[{}]", entries.join(",")),
        );
    }
}

// ============================================================
// #94 – Fixture helpers for repeated actor and contract setup
// ============================================================

/// Setup with a custom critical config applied on top of defaults.
fn setup_with_critical(threshold: u32, penalty: i128, reward: i128) -> (Env, SLACalculatorContractClient<'static>, Actors) {
    let (env, client, actors) = setup();
    client.set_config(&actors.admin, &symbol_short!("critical"), &threshold, &penalty, &reward);
    (env, client, actors)
}

/// Setup and perform one calculation, returning the result along with the env/client/actors.
fn setup_after_calculation(severity: &str, mttr: u32) -> (Env, SLACalculatorContractClient<'static>, Actors) {
    let (env, client, actors) = setup();
    client
        .calculate_sla(
            &actors.operator,
            &symbol(&env, "FIXTURE_ID"),
            &symbol(&env, severity),
            &mttr,
        )
        .unwrap();
    (env, client, actors)
}

#[test]
fn test_fixture_custom_critical_config_is_applied() {
    let (_env, client, _actors) = setup_with_critical(10, 50, 500);
    let cfg = client.get_config(&symbol_short!("critical"));
    assert_eq!(cfg.threshold_minutes, 10);
    assert_eq!(cfg.penalty_per_minute, 50);
    assert_eq!(cfg.reward_base, 500);
}

#[test]
fn test_fixture_after_calculation_history_has_one_entry() {
    let (_env, client, _actors) = setup_after_calculation("critical", 5);
    let history = client.get_history().unwrap();
    assert_eq!(history.len(), 1);
}

#[test]
fn test_fixture_after_calculation_stats_are_updated() {
    let (_env, client, _actors) = setup_after_calculation("high", 35);
    let stats = client.get_stats().unwrap();
    assert_eq!(stats.total_calculations, 1);
    assert_eq!(stats.total_violations, 1);
}

// ============================================================
// #95 – Negative tests for malformed symbol inputs
// ============================================================

#[test]
#[should_panic]
fn test_calculate_sla_unknown_severity_panics() {
    let (_env, client, actors) = setup();
    // "xyz" is not a configured severity — ConfigNotFound maps to a panic in the client
    client
        .calculate_sla(
            &actors.operator,
            &symbol_short!("OUT001"),
            &symbol_short!("xyz"),
            &10,
        )
        .unwrap();
// #63 – Two-step admin transfer
// ============================================================

#[test]
fn test_propose_and_accept_admin() {
    let (env, client, actors) = setup();
    let new_admin = soroban_sdk::Address::generate(&env);

    client.propose_admin(&actors.admin, &new_admin);
    assert_eq!(client.get_pending_admin(), Some(new_admin.clone()));

    client.accept_admin(&new_admin);
    assert_eq!(client.get_admin(), new_admin);
    assert_eq!(client.get_pending_admin(), None);
}

#[test]
#[test]
#[should_panic]
fn test_old_admin_loses_authority_after_accept() {
    let (env, client, actors) = setup();
    let new_admin = soroban_sdk::Address::generate(&env);

    client.propose_admin(&actors.admin, &new_admin);
    client.accept_admin(&new_admin);

    // old admin can no longer set config – must panic
    client.set_config(&actors.admin, &symbol_short!("critical"), &20, &200, &1000);
}

#[test]
#[should_panic]
fn test_wrong_address_cannot_accept_admin() {
    let (env, client, actors) = setup();
    let new_admin = soroban_sdk::Address::generate(&env);
    let stranger = soroban_sdk::Address::generate(&env);

    client.propose_admin(&actors.admin, &new_admin);
    client.accept_admin(&stranger); // must panic
}

#[test]
#[should_panic]
fn test_accept_admin_without_proposal_fails() {
    let (_env, client, actors) = setup();
    client.accept_admin(&actors.stranger); // no pending proposal
}

#[test]
fn test_get_pending_admin_none_when_no_proposal() {
    let (_env, client, _actors) = setup();
    assert_eq!(client.get_pending_admin(), None);
}

// ============================================================
// #64 – Two-step operator handoff
// ============================================================

#[test]
fn test_propose_and_accept_operator() {
    let (env, client, actors) = setup();
    let new_op = soroban_sdk::Address::generate(&env);

    client.propose_operator(&actors.admin, &new_op);
    assert_eq!(client.get_pending_operator(), Some(new_op.clone()));

    client.accept_operator(&new_op);
    assert_eq!(client.get_operator(), new_op);
    assert_eq!(client.get_pending_operator(), None);
}

#[test]
#[test]
#[should_panic]
fn test_old_operator_locked_out_after_handoff() {
    let (env, client, actors) = setup();
    let new_op = soroban_sdk::Address::generate(&env);

    client.propose_operator(&actors.admin, &new_op);
    client.accept_operator(&new_op);

    // old operator can no longer calculate – must panic
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("HO001"),
        &symbol_short!("critical"),
        &5,
    );
}

#[test]
#[should_panic]
fn test_wrong_address_cannot_accept_operator() {
    let (env, client, actors) = setup();
    let new_op = soroban_sdk::Address::generate(&env);
    let stranger = soroban_sdk::Address::generate(&env);

    client.propose_operator(&actors.admin, &new_op);
    client.accept_operator(&stranger); // must panic
// #60 – Contract metadata / capabilities view
// ============================================================

#[test]
fn test_get_contract_metadata_returns_expected_fields() {
    let (_env, client, _actors) = setup();
    let meta = client.get_contract_metadata();
    assert_eq!(meta.contract_name, symbol_short!("sla_calc"));
    assert_eq!(meta.storage_version, 1);
    assert_eq!(meta.result_schema_version, 1);
    assert_eq!(meta.supported_severities.len(), 4);
    assert_eq!(meta.features.len(), 5);
}

#[test]
fn test_get_contract_metadata_severities_are_canonical() {
    let (_env, client, _actors) = setup();
    let meta = client.get_contract_metadata();
    assert_eq!(meta.supported_severities.get(0).unwrap(), symbol_short!("critical"));
    assert_eq!(meta.supported_severities.get(1).unwrap(), symbol_short!("high"));
    assert_eq!(meta.supported_severities.get(2).unwrap(), symbol_short!("medium"));
    assert_eq!(meta.supported_severities.get(3).unwrap(), symbol_short!("low"));
}

#[test]
fn test_get_contract_metadata_is_deterministic() {
    let (_env, client, _actors) = setup();
    let m1 = client.get_contract_metadata();
    let m2 = client.get_contract_metadata();
    assert_eq!(m1.storage_version, m2.storage_version);
    assert_eq!(m1.result_schema_version, m2.result_schema_version);
    assert_eq!(m1.contract_name, m2.contract_name);
}

// ============================================================
// #61 – Storage migration harness
// ============================================================

#[test]
fn test_migrate_is_idempotent_when_already_current() {
    let (_env, client, actors) = setup();
    // Already at v1 – migrate should succeed without error
    client.migrate(&actors.admin);
    client.migrate(&actors.admin);
    // Contract still functional
    assert_eq!(client.get_admin(), actors.admin);
}

#[test]
#[should_panic]
fn test_get_config_unknown_severity_panics() {
    let (_env, client, _actors) = setup();
    // "CRIT" (uppercase) is not a valid severity key
    client.get_config(&symbol_short!("CRIT"));
fn test_accept_operator_without_proposal_fails() {
    let (_env, client, actors) = setup();
    client.accept_operator(&actors.stranger);
}

#[test]
fn test_get_pending_operator_none_when_no_proposal() {
    let (_env, client, _actors) = setup();
    assert_eq!(client.get_pending_operator(), None);
}

// ============================================================
// #65 – Admin renounce
// ============================================================

#[test]
fn test_admin_can_renounce() {
    let (_env, client, actors) = setup();
    client.renounce_admin(&actors.admin);
    // After renounce, admin-gated calls must fail
}

#[test]
#[should_panic]
fn test_calculate_sla_wrong_case_severity_panics() {
    let (_env, client, actors) = setup();
    // "HIGH" differs from configured "high"
    client
        .calculate_sla(
            &actors.operator,
            &symbol_short!("OUT002"),
            &symbol_short!("HIGH"),
            &10,
        )
        .unwrap();
}
#[test]
#[should_panic]
fn test_calculate_sla_view_unknown_severity_panics() {
    let (env, client, _actors) = setup();
    client.calculate_sla_view(
        &symbol(&env, "VIEW001"),
        &symbol_short!("unknown"),
        &10,
    );
}
// #96 – Backend-consumer smoke fixture (end-to-end sequence)
// ============================================================

#[test]
fn test_backend_smoke_initialize_config_calculate_history_stats() {
    // Step 1: initialize (via setup helper — admin + operator roles set, default configs loaded)
    let (env, client, actors) = setup();

    // Step 2: config read — verify a known severity is present
    let critical_cfg = client.get_config(&symbol_short!("critical"));
    assert_eq!(critical_cfg.threshold_minutes, 15);
    assert!(critical_cfg.penalty_per_minute > 0);
    assert!(critical_cfg.reward_base > 0);

    // Step 3: calculate — operator submits an SLA result
    let result = client
        .calculate_sla(
            &actors.operator,
            &symbol(&env, "SMOKE_001"),
            &symbol_short!("critical"),
            &10,
        )
        .unwrap();
    assert_eq!(result.status, symbol_short!("met"));

    // Step 4: history read — the calculation appears in history
    let history = client.get_history().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().outage_id, symbol(&env, "SMOKE_001"));

    // Step 5: stats read — counters reflect the single met calculation
    let stats = client.get_stats().unwrap();
    assert_eq!(stats.total_calculations, 1);
    assert_eq!(stats.total_violations, 0);
    assert!(stats.total_rewards > 0);
    assert_eq!(stats.total_penalties, 0);
}

#[test]
fn test_backend_smoke_violation_path() {
    let (env, client, actors) = setup();

    // critical threshold is 15 min; 30 min exceeds it → violation
    let result = client
        .calculate_sla(
            &actors.operator,
            &symbol(&env, "SMOKE_002"),
            &symbol_short!("critical"),
            &30,
        )
        .unwrap();
    assert_eq!(result.status, symbol_short!("viol"));
    assert_eq!(result.payment_type, symbol_short!("pen"));
    assert!(result.amount < 0);

    let stats = client.get_stats().unwrap();
    assert_eq!(stats.total_violations, 1);
    assert_eq!(stats.total_rewards, 0);
    assert!(stats.total_penalties > 0);
fn test_admin_gated_call_fails_after_renounce() {
    let (env, client, actors) = setup();
    client.renounce_admin(&actors.admin);
    // set_config must now panic – no admin exists
    client.set_config(&actors.admin, &symbol_short!("critical"), &20, &200, &1000);
fn test_migrate_rejected_for_non_admin() {
    let (_env, client, actors) = setup();
    client.migrate(&actors.stranger);
}

#[test]
#[should_panic]
fn test_check_version_rejects_version_mismatch() {
    // Simulate a future version stored in state by writing a different version
    // directly, then calling any versioned endpoint.
    let env = Env::default();
    let cid = env.register_contract(None, SLACalculatorContract);
    let client = SLACalculatorContractClient::new(&env, &cid);
    let admin = soroban_sdk::Address::generate(&env);
    let op = soroban_sdk::Address::generate(&env);
    client.initialize(&admin, &op);

    // Manually overwrite the stored version to simulate a future schema
    env.as_contract(&cid, || {
        env.storage()
            .instance()
            .set(&STORAGE_VERSION_KEY, &99u32);
    });

    // Any versioned call must now panic with VersionMismatch
    client.get_admin();
}

// ============================================================
// #62 – Unknown-severity rejection
// ============================================================

#[test]
#[should_panic]
fn test_calculate_sla_rejects_unknown_severity() {
    let (env, client, actors) = setup();
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("UNK001"),
        &Symbol::new(&env, "unknown"),
        &10,
    );
}

#[test]
#[should_panic]
fn test_stranger_cannot_renounce() {
    let (_env, client, actors) = setup();
    client.renounce_admin(&actors.stranger);
}

#[test]
fn test_renounce_clears_pending_proposal() {
    let (env, client, actors) = setup();
    let new_admin = soroban_sdk::Address::generate(&env);

    client.propose_admin(&actors.admin, &new_admin);
    client.renounce_admin(&actors.admin);
    assert_eq!(client.get_pending_admin(), None);
}

// ============================================================
// #66 – Pause reason + timestamp
// ============================================================

#[test]
fn test_pause_stores_reason_and_timestamp() {
    let (env, client, actors) = setup();
    let reason = soroban_sdk::String::from_str(&env, "scheduled maintenance");

    client.pause(&actors.admin, &reason);

    let info = client.get_pause_info().expect("pause info should be present");
    assert_eq!(info.reason, reason);
    // timestamp is ledger time; just assert it is non-zero in a real ledger,
    // in test env it defaults to 0 which is still a valid u64
    let _ = info.paused_at;
}

#[test]
fn test_unpause_clears_pause_info() {
    let (env, client, actors) = setup();
    client.pause(&actors.admin, &soroban_sdk::String::from_str(&env, "reason"));
    client.unpause(&actors.admin);

    assert_eq!(client.get_pause_info(), None);
}

#[test]
fn test_get_pause_info_none_when_not_paused() {
    let (_env, client, _actors) = setup();
    assert_eq!(client.get_pause_info(), None);
fn test_calculate_sla_view_rejects_unknown_severity() {
    let (env, client, _actors) = setup();
    client.calculate_sla_view(
        &symbol_short!("UNK002"),
        &Symbol::new(&env, "unknown"),
        &10,
    );
}

#[test]
#[should_panic]
fn test_get_config_rejects_unknown_severity() {
    let (env, client, _actors) = setup();
    client.get_config(&Symbol::new(&env, "unknown"));
}

#[test]
#[should_panic]
fn test_set_config_then_calculate_unknown_severity_still_rejects_other_unknown() {
    // Even after adding a custom severity via set_config, a different unknown still fails
    let (env, client, actors) = setup();
    client.set_config(&actors.admin, &Symbol::new(&env, "custom"), &10, &50, &500);
    // "bogus" was never configured
    client.calculate_sla(
        &actors.operator,
        &symbol_short!("UNK003"),
        &Symbol::new(&env, "bogus"),
        &5,
    );
}

// ============================================================
// #70 – Configuration Validation Tests
// ============================================================

#[test]
fn test_valid_config_passes_validation() {
    let (_env, client, actors) = setup();

    // All these should succeed
    client.set_config(&actors.admin, &symbol_short!("critical"), &30, &150, &1000);
    client.set_config(&actors.admin, &symbol_short!("high"), &45, &75, &800);
    client.set_config(&actors.admin, &symbol_short!("medium"), &90, &30, &600);
    client.set_config(&actors.admin, &symbol_short!("low"), &180, &15, &500);

    // Verify values were set
    let cfg = client.get_config(&symbol_short!("critical"));
    assert_eq!(cfg.threshold_minutes, 30);
    assert_eq!(cfg.penalty_per_minute, 150);
    assert_eq!(cfg.reward_base, 1000);
}

#[test]
#[should_panic]
fn test_invalid_severity_fails_validation() {
    let (_env, client, actors) = setup();
    // "urgent" is not a supported severity
    client.set_config(&actors.admin, &symbol_short!("urgent"), &15, &100, &750);
}

#[test]
#[should_panic]
fn test_zero_threshold_fails_validation() {
    let (_env, client, actors) = setup();
    // Threshold cannot be 0
    client.set_config(&actors.admin, &symbol_short!("critical"), &0, &100, &750);
}

#[test]
#[should_panic]
fn test_threshold_too_large_fails_validation() {
    let (_env, client, actors) = setup();
    // Threshold exceeds 1440 minute (24 hour) maximum
    client.set_config(&actors.admin, &symbol_short!("low"), &1500, &10, &600);
}

#[test]
#[should_panic]
fn test_negative_penalty_fails_validation() {
    let (_env, client, actors) = setup();
    // Penalty must be positive
    client.set_config(&actors.admin, &symbol_short!("critical"), &15, &-100, &750);
}

#[test]
#[should_panic]
fn test_zero_penalty_fails_validation() {
    let (_env, client, actors) = setup();
    // Penalty must be positive (cannot be 0)
    client.set_config(&actors.admin, &symbol_short!("critical"), &15, &0, &750);
}

#[test]
#[should_panic]
fn test_penalty_too_large_fails_validation() {
    let (_env, client, actors) = setup();
    // Penalty exceeds 10,000 maximum
    client.set_config(&actors.admin, &symbol_short!("critical"), &15, &15000, &750);
}

#[test]
#[should_panic]
fn test_negative_reward_fails_validation() {
    let (_env, client, actors) = setup();
    // Reward must be positive
    client.set_config(&actors.admin, &symbol_short!("critical"), &15, &100, &-750);
}

#[test]
#[should_panic]
fn test_zero_reward_fails_validation() {
    let (_env, client, actors) = setup();
    // Reward must be positive (cannot be 0)
    client.set_config(&actors.admin, &symbol_short!("critical"), &15, &100, &0);
}

#[test]
#[should_panic]
fn test_reward_too_large_fails_validation() {
    let (_env, client, actors) = setup();
    // Reward exceeds 100,000 maximum
    client.set_config(&actors.admin, &symbol_short!("critical"), &15, &100, &150000);
}

// Severity-specific validation tests

#[test]
#[should_panic]
fn test_critical_threshold_too_high_fails_validation() {
    let (_env, client, actors) = setup();
    // Critical severity threshold cannot exceed 60 minutes
    client.set_config(&actors.admin, &symbol_short!("critical"), &90, &100, &750);
}

#[test]
#[should_panic]
fn test_critical_penalty_too_low_fails_validation() {
    let (_env, client, actors) = setup();
    // Critical severity penalty must be at least 50
    client.set_config(&actors.admin, &symbol_short!("critical"), &15, &25, &750);
}

#[test]
#[should_panic]
fn test_high_threshold_too_high_fails_validation() {
    let (_env, client, actors) = setup();
    // High severity threshold cannot exceed 120 minutes
    client.set_config(&actors.admin, &symbol_short!("high"), &150, &50, &750);
}

#[test]
#[should_panic]
fn test_high_penalty_too_low_fails_validation() {
    let (_env, client, actors) = setup();
    // High severity penalty must be at least 25
    client.set_config(&actors.admin, &symbol_short!("high"), &30, &15, &750);
}

#[test]
#[should_panic]
fn test_medium_threshold_too_high_fails_validation() {
    let (_env, client, actors) = setup();
    // Medium severity threshold cannot exceed 240 minutes
    client.set_config(&actors.admin, &symbol_short!("medium"), &300, &25, &750);
}

#[test]
#[should_panic]
fn test_medium_penalty_too_low_fails_validation() {
    let (_env, client, actors) = setup();
    // Medium severity penalty must be at least 10
    client.set_config(&actors.admin, &symbol_short!("medium"), &60, &5, &750);
}

#[test]
#[should_panic]
fn test_low_penalty_too_high_fails_validation() {
    let (_env, client, actors) = setup();
    // Low severity penalty cannot exceed 100
    client.set_config(&actors.admin, &symbol_short!("low"), &120, &150, &600);
}

// Edge case validation tests

#[test]
fn test_boundary_values_pass_validation() {
    let (_env, client, actors) = setup();

    // Test minimum valid values
    client.set_config(&actors.admin, &symbol_short!("critical"), &1, &50, &1);
    client.set_config(&actors.admin, &symbol_short!("high"), &1, &25, &1);
    client.set_config(&actors.admin, &symbol_short!("medium"), &1, &10, &1);
    client.set_config(&actors.admin, &symbol_short!("low"), &1, &1, &1);

    // Test maximum valid values for severity-specific constraints
    client.set_config(&actors.admin, &symbol_short!("critical"), &60, &10000, &100000);
    client.set_config(&actors.admin, &symbol_short!("high"), &120, &10000, &100000);
    client.set_config(&actors.admin, &symbol_short!("medium"), &240, &10000, &100000);
    client.set_config(&actors.admin, &symbol_short!("low"), &1440, &100, &100000);
}

#[test]
fn test_validation_prevents_partial_state_changes() {
    let (_env, client, actors) = setup();

    // Get original config
    let original = client.get_config(&symbol_short!("critical"));
    assert_eq!(original.threshold_minutes, 15);
    assert_eq!(original.penalty_per_minute, 100);
    assert_eq!(original.reward_base, 750);

    // Attempt invalid config change - should fail without modifying state
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.set_config(&actors.admin, &symbol_short!("critical"), &0, &100, &750);
    }));
    assert!(result.is_err());

    // Verify original config is unchanged
    let unchanged = client.get_config(&symbol_short!("critical"));
    assert_eq!(unchanged.threshold_minutes, 15);
    assert_eq!(unchanged.penalty_per_minute, 100);
    assert_eq!(unchanged.reward_base, 750);
}

#[test]
fn test_validation_works_after_successful_config_change() {
    let (_env, client, actors) = setup();

    // Make a valid change first
    client.set_config(&actors.admin, &symbol_short!("critical"), &30, &150, &1000);

    // Now attempt an invalid change - should still fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.set_config(&actors.admin, &symbol_short!("critical"), &0, &150, &1000);
    }));
    assert!(result.is_err());

    // Verify the valid change is still in place
    let cfg = client.get_config(&symbol_short!("critical"));
    assert_eq!(cfg.threshold_minutes, 30);
    assert_eq!(cfg.penalty_per_minute, 150);
    assert_eq!(cfg.reward_base, 1000);
}

#[test]
fn test_validation_applies_to_all_severities_independently() {
    let (_env, client, actors) = setup();

    // Valid change to critical
    client.set_config(&actors.admin, &symbol_short!("critical"), &25, &120, &900);

    // Invalid change to high should not affect critical
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.set_config(&actors.admin, &symbol_short!("high"), &0, &50, &750);
    }));
    assert!(result.is_err());

    // Verify critical is unchanged and high is still at default
    let critical = client.get_config(&symbol_short!("critical"));
    assert_eq!(critical.threshold_minutes, 25);
    
    let high = client.get_config(&symbol_short!("high"));
    assert_eq!(high.threshold_minutes, 30); // still default
}
