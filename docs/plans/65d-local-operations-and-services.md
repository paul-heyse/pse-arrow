---
title: "Plan65d: Local operations and service composition"
status: in-progress
date: 2026-11-03
adrs: []
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md, docs/plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md]
doc_role: plan
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [design, deployment, libraries, validation]
---

# Local operations and service composition

## Target and baseline

**Proposed:** direct native invocation and independent local clients consume one set of typed
operation handlers. Generated source-owned contracts provide dispatch, clients, method metadata,
local validation and discovery. Transport adapters authenticate participants and enforce their
own receipt/stream bounds; native owners retain effects, original operation identity, scientific
admission and actual settlement. No RPC is required merely to invoke an internal function.

This is the runtime companion to the [combined target](65-proto-integrated-target.md). Assume
only the actual native service/worker capabilities a consumer requires: joint execution now
proceeds alongside unfinished Plan64, with Proto delivery first. Their design and the same
code are fully eligible for replacement. No other
historical plan is an owner or prerequisite. Inspection at
`f61286a5fa5894b480efe8584c8b3d9b76d1b1fb` and the supplied dirty references establishes
**Interface-checked** mechanisms, not that the assumed post-Plan64 delivery is already complete.
No service attachment, interoperability, native settlement or performance test ran for authoring.

## Foundation assessment and complete alternatives

Current engineering RPC uses Buffa/ConnectRPC 0.9.1 over an authenticated custom Hyper Unix
connection. Native handlers address original run identity and protected results; shutdown closes
admission and drains native work before the transport deadline. Scientific-worker transport has
separate native-lane/process custody. These are useful foundations but not immutable boundaries.
The inspected baseline nested a connection result inside a task join and captured only outer
join failures. The scoped implementation below replaces that handling while retaining
independent native shutdown/cleanup failures; it does not qualify the complete service.

| Complete route | Selected use and consequence |
| --- | --- |
| Direct native handlers | Default for same-process workflows and numerical callbacks. Avoid an unnecessary codec, service lifecycle and failure boundary. |
| Released Buffa/ConnectRPC on authenticated Hyper UDS | Initial selected first-party independent boundary: preserves existing view/validation/source-generation composition and expresses the current peer custody contract. Consolidate shared transport lifecycle where it actually removes duplicate work. |
| Released ConnectRPC high-level server | Adopt only where its listener/context APIs express the complete selected local trust/lifecycle contract. Version 0.9.1 `from_listener` takes TCP and has no public custom `serve_connection`; do not schedule development-only APIs as released implementation. |
| Tonic/Prost or custom Tonic codec | Retain for externally governed gRPC protocols and compare for a local operation when it materially simplifies hosting/client obligations. It must also preserve first-party representation/validation contracts, not just emit a shorter server example. |
| Universal RPC fabric or daemon | Not selected. A common contract world does not justify adding transport to all internal work or merging unrelated native sessions. |

Tonic 0.14.6 already recognizes `unix:` endpoints; no bump is needed merely for ordinary UDS
dialing. The built-in connector is not sufficient evidence of original managed-image and
reconnect authentication. Use the existing authenticated connector until a replacement proves
the same custody. A newer released composition remains eligible through D02's exact decision.

## Interface and lifecycle design

### Native operation and transport identities

Consume [65i](65i-scientific-operations-and-product-semantics.md)'s operation/product semantics
and [65j](65j-workflow-lifecycle-and-developer-evidence.md)'s construction directions. Generated
requests enter trusted contextual constructors; decoded observations expose recorded facts;
sealed execution, publication and settlement capabilities have no generic inverse wire map.
Native `managed::workflow::validate_report` and `TerminalResult` remain custody owners, not
payload validators that can be replaced by successful decode. Shared adoption must preserve
these distinctions for service status, test evidence and scientific completion alike.

