## Why
Helper programs should be found in absolute directories only, run with bounded output and be stopped completely on a timeout, and every external program (including the hostname lookup) should go through the same runner.

## What Changes
- Only absolute directories are searched for tools.
- Output is limited in size.
- A timeout ends the whole process group.
- Every external program, including `hostname`, goes through the same runner with a timeout.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- `tools`

## Impact
Tool runner and the hostname lookup in `init`. May use a safe, existing API for the hostname instead of a process.

## Out of Scope
Choosing which binaries are trusted (change `trusted-tool-binaries`).
