---
title: "Plan60c: Library API and compatibility evidence"
status: active
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/60-engineering-code-intelligence.md
doc_topics: [current-work, design, libraries, diagnostics]
review_sources: [docs/functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md]
adrs: []
---

# Library API and compatibility evidence

## Purpose and foundation assessment

**Proposed:** artifact-qualified API adjacency and richer type-pattern questions for
[O05/O11](60-engineering-code-intelligence.md#opportunity-dispositions). Current
`crates/pse-host/src/api/{query,bridge,comparison}.rs` already supply structural `TypePattern`,
original declaration/docs, facade/definition qualification, selected importability/use joins
and comparisons. E01 should teach those features before new code. This document extends the
same owners; no new library-assessment verb or generic capability score is needed.

Existing patterns cover paths/arguments, references, tuples, slices, pointers, qualified
associated types and `_`. A wildcard is not an inferred unification variable. A shape match,
documented condition, native destination importability and compiled applicability are different
evidence. The source review's [O05/O11](../functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md#O05)
and [rustdoc-types](../libraries/rust_analyzers/rustdoc-types.md) describe the typed mechanisms;
their original artifact/context limits must survive every new join.

## Target design and library use

Navigate associated methods/types, supertraits, structured bounds, relevant impl records,
reexports and documentation links through acquired rustdoc's `Item`, `index`, `paths`,
`external_crates`, `Type`, `Generics`, `WherePredicate` and `Item.links`. IDs are local to one
JSON artifact. A cross-crate path summary is not a full definition, and a reexport is not
identical to the defining artifact. Preserve original facade/definition evidence when joining
local usages or asking native destination questions.

Compiler-resolved intra-doc links, prose references, implementation records and type dependencies
are distinct edges. Follow `Item.links` as recorded, not a second Markdown guess at its resolved
meaning. Missing definitions, omitted blanket implementations and generic applicability remain
unknown. Query only selected items/adjacency within bounded demand; share artifact loading and
local lookup indexes through the existing owner, without introducing index synchronization.

Extend `TypePattern` using rustdoc `GenericArgs`, `AssocItemConstraint`, function-pointer/array
types and scoped bounds, with syn parsing the requested structure. Retain binder scopes and
associated equality versus bounds. This supplies query shape, not trait solving. A rare shape
may still be best answered by original declaration inspection and an explicit compiler probe.
Actual contextual compatibility belongs to B03, not a second implementation here.

## Work packages

<a id="c01"></a>
### C01 — API, documentation and local-use adjacency

Consumes A01 typed evidence and existing acquired API/use/importability machinery. Deliver a
library-fit question: start from a required signature or candidate API, inspect associated
items/bounds/documented conditions, then follow actual local integrations and retained wrapper
responsibilities. Reuse current commands and A03 expansion; new API relationships must integrate
through shared evidence rather than concatenate full artifacts.

Before new source-link navigation, resolve the bounded producer-coordinate question noted in
O11: inspect an actual acquired artifact's `Span`/local `Location` conversion at an ASCII and
multibyte source location. Review comments disagree about column bases; no producer defect is
established. Make the actual producer convention and conversion explicit, with boundary tests,
without treating comment text as authority. Other artifact-only navigation need not wait for
this source-link slice.

Acceptance covers a reexport with separate definition, two artifacts reusing a numeric item
ID, a compiler-resolved doc link, prose-only reference, absent defining artifact and a locally
used API with inconclusive generic applicability. Async/unsafe/const classification must not
be presented as proof of operational fit. Use known original fixture declarations/links to
challenge adapters, not their own rendered output as expected truth. Delete replaced duplicate
link/coordinate reconstruction and migrate callers when qualified navigation replaces it.

<a id="c02"></a>
### C02 — Rich structural API patterns

Consumes A01 query validation/output and the existing matcher; does not wait for all C01 graph
navigation. Add callback/function-pointer inputs/outputs, higher-ranked bounds, arrays/const
arguments, scoped generic arguments and associated equality versus bounds through the same
query representation. Do not replace native structure with token substring matching or add a
second parser/validator. Preserve negative/synthetic impl meaning in relevant records.

Acceptance distinguishes `Iterator<Item = T>` from `Iterator<Item: Clone>`, distinct binder
scopes, different const arguments, a callback shape from an ordinary named type and repeated
wildcards from variable binding. An unsupported pattern reports its actual limitation rather
than matching broadly. Include a revealing negative counterpart to every new pattern family.
Retain known existing TypePattern forms and original declaration drill-through. Document when
a shape result should lead to B03 or an explicit probe rather than imply applicability.
Remove replaced special-case string checks and migrate query consumers/tests in the same
coherent change; do not preserve a compatibility matcher.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| C01 | planned | Acquired API graph/bridge owner | Qualified adjacency, source-coordinate decision and useful local-use consumer |
| C02 | planned | API structural matcher owner | New shape/binder/constraint cases in the original matcher with public query tests |

## Verification and handoff

**Proposed:** targeted API fixture/matcher/bridge tests plus an actual acquired-library question
through public search/inspection. Acquisition stays explicit; missing artifacts do not launch
builds. E01 can deliver current-feature guidance immediately; E03 later exercises these new
capabilities and their B03/D01 connections. Q01 owns scope-end qualification. No new dependency
or version bump is selected: verify exact pinned APIs through Context7 discovery followed by
primary/pinned source when implementation reaches consequential details. The review identifies
overview/pin drift; advertised releases are not evidence of checkout availability.
