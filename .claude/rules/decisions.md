---
description: ADRs, plans, and the decision process
paths:
  - "docs/adr/**"
  - "docs/plans/**"
---

# Decisions and plans

Two directories, two lifecycles. **ADRs record a decision** and are immutable after
acceptance. **Plans record how work is sequenced and verified** and stay living until
done. When the code and a plan disagree, the code is what runs.

## ADR front matter (design principles §H)

`id`, `title`, `status` (`proposed` | `accepted` | `rejected` | `deprecated` |
`superseded`), `date`, `deciders`, `level` (`decision` | `should-deviation` |
`must-gap`), `principles` (`AP-xx` foundations, `DP-xx` refinements, `PS-xx` process-simulator profile; accepted records keep legacy `DM-xx`), `blueprint` (the sections governed), `review`
(`path#finding`, or `not-required: <reason>`), `evidence` (a §D label),
`supersedes`/`superseded-by`, `revisit` (an *observable* trigger, not a date alone),
`verification` (the named scenario/property and analysis, test, lint or measurement that
settles it). Optional `standard` and `scenarios` fields snapshot reviewed versions and link
scenario definitions; they do not copy current status or make old ADRs invalid.

`level: must-gap` narrows scope. It never claims compliance.

Body sections: Context · Scope · Drivers · Options · Outcome (Consequences, Compensating
controls, Confirmation) · Pros and cons · More information · Status history
(append-only). A small deviation fills each section in one line — that is fine; an empty
section is not.

## Immutability

An accepted ADR changes only in its status fields. A decision that turned out wrong is
**superseded**, not edited: `just adr-supersede <old> <new>` writes symmetric links and a
status-history entry. `just adr-lint` checks this against `origin/main`, so an edit in
place is a red build, and a `PreToolUse` hook blocks it before that.

## The decision-PR rule

An ADR enters or changes status only in a PR labeled `adr` and titled
`adr: ADR-NNNN <title>`. The same PR — or a named follow-up `design:` PR — amends
the authoritative collection with a revision row in `docs/authoritative_design/blueprint.md`
and an inline `> Decision: ADR-NNNN` at the governed section's owner. Section identities
remain stable across `blueprint.md` and `sections/`. A PR may merge with `status: proposed`
only if it also carries `needs-review`.

## When an ADR is required

| Change | Needs |
|---|---|
| Alters D1–D14; adds or removes a crate; majors one of the four pinned families (arrow, datafusion, object_store, pyo3); changes the hashing contract, the Python boundary contract, metadata conventions or the commit contract; any SHOULD deviation; governance changes | ADR **and** design review (`needs-review`; Accept or Accept-scoped before `accepted`) |
| New relation family, pass, kernel contract or backend binding within an accepted decision; a small local SHOULD deviation; moving the parity pin; a deferred trigger firing | ADR (short); review at maintainer discretion |
| Bug fixes, refactors within contracts, tests, docs wording, patch bumps inside a pinned family, adding, removing or upgrading a third-party dependency, tooling | Neither; an ordinary PR with the evidence field filled |

## Plans and the register

`just plan <slug>` creates `docs/plans/NN-<slug>.md`. Front matter lists the ADRs the
plan implements. When the work lands, append `## Outcome` with what was built, **a
mistake made and corrected**, and **deviations that were deliberate**. Both of those
sections are the point of the outcome; omitting them makes the plan a memo.

### Architecture and follow-up ownership

Use the selected review standard's six foundations and scenario method for architectural
changes. Preserve architectural fitness and behavioral adequacy as separate judgments.
Packets name the responsibility that absorbs each change, the consumed contract, the relevant
scenario acceptance, and any replacement/deletion. A focused change need not invent scenarios
or mechanisms outside its scope.

The owning plan's disposition table is the current state of adopted findings:
`finding reference | scenario reference | disposition | decision/work owner | evidence or
revisit trigger`. Use open, scheduled, resolved, deferred, superseded or disproved with the
meaning in the review template. Resolved requires the correction's evidence; scheduling a
packet or accepting an ADR is insufficient. Deferred work has an owner and observable trigger.
Link packet-owned execution evidence. Keep historical review observations and accepted ADR
support at decision time intact. Indexes link current status instead of copying it.

New plans may use `review_sources` and `scenario_sources` inline lists in front matter for
navigation. References identify existing definitions; no second scenario registry is required.
The plan has one overall lifecycle status; its packet table owns packet progress. An execution
packet may instead own that packet's progress, in which case the plan links it rather than
maintaining a second status value.

### Execution rhythm in plans

Structure every plan the way AGENTS.md *Execution rhythm* runs it:

- **Packets** name targeted unit tests as their acceptance, plus the deletion of whatever
  they replace. A packet is not done while the replaced mechanism, its callers or its
  tests remain — delete them as soon as the replacement is proven by its tests. No
  compatibility paths, no tests ported onto a deleted mechanism.
- **Comprehensive qualification is separately requested by the maintainer.** A plan may
  describe relevant integration, component, solver, Python, performance and static
  checks, but their execution is not a plan-close, commit, push or merge gate. Report
  evidence for checks actually run. Documentation/tooling changes do not acquire native
  compilation or source-proof requirements.
- **Checkpoints** record current state, decisions made and the next dependency-ordered
  steps. They do not record per-command receipts, numbered rerun logs or static checks
  rerun after documentation edits. Failed-run detail is kept only while it drives a repair.

`docs/adr/register.md` holds every deferred decision with its trigger, its check, an
owner and a next-review date. A decision deferred without an observable trigger is a
decision forgotten. `just register-check` runs the rows that are due.
