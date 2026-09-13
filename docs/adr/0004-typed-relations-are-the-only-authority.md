---
id: ADR-0004
title: Adopt D1: typed Arrow relations are the only model authority
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-01, DM-02, DM-06, DM-10]
blueprint: [§D1, §4.1, §6]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A relation family needs content no typed schema can express and a JSON column is proposed for it a second time
verification: `tests/governance/no_shadow_structs.rs`; `tests/governance/banned_patterns.rs`

---

# ADR-0004: Adopt D1: typed Arrow relations are the only model authority

## Context

Blueprint D1 forbids entity-attribute-value tables, JSON columns that carry required behaviour, and hand-written Rust structs that are the "real" model. The first design review's F8 found the blueprint violating its own rule: rule bodies, invariant specs and selectors were opaque JSON text.

## Scope

Binds every durable model fact to a declared relation in the schema registry (§4.1). It does not forbid JSON for genuinely open-ended evidence: `diagnostics_findings.values` and `provenance.assertions.expected` stay JSON by name.

## Drivers

One authority per fact; a rule change must be diffable at the level of meaning; agents must be able to query "which rules read this relation".

## Options

Keep rule bodies as JSON — rejected by F8: stratification cannot be checked as a relational invariant and impact analysis has to parse text. A Rust struct model with Arrow as an export — rejected by D1: it is two authorities.

## Outcome

Every durable fact lives in a typed relation declared in `pse-schema`. Rust structs that hold model data are generated views. `reference.rule_plan_nodes`/`rule_plan_edges`/`rule_dependencies`, `selector_terms` and typed `case_sets` generators replace the JSON that F8 found.

### Consequences

The registry becomes the single point of change for the model, and code generation (ADR-0031) becomes load-bearing. A new relation is a registry edit plus a regeneration commit, not a struct.

### Compensating controls

`no_shadow_structs` fails on a hand-written mirror of a relation; `banned_patterns` greps for the JSON escape hatches; P2 invariant `rules.stratified_negation` is itself expressed over `rule_dependencies`.

### Confirmation

`rust / test` runs the governance crate on every PR; a golden snapshot per vertical slice pins the relation content.

## Pros and cons

The cost is that everything mechanical must be generated. The benefit is the one the proposal claims: structure declared once removes five parallel maintenance surfaces.

## More information

Blueprint §D1, §4.1 (what is declared once), §6 (the relation families); review finding F8.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
