---
title: Plan64/65 contracts, generation and runtime analysis
date: 2026-11-06
status: proposed
source_baseline: 604516811
standard: core-3.4
profile: process-simulator-1.5
doc_role: review
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [design, libraries, numerical-execution, current-work]
---

# Plan64/65 contracts, generation and runtime analysis

This supporting review assesses the remaining combined Plan64/65 contract, representation,
client and runtime footprint. The [principal review](plan64-65-integrated-target-and-library-leverage-2026-11-06.md)
owns the combined judgment and subsequent
recommendation numbering. Existing plans continue to own obligations and status; this document
does not amend them or authorize implementation.

**Evidence:** recommendations below are **Proposed**. Consequential library mechanisms are
**Interface-checked** by inspecting exact installed or maintained source, without compilation
or execution during this review. Repository implementation observations are source inspection,
not new functional qualification. No build, test, preparation, generation, merge or performance
campaign ran for this review.

The inspected baseline is main `604516811`, using `git show` for decisive main-source claims,
plus the paused contracts worktree at `47ba2c5e3`. That worktree contains imported result
declarations `8a23783dc`, native bridge changes `eedbf9dce`/`42570816e`, and scientific inspection
declarations `a860539c9`. Its only uncommitted product-source change is
`xtask/src/codegen/engineering.rs`: the agreed separate recorded-result adapter emission and
its control. The separate output has not been generated. The scientific inspection source is
ahead of its generated adapters. Earlier generated snapshot `1e0acb390` supplies the four
recorded-result types, not those new inspection fields. These are distinct source states.

The review applies core AP-01–AP-07 and relevant DP refinements, the efficient architecture
heuristics, the test-assessment companion and process-simulator profile 1.5. Physical meaning,
derivative scope and truthful outcomes remain independent constraints on codec and runtime
improvements. No broad diagnostic audit was initiated.

## Combined judgment

Retain source-owned semantics, immutable descriptor worlds, generated local validation and
native contextual admission. Improve their composition around actual consumer capabilities
and retained lifetimes. The valuable remaining change is broader than generating more messages:
ordinary product reads must not depend on a service feature, transport wrappers must preserve
the library capabilities they wrap, and tools should explain semantic change through the same
checker the generator already uses. A schema or receipt remains an observation, not a grant.

The full selected migration still includes engineering operation maps, developer request/fact
families, independent clients, protected-result metadata, original scientific inspection and
worker/reference crossings. The delivered host search change is a useful existing foundation,
not a reason to repeat its implementation. Source evidence supports the amendments below;
whole-lifetime throughput and memory benefits remain unmeasured.

<a id="CR01"></a>
## CR01 — Generate consumer capabilities independently of optional service activation

**Proposed; foundational composition correction.** Complete the paused recorded-result split
as a consumer-capability design, then use the same principle for the remaining migration.

**Observed.** `pse-runtime/src/lib.rs` gates `engineering_projection` on
`engineering-service`; `host_operations/engineering.rs` gates its generated module similarly.
Ordinary connected-result export exists independently. The paused generator registers
`PackResultSelection` in the existing engineering graph and moves its adapters into
`recorded-result-adapters.rs`. Making every engineering native adapter unconditional would
compile stale native tuples while science changes source shapes, defeating source-to-generation
bootstrap. The earlier broad root addition is therefore an intermediate state, not the target.

**Target.** Keep one source declaration and one engineering descriptor world. Separate pure
message/validator/descriptor availability, the ordinary recorded-result adapter closure, and
service bindings/adapters. Emit the recorded closure with
`native_adapters_scoped(..., AdapterVisibility::Crate)` and remove it from the broad service
adapter roots. Give each generated native implementation one compilation owner. Generate its
provenance and ownership inventory with the artifact. Native source ownership and outgoing-only
construction remain intact; no inverse to current permission or live custody is introduced.

For additional consumers, declare a small number of actual projection groups at the composition
owner: selected roots, direction/construction role, destination and visibility. Generate group
artifacts from those declarations. This consolidates repeated root/publication/provenance wiring
without inventing a universal policy catalogue or selecting every domain type for every consumer.
Keep domain facts in their scientific/operation owners, not in generic `pse-contract`.

