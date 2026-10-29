---
title: SurrealDB state, execution and coordination boundaries
status: proposed
date: 2026-10-29
doc_role: review
doc_topics: [architecture, storage, libraries, current-work]
---

# SurrealDB state, execution and coordination boundaries

## Judgment and evidence boundary

**Proposed:** expand SurrealDB's role as the governing model for shared operational facts and complete state transitions. Keep scientific working representations and physical execution ownership native. Retain one canonical datastore on the current patched persistent MX deployment; do not introduce KV/MX tiering or a second writable authority by default. The largest immediate simplification is completing database-owned bounded progression, not moving numerical execution into the database.

Plan58 already selects this broad architecture, especially in D03. It needs more concrete placement, interaction and retirement decisions before implementing its durable wave. This review proposes those amendments; it does not amend plans, accept an ADR, implement a capability or qualify a deployment.

Source baseline: `6a445df51`, inspected 2026-10-29. Inputs include the maintainer's untracked [colleague 1](../surrealdb_design_external_thinking_1.md) and [colleague 2](../surrealdb_design_external_thinking_2.md) documents, preserved unchanged. All five supplied library overviews were read end to end across the review team: [SurrealDB](../libraries/surrealdb.md), [SurrealKV](../libraries/SurrealKV.md), [SurrealMX](../libraries/SurrealMX.md), [Surrealism](../libraries/surrealism.md), and [tooling](../libraries/surrealdb_tooling.md). Context7 discovery was followed by tagged primary sources and repository generators/patches for consequential details. Current documentation alone is not evidence of a pinned API or activated capability.

The selected standard is Core 3.4, efficient-architecture heuristics 1.0 and process-simulator 1.5 under the repository binding. This is an ecosystem-led functional-extension review, not a diagnostic audit. Independent engine, query/connectivity and domain-boundary contributions inform this integrated judgment. No tests, benchmarks, probes, builds, installations, schema changes, service operations or data resets ran. Source inspection supports the recommendations; all proposed benefits remain unmeasured.

## What already matches, and what needs changing

| Colleague proposition | Plan58/current source | Review disposition |
| --- | --- | --- |
| Shared facts, dependencies, eligibility, assignments and accepted outcomes belong in database-managed state | Coordinator and D03 already select this; current worker still sequences several canonical operations and interprets actions around preparation | Extend to complete bounded interactions and an explicit deletion boundary: O01 |
| Native arrays, tasks and solver state remain in Rust | A/C and the public database-free local path already do this | Retain; database intent cannot witness physical release |
| KV is the default durable home, with MX for working data | Not Plan58's selection: the source-built canonical server uses persistent MX1 and SSI | Keep MX now; KV is a concrete future whole-datastore alternative, O06 |
| MX and KV are not automatically hot/cold tables | Consistent with the selected server build and D's topology limits | Make explicit; no automatic synchronization or cross-engine atomicity is presumed |
| MX persistence permits durable authority | Already selected, including local persistence/SSI patches | Retain and distinguish deployment settings from stock adapter defaults |
| Immutable bulk results stay in protected files | E retains Arrow IPC; actual result projection also inlines small encoded blocks | Preserve the existing hybrid, not a blanket claim that every payload is external: O08 |
| Surrealism can host bounded transactional rules | Existing stateless readiness module already shares native Rust policy | Retain narrow module role; extend only for a concrete contract gap, O07 |
| Claim, compute, publish in short transactions | Broadly aligned, but D03 places preparation/resource admission before execution claim | Preserve that sequencing; distinguish a preparation candidate from execution authority |
| Database features can replace custom graph and observer machinery | Partly already adopted; further physical/query and reactive choices are underdeveloped | Develop O02/O03 rather than blanket rewrites |
| Performance ladder, 100 ms crossover, sixteen-lane group-commit benefit | Colleague 2 explicitly estimates costs; repository evidence does not establish those numbers | Do not adopt as thresholds or results |

The first proposal is valuable as an option map, not a prescription to split the existing authority. The second is closer to current deployment, but its categorical performance and engine claims need qualification. KV snapshot isolation does not imply that every possible claim invariant will suffer write skew: the invariant, read dependencies and contention records matter. MX avoids an LSM compaction path but still performs GC, snapshot/log maintenance and version reclamation. Neither engine removes all coordination or maintenance cost.

