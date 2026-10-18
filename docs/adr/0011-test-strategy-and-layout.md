# ADR-0011: Test strategy: unit, integration and system tests; Gherkin descriptions; traceability

- Status: Accepted
- Date: 2026-10-18

## Context
Switching hardware is risky, so most behaviour must be verifiable with fakes, and every requirement must be traceable to a test. The layout follows the Rust book and https://www.rustfaq.org/en/best-practices-for-organizing-tests-in-rust-projects/.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- constraint:N-09 — tests apart from functional code, Gherkin traced to specs
- spec:switching/F-01b — a dry run changes nothing (used by agents)
- spec:safety/S-05 — dry run is honest
- change:test-tool-directory-hook — fake tools for end-to-end tests

## Decision
**Unit tests** are in the same file as the code, in `#[cfg(test)] mod tests`: fast, no file or process I/O, private items allowed. **Integration tests** (public API, one binary per file) are in each crate's `tests/` directory; helpers shared by several files go to `tests/common/mod.rs`; a test that touches files or starts programs belongs here. **Fakes** are never in functional files: `hostswitch-core/src/testing.rs` behind the `testing` feature, `hostswitch-backends/src/test_support.rs` under `cfg(test)`. Recorded tool output is in `crates/hostswitch-backends/tests/fixtures/`. **Test descriptions** are Gherkin `.feature` files in `crates/*/tests/features/` with scenario titles identical to the OpenSpec scenarios; `@bdd` scenarios are run by cucumber-rs (`crates/hostswitchctl/tests/cli.rs`) against the real binary with fake ids and a temporary `HOME`; the others carry `@test:<crate>/<path>::<fn>` tags to their Rust test. `scripts/check_traceability.py` verifies OpenSpec scenario -> Gherkin -> test and the ADR references. **System tests** on real hardware are opt-in and read-only by default. `hostswitchctl switch` really switches, so agents and tests use `--dry-run`.

## Consequences
- Spec, scenario and test cannot drift silently; open gaps are listed (3 today).
- cucumber-rs adds about 77 dev-only packages to the lock file.
- Some scenarios need a way to run the real binary against fake tools (`test-tool-directory-hook`).

## Alternatives considered
- Unit tests in `src/<module>/tests.rs`: tried and reverted, since the recommended layout keeps them in the same file.
- Plain Rust tests without Gherkin: no readable link to the requirements.
