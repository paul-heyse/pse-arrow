# Post-Plan44 retained results and publication inquiry

Supporting inquiry for [the principal review](../../reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md). The principal owns the integrated judgment and follow-up; this file records a bounded source/library assessment, not a competing disposition ledger.

## Status, scope and evidence basis

Focused independent design advice for coordinator integration; target purpose, not a subsystem acceptance verdict. Examined repository baseline: `91fd91513`. The existing unrelated dirty environment, tooling and Python-test edits were preserved. No repository files were edited; no builds, tests, probes or service operations were performed. This document was saved on the coordinator's subsequent instruction, outside the repository.

Applied the common worker and design-reviewer contracts, design-review and process-simulator skills, core 3.4, Heuristics for Efficient Architecture 1.0, process-simulator 1.5 and the repository binding. The applicable architectural concerns are responsibility ownership, composition, semantic authority, local reasoning and execution fit, especially AP-01/AP-02/AP-03/AP-04/AP-06/AP-07, H5/H7/H8/H12/H15/H16/H18/H19/H23/H26/H28. Scientific preservation includes PS-10/PS-11/PS-12. Prior ADRs and plans describe the current design; they were not used to prohibit consideration of a better target.

Inspected Plan44e/f, `benches/benches/native_process/results_analysis.rs`, actual `build/p44-results-resweep-2vhngy5m/cases` observations/memory/audit/inspection/profile artifacts, runtime results/snapshots/projection/artifact/analysis/physical-publication performers, canonical result/artifact/analysis/transport/file performers, their SurrealQL generators, substrate indexes and adjacent Python consumers. No stale library-utilization or capability-map document was consulted. A lightweight memory registry search for Plan44/result-analysis terms returned no relevant hit; no historical memory fact supports this inquiry.

Actual direct library pins inspected in `Cargo.toml`: DataFusion 55.1.0; Arrow/Parquet 59.3.0; object_store 0.13.2; datafusion-tracing 55.0.0; SurrealDB SDK 3.3.0 from the repository vendor path with protocol-ws/rustls. No dependency change is proposed as a prerequisite.

The strongest targets are repeated canonical metadata discovery, repeated artifact conversion, and per-row reconstruction during result analysis. These are source-established structural opportunities; their latency benefits remain unmeasured. The 21.368-second source-to-read chain cannot be classified wholesale as overhead or compared causally with the 0.274-second native interval.

## Concrete current optimization targets

### 1. Let typed selection determine its frozen inventory before discovery

**Diagnosis — Implemented, source inspected.** In [connected_results.rs:967](../../../../crates/pse-runtime/src/workflow/connected_results.rs), `result_selection` constructs a complete `ResultSnapshot` for the relation/range. `output_results` calls `results` first and only then attaches output, field, partition and predicates. [ResultSnapshot::capture:65](../../../../crates/pse-runtime/src/workflow/result_snapshot.rs) therefore fetches block pages and file-descriptor pages for every relation block intersecting that range, even for an absent or narrow output.

Consumption performs another metadata walk: [next_checked_body:469](../../../../crates/pse-runtime/src/workflow/connected_results.rs) calls `result_block_page` for unrestricted reads or `result_output_page` for selected reads. Unrestricted reads repeat discovery already represented in the immutable snapshot. Selected reads discover broadly first and narrowly second. Exact operation owners are [result_block_page:1048](../../../../crates/pse-operations/src/canonical_results.rs), [result_file_batch_descriptors:1142](../../../../crates/pse-operations/src/canonical_results.rs), and [result_output_page:604](../../../../crates/pse-operations/src/canonical_results.rs).

**Target — Proposed.** Keep one exact protected `ResultRead`, carry complete selection intent into inventory capture, retrieve its matching block/file closure, and let unrestricted readers iterate the frozen inventory. Avoid a new global cache when operation-local reuse removes this work. Preserve the distinct public Rust contract of `snapshot()`: it currently exposes the relation and row range, not the reader's output predicate. Changing that meaning requires an explicit migration.

