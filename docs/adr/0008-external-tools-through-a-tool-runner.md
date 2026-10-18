# ADR-0008: External helper programs behind a tool runner with pure parsers

- Status: Accepted
- Date: 2026-10-18

## Context
The first backends use the user-installed programs `m1ddc` (DDC on Apple Silicon) and `hidapitester` (HID access). They must be found, run safely and their output parsed without panics. `hidapitester` is GPL-3.0 and is therefore not bundled.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- spec:tools/N-03 — resolve and run external tools
- spec:tools/N-03b — display backend behaviour
- spec:tools/N-03c — parsers tolerate garbage
- spec:safety/S-04 — no shell, timeout, kill
- change:harden-tool-execution/SEC-6 — only absolute directories
- change:harden-tool-execution/SEC-8 — bounded output
- change:trusted-tool-binaries/SEC-5 — trust in tool binaries
- change:native-backends/N-03e — in-process backends
- change:tool-version-check/N-03d — tool versions

## Decision
A `ToolExecuting` trait with a `ToolRunner`: it resolves a tool from an override, then the Homebrew directories, then `PATH`; starts it with `tokio::process` (arguments as separate elements, never a shell, stdin null, `kill_on_drop`), bounds it by a timeout, captures stdout and stderr and decodes them leniently. Backends contain only command construction and sequencing; parsing is in pure functions tested against recorded output (`crates/hostswitch-backends/tests/fixtures`). The tool runner is replaceable by a fake in tests.

## Consequences
- Fast to build, parsers and sequencing are testable without hardware.
- Cost: a process per call (Bluetooth requests take 1 to 2 s because `hidapitester` always performs every read). Hardening of the runner and trust in the binaries are planned (changes `harden-tool-execution`, `trusted-tool-binaries`, `native-backends`).

## Alternatives considered
- Native in-process backends from the start: no helper programs, but private macOS APIs and `unsafe` code in the first iteration (planned, see `native-backends`).
