# ApexChainx Smart Contracts — Codex Context

> **Technical deep-dive:** Architecture, constraints, event conventions, and
> integration guidance for the ApexChainx Soroban smart contract ecosystem.

## Table of Contents

- [Overview](#overview)
- [Technology Stack](#technology-stack)
- [Core Contracts](#core-contracts)
- [Architecture](#architecture)
- [Constraints & Design Principles](#constraints--design-principles)
- [Critical Logic: SLA Calculation](#critical-logic-sla-calculation)
- [Risk Assessment](#risk-assessment)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Cross-Repo Dependencies](#cross-repo-dependencies)
- [Backend-Facing Result Schema](#backend-facing-result-schema)
- [Event Conventions](#event-conventions)
- [SC-097: Event Replay & Recovery](#sc-097-event-replay--recovery)

---

## Overview

This repository contains Soroban smart contracts that power the ApexChainx
platform. These contracts execute on the **Stellar network** and are invoked
exclusively through the backend API layer.

### Primary Responsibilities

| Function | Description |
|----------|-------------|
| SLA Calculation | Deterministic penalty/reward computation based on service metrics |
| Payment Escrow | Lock and conditionally release Stellar token payments |
| Multi-Party Settlement | Split shared outage costs between multiple parties |

### Invocation Model

```
Backend API → Contract Invocation → Result Processing → Payment Execution
```

**Key constraint:** Contracts are never called directly by the frontend.
All interactions go through the backend bridge.

---

## Core Contracts

### 1. SLA Calculator (Production-Ready)

**Status:** ✅ Implemented and tested

The SLA calculator is the primary contract in this repository. It handles
deterministic SLA computation, configuration management, and event emission.

#### Responsibilities

| Responsibility | Details |
|---------------|---------|
| SLA Computation | Calculate penalty or reward based on severity and MTTR |
| Configuration | Store and manage severity-based SLA parameters |
| Governance | Admin/operator role management with two-step transfers |
| Events | Versioned lifecycle events for backend consumers |
| History | On-chain calculation history with pruning |

#### Key Functions

| Function | Auth | Purpose |
|----------|------|---------|
| `initialize` | — | One-time setup with admin/operator roles |
| `calculate_sla` | Operator | Full SLA computation (mutating) |
| `calculate_sla_view` | Public | Read-only SLA simulation |
| `set_config` | Admin | Update severity configuration |
| `get_config_snapshot` | Public | Ordered config export for backend |

#### Critical Constraints

- ✅ Deterministic — same inputs always produce the same output
- ✅ Self-contained — no external state dependencies
- ✅ Backend-parity — must match backend SLA logic exactly

---

### 2. Payment Escrow (Planned)

**Status:** 📋 Not yet implemented

Future contract for locking and conditionally releasing Stellar token payments
based on SLA results.

#### Planned Responsibilities

| Operation | Description |
|-----------|-------------|
| `create_escrow` | Lock funds in escrow with SLA conditions |
| `release_escrow` | Release funds when SLA conditions are met |
| `refund_escrow` | Return funds when SLA conditions are violated |

---

### 3. Multi-Party Settlement (Planned)

**Status:** 📋 Not yet implemented

Future contract for splitting shared outage costs between multiple parties.

#### Planned Responsibilities

| Operation | Description |
|-----------|-------------|
| `create_settlement` | Initiate a multi-party cost split |
| `execute_settlement` | Process and distribute payments |

---

## Architecture

### Design Principles

| Principle | Description |
|-----------|-------------|
| **Stateless** | Contracts minimize on-chain state; configuration is the primary persisted data |
| **Deterministic** | Same inputs always produce identical outputs — no randomness |
| **Backend-Mediated** | All contract invocations flow through the backend API layer |
| **Network-Validated** | Stellar consensus validates all contract executions |

### Execution Flow

```
Backend  ──→  Contract Invocation  ──→  Result Processing  ──→  Payment Execution
    ↑                                                                     |
    └─────────────────────  Event Stream Replay  ←─────────────────────────┘
```

---

## Constraints & Design Principles

### Determinism

All contract computations must be **fully deterministic**. This is non-negotiable
because:
- Backend and contract logic must produce identical results
- Event replay depends on reproducible outcomes
- Audit trails require verifiable computation

### Integer Math

```
❌ Floating point:  NOT ALLOWED
✅ Integer math:    ALWAYS REQUIRED
```

No floating point operations. All calculations use integer arithmetic with
appropriate precision scaling.

### Gas Optimization

| Strategy | Rationale |
|----------|-----------|
| Minimize storage writes | Each write consumes significant gas |
| Avoid loops over unbounded data | Gas costs scale with iteration count |
| Use view functions for reads | Read-only calls have no gas cost |
| Batch operations where possible | Reduce per-operation overhead |

### Idempotency

Contracts must be idempotent where applicable:
- Re-processing the same SLA calculation returns the same result
- Duplicate event consumption does not produce errors
- Configuration updates are idempotent for same parameters

### Input Validation

All function inputs are validated at the contract boundary before any state
changes occur:
- Severity levels are checked against supported values
- Thresholds and penalties are validated as positive integers
- Addresses are verified for format correctness
- Bounds checking on all numeric parameters

---

## Critical Logic

### SLA Calculation

Inputs:

- severity
- MTTR
- threshold config

Output:

- status (met / violated)
- amount (positive = reward, negative = penalty)

Must exactly match backend logic.

---

## Risk Assessment

### Risk Matrix

| Category | Risk | Severity | Mitigation |
|----------|------|----------|------------|
| **SLA Logic** | Backend-contract mismatch | High | Golden test vectors, parity CI checks |
| **SLA Logic** | Integer precision errors | Medium | Use only integer math, test boundary conditions |
| **SLA Logic** | Edge cases (boundary MTTR) | Medium | Comprehensive boundary test suite |
| **Payments** | Double execution | High | Idempotency keys, outage_id deduplication |
| **Payments** | Missing authorization | Critical | require_auth() on all privileged functions |
| **Payments** | Wrong recipient | High | Address validation, two-step confirmation |
| **Security** | Admin privilege misuse | Critical | Two-step transfers, audit events, renounce safety |
| **Security** | Initialization errors | High | One-time init guard, verify-after-init |
| **Security** | Unauthorized config changes | High | Role-based access control, event emission |
| **Gas** | Unnecessary storage writes | Medium | Use view functions for reads, batch writes |
| **Gas** | Inefficient loops | Medium | Bound data structures, paginate history |
| **Gas** | Repeated computation | Low | Cache results where possible |

---

## Coding Standards

### Mandatory Rules

| Rule | Rationale |
|------|-----------|
| Integer math only | Floating point is non-deterministic and gas-expensive |
| Validate all inputs | Prevent invalid state transitions |
| Emit events for state changes | Enable backend audit and replay |
| Keep functions small | Improve auditability and testability |
| Avoid unnecessary state writes | Minimize gas costs |
| Use require_auth() for privileged ops | Enforce role-based access control |

### Code Organization

```
contract_name/
├── Cargo.toml
└── src/
    ├── lib.rs           # Contract entry point, storage keys, error types
    ├── tests.rs         # Integration tests
    ├── version_negotiation.rs  # Multi-contract versioning
    ├── storage_version.rs      # Schema versioning
    ├── event_schema.rs         # Canonical event definitions
    └── ...              # Additional domain modules
```

---

## Testing Requirements

- unit tests for each function
- edge case tests
- integration tests with backend expectations
- deterministic output validation

---

## Cross-Repo Dependencies

- apexchainx-be → invokes contracts
- apexchainx-fe → displays results

Important:

- contract logic must never diverge from backend expectations
- API response structure depends on contract output
- result symbol mappings are versioned through the contract-facing schema

## Backend-Facing Result Schema

The SLA calculator now exposes an explicit result schema contract so the backend
does not have to infer symbol meanings implicitly.

Current schema version:

- schema label: `v1`
- schema version: `1`

Current symbol mappings:

- status met -> `met`
- status violated -> `viol`
- payment reward -> `rew`
- payment penalty -> `pen`
- rating exceptional -> `top`
- rating excellent -> `excel`
- rating good -> `good`
- rating poor -> `poor`

Compatibility rule:\n\n- additive read-only contract helpers are preferred over changing the shape of\n  `SLAResult`\n- **Versioning**: Breaking ABI/symbol changes → MAJOR bump (v2.0.0), update `schema_version` in `get_result_schema()`.\n- Backend: Pin contract ID/version, regenerate parity tests from snapshots post-release.\n\n**Backend Dependency Expectations**:\n- Match `calculate_sla_view()` exactly with local logic.\n- Consume `test_snapshots/tests/*.json` for golden vectors.\n- Monitor git tags `vX.Y.Z` for releases.\n\n## Event Convention

Lifecycle events are versioned so backend consumers can reason about event shape
without inferring it from position alone.

Current event topic convention:

- topic 0 -> event name
- topic 1 -> event version
- topic 2 -> event-specific context such as severity or caller

Current event version:

- `v1`

Current SLA calculation event payload:

- outage id
- result status
- payment type
- rating
- MTTR minutes
- threshold minutes
- amount

---

## SC-097: Event Replay and Recovery Guidance

### Intended Event Consumption

Backend consumers should treat the SLA calculator's on-chain events as a
supplementary audit trail, not as the primary source of truth for SLA outcomes.
The canonical state is always the most recent `calculate_sla` result stored
on-chain and retrieved via direct contract reads.

### Event Replay Assumptions

- Events are emitted with a versioned topic layout (`v1`). Consumers must check
  `topic[1]` before deserialising the payload to avoid version mismatches.
- Events are not guaranteed to be present for every ledger (e.g. archival or
  network gaps). Consumers must handle missing events gracefully.
- Re-processing the same event twice must be idempotent on the backend — use the
  `outage_id` field as a deduplication key.

### Missed-Event Recovery

1. Detect a gap by comparing the last processed ledger sequence against the
   current ledger sequence from `getLatestLedger`.
2. Use `getEvents` with an explicit `startLedger` to replay missed events in
   chronological order.
3. Cross-check replayed results against `calculate_sla_view` for the same
   `outage_id` to validate consistency.
4. Log any discrepancy between the event payload and the on-chain state as a
   potential double-execution risk.

### Canonical State vs Event-Stream Interpretation

| Operation | Recommended source |
|---|---|
| Current SLA result for an outage | Direct contract read (`calculate_sla_view`) |
| Audit / history of all outages | Event stream replay |
| Config at a point in time | `cfg_upd` events + `get_config` |
| Payment amounts | Event payload `amount` field (signed integer) |

---

## Goal for Codex

Generate issues that:

- improve contract correctness
- ensure security of payments
- optimize gas usage
- guarantee deterministic behavior