**Preservation and adverse controls.** Retain exact historical attempt/manifest, original ordering and coordinates, missing ranges, IEEE values, index-versus-IPC verification, protected future unread files, late errors, cancellation and escaped allocation ownership. A narrower inventory is not permission to omit admission. Test absent output, empty row range, narrow time range, scalar finite/missing predicates, late page failure, corrupted index, historical attempt, cancellation and an unexecuted native plan surviving producer retirement attempts.

**Smallest discriminating probe — Proposed, not run.** After one setup, reuse the existing 80-state/25-output fixture. It has two dense logical batches but one selected batch. Observe construction separately from first batch and exhaustion for `results`, `output_results(x73)` and an absent selector; compare query-builder/statement deltas and exact output digests. A later >64-block fixture tests page growth but is unnecessary for the first discriminator. Dense time bounds differ from scalar finite-value bounds and must remain distinct.

### 2. Share protected selection and validation across promotion and graph construction

**Diagnosis — Implemented, source inspected.** [analyses.rs:945](../../../../crates/pse-runtime/src/workflow/analyses.rs) admits results, retrieves source revisions, snapshots and decodes dispatch relations. It then invokes scientific-artifact promotion. [artifacts.rs:517](../../../../crates/pse-runtime/src/workflow/artifacts.rs) independently admits the same results, retrieves the sources again, reads selected scientific relations and encodes standalone artifact files. Analysis subsequently reads every copied artifact member, decodes it and admits it again before graph construction.

Independent artifact ownership is valuable: analysis remains usable after producer retirement. That guarantee does not require every internal boundary to rediscover the same selection or reconstruct identical rows.

**Target — Proposed.** Compose promotion internally with the already protected selection, emitting validated scientific evidence with final artifact coordinates during the copy pass. Activate the graph only after artifact publication and exact ownership-transfer acknowledgements. Keep the public promotion operation independently callable. Reuse should not substitute mutable borrowed producer state for durable retained evidence.

**Preservation and adverse controls.** Producer retirement racing admission; cancellation before/after artifact activation; lost acknowledgement; graph-publication failure; conflicting scientific-family dispatch; partial/event trajectories; exact original row/field locators; explicit unsettled recovery and final release/drain. Current protection through the complete transfer remains necessary.

**Larger alternative — Proposed, substantial contract work.** Independently attach already sealed files rather than decode/re-encode them. [ArtifactRead::block:309](../../../../crates/pse-operations/src/canonical_artifacts.rs) currently requires standalone batch index zero, and [surreal_files.rs:fn::pse_files_v1::attach](../../../../crates/pse-codegen/src/codegen/surreal_files.rs) requires original file owner kind and selection to match. Sharing requires explicit packed-batch and ownership changes, not just another borrower. Compare extra retained backing bytes against removed copy/encoding/publication work. The lower-risk shared-pass composition should be examined first.

**Probe — Proposed, not run.** Once-generate a retained trajectory, observe protected-selection/dispatch, promotion, graph assembly and graph admission as separate child scopes. Count result admissions/source lookups, file opens, decoded/encoded bytes and canonical request builders; preserve all original artifact-coordinate assertions. Compare a shared-pass prototype against the existing complete operation with cancellation and uncertain-acknowledgement controls. Do not merely time a helper encoder.

### 3. Prepare evidence joins once

**Diagnosis — Implemented, source inspected.** [ResponseEvidence::transient_recorded:343](../../../../crates/pse-runtime/src/workflow/analyses.rs) searches `samples.iter().any(...)` for each response row. `project_response_evidence` invokes it throughout the response loop. Headers, contexts and endpoints are also repeatedly filtered. Scalar family/certification checks rebuild run/step groups for repeated response coordinates.

