## Why
Text read from hardware (device and display names) should be cleaned of control characters and limited in length before it is printed or stored.

## What Changes
- Control characters are removed and length is limited for every text that comes from hardware, before it is printed or stored.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- `discovery`

## Impact
Parsers for tool output and the places that print or store names.

## Out of Scope
Changing which names are shown.
