---
title: "Plan61c: Local RPC and native completion"
status: complete
doc_retention: while-dependent
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/61-code-intelligence-foundations.md
doc_topics: [current-work, design, diagnostics, environment, validation]
review_sources: [docs/functional_reviews/code-intelligence-protobuf-foundations-2026-10-31.md]
adrs: []
---

# Local RPC and native completion

## Target and lifetime assessment

**Proposed:** implement accepted RC02 and O03 with O06/O12/O13 consumers through generated
gRPC binary over HTTP/2 on the existing private UDS placement. Status/package context and
capability inspection are unary; an operation-local bidirectional conversation carries Start,
Cancel, observations and Terminal. No universal daemon or cross-request cancellation lookup
registry is introduced. [Plan61](61-code-intelligence-foundations.md) owns shared decisions.

Current `semantic/service.rs` authenticates the server's kernel PID through managed custody,
checks client UID and retains CpuTask through cancel+settle. Its connection-task lifetime is
not automatically sufficient when HTTP/2 multiplexes framework-owned request futures. The new
actual work owner must survive reset/drop of a handler or response stream. Keep session-owned
conversation workers until cancellation and join; transport delivery does not own CPU lifetime.

The [ConnectRPC](../libraries/proto/ConnectRPC_for_Rust.md) and
[local RPC](../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md) references describe
capabilities, but exact release sources govern. ConnectRPC0.9.1 has client Unix/custom connector
routes and a documented custom Hyper hosting path. Do not assume newer framework-managed
Server::serve_connection helpers exist. Tonic's simpler server credential integration was a
substantive alternative; explicit custom hosting/shutdown cost remains in this selection.

## Packages

<a id="c01"></a>
### C01 — Authenticated hosting and a surviving work owner

Consumes A02 unary/control contracts and qualified release APIs. Retain the existing explicitly
prepared helper and managed process custody. The custom client connector authenticates kernel
peer PID against original custody before the handshake and on every reconnect. The server
derives client identity from Unix credentials, checks current policy and inserts typed connection
context. Headers or socket names are not authentication. No source query starts another service.

Implement connection acceptance/shutdown using selected library machinery and the minimal
custom UDS bridge. Define a session-owned retention group for admitted conversation workers:
handler/stream drop signals cancellation, while the session still joins native work. Session
stop closes admission, cancels and drains those workers before engine release. Original native
request cancellation groups remain separate from the prepared session and sibling requests.

Acceptance tests peer refusal, authenticated connection/reconnect, connection loss with an
admitted worker, and shutdown while CPU work is outstanding. A fake peer with the correct UID
but wrong custody cannot establish a trusted helper. This package supplies transport/ownership
readiness; it does not claim the existing query consumers have cut over yet.

<a id="c02"></a>
### C02 — Integrated native producer, RPC and caller cutover

Consumes C01 and B01's shared fact constructors. This package owns the complete native engine
output-to-helper-client migration: typed queries, symbols, predicates, qualified locations,
importability and failure families replace `Reply::Answer { value }` and consumed category
parsing. Preserve opaque original provider observations only where that is their actual role.
Live LSP and snapshot capabilities remain distinct; no serialized HIR handles escape their world.

A conversation admits one Start with a well-scoped query, then optional operation-local Cancel;
the server streams complete observations and emits Terminal only after actual work settlement.
Settle ordering, input half-close, repeated/invalid control, deadlines and terminal uniqueness
through the one contract. Do not treat input EOF as a new query, a graceful half-close as a
forced reset, or RPC CANCELLED as proof of CPU completion. If output is lost, settlement is
unobserved to the client even when the session correctly drains. No resumable durable job state
or response replay is required.

Validate owned/view requests and inbound stream items at their meaningful boundary. Selected
response invariants belong to result construction; a request macro cannot prove them. Bound
producer queues and respect useful demand while retaining the native completion owner. ConnectRPC
HTTP/2 flow control alone does not bound all allocations or join Rayon work.

Acceptance includes successful native facts through the public caller, explicit cancellation
with settled original outcome, reset during CPU execution, sibling isolation, source revision
refusal, invalid control, partial output and session stop. Then delete old JSON-line client/server
framing, control byte, generic result conversions and obsolete wire-only tests. Migrate every
actual helper caller together; no compatibility listener/adapter remains. B02 can now complete
native composition against this working contract.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| C01 | complete | Helper transport/session owner | Production authenticated connector/reconnect, retained workers and original managed stop/drain exercised |
| C02 | complete | Native/RPC cutover owner | Actual typed producer/caller migration, settled cancellation, native/public journeys and legacy framing deletion exercised |

### Bounded transport investigation — 2026-10-31

The test-only generated service uses released ConnectRPC 0.9.1 through a custom Hyper
HTTP/2 UDS accept loop. `Http2Connection::connect_with_connector` performs kernel PID
verification before negotiation; the generated caller consumes a shared connection.
Server Unix credentials populate request extensions. Incoming unary view validation uses
the validator macro, while Start stream items require explicit validation before CPU admission.

