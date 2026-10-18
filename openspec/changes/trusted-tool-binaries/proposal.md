## Why
Once an app that holds the Input Monitoring permission starts helper programs, those programs should be trusted: either in-process backends or pinned binaries with a minimal environment.

## What Changes
- Decide how tool binaries are trusted: native backends without helper programs, or a pinned absolute path with a content hash and a minimal environment.
- The decision (and the migration) is made in design.md by the architect.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- `tools`

## Impact
Tool runner or the backend implementations; possible new dependency (`hidapi`) and per-OS code.

## Out of Scope
The tray app itself.
