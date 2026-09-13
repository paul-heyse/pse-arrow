---
id: ADR-0027
title: Define the commit contract as P2-level validity; compilability is gated separately
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-14, DM-29, DM-30, DM-08]
blueprint: [§22.2, §14.5]
review: not-required: stated as a revision-3 decision; neither review raised it as a finding
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A consumer is found treating a committed revision as compilable, or P2 grows a check expensive enough to make commits slow
verification: `tests/lifecycle` commit-contract tests; the closure-report tests in `tests/conformance`

---

# ADR-0027: Define the commit contract as P2-level validity; compilability is gated separately

## Context

Blueprint §22.2 makes a change set validated by P2 on the resulting snapshot, applied atomically, yielding a new model or case revision. Revision 3 states the commit contract explicitly because "valid" was previously ambiguous between structurally valid and compilable.

## Scope

Binds what a committed revision guarantees. It does not bind what a publishable compiled problem guarantees — that is the closure report (§14.5).

## Drivers

A commit that required full compilation would make authoring unusable; a commit that guaranteed nothing would make every consumer re-validate; charter DM-14 asks for explicit commit boundaries.

## Options

Require full P0-P10 validity to commit — rejected: an author could not save work in progress. Require nothing — rejected: every reader would need to re-check referential integrity.

## Outcome

A committed revision is structurally and referentially valid: every invariant of `reference.schema_invariants` holds, every target resolves to an identity, every expression parses. It may still fail P3-P10. Publication as a compiled problem is gated by the closure report, and every revision carries `last_closure_status`.

### Consequences

Change sets are the only write path into `authored`; an attempt to write a derived relation is rejected and an "expected" derived fact goes to `provenance.assertions`.

### Compensating controls

`last_closure_status` on every revision lets a reader tell a committed revision from a compilable one without running the compiler.

### Confirmation

`tests/lifecycle` commits a revision that fails P10 and asserts it commits, is readable, and is refused as a compiled problem.

## Pros and cons

Two gates are more concepts than one; one gate is either too slow or too weak.

## More information

Blueprint §22.2 (change-set model), §14.5 (closure proofs), §D13.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
