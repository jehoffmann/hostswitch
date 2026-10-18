## Why
The configuration file may later hold secrets and can be written by several tools. Saving should use private permissions, an unpredictable temporary file, flush to disk and cope with concurrent writers.

## What Changes
- Create the temporary file exclusively with a random name and private permissions.
- Make the configuration file and its directory private to the user.
- Flush to disk before the atomic replace; serialise concurrent writers.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- `config`

## Impact
Config save path (`ConfigStore`). Unix permission calls (no `unsafe`).

## Out of Scope
Encrypting the configuration; keychain storage (future change).
