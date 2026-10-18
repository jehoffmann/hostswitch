## Why
Both helper programs cost a process per call (1 to 2 s for a Bluetooth ping because the helper always performs every read), cannot stop at the matching reply, are GPL-licensed in one case and cannot read the real input ports of a monitor.

## What Changes
- Hardware access in-process: HID++ through a maintained HID library and DDC through a native backend that can also read the monitor capabilities.
- The helper-program backends stay as fallback until the native ones are verified on hardware.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- `discovery`

## Impact
New dependencies (HID library, platform DDC code); `unsafe` confined to a separate backend crate; security review required; may supersede `trusted-tool-binaries`.

## Out of Scope
Other vendors' proprietary protocols.
