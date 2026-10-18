# ADR-0002: Implement hostswitch in Rust

- Status: Accepted
- Date: 2026-10-18

## Context
The tool drives hardware through byte-level protocols (DDC, HID++) and runs helper programs, so memory safety and strict parsing matter. The author wants to learn and improve Rust, and Linux and Windows are future targets.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- constraint:N-02 — Rust, edition 2024, Cargo workspace
- constraint:N-01 — macOS first, other platforms later
- constraint:N-07 — core without OS or UI code

## Decision
Rust (edition 2024, stable toolchain), a Cargo workspace, `tokio` for async I/O and timeouts, `clap` for the CLI, `serde` for the configuration. Workspace lints: `unsafe_code = "forbid"` for all crates, `unwrap_used` and `expect_used` denied outside tests. `cargo build`, `cargo test` and `cargo clippy` are the source of truth; no IDE project files.

## Consequences
- One toolchain for core, backends, CLI and later the app; the same domain code runs on every platform.
- OS APIs that need `unsafe` (private macOS DDC, Windows APIs) must live in a separate backend crate, never in the core.
- Let-chains are used, so the minimum Rust version is 1.88 (the manifest still says 1.85: to be corrected).

## Alternatives considered
- A native Swift/SwiftUI app (the first prototype): macOS only, discarded in favour of Rust.
