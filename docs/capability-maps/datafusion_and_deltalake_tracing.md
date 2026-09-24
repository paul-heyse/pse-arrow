Yes. I think your intuition is directionally correct, but I would phrase it more narrowly:

> In a codebase whose semantics and execution are heavily delegated to DataFusion abstractions, `datafusion-tracing` can displace a large amount of **bespoke diagnostic validation, execution-plumbing verification, and implementation-detail testing**, because it exposes what DataFusion actually planned and executed from first principles.

I would **not** say that it can broadly replace correctness testing or runtime validation.

The reason this becomes especially powerful in the architecture you are describing is that DataFusion is no longer merely a compute engine sitting behind your code. If `TableProvider`, schemas, logical expressions, optimizer rules, `ExecutionPlan`s, catalogs, configuration extensions, etc. are your architectural primitives, then a surprisingly large fraction of your application's behavior becomes visible through the DataFusion execution model itself. `datafusion-tracing` instruments physical-plan nodes, native metrics, intermediate result previews, analyzer/optimizer phases, effective rules, plan diffs, and physical-plan creation. 

That changes the economics of validation quite significantly.

### The important distinction

I would divide code that people colloquially call "validation/testing" into four categories:

| Category | Can tracing displace it? | Why |
|---|---:|---|
| Hand-written logging / diagnostic instrumentation | **Very strongly** | DataFusion-tracing already exposes plans, operators, metrics, transformations and data flow |
| Tests that verify execution plumbing / implementation details | **Often strongly** | Much of the evidence is directly observable in plans and spans |
| Architectural contract / invariant tests | **Partially** | Fewer tests can be concentrated at abstraction boundaries |
| Semantic correctness / safety validation | **No** | Observation is evidence, not an assertion that behavior is correct |

That last distinction matters.

A trace can tell you:

```text
Filter
   ↓ rows: 1,000,000 → 8,214

HashJoin
   left rows: 8,214
   right rows: 4,018
   output rows: 7,991

Aggregate
   output rows: 43
```

and perhaps even show sampled intermediate rows.

That is extraordinary evidence about what happened.

But it does not itself establish:

```text
expected_rows == actual_rows
```

unless something turns that trace information into an assertion.

---

# Why the effect is unusually large in a highly DataFusion-native architecture

Suppose an ordinary application implements its own:

```text
business operation
    ↓
custom validation
    ↓
custom transformation
    ↓
custom batching
    ↓
custom filtering
    ↓
custom joining
    ↓
custom metrics
    ↓
custom logging
```

Each layer tends to acquire bespoke tests and instrumentation.

Now compare an architecture that deliberately expresses most of those behaviors as:

```text
TableProvider
      ↓
LogicalPlan / Expr
      ↓
Analyzer
      ↓
Logical Optimizer
      ↓
Physical Planner
      ↓
Physical Optimizer
      ↓
ExecutionPlan
      ↓
Arrow RecordBatch
```

Your application code has effectively pushed much of its operational semantics into a **single structured execution system**.

And `datafusion-tracing` sits across that execution system.

It can see the physical operators automatically because its instrumentation optimizer rule wraps each execution-plan node. It can collect the metrics DataFusion operators already expose, rather than requiring parallel metrics implementations. 

That gives you something architecturally valuable:

> **One instrumentation mechanism acquires observability over many abstractions simultaneously.**

That is where the large code reduction can come from.

---

# 1. A lot of production "validation code" is really observability code

Consider code of this kind:

```rust
debug!("starting customer filter");

let before = batch.num_rows();

let result = filter(...)?;

let after = result.num_rows();

debug!(
    before_rows = before,
    after_rows = after,
    "customer filter completed"
);
```

Or:

```rust
assert_some_runtime_condition_about_join(...);
record_join_metrics(...);
log_join_inputs(...);
log_join_outputs(...);
```

Or custom instrumentation answering:

```text
Was predicate pushdown applied?
Did projection pruning happen?
Which source was scanned?
How many rows entered the join?
How many rows exited?
Did this spill?
How much memory did it consume?
Which physical operator was chosen?
```