**Target — Proposed.** Build operation-local exact sample membership and run/step qualification groups once, retaining existing per-response decisions and locators. An ordinary Rust keyed representation is sufficient. DataFusion joins are an alternative if the evidence becomes part of an existing columnar analysis pipeline, but converting this bounded typed evidence just to delegate membership may add work.

**Preservation and adverse controls.** Conflicting/duplicate context checks, every qualification condition, original time equality, unit/output identity, withheld claims and distinct quantitative locators. A bitwise float key must deliberately preserve current equality semantics, including signed zero. Test missing sample, mismatched unit/time/output, conflicting header, duplicated valid sample, partial endpoint and withheld response. Do not infer derivative quality from graph reachability.

**Probe — Proposed, not run.** Graph assembly over already admitted evidence at 400 versus 2,000 response/sample rows; compare exact graph semantics and membership-comparison work. Current end-to-end observations do not isolate this CPU work. There is no requirement for a persistent evidence cache or new registry.

### 4. Remove duplicate standalone IPC preflight

**Diagnosis — Implemented, source inspected.** [decode_result_file:399](../../../../crates/pse-runtime/src/workflow/result_files.rs) runs `preflight`, then calls `read_file_batch`, which calls `open_file_reader`, which runs the same preflight again over identical immutable bytes/schema.

**Target — Proposed.** Open once with `open_file_reader(..., Some(1))`, then retain the existing native Arrow batch, row-count and schema checks. Arrow remains the decoder; bounded preflight still protects allocations before decoding.

**Preservation and adverse controls.** Huge/corrupt footer length, malformed nested buffers, extra batches, wrong schema, wrong row count and exceptional IEEE payloads. This removes demonstrably repeated work; it is not a measured explanation for multi-second publication.

### 5. Batch repeated graph-admission lookups inside the current transaction

**Diagnosis — Implemented, source inspected.** [surreal_analyses.rs:77](../../../../crates/pse-codegen/src/codegen/surreal_analyses.rs) receives bounded node/edge pages, then loops through edges resolving endpoints, analysis inputs and artifact-block row coverage. Many edges reference identical artifact blocks and shared context/endpoint nodes. One RPC does not eliminate repeated internal queries.

**Target — Proposed.** Resolve distinct endpoint and artifact/block coordinates once per page, verify each edge against those facts, and use native set-oriented insertion where exact immutable replay and relationship constraints permit. Keep bounded metadata transactions and activation's complete-membership reconciliation. [canonical_analyses.rs:28](../../../../crates/pse-operations/src/canonical_analyses.rs) already performs once-only edge-readiness classification through `EdgePublication`; retain it.

**Preservation and adverse controls.** Missing/cross-analysis endpoint, missing artifact membership, out-of-range evidence row, conflicting replay, duplicate edge, retirement races and lost acknowledgement. Client validation does not justify removing server verification. Native array insertion/selection is a capability lead, not a measured complete replacement.

Graph page reads call `available`, which uses `FOR UPDATE` on the analysis header. This is a candidate independent-reader contention question, not an established bottleneck. A replacement must preserve page admission versus retirement ordering. Do not remove the guard on latency intuition alone.

**Probe — Proposed, not run.** On the same retained analysis, compare 64-edge pages with repeated versus distinct artifact-block/endpoints; inspect actual query plans and server-reported wall time, then compare grouped validation with exact failure controls. For pagination growth include empty-result queries and more than one analysis, so indexes are tested against irrelevant rows. Compound indexes already exist in [substrate.rs:117](../../../../crates/pse-schema/src/catalog/substrate.rs) and [substrate.rs:141](../../../../crates/pse-schema/src/catalog/substrate.rs); propose no new index until actual access evidence identifies a gap. The current block-page query already places its cursor before the manifest upper bound because of pinned SurrealDB range selection behavior.

### 6. Preserve DataFusion optimizer intent through decoding

