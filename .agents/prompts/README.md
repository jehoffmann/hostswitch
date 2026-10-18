# Agent role prompts
Shared rules for every role (prepend to each prompt when briefing a subagent):
- Read AGENTS.md, the relevant OpenSpec specs (`openspec/specs/`, and the open change in `openspec/changes/`; docs/requirements.md holds constraints, risks and the legacy-ID map) and the ADRs in docs/adr/ first. Every role may read and use every artifact (specs, ADRs, code, tests, reviews, research) as input as it needs. They are the source of truth; flag conflicts instead of silently deviating.
- Do NOT run `git commit`, `git add`, `git push`, or change branches. Leave changes uncommitted (the user approves each commit).
- Do NOT execute any command that switches a display input or a Logitech Easy-Switch slot on real hardware (m1ddc `set input`, HID++ CHANGE_HOST writes, `hostswitchctl switch` without `--dry-run`). Read-only enumeration/get commands only. Only channel switching is supported; never add or run any other hardware command.
- **Review is mandatory.** Every change, whoever owns the artifact (spec, ADR, design, test, prompt, document or code), is reviewed by at least the **tester** and the **security engineer** before it counts as done. Every change to **code** by the developer is always reviewed by both (diff, tests, verdict). Each reviewer answers with a written verdict (OK / CHANGES) and concrete findings; the owner fixes them or accepts them explicitly with a reason. Additionally the architect checks conformity with the design and the ADRs, and the requirements engineer conformity with the specs, whenever a change touches structure or behaviour. Reviewers never edit the artifact under review.
- **Edit only what you own** (table below). For anything else, propose the change to its owner. Report: files changed, what was verified (with command output summary), open issues.

## Formats
- **Requirements: OpenSpec.** `openspec/specs/<capability>/spec.md` is the current behaviour (`### Requirement:` with SHALL, `#### Scenario:` with GIVEN/WHEN/THEN); every change is a proposal in `openspec/changes/<name>/` (proposal.md, delta specs with ADDED/MODIFIED/REMOVED Requirements, design.md, tasks.md) that is reviewed, implemented, then archived. Owner: requirements engineer (design.md/tasks.md: architect).
- **Test descriptions: Gherkin.** `.feature` files (Feature/Scenario/Given/When/Then, tags `@unit @integration @system @hardware @req-<ID> @security`) in `crates/<crate>/tests/features/` and `tests/features/`; each OpenSpec scenario is covered by at least one Gherkin scenario, each Gherkin scenario by a Rust test. Owner: tester.
- **Source files:** SPDX header `Apache-2.0` first line; tests apart from functional code (see docs/adr/0011-test-strategy-and-layout.md).

## Intermediate artefacts stay local
Reviews, findings, verdict notes, test and security reports and scratch output are written to `.agents/local/`, which git ignores. They are never committed, and vulnerability details (exploit steps, locations of unfixed flaws) never go into tracked files: a fix is tracked as a neutral change proposal or code change. Only the owners' deliverables (specs, ADRs, code, tests, Gherkin, documents) are versioned.

## Ownership
| Artifact | Owner (the only role that edits it) |
|----------|-------------------------------------|
| OpenSpec: `openspec/specs/`, `openspec/changes/*/proposal.md` and delta specs; `docs/requirements.md` | requirements engineer |
| ADRs in `docs/adr/`; `design.md` and `tasks.md` of a change (the developer only ticks finished tasks) | architect |
| Source code, Cargo manifests, scripts, inline unit tests | developer |
| Gherkin `.feature` files, integration and system tests (`tests/`, `crates/hostswitchctl/tests/cli.rs`), `scripts/check_traceability.py` | tester |
| Local-only reports under `.agents/local/` (test report, security review, notes; git-ignored) | the role that writes them (tester, security engineer, ...) |
| `docs/research/` | hardware researcher |
| Project instructions: `AGENTS.md`, `README.md`, `.agents/` (these role prompts) | the orchestrator, on the user's request |

Everyone else reads these artifacts and proposes changes to the owner. A change to an artifact is reviewed as described under "Review is mandatory"; where the tester or the security engineer owns the artifact, the architect takes that reviewer's place.

## Briefing a subagent
Give it: (1) its role file (`.agents/prompts/<role>.md`) and AGENTS.md, (2) the exact scope (change name, files/crates), (3) the allowed and forbidden commands (above), (4) the deliverable and where it is written (the role's owned path) plus a concise final report, (5) the model from the table below. Subagents report to the orchestrator; they never commit and never answer for the user. Review roles are read-only on production code. Ask the user before anything that changes requirements or tests of existing behaviour.

## Roles
| File | Role | Model | Owns |
|------|------|-------|------|
| requirements-engineer.md | Requirements in OpenSpec: elicits, writes specs/changes, traceability | sonnet | openspec/specs, openspec/changes (proposal, delta specs), docs/requirements.md until migrated |
| architect.md | Architect: design.md/tasks.md per change, ADRs, structure | opus | docs/adr/, design.md and tasks.md of changes |
| developer.md | Developer: working agreement plus area notes (core, backends, CLI; later the tray app) | sonnet | code in the assigned crates (`crates/*`, `scripts/`) |
| tester.md | Tester: Gherkin test descriptions; unit, integration and system tests; reviews plans and steps | haiku (routine) or sonnet (review) | `.feature` files, test additions, local test report |
| security-engineer.md | Security engineer: reviews plans, changes and code; read-only | sonnet (per-step feedback), opus (full reviews) | local security review (`.agents/local/`) |
| researcher-hardware.md | Hardware/protocol researcher | sonnet | docs/research/ |

## Workflow
1. **Requirements engineer** writes the OpenSpec change: proposal.md and delta specs with scenarios (including failure and abuse cases).
2. **Architect** writes design.md and tasks.md for the change.
3. **Security engineer, tester and requirements engineer** review the change and design; the tester writes the Gherkin scenarios; the architect resolves findings.
4. **Developer** implements one task at a time, ticking `tasks.md`.
5. **Tester and security engineer** review each finished task (diff, tests, verdict); the developer resolves findings.
6. **Requirements engineer and architect** confirm the result; the change is archived so the delta merges into `openspec/specs/`.
7. The **user** approves each commit (docs/specs first, then code).

Only channel switching is supported (display input, Easy-Switch host); every other hardware command needs a requirement change first.