A DataFusion-native architecture makes many of these questions directly observable.

DataFusion itself exposes per-operator runtime metrics such as output rows, bytes and compute time, and `EXPLAIN ANALYZE` already surfaces them. 

`datafusion-tracing` extends that into a hierarchical trace and can add intermediate-result previews and custom context. 

So yes: **do not recreate this instrumentation at every abstraction level.**

---

# 2. It can eliminate a lot of "did our plumbing actually work?" tests

A very common integration test amounts to:

```text
We configured feature A.

Did that eventually cause mechanism B?

Did B result in operator C?

Did C receive the appropriate data?

Did C actually execute?
```

Without introspection, developers often build explicit test seams just to establish those facts.

With your proposed architecture, the trace itself can expose:

```text
Logical plan
    ↓
optimizer transformation
    ↓
physical plan
    ↓
ExecutionPlan operator
    ↓
operator metrics
    ↓
output sample
```

The rule-level instrumentation is especially relevant here because it can identify optimizer rules that actually modified the plan and optionally record plan diffs. 

Therefore something that might otherwise require:

```rust
#[test]
fn filter_propagates_to_x() { ... }

#[test]
fn x_constructs_y() { ... }

#[test]
fn y_invokes_z() { ... }

#[test]
fn z_receives_filtered_values() { ... }
```

can sometimes be collapsed into:

```text
one end-to-end contract test
+
trace inspection / trace assertions
```

provided those intermediate behaviors aren't themselves contractual requirements.

That is a major reduction in test surface.

---

# 3. More importantly, it lets you test abstractions instead of plumbing

This is where I think your architectural idea becomes strongest.

Imagine you standardize a concept around a `TableProvider`.

Rather than every implementation having separate tests for:

```text
projection
filter handling
schema propagation
execution construction
statistics
metrics
logging
```

you can establish a strong contract around that abstraction and exercise it end-to-end through DataFusion.

Then tracing supplies the **explanatory evidence** for failures.

So your test suite can move toward:

```text
                  ┌─────────────────────┐
                  │ abstraction contract │
                  └──────────┬──────────┘
                             │
                         few tests
                             │
                 execute through DataFusion
                             │
          ┌──────────────────┴──────────────────┐
          │                                     │
     expected output                       trace evidence
                                                   │
                           logical → physical → runtime
```

rather than having unit tests for every piece of glue between those stages.

This is often a substantially better testing architecture.

---

# 4. A particularly compelling model is "thin assertions, rich evidence"

Rather than writing lots of code to reconstruct what happened, tests can assert only what is actually a contract.

For example:

```rust
assert_eq!(result, expected);
```

while the automatically captured trace tells you *why* it failed:

```text
Analyzer
  ✓ ...

Optimizer
  PushDownFilter [modified]
    plan diff: ...

PhysicalPlanner
  ...

DataSourceExec
  output_rows=100000
  ...

FilterExec
  input_rows=100000
  output_rows=217

HashJoinExec
  ...
```

That is a much healthier separation:

**Tests answer**

> Is the behavior correct?

**Tracing answers**

> What actually happened?

The mistake in many codebases is making the test suite answer both questions by manually probing every intermediate stage.

That creates brittle tests.

---

# 5. Traces can themselves become test inputs

You can go one step farther than simply *looking* at traces.

Because Rust `tracing` is structured, a test subscriber/layer can capture span fields and events programmatically.

You can therefore create tests around higher-level execution invariants such as:

```text
scan exists
AND
filter exists downstream
AND
scan rows > filter rows
AND
no spill occurred
```

or:

```text
optimizer effective rules includes PushDownFilter
```

or:

```text
physical plan contains no repartition
```

That starts turning `datafusion-tracing` into an **architectural testing surface**.

I would, however, be conservative here.

Instead of asserting an entire trace:

```text
A → B → C → D → E → F
```

assert something semantically meaningful:

```text
predicate reached storage
```

because exact optimizer plans legitimately change across DataFusion releases.

DataFusion's own documentation makes a similar distinction: it maintains explicit regression testing such as `sqllogictest`, while its explain/metrics infrastructure exists to expose execution behavior. 

