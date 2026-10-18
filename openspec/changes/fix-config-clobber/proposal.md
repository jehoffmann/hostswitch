## Why
Looking at the hardware (`list`) must never put the user's configuration at risk. Remembering devices should leave a configuration that cannot be loaded untouched, a failed save should be reported, and fields written by newer versions should survive a save.

## What Changes
- Remembering devices never overwrites a configuration that could not be loaded.
- A failed save is reported, not ignored.
- Fields the program does not know are kept when it saves.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- `offline-devices`
- `config`

## Impact
Config load/save and the device-remembering path of the CLI. No new dependency.

## Out of Scope
Changing the configuration format or the version number.
