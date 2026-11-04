# Runtime execution and local Proto services

Inspected 2026-11-03 at source `f55951cfe995a689f9b75185b8e89cb362021616`, including
the uncommitted Proto reference additions. Exact installed ConnectRPC 0.9.1 and selected
current source paths are **Interface-checked**. The integrated realization below is
**Proposed**; no service attachment, cancellation campaign, independent-client execution or
performance measurement was performed. The target is local operation with future extension
seams, not remote deployment or distributed solving.

## Proposed target

Make local service boundaries typed projections of native operation owners. Use generated
Buffa messages and ConnectRPC service/client contracts for operations that need an independent
client or an explicitly prepared isolated worker. Keep direct Rust calls for same-process work
and high-frequency mathematical callbacks. Share operation meaning, generated metadata and
diagnostic interpretation, without forcing every internal call through serialization or a daemon.

The service has four separately owned responsibilities: contract publication, transport
admission, native operation execution/settlement, and observation/result delivery. Generated
services and standard runtime components replace generic dispatch, client, protocol and
inspection machinery; they do not own scientific effects merely because the handler returns.
An RPC deadline, stream drop or HTTP/2 reset is an observation/transport event until the native
operation owner reports what settled.

Current source already provides a strong working route. The
[engineering RPC owner](https://github.com/paul-heyse/pse-arrow/blob/f55951cfe995a689f9b75185b8e89cb362021616/crates/pse-runtime/src/host_operations/engineering/rpc.rs)
uses generated clients and handlers, authenticated Unix sockets, request adoption and native
workflow operations. Its `EngineeringEndpoint` binds original managed invocation, prepared
artifact observation and native build identity. `AuthenticatedConnector::call` authenticates
kernel peer custody and the prepared image on every connection before HTTP/2 negotiation;
`EngineeringClient::connect` checks the returned descriptor digest and compiled contract. Its
server separately authorizes kernel UID/GID/PID and bounds retained connections. The
[ingress owner](https://github.com/paul-heyse/pse-arrow/blob/f55951cfe995a689f9b75185b8e89cb362021616/crates/pse-runtime/src/host_operations/engineering/rpc/ingress.rs)
reserves aggregate decode/retention allowance before polling the body and retains it through
the response. These are observed mechanisms, not proof that every client journey is qualified.

Blueprint §21.7/§21.8 and §23 own native operating and failure contracts. The
[local RPC reference](../../../../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md) and
[ConnectRPC reference](../../../../libraries/proto/ConnectRPC_for_Rust.md) add substantial
capabilities beyond this route; the following sections select their useful compositions.

## Contract dispatch and operational service composition

ConnectRPC 0.9.1 exposes two useful physical realizations of `Dispatcher`:

| Mechanism | Local use and material condition |
| --- | --- |
| Generated `FooServiceServer<T>` | Static service-specific dispatch for a fixed small worker; avoids the dynamic lookup/type-erased path. It does not supply Router's per-route limit configuration. |
| Generated `FooServiceExt::register(Router)` | Composes engineering, observation, health and reflection services; attaches generated `Spec` metadata rather than a separately maintained method registry. |
| `Router::try_merge_in_place` | Compose modules with transactional collision refusal. Prefer it to accidental duplicate registration panic or `allow_overrides` last-wins behavior. |
| `Router::{methods,has_method,with_spec}` | Inspect actual procedure registration and attach metadata to a deliberate low-level handler. Descriptors alone do not establish callable routes. |
| `dispatcher::Chain` | Compose fixed dispatchers without converting every service to one dynamic registry when that is the better execution boundary. |

Use Router when independently composed services and operational routes justify it; retain
generated dispatch for a worker whose fixed service benefits from less machinery. A common
semantic contract need not force identical physical dispatch. Exact installed `router.rs`
confirms transactional merge and whole-profile `with_route_limits`; generated registrations
attach `Spec`. See [Router 0.9.1](https://docs.rs/connectrpc/0.9.1/connectrpc/router/struct.Router.html)
and [Dispatcher](https://docs.rs/connectrpc/0.9.1/connectrpc/dispatcher/index.html).

Use optional `connectrpc-health` 0.9.0 and `connectrpc-reflection` 0.9.0 on the same authorized
listener where operational consumers benefit. They are eligible companion dependencies, not
currently asserted workspace installations. `StaticChecker::register_with_status` permits
initial `NotServing`, followed by an explicit ready transition; `shutdown` changes current
entries but does not prohibit later registration. Health Watch is coalesced current state,
not durable progress history or proof of scientific readiness.

`Reflector::from_descriptor_set_bytes` publishes an approved dependency closure;
`Reflector::from_descriptor_pool` can reuse Buffa's linked pool. Choose one retained descriptor
owner. `with_services` changes advertisement, not access to other descriptor symbols.
Authorize reflection routes or provide a deliberately reduced descriptor bundle. Merging
sets uses first registration for repeated filenames and may partially mutate on failure;
discard a failed builder rather than treating merge as transactional schema registration.
Health/reflection installation helpers apply specialized limits; manual registration must
apply them explicitly. See [health](https://docs.rs/connectrpc-health/0.9.0/connectrpc_health/)
and [reflection](https://docs.rs/connectrpc-reflection/0.9.0/connectrpc_reflection/).

## Admission and trust before work

Keep preparation explicit: opening a client, reading descriptors, or asking for help must not
launch or compile the service. Keep Unix credentials and original managed-image custody in
the connector/server layer; a socket path is a locator, not an identity. A reconnect must
repeat authentication. This extends the current native route rather than replacing it with
an RPC header claiming the peer's identity.

Early rejection belongs at the HTTP/Tower or accepted-connection boundary. Typed interceptors
operate later and may already incur buffering/decompression. Use
`ConnectError::into_http_response(request.headers())` for protocol-correct early rejection;
an arbitrary HTTP error can obscure the reason to generated gRPC/Connect clients. Interceptors
remain useful for shared typed validation, authorization using trusted context, and diagnostic
correlation. An interceptor must not replay an effectful invocation merely because a generic
retry facility is available. See [interceptors](https://docs.rs/connectrpc/0.9.1/connectrpc/interceptor/index.html)
and [protocol-aware errors](https://docs.rs/connectrpc/0.9.1/connectrpc/struct.ConnectError.html).

Configure the complete resource profile at each effective route:

| Control | Actual scope and proposed use |
| --- | --- |
| `Limits::with_max_request_body_size` | Total received body for unary/server-streaming input, not cumulative client/bidi upload. |
| `with_max_message_size` | Individual message including decompression expansion; does not bound the retained stream population. |
| `with_element_memory_limit` | Binary decode element amplification. JSON can materialize owned values before the comparable check, so it is not an equal peak-memory guarantee. |
| `Router::with_route_limits` | Replaces a whole profile after registration. Replacing a route may discard attached limits. Reapply configuration to the effective registration. |
| `ClientConfig` and `CallOptions` decode limits | Protect independent readers too. Server admission does not constrain a malicious or mismatched response producer. |
| Native aggregate admission | Own concurrently retained payloads, conversions and response buffers beyond per-message library limits; preserve current ingress reservation until an equivalent complete mechanism is demonstrated. |

Future upload services must explicitly define cumulative message/count/retention policy; a
4 MiB message limit does not create a 4 MiB RPC. Do not impose an arbitrary cumulative limit
on legitimate long streams solely to simplify accounting. Use lazy production, bounded
retention and the actual operation's finite selection/extent. These distinctions resolve the
broader limit wording in the local-RPC reference using the narrower ConnectRPC addendum §1.2
and installed `service.rs`. See [Limits](https://docs.rs/connectrpc/0.9.1/connectrpc/struct.Limits.html).

## Native operation lifetime and truthful completion

Separate an operation's durable identity from any particular RPC. Submit an operation through
the native owner; observe, cancel, reconnect and read its original outcome through that identity.
Do not silently resubmit a scientific effect when an acknowledgement is missing. An explicit
cancel request must leave a way to observe settlement even when the initiating call is gone.

The current server's `serve` closes admission and calls `engine.shutdown()` before applying its
finite transport-connection drain. Preserve this order. Timeout of a response stream does not
prove CPU exit, native callback completion, database rollback or release of a protected result.
Physical workers and connection tasks have separate join/cleanup responsibilities; independent
failures must survive aggregation rather than being replaced by the first error.

ConnectRPC supplies useful transport controls, not the entire operation lifecycle:

| Mechanism | What the target uses it for | What remains native |
| --- | --- | --- |
| `RequestContext::{deadline,time_remaining}` | Propagate the remaining request budget to downstream calls with `CallOptions::with_timeout`, allowing time for response/cleanup. | Original operation deadline, admission, durable effect policy and physical settlement. |
| `DeadlinePolicy::{with_max,with_default_timeout}` | Bound receipt plus handler work for methods whose contract declares such a budget. Default policy has no cap/default timeout. | Scientific work policy; avoid a global short transport timeout silently constraining simulations. |
| `with_enforce_on_streams(true)` | Extend an absolute request budget to a response stream where appropriate. | Detached/CPU worker cancellation and join; the wrapper drops the stream at a yield point. |
| `with_inter_message_timeout` | Detect stalled transport/progress producers under a method-specific contract. | Distinguish a legitimately quiet long numerical phase from an unhealthy connection. |
| `Server` connection controls | Connection age/grace, idle reclamation, header timeout, concurrent streams and keepalive when adopting the library server path. | Native work drain and prepared Unix peer custody. Do not assume these controls apply to the current hand-hosted Hyper path automatically. |

The installed `deadline.rs` explicitly warns that detached tasks need caller cleanup and that
the default stream policy is unbounded. Prefer purposeful method policies to a blanket
timeout. See [DeadlinePolicy](https://docs.rs/connectrpc/0.9.1/connectrpc/struct.DeadlinePolicy.html)
and [Server](https://docs.rs/connectrpc/0.9.1/connectrpc/struct.Server.html).

## Streaming, client readiness and large payloads

`Http2Connection::connect_with_connector(...).shared(max_pending)` is the appropriate extension
of the current authenticated connector. Clone `SharedHttp2Connection` across generated clients
instead of adding an application-wide RPC mutex. Its readiness/reconnection state is useful;
`max_pending` is not a payload limit or a scientific concurrency policy. Multiple connections
and Tower load-aware balancing are eligible only if actual connection contention justifies
them. Local scientific workloads do not require a connection pool by assumption. See
[HTTP/2 client](https://docs.rs/connectrpc/0.9.1/connectrpc/client/index.html).

`ClientRequestStream` accepts asynchronous `Stream<Item=Req> + Send + 'static` production;
HTTP/2 polling can carry backpressure to the producer. Keep intermediate channels bounded or
lazy so the application does not defeat that control with an eager queue. `stream_iter` serves
an already available collection. For bidi operations, `into_split` enables independent send and
receive tasks: `close_send` ends input; dropping the receive half cancels the RPC. Half-close
is not commit acknowledgement. ConnectRPC 0.9.1's abandoned-upload cleanup releases transport
resources when a handler stops reading, but does not undo processed messages.

Consumers must preserve `ServerStream::message()`'s three outcomes: next message, successful
terminal `None`, or terminal error. Errors remain sticky and inspectable through `error()`;
`trailers()` carries available termination metadata. A valid prefix remains partial when
later decoding, deadline or remote status fails. EOF without the protocol's valid terminal
status cannot become a successful scientific result. This composes with, rather than replaces,
native result completeness and qualification. See [ServerStream](https://docs.rs/connectrpc/0.9.1/connectrpc/client/struct.ServerStream.html)
and [0.9.1 changelog](https://github.com/connectrpc/connect-rust/blob/v0.9.1/CHANGELOG.md).

For large eligible binary responses, `Encodable::encode_segments` and `EncodedBody` can retain
reference-counted large field segments through HTTP body framing. `OwnedView` is useful when
its backing bytes already exist and outlive the response. Owned messages, JSON, compression,
Connect unary responses, or an interceptor path may force contiguous materialization. Use
`Response::compress(false)` for an explicitly justified already compressed/view-backed response,
not as a universal policy. `CompressionPolicy`/`CompressionRegistry` and custom
`CompressionProvider` supply the protocol-native extension point. Benchmark the complete
producer→selection→encoding→transport→decode→release operation before claiming benefit. See
[payload API](https://docs.rs/connectrpc/0.9.1/connectrpc/payload/index.html) and the tagged
[guide](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md).

## Complete alternatives and recommended boundary

| Complete route | Judgment |
| --- | --- |
| Native call, native result and passive inspection | Retain for same-process work and solver callbacks. It removes serialization, lifecycle and failure machinery when no independent client or isolation requirement exists. |
| Generated local Buffa/ConnectRPC service over existing native owners | Recommended for independent engineering, code-intelligence and explicitly isolated scientific clients. Reuse operational components and source-derived contracts while preserving native operation identity and settlement. |
| Replace every internal boundary with one RPC fabric | Reject: it moves interpretation and adds conversion/coordination on hot paths without establishing a consumer benefit. Extensibility follows explicit capabilities, not message count. |
| Consolidate all Proto use into Tonic/Prost | Eligible but weaker on current evidence for PSE-owned view/reflection/validator composition. Keep Tonic/Prost for externally specified protocols; avoid wholesale replacement solely for stack uniformity. |
| Retain all current custom hosting forever | Reject as a design assumption. Compare `Server` and generated dispatch against the actual Unix/image admission and shutdown responsibilities. Adopt a library host only if those obligations remain expressible and replaced code disappears. |

Rich errors should provide stable codes and typed field/state/resource/request details, with
bounded public presentation and full retained diagnostics independently accessible. Standard
`google.rpc` details and `tonic-types::StatusExt` can serve Tonic consumers; ConnectRPC's typed
details serve the owned application services. Neither replaces the native scientific outcome
taxonomy. Exact error and result composition is developed in [diagnostics and results](diagnostics-and-results.md).

## Changes, controls and evidence limits

Adding a new service should add its source-owned operations and generated registrations, not
another handwritten method-name/health/diagnostic registry. Adding a worker implementation
should alter the prepared binding and supported capabilities, not the meaning of cancellation
or derivative products. A new result consumer should reuse typed original outcomes and protected
read selection rather than reconstructing success from HTTP status.

When implemented, remove duplicate dispatch tables, mechanical wire/client adapters, bespoke
health/reflection protocol handling and replaced transport glue for their actual consumers.
Retain domain validation, original-request settlement and resource custody unless a concrete
library composition fully discharges them. Breaking service versions are acceptable; temporary
dual production routes are not the target architecture.

The discriminating controls are replacement of a prepared socket peer during reconnect;
refusal before body polling; cumulative streaming versus per-message limits; descriptor
disclosure beyond advertised services; partial messages followed by terminal failure; explicit
cancel followed by original settlement; worker/client death after an effect before reply;
and a view-backed response retained across an await. Static evidence establishes the routes;
the controls are required implementation evidence, not tests run by this engagement.

Context7 found protocol-level Connect documentation, but did not resolve a Rust-specific
library account. It therefore supplies only the cross-language protocol fact that streaming
HTTP 200 is not RPC success. Exact installed 0.9.1 `service.rs`, `router.rs`, `deadline.rs`,
`client/mod.rs`, the supplied addenda and tagged primary documentation supplied Rust-specific
mechanisms. Newer Tonic material is an eligible research route, not proof of a workspace move.