---

# What I would actively remove

In the architecture you're describing, I would be aggressive about removing duplicated code whose primary purpose is:

- recording row counts around operations;
- recording operator timing;
- maintaining separate execution-stage metrics;
- printing plans manually;
- logging logical-to-physical transitions;
- reporting which optimizer transformations fired;
- dumping intermediate batches for debugging;
- maintaining bespoke "execution breadcrumb" state;
- validating that ordinary DataFusion machinery was invoked when that fact is directly observable;
- highly granular integration tests whose sole purpose is proving glue code connected A → B → C.

`datafusion-tracing` is specifically designed to expose execution steps, native DataFusion metrics and intermediate result previews through standard tracing/OpenTelemetry infrastructure. 

That is exactly the kind of redundant code you should try to centralize away.

---

# What I would absolutely keep

There is another category of "validation" that tracing should **not** replace.

For example:

```text
schema invariants
domain constraints
authorization checks
transactional guarantees
required nullability constraints
uniqueness assumptions
overflow / numeric correctness
failure semantics
retry/idempotence guarantees
concurrency guarantees
storage durability
correct results for corner cases
```

These are properties that need to be **enforced or asserted**, not merely observed.

For example, seeing:

```text
HashJoinExec output_rows = 200
```

does not tell you whether the correct answer was 200 or 199.

Similarly, seeing:

```text
FilterExec selectivity = 0.043
```

doesn't prove the filter predicate has correct semantics.

That requires a correctness oracle:

```text
expected == actual
```

So tracing cannot replace that class of testing.

---

# There is an even larger architectural implication

What you're describing ultimately isn't really about `datafusion-tracing`.

It's about **reducing the number of independently implemented semantic layers in the system**.

If you make DataFusion canonical for:

```text
schema
        ↓
data contracts

Expr
        ↓
computations / predicates

LogicalPlan
        ↓
relational semantics

optimizer
        ↓
transformation

ExecutionPlan
        ↓
execution

Metrics
        ↓
runtime introspection

datafusion-tracing
        ↓
observability
```

then application code increasingly stops maintaining parallel representations such as:

```text
our schema representation
our operation representation
our dependency graph
our optimizer representation
our execution-state representation
our metric representation
our debugging representation
```

That reduction is much more important than simply deleting tests.

It gives you **one source of operational truth**.

And then tracing lets you interrogate that source of truth.

---

# So I would revise your statement slightly

Instead of:

> "`datafusion-tracing` can displace a large volume of production validation and testing code."

I would use:

> **In a deeply DataFusion-native architecture, `datafusion-tracing` can displace a large volume of bespoke instrumentation, diagnostic validation, execution-plumbing checks, and implementation-detail tests by making DataFusion's own planning, optimization, execution, metrics, and intermediate data flow directly observable. This enables the remaining test suite to become substantially smaller and more contract-oriented: tests establish semantic correctness and architectural invariants, while tracing supplies first-principles evidence of how those invariants were realized.**

I think **that is a very defensible architectural claim**.

And there is a second-order effect that may actually be more valuable than raw test-count reduction: **failure diagnosis becomes dramatically richer while the tests themselves become simpler**.

Instead of maintaining 10 intermediate assertions so that a failing test tells you where execution went wrong, you can maintain one or two genuinely important assertions and use a complete execution trace to diagnose the failure. That generally produces a codebase that is both **less brittle and more observable**. 

The architecture I would target is therefore approximately:

```text
          DATAFUSION ABSTRACTIONS
 schemas / providers / expressions / plans
                    │
                    │ canonical behavior
                    ▼
              DATAFUSION ENGINE
                    │
           ┌────────┴────────┐
           ▼                 ▼
    CONTRACT TESTS     DATAFUSION-TRACING
           │                 │
   "is it correct?"    "what happened?"
           │                 │
           └────────┬────────┘
                    ▼
             compact evidence-rich
                validation layer
```

That is substantially different from trying to replace tests with logs. It is **moving behavioral introspection into the same abstraction/execution system that owns the behavior**, which is exactly why the potential code reduction is so large.

