---
id: ADR-0017
title: Adopt D14: memoize on declared inputs, the pass version and the engine profile
status: superseded
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-31, DM-32, DM-33, DM-28]
blueprint: [§D14, §14.3, §14.4]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: ADR-0041
revisit: A measurement shows sub-pass granularity finer than per-instance is needed (the same trigger as ADR-0020), or an undeclared read is found in a pass
verification: `tests/engine` memo-key tests; the two-process byte-equality test for the plan fingerprint (ADR-0019)

---

# ADR-0017: Adopt D14: memoize on declared inputs, the pass version and the engine profile

## Context

Blueprint D14 keys memoization on the content hashes of a pass's declared inputs, the pass version and — for plan-executing passes — the engine profile. The first review's F13 asked whether `salsa` should own this, and the second review's R2-1 found the plan fingerprint not reproducible as specified.

## Scope

Binds the memo key's shape and the invalidation granularity. Whether `salsa` is adopted is ADR-0020; what the fingerprint is for is ADR-0019.

## Drivers

An undeclared read is a contract violation, not something to track; a value change must not invalidate structure; an engine upgrade must invalidate exactly the plan-executing passes.

## Options

Track dependencies automatically with `salsa` — deferred by ADR-0020: it duplicates the declared inputs. Key on file mtimes — rejected: not semantic (DM-32).

## Outcome

The memo key is `(pass_id, pass_version, content hashes of `cache_key_inputs`, engine profile hash for plan-executing passes)`. Backdating is inherent: byte-identical outputs leave the next pass's key unchanged.

### Consequences

Every pass must declare its inputs exhaustively, and `pass_specs.cache_key_inputs` becomes part of the reviewed surface of a pass.

### Compensating controls

The §14.4 invalidation table is explicit per change kind; the canonical graph never depends on a parameter value, so a parameter change invalidates exactly the case-bound rows.

### Confirmation

`tests/engine` asserts that a re-run on changed inputs producing identical bytes does not invalidate downstream passes.

## Pros and cons

Declared inputs are work; automatic tracking is a second dependency system beside `pass_specs`.

## More information

Blueprint §D14, §14.3 (pass engine), §14.4 (incrementality); review findings F13 and R2-1.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
- 2026-09-13 — superseded by ADR-0041.
