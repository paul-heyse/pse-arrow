---
title: "Plan 52b: Local knowledge and research"
status: in-progress
date: 2026-10-23
adrs: [ADR-0222]
review_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, design, libraries, storage, environment]
---

# Plan 52b: Local knowledge and research

## Target, baseline and authority

**Proposed:** retrieve the relevant current source, rationale, exact-version library evidence
and selected history for a concrete engineering question. Owns O05/O06/O24/O28; progress and
I03/I04 live at [the coordinator](52-agent-effectiveness-and-engineering-workflows.md).
Knowledge access is derived. Native architecture, plan status, original evidence and scientific
research authority do not move into the retrieval layer.

Reuse `scripts/docs.py` discovery and `document_metadata` roles, `workspace_context` native scope,
`session_retrospective` as the history adapter, and Plan51's evidence source-cut/generation
mechanics. `EvidenceSnapshot::validate` intentionally recognizes particular native codecs;
do not widen it into a generic note blob. Canonical research in `workflow/research.rs` and
`pse-schema/src/catalog/research.rs` describes scientific datasets/selections/physical ancestry.
Development knowledge needs a separate derived family in the existing disposable tooling catalogue.

## Selected realization and alternatives

The maintainer selected a structured, domain-model-first pivot on 2026-10-23. Primary questions
select concepts by the identities and contracts of their existing owners, with exact predicates
and declared relationships. Document roles/topics/owners, authored ADR identities and statuses,
native `ScopeItem` identities/status ownership, registry declaration keys and producer-declared
production paths are facts to consume, not text to rediscover. AP-04 and the core's semantic-model-
first requirement govern construction, validation and query behavior, not only result labels.

Source-only exact adapters serve these questions without a database. The disposable SurrealDB
projection accelerates selected structured queries without becoming a new semantic authority.
Lexical matching is an explicit discovery fallback for unstructured text or missing declarations;
semantic similarity remains unselected. Secondary topics come from native metadata, rather than
a new arbitrary tagging scheme. A topic or term match establishes neither identity nor relationship.

Bounded lexical fallback uses the existing literal/OR-term Python regex once on captured bytes;
redundant per-file ripgrep subprocesses were removed. Ripgrep remains a manual discovery tool.
This avoids duplicate matching/process work without adding a search engine or claiming measured speed.
No extra SQLite database, universal indexing daemon or general memory product is selected.