Yes. I did a source-level inventory against current delta-rs `main`, at commit `6a954aff7503103f9cc7a9c7878fe330ec57e080` from September 18, 2026. The main conclusion is that **delta-rs already has a fairly strong tracing substrate, but it is not yet systematic enough to serve as the complete observability architecture by itself**.

The right model is:

```text
delta-rs native tracing
        +
datafusion-tracing 55
        +
optional object_store instrumentation
        +
a very thin application-owned root-span/metrics layer
```

That combination can give you essentially end-to-end observability from a Delta operation, through file pruning and DataFusion planning/execution, through Parquet/object-store I/O, and finally through Delta commit/conflict handling.

## 1. What delta-rs provides natively

The Rust core has an unconditional dependency on `tracing`; it is not hidden behind a tracing feature flag. The library emits spans and structured events but generally leaves **subscriber/exporter configuration to the application**.

The Python binding adds a turnkey OpenTelemetry layer and explicitly documents tracing as supported functionality. 

The important architectural distinction is:

| Capability | delta-rs |
|---|---|
| Rust `tracing` spans | Yes |
| Structured span fields | Yes |
| Dynamic span field recording | Yes |
| `trace!` / `debug!` / `info!` / `warn!` / `error!` events | Extensive |
| Span propagation into blocking workers | Yes |
| OpenTelemetry-aware Python initializer | Yes |
| Built-in Rust OTLP initializer | No; application owns subscriber |
| Automatic tracing of every Delta operation | No |
| Automatic tracing of every object-store request | No |
| Automatic attachment of all Delta operation metrics to spans | No |
| Automatic DataFusion node tracing | No, but cleanly composable with `datafusion-tracing` |

---

# 2. Native structured-span inventory

These are the important **actual spans**, rather than ordinary logging calls.

| Layer | Span/function | Important fields | What it gives you |
|---|---|---|---|
| Write | `write_operation` | `operation="write"`, `mode`, `table_uri` | Top-level write execution |
| Update | `execute` | `operation="update"`, `version`, `table_uri` | Update execution |
| Delete | `execute` | `operation="delete"`, `version`, `table_uri` | Delete execution |
| Merge | `execute` | `operation="merge"`, `version`, `table_uri` | Merge execution |
| Optimize | `execute` | `operation="optimize"`, `version` | Optimize execution |
| Optimize planning | `create_merge_plan` | `operation="create_merge_plan"`, `version` | Compaction/Z-order planning |
| Candidate discovery | `find_files` | `version`, `has_predicate`, `partition_scan`, `candidate_count` | Whether DML used partition-only pruning and how many files survived |
| Record-level file matching | `find_files_scan` | `version`, `total_files`, `matching_files` | Exact file reduction before DML rewrite |
| Transaction | `commit_with_retries` | `base_version`, `max_retries`, `attempt`, `target_version`, `conflicts_checked` | Excellent transaction/concurrency visibility |
| Delta log read | `read_commit_entry` | `version`, `path` | Transaction-log read latency |
| Delta log write | `write_commit_entry` | `version`, `tmp_path`, `commit_path` | Commit-file publication |
| Delta log abort | `abort_commit_entry` | `version`, `tmp_path` | Failed commit cleanup |
| Storage retry | `put_with_retries` | `path`, `size` | Retried writes |
| Storage retry | `delete_with_retries` | `path` | Retried deletes |
| Parquet output | `upload_parquet_file` | function args incl. path, dynamically recorded `rows`, `size` | Individual output-file production |
| JSON writer | `flush` | dynamically recorded `batch_count` | Writer flush windows |
| Checkpoint | `create_checkpoint_for` | `operation="checkpoint"`, `version`, `table_uri` | Delta checkpoint creation |
| Log compaction | `compact_logs_for` | `operation="log_compaction"`, `start_version`, `end_version`, `table_uri` | Transaction-log compaction |
| Vacuum listing | `list_files_parallel` | `operation="vacuum"`, `partition_depth`, `scan_concurrency` | Parallel vacuum scan |
| Vacuum listing | `list_files` | `operation="vacuum"`, optionally `mode`, `prefix` | Per-prefix/full listing |
| Vacuum traversal | `list_with_delimiter` | `operation="vacuum"`, `level`, `prefix` | Partition tree traversal |
| FSCK listing | `list_files` | `operation="filesystem_check"` | File enumeration during filesystem check |

