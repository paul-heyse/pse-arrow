---
title: Study execution and database boundary research
date: 2026-10-28
doc_role: evidence
doc_owner: docs/design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md
---

# Research basis and limits

This is supporting research for the principal review, not an adoption decision or a
second disposition ledger. Source is `64a217de26385129fda8d65d3d62acd4f7117fa3` plus
the uncommitted inline-child changes in `canonical_transport.rs`,
`canonical_transport_tests.rs` and `transport_lifetime.rs`. Plan57j's existing dirty
checkpoint remains included context. The ten untracked library documents are operator
inputs, preserved without staging. No product tests, builds, services or measurements
were run for this research. Library contracts below are **Interface-checked**; alternatives
and benefits are **Proposed**.

Three research contributors read the nine new bytes/egobox/Hyper/ndarray/object_store/
Prost/rand_xoshiro/tracing/Tonic overviews end to end (8,043 lines), then the Surrealism,
SurrealDB, SurrealKV and SurrealMX references (4,497 lines). Context7 resolution and
concept queries preceded exact installed-source, tagged GitHub and primary-documentation
inspection. Misleading search matches were rejected; overview/latest APIs were not
assigned to older repository pins. The coordinator inspected decisive composition and
configuration sources. This account preserves the findings useful to the review rather
than an exhaustive feature inventory.

## Actual ecosystem and application boundary

`scripts/surreal_source.py` selects SurrealDB 3.3.4 at upstream revision
`1ac2b1dba7cd177621a4cf5d507caebeb2414395`, with repository patches, exact MX 1.0 and
`--no-default-features --features storage-mem,cli,mcp,surrealism`. The MX adapter calls
`with_serializable_snapshot_isolation()`. `surreal_server.py::database_endpoint` chooses
`aol=sync&sync=every&versioned=false`, snapshots and finite engine limits. Thus stock
3.3.4's MX 0.27/SI description does not describe this selected server. KV 0.21.4 remains
in the upstream manifest; its backend is not selected. KV 1.0 is a genuine alternative
requiring adapter/build/migration assessment, not a currently exposed capability.

Python `pse._workflow.Runtime` constructs `_NativeRuntime`. The PyO3 constructor in
`crates/pse-py/src/workflow.rs` acquires local resources, verifies the loaded module,
constructs a local Rust runtime and adds deferred canonical connectivity. Python has no
declared SurrealDB SDK dependency. A database-facing command client is an alternative to
this durable control path, not the current public interface.

