---
title: "Plan68c: SurrealDB v4 query improvements"
status: in-progress
date: 2026-11-07
doc_role: plan
doc_owner: docs/plans/68-surrealdb-v4-cutover.md
doc_topics: [current-work, design, performance, storage, validation]
review_sources: [docs/functional_reviews/surrealdb-v4-cutover-2026-11-07.md]
---

# SurrealDB v4 query improvements

## Target and baseline

**Proposed.** Preserve the meanings and bounds of existing canonical and private
queries on the new engine. Use explicit read-only transactions for a current private
knowledge read. Qualify v4 access paths on existing workbench, readiness, graph,
artifact and evidence operations. Use the existing diagnostics to observe what the
engine did. Evaluate faster graph predicates and optional ranked discovery on
owned fixtures before selecting either for ordinary use.

The source baseline is `6da5f6289` with patched SurrealDB 3.3.4. Execution assumes
the [Plan67](67-storage-lifecycle-and-build-reuse.md) fixture, resource and retirement
contracts have been delivered. Plan66 continues to own scientific permission,
protected custody and developer evidence. Its current study, result and developer
consumers supply the receiving cases. Use the exact private 4.0.0 candidate from
[68a](68a-surrealdb-v4-supplier-runtime-and-extension.md) and its selected schema and
transport from [68b](68b-surrealdb-v4-schema-transport-and-authority.md). The coordinator
owns the disposition of [O02](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O02),
[O05](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O05),
[O06](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O06),
[O07](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O07) and
[O11](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O11). The review's
official v4.0.0 memory fixtures establish selected plan shapes. Qualification uses
the owned candidate, current PSE schema, actual clients and matched v3 controls.
[Plan68 coordinator](68-surrealdb-v4-cutover.md) ·
[Plan66e protected data](66e-protected-data-and-result-delivery.md)

## Implementation and investigations

### C01 — Private knowledge snapshot

Change `EvidenceStore::knowledge_select` in the existing private store to submit
`BEGIN READ ONLY` in its bound SurrealQL script through the canonical gRPC client.
The 4.0.0 statement gives the enclosed reads one snapshot and refuses writes. The
BEGIN slot returns snapshot status in place of the plain BEGIN value. Account for
that slot in the response and preserve the penultimate page and terminal COMMIT
contract. Verify current snapshot status, expected cardinality and terminal
completion before using the page. Select no positive `MAX STALENESS`. The 0.14.1
gRPC request contract has no read-only option for this path.

Use the same snapshot for interpretation marker, active-generation pointer,
membership and provenance. Preserve the generation, issuer and exact question in
the cursor. A switch of active generation between pages requires the existing
changed-generation refusal where the selected pointer differs. Keep
`FOR UPDATE`, lease and ticket acquisition, capture, publication and other guarded
effects on their existing transaction paths. Keep canonical graph anchor
before/after checks where they detect changed owner or revision.