**Alternatives.** Enabling `engineering-service` for ordinary exports couples a storage/read
capability to transport dependencies and preserves the bootstrap problem. A second handwritten
wire model duplicates source meaning. A separate descriptor world is justified only if it has
an independent interpretation/evolution lifetime; module visibility alone does not justify one.
A new domain-contract crate could be worthwhile if the current source declaration dependencies
continue to pull the full runtime into generation, but requires an explicit crate/ownership
decision and a whole dependency comparison, not a mechanical file split.

**Library fit.** Buffa-codegen 0.9.2 supplies `CodeGenConfig`,
`SharedCorpusContext::new`, `shared_corpus_context`, feature gating and `apply_companions`.
The existing `ProjectionGeneration` already establishes shared corpus state; preserve it rather
than rebuild an index for each projection group. Exact source checks both corpus equality and
pointer/oneof representation inputs. `apply_companions` is an insertion helper, not complete
packaging admission: it can leave orphan companions and repeated application duplicates includes.
Retain `Suppliers::admit` path/collision checks and explicit ownership where these are still needed.
Do not replace the existing typed composition with a new generic plugin framework.

**Migration/deletions.** Remove the recorded root from broad native adapter emission, service-only
gating of the required pure facade, redundant per-group publication facts and superseded manual
lineage maps after the real consumer moves. Preserve explicit historical metadata interpretation
where durable artifacts require it; it is a bounded reader obligation, not a second current writer.

**Acceptance scenarios.** Ordinary export and independent passive inspection compile without
service activation; the service also compiles without duplicate trait implementations. Add one
source field and regenerate before compiling its native adapter; the generator must remain usable
while the old adapter is stale. Unknown context cannot issue permission. One root/rule change
must reach the actual export, descriptor, validator, reader and provenance.

**Principles:** AP-01–AP-06, DP-01, DP-15–DP-17, DP-24; heuristics H2, H4, H12, H19.

<a id="CR02"></a>
## CR02 — Preserve bounded and segmented encoding through validation and ownership wrappers

**Proposed; concrete library capability currently narrowed by integration.**

**Observed.** Main's `rpc/outgoing.rs::Validated` and
`rpc.rs::AccountedResponse` implement `Encodable::encode` only. ConnectRPC 0.9.1's default
`encode_segments` calls that contiguous encoder. The generated streaming dispatcher invokes
`encode_segments`, but these wrappers cannot forward an underlying segmented implementation.
`AccountedResponse` also checks `maximum_bytes` after encoding has allocated its output.
The present owned `Vec<u8>` response payloads do not themselves earn segmentation; changing
only the wrapper would not establish a copy reduction.

**Target composition.** Preserve validation, complete-message byte admission and retained payload
ownership while supporting the supplier's complete response abstraction. For selected large IPC
or original-file responses, compare an owned response, a buffer-backed generated view and
`PreEncoded` over already admitted bytes. Make validation/admission operate on the same immutable
message realization before publication. Delegate `encode_segments` where the body can actually
share large `Bytes` or view-backed fields; retain the contiguous route for small owned messages.
Every emitted segment must retain the actual `PayloadOwner` until its last consumer releases it.
Check aggregate segment length with checked arithmetic against the original complete-message
allowance before exposing the first segment.

Use Buffa 0.9.2 `Message::try_encode_bounded_with_cache` or
`SizeCachePool::try_encode_bounded` for suitable owned boundaries. The exact implementation
computes size, applies the protobuf ceiling and requested budget, then writes using that same
size cache. This can replace a separate size pass plus an unbounded encoder; it does not bound
earlier native mapping, validation or heap use. Pool working state at the actual sequential
encoder/request owner, not in a mutable process-wide schema cache.

For buffer-backed output, Buffa's `Rope::with_backing` and ConnectRPC's generated view encoders
already implement reference-counted segments. ConnectRPC-codegen 0.9.0 supports
`Options::encodable_impls = EncodableImpls::AllMessages` when owning-crate placement is needed
for non-service message views. Its default output-only generation may already suffice; select
the additional population only for an actual external consumer.

