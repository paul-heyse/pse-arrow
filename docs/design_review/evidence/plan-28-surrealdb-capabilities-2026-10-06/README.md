# SurrealDB capabilities for Plan 28

This is bounded library evidence for the adopted Plan 28 substrate, not a product
qualification, a second status ledger, or a new database design. It informs the remaining
A/B/C/D/E implementation and the EF04/EF05 audit. Plan 28's coordinator retains the sole
US/EF/F disposition authority through its
[finding table](../../../plans/28-surrealdb-unified-substrate.md#finding-dispositions).
Local implementation and assembled acceptance owners remain
[28a](../../../plans/28a-canonical-substrate-and-revisions.md),
[28b](../../../plans/28b-selected-compilation-and-reuse.md),
[28c](../../../plans/28c-durable-execution-and-studies.md),
[28d](../../../plans/28d-connected-results-and-analysis.md), and
[28e](../../../plans/28e-rebuild-retirement-and-qualification.md). The dated reviews retain
their original observations rather than current dispositions.

## Evidence boundary

The selected shared [SurrealDB guide](../../../../.codex/skills/neo4j-surrealdb/surrealdb/GUIDE.md)
and its capability briefs/probe ledger cover the Rust SDK, core and server image at 3.3.0.
The server source is release commit
[`238bfeb11f5725bebed370167656748df8067595`](https://github.com/surrealdb/surrealdb/tree/238bfeb11f5725bebed370167656748df8067595);
the captured official documentation is
[`surrealdb/docs.surrealdb.com` commit `82b7ac1935fcbb400e80cf357dd68f47182bad26`](https://github.com/surrealdb/docs.surrealdb.com/tree/82b7ac1935fcbb400e80cf357dd68f47182bad26),
the documentation `main` snapshot on 2026-10-05. The probes ran against the pinned 3.3.0
server image and SDK; the local rustdoc/probe toolchain was Rust 1.99.0. The skill overview
reports 122 probes (116 behavior and 6 compile), while generated `content/PROVENANCE.json`
counts 121 confirmed probes (115 behavior and 6 compile). This note relies on named probe IDs
rather than that inconsistent aggregate. These observations describe that version and tested
profiles, not a guarantee for another backend, deployment topology, or future release.

Remote HTTP/WebSocket/gRPC parity probes used a memory-backed server. The separate
RocksDB isolation/conflict controls used the embedded backend; neither establishes the
production remote gRPC/RocksDB profile. Core `Datastore::run_streaming` partial-row retraction
is the `SB103` probe, separate from release-source inspection of SDK gRPC End handling.
Those scopes must stay distinct when selecting application completion and recovery controls.

Official docs and project pages read through Context7/web are cited as current public sources
below. They can move independently of 3.3.0. SurrealKit is specifically outside the pinned
skill probes. No server, migration tool, restore, or performance experiment was run for this
note.

## Adoption decisions by operation

| Capability | Decision for Plan 28 | Boundary and reason |
|---|---|---|
| Direct-ID and bounded bulk reads | **Adopt** | D’s exact run/revision/result-set/output/coordinate keys are the right database access path. Bind typed record IDs, not strings that merely look like IDs (`SB006`); avoid wide table scans and hydrate only the selected rows/blocks. Batch a bounded set of known IDs when it preserves the caller’s exact selection. Database order, current heads, or a “latest” query does not replace A/C identity and eligibility checks. |
| Namespace/database selection | **Adopt** | Select both namespace and database explicitly on every connection before queries. Embedded storage has no default, and a remote server’s `main/main` is selected only after `use_defaults()` (`SB001`, `SB050`). An implicit default is an avoidable wrong-database hazard. |
| Schema constraints and indexes | **Adopt** | Use schemafull typed fields, unique indexes for uniqueness, and `TYPE RELATION … IN … OUT … ENFORCED` where endpoint existence is an invariant. The local 3.3 DDL brief covers 59 statements on Mem, SurrealKV and RocksDB; probe controls show ordinary relations allow duplicate/dangling edges unless constrained (`SB020`, `SB022`). Selectivity and exact numeric/index semantics still need consumer tests. |
| `INFO` and `EXPLAIN [ANALYZE]` | **Adopt for diagnosis; not as a receipt format** | `INFO FOR DB/TABLE/INDEX` and `EXPLAIN` help inspect definitions and representative query plans. The official EXPLAIN page calls its shape informational and subject to change; E4 must use the owning complete-operation measurements, not parse this output as a stable proof or timing receipt. Index and graph plan shapes should be checked for the actual selected row/block queries. [EXPLAIN](https://github.com/surrealdb/docs.surrealdb.com/blob/82b7ac1935fcbb400e80cf357dd68f47182bad26/src/content/reference/query-language/statements/explain.mdx) |
| Record lock versus predicate guard | **Do not use a predicate lock as the claim authority** | The pinned transaction controls demonstrate write skew on Mem, SurrealKV, RocksDB and TiKV when two transactions read the same predicate and update different records (`SB091`, `SB110`). `FOR UPDATE` accepts record IDs, not a predicate/table target, and does not turn a range query into a locked predicate (`SB092`). SDK conflicts are not automatically retried, and error typing varies by backend (`SB033`, `SB053`, `SB089`, `SB090`, `SB109`). Use C’s exact per-occurrence claim/fencing and terminal transition as authority; a short transaction or record lock may protect that known record, but does not prove the whole readiness predicate remained unchanged. Retry only the owning operation under its existing identity and budget. |
| Immutable lifecycle roots | **Adopt** | Keep revision, run, attempt and closed result-set identities explicit and immutable; navigate to exact owners rather than rewriting a mutable problem row or silently resolving “latest usable.” Ordinary record links can outlive targets (`SB023`), so use deletion/reference rules deliberately and preserve C/A protected-reader and retirement fences. SurrealDB record identity is a storage mechanism, not lineage or scientific authority. |
| SDK query completion and gRPC streaming | **Adopt with two completion layers** | Use `.check()` or inspect each statement result: `db.query()` may return `Ok` while a statement failed (`SB013`). The 3.3 Rust SDK’s gRPC route can stream result items incrementally; this is a transport opportunity, not permission to publish partial rows. Buffer/retain rows provisionally and seal only after SDK stream termination and application terminal success. The SDK `query_inner` End-frame check validates a terminal frame and `result_count`; an End frame reporting zero is still valid when zero results arrived. It does **not** establish the application’s expected statement count, terminal-zero/staging invariant, run eligibility, or durable acknowledgment. D’s Arrow/export boundary and C’s terminal seal remain separate checks. [Rust gRPC connection](https://surrealdb.com/docs/reference/rust/methods/connect), [release-pinned gRPC response handling](https://github.com/surrealdb/surrealdb/blob/238bfeb11f5725bebed370167656748df8067595/surrealdb/src/engine/remote/grpc.rs) |
| Export/import and offline backup | **Conditional on the E3 restore journey** | SDK/CLI export produces a logical SurrealQL dump; import inserts records and is not an upsert (`SB118`). Restore into a controlled empty target (or an explicitly cleared one), then verify exact IDs, schema/interpretation and critical queries before use. Official self-hosted guidance distinguishes logical exports from storage snapshots; direct SurrealKV/RocksDB directory copies are safe only while no process has the store open. Neither a dump nor an offline snapshot alone proves abrupt-crash durability; E3’s acknowledged-write kill/reopen test does. [Backups and recovery](https://github.com/surrealdb/docs.surrealdb.com/blob/82b7ac1935fcbb400e80cf357dd68f47182bad26/src/content/manage/self-hosted/backups-and-recovery.mdx) |
| Native functions and study claims | **Use existing versioned transactional owners; don’t add a second scheduler** | Native query functions can filter/project bounded structural records near the store. The selected design may use its existing versioned database functions for atomic claim/fence operations where C owns them. Preserve that one owner and its exact version/identity. Do not copy study readiness into another database function, let CRUD bypass eligibility, or move solver/scientific policy to storage. C’s scoped claim, cancellation and recovery journeys remain necessary. |
| Relation direction, recursion and graph caches | **Adopt regular relations and bounded traversal; optimize conditionally** | Use regular relation records for provenance/lineage edges that need typed fields, indexes, lifecycle references or methods. A `LIGHTWEIGHT` relation is only an endpoint pair: no record payload, fields, indexes, events, changefeed or live subscription; duplicate `RELATE` is idempotent because the pair is its ID. It is unsuitable for metadata-bearing lifecycle relationships. Recursive traversal/shortest path can serve bounded dependency/incidence exploration, not numerical sensitivity, solver routing, or a general graph algorithm. 3.3 `INLINE` edge fields and table `INLINE EDGES` / `INLINE REFERENCES` caches trade write/storage work for selected traversal reads; use only after E4 identifies a hot compatible shape. Field inlining helps only when the predicate is inside the graph path. The docs describe these 3.3 features, but the skill’s named probe set does not qualify their performance. [DEFINE TABLE](https://github.com/surrealdb/docs.surrealdb.com/blob/82b7ac1935fcbb400e80cf357dd68f47182bad26/src/content/reference/query-language/statements/define/table.mdx), [DEFINE FIELD](https://github.com/surrealdb/docs.surrealdb.com/blob/82b7ac1935fcbb400e80cf357dd68f47182bad26/src/content/reference/query-language/statements/define/field.mdx) |
| Events, live queries and changefeeds | **Unnecessary for current lifecycle authority** | Live select is available over WebSocket or embedded, not HTTP; subscriptions end on WebSocket reconnect and are not resubscribed (`SB051`, `SB101`). `SHOW CHANGES` needs a declared, time-bounded `CHANGEFEED`; without one it silently returns an empty result (`SB043`). Embedded async events need maintenance tasks to run (`SB104`). These can support a concrete observer or short-lived integration, but not worker readiness, durable queueing, a solver result, or the completion authority. Reconnect/resubscribe and registration cleanup belong to any adapter that later adopts live queries. |
| Surrealist, SurrealKit and other tooling | **Surrealist: optional human diagnosis. SurrealKit: not required; investigate only for a concrete deployment need.** | Surrealist provides a UI for query/data/schema inspection, not headless acceptance evidence ([project](https://github.com/surrealdb/surrealist)). SurrealKit documents desired-state `.surql` synchronization and phased rollouts, but this is a distinct CLI/library and migration authority, outside the pinned 3.3 probes. It conflicts with Plan28’s clean regeneration/hard pivot if it introduces an independently authored schema or legacy-preserving rollout. If operations later require it, feed it only generated canonical SurrealQL and first verify exact 3.3 compatibility in a disposable target; do not add a migration dependency or compatibility path preemptively. [SurrealKit](https://github.com/surrealdb/surrealkit), [schema migration](https://surrealdb.com/docs/manage/schema-migration) |

## Selected acquisition and invariant placement

A/B should group exact structural demand pairs and selected extent metadata before fetching
payloads. Independent `scope IN scopes AND name IN names` predicates describe a Cartesian
product, not the original demanded pairs. Bound the pair tuples or explicit IDs, carry the
namespace inventory and its guard premises forward, and process only newly acquired rows and
unresolved references. The native bulk capability removes redundant crossings only if the
caller stops singleton lookup/import/probe loops and accumulated rescans. The audit's source
operation counts are not measured latency. Total extent admission matters because awaited SDK
indexed results collect batches; a wire-frame cap alone is not a whole-response memory cap.

For a negative name or namespace-membership premise, snapshot isolation and a successful
range query are insufficient. Lock/touch a stable named guard in the same decision, and
require every mutation capable of invalidating that premise to participate. `FOR UPDATE`
registers an optimistic commit conflict for explicitly named IDs, including absent IDs; it is
not a blocking row mutex or predicate lock (`SB091`, `SB092`, `SB110`). Definite conflicts
retry the complete guarded read/decision/write; unknown acknowledgments settle the original
operation identity before any retry. This retains local retry scope without a second scheduler.

READONLY fields reject changed values but permit the same existing value in UPSERT in the
release's upstream language tests. They do not by themselves prevent delete/recreate or
establish lifecycle mint/release authority. Synchronous event `THROW` rolls back the triggering
write (`SB122`, memory profile); asynchronous events run after commit and cannot own atomic
admission. A generated field/event may reinforce a concrete invariant, but history roots must
remain movable while run/attempt/analysis association stays lifecycle-owned. No blanket schema
immutability or independently authored trigger policy is selected.

Direct bound IDs, projections and indexes are preferable to wide scans for exact result keys.
Use version-scoped EXPLAIN/INFO to check an actual representative plan; the native optimizer
still needs selective predicates and indexes. Recursive deduplicated collection answers a
different question from path enumeration, and bounded traversal termination does not prove
scientific closure. INLINE field predicates can reduce metadata edge fetches only when the
predicate sits in the graph traversal; LIGHTWEIGHT edges imply endpoint enforcement but lack
metadata and distinguish only the endpoint pair. D preserves regular scientific lineage edges
and evaluates inlining for an actual consumer before claiming a performance benefit in E4.

## What needs a deciding check

The pinned probes already settle SDK-level semantics used here: typed ID behavior, explicit
namespace/database selection, statement error extraction, write-skew/record-lock limits,
WebSocket reconnect, changefeed requirements, partial stream retraction and import collision
behavior. Core streaming retraction is not an executed gRPC adapter control. The 3.3
SDK implementation requires a gRPC query End frame and validates its
`result_count` against received results; an End with zero results can still be transport-valid.
The application must separately require its expected query/result shape and C's terminal-zero
invariant. See the release-pinned
[gRPC query response handling](https://github.com/surrealdb/surrealdb/blob/238bfeb11f5725bebed370167656748df8067595/surrealdb/src/engine/remote/grpc.rs#L780-L836).
No duplicate library probe is justified for the named semantics.

E3 still needs its planned selected-profile crash/reopen and interrupted restore journeys,
because those are application durability and identity claims, not SDK syntax claims. If gRPC
is the chosen deployment transport, the assembled test should include a multi-row stream and
the missing-frame/count error path; independently assert final application terminal status and
that no staged prefix is eligible after error/cancellation. If an operator selects SurrealKit,
only then run a small disposable compatibility check: generated schema input, sync/rollout,
and restart under the exact server release, with no second schema source. If E4 indicates a
representative traversal bottleneck, measure the real complete operation before enabling
INLINE caches; the current evidence is not a performance result.

For EF04, inspect the final dependency/features-powerset and focused build closures after the
selected SDK/engine is wired; this note cannot predict the repository’s resolved closure. For
EF05, verify full dirty-source attestation remains at the executable/publication edge and
preparation identity still contains its complete relevant implementation/provider/config
inputs. Database INFO/EXPLAIN, export timestamps and server-generated identities cannot replace
either audit.
