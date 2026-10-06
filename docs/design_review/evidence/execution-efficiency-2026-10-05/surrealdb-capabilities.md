# SurrealDB capability and integration evidence for execution efficiency

This supports the [principal execution-efficiency design review](../../reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md). It assesses
library fit and competing realizations, not the principal verdict or a migration authorization.
The target is a locally deployed, extensible process simulator; build and runtime burdens receive
equal attention. Existing decisions are design subjects, not restrictions on the alternatives.

## Scope, baseline and evidence

**Interface-checked, 2026-10-05:** `main` at
`6498b013e579ec8039573200c92282eb8715b52e`, plus concurrent numerical changes in
`pse-backend-native` root isolation and engineering-accuracy producers, `pse-math` factorable
execution/tests and runtime dynamics accuracy. Those changes were preserved and are not library
qualification evidence. The selected assessment standard is Core 3.4, process-simulator 1.5 and
the execution heuristics. Relevant obligations include AP-02/AP-04/AP-07, DP-07/09/14/19/22 and
PS-01/05/07/09/11/12. No product implementation, download, test or benchmark was performed here.

Library mechanisms were checked through the shared `neo4j-surrealdb` skill's routes, eighteen
SurrealDB capability briefs, selected catalogs, exact source, existing probe receipts, current
Context7 queries and official documentation. The skill's source profile is SurrealDB SDK/core
3.3.0, upstream commit `238bfeb11f5725bebed370167656748df8067595`; documentation is main commit
`82b7ac1935fcbb400e80cf357dd68f47182bad26`. The server receipt names image digest
`sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20`.
Neo4j evidence uses Python driver 6.3.1 and server 2026.09.0, Community with separately identified
Enterprise single-instance probes. This does not qualify a Rust Neo4j client.

**Tested, historical skill scope:** the SurrealDB receipt records a passed run at
`2026-10-05T17:35:31Z`; probe references below identify its named run/control cases. They establish
small mechanisms on the engines/transports named by each probe, not this simulator's behavior.
Current docs are discovery and contract evidence; pin-specific source/probes control a conflicting
version-sensitive claim. `surrealdb-core` explicitly describes itself as unstable, including patch
changes. The runtime SDK is a different integration surface from direct core embedding.

**Not measured:** this checkout's graph loading, preparation, build times, memory, concurrency,
query latency and end-to-end scientific workflows. The skill describes a comparison benchmark,
but the inspected `compare/bench/results/` directories are all marked `-smoke`, and the comparison
matrix supplies no production timing anchors. No benchmark numbers or build-time ranking are
transferred here. Some brief introductions retain earlier counts while later catalogs/probes expand
coverage; named evidence is more reliable than an inventory total.

## What the analogy does and does not establish