The file-pruning instrumentation is particularly useful. `find_files` records whether it used the cheaper partition-only route and how many candidate files survived; `find_files_scan` separately records total and matching files. 

That means a trace of an `UPDATE`, `DELETE`, or `MERGE` can already tell you:

```text
execute { operation="merge", version=418 }
└── find_files
    ├── has_predicate=true
    ├── partition_scan=false
    └── candidate_count=17
        └── find_files_scan
            ├── total_files=12,483
            └── matching_files=17
```

This is a very valuable boundary between **Delta semantic pruning** and the subsequent DataFusion work.

---

# 3. Transaction tracing is one of the strongest areas

`commit_with_retries` deserves particular attention. It is much richer than a simple timing span.

It starts with:

```text
base_version
max_retries
attempt
target_version
conflicts_checked
```

and dynamically updates `attempt`, `target_version`, and `conflicts_checked`. It also emits structured warning/error/info events as concurrency is encountered. 

Successful commits emit `version` and `num_retries`; failed/conflicting transactions record relevant error information. 

So you can obtain a trace resembling:

```text
delta.merge
├── ...
└── commit_with_retries
    ├── base_version = 1027
    ├── max_retries = 10
    ├── attempt = 3
    ├── target_version = 1030
    └── conflicts_checked = 2
        ├── WARN table updated during transaction
        ├── DEBUG all conflicts resolved
        └── INFO transaction committed successfully
```

For diagnosing optimistic-concurrency overhead, that is already close to what I would design from scratch.

---

# 4. Span propagation across worker boundaries

This is another strong piece of the implementation.

delta-rs has helpers that explicitly capture both:

```rust
dispatcher::get_default(...)
Span::current()
```

before spawning blocking work, then restore the dispatcher and enter the captured span inside the blocking task.

This occurs both in the kernel's `spawn_blocking_with_span()` helper and in its `ReceiverStreamBuilder::spawn_blocking()` path.

That matters because otherwise a trace such as:

```text
checkpoint
  └── snapshot reconstruction
```

could lose its parent when snapshot processing is moved to a blocking pool.

So the library isn't merely emitting spans; some thought has gone into preserving causal trace context across concurrency boundaries.

---

# 5. The event layer is much broader than the span layer

Below those structured spans is extensive use of standard `tracing` events.

Examples include transaction conflicts and retries, snapshot serialization/deserialization decisions, stats-projection diagnostics, DataFusion session fallback warnings, predicate/pruning diagnostics, schema-conversion warnings, multipart-upload failures, commit-log errors, and numerous scan/planning debug events.

I would classify them as:

```text
Spans
    durable performance + causal structure

Events
    diagnostic detail occurring inside those spans
```

This is the right distinction. I would not try to promote every existing delta-rs event into another span.

---

# 6. Delta operation metrics are richer than the tracing fields

This is probably the largest opportunity for a very thin improvement layer.

delta-rs already computes rich operation metrics, but **does not systematically attach them to its operation spans**.

For example:

| Operation | Existing metrics |
|---|---|
| Write | added files, removed files, partitions, added rows, execution time, commit retries |
| Update | added/removed files, updated rows, copied rows, execution time, scan time |
| Delete | added/removed files, deleted rows, copied rows, execution time, scan time, rewrite time |
| Merge | source rows; inserted/updated/deleted/copied/output rows; scanned/skipped files; added/removed files; execution/scan/rewrite time |
| Optimize | files added/removed, detailed file-size metrics, partitions optimized, batches, considered/skipped files, planner strategy, stable ordering, max bin span |
| Commit | retries, whether checkpoint was created, log files cleaned |
| Vacuum | dry run and deleted files; additional start/end transaction-log metrics |
| FSCK | dry-run state and removed files |
| Restore | removed/restored files |

