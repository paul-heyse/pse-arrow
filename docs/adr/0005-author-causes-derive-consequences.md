---
id: ADR-0005
title: Adopt D2: authors write primitive facts, passes derive everything else
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-13, DM-23, DM-46]
blueprint: [§D2, §14.1, §22.2]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A derived relation acquires an authored write path, or derivation rows exceed the §24.3 benchmark's budget per equation
verification: `tests/governance/banned_patterns.rs` (no write path into derived relations); the §24.3 derivation-cost benchmark

---

# ADR-0005: Adopt D2: authors write primitive facts, passes derive everything else

## Context

Blueprint D2 confines the authored model to primitive facts (topology, materials, quantities, phenomena, laws, options, specifications) and derives participation, roles, boundary classification, property requirements, method selection, equations and orderings by contracted passes. The first review's F14 questioned the cost of per-row derivations.

## Scope

Binds what may be authored and what must be derived, and requires every derived row to carry a derivation. It does not fix the derivation granularity: `schema_relations.derivation_granularity` declares `row` or `rule` per relation (§14.2 rule 4).

## Drivers

IDAES's largest maintenance burden is imperative `build()` methods that restate consequences; a derived fact that can also be authored is two authorities; provenance must survive the pipeline.

## Options

Author everything and validate — rejected: that is IDAES's position and the maintenance surface the proposal exists to remove. Store no derivations — rejected: negative completeness ("why is this row absent") becomes unanswerable.

## Outcome

An IDAES `build()` method becomes a template plus inference rules. A derived relation an author tries to write is rejected or downgraded to `provenance.assertions`. Every head row carries a derivation at the granularity its relation declares.

### Consequences

Authoring gets less expressive on purpose; escape hatches become change ops (§22.2) or assertions. Per-row derivations cost storage, which is why `rule`-granularity relations reconstruct theirs from the rule plus the §5.1 ID formula.

### Compensating controls

The §24.3 benchmark measures derivation rows and bytes per equation so the granularity assignment stays evidence-based rather than a guess.

### Confirmation

Golden snapshots per slice include the derivation relations; the closure report (§14.5) shows which categories are still open.

## Pros and cons

Deriving more means an author cannot express an exception locally; the compensating control is that exceptions become declared assertions the compiler can contradict.

## More information

Blueprint §D2, §14.1 (pass pipeline), §22.2 (change sets); review finding F14; ADR-0017 for what invalidation follows.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
