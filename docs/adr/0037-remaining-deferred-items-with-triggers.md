---
id: ADR-0037
title: Defer datafusion-tracing, instrumented-object-store, datafusion-ffi and egglog, each with a trigger
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-58, DM-43, DM-40, DM-59]
blueprint: [§3.3, §14.2, §23.1, §22.1]
review: not-required: recorded in the second review's observations; the deferrals themselves are blueprint §3.3 positions, not findings
evidence: Proposed
supersedes: []
superseded-by: null
revisit: Any one of the four triggers fires: a `datafusion-tracing` release matching the pinned engine; an `instrumented-object-store` release matching it; a third-party compiled kernel package under §22; a cross-process, cross-version `egglog` extraction-determinism test passing
verification: Register rows R-03, R-04, R-06; `rust / family-check` (a tracing crate at 55.0.0 against a 55.1.0 engine fails it)

---

# ADR-0037: Defer datafusion-tracing, instrumented-object-store, datafusion-ffi and egglog, each with a trigger

## Context

Blueprint §3.3's "deferred with a stated trigger" list and §14.2 rule 8 defer four mechanisms. The second review observed that `datafusion-tracing` and `instrumented-object-store` top out at 55.0.0 against a 55.1.0 engine and should not be added yet.

## Scope

Binds four deferrals into one record so the register has one place to point. `salsa` (ADR-0020) and `LogicalPlan::Extension` (ADR-0025) have their own records because each carries its own design argument.

## Drivers

A dependency one patch behind the pinned engine would violate the family rule (ADR-0018); a rewrite engine whose extraction is nondeterministic across processes cannot be on a correctness path; an FFI extension surface with no third-party consumer is machinery for nothing.

## Options

Add `datafusion-tracing` now and pin around the family rule — rejected: the family rule exists precisely to prevent that. Adopt `egglog` as an optimization behind a flag — that *is* the position: optional, phase 4, never a correctness dependency.

## Outcome

None of the four is a dependency today. `datafusion-tracing` and `instrumented-object-store` enter when a release matching the pinned engine exists; `datafusion-ffi` enters only if §22's extension model admits compiled third-party kernel packages; `egglog` stays optional, phase 4, gated on cross-version extraction determinism, and its renderings are never hashed.

### Consequences

Observability in the meantime is `tracing` spans at pass and run boundaries (§23.1); DataFusion emits no spans of its own, which is a known gap rather than a hidden one.

### Compensating controls

Register rows R-03, R-04 and R-06 carry the triggers, owners and check commands, so each deferral is revisited monthly rather than forgotten.

### Confirmation

`rust / family-check` mechanically prevents the first two from being added at a mismatched version; hand-written canonicalization (§7.4) stays complete on its own so `egglog` can never become load-bearing.

## Pros and cons

Four deferrals in one record risks burying an argument; three register rows and two sibling ADRs keep each trigger individually visible.

## More information

Blueprint §3.3 (deferred with a stated trigger), §14.2 rule 8, §23.1 (observability), §22.1; the second review's observations; register rows R-03, R-04, R-06.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
