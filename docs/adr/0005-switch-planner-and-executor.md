# ADR-0005: Pure switch planner and a never-failing executor with per-step results

- Status: Accepted
- Date: 2026-10-18

## Context
A switch changes several things on real hardware. Losing the keyboard before the display follows would leave the user stuck, and a hung tool must not hang the program.

## Requirements
Satisfies or concerns (`spec:` = openspec/specs, `change:` = openspec/changes, `constraint:` = docs/requirements.md):
- spec:switching/F-01 — switch to a host profile
- spec:switching/F-01a — order of steps
- spec:switching/F-03 — several displays
- spec:switching/F-04 — optional parts
- spec:switching/N-04a — a failed display protects keyboard and mouse
- spec:switching/N-04b — every step is time-bounded
- spec:switching/N-04c — missing backend
- spec:switching/F-01c — result lines and exit status
- spec:offline-devices/F-13b — offline switching is retried once
- change:write-timeout-semantics/SEC-10 — a write with unknown outcome

## Decision
`planner::plan` turns a profile into ordered steps (displays, then keyboard, then mouse). `SwitchExecutor` runs them one after another and always returns a report with `Success`, `Failure` or `Skipped` per step; it never fails as a whole. A failed or timed-out display step makes every later keyboard and mouse step `Skipped`; other displays still run; a keyboard failure does not skip the mouse. Each step is bounded by `tokio::time::timeout`, which drops the future instead of awaiting it, so a backend blocked on a worker thread cannot delay the caller. A failed step is retried once (two attempts, 300 ms apart); `BackendUnavailable` is not retried; an offline device gets the one retry even if attempts are configured as 1. Device aliases are tried inside one attempt (ADR-0007).

## Consequences
- The user always gets a per-step result and an exit status (0, 2 for partial failure).
- The outcome of a write that runs into the timeout is being made explicit (change `write-timeout-semantics`).
- Display steps run sequentially; many displays take longer but stay predictable.

## Alternatives considered
- Run all steps in parallel: faster, but it removes the safety order.
- Abort on the first error: leaves a half-switched desk with no report for the remaining steps.
