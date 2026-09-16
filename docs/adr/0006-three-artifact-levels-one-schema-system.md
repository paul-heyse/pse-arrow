---
id: ADR-0006
title: Adopt D3: one catalog holds CanonicalModel, CanonicalMathGraph and CanonicalMathProblem
status: superseded
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-13, DM-21, DM-23]
blueprint: [§D3, §5, §14.1]
review: not-required: neither review raised a finding against D3
evidence: Proposed
supersedes: []
superseded-by: ADR-0068
revisit: A fourth artifact level is proposed, or a level needs identity, hashing or provenance conventions the other two do not share
verification: `tests/governance/tests/every_crate_registered.rs`; the lifecycle test layer in `tests/lifecycle`

---

# ADR-0006: Adopt D3: one catalog holds CanonicalModel, CanonicalMathGraph and CanonicalMathProblem

## Context

Blueprint D3 states that the three artifact levels are stages in one catalog sharing identity conventions, extension types, hashing and provenance, not three applications.

## Scope

Binds the artifact levels to one schema system and one catalog. It does not bind the pass boundaries between them (§14.1 does).

## Drivers

Three catalogs would mean three identity schemes and three hashing contracts; the reproducibility contract (§20) needs one manifest shape.

## Options

Separate stores per level — rejected: the snapshot hash of §5.3 step 7 could not span them. A single flat namespace — rejected: the seven catalog namespaces are what makes authority legible.

## Outcome

One catalog, seven namespaces, one identity and hashing contract (ADR-0023), one manifest (§20.2).

### Consequences

A change to the hashing contract or to the extension-type set touches all three levels at once, which is the intent.

### Compensating controls

The snapshot catalog (§5.4) is the only reader of artifact bytes; `tests/lifecycle` exercises publish, read back and query for each level.

### Confirmation

`docs / build` publishes the relation index (Appendix B) so the level of every relation is visible.

## Pros and cons

Coupling the levels is the point: the alternative is three products.

## More information

Blueprint §D3, §5 (identity and catalog), §14.1 (pass pipeline).

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
- 2026-09-15 — superseded by ADR-0068.
