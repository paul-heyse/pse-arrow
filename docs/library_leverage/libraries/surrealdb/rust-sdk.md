# SurrealDB Rust SDK (`surrealdb` / `surrealdb-engine-api`)

Realizations considered:
- Installed: `surrealdb` 3.3.4 and `surrealdb-engine-api` 3.3.4 as a vendored, locally edited path dependency. The packaged sources are at upstream revision `1ac2b1dba7cd`.
- Clean upstream 3.3.4: reconstructed in scratch by reverse-applying the recorded byte deltas (`git apply -R` of both patch files). The tree is at `…/b-surrealdb/upstream/{sdk,eapi}`.
- Upgrade route: none. 3.3.4 (published to crates.io 2026-10-13, GitHub release 2026-10-19) is the newest released 3.x SDK as of 2026-10-27.

Research date 2026-10-27. Sources:
- Reconstructed upstream 3.3.4 source: `src/engine/remote/grpc.rs`, `src/opt/config.rs`, `src/opt/grpc.rs` and `engine-api/src/lib.rs`.
- The crates.io versions API for `surrealdb` (newest 3.3.4) and `surrealdb-protocol` (0.14.1, 2026-10-26).
- `gh release list` for surrealdb/surrealdb (release bodies are empty).
- `gh api compare main...v3.3.4`: the tag is 403 commits ahead of public `main`, and public `main` was last touched 2026-09-04. No public 3.3 or 3.4 development branch exists.
- The surrealdb-protocol repository commit log, 2026-07 to 2026-10-26.
- Context7 `/websites/surrealdb` (Rust live-query and query-stream pages).

## Types

### surrealdb-rust-sdk/grpc-request-lifecycle-controls
- **Need it serves:** limit how long a remote request may run, how large its responses may be and how a connection's session state is rebuilt. Use it when a caller wants per-operation time and size guarantees over the native gRPC transport.
- **Recognizable code shapes:**
  - Callers wrap every SDK call in `tokio::time::timeout`.
  - Code counts or caps result sizes after decode.
  - Code re-issues `use_ns`/`signin` after a reconnect.
  - A fork or patch adds per-request context to the SDK's engine trait.
- **Mechanisms and APIs (upstream 3.3.4):**
  - `Config::query_timeout(Duration)` and `Config::transaction_timeout(Duration)` are set per connection. The gRPC engine copies `query_timeout` into every protobuf `RequestContext.timeout`. The protocol defines that field as a server-side ceiling ("the server MAY fail sooner").
  - `GrpcConfig::max_message_size(usize)` combines with the server's advertised `Capabilities.limits.max_message_bytes`. Together they set tonic's `max_decoding_message_size`/`max_encoding_message_size`. This is a per-message bound, applied from the handshake onward.
  - `QueryRequest.max_batch_records` is the client's batch-size ask. The server clamps it to `Limits.max_batch_records`.
  - Capability negotiation (`GetCapabilities`) gates live queries, export and compression.
  - Every session keeps an internal replay log (`Replayable`: Use/Set/Unset/Signin/Signup/Authenticate/Invalidate). The log is coalesced, so a later Use or Set replaces the entry it supersedes. It is replayed to build a cloned session.
  - Errors carry `QueryError::TransactionConflict` and other typed `query_details()`.
- **Obligations discharged:**
  - A coarse, connection-wide server-side time ceiling.
  - A per-message decode ceiling negotiated with the server.
  - Rebuilding session setup (namespace, database, variables, authentication) when a session is cloned.
  - Typed conflict classification.
- **Residual caller obligations:**
  - Per-operation absolute deadlines, including time spent queuing, reconnecting and decoding.
  - Admission and concurrency limits.
  - Cumulative response budgets across a multi-frame query.
  - A guarantee that the physical socket closed before a failure is reported.
  - Deterministic shutdown. Upstream 3.3.4 has no `disconnect`/close API: a search of the reconstructed `sdk/src` and `engine-api/src` for disconnect/close/shutdown found only the WebSocket reconnect states.
  - Refusing replay for state that was never acknowledged.
  - Retry policy.
- **Semantic conditions:**
  - The timeout is relative and measured by the server, with no client-side clock. The query and transaction timeouts are fixed when the connection is built. No public builder accepts a per-request timeout.
  - The size limit applies to each message; it cannot see the sum of many legal frames.
  - Replay coalescing can reorder or merge setup relative to the order in which the server acknowledged it.
  - Transport is a tonic `Channel` built inside the SDK. No public seam injects a caller-built channel, tower layer or connector.