**Alternatives and limits.** A compact owned response may release a much larger request buffer
and be better than a retained view. Compression and some interceptors flatten segments. Bare
view responses are proto-only; `PreEncoded` JSON support decodes and reserializes. Keep the current
binary-gRPC-only service policy unless a separately justified client requires another protocol.
Do not use `PreEncoded::from_bytes_unchecked` as a substitute for world/type validation. Source
inspection establishes the narrowed capability and post-encode check order, not a measured
performance defect or a demonstrated current overrun of the surrounding admission allowance.

**Acceptance scenarios.** Exact-fit and one-byte-over outputs produce either the whole admitted
message or no publication. An invalid outgoing rule refuses before the first segment. A stalled
client retains all relevant reservations; cancellation releases them only after actual body/native
settlement. Compare a large payload, small summary of a large request, compressed response and
late nested failure through producer→encode→transport→client consume. Preserve original failure
details. Measure complete lifetimes before claiming memory/latency improvement.

**Principles:** AP-02, AP-07, DP-13–DP-16, DP-19–DP-23; H6, H16–H20.

<a id="CR03"></a>
## CR03 — Make semantic evolution an actual CLI/client capability

**Proposed; complete an existing semantic checker through a real consumer.**

**Observed.** `pse-host/src/schema/compare.rs::compare` compares structural records and encoded
option bytes; native classifications use Buf categories. Its limitations correctly say these
do not establish semantic equivalence. `cli/investigation/schema.rs` publishes those observations,
but the inspected schema owner does not invoke `pse_contract_proto::compare_descriptors`.
The shared native checker and source-derived coverage already exist in `pse-contract*`.

**Target.** Add a separately named semantic assessment to selected schema comparison and
independent engineering clients. Resolve each input against its own immutable interpretation
policy and source-selected coverage, invoke the existing checker, and report Compatible,
Incompatible or Unknown with original subject/category details. Keep structural/Buf classifications
alongside it; neither overrides the other. Imported arbitrary schemas without a known native
policy remain observational/Unknown. This completes A02/A03/B01/G02 rather than creating a second
semantic engine or reopening B04's no-dynamic-validator decision without a new consumer.

Keep the independent host product-free. Host worlds use their own source-selected policies;
engineering/scientific normalization stays with its product owner and can return an original
typed assessment through an explicitly prepared inspection/service boundary. Descriptor metadata
alone cannot supply a missing domain normalizer. Do not import the product runtime into the host,
silently launch it, recompile source or adopt a foreign descriptor pool to make this feature work.

Matching a compiled runtime world may ignore only genuine `SourceCodeInfo`, as the current
`same_runtime_world` contract requires. Preserve original artifact digests and unknown options.
Share immutable parsed compiled worlds at their existing generated owner; `Bundle::compiled_world`
currently decodes embedded descriptor sets during matching. Reuse can remove repeated static
decoding, but must never reinterpret old descriptors through today's pool or use filename/pointer
equality as semantic identity. No global expanding descriptor registry is required.

**Alternatives.** Retaining the current structural-only tool is honest but leaves a ready semantic
capability inaccessible to the intended independent user. Byte-identical options are too strong
for presentation changes and too weak when the original interpretation is unavailable. A separate
CLI-authored table would duplicate the source-derived policy and is rejected.

**Acceptance scenarios.** Compare wire-identical unsigned/signed or ordered/set native meaning
changes; presentation-only changes; stripped annotations; absent original interpretation; changed
effective features/imports; unknown options and two equal-name foreign pools. Missing coverage
must not become vacuous compatibility. Historical semantic read success must not create current
permission, restoration or settlement authority.

**Principles:** AP-04–AP-06, DP-01–DP-04, DP-09, DP-15, DP-21–DP-24; H12, H18–H20.

<a id="CR04"></a>
## CR04 — Finish mechanical operation cutovers through typed native contracts

**Proposed; remaining accepted migration with a stronger extension target.**

