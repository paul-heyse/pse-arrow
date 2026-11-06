---
title: "Plan65f: Diagnostics and result delivery"
status: in-progress
date: 2026-11-03
adrs: [ADR-0224]
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md, docs/plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md]
doc_role: plan
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [design, diagnostics, validation, performance]
---

# Diagnostics and result delivery

## Target and examined foundations

**Proposed:** native diagnostic/outcome owners produce one structured meaning with generated
independent projections. CLI, local services, retained readers and external telemetry consume
that meaning without parsing display strings or independently reconstructing failure classes.
Preserve complete original evidence separately from bounded public details. Keep numerical
results in Arrow/native forms suited to their access paths; Proto carries selected metadata,
typed failures, progress interpretation and protected references.

The [coordinator](65-proto-integrated-target.md) owns combined decisions and overall completion;
F01–F06 below own this companion's work. Execution proceeds alongside unfinished Plan64;
each consumer needs its actual native prerequisite, rather than whole-Plan64 completion. Inspected source at
`f61286a5fa5894b480efe8584c8b3d9b76d1b1fb` and supplied dirty reference additions establish
**Interface-checked** mechanisms only. No reporting, publication, science or overhead campaign
ran during authoring; the execution checkpoint below distinguishes subsequent delivery.

`pse-diagnostics::observation` already holds bounded copied facts, coverage and retention;
runtime `diagnostic_documents` and `modeling::diagnostic_source_map` preserve source chains,
notes and original source occurrences. `result_projection`/`result_inspection` select native
scientific fields and protected products. Canonical result blocks own Arrow payloads and
publication. Perfetto and OTLP already have standard external protocols. These mechanisms
can be replaced by a better complete realization, but their required distinctions survive.
Blueprint §23, §19 and §20 describe the existing meaning and publication contracts.

## Execution checkpoint — 2026-11-04

Actual `ScientificPermission` and `ProductPermission` now declare one-way observation
projections at their native completion owner. They retain original task/step/point,
requested/observed coverage, original outcome, refusal/qualifier facts and independent
`LocalValidity`. The latter's shared registry record carries explicit wire tags; these
presentation facets preserve its native semantic and encoding identities. Generated adapters
borrow the original facts, validate outgoing local rules, and expose no inverse constructor
for either permission snapshot. Whole-completion, selected-attempt, service/CLI population
and the remaining diagnostic categories are still required.

**Tested, targeted on main:** `scripts/pse test pse-runtime --lib --features
canonical-tests,engineering-service -E 'test(scientific_snapshots_preserve_governing_scope) |
test(result_metadata_projection_tests) |
test(canonical_durable_constant_completion_reopens_exact_manifest)'` passed 6/6, zero
failures/pending/build warnings, configured unbounded mode, cache off and prepared compiler/KLU.
Original capture and fixture/native settlement completed. The new control independently
decodes actual completion-derived facts through Buffa `DescriptorPool`/`DynamicMessage`, with
literal tags and expected decisions: qualified-unclosed remains qualified, failed or missing
original scope cannot be promoted by certified local mathematics, and point identity and
signed-zero bits survive. The schema control
`scripts/pse test pse-schema --lib -E 'test(local_validity_wire_presentation_preserves_native)'`
passed 1/1 with zero failures/pending/build warnings. These targeted results close no package.
Buffa's independent read interface is characterized in its
[pinned dynamic decoder source](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs).
Developer full-evidence delivery and its remaining lifecycle population are owned by
[65j](65j-workflow-lifecycle-and-developer-evidence.md); protected scientific reads by
[65c](65c-storage-projections-and-history.md).

