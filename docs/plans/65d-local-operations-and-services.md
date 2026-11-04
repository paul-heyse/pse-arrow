---
title: "Plan65d: Local operations and service composition"
status: proposed
date: 2026-11-03
adrs: []
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md]
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
| D01 | planned | Completed Plan64 baseline; 65a effects/contract identity and 65b ingress semantics | Reconcile actual operation/transport populations and original identity/effect contracts. Select the shared native handler and authentication interfaces; remove no scientific guard as mechanical dispatch. Tests compare direct/client typed outcomes and refuse untrusted peer/context before native work. |
| D02 | planned | D01 usable custody/lifecycle contract | Decide hosting with an exact released composition spike: custom Hyper versus available library host/Tonic adapter. Exercise original-image/reconnect refusal, preface/header timeout, shutdown with active native work and inner connection failure. Select library hosting only if it expresses every consumed obligation and removes worthwhile glue; otherwise use the selected released Hyper adapter. Record the chosen version/features and delete the losing production route. |
| D03 | planned | D01; narrow 65a generated registrations; D02 selected host | Integrate typed effects, route policy, early protocol-aware rejection, binary ingress, whole-limit profiles and optional authorized health/reflection. Tests attack duplicate/replaced routes, unauthorized symbol disclosure, unsupported codec and no implicit preparation. Delete hand-maintained route/effect/health protocol mirrors. |
| D04 | planned | D03; narrow 65f result and 65c identity contracts; 65i/65j authority design | Migrate submission/status/cancel/progress/readback to original identity and explicit terminal/partial outcomes. Decode observations without reconstructing native permission or settlement. Lost ACK never auto-replays effects. Test foreign/historical verdicts, cancelled observer with continuing work, reconnect, duplicate changed-request conflict and protected read after disconnect. Remove superseded polling/status/error translations. |
| D05 | in-progress | D02/D03; 65e actual multi-session requirement | Inner/outer task failure and independent shutdown aggregation corrected and targeted-tested; remaining custody/session population stays open. Carry native/allocation custody through bodies and actual drain; test stalled readers, abandoned uploads, deadlines, long silence, child failure and session-scoped concurrency. Delete replaced choreography only after its complete population moves. |
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

## Physical realization and alternatives not selected

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