**Observed.** `engineering/rpc/wire.rs` still translates request fields, IDs, intent/backend
enums, progress alternatives and result pages manually. Examples include `prepare_from_pb`,
`submit_from_pb`, `progress_value_to_pb` and `page_to_pb`. Some parse errors become request
strings or a generic unsupported-intent message. Adjacent generated adapters and structured
projection diagnostics already provide a more consistent mechanism. The remaining maps are a
finite actual consumer population, not evidence that the whole source graph should become proto-first.

**Target.** Annotate the actual source request/fact owners and generate mechanical crossings,
including nominal IDs, missing/default distinctions and unknown alternatives. Keep original-parent
membership, scientific interpretation, provider readiness and exact operation identity in their
existing fallible owners. Carry the original projection failure category/field path through the
shared diagnostics instead of translating it to prose at each method. Derive native/JSON/proto
local constraints once; retain distinct storage/Arrow formats for distinct consumers.

Bring useful operation facts together at their existing service composition: request/response
owner, declared effects, applicable local validation and bounded response policy. Generate
client discovery/constraint explanation from them. This can remove response helper repetition
and parallel client copies while leaving authorization and contextual readiness native. Do not
infer idempotency from unary shape, method spelling or `read_only` alone.

**Alternatives.** A generic `DynamicMessage` adoption layer for all operations moves domain
interpretation into a new runtime and weakens local reasoning. Retaining manual translators is
reasonable for genuinely semantic transforms, but each must name that transformation and why
the mechanical generator does not own it. Do not label every remaining map mechanical merely
because its code is short.

**Acceptance scenarios.** Add one request alternative and one diagnostic rule through their native
owners; independent clients discover both. Exercise unknown alternative, invalid ID extent,
explicit false/zero/empty, omitted optional, stale parent and outgoing-invalid mapping. Rejection
leaves the parent and original operation unchanged. Delete replaced maps/tests/fixtures after
their actual callers migrate; native admission and independent expected meanings remain.

**Principles:** AP-01–AP-06, DP-01, DP-03, DP-13–DP-17, DP-21, DP-24; PS-01/PS-12.

<a id="CR05"></a>
## CR05 — Consolidate library task accounting only where complete settlement survives

**Proposed; retain the native settlement seam, selectively reduce async bookkeeping.**

The current engineering server already uses Tokio `JoinSet`, drains original native work before
bounded transport shutdown, and aggregates independent connection failures. Scientific transport
uses `OwnedExecutor`, an `Arc<Mutex<Vec<JoinHandle<()>>>>`, repeatedly taking and joining batches
because Hyper tasks can create further tasks. It separately retains an authenticated socket read
side until actual peer EOF. These are materially different obligations.

Tokio-util 0.7.19 `TaskTracker` is a credible replacement for retaining successful task handles
where only completion accounting is required. Its `close()` enables `wait()` but does not prevent
new tasks. `wait()` observes tracked future destruction; it does not prove socket EOF, child-process
drain, native thread completion or durable publication. A tracker also does not preserve task
panic/results by itself. Any selected replacement must retain typed panic/error collection and
an explicit end of task creation. Preserve current complete joins where introducing that machinery
would cost more than the loop it removes.

Do not introduce `tokio-graceful-shutdown` merely to replace a flat task set. Its hierarchy is
eligible when several actual service subtrees need independently owned shutdown/failure policies;
that broader design was screened, not source-qualified here. Likewise `tokio-rayon` can bridge
CPU work into async code, but dropping the receiving future does not establish physical completion
of opaque computation. Keep native finish/cancellation ownership and the configured host resource
policy. No new CPU partitions, job caps or global task manager follows from this review.

**Target/deletions if selected.** One transport-local task accounting owner using the library
tracker plus retained typed failure collection can replace the mutable successful-handle vector
and bespoke batch bookkeeping. Keep OS peer authentication, retained EOF custody, native settlement
and original failure aggregation. A replacement that only awaits an empty tracker is insufficient.

**Acceptance scenarios.** A task spawns a child during shutdown; one task panics; a peer sends final
H2 frames after logical completion; native cleanup independently fails; a waiter departs; and a
blocking computation continues after async cancellation. Every required owner must physically
settle and every original independent failure remain observable. Current passing historical
controls are requirements to preserve, not new review results.