**Diagnosis — Implemented, source inspected.** [SnapshotProvider:354](../../../../crates/pse-runtime/src/workflow/result_snapshot.rs) correctly advertises `Unsupported` filter pushdown. Projection/limit are delegated to `StreamingTable`; its partition first gets a full checked batch through `read_block`. Narrow native queries therefore decode/admit complete relation batches before projection.

**Target — Proposed.** First remove duplicate metadata walks. Then assess projection-aware decoding and sound output/time pruning under a deliberate projected-registry contract. Arrow 59.3.0 provides `FileReaderBuilder::with_projection(Vec<usize>)`; DataFusion distinguishes `Exact`, `Inexact` and `Unsupported`. Native optimizer intent should reach the producer only where its semantics and required assurance can be preserved.

Two constraints prevent a simplistic pushdown claim. Current `FieldCheckedBatch` admission expects the full relation schema. Also [canonical_files.rs:677](../../../../crates/pse-operations/src/canonical_files.rs) hashes the entire physical file when opening it. Column projection alone therefore does not remove whole-file integrity I/O. `ResultSnapshot` caches an opened file reader, not every decoded result batch.

**Preservation and adverse controls.** Residual filters, filter-before-limit, null/missing and finite/time semantics, original metadata, ordering, signed zero, duplicate rows, shared pool and stream cancellation. Test predicates whose columns are absent from output projection; compare native plans and actual decoded/file bytes against the Unsupported reference. A scalar selected-output query and a generic native filter are not identical contracts.

`DataFrame::cache` materializes reusable results; cloned plans/providers do not promise cached results. Use it only when reuse warrants retention. Do not replace bounded ownership with unconditional full materialization. Existing physical-plan tests use configured runtime state, functions, pool and frozen inventory, an important strength.

### 7. Optimize cold physical publication without overlooking warm reuse

**Diagnosis — Implemented, source inspected.** [put_physical_rows:3488](../../../../crates/pse-runtime/src/workflow/durable.rs) serializes each physical relation to derive content identity, then serializes it again for publication. Each emitted block performs a separate `store.edit`, followed by a context receipt edit. Semantic relation ownership has become serialization and transaction granularity.

**Target — Proposed.** Compare once-encoded bounded spool plus grouped source admission against double encoding/per-block edits. Preserve selective replay, exact relation identity, structural references and atomic package visibility. Grouping unrelated lifetimes indiscriminately would lose useful isolation; any spool needs charged retention and drain.

This is a cold-path target. [physical_cache.rs](../../../../crates/pse-runtime/src/workflow/physical_cache.rs) already retains exact document admissions and row receipts, compares original owners/sources, coalesces document admission and freshly protects reused receipts. It is incorrect to describe repeated preparation as universally rebuilding physical data. [worker.rs:909](../../../../crates/pse-runtime/src/workflow/worker.rs) already groups physical source documents into one package edit; do not generalize per-block IPC behavior to that path.

**Probe — Proposed, not run.** Separate same-runtime warm preparation, fresh-runtime same physical input and changed physical rows under unchanged nominal compiler context. Count encodes and canonical requests; include cancellation and source reclamation. A single cold source-to-read case cannot establish warm-path cost. Preserve exact source checksums and fresh current authority; content identity alone does not grant reuse permission.

### 8. Use stored-analysis reopening explicitly

**Diagnosis — Implemented, source inspected.** `analysis(key)` reopens an active graph without rerunning scientific analysis. `result_analysis` creates a fresh artifact and fresh analysis occurrence. [blueprint §20.3](../../../../docs/authoritative_design/sections/identity-and-publication.md) separates occurrence identity from scientific meaning.

**Target — Proposed consumer guidance and conditional design.** Repeated consumers should retain/reopen the existing key when they want the same occurrence. If repeated creation of different selections over identical evidence is useful, compare shared prepared evidence or a complete base graph with separately admitted occurrences. Do not silently return an old occurrence from explicit creation.

