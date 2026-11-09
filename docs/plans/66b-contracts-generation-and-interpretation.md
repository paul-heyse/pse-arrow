---
title: "Plan66b: Contracts, generation and interpretation"
status: in-progress
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/66-integrated-simulation-target.md
doc_topics: [design, libraries, validation]
review_sources: [docs/plan_improvement_reviews/plan64-65-integrated-target-and-library-leverage-2026-11-06.md, docs/plan_improvement_reviews/plan64-65-contracts-and-runtime-analysis-2026-11-06.md]
---

# Contracts, generation and interpretation

## Remaining target and baseline

Plan66 is the sole go-forward target. This companion owns shared source correspondence,
generation, representation and interpretation changes, consuming review I04 and CR01–CR04.
The [coordinator](66-integrated-simulation-target.md) owns combined scope and decision routes;
[A](66a-scientific-operations-and-product-semantics.md) owns scientific meaning,
[C](66c-preparation-execution-and-service-lifecycles.md) native preparation and execution,
[E](66e-protected-data-and-result-delivery.md) protected data consumers, and
[F](66f-developer-operations-clients-and-evidence.md) developer and independent client cutovers.
These are consumer dependencies below whole-document completion, not sequential document gates.

**Interface-checked:** source inspection at main `66341514d` establishes the following starting
mechanisms. They are prerequisites; this plan does not commission their implementation again:

- Source-owned `pse-contract` declarations, generated native adapters and field metadata v2;
  the maintained Buffa0.9.2 family and protovalidate-buffa0.7.2 supplier composition.
- Typed semantic metadata and the shared descriptor comparison engine in `pse-contract-proto`;
  exact source/runtime-world checks retain options, imports, features and unknown metadata.
- Descriptor-guided independent-host search/count and bounded payload inspection, without the
  retired full-declaration cache. Existing coverage remains bounded by its named tests.
- Native recorded-result context observations in `workflow/result_selection.rs`, and original
  selected-root/instance/fixture inspection observations in `workflow/modeling/documents.rs`
  and `inspection_fixture.rs`. Source declarations being present does not establish generated
  availability or a migrated consumer.

The earlier contracts and result branches supplied candidates for the recorded-result cut.
Main contains the integrated finite source-forward route below. Compare any further candidate
by patch and actual consumer before carrying it into a new declaration, generated output and
receiver delivery.

## Target responsibilities and composition

One declaration owns each native meaning. Generate passive observations, structural/local
validation and mechanical correspondence from that source; retain fallible contextual admission
in the native owner. Decoding a declaration, historical receipt, provider description or selected
result cannot mint current execution, membership, publication or restoration permission.

Generation scope follows useful consumer capabilities. Ordinary recorded-result export needs
messages, validators, descriptors and its small output-only adapter closure; it must not acquire
an engineering service dependency. The engineering service and independent host have their own
compilation and presentation needs while sharing source declarations. Each descriptor pool owns
its exact complete import closure; source agreement does not make distinct pools interchangeable.
A semantic world is not a requirement to compile every adapter whenever one consumer is built.

In `xtask/src/codegen/engineering.rs`, register the native roots
`PackResultSelection`, `PackResultPredicate`, `PackResultContext` and `PackScientificState`
under their source-declared `RecordedResultSelection`, `RecordedResultPredicate`,
`RecordedResultContext` and `RecordedScientificUsability` projections. Emit the selected closure
with `native_adapters_scoped(..., AdapterVisibility::Crate)` into an ordinary-consumer artifact;
remove those roots from the broad service adapter closure. Keep one generated implementation
owner for every native/projection pairing. Export no inverse observation constructor. The
original scientific inspection expansion remains rooted transitively at `ModelingInspection`;
generate its new fields and nested fixture types without a second inspection schema.

Make source-to-generation bootstrap reproducible when native source has advanced beyond old
generated adapters. Separate the source extractor/generator dependency closure from stale runtime
adapter compilation; do not hand-edit generated files, temporarily weaken declarations, or add
a hand-maintained mirror. Use an explicit adapter-group declaration only where actual additional
consumers justify it: roots, direction, destination and visibility must feed generation and its
ownership output once. A new crate per projection and a universal policy catalogue are unnecessary.

