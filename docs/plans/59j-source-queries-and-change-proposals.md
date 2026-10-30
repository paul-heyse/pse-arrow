---
title: "Plan59j: Source queries and change proposals"
status: active
date: 2026-10-30
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, design, libraries, diagnostics, validation]
review_sources: [docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md, docs/functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md]
---

# Source queries and change proposals

## Purpose, baseline and boundaries

**Proposed:** give agents a direct change→source→query→preview→owner workflow without preparing
a semantic session. This companion owns the source/proposal portions of review I07–I09/I16;
the [coordinator](59-agent-cli.md#library-integration-dispositions) owns their disposition.
The baseline is `0c8c17ebf`, with production at the paused `09e878957` checkpoint. No J package
is delivered by authoring this document. Core3.4, efficiency heuristics1.0, test assessment1.2
and applicable PS1.5 guidance apply through the binding.

D03's native lossless outline/show and coarse Rayon batch are implemented foundations, not
evidence for the extensions here. A03 owns outer output and wire/schema policy; A04 owns
artifact addressing/retention. D01 owns focused Git changes; D06
owns generated-output protection/regeneration routes. G owns semantic context and producers;
G08 owns the conditional filesystem-application decision. D08 is deferred; D09 preserves its
historical Git publication investigation without becoming an application prerequisite.

The operator accepted review RC03/RC04 on2026-10-30: migrate immature response/proposal shapes
directly and remove the semantic-session prerequisite for syntax-only queries/previews.
A08 publishes any required governing amendments before dependent implementation. This plan
does not amend architecture or ADRs, start implementation, prepare tools or authorize edits.

## Target and mechanism choices

One source acquisition contract addresses explicit working bytes, index bytes or committed
blobs. It binds repository context where applicable, exact path bytes, source kind and revision
or digest; a mutable HEAD selector is resolved before becoming an immutable source reference.
Positions/ranges are reusable inputs, not synthetic symbol names. Preserve exact UTF8 byte
ranges and documented conversions from human line/column coordinates. An impl label is not
resolved identity. Working-tree acquisition is not an atomic multi-file snapshot.

Use operation-owned borrowed page/view records under A03. Query scope, examined coverage,
returned records and omitted detail are distinct. Every page retains source/backend context;
continuations either bind an immutable source or refuse when the working source changed.
Do not reconstruct old dirty bytes from current files or discard sole original evidence as
regenerable. A04 supplies retention where a consumer promises exact-original-source replay.
Fresh queries can instead declare fresh-source semantics without storing every response.

| Mechanism | Selected responsibility and limits |
| --- | --- |
| `ra_ap_syntax0.0.360` | Rust declaration/context reads and local CST construction, including recovery and explicit editions. Use native offset/range/ancestor access; syntax is not resolution. |
| `gix0.90.0` | Reuse the repository owner to address selected index/tree/blob bytes and diff locations. Raw selected blobs do not depend on current worktree attribute conversion. Preserve byte paths and staged/unstaged separation. |
| `ast-grep-core` plus tree-sitter-rust | Selected code-shaped pattern/capture/query/replacement consumer, fallible pattern construction and explicit strictness. Qualify exact pins/runtime-grammar ABI before adoption. No third always-running parser service. |
| Existing syn/quote/prettyplease | Owned generated fragments and existing generators. Use typed parsing/visitors/folds and token interpolation where useful; do not whole-file round-trip authored text and lose comments/layout. |

The full capability references and exact-release leads remain in review I08/I09/I16. In
particular, inspected ast-grep0.51.0 insertion helpers contain `todo!()`, replacement enumeration
can omit nested overlaps, and native `SyntaxEditor` edits must use its actual editing root.
These limits constrain selected operations rather than justify bespoke pattern engines.

Share one minimal change proposal across syntax and semantic producers: original source
references, affected files and file operations, exact replacement ranges/text, provider mode/
context, conflicts/omissions and unresolved placeholders. Backend-local nodes stay inside their
producer. All matches, selected nonoverlapping edits and applied writes are different concepts.
A proposal establishes no equivalence proof, compilation success or publication transaction.
Generated fragments cannot silently consume additional effects or alter evaluation order.

## Package state

The compact proposal output now has payload version2. Its one borrowed serialization/schema
view supplies exact source identities and selected original/replacement regions; complete
original inputs remain inside the checked proposal for validation. The declared schema fixes
that version through the same constant and rejects version1. This is a direct immature-wire
cutover under RC03; it adds no apply authority or historical-response compatibility path.

The maintainer authorized the remaining whole series, including J03–J05, on 2026-10-30.
The current source implements addressed working/index/blob acquisition, positional/range reads,
source-bound nested declaration pages, shared proposals with file-operation intent, and explicit
syntax query/preview producers. Focused actual CLI controls now exercise raw staged/committed
bytes, Unicode positions, stale continuations and structural previews. Projection-safe provider
context and generated-hit composition are delivered; semantic producers now translate into the
same proposal contract. The earlier paused-scope statements describe the previous wave, not
current authorization. Final parser publication and series qualification remain with D07/P01.

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [J01](#j01) | complete | 59j | Working/index/blob acquisition, Unicode positions/ranges, nested bounded pages and exact-source continuations pass actual CLI controls |
| [J02](#j02) | complete | 59j | Shared checked proposal and compact version2 view serve syntax and native semantic/file-operation producers; stale/overlap/placeholder controls pass |
| [J03](#j03) | complete | 59j | Native syntax queries, captures/context constraints and explicit immutable previews pass targeted and actual CLI controls |
| [J04](#j04) | complete | 59j | Actual generated hit follows declared contributor/emitter and regeneration routing; unknown authored location remains unknown |
| [J05](#j05) | complete | 59j | Actual source-only query→preview→origin→emitter journey passes without semantic preparation or source writes; series qualification remains separate |

<a id="j01"></a>
### J01 — Addressed source and bounded reading

**Consumes:** D03's working readers; A03's owned-view/schema slice. Git sources reuse the
existing repository owner; generic D01 packet assembly is not selected. Exact-source retention
consumes only A04's artifact slice where needed.

Extend outline/show with positional enclosing-declaration/range selection and reusable source
locators. Preserve qualified paths as useful human navigation, but avoid requiring them when
a compiler/diff already supplies a location. Read working/index/blob sources without checkout
or Cargo preparation; state direct-base versus merge-base semantics at Git comparison inputs.
Markdown retains its existing CommonMark reader. A malformed Rust neighbor must not erase a
useful selection. No persistent graph or parse cache is required for ordinary one-file reads.

Make the declaration collection the bounded/pageable result unit. A record allowance must
bound nested outline entries as well as batch operands; mandatory context and indivisible
records survive. Replace the unbounded response construction in affected outline/show and
lib-outline consumers together, with A03 directional view schemas.

**Acceptance:** positional selection in duplicate impl labels; exact Unicode slicing; missing
and malformed sources; staged/working/committed differences; nonUTF8 path handling; nested
outline budget actually limits entries; source edit rejects an exact-source continuation.

<a id="j02"></a>
### J02 — Shared source-bound proposal

**Consumes:** J01 source identity/location contract and A03 typed serialization. This package
does not require the J03 pattern engine or G03 semantic session.

Implement proposal constructors and validation once. Reject inconsistent source/ranges,
overlaps in the selected edit set and unsupported file operations; retain conflicting match
candidates as explicit findings rather than dropping them. Expose a diff/inspection view and
opaque original evidence where useful. No factory permits syntax-derived records to claim
resolved symbol identity. G07 translates semantic source changes into this same contract,
retaining its synchronization/configuration context and placeholders.

**Acceptance:** two independently authored syntax/semantic-shaped inputs use the same proposal
consumer; stale identity, out-of-range UTF8 boundaries and overlapping selected edits refuse;
unresolved placeholders remain visible. Proposal creation modifies no source or Git index.

<a id="j03"></a>
### J03 — Syntax queries and structural previews

**Consumes:** J01/J02 working contracts and the applicable A08 authority slice. No G01–G03
preparation is a prerequisite. Use an explicit syntax mode in the discoverable code family;
exact parser spelling is fixed with A02/A06, not by copying illustrative review commands.

Deliver code-shaped patterns, named/sequence captures, composed context constraints, selected
capture extraction and replacement previews over an explicit source set. Use native
`Pattern::try_new`, matcher composition, `MetaVarEnv` and supported `TemplateFix`/edit operations
after exact-release qualification. Report grammar/pattern version, matching strictness,
parse-recovery regions, examined scope and limits. Enumerate nested matches separately from
the nonoverlapping edit proposal. Include selected typed generated-fragment construction when
it removes handwritten token/string rewriting; keep original untouched ranges exact.

Never silently satisfy semantic rename/SSR through syntax matching. A missing semantic
provider returns unavailable with syntax offered only as a separately selected alternative.
Thread-affine parser objects stay worker-local; reuse the coarse shared-pool tokio-rayon
execution owner, including retained completion after cancellation, rather than another pool.

**Acceptance:** same spelling/different bindings remain syntax candidates; capture-preserving
preview, nested matches, malformed syntax, edition/macro boundaries and native API refusal
retain honest scope. Cancellation joins actual CPU completion; no source writes or compiler
checks occur in preview. Delete replaced matching/preview duplication when consumers move.

<a id="j04"></a>
### J04 — Generated-source drill-through

**Consumes:** J01 locators and D06's declared output/contributor routing. No J03 or G session
is required. Reuse existing `pse-codegen::codegen::provenance::SourceRelationship` records,
including declaration key/kind, generator path/symbol and source-role/identity relations.

A generated hit offers its known emitter/declaration and regeneration next action. Preserve
the distinction between the output map's protection/command and contributor lineage. Missing
authored locations remain unknown, never inferred from matching names. Extend a producer only
for a demonstrated missing relationship. Move affected source/query/owner consumers together;
do not create another mutable provenance index or an alternate generated editing path.

**Acceptance:** follow a known generated hit to its declared emitter, retain an unknown
location, and show the regeneration action without editing generated output.

<a id="j05"></a>
### J05 — Integrated source consumer

**Consumes:** J01–J04 and A06/D07 discovery/reference slices. No D01 packet prerequisite.
Core source commands remain useful without completing every D01/G/H/I producer.

An actual CLI consumer selects a changed location, reads addressed source, narrows a structural
query, inspects a preview and follows generated ownership where applicable. Missing semantic
preparation must not prevent this journey. Expose selected addressed results directly; G07
and G08 remain independent semantic-production and conditional-application consumers. J03–J05
are included in the maintainer's remaining-series execution authorization.

**Acceptance:** real CLI outputs, matching directional schemas, stale source and unavailable
provider controls establish the selected path. Schema agreement alone does not establish
correct source/meaning: independently authored snippets and ranges challenge the producer.
Delete actually replaced readers/formatters/preview callers after migration, retaining useful
independent source tools. Update E03 guidance alongside delivered behavior.

## Dependencies, verification and handoff

J01 enables J02 and J04 independently; J02 enables shared-proposal consumers including G07;
J01/J02 enable J03; J01–J04 enable J05. Semantic preview adds G03/G01 prerequisites, not the
reverse. G08 can qualify application from J02 plus an actual J03 or G07 producer and D06;
syntax application does not require G07. Code sharing does not merge filesystem and Git
publication guarantees. A08 gates only changed governing contracts, not unrelated inquiry.

## Outcome

**Implemented:** J01–J05 and their D03/D06/A03 consumers are integrated. Working bytes, raw
index blobs and resolved committed blobs keep distinct identities. Nested declaration and query
pages preserve source/backend/omission context, including when payload fields are hidden.
The shared proposal retains complete original inputs internally and serializes exact identities
and selected regions through one version2 view. Syntax and semantic authority remain distinct.

**Tested, 2026-10-30:** the coordinator's first focused65-test command included the actual
`source_addressing` and `source_queries` targets plus source/proposal/query/owner controls.
`working_staged_and_committed_unicode_sources_are_selected_without_checkout_preparation`,
`nested_declaration_pages_retain_coordinates_and_refuse_changed_working_source` and
`unicode_scalar_positions_ranges_and_batch_budget_use_original_bytes` exercised the addressed
source consumer. `generated_query_preview_origin_and_emitter_form_an_actual_source_only_consumer`
followed real CLI query/preview/origin/emitter outputs, validated directional schemas and verified
unchanged generated bytes. Other actual query controls preserve nested matches, captures,
malformed regions, unavailable semantics, undefined replacements and overlap refusal. G's native
rename, assist, file-rename and scoped SSR consumers independently exercise this same proposal.
Whole-series integration/static qualification remains with P01; no scientific result follows.

**Mistakes corrected:** generic record trimming could separate context from payload detail and
compact proposal serialization initially exposed complete unrelated source files. Provider context
now survives projection; one schema/serialization view carries selected regions while checked
originals stay internal. Nullable collections page without splitting fixed tuples.

**Deliberate limits:** no source application is advertised. G08 records why preflight plus
ordinary file replacement does not protect concurrent editors across multiple files. Raw Git
source reads do not apply worktree filters, and syntax candidates never claim resolved identity.
No performance benefit was measured; no solver/provider preparation or scientific campaign is
required for this source workflow.