Root selection currently occurs after complete graph assembly. Exact reachability can need full input; pushing roots earlier is not automatically sound. Reuse of stable preparation is the safer first alternative. Introduce extra cache/materialized graph state only if actual repeated demand earns lookup, retention and invalidation.

**Probe — Proposed, not run.** Compare explicit `analysis(key)` reopen with fresh `result_analysis` on the same retained attempt, separating header/page reads from creation. Verify distinct occurrence identity for explicit creation, unchanged scientific graph meaning, independent owner retirement, and method/configuration/result-manifest changes preventing inappropriate reuse.

## Existing strengths and fixes to retain

Exact immutable result snapshots; native Arrow random access and per-snapshot file-reader cache; original indexed output coordinates checked against IPC; bounded graph pages; independent artifact retention; explicit uncertain-publication recovery; configured DataFusion state/shared pool; bounded projection retention with actual occurrence identity; physical admission/receipt reuse; once-only graph edge-readiness classification; separation of semantic reuse, allocation ownership and durable retention.

Local result table discovery does not encode payloads. `RunResult::table_by_id` supports weighted projection retention and verifies membership/intrinsic failure even on cache hits. Do not propose a blanket new result cache without recognizing this implemented path. Fresh stored result readers and scientific-result projection reuse have different owners/lifetimes.

The Plan44 result resweep's original scientific, exact transport, pagination, index and cleanup assertions should remain the preservation controls. The target is to remove repeated physical work while retaining the meaning and assurance those controls establish.

## Current artifacts and interpretation

Primary artifact directory: [build/p44-results-resweep-2vhngy5m/cases](../../../../build/p44-results-resweep-2vhngy5m/cases). Relevant files are `case-audit.json`, each case's `*-observations.jsonl`, `*-memory.json`, `diagnostics/observation.json`, `diagnostics/audit-outcome.json`, `diagnostics/terminal-inspection.json`, CPU capture metadata and `.perf.profile.json.gz`.

The three retained-read once-only intervals were 0.231960785 s, 0.623354882 s and 0.535450632 s. They cover, respectively, 400/2,000/480 dense rows and 861/2,583/1,021 graph rows over 14/42/16 graph pages. The matched 80-state five/25-output controls share native step/setup counts; the 96-state case changes the mathematical problem. These are not statistical speed comparisons or isolated graph/kernel timings.

The 80-state five-output chain records 21.368206581 s. Named owner intervals include modeling source publication 3.882030359 s, simulation preparation 7.860528460 s, simulation/publication 3.625076986 s, analysis activation 3.446489984 s and original transport oracle 1.426283784 s. They are inclusive/nonexclusive owner observations; do not sum them into exclusive CPU attribution. Whole audit observations of 22.309–60.674 s include fixture setup/retirement and differ from requested access intervals. The original checks deliberately read/compare source/result/artifact content; that assurance work is not automatically production overhead.

The invocation's native worker interval is 0.274080009 s in the publication-chain record, nested within invocation and publication scopes. It cannot serve as a same-scope replacement timing for the full chain. Required durable publication has its own 1.850411407 s recorded interval in that invocation; this is not the full cold source/publication lifecycle.

All four original audit cases report completed status, zero assertion/diagnostic/cleanup failures, one actual native join each, and final charged bytes zero after retained runtime teardown. Logical reservation peaks around 14–40 GiB and RSS around 375–398 MB concern different populations; neither establishes a narrower sound admission allowance.

## Existing instrumentation and remaining tooling needs

[canonical_transport.rs:328](../../../../crates/pse-operations/src/canonical_transport.rs) records `canonical.query.builders`. [CanonicalStatistics:505](../../../../crates/pse-operations/src/canonical_transport.rs) records received/succeeded/failed/timed/untimed statements, failed requests and `canonical.query.latest_sample` with actual per-response server wall nanoseconds. These are not server CPU intervals. Counters currently lack a query-owner label, so an isolated call or narrow child operation is needed before assigning counts to a particular performer.

