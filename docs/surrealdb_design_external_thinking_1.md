Using **SurrealMX 1.0 and SurrealKV 1.0 as the baseline**, I would favor your state-management approach for **shared application state and meaningful state transitions**. The performance-sensitive boundary is how much work you put between those transitions.

My recommendation is:

- **SurrealDB owns shared meaning:** records, relationships, eligibility rules, ownership of work, accepted results, and transitions that must be consistent.
- **SurrealKV provides the default durable home for that state.**
- **SurrealMX provides a transactional, memory-resident working dataset when that is independently useful.**
- **Native Rust owns execution machinery and computational working memory:** tasks, buffers, numerical arrays, handles, and rapidly changing intermediate values.
- **Surrealism hosts custom Rust logic that benefits from participating directly in database operations.**

The key is that you can obtain most of the complexity reduction from database-managed state **without making every computational step a database operation**.

I read the attached references and checked the relevant upstream architecture, transaction, and runtime documentation. The distinctions below are architectural recommendations, with implementation facts cited separately.

## 1. Separate storage placement, state semantics, and execution placement

Your question combines three decisions that are easier to assess independently.

| Decision | Main question | Options |
|---|---|---|
| **State semantics** | What guarantees and access patterns does this state need? | Ordinary Rust ownership; transactional KV; full SurrealDB records, queries, indexes, relationships, and events |
| **Storage placement** | What lifetime, recovery, and capacity requirements apply? | SurrealMX memory-first storage; SurrealKV persistent storage |
| **Execution placement** | Where should the algorithm or transition run? | Native Rust; SurrealQL; Rust compiled into a Surrealism module |