- **Version / build conditions:**
  - The `protocol-grpc` feature (tonic 0.14, prost) and protocol crate 0.13.x.
  - These statements cover 3.3.4 only. The public repository shows no development line beyond the 3.3.4 tag, and release notes are empty. A future SDK is unobservable, not proven to lack these controls.
- **Limits and negative results:**
  - Searched the reconstructed upstream SDK and engine-api sources at 3.3.4 for: per-request deadline, cumulative response allowance, admission semaphore, owned-connection join, explicit disconnect, tower layer or channel injection.
  - Only the connection-level timeout, the per-message limit and the coalescing replay log exist.
- **Evidence:** Interface-checked: reconstructed upstream 3.3.4 `grpc.rs` (`context()` sets `timeout: self.query_timeout`; `connect` sets the decoding limits; `GrpcSession::record` coalesces), `opt/config.rs` and `opt/grpc.rs` (the only setter is `max_message_size`). Also the crates.io versions list and the GitHub release and branch listings.

### surrealdb-rust-sdk/grpc-live-subscription
- **Need it serves:** deliver a push notification when records matching a `LIVE SELECT` change. A waiter can then wake on change instead of re-querying on a timer.
- **Recognizable code shapes:**
  - `loop { query; if done break; sleep }` over state that another process writes.
  - Wait helpers that carry a configurable poll interval.
- **Mechanisms and APIs:**
  - `db.select(table).live()` or `db.query("LIVE SELECT …")` followed by `IndexedResults::stream::<Notification<_>>(idx)` gives a `futures::Stream` of `Notification { query_id, action, data }`.
  - On gRPC, the engine issues the `Subscribe` server-streaming RPC (`SubscribeRequest.live_query_id`) after the `LIVE` statement runs. A spawned task (`pump_notifications`) forwards frames to an `async_channel`.
  - `kill(uuid)` ends the subscription.
  - The feature is gated on the server reporting `LIVE_QUERIES`.
- **Obligations discharged:**
  - Server-side change detection for one table or predicate.
  - Push delivery over an open stream while the connection lives.
- **Residual caller obligations:**
  - Catch-up after any gap, because notifications are not replayed.
  - Correctness when no notification arrives: a polling fallback or a durable change log.
  - Holding a long-lived connection and stream per waiting process.
  - Session lifetime: the live query belongs to the session and connection that registered it.
  - Bounding the notification buffer.
- **Semantic conditions:**
  - Delivery is at most once and tied to the session. Disconnecting loses the subscription and every notification in flight.
  - The live query keeps running server-side after a stream is dropped, until `kill` is called or the session ends.
  - Notifications are produced on the node that commits the write. A single-node server reaches every local session.
  - The Context7 SDK docs for JavaScript and Mojo describe live queries as WebSocket-only. The Rust 3.3.4 gRPC engine implements them through `Subscribe`, so documentation and source disagree by transport. The source is authoritative for 3.3.4.
- **Version / build conditions:**
  - SDK 3.3.4 with `protocol-grpc`.
  - The server must advertise `LIVE_QUERIES` and allow live queries in its capability configuration.
- **Limits and negative results:** a client that bounds each operation's lifetime and response allowance does not obviously host an unbounded stream. A long-lived subscription is a separate request class with its own lifetime.
- **Evidence:**
  - Interface-checked: reconstructed upstream 3.3.4 `grpc.rs` (`subscribe_live`, `pump_notifications`, the `LIVE_QUERIES` gate) and protocol 0.13.1 `SurrealDbServiceClient::subscribe`.
  - Context7 `/websites/surrealdb` Rust `live()` and `.stream()` examples, plus the JavaScript and Mojo transport notes.
  - Server-side gRPC `Subscribe` behaviour was not inspected; only the client and protocol were.

## Current applicability