Session-retained workers own actual `CpuTask` handles until settlement. Input half-close
permits completion, repeated Cancel is idempotent, and repeated Start is invalid. Output drop,
deadline and reset request cancellation without transferring CPU completion to the transport.
Bounded output must not prevent shutdown from draining work. Terminal may be delivered into
an available slot after settlement; session stop need not wait forever for a full output queue.
An unobserved terminal remains unobserved to the client.

The fixture uses controlled real Rayon work and an injected expected kernel PID. It does not
qualify the actual managed invocation-custody lookup, automatic reconnect within one client,
full HIR/native provider behavior or production caller migration. C01 must integrate the original
`authenticate_payload_peer` verifier on every connector/reconnect before handshake; C02 must
exercise all actual helper callers and delete the old framing at cutover. No compatibility
listener or public RPC service was introduced by this investigation.

## Verification

**Tested — 2026-11-01, zero failure baseline:** fourteen controlled scenarios passed in
`plan61_rpc_qualification` as part of [61b's 61-test command](61b-typed-investigation-and-agent-answers.md#verification).
They exercise the production router and surviving worker owner with real `CpuTask` work:
half-close, explicit/repeated Cancel, reset/handler loss, unary loss, metadata-only deadlines,
sibling isolation, invalid controls, blocked output during shutdown, validation field locations
and terminal invariants even without local cancellation. The duplicate test-only Session and
handler implementation were removed. Controlled CPU work does not itself qualify HIR or custody.

**Tested, composite native evidence:** `scripts/pse test pse-host --lib --features native-hir
-E 'test(semantic::native::) | test(semantic::tests::) | test(semantic::engine::tests::)'`
selected thirteen tests. Eleven passed; helper metadata and an incorrectly migrated test
selector caused two failures. After correction, the documented top-level route
`cargo t -p pse-host --lib --features native-hir -E 'test(semantic::tests::native_snapshot_authenticated_cancel_settles_one_request_and_preserves_the_session) | test(semantic::tests::native_cli_reuses_original_session_fences_sources_and_proves_stop)'`
passed both originally failed journeys. This is composite focused evidence, not a fresh
whole-series run. The native cancellation journey first authenticates a real managed helper,
cancels and settles one actual HIR request while its sibling remains usable, then stops the
original helper and reuses the same client against a replacement same-UID socket. Reconnect
refuses before sending HTTP/2 bytes. The live-LSP journey exercises ordinary public inspection,
checked edit previews, source/macro-input invalidation, session reuse and original stop/drain.

**Tested:** `scripts/pse tools prepare native-analysis` succeeded after migrating its metadata
verifier; `scripts/pse test pse-host --lib -E 'test(native_analysis_protocol_has_real_schema_documents_and_no_phantom_catalog)'`
passed. It rejects legacy versions and unrelated service advertisements. All four actual public
native/API composition journeys passed under `cargo t -p pse-host --test composed_investigations`.
Default and native-feature all-target checks pass; [61b](61b-typed-investigation-and-agent-answers.md#verification)
owns the shared command accounting. Q01 retains assembled/static qualification.

## Outcome

**Implemented and Tested:** C01/C02 are complete. Generated gRPC/HTTP2 over the original private
UDS replaces private newline JSON and the cancellation control byte. Every physical connection
authenticates original managed custody before handshake; server identity comes from Unix
credentials. All actual query/status/package/symbol/preview consumers use typed responses.
There is no compatibility listener or generic `Reply::Answer { value }` production adapter.

Session-retained workers own actual CPU tasks beyond handler/stream loss and deadline expiry.
Stop serializes against admission, cancels, drains workers and then releases the engine. Every
Run validates one Start, keeps repeated Cancel idempotent, permits input half-close and preserves
the original result separately from control/deadline failure. Terminal separates completion,
cancellation and original outcome; ordinary clients validate it even without cancellation.
LSP detachment remains distinct from returned protocol work and joined native CPU work. Missing
terminal delivery remains unobserved. Bounded queues cannot hold session drain indefinitely.

The maintained helper version and generated service identity now qualify explicit preparation;
legacy selectors retain original native administration only. RPC tracing instruments admission,
control cancellation and settlement without full payload logging or a collector prerequisite.
Durable explanation is in blueprint §21.7 and the agent environment/CLI guidance. ADR-0235 stays
proposed, its review stays scoped to A02, and D/Q01 and measured benefit remain separate.

**Proposed:** targeted service/connector/native tests exercise real ownership boundaries;
standalone gRPC examples are insufficient. grpcurl/buf curl can challenge descriptor and protocol
behavior but cannot prove original custody or CPU drain. Helper preparation remains explicit.
Q01 owns assembled qualification; no new finite CPU/RAM allocation or serial independent work
policy is introduced by these lifetime requirements.
