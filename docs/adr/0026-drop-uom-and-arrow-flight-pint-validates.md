---
id: ADR-0026
title: Drop uom and arrow-flight; pint validates what the adapter emits and never defines a quantity
status: superseded
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-42, DM-58, DM-43]
blueprint: [§3.1, §3.3, §21.2]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: ADR-0067
revisit: A kernel genuinely needs compile-time unit typing, or a remote Arrow transport becomes a requirement
verification: `rust / deny` (`deny.toml` bans `uom` and `arrow-flight`); `python / lint` banned-api rule on `assert_units_consistent`; `tests/governance/tests/banned_patterns.rs`

---

# ADR-0026: Drop uom and arrow-flight; pint validates what the adapter emits and never defines a quantity

## Context

Blueprint §3.3's "explicitly not added" list drops `uom` and `arrow-flight`, and the second review's R2-4 found §3.3 and §21.2 contradicting each other about whether a Python units library is an authority.

## Scope

Binds the units authority and removes two dependencies with no consumer. The quantity-type contract itself is ADR-0008.

## Drivers

One authority per fact (DM-02): the registry defines quantities and the unit set; `uom`'s stock SI system cannot express the currency dimension costing needs; `arrow-flight` has no consumer, and a dependency with no consumer is a supply-chain surface for nothing.

## Options

Compile-time unit typing in kernels with `uom` — rejected: `KernelSpec` signatures are checked by P10's unit inference and by the per-kernel parity suites, and the currency dimension does not fit. Keep `arrow-flight` for a future service — rejected: add it when a service exists.

## Outcome

`uom` and `arrow-flight` are banned in `deny.toml`. `pint` is present only because Pyomo's units machinery is pint: it validates what the adapter emits and never defines a quantity, and no relation is ever derived from a pint object.

### Consequences

The adapter calls `pyomo.util.check_units.identify_inconsistent_units` — not a bare `assert_units_consistent` — so a disagreement names the offending components in a `compile.math` finding; if the two disagree, the adapter has a bug.

### Compensating controls

A ruff `banned-api` rule rejects `assert_units_consistent` in the Python package; `deny.toml` carries both bans with reasons and review dates.

### Confirmation

`python / lint` and `rust / deny` are required checks; the unit-inference tests in `tests/conformance` are the positive side.

## Pros and cons

Dropping `uom` means unit errors are caught by inference rather than by the type system; the currency dimension makes the type-system route unavailable anyway.

## More information

Blueprint §3.1, §3.3 ("explicitly not added"), §21.2 (adapter algorithm step 1); review findings R2-4 and R2-5.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
- 2026-09-14 — superseded by ADR-0067.
