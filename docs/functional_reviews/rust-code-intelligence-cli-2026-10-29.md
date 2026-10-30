---
title: Rust code intelligence for the agent CLI
status: proposed
date: 2026-10-29
doc_role: review
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [design, libraries, environment, diagnostics, validation]
---

# Rust code intelligence for the agent CLI

## Judgment, scope and evidence

**Proposed:** extend the agent CLI with task-shaped code intelligence: understand selected
code, discover usable APIs, explore a change, construct a call, propose an edit, and investigate
a difficult compiler question. These are valuable activities beyond definition lookup. They
should share analysis and presentation foundations without running every analysis for every
question.

The strongest additions to the initial conversational assessment are selected implicit-semantic
annotations, contextual completion, related-test navigation, semantic edit proposals, API
capability search, actionable examples and dependency migration assessment. Compiler ownership,
async state and representation inspection also have credible occasional consumers here; their
greater integration cost is a reason for separate activation, not for losing them from the
target discussion.

This is the requested **functional-extension review**, with a library-led exploration of agent
development workflows. It is not a diagnostic audit, plan-completion audit or implementation.
The agent's own intended use is the evaluation lens. Plausible frequency and usefulness are
reasoned judgments, not measured demand or savings.

Source baseline: `93558a8b82a729fbdb44b86b44ee8923fe78dba3`, 2026-10-29. The only initial dirty
content was the user-supplied, untracked `docs/libraries/rust_analyzers/` directory. All eight
requested documents were read end to end across the preceding assessment and this engagement,
with parallel library research and coordinator synthesis. The two compiler-internals documents
are byte-identical (`cmp -s`, exit 0); they supply one body of material. The untracked inputs
were preserved without edits or staging.

**Interface-checked:** inspected repository entrypoints and documented upstream mechanisms.
This does not establish that every development API exists with the same signature in the
checkout's pinned toolchain. **Proposed:** all CLI capabilities, integration choices and delivery
scenarios below. No analyzer was started, compiler driver built, dependency installed, product
test run or performance benefit measured. No production, configuration, architecture, ADR or
Plan59 document was changed by this review.

The selected standard is core 3.4, efficient architecture heuristics 1.0 and process-simulator
1.5 through the repository binding. This scope changes developer tooling, not scientific
formulation or solver behavior. Scientific correctness and original outcome ownership remain
with their existing owners; code analysis establishes none of PS-G1–PS-G3 by itself.

## Current capabilities and integration points

The source and the planned CLI must remain distinguishable:

