## Why
Keep the dependency set small and checked: minimal tokio features for the core, and advisory and license checks (`cargo deny`) over the production and development dependencies.

## What Changes
- Reduce enabled tokio features in `hostswitch-core` to what it uses.
- Run `cargo deny` (advisories, licenses compatible with Apache-2.0, bans) in the developer workflow once installed.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- None

## Impact
Cargo manifests and developer documentation only; no behaviour changes.

## Out of Scope
Replacing dependencies.