In the publication-chain terminal inspection, related operation sequence 1328 aligns by phase order and clocks with analysis activation. It records 191 query builders and 912 received statements, cumulative_partial=false. This mapping is an inference from execution order/timing; operation kind is generic `workflow.activity`. The simulation operation sequence 1319 has partial cumulative counters in that terminal snapshot and must not be summed as complete evidence. Related result access sequence 1332 reports seven builders/53 statements for the response-oracle access, not the whole result chain.

[pse-engine/session/assurance.rs:86](../../../../crates/pse-engine/src/session/assurance.rs) already enables DataFusion planning phase-only spans, physical operator metrics and preview_limit(0). Full optimizer-rule tracing is not a prerequisite for the initial experiment. Native query metrics must be interpreted at the correct completion/drop boundary; streaming does not bound all blocking operator working memory.

The publication-chain CPU capture covers ten seconds and is marked Partial/source_partial=true with sampling_loss_unknown=true. The Samply profile has meta.symbolicated=false, 273 samples over eleven client threads, and raw address names for its major sampled frames. The separate symbols extraction resolves only a few Tokio/WS-router frames; this does not supply whole-profile attribution. The ten-second timeline is Partial with fifteen open intervals. These artifacts do not establish complete client CPU, owned-server CPU, phase attribution or diagnostic overhead.

First use existing counters and narrow operation boundaries. If attribution remains material, schedule phase-aligned, symbolized captures of both the actual client and authenticated owned server, preserving process identity and clock uncertainty. Compare ordinary contract diagnostics with selected detail capture to isolate observer interference while preserving scientific assessment. Add decoded/encoded bytes or query-owner labels only where the next concrete question cannot be answered otherwise; no general tracing platform, permanent execution accounting, environment attestation or long statistical campaign is required by these architectural judgments.

## Library documentation calls and outcomes

Loaded `.codex/skills/datafusion/SKILL.md`, `.codex/skills/datafusion-tracing/SKILL.md`, `.codex/skills/neo4j-surrealdb/SKILL.md` and the Context7 MCP skill. Used the current offline pinned contract briefs for DataFusion pushdown/consumption/storage reuse, Arrow file projection, tracing metrics/planning phases and SurrealDB SQL/index capabilities. The checked pin match prevents importing current-main signatures as exact-release proof.

Context7 call 1: `mcp__context7__resolve_library_id`, libraryName `Apache DataFusion`, query `DataFusion Rust 54 physical plan reuse SessionContext DataFrame cache collect execute_stream TableProvider filter projection limit pushdown metrics`. Outcome: selected `/apache/datafusion`, official repository, High reputation, 8,611 snippets, score 61.62. The discovery query initially mentioned 54; actual inspected consumer and offline reference are 55.1.0, so that query is not version qualification.

Context7 call 2: `mcp__context7__query_docs`, libraryId `/apache/datafusion`, language Rust, query `DataFusion Rust TableProvider scan projection filters limit supports_filters_pushdown for remote data sources, execute_stream versus collect and DataFrame cache`. Outcome: official current-main provider guidance, per-filter pushdown classification, lazy stream consumption versus buffered collection. Its current-main scan signature uses `Option<&[usize]>`; actual consumer/offline 55.1.0 contract uses `Option<&Vec<usize>>`. Recommendations were checked against the actual pin, not transferred mechanically.

Primary links returned:

