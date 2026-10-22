---
title: "Plan49 remaining scope and library integration"
date: 2026-10-22
doc_role: review
doc_owner: docs/plans/49-library-native-remediation.md
doc_retention: while-dependent
doc_topics: [design, libraries, storage, preparation, validation]
---

# Plan49 remaining scope and library integration

## Judgment, scope and evidence

**Proposed:** deepen SurrealDB integration through complete bounded study and protected-result
operations, make typed specialization incrementally derived in Salsa, and retain native
DataFusion predicate plans for the lifetime of a result selection. Add bounded graph
explanations as a useful product capability. Develop Z3 configuration exploration as a
separate proposed capability with an explicit selection boundary; it is not a prerequisite
for ordinary preparation, dispatch or numerical execution.

The strongest opportunity is composition of capabilities already present. The selected
SurrealDB server already contains SurrealMX 1.0, SSI, synchronous persistence, finite controls,
generated functions, graph relations, lifecycle events and integrated schema tooling.
Recommending these again would miss the remaining work. Complete operations can remove
client coordination while preserving semantic owners and physical custody.

This is the operator-requested combined reassessment of remaining Plan49 and functional
opportunities, with SurrealDB the primary ecosystem, Salsa and Z3 additional investigations,
and graph/data execution adjacent investigations. It has no diagnostic-audit prerequisite
and is not a completion audit. The operator permitted reopening implemented foundations
where materially beneficial and requested coordinated **proposed** plan revisions. Production
implementation remains paused. Generated historical data need not be preserved.

**Interface-checked:** clean `main` at `fa7868336`, 2026-10-22; all Plan49 companions,
coordinator handoff and source coverage; the revised in-use-library review and audit routes;
relevant current implementation; complete supplied SurrealDB, Salsa and Z3 overviews; linked
primary documentation, tagged upstream source and installed release source. Three independent
library investigations and an independent architecture challenge informed this account; the
principal reviewer reconciled their results with executing consumers. The challenge sharpened
per-item cursor progress, composite descriptor coverage, exclusive accounting and entity-fragment
root assembly; those corrections are incorporated below. Core 3.4, Efficient Architecture Heuristics 1.0, Process Simulator
1.5 and the pse-arrow binding govern the assessment.

No product tests, builds, service changes, dependency updates, store operations or benefit
measurements were performed for this review. Earlier receipts keep their original scope.
Descriptions of code below are **Implemented/source-inspected**, not new Tested claims.
All new capability and benefit recommendations remain **Proposed**. Capability orientation comes
from the supplied [SurrealDB](../libraries/surrealdb.md), [Salsa](../libraries/salsa.md) and
[Z3](../libraries/z3.md) documents; consequential followups used their primary links and exact
sources rather than assuming an overview declares the installed contract. Current scheduled scope,
package progress and opportunity dispositions belong only to the
[Plan49 coordinator](../plans/49-library-native-remediation.md).

## Reconciliation of the remaining series

The original authoring baseline `e08b1f1dd` is historical. Current implementation includes
changes described as future work in several companion baseline paragraphs. The pause handoff,
actual consumers and bounded inquiry outcomes determine what still needs doing; an
`in-progress` row does not mean that its entire mechanism remains unwritten.

