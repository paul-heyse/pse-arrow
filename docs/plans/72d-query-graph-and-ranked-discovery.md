---
title: "Plan72d: Query, graph and ranked discovery"
status: active
date: 2026-11-09
adrs: [ADR-0267]
doc_role: plan
doc_owner: docs/plans/72-surrealdb-functional-improvements.md
doc_topics: [current-work, libraries, design, performance, diagnostics]
review_sources: [docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md, docs/functional_reviews/surrealdb-functional-improvements-2026-11-09/discovery-and-explanations.md]
---

# Query, graph and ranked discovery

**Proposed.** Engineers and developers search the complete admitted source population, obtain
ranked evidence where useful, and follow bounded provenance paths without reconstructing the
graph in client loops. This owns [O07–O09](../functional_reviews/surrealdb-functional-improvements-2026-11-09/discovery-and-explanations.md).
The [coordinator](72-surrealdb-functional-improvements.md) owns adoption and shared decisions.
No Plan68 scope is inherited. Plan66 retains scientific interpretation and acceptance.

## Baseline and foundation choices

**Interface-checked**, 2026-11-09 at `21f703887cc307f986975f9a2fba998f8bd2afa7` plus the
concurrent working tree: `knowledge::selection` materializes up to 257 subjects before text
filtering; canonical graph requests use indexed relation-table pages with key order and an
extra row. Generated native relations already use inline caches and tracked references in
selected families. These are foundations to improve, not missing first adoptions.
[Knowledge source](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/xtask/src/development/knowledge.rs) ·
[Graph source](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-operations/src/canonical_explanations.rs).

Apply core 3.4, efficient architecture 1.0, process-simulator 1.5 and the binding through
[the selected standard](../design_review/design_principles/standard.toml). AP-04/DP-04/DP-07
preserve identity, multiplicity and explanation meaning; AP-07 favors reduction before expansion.
Blueprint §20 owns scientific/canonical custody; a query result is not execution or release authority.

## Target contracts and physical realization

One request declares selection, mode, predicate, requested projection, ordering and continuation.
Complete-population search filters every admitted member before paging. Filtering an explicitly
provided bounded population remains a distinct intentional operation, with its coverage visible.
Bind cursors to selected generation/cut, query, mode and interpretation. A returned page cap is
not a claim about source completeness. Keep ordinary reads nonexecuting when explaining their plan.

