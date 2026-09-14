---
id: ADR-0019
title: Record the DataFusion plan fingerprint as evidence; keep it out of the memo key
status: superseded
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-48, DM-31, DM-15, DM-59]
blueprint: [§14.2, §14.3, §20.2]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: ADR-0044
revisit: `datafusion-proto` gains a documented byte-stability guarantee, or the two-process byte-equality test fails after a DataFusion upgrade
verification: `tests/engine` two-process plan-encoding byte-equality test; `rust / test`

---

# ADR-0019: Record the DataFusion plan fingerprint as evidence; keep it out of the memo key

## Context

Revision 2 put a `datafusion-proto` plan fingerprint into the rule passes' memo key. The second review's R2-1 measured six distinct encodings of one plan over a schema with five field-metadata keys, because `datafusion-proto` serializes metadata in map iteration order.

## Scope

Binds what the fingerprint is for and what it is not. The rest of the memo key is ADR-0017's subject.

## Drivers

The plan is a pure function of the rule plan, the catalog snapshot and the engine profile, all already in the key, so hashing it adds no information to reuse validity; an unstable key is a silent cache miss or, worse, a false hit across versions.

## Options

Keep the fingerprint in the key and canonicalize metadata — rejected: it still assumes cross-release protobuf stability the project cannot verify. Drop the fingerprint entirely — rejected: it is genuinely useful evidence in the pass record.

## Outcome

The fingerprint leaves the memo key and lives in `provenance.pass_records.plan_fingerprints` as evidence, defined as `pse.planfp.v1` and versioned like `pse.canon.v1`. Schema and field metadata are canonicalized at construction (§4.3) so the bytes are reproducible within a release.

### Consequences

Beside it the pass record stores the plan's `EXPLAIN` in `pgjson` and the ordered list of rules that actually fired, from the analyzer and optimizer observer closures — queryable structure, never identity.

### Compensating controls

The platform `LogicalExtensionCodec` encodes a snapshot table as `(snapshot_id, relation_id, version, content_hash)` and a kernel UDF by its `KernelSpec` digest rather than by name, so the bytes do not depend on in-process naming.

### Confirmation

A CI test encodes the same plan in two fresh processes and asserts byte equality; cross-release stability is explicitly not asserted.

## Pros and cons

Evidence that is not a key can drift without breaking correctness, which is exactly the property wanted here.

## More information

Blueprint §14.2 rule 5, §14.3 (pass engine), §20.2 (manifest); review finding R2-1; DataFusion capability map PROBE C (measured).

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
- 2026-09-13 — superseded by ADR-0044.
