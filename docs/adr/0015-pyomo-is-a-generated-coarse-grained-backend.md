---
id: ADR-0015
title: Adopt D12: one generated Pyomo adapter, no IDAES class hierarchy
status: superseded
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-41, DM-42, DM-43, DM-52]
blueprint: [§D12, §21.1, §21.2]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: ADR-0083
revisit: A Pyomo-ecosystem tool needs a per-node API the coarse-grained bundle cannot serve, or a native equivalent makes the adapter redundant
verification: `python / parity` bundle round-trip; `python / test` `test_forbidden_extra_keys` and `test_any_lint`

---

# ADR-0015: Adopt D12: one generated Pyomo adapter, no IDAES class hierarchy

## Context

Blueprint D12 has one adapter consume a `CanonicalMathProblem` bundle over the Arrow C stream interface and construct Pyomo objects, recreating no IDAES class hierarchy. The second review's R2-4 found the Python boundary specified by prose the named libraries do not enforce.

## Scope

Binds the adapter's shape and its purpose (parity and Pyomo-ecosystem tools until native equivalents exist). The units authority question is ADR-0026's.

## Drivers

Per-node calls across the boundary are the cost model that kills a parity run; an unenforced contract is a convention, not a contract.

## Options

Recreate IDAES's classes in Python — rejected: a second model authority. Row-oriented transfer — rejected: one stream per table is the coarse, typed unit charter DM-37 asks for.

## Outcome

One adapter, one bundle, one stream per table. Contract classes are generated; structuring is strict (`forbid_extra_keys`); an import-time lint fails on `Any`, bare `dict` or bare `list` fields; the numpy boundary refuses nullable columns.

### Consequences

The adapter is memoized and coarse-grained; every table crosses as `__arrow_c_stream__`, so pyarrow is a consumer rather than the contract.

### Compensating controls

The bundle carries a per-column loss profile saying which side enforces what (F16); pre-flight checks run before any `ConcreteModel` is constructed.

### Confirmation

`python / test` runs the boundary tests; `python / parity` runs the adapter against IDAES 2.12.0 in the solver container.

## Pros and cons

A coarse adapter cannot express incremental model edits; the platform's answer to an edit is a new case overlay (D13), not a mutation.

## More information

Blueprint §D12, §21.1 (extension module), §21.2 (adapter algorithm), §21.5 (Python contracts); review finding R2-4.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
- 2026-09-24 — superseded by ADR-0083.
