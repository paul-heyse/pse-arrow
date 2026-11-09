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
verification: Original connected-result export and portable read preserve exact run/attempt/range/predicate bits and manifest/interpretation; absent or incomplete current context, noncanonical encoding, mixed interpretations and continued export cursors refuse; duplicate schema/nested-field metadata refuses before map conversion.
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
plus the nonempty problem/revision, closed-manifest digest and native interpretation. The
connected exporter requires a fresh reader at the start of that original selection; a suffix
cannot inherit the original range and usability. Writers refuse inherited reserved lineage
keys and other selected-result representation versions. Unrelated schema metadata remains intact.

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

The connected exporter and portable reader use the same current representation. Missing or
invalid v1 metadata and mixed interpretations refuse. The predecessor keys are recognized
only for refusal; they have no writer or reader. Wire representation version and original
native interpretation remain different meanings.

Arrow's metadata maps cannot retain duplicate-key evidence. The existing admitted IPC
preflight therefore checks schema and nested field metadata keys before `fb_to_schema` or
stream decoder conversion, for both file and stream consumers. Borrowed-key scratch stays
inside the existing admitted schema inventory; bounds and complete-message checks remain.

### Compensating controls

Check raw/text limits without decoding input. Keep the same protected native selection and
original scope; neither successful comparison nor decoded usability constructs permission.
Challenge duplicate keys, optional field presence, nonfinite/missing predicate semantics,
absent or incomplete original context, altered manifest/interpretation, mixed or absent
representation versions and suffix export independently.

### Confirmation

The maintained Buffa bounded encoder and current Arrow pre-map acquisition seam were inspected;
Context7 confirms the upstream encoding contract. Plan66 B04 and E02/E03 own current
representation, migration and functional evidence. This proposed decision makes no
qualification or measured benefit claim.

## Pros and cons

One generated observation removes repeated field interpretation and retains exact bits.
It introduces a versioned binary metadata key and a small base64 envelope; human-readable
inspection remains a separate generated presentation. Earlier metadata representations are
outside the current reader contract.

## More information

- [Plan66b](../plans/66b-contracts-generation-and-interpretation.md), [Plan66e](../plans/66e-protected-data-and-result-delivery.md)
- [Native selected-result owner](../../crates/pse-runtime/src/workflow/result_selection.rs)
- [Maintained bounded Buffa encoder](../../vendor/buffa/src/message.rs), [upstream Message encoding](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/message.rs)
- [base64 Engine API](https://docs.rs/base64/0.22.1/base64/engine/trait.Engine.html)
- [Existing IPC stream admission](../../crates/pse-columnar/src/ipc_stream.rs)

## Status history

- 2026-11-06 — proposed before the selected-result metadata cutover under authorized Plan65 execution.
- 2026-11-09 — reconciled the proposed contract with the current-only Plan66 cutover and original selection cursor.
