# SurrealDB canonical acquisition and analytical boundary

Bounded source evidence for the coordinator's production execution efficiency review,
examined 2026-10-07. This records current implementation and version-backed options;
it supplies no adoption verdict, plan disposition, test pass or measured performance claim.
Return to the [principal review](../../../reviews/design_review_production-execution-efficiency_2026-10-07.md).
The checkout includes concurrent Plan 28 changes. No production, test, configuration,
plan or shared skill files were changed. No probe or test was run.

## Version and source boundary

The workspace declares `surrealdb = "=3.3.0"`, defaults disabled, `protocol-grpc` enabled;
DataFusion is exactly 55.1.0. `scripts/surreal_server.py::serve` selects remote RocksDB
with `sync=every&versioned=false`; inherited `SURREAL_*` settings are removed and replaced
by the owned resource profile. These are source facts, not a live deployment inspection.

The selected `neo4j-surrealdb` and `datafusion` skills were read. Context7 resolution
selected `/surrealdb/docs.surrealdb.com` and `/apache/datafusion`; queries covered SELECT
index diagnosis, Rust transactions/batching, and provider pushdown. Those current docs
are discovery evidence, not release guarantees. SurrealDB conclusions below use
[v3.3.0 source](https://github.com/surrealdb/surrealdb/tree/v3.3.0), also captured in the
skill from release commit `238bfeb11f5725bebed370167656748df8067595`.
Fresh raw v3.3.0 downloads were byte-identical to the skill's `dbs/executor.rs`,
`exec/planner/select/mod.rs`, `exec/index/analysis.rs`,
`exec/planner/select/pipeline.rs`, `exec/physical_expr/subquery.rs`,
`idx/planner/{plan,tree}.rs`, and `dbs/iterator.rs`; the SDK's `engine/remote/grpc.rs`
was also byte-identical. No downloaded source was written here.
Prior dated review/evidence documents were leads only; their probe results are not
newly executed evidence for this production profile.

## Current acquisition and actual index shape

`crates/pse-operations/src/canonical_selection.rs::next_membership_pages` groups one to
eight independent inventory cursors into one `BEGIN ... COMMIT` RPC. Each SELECT receives
`LIMIT 64 / cursor_count`; the combined returned memberships are at most 64. This is
statement batching, **not SQL GROUP BY**. Each SELECT applies problem, optional scope,
key cursor, revision interval, optional `out.kind/out.closed`, and an optional correlated
edge-existence SELECT. `PROTECTED_BEGIN` in `canonical.rs` locks the exact retention
guard and validates the live revision protection. Decoding every page precedes advancing
any cursor premise. Exact terminal inventories, including empty ones, are part of reuse
correctness; returning the first convenient positive rows cannot replace them.

`crates/pse-operations/src/generated/surreal.surql` declares:

| Query concern | Existing index | Physical implication |
|---|---|---|
| Membership key cursor | `membership_pages(problem,key)` plus unique `key` | Credible problem/key access path; actual selected path and sort elimination need inspection. |
| Scoped name selection | `membership_selection(problem,scope,name,from_sequence)` | Supports leading problem/scope/name constraints; a scope-only inventory cannot automatically use the trailing sequence after an unconstrained name. |
| Revision interval | `membership_active(problem,to_sequence)`, `membership_logical(problem,logical,from_sequence)` | Different access shapes; historical interval filtering may remain residual. |
| Kind | `manifest_kind(kind)`, `membership_version(version)` | Kind lives on the referenced manifest, not directly on the membership. A remote kind index does not establish that this query is driven from it. |
| Supplier edge existence | `edge_source(source_version)`, unique `edge_ordinal(source_version,ordinal)`, `edge_target(target_scope,target_name)` | No declared compound index on all three source/target equality fields. Either source or target candidates can still require filtering. |

The memberships are real relation records pointing from problems to version manifests.
`canonical_edges` is a schemafull table of source-version and scoped-name fields, not a
`TYPE RELATION` adjacency table. Replacing its scoped-name semantics with resolved endpoint
IDs is a semantic change, not a free graph-index switch. Index declarations are owned by
`crates/pse-schema/src/catalog/substrate.rs`; `pse-codegen` emits the schema.

## What the 3.3.0 planner does and does not establish

The [executor](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/core/src/dbs/executor.rs)
tries streaming planning first and falls back to legacy evaluation only on
`PlannerUnsupported`/`PlannerUnimplemented`. Therefore the older planner's broad
non-indexed-OR fallback is **not sufficient evidence that the current query table-scans**.
The [SELECT planner](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/core/src/exec/planner/select/mod.rs)
resolves bound parameters and folds eligible constant expressions before index analysis.
Optional `NONE OR ...` clauses may simplify; field permissions and fetching side effects
constrain simplification. Unsupported residual clauses do not inherently prevent another
conjunct from driving an indexed scan.

The [index analyzer](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/core/src/exec/index/analysis.rs)
supports equality/range candidates, compound prefixes, union paths and eligible bitmap
compositions. Compound access consumes leading equality columns and then a range on the
next column; conditions after a gap or range become residual. Candidate selection is
heuristic, so the existence of several useful indexes does not prove optimal selection.
The [pipeline](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/core/src/exec/planner/select/pipeline.rs)
eliminates sorting only when the input ordering satisfies the request; otherwise it can
use bounded top-K state, an ordinary sort or explicit tempfile sorting. Top-K bounds
retained candidates, not the records examined. An ordered index path with filtering can
still examine many rejected records before producing a page, particularly an empty page.

The [subquery evaluator](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/core/src/exec/physical_expr/subquery.rs)
installs the outer row as `$parent/$this`, executes the inner plan and collects its values.
Its batch contract says each row runs the subquery. Thus the supplier predicate has a
credible per-candidate correlated-work concern. The inner `LIMIT 1` bounds its answer,
not necessarily the edge candidates examined to prove absence. This establishes an
execution mechanism, not the actual candidate counts of this particular query.

**Review inference:** returned-row and RPC-size limits establish admission/transfer
boundaries; they do not establish selective source work. Inspect the actual bound SELECT
variants with release-compatible EXPLAIN (including active/historical, kind/scope,
positive/negative supplier and skewed fan-out shapes) before claiming a scan defect or
indexed scaling. Use the 3.3 statement form
`EXPLAIN [ANALYZE] [FORMAT TEXT | JSON] SELECT ...`, documented by the
[official EXPLAIN reference](https://surrealdb.com/docs/reference/query-language/statements/explain)
and verified against v3.3.0's
[`surrealdb-syn` parser](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/syn/src/parser/stmt/mod.rs):
it constructs `Expr::Explain { format, analyze, statement }`, defaulting to text.
Plain EXPLAIN formats the plan without executing it; ANALYZE executes and drains the
inner plan to collect metrics. The SELECT planner also accepts the trailing
`SELECT ... EXPLAIN [FULL]` clause: it maps EXPLAIN to JSON ExplainPlan and FULL to
JSON AnalyzePlan. That clause does not imply the older planner's output schema.
Operator output-row metrics are not automatically a complete accounting of underlying
or nested records examined, and the informational format is subject to change.
No EXPLAIN/ANALYZE experiment was run here.

## Version-backed options and costs

1. Preserve exact tuple/ID demand and use native multi-statement batching where the same
   guarded operation permits it. The [3.3.0 SDK query builder](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/src/method/query.rs)
   chains queries and indexes their results. Awaiting a query retains the whole indexed
   response. Streaming rows remain provisional until statement/transaction completion.
   Larger batches amortize calls and protection checks but increase response retention,
   transaction duration and the conflict retry unit; they do not fuse scans automatically.
   `canonical.rs::protected_query` already owns up to eight retries only for definite
   transaction conflicts and rebuilds the guarded operation. Do not add another retry layer.

2. If the supplier query is demonstrably hot, a generated compound edge index on
   `(source_version,target_scope,target_name)` can narrow the existing equality predicate
   without changing semantic edge ownership. Alternatively target-driven acquisition can
   select candidate source versions first and intersect them with the exact protected
   memberships. Both must preserve selected-revision intervals, complete negative
   inventories, order and bounded transfer. A new access strategy is proposed, not
   measured; the current analyzer need not choose the new index without plan inspection.

3. Scope/key or kind-oriented physical projections can remove repeated residual work
   if representative plans establish it. Keep them generated from the one authority,
   with immutable-version mappings and complete mutation/retirement maintenance. Every
   new index adds build/storage and write/delete maintenance. The [release document index
   implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/core/src/doc/index.rs)
   processes index changes as records change. Neither extra indexes nor graph inline
   caches are a cost-free default. Schema initialization currently binds an interpretation
   and generated schema digest; upgrades require an explicit compatible data/schema
   transition or rebuild, not hand-edited generated output or a second migration authority.

4. Transactions and the [SDK begin handle](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/src/method/begin.rs)
   can compose structural reads/writes. Statement errors still require `check()` or
   inspection; a successful request alone does not establish successful statements.
   Existing exact guards and application pins cover concrete retention/concurrency gaps;
   a generic predicate SELECT is not a substitute. Keep scientific preparation and
   numerical execution outside long database transactions.

## Result access and DataFusion boundary

Concrete upload/read framing in the current tree:

- `pse-runtime/src/workflow/result_projection.rs::store_result_table` splits projected
  relations into row chunks of `64 / scalar_width`, then emits independently bounded
  IPC blocks. `result_blocks.rs::visit_blocks_async` awaits each visitor before the next
  block. Thus a wide projected relation can create many small sequential append RPCs,
  each paying block encoding and the ingestion transaction. Trajectories first group
  by symbol/sample and use one output index per bounded block rather than indexing
  every scalar sample. An empty relation still writes one independent IPC block.
  Current scalar widths are 10 for solve variables and 8 for solve constraints, giving
  chunks of six and eight rows respectively even when the IPC payload is far below
  the 512 KiB block cap; fit parameters/observations use widths two/four.
- `canonical_execution.rs::append_execution_batch` sends one fenced function call per
  block, carrying the IPC bytes, block metadata and arrays of cells/output indexes.
  `pse-codegen/src/codegen/surreal_execution.rs` bulk-inserts those arrays inside the
  same operation; there is **no per-cell RPC** in this route. It updates the result set
  and creates the operation receipt. The function returns `SELECT *` from the saved
  `canonical_result_batches` record, including its full IPC payload; Rust decodes it
  and compares `saved != batch`. Consequently the normal acknowledgment echoes the
  scientific bytes back across the boundary, not merely their digest/identity.
- `canonical_results.rs::result_block` opens a protected transaction to fetch exact
  block metadata and compare it with the supplied immutable metadata, then invokes
  `result_payload`, which opens another protected transaction for the exact batch.
  This is at least two sequential protected RPCs per block, following any discovery
  page RPC. Metadata+payload selection could be composed in one guarded operation
  while retaining the comparisons and protection lifetime; the source does not prove
  that the separate transactions are required by the result contract.
- `canonical_execution.rs::reconcile_closed_attempt` lists result sets, then scans
  each set's immutable batch headers in sequential 64-item pages, establishing exact
  contiguous coverage and framed digests before admitting the manifest. This is
  completeness work, not accidental blob decoding; grouping independent header pages
  is conditional on preserving the same closed-attempt checks and bounded retention.
- Canonical **source** hydration already groups exact objects: `canonical_staging.rs`
  permits up to 64 object headers and packs selected block requests under a combined
  512 KiB payload bound. `modeling/canonical.rs::selected_objects` reserves complete
  decode extents and accumulates each demanded source before finishing the batch.
  Do not describe this as a current singleton-source lookup mechanism. Grouped transfer
  does not remove full required source assembly or its lifetime reservation.

The [3.3.0 gRPC SDK](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/src/engine/remote/grpc.rs)
converts bound variables to protobuf values. `open_query` sends an empty
`accepted_encodings`, requesting row-oriented values, not native Arrow-columnar frames.
`query_inner` accumulates batches by statement, then requires the terminal End frame
and matching result count. The internal stream does not make awaited indexed results
incremental consumer processing. IPC bytes are opaque byte fields within these row
values; storing IPC does not make surrounding metadata RPCs Arrow-native.

**Proposed options:** replace successful append payload echoes with a sufficient
immutable acknowledgment (preserving exact operation settlement), compose per-block
metadata/payload reads, or admit a bounded set of blocks through one append function.
These remove concrete transfer/call work if adopted. Larger units enlarge memory,
retry and cancellation exposure and must preserve ordinal coverage, ingestion fencing,
idempotency and closure. Changing the 64-cell unit requires adjusting its actual
admission and replay-comparison contracts; removing the bound without a replacement
is not an optimization. No latency or throughput benefit has been measured here.

`canonical_results.rs::result_output_page` filters output index metadata before fetching
scientific bytes. Dense records use `(result_set,output,partition,start)`; sparse rows use
`(result_set,output,partition,row)`. The sparse numeric index is
`(result_set,output,projection.projection,row)`, omitting partition; numeric and row-order
paths therefore trade selectivity against ordering/partition residual work. Second-stage
block metadata acquisition uses a bounded `key IN $indexes.batch`, followed by exact-ID
blob reads. These are good existing reduction boundaries; no claim that every result
query examines only its returned rows follows from them.

DataFusion 55.1.0's actual trait owner is
[`datafusion/session/src/table.rs`](https://github.com/apache/datafusion/blob/55.1.0/datafusion/session/src/table.rs)
(catalog re-exports it). `scan` preserves filters → limit → projection semantics;
`Inexact` pushdown cannot safely accept limit pushdown. Filter fields may be needed even
when omitted from the output projection. A provider scan limit is a minimum production
obligation when enough rows exist, not a guaranteed memory ceiling. The scoped engine
provider/session sources inspected contain generic/immutable providers and delegation;
they do not establish automatic SurrealQL pushdown. Once data is materialized into Arrow,
later DataFusion filtering cannot undo database acquisition work already performed.

Keep connected identity/eligibility and selective structural/result metadata acquisition
at the canonical store, admitted Arrow analytics at DataFusion, and Symbolica/native
mathematics at their numerical owners. A custom provider is conditional on an actual
analytical demand and preservation of protection lifetime, SQL/null/ordering semantics
and completion behavior. It is not required merely because both libraries are present.

## Verification limits and handoff

Source inspection and Context7 documentation retrieval: completed. Tagged source identity
comparison: completed for the nine named files. Product tests, probes, EXPLAIN runs and
performance measurements: `not_run` by assignment. The maintainer-interrupted native
suite is not a discovered failure; fixture setup/teardown time has not been split here.
The coordinator owns any decisive scoped experiment, verdict and plan changes.
