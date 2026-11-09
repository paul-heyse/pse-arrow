---
title: "Plan66c: Preparation, execution and service lifecycles"
status: in-progress
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/66-integrated-simulation-target.md
doc_topics: [design, numerical-execution, preparation, validation]
review_sources: [docs/plan_improvement_reviews/plan64-65-integrated-target-and-library-leverage-2026-11-06.md, docs/plan_improvement_reviews/plan64-65-contracts-and-runtime-analysis-2026-11-06.md]
---

# Preparation, execution and service lifecycles

## Remaining target and foundations

Plan66 is the sole go-forward target. This companion owns remaining preparation validity,
native/service handoff, prepared-worker and physical-settlement changes. It actions I05/I06,
CR05 and the lifecycle parts of CR02/CR04. Scientific operation semantics belong to
[A](66a-scientific-operations-and-product-semantics.md), mathematics and derivative premises to
[D](66d-mathematical-representation-and-numerical-execution.md), generated correspondence to
[B](66b-contracts-generation-and-interpretation.md), custody to
[E](66e-protected-data-and-result-delivery.md), and actual clients/evidence to
[F](66f-developer-operations-clients-and-evidence.md). The
[coordinator](66-integrated-simulation-target.md) owns combined decisions and
[L](66l-required-population-and-assembled-acceptance.md) assembled acceptance.

**Interface-checked:** main `66341514d` already contains reentrant generic native supervision,
service-resident resumable drain, unique surrogate retirement actions, result-bearing native
finish paths, original identity/context checks, authenticated local binary-gRPC ingress and
connection error aggregation. Native bridge snapshotting outside disposable worktrees, exact
fixture prerequisite assessment, complete passive source recipes and a provider-free compiled
f64 Value/First live-child path also exist. These mechanisms are prerequisites, not new framework
packages. Earlier focused tests do not qualify every provider, transport or shutdown population.

The historical “compiler cache bridge context changed” refusal came from a stale standalone
compiler receipt. C01 records the explicit refresh and confirmed root, nested and sibling
receiving. A new preparation checks its actual inputs and receipt under the same strict identity.

## Preparation validity and execution placement

Preparation is explicit and attributable to consumed inputs. Separate immutable caller-selected
configuration, installed tool/provider selection, generated dependent artifacts and runtime
allocation. Normalize external input once; do not reinterpret installed compiler/bridge values
as a fresh caller override on the next command. Verify the actual selected executable, bridge
content and KLU/provider dependencies in their owning order. A changed relevant mode, toolchain,
provider, artifact or allocation must refuse or invalidate dependent preparation; an unrelated
capability or disposable checkout path must not invalidate it spuriously.

Use `crates/pse-host/src/native.rs` and `native/providers.rs` as the preparation owners, including
their existing retained bridge identities. If preparation refuses again, compare the exact
selected compiler, environment, normalized inputs and receipt. The existing explicit preparation
route selects the matching immutable provider and bridge. Its controls cover later ordinary use
from sibling worktrees, capability subsets and removal of the preparing checkout. Help,
discovery, listing, attachment and passive reads never prepare implicitly.

Preserve the maintainer-selected unbounded ordinary host mode, native profiles, incremental
compilation and shared sccache. Do not introduce smaller CPU partitions, cache-off acceptance,
shared target directories or new host caps. Scientific pool/exclusion and explicit finite
allocation retain their distinct contracts. A preparation receipt is not a native job permit;
allocation ownership follows the actual active work.

Reuse scientific preparation only for unchanged consumed premises. Source/model/physical context,
datum, structure, derivative order/support, provider identity and specialization determine the
dependent products; compatible value-only overlays may reuse them. D owns numerical realization
and invalidation; C integrates that lifetime with preparation and active execution. Do not add a
second generic cache alongside current Salsa/native preparation or collapse mutable sessions into
immutable artifacts.

## Native operations, effects and service lifetime

Transport decodes, validates and authenticates; native handlers decide contextual membership,
scientific support and effects. Retain original operation/request/run/attempt/step/point identities,
prepared generation and governing source. Registration and effects should follow the actual typed
service composition and B's source-owned correspondence. A universal dynamic-message operation
interpreter would add a competing meaning owner.