The November 5 retirement integration adds source-owned keyed multi-cause outcomes through
kernel banks, mathematical suppliers, retained adapters and worker protocol projections.
Original action failure and independent cleanup/panic-payload destruction failures survive
composition; wrapper attribution occurs once at its owning bank. Unknown local retained
extent remains conservative, and opaque wire observations cannot restore native authority.
[65e's retirement contract](65e-scientific-portability-and-execution.md#transactional-handoff-and-explicit-placement-finish--november-5-amendment)
owns exact transport/source references, Hyper's erased-error limitation and lifecycle acceptance.
This advances F02's failure population without completing general diagnostic delivery or
whole-completion/service/CLI adoption.

## Selected contract and presentation design

Define separate source-owned diagnostic, observation, scientific outcome and result-description
records. They are related by typed identities, not merged into one universal status envelope.
The pure contract declarations and typed metadata from [65a](65a-contract-foundations-and-generation.md)
generate wire and selected native/JSON presentation from those owners.

The diagnostic projection carries stable code, category/severity/stage, affected scientific
and operation identities, original native status when present, source references, related
causes and selected notes/help. A source occurrence, shared expression identity, display path
and query coordinate remain different fields. Missing source correspondence is unavailable
information, not fabricated coordinates. Non-UTF-8 native paths retain their byte representation.

The public representation separates:

| Record | Interpretation |
| --- | --- |
| Native failure/outcome | Original category, causes and scientific status. It can be rendered locally without remote serialization. |
| Trusted full diagnostic | Complete selected violations/related evidence under the owner's retention/custody rules. Validation compiler/evaluator failures have their own categories; they are not ordinary violations. |
| Bounded detail | Summary and an explicit complete/partial/unavailable state, omitted extent where known, and retained-evidence locator where one exists. Unknown original counts remain unknown. |
| Protected evidence page | Original selection, issuer/world/operation binding, page extent/continuation and coverage. A locator does not bypass protection or guarantee indefinite retention. |

Use ConnectRPC 0.9.1 `ConnectError::with_detail` with `ErrorDetail::from_message` for typed
protocol details; `with_source`/`with_shared_source` retain original local causes and do not
serialize them. Standard `google.rpc` messages are used where their distinctions fit:
`BadRequest`, `ErrorInfo`, `PreconditionFailure`, `ResourceInfo`, `RequestInfo` and `Help`.
`RetryInfo` suggests delay but never grants permission to repeat scientific effects. Generate
compatible Buffa details rather than adding a second Proto runtime just for standard messages.
For external Tonic consumers, independently versioned `tonic-types::StatusExt::check_error_details`
is the fallible option; malformed recognized detail and absent/unrecognized detail differ.

Do not depend on a bounded public validation message to retain the original map-entry selector
or all violations. The outer projection records scope/omission and a full-evidence reference.
Generated field/rule paths map to source through actual declaration relationships. Validation
filter, policy identity and logical clock context from [65b](65b-representation-validation-and-interpretation.md)
must accompany results when they change interpretation. Filtered success is not full validation.

## Selection, delivery and observability

### Scientific products, developer evidence and native authority

[65i](65i-scientific-operations-and-product-semantics.md) owns scientific product-use decisions;
[65j](65j-workflow-lifecycle-and-developer-evidence.md) owns developer evidence and retained
closure. F01's actual population includes native action, selection/inventory, fixture,
assertion, capture and settlement observations. A passing assertion expecting scientific
refusal remains a passing test, not successful science. Listing and compiler-only checks
retain their own outcome meanings. Scientific metadata carries task/step/point scope, local
validity, governing coverage and final selected attempt; F must not reconstruct permission
from row presence or a summary status. Raw evidence and scientific-use selectors are explicit.

F owns faithful presentation. `Report::publish`, `managed::workflow::validate_report` and
`TerminalResult<T>` retain original attribution and native custody. Generate one-way
observations without inverse constructors for verdict/readiness/publication/settlement.
The selected [ConnectRPC error implementation](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/error.rs)
supplies `ErrorDetail::from_message`, `ConnectError::with_detail` and local `with_source`:
use typed wire detail for recorded facts and preserve Rust causes locally. These operations
do not authenticate a performing report or renew a scientific grant. Buffa generated local
validation similarly establishes record rules, not native authority.

F03 reads developer evidence through its existing report/inventory owner without requiring
canonical database migration. Scientific products continue through C's manifest/protection
owner. Both expose family-specific absent/expired/incomplete/incompatible/corrupt outcomes;
bounded views identify omissions and do not manufacture complete evidence.

Push field/product/detail selection to the performing producer before native hydration, copying
or encoding. Retain the original full route when a consumer requests full evidence; do not
implement full reads as repeated public pages that each reparse/reconstruct the same source.
Result metadata identifies axes/time/layout, physical quantities, completeness, qualification
and Arrow payload protection. Do not replace columnar arrays with repeated generic values.

### Complete selections and original evidence acquisition

The November 5 review [I06](../plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md#I06),
[I07](../plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md#I07)
and [I12](../plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md#I12)
extend the remaining F population. Selection intent, governing attempt, scientific coverage,
examined scope, returned extent and terminal completeness must survive producer, canonical
reader and independent presentation. Preserve the implemented F04 readable-root assurance:
an output filter is not permission to omit global scalar keys or mandatory scientific inputs.
Consume I's producer-owned experiment/trajectory/phase coordinates rather than infer
scientific permission from stored rows or projected labels.

F03's first large developer-evidence consumer is offline protected acquisition and staged
bundle export from J04. A bounded metadata response names the original ordinal/role/extent/
digest and recorded omission commitments; selected members are acquired incrementally.
The current compact input/output profile remains bounded at 64 MiB. Both physical profiles
consume one interpretation-three commitment model and performing-report attribution.
Do not copy the full original closure into a Proto message merely to render bounded detail.
Scientific result protection remains C-owned; no developer report migration into that store
is implied. A service chunk route enters D06 only when an independently selected client
requires it, not as a prerequisite for offline delivery.

Use Tokio-util 0.7.19 `ReaderStream`/`StreamReader` where a consumed I/O boundary needs
stream adapters and Bytes 1.12.1 `Bytes::from_owner` where backing ownership must outlive an
await. They do not supply original-file pinning or publication atomicity; small slices may
retain a larger allocation. Buffa 0.9.2 `OwnedView`, `ViewEncode`, `SizeCachePool`
and segmented encoding apply only after producer selection, with actual backing retained.
Compression or interceptors may flatten segments; measure the complete producer-to-release
route before claiming benefit. Sources:
[Tokio-util I/O](https://docs.rs/tokio-util/0.7.19/tokio_util/io/index.html),
[Bytes ownership](https://docs.rs/bytes/1.12.1/bytes/struct.Bytes.html),
[Buffa encoding](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/encode_sink.rs).

F02 must represent independent operation, native finish, reap and I/O-join failures without
flattening original causes or treating unknown settlement as successful cleanup. Use
ConnectRPC 0.9.1 `ErrorDetail::from_message`/`with_detail` for bounded recorded facts and
`with_source` for local causes; the [error source](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/error.rs)
defines that separation. A detail reference is not a readable-file capability.

F03/F04 closure includes a bundle above 64 MiB, slow or escaped readers, corrupt/missing later
members, original omissions followed by later file appearance, failed final publication,
empty selection, late terminal error and independent release failure. J04 must establish the
actual acquisition/release owner first: current InvocationStore has no general prune/dispose
operation. Delete replaced full materialization, copied field maps and reference/cleanup
translations with complete migrated consumers; neither codec replacement nor an honest
size refusal alone establishes ordinary-workload fitness.

### Selected hydration and inspection assurance — 2026-11-04

**Selected target; Interface-checked:** F04's readable-value inspection uses the existing
protected snapshot owner with Arrow 59.3
[`FileDecoder::with_projection`](https://docs.rs/arrow-ipc/59.3.0/arrow_ipc/reader/struct.FileDecoder.html#method.with_projection)
before native array hydration. Derive one retained-root set from requested paths, existing
context behavior, all scalar primary keys, all bound relation-check dependencies and the unchanged
scientific-permission algorithm's input roots. Apply the same projection before that algorithm's
first read; compare a cursor against its resolved selection without a second full read. A nested
selection retains its complete root and current composite-context behavior. Do not route through
DataFusion's current snapshot scan merely to obtain projection: that path already hydrates and
exports a full checked batch before projecting.

Reuse `AdmittedFile`'s complete source schema/footer/block/layout checks, original snapshot
protection, cancellation/close/failure machinery and pre-allocation grants. Construct each native
decoder's projection per read; a projected decoder must not contaminate a later full read.
The existing `ValidationContext::column` and `PreparedLocalContract::evaluate` validate every
retained complete root. The original `row_checks::bind` predicates retain all dependencies via
DataFusion 55.1 [`Expr::column_refs`](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/enum.Expr.html#method.column_refs)
and all predicate evaluation. Bind/refuse before reading if that closure cannot be determined.
The existing scalar-key algorithm checks declaration-bound keys over the entire original batch
before slicing returned rows. Reuse these owners; no handwritten IPC parser, synthetic relation
or arbitrary-subset `FieldCheckedBatch` is introduced.

Expose the actual inspected check coverage on the native observation. Omitted roots' local
values are unchecked; original-byte hashes alone do not establish their semantic validity.
Only the affected inspection request/response roots move to version three, with active callers
migrated and superseded versions refused before body decoding. Full IPC/scientific reads keep
their full-relation admission and permission boundaries. Protected values can still be useful
observations without granting scientific use. The proposed
[ADR-0224 amendment](../adr/0224-native-result-inspection.md#plan65-selected-hydration-amendment--2026-11-04)
records this assurance change before implementation; formal decision status remains proposed.

The complete selected IPC block is still read and selected arrays may retain its backing buffer.
Keep decoded-byte accounting honest; there is no measured memory/throughput benefit claim.
Targeted acceptance extends admitted-file and canonical inspection controls for selected roots,
full/projected/full reuse, selected malformed data, omitted-field assurance, predicates/keys
outside the returned window, nested/context/cursor semantics, refusal before read and unchanged
full/scientific admission. Source contracts are in
[IPC admission](../../crates/pse-columnar/src/ipc_file.rs),
[column validation](../../crates/pse-relations/src/validate/prepared.rs),
[row-check binding](../../crates/pse-relations/src/validate/row_checks.rs) and
[scalar-key validation](../../crates/pse-relations/src/result_batch.rs).

Main `648b0b7db` integrates this slice from reviewed `2abec778c`. Native receiving tests pass
26/26 across IPC/relation admission, canonical inspection, header refusal, full-reader range/
cancellation, empty demand, escaped buffers/reclamation and Stable invocation scope, with
zero failures/retries/build warnings and complete native capture/fixture settlement. The route
is `scripts/pse test pse-columnar pse-relations pse-runtime --lib --features
pse-runtime/native-study-tests --profile local` with the named control families above, configured
unbounded/cache-off and prepared compiler/KLU/solver. The isolated source had independently
passed 8 component, 17 runtime and one scope control plus all 14 generation targets.
Complete per-read projected schema storage is admitted before body reads; private batches retain
working leases even when no array buffers exist. Omitted local values remain explicitly unchecked.

All seventeen integrated files matched the worker commit before clean, quiescent worktree
removal; original invocation evidence remains outside it. Union regeneration and the requested
Clippy/functional consolidation are tracked in the coordinator. This delivers the selected
hydration slice, not every F04 producer/client population or measured benefit.

Use Buffa eager views/`OwnedView`/`ViewEncode` and segmented output only where end-to-end
ownership removes materialization. Lazy views lack some owned/reflection/text capabilities
and may repeat decoding; they are not the default page representation. A compact copy can
release a large backing allocation. Compression, JSON and middleware flattening must be
included in the physical comparison. Decoder limits and live retained memory are separate.

Progress remains an observation of the original operation. Final scientific outcome, durable
acknowledgement and actual native settlement have separate performing owners. Preserve a
valid partial prefix and terminal failure/extent; a stream reaching EOF does not prove a
complete result. Observation expiry, disabled collection and dropped details are explicit
coverage states and do not cancel work or establish scientific completion.

Correlate existing request, invocation, operation, producer sequence, result and trace/span
identities without minting a competing universal ID. Retain standard OTLP and Perfetto schemas;
their exporters consume selected shared metadata but preserve timestamps, extent and protocol
rules. Generate a selected fact vocabulary and sink adapters where current string-based
record families/units duplicate native meaning. Do not encode every primitive event or retain
every full diagnostic solely for observability.

Use field sensitivity metadata and Buffa `debug_redact` to protect routine generated Debug
representations. Redaction of Debug does not redact ProtoJSON, text or binary inspection;
explicit payload inspection therefore requires its own authorization and display policy.
Optional Tokio-console remains an explicitly prepared local task/runtime diagnostic route,
using the required Tokio tracing configuration. It is not a completion or cancellation owner.

### Semantic RPC failure delivery — 2026-11-04

Main `0e36b4863` integrates the semantic service/client failure population. The native
`semantic::Error::Rpc` retains the actual `connectrpc::ConnectError` and its local source;
an output-only `RpcObservation` records interpreted details without granting native refusal
or settlement authority. Absent, unrecognized, malformed, conversion-refused, ambiguous,
oversized and recognized native details remain distinct. A transport `InvalidArgument`
alone cannot become a native validation refusal. Native diagnostic identity, canonical
validation correction/selectors and failure state/phase survive the CLI and investigation
projections; the duplicate JSON RPC-detail shadow is removed.

The selected [ConnectRPC 0.9.1 error functionality](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/error.rs)
provides original status/details and `with_source` for a local cause. The producer constructs
a fresh public error from an owned native error and explicit transport code/message; it does
not reuse arbitrary inherited details. Buffa decoding and canonical violation conversion
remain fallible interpretations, separately inspectable from that original cause. Explicit
caller redaction covers native codes and canonical validation strings. Public recursive
diagnostic-source/related/aggregate traversal omits raw RPC causes while preserving independently
captured sibling facts and causes. Generated developer-evidence output carries the same passive
`ErrorRecord.semantic_rpc` meaning; no wire observation reconstructs a native error capability.

Twenty-one selected controls passed before main integration, including real HTTP/2 transport,
literal independent wire inputs, dynamic descriptor readback, malformed/unknown/oversized
details, canonical corrections and recursive privacy. The host all-targets check passed with
zero warnings; original capture and settlement completed. Full main receiving `just codegen`
passed all fourteen targets; the CLI catalogue and generated source provenance changed,
while the already integrated developer-evidence output stayed current. The known Buffa
reserved `into` accessor notice is distinct from Rust compiler warnings.

Main `4c8130da1` fixes a further routine-disclosure gap in the copied semantic failure:
opaque backend JSON strings and keys now obey the same explicit caller redaction policy,
including nested cancellation failures. Unchanged public keys are reserved before redacted
keys; visibly generated collision suffixes preserve every value. Native errors, original RPC
details/local causes and raw wire-codec round trips remain exact. This applies to ordinary LSP
refusals as well as recognized native RPC details, without changing explicit payload/source
inspection. Main `0c8f48b4c` also removes the dependency-usage consumer's conversion of a
typed semantic error into ownership text. Its terminal projection can now retain the selected
RPC observation and original diagnostic category.

Sixteen targeted host controls passed on main with zero failures/build warnings, configured
unbounded/cache-off mode and complete original capture/settlement. The selection includes the
full-envelope opaque-disclosure regression, existing RPC/failure controls and dependency-usage
delivery. Two initial unscoped launches refused manager attachment before building; the actual
user-application-scope route supplied the passing evidence. Main
`scripts/pse check pse-host --all-targets` also passes with zero errors/build warnings and
complete original capture/settlement in cache-off, unbounded mode. These results cover this semantic family, not every F02 producer,
scientific result or F06 assembled journey.

Main `770923209` also removes a native private-channel classification gap. The compiler
probe and runtime now share
[`decode_resource_refusal`](../../crates/pse-contract-proto/src/support.rs): Buffa's
message/depth/unknown/element limits and the maintained backing-allocation limit, layout
overflow and reservation refusals retain their original typed errors and report resource
failure. Malformed wire and an unaccounted view remain distinct. The exact extension variants
are characterized by the maintained
[Buffa error source](../../vendor/buffa/src/error.rs); upstream's
[decode-options guide](https://github.com/anthropics/buffa/blob/main/docs/guide.md)
characterizes the standard decode limits. This classification grants no native or scientific
admission. The shared project handler now owns large drain errors behind their concrete boxed
type, retaining original action/source and related-drain relationships rather than flattening
either into text. Receiving resource/cause and aggregate-error controls are part of
[65h's consolidation qualification](65h-integration-and-qualification.md#consolidation-qualification--2026-11-04);
the full remaining F02 population stays open.

The consolidation also removes the engineering service's three hand-written source,
contract and effect response maps. RPC callbacks now call the existing source-derived
`engineering_projection::encode_*` adapters, then retain the same `AccountedResponse`
and outgoing-envelope validation owners. The generator's native-to-Buffa projection
mechanism is characterized in
[`boundary_projection`](../../crates/pse-codegen/src/codegen/boundary_projection.rs);
the actual consumers are
[`engineering/rpc.rs`](../../crates/pse-runtime/src/host_operations/engineering/rpc.rs).
Operation snapshots share the original native package/preparation/study through `Arc`;
stored result cursors retain an allocation lease acquired before boxing. This removes
repeated interpretation and native payload cloning without claiming exact allocator/RSS
accounting or a measured memory benefit. Final receiving tests are owned by the same H
checkpoint; these changes do not complete the remaining F04 producer/client population.

### Original arithmetic refusal fidelity — November 6

**Implemented and Tested in the focused receiving selection:** original output-accuracy assessment now retains
the reason exact arithmetic is unavailable through the existing budget-owned `GoalFailure`.
The mathematical owner supplies one `PointArithmeticAssessment` implementation. Its typed
unavailable result identifies the first failed inventory/predicate, including original
instance/source, obligation kind/scope, requested/minimum order, representation and fidelity.
NIL provenance is explicitly missing; an unrepresented guard can still have known provenance.
Inventory refusal retains its first original auxiliary, including owner, opacity role,
bounds/fidelity and any owned function name; retained accounting includes that heap storage.
Existing optional projection consumers use that same implementation and retain their
availability semantics.

The runtime preserves original `FactorableError` causes, IBEX
`PointArithmeticEvidence::Incomplete::{reason,work}`, residual-map refusal and unusable
enclosure details. Native status alone supplies no failed-guard identity: report that
attribution as missing rather than selecting a nearby source token. Cloned goal failures
share the same retained cause/charge. The classification remains `EvaluatorUncertainty`;
no feasible point, guard check or scientific acceptance is changed. Original cancellation,
contract errors and enclosing allocation refusal retain terminal precedence. Actual native
Boundary attribution, if needed, must follow encoded-guard origins including deduplication;
static source maps alone cannot establish interval guard admission.

The native producer now exposes `Ibex::point_arithmetic_preflight` with a typed
`ArithmeticProfileRefusal`: requested order, admission stage, failed metric and exact
observed/limit or overflow values. Values beyond diagnostic signed-integer range retain
exact decimal text. Preflight reports zero native work scoped to this optional arithmetic
assessment, not to the enclosing solve. The runtime retains this original cause under
`GoalFailure` before uncertainty buffers or native preparation, preserving the original solve
result. It does not broaden optional handling of actual allocation/work/cancellation failures.
The actual exchanger now identifies 19,512 source declarations versus an 8,192 adapter cap;
none is claimed to be the native failed guard. Profile/source/order/overflow and hard-error
controls pass in the 43-test receiving selection (42 passed, one scientific failure); complete
scientific acceptance and the full F02 population remain open.

Sources: [projection owner](../../crates/pse-math/src/factorable.rs),
[runtime arithmetic assessment](../../crates/pse-runtime/src/math/solves/engineering_accuracy/arithmetic.rs),
[shared retained failure owner](../../crates/pse-runtime/src/math/solves/engineering_accuracy.rs)
and [IBEX native result/work contract](../../crates/pse-backend-native/src/root_isolation.rs).
Five passing focused controls cover predicate/order/provenance distinctions, original auxiliary
ownership/heap lifetime, native reason/work,
typed source preservation and required-error precedence. Passing these does not qualify the
remaining exchanger comparisons or the complete F02 diagnostic population.

## Packages

| ID | State | Prerequisite | Delivery and targeted acceptance |
| --- | --- | --- | --- |
| F01 | in-progress | Actual native consumer readiness; I01/J01 semantic design; 65a metadata and 65b policy identities | Completion-owned permission/local-validity and developer evidence observation slices are delivered; complete diagnostic/observation/outcome/result-description products across native diagnostics, runtime documents, producer facts, scientific product-use scope/coverage/selection and developer action/inventory/fixture/assertion/capture/settlement. Preserve every selected distinction with exhaustive mappings and independent records. |
| F02 | in-progress | F01; 65a generated adapters and 65b violations; D05/E04 explicit finish outcomes | Semantic RPC family is integrated with typed detail dispositions, original causes, canonical corrections and recursive redaction. Complete the remaining native-to-CLI/local error populations, including independent scientific action/reap/I/O-join failures and unknown settlement. [ConnectRPC 0.9.1 error APIs](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/error.rs) provide ErrorDetail::from_message/with_detail for bounded facts and with_source for local causes, not native authority. Recognized malformed, unknown, absent, multi-cause, rule/compiler/evaluator failures and oversized details must differ. Delete replaced string classification, cleanup string flattening and duplicate field maps with complete consumers; keep native rendering where consumed. |
| F03 | planned | F01/F02; narrow C scientific or J04 developer acquisition/protection contract according to family | Deliver bounded metadata/details with complete protected evidence readback. First large developer consumer is offline original-member acquisition/staged export, sharing J's interpretation-three commitments with the compact 64 MiB profile. Use [Tokio-util 0.7.19 I/O adapters](https://docs.rs/tokio-util/0.7.19/tokio_util/io/index.html) only for transfer mechanics and [Bytes 1.12.1 owner-backed buffers](https://docs.rs/bytes/1.12.1/bytes/struct.Bytes.html) where async lifetime needs them; native owners retain attribution/publication/protection. Test a bundle above 64 MiB, omitted selectors, unknown counts, missing spans, corrupt later member, slow/escaped reader, expiry and failed final publication/release. No summary issues a verdict or implies complete retained evidence. Remove replaced inline blobs/full-closure materialization/readers together; D06 service transfer is conditional, not required for offline delivery. |
| F04 | in-progress | F01 selected metadata; 65d original-operation streaming/custody; I producer-owned coordinates/attempts; C query/protection | Selected readable-root hydration with explicit narrower coverage is integrated and targeted-tested on main under proposed ADR-0224. Complete remaining progress/result producers and clients with selection intent, governing attempt, semantic coverage, returned extent and terminal completeness kept distinct. Preserve required global keys/scientific inputs and exact experiment/trajectory/phase associations; a selected Arrow view is not full scientific admission. [Buffa 0.9.2 segmented encoding](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/encode_sink.rs) and OwnedView apply only after producer selection when their retained backing fits. Test selected/full/empty output, continuation, old mapping, partial final failure, escaped reader and backing release. Compare useful native/Arrow products; delete duplicate projection/extent passes only when equivalent protection moves. |
| F05 | planned | F01 fact vocabulary; working F02/F04 consumers | Generate shared observation/sink projections and correlate CLI, OTLP, Perfetto and optional task observations through existing identities. Test unknown/disabled/stale/lost coverage, precise timestamps and Debug-versus-explicit-export redaction. Retire replaced string-unit/classification mirrors, retaining externally required wire schemas. |
| F06 | planned | F02–F05; I scientific products and J05 developer population; 65g reader | Complete produce→summarize→publish→readback across validation/preparation/solve/provider/store/host failures plus list/check/rustdoc/test and exact rerun. Preserve native verdict ownership, raw versus scientific selection, disconnect and retained reads. Measure complete paths for materialization claims; hand evidence to H. |

Each migration includes all selected native categories and their actual CLI/service/retained
readers, not just a demonstration error. F01 records the finite population at the post-64
baseline; new variants during implementation extend that same owner. A removed production
path takes its obsolete mechanism tests and fixtures with it, while still-needed scientific
and custody assertions migrate to the actual replacement behavior.

## Acceptance and design consequences

Challenge a new solver termination, shared expression with two source occurrences, missing
query coordinates, unknown enum, malformed canonical violations, lost public detail and
observation expiry while work remains active. Independent readers must preserve distinctions
without understanding the solver implementation or parsing messages. Full evidence after a
connection closes remains governed by its actual retention and protection lifetime.

Compare separate per-client records (low immediate work, repeated interpretation), a universal
envelope (uniform shape, inadequate semantics and bulk conversion), and the selected related
owner-derived records (shared mechanics with explicit meaning). Select the latter because
it removes duplicated interpretation while retaining useful native/columnar execution.

Changes to diagnostic metadata/publication contracts use the coordinator's proposed decision
route before implementation. No accepted ADR, numerical qualification or measured overhead
benefit follows from plan authoring. Component tests precede integrated qualification in
[65h](65h-integration-and-qualification.md); the failure baseline is zero.

## Sources

- [Diagnostics/results capability and source account](../library_leverage/libraries/proto/briefings/diagnostics-and-results.md), [representation account](../library_leverage/libraries/proto/briefings/representation-reflection-and-validation.md).
- [ConnectRPC 0.9.1 errors](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/error.rs), [tonic-types 0.14.6 StatusExt](https://docs.rs/tonic-types/0.14.6/tonic_types/trait.StatusExt.html).
- [Validation reference](../libraries/proto/protobuf_validation_rust.md), [local RPC/diagnostics reference](../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md), [reflection/PBJSON](../libraries/proto/prost_reflection_and_pbjson.md).
- Current source: `crates/pse-diagnostics/src/observation.rs`, runtime `workflow/{diagnostic_documents,result_projection,result_inspection}.rs`, `workflow/modeling/diagnostic_source_map.rs`, `tooling/capture/timeline.rs` and canonical result/publication owners.
