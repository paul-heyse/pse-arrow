---
title: "Plan62b: Source investigations and transformations"
status: complete
doc_retention: while-dependent
doc_retention_reason: "Companion to retained Plan62; exact origin and guarded transformation evidence supports subsequent consumers."
date: 2026-11-01
doc_role: plan
doc_owner: docs/plans/62-code-intelligence-capability-expansion.md
doc_topics: [current-work, design, libraries, diagnostics]
review_sources: [docs/functional_reviews/code-intelligence-capability-expansion-2026-11-01.md]
adrs: [ADR-0233, ADR-0235]
---

# Source investigations and transformations

## Target and current foundations

**Proposed:** improve O06/O07 investigations and previews, with O13's precise source/field origins.
The [coordinator](62-code-intelligence-capability-expansion.md) owns dispositions and dependencies.
Baseline `4b2f8df34` already supports full ast-grep rules, Tree-sitter selectors, captures,
relational predicates, immutable source guards and overlapping proposal candidates. The new value
is interpreted source evidence, useful match explanation, native Rust edits and consolidation.

Existing owners are `source_read.rs`, `source_query.rs`, `source_query/native.rs` and
`source_proposal.rs`; generated association recognition is in xtask's code-intelligence generator,
and authored-origin inspection in `schema/compare.rs`. Preserve these authorities. No new parser
service, edit-application operation or workspace transaction manager is selected.

## Native mechanisms and fidelity

Use [RA syntax](../libraries/rust_analyzers/ra_ap_syntax.md),
[ast-grep](../libraries/rust_analyzers/ast-grep-core.md),
[Tree-sitter Rust](../libraries/rust_analyzers/tree-sitter-rust.md) and
[Syn/quote/prettyplease](../libraries/rust_analyzers/syn_quote_prettyplease.md) according to
the facts required. The review's exact-source corrections are binding inputs to API selection:
standalone RA syntax .360 differs from native .358; Syn is3.0.5 and Tree-sitter .27.0/grammar .24.2.
No whole-file Syn round trip should replace lossless malformed-source/comment support.

RA typed attribute ownership and string value APIs supply interpreted facts. `map_range_up`
shifts quote-free source spelling; `escaped_char_ranges` plus token start maps decoded characters
to actual escape spans. Syn `LitStr::parse_with` interprets embedded Rust but gives enclosing-literal
spans. `cfg_attr` inventory retains predicates; it does not evaluate a build's effective attributes.

Native ast-grep `RuleCore`, captures, explicit `stopBy` and Tree-sitter field/anchor/repetition
queries compose relationships. A repeated capture's spelling is not binding identity; sibling
order is not dominance or resource pairing. Tree-sitter outer attributes can be declaration
siblings; RA ownership is preferable for that Rust-specific question.

For compound edits use RA `SyntaxEditor`, `SyntaxFactory`, `ast::make`, where-clause and removal
helpers with precedence/separator handling. ast-grep .51's `before/after/append/prepend` are
`todo!()` and cannot be selected. Prettyplease remains appropriate for owned generated Rust;
handwritten edits preserve unrelated comments/layout. AST mutation does not itself provide a
minimal patch, semantic validity or a multi-file transaction.

## Packages

### B01 — Interpreted source and explained structural relations

Consumes T01 and T02 source-fact slices only where public contracts need them; pure source reading
remains local and independent of native preparation. Extend lossless outlines with typed literal,
documentation and attribute observations carrying written representation, interpreted value and
exact source mapping. Preserve parse issues and unavailable interpretations.

Compose match, receiver/arguments, enclosing declaration and authored attributes into one
relationship answer. Expose selected native pattern/capture interpretation without a new query
language. Same-owner constraints exclude nested declarations even when spellings match.

Acceptance: select the logical newline in an escaped string and highlight the complete escape;
retain raw-string delimiters and C-string bytes; associate inner/outer docs once; inventory nested
`cfg_attr` without evaluating it; malformed surrounding Rust retains available facts and issues.
A relational result preserves its shared source identity and actual containment relation.

### B02 — One demand-aware selection and projection substrate

Consumes T01; coordinate with B01 consumers, but no prerequisite on all B01 enrichments. Reconcile
the duplicate traversal/capture/result construction in `source_query.rs` and `source_query/native.rs`.
Use compiled native selectors, compatible worker-local parsing and one result projection. Make
witness/first-N/full-inventory/exact-total demand explicit; diagnostics and proposal production
consume the selected result rather than rescan unconditionally.

