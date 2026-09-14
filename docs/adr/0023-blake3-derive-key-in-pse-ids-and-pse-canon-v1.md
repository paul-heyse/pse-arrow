---
id: ADR-0023
title: Give pse-ids sole ownership of hashing, with derive_key contexts and pse.canon.v1
status: superseded
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-15, DM-11, DM-48, DM-40]
blueprint: [§5.1, §5.3, §20.1]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: ADR-0045
revisit: A canonicalization constant changes (alignment, metadata version, legacy flag, compression), which is a new `pse.canon.vN` and invalidates every stored hash
verification: `tests/governance/blake3_owner.rs`; the canonicalization property tests in `tests/conformance` (re-batching, metadata re-insertion order, dictionary re-encoding, Parquet round trip)

---

# ADR-0023: Give pse-ids sole ownership of hashing, with derive_key contexts and pse.canon.v1

## Context

Blueprint §5.3 defines content identity over the canonical IPC encoding only and versions the serializer as `pse.canon.v1`. The first review's F12 required the hashing contract to name its constants and its scope; §3.1 makes `pse-ids` the sole `blake3` dependant.

## Scope

Binds who may hash, what is hashed, and the four IPC constants that are part of the contract. Float canonicalization is ADR-0030; the plan fingerprint is ADR-0019 and is not a content hash.

## Drivers

A hash computed in two places will diverge; Parquet embeds `created_by`, so its bytes change on upgrade while the data does not; alignment alone changes IPC bytes (872 versus 1032 for identical rows).

## Options

Hash Parquet artifacts — rejected: not stable across writer versions. Let each crate hash what it owns — rejected: the canonicalizer, not the hash, is what tests cover, and it must exist once.

## Outcome

`pse-ids` is the only crate that depends on `blake3` and holds the canonical IPC serializer. Every 128-bit semantic ID uses a `derive_key` context. `pse.canon.v1` names alignment 64, `MetadataVersion::V5`, no legacy format and no compression; changing any of them is a new version.

### Consequences

Every artifact hash is tied to the IPC path; `SERDE_ARROW:*` or otherwise unregistered metadata keys are an error rather than stripped; nothing may hash a `Debug` rendering, because three libraries in the dependency set leak hash-container iteration order into theirs.

### Compensating controls

`blake3_owner` fails if any other crate declares `blake3`; `banned_patterns` greps for `{:?}` reaching a hash input in `pse-ids`.

### Confirmation

Canonicalization property tests assert that two logically identical relations split into different batches hash identically, and that a Parquet round trip does not change the hash.

## Pros and cons

One hashing crate is a bottleneck for changes; it is also the only way the contract can be tested in one place.

## More information

Blueprint §5.1 (identity), §5.3 (canonical serialization and hashing), §20.1 (artifact store); review finding F12.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
- 2026-09-13 — superseded by ADR-0045.
