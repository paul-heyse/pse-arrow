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

Start with source-selected lexical search: exact bounded file reads and excerpt matching, native
role filtering and dirty-source detection. Implementation uses the existing literal/OR-term Python
regex once on captured bytes; redundant per-file ripgrep subprocesses were removed. Ripgrep remains
a manual discovery tool. This refinement avoids duplicate matching/process work without adding an
index or search engine; no speed claim is inferred. This provides a useful server-independent consumer
without writing a general search engine. The selected persistent target is SurrealDB full-text
and typed graph links in the existing tooling database, with explicit import/refresh and atomic
generation activation. B04 adds that integration once useful source questions work. No extra
SQLite database or universal indexing daemon is selected.

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

After FND00's short relation-family ADR/declaration slice, introduce separately typed source,
relationship and selected excerpt/reference records through existing registry/generators. Consume
A02 producer relationships, B01 selected source facts, B02 explicitly selected historical facts,
B03 research evidence and existing development-evidence references. User questions/agent
interpretations retain their origin; a suggestion is not an accepted plan decision. Do not store
canonical scientific research objects under a new generic meaning.

Reuse tooling-database `evidence_store` staging/activation/lifecycle patterns without falsely
encoding all knowledge as accepted EvidenceFact codecs. Keep original payloads at their owners;
store compact source identity, locator, basis and exact membership. Explicit import stages a new
generation then atomically activates it; interruption cannot expose half a corpus. Reads select
a cut, retain missing/stale sources and do not auto-open unprotected historical absolute paths.

Use native graph traversal and full-text indexes for actual question→source→experiment→diagnostic
paths. [SurrealDB hybrid search](https://surrealdb.com/docs/learn/data-models/vector-search/hybrid-search)
and [derived tables](https://surrealdb.com/docs/reference/query-language/statements/define/table)
are concrete mechanisms; functions/views may supply recurring selected queries. Events/changefeeds
are optional invalidation signals, not a new durable scheduler. Surrealism is selected only for
a useful computation not already well served by typed queries/DataFusion. No separate GQL server,
extra database deployment or duplicate scientific evaluator is needed. Compare recurring typed
database queries with the existing supplied-source/offline DataFusion route before adding database
computation; choose the simpler mechanism for its actual consumer.

Acceptance: navigate one actual design/library question through exact evidence to an experiment,
query lexical and graph views on the same cut, import twice without inflation, interrupt import,
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

I03 selects source-fresh lexical delivery and no automatic semantic index. The exact
llama.cppv0.7.0 source and Qwen3-Embedding-0.6B-Q8_0 weights in `.config/local-retrieval.toml`
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
pipeline. The finite `semantic_retrieval compare` experiment remains opt-in with authenticated
runtime/model/source identities and no implicit installation or service. Reranking, fusion and
Agent Memory remain unselected; revisit when a concrete persistent concept-retrieval miss warrants
a narrower comparison. This outcome does not establish that those mechanisms lack value generally.
