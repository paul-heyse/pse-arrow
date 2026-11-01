---
title: "Plan61d: Schema, payload and contract investigation"
status: complete
doc_retention: while-dependent
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

Qualify text-format loss explicitly: the selected Buffa 0.9.2 generated reader skips unknown
named fields. Before exposing strict ingress or a lossless text conversion, select a supported
strict reader/generator improvement or report the interpretation as partial and retain the
original text. Do not equate a successful parse with preservation of every authored field.

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
| D01 | complete | Descriptor investigation owner | Offline discovery/navigation, complete immutable worlds and guarded expansion exercised |
| D02 | complete | Payload inspection owner | Passive binary/JSON/textproto explanations, static validation coverage and malformed-input limits exercised |
| D03 | complete | Contract/origin investigation owner | Native classifications and qualified generated-origin drill-through exercised |

### Execution checkpoint — 2026-11-01

Execution covers D01–D03 and the coordinator's Q01. The native host has been explicitly
restored after `cargo clean`. D01 owns immutable complete descriptor acquisition and shared
search/inspection references; D02 and D03 consume that same bundle rather than acquiring
independent schema worlds. Payload explanation and comparison/provenance are separate
implementation assignments; the coordinator owns integration and public journeys.

Textproto ingress uses descriptor-backed `protoc`, preserving original text and diagnostics;
successful parsing is distinct from validation. Dynamic schemas expose constraints without
claiming compiled validation. Generated declaration associations use explicit emitted Rust
syntax and exact input/output identities. No new daemon, runtime CEL interpreter or persistent
schema graph is selected. The coordinator owns Q01 assembled acceptance and scope-end checks.

## Outcome

**Implemented:** D01–D03 extend `search schema`, `inspect schema`, `inspect payload` and
`inspect origin`. Descriptor construction validates the full declared closure before linking;
private bundle state prevents consumers from mutating bytes, identity or linked metadata
independently. Native paths, original artifact digests, presence, unknown wire data, options
and interpretation limits remain explicit. Supplied dynamic constraints are not claimed to
have run through compiled validation. Text parsing and native compatibility classification
use the installed tools through existing supervised-child ownership and actual drain.

**Implemented:** producer-owned associations derive from explicit generated message names and
numbered accessor syntax using the existing `syn` parser. They bind descriptor, emitter,
configuration, authored source and output bytes. Working-source origin lookup uses the actual
enclosing Rust declaration and offers guarded authored-source expansion. Historical/index
hits do not inherit current-generation origin; schema associations do not establish native
Rust identity. There is no new service, mutable global pool, dynamic CEL evaluator or second
descriptor implementation.

**Tested — 2026-11-01, local Linux, configured unbounded execution:**

```text
cargo t -p pse-host --lib --test schema_payload_workflows -E 'test(schema::) | test(schema_descriptors) | test(plan61_) | test(semantic::wire::) | binary(schema_payload_workflows)'
```

58 selected tests passed, zero failures against zero (682 unselected). This covers original
bundle/query/reference identity, unused missing imports, conflicting files, option metadata,
service navigation, extension links, raw paths, configuration rebasing, strict JSON/text input,
payload presence/unknowns, selected validation, native Buf classifications and qualified
source associations. An actual production `Location` origin inspection and its returned guarded
authored-source action also succeeded. Offline `grpcurl` described both production services;
Buf 1.75.0 and protoc 36.2 supplied the selected native interpretations. These controls do not
establish universal protobuf/CEL conformance, native generic applicability or measured benefit.

**Interface-checked:** full `just codegen` passed all ten targets; the subsequent
code-intelligence-only regeneration refreshed emitter-qualified sidecars after lint fixes.
The focused generator qualifier test passed, including refusal of unknown assertion patterns.
Q01's final scope-end evidence remains with the coordinator.

### Mistakes corrected

Initial integration missed request/config-relative descriptor rebasing, extension parent links,
failure envelopes for unavailable Buf, raw temporary paths and ordinary generated-origin
navigation. Review exposed and corrected these, plus historical-origin and unguarded-action
errors. Generated JSON's unknown-key behavior required explicit schema-key preflight rather
than assuming parsing was strict. Focused tests challenge the corrected boundaries.

### Deliberate deviations

Strict descriptor-backed text parsing uses protoc because Buffa 0.9.2's generated reader drops
unknown names; required-field warnings remain distinct from validation. `syn` reads explicit
generated syntax instead of introducing another parser dependency. Generated lint qualification
preserves comments and permits assertions only in pinned reflection functions with exact known
immutable-metadata/self-encoding invariants; unknown functions or assertion messages refuse
generation. No acquired-input decoder receives a blanket assertion exemption. Source spans
remain unqualified producer coordinates; the source action reads verified original file bytes
without inventing a declaration byte range.

### Investigation findings — 2026-10-31

The qualification corpus is authored under `crates/pse-host/tests/fixtures/plan61`;
xtask's current explicit `codegen --only code-intelligence` route owns its generated test
output alongside production contracts. The qualification corpus itself is not the
production D01–D03 interface. Test-only host consumers investigate complete descriptors,
direct projection and payload interpretation without launching an analyzer or schema service.

**Tested:** Buf 1.75.0 built the complete corpus and offline `grpcurl -protoset … describe
plan61.qualification.v1.Investigation` identified its unary Inspect and bidirectional Run
methods. grpcurl reports `dev build <no version set>` locally; the installed-tool document's
version alone is not provenance for that executable. Independent wire inspection with
`protoscope -descriptor-set … -message-type plan61.qualification.v1.Payload -print-field-names`
identified an explicitly encoded count zero and unknown field 127 without inventing a name
for the latter.

**Tested:** a temporary source copy renaming field 6 from `label` to `renamed_label` passed
`buf breaking --config <WIRE configuration>` (exit 0), while WIRE_JSON and FILE returned
exit 100 with field-name/json_name change diagnostics. Each configuration retained the original
locked imports. Intentional breaks are classification evidence, not a compatibility veto.

**Interface-checked:** the actual Buf descriptor span for a field preceded by `/* π */`
starts at Unicode column 10, while the corresponding UTF-8 byte column is 11. D03 must
convert descriptor coordinates against the original source and retain its guard; treating
columns as byte offsets would offer an incorrect source-linked action. Descriptor identity
and generated names still do not establish native HIR/rustdoc/Cargo-world identity.

The generator's shared runtime pool strips SourceCodeInfo; the separately generated full Buf
descriptor retains it. They derive from the same import corpus but are different byte artifacts.
Use the full immutable descriptor for source-aware investigation, retain its original identity,
and treat the stripped runtime pool as a derived projection rather than the source authority.

**Proposed:** targeted fixtures challenge transformations with known authored meanings rather
than reproducing generator output as the only oracle. Independent tools share some underlying
libraries; state that limit. Q01 owns final public journeys and static checks. No schema-aware
inspection result establishes scientific correctness, native generic applicability or measured
context efficiency by itself.
