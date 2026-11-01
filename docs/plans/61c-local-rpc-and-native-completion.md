---
title: "Plan61c: Local RPC and native completion"
status: active
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
| C01 | planned | Helper transport/session owner | Authenticated UDS hosting and retained work ownership under transport loss |
| C02 | planned | Native/RPC cutover owner | Actual native producer-to-caller migration, settlement acceptance and legacy deletion |

**Proposed:** targeted service/connector/native tests exercise real ownership boundaries;
standalone gRPC examples are insufficient. grpcurl/buf curl can challenge descriptor and protocol
behavior but cannot prove original custody or CPU drain. Helper preparation remains explicit.
Q01 owns assembled qualification; no new finite CPU/RAM allocation or serial independent work
policy is introduced by these lifetime requirements.
