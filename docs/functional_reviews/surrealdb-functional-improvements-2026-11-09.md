# SurrealDB-backed functional improvements

Reviewed 2026-11-09 against `4f07766942ff3b21274fdfb686ad483031439462` and the
concurrent dirty working tree. The inspected canonical analyses, migration, staging and
native-service diffs are formatting changes; other concurrent work was preserved and is
not claimed as newly assessed. The [discovery account](../library_leverage/libraries/surrealdb/briefing.md)
and its [coverage](../library_leverage/libraries/surrealdb/coverage.md) supply the complete
input population, including all twelve briefs and both reusable contract notes. The original
three capability references were consumed during discovery and revisited by topic here.

This is a functional-extension review under core 3.4, efficient architecture 1.0,
process-simulator 1.5, test assessment 1.2 and the repository binding selected by
[standard.toml](../design_review/design_principles/standard.toml). Current source and
selected primary interfaces are **Interface-checked**. Recommendations, changed interactions
and benefits are **Proposed**. No new runtime probe, performance measurement, scientific
qualification or implementation ran. Historical evidence retains its original conditions.

## Integrated direction

Make the existing canonical system easier to author, query, explain, observe and operate by
exposing complete intent to native database and tooling capabilities. Preserve exact scientific
meaning and original effect custody, while replacing incidental representations, protocols and
coordination where a superior complete realization is available. Breaking interfaces, engine
selection, schema ownership and execution placement are eligible changes.

The recommended design combines owner-derived native contracts and engine-normalized schema
inspection; operation-shaped queries and classified relationships; explicit transactional
procedures; typed policy boundaries; refreshable observation; and reusable owned schema/test
workflows. It retains MX and the bounded direct client as the current complete realizations,
while identifying the premises under which KV, native buckets or another client becomes better.
This is a design judgment under present evidence, not a permanent backend constraint.

Richer workflows are in scope without an existing caller: an engineer can ask a deeper provenance
question, subscribe to a study's current status, inspect a schema change before applying it, or
diagnose a slow selected read. Those plausible uses justify assessment. They do not establish
invented demand, relevance scores or numerical speedups. Unrelated new products and universal
infrastructure are not justified solely by the presence of an upstream feature.

The previous [cutover review](surrealdb-v4-cutover-2026-11-07.md) keeps its historical lens.
Its narrow first-use and consumer deferrals are reconsidered here, rather than adopted as
eligibility rules. [Plan68](../plans/68-surrealdb-v4-cutover.md) owns remaining platform work;
[Plan66](../plans/66-integrated-simulation-target.md) owns simulation consumers and acceptance.
This review schedules neither and confers no completion credit.

The maintainer also identified [Plan71](../plans/71-workflow-and-simulation-test-feedback.md)
as actively executing during this review. Its test-infrastructure changes are concurrent inputs,
even where an index still calls the series draft. O15–O18 must reconcile with its actual
implementation/status owners before planning overlapping changes; the review does not interpret
in-flight work as an independently confirmed missing capability.

## Opportunity decisions

The principal review owns these references and integrated dispositions. Supporting documents
provide the full current-to-proposed workflow, enabling mechanisms, alternatives, counterconditions,
delivery scenarios and source limits. Conditional parts do not turn the whole opportunity into a
request for more discovery. They identify the decision that could change the preferred realization.

