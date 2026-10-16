---
title: Causal diagnostics, qualification and closeout
status: in-progress
date: 2026-10-15
adrs: []
review_sources:
  - docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md
scenario_sources:
  - docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md
doc_owner: docs/plans/41-simulation-pipeline-efficiency-and-effectiveness.md
doc_topics: [design, validation, preparation, documentation]
---

# Plan 41f: Causal diagnostics, qualification and closeout

## Purpose and timing

This companion owns O08 and the combined acceptance/closure of [Plan 41](41-simulation-pipeline-efficiency-and-effectiveness.md). F0/F1 are early supporting functional work, delivered incrementally with the owning corrections. F2–F7 follow functional completion and settled investigation choices; they are not prerequisites to every local test and are not repeated whole-product campaigns.

**Interface-checked basis:** the principal review and existing operation/context observation owners, current work counters, protected result reads and prior audit's limited outcomes. **Proposed:** diagnostic additions and acceptance below. Blueprint §23.1 owns enduring observation meaning; §24.2/§24.4 and the qualification command guide own reporting and review routes. Existing Plan40 capabilities are foundations, not another backlog imported into this series.

The user needs to see what a running simulation is doing and why unnecessary work occurs. Extend current operation navigation/current/cumulative/recent views and selected profiling rather than creating another telemetry system, root-cause engine or scientific evaluator. No extra residual, property, derivative or solve request may be issued just to answer an inspector.

## Incremental causal observations

F0 selects only facts needed to discriminate the review's hypotheses and acceptance. Existing `pse-diagnostics::observation` supplies bounded copied observations, integer/counter/generation semantics and relationships; runtime composes process/operation access, and actual producers own domain units. A field is not required merely because a library exposes it. Reuse existing native/preparation counts where they already distinguish the operation.

| Responsibility / producer | Useful distinction and consumed interpretation |
| --- | --- |
| Import resolution and specialization, A1–A3 | Lookup requests, candidates actually examined, stable context builds and mutable overlay/function construction. Selected output size is not evidence of examined scope. |
| Selected plans and grouped projection, A4/A5 | Requested rows/coordinates versus broad/selected structure construction, observation-structure builds and reused projection lookup. Coupled provider work is distinguished from avoidable unrelated assembly. |
| Cancellation/shared/compiler admission, B1–B3 | Requested cancellation, queued versus active owner, producer/waiter linkage, expiry, native stop and actual join. A caller's stopped wait is not producer drain. |
| Result/protocol/resource ownership, C3/C4/B4 | Metadata RPC phase and retained/transient ownership, request/ACK phase, protected snapshot/file lifetime and idle/active borrower. Pool and process peaks have different scopes; no blanket RSS-based eviction. |
| Sparse/native preparation, D0/D1 | Complete support/map construction, selected direct/Krylov setup, factor/refactor and known versus opaque allocation scope. Sparse input does not bound direct fill. |
| Window and point products, D2/E1/E2 | Worker construction/rebind/reset, requested demand, actual integration/replay/upgrade and completed curvature reuse. Native report reuse is not checkpoint reuse; a changed IVP is visible as reset. |
| Selected actions/evaluators/routing, E3/E4/D3/D4 | Action versus expanded products, compile/setup versus evaluation, native method and eligibility/refusal. Observed work is not an unsupported speedup or convergence claim. |

Keep metric names finite and producer-defined; use existing bounded context associations rather than embedding arbitrary expression/model names in metric keys. Label units, operation/attempt/point/segment generation, clock, availability and resets. Absolute values, additive counts and occupancy gauges are different; current integer publication alone must not be interpreted as an additive counter. Saturation, dropped detail, unsupported observations and process-death prefixes stay explicit. No observation must own scientific cancellation or keep an unrelated computation alive.

Active-owner links must remain useful while work is running. Assess existing root/history eviction and managed-process propagation: a late reader must not silently associate current children with an evicted or replacement root. Use bounded active relationships/explicit partiality within the existing hub where needed, not unlimited retained histories. Distinguish fresh current activity from the last safe stale checkpoint, and retain reset/loss semantics in recent deltas and summaries. Parent/child inclusive times cannot simply be added into an exclusive CPU total.

F1 integrates each selected producer slice with its actual operation scope and Rust/Python/CLI interpretation as the owning package changes. Changes in generated public diagnostics follow authoritative declarations and regeneration. Validate enabled/disabled observations against actual scientific call counts and outcomes. Numerical scheduling under elapsed limits need not produce bitwise identical trajectories solely to test observation opt-out; the promised scientific contract and extra-work prohibition remain.

## Combined acceptance selection

F2 consumes package outcomes, actual implementation and inquiry decisions, not just planned interfaces. Every F01–F14 must have its corrective evidence; every O01–O08 must have an implemented selection or supported no-adoption/explicit scope decision. A missing mandatory capability cannot be marked optional because a test refuses it safely. Retain source S01–S10 IDs with the coordinator's routes; no duplicate scenario registry or new per-command diary is required.

