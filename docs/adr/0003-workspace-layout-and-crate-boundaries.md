# ADR-0003: Cargo workspace with core, backends and front-ends

- Status: Accepted
- Date: 2026-10-18

## Context
Switching logic must be testable without hardware and reusable by a command line, a tray app and later other platforms.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- constraint:N-07 — core domain and wire protocol free of OS and UI code
- constraint:N-08 — vendor-neutral names in code and output
- spec:cli/F-07 — the CLI is one front-end among several

## Decision
Crates under `crates/`: `hostswitch-core` (domain, configuration, planner, executor, host detection, config bootstrap; depends only on serde, serde_json, uuid, thiserror, async-trait and tokio time/sync), `hostswitch-backends` (tool runner, `m1ddc` and `hidapitester` wrappers, HID++ framing and parsers; depends on core), `hostswitchctl` (CLI; depends on both), and later `hostswitch-app` (see ADR-0012). Dependency rule: front-ends -> backends -> core; the core never depends on an OS, UI or process crate and contains no `unsafe`. Type names are vendor-neutral (`Peripheral`, `PeripheralBackend`).

## Consequences
- Everything in the core can be tested with fakes.
- A new front-end or backend does not touch the core.
- Network code (ADR-0013) goes into the core behind a transport trait, so it needs no OS crate.

## Alternatives considered
- One crate: simpler, but the CLI would pull in process and OS code and the core could not be reused on other platforms.