**Principles:** AP-03, AP-06–AP-07, DP-13, DP-19–DP-23; PS-11; H4, H17, H21–H23.

## Sequencing, scope and evidence

CR01 is the immediate dependency for the paused result consumer and scientific source expansion.
CR04 should migrate coherent request/response families with their real independent client; it need
not wait for every family. CR03 is a ready diagnostic consumer of the shared checker. CR02 then
selects the expensive boundaries that actually benefit from buffer-backed data. CR05 is conditional
on a simpler complete lifetime realization, not a mandatory runtime rewrite. These amendments
belong with the remaining 65a/b/c/d/e/f/g/j and 64i/j/k consumers; they do not close their packages.

Keep the real scientific and developer journeys as integration threads. Scientific observations
must expose original coordinate/unit/basis/datum/fixture meaning and actual winning selections;
no codec, preparation record or eligibility count establishes stability. Source/worker/native-root
decoding must not gain execution permission. Developer readback/rerun must bind the original
inventory, configuration, artifacts and failure. Independent native/reference expectations remain
separate from generated round trips.

For test organization, keep cheap contract-policy tests local and retain necessary generated-code
compile and actual provider/service controls. The current `xtask` integration imports real runtime
owners, so a pure toy graph cannot replace its integration evidence. Distinguish those claims rather
than making every metadata question rebuild native workflows or replacing all integration tests
with mocks. Freeze a coherent source/dependency snapshot before required receiving checks; avoid
invalidating its source witness while it compiles. The compiler-bridge context refusal after a
successful preparation remains an unresolved paused observation for the preparation review lane,
not a diagnosed library defect here. No build-cache or host-policy changes are recommended.

## Coverage appendix

All fourteen Proto and ten runtime overview documents received capability screening by introduction,
scope/version and headings. Selected sections were read in depth where named above. Screening is
not API qualification, library selection or complete assessment of every API.

| Proto inventory document | Review use and depth |
| --- | --- |
| `Buffa.md` | Focused: codegen composition, owned/view encoding, descriptors, budgets and feature boundaries; exact maintained source checked for named APIs. |
| `ConnectRPC_for_Rust.md` | Focused: response abstractions, generated ownership, limits, protocol/codec restrictions; exact 0.9.1/0.9.0 source checked. |
| `Local_RPC_and_Diagnostics_in_Rust.md` | Focused: local identity, stream lifetime, encoding and release corrections; transport development-only claims excluded. |
| `protobuf_validation_rust.md` | Focused orientation and maintained validator README; existing strict lowerer retained. Alternative runtime/build validators screened, not adopted. |
| `protobuf_generator_ecosystem_rust.md` | Screened packaging, feature graphs, external mappings and multi-output generation; no new generator selected. |
| `protobuf_compilation_and_orchestration_protox_protoc_and_buf.md` | Screened descriptor/image/compiler roles; Buf compatibility kept distinct from semantic interpretation. No compiler substitution qualified. |
| `protobuf-grpc-installed-tools.md` | Screened available independent tooling; installation inventory is not deployed interface evidence. |
| `Rust_first_proto_contracts_and_proto_rs.md` | Screened released 0.11.26 versus development 0.12 material. No new shadow runtime or transport selected. |
| `prost.md` | Screened alternate codec/build capabilities; not a reason to add a second message runtime. |
| `prost_reflection_and_pbjson.md` | Screened reflective/JSON/WKT alternative surface; no new adapter or dynamic validator selected. |
| `tonic.md` | Screened interoperability/transport alternative; its 0.14.8 overview and adjacent 0.14.9 corrections are not exact repository qualification. |
| `rust_protobuf_ecosystem_overview.md` | Broad capability map; hypotheses only. |
| `Protobuf_SurrealDB_v3.3.md` | Screened native value/gRPC/query/transaction distinctions; no storage or changefeed redesign concluded in this lane. |
| `Protobuf_SurrealDB_v4.md` | Screened to identify release boundary; 4.0-only database capabilities excluded from adoption recommendations. |