Preserve exact totals when requested and native early stopping otherwise. Use potential-kind
filtering where applicable. Distinguish Tree-sitter intersection/containment, parser/query
cancellation and match-capacity exhaustion. Retain nested/overlapping candidates; ast-grep
`replace_all` skips nested matches and is not a drop-in replacement. Preserve matcher-consumed
versus full-node edit extents. Thread-confined syntax handles produce owned/plain result records.

Move syntax search, template previews and composed investigation consumers together; delete
replaced walkers, projections, callers and tests after targeted replacement tests pass. Different
parsers can remain where fidelity requires them; consolidation does not mean one parser universe.

Acceptance: witness traversal stops upstream; exact totals are accurate; cancellation during
parse and selection remains incomplete; a containing-range query rejects crossing captures;
nested candidates remain selectable and exhausted capacity cannot report complete absence.

### B03 — Grammar-aware and native semantic previews

Consumes T01, the proposal slice of T02 and B02's selected-candidate interface. Grammar-only
operations need no native session. Native SSR additionally consumes a prepared supported world
and A02's context-qualified facts where explanation uses them; extended A01 is required only for
generated/macro-dependent inputs.

Implement supported compound preview operations: add a where-bound, remove a grouped import or
bound with valid separators, insert an associated item/field and construct a precedence-correct
expression. Use editable-tree handles and annotations/mappings; do not treat relative-position
lookup as durable identity. Appending a predicate does not deduplicate or prove it necessary.
Convert edited trees to scoped candidates under the existing source/proposal guard owner.

Adapt native .358 `Analysis::structural_search_replace` returning `SourceChange` to that same
proposal owner. It can resolve aliases/paths, supported UFCS and receiver adjustments. Live LSP
already has its distinct SSR route; do not silently mix worlds. The native public match fields
do not expose an addressed capture inventory; keep that unsupported rather than manufacture
captures by reparsing rewritten text. `parse_only` is not a grammar-only validation promise.

Acceptance: existing/new where clauses preserve unrelated comments; grouped-use removal reparses;
stale sources and overlaps refuse selection. Aliased/qualified references to one definition match
semantic SSR while an unrelated same-spelled item does not. Range restriction, cancellation and
unknown paths preserve original errors. Previews never apply edits. Compilation is separate evidence.

### B04 — Typed generator recognition and exact field origins

Consumes T01 and the existing authenticated descriptor/association foundation; no dependency on
new semantic preparation. Replace token-display comparisons in the generator with typed Syn
`Expr::MethodCall` receiver/method/args checks and numeric literal/pattern recognition using
`LitInt::base10_parse` with explicit accepted shapes/suffixes. Preserve conservative treatment
of multiple-member oneof routes and ambiguous/nonliteral mappings.

Use existing manifest field associations for a selected generated field/accessor, extending the
current enclosing struct/impl-only filter in `schema/compare.rs`. Verify original source/output,
descriptor and producer identities before navigating to an authored proto field. A field number
or matching name alone is insufficient. Explore producer annotations only where the actual plugin
emits a reliable mapping; do not assume `GeneratedCodeInfo` exists.

Update the generator and run `just codegen`; never hand-edit generated outputs. Migrate origin
inspection and remove token-string recognition/tests in the same integrated change. Acceptance:
spacing and raw identifiers do not change mapping; accepted numeric forms agree; another receiver,
nonliteral pattern, ambiguous oneof and stale manifest yield explicit unresolved/stale outcomes.
Precisely mapped field/accessor selection reaches the exact authored field.

## Package state

| Package | Status | Dependencies | Completion boundary |
| --- | --- | --- | --- |
| B01 | complete | T01; T02 source slice where needed | Interpreted literal/doc/attribute facts, exact escape intervals and explained relationships exercised |
| B02 | complete | T01; existing native selector contracts | One demand-aware selector and migrated consumers; exact/first/inventory and cancellation controls exercised |
| B03 | complete | B02; T02 proposal slice; selected native world for SSR | Selected-node grammar previews and original-line-scoped native SSR with guards and CRLF/Unicode controls |
| B04 | complete | T01; existing descriptor/origin contracts | Typed Syn recognition and descriptor-qualified native proto byte ranges; actual public field-to-authored-source expansion passed |

