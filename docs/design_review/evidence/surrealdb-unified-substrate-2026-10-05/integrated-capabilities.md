# SurrealDB as the complete canonical substrate

Supporting evidence for the
[principal review](../../reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md).

## Scope and evidence basis

**Proposed:** make SurrealDB the sole durable authority for authored problems and revisions,
dependency relations, reusable compilation products, durable runs, scientific tables and
provenance. Graph identity belongs to scientific entities, revisions, products and runs;
scalar results need not become separate graph vertices. Default persistence includes the
selected model, configuration, results and diagnostics automatically. Checkpoints are a
selective continuation facility, distinct from durable final outcomes.

**Interface-checked:** research used Context7 `/surrealdb/docs.surrealdb.com`, separately
querying transaction/absence semantics, scientific tables, schema/interfaces and historical
versions. Primary fallback was the shared skill's source corpus at SurrealDB tag `v3.3.0`,
commit `238bfeb11f5725bebed370167656748df8067595`, and documentation commit
`82b7ac1935fcbb400e80cf357dd68f47182bad26`. The corresponding
[upstream tree](https://github.com/surrealdb/surrealdb/tree/v3.3.0)
makes them inspectable independently of the skill installation. Source paths use the
skill corpus's package-root layout (`surrealdb`, `surrealdb-core`, `surrealdb-types`);
resolve these package roots within the upstream workspace rather than treating them as
guaranteed top-level upstream directories.

**Tested, historical:** `neo4j-surrealdb/surrealdb/skill_improvement/evidence/probe-results.json`
records its 2026-10-05 execution at 17:35:31 UTC. SB/SX identifiers below refer to this
existing corpus, not new tests in this repository. Server image digest was
`sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20`.
Source tests examined here are **Interface-checked**, unless explicitly tied to those
executed probes. No numerical speed, capacity, build-time or crash-recovery result transfers
from capability existence. Context7 includes older 2.x examples; exact 3.3 source wins when
features or semantics differ. Core APIs are not a stable patch-level integration contract.

## The complete positive implementation route

**Proposed:** persist typed problem/entity payloads, revision membership, explicit dependencies,
product input keys, product payloads, immutable run selection/configuration, result blocks,
diagnostics and outcome records in one namespace/database. Use schemafull tables, typed
record references, enforced classic relations, unique/composite indexes and transaction
functions. Entity-level graph queries connect a result block to its run, selected revision,
compiled product, scientific entity and authored provenance. Array/object/bytes fields
permit dense table blocks without a graph node per number.

This changes the economic comparison. The candidate can remove the PostgreSQL operational
catalog and its generated query layer, mandatory Delta publication and manifest coordination,
cross-store intent/head recovery, duplicate identities and redundant serialization. It can
also replace selected authored-table ingestion and document materialization. Those benefits
compound across one complete run; they are not captured by comparing one JSON insertion
with one existing insertion. See [integration and removal](integration-and-removal.md).
Scientific meanings and externally exercised export interfaces need explicit cutover, but
the present Delta/Arrow/SQL representation is not a permanent acceptance condition.

**Proposed:** retain library-owned symbolic and numerical execution as a projection from an
immutable revision/product. The database owns stored scientific meaning and durable
relationships; shared semantic libraries own physical typing, specialization and numerical
lowering. Arrow can remain an ephemeral columnar interchange or export format without
being another canonical database. Salsa can remain an optional pure incremental accelerator
without owning persisted truth. This is still a single canonical database target.

## Revisions, negative dependencies and concurrent edits

**Interface-checked:** [transaction documentation](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions)
specifies snapshot isolation and write-conflict checking. It does not promise serializable
predicate reads. Plain reads can miss concurrent changes that matter to later decisions.
`FOR UPDATE` registers selected records for commit conflict detection, rather than acquiring
a PostgreSQL-style row lock. SB091 observes native write skew without it and conflict with
it; SB092 records restrictions on its record targets. IndexedDB SB114 differs from the
native observation, so qualify the selected backend rather than assuming identical detail.

**Interface-checked, decisive positive route:** `surrealdb-core/src/exec/operators/fetch.rs`,
`fetch_raw_record_for_update`, explicitly fetches from the datastore rather than read cache
and registers the record key even when the record does not exist. This supports a named
negative lookup dependency. `surrealdb-core/src/kvs/tests/select_for_update_test.rs` covers
RocksDB competing writes, successful uncontended reads, untracked ordinary reads and
registration with `LIMIT 0`; it is not itself a new executed absent-key test. Arbitrary
absence such as “there is no matching member in this namespace” still needs a concrete key:
an existing namespace/name-bucket record updated by all matching mutators is a simple route.
Deleting an input is an invalidating mutation, not an omission from the recorded read set.

**Proposed revision representation:** a per-problem linear revision head updated with CAS,
immutable payload versions, indexed membership intervals `(problem, logical identity/name,
valid_from, valid_to)` and a separate indexed active-membership selection. An edit closes
the prior membership interval and creates the new one inside the same transaction as its
new revision and head advance. No complete membership copy per edit is necessary. Concurrent
writers both modifying the same problem head conflict; whole transaction retry reruns all
reads and checks. Products and runs store exact selected payload references, not a moving
head. A compiler's selected closure can consequently be fetched directly by exact identity.

**Qualification limit:** composite index availability does not prove an efficient plan for
every interval predicate. Global inventory at a historical revision can examine accumulated
membership history; state that cost honestly. Current inventory and selected historical
closure have different access paths. Qualify representative `EXPLAIN` plans for identity,
name resolution, current membership and historical inventory. No contradiction to the
interval/CAS route was found in the inspected sources. No bespoke persistent manifest trie
is required by this evidence, and no historical query efficiency guarantee is asserted.

**Interface-checked alternative, not yet an exact revision contract:** `SELECT ... VERSION`
and source language tests cover historical update/deletion, graph traversal, dereference,
subqueries and historical schema/index information. Relevant tests include
`language-tests/tests/language/statements/select/version_mutations.surql`,
`version_index_consistency.surql`, `version_schema_consistency.surql`, and reproduction
`7245_version_recursive_graph.surql`. Examples bracket `time::now()` with `SLEEP`.
Research did not establish a public returned successful-commit epoch or snapshot handle
that atomically labels a semantic revision. A clock timestamp alone must not stand in for
that contract. Existing SB002/040 backend observations are narrower and older than these
source fixtures; historical versioning deserves backend-specific qualification before use.

**Proposed product maintenance:** persist the actual positive reads, named negative reads,
membership scopes and deletion-sensitive dependencies used in compilation. Traverse reverse
dependency edges to find affected products; publish their invalidation/state alongside the
edit. A small edit need not reload every unrelated entity. Shared semantics may evaluate
the selected immutable closure outside the transaction, then conditionally publish its
product under exact revision/compiler/library/configuration keys. DB schema or a general
live subscription does not automatically discover compiler dependencies.

## Scientific tables and connected analytical queries

**Interface-checked:** [SELECT](https://surrealdb.com/docs/reference/query-language/statements/select)
supports field projection, nested values, record-ID ranges, filtering, grouping, aggregates,
ordering, limits, fetch and graph navigation. Bulk array `INSERT` is a single statement
(SB016 atomic behavior). Queryable scientific results can be row records or operation-sized
typed blocks with coordinate/quantity arrays, shape and unit/convention references. Pick
physical granularity by the operations: repeated time-window/range filtering benefits from
indexed row or partition keys; dense native kernels can read contiguous block payloads.
Embedding a huge opaque block alone would lose the connected query benefit. Conversely,
forcing every scalar into an individually related record is unnecessary.

**Proposed:** store exact scientific payloads and an explicitly derived finite numerical
projection in the same canonical result representation. Query selection can follow
run/revision/provenance edges and then filter/aggregate indexed coordinates and quantities.
Pre-simulation queries can select compatible authored entities, validity regions, initial
conditions and prior runs; post-simulation queries can compare runs by immutable model and
configuration identity. This is a legitimate database-native scientific table route, not
merely a pointer catalog around externally authoritative files.

**Interface-checked execution support:** the 3.3 planner contains indexed source selection,
graph predicate/limit pushdown and edge/vertex folding. In
`surrealdb-core/src/kvs/tests/graph_semijoin_test.rs`, an indexed scalar AND single-hop graph
predicate lowers to `BitmapGraphScan` plus `BitmapAnd`. The test declines multi-hop,
bidirectional and unindexed shapes; unique equality anchors deliberately remain streaming.
Index existence is not a promise that every graph/table composition uses that operator.
Check the intended result queries with `EXPLAIN FULL` on the chosen schema.

**Interface-checked, correction to a narrow streaming assumption:** public SDK
`surrealdb/src/method/query.rs` exposes `Query::stream_items()`, producing `StreamItem::Row`
and `StatementEnd`. Its bounded channels can backpressure callers. Rows are provisional:
a failing statement end means its emitted rows must be retracted, including transaction
rollback. Dropping the stream closes forwarding, but execution still finalizes its transaction;
non-emitting phases need not stop immediately. This is query result streaming, distinct
from `LIVE SELECT`. The gRPC engine explicitly overrides `query_stream` to forward rows
while the server produces them (`surrealdb/src/engine/remote/grpc.rs`). The default engine
route replays a completed buffered response. Do not attribute true bounded-memory server
streaming to every transport or to the mere existence of this method.
The SDK's `protocol-grpc` feature is explicit rather than included in its default remote
feature set; select it when this streaming route is part of the application contract.

**Interface-checked limits:** `exec/operators/sort/external.rs` implements disk external
merge sort for `TEMPFILES`, gated by the `storage` feature. Ordinary sort paths are in
memory. Aggregate operators process batches but retain group state and consume the complete
input before producing output (`exec/operators/aggregate.rs`); this evidence
does not establish universal aggregation spill or Arrow-style columnar vector execution.
Surreal's record/value engine and DataFusion's columnar operators serve different physical
workloads. This is a qualification risk for large cross-run analytics, not evidence that
all scientific tables must remain in Delta. First qualify the complete database target.
An external bulk exception needs a demonstrated operation/capacity requirement, explicit
authority and recovery semantics; it should not be inherited automatically from today.

## Scientific value exactness

**Tested, historical:** typed record IDs differ from strings (SB006); `NONE` removes a
field while `NULL` remains a value (SB007); conversion of a Rust `u64` beyond signed range
wraps (SB046). SDK `SurrealValue` conversion is not generic serde conversion (SX002).
The value oracle observes signed-zero loss on HTTP/WebSocket paths and favorable
gRPC/embedded parity for its tested corpus. These are scoped protocol observations, not
a guarantee for every scientific bit pattern or future SDK.

**Proposed:** represent full-width identities/counters with checked appropriate encodings,
and preserve exact IEEE-754 payload bits in bytes or another lossless declared codec where
scientific identity requires them. Store the finite numeric projection used for database
predicates/aggregates together with its derivation. Exceptional values, signed zero and
NaN payload identity must have explicit meaning. Decode into native numeric arrays only
at the execution boundary. A tagged scientific missing value must not be conflated with
database `NONE`, `NULL`, a deleted field, NaN or an absent record. One authoritative codec
can serve embedded and remote paths; it is less machinery than multiple store-specific
scientific encodings. Qualify exact round trips on the selected protocol and bulk route.

## Schema, functions and integrated interfaces

**Interface-checked:** schemafull/strict tables, typed fields, literal object/array unions,
assertions, functions, events, enforced classic relations and indexes can lower authored
semantic declarations directly. `surrealdb-types/src/kind/literal.rs` checks literal object
shape, including exact field count. This is richer than schemaless document validation.
It can replace duplicate relational declarations, mechanical operational queries and parts
of generated DTO/interface scaffolding. Database schema can itself be generated once from
the authoritative scientific declaration, or the database declaration can be selected as
authority; these are alternatives, not permission to keep two independently authored schemas.

**Interface-checked:** generated GraphQL provides configured schema/CRUD over typed tables
(SB105/107), and `DEFINE API` exposes named domain functions. Current docs surfaced
SurrealKit introspection/typegen (`generate`, `render_typescript`, `write_typescript`);
its exact version and Rust/Python output were not qualified here. Do not promise complete
generated scientific Python/Rust contracts from a TypeScript discovery. DB functions and
schema introspection provide a substantial integration route even without that tooling.

**Tested, historical / Interface-checked source:** custom API responses require the supported
HTTP body forms (SB136); a returned HTTP error is not the same as transaction `THROW`.
`surrealdb-core/src/api/invocation.rs` disables internal permission checks after API authorization;
domain API authorization and all-or-nothing mutation semantics must therefore be designed
explicitly. GraphQL ID encoding also differs between key and full record reference (SB105).
Mechanical generated CRUD does not define a scientifically valid simulate/compile operation.

**Interface-checked:** MCP exposes database query/GraphQL and related tools; named custom
APIs/functions or a thin domain MCP tool can compose the same canonical store. Full-text,
vector search and live notifications can support authored discovery and run observation.
They need not become separate project services. SB101 finds that reconnect ends a live
stream without resubscription; SB135 requires explicit `KILL` rather than assuming stream
drop cleans registration. Use live messages as hints and reread canonical state. Declared
changefeeds have retention and maintenance requirements (SB043/104), not an unlimited
event log or automatic durable queue.

## Structural compilation and native mathematics

**Proposed, substantive DB route:** queries/functions can perform finite structural checks,
referential validation, affected closure traversal, selected dependency expansion, scoped
name lookup and persisted structural products. Typed scientific declarations can be
relationally compiled into database schema and functions. Surrealism modules offer a
Rust/WASM extension route for suitable shared finite semantic kernels. The database is
not restricted to passive document storage, and current Rust consumers are not proof that
equivalent structural work cannot move into it.

**Interface-checked limit:** a database field type or graph closure is not automatically
physical unit/type inference, recursive specialization termination, property-validity
reasoning, expression typing or exact native mathematical preparation. Current
`pse-modeling/check.rs` and `specialize.rs` explicitly perform those operations; the compiler
also has tracked selected/specialized products. Functions can host bounded fixed-point
algorithms, but qualification must establish termination, completeness and diagnostics for
the selected scientific language. Materialized views are not an implicit solution: docs
for `DEFINE TABLE ... AS SELECT` update on the source table, not arbitrary linked-table
changes, and imports can bypass their maintenance. Likewise `FOR UPDATE` does not discover
the semantic read set.

**Proposed placement:** shared semantic Rust libraries can be called by an external compiler
worker or a qualified extension where practical. Symbolica/Numerica, faer and class-specific
native solvers retain their exact execution responsibilities. No source establishes that
the SurrealQL math catalog replaces symbolic transformations, derivatives, sparse numerical
factorization, nonlinear solver callbacks or thermodynamic libraries. Persist their exact
input/product identity and diagnostics in the database. Project graph algorithms such as
SCC/topological ordering can use a compact typed projection and an existing graph library;
avoid repeated database traversal for every native inner-loop operation.

## Durable default runs, claims and terminal outcomes

**Proposed:** persist a run's immutable revision/configuration and claim/fencing identity
before execution. Claim a named candidate using transaction read/write conflict checking,
then retry on conflict. This replaces the present PostgreSQL `SKIP LOCKED` claim mechanism
with an explicit database-native contract; ordered discovery is separate from exclusive
ownership. A lease generation prevents a restarted/stale worker from sealing another
worker's outcome. Native execution takes place outside long-lived database transactions.

**Proposed:** write bounded result/diagnostic batches under an unpublished immutable attempt
identity. One terminal transaction verifies the current fencing identity and completeness,
then seals the manifest and terminal outcome together. Readers select sealed outcomes;
they cannot observe a success pointing to incomplete results. A crash before sealing leaves
recoverable unfinished/staged state; a crash after sealing leaves the same complete outcome.
If an external bulk exception is later justified, this atomicity requires a separately
specified external-object publication protocol. Keeping payloads in the database avoids
that cross-store obligation in the initial target.

**Tested, historical:** SB013 requires checking query statement results rather than trusting
the outer successful await; SB031 distinguishes transaction rollback from partial effects
outside a transaction; SB032 covers cancel/commit. SB003 establishes clean persistent
RocksDB reopen and single-owner directory behavior, not power-loss recovery. SB118 exercises
export/import into an empty store and records repeated-import failure. Import can defer
enforced relation checks, views and events (`surrealdb-core/src/doc/edges.rs` and DEFINE TABLE docs).
Backup import is not a substitute for operational transactions or validated migration.

**Qualification limit:** choose durability configuration and backend explicitly; exercise
crash/reopen at batch, claim and terminal boundaries, uncertain client-commit recovery,
orphan staging cleanup, interrupted migrations and restore reconciliation. No new crash
probe was run. Acknowledged terminal atomicity has a supported transaction implementation
route, while the product's exact recovery contract remains Proposed until exercised.

## Deployment and alternatives

**Interface-checked:** the SDK default remote features do not include the database core;
local `kv-*` features do. SurrealKV is a Rust storage route, RocksDB introduces C++/libclang.
Other optional scripting/ML/HTTP features can pull core even without a selected local
backend. Source API availability is not a build-time measurement. Place an embedded engine
in the persistence/application composition layer, not the shared mathematical foundation
or broadly unified workspace-hack feature closure. Avoid creating a database dependency
for every numerical crate. A local server with a thin client can give the same local-first
automatic persistence while isolating memory and engine build dependencies. Embedding avoids
IPC and a separately supervised process but shares process lifetime, memory and failure.

**Proposed comparison:** Neo4j remains credible when mature graph analytics/GDS and Cypher
execution dominate. Its qualified server route, GDS projections and edition-dependent
constraints deserve their own complete-operation comparison. It does not offer the same
embedded Rust database route, and GDS projection memory is not a solver replacement.
SurrealDB's typed document/table/graph/functions/interfaces combination better matches the
specific single-store candidate than a graph-only comparison implies. Conversely a simpler
local typed store plus in-memory compiler can remove much of present operational machinery;
it has to supply connected query/interfaces, typed schema and transaction semantics itself
or accept their reduced scope. PostgreSQL offers mature relational transactions and query
planning; retaining it together with mandatory Delta preserves two canonical persistence
lifecycles. Arrow/DataFusion remain strong optional bulk computation/export libraries, not
automatically mandatory authorities. These are competing complete routes, not capability
counts or a prescriptive final verdict.

## Decisive qualification boundaries

The favorable complete target has a specific implementation route. The remaining decision
evidence should concentrate on: interval/CAS revision queries and named negative conflicts;
exact scientific codec plus finite query projection; representative connected table plans
and true gRPC streaming with rollback handling; claim/fencing and atomic terminal recovery;
one authoritative schema/interface lowering; and semantic compiler placement with native
library execution preserved. No new benchmark or arbitrary probe is needed to accept a
Proposed architecture. Quantitative performance or capacity promises need separate evidence.
