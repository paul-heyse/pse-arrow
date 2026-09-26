---
id: ADR-0030
title: Canonicalize NaN and preserve -0.0 on the hashing path; never use Float64 as a distinct or join key
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-40, DM-42, DM-08, DM-15]
blueprint: [§5.3, §14.2, §19.2]
review: git:8950dd3d6ddb:docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: DataFusion changes its grouping or join equality for floats, or a rule genuinely needs a float-valued key
verification: `tests/engine` rule-compiler rejection test (`rule.float_key`); the canonicalization property tests in `tests/conformance`

---

# ADR-0030: Canonicalize NaN and preserve -0.0 on the hashing path; never use Float64 as a distinct or join key

## Context

Blueprint §5.3 step 5 hashes floats as IEEE-754 bits after mapping every NaN to the quiet NaN with zero payload and positive sign, preserving `-0.0`. The second review's R2-2 measured that the preservation claim fails under `GROUP BY`, `DISTINCT` and hash joins, which merge `-0.0` with `+0.0` and treat NaN as self-equal.

## Scope

Binds float canonicalization on the hashing path and the key discipline in rule plans. It does not change what reaches a solver: a NaN's provenance matters for diagnostics (§18.2).

## Drivers

An unqualified "`-0.0` is preserved" is a claim the engine contradicts (charter DM-59); a float join key silently merges rows a reader believes are distinct.

## Options

Canonicalize `-0.0` to `+0.0` too — rejected: the sign is meaningful in Arrow's `totalOrder` and in stored data. Allow float keys with a tolerance — rejected: tolerance is a physical decision, not a key semantics decision.

## Outcome

`canonical_f64_bits` maps every NaN to the quiet NaN with zero payload and positive sign and preserves `-0.0`, applied on the hashing path only. Preservation holds for the content hash and for Arrow ordering (`null, -1, -0, 0, NaN`, nulls first); it does **not** hold under engine grouping or joining, and §5.3 now says so. Rule plans never use a `Float64` column as a distinct or join key.

### Consequences

Analytics that group on floats collapse `-0.0` with `+0.0`; §19.2 records the collapse rather than hiding it. Null-bearing key comparisons use `distinct`/`not_distinct` semantics and Kleene booleans.

### Compensating controls

The rule compiler rejects a violating `rule_plan_nodes` row with `rule.float_key`, so the discipline is enforced at compile time rather than reviewed by eye.

### Confirmation

`tests/engine` asserts the rejection; the canonicalization property tests assert the hash is invariant under re-batching and stable across NaN payloads.

## Pros and cons

Refusing float keys costs expressiveness in rules; the measured engine behaviour is why it costs less than allowing them.

## More information

Blueprint §5.3 step 5, §14.2 rule 7, §19.2; review finding R2-2; DataFusion capability map PROBE D (measured).

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