### [CBO-009](../../../library_leverage/codebase/persistence-and-transport.md#cbo-009) — Large local SurrealDB SDK delta with a mirrored lifetime layer
- **Types:** `surrealdb-rust-sdk/grpc-request-lifecycle-controls`. The alternative route `surrealdb-protocol/generated-grpc-client` is in `surrealdb-protocol.md`.
- **Realization:** installed 3.3.4 only. No released upgrade exists; 3.3.4 is the newest SDK on 2026-10-27, and upstream gRPC development is not publicly visible.
- **Conclusion:** ruled out here as an upstream replacement for the delta. Upstream 3.3.4 covers only coarse parts of the delta: a connection-level server timeout, a per-message size cap and a coalescing setup replay log. No newer release exists to adopt.
- **What it would discharge / what remains local:**
  - The delta already builds on the upstream mechanisms. It replaces the connection-level `query_timeout` with a per-request remaining duration taken from the original deadline (vendored `grpc.rs` `context()`), and keeps the negotiated message-size limit.
  - Everything else in `PATCH.md` has no upstream counterpart at 3.3.4:
    - two-class admission with waiting slots;
    - an original absolute deadline through the whole lifecycle;
    - an owned hyper HTTP/2 connection with abort-and-join;
    - a cumulative response preflight across a whole operation;
    - an acknowledged-only, non-coalesced, 64-entry setup history;
    - no-replay restoration of a new connection generation;
    - explicit disconnect and drain;
    - the `surreal-error-bin` trailer decode;
    - W3C trace injection.
- **Prerequisites, configuration or extension:**
  - Short of upstream adoption, the smallest library-shaped seam would be an SDK constructor that accepts a caller-built `tonic::client::GrpcService` or connector, plus a public per-request context. Admission, trace injection and response-body accounting could then sit in tower layers outside the SDK.
  - That seam would be an upstream contribution, not an existing capability.
  - Restoration semantics (acknowledged-only, no coalescing, no application replay) live inside the SDK's session engine. They would still need an upstream change or a fork.
- **Semantic gaps against the CBO's preservation constraints:**
  - Upstream measures time relative to the server and gives no client-side absolute deadline through admission and decode.
  - Upstream has no physical socket join.
  - Upstream has no cumulative prepaid allowance before protobuf decode.
  - Upstream coalesces replay, so it may reorder setup relative to acknowledgment.
- **Unsettled premises and what would settle them:**
  - Whether SurrealDB would accept a transport-injection seam. Settled by an upstream issue or PR conversation.
  - Whether the next SDK (expected with protocol 0.14) adds per-request timeouts or close APIs. Settled by its release source.
  - Premise correction (see README): the delta's substantive source is about 3.2k changed lines, not 35k.
- **Evidence:** Interface-checked: reconstructed upstream versus vendored source, with whitespace-insensitive numstat. Applicability to the repository is Proposed.

### [CBO-016](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-016) — Waiting by polling for state owned elsewhere (store side)
- **Types:** `surrealdb-rust-sdk/grpc-live-subscription`. Changefeeds are covered in `surrealql-server-constructs.md`.
- **Realization:** installed 3.3.4. The vendored bounded engine forwards `subscribe_live` through its ordinary `execute` admission path.
- **Conclusion:** unsettled. The gRPC transport does implement LIVE through a `Subscribe` stream. Whether the bounded profile can carry a long-lived subscription is uncharacterized: the profile has an original-deadline clock, cumulative response allowances, `mark_completed` once the call returns and a drain on deadline.
- **What it would discharge / what remains local:**
  - It could deliver a wake-up hint to an in-process waiter that already holds a canonical connection, such as worker discovery of claimable attempts or study inventory changes.
  - The polling fallback, lease and claim authority, and catch-up after a gap all stay local, as the CBO's preservation constraints require.
  - It does not replace the `observe-primary` subprocess or OS-level waits.
- **Prerequisites, configuration or extension:**
  - A distinct subscription request class, outside per-operation deadlines and allowances, with its own lifetime owner and its own allowance per notification.
  - One live query per waiting process and session, plus `kill` on drop.
  - The server must advertise and permit `LIVE_QUERIES`.
- **Semantic gaps against the CBO's preservation constraints:**
  - "Bounded requests" conflicts with a stream that is open by design.
  - "No daemon" holds only if each waiting process owns its own subscription.
  - Notifications are at most once, so the fallback poll stays mandatory. That is acceptable because "correctness never depends on notification delivery".
- **Unsettled premises and what would settle them:**
  - What happens to a `Subscribe` body under the vendored bounded body wrapper after the operation completes: whether it is cut at the deadline or at the allowance. A small probe would settle it (see README, probe request P1).
  - Whether the patched server's connection-bound session and admission permit stays charged while a stream is idle.
- **Evidence:** Interface-checked: client code and protocol. Not tested. Proposed.
