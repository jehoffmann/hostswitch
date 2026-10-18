# ADR-0004: Hardware access only through backend interfaces

- Status: Accepted
- Date: 2026-10-18

## Context
Displays and keyboards or mice are reached today through helper programs (ADR-0008); native implementations and other operating systems must be able to replace them. Only channel switching may ever reach the hardware.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- spec:safety/S-01 — only channel switching reaches the hardware
- spec:discovery/F-05 — listing displays
- spec:discovery/F-05a — listing keyboards and mice
- constraint:N-01 — platforms
- change:intel-display-backend/N-01 — second display backend
- change:native-backends/R-3b — native backends
- change:hardware-command-allowlist/S-06 — closed set of commands

## Decision
Two async traits in the core: `DisplayBackend` (`list_displays`, `set_input`, `current_input`) and `PeripheralBackend` (`list_devices`, `switch_host`, `current_host`), each with `id()` and `is_available()`. A `BackendRegistry` picks the first available backend per capability at runtime. Fakes for tests live behind the core's `testing` feature. Per-OS work is limited to the display backend (macOS `m1ddc` now, native IOAVService or IOFramebuffer later; Linux `ddcutil` or I2C; Windows DXVA2); HID++ framing and parsing are shared.

## Consequences
- The executor, CLI and tests never see a tool or an OS API.
- The traits take free-form strings (display id, device id) and do not yet restrict the commands; the change `hardware-command-allowlist` replaces that by a closed, typed set.
- There is no shared contract-test suite for backends yet (a known gap).

## Alternatives considered
- Calling the tools directly from the executor: shorter, but untestable without hardware and not replaceable.
