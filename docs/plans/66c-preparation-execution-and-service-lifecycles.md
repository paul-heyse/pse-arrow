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
The current provider-free compiled f64 Value/First child path must remain an actual prepared-body
path, without reintroducing source replay. Complete other selected operation/context populations
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
| C02 | in-progress | Existing native operation/supervision foundation; B's needed family correspondence; A's governing scope | Common RPC mapping removal is integrated with contextual effects/authentication retained; selected main service/result receiving passes. Complete remaining typed effects, reconnect and original identity consumers with their replaced maps deleted. |
| C03 | in-progress | Existing queue/retirement owners; C02 selected operations; D's provider support | Actual-child pending Close, abandoned generated Prepare, and failed orphan retirement have selected receiving evidence on main. The bounded diagnostic transfer preserves keyed causes and refuses opaque rendering before projection. Remaining session, batch and departure populations stay open. |
| C04 | planned | Complete passive recipe and admitted-body foundations; A/D operation-specific permission; D's preallocation-bounded canonical reconstruction gate | Remaining selected live/receiving/reference contexts use exact native permission and original memberships; canonical binary activation requires bounded native reconstruction or a qualified alternative, and obsolete routes retire only with their actual consumers. |
| C05 | in-progress | Working C02–C04 producer/consumer slices and B04 where representation changes | Checkout activation and process-root observer token/nonblocking I/O/native-error retention are integrated. Actual prepared main reader succeeds and refuses nonregular input; selected host/product controls pass with corrected expectations and complete settlement. Preserve original interrupted/OOM evidence. Complete canonical cancellation/drain, broader transport/process failures, independent sessions and retained response lifetime. |

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

The common engineering RPC delivery now consumes source-generated ingress, edit, identifier,
status/progress and outgoing mappings. Native source-parent, membership, provider, effect and
authentication checks remain with their existing owners. Main's selected receiving passed after
the separate checkout-environment activation defect was corrected; B records its 38-control
bounded-response/result scope. This is not the remaining independent-client population.

Migrate all selected engineering contract/workbench/prepared-study and simulation/fit/horizon
submission/status/cancel/progress/read paths, plus their preparation, worker and reference service
routes. Coordinate F's generated Rust client in the same slices. Compare direct/native and remote
typed outcomes; preserve errors' original structured diagnostic class. Cover initial/reconnected
wrong image, stale context, unauthorized reflection, empty/present-default requests, changed
duplicate request and lost submit ACK. Delete replaced mechanical adapters, hand-maintained
method/effect/health mirrors and duplicate polling translations as each family moves. Native
checked edit and parent/membership guards remain.

### C03 — Close performing owners and worker sessions

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

Resolve the canonical binary Recipe activation gate with D before switching receiving away from
current JSON interpretation3. Native normalization, coefficient/partial growth and Symbolica global
registration need an enforceable preallocation envelope at their actual owner. Decoder budgets,
postconstructed node/slot limits, payload-size multipliers and process custody do not discharge
that obligation. D supplies bounded canonical reconstruction, a qualified alternative or an
explicitly approved revised contract before C activates its receiver. Retain the current JSONv3
production path until that gate and its actual receiving controls are satisfied; do not retain it
after a qualified replacement has moved every caller. This local reconstruction bound does not
change the unbounded host policy. The already implemented provider-free compiled f64 Value/First
live child is a separate admitted-body path and supplies no canonical receiving qualification.
The [mathematical owner's source investigation](66d-mathematical-representation-and-numerical-execution.md#canonical-reconstruction-activation)
identifies maintained Symbolica's inspected/pre-admitted f64 program transfer as a narrower
receiving seam. It does not restore the symbolic `AdmittedBody` returned by canonical replay.
Preserve that distinction when choosing real worker/reference consumers and their demanded
orders, original layouts and receipts; no newly serialized program grants native adoption.

Complete original qualified recipe membership across the canonical receiving population and the
actual prepared worker/reference consumers. Extend live prepared-body execution where D supplies
the matching supported representation; do not create generic restoration from a descriptor.
Refuse changed source/layout/datum/provider identity, unavailable original rule or unsupported
arithmetic/derivative path before execution. Verify original physics and demanded derivatives at
the consuming scientific companion; transport round trips alone cannot establish them. Remove
source-recompile, duplicate receiving or obsolete JSON paths only after exact replacement readers
and native receiving acceptance exist. Historical readable records can retain unavailable use.

### C05 — Verify actual shutdown and transport composition

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

Implementation is active. C01's targeted evidence and the remaining launcher/service work are
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
