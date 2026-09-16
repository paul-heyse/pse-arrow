---
id: ADR-0013
title: Adopt D10: DataFusion has four roles and never runs inside a Newton iteration
status: superseded
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-38, DM-20, DM-43, DM-39]
blueprint: [§D10, §5.4, §14.2]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: ADR-0067
revisit: A fifth role is proposed for DataFusion, or `Exact` pushdown stops holding for the key shapes §5.4 depends on
verification: `tests/conformance` wrapper-provider test (re-applies every `Exact` filter and fails on a survivor); the §24.3 benchmarks

---

# ADR-0013: Adopt D10: DataFusion has four roles and never runs inside a Newton iteration

## Context

Blueprint D10 confines DataFusion to relational assembly and inference, the snapshot catalog, batch kernel evaluation and analytics. The second review's R2-7 measured the filter shapes the catalog provider actually receives and changed how §5.4 must be implemented.

## Scope

Binds where the engine may appear. The engine profile as a declared input is ADR-0019's subject; the memory pool is ADR-0029's.

## Drivers

A solve must not be a scalar function call; an engine version must not be able to change a numerical result; `Exact` pushdown causes the optimizer to delete the filter, so an untruthful provider silently returns excluded rows.

## Options

Use DataFusion for expression evaluation inside the solver — rejected: per-iteration planning cost and an engine-version dependency on the numerics. Avoid DataFusion entirely — considered in the first review's simpler alternative; rejected because set semantics, stratified negation and per-row provenance are the deliverable of P4-P6.

## Outcome

Four roles and no more. Key equality arrives as `rel.id = FixedSizeBinary(16, …)` and is matchable; `IN` lists arrive rewritten to `OR`; conjunctions are split; the provider claims `Exact` only for the shapes it actually matches.

### Consequences

The catalog provider must be conservative: `Inexact` is always safe, `Exact` is a guarantee the engine does not verify.

### Compensating controls

A test-only wrapper provider re-applies every filter the real provider claimed `Exact` and fails on a surviving row.

### Confirmation

The wrapper-provider test runs over golden snapshots in `rust / test`; §24.3 benchmarks the pushdown path end to end.

## Pros and cons

Four roles is a discipline, not a mechanism; the wrapper provider is what makes it checkable.

## More information

Blueprint §D10, §5.4 (snapshot catalog), §14.2 (rule compiler); review finding R2-7.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
- 2026-09-14 — superseded by ADR-0067.