Merge is particularly detailed; the public documentation already encourages using its scanned/skipped-file counts and execution time for performance analysis. 

The ideal systematic wrapper is therefore not to recompute anything. It should simply bridge:

```text
Delta operation result metrics
            ↓
trace completion attributes/event
```

For example:

```text
delta.operation.completed
    delta.operation = merge
    delta.num_source_rows = 50000
    delta.num_target_files_scanned = 17
    delta.num_target_files_skipped = 12466
    delta.num_output_rows = 8132
    delta.scan_time_ms = 184
    delta.rewrite_time_ms = 627
    delta.execution_time_ms = 941
```

A subtle Rust `tracing` consideration: fields recorded onto a span generally need to have been declared when the span was created. For a generic wrapper, I would therefore use a stable set of common span fields and emit a **structured completion event for operation-specific metrics**, rather than creating an enormous root span schema.

---

# 7. Python OpenTelemetry support

Python gets a more turnkey implementation.

`init_tracing()` currently establishes:

```text
OTLP HTTP exporter
service.name = "delta-rs"
batch exporter
RandomIdGenerator
Sampler::AlwaysOn
tracing-opentelemetry layer
EnvFilter from RUST_LOG
```

and supports `OTEL_EXPORTER_OTLP_ENDPOINT` and `OTEL_EXPORTER_OTLP_HEADERS`. 

There are two implementation caveats I would account for rather than model your Rust architecture after this helper.

First, initialization uses:

```rust
tracing_subscriber::registry()
    .with(filter)
    .with(telemetry)
    .try_init()
```

and silently ignores failure if another global subscriber is already installed. 

In a larger Rust application, you don't want different libraries competing to initialize the global subscriber. Your application should construct **one subscriber tree** containing the formatting, filtering, OpenTelemetry, and other layers.

Second, although the Python `shutdown_tracing()` documentation says it flushes remaining spans, the current implementation simply calls `global::tracer_provider()` and does not explicitly call provider shutdown/force-flush. 

I therefore would not rely on that function as evidence of guaranteed exporter flushing.

---

# 8. Important coverage gaps

There are several places where I would add standardized instrumentation.

I found **no consistent operation-root span** in current core source for things such as snapshot/table opening and refresh, `load_cdf`, restore, `convert_to_delta`, generate, many constraints/metadata/property operations, or the complete vacuum/FSCK operation. Vacuum and FSCK have useful inner listing spans, but not a consistent root comparable to merge/write.

Snapshot internals have diagnostic tracing events, but they are not decomposed into systematic performance spans for things such as:

```text
checkpoint discovery
checkpoint read
JSON log discovery
log replay
protocol/metadata extraction
Add-action materialization
stats projection/materialization
snapshot cache hit / miss
```

Given your recent caching work, I think those would be especially useful.

There is also no generic delta-rs wrapper that traces **every** `ObjectStore` GET/GET_RANGE/HEAD/LIST/PUT/DELETE operation. The log store and vacuum paths instrument selected operations, but not every storage request.

Finally, field conventions are somewhat inconsistent. Some spans have `table_uri`, some do not; `operation_id` usually isn't exposed as a span field; several operation spans are literally named `execute` and depend on an `operation` field to disambiguate them.

So I would regard the current schema as useful instrumentation, but not yet a stable semantic tracing contract.

---

# 9. DataFusion tracing fits remarkably well now

I also checked current `datafusion-contrib/datafusion-tracing` HEAD (`c05e7071...`). Its workspace is now **55.0.0**, matching the DataFusion 55 generation used by the delta-rs `main` you're targeting.

