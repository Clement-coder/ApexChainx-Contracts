# Changelog

> All interface-affecting changes to `apexchainx-contracts` are recorded here.
> This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
> and follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) conventions.

---

## [Unreleased]

### Added
- `get_config_version_hash` — deterministic hash of the current config snapshot for backend parity validation
- `get_result_schema` — explicit schema descriptor for SLA result encoding (status, payment type, rating symbols)
- `calculate_sla_view` — read-only simulation of SLA calculation without state mutation or auth requirement
- `get_config_snapshot` — ordered snapshot of all severity configs with version tag

### Changed
- `get_stats` now returns a `SLAStats` struct; callers should use field access rather than tuple destructuring
- History entries returned by `get_history` include `schema_version` for result envelope versioning

---

## [0.3.0] — Operator role and pause controls

### Added
- `set_operator` — admin-only function to update the operator address
- `pause` / `unpause` — admin-only controls; `calculate_sla` panics with `ContractPaused` when paused
- `get_operator` — read the current operator address

### Changed
- `calculate_sla` now requires the `operator` address as the first argument (breaking)
- `SLAError` extended with `ContractPaused = 6`

---

## [0.2.0] — Statistics and history

### Added
- `get_stats` — cumulative totals for calculations, violations, rewards, penalties
- `get_history` — ordered log of recent SLA calculation results
- `prune_history` — admin-only compaction to bound on-chain storage

---

## [0.1.0] — Initial contract surface

### Added
- `initialize(admin, operator)` — one-time setup; stores roles and default severity configs
- `set_config(caller, severity, threshold_minutes, penalty_per_minute, reward_base)` — admin-only config update
- `get_config(severity)` — read a single severity config
- `calculate_sla(caller, outage_id, severity, mttr_minutes)` — operator-gated SLA calculation

---

## Changelog Process

When making an interface-affecting change, follow these steps:

1. **Add an entry** under `[Unreleased]` in the appropriate section (`Added`, `Changed`, `Removed`, `Fixed`)
2. **Use exact function names** as they appear in the contract interface
3. **Mark breaking changes** explicitly with **(breaking)**
4. **On release**, rename `[Unreleased]` to the version tag and date, then open a fresh `[Unreleased]` block

### Change Categories

| Category | Usage |
|----------|-------|
| `Added` | New functions, features, or parameters |
| `Changed` | Modifications to existing behavior (non-breaking) |
| `Fixed` | Bug fixes or corrections |
| `Removed` | Deprecated or deleted functionality |
| `Security` | Vulnerability patches or security improvements |
