---
title: Rust runtime lifecycles
status: draft
date: 2026-10-28
adrs: []
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, architecture, libraries, validation]
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md, docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md, docs/functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md]
---

# Rust runtime lifecycles

This supporting plan develops O06 of the [Rust-owned functional
review](../functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md) under [Plan58](58-rust-owned-system.md). It is
new target-oriented scope replacing Plan57's frontend-shaped lifecycle decomposition. The package table and
checkpoints distinguish implemented local/host slices from remaining work. Planning does not
authorize implementation, resume interrupted qualification or attribute existing native failures to Python.

The [hard-cutover review](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I02),
published at `c3b7f136d`, supplies I01/I02/I04/I10/I11/I12/I14. Its coordinator-owned dispositions
replace preservation of Python-shaped coordination with actual native ownership and canonical durable policy.

The 2026-10-29 amendment incorporates the [database-boundary review](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md)
at `a6a144bce`; its opportunity references are source-qualified to distinguish them from the earlier
review. Existing dated acceptance below is preserved. The coordinator owns dispositions, D owns
canonical meaning and C owns actual execution/resource custody. This amendment supplies no new
runtime or qualification evidence.

## Target and foundation assessment

Retain the current Tokio 1.53.3, tokio-util 0.7.19, Futures and Rayon composition, original native-job admission and actual OS-thread joins
as the foundation. Async tasks own polling/I/O and service coordination; Futures compose scoped operation work; native compute and
library teams retain their current scientific/native supervisors. `math/joined_pool.rs` explicitly drops the Rayon pool and joins every
retained worker, including partial construction. `native_thread.rs` and `math.rs` retain actual native custody. A canceled future,
finished task or dropped pool is insufficient evidence that a started foreign call and its destruction ended.

[B](58b-bootstrap-and-host-control.md) owns the delivered host integration and its deliberate Tokio move
to exact 1.53.3 for zbus 6.1 backend compatibility. Its dated checkpoints retain actual evidence and
remaining limits; C consumes the lifecycle contract without reopening that dependency choice.

The key distinctions are observing, requesting scientific stop, joining native execution, publishing durable results, draining delivery
and physically releasing resources. Existing `workflow/worker.rs`, `operation_preparation.rs`, `math/{joined_pool,native_child}.rs` and
`execution_lifecycle.rs` are the relevant foundations; do not merge their distinct authority into one generic lifecycle state.
Observation metrics are facts about those owners and never grants of release or durable success. Python-specific pull/wait ownership
retires with its actual consumers. Deferred optional PyO3 bindings do not require implementing another interpreter wait bridge in this
series.

Canonical SurrealDB governs durable study definitions, readiness, assignments and transitions through D;
C owns preparation/resource admission, actual computation, joins and physical release. A worker does not
mirror that durable workflow in another scheduler journal. Scientific calculation/progress observations remain;
bespoke build-stage, wait-lock and freshness observations retire with B/F. Do not serialize live task handles,
native workspaces or pointers to represent execution history.

A local run owns its execution according to the native API contract; a durable accepted study can outlive an observer. Stopping either
is an explicit operation policy, not a transport side effect. Shared preparation has producer/waiter ownership: one canceled waiter
cannot destroy a useful shared producer or transfer another caller's active permission. Result custody survives output retention,
delivery clones and reader close until its real allocation/owner lifetime ends.

