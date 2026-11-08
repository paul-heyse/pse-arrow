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

Start with the recorded-result ordinary reader and scientific inspection source already on main.
Compile pure generated messages/validators/descriptors without `engineering-service`; keep service
dispatch and broad service adapters gated. E's real Arrow/portable-reader cutover consumes the
small recorded-result adapter, so B01 acceptance includes that consumer closure rather than a
generated file alone. Generate scientific fixture observations with source identities, specification
presence, physical type/quantity/unit and original integration inputs; science owns producer
meaning and its independent reader. Verify changed-source generation before updated adapters
exist, plus default/service feature closures after publication. Then retire intermediate broad
root registrations, handwritten fallback adapters and obsolete generator assumptions.

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

### B03 — Finish finite source-to-consumer migrations

The migration population is the current selected engineering requests/responses/edits/progress,
modeling inspection and prepared studies; preparation/evaluate/stop/close worker crossings;
reference request/result/diagnostics; developer selection/prerequisites/reports/evidence;
independent-host configuration, requests, facts, graph/investigation and code-intelligence schema
clients; and E's selected result/storage metadata readers. Identify each existing production
caller at implementation time and keep its disposition in the owning package, not a second
permanent schema inventory. A new exemplar does not complete that population.

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

Add a source-owned provider request, method effect, diagnostic or result facet through its native
owner and follow the resulting changes through every selected generated/default/service/host/client
consumer. Preserve exact original errors, optional false/zero/empty and IEEE negative zero; exercise
unknown/duplicate/noncanonical encodings according to the operation policy. Required generation
errors are explicit, not supplier substitution. Use binary, ProtoJSON and text where consumed;
do not manufacture an unused format matrix. Remove fixtures/tests solely supporting retired maps.

## Verification, decisions and completion

Implementation is active. Selected generator and consumer evidence is recorded at the coordinator;
general bootstrap and the full consumer population remain open. Use targeted
native checks/tests with explicit `force_validate` on Arrow-consuming closures and `just codegen`
when declarations/generators change. Check receiving default/service and independent-host closures
as they migrate. L owns the final integrated population, independent journeys, hygiene and benefit
claims. A focused pass qualifies only its named source/feature/consumer population.

Metadata convention changes follow the coordinator's ADR route before affected implementation;
ADR-0250's current proposal is not automatic acceptance. No new rule waiver is introduced here.
The original-world, unknown/presence, native-authority and clean-room constraints remain binding.
Completion requires all finite selected callers moved, replaced mechanisms deleted, generated
artifacts reproducible and the applicable L acceptance complete. Preserve historical interpretation
only where an actual retained artifact requires it, with explicit unavailable adoption; do not
retain a second production path as evidence.