## Target placement: authority is distinct from representation

A fact can have one authority and several useful representations. A model revision may be canonical data, an immutable prepared Rust object and a content-addressed artifact. Only the canonical edit/publication operation changes its shared meaning; a worker's local object is a revision-bound execution representation.

| State or behavior | Proposed authority and placement | Native responsibility / limit |
| --- | --- | --- |
| Authored model/physical source, configuration and revisions | Persist and relate admitted domain records in canonical SurrealDB when shared/history is needed; retain registry-owned declarations and explicit revision semantics | Parse, type, compile and assess physical/scientific meaning through existing owners; database schema is not a replacement scientific interpreter |
| Study definitions, dependencies, realized points and policies | Canonical records/relations with indexed, bounded transition operations | Native callers construct scientific requests; recorded points remain replay authority |
| Eligibility, generation, assignment, claim, cancellation intent and publication acceptance | Short canonical transactions over the complete applicable invariant | Prepare and acquire actual resources outside transactions; stop native work and join it physically |
| Cross-process capacity or allocation promises, if a real consumer needs them | Explicit logical reservation/fence records may coordinate that promise | Actual CPU/memory admission and lifetime remain local; do not add a parallel global resource ledger without a retained need |
| Attempt lineage, useful calculation observations, accepted-result references | Canonical queryable history at meaningful analysis granularity | Produce original scientific assessments and useful progress; do not persist every implementation event |
| Mutable shared drafts, candidate graphs or session worksets | Same canonical database where queryability/atomicity removes real shared machinery; disposable scope can still use transactions | Single-owner temporary structures need no store; a direct MX working store is conditional, O09 |
| Immutable preparation artifacts and caches | Existing artifact owners; canonical references only where shared discovery/history is valuable | Salsa dependencies, compiled evaluators and compatibility-bound caches remain native; do not serialize live provider state |
| Solver vectors, Jacobians, factors, accumulators and iteration state | Native Rust/library working memory | Preserve numerical layout, reuse and class-specific solver semantics |
| Tokens, tasks, channels, file descriptors, native sessions and process handles | Native Rust/OS ownership | Database records describe intent/status; they cannot replace cancellation delivery, destruction or joins |
| Numeric result bodies | Existing protected Arrow IPC: bounded inline blocks or sealed files, with canonical manifests/lineage | Arrow/DataFusion interpretation, selective decoding, result custody and retained buffers remain with E |
| Surrealism module KV/controller memory | Reconstructible acceleration only | Never use as authoritative assignment, accepted outcome or rollback-protected coordination state |

These defaults cover both durable and temporary shared state. Persistence is not the only reason to use SurrealDB: indexed graph access, consistent multi-record transitions and inspection can justify it for ephemeral working datasets too. Conversely, sharing within one Rust operation does not automatically justify database mediation.

## Functional opportunities and recommended realization

<a id="O01"></a>
### O01 — Complete store-side progression with a native preparation handoff

**Recommend first.** A study worker should ask the canonical model for a bounded applicable action instead of owning a second interpretation of study traversal and progression. The intended workflow is select/progress a bounded population, return exact preparation premises or a settled/waiting outcome, prepare natively, claim against those premises, execute, and accept the outcome canonically.

Current source is a hybrid. `workflow/worker.rs::work_once_scoped` loops over `study_page`, `advance_study_readiness`, `study_candidates` and finalization. `work_study_candidate` fetches `study_scope`, evaluates `scope.action`, prepares/admits scientific work and evaluates an action again before claiming. The generated `surreal_studies.rs` functions and `pse-model::study_policy` already own much of the meaning. This is not evidence that all policy is duplicated: the same Rust policy is reused by native and WASM paths.

Extend that existing domain owner with complete bounded selection/progression and explicit continuation. A page with no executable work but remaining candidates is not global idle. Bind returned premises to issuing store, generation, point/dependency revisions, cancellation and relevant seed evidence. Rust prepares and prepays actual launch resources before the execution claim rechecks current canonical state. A preparation failure or stale premise must not strand a claimed execution. Do not add a universal next-action catalogue or generic workflow engine.

