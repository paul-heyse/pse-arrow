# SurrealDB protocol crate (`surrealdb-protocol`) with `surrealdb-types` proto conversions

Realizations considered:
- Installed: `surrealdb-protocol` 0.13.1. The workspace selects `flatbuffers` only; the SDK separately enables `rpc` through its `protocol-grpc` feature.
- Installed: `surrealdb-types` 3.3.4, whose `proto` feature carries the protobuf↔`Value` conversions.
- Upgrade route: `surrealdb-protocol` 0.14.1, published 2026-10-26 and relicensed to Apache-2.0 at 0.14.0. There is no matching released SDK.

Research date 2026-10-27. Sources:
- Installed `surrealdb-protocol-0.13.1` (`gen/rust/proto/surrealdb/protocol/rpc/v1/*.rs`, `*.tonic.rs`, `Cargo.toml` features).
- `surrealdb-types-3.3.4` (`Cargo.toml` features, `src/proto/`).
- The 0.14.1 crate archive, downloaded to scratch `dl/proto014/` and diffed against 0.13.1.
- The surrealdb/surrealdb-protocol commit log and `conformance/README.md` on GitHub, as of 2026-10-26.

## Types

### surrealdb-protocol/generated-grpc-client
- **Need it serves:** speak SurrealDB's native gRPC RPC directly, with a caller-owned transport. A codebase can then hold the connection, admission, deadline and response-accounting policy on a public seam instead of patching SDK internals.
- **Recognizable code shapes:**
  - A forked or patched database SDK whose additions sit mostly between the SDK's engine trait and its transport.
  - Wrappers that need per-request metadata which the SDK's builder cannot express.
  - Tests that stand up a scripted gRPC peer to simulate the server.
- **Mechanisms and APIs:**
  - `SurrealDbServiceClient<T>` is generic over `T: tonic::client::GrpcService<tonic::body::Body>`. It provides `with_origin`, `with_interceptor`, `max_decoding_message_size`, `accept_compressed` and these RPCs:
    - `get_capabilities`, `health`;
    - `attach_session`, `detach_session`, `reset_session`;
    - `use`, `set_variable`, `unset_variable`;
    - `signin`, `signup`, `authenticate`, `refresh_tokens`, `revoke_tokens`, `invalidate`;
    - `begin_transaction`, `commit_transaction`, `cancel_transaction`;
    - `query` (server-streaming `QueryResponse`), `run`, `kill`, `subscribe`;
    - `import_surql`, `export_surql`, and the ML import and export RPCs.
  - Message contracts include:
    - `RequestContext { session, transaction, timeout }`, where the timeout is a server-side ceiling;
    - `QueryRequest { …, accepted_encodings, max_batch_records }`;
    - server `Limits { max_message_bytes, max_chunk_bytes, max_query_duration, max_batch_records }`;
    - an authoritative result count in `QueryEnd`.
  - `surrealdb-types` with the `proto` feature converts protobuf `Value` to and from the SDK's `Value` (`encode_proto`/`decode_proto`, `src/proto/value.rs`). This is the conversion layer.
  - The transport can be any tower service: an owned `hyper` HTTP/2 connection, plus tower layers for concurrency limits, timeouts, header injection and response-body wrapping.
- **Obligations discharged:**
  - The wire schema, codegen and protobuf types.
  - Session, transaction and query RPCs with explicit per-request context.
  - The capability and limits handshake.
  - Value conversion.
  - Removes the need to modify an SDK in order to reach the transport.
- **Residual caller obligations:** everything the SDK's engine did above the wire:
  - session lifecycle and restoration policy;
  - result assembly from batched `QueryResponse` frames, including ordering, `QueryEnd` and trailer checks;
  - error decoding from status details;
  - live-query pumping;
  - typed conversion into application structs.
  The caller also owns its deadline, admission, response-budget and drain policy, now on a public seam.
- **Semantic conditions:**
  - Protocol semantics are versioned by the protocol crate and must match the server build.
  - The context lives in the message body, not in gRPC metadata. Per-request time ceilings therefore go in `RequestContext.timeout`, not `grpc-timeout`. The protocol comments explain this choice: keeping the WebSocket and HTTP bindings to the same contract.
  - `query` is server-streaming, so the caller must enforce any budget across frames.
  - In the published crate the generated sources carry `serde` derives and use prost and tonic 0.14.
- **Version / build conditions:**
  - The `rpc` feature (`proto` + tonic + tonic-prost + futures). Tonic 0.14.6 and hyper 1.11 are already in the lockfile.
  - 0.14 removes `ExportDirectory`, adds f16/f32 value kinds, and renews the CBOR vectors and catalogs. Moving the protocol crate means moving the server and SDK protocol in step.
