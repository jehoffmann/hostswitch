## Why
Display and device ids from the configuration are passed to helper programs and compared with each other. They should be validated and canonical when the configuration is loaded, and the dry run should show every id that may receive a write.

## What Changes
- Display and device ids are parsed into canonical form when the configuration is loaded; anything else is rejected.
- Aliases and the keyboard/mouse distinctness check use the canonical form.
- The dry run lists every id that can receive a write.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- `config`
- `switching`

## Impact
Config validation, display/device id handling and the plan output. Existing configurations with canonical ids keep working.

## Out of Scope
Verifying that a receiver slot still holds the same physical device (separate change `verify-device-identity`).