| Area | Current foundation and remaining obligation | Review consequence |
| --- | --- | --- |
| 49a / GOV02 | STO01–STO08 have selected assembled evidence. MX1.0, SSI, authenticated controls, gRPC, native graphs and SurrealKit exist. Final remaining-target publication is open. | Preserve that evidence. Add focused STO09 operation composition; do not reopen the complete engine/transport cutover or call streaming integrated. |
| 49b | Exact batch facts, physical pruning and protected readers exist; substrate v8 deletes scalar locators. All-consumer result acceptance remains open. | Finish RES acceptance and add O02/O03; do not reintroduce locators, format migration or historical backfill. |
| 49c | Direct typed resolved producers/consumers, source maps, dependency indexes, retention/firewalls and tracked policy products exist. Full retirement and preparation acceptance remain open. | Update the historical baseline; separately select the Salsa patch and finer typed derivation, O04/O05. |
| 49d | Regional Symbolica construction, demanded derivative products, C++ ownership and focused controls exist. Current optional-family dispositions are recorded. | Preserve the mathematical target and scientific checks. Change consumed preparation granularity, not mathematical authority or derivative meaning. |
| 49e | Shared supervision/requests, parameter binding, settings-aware seam, native lifetime/status work and LM policy exist with bounded evidence. | Finish actual consumer migration/acceptance. Compose store-shaped study operations without inserting SMT in dispatch. |
| 49f | Strict enums, directional documents, keyed physical custody and covariance origin/PSD work exist. | Retain independent boundary/scientific acceptance; expose any selected new explanations/configuration documents through the existing catalogue. |
| 49g | Catalogue, Rust command owners, internal tooling split and guarded maintenance exist; TOOL06 retains the hub. | Extend actual product/query contracts and operator consumers. Avoid another catalogue, observation substrate or daemon. |
| 49h | Default Rust, Python and static closeout evidence is recorded; the broad native sweep was canceled. Representative fixture changes remain proposed. | Use representative, contract-directed selection and carry O08 into acceptance; preserve canceled/not-run distinctions. |

This preserves DA-001–DA-032, F01–F10, R1–R22, RC01–RC21 and S01–S11. The
[original source routes](../plans/49-source-coverage.md) remain useful evidence navigation,
not an additional current-state ledger. Other active plans retain their owners and scope.

## O01 — Complete bounded study operations inside SurrealDB

<a id="O01"></a>

A worker discovering or recovering a study currently pays client coordination per candidate.
In [canonical_studies](../../crates/pse-operations/src/canonical_studies.rs),
`readiness_scan_page` reads a bounded key page, then calls `study_scope` and
`repair_readiness_scope` per point. `advance_study_readiness` similarly visits dependent
points. The runtime worker reaches these paths during discovery and rotating recovery.

Select generated **scope-page and repair-page operations**. One protected scope page returns
candidates with their exact immediate decision premises; the existing pure Rust policy
computes actions; one bounded application submits those actions with generation, revisions
and complete read-set expectations. Each candidate reports accepted, stale or refused
explicitly. Advance a source cursor only over a certified examined prefix, or persist the
uncompleted per-item work; a stale member must not make the page silently complete. An unknown mutation response reconciles its operation receipt before another
effect is permitted. The cursor records examined work and continuation, including stale-false
readiness recovery; it is not merely a returned-row count.

Bound total candidate, edge, premise and encoded-response extent together. A 64-candidate
page is not permission for 64 independently maximal predecessor sets. Define deterministic
partial-page continuation without dropping an occurrence's required premises. Use short
transactions and explicit conflict scope; a whole-study transaction increases retention and
retry work. If page-wide contention repeatedly invalidates unrelated candidates, apply smaller
independent guarded batches while retaining bounded bulk retrieval.

This places data gathering and guarded mutation beside the database facts while preserving
`pse-model::study_policy::{AdmittedStudy,candidate_action}` as semantic owner. Pending scientific
admission remains pending; actual outcomes establish usability and seed permission. Preserve the notification source revision and before-cursor fences, and the policy
precedence of unknown effects requiring reconciliation before cancellation. Claim-time
fencing remains authoritative even after a successful readiness repair. Existing outgoing
notification cursors and recovery scans remain until their replacement semantics are established.