| Runtime inventory document | Review use and depth |
| --- | --- |
| `tokio.md` | Screened tasks, I/O, blocking and lifecycle; repository pin is 1.53.3, overview targets 1.54.1. No upgrade inferred. |
| `tokio-util.md` | Focused task tracking/cancellation; exact pinned 0.7.19 tracker source checked, overview targets 0.7.20. |
| `bytes.md` | Focused buffer ownership/retention orientation; actual `Bytes::from_owner` use inspected. No allocator/RSS guarantee inferred. |
| `futures.md` | Screened stream/sink composition and settlement alternatives; no general stream rewrite selected. |
| `rayon.md` | Screened scoped CPU composition and nested execution; no host pool changes proposed. |
| `tokio-rayon.md` | Screened async CPU handoff and cancellation distinction; no upstream source deep dive or adoption change. |
| `tokio-graceful-shutdown.md` | Screened hierarchy versus flat task sets, shutdown limits; not an installed-source deep dive or selected dependency. |
| `zbus_and_zbus_systemd.md` | Screened typed OS custody/control; root's native-manager changes remain separately owned. |
| `sd-notify.md` | Screened readiness/watchdog/socket integration; readiness cannot substitute for native admission or settlement. |
| `cdylib.md` | Screened ABI, handles, callback/panic/loading lifetime and Arrow C interfaces; no new portable/native execution supplier qualified. |

Promising leads left uninvestigated include independent health/reflection service policy, gRPC
conformance tool integration, alternate WKT/JSON runtimes, native database gRPC, hierarchy-based
service shutdown, socket activation and alternate dynamic-library ABI suppliers. They require
named consumer contracts before adoption. This bounded supporting review does not claim complete
upstream capability verification or independent coverage of the scientific/persistence lanes.

## Primary-source grounding and release limits

Context7 was consulted first for Buffa, ConnectRPC and Tokio-util questions. Buffa and Tokio-util
returned relevant material; ConnectRPC resolution returned Go/general protocol documentation, so
Rust claims rely on the installed exact Rust packages, not those search matches. Context7's Buffa
results point at development/main documentation; named mechanisms were confirmed in the exact
maintained 0.9.2 source before recommendation.

- [Buffa bounded encoding, exact maintained source](../../vendor/buffa/src/message.rs),
  [size-cache ownership](../../vendor/buffa/src/size_cache.rs),
  [rope sink](../../vendor/buffa/src/encode_sink.rs), and
  [maintained accounting limits](../../vendor/buffa/pse-patch.md).
- [Buffa-codegen exact maintained source](../../vendor/buffa-codegen/src/lib.rs), particularly
  `SharedCorpusContext`, `apply_companions` and `CodeGenConfig`;
  [local maintenance declaration](../../vendor/buffa-codegen/pse-patch.md).
- [ConnectRPC 0.9.1 response source](https://docs.rs/crate/connectrpc/0.9.1/source/src/response.rs),
  [dispatcher](https://docs.rs/crate/connectrpc/0.9.1/source/src/dispatcher.rs),
  [service limits](https://docs.rs/crate/connectrpc/0.9.1/source/src/service.rs), and
  [framing](https://docs.rs/crate/connectrpc/0.9.1/source/src/envelope.rs).
- [ConnectRPC-codegen 0.9.0 source](https://docs.rs/crate/connectrpc-codegen/0.9.0/source/src/codegen.rs),
  including `EncodableImpls` and `generate_services`.
- [Tokio-util 0.7.19 tracker source](https://docs.rs/crate/tokio-util/0.7.19/source/src/task/task_tracker.rs),
  confirmed against the installed source for `close`, `wait` and spawning semantics.
- [Maintained validator contract](../../vendor/protoc-gen-protovalidate-buffa/README.md): single
  prepared scan/index, strict unsupported-rule admission, integer/error behavior and owned/view
  lowering. No replacement is recommended without preserving those complete consumed semantics.

No current version, readiness or performance claim is based solely on an overview. Exact pins
remain in `Cargo.toml`; proposed upgrades require their own affected-boundary qualification.
