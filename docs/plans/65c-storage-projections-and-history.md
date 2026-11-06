---
title: "Plan65c: Storage projections and historical interpretation"
status: in-progress
date: 2026-11-03
adrs: [ADR-0250]
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md, docs/plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md]
doc_role: plan
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [design, storage, libraries, validation]
---

# Storage projections and historical interpretation

## Target and basis

**Proposed:** generate portable Proto contracts and queryable native storage projections from
the same executable semantic owners, preserving exact values, coherent publication and original
interpretation. The database is an execution and persistence mechanism, not a second scientific
authority. Original bytes are retained only for an identified forwarding, archival or independent
reference consumer; a second copy of every row is not the target.

This companion implements the storage portion of the [combined target](65-proto-integrated-target.md).
Execution now proceeds alongside unfinished Plan64 under the joint Proto-first approach.
Its design, schema and code may all be replaced where this target is better. No other historical plan supplies work ownership or
limits this scope. Current source inspection is at `f61286a5fa5894b480efe8584c8b3d9b76d1b1fb`,
including the concurrent projection changes as provisional evidence. The post-Plan64 delta
must be reconciled at execution entry, not mistaken for current completion.

The [storage briefing](../library_leverage/libraries/proto/briefings/surrealdb-storage-and-projections.md),
[applicability](../library_leverage/libraries/proto/applicability.md), and source references below
are **Interface-checked** inputs. Local compositions and benefits remain **Proposed**. No
database, migration, replay or performance campaign was run to author this document.

