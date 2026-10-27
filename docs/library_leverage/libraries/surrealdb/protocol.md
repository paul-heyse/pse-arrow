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