## Verification and handoff

The annotation assessment below predates implementation. Its open/next-step statements are
historical; the [Outcome](#outcome) owns the delivered boundary.

### B04 annotation assessment — 2026-11-01

**Interface-checked:** selected buffa/codegen/packaging0.9.2 and connectrpc-codegen0.9.0 response
constructors supply names/content but no generated annotations. The locally corrected validator
0.7.2 explicitly assigns `generated_code_info: None`. Descriptor SourceCodeInfo supplies authored
locations, not generated Rust byte correspondence. Inspected sources are the installed plugins'
`main.rs`, Buffa `GeneratedFile`, Connect `codegen.rs` and local validator `emit/mod.rs` constructors.

**Tested:** a direct-plugin scalar-message/unary-service probe against all four selected executables
returned four successful responses, nine files and zero File.generated_code_info fields. The
Connect request initially failed for missing extern_path; adding `extern_path=.=crate::probe`
passed. This establishes bounded emission behavior, not generated-code compilation or origin
navigation. No production generator/output was edited.

Do not select annotation acquisition for these producers. Continue B04's typed Syn recognition
and authenticated field/accessor manifest associations. Revisit only when a selected producer
emits reliable descriptor paths/generated-byte spans and their mapping survives postprocessing.
The investigation is settled; B04's actual implementation and acceptance remain open.

**Proposed:** source fixtures expose punctuation, nested ownership, escapes, malformed syntax,
Unicode boundaries and overlaps. Native SSR needs actual semantic fixtures in addition to proposal
unit tests. Generator tests include an independent known descriptor/field expectation; comparing
two outputs of the same recognizer is insufficient. Q01 owns assembled source-to-proposal and
generated-field-to-proto journeys plus scope-end static checks. No new feature is implemented,
tested or measured by publishing this plan. Persistent parsing, whole-grammar typed wrappers,
embedded-language injection and automatic edit application are not completion prerequisites.

## Outcome

**Implemented:** interpreted source facts and structural relations feed shared demand-aware
selection. Selected-node templates avoid scanning unrelated siblings; native SSR passes the
selected original interval to the analyzer and returns the existing guarded proposal. Native
edits remain previews and do not mutate sources.

B04 uses typed Syn expressions/patterns for generated accessor/tag recognition and
tree-sitter-proto0.6.0 for authored byte locations. Buf remains the sole semantic compiler and
descriptor graph. The generator joins descriptor-qualified message path, actual field name and
tag to native syntax, records its producer/profile and input hashes, and publishes exact ranges
with normal generation. Runtime origin expansion authenticates those recorded ranges; it does
not reconstruct columns. Ambiguity, recovery, unknown profiles and unsupported proto2 groups
remain unavailable. Generated annotations are still unselected for the inspected producers.

**Tested, local Linux, zero-failure target:** source controls passed 31/31
(`01a1961f-fcf8-721d-a41b-319b3aca8455`), final affected schema/dependency/source-command controls
32/32 (`01a196d2-4aff-7189-8d92-5ce40a3b80eb`), generator association controls 6/6
(`01a196c0-2009-73b5-a847-0dc19ba02e6e`), and the final 14-case native population includes
selected-line SSR with Unicode/CRLF and source-preservation assertions. Actual public origin
`01a196c7-1c0b-778a-9229-7b260d671be7` expanded `NativeTestCandidate.snapshot_update` to
`NativeSnapshotUpdate snapshot_update = 13;` through guarded original-source expansion
`01a196c7-242e-7300-886e-8f51344ebc84`. The coordinator [Outcome](62-code-intelligence-capability-expansion.md#outcome)
owns commands, generation/static checks and initial failures with corrected reruns.

A mistake corrected was relying on descriptor column conventions for UTF-8 byte correspondence;
native authored syntax eliminates version-dependent display-width reconstruction. A separate
raw-identifier join now treats logical outline names as logical names, parsing only the actual
Syn type where needed. Deliberately unselected are a second protobuf semantic compiler, generator
annotation acquisition without a producer, whole-grammar wrappers and automatic edit application.
Enduring contracts are in blueprint §21.7. No authorized delivery remains here.
