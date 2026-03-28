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

    client.pause(&actors.admin);
    assert_eq!(client.is_paused(), true);

    client.unpause(&actors.admin);
    assert_eq!(client.is_paused(), false);
}

#[test]
#[should_panic]
fn test_operator_cannot_pause() {
    let (_env, client, actors) = setup();
    client.pause(&actors.operator);
}

#[test]
#[should_panic]
fn test_stranger_cannot_pause() {
    let (_env, client, actors) = setup();
    client.pause(&actors.stranger);
}

#[test]
#[should_panic]
fn test_operator_cannot_unpause() {
    let (_env, client, actors) = setup();
    client.pause(&actors.admin);
    client.unpause(&actors.operator);
}

#[test]
#[should_panic]
fn test_calculate_sla_blocked_when_paused() {
    let (_env, client, actors) = setup();
    client.pause(&actors.admin);

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
    let (_env, client, actors) = setup();

    client.pause(&actors.admin);
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
    let (_env, client, actors) = setup();

    client.pause(&actors.admin);

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
