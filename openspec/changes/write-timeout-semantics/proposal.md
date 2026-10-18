## Why
When a write runs into the step timeout its outcome is unknown. The run should say so, never repeat it or send it to another path, and timeouts should fit the slowest backend.

## What Changes
- A timed-out write is reported as "unknown" and never retried or sent to an alias.
- The step timeout is at least the worst case of the backend for a single attempt.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- `switching`

## Impact
Executor outcome type and its use by the CLI; timeouts of the Bluetooth path.

## Out of Scope
Reading the device state back to resolve the unknown outcome (optional follow-up).
