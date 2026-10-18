## Why
N-03 says tools are "discovered, version-checked and wrapped", but the code resolves and runs tools without ever asking for their version, so an unsupported tool version only shows up as odd output.

## What Changes
- The version of each helper tool is read once, shown to the user and compared with the supported range; an unsupported version is reported clearly.

## Capabilities
### New Capabilities
- `diagnostics`

### Modified Capabilities
- None

## Impact
Tool runner and a new `doctor`-style command or section of `status`.

## Out of Scope
Installing or updating tools.