| Owner inspected | Current or proposed role | Implication for this extension |
| --- | --- | --- |
| `crates/pse-host/src/main.rs`, `lib.rs` | Current narrow native host, explicit preparation, command execution and operating tools | There is no delivered Plan59 semantic CLI to extend in place. Avoid describing the proposal as an existing feature. |
| `crates/pse-host/src/correctness.rs` | Current Cargo selection, unit-graph and force-validation integration | Reuse actual correctness selection for compiler actions; do not construct an independent build policy inside an analyzer adapter. |
| `xtask/src/development/affected.rs` | Current conservative reverse package reach and changed-source preview | Semantic references can explain likely consumers; they cannot silently replace conservative selection with a narrower claim. |
| `crates/pse-host/src/foreground.rs`, `activity.rs` | Current owned process execution, actual settlement and passive observation | Analysis process ownership can compose with these mechanisms; a canceled query is not evidence of process drain. Ordinary activity diagnosis must work without semantic preparation. |
| [Plan59a A03–A06](../plans/59a-cli-foundation.md) | Proposed envelope, projection/detail budget, state resolution, prepared tools and discovery | Add semantic payloads and capability discovery to these shared contracts rather than a second output/store framework. |
| [Plan59c C07](../plans/59c-tests-and-fixtures.md#c07) | Proposed shared resolved operation, followed by actual binary/list capture | Analysis context should compose with the selection it can represent, explicitly exposing differences from compiler-unit context. |
| [Plan59d D01–D03](../plans/59d-knowledge-and-workflow-verbs.md) | Proposed briefs, dependency/API lookup, isolated probes and syntax reading | These remain useful. Semantic capabilities add resolution and task-oriented composition; they do not make every source read require a server. |
| [Plan59f](../plans/59f-simulation-observability-and-diagnostics.md) | Proposed scientific/process observation over existing product owners | Join evidence to selected source when identity permits; never infer live execution from a static call relation. |

The checkout already declares `cargo_metadata` and `syn`; `rust-toolchain.toml` selects
`nightly-2026-09-29` with `rust-analyzer`, `rust-src` and `llvm-tools`. It does not declare
`rustc-dev`. The supplied native analyzer briefs discuss `ra_ap_*` 0.0.358, while compiler
overviews and live development API pages span other compiler revisions. No exact-pin driver
or analyzer-extension compatibility was exercised here.

Plan59 records stopped/reverted production execution and active planning. Its historical
exclusions concern rust-analyzer MCP and rustdoc-mcp for Codex. This request explicitly opens
CLI code intelligence for review; it does not require either MCP server, editor configuration
changes or reinstatement of the rejected rustdoc-mcp implementation.

## The interaction I would use

The useful unit is an explicit question at a source position, selected range, symbol, package
or comparison. A combined answer is appropriate when its parts answer that same question.
For example, explaining a call should combine its signature, inferred receiver type, selected
definition and trait origin. It should not automatically compute every caller, implementation,
reference, test and documentation paragraph.

Illustrative syntax below expresses behavior, not a settled command grammar or available CLI:

```text
pse code explain crates/pse-host/src/foreground.rs:LINE:COLUMN
pse code explain FILE:START-END --focus types,captures
pse code refs SYMBOL --scope pse-host --include tests
pse code calls SYMBOL --direction incoming --depth 1
pse code candidates FILE:LINE:COLUMN --kind methods
pse code edit rename FILE:LINE:COLUMN --to NEW_NAME --preview
pse api CRATE --search QUERY --kind method
pse dep CRATE --why --features
```

Use a source position when the question is about a particular invocation: its context is
more precise than a bare method name. Qualified symbols and bounded fuzzy discovery remain
useful for finding the position. Ambiguous matches produce a selection list. Source positions
need explicit coordinate conventions and correct Unicode conversion between CLI, byte offsets
and negotiated LSP position encoding; returned edits must address the same source revision.

A simple answer should often fit in a few hundred tokens. A focused combined response should
aim for roughly 800–1,200 tokens unless the task asks for more. This is a presentation aim,
not a new mandatory tokenizer or serialized-byte limit. Reuse A03's detail/record budgeting.
Keep signatures, relevant bounds, uncertainty, provider/context and result status intact;
reduce snippets and optional expansions first. If mandatory content exceeds the requested
detail, return it with an explicit budget explanation rather than corrupting the contract.

Progressive reveal should support selecting a result directly, not asking the agent to
reconstruct its query. A small number of typed next actions can open source, selected docs,
references, a related implementation, a candidate test or an edit preview. Full artifacts
are useful for voluminous expansions and compiler output. Small results can be returned
directly; do not persist every lookup merely to create a continuation token.

Lists distinguish the semantic scope, completed examination and displayed subset. Include
shown/omitted counts where actually known; report an unknown total when the backend has not
computed it. Reference search may examine a large workspace even when only ten rows are
printed. Scope filters must not be represented as upstream work reduction unless the backend
actually supports them. No-match, preparation-required, incomplete model, unsupported query,
stale revision, canceled work and presentation truncation are different outcomes.

## Functional opportunities

Opportunity identities are local to this review. Priorities below express recommendations,
not accepted packages or mandatory adoption of every mechanism.

<a id="O01"></a>
### O01 — Discover and resolve the code I actually mean

**Recommend as a foundation.** When names recur across traits and wrappers, text search finds
candidate spellings but cannot settle the selected declaration. I would use position-based
definition/type navigation repeatedly before editing generic adapters and ownership code.

Return a compact symbol card: declaration kind, available signature/type presentation,
definition location, source excerpt and inherent/trait origin information. These are separate
typed fields only where the selected backend supplies them; rendered hover content remains
rendered content. Include parameter help when the position is in a call. Preserve multiple
targets and unknown inference. Module-parent navigation and
workspace symbol search help locate the card when I remember a concept but not its path.

Mechanisms: IDE `symbol_search`, `file_structure`, `parent_module`, `goto_definition`,
`goto_type_definition`, `hover`, `signature_help`; HIR `Semantics::resolve_path`,
`resolve_method_call`, `type_of_expr` and module scopes. Standard LSP supplies much of the
navigation surface. A declaration inventory, accessible scope and usable import path are
different views, not one universal list.

Keep D03's cheap syntax outline/show. A tolerant `ra_ap_syntax`/Rowan parser is an eligible
alternative if incomplete edits make the planned `syn` extraction unhelpful. Do not migrate
parsers merely to share a library name: compare error-tolerant reading needs against added
dependency coupling. Semantic mode need not be a prerequisite for reading an unfinished file.

Delivery would be demonstrated by resolving same-spelled methods at distinct use sites,
presenting ambiguity honestly, and retaining a useful source-only read when semantics are
unavailable. This is navigation evidence, not successful compilation.

Sources: [IDE overview §§3–4](../libraries/rust_analyzers/ra_ap_ide.md),
[HIR overview §§3, 5–7](../libraries/rust_analyzers/ra_ap_hir.md),
[upstream Analysis surface](https://rust-lang.github.io/rust-analyzer/ide/struct.Analysis.html).

<a id="O02"></a>
### O02 — Explain implicit semantics in a selected range

**Recommend beyond basic lookup.** Iterator/future chains, closures and borrowing often hide
the information needed to choose an edit. I would ask for intermediate types, captured values,
binding modes or implicit adjustments around the expression under investigation.

Offer a range-scoped annotated view with selectable categories: chain/local/closure-return
types, argument names, generic arguments, inferred reborrows/adjustments, capture information
and relevant exits or suspension points. Place the explanation beside its source span. Resolve
expensive hint details only when selected; suppress obvious/repeated annotations. This is an
effective combined output because the annotations explain the code already being read.

Mechanisms: `inlay_hints`, `inlay_hints_resolve`, `InlayHintsConfig`, `highlight_related`;
HIR `expr_adjustments`, pattern/binding queries, `Closure::captured_items`, `capture_types`
and `fn_trait`. A future-output or iterator-item query is useful when the receiver's full
concrete type is much larger than the property I need.

An IDE hint is not a borrow-checker verdict. A capture list is not measured memory retention.
Structured fields should be supplied only where a typed backend/protocol exposes them;
rendered hover text can remain a rendered explanation, not a string-parsed semantic authority.

Delivery scenario: explain the intermediate types and captures in a selected async or iterator
chain, then expand one uncertain adjustment without returning annotations for the whole file.

Sources: [IDE overview §4](../libraries/rust_analyzers/ra_ap_ide.md),
[HIR overview §§5, 7](../libraries/rust_analyzers/ra_ap_hir.md),
[inlay configuration](https://rust-lang.github.io/rust-analyzer/ide/struct.InlayHintsConfig.html).

<a id="O03"></a>
### O03 — Explore the consumers affected by a proposed change

**Recommend early.** Before changing an error variant, constructor or ownership API, I want
actual references grouped into a small inspection set. This would replace repeated grep and
manual disambiguation, while leaving conservative test selection intact.

Return semantic references grouped by file or containing item, with concise snippets and
optional production/test/import/generated categories. Preserve uses in types, patterns,
constructors and supported macro expansions rather than limiting the view to calls. Combine
with changed-package ownership and reverse dependency reach when I ask for a change brief.
Each relationship retains its source: package reach, symbol reference or declared generation.

Mechanisms: `find_all_refs`, `SearchScope`, D01 changes/briefs and the current affected owner.
Semantic analysis can prioritize what to inspect; an absent edge is not permission to omit
correctness work, external consumers or inactive configurations. Generated results should
link to their generator owner and remain protected from editing.

Delivery scenario: change a declaration with unrelated same-named symbols elsewhere, inspect
its resolved consumers, and separately obtain the conservative proposed validation selection.
No result should claim this is a complete behavioral impact analysis.

Sources: [IDE overview §3](../libraries/rust_analyzers/ra_ap_ide.md), current
`xtask/src/development/affected.rs`, [Plan59c C03/C07](../plans/59c-tests-and-fixtures.md).

<a id="O04"></a>
### O04 — Trace a bounded call or implementation neighborhood

**Recommend as an explicit expansion.** To understand a control path such as foreground
launch, I would trace incoming/outgoing calls and the trait implementation involved, then
read the selected source. This is more useful than a workspace-wide call-graph dump.

Return a one-hop neighborhood by default, with caller/callee location, relationship kind,
call-site spans and explicit indirect/unresolved edges where available. Additional depth is
requested, with cycles/repeated nodes represented rather than enumerating all paths. Trait
views distinguish declaration/supertrait structure, related impl declarations and resolution
of one invocation. A selected trait predicate is a different query again.

Mechanisms: `call_hierarchy`, `incoming_calls`, `outgoing_calls`, `goto_implementation`;
HIR supertrait/item navigation, `dyn_compatibility`, `Type::impls_trait` and predicate queries.
`Impl::all_for_type` is shallow and excludes blanket impls. A declaration list must not be
labeled the exhaustive set of applicable implementations.

Dynamic dispatch, function pointers, callbacks, FFI and generated execution limit static
reachability. A static edge does not establish that a path ran. This remains a source
investigation, not a scheduler, runtime trace or proof of cancellation behavior.

Sources: [HIR overview §§5–6, 8](../libraries/rust_analyzers/ra_ap_hir.md),
[IDE overview §3](../libraries/rust_analyzers/ra_ap_ide.md),
[implementation enumeration contract](https://rust-lang.github.io/rust-analyzer/hir/struct.Impl.html#method.all_for_type).

<a id="O05"></a>
### O05 — Discover a usable library capability before knowing its name

**Recommend as an enrichment of D02.** Exact item lookup is useful after I know the API.
Often the earlier question is which API can perform a conversion, expose an iterator, accept
this buffer or support an object-safe boundary. I would use bounded capability search and
trait-aware type summaries while designing an integration.

Support name/docs search and structured filters for item kind, input/output type references,
trait and safety/async/const properties. Return candidate signatures, short relevant docs,
public exposure paths, inherent versus trait methods, imports, associated types and material
bounds. Expand supertraits, blanket/negative/synthetic impl metadata, required/provided
methods and dyn compatibility as needed. Deprecation, `must_use`, non-exhaustiveness, stripped
members, ABI and `repr` matter when relevant; omitted metadata must not look like a guarantee.

Mechanisms: rustdoc JSON `Crate.index`, `paths`, `external_crates`, `Use`, `Type`, `Generics`,
`WherePredicate`, `Trait`, `Impl` and resolved documentation links. Public import paths and
definition locations remain distinct. Follow an external item by addressed crate/context,
not by generating an entire dependency corpus for every lookup.

Rustdoc's existing HTML signature search is a useful human-facing alternative. Approximate
signature matching is not Rust trait solving. Begin CLI search with typed structural/name/docs
filters; a richer signature-query parser earns separate consideration if actual use warrants
it. Do not reproduce rustc normalization or parse generated search JavaScript as a durable API.

Delivery scenario: find candidate methods for a required operation, select the public path and
trait import, inspect the constraints, then use D02's explicit probe for consequential
applicability uncertainty. Documentation presence under `cfg(doc)` is not ordinary-build
availability. Known gates, unknown conditions and the extraction context must stay visible.

Sources: [rustdoc overview §§4, 8–9](../libraries/rust_analyzers/rustdoc.md),
[rustdoc-types overview §§2–10, 13](../libraries/rust_analyzers/rustdoc-types.md),
[native signature search](https://doc.rust-lang.org/rustdoc/read-documentation/search.html),
[documentation configuration](https://doc.rust-lang.org/rustdoc/advanced-features.html).

<a id="O06"></a>
### O06 — Turn documentation into an actionable example

**Recommend; make extraction/scraping selective.** After finding an API, I would request a
small relevant example including hidden setup, ownership conventions and its verification
expectation. This avoids copying a fragment that only makes sense inside a doctest wrapper.

Return authored source, relevant setup, source location, selected dependency/context and
the example mode: compile-and-run, compile-only, expected compile failure/panic, ignored or
conditioned. Expand hidden lines and transformed doctest code on demand. Offer an explicit
handoff to a probe/test; reading an example never executes it.

Mechanisms: `Item.docs`, rustdoc doctest attributes and nightly `--output-format doctest`.
Doctest extraction has a separate schema from API JSON. Preserve that separation and the
actual compiler/test result if subsequently run. An expected compile failure may fail for
an unintended reason; an ignored example supplies no execution evidence.

Scraped examples can reveal real uses in selected example targets when authored docs are
insufficient. Treat them as optional enrichment, not an acquisition prerequisite or a complete
source-reference search. Native rustdoc already provides useful example presentation; reuse it
for browsing instead of constructing an independent example-index platform.

Sources: [rustdoc overview §§5, 10](../libraries/rust_analyzers/rustdoc.md),
[doctest modes](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html),
[experimental extraction](https://doc.rust-lang.org/rustdoc/unstable-features.html),
[scraped examples](https://doc.rust-lang.org/rustdoc/scraped-examples.html).

<a id="O07"></a>
### O07 — Find tests worth reading and running from selected code

**Recommend as navigation integrated with C05/C07.** When I inspect a production function,
candidate tests are a valuable next expansion. They help find expected behavior and examples
without searching an entire test tree.

Return test locations, their relationship to the selected code where supplied, and recognized
runnable metadata. Allow a combined consumer/test view and an explicit handoff to existing
test discovery/selection. The repository's execution owner resolves capabilities, features,
fixtures, actual binaries, ignored policy and final commands. Do not execute raw analyzer
runnable arguments as a route around those contracts.

Mechanisms: `related_tests`, `runnables`, `discover_tests_in_file`,
`discover_tests_in_crate`, `discover_test_roots`; negotiated LSP test extensions where supported.
These are source-level candidates, not measured coverage, sufficient verification, exact
nextest enumeration or proof of every generated/custom-framework test's existence.

Delivery scenario: navigate from a selected production declaration to candidate tests, read
one, and select it through the original test owner. Missing candidates must not suppress
conservative affected selection or imply no tests exist.

Sources: [IDE overview §7](../libraries/rust_analyzers/ra_ap_ide.md),
[runnables and related-test protocol](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/lsp-extensions.md),
[Plan59c](../plans/59c-tests-and-fixtures.md).

<a id="O08"></a>
### O08 — Use completion as constrained discovery during code construction

**Recommend as a useful optional mode.** A text-based agent can benefit from completion
without an editor UI: which methods fit this receiver, which constructor is accessible,
which import makes an item available, or what arguments this call expects.

Return a bounded list with candidate signature, short reason/detail, necessary import and
insertion preview. Resolve docs/additional edits for the chosen candidate. Distinguish method,
import and expression-synthesis modes; do not flood a type question with keywords/snippets.
Signature help can join O01/O02's explanation rather than becoming another command family.

Mechanisms: `completions`, `CompletionConfig`, relevance ranking and
`resolve_completion_edits`; HIR method candidates and `find_use_path`. Candidate ranking is
heuristic, and the insertion may include imports or snippets requiring interpretation.

Term search is a credible later experiment for a specific expected-type question. It has
native `completion.termSearch.fuel` / `assist.termSearch.fuel` controls and is not exhaustive.
Keep it explicit and bounded; an exhausted search does not establish that construction is
impossible. Do not introduce a general synthesis service as a prerequisite for method lookup.

Delivery scenario: discover a trait method and its import at an incomplete call site, inspect
the chosen insertion, and check the resulting code through the ordinary owner.

Sources: [IDE overview §4](../libraries/rust_analyzers/ra_ap_ide.md),
[HIR overview §§3, 6, 8](../libraries/rust_analyzers/ra_ap_hir.md),
[completion configuration](https://rust-analyzer.github.io/book/configuration.html).

<a id="O09"></a>
### O09 — Propose coherent semantic edits

**Recommend after reliable reading/context foundations.** Rename, contextual assists and
semantic structural replacement could remove error-prone manual work. I would use them for
renaming one entity, generating required trait members, filling matches, extraction, imports
and repeated call migrations.

First list available actions at a selected range. Resolve one action into a reviewable diff
with affected paths, original source revisions, unsupported regions and any filesystem edits.
Semantic rename must distinguish unrelated same-spelled declarations. SSR should match the
selected resolved call pattern, preserving captured expressions rather than replacing text.

Mechanisms: `prepare_rename`, `rename`, `will_rename_file`, `assists_with_fixes`,
`AssistResolveStrategy`, `SourceChange`, `structural_search_replace`; LSP rename/code actions
and experimental SSR. Resolve actions lazily instead of computing every proposed rewrite.
Support only the selected release's actual scope controls, not invented server-side filters.

A proposal is not an applied transaction or behavioral equivalence proof. A subsequent apply
operation, if selected for implementation, must preflight affected files, refuse detected stale
content, preserve concurrent edits and enforce protected/generated ownership. Preflight alone
does not prevent a race between checking and writing or make sequential writes atomic. Select
an established concurrency/publication strategy with explicit partial-failure behavior, or
retain preview-only delivery rather than build an elaborate editing transaction system.
Snippets, file creation/moves and unsupported macro origins need explicit handling.
Compilation/tests remain proportionate to the edit. Preview-only delivery has value even if
application initially remains with existing editing tools.

Prefer native semantic transformations when their applicability fits; retain `apply_patch`
and syntax-oriented tools for edits that do not need name resolution. No automatic rename or
fix-all follows from reading a diagnostic. Filling a placeholder is not implemented behavior.

Sources: [IDE overview §5](../libraries/rust_analyzers/ra_ap_ide.md),
[SSR mechanism](https://rust-lang.github.io/rust-analyzer/ide_ssr/index.html),
[native assists](https://rust-analyzer.github.io/book/assists.html).

<a id="O10"></a>
### O10 — Explain diagnostics with the right evidence and next action

**Recommend early, composing with existing result handling.** During incomplete edits, IDE
diagnostics and selected failed obligations can help identify unresolved names or bounds.
After a check, the actual compiler diagnostic is more valuable than another approximation.

Return provider-specific diagnostics with primary/secondary spans, children, macro-expansion
context, related source and suggestions/applicability. A selected failure can expand to the
relevant API, inferred type, obligation or proposed fix. Rustc/Clippy results should reuse
the original invocation's structured output, not trigger another check just to explain it.

Mechanisms: IDE `syntax_diagnostics`, `semantic_diagnostics`, `full_diagnostics`,
`get_failed_obligations` and predicate inspection; Cargo JSON compiler messages and rustc JSON
diagnostic fields. IDE `full_diagnostics` is not a Cargo/Clippy execution. `NotProven`, unknown
and contradiction must remain distinct. Native diagnostics may differ from compiler lint
handling and cannot certify a clean build or borrow correctness.

Extend explicit documentation checks with scoped rustdoc link, code-block and documentation
diagnostics when the changed Rust API calls for them. Native rustdoc lints do the work;
coverage/missing-example reports are triage leads, not a new quality score or automatic gate.
This complements Markdown link checks rather than replacing their owner.

Delivery scenario: read one original compiler failure with all relevant spans, inspect its
suggestion, and navigate to its type/definition without rerunning the command. An IDE-only
failure and an actual compiler rejection remain visibly different records.

Sources: [IDE overview §§5, 8](../libraries/rust_analyzers/ra_ap_ide.md),
[rustc JSON](https://doc.rust-lang.org/rustc/json.html),
[rustdoc lints](https://doc.rust-lang.org/rustdoc/lints.html),
[Plan59b B02](../plans/59b-runs-and-continuity.md#b02).

<a id="O11"></a>
### O11 — Explain dependencies, features and the selected build context

**Recommend as D02/C07 enrichment.** I would use a short answer to why a backend is present,
which users activate a feature and what a selected command will build, especially when
analysis and compiler behavior disagree.

Start with exact package/version/source, direct users and a short inclusion path. Expand
optional/default/target conditions, selected features and inverse activation paths. Distinguish
manifest declarations, locked packages, resolved features, planned compiler units and observed
artifacts. Descriptive license metadata is useful context; it creates no rejection policy.

Mechanisms: `cargo_metadata` package/dependency/resolve data; `cargo tree -e features -i`
for explanation; command-specific nightly `--unit-graph`; captured `CompilerArtifact`,
`CompilerMessage` and `BuildScriptExecuted` records. Unit graphs describe planned work without
compiling it. Cargo metadata does not represent all per-kind compiler-unit feature contexts.
Pretty tree output is not the authoritative graph, and duplicate versions are not an inventory
of repeated compilations.

Guppy's `PackageGraph` reverse/feature queries merit consideration for repeated multi-context
reachability or feature simulation. Straightforward metadata queries do not automatically
justify another graph layer. An all-features graph used for simulation is not the current
build context. Hakari changes feature declarations; it is not needed for read/query tooling
and should not be adopted through this opportunity.

Delivery scenario: explain why a dependency and feature are present, show the chosen unit
context separately, and identify an analyzer/compiler context mismatch without changing
features or silently preparing providers.

Sources: [dependency overview §§2–3, 5–7](../libraries/rust_analyzers/rust_dependency_tooling.md),
[Cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html),
[unit graph](https://doc.rust-lang.org/cargo/reference/unstable.html#unit-graph),
[Cargo tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html).

<a id="O12"></a>
### O12 — Produce a dependency or public-API migration brief

**Recommend as an explicit occasional workflow.** When updating a library, I want to know
which newly available capabilities are useful and which consumed APIs changed. When editing
our own public surface, I want an intelligible account of changed paths, signatures and bounds.

Compose a context-matched API difference with likely workspace consumers and selected semantic
references. Highlight additions/removals, reexports, associated items, trait defaults, bounds,
deprecations and compatibility diagnostics. Expand one change to its old/new declaration,
documentation, actual consumers and an optional probe. No update is performed by this read.

Mechanisms: `public_api::Builder::from_rustdoc_json`, `PublicApiDiff`,
`cargo-semver-checks`, rustdoc typed API structure and O03/O11. Prefer existing compatibility
analysis over a bespoke SemVer engine. A structural difference is not automatically breaking;
SemVer checks are scoped and incomplete, and behavioral changes can remain invisible.

Both sides need source/version and extraction context. Document-local numeric IDs cannot
match symbols across builds; path/semantic matching must account for moves and reexports.
Different schema/compiler/target/features can explain a difference without an API change.
Hosted docs.rs JSON is useful for exploratory version research with explicit provenance; it
must not silently stand in for the checkout's selected local API.

Delivery scenario: inspect one deliberate dependency change without modifying the workspace
lock, follow a changed API to local uses, and distinguish compatibility findings, candidate
opportunities and questions requiring a compile/runtime probe.

Sources: [rustdoc overview §§9, 13](../libraries/rust_analyzers/rustdoc.md),
[rustdoc-types overview §§12–13](../libraries/rust_analyzers/rustdoc-types.md),
[public-api](https://docs.rs/public-api/latest/public_api/),
[cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks),
[hosted JSON](https://docs.rs/about/rustdoc-json).

<a id="O13"></a>
### O13 — Inspect generated code, macros and source provenance

**Recommend as a selected expansion.** In this repository, generated relations, derives and
native bindings can obscure the declaration or method I am inspecting. I would ask what one
macro produced and which authored source or generator owns a generated result.

Return the selected expansion, macro identity and available source mapping; summarize large
results and retain an optional artifact. Follow generated definitions back to authored tokens
where supported. Explicitly report invocation-level fallback, ambiguous mapping and missing
proc-macro/build outputs. Expansion text alone does not preserve complete hygiene information.

Mechanisms: IDE `expand_macro`; HIR expansion/descent, `original_range`,
`original_range_opt` and related original-node mapping; D06's proposed generated-owner map.
Registry-generated files and macro-generated virtual code are different provenance kinds.
Neither grants permission to edit generated output. Missing preparation is not an empty
successful expansion.

Delivery scenario: inspect one generated impl, find the invocation or generator owner, then
read the editable source. A large expansion is not automatically attached to every reference.

Sources: [HIR overview §4](../libraries/rust_analyzers/ra_ap_hir.md),
[IDE overview §8](../libraries/rust_analyzers/ra_ap_ide.md),
[macro protocol](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/lsp-extensions.md),
[Plan59d D06](../plans/59d-knowledge-and-workflow-verbs.md#d06).

<a id="O14"></a>
### O14 — Investigate ownership, destruction and async state with compiler evidence

**Retain as valuable explicit escalation.** The difficult questions here include whether a
guard survives an await, what is moved versus borrowed, why a future is not `Send`, and what
Rust drops on an error/unwind path. These have credible consumers in native/resource lifecycle
work, even though ordinary diagnostics settle many cases.

Use a progression: original diagnostics and IDE capture/type explanation; selected compiler
THIR/MIR artifacts; then a pin-specific structured driver only for repeated questions that
need it. The result should name source places, suspension points, adjustments, destruction
obligations and relevant control-flow relationships. Raw region IDs are not an explanation.

Mechanisms: `rustc_driver::Callbacks::after_analysis`, `TyCtxt::typeck`,
`TypeckResults::{expr_ty, expr_ty_adjusted, expr_adjustments, type_dependent_def_id}`;
THIR's typed/desugared body and destruction scopes; MIR initialization/liveness/move paths,
`Analysis::iterate_to_fixpoint`, `ResultsCursor`, `ResultsVisitor`, drop elaboration and
coroutine saved-local/capture information. Keep storage existence, initialization, future use
and active loans distinct, with the queried MIR phase and may/must interpretation.

Borrow detail can use `get_bodies_with_borrowck_facts`, `BorrowSet`, `RegionInferenceContext`
and place-conflict machinery. This is a heavier option: upstream warns that extraction copies
bodies, can encounter already-stolen MIR and is unsuitable for typical compilation. Generic
dataflow is largely intraprocedural/conservative; it is not a replacement borrow checker.
Retain Polonius fact export only as a research lead; full execution/comparison needs a separate
question and must not change the production checker merely to explain an error.

A drop edge does not prove external process termination, native callback completion, executor
scheduling or actual memory release. Async state layout identifies representation, not observed
allocation or cancellation behavior. Use original runtime evidence for those claims.

Delivery scenario: explain a selected guard's capture/suspension or partial-move destruction
obligation, with source spans and honest unsupported cases; compare against the original
compiler result. Do not require a custom compiler driver before delivering O01–O13.

Sources: [compiler overview §§2–5](../libraries/rust_analyzers/rust-compiler-internals.md),
[THIR](https://rustc-dev-guide.rust-lang.org/thir.html),
[dataflow inspection](https://rustc-dev-guide.rust-lang.org/mir/dataflow.html),
[borrow-fact extraction](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_borrowck/consumers/fn.get_bodies_with_borrowck_facts.html),
[filtered MIR artifacts](https://rustc-dev-guide.rust-lang.org/mir/debugging.html).

<a id="O15"></a>
### O15 — Inspect constants, concrete layouts and code-generation consequences

**Retain as selected specialist investigations.** I would use evaluated constants/discriminants
for const-generic or array mismatches, size/offset/ABI information before an FFI change, and
concrete generic-instance information when an abstraction has surprising code-size effects.
These are separate questions; none should become automatic metadata on every symbol card.

| Question | Useful output and enabling route | Limit and lighter alternative |
| --- | --- | --- |
| What value does this constant have here? | Compiler CTFE value/error with substitutions/context; `const_eval_*` and static-initializer queries | Generic or pointer-bearing values may not be scalar JSON. A small explicit probe or compile-time assertion often suffices. Analyzer `Const::eval` is a separately labeled result. |
| What is the representation of this concrete type? | Target/toolchain-bound size, alignment, offsets and variant/niche details; compiler `LayoutOf::layout_of` | Start with `size_of`, `align_of`, `offset_of!` or selected `-Zprint-type-sizes`. Analyzer recursive layout is explanatory, not a compiler ABI guarantee. |
| How is this function passed at the native boundary? | Concrete argument/return ABI report; `FnAbiOf` queries or selected backend IR | Rustdoc ABI/unsafe/`repr` metadata helps inspect declarations but proves neither portable layout nor C-side agreement. |
| Which generic instances or shims are emitted? | Definition-to-substitutions/codegen-unit relationships; `Instance`, mono-item collection | Emission is not execution. Selected native compiler output or established binary/LLVM inspection tools may be enough. |
| What changed between selected compiler passes? | Body/pass-filtered before/after MIR and source pointers | Useful for one optimization question; no speed claim follows without relevant generated-code and workload evidence. |

HIR/THIR can explain desugaring before CFG details are necessary. Analyzer `view_hir`,
`view_mir`, recursive layout and constant queries are eligible debugging views with explicit
provider identity. Its highly experimental function interpreter is not a replacement for
compiler CTFE, probes, native execution or numerical validation; do not expose it as general
trusted evaluation.

Compiler self-profile/timing machinery is a conditional tool-debugging lead if analysis/build
cost becomes the actual investigation. It is not a recommendation to revive build monitoring
or add routine profiling to code intelligence. Scientific calculation observation remains the
priority already selected in Plan59f.

Delivery scenario: inspect one concrete boundary type or constant in the selected target,
retain unsupported/generic cases, and avoid presenting the result as cross-platform ABI or
runtime-performance proof. Full custom MIR passes/rewrites are outside this proposal.

Sources: [compiler overview §§4–6](../libraries/rust_analyzers/rust-compiler-internals.md),
[constant evaluation](https://rustc-dev-guide.rust-lang.org/const-eval.html),
[layout interface](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/layout/trait.LayoutOf.html),
[ABI interface](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/layout/trait.FnAbiOf.html),
[monomorphization](https://rustc-dev-guide.rust-lang.org/backend/monomorph.html).

<a id="O16"></a>
### O16 — Assemble a focused investigation packet across source and evidence

**Recommend compositional support; avoid automatic enrichment.** I would use a packet for
one edit or failure: selected definition and contract, relevant callers/references/tests,
pinned dependency APIs, original diagnostic and a few next actions. That reduces repeated
orientation and gives another agent a useful starting point.

Build this from explicit selected views, with per-section provenance and detail budgets.
Allow a compact summary plus double-click expansions, and batching of related positions in
one analysis context. Do not turn the packet into an independent maintained knowledge graph,
repeat an entire architecture review, or attach all library documentation by default.

When Plan59f's original observation identifies a code location/build, offer a source follow-up
against matching sources. If the running binary differs from the working tree, make that
difference visible; current source is not silently the source of the observed execution.
Static callers can guide investigation but cannot invent causal runtime edges or map scientific
model elements to Rust declarations without an actual mapping owner.

Mechanisms: D01 briefs/changes, A03 projection and next actions, B02 original diagnostics,
F01 correlation and O01–O15's selected outputs. Emit derived packet content with source
references; do not maintain a second mutable plan, failure or scientific-status ledger.

Delivery scenario: an agent opens an original failure, follows the implicated call to its
definition and relevant API/tests, and hands that focused evidence to a colleague without
recompiling, rereading unrelated documents or misattributing the running source generation.

Sources: [Plan59d D01](../plans/59d-knowledge-and-workflow-verbs.md#d01),
[Plan59a A03](../plans/59a-cli-foundation.md#a03),
[Plan59f F01–F03](../plans/59f-simulation-observability-and-diagnostics.md).

## Capability coverage and deliberate non-adoption

The eight documents contain overlapping APIs, implementation infrastructure and genuinely
different outputs. The opportunities above retain useful activities; this disposition closes
the remaining families without turning every available API into a new command:

| Capability family from the supplied documents | Disposition and reason |
| --- | --- |
| Lossless/tolerant syntax, AST construction, tree editing, module/item structure | O01/O09; useful where reading incomplete code or semantic edits require it. Rowan is backend infrastructure, not its own agent output. |
| Salsa queries, ItemTree/DefMap decomposition, immutable snapshots, cancellation, LRU/GC | Shared analysis realization below; delegate incrementality instead of creating a second query cache. |
| CrateGraph, project model, rust-project.json, VFS/source roots, load-cargo | Preparation/context mechanisms. Non-Cargo support only if a concrete consumer needs it; static loading alone does not synchronize changes. |
| Names, scope, symbols, definitions, types, signature help, module navigation | O01/O02/O05/O08. Preserve discovery versus use-site resolution. |
| Generics, substitutions, associated types, supertraits, dyn compatibility, impls, predicates | O01/O04/O05/O10; do not collapse declared relationships and proven applicability. |
| References, callers/callees, monikers | O03/O04; identities remain scoped and cross-build matching is qualified. |
| Inlay hints, captures, adjustments, binding modes, related exits/yield points | O02/O14. Select categories/ranges rather than every hint in every read. |
| Semantic tokens, colors, HTML highlighting | Low direct value to a text agent. Use category-filtered annotations or related occurrences when useful; retain full token export only for an actual renderer/consumer. |
| Completion, imports, postfix transformations, term search | O08/O09; candidate generation and edit preview, with synthesis explicitly optional. |
| Diagnostics, failed obligations, suggestions, fixes | O10/O14; provider and compiler verdict remain separate. |
| Rename/file rename, assists, SSR, multi-file SourceChange | O09; preview has independent value. Application is a separately specified effect. |
| Tiny editor actions: enter handling, join lines, brace matching, selection extension, move selection | No standalone CLI integration recommended now. Ordinary source/edit tools suffice; expose through a selected assist only if it advances a real task. |
| Macro expansion, token descent, original source mapping, generated code | O13; missing or lossy mappings remain explicit. |
| Runnables, test discovery, related tests, benches | O07; navigation/candidate role, original execution owner retained. |
| Analyzer HIR/MIR views, constants/layout, internal interpretation | O14/O15; selected debugging artifacts. Interpreter remains experimental, not a general evaluation product. |
| SCIP/LSIF, StaticIndex, offline cross-reference export | Conditional alternative for repeated immutable-revision browsing or an actual external code browser. Batch cost, scope and staleness must earn their upkeep; no duplicate live/persistent graph by default. |
| Analyzer analysis-stats, unresolved-reference tools, RA_LOG/RA_PROFILE, status/progress/memory views | Use native mechanisms to diagnose the analysis tool when needed. Compact health/readiness is useful; a new observability stack or routine whole-workspace analysis campaign is not. |
| Rustdoc API graph, aliases/reexports, structured attributes/stability/hidden structure | O05/O12/O15. Unknown/stripped parts are not empty declarations or universal stability claims. |
| Rustdoc HTML search, themes, browser assets and presentation configuration | Native browsing/signature search remains useful. No CLI theme/browser asset API or parser over internal search JavaScript. |
| Docs links/examples/doctest extraction/scraping/lints/coverage | O06/O10/O12. Coverage is a lead, extraction is not execution, scraping is optional. |
| cargo metadata/tree, reverse graphs, Guppy feature simulation, unit graphs and Cargo artifacts | O03/O11/O12; each retains its level of precision. Hakari is not selected for this read-only capability. |
| Public API/compatibility tools | O12; existing public-api and cargo-semver-checks before bespoke rules. |
| rustdoc-types fast hashing, rkyv archives, serialization/format migration | Conditional physical choices for repeated large artifacts. rkyv requires JSON-to-archive conversion; it is not zero-copy JSON. No eager full-corpus archive requirement. |
| rustc HIR/THIR/type checking, MIR stages, moves/loans/dataflow/drop/coroutines | O14/O15; source-linked explanations, appropriate compiler phase and explicit uncertainty. |
| CTFE, layout/ABI, instances, monomorphization, optimizations/codegen | O15; concrete investigations, not runtime proof or a default compilation pass. |
| Compiler query caching/incrementality and diagnostic emission | Reuse original compiler output and internal mechanisms; no stable external compiler database is implied. |
| Custom MIR transformations/pass injection, replacement borrow checker, automatic Polonius execution | Not recommended. Large semantic/maintenance obligation without a selected code-intelligence need. Fact export is an explicit future research option. |

## Proposed realization and alternatives

### One task surface over distinct evidence providers

The CLI should own query intent, source/configuration selection, presentation and next actions.
Libraries own syntax, inference, references, transformations, documentation extraction and
compiler analysis. Keep provider-specific capabilities available rather than forcing all
providers into the same lowest-common-denominator response.

The necessary semantic distinctions are small: query subject, selected context/revision,
relationship kind, provider, result/availability state, source locations and optional proposal
or artifact. Do not build a general source ontology, graph query language or universal symbol
database before concrete outputs need them. Library-local handles and rustdoc numeric IDs
must not become durable cross-revision identity. A source pointer/candidate handle may be
revision-scoped and reacquired after edits; not every lookup requires a permanent UUID.

### Prefer managed LSP for ordinary live semantics; keep richer native options open

| Realization | Where it fits | Judgment and reopening condition |
| --- | --- | --- |
| Existing text/syntax tools plus Cargo/rustdoc | Cheap source reads, package facts, pinned API structure | Retain as useful independent capabilities. They do not resolve every use site. |
| Rust-analyzer LSP | Ordinary navigation, references, completion, hints, code actions, ongoing workspace management | Preferred initial semantic boundary. Upstream identifies LSP as the more stable integration boundary. Negotiate extensions and verify selected binary compatibility. |
| Embedded `ra_ap_ide` with project-model/load-cargo | Rich typed results or application control not available through selected LSP | Eligible if material O02/O04/O13 capabilities cannot be represented faithfully otherwise. Compare complete VFS/config/proc-macro/watch/cancel ownership before choosing it. |
| Direct HIR or lower internal crates | Specialized structured semantic queries beyond the IDE facade | Use only for a specific contract gap. Do not embed the whole frontend merely to get a definition location. |
| SCIP/LSIF or `StaticIndex` snapshot | Repeated browsing/export of a fixed revision | Conditional alternative, with exact snapshot scope. Do not require alongside live analysis without a distinct consumer. |
| Normal compiler diagnostics/probes/filtered artifacts | Correctness result and selected hard semantic questions | Prefer before a custom driver. Original output can be reused for explanation without rerunning work. |
| Pin-specific `rustc_driver` component | Repeated structured compiler questions that lighter outputs cannot answer | Separately prepared and capability-checked. Avoid loading compiler-private internals into every ordinary host invocation. |

LSP does not imply a network service or MCP server; stdio can connect a client and server.
Standard requests and rust-analyzer-specific extensions have different compatibility promises.
The extension documentation is best-effort; server capability negotiation/source resolve exact
contracts. Debug HIR/MIR/hover text must not be parsed into a falsely authoritative typed model.
If structured adjusted types/capture provenance require native integration, that is a genuine
design question for selected opportunities, not grounds to silently omit their value.

External compiler drivers require `rustc-dev` and `llvm-tools` and couple to compiler-private
APIs. Stopping before codegen can avoid linking but does not eliminate dependency/proc-macro/
build-script preparation. The current pin's exact interface remains unresolved. No alternate
nightly or stable override is recommended.

Primary boundaries: [analyzer architecture](https://rust-analyzer.github.io/book/contributing/architecture.html),
[LSP extension contracts](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/lsp-extensions.md),
[external compiler drivers](https://rustc-dev-guide.rust-lang.org/rustc-driver/external-rustc-drivers.html).

### Reuse analysis without creating a coordination platform

Repeated CLI invocations should have a credible reuse route. Reloading the workspace for every
definition/reference request forfeits much of an incremental analyzer's value. Compare a
demand-started reusable session with explicit bounded batch sessions during planning. A CLI
process cannot simply attach another client to an editor-owned stdio server; editor sharing
requires an actual supported broker and must not be assumed.

Recommended direction: one owned analysis session per compatible checkout/configuration as
needed, with concurrent queries sharing preparation, explicit readiness/status/stop, and a
simple documented idle/session-end policy. It should survive an individual query only when
there is an intended reuse lifetime. Canceling one client request must not tear down the shared
session. An input revision change may legitimately cancel all outstanding old analysis
snapshots; affected requests return canceled/stale and reacquire current context rather than
silently mixing generations. Session teardown must settle its owned analyzer/proc-macro
children through the existing host mechanism. No durable job queue, distributed lease
protocol, database or resumable query engine
is justified for disposable analysis state.

Use analyzer-native Salsa invalidation, snapshots and cache management. The adapter supplies
file/configuration changes, including newly created/deleted files, generated output changes,
manifests and source roots. A Git HEAD identifier alone is insufficient in this shared dirty
checkout. Identify the synchronized analysis generation and expose pending reload/build/macro
state or observed source mismatches. A coherent engine snapshot does not establish a globally
atomic filesystem snapshot: ordinary watcher delivery alone provides no such guarantee.
Stronger cross-file capture, exact change synchronization and continuation validity remain
planning questions. Do not invent a per-query whole-repository hash scan or second fine-grained
dependency cache when the engine already owns incremental inputs.

Compatible batching is useful: explain a few related positions/ranges in one context, hydrate
selected hints/docs/actions together, and avoid repeated file or crate generation. Do not
impose arbitrary serial execution or smaller workstation CPU/RAM caps. Use the maintainer's
ordinary placement and upstream supported concurrency; any additional bound needs an actual
query/lifecycle reason.

Cold preparation must be explicit. Help, catalogue, source-only reads, cancellation and ordinary
activity diagnosis must not start analyzer builds or require compiler-private components.
Do not enable a second automatic Cargo check loop beside the existing correctness owner.
Disabling check-on-save alone does not suppress build-script/proc-macro preparation. Native
`cargo.buildScripts.overrideCommand` can align that work with the repository's admitted
environment, but its Cargo JSON and process behavior must be verified for the selected route.
Useful restricted analysis can run with missing preparation if the incompleteness is explicit.

Native controls include build-script/proc-macro settings, cache priming, thread settings,
LRU capacities, cancellation, progress and `RA_LOG`/profiling. Start demand-driven rather than
priming every analysis stage. Full workspace references and empty-result proofs can still be
expensive; depth/output limits do not make them cheap. No latency or memory promise follows
from these structural choices.

Sources: [analyzer overview §§3–4, 8–10](../libraries/rust_analyzers/rust-analyzer.md),
[configuration](https://rust-analyzer.github.io/book/configuration.html).

### Artifact/context fidelity without a second authority

Rustdoc JSON should retain D02's package/source/features/target/cfg/toolchain/schema identity
and check `format_version` before typed decoding. Context includes extraction flags and
documentation-specific conditions; published hosted JSON carries its own provenance.
`rustdoc-types` is the schema, not a complete trait solver, cache, source index or compatibility
engine. Optional rkyv/fast hashing may help repeated large reads after their conversion and
retention costs are justified.

Analysis context and compiler selection should share facts through their existing owners but
remain honest about representational differences. A file can belong to multiple crate
instances. Metadata-level unified features do not by themselves establish every host/target/
dev unit context. Return alternatives or request a precise selection where that changes the
answer. Do not silently analyze all features or run a full feature matrix to answer a normal
question.

Preserve origin and scope through composed packets: declared API, analyzer inference, compiler
check, compatibility diagnostic, observed runtime fact and heuristic candidate are distinct
evidence kinds. Original artifacts may be referenced rather than copied into every response.
Persist only artifacts that earn reuse, inspection or handoff value. User-authored probe inputs
and edit intent are not disposable analysis caches.

## Priorities, interactions and subsequent planning decisions

### Architectural assessment of the proposed direction

AP-01/AP-02 favor one integration owner for provider-specific loading/protocol details while
ordinary CLI consumers depend on the query meaning they actually use. The main alternative,
embedding frontend internals throughout handlers, would make analyzer upgrades affect unrelated
commands. LSP is the preferred starting boundary, not a reason to discard useful structured
capabilities that require a different integration.

AP-03 favors composition of selected answers and original evidence over copied investigation
workflows. O16 should assemble existing views, not create another interpreter of diagnostics,
test selection or scientific outcomes. AP-04/AP-05 require the explicit distinctions above:
use-site resolution versus candidate discovery, declared impl versus applicability, source
revision versus binary generation, compiler evidence versus IDE inference, and proposal versus
applied edit. A uniform JSON envelope without these meanings would be inadequate.

AP-06 favors independently usable source/API reads and query-local verification. Neither a
database nor full native solver preparation should be necessary to understand an ordinary
Rust declaration. Actual native/compiler actions retain their real prerequisites. Selected
consumer journeys should challenge ambiguity, stale context, missing macros and concurrent
edits; tests of schema shape alone would not establish useful semantics.

AP-07 and DP-09/DP-10 favor shared incremental preparation, native invalidation, selective
hydration and deliberate analysis lifetimes. Cold reload per lookup, eager whole-workspace
indexing, item-by-item rustdoc generation and parsing enormous debug dumps into a second
database are materially worse starting points without a specific consumer benefit. The
remaining tradeoff is the lifecycle burden of warm reuse versus batch loading; this review
does not claim that performance comparison has been measured. DP-13–DP-16 support native
analysis/refactoring/compatibility mechanisms and thin adapters over bespoke equivalents.

These are qualitative assessments of the proposed realization, not acceptance of an
implemented design or a new formal gate campaign.

The recommendation is a broad functional target with selective delivery, not sixteen new
mandatory services or command families:

| Group | Recommended capabilities | Why and dependencies |
| --- | --- | --- |
| First useful semantic journey | O01, O03, selected O02; O10 original-diagnostic navigation; O11 context facts | Repeated agent questions with a narrow consumer path. Settle context, reusable session and output before breadth. D02/D03 remain independently useful. |
| High-value extensions of that journey | O04, O07, O08, O13; O05/O06 API enrichment; O16 selected composition | Reuse much of the same analysis/API preparation. Deliver independently where supported; do not make every extension a blocker for initial navigation. |
| Explicit edit and migration workflows | O09, O12 | Potentially substantial error reduction, with revision-bound previews, native transformations and compatibility tools. Preview can precede apply. |
| Specialist investigations | O14, O15 | Credible ownership/native-boundary value. Prefer existing diagnostic/artifact/probe routes; custom driver only for selected structured needs. |
| Conditional representations/research | Static index export, archives, term synthesis, Polonius facts, detailed codegen/profile investigations | Retain triggers and useful mechanisms; no default installation, eager corpus or background campaign. |

These groups are not a strict serial chain. For example, pinned API capability search can
advance without a live semantic session, while related tests can use that session without a
compiler driver. Detailed execution packages belong to separately instructed planning.

Consequential decisions to carry into that planning:

1. Select the actual end-user query families and map their required structured fields to the
   pinned LSP/native capabilities. Do not promise fields unavailable through the chosen route.
2. Compare reusable versus explicit batch analysis using real repository questions; assess
   cold preparation, repeated queries and edits only enough to settle the lifecycle choice.
   No universal benchmark campaign or numeric performance gate follows from this review.
3. Establish source synchronization, cfg/crate-instance identity, Unicode positions, request
   cancellation and stale continuation/edit behavior in the shared checkout.
4. Route build-script/proc-macro work through existing preparation and placement; preserve
   force-validation wherever actual Arrow-consuming correctness work occurs.
5. Decide whether selected semantic edit delivery is preview-only or includes application.
   If applying, specify the coherent multi-file publication/concurrent-edit boundary first.
6. Keep native/compiler-private components separately prepared; select one real compiler
   question before taking on exact-pin driver maintenance.
7. Enrich D02's API availability wording: known gates and extraction context can be reported,
   while arbitrary ordinary-build availability still needs the actual selected compiler/probe.

### Rule and plan impacts

<a id="RC01"></a>
**RC01 — Extend Plan59's capability/output scope.** A03/A06 and D01–D03 would gain selected
semantic queries, typed availability/context, source/proposal views and discovery entries.
Reuse the existing envelope and store resolver. Token-size aims refine presentation rather
than redefining `--budget` as byte truncation. Existing planned scope is not amended here.

<a id="RC02"></a>
**RC02 — Decide the analysis-session lifecycle explicitly.** A reusable analyzer is additional
developer-tool process scope. It is not automatically covered by detached command supervision
or a reason to replace the invocation ledger. A05/B03/operating guidance must define any
selected owner, preparation, child settlement, reuse and teardown. Reuse existing mechanisms;
any new crate/service or enduring contract change follows its normal scoped decision route.

<a id="RC03"></a>
**RC03 — Preserve selection and evidence authority.** C07 owns actual command resolution;
C05/nextest own actual binary/test listing; F owns scientific observation. Related tests,
static calls, analyzer diagnostics and documentation projection cannot supersede these. New
diagnostic/evidence payloads and source-generation correlation need explicit contracts where
they change the current target.

<a id="RC04"></a>
**RC04 — Keep MCP/configuration exclusions scoped.** CLI integration can use stdio LSP without
installing an MCP bridge, modifying Codex/editor settings or reviving the rejected rustdoc-mcp
trial. Selected plan wording should distinguish the newly proposed CLI capability from those
historical exclusions. The review authorizes no runtime configuration change.

<a id="RC05"></a>
**RC05 — Treat edits and specialized compiler work as distinct effects.** Semantic apply,
probe execution, dependency/API generation and compiler-private preparation must be explicit.
Protected/generated ownership and existing compilation/verification policy still apply.
Third-party dependencies use the ordinary exact-pin policy; new crates or contract changes
use the decision route when triggered, not a blanket new approval requirement for libraries.

## What would establish useful delivery

The following are representative consumer journeys for later planning, not tests run here or
a requirement to build an exhaustive fixture/check framework:

| Journey | Observable value and necessary boundary |
| --- | --- |
| Resolve and understand an overloaded call | The agent selects the actual method/type, reads only the relevant definition and obtains a selected implicit-semantic annotation. Ambiguity/unknowns remain visible. |
| Investigate an ownership API edit | References, one caller neighborhood and candidate tests guide inspection; conservative validation remains separately selected. |
| Construct an unfamiliar library call | Capability search returns a usable public path/bounds/example, contextual candidates identify an import, and an explicit probe resolves consequential applicability doubt. |
| Preview a semantic migration | Rename/SSR avoids unrelated names; the preview covers affected files. Detected source changes invalidate the proposal; any selected apply route exercises its actual concurrency/publication and partial-failure contract. |
| Continue after source/configuration changes | Add/delete/edit files and change features; the next query updates or reports stale/incomplete context. It never silently reuses the previous configured world. |
| Diagnose missing preparation | Source-only reads and original run diagnosis remain usable; semantic absence is distinguished from unavailable macro/build context. |
| Explain an original compiler failure | Relevant spans, children and suggestions survive; follow-up source/API inspection uses their actual generation or exposes the mismatch. |
| Investigate one async or FFI question | Selected compiler/probe evidence answers the stated capture/drop/layout question without claiming external-resource settlement or cross-target ABI correctness. |
| Reuse and stop analysis | Repeated useful queries share preparation; independent query cancellation and whole-session teardown have distinct observed outcomes. No orphan child or fabricated success is accepted. |

Actual benefit should be assessed through agent use of these journeys: whether the response
changes the next action, avoids redundant reading, or exposes a wrong assumption. Timings and
memory claims require named measurements if made later. A passing small fixture or an inspected
upstream signature alone would not establish usability across this workspace.

## Research coverage and evidence limits

| Supplied input, read end to end | Principal contribution to this review |
| --- | --- |
| [ra_ap_hir](../libraries/rust_analyzers/ra_ap_hir.md) | Resolution/scopes/types/traits, captures, macros/origin, constants/layout, predicate and semantic API boundaries. |
| [ra_ap_ide](../libraries/rust_analyzers/ra_ap_ide.md) | Navigation, references/calls, hints, completion, diagnostics, edits, tests, static indexing and snapshots. |
| [rust dependency tooling](../libraries/rust_analyzers/rust_dependency_tooling.md) | Metadata/tree/Guppy, feature and package reach, unit graphs, artifacts and interpretation limits. |
| [rust-analyzer](../libraries/rust_analyzers/rust-analyzer.md) | Syntax/incremental architecture, loading/VFS, LSP/native/index alternatives, configuration, execution and tool-health mechanisms. |
| [compiler internals](../libraries/rust_analyzers/rust-compiler-internals.md) and [continued](../libraries/rust_analyzers/rust-compiler-internals-continued.md) | Identical material: dataflow/borrow/drop/coroutine/pass mechanisms and compiler integration; primary follow-up broadened constants/layout/ABI/instances. |
| [rustdoc-types](../libraries/rust_analyzers/rustdoc-types.md) | Typed API graph, reexports, constraints/traits/attributes, schema/context identity and archive options. |
| [rustdoc](../libraries/rust_analyzers/rustdoc.md) | Signature search, examples/doctests, diagnostics, JSON/HTML, compatibility analysis and acquisition alternatives. |

Context7 was consulted for analyzer, Cargo/rustdoc and compiler documentation, followed by
primary upstream docs/source for consequential details. Some exact versioned docs.rs analyzer
pages were unavailable; development API pages supplied capability evidence, not verification
of every 0.0.358 signature. Compiler-private pages also span revisions, so exact pinned-driver
compatibility remains unresolved. Empty search results were not treated as capability absence.

The root integrated independent library-research judgments and current code/plan inspection.
No performance or benefit was measured. No implementation, plan status, formal ADR acceptance,
scientific qualification or previously stopped campaign is advanced by publication. Selected
opportunities can feed subsequent plan creation with their concrete mechanisms and limits intact.
