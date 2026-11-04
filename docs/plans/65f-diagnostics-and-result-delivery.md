---
title: "Plan65f: Diagnostics and result delivery"
status: proposed
date: 2026-11-03
adrs: []
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md]
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

## Packages

| ID | State | Prerequisite | Delivery and targeted acceptance |
| --- | --- | --- | --- |
| F01 | in-progress | Actual native consumer readiness; I01/J01 semantic design; 65a metadata and 65b policy identities | Completion-owned permission/local-validity and developer evidence observation slices are delivered; complete diagnostic/observation/outcome/result-description products across native diagnostics, runtime documents, producer facts, scientific product-use scope/coverage/selection and developer action/inventory/fixture/assertion/capture/settlement. Preserve every selected distinction with exhaustive mappings and independent records. |
| F02 | planned | F01; 65a generated adapters and 65b violations | Migrate native-to-CLI and local error projections to typed details and category-preserving sources. Recognized malformed, unknown, absent, multi-cause, rule/compiler/evaluator failures and oversized details must differ. Delete replaced string classification and duplicate field maps; keep native rendering where consumed. |
| F03 | planned | F01/F02; narrow C scientific or J developer retention contract according to family | Deliver full evidence with bounded details and protected readback. Exercise omitted selectors, unknown counts, missing spans, expired locators, failed publication, report without inventory and forbidden verdict reconstruction. No summary implies complete retained evidence. Remove replaced inline blobs/readers together. |
| F04 | planned | F01 selected metadata; 65d original-operation streaming/custody | Migrate existing post-64 progress and result-inspection producers/clients. Selection reaches producer; selected/full/empty output, continuation, partial final failure and backing-buffer release retain original semantics. Compare useful native/Arrow products rather than force a single format. Delete duplicate projection/extent passes only when equivalent protection moves. |
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
