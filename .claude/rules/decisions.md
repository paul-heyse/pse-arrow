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

## ADR front matter (charter §H)

`id`, `title`, `status` (`proposed` | `accepted` | `rejected` | `deprecated` |
`superseded`), `date`, `deciders`, `level` (`decision` | `should-deviation` |
`must-gap`), `principles` (`DM-xx`), `blueprint` (the sections governed), `review`
(`path#finding`, or `not-required: <reason>`), `evidence` (a §D label),
`supersedes`/`superseded-by`, `revisit` (an *observable* trigger, not a date alone),
`verification` (the test, lint or benchmark that shows the decision still holds).

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
`docs/authoritative_design/blueprint.md` with a revision row and an inline
`> Decision: ADR-NNNN` at the governed section. A PR may merge with `status: proposed`
only if it also carries `needs-review`.

## When an ADR is required

| Change | Needs |
|---|---|
| Alters D1–D14; adds or removes a crate; adds, drops or majors a dependency family; changes the hashing contract, the Python boundary contract, metadata conventions or the commit contract; any SHOULD deviation; governance changes | ADR **and** design review (`needs-review`; Accept or Accept-scoped before `accepted`) |
| New relation family, pass, kernel contract or backend binding within an accepted decision; a small local SHOULD deviation; moving the parity pin; a deferred trigger firing | ADR (short); review at maintainer discretion |
| Bug fixes, refactors within contracts, tests, docs wording, patch bumps inside a pinned family, tooling | Neither; an ordinary PR with the evidence field filled |

## Plans and the register

`just plan <slug>` creates `docs/plans/NN-<slug>.md`. Front matter lists the ADRs the
plan implements. When the work lands, append `## Outcome` with what was built, **a
mistake made and corrected**, and **deviations that were deliberate**. Both of those
sections are the point of the outcome; omitting them makes the plan a memo.

`docs/adr/register.md` holds every deferred decision with its trigger, its check, an
owner and a next-review date. A decision deferred without an observable trigger is a
decision forgotten. `just register-check` runs the rows that are due.
