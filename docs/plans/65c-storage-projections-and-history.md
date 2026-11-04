---
title: "Plan65c: Storage projections and historical interpretation"
status: proposed
date: 2026-11-03
adrs: []
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md]
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
It assumes Plan64 has been implemented before execution. Its design, schema and code may all
be replaced where this target is better. No other historical plan supplies work ownership or
limits this scope. Current source inspection is at `f61286a5fa5894b480efe8584c8b3d9b76d1b1fb`,
including the concurrent projection changes as provisional evidence. The post-Plan64 delta
must be reconciled at execution entry, not mistaken for current completion.

The [storage briefing](../library_leverage/libraries/proto/briefings/surrealdb-storage-and-projections.md),
[applicability](../library_leverage/libraries/proto/applicability.md), and source references below
are **Interface-checked** inputs. Local compositions and benefits remain **Proposed**. No
database, migration, replay or performance campaign was run to author this document.

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
transaction predicate moved into wire validation. The actual protected readback journey and
the complete C02 population remain open.

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
These controls do not establish protected database readback or assembled storage acceptance.

| ID | Status | Prerequisite | Result and completion obligation |
| --- | --- | --- | --- |
| C01 | in-progress | Actual native server/codec baseline; working 65a declarations/options and 65b fidelity policy | Result manifest/block metadata now has source-owned native/Proto mappings and explicit tags; source semantic/encoding preservation and three independent value controls pass. Complete protected readback and remaining exact field-policy controls; existing native transaction predicates remain authoritative. |
| C02 | in-progress | C01 working generator; 65a usable contract graph and emitted adapters | Source-owned original run/attempt and result set/batch/output mappings plus the borrowed protected-result snapshot are generated on main with typed canonical lifecycle facets. Complete actual protected readback and the selected scientific/runtime metadata and canonical request/result population, preserving transaction predicates and preflight until equivalence; delete replaced mechanical copies with consumers and fixtures. |
| C03 | planned | C01; 65b original-policy/adoption; 65i product-use and 65j evidence design; 65f evidence shape | Deliver retained artifact headers, complete family-specific context/descriptor/policy/mapping resolution and optional hybrid storage. Historical read, forwarding and current scientific use remain distinct. Test absent/expired/incomplete/incompatible/corrupt dependencies and active-reader protection; no retention of live fixture/worker readiness. Delete replaced ad hoc headers and repeated interpretation lookup. |
| C04 | planned | C02/C03 schemas and protected publication; 65i dependency-validity contract | Implement coherent rebuild/activation and selected retained-data migration using shared option-evolution decisions. Separate artifact rebuilding from scientific invalidation and renewed admission. Attack interruption before/after publication, uncertain ACK, stale membership and unavailable original policy. Delete the replaced write path after complete cutover; retain only deliberately supported historical readers. |
| C05 | planned | Actual graph/query consumers from C02; C04 interpretation generation | Migrate graph/query projections to explicit pair/occurrence identity and complete freshness dependencies. Prefer native access paths; repair initial/update identity and related-record invalidation controls. Remove redundant adjacency/materialization machinery where replaced. |
| C06 | planned | C03/C04; an identified continuity consumer | Deliver durable incremental consumption only for that selected consumer: cursor/ordinal, duplicate policy, acknowledgement crash windows, expiry and reconstruction. If no selected operation promises continuity, close with a documented non-selection and implement no generic replay engine. |
| C07 | planned | C01–C06 dispositions and all selected migrations | Supply 65h storage/publication acceptance, selected complete-operation measurements and enduring schema/recovery documentation. Reconcile all migrated producers, readers, generators, fixtures and deletions before completion. |

The first actual vertical slice is retained diagnostic/result metadata from 65f. It establishes
one useful generated mapping, independent read and original-policy behavior. C02 must then name
the complete selected population from source inventory; the first slice is not its closure.
Canonical function signature/bind/result metadata is included where it removes repeated
mechanical interpretation (CBO-046), but Proto does not type-check SurrealQL or replace its
transaction logic. CBO-066 and CBO-008 remain separate fidelity and retained-product obligations.

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
