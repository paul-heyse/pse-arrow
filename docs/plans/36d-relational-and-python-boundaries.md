---
title: Relational and Python boundaries
status: abandoned
superseded-by: docs/plans/38-efficiency-and-solver-target.md
date: 2026-10-11
adrs: []
review_sources:
  - docs/design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md
  - docs/design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md
scenario_sources:
  - docs/design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md
doc_owner: docs/plans/36d-relational-and-python-boundaries.md
doc_topics: [architecture, preparation, storage, validation, environment]
---

# Plan 36d: Relational and Python boundaries

> **Superseded 2026-10-13 by [Plan 38](38-efficiency-and-solver-target.md).** Remaining execution and acceptance obligations now belong to the [new series transfer](38-efficiency-and-solver-target.md#plan-36-transfer-and-supersession). The inactive `abandoned` lifecycle value records replacement of this execution plan, not completion or loss of its delivered work. All packet statuses, running notes, decisions and checkpoints below are historical; failed/interrupted evidence is unchanged. Do not resume this plan.


Part of [Plan 36](36-efficiency-alignment.md). The coordinator owns the combined target, accepted rule decisions, finding dispositions and assembled acceptance. This document owns the contracts and progress of its packages. Source baseline refreshed to `2d6bf6ae7d03a8a9f6f109696bfe3d9cc8455466`, inspected 2026-10-11. Diagnoses are **Interface-checked**; targets and acceptance controls are **Proposed**. Isolated exact-pin/prototype discriminators are **Tested** in the [boundary investigation evidence](36-investigation-evidence/boundaries.md); no product implementation or product qualification is established.

## Implementation packets

The maintainer authorized accepted-target implementation on 2026-10-11; the packet table records current progress. Prerequisites refer to delivered, verified consumed slices. A bounded investigation ends with an explicit selected remedy, retained contract or supported limitation and its reasoning; an unsupported optimization remains unselected.

| Packet | Required input | Delivery and acceptance | Status |
|---|---|---|---|
| P-D1 | Actual factory/registry owners | Retain exact bound assembly/context and migrate all Runtime consumers without cycles or query-scope equivalence guesses. | In progress |
| P-D2 | P-D1 retained native owner for context-sensitive reuse | Carry admission through unchanged ownership; lazy diagnostics and premise-preserving structural trust. | In progress |
| P-D3 | Existing transport bounds/digest framing; P-B4/B5 integration slices | Owned bounded IPC transport and scoped digest reuse; preserve fresh-store integrity and malformed-input safety. | In progress |
| P-D4 | Existing qualified file binding and shared pool | Lazy file-cache capacity with truthful inactive inspection and race/refusal controls. | In progress |
| P-D5 | Current exact serde_json pin | Own float_roundtrip explicitly and verify challenging exact bits in solver-independent closure. | Complete |
| P-D6 | Accepted RC15; actual installed-extension feature identity | Interactive build selection and explicit validated correctness dispatch with truthful binary provenance. | In progress |
| P-D7 | Selected follow-up question and isolated owner | Decide D-N1 and remaining boundary opportunities using the bounded controls below; no automatic public-setting or product expansion. | In progress |

## Judgment and consequential choices

F23 contains three independent corrections: stable validation preparation and admission carriage; success-path diagnostic/structural work; and IPC transport copies. They should share preservation contracts but remain separately executable. Interactive extension mode is an operator decision and a build/provenance package, not a prerequisite for the other corrections. F24 should preserve actual public Rust inspection and the existing native relational contracts; its confirmed resource correction is lazy file-cache activation. F29 is an early, independent declaration fix. D-N1 remains a qualified cancellation investigation.

These boundaries meet AP-01–AP-07 by placing actual engine implementation retention in the engine, structural and local-field admission in relations, transport ownership in the canonical/result integration, and Python signal policy in the Python boundary. Physical/model semantics, independent post-solve verification and each new numerical state's admissibility remain unchanged. A digest or retained preparation owner establishes neither scientific correctness nor current store eligibility.

Accepted RC15/RC16 and their routes are recorded in the [coordinator](36-efficiency-alignment.md#rule-decisions). The bounded investigation specifies removal of the public unused hashing option as a separate concrete proposal pending operator approval; the original twenty choices left that public setting unselected. Hash identities/framing remain unchanged. No selected package depends on that removal.

`ResourceBudget::hashing_may_use_pool` is handwritten in `crates/pse-runtime/src/budget.rs`, projected by the macro in `settings.rs` and `pse-py/src/inspection/settings.rs`; `_native.pyi` is generated. It is not a registry declaration. Removing BLAKE3's rayon feature is independently selected by the build package using its whole-workspace source/closure evidence; it does not authorize removal of the public setting or restrict library eligibility. Hash framing and algorithms are not changed here.

## P-D1: bind and retain the actual immutable model

`EngineFactory::model_assembly` (`pse-engine/src/session/factory.rs`) uses a weak registry-keyed map. `validation_context` obtains a temporary strong assembly, initializes its validation slot, returns only that slot and drops the assembly. `NativeValidation` owns a cloned `SessionState`, not the enclosing assembly. `workflow::Runtime` currently retains the registry and factory (`pse-runtime/src/workflow/mod.rs`) and its accessor reacquires validation. Sessions/preparations do retain assemblies, so this is specifically a validation-only lifetime defect; physical inventory's derived partition factory has a different model map.

Introduce an engine-owned fixed-registry bound model handle, created after factory configuration is complete. The handle retains the actual factory owner, registry, strong `ModelAssembly` and its reusable validation context. Naming and exact fields are local implementation choices: the required contract is one explicit immutable binding and strong lifetime, not a new service. Keep the general factory's weak map for arbitrary short-lived registries; do not replace it with an unbounded strong map. Runtime construction retains the bound handle, and runtime cloning shares it. Python's acquired process owner (`pse-py/src/inspection/runtime.rs`) retains the same bound handle used by workflow runtimes. Pure/modeling source preparation that accepts a factory explicitly retains a handle for that operation's real lifetime.

Construct physical inventory's derived factory/handle separately, since `with_target_partitions` refreshes factory identity. Its partition demand must remain recorded and consume the shared deployment CPU owner. Neither the shared `RuntimeEnv` nor the registry's process-global derived cache is the right place for validation: one environment/registry can legitimately serve different actual native implementations.

Keep conservative actual-owner equality. `function_inventory` hashes sorted names and the DataFusion version, not UDF implementation equivalence. A same-named replacement UDF must invalidate affected preparation; function hashes, signatures and settings are insufficient. `Functions` in `session/functions.rs` already uses actual Arc ownership. Reusing the exact retained context makes existing `PreparedLocalContract::same_native_owner` useful without inventing a second equivalence mechanism.

Do not share deployment validation indiscriminately into selected/query state. `EngineSession::bound_state` (`engine_session.rs`) installs selected policy settings and catalog bindings; `NativeValidation::prepare` includes execution start time in simplification. Keep an exact context for a changed selection/query snapshot. Where the preparation path supplies the same retained immutable snapshot, pass that owner explicitly rather than minting another context. `bind_candidates`, `EngineSession::validation_context` and `NativeExecutionContext::bind` are actual migration sites, but their different semantic scopes must remain represented. Future optimization across distinct scopes needs a complete consumed contract, not an assumed name hash.

Avoid ownership cycles. The validation planner must not retain its enclosing assembly, and the bound handle must not be placed in that assembly's SessionState extensions. `assembly.rs::retained_extension` explicitly prohibits enclosing session/assembly retention. Cache inspection UDTFs intentionally capture a pre-registration validation snapshot (`cache_service/inspection.rs`); replacing it with a final snapshot containing those same UDTFs risks recursive/cyclic ownership. Preserve that acyclic order.

Migrate all repetitive runtime consumers in the package: document/package load (`workflow/worker.rs`), `modeling.rs`, canonical full-source/edit/fit admission, physical admission, `modeling/pure.rs`, result encoders (`modeling_results`, simulation, fitting, modeling/trajectory/results/analysis tables), study tables, objectives/numerics/analyses, canonical connected readers, Python physical/modeling ingress and registry reflection. Existing `Runtime::validation_context` may become a simple retained accessor; it must stop rebuilding. Reader/result code should consume the actual owner supplied by its runtime, not rediscover an owner per relation.

Acceptance needs store-free engine/relations controls: successive factory-bound/runtime access returns the exact context after unrelated sessions drop; repeated schema preparation stays once; same-named foreign UDF, changed native settings and foreign registry refuse/reprepare; distinct derived physical factory remains distinct; bound-handle drop releases assembly after remaining consumers drain. Read existing assembly/evaluation counters directly. No benchmark is necessary to establish removal of reconstruction, and no speed claim follows without measurement.

## P-D2: carry admission and make successful validation cheap

`Collection::finish` (`pse-relations/src/columnar/collection.rs`) obtains `T::finish`'s checked batch, wraps identical arrays in `OwnedRecordBatch::from_reserved`, then reconstructs `FieldCheckedBatch::from_parts`, dropping local admission. Preserve the existing admission when moving the same immutable arrays into an allocation owner. This is an ownership operation, not semantic readmission. Use a narrow internal operation that transfers the evidence and owner together; no public unchecked constructor should mint admission.

Maintain the distinction between structural safety, local field/domain validity and relational keys/references/completeness. Admission preservation through `take_reserved`, filtering, permutation, concatenation and retention remains conditional on their established premises. Mixed-owner concatenation cannot inherit one owner's local admission. Projection/casts or new values re-establish affected obligations. Generated completion and allocation wrapping do not prove keys or complete populations.

`validate/prepared.rs::evaluate_selected` performs full structural validation before every evaluation. `occurrences.rs::Node::{root,visit}` constructs per-value locations/coordinates and visibility/index vectors, although native predicate kernels already evaluate bounded chunks. Replace eager diagnostic coordinates with compact traversal/selection information sufficient to reconstruct exact root row and nested path only for failing values. Use the original array for whole visibility, `slice` for contiguous strict ranges, and Arrow `take` only for genuinely scattered selection. Preserve dictionary/run-end/union mappings, tagged inactive-arm and null-parent masking, false-or-unknown CHECK refusal, exact rule/row/path observations, cancellation and complete violation counts despite bounded samples.

Remove structural rescans only when a concrete trusted structural owner exists. A safe `RecordBatch` constructor checks field/row shape, not all array offsets/null buffers; the Rust type alone is insufficient. Raw Arrow/FFI/unchecked producer paths retain structural admission. Validated IPC decoders and sound safe construction/transforms can establish/carry structural trust. The local-field capability can carry this alongside its current contract without a heavyweight certificate subsystem. Public generic `evaluate` should retain defensive behavior unless its API explicitly consumes that premise.

`BatchBuilder::finish` also reevaluates appended admitted chunks. Track which segments were appended with same-owner local admission and validate only new or changed segments; final cross-row/relational obligations still run where required. Do not claim this is trivial if CHECK expressions are context-sensitive: retained immutable native scope must match.

Acceptance: nested nullable/tagged/list fixtures challenge masks and exact failure paths; contiguous and scattered selection produce equivalent results; malformed raw arrays refuse before kernels; admitted unchanged collection wrapping does not increase evaluation count; newly appended invalid rows fail; cancellation and bounded sample truncation never mean valid. Diagnostic scratch remains bounded, and every correctness closure activates force-validation.

## P-D3: owned bounded IPC transport

Implementation checkpoint: bounded borrowed preflight now precedes one owned
`StreamDecoder` path. Canonical readers retain one prepared schema, move the verified
store payload into an Arrow buffer, and attach the complete backing-capacity charge to
that buffer before decoding. Output aliases share that charge; newly allocated alignment
buffers receive their own charge, including after reader destruction. Scientific append
consumes an immutable payload/digest owner and fresh store reads independently verify
integrity. The native byte codec transfers unique storage and isolates shared storage.
Two focused columnar ownership controls pass after correcting a fixture that retained
its own input alias. Nineteen Runtime IPC controls now pass, including independently
readable supported scalar layouts, aggregate null-node and metadata-complexity refusal,
malformed frames, nested slices and owned-buffer lifetimes. Writer self-preflight is
removed; early schema-string bounds and the resulting writer path pass the affected
controls. Fresh-read preflight remains. Canonical reclamation controls remain
open for this change.
The buffered canonical file exporter retains finish, flush, sync and existing publication
lifecycle. Neither this checkpoint nor the passing compile closes P-D3 or P-D7.

Current `result_blocks::{encode_block,decode_block}` preflights writer output, then reads borrowed bytes with `StreamReader<Cursor<&[u8]>>`. Canonical codecs add Vec copies (`pse-operations/src/canonical_codec.rs`), and append recomputes digest (`canonical_execution.rs`). Prefer a transport-owned immutable Bytes path to Arrow `Buffer` and `StreamDecoder`, retaining the byte allocation/accounting owner through decoded buffers. Default alignment handling can still copy misaligned buffers; slices may retain larger backing allocations. Measure complete live memory if making benefit claims.

Do not wholesale replace `pse_model::Bytes` without accounting for its semantic framing, serde/schema, heap accounting, `into_vec` and widespread generated consumers (`pse-model/src/bytes.rs`). A narrow admitted transport owner is preferable where it removes copying without exporting incidental buffer mechanics into domain contracts. It should bind immutable payload, size/shape purpose, digest and accounting, with no public bypass. Canonical writer/descriptor construction may consume that admitted owner to reuse its digest within the unchanged controlled operation. Fresh raw inputs and fresh store reads verify integrity independently; a stored digest and an old writer admission cannot replace verification at a store boundary. Unknown-ack replay uses the original immutable request, not a fresh reencoding.

Retain narrow untrusted preflight for exact schema, one batch, explicit EOS/no trailing data, result versus source bounds, schema depth/field/type restrictions, node/row/value counts, valid required buffer lengths, nonoverlap, unsupported dictionaries/compression/variadic buffers and constructor panic/preallocation exposure. The pinned `StreamDecoder` still calls `fb_to_schema` and `RecordBatchDecoder`; `create_struct_array` creates `BooleanBuffer::new` before all validation completes. Existing `oversized_node_and_short_validity_buffers_are_refused_before_arrow_constructor` is an explicit preservation witness. Keep decoder validation enabled; no `with_skip_validation` shortcut is proposed.

Separate writer self-check elimination from decoder migration. Keep writer input shape and scratch bounds and bounded output writer. Remove self-preflight only after controls establish the writer's supported output obeys the admitted transport contract. Retire only decoder checks demonstrated redundant under pinned 59.3 behavior; `catch_unwind` in a discriminator harness is evidence of the failure domain, not an adequate production safety replacement.

Acceptance uses the existing result/source IPC refusal suite plus exact-pin differential controls: schema/depth/node/short-null/malformed offsets/overlaps/trailing data, zero rows, sliced nested children, exact float bits, unaligned Bytes, deterministic splitting and allocation refusal. Decode retained arrays after reader drop/reclaim; output still owns charges. Keep 512-KiB scientific block and separate canonical-source purpose. Package joins the canonical publication/read owner: do not independently alter receipt identities or digest framing.

## P-D4: lazy file-cache capacity, preserve inspection

Implementation checkpoint, 2026-10-11: all six focused engine file-cache controls
pass, covering inactive construction/inspection, qualified one-time activation,
partial refusal rollback/retry, concurrent first activation and truthful failed-bind
namespace accounting. Three bound-model controls also pass for retained validation
after session departure, acyclic release, actual reflection admission reuse and
same-named foreign implementations. Runtime/Python consumer qualification remains
open; these controls establish no latency or throughput claim.

`NativeCacheService::new` constructs metadata/statistics/listing Envelopes whose constructor reserves capacity immediately; default runtime DataFusion receives `unbound_config()` with file cache limits zero. At a sufficient pool, defaults reserve 50+16 MiB without entries. This is accounted capacity, not observed RSS. Existing public `EngineSession::prepare_sql` (`session/native.rs`) and `pse_rules::invariants::run_invariants` (`invariants.rs`) establish real inspection routes. Physical inventory prepares grouped relational roots. Preserve these, syntax/model caches, actual function/field/effect contracts, shared pool, generations, store namespaces, I/O/query/output gates and completion/drain semantics.

Make optional file-cache capacity activate only on an actual qualified file-source binding. `bind_state_with_generation(Some(..))` enables metadata/statistics but deliberately disables listing, so activate only demanded families; the None path activates none. Keep policy limits separate from active reservations. A fallible shared lazy file-cache owner can allocate the native metadata/statistics/listing envelopes once when `bind_state`/`bind_state_with_generation` establishes a reusable source generation. Unqualified stores remain uncached. Concurrent first activation cannot double-reserve; partial initialization rolls back. Reserve-before-retain and finite total remain mandatory. Select a typed native resource error on activation-capacity refusal, with staged new envelopes rolled back and retry permitted. Keep existing explicit generation/namespace bypass behavior separate; neither failure may masquerade as valid reused cache. Budget validation should distinguish mandatory active commitments from optional inactive allowances while enforcing activated capacity and operational headroom.

Retain stable report rows with configured policy limits and zero inactive reserved/retained bytes, avoiding an unnecessary Python report schema change. Inspection is read-only and must not activate file capacity. Explicit clear/generation invalidation empties retained entries according to existing lifecycle; whether active envelopes retain capacity after clear is an explicit existing cache policy, not inferred disposal permission. Keep `SharedIo`: its gate spans stores, which a separate `LimitStore` on each store does not reproduce. Native cache defaults alone do not provide all these obligations.

Acceptance: constructing default runtime reserves no file envelopes; querying reports/EXPLAIN does not activate them; qualified first bind activates exactly once; mutable unqualified source bypasses; namespace/generation isolation, capacity refusal, clear and waiter/drain controls still pass. Optional Python study query/TableProvider product is independently deferred until an actual set-oriented inspection need selects it. It is not prerequisite cleanup.

## P-D5: own JSON float roundtrip

Declare `serde_json = { version = "=1.0.151", features = ["float_roundtrip"] }` once in workspace dependencies (`Cargo.toml`). Clarabel 0.11.1 currently requests the feature transitively; the pinned parser has different cfg paths. This is a current feature-ownership hazard, not demonstrated corruption of the existing unified binary. An isolated solver-independent 1.0.151 probe found 2917 shortest-decimal roundtrip bit mismatches across 10000 deterministic bit draws without the feature, and zero with it; nonfinite draws were skipped. The evidence records an exact counterexample and commands. Keep solver-independent shortest-decimal-to-f64-bit controls, including challenging finite/subnormal/exponent values, and verify the narrow closure owns the feature. No dependency bump, wholesale resolution or new rule change is needed.

## P-D6: select and report the actual extension mode

Implementation checkpoint, 2026-10-11: the ordinary interactive extension now
builds and imports with actual empty feature flags; correctness capture refuses
that same artifact. Its imported path and binary digest were observed, and stubs
were regenerated from its actual metadata. Product schema regeneration exposed
an unsupported external progress-writer enum shape; the Rust owner now uses the
typed tagged representation and its Python consumer follows that declaration.
The native interactive build also completes and regenerates stubs from its
compiled metadata. Its imported binary reports `native-solvers,solver-highs`
without `force-validate`; correctness capture refuses that same artifact.
An admitted direct probe of that native interactive artifact refuses all nine
malformed integer/unsigned-millisecond inputs at the native settings boundary.
Both validated artifacts, ordinary interactive boundary probes and installed
workflow acceptance remain open.

Use production-mode compilation for ordinary interactive `py-sync` and its native variant, retaining local admission and IPC/FFI structural safety. Correctness routes explicitly select/build a force-validated extension before collecting tests. The compiled binary exports its actual feature flags from Rust `cfg`, and runner provenance binds that report to the imported extension path/build identity. Remove the hardcoded validated/native feature claim; a stale or wrong-mode imported extension must refuse correctness execution rather than quietly label an unvalidated run as tested.

Migrate native and ordinary Python test dispatch, parity/setup consumers, editable-extension sync, inspection/provenance reports and their documentation together. A development install must not silently become a correctness install by environment text alone. Preserve the selected native-prefix/ABI contract, explicit validation of actual Arrow target/host/dev closures, and interpreter/lock identity. Do not change which scientific checks are mandatory. Acceptance includes both ordinary/native interactive builds and force-validated correctness builds, stale-import/wrong-path/wrong-mode refusal, truthful reported flags and a representative malformed-input boundary refusal in both modes. Compile and test only the changed owners until assembled scope-end qualification. This package is independent of P-D1–P-D5.

## D-N1: bounded stream cancellation investigation

D-N1: canonical `TableStream::from_canonical` directly `block_on`s a reader pull (`pse-py/src/inspection/stream.rs`); the ordinary blocking bridge (`workflow.rs`) polls signals at 100 ms and cancels then drains. PyArrow releasing the GIL does not itself ensure Python main-thread signal checks during a native pull. Source inspection plus an isolated entered-C-stream delayed-callback/SIGINT discriminator confirms the signal-policy gap sufficiently to select the Python-controlled correction below. The discriminator is not a canonical reader/RPC/protection fixture. A dedicated production delayed-read/SIGINT acceptance control still requires known extension mode and an exclusive fixture, without a full solve. Confirm the main thread entered the pull before signal; compare signal-before-pull and ordinary bridge controls. Record exception, cancellation request, native settlement, returned/retained arrays and protection release. Deferral to delayed completion confirms the gap; prompt cancel-and-drain refutes it; signal between pulls is inconclusive.

Select an explicit Python-controlled next consumption route in the Python boundary using the existing signal/cancel/drain owner. Generic C-stream callbacks remain interpreter-independent. Preserve one consumption, exact schema, refusal of requested casts, EOF versus failure/cancel, late error retention, arrays after close and same-executor reentry refusal. Per-RPC cancellation token consumption belongs to the canonical coordination owner, not an ad hoc Python-only SDK change.

ADR-0181 separates ordinary caller release from physical administration. `Runtime.close()` releases that caller and its aliases; already admitted pulls/publication/native work retain continuation authority. A delayed-read control must explicitly join its owned read/response and use the authentic fixture/deployment physical-close owner when claiming transport drain. Ordinary caller close alone proves neither physical drain nor remote effect settlement. The dated implementation checkpoints below retain their original close semantics and evidence scope.

## P-D7: bounded boundary investigations and opportunities

Implementation checkpoint, 2026-10-11: physical admission and row receipts now
reuse the existing weighted retention owner with one shared 16-MiB allowance.
Distinct exact sources can coexist; misses no longer clear unrelated products.
Each use still acquires fresh canonical protection, row candidates compare the
original checked sources, and explicit clear fences both product families and
in-flight generations. The canonical control now covers alternating revisions,
their row receipts, pressure and escaped owners. Its execution remains open.
This slice does not close the delayed Python pull or other P-D7 obligations.

Implementation checkpoint, 2026-10-11: `TableStream.next_batch` and iteration now
select one Python-controlled consumption of the same native stream. Its boundary
owner checks interpreter signals while the native pull runs, requests cancellation
without taking the producer's lock, joins the original pull and then propagates
interruption. Capsule and controlled consumption remain mutually exclusive;
generic C callbacks stay interpreter-independent. Engine contention/ownership and
Python schema/EOF/cancellation controls are written. The original reader token now
interrupts a pending canonical RPC future; repeated controlled pulls retain the original
failure instead of turning it into EOF. The delayed canonical control uses the actual
named block reservation as its entered-pull barrier and additionally waits for
receipt on the stopped server's actual established socket before interrupting.
It withholds only its selected, authentically owned disposable server, checks
that a dispatched late response retains its grant after the caller joins, and
checks release after actual transport drain.
Compilation, installed
extension/stub generation and the actual delayed canonical SIGINT/drain journey
remain open; this source change is not responsiveness acceptance.

Production close now uses the exact protection-release tracker in ordinary
Operations builds, rather than only in fixture-enabled builds. It refuses live
owners, seals new protection admission before disconnect and joins pending
releases. The original cleanup failure remains observable on repeated close.
Four focused store-free controls pass with explicit Arrow validation and no
requested canonical-test feature, including both close/admission orders, reusable
midlife drain observation and cancelled/panicking/failed release retention.
Actual installed Python close, physical transport drain and delayed SIGINT
acceptance remain open.

The inbound response handoff now carries the same actual prepaid reservation
through nested Operations requests, spawned protection acquisition and SDK
response decoding or discard. Native raw frames drain before successful delivery,
so retained Arrow buffers can take their exact extents without retaining the full
response allowance. Queued-response abandonment explicitly drops decoded values
before grants; retired correlation metadata holds only clearable cells. Sixteen
engine API controls, three focused native WebSocket controls and four focused
Operations/columnar finite-pool controls pass. Runtime selection/IPC controls and
the installed delayed-read journey remain open. Reader metadata and moved receipt
keys retain their actual charges through ordinary abandonment.

The checked gather now uses Arrow's whole-column take for scalar/byte layouts and
keeps selected-range copying for enclosing list layouts. Four focused relation
controls pass with validation enabled, including nullable bit-preserving ordered
duplicates and a selected nested list under an allowance too small for the
unselected children. Original finite-number and nested semantic-field admission
remain unchanged. No throughput improvement is claimed.

These questions have explicit investigation dispositions in the [evidence](36-investigation-evidence/boundaries.md#remaining-opportunities-explicit-dispositions); they are not silently promoted new findings:

- `take_reserved` (`columnar.rs`) uses per-index `MutableArrayData` whenever selected extent is available, including primitive/string columns. Route types without the list-child overpreallocation premise through Arrow's vectorized take; keep selected-range copying for nested lists and checked reserve-before-allocate extents. Preserve duplicate order/null/bounds behavior. Schedule as small relation-kernel work after trusted-owner carriage; benefit measurement is optional for the small-subset production caller.
- Physical sorted-source declarations belong to `ImmutableTable::scan` (`session/materialized.rs`) and its admitted ordering premise. Use pinned `MemorySourceConfig::try_with_sort_information` only for actual established order, across batches/partitions and matching comparator/null semantics. Do not infer sort from primary-key declaration. Retain native merge/conflict preparation; cold cached planning is not itself a demonstrated defect.
  Source inspection of the current candidate, captured-provider, columnar-argument
  and captured-computation constructors finds no retained ordering proof: their
  checked fields/keys establish different obligations, and capture concatenation
  does not establish cross-batch order. Retain unadvertised ordering for these
  sources. Reopen only for an actual consumed sorted producer carrying complete
  comparator/null/partition evidence; adding a second order authority merely to
  advertise this optional opportunity is unselected.
- Single-slot physical admission (`workflow/physical_cache.rs`) may thrash for interleaved distinct sources. The worker accepts distinct PhysicalSource values on one Runtime, so select an accounted weighted multi-entry cache for alternating sources within the existing 16-MiB retention allowance. Preserve exact owner key, fresh caller protections, flights, clear epochs and eviction safety. One-shared-source workloads do not justify the change by themselves.
- Buffered IPC file export (`connected_results.rs::export_ipc`) can use the pinned buffered writer, preserving finish/flush/error and destination lifecycle. This is cold-path local work, independent of store decoder admission.
- DataFusion metadata restoration/UDF wrappers address a real upstream extension-semantics gap. Retain them and the narrowly justified leaf-expression pushdown opt-out until exact newer-source evidence establishes complete field preservation. Library eligibility is unrestricted; neither broad default-feature removal nor wrapper deletion follows from absence of an internal caller.
- `SessionContext::state()` in `bound_state` clones the UDF registry. Retain this scope-sensitive snapshot initially. Revisit only if remaining per-selection clone cost is material after P-D1 and exact selected settings/catalog/execution-time semantics permit a shared immutable snapshot; do not broaden validation-owner equality to remove a small clone.
- Registry reflection currently acquires executor/shared runtime and validates a frozen schema batch each call. Select cached admitted immutable reflection under the actual bound owner with real native CHECK semantics and independently consumed streams; retain heavier process runtime acquisition until the lower owner can provide the same actual native contract. `import pse` itself does not acquire the canonical runtime; no new import/startup defect or measured dominance is claimed.
- cattrs declaration/fit conversion belongs with preparation/authoring. `codec.converter()` intentionally returns mutable isolated hooks; caller-specific hex identity overrides and strict/version-header rules are semantic. Reuse an immutable private hook/converter owner or reduce conversion passes only while preserving isolation; do not expose one global mutable singleton. Coordinate this seam with the preparation package rather than adding duplicate work.
- The 4096-row analysis evidence restriction (`workflow/analyses.rs`) is an explicit complete-campaign refusal, not newly diagnosed silent truncation. A larger streaming analysis product needs its own demand/algorithm contract. Keep it visible as a support limit; do not invent an efficiency defect.
- Arrow compression remains deliberately unproposed. Python cross-output/query and inbound Arrow APIs remain separate product proposals; repeated same-block cross-output decode reuse is deferred to the result owner, and native coalescing is retained until a concrete small-batch consumer requires change. Dictionaries/variadic buffers remain refused by the transport contract. SHA/BLAKE3 algorithm substitution or mmap/rayon observation hashing is not selected; immutable observation reuse belongs to the observation owner, including replacement/truncation safety. Retain quick-cache versus completion-owned Flights distinctions, native SurrealDB bulk/server function use and proportional tracing. Membership page/index/protected-guard and SDK lifecycle opportunities route to canonical coordination, not this boundary package.

Pool churn from original F-03 is shared with result publication/scheduling: use cursor/collection reservations and chunk growth, notify on real capacity release or relevant spillable departure, preserve reserve-before-allocate and waiter progress. Do not delete finite-pool tracking or independently tune wake policy here. Schema/metadata validation at a fresh reopened block and retained copies for external ownership isolation remain justified.

## Evidence and verification boundary

Context7 was queried for Arrow/DataFusion/PyO3/serde_json, then checked against local pinned Arrow 59.3.0, DataFusion 55.1.0 and serde_json 1.0.151/Clarabel 0.11.1 sources. Relevant exact source includes Arrow `reader/stream.rs::{decode,finish,with_require_alignment}`, `reader.rs::create_struct_array`, `arrow-buffer::Buffer::from(bytes::Bytes)`, and DataFusion `memory.rs::try_with_sort_information`. Current documentation is discovery, not a cross-version proof. No stale library catalog/capability maps were consulted.

Planned correctness controls use repository `scripts/pse-env --` and `just unit-package`/Python routes with actual force-validation; compile touched scopes during implementation. Installed-extension mode must come from a Rust `cfg`-derived value tied to the actual extension binary, not the hardcoded `['force-validate','native-solvers']` assertion currently in `scripts/native_tests.py`. Scope-end integrated qualification follows repository timing and root ownership. Product checks during plan authoring are `not_run`; isolated decoder/kernel/JSON and signal-policy controls are recorded in the evidence with their exact commands, conditions and limits. No product timing, production responsiveness, scientific or whole-product qualification claim is made.


## Investigation outcome and implementation handoff (2026-10-11)

All assigned questions, including optional opportunities, now have a selected, retained, separate-proposal or deferred disposition in the [supporting evidence](36-investigation-evidence/boundaries.md). That document specifies the bound-model interface/acyclic assembly order, document-source versus selected/query CHECK context transfer, private structural/admission carriage, diagnostic selections, per-type kernel routing, per-check IPC retain/delete decisions, lazy per-family activation/rollback, real ordering premises, actual binary-mode reporting and Python-controlled pull ownership. It names each consumer/migration and remaining acceptance control. Plan 36a owns private thread-owned declaration/fit conversion hooks and strict direct conversion; this packet preserves its actual Rust admission/native context seam.

**Tested, isolated only:** Arrow 59.3.0 with force_validate accepts missing EOS/overlapping primitive spans and panics on malformed short struct validity/oversized struct nodes; native validation rejects malformed string offsets. Default alignment handles a shifted owned payload and retains arrays after input drop; mandatory alignment refuses it. A primitive take fixture preserves duplicates/order/nulls. The serde_json 1.0.151 standalone feature discriminator establishes the feature's precision obligation without assuming current product corruption. An entered C-stream pull delivered KeyboardInterrupt after its delayed completion, while the before-pull and periodic cancel/drain controls separated the signal-policy cause; previously yielded arrays survived close. These fixtures do not qualify the replacement transport or canonical reader.

**Proposed:** all product corrections. P-D1–P-D7 remain Scheduled for separately authorized implementation. No original finding is resolved, no older plan/index/review is changed, and no product/static/integration/scientific/performance gate ran. The installed extension lacks an authoritative cfg report, so its actual force-validation mode and canonical interruption protection/RPC settlement remain explicit acceptance prerequisites. The new owned probes/logs are supporting evidence; their target/shared-module outputs are disposable and scoped-ignored.

## Scoped implementation outcome: P-D5, 2026-10-11

**Implemented:** the exact workspace serde_json declaration explicitly owns
`float_roundtrip`; dependency resolution no longer supplies this guarantee incidentally
through a solver. **Tested:** `scripts/pse-env -- just unit-package pse-ids json_`
passed 2 controls, 0 failures, 46 unselected tests (failure target zero), covering the
known bit counterexample, challenging finite/subnormal/extreme values and finite-bit
property roundtrips. **Interface-checked:** the actual locked/offline solver-independent
`cargo check -p pse-ids --lib --no-default-features -Z unstable-options --unit-graph`
contained 78 units; all three serde_json unit roles enabled the declared feature and
Clarabel was absent. This closes P-D5/F29 only, with no scientific or product-wide claim.

The corrected mistake was relying on transitive solver feature activation to express a
JSON transport requirement. Deliberately retain the pinned dependency and existing
finite-number boundary; this change does not introduce nonfinite JSON or alter hashing.

## Implementation checkpoint, 2026-10-12

P-D1–P-D7's selected boundary mechanisms are implemented: retained bound contexts,
private structural admission carriage and lazy diagnostic selection, owned bounded IPC,
lazy qualified file-cache capacity, explicit JSON feature ownership, truthful extension
modes and Python-controlled pulls. The unselected public hashing-setting removal R36-02
has not been installed. Rust SQL/invariant and physical inventory roles remain.

All four actual installed extension modes pass their 13 direct integer/millisecond
boundary controls; interactive binaries refuse validated correctness capture. Both
validated builds generate their actual stubs without compiler warnings. The ordinary
validated Python boundary/transfer/registry selection passes 66 tests. A later native
getter repair preserves the full binding digest and supplies the `blake3:` prefix required
by the typed consumer; the rebuilt native validated artifact passes the direct controls.
Its bound-solve integration now passes in the actual 49-test native Python journey.

The real entered-pull SIGINT control passes on the exact preserved private history:
response protection remains until transport drain, close joins the controlled pull,
and escaped Arrow arrays remain readable. The ordinary signal bridge passes too. These
observations are not a universal cancellation SLA. Startup checkout/image observation
uses the Python blocking boundary; its actual constructor thread-progress control passes
in that native Python journey. Managed Python and enclosing native acceptance remain open.

Native response snapshots now have an explicit drain witness separate from the receiver's
decoded-value hold. Reply delivery joins those snapshots under the same absolute request
deadline, including cancellation and expiry, before returning to the next producer. Joined
owners retain observable drain state after allocation release without retaining grants in
retired metadata. All eighteen engine API and twenty-seven native WebSocket controls pass;
independent source readback found no material issue. These scoped results preserve the
existing semaphore completion semantics and do not replace native/Python assembled acceptance.

## Maintainer stop handoff, 2026-10-12

Execution stopped after the requested immediate wrap-up. The [coordinator’s stop checkpoint](36-efficiency-alignment.md#maintainer-stop-checkpoint-2026-10-12) owns current evidence boundaries, worktree reconciliation and the remaining execution route. The actual installed validated extension precedes the latest generated seal change. Rebuilding it and repeating installed-boundary and managed/native journeys remain in the coordinator’s acceptance route.