Use a parameterized native query exposing generation, authorization, lexical/structural filters,
projection and limit-plus-one to v4. Choose direct IDs and useful compound indexes before another
materialization or cache. Remove forced index hints where native planning supplies the intended
path; retain a hint only for a known access contract. Count/existence queries use native index-only
or bounded witnesses; exact membership never reduces to a cardinality comparison.
[Index mechanisms](https://surrealdb.com/docs/reference/query-language/statements/define/indexes).

Represent a provenance occurrence with its own attributed relation record. Use lightweight edges
only when ordered endpoint-pair identity is the complete meaning; use tracked references with
explicit deletion semantics for direct associations/inverse navigation. Keep inline caches and
small inline edge predicates where the chosen traversal consumes them. Do not add a cache to an
indexed scan that cannot use it. Begin path explanations with native SurrealQL/composed queries,
retaining suitable indexed paging for evidence-rich ordered neighborhoods. GQL is a selected
alternative only when its supported path semantics simplify a specific explanation; no arbitrary
mutation-capable graph endpoint is exposed.

Ranked mode uses analyzer-backed BM25, `search::score()`/highlighting and stable key ties, alongside
exact lookup. Quality is judged against a small authored set of representative questions rather
than vendor figures. Exact semantic distance/rank fusion is the first semantic realization for a
selected corpus; ANN is not a prerequisite. Numerical/scientific equivalence remains with its owner.
[Ranking](https://surrealdb.com/docs/learn/data-models/full-text-search/scoring-and-ranking).

## Packages

### Execution checkpoint — 2026-11-10

Execution is authorized against the current Plan73 result owners under proposed ADR-0267.
Select complete-population native queries, best-excerpt BM25 with stable ties and an atomic
ranking-population epoch, and bounded evidence-preserving SurrealQL paths. The maintainer
selected lexical/graph retrieval; no embedding-model lifecycle is added. GQL and materialized
inverse collections remain unselected. Native relation occurrence/evidence identity survives.
Shared generated publication has one integrator; current package tables retain acceptance.

The current source implements complete selected-generation filtering, explicit selected-set
scope and exact/ranked modes, best-excerpt BM25 with stable ties, and an atomically advanced
ranking-population epoch. Native path execution stages one bounded read-only snapshot,
including the root header, then pages frozen evidence without a mutable refetch cursor.
The snapshot bytes/allocation and original deadline remain with pages; native activity ends
at complete receipt. Structured project and developer readers consume these owners.
Native receiving against the prepared source-built server has passed; the Outcome links
the selected cases and their limits. The maintainer's lexical/graph choice closes semantic retrieval
for this scope; a new corpus/model requirement is its revisit trigger. GQL and materialized
inverses do not simplify the selected attributed, occurrence-preserving path question.

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| D01 | Complete | Current knowledge/query builders; owner-approved selection meaning | Complete selected-population predicates precede pagination; typed exact/bounded modes, cursors, projections and native count/existence access are integrated. |
| D02 | Complete | D01 query contract; declared graph meaning and generated-schema changes from 72a where needed | Bounded multi-hop provenance/dependent explanations, occurrence/evidence-preserving relation choices and suitable native access replace client reconstruction. |
| D03 | Complete | D01 selection/cursor contract and existing analyzer corpus | Ranked lexical mode, explanations of matches and deterministic continuation are delivered alongside exact lookup. |
| D04 | Complete | D02 concrete path examples and D03 judged questions | Close the bounded GQL/materialization/semantic retrieval decisions; implement selected extensions or record explicit rejected branches and triggers. |

<a id="D01"></a>
### D01 — Complete intent and native selection

Move `knowledge::selection` text/structural predicates into the selected-population query before
the response cap. Migrate subject/source/production callers and the canonical metadata readers
that share the same semantics; do not silently treat one example as every consumer. Replace
intermediate arrays and repeated hydration where native composition removes them. Inspect plans
to choose useful compound, bitmap or expression access, and regenerate changed declarations.
No benchmark is needed to justify removing a semantically accidental prefilter cap.

Targeted acceptance: authorized matches beyond the former 257-row population; empty, sparse,
late-key and duplicate-value cases; exact bounded-selection coverage; stable continuation and
generation-change refusal. Index readiness and permission behavior remain correct. Delete the
old capped-selection path and its obsolete expectations with migrated callers.

<a id="D02"></a>
### D02 — Evidence-bearing path explanations

Extend the existing graph request owner with explicit direction, selected cut and finite path
scope. Return ordered paths/edges, their evidence and scope completeness. Distinguish a useful
path from complete dependency closure. Keep cycles and parallel occurrences explicit. Classify
each migrated relation with its actual writers/consumers before changing storage; 72a owns the
generated representation, 72c owns effect/custody guards. A graph trace cannot replace semantic
dependency admission.

Targeted acceptance: cycles, hubs, denied nodes/fields, equal endpoints with distinct occurrences,
missing/deleted roots and deterministic continuation. Remove replaced scalar traversal loops,
duplicate inverse maintenance and unused indexes only for families actually migrated. Native
inline properties use a predicate inside the graph path; folded adjacency is engine-owned.

<a id="D03"></a>
### D03 — Ranked lexical discovery

Expose explicit ranked lookup with excerpts/highlights and graph expansion through D02. Bind
ranking interpretation and stable tie identity to its cursor. Reuse current authorization and
selected source membership. Do not turn a relevance result into an admitted scientific object.
Delete post-hoc client ranking or duplicated analyzer interpretation replaced by native scoring.
Targeted acceptance uses authored exact, paraphrase, tie and irrelevant questions, unauthorized
candidates and continuation across the same selected cut. These are functional expected-result
cases; quantified retrieval or speed improvements are not prerequisites.

<a id="D04"></a>
### D04 — Bounded decisions for richer retrieval and graph realization

The query owner closes each branch using source contracts and concrete request examples. Enable
GQL only if a selected supported path avoids awkward application traversal; first resolve its
build/entry surface, read authority and cancellation ownership, then replace that one path.
[GQL](https://surrealdb.com/docs/learn/querying/gql/overview). Materialized inverse syntax is
already present in parser-next 4.0.0; adopt it only after checking transactional update/delete
and migration semantics for the selected relation, because stale inverses are a correctness risk.
[Parser corpus](https://docs.rs/crate/surrealdb-parser-next/4.0.0/source/tests/corpus/stmts/define/field.surql).
Use tracked-reference/index lookup if that contract is unsuitable; do not block D02 on it.

For semantic discovery, identify a corpus/model/input interpretation and judged paraphrase
questions; choose exact selected-corpus distance plus `search::rrf()` if this adds useful matching.
Choose HNSW/DiskANN only when corpus shape establishes a serious exact-scan feasibility problem;
then evaluate recall and filtering controls that distinguish the algorithms. This is not a
mandatory benchmark campaign. Spatial retrieval and Agent Memory are not selected: their
geographic/ontology meanings do not serve this source-owned target. Reopen only for a named
within-scope requirement; no dormant backend is retained.

## Coverage, verification and remaining evidence

| Review mechanisms | Owning package/disposition |
| --- | --- |
| Unified planning, parameter access, direct IDs, compound/expression/multivalue indexes, bitmap fusion, count/existence and index readiness | D01; conditional uniqueness only for its actual declared invariant |
| Conventional/lightweight/enforced edges, references/deletion, inline properties/caches and folding | D02 per relationship semantics |
| GQL, materialized inverses, exact vectors, HNSW/DiskANN and rank fusion | D04 bounded decisions; no indefinite research dependency for base delivery |
| BM25, highlights, stable ranked paging | D03 |
| Spill/scheduling/plan counters | 72h consumes the queries; no host resource policy change here |

**Proposed verification.** Compile affected targets and run focused query/graph/ranking cases
through `pse check`/`pse test`; generated declarations use `just codegen`. Run the selected joined
journeys in 72i at scope end. Only a serious unresolved degradation risk warrants extra measurement.
No product tests or probes ran during plan authoring. Publication is not implementation or qualification.

## Outcome

**Implemented**, 2026-11-10. D01–D04 deliver complete selected-generation predicates before
paging, explicit bounded-set coverage, exact and BM25 modes, one selected-subject hydration,
native best-excerpt ranking and deterministic continuation. Knowledge interpretation v4 and
query/page/explanation v2 bind the cut and an atomic ranking epoch covering staging/reset.
The explicit admitted corpus limit is 4096; it is not the former accidental 257-row prefilter.

Native bounded paths preserve root/header/cut, cycles, parallel occurrences and attributed
evidence in one read-only snapshot. Whole-sequence completeness, remaining paths and frozen
offset pages remain distinct from dependency closure. Original deadline and cumulative
work/byte/allocation bounds survive paging; no mutable refetch cursor is retained. Ordinary
project readers and developer explanation commands consume these owners.

**Tested** under Linux, the pinned toolchain and the selected source-built v4 server:
[discovery receiving](72-surrealdb-functional-improvements.md#discovery-receiving) passed
7 private knowledge/follow cases and 3 graph cases against zero failures. These exercise
late matches across 300 staged subjects, selected IDs, duplicate/tied excerpts, Unicode
snippets, irrelevant symbol terms, generation/epoch refusal, denied reads, cycles, parallel
evidence, isolated roots, frozen mutation, expired/oversized paths and static/analyzed plans.
The joined native population also passed; scope-end checks have their
[single aggregate owner](72-surrealdb-functional-improvements.md#72dh-execution-outcome).

**Mistake corrected.** BM25 scores are native finite floats, not scientific bit-cell values.
Native offsets require `HIGHLIGHTS` on the index; Unicode scalar slicing now uses those
offsets rather than searching authored marker text. Bare imperative blocks also cannot be
analyzed by this v4 IR: the same checked path body now uses a native SELECT-value envelope,
without preliminary traversal or invented counters.

**Deliberate deviations.** Lexical and graph retrieval are the selected target. Symbol-token
expected results do not establish natural-language paraphrase or semantic equivalence.
GQL, materialized inverses and embeddings add no selected responsibility here; their named
participant/meaning triggers remain above. No speedup is claimed. Enduring contracts are in
blueprint §20.5/§21.7 under proposed ADR-0267. This companion remains available for 72i's
series-wide acceptance and retirement; it has no remaining D01–D04 implementation work.
