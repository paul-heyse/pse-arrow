---
id: ADR-0250
title: Retain selected-result metadata through its source-owned observation
status: proposed
date: 2026-11-06
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-05, DP-01, DP-02, PS-12]
blueprint: [§19.2, §20.5]
review: "not-required: bounded selected-result metadata projection within authorized Plan65 A06/C02/F04; existing native reader and protection authority remain unchanged"
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A selected consumer needs a different retained interpretation, larger observation, semantic rather than exact serialization comparison, or native adoption from this passive record.
verification: Original connected-result export and portable read preserve exact run/attempt/range/predicate bits and manifest/interpretation; changed context, noncanonical encoding, mixed interpretations and duplicate schema/nested-field metadata refuse before map conversion; historical absent-v1 artifacts retain their explicit reader.
---

# ADR-0250: Retain selected-result metadata through its source-owned observation

## Context

The protected result reader exports selected Arrow data with a handwritten lineage-key map.
Portable inspection repeats that field correspondence. Plan65 C02 needs exact selected intent
and original interpretation, while A06 now supplies an output-only generated observation
from the actual `PackResultSelection` owner. A generated type without a production consumer
would leave that duplication intact.

## Scope

Amend blueprint §19.2/§20.5 for new connected-result Arrow exports and their offline portable
reader. Canonical relations, scientific permission, semantic hashing and protected-read
admission retain their owners. This is a versioned observational metadata representation.

## Drivers

One native declaration must supply producer and reader correspondence. Preserve exact
predicate endpoint bits, optional presence, original row coordinates and physical/native
interpretation without requiring an offline scientific runtime or a second query authority.
Small local IPC counterexamples must exercise refusal before native metadata normalization.

## Options

Retaining parallel string keys preserves a second field map. A larger synthetic inspection
document adds unrelated presentation responsibilities. Generated ProtoJSON is useful for
human inspection, but this machine-fidelity operation would need additional JSON naming,
default and numeric normalization rules. Select one bounded generated binary observation
encoded as an Arrow metadata string, with exact expected-byte comparison at this version.
No general Protobuf canonical-serialization guarantee is assumed.

## Outcome

New exports write `pse.canonical.selection.proto.v1`: standard padded RFC4648 base64 of the
generated `RecordedResultSelection` message. The native source owner includes exact original
run, attempt, relation/version, row range, optional field/predicate and recorded usability,
plus problem/revision, closed-manifest digest and native interpretation. New writers remove
their reserved obsolete lineage keys before emitting this single record. Unrelated schema
metadata remains intact.

Use maintained Buffa 0.9.2 `Message::try_encode_bounded(65_536, &mut sink)`, which performs
checked size/cache traversal and rejects the byte budget before writing. The existing
base64 0.22.1 `Engine` with `engine::general_purpose::STANDARD` encodes only that bounded
native output; the maximum encoded text is 87,384 bytes. The offline reader compares input
text with the bounded generated encoding of its declared original selection. It does not
decode untrusted bytes into a native owner. Duplicate singular fields, unknown/trailing bytes,
alternate base64 or wire encodings, changed context and changed endpoint bits cannot match.
This establishes local serialization fidelity and artifact/manifest agreement, not producer
authenticity, scientific usability or transferable read permission.

### Consequences

The real connected exporter and portable reader migrate together. Present-invalid v1 metadata
never falls back to the historical interpretation. Mixed v1 and reserved predecessor lineage
keys refuse as ambiguous. Only artifacts without the v1 key use the explicit historical
reader where retained-artifact promises require it; predecessor keys have no new writer.
Wire representation version and original native interpretation remain different meanings.

Arrow's metadata maps cannot retain duplicate-key evidence. The existing admitted IPC
preflight therefore checks schema and nested field metadata keys before `fb_to_schema` or
stream decoder conversion, for both file and stream consumers. Borrowed-key scratch stays
inside the existing admitted schema inventory; bounds and complete-message checks remain.

### Compensating controls

Check raw/text limits without decoding input. Keep the same protected native selection and
original scope; neither successful comparison nor decoded usability constructs permission.
Challenge duplicate keys, absent/empty optional values, nonfinite/missing predicate semantics,
altered manifest/interpretation, mixed versions and a historical artifact independently.

### Confirmation

The maintained Buffa bounded encoder and current Arrow pre-map acquisition seam were inspected;
Context7 confirms the upstream encoding contract. Plan65a/C02/F04 own generation, migration
and actual functional evidence. This proposed decision precedes consumer implementation and
does not close those packages or claim measured benefit.

## Pros and cons

One generated observation removes repeated field interpretation and retains exact bits.
It introduces a versioned binary metadata key and a small base64 envelope; human-readable
inspection remains a separate generated presentation. Historical reading remains explicit.

## More information

- [Plan65c](../plans/65c-storage-projections-and-history.md), [Plan65f](../plans/65f-diagnostics-and-result-delivery.md)
- [Native selected-result owner](../../crates/pse-runtime/src/workflow/result_selection.rs)
- [Maintained bounded Buffa encoder](../../vendor/buffa/src/message.rs), [upstream Message encoding](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/message.rs)
- [base64 Engine API](https://docs.rs/base64/0.22.1/base64/engine/trait.Engine.html)
- [Existing IPC stream admission](../../crates/pse-columnar/src/ipc_stream.rs)

## Status history

- 2026-11-06 — proposed before the selected-result metadata cutover under authorized Plan65 execution.