Keep selected local binary gRPC over HTTP/2 Unix sockets unless a concrete consumed requirement
changes that support. Initial and reconnected peers require kernel credentials and original-image
custody; a socket path, header PID or descriptor is not authentication. Preserve early method,
protocol, codec and HTTP-version refusal before body polling or collective allocation. Apply whole
route limit profiles: Connectrpc0.9.1 `Router::with_route_limits` replaces the profile, while
body/decompressed-message/element limits remain distinct. Health/reflection expose only the
authorized prepared service world. New hosting alternatives must actually eliminate glue while
expressing these obligations; do not reopen a working released Hyper composition merely to
substitute a library name.

Submission has a native effect boundary. Definite non-submission, accepted execution, abandoned
method receiver and uncertain network reply are different outcomes. A lost ACK never causes
automatic replay. Duplicate original request identity can resolve to its original operation;
changed content under that identity refuses. Progress is observational: dropping a stream does
not cancel native work or publish a terminal verdict. Explicit cancellation targets the original
operation; acknowledgement is not physical settlement. Reconnect reads original status/products
under current custody and E's protected-result rules.

Use Tokio1.53.3 `mpsc::Sender::reserve_owned` followed by `OwnedPermit::send` for the selected
queue reservation/handoff. Reserve before committing control-map state; the handoff has no
cancellable `send(...).await` gap. Define disposition for cancellation before reservation,
after reservation, after ownership transfer and after native completion. A semaphore's owned
permit moves into the actual native job; observer/future disappearance cannot release it while
native work continues. Cancellation control must remain reachable while the mutable numerical
lane is occupied. `FuturesUnordered` can poll independent control and numerical transports;
its local in-flight shape is not a host concurrency cap.

