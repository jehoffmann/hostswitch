## Why
N-01 and R-4 promise Apple Silicon and Intel. The only display backend wraps m1ddc, which does not work on Intel Macs (ddcctl needs the Intel framebuffer path).

## What Changes
- A second display backend for Intel Macs is selected at runtime, behind the same display interface.

## Capabilities
### New Capabilities
- None

### Modified Capabilities
- `discovery`
- `switching`

## Impact
New backend in `hostswitch-backends`; possible new external tool or native code; `unsafe` only outside the core crate if a native path is chosen (needs security review).

## Out of Scope
Linux and Windows backends.