Use the existing private-store scoped-role fixture to compare Subjects, Graph and
Productions pages, full membership, key order and the current response ceiling.
Exercise no match, changed generation, foreign cursor, attempted write, malformed
or stale BEGIN status, and missing terminal result. Also prove the following
admitted read still works after an interrupted request. If the owned gRPC route
cannot supply the read-only status and refusal contract, preserve the existing
v4 plain BEGIN path and record the specific supplier or receiving limitation.
A regression in supported reads or settlement blocks their cutover.
[Knowledge query and envelope](../../xtask/src/development/knowledge.rs) ·
[Scoped-role fixture](../../xtask/src/development/evidence_tests.rs) ·
[Read-only BEGIN](https://surrealdb.com/docs/reference/query-language/statements/begin#read-only-transactions) ·
[Tagged gRPC server](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/server/src/rpc/grpc.rs)

### C02 — Catalog access paths and index readiness

Run the same prepared operations, bound values and roles on the v3 baseline and
fresh v4 schema. Use the v4 bytecode executor's documented key lookups, range and
ordered-limit paths, permission filters and index plans where the existing query
shape allows them. Capture `EXPLAIN FORMAT JSON` and opt-in
`EXPLAIN ANALYZE FORMAT JSON` through the existing observation owners. Analyze an
effect-free read only on an owned fixture. For a read predicate inside a writer,
explain the same bound read in isolation and test the writer through its actual
admission. Record index selection, range, residual predicates, sort and pushed
limit where present. Treat absent counters as unavailable. Plan shape supports a
work hypothesis. End-to-end latency, allocation or storage claims require matched
measurements across equivalent inputs, roles and outcomes.

| Current receiving operation | Comparison and required invariants |
| --- | --- |
| Study workbench `select_rows` | Compare sparse and dense branch/optional-filter selections, early and late key cursors, ties and permitted or refused actors. Preserve exact pages, order, limits and original result locators. Verify whether uncorrelated work runs once and whether a residual predicate prevents ordered-limit pushdown. |
| `StudyCandidatesQuery` and `READINESS_CANDIDATES_SQL` | Exercise both cancelled and eligible paths and the composite `study_ready_candidates` range over study, eligibility and ordinal. Validate empty selection, boundary and late pages, and simultaneous readiness changes. Candidate order and original scientific policy must survive planner changes. |
| Protected result, artifact and analysis lookup | Exercise bound record identity, artifact/ordinal and analysis/input access with missing and foreign keys and selected versus denied roles. Preserve protected read tickets, precise failure, original Arrow/Proto interpretation, stream extent and release. Compare plans separately from user-visible proof of exact returned rows. |
| Private evidence activation | Retain `SELECT count() AS n FROM evidence_facts WHERE generation=$generation GROUP ALL` and the unique `(generation, member)` index. v4 may select `IndexCountScan` on the matching prefix without record fetch. Exercise empty count, nonempty underfilled or extra membership, and concurrent staging and retirement through the same activation and readback guards. Keep zero-member handling explicit. Keep scientific seals that also sum bytes or batch counts on their full verification path. |

Fresh schema installation and index readiness are separate checks. Use
`INFO FOR INDEX ... ON TABLE ...` before admitting a consumer that depends on a
deferred or concurrent build. Test missing, interrupted or failed construction,
identical `DEFINE INDEX OVERWRITE`, changed definition and analyzer changes.
The tagged unchanged-index case can retain an existing ready build. Require
effective definition and readiness, and explicitly verify any rebuild required by
a changed analyzer. Compare a new expression, array or conditional-unique index
only when a concrete existing predicate or identity warrants it. Cover `NONE`,
`NULL` and duplicate keys before adoption. Bitmap fusion already exists on the
baseline. Qualify the actual v4 plan rather than requiring a particular operator
for every workload. Keep the equivalent existing layout if an alternative lacks
benefit or weakens guards.
[Workbench source](../../crates/pse-codegen/src/codegen/surreal_study_workbench.rs) ·
[Study candidates](../../crates/pse-operations/src/canonical_studies.rs) ·
[Artifact and analysis controls](../../crates/pse-operations/src/canonical_analyses.rs) ·
[Evidence activation](../../xtask/src/development/evidence.rs) ·
[Evidence index owner](../../xtask/src/development/catalog/evidence.rs) ·
[Executed plans](https://surrealdb.com/docs/reference/query-language/statements/explain) ·
[Index selection and counts](https://surrealdb.com/docs/reference/query-language/statements/define/indexes) ·
[Tagged unchanged-index test](https://github.com/surrealdb/surrealdb/blob/v4.0.0/language-tests/tests/language/statements/define/index/overwrite_unchanged.surql)

### C03 — Graph receiving and bounded diagnostic projection

Drive the six existing relation-table statements in
`CanonicalGraphRequest::sql` through the actual reader and its
`inspect_graph_explanation` diagnostic. Cover study prerequisites and consumers,
run sources, artifact consumers, and analysis sources and targets. These statements
use explicit B-tree indexes and a page-plus-one sentinel. Preserve selected node,
directions, parallel occurrence identities, ordering, scoped continuation, and
the distinction between a protected locator and permission to fetch it. Exercise
an isolate, a late or exhausted page, an invalid cursor, a changed revision and
denied actors. Compare a real read and the same bound plan under the same role.
A plan cannot prove graph completeness or authority.

Separately exercise the current `KnowledgeMode::Graph` traversal over
`knowledge_subject_edges`. Reuse `knowledge_explain` for the static plan and
execute its bound item query for analysis inside the owned fixture. Keep the
existing public knowledge explanation read path static. The official v4 memory probe observed
`Values → Sort → Limit` with graph counters attached to `Values`. Its missing
predicate-scope marker did not establish payload pushdown. Extend the existing
bounded `graph_plan_observations` selection to retain allowlisted graph-specific
counters when they appear on `Values` with no selected attribute. Keep missing
fields distinct from measured zero and preserve the current nesting and operator
limits. Test an explicit payload and record scope, counter-only `Values`,
omitted counters and projection bounds. Reuse the existing raw plan output to
check observations against what the engine returned.

Use the exact knowledge traversal with early and late pages, selective relation
and basis, parallel edges, overflow hubs, folded and delta entries, denied fields
and generation switches. Compare the current query with an inline predicate
candidate only where its actual filter can be evaluated before record fetch.
Adopt only with exact authorized results, same continuation behavior and
observed improvement on the named workload. Otherwise retain the qualified
v4 query and record the comparison. Inline and lightweight relations are already
available in the old family. Keep stored occurrences. Defer materialized
back-references, new GQL projections and live or durable change delivery until a
named consumer and an equivalent scope or recovery contract exist.

The standard v4 build includes ISO GQL for multi-hop and shortest-path
patterns. SurrealQL remains the general query language and directly serves
the six indexed relation readers. The pinned 0.14.1 native gRPC service has
no dedicated GQL method. The SurrealQL `eval::gql` bridge requires enabling
dynamic evaluation, while the native HTTP GQL route admits both reads and
mutations. Keep the canonical build on its existing bounded transport.
Revisit the supplier feature for a named pattern that simplifies a current
consumer, with a bounded scoped read and a matched comparison.
[GQL overview](https://surrealdb.com/docs/learn/querying/gql/overview) ·
[Tagged server features](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/server/Cargo.toml) ·
[Tagged native gRPC implementation](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/server/src/rpc/grpc.rs) ·
[GQL capabilities](https://surrealdb.com/docs/learn/security/authorization/capabilities#gql-entry-surfaces)
[Canonical graph owners and tests](../../crates/pse-operations/src/canonical_explanations.rs) ·
[Private knowledge traversal](../../xtask/src/development/knowledge.rs) ·
[Tagged v3 inline tests](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/kvs/tests/inline_props_test.rs) ·
[Tagged v4 graph tests](https://github.com/surrealdb/surrealdb/tree/v4.0.0/language-tests/tests/language/graph) ·
[Tagged materialized reference authorization](https://github.com/surrealdb/surrealdb/blob/v4.0.0/language-tests/tests/language/statements/define/field/materialize/authenticated_read.surql)

### C04 — Lexical discovery on the existing corpus

Qualify the existing generation-scoped `text @0@ $text` path using the generated
`knowledge_excerpt_text` index and `knowledge_text_v2` analyzer. Compare the
fresh v4 posting and filtering plan against the same bounded structural population
and key-ordered v3 cursor. Verify no match, analyzer boundaries, ties, updated and
deleted terms, contributor/source attribution, denied access and mixed-generation
refusal.

Compare a separate score-ordered `search::score(0)` BM25 top-K only on that
authorized population. Check the actual score-ordered plan for native top-K
optimization and use a stable key tie-breaker. If adding deterministic order or
scope filtering defeats the intended plan, assess the complete candidate anyway.
Use independently expected matching identities and judged discovery examples.
Adopt a separate opt-in bounded read only when a real knowledge consumer benefits
and the scoped workload shows useful ranking or less work. Otherwise retain exact
lexical lookup. Ranked discovery grants no rerun, interpretation or evidence
authority. Keep on-demand scoped aggregation first. Defer maintained summaries,
ANN/hybrid search and geospatial indexing until a named workload justifies a new
derived or retrieval owner. The withdrawn Plan67 history pilot supplies no
production consumer for its compound expiry index.
[Knowledge query](../../xtask/src/development/knowledge.rs) ·
[Analyzer and index declaration](../../xtask/src/development/catalog/knowledge.rs) ·
[Full-text ranking](https://surrealdb.com/docs/learn/data-models/full-text-search/scoring-and-ranking) ·
[Maintained view semantics](https://surrealdb.com/docs/reference/query-language/statements/define/table)

### C05 — Spill, mixed load and physical observation

Reuse the native service's owned `--temporary-directory` and `TMPDIR`. Qualify
4.0.0 sort and grouping spill within a disposable generation. The distinct
and hash-join spill implementations belong to GQL, which the selected server
feature closure leaves disabled. Retain their tagged supplier evidence for a
future GQL consumer.
Exercise `SURREAL_MEMORY_LIMIT`, `SURREAL_QUERY_MEMORY_MINIMUM` and
`SURREAL_QUERY_MEMORY_MAX` as query controls alongside the independent MX and
host bounds in 68a. The v4 cooperative limit can wait and overrun and supplies no
filesystem quota. Verify the effective limit and path before admission. A bad
directory must not silently disable the selected protection. Test a successful
spill and physical drain, cancellation, invalid and full scratch, interruption
with residual allocation, owner-issued cleanup and fresh reopen.

Use the existing query diagnostics and Plan67 allocated-byte census for this
case. Compare native query, storage and compaction observations through the
existing scoped metrics or OTLP route without exposing root credentials or
unbounded labels. Keep counters distinct from allocated bytes and physical
release. Compare actual coexistence of long scans, index builds and small
canonical writes using `SURREAL_MORSEL_PLACES` and `SURREAL_MORSEL_OLTP_RESERVE`
before choosing any non-default scheduler setting. Keep the existing host
resource selection. Keep the qualified default when tuning demonstrates no
benefit. An unresolved resource or settlement regression blocks the affected
supported route. An unselected tuning option does not hold the cutover.
[Native launcher](../../crates/pse-operations/src/native_service.rs) ·
[Plan67 resource retirement](67e-resource-retirement-and-storage-verification.md) ·
[Runtime limits and scheduling](https://surrealdb.com/docs/reference/cli/db/environment-variables) ·
[Tagged observability contract](https://github.com/surrealdb/surrealdb/blob/v4.0.0/doc/OBSERVABILITY.md)

**Implemented, private candidate.** The native owner records explicitly
selected query-memory values as positive integer byte counts and forwards
only those values across its cleared environment. It checks the private
scratch directory and a small synced write before starting the server.
The prepared 68a generation is now available. Live spill, cancellation,
mixed load and allocation readback remain open on that generation.

**Tested.** Native `pse test pse-operations --lib -F
pse-operations/native-service -E
'test(=native_service::tests::native_service_query_memory_preserves_only_explicit_byte_limits)'`
passed one test with zero failures against the zero target and complete
settlement. It covers byte-setting validation and original record readback.
[Tagged distinct spill](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/core/src/exec/vm/distinct_spill.rs) ·
[Tagged join spill](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/core/src/exec/vm/join_spill.rs)

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| C01 | Complete for the scoped snapshot adoption | 68b bounded client, current private knowledge fixture | `BEGIN READ ONLY` is in use on `main`. The snapshot-status and terminal-slot control and the live scoped-role and cursor-fence selection exercised the existing knowledge path. Broader consumer coverage follows the owning query packages. |
| C02 | In progress for matched evidence | Matched v3 baseline, 68b definitions, 68d fixture selection | Selected v4 catalog, index and count controls ran. Compare equivalent access paths and measured work on the existing lanes. Expand protected-result cases as [66e E02–E03](66e-protected-data-and-result-delivery.md#packages) supplies the actual reader and [66b](66b-contracts-generation-and-interpretation.md#packages) its interpretation. |
| C03 | In progress for optional graph comparison | Existing scoped canonical and private knowledge readers | The native graph receiving control and bounded `Values` counter projection ran on the existing paths. Compare an inline predicate on the current private corpus. Wider product traversal, materialized references or GQL needs an actual scoped consumer and equivalence evidence. See the [Plan66 dependency map](68-surrealdb-v4-cutover.md#dependencies-on-plan-66). |
| C04 | In progress for optional ranking | Current private corpus, C01 snapshot, C02 index readiness | The selected v4 lexical controls exercised the current scoped search. Compare deterministic BM25 top-K on the same corpus and decide whether it improves the named workflow. Wider product or developer discovery needs its own ready consumer. |
| C05 | In progress for live qualification | Available 68a runtime and Plan67 retirement controls | The launcher's explicit query-memory and scratch wiring is implemented, and the byte-limit control passed. Exercise spill, cancellation, crash residuals, cleanup and mixed load before crediting a resource benefit or adopting non-default scheduler tuning. |

Use targeted `pse` selections for the existing canonical graph-explanation,
study-candidate and analysis controls in `pse-operations` and the native scoped
evidence and knowledge selection controls in `xtask`. Select exact filters and
fixture roles from the completed Plan67 inventory. Arrow-consuming correctness
runs use explicit force-validation. Record fixture generation, roles, native
server receipt, command, actual result and failure count against zero. Give
[68e](68e-surrealdb-v4-fresh-generation-and-acceptance.md) the required
receiving outcomes separately from optional query choices and measured benefits.

## Private candidate checkpoint — November 7

**Implemented.** The private knowledge query reads one current snapshot
and validates the BEGIN and COMMIT slots. The existing native fixtures
now request evidence count and lexical and graph plan observations. The
canonical graph projection retains allowlisted counters on `Values`
without assuming predicate placement. The deployed graph and lexical shapes
are the current comparison baselines for the remaining benefit work.

**Tested, supplier interface.** The official v4.0.0 embedded memory CLI
returned one matching member and an explicit zero for a missing prefix
using the generation count shape and compound unique index. This exercises
the count semantics independently of the candidate transport.

**Tested, targeted.** Native `pse test xtask --lib --features evidence-tools`
selected `knowledge_read_only_snapshot_requires_current_typed_status_and_terminal_completion`.
Native `pse test pse-operations --lib` selected three graph projection
controls for counter-only `Values`, explicit scope and projection bounds.
All four tests passed with zero failures against the zero target.

**Tested, compile.** The native `pse check pse-operations xtask --all-targets
-F pse-operations/native-service,pse-operations/canonical-maintenance,xtask/evidence-tools,xtask/host-contracts`
selection passed with explicit Arrow force-validation, zero errors and
warnings against the zero target, and complete settlement. Selected live
catalog, lexical and graph controls then ran on the exact prepared release.
The graph fixture separated catalog installation from permission and read
assertions, and its targeted rerun passed. The live private knowledge read
is covered by [68d](68d-surrealdb-v4-developer-and-test-receiving.md#private-candidate-checkpoint--november-7).
The package table owns the remaining matched comparisons and live resource work.
