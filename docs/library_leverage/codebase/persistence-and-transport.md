# Persistence, transport and relational engines

This page covers `pse-operations` (the canonical SurrealDB store), the vendored SDK, `pse-engine`
(DataFusion), `pse-relations` and `pse-columnar`. Contracts are owned by
[§20](../../authoritative_design/sections/identity-and-publication.md#section-20),
[§4](../../authoritative_design/sections/schema-and-relations.md#section-4) and
[§3.3](../../authoritative_design/sections/workspace-and-dependencies.md#section-3-3). The baseline is
on the [entry page](README.md). These opportunities share one premise: SurrealDB stays canonical,
and durability spans multiple processes without a daemon. Every opportunity here is judged within
that premise.

**System picture.** SurrealQL is used in depth. The registry-generated `surreal.surql` declares
about 186 stored functions, 122 indexes, relation tables, record references and sealing events.
Guarded reads use `SELECT … FOR UPDATE`, and the original request clock is enforced inside the
transaction. DataFusion is also used deeply: cache-plan extension nodes, `SpillManager`, `Cache`
with `DefaultCache`, custom `TableProvider`s with pushdown and statistics, catalogs, an
`ObjectStoreRegistry`, UDFs and the shared `MemoryPool`. Validation is lowered to DataFusion
expressions and anti-joins. Arrow kernels are called, not re-implemented. The following are
legitimately custom:
- the exact `u64`/`Decimal` wire codec;
- identity hashing;
- the retention and protection protocol;
- deadline semantics that must survive cancellation.

<a id="cbo-009"></a>

## CBO-009 — Large local SurrealDB SDK delta with a mirrored lifetime layer

- **Operation and consumers.** Connection lifecycle, absolute deadlines, admission, response-size
  allowances and session restoration for every canonical RPC.
- **Current mechanism.**
  - `vendor/deltas/surrealdb.patch` is 35,374 lines and `surrealdb-engine-api.patch` is 3,461
    lines, both against SDK 3.3.4. `vendor/surrealdb/PATCH.md` describes what they add:
    - bounded admission;
    - original absolute deadlines;
    - owned HTTP/2 connections;
    - response-allowance preflight;
    - session restoration.
  - Above the SDK, `pse-operations` mirrors that lifecycle:
    - `transport_lifetime.rs` tracks phase, activity counts and close rounds.
    - `transport.rs` carries task-local deadlines.
    - `canonical_execution.rs` polls `RequestContext::was_dispatched`.
    - A scripted gRPC test peer (`canonical_grpc_peer.rs`) stands in for the server.
- **Why it matters.**
  - This is the largest single ownership burden at a library boundary in the persistence
    subsystem.
  - Every SDK move requires reconstructing and re-qualifying the delta.
- **Patterns.** LOP-10 (primary), LOP-08, LOP-11.
- **Footprint.** The vendor tree, `transport*.rs`, the test peer and lifetime tests. Every canonical
  call depends on it.
- **Preservation constraints.**
  - Per-operation absolute deadlines through admission, dispatch, decode and delivery.
  - No automatic replay of application queries or transactions.
  - Physical socket join on disconnect.
  - Allowances are prepaid before protobuf decode.
  - Sixteen-lane parallelism.
- **Counterevidence.**
  - These are real operational guarantees that SDK 3.3.x lacked.
  - The delta is byte-preserving and documented.
- **Status.** Current promise.
- **Research questions.**
  - Which `PATCH.md` items now have upstream equivalents in a current SDK release? Candidates
    are request timeouts, response limits, session restore, tower layers and explicit channel
    shutdown.
  - Could the delta shrink to a smaller extension seam?
  - Could the delta be contributed upstream?
- **Library assessment correction (2026-10-27).** The 35,374-line patch is byte-preserving and mostly whitespace and reflow. The substantive delta is about 2.0k added SDK source lines, about 1.2k engine-API lines, and about 1.5k test lines. No SurrealDB release after 3.3.4 exists as of 2026-10-27. See [surrealdb](../libraries/surrealdb/README.md) and [LOP-X004](../assessment-patterns.md#lop-x004).
- **Comparative review correction (2026-10-27).** "`canonical_execution.rs` polls `RequestContext::was_dispatched`" refers to a test (`canonical_execution_lookup_queue_consumes_original_clock`), not production. At `3e9a2525b` the whitespace-insensitive delta is SDK `src` +2,834 / −477 (including 395 extracted wire-schema lines, the 316-line test-support module and about 60 lines of test edits) and engine-API +1,196 / −153 (including the new 643-line `request.rs`); lines run up to 530 characters, so line counts understate the logic. SurrealKit is built against the vendored SDK but runs the unbounded upstream-transport profile, so its dependence on the edit is incidental. A later probe of a minimal client on `surrealdb-protocol` 0.13.1 and `surrealdb-types` 3.3.4 reached 37/37 parity with the SDK path for the production script shapes once four SDK rules were reproduced (script trim, statement compaction, `take` conversion and error rebuild), in about 350 transcribed lines; the error rebuild depends on the `#[doc(hidden)]` `Error::from_wire` (*Measured*, one in-memory fixture run; [evidence](../../design_review/evidence/library-leverage-surrealdb-direct-client-2026-10-27/README.md)). See the [comparative review](../reviews/canonical-store-transport.md#cbo-009).

<a id="cbo-010"></a>

## CBO-010 — Conflict retry and guarded-transaction preludes assembled per site

- **Operation and consumers.** Retry definite transaction conflicts and capture-busy states for
  guarded reads, mutations and staging, always within the original request clock.
- **Current mechanism.**
  - `transport.rs` has `protected_query` with a shared `conflict_backoff`.
  - `canonical.rs` has a second protected-query variant.
  - `canonical_staging.rs` inlines the same exponential formula with UUID-byte jitter. A third
    form of it has no jitter.
  - Attempt caps (`GUARDED_ATTEMPTS`, `MUTATION_CONFLICT_ATTEMPTS`) and retry predicates
    (`is_conflict`, `is_capture_busy`, `retryable_stage`) are decided at each loop.
  - There are 86 non-test `BEGIN;` script-assembly sites. Each composes the guard, deadline and
    RPC-timeout prelude as text, for example in `canonical_projects.rs` and `canonical.rs`.
- **Why it matters.**
  - "Is this retryable, and how is it paced" is spelled out several times.
  - The transaction prelude is a repeated textual form. A stored-function wrapper or one typed
    builder could make it declarative.
- **Patterns.** LOP-02, LOP-09, LOP-01.
- **Footprint.** Four retry loops read. Script assembly sites counted.
- **Preservation constraints.**
  - The original clock bounds all attempts.
  - No detached retry after cancellation.
  - Each attempt rebuilds the complete decision.
  - Typed `exhaust_conflict` attribution.
- **Counterevidence.**
  - The loops are small, and the task-local clock scoping may not compose with a generic retry
    crate. A local helper may be the whole remedy.
- **Status.** Current mechanism, fragmented.
- **Research questions.**
  - Can a retry crate (`backon`, `tryhard`) take a per-attempt absolute deadline and a
    classification closure?
  - Does SurrealQL offer a parameterised transaction prelude, such as a stored function or a
    `DEFINE` construct, that keeps the guard inside one declaration?
- **Library assessment correction (2026-10-27).** There are 84 non-test `BEGIN;` lines. Of these, 61 already call a stored `fn::` guard and 60 pass `$pse_rpc_expires_at`. Fifty-one are exactly `BEGIN; RETURN fn::…; COMMIT;`, so the textual remainder is about 20–25 multi-statement preludes. Capture-busy retries do not count toward the attempt cap, and a fourth backoff formula exists in `canonical.rs` initialization.
- **Comparative review correction (2026-10-27).** Retry: there are ten retry loops with five pacing forms, not four of each. The additional loops are `begin_stage_inner` (no jitter), `cleanup_stage_query`, two in `canonical_retention.rs` (`yield_now`) and `acquire_source_stage` (fixed 20 ms). The `conflict()` predicate is duplicated in two modules, and three loops (`activate_source_stage`, `admit_product`, `acquire_source_stage`) interleave receipt settlement between attempts. Preludes: the `BEGIN;` census misses most of the residue, which is composition, not guard logic. The protected-read envelope is built from `PROTECTED_BEGIN`, with 29 references, about 20 pin re-check postludes in two variants, 29 four-variable protection bind sets, 65 `num_statements().saturating_sub(2)` result-slot computations and three near-identical project-authority wrappers (`project_query`, `context_query`, `research_query`). Pure single-call scripts number 48 by the reviewer's regex, not 51. See the comparative review: [retry policy](../reviews/execution-lifecycle-and-observation.md#cbo-010) and [SurrealQL side](../reviews/canonical-store-transport.md#cbo-010).

<a id="cbo-011"></a>

## CBO-011 — Single-flight, I/O gates and load admission beside caching libraries

- **Operation and consumers.** Deduplicate concurrent loads and bound native I/O and load
  admission. Consumers are the cache service, model results and object-store reads.
- **Current mechanism.**
  - `pse-columnar/src/flight.rs` (454 lines) implements completion-owned single flights. It
    holds shared futures in a `Mutex<HashMap>` with an atomic waiter count and cancels on the
    last waiter.
  - `pse-engine/src/cache_service/flight.rs` wraps it.
  - `stores/admission.rs` wraps the object store as `SharedIo`, with a task-local semaphore
    lease chain.
  - `cache_service/load.rs` has its own counters, semaphore and reservation guard.
  - Comments note that DataFusion's `DefaultCache` does not expose evictions.
  - `quick_cache` is in the workspace but is used only for math retention in `pse-runtime`.
- **Why it matters.** Several bounded-concurrency and deduplication mechanisms are coordinated
  only by convention.
- **Patterns.** LOP-08, LOP-03, LOP-01.
- **Footprint.** Four modules read. Callers were not traced.
- **Preservation constraints.**
  - A native load keeps its leases until it actually exits, even after every waiter leaves.
  - Cancellation on the last waiter.
  - Typed shared errors.
  - The DataFusion pool is the single budget.
- **Counterevidence.**
  - The exit-joined semantics differ from `moka` `try_get_with` and `quick_cache` placeholder
    guards.
  - The remaining opportunity may be narrow, such as `object_store::limit::LimitStore` for the
    I/O gate.
- **Status.** Current mechanism.
- **Research questions.**
  - Can a library single-flight retain its producer through waiter cancellation?
  - Does `LimitStore` compose with the nested-scope re-entrancy that `SharedIo` implements?
- **Comparative review correction (2026-10-27).** `pse-engine/src/cache_service/flight.rs` has no consumer. The "cache service" consumer does not exist; the production flights live in `MathService` and `physical_cache.rs`. See the [comparative review](../reviews/execution-lifecycle-and-observation.md#cbo-011).

<a id="cbo-012"></a>

## CBO-012 — Imperative nested-occurrence traversal beside the plan-based validation route

- **Operation and consumers.** Relation admission: find the rows and nested occurrences that
  violate declared obligations, with exact occurrence paths for diagnostics.
- **Current mechanism.**
  - `pse-relations/src/validate/occurrences.rs` (612 lines) walks nested paths in 4096-row
    chunks and calls `take` per node.
  - `validate/obligations/nested_values.rs` expresses nested-value checks as DataFusion plans.
  - `validate/predicates.rs` carries an adapter for Delta's pinned SQL parser, used for CHECK
    text.
- **Why it matters.** Two routes answer a similar "which rows violate" question, and the adapter
  ties one of them to a parser version.
- **Patterns.** LOP-09, LOP-05, LOP-11.
- **Footprint.** About 1.2k of `validate/`'s 5.4k lines read.
- **Preservation constraints.**
  - Exact diagnostics per occurrence path.
  - Memory bounded by the pool.
  - Visible-value semantics.
- **Counterevidence.**
  - Whether the two routes actually overlap was not traced.
  - The imperative path may supply location detail that the plan route cannot.
- **Status.** Current mechanism. Uncertain.
- **Research questions.** Can `unnest` with ordinals, or the array lambda functions already in
  use, yield occurrence coordinates in the plan route?
- **Library assessment correction (2026-10-27).** The plan route already builds occurrence ordinals by unnesting a `range` beside the list. DataFusion has no native ordinality. See [datafusion](../libraries/datafusion/README.md).

## Examined and not retained

- **Generated codecs.** The registry-generated Surreal codec (`generated/surreal.rs`) and the
  Arrow row codecs are imperative per-field code, and one declaration yields five forms. This is
  the intended single-authority generation (prime directive 2). The cost question is recorded
  under [LOP-X003](../assessment-patterns.md#lop-x003) and
  [CBO-040](registry-generation-and-python-boundary.md#cbo-040), not as a separate opportunity.
- **Drain waits.** The poll-and-sleep waits for owner drain and file locks are recorded under
  [CBO-016](execution-lifecycle-and-observation.md#cbo-016) and
  [CBO-003](cross-cutting-mechanisms.md#cbo-003).
