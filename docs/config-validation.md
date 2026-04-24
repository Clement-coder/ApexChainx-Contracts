# Configuration Validation Rules

This document outlines the validation rules enforced by the `set_config` function in the SLA Calculator contract to ensure safe and meaningful configuration parameters.

## Overview

The SLA Calculator contract validates all configuration updates to prevent admin-side misuse and unexpected runtime behavior. Invalid configuration writes will fail deterministically with specific error codes.

## Supported Severities

The contract only supports four severity levels:

- `critical` - Highest priority incidents
- `high` - Important incidents  
- `medium` - Standard incidents
- `low` - Low priority incidents

## Validation Rules

### General Rules (Apply to All Severities)

#### Threshold Minutes
- **Range**: 1 to 1440 minutes (24 hours maximum)
- **Purpose**: Prevents unrealistic thresholds (0 minutes) or extremely long thresholds
- **Error**: `InvalidThreshold` (error code 8)

#### Penalty Per Minute
- **Range**: 1 to 10,000 units per minute
- **Purpose**: Ensures penalties are positive and within reasonable bounds
- **Error**: `InvalidPenalty` (error code 9)

#### Reward Base
- **Range**: 1 to 100,000 units
- **Purpose**: Ensures rewards are positive and within reasonable bounds
- **Error**: `InvalidReward` (error code 10)

### Severity-Specific Rules

#### Critical Severity
- **Threshold**: Maximum 60 minutes
- **Penalty**: Minimum 50 units per minute
- **Rationale**: Critical incidents should have short response windows and significant penalties

#### High Severity  
- **Threshold**: Maximum 120 minutes
- **Penalty**: Minimum 25 units per minute
- **Rationale**: High incidents should have reasonable response windows and moderate penalties

#### Medium Severity
- **Threshold**: Maximum 240 minutes (4 hours)
- **Penalty**: Minimum 10 units per minute
- **Rationale**: Medium incidents can have longer response windows but still need meaningful penalties

#### Low Severity
- **Penalty**: Maximum 100 units per minute
- **Rationale**: Low severity incidents should have lower penalties relative to higher severities

## Error Handling

### Error Codes

| Error Code | Error Name | Description |
|------------|------------|-------------|
| 8 | InvalidThreshold | Threshold minutes outside valid range or severity-specific limits |
| 9 | InvalidPenalty | Penalty per minute outside valid range or severity-specific limits |
| 10 | InvalidReward | Reward base outside valid range |
| 11 | InvalidSeverity | Severity not in supported list (critical, high, medium, low) |

### Deterministic Failure

All validation failures are deterministic:
- The same invalid parameters will always produce the same error
- No partial state changes occur - validation happens before any storage updates
- Error codes are specific to help identify the exact validation issue

## Default Configuration Values

The contract initializes with these validated defaults:

| Severity | Threshold | Penalty/Min | Reward Base |
|----------|-----------|-------------|-------------|
| critical | 15 min | 100 | 750 |
| high | 30 min | 50 | 750 |
| medium | 60 min | 25 | 750 |
| low | 120 min | 10 | 600 |

## Best Practices for Backend Operators

### 1. Gradual Changes
- Make incremental changes rather than drastic jumps
- Test new configurations in a staging environment first

### 2. Severity Consistency
- Maintain logical progression between severities
- Higher severities should generally have lower thresholds and higher penalties

### 3. Economic Considerations
- Consider the total economic impact of penalties and rewards
- Ensure penalty structures incentivize desired behavior

### 4. Monitoring
- Monitor SLA calculation results after configuration changes
- Watch for unexpected patterns in violations or rewards

### 5. Validation Testing
- Use the `calculate_sla_view` function to test configurations before applying
- Verify edge cases (threshold boundaries) work as expected

## Example Valid Configurations

```rust
// Valid critical configuration
set_config(admin, critical, 30, 150, 1000)

// Valid high configuration  
set_config(admin, high, 45, 75, 800)

// Valid medium configuration
set_config(admin, medium, 90, 30, 600)

// Valid low configuration
set_config(admin, low, 180, 15, 500)
```

## Example Invalid Configurations

```rust
// Invalid: threshold too high for critical
set_config(admin, critical, 120, 100, 750) // InvalidThreshold

// Invalid: penalty too low for high severity
set_config(admin, high, 30, 10, 750) // InvalidPenalty

// Invalid: negative reward
set_config(admin, medium, 60, 25, -100) // InvalidReward

// Invalid: unsupported severity
set_config(admin, urgent, 15, 100, 750) // InvalidSeverity
```

## Implementation Notes

- Validation occurs before any state changes
- All validation rules are enforced at the contract level
- The contract emits events for successful configuration updates
- Failed validations do not emit events or consume gas beyond the validation check
