//! Severity level enumeration and validation utilities.
//!
//! The SLA calculator supports exactly four severity levels with a strict
//! canonical ordering: critical > high > medium > low. This ordering is
//! used for config snapshot generation, validation, and backend-facing
//! deterministic reads.
//!
//! # Severity Properties
//!
//! | Level | Priority | Default Threshold | Max Threshold |
//! |-------|----------|------------------|--------------|
//! | critical | Highest | 15 min | 60 min |
//! | high | High | 30 min | 120 min |
//! | medium | Standard | 60 min | 240 min |
//! | low | Low | 120 min | 1440 min |
//!
//! # Determinism
//!
//! The `supported_severities()` function returns severities in a fixed order
//! to guarantee deterministic iteration across all consumers.

use soroban_sdk::{symbol_short, Env, Symbol, Vec};

pub fn supported_severities(env: &Env) -> Vec<Symbol> {
    let mut out = Vec::new(env);
    out.push_back(symbol_short!("critical"));
    out.push_back(symbol_short!("high"));
    out.push_back(symbol_short!("medium"));
    out.push_back(symbol_short!("low"));
    out
}

pub fn is_known_severity(env: &Env, severity: &Symbol) -> bool {
    supported_severities(env).contains(severity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, Env};

    #[test]
    fn test_severity_list_is_deterministic() {
        let env = Env::default();
        let first  = supported_severities(&env);
        let second = supported_severities(&env);
        assert_eq!(first, second);
    }

    #[test]
    fn test_severity_list_contains_expected_values() {
        let env = Env::default();
        let sevs = supported_severities(&env);
        assert_eq!(sevs.len(), 4);
        assert!(sevs.contains(&symbol_short!("critical")));
        assert!(sevs.contains(&symbol_short!("high")));
        assert!(sevs.contains(&symbol_short!("medium")));
        assert!(sevs.contains(&symbol_short!("low")));
    }

    #[test]
    fn test_unknown_severity_is_rejected() {
        let env = Env::default();
        assert!(!is_known_severity(&env, &symbol_short!("ultra")));
    }
}