The [November 5 review I06](../plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md#I06)
sharpens the remaining selected-read and historical-interpretation work at baseline
`1230a7e83`. It identifies no new storage-codec defect: the existing exact native codecs,
borrowed preflight, guarded effects and protected readers remain useful foundations.
This revision selects concrete consumer integration below; its new targets are **Proposed**,
without reclassifying historical tests as final-source qualification.

## Foundation assessment and alternatives

The actual server baseline is patched SurrealDB 3.3.4 with SurrealMX 1.0.0, not an inferred
upstream profile. `pse-host::server` selects the source/engine/patch tuple. Its patches add SSI
and explicit engine controls, connection-bound sessions, global session admission, independently
bounded typed error metadata, API rollback and controller/memory/deadline controls for modules.
These are capabilities and preservation obligations to compare, not a requirement to retain
every patch forever. A version move must replace the consumed behavior, not merely raise the
server version. SurrealDB v4 is outside this series.

Current `canonical_codec` already uses native Decimal for full-range `u64`, checks integrality
before decode, keeps text distinct from record IDs, and retains IEEE bits alongside useful finite
query projections. `canonical_client::wire` performs cumulative borrowed preflight before owned
Prost decode. `results` requires native QueryEnd, statement accounting and terminal metadata.
`canonical_results` retains protected-read/release obligations beyond a reader's lifetime.
Generate their mechanical structural correspondence; do not delete these guarantees in favor
of a successful ordinary decode.

| Complete option | Selection and rationale |
| --- | --- |
| Native records only | Default for scientific/runtime facts queried by the application. Compact and queryable; it does not promise unknown-field or original-byte preservation. |
| Opaque Proto only | Select for an immutable portable artifact whose reader needs the original contract, with a typed index/header for discovery. Reject for ordinary queryable state because it reconstructs database operations outside the database. |
| Hybrid original plus native projection | Select only where both archive/forward fidelity and native query access have real consumers. The projection is derived and rebuildable; its freshness and interpretation are explicit. |
| Arbitrary runtime descriptor-to-record mapper | Do not make this the first-party default. Generated checked mappings provide stronger local reasoning for locally owned declarations. Keep dynamic mapping eligible for an admitted externally supplied schema population. |
| Standalone byte-store replacement | Do not select as the combined store: raw byte/history primitives do not discharge graph/query, permission, publication or recovery obligations. A future independent byte-store responsibility can reopen it. |

## Chosen representation and publication design

### One mapping declaration with several physical products

Consume 65a's pure `pse-contract` structural graph and typed semantic options. Domain owners
declare storage participation, identity/key policy, relationship role, optionality, ordering,
numeric/temporal policy and retained interpretation. Scientific owners still supply contextual
validation; the pure graph must not import database values or Arrow. Codegen emits native
converters, structural DDL and portable descriptors as separate projections of that declaration.
Do not infer database behavior from a field name or comment.

The mapping revision identifies the native interpretation and rebuild logic. Schema/world
identity, scientific interpretation, validation-policy identity, mapping revision and encoded
byte checksum remain distinct. A record may carry only the subset its consumers require, but
that subset must suffice to reconstruct its declared meaning. Do not create one global digest
that invalidates unrelated artifacts on every generator edit.

Use `SurrealValue::{kind_of,into_value,from_value}` and native `Kind` as structural primitives
where their exact conversion matches the declaration. Generate explicit adapters for full-range
unsigned integers, structured identities, exact bits and temporal intersections. `SurrealValue`
derive does not emit permissions, indexes, multi-record predicates, retry policy or migration.
`surrealdb_types::{encode_proto,decode_proto}` serves explicit native-Value interchange; it is
not an arbitrary domain-message decoder. At 3.3.4 encoding clones a `Value`, and ordinary decode
has no caller-selected aggregate pre-allocation budget. Preserve admission before it is used
on uncontrolled data.

Native object conversion normalizes ordering and overwrites duplicate keys. Where the accepted
wire contract forbids duplicate keys, reject them before conversion. Forwarding a message,
adopting its known native fields and inspecting its JSON form are different operations. Unknown
fields can survive Buffa forwarding but disappear in a native projection or JSON conversion.
Exact archive promises therefore retain original bytes, not merely decode/re-encode equivalence.

### Historical reads and changes

Historical interpretation binds the original descriptor world, semantic interpretation,
validation policy and mapping revision. A historical read checks integrity and the original
policy promised for that artifact. Applying today's stricter policy is an explicit re-admission
or migration, not a silent read-time upgrade. Current scientific use separately performs the
contextual admission it requires. Retaining an old reader for promised evidence is distinct
from keeping two active production write paths.

Publish a hybrid artifact and its projection atomically where one transaction can contain
both. If the payload is an independently protected block, stage it under existing custody,
commit the exact reference/manifest, and retain uncertainty until original-request settlement.
No descriptor or validator can establish that an uncertain write did not occur. Definite
conflicts retry one fresh guarded decision; missing acknowledgements settle the original
immutable operation before another effect is permitted.

For schema/mapping changes, construct a new interpretation and derived projection privately,
verify original membership and complete dependencies, then activate one coherent generation.
Interruption preserves the previous active generation. Resume/restart only the owned rebuild
unit; do not expose partial projections as current. A design-phase destructive reconstruction
is allowed when no retained-data obligation applies, but it must be an explicit selected
operation, not an implicit consequence of opening a reader.

### Graphs, views and incremental consumers

Use ordinary relation records when edge occurrences have independent identity, attributes or
event visibility. Lightweight relations are eligible for propertyless endpoint-pair topology;
do not reuse pair identity as occurrence/event identity. Compare native inline caches and
in-path predicates before adding serialized adjacency or another cache.

A derived view must declare every input capable of changing its answer, including related
record changes and absent lookups. If the native maintained-view mechanism cannot observe those
dependencies, use a selected current query or explicit revision-bound rebuild. Verify stable
derived-row identity across initial and subsequent population; a mapping-version label alone
does not establish freshness.

Live observation is current-state delivery. Durable incremental consumption is selected only
for a consumer that requires continuity. That adapter owns source cursor plus mutation ordinal,
stable event identity, acknowledged progress and snapshot/rebuild on retention gaps. Inclusive
cursors may repeat entries; table-filtered empty batches may precede later relevant mutations.
Neither empty output nor a live reconnect proves caught-up status. Persist acknowledgements
with the consumer effect where atomicity is required, otherwise make duplicate handling explicit.

### Selected-result intent and exact reader population

The first remaining complete consumer is the existing
[`canonical_connected_results_reopens_declared_arrow_exact_range_and_cancellation` route](../../crates/pse-runtime/src/workflow/connected_results.rs),
with [selected snapshot/escaped reader controls](../../crates/pse-runtime/src/workflow/result_snapshot.rs).
Carry an exact original run/attempt, selected result set, column/coordinate intent and requested
range from the native reader into query/projection planning. Returned extent, whether the
request's semantic scope is complete, observation coverage and governing scientific permission
remain separate facts. A selected source query should avoid unrelated detail hydration, but
must retain the global scalar keys, source/schema/block protection and original predicates
needed to establish its answer. F04's narrower readable-root inspection does not become
full scientific admission or full-file validation.

Use the existing canonical query/protected-result owners for current reads. Add an index,
native query projection or mapping only where this selected consumer actually uses it; retain
filters and field selection at the native access path until representation/admission requires
materialization. A cached/maintained view is not the default: select a revision-bound view only
when every related-record and absent-lookup dependency is accounted for. Otherwise execute the
current guarded query. No new graph export or adjacency cache is needed for ordinary selected
result access.

Scientific product references from 65i add original experiment, producing attempt and
trajectory/output coordinates where consumed. Storage preserves those identities without
recomputing final-attempt choice or product permission. F owns user-visible selection/extent
and diagnostics; G owns independent presentation. The protocol metadata snapshot is passive;
the actual `ResultRead` protection and release task remain native. Migrate the selected
producer, original manifest metadata, current query, retained interpretation and reader together,
then delete replaced full-detail hydration, duplicate extent passes or field maps.

### Library realization and historical acquisition

For these selected mappings, generate explicit native conversions over SurrealDB-types 3.3.4
`SurrealValue::{kind_of,into_value,from_value}` and `Kind` only where their consumed
semantics fit. Preserve full-u64 Decimal handling, IEEE bits and domain-key conversions in
the declared mapping rather than relying on convenience casts. The
[trait source](https://docs.rs/crate/surrealdb-types/3.3.4/source/src/traits/surreal_value.rs)
and [3.3 capability guide](../libraries/proto/Protobuf_SurrealDB_v3.3.md)
characterize those structural APIs; they do not provide permissions, transactions or aggregate
decode admission. Use `encode_proto/decode_proto` solely for a selected native-Value
interchange consumer under the [3.3.4 codec contract](https://docs.rs/crate/surrealdb-types/3.3.4/source/src/proto/mod.rs).

Independent metadata inspection uses Buffa 0.9.2 `DescriptorPool::decode`,
`DynamicMessage::decode` and `field_by_number` with the artifact's approved immutable
world. The [dynamic source](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs)
characterizes decoding/reflection, not storage or scientific authority. Resolve original
descriptor/policy/mapping dependencies once for the retained artifact and keep their custody
for the reader's lifetime. Missing original interpretation reports incompatibility/incompleteness;
using today's schema because its field names match is not historical admission.
Source/scientific artifact reconstruction consumes the E02/E03 contract in
[65k](65k-scientific-artifacts-and-native-reconstruction.md), not mutable provider-session
readiness from 65e.

The first consumer does not require durable incremental delivery. C06 begins with a bounded
decision for a named external analysis/export consumer: specify continuity, event identity,
cursor/ordinal, acknowledgement/replay window and expiry recovery before selecting a changefeed
or outbox. If no consumer requires that promise, retain current reads and close the investigation
with documented non-selection. Upstream 3.3.5/MX 0.27 observations do not replace the patched
3.3.4/MX 1.0 baseline, and v4 stays excluded.

## Scientific use, evidence closure and independent changes

Consume [65i](65i-scientific-operations-and-product-semantics.md)'s operation/product-use
semantics and [65j](65j-workflow-lifecycle-and-developer-evidence.md)'s family-specific evidence
contract. A scientific retained read resolves original request/context, completion scope,
manifest/payload and the descriptor, interpretation, validation and mapping versions actually
needed by its promised reader. A developer report instead needs its original selection and
inventory for exact rerun; it need not move into canonical scientific storage. Share reference
mechanics only where useful, preserving existing owners and avoiding a second evidence store.

Use the selected [surrealdb-types 3.3.4 Proto codec](https://docs.rs/crate/surrealdb-types/3.3.4/source/src/proto/mod.rs)
for native Value interchange only. Its `encode_proto`/`decode_proto` operations do not supply
these family retention dependencies or scientific-use decisions. Generate checked native field
mapping with `SurrealValue::{kind_of,into_value,from_value}`, while native publication and
protected-reader owners keep atomic reference publication and release. This separates library
conversion from the context it cannot establish; the [3.3 guide](../libraries/proto/Protobuf_SurrealDB_v3.3.md)
characterizes the mapping APIs and numeric/duplicate limitations in detail.

Resolve retained interpretation once per appropriate immutable artifact/world, then carry it
through reads. Missing dependencies yield incomplete or incompatible evidence, not an empty
successful result. Expiry, absence, corruption and failed release remain distinct. An active
reader retains its required protection; historical storage does not keep a live fixture, worker
or evaluator ready. Old product permission is an original-context observation: current scientific
selection reuses or renews admission only under 65i's explicit premises.

Separate schema/projection rebuild from scientific dependency invalidation. Diagnostic-only
metadata evolution may rebuild a projection without rebuilding unchanged mathematics. A selected
correlation or physical datum change must invalidate affected science even with identical wire
shape. C01/C04 consume A's shared metadata checker for option meaning and evolution; they do not
reconstruct its rules or equate a wire-compatible revision with compatible interpretation.

## Selected graph and continuity decisions — November 6

**Interface-checked, bounded decision:** C05 selects the existing direct canonical relation
queries and no additional path/subgraph export, adjacency cache or maintained view. The actual
`CanonicalStore::graph_explanation` consumer binds an exact run/revision, artifact/revision,
analysis node/revision or study-point/revision/generation. Its indexed, keyset-paged queries
retain relation occurrence identity and parallel edges, validate the complete page including
its sentinel, and recheck the selected anchor. It explicitly reports one immediate neighborhood
under current metadata, not an atomic graph snapshot, scientific dependence or payload access.
That performing query remains with its current owner; generated value conversion supplies no
projection-freshness promise. Broader path/pair semantics are not selected by this execution.

Sources: [request, guarded query and evidence scope](../../crates/pse-operations/src/canonical_explanations.rs),
with existing controls `canonical_graph_explanation_preserves_isolates_parallel_edges_and_protected_inputs`
and `qualify_graph_seek_pages`. These controls were inspected, not rerun for this decision.
Complete C02 selected-result query/protected-read migration and C04 activation remain separate.
This decision also does not close A05/G01's remaining native/protocol graph correspondence.

C06 selects no durable external changefeed/export-continuity consumer. The bounded source
inquiry covered canonical graph/history/result/export and live simulation owners under
`pse-operations`, `pse-runtime`, `pse-host` and `pse-schema`. Existing terminal-attempt
manifest/payload acquisition preserves passive history; graph continuation is an interactive
page cursor; live simulation uses bounded in-process channels. None of these inspected
contracts promises persisted external acknowledgement, replay or retention-gap recovery.
Generated gRPC live-query descriptors do not establish an adopted consumer. Accordingly add
no generic outbox, broker or changefeed adapter. This is a target-scope decision for examined
first-party consumers, not a claim about out-of-repository or future users.

Sources: [historical protected results](../../crates/pse-operations/src/canonical_results.rs)
and [live simulation owner](../../crates/pse-runtime/src/workflow/live_simulation.rs).
Reopen only for a named external consumer with a required continuity/acknowledgement promise,
or a selected graph output whose semantics/dependencies the current query cannot supply.
Then apply the exact patched 3.3.4/MX 1.0 source contract above; v4 remains excluded.
No new implementation, database qualification or performance benefit is claimed by either
non-selection. C01–C04 and C07 remain open.

## Packages and dependencies

### First retained-result mapping — 2026-11-04

The registry declarations for `canonical_result_manifests` and `canonical_result_blocks`
now own explicit stable field tags and the `CanonicalResultManifest` /
`CanonicalResultBlockMetadata` boundary records. Existing registry generation emits their
native rows and checked Proto adapters from the same fields that already drive native
Surreal converters. The selected database representation retains Decimal unsigned values,
authoritative floating-point bits and query projections, and explicit optional binary data.
The metadata records do not construct `ClosedManifest`, protected reads or publication authority.

The source control `result_wire_presentation_preserves_native_semantics_and_encoding` verifies
that the presentation preserves native semantic and encoding identities, while changing an
unsigned numeric domain still changes semantic identity. The complete generation route has
published both records on main. Three independent value controls now exercise complete unsigned
extent, declared text and opaque bytes, signed zero, optional empty binary data, raw NaN bits,
and unknown-field read versus strict adoption. Existing native finite-coordinate storage
admission remains independent of readable Proto values. Protected-original context and the
remaining selected storage population are still required; this closes no C package.

For independent value reading, Buffa 0.9.2 `DescriptorPool::decode` links the published complete
descriptor closure, `message_index` resolves the selected record, and `DynamicMessage::decode`
reads its bytes without a generated message type. `field_by_number` exposes explicit
`Value::U64` and `Value::Bytes` values, including raw IEEE bits and optional empty binary data.
Use this selected immutable world rather than interpreting ProtoJSON as a database value.
The [versioned dynamic implementation](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs)
and [value representation](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/value.rs)
characterize the API. Readable raw bits and retained unknown fields remain separate from native
storage adoption and scientific use.

The table is this companion's native progress owner. A dependency
names a usable slice, not completion of every package in another companion.

The integrated source slice now declares run/attempt, result-set/batch/output metadata and
the actual `ResultSetDescriptor` alongside existing native fields. `ResultRead` supplies a
borrowed `CanonicalProtectedResultSnapshot`; its live protection owner stays local and has
no generated inverse. The initial publication refusal exposed unsupported canonical lifecycle
facets. Shared native canonical-policy interpretation and original-world field correspondence
now support those facets, and combined generation publishes all 13 targets on main. No native
transaction predicate moved into wire validation. The actual constant-completion fixture now
observes its protected reader through the published descriptor, including closed empty/nonempty
set counts, then successfully reads the original payloads through that same native reader.
The complete C02 population and broader retained-policy/readback obligations remain open.

**Tested, targeted on main, 2026-11-04:** `scripts/pse test pse-runtime --lib --features
engineering-service -E 'test(result_metadata_projection_tests)'` passed four selected controls,
zero failures/pending/build warnings, configured unbounded mode and cache off, with original
capture and native settlement complete. The fourth control independently reads actual attempt
metadata, including signed expiry, full unsigned generation, recorded terminal phase and
completion bytes; it grants no closed-manifest or live result-reader authority. Expanded
`scripts/pse test pse-schema --lib -E
'test(result_wire_presentation_preserves_native_semantics_and_encoding)'` passed one selected
control with the same zero-failure/warning baseline, preserving native semantic/encoding
identity for all seven selected metadata tables while detecting a real scalar-type change.
The targeted `canonical_durable_constant_completion_reopens_exact_manifest` control with
`canonical-tests,engineering-service` also passed on main, one selected test and zero failures,
pending records or build warnings, with its actual native fixture and capture fully settled.
Its independent descriptor read covers the original run/attempt/manifest and every closed set;
subsequent payload reads retain the original protection owner. The first probe failed one
assertion by treating an omitted implicit zero count as missing semantic data; descriptor-based
`ReflectMessage::get` now reads the declared default and still detects lost nonzero values.
These controls do not establish the entire retained-policy population or assembled storage acceptance.

| ID | Status | Prerequisite | Result and completion obligation |
| --- | --- | --- | --- |
| C01 | in-progress | Actual patched server/codec baseline; working 65a declarations/options and 65b fidelity policy | Preserve the delivered manifest/block mapping and its independent value controls. Complete exact checked field policies using 3.3.4 [SurrealValue/Kind characterization](../libraries/proto/Protobuf_SurrealDB_v3.3.md), retaining explicit u64 Decimal, IEEE-bit and identity conversions. Native predicates and borrowed preflight remain authoritative; test malformed/duplicate native values before any replaced admission is deleted. |
| C02 | in-progress | C01 generator; working 65a graph/adapters; F04 selected-read contract | Preserve delivered run/attempt/set/batch/output metadata and protected snapshot controls. Complete the selected scientific/runtime and canonical request/result population, including exact column/coordinate/range intent through the named connected-results consumer. Use generated native codecs and Buffa 0.9.2 [independent dynamic metadata read](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs); decoding cannot construct ResultRead custody. Delete replaced field maps/full-detail hydration only with their producer/readers and preserved predicates/global keys. |
| C03 | planned | C01; 65b original interpretation; 65i producer references; K-owned E02 where source reconstruction is consumed; 65f retained-evidence contract | Resolve the original family-specific source/context, descriptor, validation and mapping closure once under actual reader custody; add original payload storage only for a selected preservation consumer. Current scientific use remains separate from historical read/forwarding. Exercise absent/expired/corrupt policy and ambiguous/foreign experiment/attempt references. Delete duplicate headers/interpretation lookup; don't retain live fixture/worker readiness or infer authority from a reflected snapshot. |
| C04 | planned | C02/C03 publication; 65i dependency validity and shared A/B option evolution | Stage the new declared mapping privately, validate its original membership/dependencies, then activate one coherent generation. Retain current guarded-query access while a derived view is incomplete/stale. Contrast mapping-only rebuild with physical data/phase/derivative invalidation; test crash before/after activation, uncertain ACK and unavailable original policy. Delete replaced write paths only with all writers/readers migrated; historical readers retain their deliberate promise. |
| C05 | complete | [Bounded current-consumer decision](#selected-graph-and-continuity-decisions--november-6); selected C02/C04 migration keeps its own owners | Retain existing occurrence-preserving canonical relation queries. No additional path/subgraph, adjacency or maintained-view consumer selected; no new freshness machinery. Reopen for a named output/dependency requirement beyond the current guarded query. Interface-checked source disposition only; no fresh database qualification. |
| C06 | complete | [Bounded continuity non-selection](#selected-graph-and-continuity-decisions--november-6); C03/C04 remain open | No examined first-party consumer requires durable external cursor/ack/replay continuity. Retain passive protected historical reads and native live-channel semantics; add no changefeed/outbox/broker. Reopen for a named continuity promise under the exact patched server contract. No new implementation or test claim. |
| C07 | planned | C01–C06 decisions and all selected migrations | Hand H the actual selected producer→current query→protected read→original interpretation population, with empty/full/selected ranges, late terminal failure and failed-release controls. Supply matched complete-operation measurements only where claimed and enduring mapping/recovery guidance. Reconcile all producers/readers/generators/fixtures and deletions; emitted schemas or a native Value round trip alone cannot close publication/recovery. |

The first actual vertical slice is retained diagnostic/result metadata from 65f. It establishes
one useful generated mapping, independent read and original-policy behavior. C02 must then name
the complete selected population from source inventory; the first slice is not its closure.
Canonical function signature/bind/result metadata is included where it removes repeated
mechanical interpretation (CBO-046), but Proto does not type-check SurrealQL or replace its
transaction logic. CBO-066 and CBO-008 remain separate fidelity and retained-product obligations.

### Selected export observation — November 6

The next concrete A06/C02/F04 migration is connected Arrow export → portable artifact
validation. [Proposed ADR-0250](../adr/0250-source-owned-result-selection.md) precedes that
metadata cutover. The actual `PackResultSelection` source owns exact original run/attempt,
relation/version, row window, optional field and predicate endpoint bits, recorded usability,
and problem/revision/closed-manifest/native interpretation. Its generated output-only
`RecordedResultSelection` cannot construct a protected reader or scientific permission.

New exports replace the handwritten lineage-key writer with
`pse.canonical.selection.proto.v1`, containing standard padded base64 of the generated binary
observation. Maintained Buffa 0.9.2
[`Message::try_encode_bounded`](../../vendor/buffa/src/message.rs) uses one checked size/cache
pass and rejects a 65,536-byte budget before writing. The existing pinned base64 0.22.1
[`Engine`/`STANDARD`](https://docs.rs/base64/0.22.1/base64/engine/trait.Engine.html) encodes only
that bounded native output, at most 87,384 text bytes. The portable reader compares input
text with the same version's bounded generated observation of its declared original selection.
It performs no untrusted decode or normalization. Alternate encodings, duplicate singular
fields, unknown/trailing bytes, changed predicates and original-context substitutions refuse;
this is exact local serialization fidelity, not a universal Protobuf canonicalization claim.

Present-invalid or mixed predecessor/v1 metadata never falls back. Retained absent-v1
artifacts keep an explicit predecessor reader; the old representation has no new writer.
The native protected source remains the authority, and offline artifact/manifest agreement
establishes neither authenticity nor current scientific use.

The IPC prerequisite is integrated on main: schema and nested-field duplicate keys refuse
before Arrow map conversion in both stream and admitted-file readers. Sorting borrowed keys
uses the existing prepaid schema inventory; no new parser or duplicate field interpretation
is added. The isolated `scripts/pse test pse-columnar --lib --profile local -E
'test(ipc_stream::tests) | test(ipc_file::tests)'` passes 14/14, zero failures/build warnings
against zero, with normal incremental/sccache and configured unbounded placement
(`01a1b17a-b0b0-7417-803b-4ccd5bbb21e1`). Actual generated metadata producer/portable-reader
cutover and its integrated acceptance remain in progress; this prerequisite does not close C02.

## Additional capability dispositions

Surrealism transaction-local decode/validation is a bounded placement investigation only when
a selected operation avoids a material transfer or enforces a database-local contract. Compare
native generated fields first. If selected, reuse the prepared module owner, patched global
controller/deadline controls and immutable descriptor/policy cache per controller. Module KV
is volatile and reconstructible, never durable schema history, acknowledgement or deduplication.
The decision must name the replaced external work and actual resource lifetime; otherwise
retain external/native execution without a new module.

Custom HTTP byte APIs remain an alternative only if they simplify the complete bounded
canonical route while preserving effects, admission and settlement. Local API rollback is
already patched, so unpatched upstream hazards cannot be asserted as local defects. GraphQL/GQL
remain eligible database-aware query surfaces, not automatic scientific admission. Kafka and
`surreal-sync` are not local prerequisites; revisit for a concrete broker consumer and verify
unknown fields, framing and tombstones. Standalone SurrealKV/MX and alternative CBOR/FlatBuffers
codecs receive no parallel production path merely for uniform coverage.

## Verification, migration and handoff

**Proposed targeted controls:** generated converter agreement against independently specified
value cases rather than another copy of the same mapping; full u64 boundaries/fractional
Decimal refusal; negative zero/raw diagnostic bits; structured keys; signed duration refusal;
duplicate keys; unknown forwarding versus native adoption; changed validation policy; old
descriptor availability; view staleness after related-record mutation; initial/update row IDs;
inclusive replay and empty filtered pages; retention expiry; crash after effect before ACK;
failed cleanup independent of primary failure. Missing completion or unknown interpretation
must refuse, not return an empty successful result.

Run targeted generator/codec/store tests while implementing. Arrow-consuming correctness
closures explicitly enable force-validation. Run `just codegen` for declaration/generator
changes. The root performs formatting at the prescribed boundary; integrated/native fault
campaigns and static checks occur once at scope end under 65h. Record actual command, mode,
scope, zero failure baseline and result; no authoring-time product qualification is claimed.

Completion requires all selected mappings, readers, retained interpretations and activation
routes, plus their obsolete mechanism deletions. A schema emitted or a native Value round trip
alone does not establish application publication/recovery. Required schema/metadata decision
changes follow the coordinator's proposed ADR/design route before implementation. Existing
architecture is not edited by this plan.

## Sources

- [Detailed storage research and precise upstream references](../library_leverage/libraries/proto/briefings/surrealdb-storage-and-projections.md).
- [Supplied full 3.3 capability guide](../libraries/proto/Protobuf_SurrealDB_v3.3.md), especially A1–A3, A8–A11; upstream and patched-local observations are distinct.
- [Actual source-build integration](../../scripts/surrealdb-source/source-build.md) and [server declaration](../../crates/pse-host/src/server.rs).
- [Exact native codec](../../crates/pse-operations/src/canonical_codec.rs), [borrowed wire preflight](../../crates/pse-operations/src/canonical_client/wire.rs), [statement completion](../../crates/pse-operations/src/canonical_client/results.rs), [protected result owner](../../crates/pse-operations/src/canonical_results.rs).
- [surrealdb-types 3.3.4 public Proto codec](https://docs.rs/crate/surrealdb-types/3.3.4/source/src/proto/mod.rs) and [protocol 0.13.1 Value schema](https://github.com/surrealdb/surrealdb-protocol/blob/v0.13.1/surrealdb/protocol/v1/value.proto).
