# Query planning, indexes and search

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283` plus the current
dirty Plan66 working tree. Representative query builders and declarations are
**Interface-checked**. Benefits and alternative query designs are **Proposed**; no
matched query experiment, retrieval evaluation or performance campaign ran.

## Existing work and avoidable amplification to investigate

The [canonical graph builder](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_explanations.rs)
emits indexed, key-ordered neighborhood pages. The
[result reader](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_results.rs) selects
bounded metadata and protected batch descriptions. The
[private knowledge reader](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/xtask/src/development/knowledge.rs) combines
generation-scoped structural selection, `text @0@ $text`, graph traversal, reference-field
lookups and deterministic key cursors. Its declaration owner is the
[knowledge catalog](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/xtask/src/development/catalog/knowledge.rs).

In subject mode, `selection` currently materializes a structural population with
`LIMIT 257`, then filters that population by text and orders its page. The cap represents
an actual bounded selection, not proof of global top-K or a complete corpus search.
Its separate population and item plans are valuable existing inspection seams. Compare
the intended discovery behavior before fusing the statements or moving a filter.

## Capability families and availability

| Family | Release relevance | Existing use or useful question |
| --- | --- | --- |
| Unified bytecode executor, parameter-aware index access and hoisted work | Central v4 change | Can existing filters remain visible to one plan instead of array materialization or caller loops? |
| B-tree/compound/expression indexes and direct record lookup | Earlier base with v4 planner improvements | Does the actual selected predicate and ordering use the intended keys? |
| Count indexes and bitmap fusion | Earlier facilities expanded in v4 | Can counts and combined filters avoid record hydration or redundant summary state? |
| Indexed top-K and full-text posting improvements | V4 improvements over existing lexical facilities | Can the current knowledge workflow benefit from deterministic ranked discovery? |
| Exact vector search, HNSW, DiskANN and rank fusion | Earlier foundations, v4 filtering/layout improvements | Does a current retrieval question need semantic matching, and what corpus/embedding owner would it require? |
| Spillable sort/group and shared scan scheduling | V4 execution/resource facilities | Can large existing queries avoid intermediate materialization or latency interference? |
| Spatial indexes | New v4 acceleration | No geographic retrieval consumer was established in the inspected flows. |

## Priority directions

**First: express bounded intent as one optimizable query.** Compare the actual knowledge
subject query and protected metadata selections with a predicate/project/order/limit
composition that lets v4 select an access path. The candidate could remove repeated
hydration, intermediate arrays, client-side filtering or per-item queries. Establish
whether the current population bound is intentional semantics or an implementation
limit before changing which matches are eligible. Keep source coverage and authorized
generation separate from the number of returned items.

Compare hinted current indexes, unhinted v4 planning and an appropriate compound index.
An existing `WITH INDEX` is evidence of intent, not a permanent requirement. Parameter
binding, optional predicates and enum/numeric kind changes can alter eligibility. The
discriminating question is whether the complete equivalent request touches less state,
including negative answers and sparse late-key matches.

**First: simplify repeated cardinality and existence work.** The official
[index reference](https://surrealdb.com/docs/reference/query-language/statements/define/indexes)
documents `COUNT WHERE` for an exact predicate and ready index, while ordinary covering
B-trees can already answer eligible counts. Investigate reusable indexed counts before
adding maintained counters. For pure existence, a bounded indexed witness can be preferable
to either aggregate. This can remove redundant counting structures or reads; it does not
replace exact set reconciliation with a cardinality comparison.

The countercondition is write and maintenance amplification. A separate count index may
duplicate an index already needed for lookup; a dynamic predicate may not match its stored
condition. Query permissions and readiness remain part of the chosen route. Compare the
existing generation count guard and one actual workbench count rather than proposing an
index per reported statistic.

**Next: combine selective access before expansion.** Bitmap fusion can intersect indexed
structured, lexical and eligible graph candidates before fetching records. The research
target is an existing query combining these restrictions, not an inventory of bitmap
operators. Compare compound versus fused indexes and the required graph representation.
Large low-selectivity candidate sets or planner fallback can make a simpler access path
better. Verify the physical plan; query syntax alone does not establish early filtering.

**Next: evaluate ranked lexical retrieval on the current corpus.** Add a comparison query
using `search::score` with a stable identity tie-breaker, on the same authorized population
as the current exact lexical workflow. The official
[ranking guide](https://surrealdb.com/docs/learn/data-models/full-text-search/scoring-and-ranking)
describes analyzer-backed matching, BM25 scoring and highlighting. Better relevance needs
judged user questions; fewer returned rows alone does not prove a better discovery tool.
Check that generation filtering and deterministic order preserve useful top-K execution.

**Conditional: semantic or hybrid retrieval.** Exact vector comparison may suffice on an
already small selected corpus. HNSW/DiskANN add embedding production, model/provenance,
index memory, rebuild and quality responsibilities. Compare those complete lifecycles
with lexical search before adopting ANN. Conjunctive retrieval and reciprocal-rank fusion
answer different questions. Approximate recall is distinct from floating-point ranking
changes in exact distance calculations. No existing embedding pipeline was established
by the inspected lexical/graph flows, so this remains a lower-priority extension of an
existing retrieval responsibility, not an assumed deployed vector requirement.

## Execution resources and inspection

The supplied v4 reference describes spill and morsel scheduling. Current
[Plan68 C05](../../../../plans/68c-surrealdb-v4-query-improvements.md) distinguishes enabled
sort/group spilling from distinct/hash-join spilling in the unselected GQL closure. The
native launcher already owns explicit scratch and selected query-memory settings. Their
wiring is not a measurement of successful spill, physical cleanup or mixed-load benefit.
Keep workstation resource policy; discovery supplies no reason for smaller host caps.

Compare long scans/index builds alongside small canonical writes before tuning worker
places or OLTP reserve. Spilling trades resident memory for I/O and retained temporary
space; a node query budget is not a strict bound on all process allocations. Runtime
environment documentation timed out in this pass, so exact defaults remain supplier
questions rather than freshly verified settings.

The official [EXPLAIN reference](https://surrealdb.com/docs/reference/query-language/statements/explain)
provides static and executed JSON plans but warns that their shape can change. Use bounded
observations, retaining unknown counters. Operator output rows do not establish input rows
examined. [Observability research](observability-and-performance.md) owns interpretation
and retention of these observations.

## Next work and evidence limits

Prioritize one existing knowledge request, one cardinality guard and one protected metadata
page. For each, compare equal membership, authorization, order and cursor continuation
across empty, selective, broad, skewed and late-key cases. Include index build/error state,
concurrent updates and relevant numeric representations. Capture static plans first;
measure only candidates whose semantics agree. No v3 executable is required to compare
the current v4 query against a better v4 realization.

Coverage includes original v4 reference §§2/5 and addendum §§3–5/13. The failed guessed
tagged planner path is not source evidence. This pass uses inspected local builders,
official documentation and the supplied capability reference; it does not establish a
resolved API census, all planner rules, or benefit outside named candidate workloads.
