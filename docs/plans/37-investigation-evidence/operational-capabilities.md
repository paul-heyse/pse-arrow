---
title: Plan 37 operational capability investigation
status: active
date: 2026-10-12
doc_role: evidence
doc_owner: docs/plans/37-surrealdb-lifecycle-and-integration.md
doc_topics: [current-work, storage, libraries]
doc_retention: while-dependent
---

# Operational capability decisions

[Plan 37](../37-surrealdb-lifecycle-and-integration.md) owns status, disposition and acceptance; [37c](../37c-tooling-and-operational-integration.md) owns the implementation contracts. The source-led inquiry below settles I2/I3/I6 and inventories actual I5 consumers. The subsequent selective fixture evidence settles the bounded I5 access-path decision; representative runtime/query performance remains unmeasured. Neither contribution authorizes optional adoption.

Baseline: `a6aedd31c51688e3686723306d7deac14bd4072c`, with concurrent tooling/resource-owner edits preserved. Source inspected on 2026-10-12. **Implemented, source-inspected:** consumer behavior below. **Interface-checked:** cited exact-release library contracts. Recommendations are **Proposed**. The initial source inquiry ran no builds, installations, services, probes or tests; the later fixture observations are labeled separately below. No representative performance was measured. OP01 selects fresh stores/evidence, RC05 retains patched WS, and local-first/truthful retry remain the supported boundary.

Context7 resolved `/surrealdb/surrealdb` and supplied current query/connection documentation. Its moving-main results were leads only; claims below use exact 3.3.0 source and the loaded neo4j-surrealdb skill's version qualifications, without a utilization catalog or capability-map refresh.

## I2 — Notification and follow-up effects

There is a concrete possible notification consumer. `crates/pse-runtime/src/workflow/worker.rs` declares `WorkerSettings.poll` with a 500 ms default; the serving loop sleeps for the configured interval or cancellation. `work_once_scoped` pages studies, advances persisted readiness and reads candidate pages. `crates/pse-operations/src/canonical_studies.rs::{advance_study_readiness,study_candidates}` and `crates/pse-codegen/src/codegen/surreal_studies.rs` own bounded readiness advancement, notification cursors and compare-and-set updates. These persisted notifications are domain readiness state, not subscriptions to database LIVE notifications.

Different repeated operations have different purposes. `canonical_staging.rs::acquire_source_stage` checks an exact source-edit receipt, attempts generation acquisition, checks the receipt again after races, and backs off under one original deadline; it cannot treat a notification as proof of a committed edit. `canonical_result_retention.rs::{forget_run_results,reclaim_result_page}` advances explicit retirement/collection state. `scripts/test_resources.py::reclaim_reports` retries bounded eligible cleanup and is called by `scripts/validation.py` after finishing the owned report. Supervisor polling observes process identity, service admission and physical drain; a database feed cannot replace those observations.

