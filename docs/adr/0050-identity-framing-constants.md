---
id: ADR-0050
title: Freeze the identity framing constants, canonical frames and the phase-1 relation envelope
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-11, DM-15, DM-48]
blueprint: [§4.3, §5.1, §5.3, §14.3]
review: not-required: the constants make ADR-0045's and ADR-0007's accepted contracts implementable without changing them; the revision-5 review already carries the Accept verdict for the contracts themselves
evidence: Proposed
supersedes: []
superseded-by: null
revisit: any constant listed here has to change — that is a new version string and a superseding record, never an edit
verification: `crates/pse-ids/tests/golden_vectors.rs` (frozen hexadecimal vectors for every derive context, `derive_hash`, `snapshot_id` and `encoding_checksum`); the canonical metamorphic fixtures under `tests/conformance/tests/`

---

# ADR-0050: Freeze the identity framing constants, canonical frames and the phase-1 relation envelope

## Context

Blueprint §5.1 and §5.3 state the identity and canonicalization *contracts*; ADR-0045 separates logical content from encoded integrity and ADR-0023 gives `pse-ids` sole ownership of hashing. Several wave-1 packets (`pse-ids`, the canonicalizer, `pse-schema`, `pse-catalog`, `pse-mathir`, `pse-compiler`) have to produce the same bytes independently, and two of them are written in parallel. The constants those bytes depend on are stated in prose across three sections and nowhere as one list.

## Scope

Freezes the numeric and textual constants of the identity contract and nothing else. It adds no identity kind, changes no rule about *what* is hashed (§5.3 steps 1–3 are unchanged), and does not raise the envelope — R-23 owns that. Float handling is ADR-0030's and is cited, not restated.

## Drivers

Reproducibility: a hash is a contract between processes, so its framing has to be a written constant, not an implementation detail. Correctness: `derive_key` and a truncated plain hash produce different bytes, so the context strings are part of the wire format. Parallel implementation: two packets coding against the same frame need one authority for it.

## Options

Leave the constants in each implementation and compare with a golden test — rejected: the golden test would record whatever the first implementation did. Version each constant independently — rejected: the frame is one contract; a single version string per frame is what a reader can check. Defer the envelope to a runtime setting — rejected: an unbounded preflight cannot reserve before allocating (§14.3).

## Outcome

**Framing.** Every framed part is a u64 little-endian length followed by that many bytes. Fixed-width identities and hashes keep their declared width (16 and 32 bytes); a schema version is u32 little-endian. There is no `Debug` or display serialization anywhere on a hashing path.

**Derive contexts.** `blake3::Hasher::new_derive_key(context)` is the only construction. The frozen contexts are the six of §5.1 — `pse:named:v1`, `pse:symbol:v1`, `pse:equation:v1`, `pse:term:v1`, `pse:conn:v1`, `pse:node:v1` — plus `pse:registry:v1` (registry fingerprint), `pse:stage_key:v1` (the §14.3 memo key), `pse:settings:v1` (the semantic-settings hash) and `pse:mathir:node:v1` (structural and subtree hashes). A 128-bit identity is the **first 16 bytes of the extendable output**, a defined digest rather than a truncation.

**Registry identity bootstrap.** `REGISTRY_PACKAGE_ID = named_id(SemanticId::NIL, "pse.schema")`. Within it, a relation is named `relation:<ns>.<name>@<v>`, an enumeration `enum:<Name>`, an invariant `invariant:<relation>:<name>`, a pass `pass:<P>@<v>` and a rule `rule:<name>@<v>`.

**`pse.canon.v2` preimage.** `len‖"pse.canon.v2"`, then `relation_id`, then the schema version as u32 little-endian, then the registry fingerprint, then `len‖metadata stream` and `len‖data stream`. Each stream is a finished Arrow IPC stream written with `IpcWriteOptions::try_new(64, false, MetadataVersion::V5)` at its uncompressed default, carrying exactly one batch — including a zero-row batch for an empty relation. All padding bytes are zero.

**`pse.snapshot.v2` frame.** The version string, the registry fingerprint, the snapshot kind, a counted list of `(role, snapshot_id)` semantic parents sorted by role, then a counted list of `(port, namespace, relation_id, schema_version, logical_hash)` members sorted by the first four fields. Text sorts by UTF-8 bytes, identities by raw bytes, versions numerically. A duplicate parent role or member port is an error; required empty relations appear explicitly; a sidecar never enters its own membership.

**Envelope.** `Envelope::PHASE1` is 1,000,000 rows and 256 MiB of normalized value, offset and validity buffers per relation, and every Utf8/List offset must fit `i32`. A deployment may only tighten it. Preflight uses checked arithmetic and reserves before allocating; exceeding a bound is `runtime.resource_limit`, never a partial publication.

**Floats.** Unchanged from ADR-0030 and applied only to the hashing copy.

### Consequences

Changing any of these constants is a new version string (`pse.canon.v3`, `pse:named:v2`) and a superseding record, because every stored `logical_hash` and `snapshot_id` in every golden store is a function of them. The 64-byte IPC alignment and `MetadataVersion::V5` are part of the identity, so an Arrow upgrade that changes either is a format decision, not a pin bump.

### Compensating controls

`crates/pse-ids/tests/golden_vectors.rs` holds frozen hexadecimal vectors for every context and every frame, so a change to the framing is a failing test with a diff rather than a silently different store. The canonical metamorphic fixtures assert that identity-preserving transforms keep the hash and that distinguishing ones change it.

### Confirmation

`just test-package pse-ids -p pse-relations` runs the golden vectors; `just test-package pse-tests-conformance -p pse-relations` runs the metamorphic and round-trip fixtures. Both are named in `verification:`.

## Pros and cons

One list of constants is a single point of change and therefore a single point of failure; it is also the only way two independently written implementations can be checked against the same bytes. The alternative — constants distributed over prose — is what made `pse.canon.v1` unrepairable under its own name.

## More information

Blueprint §5.1 (identity forms and derived-id formulas), §5.3 (canonical serialization and hashing, steps 4–8), §4.3 (metadata conventions), §14.3 (the pass engine and stage keys); ADR-0007, ADR-0023, ADR-0030, ADR-0045, ADR-0046; register row R-23 (a larger envelope); [plan 03](../plans/03-wave-1-foundations.md) §I.1 and packets K-1 and B-canon.

## Status history

- 2026-09-13 — accepted. Evidence is `Proposed`: the constants are written, the implementation and its golden vectors land with packet K-1.