The selected finite bootstrap cut moves the four passive recorded-result declarations to the
existing `pse-model` owner and keeps `from_read` contextual assembly in runtime. A positive pure
generation feature compiles those source declarations without runtime adapters. It publishes
`recorded_result.proto`, one generated message/validator/adapter owner and a complete recorded-result
descriptor pool. Full engineering generation regenerates that source file and imports its exact
definition with per-type Buffa extern mappings; it does not emit those types again. Admit imported
definition ownership, exact dependency/file agreement and output/provenance collisions before
publication. Both early and full generation preflight their own complete outputs. Full engineering
retains its complete pool; recorded-root reflection uses its own pool. Never share
`SharedCorpusContext` across differing corpora or invoke native callbacks with a foreign pool's
descriptors merely because names or fields match. Exercise each own-pool traversal, exact full-world
import and foreign-world refusal, plus a source-forward shape change before stale runtime adapters
compile. This is dependency factoring within the existing source-owned contracts and exact-world
policy, not a new metadata convention, scientific schema or current permission.

This finite cut is integrated on main. Its owning checkout passed six generator controls,
eleven ordinary-result controls with `engineering-service` excluded, and eighteen service/result
controls, all against zero with no build warnings. An actual source-field removal compiled the
early generation route while the old adapter still described the prior shape; the probe restored
the original declaration and regenerated before integration. Independent Buf encoding agrees with
the original recorded-result golden bytes. The early/full descriptor pools remain separate, and
the finite graph rejects future transitive re-registration outside its supported imported cut.
General runtime-root bootstrap is still open. Full combined `just codegen` passed fifteen targets
on main. Main's six selected generator controls and nineteen ordinary-result/developer-query
controls subsequently passed, zero failures/pending/build warnings; the ordinary receiving excludes
`engineering-service`. The service receiving closure remains separate from these passes.

The scientific worker now has an early generation closure. Its passive Request and Response
declarations live with the `pse-kernels` portable worker vocabulary. Runtime retains admission,
diagnostic budgets and effects. The early and aggregate routes call the same projection producer
with Buffa 0.9.2 and the maintained validator composition. A temporary `Evaluate.weights`
removal generated while the previous adapter still projected that field. The original shape
was restored and full generation published all 15 targets. The worker messages and validators
stayed byte-identical. Source-forward generation for the other runtime-owned families remains
open. [Worker generator](../../xtask/src/codegen/scientific_worker.rs) ·
[Portable declarations](../../crates/pse-kernels/src/worker_protocol.rs)