The alternative is compiling the **same** pure Rust policy to a Surrealism module and evaluating
it inside the transaction. That could remove the second crossing and stale-action interval.
It is a credible further pivot, not equivalent to independently translating policy into SQL.
Select the two-phase bounded route now; retain a focused Surrealism decision if WASI compatibility,
module ABI/identity, finite execution, installation and recovery make the complete transactional
route simpler. Native solver execution and physical drain would still have their current owners.
See [module architecture](https://surrealdb.com/docs/learn/extensions/guides/module-architecture)
and [3.3.4 module capability checks](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/surrealism/mod.rs).

**Delivery:** STO09 with RUN03/RUN05 consumers and QUAL02/QUAL03 scoped controls. Expected
benefit is fewer serial crossings and repeated scope acquisition, not a measured speedup.

## O02 — Return one protected result metadata product

<a id="O02"></a>

[Result snapshot construction](../../crates/pse-runtime/src/workflow/result_snapshot.rs) fetches
`result_block_page`, retains it, then fetches `result_file_batch_descriptors`.
[Canonical result opening](../../crates/pse-operations/src/canonical_results.rs) subsequently
reads the batch descriptor and physical instance descriptor. Existing `result_output_page`
already demonstrates database composition by returning indexes and blocks together.

Select one generated protected page operation returning selected blocks with their exact batch
descriptors. Deduplicate descriptors by stable file/batch identity inside the bounded product.
Return only needed metadata, retaining attempt/result-set identity, manifest membership,
ordinal/output/partition constraints, schema, interpretation, digest and extent correspondence.
Require exact one-to-one descriptor coverage of the returned blocks: missing or duplicate
descriptors are corruption, never an inner-join disappearance. Preserve `files.require_current`
and the existing batch validation. Charge the composite transport product and retained page
through consumption or cancellation.

An embedded physical file descriptor is a candidate, not automatic permission to remove
`descriptor_for_instance`: that operation checks current incarnation and physical state.
Remove a subsequent read only if an owner-bound admitted descriptor carries the same live
placement/protection premises through file opening. Retain ticket freshness, actual protected
file admission and maintenance serialization. A metadata join cannot establish immutable
physical custody by itself.

**Delivery:** RES07, coordinated with RES03/TOOL04 and targeted protected-read controls.
This needs no streaming transport or IPC format change. It removes a specific repeated crossing
while retaining existing integrity checks and original coordinates.

## O03 — Prepare DataFusion selection once, then consume batches

<a id="O03"></a>

The typed [ResultReader](../../crates/pse-runtime/src/workflow/connected_results.rs) reconstructs
`scalar_selection_expression` in its block loop. `block_may_match` invokes
`prepare_pruning_members` with one container, creating a physical expression and pruning predicate.
For a surviving block, `scalar_selection_mask` calls `evaluate_scalar_expression`, which creates
another physical expression. The SQL provider already prepares pruning across its frozen inventory.

Select an immutable **selection-local native predicate plan**: lower once under the actual
session/schema, retain `Arc<dyn PhysicalExpr>` and an optional `PruningPredicate`, evaluate page
statistics together and reuse the same residual expression on admitted batches. Bind it to the
selected relation, schema, complete predicate, actual session functions/configuration and pool
lifetime. Prefer selection ownership over a new global predicate cache.

The [55.1.0 session API](https://github.com/apache/datafusion/blob/55.1.0/datafusion/core/src/execution/session_state.rs)
returns the reusable expression; the [pruning implementation](https://github.com/apache/datafusion/blob/55.1.0/datafusion/pruning/src/pruning_predicate.rs)
accepts multiple statistics containers. Unsupported pruning keeps containers. Preserve all-None,
null/NaN/infinity/signed-zero behavior, full protected admission, original offsets and residual-before-limit.
Retain a lightweight scalar path if a single-block operation does not need pruning construction;
this is one semantic implementation with demand-specific preparation, not two predicate authorities.

SurrealDB narrows visible metadata and owner/partition scope; DataFusion evaluates scientific
row predicates over Arrow. Moving complete numeric payloads into SurrealDB would add a format,
codec and scientific interpretation migration without establishing a better route here.

**Delivery:** RES07 with RES02/RES06 acceptance. Count plan construction in targeted controls;
measure end-to-end latency only if making a quantitative claim.

## O04 — Update Salsa safely and make retention observations explicit

<a id="O04"></a>

The repository pins 0.28.4; the supplied overview describes 0.28.5. Its advanced incremental
features mostly already exist in 0.28.4. The patch adds validation of interned/memo access and
requires exclusive memory inspection: [changelog](https://github.com/salsa-rs/salsa/blob/salsa-v0.28.5/CHANGELOG.md),
[advisory](https://rustsec.org/advisories/RUSTSEC-2026-0308.html). Current interned keys and
implicit multiple-argument query keys do not satisfy the advisory's complete exemption.

Select exact 0.28.5 independently of O05. `CompilerWorkspace::retention_usage(&self)` and
`metadata_bytes(&self)` currently call the old `memory_usage(&self)`. The
[new contract](https://github.com/salsa-rs/salsa/blob/salsa-v0.28.5/src/database.rs) needs mutable
access, cancels readers and waits for other handles. Collect at an exclusive lane maintenance/
admission boundary, or return a clearly scoped last observation. Do not introduce a hidden
lock or retain a clone while waiting for exclusive access.

LRU discards values while retaining dependency metadata; backdating still needs the old memo
value. Inputs live until database destruction. Low-durability interned and stale tracked-output
reclamation already exist. Interning outside an active tracked query receives maximal durability/
revision; changing revision retention alone will not reclaim those keys. The direct
`complete_modeling_cancellable` → `grouped::complete` → `BodyKey::new` route is a concrete
lifecycle to inspect. Keep the shared-root census and bounded generation rotation until an
actual replacement covers those allocations.

**Delivery:** PRE01. Existing reuse/churn/oversize expectations remain; add exclusive observation
and reader-drain controls for the changed API. Dependency update is ordinary pinned-family work,
not a reason to reopen unrelated dependencies or introduce a new ADR.

## O05 — Make typed specialization an incremental entity graph

<a id="O05"></a>

The clearer typed producer makes this more valuable and feasible than the original speculative
I07. [specialized](../../crates/pse-compiler/src/workspace/modeling.rs) still specializes the
selected root when its checked package changes; `projection` rebuilds root indexes and its
identity graph. Unchanged canonical bodies already reuse `BodyKey` → `semantic_body`.
The opportunity concerns selected-root semantic derivation and projection facts, not a claim
that every reached edit rebuilds every mathematical evaluator.

Select query-owned typed derived entities keyed by project declaration/instance/occurrence
meaning, with separately tracked physical contract, resolved body, membership and diagnostic
fields. Downstream queries consume the fields they actually need. Keep deterministic unique
production and project IDs: Salsa tracked identity includes the producing query and identity
fields; it is not a durable model identity. See [tracked identity/lifecycle](https://salsa-rs.github.io/salsa/plumbing/tracked_structs.html)
and the [versioned API](https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html).

Separate numerical, guard/validity, incidence, topology and diagnostic projections. Continue using
petgraph/rustworkx for the actual algorithms. A source-only change can refresh attribution without
changing mathematical fields only when those fields contain no stale consumer-visible diagnostics.
Membership additions, deletions, absent lookups, provider contracts and finite scope stay tracked.
Root assembly still establishes conservation, complete coupled structure, deterministic order
and original-domain obligations over the actual selected root.

Define immutable entity fragments with their own local node handles and explicit root
import/reindexing. Track selected forms/presets, bindings, function/physical contracts,
documents and absence at entity scope; retaining a whole `CheckedPackage` dependency would
keep whole-root invalidation. Preserve connection/contribution obligations and accumulation
across descendants, including original signed roots at root finish. Migrate one coherent
typed producer and its consumers first, then retire the corresponding whole-root derivation. Per-instance immutable `Arc` queries are a simpler alternative where
tracked fields add no useful distinction. Avoid one query per edge or introducing immortal
inputs for every derived entity. Query-owned derived key production may improve reclamation,
but does not eliminate root metadata or justify removing finite policy.

`#[tracked(specify)]` can expose a batch analysis through entity-addressable outputs, but only
for entities produced by that invocation and without LRU on that query. Prefer tracked fields
or immutable batch results when simpler. Accumulators are eligible for advisory diagnostics,
not required admission/validity facts; cyclic Salsa evaluation is not a recycle or nonlinear solver.
Shared readers remain bounded to one immutable revision and must drain before writes. Persistence
stays unselected without a concrete restart-reuse case.

**Delivery:** PRE08, promoting I07 into a proposed target. PRE03/PRE05 remain useful foundations;
MAT consumes immutable demanded-region descriptions and keeps native compilation and mutable
evaluators outside tracked queries. Acceptance compares incremental and clean preparation after
local, membership, source-only and physical/provider edits, including root switching and deletion.

## O06 — Explore compatible configurations with Z3 and explain conflicts

<a id="O06"></a>

A credible new workflow is: an engineer supplies allowed methods/backends and finite restrictions
for several cases, asks which joint assignments are compatible, and receives either proposed
choices or attributed conflicting requirements. Existing routing answers concrete eligibility;
this operation asks whether any combination satisfies the joint restrictions.

Develop a bounded proposed configuration operation before selecting production integration.
Compare it with bounded enumeration using existing `capability_for(settings)` and typed
ineligibility reasons; a small finite choice set may need no native solver.
Inputs contain finite choices, hard permissions and explicit compatibility facts produced by
existing owners. `Solver::check_assumptions`, `assert_and_track`, `get_unsat_core` and `get_model`
provide the mechanism: [0.21.1 Solver source](https://github.com/prove-rs/z3.rs/blob/z3-v0.21.1/z3/src/solver.rs).
Bind both selected and rejected alternatives for a complete scenario. Cores are attributed
conflicts, not necessarily minimal or unique. Models propose choices; actual request admission
and revision-fenced application remain mandatory.

A satisfying assignment cannot predict scientific success, grant seed/start permission, turn
pending admission into accepted facts or override native capability. Author constraints in one
typed finite input vocabulary; consume compatibility results from current routing rather than
reimplementing scientific predicates in an independent SMT catalogue. No raw SMT or generic
callback policy language is needed. Pure encoding descriptions may be incrementally prepared;
solver execution, deadlines and timeout-dependent answers are outside Salsa tracked functions.

Rust binding `=0.21.1` and native engine versions are separate decisions. The tagged
[build script](https://github.com/prove-rs/z3.rs/blob/z3-v0.21.1/z3-sys/build.rs) can detect a
system installation or acquire another build, and metadata overrides do not prove linked runtime
identity. PRE09 must choose one pinned native acquisition mode and verify header/link/runtime
agreement. Context-bound ASTs stay within a worker; return owned domain choices/explanations.
Interrupt and join native work under original deadlines. `Unknown` remains inconclusive with
its reason, never infeasible or eligible; internal solver limits are not a hard memory envelope.

`Optimize` is a later choice only for authored preferences such as minimizing changed choices;
scientific requirements stay hard. Ordinary weighted tears already use HiGHS MILP with independent
`FlowGraph::witness`, and graph SCC/matching/topological analysis already has fitting algorithms.
Keep those routes. Development-time finite counterexample checks can be a smaller useful entry
when ordinary tables/property tests become unwieldy, but they establish the abstract encoding's
property, not proof of Rust or concurrent store behavior. Fixedpoint/SPACER remains a separate
verification lead: a reachable bad state has different query interpretation from a safe state.

**Delivery:** PRE09 supplies a concrete operation/acceptance and adopt-or-defer decision before
any new binding. A selected implementation joins RUN native ownership and BND/TOOL catalogue
consumers. This new functionality is explicitly proposed; RC16's already accepted policy
consolidation does not authorize it. Ordinary preparation and study execution do not wait on it.

## O07 — Bounded graph explanations using the existing graph stack

<a id="O07"></a>

Add useful typed questions: which immediate predecessor blocks an occurrence, which recorded
source produced a run, and which retained analysis consumes an artifact? Existing
`canonical_study_dependencies`, `canonical_run_sources`, `canonical_analysis_edges` and
`canonical_analysis_inputs` support native retrieval; existing flat pages and study scopes
provide foundations. Return edge kinds, identities and evidence coordinates, with explicit
neighborhood scope and partial continuation. Distinguish an observed blocker from a complete
explanation and obtain live status under the same selected revisions.

For compiler/structural work, retain `Dependencies` numerical/guard roles, petgraph BFS/SCC/
toposort and rustworkx deterministic topological ordering. The
[existing lexicographical routine](https://github.com/Qiskit/rustworkx/blob/0.18.1/rustworkx-core/src/dag_algo.rs)
already serves structural incidence. Share immutable views only when node/edge meaning, isolates,
ordering and provenance agree. Flowsheet topology, numerical incidence, study dependencies and
provenance are different graphs; connectivity does not establish physical independence or
scientific usability. Neither a universal graph nor an additional graph engine is selected.

**Delivery:** TOOL08 product explanation contract and operator consumer, with BND05/TOOL01 for
public documents and STO09 native access where relevant. Start with immediate blockers and
bounded source/consumer neighborhoods; do not require all-path enumeration or transitive closure.
ISO GQL is an optional query expression, not another graph or a required external API.

## O08 — Make remaining work and qualification representative

<a id="O08"></a>

Update each companion's current foundation and actual remaining cutover, without relabeling
historical probes or source inspection as acceptance. Keep a single live status owner. Order
new foundation work before its real consumers; finish known retirements alongside replacements.
Optional Z3/Surrealism decisions cannot delay settled native, boundary or result obligations.

Keep the representative [49h route](../plans/49h-qualification-and-design-publication.md#representative-functional-checks):
short legally admitted original windows, independent identities for variants, page-limit-plus-one
fixtures and fresh databases only where installation/lifetime/isolation is the subject. No false
schema fingerprint or weakened scientific assertion can replace genuine contract coverage.
New operation tests should distinguish returned extent from examined work, page atomicity from
per-candidate outcomes, stale decisions from unknown effects and logical feasibility from actual
scientific admission. Retain complete default/scale campaigns as separately selected scope.

No new benchmark campaign is required for static placement decisions. Quantitative claims need
actual end-to-end comparison, including preparation, transfer, execution, retained state and
recovery. The canceled native sweep remains incomplete; this review does not resume it.

## Other capability dispositions and composition

| Family | Decision and reopening condition |
| --- | --- |
| SurrealDB typed records, links, constraints, ENFORCED edges, INLINE adjacency, generated functions and sync lifecycle events | Retain existing integration and extend complete operations. Do not treat already installed facilities as new adoption. |
| Indexes, count indexes, materialized views and `EXPLAIN ANALYZE FORMAT JSON` | Inspect actual query shapes for selected operations; LIMIT alone does not bound examined work. Use summaries for operator needs, not a replacement for guarded per-owner obligation counters. Explain output is version-scoped diagnostics, not a new stable parser contract. |
| gRPC incremental streaming | Current canonical `bounded_query` buffers results; local SDK `stream_items` refuses payload-drain ownership. Extend only for a concrete stream whose aggregate ownership, original deadline, late statement/transaction error and drop/drain are qualified. Joined bounded pages are selected now. |
| LIVE, changefeeds and async events | Optional observation/wakeup followed by authoritative reread. Catch-up needs bounded retention/cursor ownership; imports and missed delivery matter. No sole readiness authority, exactly-once external effects or replacement for receipts/native drain. |
| Surrealism | Keep the same-source policy placement inquiry from O01. Require pinned module identity and effective finite execution/deployment controls before selection; no independently authored SQL copy of Rust scientific policy. |
| SurrealKit and privileged maintenance | Reuse frozen artifacts and generated schema source. Preserve bootstrap/restore/activation phase separation, protected physical work and authenticated original-operation settlement. |
| Temporal VERSION, files/buckets, alternate backends, PostgreSQL wire, HTTP/GraphQL/MCP, full-text/vector/geospatial | No additional adoption for the examined workflows. New historical-version or retrieval/API capability must define its actual product need and lifecycle. Engine files do not establish protected scientific payload equivalence; PostgreSQL wire is not the existing DataFusion SQL contract. |
| Symbolica/Numerica | Retain demanded regional math, exact algebra already used and I11–I13/I25 dispositions. Finer preparation can improve its inputs; no additional generic rewrite engine, graph canonicalization, tensor/ODE migration or precision fallback is justified here. |
| Tokio/tokio-util/Rayon | Retain supervised cancellation trees, flat native stop leaves and explicit joins. Bounded concurrency may run pure immutable readers, but dropping/aborting a task or pool never proves foreign destruction. No new executor/task tracker supplies the missing physical custody automatically. |
| Datalog/dataflow or another graph engine | No distinct gap established beyond SurrealDB operations, Salsa and existing graph algorithms. Reopen for a named incremental relational/recursive capability that those owners cannot realize simply. |

Consequential ecosystem references include the [tagged SDK query implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/src/method/query.rs)
and [local payload ownership guard](../../vendor/surrealdb/src/method/query.rs),
[tagged event execution](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/doc/event.rs),
[tagged change capture](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/doc/changefeeds.rs),
[EXPLAIN](https://surrealdb.com/docs/reference/query-language/statements/explain),
[count-index execution](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/exec/operators/scan/index_count.rs)
and [experimental bucket contract](https://surrealdb.com/docs/reference/query-language/statements/define/bucket).
For Salsa retention/optional features, see [interned lifetime](https://github.com/salsa-rs/salsa/blob/salsa-v0.28.5/src/interned.rs),
[backdating](https://github.com/salsa-rs/salsa/blob/salsa-v0.28.5/src/function/backdate.rs),
[exclusive storage mutation](https://github.com/salsa-rs/salsa/blob/salsa-v0.28.5/src/storage.rs),
[`specify`](https://github.com/salsa-rs/salsa/blob/salsa-v0.28.5/src/function/specify.rs)
and [persistence tests](https://github.com/salsa-rs/salsa/blob/salsa-v0.28.5/tests/persistence.rs).
For Z3's separate optional mechanisms, see [Optimize](https://github.com/prove-rs/z3.rs/blob/z3-v0.21.1/z3/src/optimize.rs)
and [Fixedpoint](https://github.com/prove-rs/z3.rs/blob/z3-v0.21.1/z3/src/fixedpoint.rs).

The resulting composition keeps canonical facts and short mutations in SurrealDB, pure derived
semantic products in Salsa, mathematical execution in Symbolica/Numerica and class-specific
native libraries, and row analysis in Arrow/DataFusion. Optional Z3 reasons over explicit
finite choices. Semantic authority and execution placement remain separate: better placement
is welcome where the consumed contract and complete lifecycle fit.

## Rule impacts, decision routes and handoff

No selected core improvement requires reversing RC01–RC21. O01/O02/O03 are physical operation
and lifetime changes within the accepted canonical/result contracts; O04/O05 refine the accepted
pure preparation and retention target. Existing semantic IDs, original guards, physical custody,
scientific qualification and effect permissions remain preservation requirements.

There are two **conditional** rule impacts if later choices exceed those boundaries:

- **RC22:** executing shared Rust study policy inside SurrealDB through Surrealism would change
  the execution/deployment contract at blueprint §19.3/§20 and the selected server profile.
  GOV02 must present module identity, failure/resource/cancellation semantics and the proposed
  authority placement for explicit operator confirmation before fixing that target. Keeping the
  current rule retains O01's batched Rust-policy route.
- **RC23:** admitting authored configuration reasoning as a new operation changes the functional
  request/selection contract and potentially replay identity at blueprint §19.3/§21. PRE09 must
  present its finite vocabulary, result/Unknown interpretation and actual native binding for
  explicit operator confirmation before adoption. Keeping the current target leaves configuration
  exploration unimplemented and does not block ordinary execution.

These are proposed consequences, not accepted decisions. Dependency patching alone requires no
ADR; a new binding/operation contract or changed architecture takes its applicable short ADR or
ADR/design route through GOV02. New public explanations use the existing direction-aware
catalogue; metadata/commit changes follow their current decision route rather than an implicit
permission from this review. The original review and audit are preserved unchanged.

[Plan49](../plans/49-library-native-remediation.md) owns proposed package routing and current
inquiry dispositions. Completion of this review means a usable reasoned target proposal and
coordinated plans, not implementation, assembled qualification or measured benefit.
