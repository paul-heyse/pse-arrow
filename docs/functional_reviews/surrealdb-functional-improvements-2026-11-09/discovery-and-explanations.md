# Discovery and explanations

Supporting assessment for the [principal review](../surrealdb-functional-improvements-2026-11-09.md).
Inspected 2026-11-09 at `4f07766942ff3b21274fdfb686ad483031439462` with the
concurrent Plan66 working tree preserved. Local source and selected primary interfaces are
**Interface-checked**; all recommendations and benefits are **Proposed**. No query experiment,
retrieval evaluation or performance measurement ran. The principal owns integrated dispositions.

## Present capabilities and the useful next interaction

The [knowledge reader](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/xtask/src/development/knowledge.rs)
already provides typed source/subject/production discovery, structural filters, lexical excerpts,
one-neighborhood traversal and cursor-bound generation identity. Its subject route first gathers
up to 257 structurally eligible subjects and then applies text selection to that population.
The [canonical graph reader](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/src/canonical_explanations.rs)
provides stable, bounded relation pages with evidence attribution; it uses named secondary indexes
rather than native adjacency traversal. These are valuable strengths, not missing first adoptions.

The proposed interaction lets an engineer ask “show relevant source, explain how this result
depends on it, and continue within the same selected cut.” Query mode, source coverage, ranking
and explanation completeness remain explicit. Better retrieval does not make a discovered record
eligible for execution, scientific interpretation, retention release or rerun.

Assessment uses core 3.4, efficient architecture 1.0, process-simulator 1.5 and the binding.
AP-04/DP-04/DP-07 protect identity and graph meaning; AP-07/H5–H10/H15 judge physical access.
Blueprint §20–§21 own current data/custody meaning; the richer interactions are proposed scope.

<a id="O07"></a>
## O07 — Search the selected population before paging

**Recommend.** Make complete selected-population filtering a first-class query behavior.
A developer looking for a matching contract should not have matching behavior depend on an
incidental structural preselection cap. Define two distinct intents where both are useful:
search all admitted members of the selected generation, and filter an explicitly supplied
bounded selection. The latter may preserve today's bounded-population operation; its bound
must be part of the requested semantics and returned coverage, rather than silently stand in
for complete search. This is an acceptable breaking clarification of query behavior.

Use one native parameterized predicate/projection/order/limit composition wherever it preserves
the intended set. Push generation, type/topic/text and cursor predicates before hydration;
retrieve the extra row needed to determine continuation. Favor direct ID access for exact keys.
Let the v4 executor see composed intent so its bound-parameter access, bitmap fusion and
uncorrelated work planning can operate. Keep output/source coverage independent from page size.

Choose ordinary compound/B-tree indexes serving real selection and ordering first. Use native
index-only counts for browse totals where permissions and readiness permit. A separate
`COUNT WHERE` index is justified for a repeated fixed predicate without a useful covering index;
it should not become one counter per reported statistic. Use an indexed existence witness when
the question is existence. Exact imported membership still needs set reconciliation; a matching
count cannot replace it. Expression/multivalue indexes and conditional uniqueness are eligible
for matching normalization/membership contracts, not blanket additions.

Compared alternatives are retaining the materialized array, unhinted composed planning, and an
explicit access index. Choose composed intent as the public behavior; retain an index hint only
where a representative plan shows it is useful. This can remove intermediate population arrays,
repeated hydration and accidental caller-side joining. It is not a promise that every composed
query is faster: broad predicates, array-index expansion and weak selectivity can favor a scan.

