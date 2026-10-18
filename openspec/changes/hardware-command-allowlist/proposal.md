## Why
Only channel switching may ever reach a monitor or peripheral. Today this holds by construction, but nothing in the types prevents a future change from sending another VCP code, a different display-tool sub-command or another HID++ feature (power, reset, firmware update, pairing). A damaged or unusable monitor is not recoverable by software.

## What Changes
- Hardware commands become a closed, typed set; there is no way to send free-form text or arbitrary bytes to a display tool or device.
- Input values come only from the user's profile; their range is validated.
- Writes are rate-limited.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- `safety`

## Impact
Backend interfaces and the request builders for m1ddc and HID++.

## Out of Scope
Adding any new kind of hardware command (needs its own requirement).
