# ADR-0001: Record architecture decisions

- Status: Accepted
- Date: 2026-10-18

## Context
Design knowledge lived in one `architecture.md` that mixed structure, decisions and plans. It did not say *why* something was built that way and could not be traced to the requirements. Requirements are now OpenSpec specs (`openspec/`), which answer *what* the system must do.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- constraint:N-09 — test descriptions and specs are traced; decisions join that chain

## Decision
Architecture is recorded as ADRs in `docs/adr/NNNN-slug.md` (one decision each, numbered, never renumbered). Sections: Status (`Proposed`, `Accepted`, `Superseded by ADR-NNNN`), Date, Context, **Requirements**, Decision, Consequences, Alternatives considered. The *Requirements* section lists what the decision satisfies or concerns, one per line, as `spec:<capability>/<ID>`, `change:<name>[/<ID>]` or `constraint:<ID>`. An accepted ADR is not rewritten: a new ADR supersedes it. Per-change design stays in `openspec/changes/<name>/design.md`; when a change is archived, decisions with lasting effect become ADRs. `docs/adr/README.md` is the index. `scripts/check_traceability.py` verifies the format and that every referenced requirement exists.

## Consequences
- The chain is requirement (OpenSpec) -> ADR -> code and tests (Gherkin `@test` tags).
- ADRs are short, so the cost of recording a decision is low; the index must be kept in step (the script checks it).
- No single overview document: the structure is the set of ADRs, ADR-0003 holds the crate layout.

## Alternatives considered
- One `architecture.md`: easy to read, but loses the reasons and the trace to requirements (replaced).
- Design only in per-change `design.md`: good for a change, but no lasting overview after archiving.