Use `CancellationToken` for cooperative notification and `JoinSet`/`FuturesUnordered` where callers need task results. Consider
`TaskTracker` for a high-volume async population requiring destruction completion without accumulating task results. Its `close()`
permits `wait()` to complete only when the tracker is also empty; it does **not** stop registration. The application owner must seal new
admission atomically with its shutdown policy before declaring population drain complete. This contract is documented by [tokio-util
0.7.19](https://docs.rs/tokio-util/0.7.19/tokio_util/task/struct.TaskTracker.html). Dropping the tracker does not abort tasks, and
tracked async destruction does not replace native joins.

Compare tokio-graceful-shutdown 0.20.0 only for genuine long-lived subsystem roots that need common hierarchical stop/error routing.
`Toplevel`, `SubsystemBuilder`, `NestedSubsystem::join` and `ErrorAction` are candidates to consolidate such root coordination; no new
dependency is selected by this draft. [The library overview](../libraries/tokio-graceful-shutdown.md) identifies useful mechanisms and
limits; C01 verifies exact source/API contracts. Per-case/per-connection subsystem objects and replacement of native joins are outside
the comparison. Preserve ordinary native composition whenever the candidate adds more lifecycle machinery than it removes.

Core 3.4 and process-simulator 1.5 constrain lifecycle/scientific ownership; the coordinator's
[T01](58-rust-owned-system.md#t01)/[T02](58-rust-owned-system.md#t02) shared contract and ADR/formal-review work precede incompatible
API/governance changes. Host boot/PID/unit identity and physical containment belong to the host plan; durable transitions belong to the
control plan; selective result and delivery custody belong to the results plan.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [C01](#c01) Settle lifecycle composition | in progress | Runtime integrator | Exact-source root comparison decided and shared contracts accepted |
| [C02](#c02) Native operation ownership | in progress | Runtime/math owner | Observer/stop/join behavior implemented and targeted tests pass |
| [C03](#c03) Shared preparation and worker admission | planned | Runtime worker owner | Producer/waiter and nested admission behavior integrated |
| [C04](#c04) Service-root shutdown and host handoff | in progress | Runtime/host integrators | Admission sealed, failures retained and actual owned work drained |
| [C05](#c05) Publication and delivery settlement | in progress | Runtime/control/results integrators | Native completion, durable publication and output release remain distinct |
| [C06](#c06) Consumer cutover and qualification handoff | in progress | Runtime integrator | Replaced wrappers deleted and H scenarios ready |

<a id="c01"></a>
## C01 — Decide the smallest sufficient composition

Consume T01/T02's early shared-contract and consumer scope, then settle ownership for local operation, durable worker, shared
preparation, output reader and process-root populations. Contracts name admission authority, cancellation source, result/error owner and
actual completion witness; they need not create one new type per distinction. Compare existing tokens/tasks/native joins with
tokio-graceful-shutdown 0.20.0 for a named long-lived worker/service root only. Inspect exact APIs, child-start behavior,
subtree/shallow completion, panic/error routing, timeout/abort and propagation to ordinary tasks. Context7 resolution did not identify
that crate and the direct hosted 0.20 Toplevel page was unavailable during authoring; this is an unresolved candidate, not absence
proof.

Exit with either native primitives retained or a concrete root-only adoption and its removed coordination code. Source analysis can
decide suitability; an unresolved shutdown premise needs one small falsifying lifecycle experiment during execution. A timeout must
remain an incomplete drain outcome when blocking work remains, under either choice. Do not add a restart framework, global scheduler,
protocol task model or generic library-independent lifecycle abstraction. Coordinate the outcome with host/control/results consumers and
required decision approval before dependent implementation. Routine task-local organization remains implementation judgment.

<a id="c02"></a>
## C02 — Implement native operation ownership and truthful completion

After C01's contract is approved, integrate it into `PreparedOperation::start`, `RunHandle` and the math supervisor. Native callers can
observe, request stop and await original completion without Python. Distinguish observer detachment from an operation's selected
stop-on-owner-close behavior; retain ownership when a caller drops its response future. A started blocking call remains charged and
joined through callback failure, destruction and OS exit. Propagate original clocks, operation identity and typed causes; do not
recreate numerical iteration or change solver stop semantics. When one cleanup fails, attempt every independent owned drain and retain
individual failures before returning the original/combined error. Uncertain destruction cannot authorize replacement work. Targeted unit
tests exercise cancellation before start, during execution, delayed destruction, failed callback and partial native-team construction.
Delete superseded frontend wait orchestration only with migrated consumers; the underlying native join implementation remains
authoritative.

<a id="c03"></a>
## C03 — Compose shared preparation and worker admission correctly

Consume C02 and A's preparation/study contract. Keep the existing scoped Futures/native worker route, sharing immutable preparation only
while full identity and validity match. Producer lifetime, waiting caller lifetime, cached payload lifetime and active compute admission
remain distinct. Remove Python/source-currency work whose only consumer has retired; retain real input/environment compatibility checks
at the scientific/native admission boundary rather than on every read. For studies, preserve the selected workstation allocation and
useful concurrency. Bound queued and in-flight cases using existing policy while reserving required nested native populations before
dispatch. Cold multi-case batches, retained idle lanes and publication must not consume all parent capacity and then wait for
unavailable child capacity. Do not fix this by new CPU/RAM caps or global serial execution. Carry a real allowance to the original
execution owner instead of dropping authority at a wrapper boundary. Targeted tests cover canceled single/shared waiters, cold
concurrent cases, retained-to-fresh fallback and failed construction without leaked permits/populations. Remove replaced admission
wrappers and their obsolete tests once these consumers use the owner.

For persisted work, consume [D03](58d-canonical-control-and-connectivity.md#d03)'s bounded applicable
action, exact preparation premises and continuation from
[database-boundary O01](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O01).
A candidate is not an execution claim. Keep source loading, seed resolution, scientific preparation and
actual launch-resource admission outside transactions; carry the original launch owner/prepayment through
D's current premise recheck and execution claim. Return the prepared seed/revision/generation evidence
needed by the canonical owner rather than independently deciding durable eligibility again in the worker.
On a stale or cancelled premise, release the unused native admission without recording native start.
Continuation over an unproductive page must not be mistaken for exhaustive no-work.

The contract can be agreed alongside D03; a production worker needs the implemented progression/claim
slice and actual C admission owner, not merely an agreed interface. Keep native scheduling, immutable
preparation sharing and the physical effect loop. Remove worker-side paging/readiness/action sequencing
only where the complete canonical operation covers it. `CandidateGroup` suppresses duplicate local
preparation and is not a competing claim authority; retain it unless the replacement eliminates that
wasted-work role or supplies a deliberately selected alternative. A database allocation record cannot
replace actual CPU/memory admission, and this scope adds no parallel global resource ledger.

Targeted controls pair two workers discovering one point with exactly one current claim, stale prepared
evidence/cancellation before claim with no native dispatch, and retained-to-fresh preparation without
lost admission. D's logical authorization and C's physical ownership must remain separately observable.

<a id="c04"></a>
## C04 — Join service roots and hand physical ownership to the host owner

Consume C01's root decision, C03 worker admission and [B03](58b-bootstrap-and-host-control.md#b03)'s implemented identity/release slice.
Stop new root admission before canceling/draining existing work; `TaskTracker::close` alone is not that seal. When results matter,
actively drain `JoinSet`/Futures completions rather than accumulate them. If C01 adopts TGS, use it only around the named long-lived
roots; child tokens and ordinary tracked tasks handle frequent work, and original native joins still establish compute exit. Worker
service stop must not erase accepted durable intent; actual attempt policy decides completion or recoverable interruption. Process child
cleanup retains original boot/PID/start/unit attribution, borrowed-child constraints and physical release witness through the host
owner, even if the database is unavailable. Root deadline or forced task abort reports unfinished cleanup where appropriate. Targeted
tests race admission against shutdown, fail one sibling while another drains, stop idle and busy roots, and keep a blocked native child
owned after async timeout. Delete superseded root coordinators and process-release duplication only after host/runtime consumers migrate
together.

Canonical cancellation, lease expiry or reassignment records logical intent/authority; none proves that
an old native child has stopped. Deliver stop to the original token and drain the original owned work
even if the database is unavailable. A replacement worker must not accept the old worker's late output
under its new fence. Notifications may wake this root, but D's bounded canonical catch-up determines
applicable work; missed wakeups must not require a second local scheduler journal. Controls include
replacement while an old child is slow to stop and failure of database readback during native cleanup.

F02's disposable test resources consume a real owned fixture/worker capability with private constructors and explicit
borrowed-service constraints. Nextest groups are scheduling, not permissions. Settle owned resources after a
failed scientific assertion as well as success, retaining useful diagnostics outside disposable state; `Drop`
does not establish scientific success. Do not reproduce global test-name/effect tables, success-gated retention
or a resumable fixture terminal-composer. Keep runtime checks at external uncertainty: process death,
authentication, unknown ownership and actual interference with shared services. Minimal physical helpers must
remain below the DataFusion-bearing testkit closure so B's cold root can use them.

<a id="c05"></a>
## C05 — Preserve publication and result-delivery lifetimes

Consume C02 plus implemented [D03](58d-canonical-control-and-connectivity.md#d03) durable publication policy,
[D02](58d-canonical-control-and-connectivity.md#d02) original command reconciliation and
[E03](58e-retained-data-and-delivery.md#e03) protected-result delivery slices. A scientific terminal result is distinct from successful
durable publication, acknowledgement, response delivery and physical output release. Reconcile uncertain publication under original
identity; never rerun a completed scientific effect merely because a client lost its response. Cancellation of a reader or protocol
connection stops observation/delivery according to its policy, without canceling unrelated durable work or releasing buffers still
retained by another consumer. Reuse protected snapshots and charged allocation custody through clones/slices and transport drains. No
generic all-purpose settlement receipt is introduced. Optional RMCP, if selected elsewhere, uses these same semantics; endpoint close
cannot stand for domain/native drain. Targeted tests cover late delivery, reader close with retained output, publication failure after
native completion, lost acknowledgement and abandoned delivery while another reader remains live. Delete frontend-only settlement
files/bridges whose last consumer retires, keeping original owners.

A completed payload and a canonical accepted reference are distinct even when an encoded block is
small enough to remain inline. For sealed external payloads, retain the original publication identity
when file construction succeeds but canonical append/acknowledgement fails. Never rerun science to
repair an uncertain acknowledgement or treat a database complete record as proof of unfinished file
construction. Targeted settlement covers that seal/commit boundary, stale late publication and another
reader retaining the original result after publisher/connection close. E owns actual payload protection;
D owns accepted lineage and fences. These cases implement
[database-boundary O08](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O08).

Store construction establishes schema applicability; it cannot alone settle a later migration boundary.
Consume [D05](58d-canonical-control-and-connectivity.md#d05)'s explicit admission/version-fence and
in-flight-attempt policy before publication can cross that boundary. A closing store must not manufacture
completion or cause a second scientific execution. Distinguish
database transaction visibility, immutable payload publication and host executable replacement; share compatible
filesystem mechanics without imposing one universal publication protocol.

<a id="c06"></a>
## C06 — Cut over callers and hand off assembled lifecycle qualification

Move native APIs, workers, host roots, result readers and any selected adapter to C02–C05 contracts as each becomes usable. Delete
interpreter-specific wait/pull orchestration and obsolete lifecycle metrics/tests with their retired consumers; optional future bindings
must supply their own bounded contract and tests. Retain attribution, native callback, stack/admission and physical release failure
obligations even when they occurred through the removed interface. Coordinate source and test edits on shared runtime/host/results files
rather than treating this document as an isolated edit boundary.

Each unit integrates its native owner, actual callers, configuration and tests with old-path deletion.
Temporary incomplete development paths are recorded without acceptance credit; no second controller or Python
fallback is required between commits. Simplify ordinary unbounded host bookkeeping while retaining explicit
scientific demands and selected timing/reference exclusions; do not introduce a resident admission daemon.

## Verification and current checkpoint

**Proposed:** changed native owners use `just check-package <pkg>` and targeted `just unit-package <pkg> <filter>`; actual Arrow
correctness closures carry force-validation. Meaningful controls must distinguish early cancellation from real native exit, admission
closure from tracker emptiness, and native success from publication/delivery success. H owns the assembled concurrent-study
stop/reconcile, failure during cleanup and retained-output journeys, then scope-end integration/component/solver checks through
[H](58h-qualification-and-publication.md), `just hygiene` and manual checks. No performance claim follows from fewer wrappers. All
production implementation, probes, tests and builds are **not_run** in this planning task. Zero remains the execution failure baseline;
no prior interrupted run supplies new acceptance credit. C completion requires migrated consumers, immediate replacement deletions and
the coordinator's H evidence, not merely a successful cancellation request.

## Selected host handoff checkpoint — 2026-10-28

The database-free B02/B03 wave implements C04's foreground process/physical fixture handoff.
Native performing capabilities keep private construction, original intent/bus reference/kernel
identity, explicit settlement and combined failures; borrowed capabilities cannot stop units.
Caller-only/workload scopes and the same-image exec gate replace Python foreground coordination
for selected ordinary commands. Cancellation preserves one startup and one drain deadline and
attempts all independent cleanup. SIGKILL release is enforced by manager dependencies, including
escaped/TERM-ignoring descendants; no Python or database owner must remain alive for cleanup.
Native tests establish their own private parent application scopes so their cleanup cannot
acquire authority over an enclosing test runner. Parent-application stop and witnessed gated
startup death now have real-manager controls through the normal command boundary.

**Tested:** the standard admitted foreground suite passes 10 tests and manager suite passes 9 tests
against zero failures, including one launch reference across repeated reconciliation. The
request Debug redaction control also passes. [58b](58b-bootstrap-and-host-control.md#selected-wave-verification-and-outcome--2026-10-28)
owns final assembled evidence and remaining fault cases. These host/process controls do not
qualify broader runtime blocking-native cancellation/destruction, durable worker roots,
scientific fixture orchestration or C04/C01–C06 package completion. No canonical service was
stopped/restarted to supply these fixtures. Existing data and unknown services were preserved.

The follow-on compiler/KLU wave exercises the same host containment with a real interrupted
CMake construction: controller SIGTERM returns143 and independent PID/cgroup readback confirms
physical drain before reporting settlement. No generation is selected on that interruption.
The database-free Runtime selection also passes
`cancelled_serial_preparation_keeps_loan_and_storage_until_child_drain`, retaining its original
loan/storage assertions. The two selected KINSOL scientific tests now share one Runtime between
source admission and execution services; their original binding guard remains intact.
[58b's compiler/KLU checkpoint](58b-bootstrap-and-host-control.md#explicit-compilerklu-consumer-checkpoint--2026-10-28)
owns the commands and bounded results. These controls extend the selected C04 handoff evidence;
they do not complete C01–C06, qualify arbitrary solver cancellation, or select a new Runtime
root/lifecycle framework.

## Local operation ownership checkpoint — 2026-10-28

The selected local C01/C02 slice retains the existing supervisor, cancellation lease and
native joins. Dropping a wait future or passive observer does not stop a still-owned run;
explicit cancel and final immediate-owner departure request stop while original supervisors
retain physical cleanup. Logical caller close seals new admission, not already admitted work.
No new lifecycle framework or long-lived worker policy is selected. The public local caller
uses existing lane shutdown for final fixture settlement, including on assertion failure.
[58a's local checkpoint](58a-native-product-and-scientific-workflows.md#native-local-operation-checkpoint--2026-10-28)
owns the composed acceptance and limits; this does not close the broader C packages.


## Native service shutdown handoff — 2026-10-29

C04's additional host slice now owns ordinary workload/service forests and the isolated canonical
fixture's terminal closure, original ingress/connection join, failure-retaining parallel shutdown
and explicit disposable-state retirement. Actual process-stop and intentional-body-panic controls
pass with original units independently absent afterward. [58d's checkpoint](58d-canonical-control-and-connectivity.md#native-host-dependency-execution-checkpoint--2026-10-29)
owns the final command and live limits; [58b](58b-bootstrap-and-host-control.md#current-execution-handoff--2026-10-29)
owns host acceptance and next consumer migration. C04 remains in progress; this does not qualify
arbitrary blocking solver cancellation, shared-store shutdown or durable worker-root policy.

## Native worker and publication checkpoint — 2026-10-29

The worker consumes one bounded canonical progression operation and an opaque preparation
handoff. Native source/seed/resource preparation remains outside the claim transaction;
canonical recheck grants the actual attempt. The former client paging/readiness/action/
finalization choreography and repeated authoritative policy interpretation are deleted.
CandidateGroup retains local preparation suppression, while the shared mathematical owner
retains producer/waiter admission and original native joins.

C04 now registers SIGINT/SIGTERM before startup and keeps one process-root stop listener
through readiness/serving. Native session close and mathematical lane shutdown return typed
settlement failures; callers preserve original failures while draining every independent owner.
OS thread joins survive cancellation of a queued blocking joiner. Physical lane population is
released only after actual native/foreign destruction. The native service drains independent
connections and ingresses concurrently before stopping their required server, under the same
original deadline and with failures from both stages retained. A real multi-connection journey
exposed the former stop-versus-release RPC race; the corrected cancellation journey passes.

**Tested:** `scripts/pse-env --native=compiler,klu -- scripts/cargo-bin/cargo-pse-correctness
nextest run --no-fail-fast -p pse-runtime --locked --test native_study --features
native-study-tests,solver-kinsol`, with private service/module selectors,
passes two controls against zero failures (run `9271e114-b1bf-42b8-b635-8caacee6915b`). These cover
two competing workers, actual two-point KINSOL continuation, independent variable/residual
answers, recorded source/point/attempt lineage, bounded connection-only browsing, protected
reader close refusal, publisher close/replacement and cancellation before any claim/native entry.
The first numerical assertion used a tighter answer tolerance than its requested accuracy;
the fixture now supplies explicit original variable/equation requirements, preserving the
independent expected answer. Both successful private fixtures are physically drained/disposed;
earlier failed fixture diagnostics remain retained.

**Tested:** `just test-package pse-operations --features
canonical-tests,native-service,canonical-maintenance,test-support --lib --test native_service
--filter-expr 'test(native_publication_controls::) | test(http_control::)'` through
`scripts/pse-env`, with explicit private module/helper selections, passes four controls against
zero failures, 517 excluded, and no warnings (run `dd968474-b158-43b8-84db-b3d2c1b39a3e`). The three
publication controls cover inline/sealed construction before acceptance, lost append/terminal
ACK with failed readback and exact original replay, stale late publication under a replacement
fence, and independent output custody through abandoned delivery and publisher/reader close.
The last protected consumer gates physical collection. Partial transport fixtures confer no
scientific success; the real KINSOL journey above supplies bounded scientific evidence.

C01–C06 remain open for their remaining integration/retirement and assembled qualification.
Active-native cancellation/crash replacement, migration-crossing publication, lazy/cold pool
population and other scientific classes are not qualified by the two bounded native cases.
The still-used Python/PyO3 facade and isolated reference exchange retain G03/F04 ownership.
No performance or arbitrary blocking-solver cancellation bound is claimed.

The later admitted-native cancellation control holds a real native session entry after its
stop flag is observed. Worker completion remains pending until the barrier is released;
the original owners then join, and an independent replacement finalizes cancellation without
a second attempt/native entry or usable-science claim. The unwind guard releases the barrier
before a dependent join can run, preserving failure-path cleanup. **Tested:** the same
force-validated `native_study` target with `-E 'test(active_cancellation)'` passes one control
against zero failures, two excluded, no warnings (run `85a97867-f496-4102-b677-41584d6990e1`).
This closes the bounded admitted-entry cancellation gap above, while actual worker crash,
migration-crossing publication, broader populations and arbitrary blocking foreign solvers
retain their separate acceptance obligations.