Published Pagefind serves website search and remains useful. Its
[search API](https://pagefind.app/docs/search-config/) may support selected headless consumers,
but its [Node API](https://pagefind.app/docs/node-api/) is indexing, not proof of headless query
support or dirty-source freshness. I03 records whether reuse is useful; otherwise retain source
search and the selected Surreal target. [SQLite FTS5](https://www.sqlite.org/fts5.html) is an
unselected alternative if actual Surreal search integration proves disproportionate. Selecting it
requires revising B04's target and acceptance before dependent work: specify whether only lexical
indexing moves while Surreal retains graph projection, and preserve one exact source-membership
cut across them. It cannot satisfy unchanged Surreal lexical/query-plan acceptance by substitution.
Do not build equivalent lexical indexes in parallel. Source-only search remains the fallback
for unavailable persistent access, not completion of B04's persistent delivery.

## Work packages

<a id="B01"></a>
### B01 — Source-fresh lexical knowledge access

Select explicit roots/roles and exclude build, environments, secrets and unrelated repositories.
Return bounded excerpts, native owners/stable section IDs, source revision or dirty-content
identity, role/current-versus-historical basis and completeness. Respect deletions/renames and
source changes between selection and detail read. Keep broad source membership separate from
current document authority. A historical decision remains findable without outranking its
current replacement merely because its wording matches better.

The first consumer asks where stale-result acceptance is prevented and follows one current
contract into its source/tests and historical rationale. Source-only search remains usable with
the database unavailable; indexed results explicitly report stale/missing membership and may
fall back to current source search. Do not silently rebuild/import merely to answer a read.

Acceptance: exact excerpts/roles for concept and literal questions, dirty edit overlay, deleted
source invalidation, an attractive obsolete result and an unavailable server. Validate selection
against independently named sources, not only self-retrieval from generated chunks. I03's lexical
outcome names supported operation/freshness and selected backend. Package tests stay in the
host search/composition owners; B04 supplies actual persistent agreement for the same cut later.

<a id="B02"></a>
### B02 — Relevant session discovery and attributable recall

Replace recent-date-directory selection inside `session_retrospective.inspect`. Discover sessions
by relevant activity even when they started before the interval; include explicitly associated
worktrees/subagents. Session/source identity and record timestamps establish membership. File
metadata can prioritize discovery but cannot prove a complete negative answer. Bound discovery
and return continuation/incomplete coverage when unfinished; do not trade the old omission for
an unbounded full-history scan per question.

I04 checks the installed Codex protocol/schema against the official
[App Server read surfaces](https://learn.chatgpt.com/docs/app-server): activity-sorted cursor
listing and thread/turn reads, with `useStateDbOnly` where actually supported. Inspection must
not start/resume a thread or invoke default state repair. Otherwise select a local incremental
file-discovery/stream adapter retaining source identity, read position and source changes. Its
metadata index is rebuildable and not another memory authority.

Deduplicate originating session/logical call plus record role while preserving distinct attempts,
outputs and inherited provenance. Classify quoted failures, retries and polls as observations,
not confirmed incidents or productivity totals. Raw conversation excerpts are explicit selected
reads, not default bulk ingestion into the knowledge store. Never write global memories.

Acceptance: a pre-window session active in-window, related worktree/subagent, inherited call,
distinct identical-output attempts, oversized record and interrupted bounded scan. Answer one
selected rationale question with exact historical attribution; current authorization still comes
from the current conversation. Migrate existing retrospective consumers and remove the
creation-date/exact-cwd assumption as their population definition.

<a id="B03"></a>
### B03 — Exact-version research packets

Compose task-local evidence from Cargo/uv pins, locked metadata, actual enabled features,
installed/patched sources, upstream API/tests, migration differences and unresolved semantics.
Use [Cargo metadata](https://docs.rs/cargo_metadata/latest/cargo_metadata/) and
[rustdoc JSON](https://docs.rs/about/rustdoc-json) only with their target/features and declared
format version. Do not equate hosted registry docs with the patched SurrealDB server or native
adapter. The packet identifies primary evidence versus overview/agent interpretation.

The first consumer compares a concrete library feature/upgrade against real callers, exposing
one signature fact and one unresolved lifetime/behavior question. Keep extraction selective;
source links and a small dossier beat a permanent mirror of every dependency. Reuse the
library-research route, supplied Surreal references and exact primary links. No dependency bump
or install is an implicit packet read. Shared capability skills stay repository-agnostic.

Acceptance: an exact pinned API and a patched-source difference can be reconstructed; wrong
feature/version rustdoc is visibly unsuitable. Unknown JSON formats/refused source access remain
unknown. A05 consumes the packet without depending on B04 persistence.

<a id="B04"></a>
### B04 — Derived development-knowledge and experiment relations

After FND00's short relation-family ADR/declaration slice, project the smallest sufficient set
of owner-native development concepts above immutable source versions. Keep document role and
publication scope separate from decision status and package state. A currently published proposal
is still proposed. Use typed records, meaningful source roles and closed relationship kinds;
construction and queries enforce their applicable combinations.

Documents/contracts/decisions consume interpreted metadata and stable section owners; plan
packages consume native scope rows. Declarations, generators and generated artifacts consume
A02's exact production records. Retain declaration–generator–specific-output triples: splitting
them into two generic edges must not create a false cross-product between a generator's outputs.
Unavailable registry declaration locations remain unavailable. Multiple contributors keep their
source roles; a catalogue registration is not a type-definition location.

Evidence references use existing authority/codec/source/version/member identities and original
adapters. Narrow experiments/diagnostics retain their producer meanings. An explicitly selected
research question or agent interpretation keeps that origin; there is no new experiment/question
lifecycle or scientific research authority. Native state and topic inheritance come from actual
owners and retain the lifecycle/site policy versions that affect interpretation. Generic source
files and excerpts are provenance carriers, not substitutes for all these concepts.

Primary finite queries select kind, owner, native identity, document role, declared topics and
typed relationships by exact conjunction. They answer, for example, who generates an exact output,
what a named native package depends on, which owner governs a stable section, and which explicitly
linked evidence supports an item. A missing governing-contract or qualification-evidence link is
reported as undeclared in the inspected population; neither prose proximity nor a shared tag fills it.
Optional full-text filtering occurs within a structurally selected population.

Reuse tooling-database `evidence_store` staging/activation/lifecycle patterns without falsely
encoding all knowledge as accepted EvidenceFact codecs. Keep original payloads at their owners;
store compact source identity, locator, basis and exact membership. Explicit import stages a new
generation then atomically activates it; interruption cannot expose half a corpus. Reads select
a cut, retain missing/stale sources and do not auto-open unprotected historical absolute paths.

Use native schema/constraints, indexes and graph/record queries for the selected exact concept
and production paths; full-text indexes support optional discovery within that population. [SurrealDB hybrid search](https://surrealdb.com/docs/learn/data-models/vector-search/hybrid-search)
and [derived tables](https://surrealdb.com/docs/reference/query-language/statements/define/table)
are concrete mechanisms; functions/views may supply recurring selected queries. Events/changefeeds
are optional invalidation signals, not a new durable scheduler. Surrealism is selected only for
a useful computation not already well served by typed queries/DataFusion. No separate GQL server,
extra database deployment or duplicate scientific evaluator is needed. Compare recurring typed
database queries with the existing supplied-source/offline DataFusion route before adding database
computation; choose the simpler mechanism for its actual consumer.

Acceptance: answer the concrete exact concept queries above, reject invalid concept/relationship
combinations, and demonstrate two declarations sharing one generator without invented output
correspondence. Navigate one actual design/library question through exact evidence to its original
experiment/diagnostic references. Query structured and optional lexical/graph views on the same cut, import twice without inflation, interrupt import,
modify/delete a source, and drop/rebuild the projection while native facts remain unchanged.
Explain exact query plans on the selected custom server. B01 source-only operation must survive
index loss. Delete any replaced ad hoc knowledge parsers only after their consumers migrate.

<a id="B05"></a>
### B05 — Bounded local semantic retrieval decision

Resolve I03's second part against useful B01 questions. Candidates are downloadable
[Qwen3-Embedding-0.6B](https://huggingface.co/Qwen/Qwen3-Embedding-0.6B) and
[Qwen3-Reranker-0.6B](https://huggingface.co/Qwen/Qwen3-Reranker-0.6B), optional rank fusion with
native BM25/vector retrieval, and a compatible local runtime. Qualify exact weight/runtime
revisions and Blackwell compatibility. Batch inference is the first simple realization;
[llama.cpp server](https://github.com/ggml-org/llama.cpp/tree/master/tools/server) is a candidate
only when repeated consumers justify a shared process. Avoid duplicate model loads per agent.

Compare real concept/code/prose questions with the lexical baseline, including misleading old
material and exact-symbol queries. Selection requires useful retrieved evidence, correct source
attribution, free local operation and a manageable lifecycle. No arbitrary numeric ranking gate
or new paid evaluation API. If not worthwhile, record lexical-only delivery and revisit when a
specific concept-retrieval miss persists. A positive selection includes actual B04 integration,
model/index identity, changed-source invalidation and unload/stop behavior; a running model alone
is not a delivered retrieval workflow. Similarity or generated summaries never establish truth.

Agent Memory remains unselected: the review's deployment/local-model uncertainty is unresolved.
Revisit only if a supported free local route is demonstrated with less total machinery than this
composition. Preserve provenance/supersession ideas without implementing a general memory product.

## Verification and dependencies

**Proposed:** B01/B02/B03 have independent useful consumers. B04 needs working source facts and
its schema/decision slice; B05 needs the lexical baseline, not the entire series. Source cuts,
exact roles and missing populations receive targeted tests. Persistent import/search and optional
model use need real selected-server/runtime consumer checks. 52h owns assembled qualification;
no query/index success qualifies scientific results or authorizes following historical instructions.

## Selected host execution and inquiry outcome

**Implemented and Tested (2026-10-23):** B01 source-only lexical access, B02 activity-based
resumable recall, B03 exact-version packets and B05's bounded decision are delivered. Native B04
remains in integration at the coordinator.

The selected host command
`scripts/pse-env --resource-class light -- python -m unittest scripts.tests.test_workspace_sources scripts.tests.test_workspace_tasks scripts.tests.test_workspace_context scripts.tests.test_session_retrospective scripts.tests.test_knowledge_store scripts.tests.test_semantic_retrieval scripts.tests.test_semantic_navigation scripts.tests.test_semantic_edits scripts.tests.test_evidence_store scripts.tests.test_setup scripts.tests.test_docs`
passed 117 tests against the zero-failure baseline. It covers source deletion/drift, scope changes,
unknown versions/targets/features, partial discovery, historical attribution, edit reconciliation
and source-only operation. It establishes host behavior, not native B04 or assembled Plan52 qualification.

Actual context consumers reconstructed a current native A03 status from its coordinator, compared
source-only task composition with persistent enrichment, and reconstructed SurrealDB3.3.4's local
SDK declaration with signature leads from `vendor/surrealdb/src/method/query.rs`. For the unavailable-DB
question, source-only composition remains the simpler selected base; native enrichment has separate
installation/freshness obligations. Missing references remain visibly unavailable. API lifetime
semantics and an unsupplied compiled feature closure remain unresolved in the packet.

I04 selects the bounded incremental file adapter. Installed Codex0.163 protocol/schema inspection
identified listing/read capabilities, but no isolated/live App Server read-only runtime was qualified.
The adapter's actual bounded two-session rationale consumer read 478 records/3,742,209 bytes and
selected the assistant review record in session `01a16b0b-f97f-7b23-bb53-e4bee9cadade` at
`2026-10-23T23:31:54.625Z`, byte offset1562169: scientific datasets/selections/physical ancestry
retain their meaning, while development research belongs in the tooling projection. This is
attributed prior reviewer rationale, not current authorization or full-history coverage. An actual
changed live source was refused before an excerpt read, as intended.

I03 selects exact structured access with source-fresh lexical fallback and no automatic semantic index. The exact
llama.cppv0.7.0 source and Qwen3-Embedding-0.6B-Q8_0 weights in the experiment's former `.config/local-retrieval.toml`
ran one finite batch on RTX5090/CUDA13.4: 11 selected sources,41 explicit excerpts,6 concept/code/prose/
symbol/historical questions and47 inputs,1024 finite L2-normalized dimensions,29/29 GPU layers.
**Measured:** that one cold process took2.291 seconds; this is compatibility/finite-batch timing,
not a retrieval or productivity speed claim. Host controls passed five tests. No model process
or owned GPU allocation remained afterward. Initial observation guards were corrected against
pinned source/native logs; source-cut drift correctly refused a comparison before reload. Native
GGUF token-classification warnings remain observed, not a warning-free claim.

Usefulness was mixed: freshness/symbol questions did not improve; the code question ranked a guide
higher without the required detailed passage; raw similarity preferred the obsolete dependency
policy; current-scope ordering still buried the desired functional-review skill. Lexical results
already supplied these sources. This bounded sample does not justify a new native vector/index
pipeline. The completed candidate experiment and its runtime/model declarations are retained in
[Git](https://github.com/paul-heyse/pse-arrow/tree/498c44786/scripts/semantic_retrieval.py),
with authenticated runtime/model/source identities. Its unused helper/configuration were removed
after the structured pivot; ordinary operation starts no model or semantic index. Reranking, fusion and Agent Memory remain unselected. A missing answer first calls for checking
the domain model, declared relationships and exact query coverage; the trial creates no semantic
implementation backlog. This outcome does not establish that those mechanisms lack value generally.

### Structured pivot checkpoint

The earlier host/lexical and model comparison evidence above retains its original population.
It does not close the subsequent structured-query pivot. A01/A02/A05/B01/B04 are in progress for
exact owner-native selectors, interpreted inherited metadata, typed concept/production projection
and their actual consumers. The requested SurrealDB/tooling/SurrealKV capability review informs
the physical realization before it is finalized. Source identity, atomic activation, exact replay
reconciliation and source-only operation remain required. No new storage authority is selected.

### Structured library mechanisms

**Interface-checked:** the requested `surrealdb.md`, `surrealdb_tooling.md` and `SurrealKV.md`
references were read end to end, with consequential checks against custom SurrealDB3.3.4
source revision `1ac2b1dba7cd177621a4cf5d507caebeb2414395` and tagged upstream APIs.
Selected implementation uses SCHEMAFULL/closed vocabulary/assertions, ordinary ENFORCED
provenanced relations, exact Production assertions and compound indexes around real selectors.
Typed record references need explicit existence and same-generation constraints; REFERENCE alone
tracks references and deletion policy, not existence. Physical aliases, if used, are mechanically
derived and strictly reconciled; they must not create independently mutable relationship meanings.
Bound queries select a cut once and check every statement result. Native EXPLAIN qualifies actual
lookup/traversal paths; LIMIT alone proves no bound on examined work.

LIGHTWEIGHT relations cannot carry this schema/provenance/index meaning and are unselected.
Small relationship-kind/basis inline fields are candidates only for actual adjacency predicates;
64-character hashes do not fit the default64-byte inline-property payload with overhead. Broad
index combinations, materialized active-pointer views, database functions, GQL/GraphQL, a second
migration lifecycle, ANN and WASM computation add no required current capability.

The selected custom MX adapter uses serializable snapshot isolation and rejects temporal options
and versioned reads. SurrealKV1.0 get-at/history/checkpoint/retention APIs are separate backend
capabilities, not available through that selected adapter. No SurrealKV store or backend change is
selected: temporal storage would still not replace owner source versions and exact corpus membership.
Runtime assertions, structured/empty/paginated queries and bound plans remain to be exercised on
the custom server; this source review establishes no measured performance or native qualification.