The library-context [graph-native review](https://github.com/paul-heyse/library-context/blob/681ac5299d59c60553fb7dee6c6afd2020afc57e/docs/design_review/reviews/design_review_graph-native-target_2026-10-05.md)
and [capability assessment](https://github.com/paul-heyse/library-context/blob/681ac5299d59c60553fb7dee6c6afd2020afc57e/docs/design_review/reviews/design_review_surrealdb-capabilities_2026-10-05.md)
are relevant evidence, not decisions for this repository. Their useful transferable mechanisms are
compile/admit before publication, short storage transactions, distinct semantic/physical identities,
coarse connected retrieval, compact attributed graphs and reuse of validity under unchanged premises.

Their diagnosis included hundreds of semantic relation tables, repeated database reference scans,
compiler work within a store transaction and a graph evidence-serving product. Their disposable-data,
single-operator snapshot lifecycle and search requirements do not transfer automatically. Pse-arrow's
scientific compiler already operates on typed in-process inputs; PostgreSQL owns operational state,
and Delta owns published relations (blueprint §1.1–1.2, §20). Changing PostgreSQL therefore does not
automatically remove the source reconstruction, specialization or numerical preparation under review.

## Current responsibilities and actual replacement seams

**Interface-checked source:** these paths identify consumers and ownership, not a whole-product
performance result.

| Responsibility | Current mechanism and source | What a database alternative can change |
|---|---|---|
| Source documents and immutable selected revisions | `pse-runtime/src/workflow/modeling.rs`: `document_inputs`, `modeling_from_documents`, `ModelingPackage`, `PackageAdmission`; authoring driver package/document admission | Persist documents and typed declarations; index selection and references; replace selected document/Arrow transport steps if that becomes the authority boundary |
| Checked definitions and specialization | `pse-compiler/src/workspace/modeling.rs`: `ModelingRevision`, `Catalog`, `Request`, `TrackedDocuments`, `document_table` | Supply an admitted selected closure. A database graph does not itself replace the physical type checker, specialization or complete dependency keys |
| Reuse and evaluator preparation | `pse-runtime/src/math.rs`: service workspace, artifact retention, flights and worker resources; compiler workspace and modeling queries | Persist selected reusable descriptions or diagnostics; avoid reloading unrelated authored content. Preserve or improve pure incremental queries and native artifact lifetime independently |
| Source export | `modeling.rs`: `SourceExport::matches`, `source_tables`, `encode_source_tables` | A direct typed store path could avoid an Arrow encode/decode boundary for selected source operations. Existing exports already reuse checked buffers across package clones; do not count their repeated encoding as established without examining misses |
| Durable attempts, jobs, studies, streams | `pse-operations/src/store.rs`, `attempts.rs`, `streams.rs`, `studies.rs`; `queries/jobs.sql`, `queries/catalog.sql` | Replace PostgreSQL pool, generated SQL/COPY/query adapter and installation dependencies with engine-specific repositories and typed codecs |
| Notification and recovery | `pse-operations/src/listener.rs` | LIVE SELECT can replace LISTEN plumbing, but reconnect resubscription and authority reread remain |
| Exact publication and offline reading | `pse-catalog/src/delta/publication.rs`, `manifest.rs`, `provider.rs`, `settlement.rs`; operational catalog | An all-database durable realization could remove selected member-write/manifest coordination. Keeping files or Delta preserves a cross-resource commit gap and its recovery responsibility |
| Relational inspection and scientific result boundaries | `pse-catalog/src/inspection.rs`: `TableReader::query`, `new`, `next_batch`; engine sessions | Indexed graph/document queries can serve connected inspection. Existing SQL and leased Arrow batch consumers require an explicit replacement/lowering, not the PostgreSQL wire front-end |

The existing direct `modeling_package(rows, physical)` path admits typed declarations without text
serialization. `modeling_from_documents` does pass through source batches and generated row decoding.
This is a concrete representation seam to compare with direct typed admission; it is not evidence
that a new database is required to remove it. `document_table` is already a tracked query over an
admission plan and document input. The compiler/workspace retention lifetime and query-key granularity
remain decisive; persistent graph storage is orthogonal to memoization.

## Typed graph/document representation and compiled schemas

**Interface-checked:** SurrealDB can store nested documents, typed record IDs and identity-bearing
relation records in one engine. `SCHEMAFULL`, a `STRICT` database, required/optional fields,
literal unions, nested object/array types, `ASSERT`, unique indexes and typed relation endpoints
offer a richer structural realization than an untyped graph property bag. Official
[literal types](https://surrealdb.com/docs/reference/query-language/language-primitives/data-types/literals)
include discriminated object alternatives; exact 3.3 `surrealdb-types/src/kind/literal.rs`
contains object, array and scalar literal kinds. Historical probes SB011/028–030/047 confirm
coercion, closed shape, strict table admission, required fields and unique-index construction cases.

**Proposed opportunity:** lower the existing registry's structural contracts into database schemas
and typed codecs, or make typed model declarations directly produce a selected database realization.
Physical families can group related semantic variants with a typed tag/payload union instead of
one table per declaration kind. Derive shape, enum values, endpoint table sets and local scalar
checks mechanically; keep physical indexing decisions with the integration owner. This can remove
duplicated wire declarations and broad generated artifacts only where their consumers move.
The registry is not intrinsically required forever, but any alternative must retain one executable
authority for each consumed meaning. Independently authored DDL and Rust rules recreate that problem.

Structural database types do not discharge unit/basis/reference-state compatibility, physical
method applicability, scoping, finite specialization, conservation or derivative obligations.
SurrealQL coercion is observable: `TYPE int` accepts `1.0` as `1` (SB011). A strict scientific
adapter must decide whether such coercion is permissible rather than assuming Rust newtype parity.
Database signatures are not a replacement for a typed scientific operation contract (PS-01/07).

There are three distinct type-system opportunities. First, compile storage types from declared
model types so local shape/enum/range checking is delegated. Second, execute set-based resolution,
reference closure and applicable structural predicates over indexed records instead of reconstructing
them in host maps. Third, execute actual contextual inference/specialization near the stored model.
The third is a scientific compiler realization, not ordinary database DDL: it needs lexical scope,
selected revisions, parameter substitution, source spans, cycle policy and diagnostic provenance.
Current `pse-modeling/src/check.rs` already uses petgraph SCC/toposort; `specialize.rs` performs
demand-driven construction with recursive-parameter, child-construction and lazy-alias cycle
rejections and calls `expression::infer`/quantity operation inference. `TypeContext` carries the
physical environment. Database recursion, joins or a shared module could implement selected
pieces under the same authority, but a transitive dependency query is not that inference algorithm.

This does not require preserving the current checker wholesale. A relationally compiled type
system can be a credible target when its inference rules and finite closure contract are explicit,
the native engine handles the needed fixed point/set operations and the scientific distinctions
survive. Compare its complete preparation and update operation with the current checker plus
Salsa and with established reasoning/graph libraries. Type unions and stored constraints alone
neither establish that route nor disqualify it. No such full inference lowering or execution probe
was inspected here, so its benefit and adequacy remain Proposed.

**Tested, historical graph cases:** SB020–027 and SB117 show parallel edges, endpoint enforcement,
delete behavior, record references, recursive paths and bulk adjacency construction. `RELATE` can
create duplicate or dangling edges unless the chosen contract adds uniqueness and `ENFORCED`.
`INSERT RELATION` creates graph adjacency; ordinary `INSERT` of edge-shaped objects does not.
Classic relation records can carry assertion identities/provenance and participate as endpoints;
exact `core/src/doc/edges.rs` prohibits that for lightweight relations. Reify n-ary assertions,
ordered operands and role assignments rather than converting them into ambiguous binary links.
Do not make `(in,out)` unique where distinct assertions between the same endpoints are legitimate.

The required scientific structures remain different: package imports and semantic dependencies,
flowsheet connections, equation-variable incidence and solve partitions (PS-05). One engine can
persist all of them, but must not make their edge meanings interchangeable. A topology projection
must preserve isolates, parallel identities, direction, scope and its mapping to source entities.

## Values and fidelity at the Rust boundary

**Tested, historical SDK cases:** 3.x uses `SurrealValue` rather than serde alone (SX002);
`RecordId` is a distinct value, and a string that looks like an ID is still a string (SB006).
`NONE` and `NULL` differ (SB007). Nested values provide a credible fit for authored definitions,
case policies and structured scientific outcomes without flattening every field into graph edges.

The adapter has real obligations. The skill's values oracle records signed-zero loss over ws/http;
u64 values above `i64::MAX` wrap on binding (SB046), and serde JSON can introduce further number
loss. Pse-arrow canonical framing preserves signed zero and typed identity domains (blueprint §5.3).
Use a specified lossless codec, checked ranges and typed IDs; where necessary preserve float bits
in an explicit representation and expose numeric projections separately. Neither a successful
JSON round trip nor schema `float` guarantees canonical scientific fidelity. That codec has not
been implemented or qualified here. Remote grpc parity for recorded values is a lead, not blanket
protocol fidelity for every compound simulator type.

Generated `SurrealValue` derives may reduce adapter code, but exposing database value types through
all mathematical owners would increase coupling. Keep portable scientific types and absorb store
details at their persistence boundary. Mechanical generation can preserve locality without forcing
a generic backend abstraction or two editable model representations.

## Queries, indexes and complete operations

**Interface-checked:** parameterized SurrealQL, arrows, record links, `FETCH`, array/object helpers,
correlated subqueries, aggregates, functions and recursive idioms support selected connected reads.
The [query optimization guide](https://surrealdb.com/docs/learn/querying/concepts-and-guides/query-optimisation)
and [EXPLAIN](https://surrealdb.com/docs/reference/query-language/statements/explain) expose physical
plans. Exact 3.3 `core/src/kvs/tests/graph_semijoin_test.rs` checks indexed scalar restrictions
intersected with a single-hop graph predicate (`BitmapGraphScan`, `BitmapAnd`), and explicitly
declines multi-hop, both-direction and unanchored forms. A unique equality anchor stays streaming
rather than eagerly draining adjacency. `exec/planner/source.rs` contains graph predicate/limit
pushdown and edge-to-target folding. These are concrete native opportunities, not merely attractive
syntax, and are not an arbitrary-traversal optimization guarantee.

Good candidate operations include selected package/reference closure, type/property eligibility,
lineage inspection, all dependents of an authored item and source-linked explanation of a selected
result. A function can return the complete selected segment and required fields without host-driven
single-key hydration. Predicates, projection and useful bounds should remain visible to the engine.
Global structural analysis may instead stream one compact projection into a native library and
reuse it; repeatedly traversing the database during every solver iteration is a different workload.

The graph/document query language has no general ANSI `JOIN` operator; indexed record-link joins
and subqueries can express useful joins, with different plans. A DataFusion SQL operation with
joins/aggregates/spilling cannot be assumed to transfer efficiently or semantically by rewriting
it as nested selects. SurrealDB's own executor uses operator/batch planning; describing every
database operation as scalar would also be inaccurate. Inspect the specific plan and crossing.

**Limits:** depth is not a work bound; fanout and walk enumeration can multiply before a small
answer is returned. Hydration can retain broad payloads. Recursive output/path equality must retain
the assertion identities and order consumers need. Row/byte/frontier budgets and honest partiality
belong to the operation; a timeout is not a complete empty answer (AP-07, PS-12).

Precomputed `AS SELECT` views are useful for selected stable aggregates, but not a general compiler
incrementality mechanism. The [table contract](https://surrealdb.com/docs/reference/query-language/statements/define/table)
and exact source/docs say only writes to the `FROM` table trigger updates; linked table changes
can leave them stale. Import skips view updates and requires an explicit rebuild. SB124/125 cover
a small grouped view and computed-versus-write-time values. A compiler dependency graph needs
deletions, missing-name dependencies, relevant field changes and invalidation of all affected
products; a view or LIVE SELECT does not automatically provide Salsa's dependency tracking/backdating.

## Native functions, modules and mathematical execution

**Tested, historical SB120:** `Surreal::run` invokes typed SurrealQL functions. Named functions can
own fixed graph selection, eligibility, hydration, small reductions and operational transitions.
Their definitions and revision become inputs to reproducibility if an output depends on them;
pinning rows without the function/schema realization leaves behavior mutable.

**Interface-checked only:** 3.3 supports experimental Surrealism modules through WASM, with the
captured dependency on Surrealism 0.6.0. The skill did not execute `DEFINE MODULE`. The adjacent
library-context assessment inspected WASI P2/ABI2, transactional query host calls and runtime
memory/timeout/state controls. This suggests selected portable Rust kernels could execute near
data without reimplementing meaning in another language. It does not qualify this project's
Symbolica/Numerica/native-solver dependency graph under WASI, shared library state, cancellation,
floating-point behavior or ABI. Stateful instances and module KV are not durable semantic inputs.

**Assessment:** keep expression compilation, derivatives, sparse layouts, factors and native
solver iteration in their suitable libraries unless a particular database execution route actually
improves the complete operation. Scope a module to a proven portable kernel when it removes a
specific transfer or coordination burden. Do not build a second scientific interpreter in
SurrealQL to justify consolidation. Conversely, Rust semantic ownership does not require every
selection or deterministic structural check to execute host-side (H3, PS-09/11).

The searched skill function catalog and module contracts establish graph/document, arithmetic,
search and extension primitives, not a qualified replacement for physical finite compilation or
native NLP/DAE solving. This is a bounded integration conclusion, not a claim that no future module
could implement a particular scientific operation.

## Local deployment and build burden

**Interface-checked source; Tested historical SX001/003/005/006:** the 3.3 SDK default features
are `protocol-ws`, `rustls`, `parse`; a remote client avoids the embedded database core. Selecting
`kv-*` pulls in the local engine/core. Other engine-oriented features can pull the core even without
providing a local backend. SurrealKV is a pure-Rust persistent choice; RocksDB requires C++ and
libclang. The skill's default-client compile probe establishes a Rust 1.95 floor, not compatibility
with this checkout's entire compiler/profile/native feature set. The recorded docs.rs extraction
failure in a transitive DISKANN crate on a different nightly is an environment/version warning,
not a proof this repository fails to build it.

| Local realization | Build and operational consequence | Runtime consequence |
|---|---|---|
| Correct existing architecture | Retains existing DataFusion/Delta/native dependencies and PostgreSQL tooling; can prune unused features and reduce generated output independently | Local pool/service plus columnar files; changes can remove repeated work without importing another engine |
| Embedded SurrealKV | Adds core/executor/index/storage closure to application build; removes separate server installation and IPC for adopted roles | One process owns the store; allocator/cache/native working sets and failure share the application's envelope |
| Embedded RocksDB | Adds core plus native C++ build/bindings; existing numerical native tooling does not make this cost disappear | Persistent embedded operation, backend-specific tuning and directory ownership |
| Local SurrealDB server with remote SDK | Small application-side client relative to embedding; server must still be installed/upgraded and qualified | Shares store across clients/processes; isolates database resource/failure scope, adds serialization and IPC |
| Neo4j local server | Independent Java/server deployment and Rust client integration work; this skill qualifies its Python route | Rich graph execution/GDS, separate heap/page cache/projection state |

SB003 establishes single-handle RocksDB directory ownership in the tested opening scenario;
local embedded persistence therefore needs one clear owner and cloned handle access, rather than
each worker reopening the same directory. Server deployment is credible for multiple worker
processes even on one machine. Local-first does not mean memory-only or single-threaded, and does
not force embedding. Optional ephemeral numerical work can remain store-free.

Exact SurrealDB workspace source requests `object_store = "0.13.2"`, matching the current family
at source level. This is promising integration evidence, not a resolved combined dependency graph.
Core can enable cloud object-store features; remote-only avoids that core closure. Cargo feature
unification, native libraries and optional WASM/scripting/search features must be examined together
before claiming build savings. CJK dictionary build scripts need network/cache preparation (SX006).
No license is used to exclude a technically suitable candidate.

The coordinator's build investigation identifies the workspace-hack's broad dependency closure,
repository-wide `SOURCE_IDENTITY` propagation into mathematical artifact keys and eager native
environment setup as separate possible build/reuse burdens. An embedded database dependency added
to that same unified foundation closure could make unrelated targets pay for its engine; optional
features alone do not establish isolation when feature unification exposes them broadly. Place
the engine at the actual composition/store owner, or use a separate local service/executable,
and inspect the generated closure before claiming locality. A remote client and an embedded
choice can coexist as supported packaging profiles if their additional maintenance is justified;
they need not imply two canonical stores. Correcting identity and dependency granularity is a
competing remedy, not a mandatory interim implementation before adopting a justified graph target.

## Transactions, recovery and publication exactness

**Tested, historical SB031–033/053/089–092/109–114:** explicit transaction blocks and client
transactions commit or cancel bounded database changes. Concurrent same-record writes conflict;
the Rust SDK does not supply the application's retry loop. Native-engine snapshot isolation allows
write skew unless decision records are brought into conflict checking with `FOR UPDATE`. Its
locking targets are record IDs; table-predicate `FOR UPDATE` is not the current job-queue primitive.
Current [transaction docs](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions)
describe a common snapshot-isolation contract, but SB114 reports IndexedDB rejecting the write-skew
case even without locked reads. Keep backend-specific observations and avoid a blanket guarantee.

Current `queries/jobs.sql` atomically selects an available ordered job with `FOR UPDATE SKIP LOCKED`
and updates ownership. `queries/catalog.sql` names row/advisory lock ordering and commit/retention
protection; publication depends on more than storing one graph. SurrealDB can implement a different
optimistic claim/CAS protocol or local single-owner queue, but neither is a drop-in translation.
Predicate membership changes, absence reads, concurrent inserts, leases and retention require a
specific invariant-preserving realization. There is no general serializable-isolation claim here.

The adapter must inspect every query statement: `query().await` can succeed while a statement
failed (SB013); without an explicit transaction earlier statements can commit. Bulk INSERT is
atomic within its statement (SB016), whereas `INSERT IGNORE` can silently omit unique-conflicting
rows (SB048). Query import modes skip some ordinary field/event/view/endpoint checks; exact
`doc/edges.rs` defers `ENFORCED` endpoint validation during import. Import/export convenience is
not semantic admission or persisted-content reconciliation. Typed errors are uneven: unique
violations and embedded conflicts can require message interpretation (SB015/033/089/090/109),
unlike the current SQLSTATE classification. Keep that brittle detail inside the adapter.

LIVE SELECT can simplify push updates (SB041/051), but the stream ends on ws reconnect and is not
resubscribed (SB101). Dropping a stream leaves its registration alive in the tested scenario
(SB135); explicit cleanup/reconnection remains. The current listener already treats notifications
as latency hints and rereads authority after resync. Preserve that behavior rather than turning a
notification into lifecycle truth. Changefeeds require declaration, retention and maintenance;
an undeclared feed returns empty (SB043). Core streaming may send rows before final transaction
failure and retract them (SB103); avoid exposing them as committed results prematurely.

**Proposed publication alternatives:**

- With all selected immutable semantic/results content in one database, build a private revision,
  admit/reconcile it, quiesce its writer and publish a small handle transition. A same-database
  transaction can cover its catalog records. Separate databases do not become a cross-database
  transaction, and immutable publication is an application protocol, not a frozen-database feature.
- With Delta/Parquet/native artifacts retained externally, short database metadata publication
  still follows prepared effects. The intent, uncertain-commit settlement and cleanup obligations
  survive unless a simpler adequate artifact protocol replaces them. Bucket/file effects must not
  be assumed to roll back with a database transaction.

SB118 exercises logical export/import and SB003 a clean reopen; neither establishes power-loss
durability, interrupted bulk-load recovery, publication crash safety or destructive retention for
this product. Those remain decisive implementation evidence gaps for any broad replacement.

## Broader capabilities and Neo4j comparison

Full capability eligibility remains: future authoring/search tools can legitimately use full-text
BM25, exact search, HNSW/DISKANN, database functions, events, APIs or projections even without a
current production consumer. Adoption should remove machinery or supply a justified new capability.

| Capability | Credible role | Limits affecting this review |
|---|---|---|
| Full-text and vector search | Find authored units, physical methods and source-linked examples | SB035/038/039/098 establish primitives; relevance is heuristic and an absent vector index can return empty silently. Scientific eligibility remains typed and exact |
| Events and computed fields | Small transactional derived state | Adds write amplification/hidden effects; computed fields execute at reads. Avoid reconstructing all dependency semantics in triggers |
| Custom API/GraphQL/MCP | Authoring/UI/tool integrations over domain operations | Generated CRUD is not scientific admission; endpoint auth and transaction failure semantics differ. PostgreSQL wire executes SurrealQL/GQL and returns protocol-specific rows, not existing SQL/Cornucopia compatibility |
| Files/buckets | Selected artifacts with immutable object names | Experimental controls and external-resource failure remain; object-store backends not run by the skill |
| Temporal `VERSION` | Selected record history | SB002/040 require versioned SurrealKV/RocksDB; memory/TiKV refuse it. It does not freeze schemas/functions, define semantic revision or retain native compiled products |
| ISO GQL | Graph-shaped tools | Skill catalog tests a subset; Cypher `MERGE/WITH/UNWIND/CREATE` are not assumed portable. Rust SDK SurrealQL is the simpler qualified route than direct unstable core for GQL |

Neo4j is a serious alternative when complex graph-query composition or GDS is central. Its GDS
Community probes GD01–07 cover PageRank, SCC/WCC, Louvain/Leiden, similarity and kNN over tiny
graphs; the graph catalog retains an in-memory projection until dropped. That projection still
adds materialization and lifetime, and does not replace physical incidence/rank/native solving.
Community lacks the existence/type constraints that its Enterprise probes SC06/07 supply. Nested
scientific documents need an explicit encoding or reification because property storage does not
accept arbitrary nested objects. This is a technical fit distinction, not a license rejection.

The current [Neo4j property API](https://github.com/neo4j/neo4j/blob/2026.07/community/graphdb-api/src/main/java/org/neo4j/graphdb/Entity.java)
and [memory guide](https://neo4j.com/docs/operations-manual/current/performance/memory-configuration/)
support those representation/resource considerations. The inspected comparison is asymmetric:
Neo4j Python versus SurrealDB Rust. No Rust Neo4j client, whole-process packaging comparison or
large-graph GDS performance was qualified. Do not select SurrealDB solely because the local skill
does not cover another candidate's Rust route.

## Competing remedies and decisive remaining evidence

All benefits below are **Proposed**; no overall winner or speedup is established by this document.

| Alternative | Mechanism that buys improvement | Machinery removed or retained | Decisive gap |
|---|---|---|---|
| Correct current architecture | Finer valid reuse, direct typed admission where suitable, demand-aware preparation and operation-shaped projections | Removes repeated work at its actual owners; retains columnar/operational stack | Identify specific repeated operations and necessary dependencies; do not assume current wrappers/lifetimes are adequate |
| Selected SurrealDB graph/document roles | Indexed authored catalog/dependencies and coarse inspection; compiler receives selected immutable typed inputs | Can remove selected ad hoc lookup/export/hydration machinery; adds graph codecs and publication/index lifetime | Demonstrate an operation whose complete selected route is simpler than typed maps, Salsa or relational projection |
| Broader SurrealDB operational/semantic consolidation | One local durable owner and bounded transactions; all-database publication where content fits | Can remove PostgreSQL/Cornucopia/COPY installation and selected catalog/Delta protocols; cannot count code with remaining Arrow/SQL/file consumers as deleted | Invariant-preserving job claim/leases/CAS/retention, lossless types, crash/reopen and native artifact boundary |
| Database-centric compiler/math | Functions/modules reduce specific transfers; portable shared kernels preserve semantic ownership | Can fuse a suitable structural operation; otherwise adds scientific interpreter/module state/build and transactional resources | A compatible kernel/ABI/resource route that preserves scientific contracts and actually removes work |
| Neo4j graph owner or projection | Rich graph language/GDS at selected graph workloads | Adds server/client/projection lifecycle; can delegate graph algorithms | Exact scientific graph mapping and Rust/local integration rather than assumed Python parity |

The substantial choice is between operation-shaped storage and execution compositions, not
between a relational model and a graph ontology in the abstract. Typed declarations may lower to
both relational and graph views under one authority; eliminating unnecessary conversion may be
sufficient in one operation while durable graph indexing is justified in another.

The later principal review should settle: which ordinary edit/re-solve/study work repeats;
which selected graph query removes host work; whether a store owns declarations or only derived
views; what results must remain SQL/Arrow/file interoperable; how local workers share a durable
owner; and which crash/concurrency guarantees users actually need. Preservation requirements for
durable models/results/history must be confirmed before importing the analogy's disposable-data
cutover. Current ADR restrictions may change through the design route; library selection does not
apply those changes by itself.

## Source and probe routes for independent examination

Repository owners: blueprint §4/5/6.15/20/22 and §1.1–1.2; the current source paths above are rooted
under `crates/`. Exact API families currently include Salsa 0.28.4, DataFusion 55.1.0,
Arrow/Parquet 59.3.0, object_store 0.13.2 and the vendored Delta profile; manifest/lockfile are the
consumer version authorities. Shared `salsa`, `datafusion`, `deltalake`, `symbolica-faer-oximo` and
`native-solver-libraries` skills were loaded for responsibility and version comparisons.

Within shared `neo4j-surrealdb/`, reproducible read-only routes are:

- The parent `SKILL.md`, `surrealdb/GUIDE.md`, `reference.md`,
  `content/routes/tasks.md`, `content/catalogs/surrealql.md` and `content/PROVENANCE.json` own coverage/pins.
- `surrealdb/content/capabilities/` briefs named `surrealdb.graph-modelling`,
  `schema-and-constraints`, `values-and-types`, `querying`, `transactions-and-concurrency`,
  `features-and-builds`, `connecting-and-engines`, `sdk-crud`, `bulk-writes-and-upsert`,
  `real-time`, `api-and-files`, `security-and-capabilities`, `full-text-search`, `vector-search`,
  `iso-gql`, `errors`, `core.embedding` and `server.frontends` provide the per-mechanism contracts.
- `surrealdb/skill_improvement/evidence/probe-results.json` records named SB/SX cases;
  `content/index/behaviors.tsv`, `values.tsv`, `gql.tsv`, `sql-features.tsv` locate coverage and
  upstream test references. Briefs are not identical in freshness to the expanding catalogs.
- `surrealdb/content/corpus/surrealdb-core/src/doc/edges.rs`,
  `exec/planner/source.rs`, `kvs/tests/graph_semijoin_test.rs` and
  `surrealdb-types/src/kind/literal.rs` supply inspected graph/schema implementation evidence.
  Their upstream counterparts are under tag
  [v3.3.0](https://github.com/surrealdb/surrealdb/tree/v3.3.0/surrealdb).
- `surrealdb/content/corpus/surrealdb/Cargo.toml.orig`, `surrealdb-core/Cargo.toml.orig` and
  `repo/Cargo.toml` distinguish client/embedded feature closure and workspace dependencies.
- Neo4j `content/capabilities/gds-community.md`, `enterprise-single-instance.md` and
  `skill_improvement/evidence/probes/probe-results.json` identify edition and projection evidence;
  `compare/bench/README.md` describes the intended benchmark, not a usable measured receipt here.

Context7 used the resolved official `/surrealdb/docs.surrealdb.com`, `/neo4j/neo4j` and
`/neo4j/docs-operations` IDs for traversal, transaction isolation, SDK/build/schema/query placement
and Neo4j representation/deployment. It returned mixed-era examples, including memory versioning
and 2.x feature spellings. Such examples are not recommendations for 3.3.0. No external evidence
used here establishes product performance, full backend fidelity or scientific correctness.