The [ConnectRPC 0.9.1 Router](https://docs.rs/connectrpc/0.9.1/connectrpc/router/struct.Router.html)
provides registration/dispatch, and [ServerStream](https://docs.rs/connectrpc/0.9.1/connectrpc/client/struct.ServerStream.html)
provides message/terminal-error observation. Neither grants native effect or drain authority.
Use those mechanisms with the original native owner: a terminal transport observation may be
readable while native effects remain unsettled. The local connector's trusted context remains
an input to authorization, never reconstructed from reflected options or a client claim.

Retain a native handler interface for inspect/edit/admit/prepare/submit/status/cancel/read and
scientific operations actually supported by their owner. Generate service bindings from typed
contracts, with method effects from 65a's typed custom options. Effective Router registration
and native authorization consume that same declaration; no second method-name/effect table.
Physical metadata discovery does not establish availability or scientific permission.

Distinguish request/correlation ID, durable operation ID, native attempt, worker session,
prepared generation, invocation, trace/span and result identity. A failed submit reply leaves
the original operation addressable. Reconnect queries that identity; it never silently submits
again. An explicit cancellation request addresses original native work and leaves a way to
observe its terminal state after the requesting stream closes. Known failure, unknown effect,
scientific refusal and unavailable observation remain typed distinct outcomes.

The local transport accepts only authenticated peer context derived from kernel/original
prepared-image custody. A socket path and caller-supplied header are locators/claims, not
authority. Authenticate initial connection and every reconnect before accepting HTTP/2 work;
do not infer new capability from reflection metadata. Method authorization uses trusted context
and declared native effects. Connection-only readers never bootstrap a database or launch science.

### Protocol, registration and operational services

Select binary gRPC over HTTP/2 for effectful first-party local operation routes. Do not expose
JSON or Connect GET on these routes merely because the runtime can decode them. Unsupported
protocol/codec requests must be rejected before expensive body materialization. Explicit
human/offline JSON/text inspection remains 65g's separate operation; a future read-only Connect
route needs a deliberate method contract. Idempotency alone never authorizes `NO_SIDE_EFFECTS` GET.

Use `Router` where independently composed engineering and operational services justify dynamic
registration. Generate `Spec`-backed entries, attach whole `Limits` profiles after final
registration, and reject duplicate paths transactionally. Fixed single-purpose workers may keep
generated monomorphic dispatch or `dispatcher::Chain`; that is a different population, not a
second production route for the same service. A path present in a descriptor is not necessarily
callable; tests compare intended registration and authorization with actual routes.

Health and reflection share the authorized listener where useful. `StaticChecker` begins
NotServing until the actual service is ready; health is operational state, not scientific
readiness. Watch is coalesced latest state, not durable progress. `Reflector` consumes the
explicitly approved descriptor world; `with_services` filters advertisement rather than all
symbol disclosure. Either authorize the entire disclosed corpus or publish a reduced closure.
Offline descriptors remain available to independent readers without a live reflection server.

### Work, allocation and deadlines

Receipt, decoded request, native adoption, execution, queued output, encoded body and retained
client result have separate lifetimes. Preserve aggregate admission through the last owned
allocation, including cancellation, stalled consumption and actual I/O/native drain. A transport
message/window cap is not aggregate memory ownership. Do not recreate a resource registry if
existing native reservations can be carried through the chosen representation.

`Limits::with_max_request_body_size` bounds unary/server-streaming input bodies.
`with_max_message_size` bounds one decoded message including expansion;
`with_element_memory_limit` bounds binary element materialization. They do not cap cumulative
client/bidi input or application-retained populations. Select finite operation extent or bounded
retention/backpressure for those methods; do not invent a small total-upload cap for a legitimate
long stream. Independent clients also configure response message/element bounds.

Preserve the original optional scientific deadline separately from request receipt/response
budget. Use `RequestContext::time_remaining` with downstream `CallOptions::with_timeout` where
the call consumes that budget. `DeadlinePolicy::with_enforce_on_streams(true)` can enforce
selected absolute stream deadlines; `with_inter_message_timeout` detects silence only when the
method promises a cadence. A quiet numerical solve is not automatically stalled. Stream wrappers
drop at observation/yield boundaries; detached/blocking/native work remains owned until it exits.

### Streams, connection failures and shutdown

Clients consume next message, valid terminal success, and terminal failure distinctly.
`ServerStream::message` returns `Some`, successful `None`, or sticky `ConnectError`; preserve
`error()` and `trailers()`. A valid prefix followed by failure stays partial with its extent.
Half-close ends input, not commit or native settlement. Bidi `into_split`/`close_send` permits
independent send/receive ownership; dropping receive cancels only the RPC's transport work.

Use `Http2Connection::connect_with_connector(...).shared(max_pending)` for reusable authenticated
clients rather than an application-wide RPC mutex. Do not treat pending-call count as scientific
concurrency or payload admission. HTTP backpressure must reach lazy producers; an eager
unbounded intermediary defeats it. Consolidate connection lifecycle with all inner Hyper results,
task join failures, listener failure, native shutdown and transport drain preserved in settlement.
Stop admission first; settle native effects; then drain/close transport and retain every independent
cleanup failure. Neither a stopped listener nor an empty client stream proves native release.

### Submitted work, lost observers and explicit finish

The November 5 review [I02](../plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md#I02)
and [I07](../plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md#I07)
sharpen D04/D05 without changing their existing progress. D owns the shared transport/task
contract; [65e E04](65e-scientific-portability-and-execution.md) owns the scientific
handle-map mutation, native job submission and placement finish. Definite non-submission,
submitted execution, lost acknowledgement and physical connection/process loss are separate.

Select Tokio 1.53.3 `mpsc::Sender::reserve_owned` and `OwnedPermit::send` at the
scientific handoff: wait for capacity before reversible preparation admission, then commit
admission and synchronous submission with no intervening suspension. This removes the current
control-map insertion before a cancellable `send(...).await`. Stop/Shutdown cancellation
signals remain independently available to interrupt active native work rather than waiting
behind that lane. The [Sender cancellation contract](https://docs.rs/tokio/1.53.3/tokio/sync/mpsc/struct.Sender.html#method.reserve_owned)
and [owned permit](https://docs.rs/tokio/1.53.3/tokio/sync/mpsc/struct.OwnedPermit.html)
characterize capacity ownership, not native scientific admission.

ConnectRPC 0.9.1's `service::with_request_deadline` wraps unary dispatch in
`tokio::time::timeout_at`; expiry may drop a method future while its submitted native
job continues. Preserve the native permit through computation/bookkeeping. A failed response
oneshot send records abandoned observation; it must not unconditionally terminate unrelated
prepared handles on an otherwise live parent connection. Abandoned Prepare requires a
performing-owner disposition; E04 closes that unacknowledged preparation after settlement
unless an explicitly selected original-identity recovery operation makes it addressable.
This decision applies to definite observing-method receiver departure. Successful oneshot
delivery is not a network/client acknowledgement: later reply loss remains uncertain under
the original handle/process identity and existing parent-custody policy.
Actual parent/connection loss retains the existing whole-process containment policy.
See the [exact service implementation](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/service.rs).

Use result-bearing Tokio 1.53.3 `JoinSet` for task outcomes, including joining after
`abort_all`; collect every independent teardown before returning aggregate errors.
For generic math-job supervisors that retain physical native custody, signal the original
native stop and drain with `join_next`/`poll_join_next` instead of aborting the supervisor.
Keep registration/closure and finite completed-task admission under the existing service owner;
shutdown-observer departure must not drop a taken JoinSet and abort those tasks. E04's
[performing-owner refinement](65e-scientific-portability-and-execution.md#production-finish-ownership--execution-refinement)
also covers failed final oneshot publication, retained drain outcomes and shared-surrogate
retirement. Its remaining native implementation is distinct from the already-tested
connection/session shutdown aggregation.
Tokio-util 0.7.19 `CancellationToken` signals cancellation; `TaskTracker` tracks
destruction without retaining outputs, and `close` does not close admission. It is not
a substitute for either result collection or the native admission state. Tokio-rayon 2.1.0
submits eagerly and dropping its observation handle does not stop the closure: admission
must be acquired before submission and retained inside physical work. Sources:
[JoinSet](https://docs.rs/tokio/1.53.3/tokio/task/struct.JoinSet.html),
[TaskTracker](https://docs.rs/tokio-util/0.7.19/tokio_util/task/struct.TaskTracker.html),
[tokio-rayon](https://docs.rs/tokio-rayon/2.1.0/tokio_rayon/).

The explicit scientific finish operation belongs to the existing placement owner, not an RPC
observer: close admission, request the correct session/process shutdown, reap and join I/O,
and return original typed failures together. Keep Drop as abandoned-owner containment.
Unknown reap/I/O settlement retains custody/admission; merely attempting cleanup cannot
release it. F projects the outcomes, while J records them without acquiring control.
No new supervision framework or universal lifecycle enum is selected for this flat chain.

Acceptance must distinguish cancelled reservation, definite enqueue failure, cancellation
after enqueue, abandoned Prepare reply, deadline expiry with another usable handle, and
actual parent loss. Combine original operation failure, reap failure, I/O panic and delayed
destruction; all attempted failures survive, and charges end only after physical settlement.
Remove the pre-enqueue mutation/unconditional reply-loss shutdown and ignored explicit-finish
join results only with their complete caller/test migration. This remains planned work;
the source-reviewed failure paths were not fault-executed during plan authoring.

Worker `Stop(handle)` cancels only that handle's work; `Close(handle)` retires only its prepared
state after invocations settle, as defined in 65e. Explicit session shutdown may retire that
session's remaining handles; process shutdown closes admission and drains every owned session.
Neither is inferred from a handle close or transport disconnect. If a binding intentionally supports one session/connection, declare and enforce that
profile rather than implicitly globalizing serialization. Scientific capabilities and exact
evaluation semantics are detailed by 65e; this companion owns only reusable transport behavior.

## Packages

This table owns D-package progress, including the delivered D05 slice; 65h owns assembled evidence.

| ID | Status | Prerequisite | Delivered behavior, migration and discriminating acceptance |
| --- | --- | --- | --- |
| D01 | planned | Current usable native baseline; 65a effects/contract identity and 65b ingress semantics | Reconcile actual operation/transport populations and original identity/effect contracts. Select the shared native handler and authentication interfaces; remove no scientific guard as mechanical dispatch. Tests compare direct/client typed outcomes and refuse untrusted peer/context before native work. |
| D02 | planned | D01 usable custody/lifecycle contract | Decide hosting with an exact released composition spike: custom Hyper versus available library host/Tonic adapter. Exercise original-image/reconnect refusal, preface/header timeout, shutdown with active native work and inner connection failure. Select library hosting only if it expresses every consumed obligation and removes worthwhile glue; otherwise use the selected released Hyper adapter. Record the chosen version/features and delete the losing production route. |
| D03 | in-progress | D01; narrow 65a generated registrations; D02 selected host | Engineering binary-gRPC ingress now rejects other protocols/codecs, methods and HTTP versions before body polling or collective allocation; three targeted controls pass, preserving library error framing and response custody. Complete typed effects, the remaining route population, whole-limit profiles and selected authorized health/reflection. Attack duplicate/replaced routes, unauthorized symbol disclosure and implicit preparation; delete replaced hand-maintained route/effect/health protocol mirrors. |
| D04 | planned | D03; narrow 65f result and 65c identity contracts; 65i/65j authority design | Migrate submission/status/cancel/progress/readback to original identity and explicit terminal/partial outcomes. Decode observations without reconstructing native permission or settlement. Lost ACK never auto-replays effects; distinguish definite non-submission, submitted execution, method receiver departure and uncertain network reply. E04 owns performing-handle retirement, while actual parent loss retains whole-process containment. Test foreign/historical verdicts, cancelled observer with continuing work, reconnect, duplicate changed-request conflict and protected read after disconnect. Remove superseded polling/status/error translations. |
| D05 | in-progress | D02/D03; 65e actual multi-session requirement | Canonical generic supervision/publication and service-resident resumable task/lane drain are integrated at `1b2240537`; the combined default native local receiving selection passes 47/47 with zero failures/build warnings. Replaced detached submission wrapper/metric are deleted and declarations regenerated. [E04's performing-owner refinement](65e-scientific-portability-and-execution.md#production-finish-ownership--execution-refinement) retains exact custody, bounded record and library APIs. Complete remaining shared-surrogate/session and transport population, stalled/abandoned readers/uploads, actual deadlines/child failure and independent sessions. Preserve unknown reap/I/O custody and connection-loss containment; focused default controls do not qualify the whole population. |
| D06 | planned | D04/D05 producer ownership; selected large response consumer | Apply views/segmented encoding only where producer selection and representation already avoid work. Compare complete producer→encode→transport→decode→release. Compression/interceptor flattening and backing-buffer retention are countercontrols. Keep contiguous output when simpler; delete duplicate materialization only with equivalent extent protection. |
| D07 | planned | All D functional migrations and selected 65c/e/f/g consumers | Run actual independent service/client journeys and failure/settlement population through 65h; document protocol/support/operation semantics, remove replaced paths and publish operator extension guidance. |

D02 is a bounded decision package, not a reason to leave the architecture unspecified forever.
Its baseline outcome is the released custom-Hyper adapter. Failure of an unavailable newer API
cannot block D03 onward. No dependency requires completion of unrelated storage/science packages;
consume the narrow result/effect/identity slices each operation uses.

### Initial connection settlement correction — 2026-11-04

**Implemented:** the engineering RPC connection owner now
handles both layers of `Result<Result<(), hyper::Error>, tokio::task::JoinError>` through one
local `connection_failure` operation. Active joins, graceful drain and post-abort drain all
retain inner connection failure as well as outer task failure. The original typed errors are
kept through the existing `EngineeringServiceError::Owner` source rather than reduced to a
display string. Independently returned native shutdown failures remain alongside them in
`EngineeringServiceError::Shutdown`.

The exact consumed library contracts are [Hyper 1.11.1 HTTP/2 Connection](https://docs.rs/hyper/1.11.1/hyper/server/conn/http2/struct.Connection.html)
and [Tokio 1.53.3 JoinSet](https://docs.rs/tokio/1.53.3/tokio/task/struct.JoinSet.html).
The former is a future whose output is `Result<(), hyper::Error>`; `graceful_shutdown` requests
shutdown but the connection must still be polled to completion. `JoinSet::join_next` returns
the task's output inside a separate `Result<_, JoinError>`. `abort_all` requests task abort
without removing tasks, so the owner continues joining them. Only an `is_cancelled()` result
after this owner's explicit abort request is expected cleanup; an earlier cancellation or
panic remains failure, and an inner connection error is retained even during abort drain.
Native work still settles before transport abort and retains its own failures.

Three targeted JoinSet controls distinguish successful connections, inner typed failure,
independent native shutdown refusal, task panic and expected versus unexpected cancellation.
**Tested:** native `scripts/pse test pse-runtime --lib --features engineering-service -E 'test(connection_settlement_)'`
reconciled three selected/completed tests, zero failures against a zero-failure baseline,
complete original capture and settled native test ownership. Actual source-build selection
retained Arrow force-validation for the consuming closure. These controls exercise the
nested task-result and aggregation mechanism; they are not actual socket/client journeys.
This slice contributes to D01/D05; it does not establish authentication, full streaming,
multi-session worker behavior or D07 assembled acceptance.

### Shared native request composition — 2026-11-04

The native deadline boundary now returns futures-util 0.3.34
[`BoxFuture`](https://docs.rs/futures-util/0.3.34/futures_util/future/type.BoxFuture.html):
one pinned, type-erased `Send` future. This checks transferability once and prevents
deeply nested request/settlement types from overflowing compiler trait evaluation.
Tokio still owns timeout polling and task-local destruction; enclosing minimum
selection remains lazy. The settlement wrapper boxes its child before constructing
the async task-local scope, removing duplicated large inline child state.

Seven targeted `within_clock_`/`settlement_scope_` controls passed on main with zero
failures/build warnings, including ready/cancel/unpolled destruction, parent-clock
selection, expired-clock ready priority and large-child layout. The subsequent
22-control actual capture/append selection also passed; [64g](64g-analyses-studies-and-applications.md)
owns that acceptance scope. No compiler recursion limit, warning suppression or
manual `Send` implementation was added. General operation scopes keep their existing
inline child and local-future contract. This is a D05 prerequisite, not D completion.

### Engineering binary ingress

The engineering adapter uses ConnectRPC 0.9.1
[`Protocol::detect`](https://docs.rs/connectrpc/0.9.1/connectrpc/enum.Protocol.html#method.detect)
and its `RequestProtocol::codec_format` to admit only binary gRPC POST over HTTP/2.
Classification precedes memory registration and body polling. JSON, Connect, gRPC-Web,
Connect GET and unsupported method/version requests refuse through the library's
[`ConnectError::into_http_response`](https://docs.rs/connectrpc/0.9.1/connectrpc/struct.ConnectError.html#method.into_http_response),
preserving protocol-specific error framing without granting an ingress allocation.
This replaces the engineering adapter's implicit acceptance of all library protocols;
human/offline inspection remains its separately owned operation.

**Tested, targeted:** `scripts/pse test pse-runtime --lib --profile local` with
`engineering-service,property-coolprop,native-solvers`, configured unbounded placement,
cache off, native profiles and explicit Arrow validation passed the three
`test(engineering_ingress_)` controls against zero failures. Actual registered binary
dispatch accepts the library's bare/proto/parameterized content-type forms. Unsupported
profiles leave the body unread, dispatch untouched and grants absent; protocol refusal
also precedes an exhausted pool. Two admitted response bodies retain collective custody,
a third refuses before body reads, and dropping one permits another actual dispatch.
These controls were part of the eight-control run whose separate scientific EOF failure
is tracked by [65h](65h-integration-and-qualification.md#resumed-joint-receiving-verification).
This does not close D01–D03's full route/effect/hosting population.

## Physical realization and alternatives not selected

### November 6 execution refinements

**Implemented; targeted transport controls pass:** the generated Engineering client now has six
operation-specific helpers. Fixed contract/workbench/prepared-study documents share their
producer's declared document ceiling; result, study-result and original-file reads derive
their complete encoded-message ceiling from the validated original request. ConnectRPC
0.9.1 `CallOptions::with_max_message_size` selects that per-call limit; ordinary calls keep
their explicit default. `with_element_memory_limit` remains an independent finite 32 MiB
decode profile. A legal blob above 4 MiB can now reach its selected consumer, but full legal
nested-population eligibility under that separate element profile remains open. Do not
infer it from a producer's encoding reservation. See the
[consuming helpers](../../crates/pse-runtime/src/host_operations/engineering/rpc/responses.rs)
and [ConnectRPC limit reference](../libraries/proto/ConnectRPC_for_Rust.md).

The private scientific-worker HTTP service now rejects nonbinary/non-gRPC profiles before
body polling or dispatch, using the library's `Protocol::detect` and codec-format result.
Kernel peer authentication and the two-stream numerical/control arrangement remain active.
Actual UDS/H2 controls accept selected responses above the ordinary 4 MiB default, enforce
the complete encoded-message ceiling including metadata, refuse invalid requests before
dispatch and exercise the separate element ceiling. Private ingress controls prove unsupported
profiles leave bodies unread; these pass in the 146-test receiving selection. Broader actual
service/client migrations and complete nested populations remain open.
The [private transport](../../crates/pse-runtime/src/scientific_worker/transport.rs) and
[local RPC reference](../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md) characterize
the consumed gate and service profile.

`JoinedPool::finish`, retained-surrogate finish and consuming `SurrogateHandle::finish`
report actual worker/actor departure and aggregate independent original action, Stop,
destruction and join failures. Rayon `ThreadPoolBuilder::spawn_handler` supplies the actual
OS `JoinHandle`s; pool drop alone requests termination. Existing actor gate/join ownership
fences later commands and keeps original charges through physical actor/team TLS departure.
Drop remains abandonment containment. Shared `PreparedSurrogate` retirement and separately
admitted outer job-supervisor drain retain their distinct remaining obligations. Sources:
[pool owner](../../crates/pse-runtime/src/math/joined_pool.rs),
[surrogate owner](../../crates/pse-runtime/src/math/surrogate.rs) and
[Rayon lifecycle reference](../libraries/runtime/rayon.md). Receiving fault controls must
settle constructor/destructor combinations as well as successful construction followed by
retirement; source integration alone does not close D05.

The public surrogate preparation future also establishes callback custody before its first
poll. Final evaluator destruction retains the original construction loan through arbitrary
panic-payload settlement; no nonfinal callback owner waits behind a live evaluator. The
ordinary admitted-job pipeline now accepts a typed fallible retirement action for successful
actors that cannot be published after cancellation/deadline/retention refusal. This invokes
actual actor retirement before releasing compute and memory, preserving both the stop and
independent cleanup failure. It is the same supervisor/admission path, not another lifecycle
registry. The latest constructed-actor/TLS fault control is pending receiving execution.

`Encodable::encode_segments`, `EncodedBody`, Buffa `OwnedView`/`ViewEncode` and backing-aware
rope can reduce conversion where actual source buffers exist. They do not eliminate required
numeric arrays or make compressed/JSON/intercepted paths segmented. `Response::compress(false)`
is a per-response choice when already compressed or view-backed data benefits; no universal
compression policy is prescribed. Use native `CompressionRegistry`/`CompressionProvider` only
when a real consumed encoding needs it. Whole-operation measurement, not a codec microbenchmark,
supports a benefit claim.

Multiple connections, load-aware balancing, socket activation, mTLS, Connect GET, browser
delivery, xDS and public gateways are not prerequisites for the local target. Keep extension
seams in typed operation and transport interfaces, but add machinery only for an actual
consumer/topology. `granc`, grpcurl and buf curl are independent tooling candidates owned by
65g, with protocol/Unix/message-limit support checked for each chosen journey.

## Verification and completion

**Proposed controls:** full native/client correspondence for every migrated operation;
authenticated replacement socket and same-PID image replacement; decode refusal before effects;
per-message versus cumulative retained input; malformed/unknown rich details; partial payload
followed by terminal failure; long quiet solve; explicit cancel with actual settlement;
crash after effect before reply; multiple sessions with one close; view retained across await;
native shutdown failure combined with connection failure. Independent clients must check
terminal metadata rather than equate HTTP 200, EOF, health or reflection with success.

During implementation compile touched crates and run targeted host/runtime tests with explicit
force-validation for Arrow-consuming closures. Regenerate declarations through `just codegen`.
The root applies prescribed end-of-turn formatting and scope-end hygiene; 65h selects assembled
native/service/independent-client qualification and measurements once functional scope is ready.
Report actual command, mode, scope, zero failure baseline and result. No current product
qualification follows from this document.

Completion includes all selected producers/clients/readers, exact operation/custody semantics,
relevant diagnostics and generated declarations, deletion of replaced dispatch/mapping/lifetime
mechanisms, actual shutdown/uncertain-effect controls and enduring documentation. New service
or host/metadata decisions follow the coordinator's proposed decision route before implementation;
this plan does not amend accepted architecture.

## Sources

- [Runtime capability research](../library_leverage/libraries/proto/briefings/runtime-and-local-services.md), [full ConnectRPC reference](../libraries/proto/ConnectRPC_for_Rust.md), [full local RPC/diagnostics reference](../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md), [Tonic reference](../libraries/proto/tonic.md).
- [Current engineering transport](../../crates/pse-runtime/src/host_operations/engineering/rpc.rs), [ingress custody](../../crates/pse-runtime/src/host_operations/engineering/rpc/ingress.rs), [worker transport](../../crates/pse-runtime/src/scientific_worker/transport.rs).
- [ConnectRPC 0.9.1 Router](https://docs.rs/connectrpc/0.9.1/connectrpc/router/struct.Router.html), [Limits](https://docs.rs/connectrpc/0.9.1/connectrpc/struct.Limits.html), [DeadlinePolicy](https://docs.rs/connectrpc/0.9.1/connectrpc/struct.DeadlinePolicy.html), [ServerStream](https://docs.rs/connectrpc/0.9.1/connectrpc/client/struct.ServerStream.html).
- [Released hosting source](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/server.rs), [released client source](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/client/mod.rs), [Tonic 0.14.6 Endpoint](https://docs.rs/tonic/0.14.6/tonic/transport/struct.Endpoint.html), [health companion](https://docs.rs/connectrpc-health/0.9.0/connectrpc_health/), [reflection companion](https://docs.rs/connectrpc-reflection/0.9.0/connectrpc_reflection/).