SurrealDB’s query layer handles query execution, permissions, index maintenance, and record operations. The underlying engine supplies transactional operations on keys and ranges. Consequently, a graph traversal or indexed query is executed by **SurrealDB over SurrealMX or SurrealKV**; it is not an algorithm that you independently assign to the storage engine. [Database](https://surrealdb.com/docs/learn/data-models/architecture?utm_source=chatgpt.com)

There are also two materially different ways to “use SurrealMX”:

| Access path | What you obtain |
|---|---|
| **Native Rust → SurrealMX directly** | Ordered KV access, transactional snapshots and updates, concurrency control, optional persistence |
| **Native Rust → SurrealDB → SurrealMX** | Those storage capabilities plus SurrealDB’s query language, record model, schema, permissions, indexes, relationships, and reactive mechanisms |

Direct SurrealMX is therefore a useful middle option between ordinary Rust collections and a full database interface. It can replace substantial custom synchronization machinery without requiring every operation to traverse the SurrealDB query layer. Its direct API includes transaction operations and lower-overhead point access. [GitHub](https://github.com/surrealdb/surrealmx?utm_source=chatgpt.com)

### SurrealKV and SurrealMX are not automatically two tiers of one datastore

In the documented SurrealDB model, the backend is selected when a **datastore is opened**. There is no ordinary per-table setting that routes one table to SurrealMX and another to SurrealKV. The server’s storage path selects the backend; the table definition does not. [Database](https://surrealdb.com/docs/reference/cli/db/commands/start?utm_source=chatgpt.com)

Therefore, if you use both, the normal possibilities are:

- Two SurrealDB datastores, one using each engine.
- A SurrealDB datastore plus a separate, directly accessed SurrealMX instance.
- Separate directly accessed engine instances.

**The resulting architectural implication is that those stores are separate consistency domains.** You should not assume an atomic transaction across them, automatic promotion of records from MX to KV, or automatic synchronization of duplicate records.

This matters to your complexity objective: adding a second store can reintroduce coordination work unless its responsibility is clearly bounded.

## 2. Where I would put different categories of state

The first question I would ask about any value is:

> **Does this represent a fact that other operations must agree on, or is it a temporary representation used to perform one operation?**

That distinction is usually more useful than whether the value changes frequently.

### A practical placement matrix

These are recommended defaults for your combination of technologies.

| State category | Default placement | Reason |
|---|---|---|
| Domain entities, configuration, dependency relationships, accepted results, provenance | **SurrealDB backed by SurrealKV** | Durable, queryable authority |
| Job status, attempt identity, ownership, checkpoints, completion records | **SurrealDB backed by SurrealKV** when restart recovery matters | Recovery and coordination derive from persisted state |
| Temporary shared graph, candidate set, session workspace, or intermediate dataset requiring database queries | **SurrealDB backed by SurrealMX** | Full database semantics are useful, while memory residency and a bounded lifetime are acceptable |
| Shared working state requiring atomic multi-key operations and ordered access, but no SurrealQL or graph model | **Direct SurrealMX** | Transactional state management with a smaller abstraction cost |
| Solver vectors, Arrow buffers, parsing structures, per-iteration numerical state, temporary accumulators | **Native Rust memory** | Direct typed access and efficient algorithm-specific layout |
| Futures, task handles, channels, cancellation tokens, sockets, file descriptors, GPU allocations | **Native Rust runtime** | These are execution resources; their logical status can be represented in the database |
| Reusable immutable lookup tables or pure-function memoization | **Native cache or Surrealism module cache**, depending on the caller | Reconstructible acceleration with explicit versioning |
| Copies of authoritative data loaded for computation | **Native Rust working representation** | A revision-tagged input snapshot, rather than another independently writable authority |

A particularly useful pairing is:

- Database: “Cancellation has been requested for run 42.”
- Runtime: the actual cancellation token used by the task executing run 42.

Similarly:

- Database: “Attempt 7 owns this job.”
- Runtime: the task handle, buffer allocation, and worker executing attempt 7.

The database representation gives the system shared meaning. The runtime representation makes execution efficient.

### SurrealMX can also persist

SurrealMX is **memory-first**, not necessarily disposable. It supports snapshots and append-only logging. Its recovery guarantee depends on the chosen logging and synchronization policy, and its active dataset still needs to fit in RAM. Snapshot-only recovery, asynchronous logging, and synchronous logging with fsync have materially different failure and performance behavior. [Database](https://surrealdb.com/docs/running/in-memory?utm_source=chatgpt.com)

Thus, “must survive restart” does not mechanically imply that MX is impossible. You could deliberately choose MX as an authoritative store with appropriate durability settings.

For a system already using both engines, however, I would normally give them these responsibilities:

- **KV:** durable authoritative state.
- **MX:** a bounded working dataset whose memory residency and transactional access produce a specific benefit.

That division is easier to reason about than continuously maintaining two authoritative copies.

### Frequently read data does not automatically need an MX copy

SurrealKV already has memory-resident structures, including memtables and cached table blocks. A hot read from a KV-backed database need not perform a physical disk read. [surrealdb.com](https://surrealdb.com/static/whitepapers/surrealkv.pdf?utm_source=chatgpt.com)

Consequently, “this data is read frequently” is insufficient justification for duplicating it in MX.

A stronger justification would be:

> “This working dataset has substantial intermediate write traffic, needs shared transactional access, fits in memory, and can be rebuilt or published at explicit boundaries.”

That gives MX a distinct role and avoids creating a cache-invalidation problem merely to replicate what the persistent engine’s caches already accomplish.

## 3. Your state-management intuition is strongest at shared transitions

The complexity benefit comes from replacing scattered coordination rules with explicit state and transitions.

For example, application coordination can otherwise become distributed across:

- Which task currently owns a value.
- Which channel received which message.
- Whether a callback ran.
- Whether another component updated its local cache.
- Whether a process failed between two actions.

A state-oriented design can instead expose a smaller set of questions:

- Is this work eligible?
- Which attempt owns it?
- Which input revision does it use?
- Has a result been accepted?
- What must happen next?

That is particularly valuable when several workers or operations interact with the same state, or when you need inspection and recovery.

SurrealDB itself uses this pattern: its documented index-build coordination moved ownership and progress into durable catalog state so nodes sharing a storage engine could agree on build ownership, replay completion, and query eligibility. The runtime still performs the indexing work. [SurrealDB](https://surrealdb.com/releases/3.1?utm_source=chatgpt.com)

### The transition should represent meaningful work

Good candidates include:

- Claim a work item.
- Reserve a resource.
- Validate and accept a result.
- Publish a batch.
- Mark a dependency satisfied.
- Advance a workflow stage.

An individual numerical update, parser cursor movement, or temporary accumulator increment usually has no independent meaning outside its algorithm. Persisting or transactionally publishing every such update adds an interface boundary without necessarily simplifying the program.

**The useful granularity is a state change that another operation needs to observe or coordinate against.**

For a single-owner object used by one task, ordinary Rust ownership may already provide the simplest correct design. The benefits of database state grow as the need for shared consistency, queryability, and recovery grows.

### Transactions still require an explicit invariant

There is a relevant difference between the native engines:

- **SurrealMX 1.0:** native update transactions default to strict serializability, tracking reads and writes.
- **SurrealKV 1.0:** snapshot isolation with write-conflict detection; selected reads can be tracked with `get_for_update`, but ordinary reads and scans do not provide general serializable behavior. [GitHub](https://github.com/surrealdb/surrealmx?utm_source=chatgpt.com)

Through SurrealDB, use the guarantees exposed and selected by the database integration. The underlying engine’s strongest available mode does not establish which mode every higher-level operation uses.

For instance, “claim this specific job if it is still available” can be implemented around an atomic transition on that job. “Never allow more than ten jobs across these records” requires a deliberately designed shared invariant—such as a capacity record or another suitable constraint mechanism.

Database-managed state reduces coordination code when the transactional boundary actually covers the invariant.

## 4. What is the performance hit?

There is no defensible single percentage for “state management versus runtime coordination.” There are several different comparisons.

### 4.1 The costs accumulate at different boundaries

| Comparison | Additional work that may be introduced |
|---|---|
| Ordinary Rust objects → direct SurrealMX | Key/value representation, lookup, MVCC, transactional bookkeeping, commit validation, conflict retries |
| Direct SurrealMX → SurrealDB over SurrealMX | Request dispatch, statement execution, query planning/evaluation, value conversion, schema and permission checks, index maintenance, result construction |
| Embedded SurrealDB → separate database server | Transport, wire encoding, network round trips, connection and request queues |
| Memory-only state → durable state | Logging, synchronization according to policy, recovery metadata, and background storage maintenance |
| Native Rust function → Surrealism function | WASM invocation, argument/result transfer, host callbacks, and potentially controller acquisition or initialization |

The query-layer responsibilities are documented separately from the storage interface. Embedded SurrealDB removes the database network boundary, while retaining database execution and conversion work. [Database](https://surrealdb.com/docs/learn/data-models/architecture?utm_source=chatgpt.com)

This explains why an extremely fast storage engine does not make a database operation equivalent to accessing a field in a Rust struct.

However, whole-program performance can still improve. A database operation may replace repeated reads, several round trips, duplicated searches, or synchronization across multiple application components. The comparison must include the work eliminated as well as the work added.

### 4.2 What the SurrealMX benchmark establishes

The upstream SurrealMX 1.0 benchmark reports approximately:

| Operation | Published aggregate throughput |
|---|---:|
| Point reads | **21.5 million/s** |
| Creates | **1.89 million/s** |
| Updates | **1.60 million/s** |

These figures come from a 32-core Threadripper system, 15 million keys, 48 worker threads, and 128 clients, using the best of three runs. They are storage-engine measurements. [SurrealDB](https://surrealdb.com/blog/introducing-surrealmx-surrealdbs-new-in-memory-storage-engine?utm_source=chatgpt.com)

They support your impression that MX can be a very fast transactional foundation.

They do **not** establish:

- A per-SurrealQL-operation latency.
- The overhead compared with your native Rust implementation.
- The throughput of a complete state transition involving multiple records and indexes.
- The cost of executing Surrealism functions.
- Tail latency under your application’s contention pattern.

The point-read benchmark specifically uses borrowed-value inspection through `with_value`. A query returning typed records through SurrealDB performs additional work. [GitHub](https://github.com/surrealdb/surrealmx?utm_source=chatgpt.com)

Also, dividing one second by 21.5 million gives reciprocal aggregate throughput—not the latency experienced by one request in a concurrent benchmark.

### 4.3 Backend speed only accelerates the backend portion

As a hypothetical illustration, suppose an operation spends:

- 20% of its time in storage access.
- 80% in query execution, conversion, application computation, and other work.

Making storage ten times faster changes total time to:

\[
0.80 + \frac{0.20}{10} = 0.82
\]

That produces about a **1.22× overall speedup**.

This is an illustrative calculation, not a measurement of SurrealDB. It shows why the question should be “what fraction of my operation is actually storage work?” before choosing a second engine for performance.

### 4.4 Operation granularity can dominate the result

Suppose a computation performs 100,000 internal updates.

Two designs can both use authoritative database state:

| Design | Database interaction |
|---|---|
| Publish every internal update | Approximately 100,000 database operations |
| Claim work, obtain inputs, compute locally, publish result | A small number of meaningful database operations |

The latter retains database-managed ownership and completion semantics while keeping the hot loop in native Rust.

A useful approximate model is:

\[
T_{\text{job}}
\approx
T_{\text{computation}}
+
N_{\text{boundaries}}T_{\text{boundary}}
+
T_{\text{maintenance}}
+
T_{\text{waiting/retries}}
\]

Batching reduces the number of crossings. Choosing a coherent transaction boundary also reduces the number of times the system must establish and validate shared state.

Batching has limits: excessively large transactions hold resources longer and create more opportunities for conflict. The target is a bounded unit of useful work.

### 4.5 Durability policy is part of the performance comparison

SurrealKV’s standalone API defaults to `Durability::Eventual`; `Durability::Immediate` waits for synchronization of the write-ahead log before returning. A database integration can select its own policy. [GitHub](https://github.com/surrealdb/surrealkv?utm_source=chatgpt.com)

SurrealKV’s group commit can amortize synchronization across concurrent transactions, but synchronization remains real work. SurrealMX also incurs logging and synchronization costs when configured for corresponding durability. [surrealdb.com](https://surrealdb.com/static/whitepapers/surrealkv.pdf?utm_source=chatgpt.com)

Therefore, compare either:

- Two paths with the required durability guarantee; or
- Two deliberately different guarantees, with the performance tradeoff made explicit.

Otherwise, an apparent engine speedup may largely reflect acknowledging work before it is durable.

## 5. Where Surrealism fits—and the additional state boundary it introduces

Surrealism is valuable when a custom calculation or rule belongs naturally inside a database operation.

Examples include:

- Specialized validation that needs existing records.
- Domain-specific normalization.
- A bounded calculation used to update related records.
- A reusable algorithm that multiple queries should invoke consistently.

Its major integration benefit is that **database callbacks from a module run within the invoking transaction**. The module’s database writes and the originating query can therefore commit or roll back together. [SurrealDB](https://surrealdb.com/surrealdb/extensions?utm_source=chatgpt.com)

That is a substantial architectural capability: it can eliminate application coordination around a multi-step, data-dependent transition.

### 5.1 Surrealism is a distinct execution environment

The Rust code runs as a WASI component. It does not directly borrow arbitrary Rust objects from the surrounding process or link into SurrealDB as an ordinary native function. Values cross an explicit interface. [Database](https://surrealdb.com/docs/learn/extensions/guides/module-architecture?utm_source=chatgpt.com)

The host implementation decodes FlatBuffers inputs, invokes database functionality, and encodes results. Consequently, a database query issued from a Surrealism function still performs database work and transfers values across the host–guest boundary. [source](https://docs.rs/surrealism-runtime/latest/src/surrealism_runtime/host.rs.html?utm_source=chatgpt.com)

My placement recommendation is therefore:

- Favor **bounded, substantial calls** that perform useful work per invocation.
- Fetch or process inputs in batches when that preserves the intended semantics.
- Keep ordinary internal algorithm calls inside the module.
- Avoid turning a per-record loop into a sequence of host queries when a bulk query can obtain the needed inputs.

For an embedded Rust application, there may be no network round trip for Surrealism to eliminate. Its strongest justification then becomes transactional integration, shared database behavior, or module deployment—not physical proximity alone.

### 5.2 There are three different kinds of state inside this arrangement

| Location | Lifetime and semantics | Appropriate use |
|---|---|---|
| **SurrealDB records** | Governed by the database transaction and selected storage backend | Authoritative facts and coordinated transitions |
| **Surrealism module KV** | Shared across invocations of a loaded module runtime; volatile | Reconstructible module cache |
| **WASM instance memory** | Belongs to a particular pooled controller; may survive warm reuse | Initialized resources, immutable lookup structures, scratch memory |

The module KV implementation is specifically an in-memory `BTreeMap` protected by an `RwLock`. **It is not SurrealMX**, and it is lost on restart, module eviction, or runtime destruction. [Rust](https://docs.rs/surrealism-runtime/latest/surrealism_runtime/kv/index.html?utm_source=chatgpt.com)

Controllers have separate WASM instances. Pooling can preserve an instance’s memory for reuse, but another concurrent invocation can use a different controller with different instance state. [Rust](https://docs.rs/surrealism-runtime/latest/surrealism_runtime/runtime/struct.Runtime.html?utm_source=chatgpt.com)

### 5.3 Module cache changes do not participate in database rollback

The module KV implementation updates its map directly. It does not join the database’s transaction. From that implementation, the architectural consequences are:

- A cache update can survive a subsequently failed database transaction.
- A value derived from uncommitted records can outlive those records’ rollback.
- A separate `get` followed by `set` is not automatically one atomic read-modify-write operation. [source](https://docs.rs/surrealism-runtime/latest/src/surrealism_runtime/kv.rs.html?utm_source=chatgpt.com)

I would use that cache for immutable resources or memoization keyed by input identity and version. I would keep work ownership, quotas, completion, and deduplication decisions in transactional records.

For code reuse, a useful design is a portable pure-Rust domain crate used by both the native application and the Surrealism module. That lets you choose execution placement without maintaining two versions of the algorithm.

## 6. Which operations should occur where?

The following is the execution policy I would start with.

| Operation | Preferred execution location | Decision criterion |
|---|---|---|
| Indexed filtering, graph traversal, relationship lookup, supported aggregation | **SurrealDB query layer** | Let the database operate on its own access structures |
| Schema checks and straightforward record constraints | **SurrealDB schema/query definitions** | Keep the rule attached to the shared data contract |
| Claiming work, reserving capacity, accepting completion | **A short database transaction** | All participants must agree on the transition |
| Custom bounded Rust rule participating in that transition | **Surrealism** | The rule benefits from the same database transaction |
| Maintaining a small related record or invariant after a write | **Synchronous database event**, where appropriate | Its cost and failure should belong to the original transaction |
| Deferred database-side work | **Asynchronous event**, where its timing and retry semantics fit | Completion need not be atomic with the triggering write |
| Large numerical computation, repeated parsing, tensor operations, GPU work | **Native Rust execution** | Work benefits from native libraries, data layout, or independent resource control |
| Temporary shared ordered state with transactional updates | **Direct SurrealMX** if full database semantics are unnecessary | The KV abstraction itself satisfies the requirement |
| Repeated queries over a temporary structured working dataset | **SurrealDB over SurrealMX** | Queryability and transactional relationships justify the higher layer |

These are starting rules, not absolute size thresholds. An expensive computation can sometimes belong close to the data if it avoids transferring a much larger dataset. A small computation can belong in native Rust if it is invoked millions of times over already-local buffers.

### Reactive execution has its own latency and reliability semantics

Synchronous events execute inside the originating transaction, adding their work to its latency. Asynchronous events are enqueued with the originating change but execute later in a separate transaction, with at-least-once behavior.

A particularly relevant detail is that the documented asynchronous-event processing interval defaults to **five seconds**, and is configurable. A faster storage engine does not remove that scheduling delay. External side effects also need to tolerate retries. [Database](https://surrealdb.com/docs/reference/query-language/statements/define/event?utm_source=chatgpt.com)

Similarly, live queries provide change notifications; changefeeds provide retained changes for replay and catch-up. [Database](https://surrealdb.com/docs/learn/querying/real-time/live-queries?utm_source=chatgpt.com)

For a worker system, my recommendation would be to treat a notification as a prompt to inspect authoritative work state. Recovery should be possible from that state even if the notification was missed.

## 7. An architecture that preserves the complexity advantage

Consider a computation run: it might characterize a library feature, process a dataset, or solve a model.

I would organize it around the following sequence.

### A. Claim work and identify its inputs

In a short transaction:

- Verify eligibility.
- Record an attempt identity and ownership.
- Record the input revision being used.
- Establish any resource reservation needed for correctness.

The database now holds a concrete statement of what is being attempted.

### B. Obtain a consistent working input

Read the required input data and materialize the native representation needed for computation.

End the database transaction once the necessary input snapshot has been acquired, unless the operation genuinely requires it to remain open.

For MVCC systems, unnecessarily long-lived snapshots can retain historical versions while other operations continue writing. This is one reason to avoid keeping a database transaction open for the entire duration of a long computation. The attached SurrealMX reference describes this retention and reclamation behavior in detail.

### C. Execute in native Rust

Keep these local:

- Algorithm workspaces.
- Temporary arrays and buffers.
- Native library handles.
- Per-iteration state.
- Task scheduling and cancellation machinery.

Publish progress at the frequency required by observers or recovery requirements. A checkpoint is useful when it defines a recovery point; an internal variable update need not be a checkpoint.

### D. Use MX only where it earns its role

If the computation needs a shared transactional working dataset, use MX for that dataset.

Examples might include:

- A temporary candidate graph queried by several stages.
- Shared intermediate records updated by multiple workers.
- A session workspace with independent query and inspection needs.

If one task owns a vector throughout a calculation, ordinary Rust memory may already satisfy its requirements with less complexity.

### E. Validate and publish the result

In another short transaction:

- Check that the attempt remains authorized to publish.
- Check that the result’s input revision is still acceptable.
- Store or reference the result.
- Mark the work complete.
- Apply the relevant dependency or workflow transition.

This prevents a stale worker from overwriting a newer accepted state.

The long computation happened outside the transaction, while **the decision to accept its outcome remains transactional**.

### One fact can have several representations

Under this design, related data may legitimately exist in all three places:

| Representation | Role |
|---|---|
| KV-backed record | Authoritative configuration, revision, and accepted outcome |
| MX working dataset | Temporary shared state derived for the run |
| Native Rust objects | Execution representation used by the algorithm |

That is manageable when the direction of authority is explicit: inputs are read at a revision, temporary state is derived, and results are accepted through a defined publication transition.

The complexity grows when all three representations can independently change the same authoritative fact and must continually reconcile with one another.

## 8. How I would determine the actual performance crossover

The missing measurement is **latency and resource cost per meaningful application transition**, not raw KV operations per second.

I would compare a small set of representative operations—such as claiming work, obtaining inputs, and publishing a result—through the relevant paths:

| Variant | What it isolates |
|---|---|
| Native Rust implementation with the required synchronization | Existing runtime cost |
| Direct SurrealMX 1.0 | Cost of transactional KV state |
| Embedded SurrealDB over MX | Additional cost and benefit of database semantics |
| Embedded SurrealDB over KV at the required durability | Persistent-storage contribution |
| Surrealism versus native execution of the same bounded algorithm | Execution placement and data-transfer costs |
| Server access, if used | Transport and request-queue contribution |

Keep the consistency requirements, result shape, and workload equivalent. A native implementation that omits recovery or atomic multi-record behavior is doing less work than the database version.

Measure:

- Median and tail latency, especially p95/p99.
- Sustained throughput at realistic concurrency.
- CPU and peak resident memory.
- Conflict and retry rates.
- Bytes materialized or converted.
- Warm versus cold behavior.
- Sensitivity to batch size and contention on popular records.

For database queries, `EXPLAIN ANALYZE` can expose the executed operators and their metrics, helping distinguish storage access from scans, filtering, aggregation, and result construction. [Database](https://surrealdb.com/docs/reference/query-language/statements/explain?utm_source=chatgpt.com)

I have not benchmarked your application, so I would not assign a universal slowdown factor. **My initial design choice would be to centralize shared operational state and its invariants in SurrealDB, keep computational working memory native, and give MX a specific transactional working-set role.** Then I would optimize the measured boundaries—particularly repeated small calls, unnecessary data conversion, long transactions, and duplicated state—before moving more authority back into custom runtime coordination.