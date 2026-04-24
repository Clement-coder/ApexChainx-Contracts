# ApexChainx Smart Contracts

Soroban contract repository for the ApexChainx system.

This repository is the execution-layer side of the 3-repo architecture:

- `apexchainx-fe` -> frontend
- `apexchainx-be` -> backend and integration layer
- `apexchainx-contracts` -> Soroban smart contracts

System flow:

`User -> FE -> BE -> Contracts -> BE -> FE`

Important rule:

- contracts are not called directly by the frontend
- the backend is responsible for invoking contracts and translating results back to the UI

## Overview

`apexchainx-contracts` contains the Soroban-side SLA logic for ApexChainx.

At the current checked-in state, this repository contains one active contract crate:

- `apexchainx_calculator`

This contract is responsible for deterministic SLA calculation and related contract-side state such as configuration, statistics, pause state, and calculation history.

## Current Stack

- Rust
- Soroban SDK 21
- Cargo

Main crate manifest:

- `apexchainx_calculator/Cargo.toml`

## Current Contract Surface

The active contract is in:

- `apexchainx_calculator/src/lib.rs`

The current implementation includes:

- initialization with admin and operator roles
- severity-based SLA configuration
- admin-controlled config updates
- operator-gated `calculate_sla`
- read-only `calculate_sla_view`
- backend-friendly `get_config_snapshot`
- pause and unpause controls
- cumulative SLA statistics
- history retrieval and pruning

Tests live in:

- `apexchainx_calculator/src/tests.rs`

## Project Structure

```text
apexchainx-contracts/
├── docs/
│   ├── CODEX_CONTEXT.md
│   └── PROJECT_CONTEXT.md
├── apexchainx_calculator/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── tests.rs
├── CONTRIBUTING.md
├── README.md
```

## What Is Actually In This Repo

Only the SLA calculator contract is currently checked in.

That means this repo does not currently contain:

- `payment_escrow`
- `multi_party_settlement`
- deployment scripts
- a top-level Cargo workspace

If those are planned, they are future work rather than part of the present repository state.

## Local Setup

### Prerequisites

- Rust toolchain
- Cargo
- optional: Soroban CLI for deployment workflows

### Run Tests\n\n```bash\ncd apexchainx_calculator\ncargo test\n```\n\n### Test Vector Artifacts for Backend Parity\n\nRun `cargo test` to generate/update canonical SLA test vectors as JSON snapshots:\n\n```\napexchainx_calculator/test_snapshots/tests/*.json\n```\n\n**Key Vectors**:\n- `test_backend_parity_threshold_boundary_cases.*.json`: SLA met/viol boundaries\n- `test_backend_parity_reward_tier_cases.*.json`: Reward tiers (top/excel/good)\n- `test_stress_1000_calculations_mixed_severities.*.json`: Performance aggregates\n- `test_config_snapshot_is_deterministic_and_complete.*.json`: Full config\n\n**Backend Usage**:\n1. Consume snapshots for parity tests: Input (severity/mttr) → match contract `calculate_sla_view`\n2. Use `get_config_snapshot()` + `get_result_schema()` for schema validation.\n3. Maintenance: `cargo test` after SLA changes → snapshots auto-update.\n\nVectors ensure contract/backend parity without manual duplication.\n\n### Build The Contract\n\n```bash\ncd apexchainx_calculator\ncargo build\n```\n\n### Build WASM\n\n```bash\ncd apexchainx_calculator\ncargo build --target wasm32-unknown-unknown --release\n```\n\nExpected artifact:\n\n- `apexchainx_calculator/target/wasm32-unknown-unknown/release/apexchainx_calculator.wasm`\n\n## Deploy-Oriented Workflow

The current repository does not ship deployment scripts, but the existing crate
is ready for a manual Soroban deployment flow.

### 1. Build the release WASM

```bash
cd apexchainx_calculator
cargo build --target wasm32-unknown-unknown --release
```

### 2. Deploy the contract

Example:

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/apexchainx_calculator.wasm \
  --source-account <source-account> \
  --network <network-name>
```

Save the returned contract ID for later invocation.

### 3. Initialize the contract

The current `initialize` function accepts:

- `admin: Address`
- `operator: Address`

Example:

```bash
soroban contract invoke \
  --id <contract-id> \
  --source-account <source-account> \
  --network <network-name> \
  -- initialize \
  --admin <admin-address> \
  --operator <operator-address>
```

### 4. Read contract state

Useful follow-up calls after deployment:

```bash
soroban contract invoke \
  --id <contract-id> \
  --source-account <source-account> \
  --network <network-name> \
  -- get_config \
  --severity critical
```

```bash
soroban contract invoke \
  --id <contract-id> \
  --source-account <source-account> \
  --network <network-name> \
  -- get_stats
```

## Artifact Guidance

For this repository, the main artifact contributors and operators should expect is:

- release WASM for deployment:
  `apexchainx_calculator/target/wasm32-unknown-unknown/release/apexchainx_calculator.wasm`

Optional local outputs include:

- debug build artifacts under `apexchainx_calculator/target/debug`
- test binaries under `apexchainx_calculator/target/debug/deps`

## Verification Notes

As of the latest stabilization pass:

- `cargo test` passes
- the crate compiles cleanly
- the checked-in test suite is wired into the crate and runs

The current test suite covers:

- role and authorization behavior
- SLA reward and penalty logic
- pause and unpause behavior
- statistics
- audit-mode calculation parity
- history recording and pruning

## Backend Relationship

The backend repo is expected to invoke this contract and translate contract results into backend API responses.

That dependency matters because:

- SLA logic must stay aligned with backend expectations
- result encoding must remain deterministic
- API consumers in `apexchainx-fe` only see what `apexchainx-be` returns
- config reads should prefer explicit snapshot-style contract views where stable ordering matters

## Current Limitations

This repository is now stable at the crate level, but the overall contract layer is still narrow.

Examples:

- only one contract crate exists right now
- deployment automation is not checked in
- there is not yet a broader contract workspace with escrow or settlement modules
- cross-repo contract invocation is still a backend integration concern, not something managed here directly

## Governance Policy

### Admin transfer (two-step)

Admin authority is transferred via a two-step flow to prevent accidental reassignment:

1. Current admin calls `propose_admin(caller, new_admin)` — stores a pending proposal.
2. Proposed admin calls `accept_admin(caller)` — atomically promotes caller to admin and clears the proposal.

The old admin retains authority until `accept_admin` succeeds. `get_pending_admin()` is queryable at any time.

### Operator handoff (two-step)

Operator rotation follows the same pattern:

1. Admin calls `propose_operator(caller, new_operator)`.
2. New operator calls `accept_operator(caller)` to activate.

`get_pending_operator()` exposes the pending state for governance visibility.

### Admin renounce

`renounce_admin(caller)` permanently removes admin authority. This is **irreversible**: after renounce, all admin-gated functions (`set_config`, `pause`, `unpause`, `set_operator`, `prune_history`) are permanently locked. Any pending admin proposal is also cleared atomically.

Backend operators should treat a renounced contract as immutable from a governance perspective. There is no recovery path by design — if recovery is needed, redeploy and reinitialize.

### Pause metadata

`pause(caller, reason)` stores a `PauseInfo` struct containing the reason string and the ledger timestamp at pause time. `get_pause_info()` returns this metadata while the contract is paused. `unpause` clears it. This gives backend operators operational context without requiring off-chain state.

## Related Repositories

- `apexchainx-fe` -> frontend application
- `apexchainx-be` -> backend and contract bridge
