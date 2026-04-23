# ApexChainx System

## Repositories

- apexchainx-fe → frontend
- apexchainx-be → backend
- apexchainx-contracts → smart contracts

## System Flow

User → FE → BE → Contracts → BE → FE

## Rules

- FE never talks to contracts directly
- BE is the bridge
- Contracts are execution layer only