The [official Python interface](https://surrealdb.com/docs/languages/python) supplies
sync/async HTTP and WebSocket connections; inspected factory sources do not establish
gRPC support. Development documentation includes newer features, and no Python SDK
release was selected or qualified here. [DEFINE API](https://surrealdb.com/docs/reference/query-language/statements/define/api)
can host constrained application endpoints. The effective server checks API permissions,
then evaluates its body with ordinary table permission checks disabled under an auth
limit (`core/src/api/invocation.rs`); this differs from ordinary stored-function execution.
Endpoint principal/operation checks therefore need deliberate assessment.

## Transactional control, modules and connectivity

`database/canonical-schema/schema/canonical.surql` already owns deadlines, leases,
claims, generations, cancellation/recovery, original operation receipts and file-owner
registration. `pse-codegen/src/codegen/surreal_studies.rs` acquires bounded readiness
scopes, invokes pure Rust module policy and applies hints inside the database snapshot.
It does not ship readiness fact snapshots to the client. Study prepare/apply/settlement
stages protect durable discovery and uncertain effects; counting Rust async methods
does not establish unnecessary RPCs.

Surrealism 0.6.4 provides `sql`, `sql_with_vars`, `run` and module KV through host
imports. Its [host implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/surrealism/host.rs)
parses expressions with narrowed capabilities and inherits the parent transaction.
It does not expose ordinary SDK transaction/session handles. Current study module query,
function, network and KV restrictions are deliberate manifest selections. Writable
exports and bounded host expressions are eligible comparisons, not proven replacements.
Module KV is volatile, runtime-scoped state outside record transactions; aliases sharing
an artifact within one database can share a runtime. Network effects do not roll back
with records. Actual server patches separately govern controller admission, memory and
original deadlines. General WASI support does not establish native solver compatibility.

Selected gRPC Subscribe requires a session and refuses `resume_from`; lagging consumers
terminate with resource exhaustion. [LIVE notifications](https://surrealdb.com/docs/learn/querying/real-time/live-queries)
can supply wakeups, while authoritative snapshots or [changefeeds](https://surrealdb.com/docs/learn/querying/real-time/changefeeds)
must establish catch-up. Current canonical client exposes no Subscribe method.
The effective server binds gRPC sessions to accepted connection identity; generic SDK
session adoption/reconnect descriptions cannot override that selected policy.

Synchronous events share originating transactions. Async event queue insertion is
transactional; later database effects and queue removal share another transaction.
Exhausted retries can remove an event with a warning. Events do not turn passage of a
lease deadline into a document modification, nor supply exactly-once external execution.
Internal datastore task leases are not a general public application scheduler.

MX owns database snapshot pins, GC, persistence and engine resource bounds; those do not
automatically cover exported readers or host processes. [KV 1.0 transactions](https://github.com/surrealdb/surrealkv/blob/v1.0.0/src/transaction.rs)
offer SI and optimistic exact-key validation, not a transparent substitute for the
selected MX SSI contract. Co-locating payload and catalog may remove a cross-resource
protocol only if its complete semantics actually fit one transaction. Buckets' filesystem
write path does not establish KV-atomic publication. Direct engine performance does not
predict remote Python/Rust application performance.

## Transport, buffer and observation alternatives

Pinned Hyper 1.11.1 spawns dispatcher and body work through its executor. Tonic 0.14.6
offers configured executors/connectors, but Channel's body/buffering/reconnection
semantics differ from the current custom outgoing `Buf` and session restoration.
A [Tonic decoder](https://docs.rs/tonic/0.14.6/tonic/codec/trait.Decoder.html) receives a
complete framed message and could own pre-Prost shape checking. Encoded-buffer allocation
already precedes that callback. Supplier protocol 0.13.1 stubs hardcode ProstCodec and
provide no inspected codec setter. Such a change could remove duplicate WireBody framing,
but requires explicit generated-client ownership and early allocation/custody analysis.
Per-stream cancellation and known pre-serialization failure are distinct from socket
drain and remote rollback. Hyper-util Singleton is an eligible creation-coalescing
alternative; it supplies neither session restoration nor physical descendant joining.

`Bytes::from_owner` is already adopted by `pse-columnar::attach_bytes`. Its clone/slice
lifetime can retain a lease; a small slice can retain a large allocation. Arbitrary Buf
consumption may copy. Existing Data/FlushFence/OwnedIo separates consumption from physical
flush; attached bytes alone cannot discharge copied storage's completion obligation.

Object_store 0.13.2 already supplies range/conditional reads, streaming and multipart
operations. SharedIo intentionally streams local file results to retain its gate. Local
backend Update/durability semantics do not equal the canonical file-owner protocol.
The overview's 0.14.2 multipart cleanup fix belongs to an explicit coherent dependency
move, not an attributed current canonical defect. Native conditional puts do not replace
multi-record transactions or unknown-outcome reconciliation.

Tracing Instrumented enters span on poll and pinned destruction; WithDispatch installs
subscriber on poll without corresponding custom drop installation. Current ObservedBody
already handles poll/drop explicitly. Review context propagation and destructor attribution
separately, without inferring an unobserved defect. TaskTracker wait establishes tracked
destruction, but close does not seal task admission.

## Scientific capability alternatives

Current sampling already uses fresh directly seeded normalized DOE, logical ndarray
indexing, explicit exact grid/discrete mapping and admitted joined Rayon for quadratic
work. Supplier FullFactorial and mixed-variable sampling have different count/probability
semantics. ESE is eligible but its finite loop settings alone do not prove bounded random
draws or valid singleton behavior. Views/fused traversal/materialization need comparison
through actual multistart/surrogate consumers, not automatic flattening.

GP 0.36.4 has consuming update with fixed theta/normalization; pinned EGO refresh rebuilds
models. Compare actor retention, native model reuse through a coherent family move, and
ask-and-tell as distinct complete options. Recorded-point replay, RNG checkpointing,
model persistence and optimizer restart retain different meanings. Newer ndarray 0.17
and rand_xoshiro 0.8 APIs are not available to this pinned 0.16.1/0.6 integration merely
because other lockfile consumers resolve newer versions. No scientific method change,
provider upgrade, benefit measurement or implementation follows from this research.