- **Typed vocabulary without the SDK (added 2026-10-27):** `surrealdb-types` 3.3.4 publicly exposes `Error` (the SDK's `surrealdb::Error` is a re-export) with `from_wire`, `with_cause`, `query_details` and `not_found_details`; `SurrealValue`; and, with feature `proto`, `TryFrom` conversions in both directions for protobuf `Value`, `Object`, `RecordId` and related types plus `encode_proto`/`decode_proto`. A protocol-level client therefore keeps the SDK's value and error vocabulary. Only the roughly 60-line `proto::SurrealError → Error` mapping (`kind_str`, `proto_details`, `proto_error`) is SDK-private, and `Error::from_wire` is `#[doc(hidden)] pub`, which is the only route to the kind-specific codes (−32003 NotExecuted, −32005 Cancelled) and typed details.
- **SDK result rules a direct client must reproduce (added 2026-10-27).** A probe compared a minimal client on the generated stub with SDK 3.3.4 for 37 result observations across the documented script cases against a server 3.3.4 in-memory fixture (*Measured*, one run per mode; [evidence](../../../design_review/evidence/library-leverage-surrealdb-direct-client-2026-10-27/README.md)). Parity required four SDK rules:
  - **Script trim:** the query builder strips trailing `;` and whitespace from each submitted query. Only source-echoing text, such as a parse-error excerpt, differs without it.
  - **Statement compaction:** indices that never receive a frame are dropped and the rest re-numbered; `Kill` results are skipped but still use an index.
  - **`take` conversion:** `Option<T>` maps `[]` and `[NONE]` to `None`, `[x]` to `x`, and more than one element to an internal error; `Vec<T>` wraps a non-array; a missing index as `Value` is `None`; conversion failures become `Error::internal(e.to_string())`; any `take` removes an error slot.
  - **Error rebuild:** trailer-first decode of `surreal-error-bin` (8 KiB envelope) with a status-code fallback, then `from_wire` plus the cause chain.

  With those rules, about 350 transcribed lines (of a 527-line client) reached full parity, including conflict classification in both unary and query-frame failures. `IndexedResults::take_errors` returns a `HashMap`, so when several substantive errors are present the SDK's choice of error is hash-ordered; a direct client can make it deterministic. The SDK's bounded profile refuses `begin()`/`commit()`, which take no request context.
- **Live-query capability (added 2026-10-27):** protocol 0.13.1 reports delivery, resumability, multiple subscribers and retention per server (`LiveQueryCapabilities`) and accepts a versionstamp cursor in `SubscribeRequest.resume_from`, so a subscription type built on the protocol can request resumption where the server supports it.
- **Limits and negative results:**
  - The crate gives no high-level client: no `IndexedResults`, `take`, typed deserialization or session object.
  - The `Value` conversion lives in `surrealdb-types`, but the SDK's result-assembly logic is SDK-private and would be re-implemented.
  - Whether `surrealdb-protocol` publishes a stable API promise was not checked; treat it as tied to the server version.
- **Evidence:** Interface-checked: installed 0.13.1 generated client (generic `T` bound, RPC list), message definitions and doc comments (`RequestContext.timeout` and `Limits`), `surrealdb-types` 3.3.4 `proto` feature and module, and the 0.14.1 archive diff.

### surrealdb-protocol/conformance-suite
- **Need it serves:** check that a client implementation behaves like the server and other SDKs, using data-defined scenarios and value vectors. It replaces hand-copied constants and some hand-written behaviour tests.
- **Recognizable code shapes:**
  - Hand-maintained tables of method names, error kinds or CBOR tags.
  - Hand-written codec round-trip tests.
  - Behaviour tests against a scripted peer.
- **Mechanisms and APIs:**
  - `spec/vectors/{cbor,proto}.yaml` provides encode and decode vectors.
  - `spec/catalogs/` lists method, transport, header, MIME and error-kind catalogs.
  - `conformance/scenarios/*.yaml` covers connection, deny, import-export, live, protocol, query, sessions, streaming and transactions.
  - A Bun runner drives an SDK through a small JSON "driver" protocol against `surreal start memory`.
- **Obligations discharged:** codec and constant parity, and end-to-end behaviour parity for the scenarios.
- **Residual caller obligations:**
  - Fault injection: socket loss, deadline expiry, oversize frames.
  - Behaviour of patched servers.
  - Writing a driver.
- **Semantic conditions:** the runner's documented transport options are `ws,http`, so gRPC coverage of the scenarios was not established. Scenarios run against an in-memory server.
- **Version / build conditions:** added 2026-10-24, published from protocol repository 0.14.x. It needs Bun.
- **Limits and negative results:** it is not a fault-injection harness. It does not replace a scripted peer that exercises deadlines, drains or allowance violations.
- **Evidence:** Interface-checked: `conformance/README.md` and the directory listings on GitHub at the 2026-10-26 head. No scenario files were read.

## Current applicability

### [CBO-009](../../../library_leverage/codebase/persistence-and-transport.md#cbo-009) — Large local SurrealDB SDK delta with a mirrored lifetime layer
- **Types:** `surrealdb-protocol/generated-grpc-client`. `surrealdb-protocol/conformance-suite` is peripheral.
- **Realization:** installed `surrealdb-protocol` 0.13.1 with the `rpc` feature, which the vendored SDK already enables, plus `surrealdb-types` 3.3.4 `proto`. No new dependency family. Moving to protocol 0.14 would also move the patched server and the SDK.
- **Conclusion:** fits with conditions. It is the only identified library route that removes the SDK patch altogether: the canonical client would own a direct protocol client over a caller-built transport. It does not delete the lifecycle machinery; it moves that machinery from a patched supplier tree to the repository's own code on public seams.
- **What it would discharge / what remains local:**
  - Removes: the vendored SDK and engine-api trees and their deltas, and the supplier-rebase burden.
  - Hyper and tower already provide the owned connection.
  - Local code still owns admission, deadlines, the cumulative response preflight and wire-schema allowances, setup restoration, result assembly and error-trailer decode.
  - The canonical code would stop using the SDK's `query().bind().request_context()` builder, `IndexedResults` and typed `take`.
- **Prerequisites, configuration or extension:**
  - A local client module of roughly the size of the delta's substantive source, about 3k lines. That size is an inference, not a measurement.
  - Porting every canonical call site from the SDK's `Query` and `IndexedResults` API.
  - Keeping the surrealkit build, which today compiles against the edited SDK, on the SDK or moving it too.
- **Semantic gaps against the CBO's preservation constraints:** none intrinsic. Every constraint becomes local policy on public seams:
  - absolute deadlines;
  - no replay;
  - socket join;
  - prepaid allowances, implementable as a body wrapper ahead of prost;
  - 16 lanes.
  The risk is re-implementing SDK result semantics, such as statement ordering and `NotExecuted`/`Cancelled` handling.
- **Unsettled premises and what would settle them:**
  - Whether the total local code is smaller than SDK plus delta plus mirror. Settle by mapping the vendored delta's functions onto the direct-client design.
  - Whether surrealkit can tolerate an unpatched SDK.
  - The comparative review decides.
- **Evidence:** Interface-checked: generated client and message types. Proposed.

**Reviewer correction (2026-10-27).** The comparative review chose this route (its option B): the bounded lifecycle moves into a `pse-operations` module over `SurrealDbServiceClient<OwnedHttp2>` and `surrealdb-types` `proto`, the vendored SDK and engine-API leave the workspace, and the two lifecycle tiers become one owner, executed instead of the next supplier rebase. The direct-client probe measured the result and error semantics at about 350 transcribed lines with 37/37 parity (see the type above); the bounded lifecycle tier is moved rather than rewritten, consistent with the estimate of about 1k lines of client semantics. Two premises resolve: SurrealKit is built against the vendored SDK but never calls `Config::bounded_requests()` and runs the unbounded upstream-transport profile (only the `surreal-error-bin` trailer decode is not profile-gated), so its dependence on the edit is incidental and dissolves under this route; and the generated codec's `surrealdb::types::…` paths become `surrealdb_types::…`, a generator change. See the [comparative review](../../reviews/canonical-store-transport.md#cbo-009).

**Reassessment qualification (2026-10-27).** The probe deliberately omitted bounded transport,
wire preflight, admission, owned-driver joins and restoration; 37/37 is not transport equivalence.
TaskTracker completion does not seal new admission. The proposed owner retains serialized
connection admission/closing, continuation rights and independent two-driver settlement. Moving
the same code requires transport/lifetime requalification; SurrealKit on an unpatched SDK was
not built.

### CBO-016 — Contextual subscription capability

The inspected source-built server receipt `v3.3.4-mx1.0.0-effective-v1-3549c2a97bd565fd2497`
uses `server/src/rpc/grpc.rs` to advertise at-most-once delivery, no resumption, one subscriber
and no retention. Subscribe rejects a nonempty resume cursor. The protocol can express more
than this server supplies; a reopened design must re-register after loss and immediately
re-read authoritative state. Future-server resumption remains capability-gated. Evidence is
Interface-checked source, not a subscription workload.