Select assembled journeys for differences that can reveal a wrong contract:

- Interactive value/structure/import/physical edits compare retained and fresh preparation and actual work scope. Add independently specified lexical/order/binding and mathematical checks where both routes could share an error.
- Public Rust/Python local physical studies exercise no-store admission, explicit/lazy occurrence policy and immutable source ownership; recorded studies exercise explicit publication and honest durable failure.
- A durable sweep and managed nonempty result read exercise complete demanded outputs, page/file-open/decode, true compiler/resource overlap, exact ACKs and retained numerical/native metrics. Choose enough points/pages to exercise boundaries; 1,000 points are not mandatory solely because the old audit used them.
- Root/optimal/transient response consumers distinguish validity, withholding, partial/events, units and independently retained artifacts. Finite response values or graph construction alone are insufficient.
- IDAS direct and Krylov cases distinguish complete sparse/all-mode storage from dense/fill allowances, with actual native routes where adapter behavior matters.
- Fitting/shooting/horizon cases distinguish worker construction from integrations, same-point exact curvature from changed scientific context, demand upgrades/replay, changed-IVP reset and causal initial/feedback chains. Qualify only selected optional mechanisms, including window waves/JIT/actions if adopted.
- Initialization/recycle/profile/scaling/branch failures and unsupported capability preserve authored specification, truthful outcome and original assessment. Frozen inner starts/branches remain frozen: worker reuse must not introduce continuation and change a recycle map. Extension/local dependency readback challenges accidental store/native coupling.

Use small controlled cases for ownership, repeated products and invalidation, with representative nontrivial numerical cases where actual native/library composition matters. Independent exact mathematics, perturbation checks and original physical assessment challenge numerical transformations; avoid expected values generated solely by the changed transformation itself. Preserve unit/basis/reference, bounds/envelopes/guards, conservation/closure and missing derivative order. Do not silently add general/higher-index DAE, hybrid sensitivity, GPU/MPI or arbitrary solver combinations beyond current supported scope.

## Live inspection, interruption and owned resources

Inspect before a RunHandle exists, during a deliberately occupied owner, across a recent interval and after cancellation. Shared waiters with different original clocks, queue occupancy and producer/native drain must be distinguishable without querying executing mutable foreign handles from another thread. Test root eviction, reset and loss/partial coverage with controlled inputs; absent data is not zero work.

Cross actual local, native-child/managed and Rust/Python boundaries only where their changed contracts require them. Preserve selected process/attempt identity without compiler/launcher/installer or environmental equivalence attestation. A useful existing deeper profile may settle a remaining question; a successful capture exit with unusable frames does not establish it. Do not rerun a profiler to restate a cause already established by direct work counters/source evidence.

Own fixture/worker/capture resources explicitly. Cancel/stop and join only owned work through registered routes, keep charges through SDK/native suffix drain and preserve other owners. Process death, caller cancellation, diagnostic writer failure, scientific terminal state and publication failure are different outcomes. A passing test body with failed enclosing cleanup is not full acceptance; report the distinction. No storage cleanup is inferred from age, size or a temporary name.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| F0 | Principal O08 and current observation foundation; owning package hypotheses | Settle bounded fact/relationship/unit/generation meanings and selected consumer fields, including current root/managed partiality. Identify which existing counters suffice and any minimal missing producer fact. No generic accounting/proof framework. | Completed — selected ordinary/public/managed numerical, work, interruption, loss/reset/opt-out and owned cleanup controls passed after the recorded repairs |
| F1 | F0's relevant slice and actual owning A/B/C/D/E implementation | Integrate selected producer facts and interpretation incrementally. Test actual counts, unchanged scientific requests, opt-out, overflow/loss/reset, active-root retention/partiality and managed scope where changed. Regenerate public contracts when declarations change; delete replaced duplicate counters/views. | Completed — selected ordinary/public/managed numerical, work, interruption, loss/reset/opt-out and owned cleanup controls passed after the recorded repairs |
| F2 | G1; A6/B5/C5/D6/E5 delivery evidence; F1 coverage; inquiry outcomes settled | Select risk-directed assembled cases and commands by S01–S10 and changed interfaces. Confirm actual consumer/deletion inventory and known exclusions; no automatic old campaign import. | Completed — S01–S10 risk-directed selection below; prior evidence retains its original scope |
| F3 | F2; relevant local/native/public contracts implemented | Qualify edit/re-solve, local/durable study composition, response meaning, sparse methods, fitting/window reuse and scientific failure semantics through selected complete journeys. Retain independent original assertions and report zero-baseline results. | Completed — selected ordinary/public/managed numerical, work, interruption, loss/reset/opt-out and owned cleanup controls passed after the recorded repairs |
| F4 | F2; B1–B3, C3/C4 and F1 implemented | Qualify interruption/shared clocks/queue/drain, genuine nonempty managed result read and publication retry/failure, active-owner live/recent navigation and actual Rust/Python/placement boundaries. Track owned cleanup separately from test-body outcomes. | Completed — selected ordinary/public/managed numerical, work, interruption, loss/reset/opt-out and owned cleanup controls passed after the recorded repairs |
| F5 | Functional acceptance and selected optional adoption complete; scoped failures repaired | Reuse F3/F4 assembled evidence and complete only remaining selected integration/component/native/Python coverage, then `just hygiene` and manual governance/docs/native/powerset checks under the repository policy. Fix scoped failures and rerun affected commands; do not introduce quality baselines, duplicate campaigns or repeat product checks after documentation-only edits. | In progress — functional acceptance complete; scope-end hygiene and manual checks |
| F6 | F3–F5 evidence and all required target choices settled | Reconcile the coordinator's F/O dispositions and rule implementation, transfer enduring contracts/guides through proper architecture/ADR routes, and write Outcomes with actual evidence, mistakes corrected and deliberate deviations. No unexamined scientific gates become passed. | Planned |
| F7 | New scope fully implemented and F6 transfer complete | Reconcile relevant existing plan/status documents only now, without claiming their unrelated campaigns passed. Repair ownership/dependencies/inbound links and apply eligible review/plan retirement preflight under ADR-0096; preserve highest-numbered/consumed records and concurrent work. | Planned |

