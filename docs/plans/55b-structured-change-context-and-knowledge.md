---
title: "Plan 55b: Structured change context and knowledge"
status: complete
date: 2026-10-25
adrs: []
review_sources: [docs/functional_reviews/mcp-development-effectiveness-2026-10-25.md]
doc_role: plan
doc_owner: docs/plans/55-mcp-development-effectiveness.md
doc_topics: [current-work, design, storage, validation]
---

# Plan 55b: Structured change context and knowledge

## Target and foundation

**Proposed:** let Codex follow exact declaration→generator→output→consumer→contract/evidence
questions through original source and native indexed owners. Owns O06/O07; the
[coordinator](55-mcp-development-effectiveness.md) owns dispositions. Useful source-only access
remains independently available, and optional tool use does not require importing a knowledge
population first.

**Interface-checked:** [knowledge.rs](https://github.com/paul-heyse/pse-arrow/blob/1bee18378a20670af31838be0d35e3173917fc58/crates/pse-operations/src/knowledge.rs) already owns
`knowledge_select`, `knowledge_explain`, facet conjunctions, attributed lexical filtering,
production triples, outgoing graph edges and immutable generation/query-cut cursors.
[knowledge_store.py](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/knowledge_store.py) supplies explicit import/query/explain
and freshness observations. Current MCP [knowledge bindings](https://github.com/paul-heyse/pse-arrow/blob/e0a7fe7ea21976f5edef43244e4c04983be2d3e8/scripts/capabilities/knowledge_queries.py)
expose source corpus and library packets rather than these indexed questions.

[workspace_domains.py](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/workspace_domains.py),
[workspace_sources.py](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/workspace_sources.py) and
[workspace_tasks.py](https://github.com/paul-heyse/pse-arrow/blob/e6368add6bdeecc383b249078fa3a01bdadda15a/scripts/workspace_tasks.py) already model concepts, producer triples,
source attribution and task packets. Undeclared governing-contract/evidence links and dynamic
consumers remain explicitly unknown. A richer query cannot substitute for a missing producer
assertion. This limits the initial graph population, not the value of exact structured inquiry.

## Target contracts and execution

Keep native knowledge request/response meaning, immutable generations and continuation binding.
Adapt finite native requests through the existing specialist/schema boundary, with truthful
store prerequisites and exact results. A read never imports/refreshes, starts a service or
silently substitutes a source-only population. Source-only concepts continue without the store;
the agent chooses that alternative explicitly when appropriate.

For relationship expansion, the original producer owns each assertion and its source cut.
Distinguish production correspondence, actual consumption, textual citation, language symbol
reference, selected test, executed evidence and accepted decision. Knowledge records point to
original evidence; they do not copy mutable run state into a second ledger. Do not infer
`tested_by` from names, directory proximity, or a selected-but-never-executed test.

SurrealDB typed record identities, attributed `ENFORCED` edges, exact predicates and selective
incoming/outgoing traversal fit repeated admitted graph inquiries. Existing literal full-text
filtering is optional exact query behavior, not ranked discovery; BM25 support does not authorize
semantic relevance or embedding search. Follow [SurrealDB reference capabilities](../libraries/surrealdb.md)
and [tooling](../libraries/surrealdb_tooling.md), with consequential 3.3.4 syntax grounded in the
[versioned traversal parser](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/syn/src/parser/idiom.rs).

Do not equate a depth limit with bounded fanout or complete impact closure. Bind node/edge/result
allowances and continuations to the actual question/generation, and report incomplete populations.
Use a direct reader composition for a small selected population; compare an exported compact
graph and established native algorithm before choosing database recursion for whole-graph work.
No new graph service, ontology registry, Surrealism module, standalone KV or memory store is needed.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| B01 | complete | 55b | Existing native indexed selection/explain is callable with exact source/freshness/cursors |
| B02 | complete | 55b | Authoritative producers emit the selected cross-boundary relationship facts |
| B03 | complete | 55b | A bounded attributable change-to-contract/evidence question uses those facts |

<a id="B01"></a>
### B01 — Adapt indexed questions without changing their authority

Expose native query/explain through the single capability catalogue, directional schemas,
strict fixed invokers and original knowledge owner. Preserve exact facet/identity/triple query
semantics, issuer/generation/query-bound cursors and absence/freshness/refusal distinctions.
Describe import/refresh separately as effects; do not trigger either from a query or help.
Nonexecuting `knowledge_explain` remains nonexecuting. Optional analyze is not added here.

A first consumer selects `generated_artifact:python/pse/_native.pyi`, follows exact production
triples and attributed contract excerpts, and reopens returned source at its digest before
editing. Existing `rg`, direct source readers and `just knowledge` remain valid alternatives.
Indexed access earns storage/preparation cost on repeated structured questions, not every read.

**Acceptance:** actual native query and MCP answer agree on a selected immutable population with
independently specified expected triples. Changed cursor predicates/generation, missing store,
stale source and unattributed text stay distinct. Domain selection/help starts no service/import.
New native documents use exact rather than truncating scientific/native result adaptation where
their contract requires it. A01 supplies host presentation; the native query defines meaning.

<a id="B02"></a>
### B02 — Produce real consumer, contract and evidence assertions

Begin with a genuine declaration/generator change spanning Rust and generated Python consumers.
Use existing registry/codegen provenance for exact production facts, existing compiler/language
or authored consumer declarations for their supported consumption facts, and original validation
selection/execution records for their different evidence relationships. Add governing-section
links only where an original owner declares or reliably extracts them.

Extend the relevant native/source producers and any registry-owned vocabulary together with
their actual consumers. New relation families use the existing decision route before production
implementation. If facts are derived, retain the deriving method/source identity and its
population/limitations; a text reference is not upgraded into a semantic guarantee. Avoid an
agent-maintained hand list of current test coverage or ownership.

**Acceptance:** the selected declaration yields its exact outputs and real consumers, excluding
other declarations sharing the generator. An unexecuted selection yields no executed-evidence
edge. Changed producer source invalidates attribution; absent/dynamic relationships stay unknown.
The complete affected consumer scope is explicit, even though one workflow demonstrates it.

<a id="B03"></a>
### B03 — Compose an actionable change-and-evidence inquiry

Consume B02's assertions and B01 when the selected population is indexed. Enrich existing task/
domain context rather than adding another universal context packet. Let the agent request exact
edge kinds/directions for a selected change, inspect the resulting path to governing source and
original evidence, and see returned/population completeness and unknown branches.

Keep selection attributable: a source reference may be stale; a retained observation may be
inapplicable to the current source/configuration; a passing targeted test is not whole-model
qualification. D01/D02 consume these facts but own affected validation and evidence applicability.
If incoming/multiple-hop traversal is needed, extend native query/cursor semantics explicitly;
do not scan and hydrate the whole repository merely to return a short path.

**Acceptance:** an actual agent resolves a generated-output edit to its authoritative declaration,
follows a real consumer/contract/evidence path and chooses the correct source edit. Protected
generated paths stay protected. Missing edges are displayed without guessed substitutes; path
continuation rejects a mismatched question/source cut. Direct source fallback is explicit.

## Verification and Outcome — 2026-10-25

**Implemented:** B01–B03 expose the original versioned native knowledge query/explain documents,
with generation/question-bound continuation and explicit missing-owner/freshness states. Real
native-stub producers declare governing §21.5. Original source-bound test assessment records
emit executed support only for explicit contract markers; selected or failed evidence retains
its actual status. Distinct producer spans and attributions remain distinct; identical section
extents are deduplicated before indexing. No inferred semantic edges, fallback store or implicit
service startup is introduced.

**Tested:** the two selected native knowledge wire/admission tests passed through
`just unit-package pse-operations` with their exact version-first filters; 158 unselected tests
were excluded. Focused source/knowledge tests and the full 2,277-case setup selection passed.
Actual indexed query, continuation and explain matched the original native documents; changed
question/generation refused and a missing owner remained unavailable without startup. The final
public-native assessment `20261025T124414.207221Z-plan55-final-public-native-982292-e158b9`
supplied one executed §21.5 fact/support edge and three exact producer/output triples. The agent
reopened source at its current digest and chose the original Rust stub emitter/PyO3 declaration,
preserving generated-output protection. Dynamic consumers remain unknown.

**Corrected mistake:** actual indexed traversal exposed duplicate governing-section extents from
three legitimate producers. Deduplicating the whole identical extent, rather than producer
assertions or different spans, restored attributable paging. Original failed evidence is retained.
See [55f](55f-qualification-and-publication.md#verification-and-outcome--2026-10-25) for exact
local receipts and positive original fixture teardown. No graph/productivity benchmark is claimed.