Delivery means selective matches beyond the old preselection, empty results, skew and late-key
matches all have correct authorized membership and stable continuation. Compare current and
replacement v4 plans at equal semantics, including unavailable/failed indexes and concurrent
generation changes. No v3 replay is necessary to establish a superior v4 query design.
The [index reference](https://surrealdb.com/docs/reference/query-language/statements/define/indexes)
confirms exact predicate requirements for count indexes and the broader utility of B-trees;
actual candidate/operator work remains unmeasured.

<a id="O08"></a>
## O08 — Explain bounded paths through canonical evidence

**Recommend the capability and representation-specific realization.** Extend one-neighborhood
inspection into selected multi-hop questions: “which retained inputs explain this result?” and
“which study results depend on this source?” A response should retain edge evidence, path
direction, selected cut, depth/work scope and whether the answer is complete for that scope.
It should distinguish a useful path from the complete dependency closure. This supplies a
plausible engineer/researcher benefit even before a new client calls it.

Choose native SurrealQL traversal/composed queries as the initial realization under the current
bounded transport. Preserve indexed relation paging when it matches required order and payload.
Use recursive/path operations only with explicit visited/occurrence semantics and finite scope;
parallel provenance occurrences cannot collapse merely because endpoints match. The existing
reader remains an input to a replacement API, not a compatibility path that must survive.

Classify relationships by their actual meaning:

| Relationship contract | Preferred physical candidate |
| --- | --- |
| Independent occurrence, evidence or mutable edge attributes | Conventional relation record with enforced endpoints where appropriate |
| Propertyless membership whose identity is exactly its endpoint pair | `LIGHTWEIGHT` relation; remove separately managed edge payload/deduplication only after that identity is established |
| Single declared association and incoming navigation | Typed `REFERENCE` with an explicit deletion policy; remove duplicate inverse maintenance where covered |
| Low/moderate-degree native traversal | Existing inline edge/reference caches; tune from actual access and mutation shape |
| Selective small edge discriminator | Inline property plus predicate inside the graph path, reducing rejected-edge hydration |

The generator already uses caches and references selectively. Existing caches will not accelerate
every indexed relation query. Adjacency folding is engine maintenance, not a reason to add another
application cache. Materialized inverses are not selected as a default: they duplicate a hot
parent value and can increase contention. The installed parser-next 4.0.0 corpus and parser
confirm `REFERENCE MATERIALIZE INTO author.posts [WHERE ...]` syntax; this corrects the
discovery's interface uncertainty. See the [4.0.0 parser corpus](https://docs.rs/crate/surrealdb-parser-next/4.0.0/source/tests/corpus/stmts/define/field.surql).
Transactional maintenance, deletion and migration interactions still require runtime/source
verification. Choose tracked-reference/index lookup by default, reopening materialization for
an identified inverse-read advantage and a verified complete maintenance contract.

**GQL is a conditional alternative backend for the same explanation**, not a separate graph product.
Its `MATCH`, optional patterns and path selectors can replace awkward multi-step extraction.
It operates over the same records/relations, but the selected server closure omits GQL and the
native gRPC interface has no dedicated GQL method. Prefer a fixed read-only operation over
exposing arbitrary GQL. Enable a changed build/entry surface only when a concrete explanation
is simpler and its authorization, cancellation, limits and result contract remain explicit.
The [GQL guide](https://surrealdb.com/docs/learn/querying/gql/overview) distinguishes this from
GraphQL and documents both read and mutation surfaces; a graph endpoint is not inherently read-only.

Compared with client-driven traversal, the selected approach keeps filter intent inside the
database and can remove repeated round trips/intermediate graphs. Compared with universal
adjacency denormalization, it preserves suitable ordered indexes and avoids unnecessary writes.
Demonstrate cycles, high-degree hubs, multiple occurrences, denied fields, missing endpoints,
generation switches and deterministic continuation. Report actual cache/block/payload counters
when available; missing counters are unknown, not zero. GQL enablement and graph schema changes
have explicit platform/schema decision impacts; they do not change model/solver graph semantics.

<a id="O09"></a>
## O09 — Rank useful evidence and expose why it matched

**Recommend ranked lexical retrieval as an explicit mode**, preserving exact structural lookup
for users who need deterministic membership rather than relevance. An engineer can ask a question,
receive ranked source excerpts, see highlighted matches and expand their evidence relationships.
Use the existing analyzer-backed corpus with `search::score()`/highlighting and a stable identity
tie-breaker. Bind rank cursors to generation, query and ranking interpretation. Return source
selection and coverage so ranking does not imply global or scientific completeness.

This is preferable to replacing exact lookup everywhere: relevance and exact set traversal answer
different questions. It also gives an interpretable baseline before adding embeddings.
The [ranking guide](https://surrealdb.com/docs/learn/data-models/full-text-search/scoring-and-ranking)
supports native lexical scoring/highlighting; better utility requires judged repository questions,
not a smaller result list or vendor performance figure.

**Conditionally recommend semantic expansion for the same discovery workflow.** A selected
source corpus and judged paraphrase questions provide a credible consumer without a pre-existing
embedding caller. Preserve embedding model/input identity, permissions and reconstructibility.
Start with exact distance over a constrained candidate population; compare with lexical candidates
and rank fusion (`search::rrf`) rather than forcing every result to match both modalities.
Use HNSW or DiskANN only if the evaluated corpus warrants their memory/storage, construction and
recall burden. V4 filtered search and quantized layouts are enablers, not an automatic choice.
Raw lexical and vector scores should not be treated as interchangeable.

Spatial indexing is not selected for the current source/study discovery target: geographical
proximity has no meaningful established predicate here. Reopen for an actual facility/location
workflow with geographical meaning. This is a functional-value judgment, not a licence or
missing-current-caller prohibition. Managed Agent Memory is likewise not selected: it introduces
a different extraction/ontology product without improving this source-owned discovery contract.

Delivery means judged exact/paraphrase questions, stable ties, no unauthorized candidates,
reproducible selected model/index interpretation, and honest approximate/exact distinctions.
Scientific numerical similarity must retain its own domain/units; a text embedding is not a
replacement for scientific equivalence. Quality and latency improvements are unmeasured.

## Discovery correspondence and remaining evidence

| Captured direction | Review disposition |
| --- | --- |
| Parameter planning, direct IDs, fused filters, compound/expression indexes, index-only counts | O07 selected; choose the minimal useful index set rather than freezing all current hints |
| Bitmap graph/full-text composition, reduced expansion | O07/O08 selected intent; actual eligible physical path verified during delivery |
| Lightweight/enforced relations, links/references, inline caches/properties, folding | O08 selected per relationship meaning; no universal conversion |
| Materialized back-references | O08 conditional, syntax interface checked; default tracked lookup retained until maintenance contract and useful write/read balance justify materialization |
| GQL patterns/path selectors | O08 conditional backend for a now-proposed explanation consumer; build/transport changes explicitly eligible |
| BM25, scores/highlights, stable ranking | O09 selected enrichment, judged relevance still required |
| Exact vectors, HNSW, DiskANN, filtered KNN and rank fusion | O09 conditional semantic expansion; exact selected-corpus comparison before ANN |
| Spatial retrieval, Agent Memory | Not selected for this functional target; stated semantic workflow triggers |
| Query memory, spill, scan scheduling and operator metrics | Cross-cutting [diagnostics assessment](diagnostics-and-execution-fit.md); no new host caps |

All three discovery briefs and the supplied v4 overview/addendum informed this assessment.
Primary index, GQL, reference and ranking pages were checked on 2026-11-09. `DEFINE TABLE`
timed out and the fetched reference page omitted materialization; installed parser source
nonetheless establishes its syntax. Specific cache/lightweight/view maintenance details retain
their supplied-reference conditions until exact source qualification. The principal review's
coverage appendix owns the full cross-account map.
