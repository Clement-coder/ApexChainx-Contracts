# ApexChainx Smart Contracts (apexchainx-contracts) – Codex Context

## Overview

This repository contains Soroban smart contracts used by ApexChainx to:

- calculate SLA penalties and rewards
- execute blockchain-based payments
- manage escrow logic
- handle multi-party settlements

These contracts run on the Stellar network and are invoked by the backend.

Reference: :contentReference[oaicite:0]{index=0}

---

## Tech Stack

- Language: Rust
- Framework: Soroban SDK
- Blockchain: Stellar (Soroban)
- Build: cargo + wasm32 target

---

## Core Contracts

### 1. SLA Calculator

Responsible for:

- calculating SLA results (penalty or reward)
- returning deterministic results
- storing configuration

Key functions:

- initialize
- calculate_sla
- execute_payment
- get_config
- update_config

Important:

- must be deterministic
- must not depend on external state
- must match backend SLA logic exactly

---

### 2. Payment Escrow

Responsible for:

- locking funds
- releasing funds on conditions
- handling refunds

Key functions:

- create_escrow
- release_escrow
- refund_escrow

---

### 3. Multi-Party Settlement

Responsible for:

- splitting payments between parties
- handling shared outage costs

Key functions:

- create_settlement
- execute_settlement

---

## Architecture

Contracts are:

- stateless where possible
- deterministic
- executed via backend
- validated by Stellar network

Flow:
Backend → Contract Invocation → Result → Payment Execution

---

## Important Constraints

- calculations must be deterministic
- no floating point errors (use integers)
- gas cost must be minimized
- contracts must be idempotent where applicable
- inputs must be validated strictly

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

## Known Risk Areas (Generate Issues)

### SLA Logic

- mismatch between backend and contract
- incorrect rounding or integer precision
- edge cases (boundary MTTR values)

### Payments

- double execution risk
- missing authorization checks
- incorrect recipient addresses

### Security

- admin privilege misuse
- contract initialization errors
- unauthorized config updates

### Gas Optimization

- unnecessary storage writes
- inefficient loops
- repeated computation

---

## Coding Rules

- use integer math only
- avoid unnecessary state writes
- validate all inputs
- emit events for important actions
- keep functions small and testable

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

Compatibility rule:

- additive read-only contract helpers are preferred over changing the shape of
  `SLAResult`
- changes that alter symbol meanings or add new backend-facing semantics should
  increment the documented schema version intentionally

## Event Convention

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

## Goal for Codex

Generate issues that:

- improve contract correctness
- ensure security of payments
- optimize gas usage
- guarantee deterministic behavior
