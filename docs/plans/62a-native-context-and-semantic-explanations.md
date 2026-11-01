---
title: "Plan62a: Native context and semantic explanations"
status: proposed
date: 2026-11-01
doc_role: plan
doc_owner: docs/plans/62-code-intelligence-capability-expansion.md
doc_topics: [current-work, design, libraries, diagnostics]
review_sources: [docs/functional_reviews/code-intelligence-capability-expansion-2026-11-01.md]
adrs: [ADR-0233, ADR-0235]
---

# Native context and semantic explanations

## Target and foundation assessment

**Proposed:** make a selected Rust world explain operations, references, captures, constraints
and applicable API candidates. This companion develops O01–O04 and O12 of the
[source review](../functional_reviews/code-intelligence-capability-expansion-2026-11-01.md).
The [coordinator](62-code-intelligence-capability-expansion.md) owns dispositions and shared
contracts. Baseline `4b2f8df34` already has explicitly prepared native snapshots, adjusted types,
read/write roles, identity/coercion observations and declared bounds. Live LSP has broader
operations; exposing an existing IDE API is still integration work, not discovery of a missing engine.

Extend `semantic/native.rs`, facts/protocol projections and existing session preparation. Keep
the compatible native0.0.358 family isolated from ordinary host/source syntax0.0.360. Source-only
operations gain no analyzer dependency. Do not pass HIR/Salsa handles across the helper boundary.
Native operations, public fact construction and answer composition must agree on unavailable,
ambiguous and incomplete observations. Display names never establish entity/type identity.

The operator confirmed extended native preparation on 2026-11-01. Current preparation disables
build-script output and proc macros; simply changing loader flags is insufficient. Build outputs,
macro executable/dylib identities and effective environment/configuration affect the world.
Queries can trigger macro expansion after loading. An immutable source snapshot does not prove
hermetic macro execution. Keep supported external-input assumptions explicit and refuse stale
recorded premises; do not invent a universal external-read monitor.

## Mechanisms and design choices

Read [HIR](../libraries/rust_analyzers/ra_ap_hir.md) original material and addendum §§1–6,
[IDE](../libraries/rust_analyzers/ra_ap_ide.md) addendum §§1,3–6 and
[rust-analyzer](../libraries/rust_analyzers/rust-analyzer.md) loading/indexing sections together.
Exact .358 sources govern native APIs; the review records consequential public-interface limits.

Use `ProjectWorkspace::run_build_scripts`/`set_build_scripts`, `LoadCargoConfig` and the native
proc-macro client with the existing preparation/session owner. Detect reported build-script
errors rather than allowing loader logging alone to imply complete readiness. Distinguish an
unavailable expansion from ambiguous source mapping in an otherwise available world. Keep a
restricted preparation profile useful; extended preparation is explicit, not a silent fallback.

Use native reference categories, expression resolvers and `Semantics::expr_adjustments` for
observed roles and adjustment chains. Capture APIs describe captured places, projections, uses,
mode and stored type; ordinary and async callable categories differ. Predicate evaluation
preserves `Holds`, `NotProven`, `Invalid`, `Unsupported` and original causes. Declared bounds,
inferred requirements and proven contextual applicability remain different observations.

Candidate discovery composes `iterate_method_candidates`, trait-aware variants, associated-item
iteration, callable substitutions and contextual coercion/unification with existing API evidence.
It explains an import, adjustment or bound instead of returning another opaque completion list.
`term_search` is an optional bounded producer: expose fuel/alternative exhaustion and `Many`;
an empty search is not impossibility. Use native source/path rendering and `traits_used` for previews.

Prefer HIR `Type::layout` with explicit errors for ordinary structured layouts. IDE recursive
layout is conditional on a usable pinned adapter: its structs are not ordinary public reexports,
and placeholder zero values cannot establish zero-sized fields. Compiler layout remains separately
target-qualified evidence. Neither represents heap allocation/capacity or measured resident memory.

`StaticIndex::compute` can improve repeated/global retrieval, but is analysis-bound and not a
portable thread-safe cache. Its .358 indexed-file records contain folds/tokens, not hints;
library-root/vendored selection bounds coverage. Deferred hints/completions are separate native
operations. Keep any index with its semantic owner or extract owned facts. Persistent indexes,
SCIP/LSIF and typed Tree-sitter wrappers need demonstrated consumer value before selection.

## Packages

### A01 — Context identity, extended preparation and world comparison

Consumes T01's governing route and the preparation/context slice of T02. Implement explicit
restricted/extended preparation selection using the existing configuration owner. Bind build
metadata, generated source, actual target/features, compiler/server/dylibs and supported
environment premises to the resulting world. Publish useful partial coverage without calling
failed required preparation ready; requests needing unavailable semantics remain unavailable.
Plural module/crate mappings and source/expansion correspondences preserve alternatives.

Retain macro processes through the session's actual ownership lifetime, including lazy expansion,
query cancellation, server failure and stop/drain. Preparation failure settles children and does
not publish a usable half-constructed session. No query implicitly runs Cargo or starts preparation.
Compare separately qualified contexts; never compare raw local IDs as portable identities.

