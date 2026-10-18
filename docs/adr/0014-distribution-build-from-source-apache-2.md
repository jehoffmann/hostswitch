# ADR-0014: Distribution: build from source, Apache-2.0

- Status: Accepted
- Date: 2026-10-18

## Context
The first plan aimed at the Mac App Store. DDC on Apple Silicon needs the private IOAVService API (guideline 2.5.1), the helper programs cannot run in the sandbox, and a hardware tool is hard for reviewers to test. The author decided against it: users build the tool themselves.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- constraint:N-06 — build from source, Apache-2.0, SPDX headers
- constraint:N-02 — Cargo as the build system

## Decision
No App Store, sandbox, signing or notarization requirements. The project is licensed Apache-2.0 (`LICENSE`); every source file starts with `// SPDX-License-Identifier: Apache-2.0` (`#` in TOML and shell files). Dependencies must be compatible with Apache-2.0; GPL programs such as `hidapitester` are used but not bundled. Private or platform-specific APIs and external tools are allowed.

## Consequences
- Native DDC with private APIs is possible later.
- Users need a Rust toolchain and the helper programs; there is no installer.

## Alternatives considered
- Mac App Store build: blocked by the private API, the sandbox and review risk.
- Notarised download: possible later, not needed now.
