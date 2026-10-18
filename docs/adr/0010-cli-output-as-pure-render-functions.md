# ADR-0010: CLI output produced by pure, tested functions

- Status: Accepted
- Date: 2026-10-18

## Context
Several commands print tables of hosts, devices and results; the layout must stay consistent and testable, and a later tray app should reuse the same wording.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- spec:cli/F-07 — commands
- spec:cli/F-07b — show prints the configuration only
- spec:cli/F-07c — status compares with the hardware
- spec:cli/F-07f — output style
- constraint:N-08 — no hard-coded manufacturer names
- change:sanitize-hardware-text/SEC-11 — sanitise text from hardware

## Decision
All printed text is built by functions in `crates/hostswitchctl/src/render.rs` that take data and return strings; commands only gather data and print. Style: capitalised `Label:` columns (Display, Keyboard, Mouse), `Name (id)`, aligned values, `Slot N`, `OK`/`FAIL`/`SKIP`. No manufacturer name is hard-coded in code or output: types are `Peripheral*`; receiver devices without a readable name are `Unifying device N` or `Bolt device N`; a name read from the hardware may contain whatever the hardware reports.

## Consequences
- The layout is covered by unit tests and by Gherkin end-to-end scenarios.
- Text from the hardware is printed as received; sanitising it is planned (change `sanitize-hardware-text`).

## Alternatives considered
- Formatting inside each command: less code, but untestable layout and duplicated wording.