Acceptance: generated `OUT_DIR` item and derive implementation resolve under extended preparation;
restricted preparation reports their limitation; build-script failure, missing dylib and changed
generated bytes cannot yield a complete valid world. A shared file in two crates is not arbitrarily
assigned. Cancel/stop during expansion settles actual owned work and does not kill an unrelated
session. These require real native preparation journeys, not only mocked loader settings.

### A02 — Role, operation, capture and obligation explanations

Consumes T01 and the first T02 typed-fact slice. Works in existing supported worlds without
waiting for A01's extended preparation. Add import/test reference roles and exact callsite anchors;
resolve index/prefix/try/await operations through native APIs. Preserve ordered adjustments,
projected closure/coroutine captures, stored versus place types and callable kind. Extend the
existing predicate projection beyond its narrow truth field; consumers retain explicit outcomes.

Move public inspection, relationship refinements and typed helper consumers together. Remove
replaced lossy projections and category-text inference where the richer fact now owns meaning.
Contextual `highlight_related` is useful relation evidence, not a CFG or runtime causal trace.

Acceptance: an import is not an executed call; READ/WRITE/TEST survive projection; implicit indexing
has its actual target; two uses of one declaration retain different occurrence anchors; projected
capture versus whole-aggregate capture and `NotProven` versus false remain distinguishable.

### A03 — Contextual callable discovery and bounded synthesis

Consumes A02 and T02's candidate slice. Use C01 only for rustdoc-backed discovery; local native
candidates need not wait for all API work. Return candidates with reasons, imports, receiver
adjustments, substitutions and supported obligation outcomes. Existing completion/assist evidence
remains the baseline; a new listing alone does not complete this package.

Compare ordinary native candidate explanation with bounded term search on selected available
values/result type. Implement synthesis only where a revealing example adds useful composition;
otherwise record the unsupported/limited question and retain discovery. No completeness promise.
Native semantic edits become guarded B03 proposals rather than direct mutation.

Acceptance: String/str method access, a missing extension import and unmet blanket bound produce
different explanations; same-spelled unrelated methods stay separate. Search exhaustion remains
incomplete, and proposed source preserves required contextual imports.

### A04 — Qualified layout and deferred semantic detail

Consumes A02's context-qualified inspection and the layout slice of T02. Deliver fallible HIR
field/variant/layout observations, lazily expanding detail. Assess recursive IDE adaptation with
an exact-release compile scenario only if hierarchy adds information the public HIR route lacks.
Record a supported refusal if inaccessible; do not fork an analyzer solely for nicer rendering.

Acceptance: nested enum, target-dependent alignment and unavailable child layout remain distinct;
zero placeholders are not real layout. Compare compiler observations only under matched premises.
A04 owns composition with the existing compiler-layout artifact route. D01 is needed only if a
new structured compiler-layout acquisition is selected; D02 capture/suspension is not a layout
prerequisite.
Defer expensive hints/candidate details through the existing demand owner instead of eagerly
computing everything and truncating presentation afterward.

### A05 — Retrieval investigation and selected reuse

Consumes A02 and a concrete repeated/global question from T03. Compare direct native operations,
request/session-owned `StaticIndex` and existing exported artifacts. Record coverage, source/config
invalidation and owner lifetime. Select an index only if it supplies a useful population or removes
demonstrated repeated work; there is no mandatory benchmark campaign or new persistent service.

If selected, implement that scoped reuse and migrate its actual consumer, with stale-world and
library-root controls. Otherwise close the investigation with a reason and observable revisit
trigger (a repeated/global retrieval consumer not adequately served directly). Export formats and
type-sitter remain alternatives, not automatic dependencies. Source trees alone do not version bytes.

## Package state

| Package | Status | Dependencies | Completion boundary |
| --- | --- | --- | --- |
| A01 | open | T01; T02 context slice | Explicit extended world, comparison and real preparation/drain tests |
| A02 | open | T01; T02 role/capture slice | Native explanations consumed through typed public answers |
| A03 | open | A02; C01 only for API-backed candidates | Contextual applicability explanation and bounded synthesis disposition |
| A04 | open | A02; T02 layout slice | Qualified layouts and explicit recursive-adapter outcome |
| A05 | open | A02; T03 repeated-query example | Retrieval decision and any selected implementation/consumer |

## Verification and handoff

**Proposed:** targeted tests cover provider extraction and typed round trips; selected native
journeys exercise actual preparation, context guards, macro failures and real settlement. No tests
were run and no new native capability was implemented by this plan. Scope-end Q01 owns assembled
qualification and applicable static checks; latency/memory improvements remain unmeasured until
an explicitly selected comparison establishes them. No serialized native IDs or unsafe type-system
garbage collection become routine memory controls. Remove replaced consumers/tests together once
their replacements pass targeted checks. The next work is T01 and the first A02/T02 consumer slice.
