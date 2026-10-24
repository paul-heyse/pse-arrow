---
title: "Plan 52a: Context, navigation and semantic editing"
status: complete
date: 2026-10-23
adrs: []
review_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md, docs/functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, design, environment, preparation]
---

# Plan 52a: Context, navigation and semantic editing

## Subsequent usability extension

New preview-capacity inspection and explicit release belong to [52n
LIFE03](52n-readiness-progress-and-edit-lifecycles.md#LIFE03); operation-aligned language
availability belongs to [52m NAV03](52m-progressive-guidance-and-operation-availability.md#NAV03).
These prospective extensions preserve this companion's recorded edit acceptance and admitted-work
obligations.

## Target and foundation

**Proposed:** help Codex identify the right owner, recover a decision, assess a design change and
make a coherent edit without reconstructing every relationship manually. Owns source-review
O01–O04/O07; [the coordinator](52-agent-effectiveness-and-engineering-workflows.md) owns progress,
rule decisions and inquiries. Source inspection supports the boundaries below, not newly tested
semantic-edit compatibility.

`scripts/workspace_context.py` already composes recipe/native scope and Git changes;
`boundary_leads` is bounded text search. `document_lifecycle.aggregate` supplies native status
and relationships. `semantic_navigation.py` and `codex_session.py` attach mcpls0.8.0 to Rust
analyzer/Pyrefly. Use those owners before introducing another language service or graph.

Generated ownership has several real chains: runtime boundary schemas and
`pse-codegen::documents::generate_catalogue`; `pse-schema` registry declarations/DocumentSpec;
xtask schema generators; and `_native.pyi` from actual PyO3 metadata through
`xtask/src/codegen/python_stubs.rs`. A text match in generated output cannot assign all these
to `boundary_documents.rs`. Exact producer relationships belong with the producer.

## Unified-interface integration — 2026-10-24

Expose the delivered task/source/generator and controlled-edit operations through canonical
registrations in CAP01/CAP02/CAP04 and actual ADP02/ADP04 bindings. Language services move behind
MCP01's lazy internal integration, preserving source versions, preview/apply protections and
full/partial/unsupported results. Literal/regex source search and LSP symbol semantics remain
native capabilities; the structured discovery target adds no semantic similarity search.

This is new interface integration scope owned by
[52i](52i-canonical-capabilities-and-progressive-reveal.md),
[52j](52j-native-contracts-and-tool-adaptation.md) and
[52k](52k-unified-entry-point-and-derived-interfaces.md), with progress at the
[coordinator](52-agent-effectiveness-and-engineering-workflows.md). Existing package statuses and
historical Outcomes retain their recorded boundaries; this addition does not silently reopen or
claim delivery of the canonical endpoint. Targeted tests accompany migrated consumers, while
52h owns expanded assembled qualification and publication.

## Design and work packages

<a id="A01"></a>
### A01 — Task context and attributable rationale

Extend `workspace_context` with task-shaped composition from a named plan/package, source paths
or symbol and explicitly selected evidence. Include native state/owner, relevant Git commits,
dirty delta, selected owned work and unresolved questions. Use existing readers for each fact;
no new editable task record. Source-only operation works without the database or history adapter.
Git `log -S/-G/-L`/blame support rationale discovery; B02 can later supply attributed conversation
leads. A historic statement is not a fresh instruction, and an old result is not current proof.

Return readable and bounded JSON views with exact basis, omissions and continuation. Keep
accepted/current/proposed/historical meaning distinct. No whole-workspace dump or implicit raw
transcript ingestion. Migrate the current context command's consumers rather than keep competing
old/new interpretations.

Acceptance: reconstruct a named partially implemented contract change with its real generator,
consumers, dirty state and next open decision. Include a retired historical rationale and a
contradicting current owner; the current owner remains authoritative. Database/history
unavailability does not prevent the source-only packet. Target `scripts/tests` for the pure
composition and exercise the actual context CLI.

<a id="A02"></a>
### A02 — Declared ownership and cross-language paths

Have each generation owner expose its declaration→generator→output relationships, then compose
with Rust/Python semantic references and current catalogue identities. Initial paths are in
memory; B04 persistence is optional access acceleration. Distinguish declared, language-resolved,
text and unresolved relationships. Include field-level correspondence only where actually
specified. Preserve dynamic dispatch, macros and string consumers as explicit coverage limits.

First consumer follows one real runtime request across catalogue, generated contracts, PyO3,
Python facade and targeted tests; a second checks a registry-owned generated path and the stub
producer so the implementation cannot encode the catalogue-only mistake. Generated files name
their edit owner, never receive hand edits. Extend `affected` only as conservative package reach,
not a semantic completeness claim. A legitimate future generator supplies its own mapping
instead of another filename heuristic in the consumer.

Acceptance: these distinct paths return correct declaration owners and source versions; an
unknown dynamic/string edge remains a lead, not exact impact. Tests derive expected links from
explicit selected declarations, not from the traversal's inferred names. Remove the inaccurate
universal generated-owner interpretation once actual consumers use this view; retain useful text
search as clearly labeled evidence.

<a id="A03"></a>
### A03 — Semantic edit preview and actual support

Resolve I01 by invoking existing mcpls `get_tool_support`, `prepare_rename`, `rename_symbol` and
`get_code_actions` on representative Rust and Python symbols. The pinned
[refactoring contract](https://raw.githubusercontent.com/bug-ops/mcpls/v0.8.0/book/src/tools/refactoring.md)
returns edits rather than applying them. Expose server readiness, stale positions/document
versions, `dropped`, `truncated` and unsupported results. Preview exact edits against their source
cut; revalidate changed files before application through the ordinary owned patch path.

Require a real cross-file Rust rename and useful supported Python edit behavior. Do not imply
language rename migrates PyO3 strings, wire names or generated contracts; A02 routes those to their
owners. If Pyrefly cannot supply rename, select a maintained compatible adapter/server only after
showing the existing bridge cannot; Serena's
[symbol tools](https://github.com/oraios/serena/blob/main/src/serena/tools/symbol_tools.py) are a
candidate for extra body-level operations. Keep ordinary reviewed edits available. Unsupported
semantic rename must stay explicit; a spelling replacement does not close that support claim.

Acceptance: actual language consumers return complete previews, a deliberately partial response
is refused as a complete rename, and source drift blocks stale edits. Verify that applied files
and edits match the approved preview; interrupted or partial application remains incomplete,
even if the edited subset compiles. This needs ordinary edit-result reconciliation, not a new
transaction framework. Compile/test the changed
fixture behavior with the correct feature environment. Record precise supported/unsupported
operations and the I01 outcome in the coordinator; expand adapters only for an actual missing
consumer capability.

<a id="A04"></a>
### A04 — Structural migrations with positive and negative examples

Use ast-grep, already part of bootstrap tooling, for shape/context transformations with
metavariables, `inside`/`has`, fixes and rewrite rules. See its
[rules](https://ast-grep.github.io/guide/rule-config.html),
[rewrites](https://ast-grep.github.io/guide/rewrite-code.html) and
[rule tests](https://ast-grep.github.io/guide/test-rule.html). Qualify executable availability
before use. Retain one reusable transformation where a real recurring pattern warrants it;
one-off narrow scripts remain appropriate otherwise.

I02 checks Python3.14 syntax and binding/import requirements. LibCST's
[metadata](https://libcst.readthedocs.io/en/latest/metadata.html) and
[codemods](https://libcst.readthedocs.io/en/latest/codemods.html) are candidates when concrete
format preservation and qualified-name resolution matter. Prefer ast-grep for supported local
shapes; if neither parser supports the actual syntax, use a bounded explicit transform and keep
semantic limits visible. Do not introduce another type checker or service for syntactic work.

Acceptance: preview a real selected migration with positive/near-miss fixtures and expected fix
snapshots, preserve Python comments/import identity where claimed, modify declarations rather
than generated outputs and regenerate affected artifacts. Compile and run targeted behavior
checks. Delete superseded textual substitution machinery once its consumers move.

<a id="A05"></a>
### A05 — Design alternative consequences

Consume working A01/A02 plus selected B03 research to answer a real pre-change question, such
as moving coordination into SurrealDB or changing a boundary field. Return the proposal,
affected contracts/owners/generators/consumers, required decision routes, likely removals and
unknowns for two plausible arrangements. Separate actual current relationships from proposed
impact. B04 knowledge retrieval can enrich the view later; it is not a prerequisite to useful
source-backed design assistance.

Use existing `ScopeItem.relationships`, declaration maps and conservative dependency reach.
No universal ontology, generated plan approval or architectural score. CLI JSON/readable text
comes first under G01's shared vocabulary. Acceptance: compare two designs whose ownership and
migration boundaries differ, expose one unresolved dynamic consumer, and retain the simpler
alternative where it serves the task. The output must help an actual planning consumer, not
merely display a populated graph.

## Dependencies and verification

A01/A02 can begin independently. A03/A04 consume actual source versions and declaration owners;
A05 needs usable A01/A02, not all 52b. B02/B04 improve retrieval without taking task authority.
G04 consumes these paths for integration. Package state stays at the coordinator.

**Proposed verification:** targeted host composition tests, representative bridge requests and
compile/unit checks for actual changed fixtures. Scope-end assembled qualification belongs to
52h. New runtime document families, if needed, follow FND00 before implementation; pure host
context does not acquire a product Arrow schema simply because JSON is returned.

## Targeted execution outcome

**Implemented and Tested (2026-10-23), structured pivot:** exact native concept adapters now
resolve stable contract sections, authored ADR identities, native plan packages, declaration keys,
generators and specific outputs. Task source paths match exact producer fields; symbol text
remains a separately labeled lead. Package state is parsed from bounded captured bytes, including
canonical status-owner contributors and policy inputs. Changed inputs refuse stale rows.

The selected host command
`scripts/pse-env --resource-class light -- python -m unittest scripts.tests.test_workspace_domains scripts.tests.test_workspace_sources scripts.tests.test_workspace_tasks scripts.tests.test_workspace_context scripts.tests.test_session_retrospective scripts.tests.test_knowledge_store scripts.tests.test_semantic_navigation scripts.tests.test_semantic_edits scripts.tests.test_evidence_store scripts.tests.test_setup scripts.tests.test_docs scripts.tests.test_document_lifecycle`
passed145 tests against the zero-failure baseline. Actual CLI consumers resolved §20.5 through its
section owner, retained ADR-0222 as proposed/Reference, and obtained A03 state through the actual
coordinator row. The A02 task packet retained the dirty generator edit and unresolved dynamic
consumers without a database or session read. Historical Git subjects remain rationale.

An actual two-arrangement comparison selected native source adapters with optional derived
enrichment over requiring a running imported corpus for base task reads. The command returned
different conservative package reach (zero Rust crates for selected host paths versus three for
the native path), proposals/removals and explicit availability/dynamic-consumer unknowns. This is
an engineering decision supported by source inspection, not a measured productivity advantage.
The initial script summarizing the successful CLI result used the wrong nesting; the summary
reader was corrected without changing the returned evidence.

The independent review exposed mixed policy cuts and row/source rereads. Those were corrected
through the existing policy/aggregate readers with captured-byte callbacks, rather than adding a
second status parser. At that host checkpoint, B04 and actual stub production remained separate obligations;
the completion account below settles them without implying assembled Plan52 or scientific qualification.

**Implemented and Tested (2026-10-23):** A03/A04 use FastMCP4.1.3's Client/StdioTransport
for connection, protocol negotiation, tool discovery and finite process lifetime, with MCP SDK2.3.0
for explicit protocol errors. Direct SDK session/framing code was replaced. The maintainer selected
FastMCP as the framework on 2026-10-23. Structured errors/readiness and one-based UTF-16 mcpls
coordinates remain explicit; framework result handling does not grant edit completeness.

`scripts/pse-env -- python -m unittest scripts.tests.test_semantic_edits scripts.tests.test_semantic_navigation`
passed 15 targeted host tests against the zero-failure baseline.
`scripts/pse-env -- python -m scripts.semantic_edit_probe --language both --output /tmp/pse-semantic-runtime-handoff.json`
passed actual mcpls/Pyrefly and pinned-toolchain Rust fixture journeys: cross-file Rust rename,
Python rename/extraction, applied-edit reconciliation, executed behavior, invalid positions and
normal/error/cancellation cleanup. Rust's fixture test passed one test. Partial/truncated previews,
source drift, non-edited basis drift, protected outputs, overlapping edits and interrupted application
receive targeted refusal/reconciliation controls. This is representative fixture support, not a
repo-wide rename or whole-product qualification; PyO3 strings/wire names/generated declarations
remain separate owner-routed changes. Python formatting is explicitly unsupported.

I02 selected ast-grep0.50.0 contextual candidate/fix rules plus LibCST1.9.0 import/scope metadata.
The selected JSON-loader migration preserves comments, import aliases and local shadows;
Python3.14 template strings/type-parameter defaults parse and compile. Its actual owned consumer
in `semantic_navigation.py` moved from text to bytes; incompatible `tomllib.loads` remains unchanged.
A01/A02/A05 acceptance and the enclosing Plan52 scope remain at the coordinator.

### Producer and native integration completion

**Implemented and Tested (2026-10-23):** A02 now publishes independently owned registry Rust,
Python, Markdown and Surreal relationships, runtime catalogue relationships and actual compiled
stub relationships. Partial generation preserves other producers. Registry fingerprints,
catalogue registrations and compiled API metadata retain their distinct identities and limits.
The inaccurate universal catalogue-owner interpretation was removed.

`PSE_RESOURCE_CLASS=build just unit-package pse-codegen 'test(provenance) | test(catalogue) | test(knowledge_schema)'`
passed4 tests,34 skipped. The selected xtask command
`scripts/pse-env --resource-class build --native=compiler,solver -- python -m scripts.arrow_validation nextest run -p xtask --bin xtask --features full-codegen --locked -E 'test(partial_generation_preserves_other_producer_relationships) | test(actual_stub_producer_is_distinct_from_catalogue_and_tracks_api_bytes)'`
passed2 tests,84 skipped. Both used explicit force-validation routes; the baseline is zero failures.
Full generation, final Surreal regeneration and `PSE_RESOURCE_CLASS=build just py-sync-native-correctness`
passed, including actual compiled API stub generation.

Actual exact consumers followed FrozenInspectionRequest from its catalogue registration through
six input/output production assertions across four generated outputs. Source inspection followed
the Rust definition, generated Rust/Python boundary, PyO3 diagnostic function and Python facade;
`PSE_RESOURCE_CLASS=build just py-unit -n 0 --basetemp OWNED_TEMP python/pse/tests/test_review_workflows.py -k test_cli_passive_journey_uses_same_owner_after_relocation`
passed1 test,12 deselected, exercising that recorded inspection journey through the rebuilt
extension. The first attempt could not collect because the shared default pytest directory was
not owned by this user. The retry used a freshly created private test directory; it did not alter
that foreign directory. One test needs one Python worker; the sixteen compiler-job default remains.

A registry-owned authored.fit_cases@4 query returned nine declared rows across its actual
producers, retaining the unavailable declaration source location. The actual `_native.pyi` query
now returns three productions from PyO3 module/metadata/exception sources through the stub
producer, rather than assigning it to the runtime catalogue. Unknown dynamic/string edges remain
unknown, and language-server references remain separately requested per language rather than a
claim of complete cross-language impact. B04's selected native/CLI acceptance is recorded in
[52b](52b-local-knowledge-and-research.md#structured-execution-outcome).
