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

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [J01](#j01) | in-progress | 59j | Minimal working-source identity/text prerequisite implemented/tested; Git sources, positional reading, bounded pages and continuation extensions remain |
| [J02](#j02) | in-progress | 59j | Replacement-only immutable proposal validation implemented/tested; complete semantic/file-operation producer integration remains |
| [J03](#j03) | planned | 59j | Explicit syntax queries and structural replacement previews |
| [J04](#j04) | planned | 59j | Generated hit to declared contributor and regeneration navigation |
| [J05](#j05) | planned | 59j | Actual server-free source/query/preview consumer journey |

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
remain outside the paused execution scope; these dependency repairs do not authorize them.

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

**Proposed verification:** targeted compile/unit/parser tests during implementation, followed
by J05's actual consumer and relevant series-end integration/static checks under AGENTS.md.
No benchmark, solver/provider build, semantic preparation or full scientific campaign is
required merely to qualify this source workflow. Benefits remain unmeasured. All packages
are planned and execution remains paused; this document establishes no runtime completion.
