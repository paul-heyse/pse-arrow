---
title: Studies, coordination and results
status: abandoned
superseded-by: docs/plans/38-efficiency-and-solver-target.md
date: 2026-10-11
adrs: [ADR-0173, ADR-0176, ADR-0177, ADR-0178]
review_sources:
  - docs/design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md
  - docs/design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md
scenario_sources:
  - docs/design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md
doc_owner: docs/plans/36b-studies-coordination-and-results.md
doc_topics: [architecture, preparation, storage, validation, environment]
---

# Plan 36b: Studies, coordination and results

> **Superseded 2026-10-13 by [Plan 38](38-efficiency-and-solver-target.md).** Remaining execution and acceptance obligations now belong to the [new series transfer](38-efficiency-and-solver-target.md#plan-36-transfer-and-supersession). The inactive `abandoned` lifecycle value records replacement of this execution plan, not completion or loss of its delivered work. All packet statuses, running notes, decisions and checkpoints below are historical; failed/interrupted evidence is unchanged. Do not resume this plan.


Part of [Plan 36](36-efficiency-alignment.md). The coordinator owns the combined target, accepted rule decisions, finding dispositions and assembled acceptance. This document owns the contracts and progress of its packages. Source baseline refreshed for the investigations: `2d6bf6ae7d03a8a9f6f109696bfe3d9cc8455466`, inspected 2026-10-11. Diagnoses are **Interface-checked**; targets and acceptance controls are **Proposed**. No product implementation, tests or measurements are established by plan authoring.

## Implementation packets

The bounded investigations are delivered below; the maintainer authorized accepted-target product implementation on 2026-10-11. Required input means the delivered, verified slice described below, not completion of an entire other document. Conditional investigations have their own decision outcomes; no finding closes merely because a packet is written.

| Packet | Required input | Delivery and acceptance | Status |
|---|---|---|---|
| P-B1 | Accepted RC06 and governing schema/decision route | Exact conflict domains, both creation/reclaim orders, cancellation and original-clock settlement across all consumers. | In progress |
| P-B2 | P-B1 claim slice; P-A5 only for engagement/affinity reuse | Readiness hints, continuous refill and bounded projected terminal discovery with policy recheck. | In progress |
| P-B3 | Accepted RC07; actual resident/transient owners | Separate population, resident memory and active stages; fitting phase handoff, explicit faer Par and chunk admission. | In progress |
| P-B4 | Accepted RC08/10; selected output/row-locator schema | Typed original-row indexing, payload-shaped blocks, explicit empty close and migrated compact/full consumers. | In progress |
| P-B5 | P-B4 close/demand contract for changed output; existing format can be improved first | Actual acknowledgment accumulator, authoritative close/seal and slim live completion reads; uncertainty readback retained. | In progress |
| P-B6 | P-B4/P-B5 sink slice for horizon; telemetry independently ready | Bounded horizon role history and noninterfering ordinary telemetry, one attempt and coherent final outcome. | In progress |
| P-B7 | P-B1 creation/retirement registration; settled replay floor | Collect only fenced settled residue, with late-effect, restore and interruption controls. | In progress |
| P-B8 | Existing authoritative seal and exact seed permission; P-B5 slim-read slice only for transport integration | Eligibility before cutoff and one admitted seed carried to start/prediction; no full-P-B5 prerequisite for completeness. | In progress |

## Investigation checkpoint, 2026-10-11

[Coordination evidence](36-investigation-evidence/coordination.md) records the exact inspected sources, proposed interfaces, optional decisions and disposable controls. **Tested:** isolated SurrealDB 3.3.0 RocksDB query/protocol shapes, with zero unexpected control failures in the corrected run and the owned server joined (exit 0). Both creator/collector commit orders, exact absent-key conflict, unchanged disjoint readers, cancellation ordering, closed-grant refusal and seed eligibility before cutoff were exercised. These are abstract authority/query fixtures; generated canonical functions, admitted scientific seeds, restore and integrated native drain remain unqualified. The first fixture setup failed on missing explicit namespace/database and was corrected; it is not counted as a product pass.

The selected physical access distinctions are concrete: historical membership retains exact intervals; current-head acceleration selects the investigated `(problem,scope,to_sequence,key)` index only with registered head equality (fixture index limit64/output64); exact descriptor/store acknowledgement uses exact record/index access; latest-result discovery already uses backward indexed order in the fixture. A 64-row result bound does not bound examined history. Terminal seed eligibility is checked before LIMIT within the same snapshot, then exact protected admission rechecks permission. Resource/fitting/horizon, typed demand/locator, close/seal and grant-settlement interfaces are specified in the evidence; optional alternatives there have explicit retain/select/separate-proposal/defer outcomes. Product implementation and original finding closure are not established.

## Selected target and preservation boundaries

The target keeps scientific policy, current effect permission, immutable values and allocation ownership separate. Pure `study_policy` decides the allowed action; canonical transactions establish current claim/cancellation/retention/terminal authority; workflow owners carry acknowledged products and demand; native libraries execute admitted numerical work. An immutable preparation, copied result or seed payload does not confer current store permission. Conversely, independently owned copied Arrow buffers do not need a database lease after decoding.

Preserve acknowledged durability before ordinary `wait()` returns, independent original-space assessment of every new numerical state, exact attempts/generations/leases, bit-exact seed payload, truthful partial/cancelled results, original absolute clocks, drain/join before resource release and recovery without rerunning science. A horizon remains one run and one attempt. Study demand and streamed private horizon output do not authorize independent per-step success or resumability.

Choose library-owned mechanisms where they supply the consumed contract: SurrealDB registered exact-record conflict checks and transactional functions; Tokio bounded queues/semaphores and cancellation; the existing shared DataFusion pool; Arrow selection kernels on original scientific rows; explicit admitted `faer::Par`. No new universal scheduler, cache, resource estimator, semantic authority or guaranteed diagnostic stream is required.

The accepted RC06–RC12 decisions are recorded in the coordinator. Their governing amendments precede dependent implementation.

The independent [canonical/preparation target review](../design_review/reviews/design_review_plan36-canonical-preparation_2026-10-11.md) accepted revised ADR-0177/0178. Its F01 identified study-association creators omitted from the initial retirement contract. **Scheduled:** P-B1 must fence summary-header and point/run membership installation, including an existing terminal run, atomically against result retirement; exercise both commit orders and uncertain retries. The Proposed contract is corrected; implementation and actual emitted-query acceptance remain open. This disposition does not close F08/F15.

## P-B1. Exact conflict domains and retry ownership (F08; L2-02; RC06)

Current evidence: `canonical.rs::{ProblemTurns::run,CanonicalStore::protected_query,PROTECTED_BEGIN,acquire_protection,release_protection}`, `canonical_execution.rs::{protected_execution_query,AttemptFence}`, `canonical_results.rs::{read_results,ResultRead::renew}`, and codegen `surreal_execution.rs::{available,study_cancelled,writer,renew,seal}`. Mutation pacing is exclusive per problem; retries/backoff occur inside that turn. Execution mutations first reread immutable run.problem. Protection/result acquisition and renewal write shared retention authority. Every study writer/heartbeat, and dispatcher/observer/finalizer, participates in the shared study write domain.

Select read-registration of an exact cancellation authority, written only by cancellation. Carry checked immutable problem and point-to-study association with the run/attempt capability; the server still verifies identity and current generation on each effect. An absent association must be registered at its exact key, or be permanently immutable after run registration. Predicate lookup of “a study point for this run” alone cannot protect a later association phantom. Include summary runs and study claim/start/observe/finalize in the migration; merely removing `writer`'s touch leaves common writes elsewhere. Alternative per-run cancellation fan-out is unnecessary on the pinned backend and introduces bounded-set membership/recovery obligations.

SurrealDB 3.3 RocksDB `FOR UPDATE` registers a snapshot key for commit-time conflict validation. It does not block another reader/writer. Two readers of the same unchanged record writing disjoint records need not conflict. A committed write invalidates overlapping registered transactions, rather than every transaction unconditionally. Context7's SELECT documentation and pinned `surrealdb-kvs-3.3.0::Transactable::getu` / `surrealdb-kvs-rocksdb-3.3.0::getu` agree; the latter uses snapshot options in `get_for_update_opt`. Exact absent records are covered; predicate/range phantoms are not. D-P1 is consequently not an unresolved prerequisite for choosing this cancellation variant; retain emitted-query race controls and requalify if backend/release changes.

Use the following complete source-retention protocol. Creator transactions register/check a source reclaim fence and reclaimed intervals, register their exact root/pin key including absence, create the durable root/pin and write a creator fence. A collector registers the creator fence **before** scanning roots/pins, registers affected exact pin/root authorities, then publishes reclaimed intervals/removes source payload and writes the reclaim fence atomically. Creator commits first: its creator-fence write invalidates the collector's earlier scan. Collector commits first: its reclaim-fence write invalidates the creator; a rebuilt transaction refuses reclaimed intervals. This two-way registration prevents creation phantoms in both orders. New root/pin creation may still contend at a source creator domain; that is a real shared effect, amortized by owned study/preparation engagements, not every result append.

An existing protection renewal registers its exact pin and reclaim authority, checks original identity/live expiry/released state, then updates only its pin. Collector registers that pin before deciding deletion. Renewal first invalidates stale collection; collection first invalidates stale renewal through the reclaim fence. An expired/released pin never revives. Continued protected reads register/check the live pin plus reclaim authority, without bumping a common lifecycle counter. Product/stage activation/source revision creation and analysis roots must use the corresponding creation/reclaim protocol; preserve named absence, membership and immutable revision premises and analysis authority incarnation/closed-through floors. Pure active-attempt root removal registers exact root/reclaim authority without writing the creator fence: no new dependency is introduced, and the run root retains that source. Collector-first still invalidates stale release; a collector observing the removed root can conservatively retain.

Give result pin creation and retirement their own analogous run/attempt/manifest protocol: creator registers exact retirement authority including absence, installs exact result protection and writes a result-creator fence; retirement registers that fence before scanning live readers and obligations on that producer, then writes monotone retirement authority. Creator first invalidates retirement's stale scan; retirement first invalidates the read admission, which then refuses. Existing result renewal checks the exact live pin, manifest and retirement authority. Under ADR-0182, retained study inputs and analysis inputs own scientifically sufficient promoted artifacts. Promotion and handoff preserve exact protected producer selection until artifact activation and consumer obligation admission; later artifact reads/obligations use artifact authority and do not pin the whole producer run. Analysis creation/input-edge installation participates in artifact collection fencing so collection cannot miss a newly retained borrower. Required source roots remain separately protected.

Local pacing follows actual written domains (run/attempt, source creators/reclaim, exact pin), with stable ordering for transactions crossing several owners; no turn survives hydration, native work or backoff. A definite typed transaction conflict rebuilds the whole decision under the original clock after releasing its turn. Timeout, lost response or cancellation after dispatch is uncertain effect knowledge: settle its exact operation receipt/fence, never replay the application mutation blindly. Transport disconnect/drain does not prove remote settlement.

Cancellation has serial-order semantics. A valid append committed before cancellation may remain in the retained prefix. Cancellation committed first invalidates/refuses a late append/renew/claim. Joined complete export racing cancellation retains original assessment under the declared cancelled terminal outcome; incomplete export supplies no usable science or seed. Do not implement “wins” by deleting a valid earlier append.

Package migration: change declarations/generators, operations adapters/fences and every producer/consumer together; regenerate instead of editing generated rows/queries. Replace broad problem/study/retention writes only after tests cover their actual premises. Controls: both commit orders for source-root creation, absent pin creation, renewal versus reclaim, result creation versus retirement, promotion versus producer retirement and analysis-input creation versus artifact collection; expiry/release; association absence; cancellation versus append/heartbeat/close/seal; late worker generation; lost acknowledgments and original-clock exhaustion. Existing broad-guard tests cannot qualify the new protocol. Remove superseded shared touches, problem lookups, gate-held retry behavior and mechanism-specific tests when replacement controls pass.

## P-B2. Ready work, continuous refill and affinity (F09; D-01/D-09)

Current `worker.rs::{work_once_scoped,serve}` restarts study/candidate discovery, reads each candidate's `study_scope`, and latches Idle until all current lanes drain. The candidate index exists and filters assigned/settled points; waiting predecessors still reach policy evaluation. Finalization touches shared authority before discovering pending work and later rereads per-point descriptor fields.

Maintain readiness as a derived discovery hint at the owners already changing point dependency/retry/recovery state. A pending predecessor count plus exact transition version is preferable to an unqualified ready boolean. Settlement, refusal, seed availability, retry eligibility, lease expiry/recovery and cancellation must invalidate/reestablish the hint. Discovery indexes ready hints; claim reruns pure policy against exact immediate premises and current fences. The hint cannot authorize work. Restart rebuild/verification derives it from authoritative point state without rerunning science.

Refill continuously as completions release dependencies; an idle lane does not impose a barrier on other lanes. Sleep only with no discoverable progress and no pending completion that can release work, with bounded polling as the recovery fallback. Read pending state before invoking the final authoritative guarded finalizer; it rechecks terminality atomically. Project case/definition fields used by outcome assembly in the bounded page instead of N point rereads. Preserve ordinal public placement independent of physical completion order.

Single-successor continuation affinity is an optimization: pass owned compatible seed/prepared engagement and, within resident admission, retained native session to its next point. Multiple workers/restart may break affinity without changing allowed starts. Never monopolize capacity with an idle chain or assume in-memory state is durable permission. Engagement acquisition/renewal failure stops new point admission but still owns already issued/uncertain work through drain (preparation package prerequisite).

Implementation checkpoint, 2026-10-11: continuous refill and explicit input-lane
owners are connected to derived readiness rows, indexed pure discovery, atomic
point-writer invalidation and bounded exact-premise repair. Persistent cursors
bound and rotate per-study notification, rebuild and verification work. Mandatory
maintenance does not consume the scientific action budget. Operations and linked
Runtime compilation succeeded; emitted-query, restart and actual lane controls
remain in progress. Independent source review found that periodic verification
across studies can still starve later studies behind an always-busy earlier study;
a rotating recovery position and populated fairness control are required before
acceptance. Full query/candidate observations and lease/failure/drain acceptance
remain open.

The final-summary owner now reads a bounded pending-state hint before parent recovery or guarded finalization, retaining the authoritative finalizer's atomic recheck. Original outcome and immutable definition arrive in one byte-bounded four-point projection rather than one metadata page followed by an individual full-point read for every occurrence. Existing finalization/recovery controls now also compare projected original bytes, policy and outcome and check that pending discovery leaves parent authority unchanged. These additions remain unrun; readiness hints and engagement-dependent affinity are still open.

Controls: independent chains with skew, chain-major layouts, retries, failed predecessors, explicit starts, refusal, cancellation, stolen/expired claims and restart; candidate/finalize counters and populated EXPLAIN. Original `(w−1)N²/2` and ten-million arithmetic are workload hypotheses, not established source counts. D-P2 and D-P3 should measure actual examined candidates, protected calls, retries and wait relative to numerical work; zero retries changes prioritization, not the broad local-write diagnosis. Remove restart-from-zero blocked traversal, round idle latch, eager guarded probes and item rereads after replacement tests.

## P-B3. Population, resident memory and active stage admission (F10a–e; RC07)

Current sources: `math.rs::{MathPolicy,workspace}`, `math/jobs.rs::{admit_entry,reserve_entry,admitted}`, `math/staged.rs::{open_session_queued,session_on_reserved}`, `workflow/run.rs::PreparedFit::start_owned`, `fitting/profile.rs`, `implicit.rs::Problem::new`, `result_export.rs::SelectedCollection::push_ref`, `pse-engine/resources.rs::ReleaseNotifyingPool`.

Select three distinct owned resources: finite admitted population (including queued requests), resident session/preparation memory, and active job/CPU allowance. An idle native session owns its population ticket, thread/foreign/cache memory and joined lifetime, but no active compute job/CPU permit. Preparation, solving and publication use their real stage admission, so a publication lane does not consume native width. Active stages wait cancellably under the original deadline; oversized requests refuse before creating work. Do not blindly convert every try-acquire to waiting: current idle sessions hold job tickets and subwork can need another, creating circular waits.

Reserve complete next-stage construction/transient memory before acquiring active job then CPU in a consistent order. Failed acquisition releases tentative resources before waiting/retry; essential resident memory remains accounted. Optional retained session/prepared state yields before a population of idle owners can leave every stage awaiting growth; pending compact work is bounded. No parent may retain active permits while waiting for independently admitted child work. Nested work borrows its existing allowance; genuinely independent work requires quiescent parent release. Memory unknowns require a conservative guard, not retrospective charging. Specify oversized/closed/deadline/cancel outcomes separately from scientific failure, and join native teardown before releasing ownership.

For fitting select estimation at its actual thread demand, then a quiescent phase boundary releasing active admission. Profile chains become independently admitted jobs with ordered result assembly; the parent retains only accounted fit/result state, not permits needed by children. This avoids several fits each holding estimation permits while acquiring more. Preserve covariance/profile meaning, deterministic chain output/order, team-stack reservation and original deadline. Alternative coordinated whole-team phase reacquisition is acceptable only if it releases estimation resources first and does not revive head-of-line blocking as the ordinary policy.

For F10c distinguish hard cap from charged live allocation. Known flat/nonflat body/workspace construction bounds and growth admission can replace 4 GiB/8 GiB blanket charges where source-backed. Foreign scratch, thread stacks, compiler construction and unknown providers stay honestly guarded before allocation; do not assert current constants are measured peaks. Unknown provider allocation remains a scoped open support limit until owner evidence supplies a bound. Coordinate this with the preparation/numerics owners rather than adding a universal estimator.

For F10d pass explicit admitted `faer::Par` per invocation into implicit scratch/factorization; nested property work defaults to Seq. No process-global faer/Rayon mutation. `Par::rayon(0)` resolves the current Rayon pool (which may be caller-installed), not invariably logical CPUs. D-P4 asks whether representative symbolic LU selects supernodal; negative evidence reduces expected gain, not admission correctness. The numerical owner also needs quiescent CPU handoff for every A03 IBEX exclusion path while resident memory/session/drain stay owned.

For F10e reserve row-copy work by bounded chunk rather than registering one unspillable consumer per row. Keep current notification semantics initially. DataFusion 55.1.0 `FairSpillPool::unregister` decrements spillable participant count; a zero-byte departure can increase another consumer's allowance. A bytes-only wake rule is wrong. Optional suppression requires proof the particular departure cannot improve the wrapped pool's admission; generic MemoryPool wrappers cannot infer that. Chunk ownership itself removes most per-row wake pressure without a new notifier model.

Controls: saturated resident sessions plus compiler/subwork, several simultaneous fits at the phase boundary, cancellation while queued and join/drain, memory pressure with yielded idle retention, oversize refusal, inherited versus explicit nested parallelism, and zero-byte spillable departure waking a blocked grower. Measure actual allocations/population before claiming memory gains. P-B3 can proceed independently of completing P-B1; F08 is not a universal prerequisite to its benefit. Remove jobs/2 width coupling, whole-fit profile charging and per-row consumers only after their replacements' targeted controls pass.

## P-B4. Demand, exact row locators and empty close coverage (F11/F13 study detail; RC08/RC10)

Implementation checkpoint, 2026-10-11: the [independent bounded target review](../design_review/reviews/design_review_plan36-result-demand_2026-10-11.md) accepts the proposed ADR-0180/P-B4–P-B6 target, with no additional material finding or rule impact. Scalar-row declaration/producer/reader migration is underway. Demand, empty-close coverage, acknowledgment accumulation and horizon streaming remain open; target acceptance is not implementation acceptance. The changed interpretation will be exercised on separately owned new fixtures, without activating or migrating a live deployment.

Current `result_projection.rs::{scalar_cells_at,store_result_chunk,metadata_prefix,store_trajectory}` emits per-field cells and lets their 128 KiB metadata ceiling choose IPC prefixes before the 512 KiB payload bound. `store_result_table` insists on a block even for empty relations. `connected_results.rs::output_results` decodes the original block and verifies selected fields. `DurableAttempt::store_tables` exports every table; ephemeral `study_inner` retains every full RunResult because its occurrence consumers currently expect full reports.

Select one **typed exact row locator per indexed scalar scientific row**, independent of field: exact attempt/set, relation-owned primary-key tuple, block identity and original row offset. Encode keys with registry-owned types/comparison, never a Float64 surrogate. The declared output selector maps owner/partition to that row; field support/missing/finite predicates execute on the decoded original row using Arrow kernels. Verify locator identity/row/key/field against original columns, preserve signed zero and exceptional bits, explicit missingness, units/quality and attempt boundaries. Retain refusal for unsupported fields. This avoids globally resorting current relations merely to simplify an index. Typed block-key bounds remain an alternative if populated access evidence justifies their ordering and range contracts; current coordinate_min/max are finite Float64 **time** bounds and cannot encode arbitrary composite keys.

Pack rows by useful payload and bounded transport metadata, independently of field count. Keep self-contained IPC preflight, message/metadata limits and accounted encoder/decoder scratch. Several trajectory symbols may share a block with exact per-symbol row ranges/time bounds; retain declared stored row ordering and selection identity. Batch small nonempty relation appends within the existing message envelope while preserving individual per-block bounds/receipts. Do not promise every block reaches the payload limit: locator metadata, schema width and atomic message ceilings remain real bounds.

The demand contract distinguishes requested materialized rows, declared empty requested relations, and unrequested detail. Close acknowledges the complete advertised demand and zero-row membership; an absent append before close means incomplete export, never emptiness. Readers get schema from registry authority and exact close membership. Completion, original assessment/usability, diagnostics, start provenance, run/attempt and demanded summary remain mandatory. A study demand explicitly selects compact summaries/KPIs, relations/members or full occurrence reports. Keep an explicit full mode. Failed/flagged detail has its own bounded selected policy; “full detail for every failure” must not silently restore unbounded live retention. Missing detail is unavailable; a rerun is reproducibility only within its stated determinism class, not the original result.

Migrate typed study documents/schema/generated Python contracts, study admission/executors, RunResult cursors, durable writer, output reader and occurrence-report consumers together. Existing consumers requesting full reports must request/receive full demand or move to compact APIs; silently dropping them is a contract defect. Ephemeral execution releases unselected full report after extracting mandatory summary and exact needed continuation seed. No general cross-output query product is needed to replace per-field scalar indexing; Arrow/DataFusion cross-output predicates are separate optional capability work.

Controls: selected scalar fields, signed zero, nonfinite diagnostics, missing/finite bounds, invalid keys, unsupported fields, original row ranges, dense multi-symbol time selection, full/compact study consumers, demanded empty sets, empty-versus-incomplete crash, batched append lost acknowledgment and cancellation during export. E-P1: bounded metadata-only reads of **exact isolated fixture set keys** (`result_set,ordinal,rows,payload_bytes`) plus append/index counts, preserving empty-set membership. No production-history scan or numerical campaign is needed. Tens-of-rows and savings remain unmeasured. Remove scalar-cell declarations/indexes/generation, projection staging and schema-only empty append/test fixtures once every reader has migrated.

Implementation checkpoint, 2026-10-11: study request/operation/definition contracts
now carry explicit output demand (versions 5/6/9). Existing complete-report
consumers select Full; Compact retains original completion and separately bounded
success/failure detail, with Rust/Python selected-detail consumers. Ephemeral
continuation retention is being separated from full report ownership. Durable
publication selects mandatory summaries and demanded relations, including
explicit empty coverage. Dense trajectory packing now supports several exact
symbol ranges in one original payload; the reader verifies those original ranges.
The combined native Runtime all-target compile passes after correcting two
allocation-pool borrows and a trajectory fixture field type. New functional
controls still await execution; this does not accept the packet. Compact
continuations now copy only original-permitted seeds and prediction sources,
preserving full identity/profile while releasing old task/start execution owners.
Typed member demand and its original-row selection controls are now written,
including absolute solve-step keys, signed zero and exceptional bits. Bounded
append batching is now written over the same projection owner: bounded groups
share one transaction while preserving original per-block operation identities,
the first offered deadline and exact lost-response settlement. Coverage advances
only from actual acknowledgments. Optional queue storage drains before polling
an allocating producer; already owned checked chunks can share submissions.
The Surreal generation is published. Independent source review found two
remaining gaps: staged byte owners could end before timed-out transport drained,
and normal durable producers still flushed before every poll. Repairs carry actual
payload grants through transport drain and give only the immutable typed projection
one memory-only cold retry under the same original cancellation/deadline. Bounds,
rollback, replay, empty-membership, continuous-charge and actual normal-export
grouping controls await execution. Horizon private-prefix streaming, generated
contracts and Python consumer verification remain open.

## P-B5. Actual acknowledgments and live completion reads (F12; RC09)

Source `store_result_table` currently discards compact acknowledgments; the writer does **not** already hold a complete set. Introduce attempt-owned bounded accumulation in the publication owner, consumed by close/manifest/seal. For each set retain next acknowledged ordinal, rows and streaming ordered digest of exact batch key/digest/rows; retain compact demand/empty membership, completion coverage and eligible seed/progress coverage. The current batch-selection FramedHasher format is incremental, so keeping every payload or all historical acknowledgment objects is unnecessary. Concurrency within a set must be ordinal-ordered or have a bounded outstanding gap buffer. Duplicate/lost-ack settlement advances once for the exact same request identity.

The server's final admission validates **all** advertised sets and counts/coverage against frozen canonical state, including completion, progress partiality and seed inventory, under exact close authority. Do not replace readback with guessed local totals or treat the original attempt generation as recovered close authority. Successful close acknowledgment returns the same authoritative closure information currently recovered through `closure_operation`. The normal acknowledged path builds its DurableRecord from the actual seal acknowledgment and digest-acknowledged original completion bytes. Uncertain append/close/manifest/seal or restart uses exact receipt settlement/server reconstruction and comparison; incomplete science cannot be promoted. Heartbeat remains owned through reconciliation, then stops/joins with its outcome checked before terminal admission.

Keep ordinary durable return after acknowledgment. No early scientific return is selected. For key-only stored-result handles select a **slim protected completion-only read**, avoiding seed inventory; repeated properties may share one explicitly live-pinned operation/read session but cannot keep permission after expiry/retirement. A separately named copied completion snapshot can survive later retirement as owned bytes, if ever authorized; do not silently change current handle semantics. Bulk terminal study pages obey the same exact permission and memory rules.

Implementation checkpoint, 2026-10-11: key-only completion properties now use a narrow protected completion owner, sharing terminal/revision/result-creator admission and live renewal with full reads. It fetches only the exact completion-set coverage, avoids manifest descriptor transport and seed discovery, and shares bounded chunk assembly/digest verification. Fresh reads still require current permission; already copied completion bytes do not authorize a later read. Inline/chunked equivalence and removed-fixture refusal controls are written; integrated Runtime execution remains pending. Actual acknowledgment accumulation and authoritative normal close/seal remain open, including the P-B4 demand/empty contract.

Controls: byte-identical acknowledged completion versus recovery, gap/duplicate/missing/extra set coverage, empty sets, close-authority recovery, rejected heartbeat join, lost seal/close/append acknowledgment, crash between stages, retired/reclaimed/expired result refusal and property reads without seed inventory. Remove normal-path full reconcile/reopen and successful-close lookup after the accumulator and authoritative acknowledgments are real; retain server readback for uncertainty/recovery.

Implementation decision, 2026-10-11: opaque Operations-issued append acknowledgments own the unchanged framed digest; server close owns complete frozen membership/extents and exact receipt checks. The accumulator cannot be constructed from caller totals or serialized descriptors. The pinned server's plain-string BLAKE3 is not byte-compatible with the existing derive-key framing. Do not substitute it or claim protection against arbitrary raw-database mutation. Recovery retains independent immutable-record digest reconstruction. This authority split is recorded in proposed ADR-0180; implementation and actual canonical controls remain open.

Implementation checkpoint: the acknowledgment accumulator, atomic complete-membership
close/seal and normal no-reopen completion route are implemented. The bounded progress
uncertainty route preserves the original sampled counters and permits only additional
optional progress within the original queued coverage. Independent source review found
no material defect in this identified slice. Actual canonical tests exposed a shared
source-grant response-slot bug before result readback and preparation engagement; its
repair is being verified. Study demand and horizon streaming remain separate open slices.

## P-B6. Bounded horizon role state and telemetry (F13 horizon/F14; RC10/RC11)

Streaming only `Loop.requests/results` is insufficient. `horizon/driver.rs::Loop` also retains all measurements/inputs/step reports and indexed accepted/estimate/background references; `workflow/staged.rs::Staged::record` clones scalar maps and warm starts into `Vec<Record>`. Other staged sequences support arbitrary historical `Start::Accepted(index)` and `predecessor_at(index)`. Select a horizon-specific bounded role-state owner (or explicit release capability), preserving general sequence history where its consumers need it.

Retain current estimator, last accepted result/seed **per role** (possibly older than the latest failed step), advanced background result/factor, current candidate permission, priors/anchors and the rolling MHE measurement/input window. Stream completed step rows and seed chunks privately under the same attempt ingestion fence; retain small step outcome/identity history only where report semantics need it. Close freezes coherent final demanded coverage once; cancelled/failed prefixes are truthful and never seed-ready independently. Full public in-memory history requires explicit demand/materialization and memory admission, rather than being the unavoidable executor representation. Coordinate the sink with P-B4/P-B5, and per-role basis reuse with preparation engagement.

IDAS sample sensitivities/interpolation/checkpoint retention remain the numerical owner's consumer-specific assessment. Do not delete state sensitivity based on a generic “IDAS can interpolate” claim: exact event/window/checkpoint and output sensitivity consumers decide demand.

For ordinary iteration telemetry select nonblocking sampling/coalescing at the tap, bounded latest state/retained events and exact dropped/partiality counters, sequence/timestamps and accounted memory. Current `Tap::observe` can block native producers to heartbeat timeout or immediately refuse async-runtime overflow, then cancel; that can interrupt before original assessment, so do not call every affected solve already scientifically sound. Storage/telemetry failure and scientific assessment stay separate. Mandatory terminal science/seed permission already have completion authority; do not invent another guaranteed phase/failure/terminal stream without an identified consumer. Explicit strict-audit telemetry may choose interruption on loss, with a separate policy and clear operational outcome. Hard work limits/cancellation remain independent of sampled events.

Controls: long horizons with intermittent failed role steps, rolling-window movement, advanced-step factor/permission reuse/fallback, general staged historical consumers, private-prefix crash/cancel, final manifest coherence, saturated telemetry on native and async producers, exact drop counts/order, strict mode and terminal transport failure. Remove redundant full-history executor vectors/record clones for the bounded horizon role and diagnostic-overflow cancellation in ordinary mode once the new consumers pass.

Implementation checkpoint, 2026-10-11: bounded per-role predecessors, rolling
windows, compact selected detail and one-attempt private prefix streaming are
written; general staged history retains its separate contract. Independent review
found post-science completion-copy refusal could lose original rows, zero-solve
Compact membership omitted mandatory empty schemas, and tightening diagnostics
were expanded eagerly. The repair admits aggregate capacity before execution,
moves original completion rows on every publication outcome, declares summary
membership before row production and defers compact tightening descriptors to
final projection. Unexpected producer-shape mismatch retains the actual issued
step and reports an explicit aggregate error. Compilation and decisive refusal,
window, zero-solve and final canonical coverage controls remain open.

## P-B7. Settled retirement before bookkeeping collection (F15; RC12)

`surreal_retention.rs::collect_run` removes result payload/indexes but retains operation receipts/guards; source/result protection tombstones also persist. Release deliberately upserts a tombstone before acquisition appears, because submitted acquisition can outlive its local response. Clock expiry alone does not prove an older transaction cannot commit after tombstone deletion.

Select owner-driven collection only after monotone retirement/replay authority fences the effect, with immutable original grant deadline/nonce/incarnation and a settled closed-through floor or equivalent exact replay fence. Source and result acquisition register the exact grant/tombstone key even when absent; final settlement registers it and stable authority, proves the original window closed and advances the floor atomically before deleting per-grant residue. An older registered acquisition either committed first and is observed/drained, or fails validation; a later attempt refuses the closed grant. No renewed grant for the same key, cleared floor under the same restored incarnation, or age-based eligibility. Blueprint §20.4/ADR-0172 analysis settlement supplies the existing pattern, not proof that protection grants already implement it.

Under ADR-0182, producer payload retirement and detailed run/study history release are independent explicit operations. Retirement remains visible to every late writer/replay path before any receipt/detail deletion. Keep permanent minimal original-identity/owner locators and retirement/phase fences needed to reject resurrection and interpret compact outcomes; audit-retained detail and unresolved effects survive. Bounded resumable history collection removes released detail only after its obligations drain, including authorized child-history collection for studies. Promoted artifact obligations survive producer payload/history collection independently. Compaction is physical housekeeping, not retirement authority; this plan adds no production deletion authority or universal retention SLA.

Controls: acquisition dispatched before release but committing after local failure, both settlement/acquisition commit orders, absent creation, deadline crossing, replay after collection, crash between cleanup pages, restore/incarnation mismatch, permanent original-owner/phase locators and retained audit/unknown-effect history. E-P4's global inventory should become bounded exact-fixture counts by owner, comparing residue before/after explicit retirement. Remove indefinite per-operation residue only with the new eligibility authority and replace tests asserting no tombstone deletion with grant-settlement safety tests.

## P-B8. Complete seed selection and admitted-object transport (A01/A02)

`canonical_execution.rs::result_seed_candidates` currently LIMITs 64 headers before `durable.rs::latest_seed` skips nonterminal attempts; an older usable sealed seed can be masked. `latest_seed` then admits/decodes one seed but returns only its ID; `with_stored_start` reopens it, `seed_document` also calls full `record`, and prediction decodes again.

Select eligibility **before cutoff within one server-query snapshot**. Atomically publish a bounded attempt-level terminal seed-selection witness during seal, bound to acknowledged seed inventory and complete-original completion; recovery/cancelled prefixes with unavailable science publish no seed eligibility. `store_seeds` already exports only `completed_modeling_seed` products, but terminal=true alone is insufficient because incomplete cancellation/recovery may leave headers. Avoid a per-step unbounded seal update loop: the witness is per attempt and refers to frozen complete inventory. The selector filters terminal witness, retirement, layout/preparation/backend before its bounded limit, using deterministic sequence/generation/step/seed-key ordering. Exact protected admission still verifies header membership and per-step completion permission. Current-query-cut semantics are enough; no promise of newest at the later instant of numerical use. If populated production-shaped evidence shows the joined scan is ordinary limiting work, select an indexed eligible-attempt discovery relation with bounded compatibility partitions and frozen-inventory identity, activated transactionally with seal, as specified in the coordination evidence; never revert to a pre-eligibility cutoff; bounded keyset pagination must expose incomplete discovery and an explicit observation contract.

Return one owned admitted seed object carrying immutable header, exact warm payload, optional prediction, completion permission/provenance and allocation ownership; transport it through selection/start/prediction and same-process continuation. Acquire one exact live protection during admission, reserve before decode, then deliberately copy out the admitted seed for the **already-authorized start operation**. Independent later discovery/use obtains fresh permission from the selected producer or promoted artifact authority; owned bytes are not cached stored permission. Durable future study inputs use an independently owned artifact admitted for the exact original solution/run/attempt/step and compatible target, rather than late SolutionId rebinding or a whole-producer obligation. Preserve the public distinction: exact coordinate/layout and backend compatibility, while numeric data and native options may deliberately differ (`run.rs::with_stored_start`). Carry profile/data provenance without incorrectly requiring equality. Explicit incompatible seeds refuse; corruption/infrastructure errors never become absence or cold fallback.

Implementation checkpoint, 2026-10-11: seed selection now returns one admitted immutable product containing its original header, copied completion evidence, warm start and optional prediction with their allocation owners. Explicit selection and latest selection each perform one protected original read; completion decoding shares that read, and immediate continuation/start/prediction share the decoded product. Native working copies remain behind native admission. Prediction anchors require the exact run and attempt. A later acquisition still checks current original-manifest and scientific permission. Shared-payload and removed-fixture refusal controls are written; compilation and actual canonical/native execution of this slice remain pending.

Controls: older usable seed plus 64 newer unsealed headers selects older; after valid seal selects newest; cancellation/recovery prefix is ineligible; total-order ties, retired/expired admission, lost seal acknowledgment, incompatible layouts/backends and permitted data/options variation. Count one selected seed's protected opens/decodes through start/prediction and compare bit-identical payload/start lineage. Also preserve exact admitted study-input reuse after producer retirement/restart while retirement-first admission refuses; these controls consume the independently owned artifact contract. Remove ID-only internal handoff, repeated full-record/seed inventory reads and decode duplication; keep public explicit SolutionId selection as identity, resolving it once.

## Remaining source opportunities and settling branches

The original inquiries contain opportunities beyond the finding IDs. They must be covered without calling unexamined breadth defective. Route D-04 repeated preparation and interleaved/twice study admission to the preparation engagement package; JSON ingress remains a real validation boundary. Retain MathPolicy.jobs=4 and existing unknown-allocation guards during this change; reference jobs=32 is a fixture, not deployment evidence. Runtime owner may revise defaults only after representative concurrent native/fit/dynamics population and allocation observations, as specified in the investigation evidence. Retain ephemeral in-memory frontier rescanning until measured study-policy effort is material; durable readiness/continuous refill remains selected. Independent fit experiments with ordered assembly, cloned sparse response/constraint structure and double-forward transient objective/gradient belong to numerical demand analysis; select parallelism only within common admission and preserve deterministic addend order. Dense uncertainty GEMM/diagonal demand requires a large-output consumer and the existing scalar-sum bit contract; no unconditional rewrite.

E/L2 opportunities: repeated payload digests, own-writer IPC preflight and byte copies route to assurance/IPC ownership; read-side preflight/admit_owned remains justified at the trust boundary until an owned checked transport proves the replacement. Fixed 64-row membership pages/history intervals, populated latest_results EXPLAIN and repeated result_analysis read_results route to preparation/selection owner; byte bounds and exact interval semantics precede any index choice. Read coalescing via Arrow BatchCoalescer and buffered IPC file writer are cold optional improvements after larger publication blocks; measure remaining tiny-block/file-write demand. JSON u64 seed arrays preserve exact bits; binary seed encoding needs version/compatibility/coverage benefits before adoption. The bounded read-side choices and their reopening triggers are dispositioned in the investigation evidence. One reconciliation absolute deadline remains authoritative; large trajectories should use accumulated normal-path coverage and bounded recovery with honest incomplete results, never per-page renewed deadlines.

`ensure_writes` config reads are advisory and must not become quiesce authority; supervisor drain remains decisive. Managed executable/generation hashing belongs to the assurance owner, with E-P2 already-measured per-pass evidence not mistaken for assembled startup time. Dead file-cache/load/predicate/store envelopes and build feature residue belong to relational/build owners (L2-01/P2); metadata-preserving DataFusion wrappers fill a real 55.1 library gap, so removing them needs exact replacement evidence. Sorted source declarations can eliminate cold re-sorts only if providers establish ordering. Existing Flights has cancellation/completion semantics beyond quick_cache's async guard, so retain it absent a fitting replacement. LIVE SELECT versus rare 20ms identical-edit polling is optional; no new subscription lifecycle now. Keep the justified SurrealDB request-context patch/hold until upstream supplies its consumed deadline/cancel/admission/drain contract (L2-04); a thin hook/upstreaming is maintenance opportunity. BLAKE3 unused knob/feature and SHA256 alternatives route to assurance/build, with repetition removed before algorithm change. Declare exact JSON float-roundtrip feature ownership via F29 rather than relying on Clarabel. Do not introduce compression (currently refused), inbound Python Arrow, new database, cross-output product or blanket library replacement without a consumer and changed-contract assessment.

Package order is authority decisions → P-B1 protocol/P-B3 admission foundation in parallel → preparation engagement and P-B2 ready consumers → P-B4 demand/locators → P-B5 ack producer/consumer integration → P-B6 streaming and telemetry; P-B7 grant/replay settlement can design alongside P-B1 but collection waits on it. P-B8 completeness can ship independently by attaching its terminal witness to the existing authoritative seal. P-B5 must later preserve that witness. Admitted-object transport consumes only the slim completion-read slice, not the complete acknowledgment/publication redesign. Targeted compile/unit controls accompany each implementation package; regeneration accompanies declaration changes. Integrated canonical/native/Python studies, horizon and interruption journeys, performance observations and static/manual checks belong to the root's final qualification package. Success remains scoped; no source inspection or focused pass closes whole-product/scientific acceptance.

## Implementation checkpoint, 2026-10-12

P-B1–P-B8 now have integrated producers and consumers: exact conflict/retirement fences,
rotating ready discovery and refill, distinct population/resident/active admission,
typed locators and compact/full demand, acknowledgment-based finish, bounded horizon
roles, ordinary sampled telemetry, settled grant collection and admitted seed transport.
General staged sequences retain history for their explicit historical-start consumers;
bounded horizons use their separate role/window owners.

Functional controls exposed and corrected three admission defects: prepaid constructor
allowances were double charged; tentative growth could repeatedly wake its own active-job
waiter; and continuation screening requested another population slot outside its retained
lane. Known construction bounds and opaque prepaid guards now remain distinct. The
original deadline/cancellation and pre-allocation growth checks remain. Independent source
review and affected controls pass. The single-population continuation journey requests
explicit variable/row accuracy and independently checks original residuals and predicted
start provenance. Recovery maintenance also yields to study discovery instead of starving it.

The scientific Python sweep prepares at most one structural view per actual lane and
accounts exactly for all 1,000 point preparations; two concurrent mutable lane owners
are intentional. Its corrected control passes in the actual 49-test native Python journey. Managed success,
cancellation, compact/empty demand and actual sixteen-owner journeys remain P-G1
qualification obligations. No production content has been reclaimed by these fixtures.

Cancellation during queued publication now preserves its exact original refusal across
the later cleanup flush: completion intent cannot survive a failed group without its
authentic ACKs. The real cancelled-study regression and original SCIP/completion controls
pass together. Response delivery now joins native payload snapshots before returning to
an unknown cold producer, while the caller's decoded-output charge remains held. The
queue's immediate zero-drain oracle and subsequent cold-producer control now pass
against the actual native SDK path.
The compact continuation ownership fixture now explicitly selects POUNCE NLP with
Optimize intent and disabled presolve, because POUNCE-convex does not export a native
seed. An unbound declared-composition override correctly refused and was removed.
All original scientific, identity, seed and task-departure assertions now pass.

Managed Rust acceptance passed the actual sixteen-owner study, cancellation and
reopened continuation journeys. Its publication/reader journey reached successful
retirement after checking exact current/history rows, analysis retention and all sixteen
native publications, then exposed an eight-page fixture bound for a nine-page collector
protocol. Two attempts each require a batch, result-set, append-receipt and generation
page, followed by the completion page. The corrected bound derives from that fixture
inventory; exact payload count, post-retirement refusal and escaped-buffer checks remain.
Independent readback agrees; all four enclosing managed Rust journeys now pass.

The managed Python rerun completed six tests before its durable 1,000-point sweep
exposed a recovery race. Assigned-page discovery could outlive the original owner's
terminal observation; recovery's fresh snapshot was already unassigned/settled, but
recovery attempted another observation and stopped the worker. Recovery now rechecks
fresh assignment/settlement before acting, including leaving retryable unassigned
points for normal claim policy. Independent source readback confirms the race and
the guard's ownership boundary. Both revealing canonical controls now pass, including
settled and retryable states and original-owner terminal projection. Rebuilt worker and
extension acceptance remains pending. The interrupted client and exact receiver
are drained through their actual owners; unfinished outcomes and retention pins remain.

The rebuilt managed cancellation journey then exposed a distinct recovery race:
background maintenance treated the cancellation bit as authority to fence a still-live
native owner. Recovery now requires terminal readback or lease expiry, and respects
the existing local candidate turn through native drain and terminal observation.
Cancellation still closes new entry and reaches the original owner's typed cancellation
path; it does not establish physical handoff to another worker. Revealing controls cover
an independent worker, an expired but locally owned attempt, and recovery after local
turn release. All four latest-source recovery/rotation/completion controls pass with
enclosing cleanup; independent source readback found no material flaw. This simulated
turn control is distinct from actual native cancellation/drain acceptance. All four
managed Rust journeys now pass on the rebuilt worker with enclosing cleanup, including
the revealing sixteen-owner cancellation journey. Managed Python acceptance on the rebuilt validated native extension was interrupted after the seal failure recorded below.

The rebuilt managed Python sweep reached native execution, then exhausted the existing
bounded definite-conflict retries while sealing an original result. The initiating
failure was the seal transaction, not its later cleanup refusal. A held-transaction
control reproduced an unnecessary conflict between terminal active-root removal and
an unrelated source creator in the same problem. The proposed ADR-0177 contract now
distinguishes pure removal from new dependency creation: release retains exact-root
and reclaim registration, while the run root continues retaining the source. The
generator removes only this release's shared creator notification. Actual creators keep
their two-way reclamation fences. Regeneration and both-order release/collector and independent-creator controls are recorded at the coordinator’s stop checkpoint; rebuilt managed acceptance remains pending. The interrupted
client/controller and exact receiver have drained through their actual owners;
unfinished outcomes and retention pins remain unchanged.


## Maintainer stop handoff, 2026-10-12

Execution stopped after the requested immediate wrap-up. The [coordinator’s stop checkpoint](36-efficiency-alignment.md#maintainer-stop-checkpoint-2026-10-12) owns current evidence boundaries, worktree reconciliation and the remaining execution route. The generator correction and both-order seal/reclamation controls are recorded there. Actual managed Python sweep success and refreshed managed/native acceptance remain open; failed or incomplete outcomes retain their ownership and pins.