- [DataFusion custom table providers](https://github.com/apache/datafusion/blob/main/docs/source/library-user-guide/custom-table-providers.md): scan intent, pushdown classification and lightweight planning.
- [DataFusion DataFrame API](https://github.com/apache/datafusion/blob/main/docs/source/library-user-guide/using-the-dataframe-api.md): execute_stream versus collect.
- [Exact Arrow IPC 59.3.0 documentation build](https://docs.rs/crate/arrow-ipc/59.3.0/json): offline captured `FileReaderBuilder::with_projection`; checked against local pinned source at `arrow-ipc-59.3.0/src/reader.rs:1183`.

Context7 call 3: `mcp__context7__resolve_library_id`, libraryName `SurrealDB`, query `SurrealDB 3.3 Rust query pagination range record ids transaction bulk inserts embedded RocksDB`. Outcome: selected `/surrealdb/docs.surrealdb.com`, official docs repository, High reputation, 21,358 snippets, score 81.42; `/surrealdb/surrealdb` was also returned.

Context7 call 4: `mcp__context7__query_docs`, libraryId `/surrealdb/docs.surrealdb.com`, language Rust, query `SurrealDB record id ranges versus LIMIT START pagination ORDER BY index SELECT EXPLAIN and batched transaction INSERT`. Outcome: official docs for native insert/manual transactions, START/LIMIT, record-id ranges and EXPLAIN ANALYZE. These are discovery leads, not evidence that record-id range syntax replaces this repository's hashed keys or that a particular query plan uses the existing indexes. The pinned offline 3.3.0 reference independently supplies EXPLAIN/WITH NOINDEX probe evidence and record-range endpoint semantics.

Primary links returned:

- [SurrealDB Rust insert](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/reference/rust/methods/insert.mdx).
- [SurrealDB Rust transactions](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/reference/rust/concepts/transaction.mdx).
- [SurrealDB START](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/reference/query-language/clauses/start.mdx).
- [SurrealDB query optimisation](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/learn/querying/concepts-and-guides/query-optimisation.mdx).
- [SurrealDB EXPLAIN](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/reference/query-language/statements/explain.mdx).

No recommendation asserts measured superiority for a different store/backend. SurrealDB and DataFusion already provide useful capabilities here. Replacing the database would be a separate investigation of the complete consumed transaction, identity, lifetime and failure contract.

## Plan44 reconciliation and rule impacts

[Plan44e](../../../../docs/plans/44e-short-run-investigation-and-optimization.md) correctly records the corrected 80/96-state retained-result matrix, preserved original assertions, inclusive clocks and cleanup. [Plan44f](../../../../docs/plans/44f-integrated-acceptance-and-follow-up.md) owns assembled acceptance and wider quality/publication boundaries. This inquiry neither invalidates those completed controls nor reclassifies them as performance qualification. It examines newly identified structural work beyond the specific corrections that those controls accepted. Previous claims that the short matrix settled its selected cheap discriminators should not be expanded into a claim that all publication/result execution-fit opportunities were exhausted.

All remedies and new experiments here are Proposed and unscheduled until coordinator/operator disposition. Source establishes the described repeated work; supplied artifacts establish only their named Tested/Measured observations. No proposed remedy is Implemented/Tested/Measured by this inquiry. No broad scientific-model or whole-product adequacy verdict follows from result/storage inspection.

Local duplicate metadata-walk removal, keyed evidence lookup and duplicate preflight removal can preserve existing contracts. Shared validated promotion/graph composition needs careful integration but need not change independent artifact meaning. Shared-file artifact retention changes ADR-0182/ADR-0187's copied standalone artifact realization and blueprint §20.4; projected registry admission changes its affected schema/validation contract. Removing per-page `FOR UPDATE` requires an explicit equivalent admission/retirement ordering argument and controls. Silently reusing old analysis occurrences changes §20.3/public creation semantics, whereas explicit `analysis(key)` reopening needs no such rule change. Grouping physical source publication must preserve declared selective replay and coherent activation; if those contracts change, record the affected authority before implementation.

These are proposed rule impacts for coordinator disposition, not applied authority changes. Exact source/artifact links above support integration into the root's principal review; this supporting inquiry is not a competing status ledger.