F3 and F4 can select independent ready journeys after F2; shared file/resource execution still needs coordination. F5 is the combined scope-end check, not a check repeated after every package. F6/F7 are documentation/closure, not another testing campaign. The first F0/F1 slices must not wait for F2.

## Verification and completion

**Proposed:** touched-package compile and targeted tests during functional implementation, code generation with declaration changes, explicit force-validation for actual Arrow-consuming correctness closures. At functional scope end use the current supported recipes through `scripts/pse-env`, relevant linked native features and installed generated Python contracts. Select exact filters from implemented surfaces and the qualification guide instead of inventing commands for nonexistent APIs now. Root runs `just turn-end` after a turn that changes files; `just ready` is for actual dependency/tool/environment changes or environment-shaped failures.

Removed construction/replay/scans can be established by source and actual counters. No statistical benchmark, comparative timing campaign or hard simulation latency gate is required. A single complete representative run is useful only if it answers a remaining material question; do not treat historical partial samples as attribution to numerical convergence. Numerical superiority/speedup remains unclaimed without its own evidence. The user's permanent elimination of environment/provenance proof applies throughout.

Completion requires integrated required corrections, settled optional decisions and adopted deliveries, removed replaced mechanisms, meaningful assembled acceptance, honest failure/omission reporting and enduring documentation transfer. Report command, mode, actual scope and zero-failure baseline, using Proposed/Interface-checked/Implemented/Tested/Measured appropriately. Do not sum overlapping tests or promote component passes to whole-product qualification. F0/F1 have the scoped A/B delivery below. F2–F7 remain Planned; no complete-series qualification or closeout is claimed.

## A/B diagnostic delivery — 2026-10-15

**Implemented:** F0/F1's preparation/lifetime slice is integrated at the performing owners.
Indexed imports publish actual bucket candidates examined. Immutable lexical inference
publishes base constructions/bindings and current frames. Selected assembly publishes
actual constructions, distinct required bodies and selected rows. Observation key requests
publish structure construction; grouped projections publish one shared lookup and its
entries. Study parent handling has a once-handled event counter. Compiler admission has a
separate wait phase before active dispatch; existing shared-flight relationships and native
stop/join facts retain their ownership. These changes add no scientific evaluation, cache
identity field, blocking diagnostic collector or replacement accounting framework.