**Can shrink/remove:** outer study/readiness/candidate choreography, redundant authoritative policy interpretation and frontend retry decisions for transitions now completed by the store. Keep the native effect loop and scientific preparation. `CandidateGroup` is local suppression of duplicate preparation, not claim authority; remove it only if the new access path eliminates its wasted-work purpose or deliberately replaces that contention handling. Keep original request/claim identity for uncertain acknowledgements.

**Demonstration:** two workers traverse sparse eligibility with continuation; one loses its preparation premise to cancellation or a dependency revision; only the current authorized attempt claims and publishes. Later analysis reads the same attempt/source/realized-point lineage. Owners: D03/D04, A05 and C03/C05.

<a id="O02"></a>
### O02 — Govern dependency access and useful summaries through existing database structures

**Recommend with O01.** An operator and worker should use the same relationship meaning while obtaining different efficient projections. The current generator already emits `INLINE EDGES 64 INLINE REFERENCES 32` where declared and typed `REFERENCE ON DELETE REJECT READONLY` links. Do not propose these as absent features or replace the registry with hand-authored DDL.

Use the registry/generator to assess actual readiness predicates, dependency lookups, candidate ordering, indexed counts and summary queries together. Compare canonical transitions updating summaries explicitly with synchronous events or supported `AS SELECT` aggregates. Prefer the smallest realization that removes client-maintained indexes/counts while preserving exclusion, generation and deletion semantics. Event/view maintenance must agree with ordinary writes, imports/reconstruction and migration; it cannot be assumed to run on every ingestion route.

`LIGHTWEIGHT` relations are only a candidate for genuinely propertyless edges. Current study dependencies carry metadata and explicit dependent/predecessor identity, so this is not a drop-in migration. High-degree/skewed graphs need the intended access path, not merely graph-shaped records. Keep physical recycles/SCC/structural and numerical graph analysis native where it operates on the prepared model; database dependencies do not replace those algorithms.

**Can remove:** proven redundant relationship mirrors, duplicate-edge bookkeeping and client summary-maintenance loops whose complete semantics move into canonical operations. Do not delete compatibility/identity validation of external inputs merely because a typed link exists. Owners: D03, A02/A05, F workbenches and registry generation.

<a id="O03"></a>
### O03 — Notifications wake consumers; canonical state governs progress

**Recommend a minimal reactive design, conditional on the specific consumer.** Use canonical state plus bounded catch-up for study progression and passive inspection. A live notification is a hint; an observer that reconnects must reconstruct current applicable state without treating a missed message as lost execution truth.