Settlement follows actual owners: stop admission, request the appropriate cancellation/shutdown,
settle native calls, reap child processes, join I/O/tasks and retain every independent error.
Connection graceful-shutdown requests must still be polled to completion. Current engineering
`JoinSet::join_next` preserves panic/inner connection failures; retain that behavior. In
`scientific_worker/transport.rs`, the `OwnedExecutor` vector/batched-join mechanism is a candidate
for simplification only if descendants, error collection and end-of-spawning remain correct.
Tokio-util0.7.19 `TaskTracker::close` does not prevent spawning, `wait` does not return task errors,
and neither reaps an OS child. `CancellationToken` is a signal, and a dropped tokio-rayon2.1.0 or
Rayon1.12.0 future does not establish native cancellation. Use tracker composition only where
these missing obligations have explicit existing owners; retain JoinSet where simpler. See
[CR05](../plan_improvement_reviews/plan64-65-contracts-and-runtime-analysis-2026-11-06.md#CR05).

## Prepared workers, receiving and explicit finish

Keep three permissions separate: live execution of an already admitted body; independent receiving
of a complete passive recipe under original qualified membership; and historical observation.
The current provider-free Value/First child restores an interpreted f64 evaluator. Any retained
placement for that work must use actual prepared execution without reintroducing source replay;
the [wire decision](#scientific-wire-retirement) checks whether it still needs a separate lane.
Complete other selected operation/context populations
only with their native owner and D's explicit support. A successful recipe decode, body allocation
or historical proof record does not authorize arbitrary provider/session restoration. Preserve
canonical receiving's existing interpretation until its actual replacement and receiving controls
qualify together; retire a format only with its last reader.

Prepared worker crossings carry exact input bits, physical/datum/layout identity, derivative demand,
scope and generation. Immutable descriptors/layouts prepare once; callback evaluation does not
re-encode a complete model or numerical arrays. Preserve native single mutable-lane semantics,
foreign-library thread/TLS requirements and CPU ownership. Scalar fill loops do not count as a
native batch. For each selected genuine multi-point operation, use a coherent native batch with
ordered point identity, state-reset semantics and independent point outcomes; keep a scalar route
where the actual provider has no batch contract.

`Stop(handle)` cancels only that handle's work. `Close(handle)` retires its prepared state after
its invocations settle. Session shutdown may close its own handles; process shutdown drains all
owned sessions. Abandoned Prepare must retire the actual newly created handle without destroying
unrelated live handles. Explicit result-bearing finish reports stop, actor retirement, process
reap and I/O errors independently, retaining banks/allocation until physical settlement. Drop is
abandonment containment; it must not replace the normal production finish result. The delivered
generic retirement hooks are the foundation; extend their real use rather than add another
supervisor or per-service detached cleanup task.

## Packages

### Bounded placement decisions — November6

**Interface-checked decision:** retain explicit engineering-service preparation/launch for
the current finite independent-client delivery. Multiple clients and reconnects consume the
actual authenticated live service; they do not require login activation or an installed
engineering-service profile. `pse-host::manager::InstalledServiceSpec` and its inspect/start/
stop operations, currently consumed by `telemetry_service`, are reusable mechanisms, but
their existence does not qualify engineering update/readiness/drain. Reopen the installed
profile when a selected client requires service continuity across the current launch owner's
lifetime; compare image/update identity, credentials, activation result and physical drain
before selecting it. C02 still delivers its service/client population and acceptance.

**Interface-checked decision:** retain the current process-isolated provider boundary for
the selected worker/native-child consumers. No independently distributed in-process provider
requiring a native plugin ABI is selected in the present delivery population. Proto exchanges
data and supplies no allocator, callback, thread/panic or unload contract. Reopen an ABI binding
for a named distribution consumer when its complete library/interface and lifetime comparison
justifies replacing process isolation. This records the bounded choice; it neither rejects
dynamic libraries nor closes C04's distinct canonical reconstruction and receiving work.

**Interface-checked decision:** retain scientific transport's current `OwnedExecutor` batch
joins for the selected worker connection. The actual I/O owner closes its request/client and
joins the public Hyper driver before `OwnedExecutor::finish` repeatedly takes and awaits all
executor tasks, including descendants registered by a preceding task. Every `JoinError` is
retained; authenticated peer EOF and native/process settlement remain separate owners. See
[`transport.rs`](../../crates/pse-runtime/src/scientific_worker/transport.rs) and CR05's exact
TaskTracker characterization. Replacing this loop with tracker `close`/`wait` would additionally
need equivalent typed panic/error capture and an end-of-spawning argument, while retaining the
same socket/native joins. No observed completed-task accumulation or separate shutdown subtree
currently justifies that added mechanism. Reopen for a selected long-lived connection with
demonstrated retained-handle growth or a concrete independent-subtree shutdown requirement.
This closes the conditional library-substitution choice, not C03/C05's remaining receiving
and physical-settlement population; no new shutdown test or memory benefit is claimed.

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| C01 | complete | Existing retained bridge/provider identities and explicit native preparation | Stale standalone compiler receipt explicitly refreshed with the existing immutable-cache implementation. Root compiler/solver selection and nested reentry succeed; the separate results checkout passes the same selection and engineering generation. Nine targeted native identity/cache controls pass with complete capture/settlement, including cross-checkout survival and changed-context refusal. This closes the reproduced preparation gap without relaxed checks or a new cache path; broader C acceptance remains separate. |
| C02 | in-progress | Existing native operation/supervision foundation; B's needed family correspondence; A's governing scope | Common RPC correspondence and the single authored-case `SubmitRun` are in use. The fixed live mixed Study and its endpoint and edit/run/reconnect phases passed. Remaining worker, reference and larger declared-case consumers stay open. |
| C03 | in-progress | Existing queue/retirement owners; C02 selected operations; D's provider support | Actual-child controls retain the original Prepare refusal or Evaluate deadline and the independent failed native finish after the generated method departs. Exact Close/Stop ownership is retained. Complete remaining selected batch/session paths; combine crossings only for an actual isolated placement. |
| C04 | in-progress | Complete passive recipe and admitted-body foundations; A/D operation-specific permission; D02's bounded symbolic restore or replay decision | Remaining selected live/receiving/reference contexts use exact native permission and original memberships. The unconsumed private root lane is retired. Production receiving still uses JSON v3; the replacement needs independent stage/receipt correspondence and cold canonical qualification. |
| C05 | in-progress | Working C02–C04 producer/consumer slices and B04 where representation changes | `ObserveStudy` joins admitted open/page work under the service and retains the original ingress grant through reply or drop. The real `ObserveRun` UDS trailer and Study reset during a live canonical capture passed. The keeper acknowledged drain with both Preserve on failed Routes and Dispose on the successful fixed sequence. Remaining service/client populations stay open. |

Typed engineering `AdmitSource` now takes an explicit registration set authorized by
the service startup. The native owner rejects unknown/duplicate choices before retaining
the request, prepares selected registrations against the actual admitted physical
context under one managed native job and its original cancellation and clock, and supplies
them to the existing document loader. The job admits the selected routes' construction
allowance in addition to its existing stack and foreign reservations. CoolProp image
verification uses fixed scratch rather than reading a full library image into memory on the
service task. Provider-free intake uses
the same route with an empty set. The original operation owns its registration set
across source inspection and exact repeated admission. The new targeted control uses
the authored Water closure. **Tested, selected:** the force-validated default-profile
runtime control passed over an authenticated Unix HTTP/2 gRPC connection with
complete settlement and zero build warnings (`01a1bf8c-3aa3-741b-ac71-14785594a48f`).
It checked original bound provider configurations, unchanged reordered replay,
unknown/duplicate and changed-same-identity refusal, and the actual smaller
admission pool. Separate launched-client and scientific-product receiving remain
with [F05](66f-developer-operations-clients-and-evidence.md#f05--deliver-complete-independent-clients)
and [66g](66g-properties-equipment-and-steady-workflows.md).

### C01 — Make preparation representations compose

Reproduce the retained-bridge context refusal using exact admitted command/environment/profile
and trace the original caller selection through normalization, installed identity and later
comparison. Correct the structural mismatch, then exercise successful unchanged ordinary use,
changed bridge bytes/compiler/mode, omitted versus explicit equivalent selections, provider subset
and preparing-worktree removal. Preserve strict compiler/KLU/provider order and independently
reported missing capabilities. Delete superseded path-based or duplicate normalization logic only
when every caller has moved. This is a bounded prerequisite to affected generation/qualification,
not a reason to rebuild all preparation for every plan package.

### C02 — Complete actual local operation/service consumers

The local service registers the versioned explicit-point and source-expansion Study RPCs from
[B01/B03](66b-contracts-generation-and-interpretation.md#shared-index-bindings-and-compiled-contract-ownership).
Both dispatch the [A01 native Study operation](66a-scientific-operations-and-product-semantics.md#a01--study-input-capabilities)
under its existing authentication, aggregate ingress admission and exact source context.
Compare whole-case submission for run-only clients under the
[wire retirement decision](#scientific-wire-retirement); distinct Study input capabilities stay separate.
F05's clients now use the selected routes, and the combined procedure is removed.
The [cutover evidence](66-integrated-simulation-target.md#shared-contract-cutover--november-8)
separates callable-route refusal from descriptor discovery and checks whether reconnect
and restore preserve original operation, variant and interpretation without restarting effects.

The common engineering RPC delivery now consumes source-generated ingress, edit, identifier,
status/progress and outgoing mappings. Native source-parent, membership, provider, effect and
authentication checks remain with their existing owners. Main's selected receiving passed after
the separate checkout-environment activation defect was corrected; B records its 38-control
bounded-response/result scope. This is not the remaining independent-client population.

Run and Study issuance now publish the original coordinates with the operation fingerprint
under one inventory insertion. Status and cancellation can observe the original pending
identity immediately. Exact replay reuses the same operation and changed input refuses. The
bounded DescribeContract handshake compares the original borrowed Buffa view directly.

Migrate all selected engineering contract/workbench/prepared-study and simulation/fit/horizon
submission/status/cancel/progress/read paths, plus their preparation, worker and reference service
routes. Coordinate F's generated Rust client in the same slices. Compare direct/native and remote
typed outcomes; preserve errors' original structured diagnostic class. Cover initial/reconnected
wrong image, stale context, unauthorized reflection, empty/present-default requests, changed
duplicate request and lost submit ACK. Delete replaced mechanical adapters, hand-maintained
method/effect/health mirrors and duplicate polling translations as each family moves. Native
checked edit and parent/membership guards remain.

The selected aqueous receiving case exposed a source-publication failure before
native science. The retained `01a1bd26-6f64-7207-8da5-08ef8c807c89` trace locates
the dispatched failure in `stage_small_edits`. A read of the original operation
before fixture teardown found no acknowledgement. The edit remains unproved.
The staging change batches eight bounded edits, uses the existing exact version
indexes for closure counts and attempts receipt readback on uncertain staging
failure within the original operation clock. The full aqueous journey remains a
separate acceptance case. [Source staging owner](../../crates/pse-operations/src/canonical_staging.rs)

The exact maximal staging request envelope, concurrent same-request replay and
lost close acknowledgement controls passed in `01a1bd44-67f1-7536-bb01-a0420dfe44d1`.
That invocation included the atomic-capture selection and its independent fixture
and scale failures. [66e capture evidence](66e-protected-data-and-result-delivery.md#migration-recovery-and-acceptance)

### C03 — Close performing owners and worker sessions

Consume [B03's native-derived worker contract](66b-contracts-generation-and-interpretation.md#b03--finish-finite-source-to-consumer-migrations).
Admit immutable demand before provider creation and carry that checked request into
binding. Still check fresh provider capability, context, generation and attempt at the
binding and invocation boundaries. Replace both managed images in the private protocol
cutover and refuse mismatched revisions, stale handles and unsupported derivative demand.
Preserve original deadline, native linearization lifetime and independent child and I/O
settlement. The persistent provider remains the performing owner for selected isolated work;
the [private root process was retired separately](#scientific-wire-retirement).
[Provider session](../../crates/pse-runtime/src/scientific_worker/session.rs)

The authenticated proxy sends preparations serially and mints each generation through
`Uuid::now_v7`, which guarantees creation order within one process. The child checks
the session before recording the highest admitted generation. This single value survives
retirement and prevents a replay from rebuilding the same handle.
[UUID 1.28 generation contract](https://docs.rs/uuid/1.28.0/uuid/struct.Uuid.html#method.now_v7)

Consume D01/D02's admitted route, witness and exact artifact/callback demands in fitting
and shooting. Their window, residual and continuation decisions stay local. Retire
their duplicate common admission and witness reservation composition as callers move.
[Numerical preparation contract](66d-mathematical-representation-and-numerical-execution.md#d01d02--shared-admission-and-exact-native-demand)

Move common deadline, cancellation, claim preflight and activity into the existing
`RunSupervisor` launch contract. Preserve original operation, attempt and physical
owner through registration, admission and settlement. Simulation, fitting and Study
supply their actual dispatch and typed result. On refusal, cancel and await the started
handle, retaining any independent finish failure alongside the initiating refusal.
Keep registered ownership until physical drain is established. Remove the repeated
launch policy with its callers and expose the original joined causes to E06.
[Run supervisor](../../crates/pse-runtime/src/workflow/run.rs)

Actual authenticated HTTP/2 receiving now drops an in-flight generated Prepare method on the
original connection, observes its completion receiver close before the late Prepared send,
retires only that orphan, and continues using both independent prepared handles. The native
handle/admission survives until actual retirement. The selected main Water/costing/worker run
passes this control and four related departure/pending-Close controls, with zero failures/build
warnings and complete capture/settlement. This is the selected preparation-delivery phase,
not all cancellation, batch or session acceptance.

A separate three-generation retirement-failure control proves that remaining owners are all
attempted, but exposed an actual composition defect: the native lane's original typed failure
aggregate stayed child-local while the parent received Unavailable and abrupt exit125. Main
retains that diagnostic control. The active correction joins the original native owner before
distinguishing completed from unsettled work, closes admission, and exposes one bounded passive
failure observation without consuming the original result. Tokio's completion wake is not join
proof; its watch channel only distributes the once-derived bytes. Connectrpc's Bytes-backed
`PreEncoded` response retains one encoding across readers. A further actual immediate parent
failure control governs the existing bounded process-finalization allowance; no remote fact
can grant native authority or replace local I/O join, reap or admission release. Main contains
the joined original-cause repair (`0733f5fd0`) and its pre-copy bounded diagnostic follow-up
(`9d80d9836`). Their controls cover the selected failed-orphan path and immediate parent finish.

Process-wide Shutdown retains original native retirement failures in the joining owner even
when the generated method receiver departs. The reply observes the bounded failure and can be
lost independently. The native join, actual child exit, I/O drain and admission release retain
their separate outcomes. Stop and Close continue to target the original handle.

Start from the delivered generic admitted-job and unique-surrogate retirement mechanisms.
Complete the selected preparation abandonment, synchronous fill, provider banks, shared-surrogate,
multi-session and explicit-close caller population. Exercise receiver loss at each handoff phase,
Stop during fill, Close with pending work, independent surviving handle, shared child retirement,
native error plus retirement error, child exit and uncertain reap. Preserve all independent
failures and keep resource ownership until physical completion. Add genuine batch support only
with existing native provider semantics and a real consumer; coordinate exact point-demand and
outcome shape with A/D/B. Remove scalar RPC loops only when that replacement is complete.

**Tested, targeted in the owning checkout:** `scripts/pse test pse-runtime --lib --features
property-coolprop` with prepared `compiler,klu,solver,isolation` passed four selected controls
(actual pending Close, existing action/survivor transport, active native batch Stop and independent
native-close failures), zero failures/warnings and complete capture/settlement. After hardening
the exact queued-generation witness, the changed Close control passed 1/1 independently. The
integrated test-only witness observes actual committed queue handoff; production Close still drains
rather than signals Stop, and explicit finish still owns reap, endpoint removal, I/O join and
admission release. This is a controlled native provider in a real child, not foreign-library TLS,
new scientific accuracy or full C03/C05 qualification. Losing a local observer is distinct from
abandoning the server's Prepare method; that real RPC-departure contract remains separate work.

### C04 — Finish selected reconstruction and execution contexts

Use [D02's bounded reconstruction decision](66d-mathematical-representation-and-numerical-execution.md#canonical-reconstruction-activation)
before changing canonical receiving. D02 first tests a producer-sealed guarded body, then
bounded recipe replay if the first route fails proof custody or execution fit. C04 owns
qualification of the original producer, request, interpretation and membership; retain the
outer backing, original cancellation/deadline and receiving owner until the work settles.
The supplier pre-allocation contract must cover the actual restore or replay, not just decoder
scratch. Use a cold receiving control after producer retirement, with an actual downstream
symbolic consumer, exact correspondence and refusal without partial publication. A corrupt
qualified hit is an error, not permission to try live source admission or another decoder.
Keep JSON v3 as the sole current production route until one qualified replacement moves its
producers and consumers, then remove the old interpretation, callers and old-wire fixtures
in the same change. B03 adopts the selected projection only if it changes. The retired private
root process's Value/First transfer only restored an interpreted numerical evaluator. It
could not serve symbolic `AdmittedBody` consumers. No change to unbounded host policy follows.

Complete original qualified recipe membership across the canonical receiving population and the
actual prepared worker/reference consumers. Extend live prepared-body execution where D supplies
the matching supported representation; do not create generic restoration from a descriptor.
Refuse changed source/layout/datum/provider identity, unavailable original rule or unsupported
arithmetic/derivative path before execution. Verify original physics and demanded derivatives at
the consuming scientific companion; transport round trips alone cannot establish them. Remove
source-recompile, duplicate receiving or obsolete JSON paths only after exact replacement readers
and native receiving acceptance exist. Historical readable records can retain unavailable use.

<a id="scientific-wire-retirement"></a>
#### C02/C03/C04 — Keep only crossings needed by a complete operation

For [Symbolica review I07](../plan_improvement_reviews/plan66-symbolica-leverage-2026-11-09.md#I07),
the deployment and Rust caller inventory found no supported private root caller or placement
obligation. [ADR-0259](../adr/0259-retire-private-root-lane.md) retires its launcher, root wire
and mechanics-only fixtures. Native-session lifecycle and supplier transfer controls stay with
their performing owners.

The selected engineering contract uses one `SubmitRun` with the original source operation,
exact parent revision, case, intent and issued run/attempt. Its retained task prepares and
executes the case. Status reports the immutable native preparation when available, including
after canonical reconnect. No supported caller needed standalone `PrepareCase`. B03 and F05
own the generated and client cutover; their acceptance retains changed-repeat, stale-parent,
lost-ACK, cancellation, settlement and protected readback.

The [scientific worker](../../crates/pse-runtime/src/scientific_worker/proxy.rs) is conditional
on actual isolated placement; ordinary engineering's in-process provider route does not need
it. A selected worker may perform a complete independent cohort or solve locally with pure
neighboring arithmetic to remove crossings. Compare with existing `Evaluate`/`EvaluateBatch`,
not just a different envelope around the same scalar calls. Preserve explicit batch eligibility,
ordered point IDs, per-point refusal, stale sessions, attempt identity, original stop and finish.
Verify original-connection, abandoned Prepare and failed retirement behavior before retiring any
crossing. Study input selection, admission, observation/cancel and protected reads retain their
distinct authorities and lifetimes.

### C05 — Verify actual shutdown and transport composition

Exercise a refused admission after dispatch with an independent finish error.
Require both causes in the original outcome and actual native join before release.
Distinguish passive wait or disconnect, cancellation request, physical exit and settled
I/O. For a worker that accepted preparation before losing its acknowledgement, observe
or retire the original attempt without submitting it again.

The canonical observer control now holds a real protected revision on the original reader
connection, cancels the pending observation and verifies the original grant was released
before returning. Its fixture independently observes the release and the reader alias refuses
fresh activity. This covers cancellation after acquisition. Connect and open still have
their separately bounded native acquisition lifetime.

The authenticated engineering HTTP/2 control abandons a partial unary upload, observes the
original ingress reservation return, and reads a valid typed response on the same connection.
It then joins the actual server and client, retaining the original socket receive half through
the final Hyper shutdown writes. The server's typed settlement remains part of the assertion.

The [Study stream control](../../crates/pse-runtime/src/host_operations/engineering/session/canonical_study_tests.rs)
holds the real canonical capture before page construction. Dropping the live response on the
[locked](../../Cargo.lock) [Hyper](../libraries/cli/hyper.md) 1.11.1 and h2 0.4.19 client resets
that stream while the same authenticated peer still serves `DescribeContract`. The original
request grant stays with the admitted task after
the transport drains. Shutdown waits for that task; joining releases the capture and the grant
without cancelling the original queued Study. The exact default-profile, force-validated
`scripts/pse test pse-runtime --lib` selection passed this control and the lost-acknowledgement
control, two of two with complete capture and settlement (`01a1c18b-2638-7358-9d7b-7fe783e4eda7`).

Original receiving exposed a native launcher gap: a caller without exported placement values
did not activate the checkout's explicit unbounded selection, so managed test launch chose the
default finite functional scope and rustc was killed at its 40 GiB limit. Test/check launch now
uses the existing environment activation owner, preserving caller precedence and explicit
finite class/memory choices. Main's actual receiving confirms ordinary placement. This is
separate from C01's refreshed immutable compiler receipt and changes no host capacity/cache policy.

A later service receiving was interrupted before tests when global OOM killed the user D-Bus
service and a compiler process. Its original foreground exited143, required terminal was absent,
and settlement remains incomplete. This is not the earlier finite40GiB launcher defect. D-Bus
recovered automatically; no manager restart, host limit, compiler width, cache or incremental
setting was changed. Other original invocations completed their compiler/test children but
reported independent manager cleanup failure; their child evidence remains distinct from clean
command acceptance. H owns its scientific child and outstanding native receiving. Fresh combined
receiving must use current source and a live original manager; an absent old scope does not
rewrite a failed historical receipt.

The one-shot observer now owns one process-root cancellation source and the existing
[`ProcessInterrupt`](../../crates/pse-runtime/src/bin/support/signals.rs), carries its token
through query, analysis, capture and canonical read, and explicitly finishes the listener after
the performing work settles. The public reader library installs no process signal handler.
[`observe.rs`](../../crates/pse-runtime/src/bin/project/observe.rs) joins its synchronous encoder
and uses the shared native
[`observer_io`](../../crates/pse-host/src/observer_io.rs) descriptor owner for input/output;
the old blocking serve path and duplicate output-writer module are deleted. Regular-file
duplication preserves shared offsets/append semantics. Pipe/terminal reopening avoids changing
the caller's descriptor flags; sockets use Rustix1.1.5 per-call `DONTWAIT` and output `NOSIGNAL`.
Tokio `AsyncFd` owns readiness and cancellation without a detached blocking pipe operation.

Use Tokio-util0.7.19 `CancellationToken::until_cancelled` for cancellable queries and DataFusion's
stream/drop contract, retaining explicit collector drain separately. Canonical acquisition/poll
and Attempt reads observe the caller token after store acquisition; services and original
connections still drain outside the cancellation race. Bare connection/open retains its existing
bounded native clock and partial-acquisition owner: prompt cancellation during those phases is
not established. Original native failures survive serialization/output/late-stop and independent
cleanup failures; passive diagnostic projections are never decoded back into native authority.

**Tested, targeted:** `scripts/pse test pse-host --lib` selected host descriptor controls passed 4/4.
The `scripts/pse test` selection over `pse-runtime:pse_runtime`, `pse-runtime:pse-project` and
`pse-runtime:product_reader` with `product-reader` passed 85
controls and exposed two obsolete test expectations; after correcting those expectations, their
focused rerun passed 2/2, with zero build warnings and complete capture/settlement against zero.
This is not a single 87-control receiving run. The all-target `product-reader` compile passed. After integration,
explicit `pse observe prepare` and actual `pse observe query` on main successfully analyzed two
original developer reports with exact member attribution and recorded settlement; a FIFO member
was refused with exit2 and native structured diagnostics without blocking. Actual canonical
cancellation/drain, independent-client departure and the wider C05 population remain open.

Selected retained-result receiving also exposed two distinct lifetime boundaries. Escaped
`SnapshotAllocator` values legitimately retain the original `Pressure`/pool with a weak math-service
reference. Optional projection caches must nevertheless end with the final `MathService` owner:
[`MathService::drop`](../../crates/pse-runtime/src/math/artifacts.rs) now calls the existing cache
fence/clear only. It introduces no native shutdown, cancellation or join in `Drop`. Escaped checked
buffers keep their own leases, and the escaped allocator still reserves after service death.
The revealing control verifies both obligations independently of RSS.

Separately, [`Published::receive`](../../crates/pse-runtime/src/math/jobs.rs) returns the delivered
value before its publisher next polls original settlement and drops its service owner. The actual
trajectory release control now joins existing `shutdown_native_lanes()` before asserting final
service/snapshot death; it keeps the original positive-with-buffer and exact-zero-after-drop
checks. No sleep, manual cache reset or weakened baseline substitutes for that join. Nine selected
owner-local controls and all-target compilation pass with zero failures/warnings, and main's
selected 23-control receive includes both new cache lifetime and actual transport-release controls
with complete capture/settlement (`01a1b371-1d64-75fe-b5f8-29f8588360d3`). This is the selected
cache/publisher boundary, not all lifecycle or worker-departure acceptance.

Complete slow/abandoned upload and download, partial progress with terminal stream error,
deadline-before-admission/after-handoff, observer departure with continued work, lost ACK,
reconnect, child crash, task panic, listener error and independent native shutdown refusal.
For shutdown, prove admission closure, continued drain polling, native finish, reap and I/O/task
join separately. Ignore cancelled joins only after explicit owned abort; preserve unexpected
cancellation and all other errors. B04 response reservation/backing bytes stay owned until last
segment release. A TaskTracker replacement needs explicit no-more-spawning and panic/error
owners; if it adds more machinery, retain the current JoinSet/vector design and close the actual
coverage gap instead. No task-library substitution is a package-completion requirement.

## Verification and decision limits

**Tested, selected.** The default-profile `scripts/pse test pse-runtime --lib` selection
with `property-coolprop,solver-ipopt,solver-diffsol,solver-kinsol,engineering-service,native-service,product-reader,product-operators`
and force validation passed the eight selected service/worker controls and both fitting controls.
The combined selection was 12/16 against the zero-failure baseline; four unrelated science
controls failed and are owned by G/H. Original capture and settlement completed
(`01a1c144-abf1-75b6-b44f-d3820c760ffe`). This evidence does not establish the assembled
mixed Study or an actual Study HTTP/2 reset during capture.

The Unix listener closes before native drain, and an original retirement failure remains
visible after confirmed method departure. Run and Study observation stop admitting work at the
original request deadline. An admitted status or capture retains its native settlement and
original error.

The selected runtime invocation `01a1bd97-fbbf-74ba-8f07-4fbfdd513645` passed the Run and Study
observation controls and the 16 coordinator controls. The aqueous case reached source staging
and exhausted the original activation clock before a solve. The staging owner now bounds each
batch by encoded protobuf extent and edit count, so narrow source edits can share a larger
transaction while wide edits split under the existing transport allowance. The original four
failed identities had a focused rerun. Aqueous publication then reached atomic activation and
exhausted the same clock. The activation transaction now verifies an empty destination once
under its original guards before skipping displaced-interval work for that fresh destination.
The actual aqueous receiving run and prepared live Study client journey remain open.

For canonical recipe receiving, JSON v3 is the active implementation. C04 activates the
binary reader only after D02 qualifies bounded guarded restore or replay for actual consumers
and cold receiving establishes the original custody. The cutover removes the replaced reader
in the same slice.

C01's targeted evidence and the remaining launcher/service work are
bounded above; neither establishes broader lifecycle qualification. Implementation uses
targeted native routes and matching prepared profiles, with Arrow `force_validate` where consumed.
Client and independently launched process journeys must exercise actual settlement and original
scientific outcomes; mocks establish only their bounded interface. L owns comprehensive population,
full Study/science journeys and integrated checks after functional scope. Retain command, mode,
scope and zero-baseline failure counts in the owning completion evidence.

Changes to kernel permission, metadata or accepted architecture follow the coordinator's decision
route before implementation. ADR-0251's selected-regime proposal does not authorize inference of
stability from phase fraction or eligibility. C transports only actual native observations agreed
by A/D; it creates no new mathematical authority. No new rule waiver or runtime resource policy
is introduced by this companion.
