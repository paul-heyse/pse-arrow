---
title: Durable execution and dependency-scoped studies
status: in-progress
date: 2026-10-05
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md, docs/design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md, docs/design_review/reviews/design_review_plan-28-completion_2026-10-06.md, docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md, docs/design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md, docs/design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md, docs/design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md]
scenario_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#representative-journeys]
---

# 28c: Durable execution and dependency-scoped studies

## Responsibility and current foundation

This companion of [Plan 28](28-surrealdb-unified-substrate.md) owns the ordinary durable-run
default, attempt lifecycle, study readiness, result staging and coherent completion. It
consumes [28a's identities, codecs and transaction contract](28a-canonical-substrate-and-revisions.md)
and [28b's selected compilation descriptions](28b-selected-compilation-and-reuse.md).
[28d](28d-connected-results-and-analysis.md) owns connected reading and analysis of sealed
outcomes. Current shared contracts and finding dispositions belong to the coordinator.

The current `workflow/run.rs::RunResult::finished` and `DurableAttempt::finish` retain attempt
completion, progress, incumbents and seeds. Ordinary scientific result tables in `results.rs`
still require explicit publication. Studies already retain sources/definitions and publish
point results through their workflow. The target extends and consolidates this durability;
it does not start from a wholly ephemeral system.

Reuse physical/modeling admission, solver routing, native attempt ownership, completion and
the scientific portion of `study_policy.rs`. Replace the complete-study snapshot/action
derivation in `Studies::admit_dispatch`, PostgreSQL job claims and the cross-store publication
protocol. Preserve Plan 27's contextual accuracy and partiality distinctions, actual warm
starts, permission restrictions, predecessor usability and occurrence identity.

## Plan 30 receiver and operation integration

[30a](30a-native-websocket-rpc-and-operation-lifetimes.md#submission-cancellation-and-settlement)
supplies the replacement RPC lifecycle to existing run/study/activation owners. C's
immutable acknowledgments, fenced claims, original starts and truthful terminal
assessment remain authoritative; reconnect or caller expiry cannot authorize another
scientific attempt. Submitted storage uncertainty and native drain are distinct.

[30b](30b-persistent-services-and-receiver-generations.md#receiver-coexistence-and-transition)
admits independent functional receiver generations/context associations while preserving
C7's bounded scientific group within each receiver and the exact reference primary/
observer lane. [30c](30c-test-ownership-and-evidence-retention.md) supplies test contexts;
[30d](30d-host-admission-and-timing-qualification.md) supplies shared host admission.
Plan 30 owns these new mechanisms and status. Existing C implementation receipts and
28e scientific qualification retain their original conditions.

## Ordinary runs and attempt state

An application run selects an immutable problem revision and records demand, resolved
configuration, compilation/provider interpretation, actual deployed artifact association and
complete outer source/build observation before native execution, under the accepted
[28h identity route](28h-native-setup-and-artifact-identity.md). Scientific results and available
diagnostics are retained automatically for normal
completion, failure and cancellation. Storage or admission failure is observable; a caller
does not receive a successfully durable outcome when retention failed. An explicitly selected
ephemeral mode remains available to numerical libraries and local tests; it is not selected
implicitly because a store argument was omitted. Runtime construction supplies or opens the
configured durable substrate through A1. Direct kernel/solver tests need no unrelated server.

Keep semantic run and actual execution attempt separate. A retry of the same effect uses its
operation identity; a genuine new attempt has its own generation and observations. Record
the selected start actually used, including source predecessor and eligibility. Progress and
incumbents are observations rather than successful result sets. A failure can retain useful
scientific data without becoming an eligible predecessor or a usable result by accident.

Candidate discovery is separate from the short transaction that claims a candidate. The
claim transaction rereads the candidate, its relevant dependency states, cancellation and
admission generations under named record conflicts. Mutations of every decision premise
must touch the corresponding guard/generation. Successful claim produces an attempt fence;
native work runs outside the transaction. If the transaction conflicts, retry the entire
decision from fresh reads, not only its final update. `FOR UPDATE` does not provide predicate
locking or turn an unguarded graph query into serializable policy.

Claim expiry/recovery and cancellation revoke stale workers' admission authority. They do
not imply that the native process has stopped. The owning supervisor cancels/drains/releases
native sessions and buffers, and keeps stale generations unable to publish progress or
completion. Storage retries have bounded operation scope; a lost acknowledgment is resolved
by operation identity and recorded state before retrying a scientific solve. Cancellation
checks include the terminal-seal transition, not merely the worker's last polling point.

## Staging and coherent results

C1 establishes the terminal envelope and staging identity before C2 and D1 need them. A
bounded writer accepts typed scientific rows/blocks using A2's exact codec, under
`run/attempt/generation/result-set` identity. It records batches idempotently with sequence
and completeness information. Staged rows are private to that attempt; canonical queries
cannot expose an unsealed prefix as a complete result. A retry with different bytes at the
same batch identity is an error, not an overwrite.

Close result-set membership before terminal admission. Each batch write conflict-checks the
attempt's open-ingestion gate. Closing ingestion fences further writes; reconcile the now
immutable batch selection and coverage outside the terminal transaction, then record its
closed descriptor. The short seal checks that descriptor and its fence/generation. Readers
follow exactly its admitted batch/coordinate selection, never every row sharing a result-set
ID. Closing after partial failure retains truthful coverage; closure is not success. Restart
can finish reconciliation/sealing of the same closed operation without rerunning native work.

After native work and required result ingestion finish, a short transaction verifies the
current fence, cancellation, scientific completion/admissibility and staged result-set
completeness. It atomically records the actual terminal outcome and admits the corresponding
result sets. Do not place an entire trajectory, graph calculation or native solve inside this
transaction. Failed/partial/cancelled outcomes can seal their available observations with
explicit quality and coverage. A success summary cannot bypass the numerical conclusion,
physical eligibility or contextual accuracy requirements.

The reader sees either an admitted terminal selection or the explicitly requested progress
class, never a mixture selected by incidental write timing. D1 uses a recorded monotonically
ordered run/attempt sequence for latest selectors; wall-clock order is not authority. Repeated
finish after lost acknowledgment settles the same terminal operation. A stale generation,
missing batch or cancellation arriving before sealing cannot produce successful visibility.

Long-lived reads and cleanup use A3's protected-selection contract. Unsealed abandoned staging
can be reclaimed after its attempt is fenced and no recovery/read obligation remains.
Retention of failed diagnostics and partial data is deliberate policy, separate from orphan
cleanup. Result backpressure limits queued batches and buffers independently of native work
and solver-state memory. The repository's memory-capped native recipes remain required.

## Study policy and preparation

Store study definitions, distinct point occurrences, dependency edges and consumed start
policies in the same substrate. A point's readiness query examines the relevant predecessor
outcomes and current cancellation/generations, or evaluates a coherent group when policy
truly requires it. The policy remains one semantic owner. Express supported structural
readiness in native queries/functions and share the existing scientific predicate kernels;
do not create separate Rust and SurrealQL versions of the same evolving scientific rule.

Migrate sweep, continuation, fitting, dynamics and composed workflows, including study
finalization and recovery. An unsuccessful predecessor does not satisfy a usable-start rule
unless the explicit policy admits its available data. Repeated parameter values remain
distinct occurrences. A fan-out study should not derive all actions anew for each individual
dispatch. Whole-study checks, where real invariants require them, occur once or in a coherent
batch and expose their actual scope.

Separate pure study-wide admission from expensive numerical preparation. Resolve necessary
definitions and graph validity before dispatch, then prepare expensive artifacts when an
attempt is ready. Share compatible immutable products through B3 across equal structural
selections. Bound case preparation and native concurrency together; separate pools cannot
independently fill the machine. The local application supervisor owns a finite worker group
and partitions its configured process budgets among the database and native workers. Existing
process-local admission enforces each worker's allocation. Expired attempts remain charged
until their native work drains. Unmanaged external workers and distributed capacity scheduling
are outside the initial qualification profile. Cancellation drains prepared work and releases native state
without deleting canonical scientific history.

## Stored-study admission and creation cancellation

The enhancement review's [F01](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#f01)
and S06 add C5 to the existing study owner. At `dacc9c3`, `start_defined_study` loops over
occurrences and calls `admit_binding_seed_need`, which freshly constructs `DeclaredStudyAdmission`.
The authoring path already retains a compatible adjacent basis, but its thousand-point unit does
not exercise stored-definition readmission. The default durable `ModelingPackage::study` and
Python journey reach the repeated path; selected-admission retention alone does not remove it.

Use the existing `DeclaredStudyAdmission` mechanism within stored creation. Reuse only its compatible
declared structure and path-demand basis; independently validate every decoded binding identity,
physical/source context, operation descriptor and bound seed need. Replace/release the basis when
its structural premises change. Do not retain an expired protected read, attempt fence, mutable
solver view or successful predecessor inside it. N2 supplies checked selected preparation; C5 owns
study composition and per-occurrence policy. N4 reconciles all actual callers, including stored/default
entry points rather than only initial authoring. Equal bindings still create distinct occurrences.

Creation must consume the caller's cancellation scope through source acquisition, admission and
canonical ingestion. Replace the detached fresh cancellation source at `start_defined_study`;
migrate direct Rust callers and the default application/Python path together. Native attempt
cancellation, claim fencing and drain remain at their current owners.

The visibility boundary is canonical **activation**, not the first write. `create_study` creates an
inactive study and ingests bounded occurrence batches before activating it. Check cancellation during
preparation and between batches, and immediately before activation. Cancelled inactive ingestion
follows existing recovery/reclamation; immutable physical-source publication need not be rolled back.
Once activation has been issued, dropping its future cannot prove it did not commit. Settle the exact
study identity under the existing operation owner, then durably cancel an activated study if required.
After acknowledged activation, record cancellation before further dispatch. No ambiguously active
study may be silently abandoned; lost acknowledgment is not permission to repeat scientific work.

| Package | Delivered behavior and prerequisites | Migration, deletion and targeted acceptance | Status |
|---|---|---|---|
| C5 — Stored creation basis and cancellation | Working N2 selected admission and canonical inactive-ingestion/activation owners. Share compatible declared entry preparation, admit every binding/seed role and carry caller cancellation through creation/activation settlement. | Move actual stored, default durable, direct Rust and Python routes; delete per-point fresh declared preparation and detached creation cancellation. Test many value points, invalid later binding, changed descriptor/settings/layout/source, distinct occurrences, cancel during hydration/admission/batches, before activation, after issued/lost-ack activation and after acknowledged activation. Preserve fences, recorded starts, inactive cleanup and drain. | Implemented; actual stored-creation and cancellation controls pass; E3/E4 pending. |

E3 includes the original thousand-point Python workload under production accuracy/resource premises.
Construction-count controls demonstrate removal of repeated preparation; elapsed-time benefit needs
E4 measurement. Actual Python failure diagnostics remain E1's obligation, not a diagnosis inferred
from the review or the unfinished run's duration.

## Remaining recovery and study slices

The [coordinator](28-surrealdb-unified-substrate.md#finding-dispositions) owns findings;
[28e](28e-rebuild-retirement-and-qualification.md#remaining-assembled-execution) owns the
assembled campaign. The [capability investigation](../design_review/evidence/plan-28-surrealdb-capabilities-2026-10-06/README.md)
supports native structural locality and operation-identity recovery in the existing C owners.

| Slice | Inputs, transition and affected consumers | Targeted acceptance and replacement |
|---|---|---|
| C2/C4 lifecycle handoff, after A3 correction | Exact source revision plus run/attempt identity creates immutable owning roots and ordinary retained scientific outcomes. Drain/cancellation releases native work through the owner; terminal readback distinguishes failure, partial usable outcome and success. Runtime durable/staging/retention and result readers share the association. | Compose generic-root refusal with a live run, active attempt, retained cancelled/partial result, reopen and reclaim. No generic retention workaround or old publication/settlement path may survive. |
| C3/C4 uncertain study claim | Occurrence/operation identity, scope, guard premises and generation yield one authoritative native claim. After an acknowledgment is lost, query that identity and settle the committed claim before another start; definite conflict retries the complete readiness decision. Owners: `canonical_studies.rs`, study policy/driver and managed workers. | Inject acknowledgment loss after committed study claim, then recover the same occurrence/start/generation without duplication. Cover competing workers, expiry, stale generation, cancel-before-start and cancel-after-start/drain. If current source already supplies recovery, validate it; otherwise extend that owning operation rather than creating another scheduler. |
| C3 progress/backpressure and managed interruption | Bounded progress delivery must allow terminal settlement and native drain even when an observer queue fills or its consumer disappears. Worker loss preserves exact started/unstaged/staged/closed/terminal distinctions. | Execute the failing event/progress journey under its real queue premise, plus two-worker kill/restart and the existing SCIP interruption workload. Require canonical terminal correctness and bounded queue/lifetime, not delivery of every diagnostic event. Remove displaced full-study dispatch/obsolete recovery callers when covered. |

Versioned native functions already own atomic structural claim/fencing, staging and sealing;
Rust shared policy owns scientific readiness and interpretation. Reuse that split and keep
function/schema generation registry-owned. A native traversal/function may compact discovery
but cannot invent another scientific predicate. LIVE subscriptions and changefeeds are optional
observer hints only: reconnect/retention can lose notifications, so authoritative decisions
must reread canonical state. No new notification transport is needed for current acceptance.

C's focused controls below do not establish deployed managed-worker recovery. Execute targeted
controls with the admitted profile; E3 then composes concurrency, cancellation, progress,
retention and restart with refreshed producers. Changes to global/contextual accuracy require
physical decision rationale and their policy owner, never a study-specific tolerance adjustment.

## Shared preparation and bulk ingestion extension

The [repository-wide extension](28-surrealdb-unified-substrate.md#repository-wide-efficiency-extension)
adds [28f N2/N3](28f-shared-numerical-preparation.md) and [28g T1/T2/T4](28g-bulk-data-operations.md)
to the durable/run/study consumers. Ready occurrences acquire bounded immutable checked
preparation under complete eligibility and fresh protection. Equal parameter values remain
distinct occurrences; compatible native owner reuse never merges claims, starts, attempt
fences, cancellation or independently assessed outcomes.

Result writers choose physical row groups by the shared payload/index/resource admission
instead of the 64-cell-driven IPC unit. Successful append returns a sufficient immutable
acknowledgment; the existing effect owner retains exact changed-replay refusal and lost-ack
settlement. Atomic indexes, closed ingestion/coverage and terminal admission remain C's contract.
Source/analysis grouping changes only after its specific effect/recovery unit is settled.
N4/T5/L4 closure and current C obligations precede E3; earlier Outcomes do not qualify these
replacement mechanisms. No new scheduler or recovery authority is introduced.

## Parallel study frontiers and durable workers

C6/C7 implement the accepted Parallel RC04 direction and consume RC03's supported native
placement. [The coordinator](28-surrealdb-unified-substrate.md#parallel-execution-integration)
owns F05/F03 dispositions; [N8–N10](28f-shared-numerical-preparation.md#parallel-admission-and-native-lifetimes)
and [L7](28h-native-setup-and-artifact-identity.md#parallel-deployment-and-supported-native-placement)
supply admission, native scope and deployment contracts. The `84a1caf17656f00f38b22e703c7cbc2b63a44d2d` ephemeral driver opens
one Staged, prepares frontier cases sequentially and uses generic serial batch execution except
for eligible native coefficient/cone bulk operations. The durable local loop calls work_once
sequentially; the currently selected two external workers do not establish sixteen-case studies.

### C6 — Independent ready sequences

Compose a bounded set of independent sequence drivers using existing async completion polling,
preparation, Staged/native sessions and assessment operations. Each sequence owns private mutable
state; immutable selected/compiled products remain shareable. The orchestration owner alone
consumes the existing pure occurrence policy and applies transitions. Do not reinterpret policy
inside another scheduler or make a backend selection depend on occurrence internals.

Issue only policy-ready occurrences and bound both active preparation and execution under the
same deployment. Replenish completed slots without a discovery-order execution barrier; place
outcomes by exact occurrence identity before deterministic reduction/report assembly. Avoid
preparing a whole frontier into retained products before admitting execution. Close completed
sequence owners promptly. Genuine continuation/start dependencies retain serial reuse until the
required predecessor result and exact seed provenance exist; repeated equal bindings remain
distinct occurrences/attempts. Preserve original default/explicit starts, retry revisions and
original-model assessment.

Retain a suitable native bulk operation for compatible ready members with its admitted thread
extent and per-member semantics. Do not require a full group before useful work can progress
or route every ordinary independent case through the generic serial batch merely because the
interface is named batch. N10's strategy constrains actual native phases without erasing the
independent-case composition target.

A failed point follows existing scientific availability policy and does not silently fail its
independent siblings. Cancellation or a fatal enclosing infrastructure condition stops new
issuance, preserves unattempted inventory, drains all issued operations and explicitly closes
sessions before resources are returned. Dropping futures is not native drain. Migrate public
Rust and Python ephemeral studies and their sweep/continuation/fit/dynamic compositions where
this ordinary driver is used; a private conformance helper is not the migration boundary.

### C7 — One coordinated durable realization

Use one primary concurrency realization selected by N10/L7: a finite group of existing sequential
managed worker handles, or bounded concurrent work_once execution sharing one runtime where its
native strategy permits. Do not multiply sixteen process workers by sixteen independent local
claim loops. The caller/observer's assisting execution consumes the same declared allowance;
otherwise it observes without an extra unbudgeted native worker. C7 activates the chosen group
only after L7 supplies numeric CPU/memory placement and receiving-process reconstruction premises.

Keep discovery distinct from atomic claim authority. Preserve exact occurrence/generation/start
lineage, heartbeat through terminal settlement, original-model outcomes, bounded progress/result
ingestion and existing uncertain-acknowledgment recovery. Worker replacement cannot duplicate
an acknowledged or uncertain started scientific attempt. Expired attempts stay charged until
native work drains under the selected supervisor. Parent/process cancellation stops issuance,
records the existing fences and drains actual children; interruption preserves recoverable
partial history rather than silently switching to ephemeral execution.

| Package | Inputs and delivered behavior | Migration, deletion and targeted acceptance | Status |
|---|---|---|---|
| C6 — Concurrent independent sequences | Accepted RC04; pure occurrence policy and working A4/B6/N8/N9/N10 slices actually consumed. Compose ready sequence workers while retaining true dependency order and suitable native bulk paths. | Move ordinary Rust/Python ephemeral study driver and composed callers; delete displaced generic single-session frontier dispatch once replaced. Test sixteen fresh non-batching cases with actual overlap, exact/repeated identities, starts, deterministic placement, dependency chains, failed-point isolation, finite admission and cancel/drain. | Implemented; ordinary overlap/cancel/drain and action-bound controls pass; managed and assembled qualification pending. |
| C7 — Durable parallel composition | C6's occurrence semantics, existing claims/settlement, N10 strategy and L7's working selected deployment/reconstruction. Compose one finite durable worker realization. | Move default durable study caller, workers, supervisor and Python entry points together; delete displaced unbudgeted assistance/serial realization where replaced. Test ordinary sixteen-point study, competing claims, stale generations, killed worker/reopen, lost acknowledgment, partial results, heartbeat/backpressure and cancellation through native teardown. | Implemented; default observers and bounded primary serve are migrated; actual managed placement and composed recovery qualification pending. |

C5's stored creation basis/cancellation stays intact; concurrent execution does not merge
occurrences or repeat declared preparation per point. C6/C7 consume B6/T7 only where their
canonical lifetime/operation changes are needed, rather than waiting for whole companion
completion. Their consumer closure joins N4/T5/L4 and E3; new package completion does not
resolve the broader finding without its complete public-route evidence.

## Complete preparation through study execution

The [preparation integration](28-surrealdb-unified-substrate.md#preparation-assurance-and-reuse-review)
adds C8. [28j](28j-pure-preparation-and-publication.md) supplies the basis/effect contract,
B7 supplies working selected preparation, and [28i](28i-runtime-validity-and-interruption.md)
supplies applicable receiving and stop ownership. C owns actual frontier, ready-candidate,
worker claim, occurrence and drain semantics; none of these becomes cached permission.

Share compatible stable preparation through actual ephemeral and durable ready execution,
including managed worker roots. Stored creation's existing basis reuse remains a strength;
it is not evidence that ready execution avoids pre-view preparation. Independent ready cases
reuse immutable products while preserving distinct occurrence/start/result placement. Dependent
sequences retain their declared order and warm-start provenance. Changed source/structural
binding/demand/provider context gets its required preparation before native claim/entry.

| Package | Prerequisite and target behavior | Completion and status |
|---|---|---|
| C8 — Ready study basis consumption | Working B7/J2, current C6/C7 dispatch/claim contracts, applicable I1/I2. Move ephemeral frontiers, durable ready candidates, ordinary Rust/Python callers and managed workers to shared preparation plus fresh case/occurrence effects. | Implemented; targeted cancellation/private-drain controls pass and displaced preparation paths are removed. Fresh selection/publication and independent numerical state remain per occurrence. 28e owns the rebuilt thousand-point and managed scientific qualification. |

The existing thousand-point flash sweep remains a supported completion journey. Small
targeted composed controls first expose basis/body/observation/encoding work rather than
only view counters. No point-count reduction, solver-tolerance relaxation or new scheduler
substitutes for the target. A stopped follower cannot destroy preparation needed by others,
and native work remains charged until it exits.

## Work packages

| Package | Prerequisite and delivered behavior | Consumer migration and deletion | Status |
|---|---|---|---|
| C1 — Lifecycle and result envelope | R0; working A1/A2 operations, codec and guard slice. Implement run/attempt identities, terminal classes, operation deduplication, cancellation fences and staging contract. | Migrate lifecycle types/registry declarations and unit policy consumers; expose the real shared contract to D1 and C2. | Implemented; focused native controls exercised; integrated qualification remains in E3. |
| C2 — Automatic durable execution | C1, B2 descriptions and D1's minimal canonical result reader. Connect ordinary Rust/Python runs, bounded staging, atomic sealing and restart recovery. | Replace optional-store implicit ephemerality and ordinary explicit publication. Delete displaced `DurableAttempt`/publication mechanisms and callers when covered. | Implemented; focused retention, progress and recovery controls exercised; linked Python controls and E3 remain. |
| C3 — Dependency-scoped studies | C1, B1 selected inputs and B3 preparation slice; C2 terminal selection. Implement native discovery, fenced claims, scoped/batched policy, starts and finalization. | Move all study drivers/workers and recovery consumers. Delete PostgreSQL jobs/leases and complete-snapshot-per-dispatch paths, fixtures and tests for those mechanisms. | Implemented; focused continuation/finalization controls exercised; migrated worker journeys compile and await E3 execution. |
| C4 — Interruptions and retained lifecycle | C2/C3 and A3 retention/read protection. Complete drain, stale generation rejection, uncertain acknowledgments and bounded staging reclamation. | Move operational maintenance and diagnostics. Remove cross-store reconciliation, publication intents and member-prefix cleanup as their last consumers migrate. | Implemented; scoped retention/race controls passed; assembled interruption journeys remain in E3. |

C1 supplies only the shared lifecycle contract; its completion is not completion of automatic
retention or studies. D1's small sealed-result reader breaks the apparent execution/query
cycle. Root owns edits to shared workflow declarations and generator invocation. Numerical
accuracy owners remain responsible for their guarantees even when their observations move.

## Verification

**Proposed acceptance:** targeted unit checks and touched-package compilation accompany
each mechanism, using repository recipes with explicit force-validation. Native checks use
memory-capped recipes. [28e](28e-rebuild-retirement-and-qualification.md) owns assembled
journeys and the final campaign; these packages do not repeat integration suites at each row.

Exercise S03/S07/S08 with independent policy expectations: distinct repeated occurrences;
eligible and ineligible predecessors; concurrent claims of one candidate; disjoint candidates;
cancellation/revision change between discovery and claim; expiry followed by a stale worker;
and whole-decision retry. Race batch writes against ingestion closure and sealing; late writes
must be refused and readers must use only the frozen admitted selection. A graph predicate
race must conflict through its named premise,
including a previously absent identity, rather than pass only because the test is sequential.

Force failure after a staged batch, after terminal commit and before acknowledgment, during
native work and during cancellation. Reopening by operation ID must distinguish unsealed
partial work, an already admitted terminal outcome and a recoverable claim. Compare expected
visible result coverage and actual start coordinates directly. A failed or partial terminal
run remains queryable without satisfying a usable-result selector.

Check that increasing independent study occurrences does not invoke full-study policy per
dispatch, and that deferred preparation prepares only ready consumers. Mechanism controls
establish removal of repeated work; timing improvement remains unmeasured until E4. Durable
process-kill recovery and storage acknowledgment behavior require A1/E3's selected server
profile, beyond pure in-memory policy units.

## Checkpoint and next step

C8 is implemented through the common B7 preparation entry used by ephemeral frontiers,
durable ready candidates and actual workers. Each occurrence retains current selection,
publication and numerical state; shared flights contain only pure work. Private receiving
drains before source protection releases. Rebuilt thousand-point and managed study controls
remain in qualification. C5/C6/C7 evidence retains its original scope; the coordinator owns
PA02/PA04 closure.

C6 now has bounded completion-driven preparation/execution, actual eligible batching,
private independent session owners and unambiguous continuation transfer. C7 now has one
bounded group on the shared runtime, local preparation ownership before canonical claims,
exact total-action admission and stop/failure draining. The ordinary durable study caller
observes the configured managed primary; its former serial assistance path is removed.
Public controls use the original model's eligible non-batching Ipopt adapter and refuse
fewer than two population slots before effects. The effect-free durable sixteen-occurrence
action-bound and ordinary native overlap/cancel/drain controls pass. Actual managed receiving
placement and primary-only native overlap are not yet accepted. L7 activation precedes that
qualification. Earlier C1–C5 evidence retains its original scope.

C5 now shares a compatible declared basis on the actual stored-definition route and checks
every binding/seed policy while preserving distinct occurrences. Caller cancellation reaches
source validation and creation; issued source/protection/study effects settle before cancellation
is reported. Exact activation and lost-acknowledgment controls fence the study identity, including
idempotent retries of an already activated study. Focused runtime and canonical-server controls
pass. Benchmark and public Python callers are migrated: the original eight-point ephemeral
and standalone durable journeys retain their distinct meaning, while the new sixteen-point
selector and managed-primary controls supply separate parallel coverage. Rebuilt Python and
actual managed journeys precede N4/E3/E4 acceptance. Earlier checkpoint evidence below keeps its original boundaries.

C1/C2/C3 now use canonical native lifecycle functions and protected exact result
selections. Ordinary runs retain original Arrow blocks, diagnostics and qualified seeds;
studies retain compact run/attempt handles rather than native reports. Candidate decisions
use the candidate and its immediate premises, with deferred numerical preparation. The local
adapter admits the study once and prepares only its bounded ready frontier. Both the local
and reopened durable continuation controls exercise the shared secant proposal followed by
the original corrector. Durable seeds retain exact portable qualified point evidence;
library-owned native factors are reconstructed only for an actual prediction consumer and
remain process-local. Explicit absent seeds use the shared policy's structured refusal.

Physical ingress, definition admission and ready occurrence preparation now share a bounded
Runtime-local immutable admission for the exact physical revision and checksum. Ordinary
physical IPC staging reuses only the original checked source owners, quantities, preconditions
and package metadata under the same Operations, registry and session owners. Each purpose
retains at most one product with a finite byte ceiling. Hits acquire fresh native protections;
active consumers retain those guards through durable root admission. Clear and eviction fence
in-flight loads without invalidating active values. Focused controls cover actual document/row
reuse, changed support despite equal physical identity, changed revisions/owners, reclamation
refusal and clear races before and after native source lookup. The actual study consumer and
retained secant/original-corrector controls preserve the scientific outcomes; assembled
capacity and performance claims remain in E3/E4.

Linked scientific consumer checks exposed a preclaim refusal being replaced by a later
cleanup error. Registration now keeps an acknowledged run header before claiming and
preserves the original typed refusal when no attempt was established. Cleanup failures
remain observable in the durable record. Focused refusal and successful-manifest controls
pass; the underlying linked reference/SCIP registration failure is being diagnosed before
those scientific consumers can qualify.

Progress observations are projected by borrowing the native event, coalesced under the finite
flush policy, and split into bounded immutable receipts with exact event counts and integer
sequences. Native controls cover batching and truthful cancellation on backpressure. Study
discovery returns an actual bounded `StudySummary`, without a fabricated metadata payload.
Parent-summary admission atomically concludes the study; live competing writers and expired
effect-free writers are handled without rerunning numerical points. Focused controls exercise
pre-cancellation, summary recovery and cancellation after summary closure.

The retained original-root prediction fixture's final surrogate correction exposed a test
precision mismatch: a relative fraction without a declared engineering scale correctly uses
the ordinary canonical allowance. The fixture now requests its unchanged numerical assertions'
precision through an explicit scalar physical allowance; the focused native rerun passed with D's
result controls. Occurrence discovery now carries actual ordinal/outcome fields in a typed
`ScopedStudyPoint`; full scientific descriptors are fetched explicitly by their exact keys. The child-process worker journeys have moved to canonical study
claims, exact retained attempts and physical revisions, preserving authored square/data,
warm-seed, killed-worker, long-SCIP cancellation and two-worker scenarios. Those journeys
compile; E3 must execute them in the supervised profile. No assembled-process or
performance qualification is established by the focused mechanism controls.

C4 implements explicit result retirement, source-root release and restartable bounded cleanup.
Live readers and retained studies/analyses prevent retirement; tombstones prevent replay from
resurrecting a retired execution. Focused controls exposed and repaired an uncertainty fallback
that returned a retired run receipt. C4's focused native controls exercise protected buffers, exact retirement conflicts,
restartable cleanup, immutable lifecycle receipts and all-source root release. Assembled
interruption and consumer qualification remain in E3. This document owns C status; the coordinator owns
finding dispositions. Integrated qualification waits for all companion functional scope.

## Outcome (recorded after implementation)

### What was built

**Implemented:** canonical run registration, fenced attempts, bounded progress/result staging,
atomic scientific terminal admission and acknowledgment settlement. Ordinary runs retain their
actual outcomes. Dependency-scoped study discovery, ready-point preparation, distinct occurrence
identity, qualified predecessor starts and recoverable summary finalization replace the old
store/publication mechanisms. Bounded exact physical admission and IPC receipt reuse retain
their source owners through run admission; cache clear fences pending loads.

**Tested (2026-10-06):** targeted native `just unit-native-package` controls under explicit
`canonical-tests,native-solvers` and force-validation passed the five
`physical_admission_cache_unit` controls, the equal-binding/failed-usable study control and
`canonical_study_reopened_continuation_uses_shared_secant_and_original_correction`.
Durable constant-result reopening and registration-refusal controls passed; scoped native
claim/retention controls passed through `just unit-package` with canonical tests enabled.
The baseline was zero failures. These focused observations precede the later build-output
replacement and do not establish E3's assembled worker interruption or E4's measurements.

### A mistake made and corrected

Registration failures could be hidden by a later cleanup/sealing failure. The run boundary now
retains the original cause and refuses a successful result or fabricated attempt header.
Cache clearing initially fenced only a later load phase; capturing the generation before the
first source await also prevents an earlier in-flight load from republishing after clear.

### Deviations from the plan, deliberate

Physical admission reuse is private and bounded to one exact document and one exact IPC
receipt set rather than introducing another public cache configuration. Retained immutable
owners are shared; mutable numerical workspaces remain per attempt. Assembled recovery,
capacity and timing claims remain owned by E3/E4.
