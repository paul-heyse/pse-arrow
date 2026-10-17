---
doc_provenance: authored
doc_retention: while-dependent
title: Exact library alternatives and limits
doc_owner: docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
---

# Library alternatives

Read-only library research and coordinator source checks, against [the baseline](README.md).
Claims below are **Interface-checked** unless explicitly described as historical skill probes.
No skill, dependency, vendor source or product code was changed. No new runtime measurements
were performed. Context7 was resolved and queried for each researched library/concept; its
current documentation was compared with resolved local source. Stale repository capability
maps and the utilization catalogue were not used.

## Async ownership and polling

- **Tokio 1.53.2:** `Runtime::block_on` polls its root future on the calling thread, even with
  the multithread scheduler. `Builder::thread_stack_size` governs runtime-created threads.
  The local `runtime/runtime.rs` and `scheduler/multi_thread/mod.rs` confirm the
  [versioned Runtime contract](https://docs.rs/tokio/1.53.2/tokio/runtime/struct.Runtime.html#method.block_on).
  Its internal large-future auto-boxing sees the supplied `F`, not all nested state or poll
  frames. A small wrapper around an already-boxed future can therefore conceal substantial
  poll demand. The threshold is an implementation detail, not a stack safety guarantee.
- **Timeout:** pinned `time/timeout.rs` polls the inner future before its delay. Polls are
  synchronous/cooperative; the Python bridge's 100 ms timeout is not preemption.
- **Futures-util 0.3.34:** `boxed`/`boxed_local` call `Box::pin`; the former adds a `Send`
  bound/type erasure. [FutureExt](https://docs.rs/futures-util/0.3.34/futures_util/future/trait.FutureExt.html#method.boxed)
  creates no scheduling boundary and does not promise smaller callee poll frames.
- **Tokio spawn:** requires owned `Send + 'static` inputs/output. Dropping a join handle
  detaches; cancellation and awaited task destruction must be composed explicitly.
  The existing Python helper accepts borrowed futures. A spawned replacement therefore
  changes ownership/recovery, not just allocation. Once started, `spawn_blocking` cannot
  be stopped by ordinary task abort; it is not a cancellation-equivalent replacement.
- **Std thread:** a chosen stack or scoped borrowing can be expressed through `thread::Builder`,
  but adds thread admission, context, error/join and signal handling. No safe forced thread
  cancellation is supplied. This alternative was checked in current official documentation;
  pinned std source was unavailable in the library inquiry and no new probe qualifies it.

## Tracing and bounded scientific summaries

Tracing 0.1.44 supplies parent/follows-from relationships and entered span context around
instrumented polls. Tracing-subscriber 0.3.23 supplies `Layer` callbacks, contextual event
lookup and typed per-span extensions. The
[versioned Layer contract](https://docs.rs/tracing-subscriber/0.3.23/tracing_subscriber/layer/trait.Layer.html)
can feed a bounded repository-owned summary without reimplementing generic dispatch/context.

Registry retains live span data and removes it at closure. Keeping span handles alive delays
closure/export; it is not a ready-made terminal-summary store. Domain counter meaning,
inclusive/exclusive accounting, responsibility retention, unknown work, history/byte bounds
and scientific identity still belong to their existing semantic owners. A subscriber or
exporter cannot recover facts already discarded before export.

The `datafusion-tracing` skill's exact 55.0.0 contracts describe relational metrics read and
aggregated on recorder drop; stream completion and retained wrappers affect finalization.
Those mechanisms support relational phases/operators/storage. They do not invent native
fit/shooting trial counters, certify exporter delivery or replace terminal responsibility
retention. Prefer the existing typed observation path for already-owned native facts;
introduce a tracing bridge only for facts actually supplied through tracing.

## Salsa and retained structural products

The Salsa skill covers the resolved 0.28.4 generation. Tracked reads and complete keys can
separate structural products from changing values/demand. Tracked-struct identity fields
and creator context matter; volatile values placed in identity defeat reuse. Equal recomputed
outputs can backdate, but cycle/provisional and durability cases have additional conditions.
Memo validation may itself inspect/execute dependencies. A hit is not zero work, and neither
LRU nor heap-size reporting supplies the repository's admission byte budget.

Existing compiler tracked inputs and retained products should be consumed before introducing
another generic cache. Salsa does not remove reconstruction performed before a query lookup.
Fresh source/physical permission and actual result-affecting demand remain distinct from a
matching content hash. See [versioned Salsa docs](https://docs.rs/salsa/0.28.4/salsa/).

## SurrealDB metadata composition

Current Cargo selects vendored `surrealdb=3.3.0` with remote WebSocket/rustls; the server parser/
protocol declarations are separately pinned. The `neo4j-surrealdb` skill supplies exact 3.3
transaction/bulk/error contracts and scoped executed probes. Context7's `/surrealdb/surrealdb`
results mix current upstream source and an older README, so their API spellings are not
transferred into this vendored boundary without checking local source.

The existing repository grouped result append is the useful consumed capability. General
bulk insert/upsert cannot substitute for its attempt fencing, immutable per-operation receipt,
ordinal and exact ACK contract. Multiple statements in one network request are not a promise
of one atomic transaction. Current upstream
[query response source](https://github.com/surrealdb/surrealdb/blob/main/src/method/query.rs)
and [executor source](https://github.com/surrealdb/surrealdb/blob/main/core/src/dbs/executor.rs)
reinforce that response/transaction outcomes must be examined; pinned source and the skill's
SB013/SB031/SB032 probes establish the version-specific limits used here. No new client
transaction, generic retry framework, bulk protocol or dependency upgrade is required for
the proposed sealed-metadata grouping route.
