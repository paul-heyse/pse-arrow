---
title: "Plan61d: Schema, payload and contract investigation"
status: active
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/61-code-intelligence-foundations.md
doc_topics: [current-work, design, libraries, diagnostics]
review_sources: [docs/functional_reviews/code-intelligence-protobuf-foundations-2026-10-31.md]
adrs: []
---

# Schema, payload and contract investigation

## Target and foundation assessment

**Proposed:** implement O05/O07/O09/O10/O11 as useful existing search/inspection workflows,
not merely expose the CLI's own schema over another endpoint. The agent can follow service→method→
input/output→field/type/presence/options, inspect an explicitly selected payload and compare a
contract change with its generated Rust origin. [Plan61](61-code-intelligence-foundations.md)
owns dispositions. Existing source/history/generated-origin/API qualification owners remain
authoritative; descriptor and Rust relationships must not be conflated.

Use A02's immutable complete descriptor artifact and B01's direct projection slice. Buffa's
shared pool/static VTable reflection serves known generated values; DynamicMessage serves
explicitly acquired runtime-selected schemas. Original binary, descriptor closure and source
information remain identifiable. SourceCodeInfo is not an exact byte-range guarantee; validate
coordinate conversion against original source before offering a source-linked action.

The [reflection/JSON](../libraries/proto/prost_reflection_and_pbjson.md),
[Buffa](../libraries/proto/Buffa.md), [compilation](../libraries/proto/protobuf_compilation_and_orchestration_protox_protoc_and_buf.md)
and [generator](../libraries/proto/protobuf_generator_ecosystem_rust.md) documents describe
distinct representations. pbjson Any/FieldMask qualifications and PUBG integer/presence options
are reasons to test actual chosen mappings, not reasons to adopt another JSON stack by default.

## Packages

<a id="d01"></a>
### D01 — Offline descriptor acquisition and schema navigation

Consumes A02 and B01 projection. Extend existing query/inspect entry points with descriptor-
qualified message, field/oneof/enum, service/method, import and constraint observations.
Preserve package/full-name/field-number meaning within the exact bundle, original options,
source comments where available and omitted/unknown relationships. Reject conflicting logical
filenames before constructing pools; do not mutate a global pool to model changing schema worlds.

Support supplied complete descriptor files without a running helper or remote registry. Embedded
protox Compiler/FileResolver is conditional on an actual source-schema diagnostic/import consumer
that descriptor input cannot serve; it must not duplicate Buf's generation compiler needlessly.
Reflection from a live service is an explicit acquisition/consumer choice, not automatic on read.

Acceptance navigates a method's inputs/outputs, presence and constraint metadata; distinguishes
same names across bundles; preserves custom options; and refuses conflicting imports. Use an
offline protoset with grpcurl list/describe as a relevant independent interpretation. Demonstrate
direct shared projection on generated values, not encode/decode into a second dynamic tree.
Remove superseded manual schema/catalog interpretation in migrated callers only.

<a id="d02"></a>
### D02 — Schema-bound payload explanation and passive previews

Consumes D01. Bind original payload bytes/artifact and exact descriptor closure before decoding.
Explain binary/textproto/qualified JSON payloads with typed fields, presence, selected violations
and useful compact views. Use eager/static reflection for known generated values and dynamic
messages when the selected schema is genuinely dynamic. Retain original bytes if replay is the
claim; re-encoding is not canonical identity. No decoded method is invoked as a side effect.

Acceptance covers absent optional versus default, unknown enum/fields, malformed/truncated
wire, invalid combinations, int64/bytes and actual Any/FieldMask behavior where supported.
Unsupported or partial interpretation stays explicit. Reconstruction/schema inference tools
can nominate an interpretation when originals are missing, but cannot establish authored truth.
Use protoscope/prototext for a concrete wire question, not as a mandatory second processing path.
No runtime validator, lazy view or new forensic subsystem is required absent its own need.

<a id="d03"></a>
### D03 — Contract differences and qualified generated-Rust associations

Consumes D01 and B02 qualified evidence/expansion. Combine existing original source/API/Git
comparison with descriptor differences: field/type/presence/constraint and RPC-shape changes.
Classify binary, JSON, Rust-source and semantic changes separately. Buf breaking is native
classification evidence, not a veto on intentional design-phase breaks.

Follow declared generated origin toward authored schema and nominated consumers. Actual Rust
uses still require original HIR/source/artifact qualification; field numbers, type URLs and
matching names do not supply it. Preserve acquired rustdoc artifact-local IDs, selected Cargo
worlds and coordinate conventions. No universal persistent Rust/schema graph is needed.

Acceptance includes wire-safe/JSON-breaking rename, constraint-only change, missing source,
same-named declarations in different bundles, generated-origin lookup and cfg-only change in
a separately qualified native world. Report unexamined contexts and ambiguous correspondence.
Delete duplicate change/coordinate reconstruction where current consumers have moved; retain
complementary original source and artifact readers.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| D01 | planned | Descriptor investigation owner | Useful offline schema navigation with immutable identity and direct projection |
| D02 | planned | Payload inspection owner | Passive schema-bound explanations with preservation and malformed-input limits |
| D03 | planned | Contract/origin investigation owner | Classified changes and qualified generated-Rust/source associations |

**Proposed:** targeted fixtures challenge transformations with known authored meanings rather
than reproducing generator output as the only oracle. Independent tools share some underlying
libraries; state that limit. Q01 owns final public journeys and static checks. No schema-aware
inspection result establishes scientific correctness, native generic applicability or measured
context efficiency by itself.
