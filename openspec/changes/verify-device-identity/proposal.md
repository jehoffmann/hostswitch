## Why
A receiver index names a slot, not a physical device, and replies should be matched exactly to their request. The device behind an id should be checked before a write.

## What Changes
- Before a write, the device behind a receiver index is checked against what the configuration expects.
- Replies and error reports are matched to the exact request.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- `safety`

## Impact
Backend request/reply matching and a device identity check; config may remember a device fingerprint.

## Out of Scope
Pairing management.
