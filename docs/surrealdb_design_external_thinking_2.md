---
title: External SurrealDB engine placement and boundary cost review
---

I'll read the remaining three references and your pse-arrow notes, then lay out a placement model with rough cost numbers for each tier.

I checked pse-arrow before answering, because your current setup changes the question. Short answer: the performance hit from managing state in the database is very large per operation and negligible per unit of real work, as long as you only write to the database when something changes state, never inside solver iterations.

## First, a correction to the framing

In your build, SurrealKV and SurrealMX aren't two tiers you place data between. A SurrealDB instance runs on exactly one storage engine. Yours is a source-built 3.3.4 with MX 1.0 (serializable isolation, every commit logged and flushed to disk, plus snapshots), and SurrealKV isn't compiled in. `scripts/surreal_source.py` builds it with `storage-mem,cli,mcp,surrealism`, and ADR-0214 and your 2026-10-28 boundary research both say so. So "KV or MX" means "which engine backs the datastore", not "where does this record go".

That leaves four places data or logic can live:

1. **Rust runtime memory**: solver state, prepared models, caches.
2. **SurrealDB on MX**: the authority for coordination state and the catalog.
3. **Sealed Arrow IPC files**: bulk numeric results, with only their manifests kept in the database. You already split it this way.
4. **Surrealism**: logic that runs inside a database transaction. This is a choice of where code executes, not a storage tier.

## The cost ladder

These are my rough per-operation estimates, not measurements; ADR-0214 itself says performance is unmeasured.

| Operation | Rough latency |
|---|---|
| Rust struct or HashMap access | 1–100 ns |
| Direct MX engine get, in-process, no SurrealDB layer | ~0.1–2 µs |
| SurrealQL statement, embedded (parse, plan, execute, convert) | ~10–100 µs |
| Same statement over local gRPC to your supervised server | ~0.1–0.5 ms |
| Durable write commit (log append plus fsync on every commit) | +0.05 ms (enterprise SSD) to ~2 ms (consumer NVMe) |
| Surrealism call (WebAssembly invocation, argument encoding) | ~µs to tens of µs, plus any queries it makes |

So a durable database transition costs roughly 10⁴–10⁶ times an in-memory update. MX being fast barely changes this: once you add the query layer, gRPC and fsync, the engine is a small share of the commit cost. The MX benchmark figures (21M reads/s) describe the bare engine, not your request path.

## The rule: think in granularity ratio, not speed

Overhead = (database transitions × cost per transition) / compute those transitions control.

- **Good placement:** 1,000 study cases, each a 2 s flowsheet solve, with about 3 transitions per case (claim, heartbeat, publish) at ~2 ms each. That's 6 ms per 2,000 ms, about 0.3%. Your 16 lanes also share fsyncs through group commit, so throughput holds up.
- **Bad placement:** saving tear-stream convergence state to the database on every iteration. 50 iterations × 20 units × 1 ms is 1 s added to a 200 ms solve, a 5× slowdown.

The state-management approach is nearly free when one transition gates at least ~100 ms of compute. It is ruinous when transitions happen at iteration granularity.

## Placement test, in order

1. **Is it read or written inside a numerical loop?** Keep it in the Rust runtime, always. Load once, compute, write back a summary.
2. **Must it survive a crash, be seen by another process or caller, or act as a fence (lease, claim, generation, receipt, deadline)?** Put it in SurrealDB. This is where the simplicity win is real: atomicity, isolation and recovery replace hand-written locks, recovery logs and actor state.
3. **Is it bulk numeric data (profiles, result tables, arrays)?** Put it in sealed files, with the manifest in the database. Bulk data in MX costs RAM, snapshot time and log bandwidth for no query benefit.
4. **Is it immutable once published (model revisions, compiled artifacts)?** The authoritative copy lives in the database or files, but caching it in the runtime by content hash is safe. Never cache mutable coordination state as if it were authoritative.
5. **Is it ephemeral and cheap to recompute (Jacobians, derivatives, flash caches)?** Keep it in the runtime and accept losing it on a crash.

The structural rule follows from SSI: never hold a database transaction open across a solve. Under SSI, a long transaction widens the conflict window and pins MVCC versions in memory. The pattern you already have is the right one: claim in a short transaction, compute with no transaction open, then publish in a short transaction that checks the fence or generation.

## When the engine choice does matter (MX vs KV 1.0)

MX is the right fit for your coordination store, for three reasons:

- **Isolation:** SurrealKV 1.0 offers snapshot isolation only, not SSI. Switching would reintroduce write skew on your claim and lease invariants.
- **Reads:** MX reads never touch disk.
- **No compaction:** MX has no compaction stalls.

What MX costs:

- **RAM:** the whole dataset must fit in memory with MVCC headroom. The upstream benchmark peaked at 2× resting memory.
- **Restart time:** recovery replays the snapshot plus the log, so it grows with dataset size.
- **No history:** no `SELECT … VERSION`.

KV 1.0 becomes the better choice only if the catalog outgrows RAM, needs historical-version queries, needs encryption at rest, or if you co-locate payloads with metadata under one backup root (your 10-22 functional review raised that last option). Even then it isn't a drop-in swap; it needs adapter, build and migration work.

## Where Surrealism belongs

Use Surrealism for small, pure decision functions that must see the same consistent snapshot as the write they gate. Your RC22 readiness policy fits that exactly: one Rust source, compiled both natively and as a module. Two cases don't fit:

- **Solvers.** WebAssembly has no rayon, has sandbox memory limits, and would hold the transaction open during the solve.
- **Per-row calls over large scans.** The per-call cost multiplies across every row.

## Where the complexity win is real

The database replaces *durable, multi-process* coordination, and there the code gets much simpler. It does not beat plain in-process coordination, such as a rayon pool over cases in one process with no durability requirement. That is simpler and 10⁴× cheaper per step. The database also adds obligations of its own: retrying SSI conflicts, unknown commit outcomes, and idempotent receipts. ADR-0214 already handles these.

The useful next step would be to measure this ladder on your actual server: an empty gRPC round trip, a read-only statement, a durable single-record commit, commits from 16 concurrent lanes, and one Surrealism readiness call. That would turn my estimates into real numbers to compare against your per-case solve times.