The existing `xtask/src/codegen/proto_projection/composition.rs` already composes the complete
typed request, Buffa corpus, maintained validators and service generation. Retain that foundation.
Buffa0.9.2 `SharedCorpusContext::new` / `CodeGenConfig::shared_corpus_context` reuse complete
descriptor analysis; maintained `protoc_gen_protovalidate_buffa::prepare(&request)` prepares once
and renders the same indexed world. Connectrpc-codegen0.9.0 receives the same Buffa configuration,
explicit extern mappings and view support. Preserve output-path/collision admission and supplier
provenance: convenience `apply_companions` does not itself prove absence of orphan or duplicate
module includes. Sources are `vendor/buffa-codegen/src/lib.rs` and the composition owner above;
the [CR01 analysis](../plan_improvement_reviews/plan64-65-contracts-and-runtime-analysis-2026-11-06.md#CR01)
records exact release conditions.

## Interpretation, local rules and native adoption

Every selected ingress and retained format needs an explicit operation policy: grammar and
supported features; scalar/enum/message shape; absent versus present default/empty; unknown
fields and alternatives; local constraints; original interpretation; and the native owner/context
needed for adoption. Use the existing declarations and interpretation machinery rather than a
parallel policy service. Unknown metadata must remain observable even where execution refuses it.

The exact runtime/source-world comparison may remove genuine `SourceCodeInfo` for that comparison
only. It must preserve file order, imports, features, options and unknowns; keep the original
artifact identity unchanged. A Rust source move is provenance, not automatically a semantic
change. Semantic compatibility uses native crossing, registry execution and canonical publication
facts, not a descriptor checksum, a name match or a foreign pool callback. Original-world
normalization executes only with its matching native owner. Historical data can remain readable
with adoption unavailable.

Extend current `pse_contract_proto::compare_descriptors` into actual CLI and client use rather
than creating another checker. The independent host's `schema/compare.rs` currently supplies
structural/Buf comparison. Add the shared semantic assessment as a separately identified result:
compatible, incompatible or unknown, with policy/coverage and original-world limits. Structural,
wire, validation and scientific compatibility answer different questions. Product-specific
normalization remains an explicitly prepared product-owner operation; the independent host stays
product-free and help/read never starts scientific machinery. F owns the user-facing cutover.

Use maintained local validation for owned and view representations, preserving integer exactness,
presence and supported CEL semantics. Malformed rules and required unsupported semantics refuse
generation; no runtime CEL fallback or discarded rule is allowed. Bounded nonselection of arbitrary
dynamic validation and the retained Rustdoc/Serde bridge are existing decisions, not unfinished
packages. Extend WKT/Any, proto2/extensions, enum aliases or JSON/text support only for the finite
consumer population that actually uses them, with explicit unknown/presence behavior. An `Any`
type URL and successful dynamic decode are not native authority.

## Physical representation and whole-lifetime limits

Select fields and rows at the producer before avoidable nested cloning and materialization.
Preserve the meaningful observation extent and checked overflow rules while deleting duplicate
preflight passes. A full ordered inspection should use its native inventory operation rather than
repeated page/decode cycles; bounded pages still need original totals, offsets and continuation.
Coordinate population and physical typing with A and the scientific companions.

Use Buffa0.9.2 `Message::try_encode_bounded_with_cache` or `SizeCachePool::try_encode_bounded`
where the owning response needs bounded output. The size pass checks limits before writing;
cache reuse amortizes encoding scratch, not native mapping or total heap use. For selected large
responses, preserve backing-aware `Rope::with_backing`, `OwnedView`/`ViewEncode` through actual
transport wrappers. Connectrpc0.9.1 `Encodable::encode_segments` defaults to contiguous `encode`;
the current `Validated` and `AccountedResponse` wrappers must explicitly preserve segmented
encoding when selected. Aggregate segment lengths with checked arithmetic before publication,
and keep response/allocation ownership alive through consumption of the last segment.

This is not a universal zero-copy requirement. An owned `Vec` producer, compression, interceptors,
JSON or retained giant backing buffers can make a compact copy preferable. `PreEncoded` has codec
conditions and JSON reserialization; raw bytes do not replace validation. Buffa maintained
`DecodeContext::with_allocation_budget` and `CodeGenConfig::allocation_accounted_views` can enforce
phase-monotonic logical backing-allocation limits before growth, including actual nested/extern/WKT
capability coverage. They do not bound allocator/RSS, owned native adoption or mathematical work.
Keep body, decompression, element, logical allocation, output, queue and resident-state limits
separate. Exact sources are `vendor/buffa/src/message.rs`, `encode_sink.rs`, `pse-patch.md` and
[CR02](../plan_improvement_reviews/plan64-65-contracts-and-runtime-analysis-2026-11-06.md#CR02).

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| B01 | in-progress | Existing source declarations, typed supplier composition and current exact pins | Consumer-scoped generation and source bootstrap work; ordinary result adapters and complete inspection fields are generated in the one world, duplicate broad roots/implementations are removed, default and service receiving closures compile. |
| B02 | in-progress | Existing metadata/checker; native semantic owners for each selected crossing | All selected crossings carry explicit presence/unknown/interpretation/local-rule/adoption policy; semantic comparison is consumable independently; stale and foreign worlds cannot acquire native authority. |
| B03 | in-progress | B01 closure needed by each consumer, B02 policy needed at its boundary | Common engineering service callers use their source-generated adapters on main; handwritten request/result/ID/progress and source-edit maps are deleted. Selected service/result receiving passes. Complete other finite worker/reference/host/developer/storage families with their actual callers and deletions; this service slice does not close the population. |
| B04 | in-progress | Actual large producer/consumer and B03 mapping for that family | Main's AccountedResponse enforces binary limits before writing and JSON limits through the existing typed sink, retaining original ownership through the last byte consumer. Thirty-eight selected service/result controls pass with zero failures/build warnings. Producer selection, other finite response families and any justified segmented consumer remain open. |
| B05 | planned | Each delivered B01–B04 family, its outcome-owned consumer | Cross-representation/evolution/extension controls and surviving-source inventories cover the complete selected population; remaining assembled verification is handed to L with explicit omissions. |

### B01 — Deliver useful generation closures and reproducible bootstrap

#### Shared index bindings and compiled contract ownership

Name the authored index-binding item in the registry, following the existing named
`ModelingUnitFactor` pattern. Generate its wire type and common local rule once and
reference it from equivalent index fields. Check effective options, tags, presence and
interpretation before moving the 19 occurrences identified in the review. Keep binder order,
lexical resolution and occurrence provenance at the declaring field. Distinct vocabularies,
identity roles and cohort page limits retain their own contracts. Retire the replaced anonymous
items and their conversions with their callers. [Modeling registry](../../crates/pse-schema/src/catalog/modeling.rs)

Start from the finite recorded-result cut. Derive separate roots for passive
`ModelingInspection` and the [two Study input capabilities](66a-scientific-operations-and-product-semantics.md#a01--study-input-capabilities).
Derive the transitive fields, alternatives and imports from those roots. Publish complete
source-aware descriptors for independent tooling and compact complete pools for each runtime
reflection world. Select compiled Rust and registered procedures independently. Gate broad
engineering messages, validators and service adapters at the service boundary once their
passive callers move. Scope each adapter to the actual crossing and keep private native
construction at its owner. A passive compiler root uses the finite contract while the full
descriptor world remains independently readable.
[Recorded-result generation](../../xtask/src/codegen/recorded_result.rs)

Use the maintained Buffa 0.9.2 `generate_with_diagnostics` with explicit `files_to_generate`
and exact `extern_paths`. The complete descriptor input resolves imports and options,
selected files determine Rust emission, and imported references resolve to the one compiled
type owner. Split descriptor files where file-granularity selection requires it. Reuse
`SharedCorpusContext` only for the same effective corpus and representation controls.
Preserve consequential generator diagnostics, exact pool identity, and maintained view
allocation accounting across admitted imports. [Buffa generator](../../vendor/buffa-codegen/src/lib.rs) ·
[Maintained view profile](../../vendor/buffa-codegen/pse-patch.md)

Use ConnectRPC codegen 0.9.0 `generate_services` and the maintained
Protovalidate-Buffa 0.7.2 `prepare` and render from the same admitted corpus and
ownership selection. Connect requires generated views at its roots. Its default outputs
mode emits view encoders for outputs in selected services, skipping absolute external
type mappings. Co-locate those implementations with their types. Use
`EncodableImpls::AllMessages` only on a finite owner when downstream services actually
require shared binary view returns. Owned returns are available where views add little.
Compile generated `Validate` implementations with their Rust types. Generated local
checks supply shape and field admission. Contextual scientific adoption stays native.
[ConnectRPC codegen 0.9.0](https://github.com/connectrpc/connect-rust/blob/v0.9.0/connectrpc-codegen/src/codegen.rs) ·
[Validator preparation](../../vendor/protoc-gen-protovalidate-buffa/src/prepare.rs) ·
[Supplier composition](../../xtask/src/codegen/proto_projection/composition.rs)

Resolve RC02 before fixing crate placement. Compile one shared message and its
validator with an actual Connect service and a passive reader. Demonstrate one Rust
type and validation owner, exact imports and pool behavior, and a one-way dependency
that excludes broad engineering Rust from the passive root. Prefer an existing suitable
owner. If the mixed ownership cannot work there, keep service-used types with their
traits and separate the genuinely passive subset. If a required independent build
boundary still needs a new crate, complete the crate ADR and design review before
adding it. Until this control settles placement, only the source-level ownership and
existing service gate are selected.

#### Existing receiving baseline and remaining controls

Start with the recorded-result ordinary reader and scientific inspection source already on main.
Compile pure generated messages/validators/descriptors without `engineering-service`; keep service
dispatch and broad service adapters gated. E's real Arrow/portable-reader cutover consumes the
small recorded-result adapter, so B01 acceptance includes that consumer closure rather than a
generated file alone. Generate scientific fixture observations with source identities, specification
presence, physical type/quantity/unit and original integration inputs; science owns producer
meaning and its independent reader. Verify changed-source generation before updated adapters
exist, plus default/service feature closures after publication. Then retire intermediate broad
root registrations, handwritten fallback adapters and obsolete generator assumptions.

The reference closure now uses the same source-owned projection producer in its
early reference-only and aggregate routes. `scripts/pse-env --recipe-default-class
functional -- sh scripts/source-change-codegen reference-exchange` emitted the new
degeneracy result from the passive model declarations with the predecessor
runtime adapters still present. No product runtime was compiled for that stage.
The runtime continues to own contextual reference admission and scientific
receiving. The default runtime exercises the passive inspection producer. The
engineering cutover moves its wire round-trip controls and native adapters to
`engineering-inspection`. That feature imports only the common generated types from
`pse-host/engineering-types`. The separate `engineering-client` selection adds the
explicit Study and Study control stubs. The broad runtime service imports the same
types and adds the expanded Study and performing engineering surface.
[Reference generator](../../xtask/src/codegen/reference_exchange.rs)

`sh scripts/source-change-codegen engineering` under `scripts/pse-env --` selects
the source-owned engineering graph without either engineering receiving feature.
It publishes the shared, inspection, explicit Study and full worlds from the same
admitted descriptor graph. Each world owns its Rust emission and exact imports.
The aggregate route collects the separate reference and worker generators alongside
engineering. [Engineering generator](../../xtask/src/codegen/engineering.rs)

Change one `ModelingInspection` field while the former service adapters still exist.
Build the extractor from the source owners, emit the new closure, then compile both
the ordinary and service consumers. Inspect planned units and actual module includes for
the ordinary result reader, explicit Study client and performing service. Show that an
unrelated service-only edit leaves passive compiled inputs unchanged. Move full-world
interop assertions to the service or independent reader. Keep ordinary result assertions
on the finite descriptor and real native reader.

### B02 — Make interpretation usable and refusal precise

Complete metadata for actual native field meanings and local constraints, including source
correspondence, optionality, ordering, exact integer/IEEE representation and original-world
identity. Exercise same-world, SourceCodeInfo-only, changed options/imports/features/unknowns,
foreign pool, stale native owner and unavailable historical normalizer. Unknown coverage remains
unknown. The semantic result is already integrated in F's existing `pse inspect schema`
comparison operation; extend its selected native-owner population rather than adding another
comparator. Structured facts keep semantic disposition and source coverage separate from Buf's
wire classifications. Parsing immutable compiled descriptors once per generated
owner may remove repeated work; never reuse them across a merely similar descriptor world.

Actual main CLI receiving compared each generated descriptor with its SourceCodeInfo-bearing
original. The explicitly selected `developer-evidence` policy reports compatible with native
coverage for `developer_evidence.proto`; all three Buf categories also report compatible.
The same structural comparison for `recorded_result.proto` under `core` reports semantic unknown,
as does an unselected semantic population. Core intentionally supplies no native owner policy;
neither unchanged bytes nor imports can supply that missing coverage. These read-only journeys
exercise the shared `pse_contract_proto::compare_descriptors` owner and do not qualify other
families, scientific meaning or present readiness. No database/compiler/provider preparation
was performed by those comparisons.

The independent host now also accepts an explicit `recorded-result` policy for
`recorded_result.proto`. It reads the single generated coverage and interpretation table from
the original projection roots through `pse_contract_proto::compare_descriptors`. The actual
CLI compares the runtime and SourceCodeInfo-bearing original descriptors as semantically
compatible under that policy, with available native coverage and compatible Buf classifications.
The default Core policy continues to report unknown semantic coverage for this family. The
observation supplies no current scientific admission or original-context restoration.
[Recorded-result generator](../../xtask/src/codegen/recorded_result.rs) ·
[Independent host comparator](../../crates/pse-host/src/schema/compare/semantics.rs)

**Tested, selected:** `scripts/pse test pse-contract-proto pse-host --lib` with the
acquired comparison and host semantic filters passed seven controls with zero
failures, 16 build warnings and complete settlement in
`01a1bd39-3eb1-747b-86ff-addefb6fdee5`. The native comparison reuses each exact
acquired Buffa pool for its file-scoped original policy and preserves the
original file occurrence inventory when acquisition deduplicates identical files.
The selected recorded-result CLI comparison reported compatible native coverage.
[Acquired descriptor comparison](../../crates/pse-contract-proto/src/lib.rs)

### B03 — Finish finite source-to-consumer migrations

For the versioned Study cutover, generate the two A01 roots and remove the combined
wire request and superseded maps after C02 and F05 move their actual callers. For the
private worker revision, project the native `ProviderDemand`, request and passive
attempt/session/generation reference. Preserve checked ordinal widths, exact output order
and typed alternative refusals. Use the native derivative-order tags selected in RC03.
Verify that the maintained projection handles the actual tuple/record variants and
checked integer conversions before fixing their emitted layout. The native declaration
owns the checked realization if a generic projection cannot express that shape. Retire
the handwritten demand/handle mirrors and both conversion directions. C03 owns fresh
session binding and coordinated image activation.
[Native provider declaration](../../crates/pse-kernels/src/provider.rs) ·
[Current worker protocol](../../crates/pse-kernels/src/worker_protocol.rs)

The migration population is the current selected engineering requests/responses/edits/progress,
modeling inspection and prepared studies; preparation/evaluate/stop/close worker crossings;
reference request/result/diagnostics; developer selection/prerequisites/reports/evidence;
independent-host configuration, requests, facts, graph/investigation and code-intelligence schema
clients; and E's selected result/storage metadata readers. Identify each existing production
caller at implementation time and keep its disposition in the owning package, not a second
permanent schema inventory. A new exemplar does not complete that population.

The PID reference now uses `ReferenceCaseContract` and `ReferenceEnvelope`. The isolated
Python receiver validates the original time, physical roles and source artifacts before
running the exact and PETSc oracles. The native receiver checks the complete typed result,
the performing interpreter and solver-stage evidence, and all eleven candidate dispositions.
The PID JSON route was removed and its native receiver was tested. The working tree also
moves covariance, degeneracy and sensitivity to generated exchange and deletes the remaining
JSON driver. The 24 independent Python controls passed after the parity environment was
refreshed. The three new native solver exchanges await the combined receiving run.
[PID receiving test](../../crates/pse-runtime/tests/reference_pid_exchange.rs)

Replace the mechanical portions of `engineering/rpc/wire.rs::prepare_from_pb`,
`submit_from_pb`, `progress_value_to_pb`, `page_to_pb` and identifier helpers with generated
correspondence where the native owner declares the meaning. Preserve parent/membership checks,
provider support, edit effects and structured domain errors in fallible native admission. For
independent-host families retain raw versus adjusted paths, unavailable facts and unknown
alternatives; do not import product runtime to obtain a shared serializer. Each change includes
its actual caller and deletion of its old map/default-compatible path. F owns the host/client
behavior; C owns admission and settlement; B supplies reusable generation once.

### B04 — Preserve producer selection and encoding capabilities

The first delivered transport consumer is engineering RPC's `AccountedResponse`: its replaced
generic encoding allocated the whole response before checking `maximum_bytes`. Its owned
Buffa messages now use `Message::try_encode_bounded_with_cache`
so the binary size pass refuses before writing the body. The existing ProtoJSON path
feeds Serde's serializer through the existing `pse_ids::sink::BoundedSink`, retaining its typed
`SinkFailure` cause instead of classifying rendered errors. Keep response/allocation ownership
through the returned bytes. This bounds logical encoded output, not native adoption, allocator
capacity or RSS. The selected bodies hold owned `Vec`/`String` fields; Connectrpc0.9.1's
`Encodable` deliberately uses contiguous output for these, so adding a rope would retain no
shared payload. Preserve contiguous output here; select segmented wrappers only for an actual
backed-view/`Bytes` producer. Sources: [maintained bounded encode](../../vendor/buffa/src/message.rs),
[existing bounded sink](../../crates/pse-ids/src/sink.rs), and
[Connectrpc0.9.1 response contract](https://docs.rs/connectrpc/0.9.1/src/connectrpc/response.rs.html).

**Tested, targeted:** native `scripts/pse test pse-runtime --lib` with
`engineering-service,native-study-tests` and the selected RPC/lifecycle, recorded-selection,
fixture-inspection and protected canonical/project export/read controls passed **38 tests,
zero failures/pending/build warnings** on main (invocation `01a1b275-01e1-7736-8001-6ffc6f660992`).
Original capture and settlement completed. New exact-limit binary/JSON and last-consumer release
controls pass; serializer prose resembling a limit remains an internal error without a typed
sink refusal. This does not measure RSS or qualify all response/producer families.

Choose expensive real inspection/result/progress responses using source and consumer evidence.
Preflight selected extents before nested copies, validate once at the correct owner, and enforce
encoded output before writing. Modify wrappers and transport together when adopting segmented
output. Test cancellation between segments, invalid nested values, overflow, empty selection,
partial stream and last-buffer release. Keep contiguous encoding where it is simpler or releases
larger backing storage. Claim memory or latency benefit only after whole producer→encode→transport
→decode→release measurement under declared compression and retention conditions.

### B05 — Make extension and compatibility acceptance representative

For the shared binding extension, add a local constraint at the named item and
check independent descriptor interpretation and fallible native adoption across the
migrated uses. Distinguish malformed structure from valid structure with unresolved
lexical names. An equation-only rule stays local. Check one generated type and the
retirement of the occurrence-specific messages.

Add a source-owned provider request, method effect, diagnostic or result facet through its native
owner and follow the resulting changes through every selected generated/default/service/host/client
consumer. Preserve exact original errors, optional false/zero/empty and IEEE negative zero; exercise
unknown/duplicate/noncanonical encodings according to the operation policy. Required generation
errors are explicit, not supplier substitution. Use binary, ProtoJSON and text where consumed;
do not manufacture an unused format matrix. Remove fixtures/tests solely supporting retired maps.

## Verification, decisions and completion

Selected generator and consumer evidence is recorded at the coordinator. General bootstrap
and the full consumer population remain open. On execution resumption, use targeted
native checks/tests with explicit `force_validate` on Arrow-consuming closures and `just codegen`
when declarations/generators change. Check receiving default/service and independent-host closures
as they migrate. L owns the final integrated population, independent journeys, hygiene and benefit
claims. A focused pass qualifies only its named source/feature/consumer population.

Metadata convention changes follow the coordinator's ADR route before affected implementation;
ADR-0250's current proposal is not automatic acceptance. No new rule waiver is introduced here.
The original-world, unknown/presence, native-authority and clean-room constraints remain binding.
Completion requires all finite selected callers moved, replaced mechanisms deleted, generated
artifacts reproducible and the applicable L acceptance complete. Use the current declared
crossing and refuse a superseded contract. Keep precise original source and presence checks
for current records. The independent host now combines Buf-authored files and the native
dependency-usage projection in one complete descriptor. Buf resolves the graph-derived names
during staging, then the generator substitutes the complete source-owned definitions before
publication and validation. The shared context, location and library profile use the same
generated adapters across their callers. The targeted generator ran successfully. Native
receiving remains open for the broader population. The coordinator tracks the separate
source-forward `ModelingInspection` closure in the
[shared-contract cutover](66-integrated-simulation-target.md#shared-contract-cutover--november-8).
