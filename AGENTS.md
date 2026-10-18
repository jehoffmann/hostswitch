# AGENTS.md — instructions for AI coding agents (tool-agnostic)

hostswitch: a macOS menu-bar app that switches display inputs (DDC) and Logitech Easy-Switch slots together. Read the requirements (OpenSpec in `openspec/`: `specs/` = current behaviour, `changes/` = proposals; `docs/requirements.md` = constraints, risks and the ID map) and the architecture decision records in `docs/adr/` before changing anything; they are the source of truth. Requirements are written as OpenSpec specs and changes, test descriptions as Gherkin `.feature` files (see `.agents/prompts/README.md`).

## Repo map
| Path | Purpose |
|------|---------|
| `Cargo.toml`, `crates/` | Cargo workspace (Rust 2024, stable). Editor-agnostic. |
| `crates/hostswitch-core` | Domain, config (schema v2, known devices), planner/executor (display-failure policy, hard timeouts), fakes behind the `testing` feature. No OS or UI code, `unsafe` forbidden. |
| `crates/*/tests/` | Integration tests (public API, one binary per file), Gherkin features and fixtures. Unit tests are inline `#[cfg(test)] mod tests` in the same file; no file or process I/O in unit tests (see docs/adr/0011-test-strategy-and-layout.md). |
| `crates/hostswitch-backends` | Wrappers for external tools (`m1ddc`, `hidapitester`), HID++ framing and parsers, `BackendRegistry`. Fixtures in `tests/fixtures`. |
| `crates/hostswitchctl` | `hostswitchctl` CLI (clap). |
| `examples/` | `config.example.json` (documented in README; checked by a test, keep it valid). |
| `openspec/` | OpenSpec: `config.yaml`, `specs/<capability>/spec.md` (current behaviour), `changes/<name>/` (proposals: proposal.md, delta specs, design.md, tasks.md). Validate with `OPENSPEC_TELEMETRY=0 openspec validate --all --strict`. |
| `docs/` | `adr/` (architecture decision records, each referencing OpenSpec requirements), `requirements.md` (constraints, risks, ID map), research. |
| `.agents/local/` | Local-only, git-ignored intermediate artefacts of the agents (reviews, findings, test and security reports, notes). Never commit them and never copy vulnerability details into tracked files. |
| `.agents/prompts/` | Reusable role prompts (researcher, developer, tester, reviewer). Tool-neutral. |
| `scripts/` | `worktree.sh` (sibling worktrees). |

Tool-specific files (`CLAUDE.md`, `.claude/`) only point here; never put project code or canonical instructions there.

## Branches and worktrees
The user's IDE (RustRover) stays open on the main clone, so the default is **no worktree**.
- Normal work: do it in the main clone on a short-lived branch (`git switch -c feature/<slug>` — only when the user asks to change branches), then fast-forward merge into `main`. `target/` is shared, so there is no rebuild cost.
- Use a worktree **only** when the user asks for parallel sessions (two agents or the user plus an agent working at once). Create it as a **sibling** of the main clone, never inside it (https://www.gitworktree.org/guides/best-practices): `scripts/worktree.sh add <slug>` -> `../<project>-<slug>` on branch `feature/<slug>`, one branch per worktree.
- Remove a worktree as soon as its branch is merged: `scripts/worktree.sh remove <slug>` (uses `git worktree remove`, then deletes the merged branch). Never `rm -rf` or `mv` a worktree; use `git worktree move`. After accidental deletion run `git worktree prune --dry-run`, then `git worktree prune`.
- Build output (`target/`) is per-worktree; rebuild after creating one.
- Tool-specific nested locations (e.g. `.claude/worktrees/`) are tolerated only when an agent app forces them; they are git-ignored and never hold the canonical copy of the project.

## Commands
- Spec/test chain: `OPENSPEC_TELEMETRY=0 openspec validate --all --strict` and `python3 scripts/check_traceability.py [--list-gaps]` (OpenSpec scenario -> Gherkin scenario -> Rust test). `cargo test --workspace` also runs the Gherkin end-to-end scenarios (`crates/hostswitchctl/tests/cli.rs`).
- Build: `cargo build` — Test: `cargo test --workspace` — Lint: `cargo clippy --workspace --all-targets` — Format: `cargo fmt` — CLI: `cargo run -p hostswitchctl -- --help` (never run `switch` without `--dry-run` during testing: it switches real hardware)

## Rules (mandatory)
1. **Docs are part of the change.** Whenever code, behavior, structure or decisions change, update the affected `*.md` files (requirements, ADRs, research, README, this file) in the same change. Keep requirement IDs stable.
2. **Git:** never commit without asking the user first and receiving an explicit yes (per commit). Commit with signing disabled: `git -c commit.gpgsign=false commit ...` — never use the user's signing key. Commit order: instructions/docs first, code second. Never push, rebase or change branches unless asked.
3. **Hardware safety:** never run commands that switch a real display input or Logitech slot (`m1ddc set input`, HID++ CHANGE_HOST writes, `hostswitchctl switch` without `--dry-run`). Read-only list/get only, unless the user explicitly asks for a live test.
4. External tools stay behind the `DisplayBackend` / `PeripheralBackend` traits; parsing lives in pure functions with fixture tests.
5. No `unwrap()`/`expect()` outside tests (clippy-enforced), no `unsafe` in `hostswitch-core`; every external call has a timeout. Code and CLI output are vendor-neutral: no hard-coded manufacturer names in identifiers, messages or output (`Peripheral*`, not a vendor prefix); a name may appear only where it is read from the hardware or stored in the user's config.
6. **Review:** every change (spec, ADR, design, test, prompt, document or code) is reviewed by at least the tester and the security engineer, and every change to code by the developer always (`.agents/prompts/README.md`: ownership and review rules; reports and findings stay local in `.agents/local/`). Only the owner edits an artifact: requirements engineer `openspec/`, architect `docs/adr/` and designs, developer code, tester Gherkin and test reports. Code runs system binaries and writes to hardware: treat inputs from config and from tool output as untrusted, never add `unsafe` to `hostswitch-core`, keep dependencies minimal.
7. Model economy: use small models (haiku) for routine tasks, mid (sonnet) for implementation/research, reserve the largest for architecture decisions. See `.agents/prompts/README.md`.
8. Distribution is build-from-source. No App Store, sandbox, signing or notarization requirements; keep dependencies Apache-2.0-compatible (no bundled GPL).