Exact 3.3.0 [WS session source](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/src/engine/remote/ws/mod.rs) clears live subscriptions on connection reset through `clear_live_queries`/`clear_pending`. [Changefeed documentation](https://surrealdb.com/docs/learn/querying/real-time/changefeeds) describes bounded feed retention; a consumer would need cursor persistence, duplicate handling and a full resync after a retention gap. Exact [event execution](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/core/src/doc/event.rs) implements finite queued-event retries, and [maintenance tasks](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/core/src/kvs/tasks/mod.rs) own their processing. Neither provides the scientific policy or successful eventual effect guarantee. Do not transfer the documented 3.3.1 write-conflict retry-budget change to 3.3.0.

**Decision:** keep explicit readiness advancement, bounded polling, receipt reconciliation and owner-driven cleanup; add no LIVE/changefeed/ASYNC dependency in this tranche. This is a consumer-specific non-adoption decision, not a claim that notifications have no consumer or cannot help.

**Alternative and implementation dependence:** an optional LIVE wake-up could reduce worker idle latency while durable readiness/CAS and periodic recovery remain authoritative. It requires explicit subscription teardown, reconnect/resubscribe and missed-wakeup reconciliation under B1's activity lifetime. No existing A/B/C correction depends on it. ASYNC is a separate decision requiring an identified repeat-safe post-commit effect; the inspected explicit retirement/publication owners do not justify moving their correctness into an asynchronous event.

**Observable revisit trigger:** measured idle discovery SQL load, an explicit responsiveness requirement below the configured poll interval, or a named repeat-safe DB follow-up whose current implementation is demonstrably redundant. A decisive wake-up experiment must interrupt subscriptions and miss a notification while proving recovery still discovers every eligible point. No latency/load claim is established here.

## I3 — Protected reads and alternative transports

`canonical_results.rs::result_output_page` is a concrete large-result consumer: it selects at most 64 output indexes and a bounded set of exact block descriptors before fetching original IPC content. It selects scalar rows or dense coordinate-block indexes, preserves ordered cursors and checks result-set/batch identities. Protected query framing checks the exact selection and expiry before committing. `ResultRead`/`ResultReadOwner`, block validation and the returned buffer ownership retain protection beyond an individual SQL response. Registry-owned `RESULT_BLOCK_BYTES`, `RESULT_INDEX_BYTES` and `RESULT_MESSAGE_BYTES` in `pse-schema/src/catalog/substrate.rs` supply finite boundaries; a complete trajectory can still contain many blocks.

`vendor/surrealdb/PATCH.md` and `canonical_transport.rs` describe the consumed production integration: original request clocks, bounded application/control admission, dispatch uncertainty, acknowledged session restoration, cancellation and physical driver drain. Disconnect is not proof of server/native-task abort. Cargo selects the vendored SDK 3.3.0 WS/rustls profile. An upstream SDK alternative cannot be substituted merely because it shares the version number.

Exact 3.3.0 [query API](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/src/method/query.rs) distinguishes awaited buffered results from `stream_items`; streamed rows are provisional until `StatementEnd`, whose error retracts preceding rows. Incremental availability is engine-dependent; an API stream over buffered results does not establish bounded whole-operation memory. HTTP/embedded/gRPC also have different session, transport and deployment boundaries. Direct embedded RocksDB is not an equivalent multiprocess local service architecture.

**Decision:** retain patched WS and the existing paged/block read as the comparator. I3 completes as a source-grounded non-replacement decision under RC05, with the actual protected-read consumer and full comparison obligations identified. No streaming or speed claim is made, and no production transport experiment is necessary to reconfirm the already selected mechanism.

**Alternative and implementation dependence:** isolated gRPC incremental-read or dedicated HTTP/WS comparison remains eligible. Any candidate must preserve value fidelity, statement/transaction finality, selection protection through consumer buffers, original clocks, unknown write settlement and physical drain. A streaming collector that holds every row is not a successful memory optimization. B1–B3 repairs remain necessary under the retained transport and do not wait for this comparison.

**Observable revisit trigger:** representative protected-result reads exceed an agreed memory/latency requirement despite finite page/block admission, or an actual consumer needs incremental rows before a page completes. The decisive experiment compares the same scientific selection and complete consumer journey, including cancellation/rollback and peak resident memory, in an independently owned fixture. It must not attach destructive transport experiments to someone else's service.

## I5 — Actual query/index consumers and next evidence

Source inventory identifies useful access paths rather than a general graph rewrite:

| Consumer and source | Actual query shape and declared index | Evidence implication |
|---|---|---|
| `canonical_selection.rs` and `canonical_capability_tests.rs` | Immutable membership interval selected by problem/scope/key, revision bounds and optional dereferenced kind; `membership_scope_pages`; inverse suppliers via target scope/name and `edge_target` | Existing tests compare ordinary, explicit-index and NOINDEX paths, exact result ordering, EXPLAIN and EXPLAIN FULL. They were read, not executed. |
| `canonical_results.rs::result_output_page` | Equality on result_set/output/partition, coordinate or row range and ordered cursor; `result_output_range`, `result_row_selection`; exact block lookup | The source explicitly rearranges cursor/range predicates for 3.3 compound-index behavior. Representative range/cursor selectivity is the useful next check, not generic traversal benchmarks. |
| `canonical_retention.rs::reclaim_page` | At most 64 closed membership candidates by problem/key, then exact guarded reachability/deletion | `membership_pages` bounds candidate discovery; LIMIT alone does not establish all scanned work or the cost of each reachability decision. |
| `canonical_result_retention.rs::reclaim_result_page`, generated retention functions | Exact run selection and persisted collection generation/cursor, bounded payload/index removal | Explain the actual selection subqueries on owned fixtures; do not run effectful collection merely to obtain an analysis plan. |
| `canonical_studies.rs`, `surreal_studies.rs`, runtime worker | Eligible study candidates ordered by ordinal; `study_ready_candidates`; bounded persisted notification advancement | Existing readiness-fanout test inspects a populated EXPLAIN FULL result. Worker discovery supplies a real consumer. |
| `canonical_analyses.rs`, substrate declarations | Analysis membership and directed source/target edges; `analysis_edges`, `analysis_edge_source`, `analysis_edge_target` | Edges carry analysis identity, kind and optional evidence; replacing them with unattributed adjacency loses meaning. |

All named declarations are in `crates/pse-schema/src/catalog/substrate.rs`; generator ownership remains authoritative. Exact [3.3.0 lightweight scanner](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/core/src/kvs/lightweight.rs) scans endpoint adjacency rather than a relation record range. Current `canonical_edges` stores source version, ordinal and logical target scope/name; it is not simply a pair of existing record endpoints. Analysis edges also have semantic properties. Neither is a fitting blanket LIGHTWEIGHT conversion. INLINE cannot turn arbitrary string/logical joins into cached endpoint traversal.

**Decision for this inventory:** keep the declared indexes/native functions and attributed relationships; do not adopt LIGHTWEIGHT/INLINE without a fitting consumer. **Remaining I5 evidence:** source inventory does not establish actual operator choices, examined work or scale performance for every result/retention query. Root should schedule selective INFO/EXPLAIN controls after final A declarations, reusing existing index/NOINDEX correctness controls and fresh generated fixtures. At this release the existing `canonical_capability_tests.rs::assert_executed_cardinality` explicitly treats EXPLAIN FULL as execution and distinguishes output cardinality from examined rows; no plan-analysis command should be assumed inert merely because its name contains EXPLAIN.

**Implementation dependence and revisit trigger:** optional access-path adoption requires the affected query/declaration generator owner and exact semantic equivalence tests. Observable trigger is a native plan showing an inappropriate scan/sort for a concrete candidate/read path, measured work exceeding its intended scaling, or a new genuinely property-free endpoint relation. Current declarations changing under A1–A3 require reinspection; no new index is justified by this inventory alone.

## I6 — Fixtures, inspection, diagnostics and wakeups

Existing native fixtures exercise the selected server, backend and supervisor identity. `scripts/tests/surreal_fixture_check.py` and `test_surreal_server.py` cover the operational route; C0/C1 add beta-specific namespace ownership and cleanup, rather than creating another service manager. Exact [testcontainers module v0.15.0](https://github.com/testcontainers/testcontainers-rs-modules-community/blob/v0.15.0/src/surrealdb/mod.rs) defaults to server v2.2, memory, root/root and log-based readiness. Those defaults do not qualify current 3.3.0 RocksDB schema/readiness or the custom lifecycle patch.

`surreal_server.py::mcp_authorization` validates private selected state, exact loopback `/mcp`, live listener and existing credentials before releasing VIEWER access. `scripts/tests/test_surreal_mcp.py::fixture_mcp_controls` already declares disposable ownership and independently checks forbidden mutation outcomes. Native MCP is an inspection surface; domain mutation tools would need to call the canonical operation owner. Launching stdio MCP over a new embedded store would change authority and is not an equivalent selected-service inspection route.

The supervisor already owns bounded rotated logs and explicit server resource/session settings in `server_environment`. Exact [3.3.0 telemetry source](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/server/src/telemetry/mod.rs) supports tracing/metrics composition; this does not identify a missing consumer for a mandatory collector. Inspection of `server_environment` found no explicit OTel collector/metrics setup there, which is narrower than claiming no external host diagnostics exist. Studio is an optional interactive client, not a schema or lifecycle owner.

Explicit report cleanup is already bounded and retried at validation completion. It can defer when a service is closed or ownership/drain is unresolved. A timer could wake that owner later but cannot make the resource eligible or restart another owner's service to dispose it.

**Decision:** retain native owned fixtures, existing selected-service VIEWER MCP, bounded logs and explicit sweeps. No testcontainers dependency, domain-write MCP, Studio requirement, collector or automatic scheduler is selected by this inquiry. C1's authenticated namespace lifecycle is a concrete implementation need independent of these optional capabilities.

**Alternative and implementation dependence:** adopt containers only for a named released-server/network reproducibility gap, with exact image digest, RocksDB profile, credentials/capabilities, schema-aware readiness and checked explicit cleanup. Domain MCP operations depend on A/B canonical contracts; diagnostics should expose a concrete owner outcome before adding export plumbing. Automatic maintenance needs a named product responsiveness requirement and retains existing eligibility/drain checks.

**Observable revisit triggers:** host binary/network isolation prevents reproducible tests; an operator needs a named domain action; an unresolved SQL/session/reclaim diagnosis lacks adequate current evidence; or eligible deferred cleanup repeatedly waits longer than an explicit product requirement. These lead to focused fixture/diagnostic/wakeup experiments, not a mandatory platform stack.

## Qualification boundary and handoff

Read-only repository searches and exact primary-source reads completed. Runtime controls, performance measurements, service effects and product qualification are **not_run**. Existing tests supply concrete reusable controls, not fresh passing evidence. I2/I3/I6 have bounded retain/non-adopt outcomes and observable triggers; I5 inventory is delivered with its selective runtime evidence still explicit. No capability absence or whole-system performance claim follows from the inspected scope. Root owns any resulting plan/status/decision updates and assembled acceptance.

## Selective I5 fixture evidence, 2026-10-12

**Tested:** `scripts/pse-env --native= --resource-class light -- python -m
scripts.tests.surrealkit_fixture_check --state /tmp/pse37-execution-SMD3bG/beta-service
--tool-root /tmp/pse37-execution-SMD3bG/tools --evidence
/tmp/pse37-execution-SMD3bG/generated-oracle-v4` completed with exit0 and zero
failures against released3.3.0 RocksDB in the explicitly owned beta tooling fixture.
The helper installed the complete production generated declaration directly,
compared its seven semantic expectations, and passed beta sync/replay/parity15/15.
It registered both namespaces before effects, checked cross-namespace credential
refusal, and proved exact namespace disposal while preserving the second fixture
until its own removal. This is owner-auth schema evidence, not record-user policy
or scientific qualification.

INFO readback found the declared membership, result block/row, readiness and
analysis edge indexes. Selective EXPLAIN selected `membership_scope_pages`,
`result_output_range`, `result_row_selection`, `study_ready_candidates` and
`analysis_edge_source` with the intended compound equality/range predicates and
finite limits. A64-row readiness fixture returned the same eight ordered ordinals
through normal, explicit-index and NOINDEX paths; EXPLAIN FULL reported IndexScan
output_rows8 and records_fetched0 for its covering ordinal projection. Those
metrics describe this tiny fixture, not examined rows at production scale or a
performance result. Other sampled range/edge paths were explained on empty tables.

**Decision:** retain the current access paths and attributed relationships; no new
index, LIGHTWEIGHT relation or INLINE cache is justified by these samples. This
settles the bounded I5 inquiry. Final A declarations still require selective
reinspection, and representative result/retention selectivity remains unmeasured.
The source-led triggers above govern any broader experiment.
