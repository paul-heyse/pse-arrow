---
title: "Plan59j: Source queries and change proposals"
status: active
date: 2026-10-30
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, design, libraries, diagnostics, validation]
review_sources: [docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md, docs/functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md, docs/functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md]
---

# Source queries and change proposals

## Current extension — 2026-10-31

**Implemented; targeted Tested:** J06/J07 extend completed J01–J05 with native structural rules/Tree-sitter
extraction and result-to-proposal hydration from
[O04/O11](../functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md#O04).
The maintainer accepted `search`/`inspect`/`propose` hard cutover and compact shared
provenance/streaming on 2026-10-31, preserving identity, outcome, coverage and useful direct
reads. K owns query/result/locator contracts; A10 publishes their governing changes before
dependent implementation. M03 owns public migration and retirement of replaced routes. Earlier
command-family and envelope wording describes the completed baseline; historical completion
and tests do not establish these extensions. Current selected producer/consumer evidence is
recorded below; the coordinator's [P02 qualification](59-agent-cli.md#p02) remains separate.

Proposals remain previews through J02/G07. Hydrating a lightweight result establishes the exact
source premises needed for a preview; it grants no apply authority and cannot fabricate semantic
identity. Known-path/range/index/revision inspection remains directly available without search.

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
| [J06](#j06) | complete | 59j | Native rules/extraction and shared per-file parse implemented; advanced producer and selected public controls pass; P02 scope-end qualification remains separate |
| [J07](#j07) | complete | 59j | Exact result hydration/shared preview validation implemented; selected native/public stale and preview controls pass; P02 scope-end qualification remains separate |

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

## Planned search extension packages

J06/J07 consume the [query/result foundations](59k-query-and-result-foundations.md) and
feed [composed investigations](59m-composed-rust-investigations.md). Historical completed rows
and Outcome cover J01–J05 only; the following packages keep their own implementation and
selected-test outcomes.

<a id="j06"></a>
### J06 — Native structural rules and Tree-sitter extraction

**Consumes:** K01's agreed node/file/entity conjunction, answer and coverage semantics and
existing J01/J03 source/pattern owners. Design and engine qualification need not wait for K02;
public common-request integration requires working K02 and the applicable A10 publication.
Repository-wide selection consumes working L01; explicit-source engine tests can proceed
without it. No semantic session or native HIR implementation is required.

Expose native alternatives/negation, node-kind/field constraints, repeated-capture equality,
ancestor/descendant/sibling relations and enclosing-declaration filters through the common typed
request. Prefer the exact-compatible ast-grep config/rule family, with the review's
`SerializableRuleCore::get_matcher(DeserializeEnv)`/`RuleCore` as integration leads, rather than
adopting lint/fix `RuleConfig` or building a general structural interpreter. Qualify workspace
pins and their compatible core/config/grammar/runtime family before selection; the review's
0.51 lead does not itself bump dependencies. Preserve native binding, strictness, field/relation
and suppression meaning. `CombinedScan` honors suppression comments and collects matches in
the examined implementation; use it only for compatible lint-like batches, never as a promised
unsuppressed streaming ad hoc search engine.

Offer Tree-sitter `Query`/`QueryCursor` for advanced extraction using current compatible grammar/
runtime and `NODE_TYPES` discovery: fields, negated fields, captures, quantifiers, anchors and
text predicates. Validate native queries at their provider boundary rather than inventing a
second syntax language. Report recovery, match-limit, timeout and unsupported predicate outcomes.
Keep byte ranges/captures attached to the original source and provider/grammar context.

Compile selectors/matchers once, parse each selected file once per compatible batch and evaluate
native rule trees over that parse where sound. Make existence/first-results stop upstream work;
exact counts or deterministic global ranking may need complete examination. Do not preserve the
current count-then-reparse collection path for every answer. Negative file predicates require
complete relevant examination; truncated/canceled work cannot establish absence. K03 owns typed
stream framing/terminal coverage, while the engine supplies complete records and actual stopping
facts. Parser objects remain worker-local through the existing CPU ownership route.

**Public consumer and acceptance:** `search` accepts a reusable typed request and `inspect`
expands captures/enclosing declarations. Independent snippets distinguish same-node versus
same-file conjunction, repeated metavariable equality, nested matches, sibling/field constraints,
failed alternatives that leak no captures and negation over incompletely examined files.
Tree-sitter controls cover quantifiers/anchors, predicates, recovery and match-limit refusal.
Include native suppression comments, malformed queries, cfg/macro/edition boundaries and
stale source; no syntax result claims semantic identity. Existence/first-results/count controls
verify examined scope rather than merely a short payload. Cancellation settles actual CPU work.

**Deletion:** replace bespoke/duplicated structural composition and count-then-collect work
where the selected native mechanisms cover it; remove old callers/tests with migration, keeping
distinct useful answer semantics. M03 removes replaced public query routes/guidance with their
working replacements. No new parser service, query VM or general DSL follows from this package.

**Implemented:** compatible ast-grep core/config0.51.0, tree-sitter0.27.0 and Rust grammar0.24.2
supply the producer. One request-local compiled-selector owner shares AST, Tree-sitter and
lexical matchers across workers; each file keeps one exact acquisition and one compatible
native tree across structural leaves. Capture environments remain per match/branch. Native
rule relations replace manual interpretation, including one prepared RuleCore/template in the
remaining explicit syntax-preview producer. The count-then-reparse path, per-file matcher
compilation and manual preview relation loops were deleted. Extraction retains every capture
range and explicitly rejects unsupported predicates/match-limit failure. Recovery keeps positive
witnesses while preventing exhaustive negatives. Quota stopping remains separate from provider
unknown/error, and cancellation retains actual CPU completion ownership.

**Tested — selected producers and consumers:**
`cargo t -p pse-host --lib -E 'test(source_query::) | test(semantic::tests::compact_native_sections)'`
passes 10/10 against zero failures, including all nine source-query controls. Native cases cover
branch-local captures, repeated equality, relation/field/sibling rules, suppression comments,
Tree-sitter anchors/predicates/quantified capture groups, recovery and cancellation. The
quantified Rust-argument case explicitly groups each anonymous comma with its following
identifier and checks all three original captures; repeating identifiers alone is not a
comma-separated sequence in the selected grammar. Recovery still preserves usable positive
witnesses without establishing exhaustive absence.

The coordinator's selected actual public investigation run
`cargo t -p pse-host --lib --test investigations -E 'test(investigation::) | binary(investigations)'`
passes 84/84 against zero failures, including source/template/preview and partial/stale output
controls. Exact command/scope-end accounting belongs to P02; these selected passes do not
establish the remaining full-series hygiene or unexercised scientific behavior. J06's targeted
implementation acceptance is complete.

<a id="j07"></a>
### J07 — Hydrate selected results into checked previews

**Consumes:** working K02 result references/locators and J06 structural candidates, plus existing
J02 shared proposal validation. Selected semantic transformations consume G07's actual native
producers; native HIR/SSR extensions add G12 only if selected. L02 lexical observations can
participate after exact source/range hydration, not by claiming their scan was a full snapshot.

Allow `propose` to consume selected results directly for exact-byte replacements, capture-aware
syntax rewrites or native rename/assists/SSR/file-operation previews. Resolve lightweight result
premises into J01/J02 exact source identity and checked ranges; refuse stale identity or explicitly
ask the existing consumer for a fresh observation according to K02's declared change policy.
Never silently reinterpret a stale selection at the same path/offset. Preserve source kind,
revision/configuration, unusual path bytes, semantic applicability and proposal provenance.
Direct known-source inspection and existing explicit proposal inputs remain useful where their
behavior is distinct; search is not a mandatory preliminary step.

Use `ra_ap_syntax` edit facilities for lossless authored regions, native ast-grep replacements for
capture-preserving patterns, and `syn`/`quote`/`prettyplease` for selected newly constructed
fragments. Do not whole-file regenerate authored code and lose comments/trivia. Maintain all
candidate matches separately from selected nonoverlapping edits; show overlaps, missing captures,
unresolved snippets, imports and unsupported file effects. Native SSR searches cannot be
fabricated from identity rewrite output; G07's existing transformation preview remains its own
supported operation. Proposal serialization follows K02's compact source/context references,
retaining checked original inputs where validation requires them rather than repeating whole
files in every match.

**Public consumer and acceptance:** follow a small search page through declaration/capture
inspection into a comment-preserving multi-file preview, with no source or index write. Include
a staged or revision source, non-ASCII coordinates, changed working source, expired result,
out-of-range UTF-8 boundary, nested overlaps, undefined capture and unsupported file effect.
Identically spelled syntax candidates cannot pass semantic rename applicability. A native
semantic preview preserves its generation and refuses stale/missing preparation. Compare
untouched bytes independently and inspect actual CLI output, not only a schema snapshot.

**Deletion and limits:** move replaced syntax/semantic preview wrappers and parser branches,
their callers, guidance and tests with the common consumer through M03. Keep one shared proposal
validator. Preview success is not apply, compile/test success, equivalence or Git publication;
G08's no-application outcome remains intact. Ordinary explicitly authorized edits and real
checks can assess a preview's usefulness separately during integrated acceptance. The
coordinator owns scope-end journeys/static checks; this document establishes no such result.

**Implemented:** lexical selections validate original checkout, observed byte-span digest and
declared freshness before exact hydration; syntax selections additionally validate their full
original source hash. Capture-template previews re-match the exact range and use the shared
proposal validator/protected-source owner. Working/index/revision sources keep their acquisition
identity. Semantic follow-ups preserve session, generation, source and lazy-candidate guards
through the original native producer; unsupported snapshot transformations do not fall back to
spelling-based edits. No preview writes source/index.

**Tested — selected preview consumers:** the producer and public commands above exercise
capture-aware previews and stale/partial-source guards. The actual
`public_text_discovery_inspection_and_preview_keep_original_bytes` consumer follows discovery
through original-byte inspection and checked preview. The selected native namesake journey
also retains its original syntax witness and guarded native identity through inspection and
proposal/test-selection follow-ups. These establish the exercised preview routes and their
no-write boundary; full P02 qualification, unrelated transformations and source application are
not inferred from them. J07's targeted implementation acceptance is complete.