**Tested/Measured — controlled work counts only, zero-failure target:**
[41a's Outcome](41a-source-specialization-and-selected-preparation.md#outcome--2026-10-15)
records the source/compiler/math counts and selected runtime consumers;
[41b's Outcome](41b-cancellation-shared-work-and-admission.md#outcome--2026-10-15)
records public stopping, independent waiter/producer clocks, queued/active separation and
actual native/TLS join. The observation cache control checks two key-request structures for
two requests, including one miss, and actual residuals under changed values. Shared waiter
latency is not counted as duplicate producer construction. The operator guide now explains
these metric meanings and inclusive waiting at
[preparation work and waiting](../dev/simulation-diagnostics.md#preparation-work-and-waiting).
Enduring consumed preparation/cancellation authority is at blueprint §14.3, revision 175.

**Proposed/not_run:** remaining F0/F1 contexts and full loss/reset/managed/public diagnostic
acceptance, F2 assembled selection, F3/F4 combined journeys, F5 hygiene/manual checks and
F6/F7 series closure/reconciliation. The current slice does not close O08 or those packets.
It supplies no elapsed speedup, whole-product scientific qualification or environment/
provenance proof, and it starts no old cold or statistical benchmark campaign.

## B/C diagnostic delivery — 2026-10-15

**Implemented:** the scoped F0/F1 slice distinguishes artifact file sealing from
bounded already-sealed metadata attachment. `artifact.metadata.attach` is the actual
attachment phase; `canonical.artifact.metadata.groups_acknowledged` and
`canonical.artifact.metadata.rows_acknowledged` advance only after exact ordered
ACK validation. An unknown ACK remains unsettled and cannot activate content.
Metadata response bounds and continuing/cleanup grants remain resource-owner
facts, separate from file payload/decode bounds and process RSS. The
[operator guide](../dev/simulation-diagnostics.md#artifact-publication-work) explains
these meanings. 41c owns the actual controlled protocol results. This incremental
slice leaves the other F0/F1 contexts and F2–F7 open.


## D/E diagnostic delivery — 2026-10-16

**Implemented:** native facts distinguish complete sparse layout capacity, selected known
workspace and genuine opaque forward/backward factor allowances. Workflow facts distinguish
worker/layout construction or reuse from fresh changed-IVP/adjoint initialization. Fitting
separates trial/demand upgrades, curvature integrations, complete publication and reuse.
Sample elimination separates shared symbolic analysis from fresh numerical factors and
multiple RHS. Existing native pool owners report actual construction, checkout and return;
explicit scalar compilation reports successful preparation, file bytes and value calls.
The [operator guide](../dev/simulation-diagnostics.md#native-work-and-reuse) owns field
interpretation, units and known limits; allowances are not RSS and entry counts need not
be successful scientific completions.

**Tested/Measured — controlled work counts only:** [41d](41d-native-storage-and-workflow-worker-reuse.md#outcome--2026-10-16)
and [41e](41e-derivative-reuse-and-evaluator-capabilities.md#outcome--2026-10-16) own the
actual sparse-growth, factor-refresh, shooting/horizon worker, common-run owner,
fit-curvature and compiled-consumer controls. Nested producers are counted once in unit
controls; collection slots are explicitly bounded. No extra scientific evaluation is
issued for observation. These incremental deliveries do not close O08, broader F0/F1
loss/reset/opt-out/active-root/managed acceptance, F2–F5 assembled qualification, or
F6/F7 closeout and older-plan reconciliation.


## Assembled selection — 2026-10-16

F2 selects current generated contracts and the actual native/Python consumers below.
Prior D/E numerical controls retain their original scope; this selection composes the
changed owners rather than replaying every old campaign. The public handoff exposed
one remaining version-6 native admission consumer; it migrates to the current version-7
request before dependent local-study acceptance.

| Scenarios | Selected distinguishing journey |
| --- | --- |
| S01/S10 | Retained/fresh value and structural edit followed by actual roots; compiler namespace, visibility and physical-admission controls. |
| S02/S09 | Unequal waiter clocks, producer expiry/drain, queued-parent cancellation, original initialization/recycle/branch outcomes. |
| S03 | Rust local and durable-origin solve/simulation/fit/horizon; Python explicit and lazy no-store execution, current evaluator selector and genuine recorded repeated occurrences. |
| S04 | Original 256 MiB reader with two actual compiler leases and 65 nonempty blocks; exact metadata ACK/retry controls; the same read while a real three-point managed solve is occupied. |
| S05 | Root/optimal/transient response validity, withheld/partial/event and owned evidence; original implicit steady response. |
| S06 | Complete sparse layout and selected KLU/SPGMR/SPFGMR consistent initialization; original-role, support/refusal and native-counter controls. |
| S07/S08 | Exact fit curvature and original analytic responses, same-point demand changes, fresh changed-IVP shooting, six-period horizon, healthy versus failed independent owner parking; selected scalar evaluator controls. |
| S02/S04/S10 | Actual managed continuation and cancellation, exact primary death returning operational unavailability with incomplete durable science; Python repeated occurrence/result reopen. |
| O08 / all contexts | Active-root eviction/partiality, reset/loss, public copied observations and CLI reads, fresh-process opt-out versus actual native work. |

Primary death and known owner replacement may end the observer's wait, but never
prove scientific completion or authorize resource release. Transient observation errors
continue conservatively. A rejected control response explicitly stops and reaps its
issued child. The selected live/read controls use existing diagnostics; no profiler is
needed to restate work already distinguished by direct counters. The selection excludes
statistical timing, the old cold 32-block/1,000-point campaign, environment/provenance
proof, new solver classes and unselected checkpoint/window-wave/JIT/SIMD mechanisms.