Compare synchronous events for small transaction-local derived changes, asynchronous events for delayed idempotent database maintenance, and live queries/changefeeds for observation. Do not put native scientific dispatch in an event or turn event delivery into an exactly-once execution claim. The [event contract](https://surrealdb.com/docs/reference/query-language/statements/define/event) distinguishes originating-transaction behavior from at-least-once asynchronous execution, whose documented default polling interval is five seconds; changing that interval does not establish an application latency result. External effects are not rolled back with database writes.

Before moving a mutation into an event, verify the effective event execution privileges and preservation of actor/grant restrictions on patched 3.3.4. Do not assume an event applies ordinary caller permission checks in the same way as a direct mutation. This joins D02's actor-bound control contract; inspect the [tagged event definition](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/legacy/expr/statements/define/event.rs) and exercise the real selected actor before acceptance.

Use changefeeds only when the consumer needs retained changes, not merely latest state or current-work catch-up. Retain useful scientific event history and command receipts; removing a transport journal does not remove those independent meanings. Candidate deletions include duplicate progress mirrors, reconnect bookkeeping and local notification queues only when canonical queries/feed semantics cover their actual contract.

**Demonstration:** a notification is missed, a subscriber disconnects, and a replacement worker/observer recovers current work without duplicated execution. Owners: D02/D03/D04, C04 and A/E/F observation callers.

<a id="O04"></a>
### O04 — Reduce connection and conversion machinery through native capabilities

**Recommend a focused SDK comparison before freezing D01/D04.** Compact browsing/cancel/reconcile must construct from connection/authentication/bounds alone, without mathematics. Existing `canonical_client.rs` uses supplier protobuf/types over a custom bounded gRPC session and owned HTTP/2 driver. It is not an SDK client, but it is also not an independently defined scientific RPC service.

Compare the exact-release Rust SDK with that client's required operations, native `Value`/`SurrealValue`, shared handle/session behavior, bounded requests, statistics and streamed result finality. Keep constrained `DEFINE API` actor operations where they implement the full grant/request/receipt contract. Prefer one useful connection realization per responsibility over a compulsory universal transport abstraction.

A replacement is worthwhile if it removes supplier-covered connection, codec, subscription or request plumbing while preserving the remaining contract. Unknown committed outcomes, issuing identity, admitted response extents, deadlines and physical driver settlement still need explicit ownership where the SDK does not establish them. Progressive rows must not become execution authorization before the relevant statement/transaction succeeds. Compare exact SDK transport behavior rather than assuming every protocol streams identically.

The [tagged 3.3.4 query implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/src/method/query.rs) supplies `stream_items`, bounded channels and terminal statement items. Its stop is observed at output boundaries, and the returned stream does not expose a public physical-drain completion handle. A non-emitting execution phase can therefore outlive stream departure. This is a concrete remaining contract gap to assess, not a reason to discard all SDK functionality or to assume the custom client can disappear wholesale.

**Can remove:** redundant JSON/document round trips in native calls, duplicate value codecs and connection policy proven covered by the selected library; control-only Runtime construction and retired Python HTTP orchestration. Do not port every legacy receipt/currency guard. Preserve the patched API evaluation/conversion rollback distinction and intentional domain refusal semantics. Owners: D01/D02/D04 and G; retained-data streaming joins E.

<a id="O05"></a>
### O05 — Make repeated engineering analysis a query over the execution model

**Recommend alongside canonical progression.** A user should compare yesterday's and today's runs by authored revision, effective context, realized design point, actual attempt, scientific qualification and selected results. Record those domain facts explicitly in the same canonical authority; do not reconstruct them from task logs or maintain a separate research scheduler history.

MX snapshots/MVCC serve consistency and recovery, not arbitrary application historical queries. Explicit domain revisions and frozen analytical cohorts remain necessary even if KV historical versions are selected later. Effective time, recording time and storage version time have different meanings. Native readers select canonical lineage and then join protected result relations with DataFusion; the database need not become the numeric array engine.

**Can remove:** independently reconstructed execution-history mirrors, repeated hydration/filter loops and client materialized summary indexes when actual consumers migrate. Preserve independent scientific evidence/oracles and explicit frozen cohort protection. Owners: A02/A05, D03 and E01/E02. Related historical opportunities in the October22 engine review remain evidence, not automatically transferred implementation scope.

<a id="O06"></a>
### O06 — Explicit engine and deployment choices, not accidental tiering

**Retain patched persistent MX now; make alternatives concrete and conditional.** Build authority is `scripts/surreal_source.py` and its patch tuple: SurrealDB3.3.4, MX1.0.0, `storage-mem,cli,mcp,surrealism`, with SSI and the canonical synchronous-AOL/every-append persistence profile. The current build does not compile SurrealKV. Stock 3.3.4 adapter descriptions are not the effective patched behavior.

MX is a durable canonical option with the configured persistence policy. Keep its current-state working set within a supported memory/version-retention envelope; use existing storage metrics to inspect long readers, GC backlog, snapshots and queues. The [MX1 persistence source](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/persistence.rs) contains group-commit machinery. That mechanism does not establish sixteen-lane batching, application throughput or a power-loss guarantee.

Reopen a SurrealKV-backed canonical datastore for a named need: supported state exceeding the MX working-set envelope, storage-level historical reads, encryption/key management or different checkpoint/backup behavior. Compare the full invariant and operational contract. [KV1 transaction code](https://github.com/surrealdb/surrealkv/blob/v1.0.0/src/transaction.rs) provides snapshot isolation and explicit read-key conflict tracking; it does not provide general predicate SSI. Guard/contention records or altered transition design may make specific invariants safe, but that is deliberate work, not automatic equivalence.

There is no recommended dual canonical copy, automatic per-table MX/KV routing, or promotion pipeline. An embedded deployment can remove a transport/process boundary, but changes crash containment, connection ownership and resource competition. Reopen it only for a concrete deployment/workflow benefit; external server ownership is not intrinsically inferior. Backend construction, adapter work, compatibility, migration and supporting decisions would precede cutover. Owners: B04, D03/D05 and H.

<a id="O07"></a>
### O07 — Use Surrealism for shared bounded policy, not a second runtime

**Retain and refine the existing stateless readiness module.** `pse-study-module` already uses typed SurrealDB values, preflights bounded scope and delegates decoded facts to `pse-model::study_policy`. Its manifest forbids arbitrary queries, functions, networking and module KV. Ordinary SQL should continue selecting scope and performing effects where that fully expresses the transition. This preserves one policy source without granting the guest broad database access.

The [tagged host implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/surrealism/host.rs) derives module context from the invoking query and narrows capabilities. This supports composing suitable database work in that context; it is not a general guarantee that every effect inside a guest function is atomic. Any proposed write-capable module must demonstrate parent-transaction rollback, authorization, mutability metadata and complete bounded output on the patched server before widening current grants.

The [module KV implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealism/runtime/src/kv.rs) is a volatile host BTreeMap with its own locks. It is not MX and its updates do not enlist in record rollback. Controller-local memory is another separate lifetime. Prefer immutable version-keyed resources or pure memoization if either cache is justified; reject using them for work ownership, deduplication or publication authority.

**Can remove:** separate native/WASM implementations of a genuinely shared rule, repeated policy conversion where typed shared representations preserve exact integer/missing-value semantics. Do not promise eliminating all decoding, or move native solvers, provider handles, long calculations or OS supervision into WASI. Owners: D03, shared model/module/codegen owners; scientific providers remain A/C-owned.

<a id="O08"></a>
### O08 — Consolidate artifact meaning without relocating bulk computation

**Retain the protected IPC hybrid and simplify metadata consumers first.** `result_projection.rs` selects bounded inline encoded blocks or `reserve_result_file`/`seal_file` and canonical descriptors. `canonical_results::ResultRead` protects exact manifests and payload ownership. Expanding the canonical domain model should reuse these result references, not reencode tables as scalar database records.

Let D answer which outputs/attempts are applicable; let E perform protected selective decode, delivery and relational numeric analysis. This permits a data-only reader without a mathematical Runtime and avoids a Python holder merely retaining native resources.

Database buckets/file values are candidates for particular artifact-management consumers, not an automatic transaction across object storage and database records. Retain the explicit publication boundary: produce/seal immutable bytes, atomically accept their authorized reference, reclaim unaccepted or released bodies only with established ownership. A database record saying complete cannot substitute for completed file construction. Cross-store protection does not disappear by renaming a file a bucket object.

**Can remove:** duplicated artifact catalogues, data-only SourceServices/math construction and frontend materialization whose retained callers use the protected native reader. Payload protection, charged Bytes/Arrow ownership and delivery drain remain. Owners: D03, E01–E04 and G.

<a id="O09"></a>
### O09 — Transactional working collections only where they earn their boundary

**Conditional useful extension, including development/research.** Shared draft workspaces, candidate graphs or evidence relationships may benefit from atomic updates and indexed inspection even if reconstructible. Prefer canonical SurrealDB records when they need the same schema, permissions, relations or transition meaning.

Direct MX offers conditional mutations, multi-key transactions, ordered scans and borrowed-value access such as `with_value`. For a named byte-oriented working collection, it might replace bespoke synchronization with less database-layer work. It also requires key encoding, compatibility, any secondary indexes and a separate consistency boundary. Current scientific preparation uses library-specific immutable products, Salsa dependency tracking and native allocation ownership; none is shown to benefit from replacing it with an engine.

Select a direct working store only after showing the actual multi-key/query contract, owned lifetime and reconstruction/publication direction, plus which lock/cache/index mechanism is retired. A second store justified merely by hot reads or replacing a single-owner vector is declined. No independent dev knowledge/evidence authority is proposed. Owners: relevant A/F capability, coordinated with D; any new crate/dependency direction follows the decision route.

<a id="O10"></a>
### O10 — Schema evolution and storage health enable the expanded model

**Required supporting capability, not an exhaustive development recovery programme.** D05's native schema/version owner must cover supported predecessor/successor transformations, write fencing, quiescent native publication and stale open handles. Storage snapshots, KV checkpoints, restore tools and database introspection do not establish that domain contract.

Use the registry's generated DDL, canonical interpretation/version and existing maintenance owners. SurrealKit's frozen rollout, schema inspection and isolated actor/API tests are useful comparison candidates; they must not become a second schema declaration authority. Reuse engine/query metrics and selected `EXPLAIN` output to inspect canonical access and retention rather than reintroducing bespoke build observation.

Disposable development state can reconstruct only within an explicitly owned regenerable scope. Supported shared/non-regenerable deployment needs its later migration, backup/restore and recovery qualification. Preserve uniquely authored inputs. Database outage must not prevent native cleanup; reopening/reconstructing a catalogue must not silently authorize an old worker to publish.

**Can remove:** obsolete converter/operator experiments, repeated whole-schema proof on ordinary immutable reads and duplicate health mirrors where their actual contract is supplied. Keep dynamic write fences and unknown-commit reconciliation. Owners: B, D05, F03/F05 and H.

## Complete interaction and failure cases

The proposed durable interaction is intentionally more precise than 'claim, calculate, save':

1. A native caller submits complete source/point/dependency intent through canonical policy.
2. A bounded database operation advances applicable state and returns a preparation candidate with exact premises or continuation/wait/refusal. Discovery does not claim physical resources.
3. Rust obtains the admitted immutable inputs, resolves seeds and prepares executable work outside a database transaction; its original native owner admits and prepays required launch capacity.
4. A short canonical transaction rechecks current revisions, dependencies, generation, cancellation and the prepared evidence before granting execution authority.
5. Native workers execute without an open database transaction. Cancellation intent reaches their original token; lease expiry/reassignment never establishes that an old process physically stopped.
6. After original scientific assessment and necessary payload construction/drain, canonical acceptance checks the attempt/fence and attaches the accepted result references. A stale worker may retain diagnostic evidence but cannot overwrite the current accepted outcome.
7. Later clients query that same history and protected results. Notifications are optional wakeups; canonical catch-up remains sufficient for applicable work selection.

Pure/local operations keep their existing direct path. The new public KINSOL local-operation evidence from Plan58 remains valid for its named scope; it is not database-progression qualification.

| Revealing scenario | What delivered capability must establish |
| --- | --- |
| Sparse eligible page, high-degree dependency or cancelled study | Bounded progress plus continuation; no false global idle or whole-graph hydration |
| Two workers prepare the same candidate | Canonical claim excludes duplicate accepted authority; native admission remains valid, wasted preparation is handled deliberately |
| Input changes during preparation | Current claim rechecks premises; stale work cannot run/publish under an old revision |
| Cancellation after claim / worker lease expiry | Durable stop/assignment meaning is distinct from actual token delivery, native exit and settlement |
| Commit succeeds but ACK is lost | Reconcile the same complete issuing-store/actor/operation tuple; absent receipt is not rollback |
| Notification loss or observer replacement | Canonical catch-up reconstructs current state; feed replay is required only for a consumer needing historical changes |
| Module writes then returns/converts an error | Correct rollback on the effective patched server; intentional refusal and cache/network effects have explicit meanings |
| Result file seals but canonical publication fails | No accepted result is invented; owned orphan handling preserves uncertain publication identity and protected readers |
| Repeated daily study with a later correction | Explicit authored/context/attempt history and immutable cohort selections preserve scientific meaning |
| Migration with existing handles/attempts | Admission seals, allowed original settlement drains, stale writers fence, successor interpretation publishes only after success |

No new probe was necessary to choose the architectural direction. Exact SDK replacement, a new module effect contract, an event-maintained invariant or backend migration would need their focused interface/failure experiments before implementation acceptance. Performance experiments are optional decision tools: compare complete equivalent transitions, bytes/conversions and contention under the required durability. Do not infer latency from reciprocal aggregate engine throughput or impose a universal threshold.

## Retirement and Plan58 amendment map

| Owning scope | Proposed amendment | Candidate deletion / retained obligation |
| --- | --- | --- |
| Coordinator | Add explicit placement/engine/topology decisions and these opportunity dispositions | Keep one disposition owner; no new execution catalogue |
| A01/A02/A05 | Native source, working-draft and study interactions consume canonical revision/point/result meaning | Remove frontend history reconstruction; retain compiler/physical/numerical semantics and DB-free local execution |
| B04/B05 | Construct/own the selected patched server independently; engine changes require deliberate deployment compatibility | Retain actual process identity/drain; startup alone is not schema readiness |
| C03/C04/C05 | Native preparation/resource handoff consumes complete canonical premises/actions | Remove authoritative state sequencing now covered by D; retain joins, local admission, tokens and useful preparation suppression |
| D01/D02/D04 | Resolve SDK/API/gRPC responsibilities against actual custody and actor semantics | Retire covered conversion/control-only Runtime/Python transport machinery without hiding uncertainty |
| D03 | Implement bounded complete progression and exact prepared-evidence claim; co-design graph access and summaries | Shrink worker paging/readiness/action loops and proven redundant projections; preserve one shared policy source |
| D05 | Version/fence the expanded domain and supported open-handle settlement | Retire obsolete schema mirrors/converter branches only with retained callers moved |
| E01/E02/E03 | Canonical lineage selects protected IPC; independent native data-only construction | Retire mathematical construction for data reads and duplicate hydration/catalogues; retain payload/file/buffer custody |
| F/G | Use the same canonical model for retained development/inspection; remove replaced wrappers, journals and declarations together | No mandatory CLI, MCP, universal registry or second evidence store |
| H | Join actual actor, progression, scientific, payload and migration cases at scope end | Keep observed/implemented/proposed claims separate; qualify persistent-deployment envelope before non-regenerable/shared support |

Priority is O01 with O02's actual access paths and O04's connection contract, supported by O07 and the essential O10 version/fence boundary. O03/O05/O08 supply their real observer/analysis/data consumers. O06 keeps MX as the starting realization and names reasons to revisit it. O09 is conditional on a genuinely shared working collection. These can form coherent consumer cuts; they are not ten mandatory serial projects or automatically authorized packages.

## Architectural assessment, alternatives and limits

AP-04 is served by making definitions, dependencies, admissibility, assignments and publication a governing domain model rather than a collection of runtime callbacks. The schema alone is not sufficient: store operations must cover the invariant and native callers must consume their decisions. AP-01/AP-02 favor existing owners and a single policy source over another scheduler or universal service. AP-07 and the execution heuristics favor bounded set-shaped transitions, useful projections and native computational layouts over per-iteration persistence or one transaction per semantic object.

The process-simulator profile requires original physical conventions, domain validity, solver-class selection and independent original-space outcome assessment to survive. Transactional acceptance cannot make an invalid numerical result valid. Native arrays/factors and coupled structural analysis remain where their libraries can execute efficiently. Conversely, keeping those numerical responsibilities native is no argument for keeping a second durable eligibility/history authority in Rust.

A thin native worker still orchestrates physical effects. It must receive actions, prepare/admit work, execute, propagate stop, join and report original outcomes; neither a database row nor a WASM function owns those objects. The simplification is removing duplicated shared-state decision paths around that necessary effect loop. The exact amount of removable code must be established in each integrated cut, not promised from library feature names.

Alternatives considered: native-only durable coordination retains too much repeated shared meaning; putting every local operation behind the database adds an unnecessary dependency; a direct-MX application store discards useful database semantics and creates another owner; KV canonical authority can serve real future history/capacity needs but requires deliberate invariant/adapter/migration work; a rich Surrealism runtime adds sandbox/effect/serialization obligations where ordinary functions and a pure policy module already suffice. None is rejected because of licensing. No new administrative credential fallback, limits regime or background watcher is recommended.

This review leaves deeper SDK behavior, event/view incremental maintenance on actual write/import paths, large/skewed access shapes, useful working-store consumers and future KV migration as named investigations. They are not evidence of missing functionality or automatic implementation blockers. A favorable recommendation changes no rule or plan status. Selected architecture/engine/transport changes use the existing ADR/design and plan-creation routes before dependent execution; accepted ADRs remain immutable. The colleagues' documents and prior review evidence remain intact.