| Reference | Recommended behavior and realization | Priority / material condition |
| --- | --- | --- |
| <a id="O01"></a>[O01 — Native contract vocabulary](surrealdb-functional-improvements-2026-11-09/contracts-and-schema-evolution.md#O01) | Derive reusable named database contracts and policy-aware native conversions from semantic owners; remove repeated structural reinterpretation. | Foundation. Native unsigned representation replaces Decimal only when its complete stored/query/wire interpretation is faithful. |
| <a id="O02"></a>[O02 — Executable schema inspection](surrealdb-functional-improvements-2026-11-09/contracts-and-schema-evolution.md#O02) | Give developers engine-normalized schema diagnostics and compiler/live comparison; derive operation-shaped inputs and useful outputs from one declared meaning. | Foundation. V2 coverage and embedded/patched-server differences remain explicit; no automatic Rust-emitter adoption or digest substitution. |
| <a id="O03"></a>[O03 — Reviewable schema evolution](surrealdb-functional-improvements-2026-11-09/contracts-and-schema-evolution.md#O03) | Expose an owned, inspectable transition with data effects and completion, combining frozen inputs, SurrealKit and native migration where suitable. | After O01/O02 for representation changes. Fresh reconstruction remains eligible; history is not obligatory for disposable state. |
| <a id="O04"></a>[O04 — Intent-level canonical procedures](surrealdb-functional-improvements-2026-11-09/canonical-operations-and-policy.md#O04) | Express guarded operations as complete typed transactional procedures, with explicit isolation and one retry owner; remove concurrency-only coordination where native tracking suffices. | High. Domain authority fences, acknowledgements and absent-dependency coverage must survive. |
| <a id="O05"></a>[O05 — Typed readiness policy](surrealdb-functional-improvements-2026-11-09/canonical-operations-and-policy.md#O05) | Replace repeated JSON-in-bytes interpretation with owner-derived typed facts/decisions and reuse stable extension preparation. | High. Choose transaction-capable extension placement only when it improves the complete operation over visible SQL plus pure Rust policy. |
| <a id="O06"></a>[O06 — Deliberate operation authority](surrealdb-functional-improvements-2026-11-09/canonical-operations-and-policy.md#O06) | Reuse named procedures behind existing native/API entry points with explicit caller or delegated authority and complete rollback. | Combine with O04/O05. A second endpoint is not itself a capability improvement. |
| <a id="O07"></a>[O07 — Complete query intent](surrealdb-functional-improvements-2026-11-09/discovery-and-explanations.md#O07) | Compose authorized selection, filtering, projection, ordering and limits so the v4 planner can reduce work before expansion; use native count/existence access appropriately. | High. Preserve or deliberately redefine selection coverage; current population caps do not imply global completeness. |
| <a id="O08"></a>[O08 — Useful graph explanations](surrealdb-functional-improvements-2026-11-09/discovery-and-explanations.md#O08) | Offer bounded relationship and path explanations with evidence, using representation suited to edge meaning and SurrealQL traversal first; use GQL where its path semantics offer a concrete advantage. | High enrichment. GQL needs feature preparation; lightweight edges cannot erase occurrence identity or evidence properties. |
| <a id="O09"></a>[O09 — Ranked discovery](surrealdb-functional-improvements-2026-11-09/discovery-and-explanations.md#O09) | Add deterministic relevance-ranked lexical discovery alongside exact selection; permit semantic/hybrid ranking as a separately justified extension. | Lexical first. Embeddings, ANN or spatial infrastructure need an assessable question and quality argument, not an already-written caller. |
| <a id="O10"></a>[O10 — Refreshable observation](surrealdb-functional-improvements-2026-11-09/observation-and-derived-state.md#O10) | Provide current status snapshots with notification-driven refresh and explicit recovery; use durable replay only where continuity matters. | Useful enrichment. A notification never settles a mutation, renews protection or grants release. |
| <a id="O11"></a>[O11 — Coherent derived views](surrealdb-functional-improvements-2026-11-09/observation-and-derived-state.md#O11) | Share summary/query definitions, narrow invalidation and maintain selected projections when their complete dependencies permit it. | Combine with O04/O07/O10. Prefer indexed on-demand computation where maintenance costs more; views do not infer all indirect dependencies. |
| <a id="O12"></a>[O12 — Proportionate persistence](surrealdb-functional-improvements-2026-11-09/persistence-and-result-delivery.md#O12) | Retain persistent MX for current transactional metadata while exposing truthful maintenance/history behavior and a complete alternative-engine decision. | Retain now. KV/DS/managed or standalone routes become preferable only when capacity, recovery or independent access warrants their changed guarantees and participants. |
| <a id="O13"></a>[O13 — Owned artifact delivery](surrealdb-functional-improvements-2026-11-09/persistence-and-result-delivery.md#O13) | Keep one protected publication/read/release contract; make native buckets an alternative complete realization rather than a parallel attachment store. | Conditional storage replacement. Native transfer, digest, original instance identity and collection obligations must be established together. |
| <a id="O14"></a>[O14 — Faithful demand-shaped transport](surrealdb-functional-improvements-2026-11-09/persistence-and-result-delivery.md#O14) | Share generated value policy and support incremental selected consumption, retaining explicit result completeness; keep the bounded direct client until a public replacement discharges its real gaps. | Foundation for O01/O13. Per-message streaming is not cumulative pre-allocation admission or physical settlement. |
| <a id="O15"></a>[O15 — Composed contract testing](surrealdb-functional-improvements-2026-11-09/developer-and-operator-workflows.md#O15) | Expand reusable schema/actor suites and independent native consumers under the original fixture owner; replace duplicated inner setup where useful. | High developer value. Preserve independent oracles and actual child/connection/database disposal. |
| <a id="O16"></a>[O16 — Structured operating workflows](surrealdb-functional-improvements-2026-11-09/developer-and-operator-workflows.md#O16) | Offer coherent structured schema/test/query operations through existing native interfaces, with optional MCP or standard clients where they remove interaction burden. | Enrich current tools first. No mandatory MCP server, SQL gateway or frontend follows. |
| <a id="O17"></a>[O17 — Joined diagnostics](surrealdb-functional-improvements-2026-11-09/diagnostics-and-execution-fit.md#O17) | Relate selected request, native plan, complete response and server incarnation using existing telemetry/report owners. | High. Retain unknown and sampled/lost observations; do not silently execute ANALYZE or treat telemetry as outcome authority. |
| <a id="O18"></a>[O18 — Credible mixed-workload execution](surrealdb-functional-improvements-2026-11-09/diagnostics-and-execution-fit.md#O18) | Make large selection and small canonical operations coexist through reduced work, suitable streaming/spill and explicit maintenance lifetimes. | After useful query shapes. Non-default scheduling follows evidence; preserve the maintainer's ordinary unbounded host selection. |

## How the recommendations compose

**A contract change is one complete operation.** O01/O02 establish declared and installed
meaning; O14 establishes transport interpretation; O03 handles transition and O15 supplies
independent receiving evidence. Named `u64` plus an unchanged widening encoder is not a complete
replacement. Keep raw scientific bits and semantic identity wherever the consumed meaning needs them.

**Query and representation are selected together.** O07/O08/O09 share authorization, source
selection, deterministic continuation and provenance. Inline adjacency cannot improve an indexed
relation scan it never serves. GQL can improve explanation expressiveness without becoming the
canonical scientific dependency authority. O17 distinguishes work removed from a merely different plan.

**Transactional progression and observation have different authority.** O04/O05/O06 own guarded
effects; O10/O11 expose and refresh their results. A scope-specific witness or native serializable
predicate can reduce contention, but a live notification is not a witness for an absent match.
Avoid putting native numerical execution inside database transactions to simplify observation.

**Persistence and delivery share custody, not necessarily a physical store.** O12/O13/O14
must agree on original identity, publication, retained interpretation and release. Multiple useful
representations are acceptable under one meaning. Do not add a second authoritative store just
to consume standalone library primitives.

Start planning with contracts/schema inspection, complete query intent, guarded procedures,
typed readiness and composed tests. Develop graph explanations, ranked discovery and refreshable
observation as user-visible enrichments on those foundations. Keep engine replacement, buckets,
ANN and additional protocol servers as explicit conditional branches with their discriminating
premises. These priorities reflect combined usefulness and burden; they are not an adoption quota.

## Rule and integration impacts

| Reference | Proposed impact and decision route |
| --- | --- |
| <a id="RC01"></a>RC01 | O01–O03/O14 can change canonical schema, value representation, interpretation identity and generation responsibility (blueprint §4/§5/§20). Plan creation must name exact changes; enduring contract/decision changes follow the ADR/design route before implementation. |
| <a id="RC02"></a>RC02 | O04–O06/O11 may change D10 execution placement, conflict fencing, authority and patched defaults. Separate semantic guarantees from replaceable framing; supersede affected accepted rationale rather than editing it. |
| <a id="RC03"></a>RC03 | O08–O11/O16 enrich current functionality and may require feature selection, public query/observation contracts and authorization changes. Current callers are not an eligibility prerequisite; adoption still needs explicit planned scope. |
| <a id="RC04"></a>RC04 | O12–O14 can change MX selection, historical interpretation and artifact custody. Record the engine-dependent isolation/recovery delta and actual participants; licence is not a rejection reason. |
| <a id="RC05"></a>RC05 | O15–O18 affect tooling, fixture and diagnostic ownership. Preserve original deadlines, independent scientific evidence, declared effects and physical settlement. Tool-only changes need no invented architectural gate. |

No production API, pin, service selection, architecture section, ADR or owning plan is changed
by this publication. A subsequent implementation plan must carry selected O references,
consumer behavior, replacement/deletion scope and the actual decision route. No separate mutable
opportunity register or automatic recommendation audit is introduced.

## Complete discovery correspondence

All paths in the first column refer to the [SurrealDB discovery account](../library_leverage/libraries/surrealdb/README.md).
Each row names a captured direction; supporting documents assess alternatives and mechanism branches.
Combined rows preserve distinct conditions rather than silently treating every listed API as adopted.

| Source brief / captured direction | Review disposition and home |
| --- | --- |
| Data/schema: shared named vocabulary; constrained finite structures | O01 recommended; coercion, enum order and independent scientific admission preserved. |
| Data/schema: native unsigned types replacing Decimal | O01/O14 conditional representation replacement within recommended generated policy. |
| Data/schema: scalar-key/record-link duplication and composite locality | O01/O08 assess selective removal; retain independent domain and storage identity where meaningful. |
| Data/schema: field constraints, FLEXIBLE and calculation effects | O01/O02 recommend explicit generated semantics; no arbitrary metadata or write-in-assertion escape. |
| Graph: indexed neighborhoods versus native traversal | O07/O08 recommended equal-answer access and explanation design. |
| Graph: inverse references and deletion policy | O08 recommended selective use; custody is independent of link tracking. |
| Graph: lightweight edges and endpoint-pair identity | O08 conditional per relation meaning; conventional attributed/occurrence edges retained. |
| Graph: inline caches, inline properties and folding | O08 physical alternatives for actual traversal, with degree/write-amplification conditions. |
| Graph: materialized inverse fields | O08 conditional on exact-release support and hot-parent write behavior. |
| Graph: GQL multi-hop/path explanations | O08 recommended enrichment direction, subject to explicit feature/grammar/authority realization. |
| Query: fused selection, unhinted/hinted and compound access | O07 recommended, with explicit source population semantics. |
| Query: native cardinality, COUNT and bounded existence | O07/O11 recommended query-first selection; not a count index for every statistic. |
| Query: bitmap fusion before expansion | O07/O08 alternative to compound access; compare selectivity and full operation. |
| Query: deterministic BM25 retrieval | O09 recommended with judged questions and stable identity ties. |
| Query: exact vector, ANN, DiskANN/HNSW and rank fusion | O09 conditional extension of current discovery; embedding/quality lifecycle explicit. |
| Query: spatial indexing | O09 no geographic proposition established; reopen for a location-aware existing workflow. |
| Query: spill, scheduling and index lifecycle | O18/O07 assess effective execution paths and readiness; unselected GQL spill not credited. |
| Transactions: native isolation replacing concurrency-only guards | O04/O11 recommended selective consolidation; incarnation/authority meaning retained. |
| Transactions: complete procedures and script framing | O04/O06 recommended; explicit isolation and original acknowledgements. |
| Transactions: genuine read-only snapshots | O04/O07 recommend for pure reads, separating any protection-writing acquisition explicitly. |
| Transactions: supplier/caller retry ownership | O04 one owner per actual transaction; uncertain outcome reconciliation precedes replay. |
| Transactions: sequences, uniqueness and staleness | O04/O12 selective primitives; gaps/ordering and freshness must fit meaning. |
| Derived state: views/counts versus another cache | O11 query-first, conditional native maintenance with complete refresh dependencies. |
| Derived state: witness granularity | O04/O11 recommended scoped invalidation or native predicate coverage. |
| Derived state: live invalidation and snapshot refresh | O10 recommended enrichment; current-state recovery is explicit. |
| Derived state: feed-driven rebuildable corpus | O10/O11 conditional where incremental value exceeds replay/retention machinery. |
| Derived state: async events, DROP inputs and live router | O11/O10 conditional variants; no loss of retained originals or assumption of durable acknowledgement. |
| Storage: MX versus integrated KV | O12 retain MX now, explicit KV alternative under changed capacity/history/recovery needs. |
| Storage: native bucket replacement | O13 conditional whole-lifecycle replacement, no dual attachment authority. |
| Storage: historical representation/versioning | O12/O13 distinguish recorded storage time from authored/effective scientific time. |
| Storage: native maintenance and reclamation | O12/O18 recommend owned observation and delegation where native controls cover effects. |
| Storage: standalone engines, encryption and value-log options | O12 assess actual adapter exposure and consumer benefit; no automatic second store. |
| Storage: DS, Earth/Cloud, TiKV and browser persistence | O12 conditional topology choices, no demonstrated need for their additional participants now. |
| Values/client: generated policy-aware native conversions | O01/O14 recommended; generic codec is not domain interpretation. |
| Values/client: streaming, shaped results, CBORv2 and FlatBuffers | O14 demand-shaped alternatives with explicit cumulative limits and fidelity gaps. |
| Values/client: SDK versus direct protocol | O14 retain bounded direct path until complete public replacement; retire replaced path if selected. |
| Values/client: SQL/GraphQL and sessions | O16/O14 selective inspection access; exact protocol semantics and persisted authority assessed. |
| Programming: acquisition/policy/publication placement | O05/O04 compare complete operations; typed pure policy remains preferred absent acquisition benefit. |
| Programming: typed carriers instead of JSON bytes | O05/O01 recommended structural replacement with bounded semantic admission. |
| Programming: init snapshots, BINDINGS and runtime patches | O05 conditional preparation/configuration choices; ABI and physical-release gaps remain explicit. |
| Programming: procedures/API ingress and invoker/definer authority | O06 recommended shared operation owner; API endpoint replacement is conditional. |
| Compilation: normalized compiler/live schema checking | O02 recommended inspectable cross-check with declared coverage. |
| Compilation: operation-shaped mutation generation | O02/O01 recommended where it removes shape/presence reinterpretation. |
| Compilation: external emitters and generated-file publication | O02 selective delegation; no bundled Rust generator or second file owner assumed. |
| Rollouts: selected native migration versus reconstruction/backfill | O03 recommended transition capability with option-specific data witnesses. |
| Rollouts: supplier history/recovery and effective-input identity | O03 recommend one tracked lifecycle; preserve native issuer/operation authority. |
| Rollouts: destructive cleanup, compatibility and completion | O03 explicit effect/conformance/readiness distinction. |
| Testing: declarative cases, actor probes and independent oracles | O15 recommended expand shared contract lane without correlated expected results. |
| Testing: external test composition and physical fixture owner | O15 recommended selected inner reuse, native outer custody retained. |
| Testing: developer facts/MCP and reports | O16/O17 structured existing operations first; optional MCP by interaction value. |
| Diagnostics: complete read explanation and physical correlation | O17 recommended version-qualified plan/request/response join. |
| Diagnostics: existing Collector and telemetry | O17 reuse existing service; no new receiver or outcome authority. |
| Diagnostics: progress/report consumption | O17/O16 structured results with bounded progress and loss visibility. |
| Diagnostics: spill/mixed load and returned allocation | O18 evidence-directed execution selection, no unsupported tuning/speed claim. |
| Coverage tails: mql.js/Mongoose, Kafka/surreal-sync, Agent Memory, Vite/Effect/templates | O09/O12/O16 no compelling current-workflow proposition for new product/broker/web stack; reopen on concrete within-scope value. Lack of a current caller alone is not rejection. |
| Coverage tails: SurrealML | Removed; not an available realization. |

## Delivery evidence and remaining limits

The supporting documents specify representative contract-evolution, concurrent progression,
graph/search, notification/replay, protected-delivery and developer/diagnostic scenarios. They
are delivery examples, not tests run here. Prototype only uncertain premises that could change
an adopted option; preserve independent expected membership, original identities and scientific
interpretation. Measurements must distinguish preparation, useful execution, transfer, retained
state and physical cleanup before claiming latency, memory, throughput or capacity benefits.

Exact tagged sources and current official documentation informed the recommendations. Rolling
documentation conflicts, materialized-reference maintenance semantics, richer binary encoding, ABI/configuration
support, v2 coverage and complete bucket custody remain explicit conditional premises. The review
does not treat incomplete vendor fetches as capability absence or turn discovery source inspection
into runtime qualification. No blanket product campaign was needed for these qualitative judgments.