[datafusion-tracing repository](https://github.com/datafusion-contrib/datafusion-tracing?utm_source=chatgpt.com)

It contributes the complementary instrumentation delta-rs lacks:

```text
logical/analyzer rules
optimizer rules
physical optimizer rules
physical-plan creation
every ExecutionPlan node
native DataFusion metrics
optional RecordBatch previews
spawned DataFusion task context propagation
```

Importantly, its query-planner wrapper is designed to **wrap the existing `QueryPlanner` rather than replace it**. That is ideal for delta-rs because you want to retain the `DeltaPlanner`, not accidentally substitute DataFusion's default planner.

So the two libraries divide responsibility cleanly:

```text
delta-rs
    Delta semantics
    transaction semantics
    candidate-file discovery
    write / merge / optimize
    commit conflicts

datafusion-tracing
    planning
    optimizer
    ExecutionPlan tree
    operator metrics
    execution concurrency
```

---

# 10. There is also an object-store tracing layer

The same DataFusion tracing project contains `instrumented-object-store`, which wraps any `object_store::ObjectStore` and traces storage operations, paths, sizes, and errors.

[instrumented-object-store](https://github.com/datafusion-contrib/datafusion-tracing/tree/main/instrumented-object-store?utm_source=chatgpt.com)

That potentially fills another gap:

```text
Delta semantic operation
│
├── Delta file pruning
│
├── DataFusion planning/execution
│   └── DataFusion Exec nodes
│
├── ObjectStore
│   ├── GET
│   ├── GET_RANGE
│   ├── HEAD
│   ├── LIST
│   ├── PUT
│   └── DELETE
│
├── Parquet writes
└── Delta transaction commit
```

The important caveat is that merely registering an instrumented store with the DataFusion `RuntimeEnv` would instrument **DataFusion's use of that store**. For true Delta-wide coverage, the delta-rs `LogStore` also needs to use that same wrapped store. I would make that part of your store/log-store construction rather than maintain separate instrumented copies.

---

# 11. The architecture I would use in your codebase

I would make observability a sibling of the centralized caching/runtime configuration we discussed:

```text
ApplicationRuntimeFactory
│
├── CachingPolicy
│   └── RuntimeEnv / CacheManager
│
└── TracingPolicy
    ├── global tracing subscriber
    ├── OpenTelemetry exporter
    ├── Delta root-operation policy
    ├── datafusion-tracing configuration
    ├── instrumented ObjectStore policy
    └── Delta metric → tracing bridge
```

Then a `MERGE` might naturally look like:

```text
request / job
└── delta.operation
    │   operation = merge
    │   table_uri = ...
    │   read_version = 1027
    │
    ├── find_files
    │   ├── candidate_count = 17
    │   └── find_files_scan
    │       ├── total_files = 12483
    │       └── matching_files = 17
    │
    ├── datafusion.create_physical_plan
    │   ├── analyzer / optimizer spans
    │   └── ExecutionPlan
    │       ├── DataSourceExec
    │       ├── FilterExec
    │       ├── HashJoinExec
    │       └── ...
    │
    ├── object_store.get_range
    ├── object_store.get_range
    │
    ├── upload_parquet_file
    │   ├── rows = 8192
    │   └── size = ...
    │
    ├── commit_with_retries
    │   ├── base_version = 1027
    │   ├── target_version = 1029
    │   ├── attempt = 2
    │   └── conflicts_checked = 1
    │
    └── EVENT delta.operation.completed
        ├── num_target_files_scanned = 17
        ├── num_target_files_skipped = 12466
        ├── num_output_rows = ...
        ├── scan_time_ms = ...
        ├── rewrite_time_ms = ...
        └── execution_time_ms = ...
```

That gives you a **single causal trace spanning Delta semantics, relational execution, physical I/O, and transaction publication**.

### Bottom line

The current delta-rs tracing implementation is substantially useful already. I would characterize it as roughly:

**strong transactional tracing + good DML/file-pruning tracing + selected storage/write tracing + broad diagnostic events, but incomplete systematic operation coverage.**

The missing pieces do **not** justify creating a parallel tracing framework. A relatively small application-owned standardization layer—combined with `datafusion-tracing` 55 and optionally `instrumented-object-store`—should be enough to turn what already exists into a comprehensive observability system.

The particularly high-value additions would be **standardized root spans, snapshot/log-replay spans, automatic emission of existing Delta operation metrics, and shared object-store instrumentation**. Those four additions would close most of the current structural gaps without duplicating functionality already present in delta-rs.