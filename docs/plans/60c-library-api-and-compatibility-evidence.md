---
title: "Plan60c: Library API and compatibility evidence"
status: active
date: 2026-11-01
doc_role: plan
doc_owner: docs/plans/60-engineering-code-intelligence.md
doc_topics: [current-work, design, libraries, diagnostics]
review_sources: [docs/functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md, docs/plan_improvement_reviews/plan60-engineering-intelligence-2026-11-01.md, docs/functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md]
adrs: []
---

# Library API and compatibility evidence

## Purpose and foundation assessment

**Proposed:** artifact-qualified API adjacency and richer type-pattern questions for original
[O05/O11](60-engineering-code-intelligence.md#opportunity-dispositions), incorporating
[improvement I05](../plan_improvement_reviews/plan60-engineering-intelligence-2026-11-01.md#I05)
and [functional O02](../functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md#O02).
The coordinator owns their dispositions; this document owns C01/C02 execution state.

Starting baseline is `42bede27a`, including Plan62's completed selected API/native acceptance
in its [Outcome](62-code-intelligence-capability-expansion.md#outcome).
`crates/pse-host/src/api/{query,bridge,comparison}.rs`
already supply structural `TypePattern`, original declaration/docs, qualified facade/definition
and local-use joins, and comparisons. Plan61 supplies typed evidence and generated contracts.
Plan62 supplies rich schema61 contracts, native doctest original/transformed/wrapper evidence,
and guarded API/native correspondence. Those are consumed foundations, not packages to rebuild.
Their completed qualification remains with Plan61/62 under its recorded conditions. Plan60
adds adjacency, richer matcher inputs and new library-fit journeys, not another rich-contract,
example-transformer or actual-call substitution producer.

Existing input patterns cover paths/type arguments, references, tuples, slices, pointers,
qualified associated types and `_`; `TypePattern::from_syn` still rejects lifetime/const/binding
generic arguments, callable-trait forms and unsupported associated arguments. Rich serialized
output contracts do not implement these input/matcher capabilities. A wildcard is not an
inferred unification variable. A shape match,
documented condition, native destination importability and compiled applicability are different
evidence. The source review's [O05/O11](../functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md#O05)
and [rustdoc-types](../libraries/rust_analyzers/rustdoc-types.md) describe the typed mechanisms;
their original artifact/context limits must survive every new join.

## Target design and library use

Navigate associated methods/types, supertraits, structured bounds, relevant impl records,
reexports and documentation links through acquired rustdoc's `Item`, `index`, `paths`,
`external_crates`, `Type`, `Generics`, `WherePredicate` and `Item.links`. In schema61,
`Trait::{items,bounds,implementations}` and
`Impl::{items,provided_trait_methods,trait_,for_,generics,is_negative,is_synthetic,blanket_impl}`
distinguish declarations, provided defaults and explicit/negative/synthetic/blanket impl evidence.
A provided default need not appear in `Impl.items`. Include selected primitive impl information
where the addressed question needs it. IDs are local to one JSON artifact; an external path
summary is not a full definition, and a reexport is not the defining artifact.

Keep declared adjacency distinct from an effective callable surface. Imports, `Deref`-mediated
methods, receiver adjustments and obligation results belong to the existing selected native
context. Preserve original facade/definition evidence and the existing `SourceBridge` guards
when joining local usages or asking destination questions. No impl record establishes use-site
applicability. Missing defining artifacts, stripped fields or unresolved links remain explicit.

Compiler-resolved intra-doc links, prose references, implementation records and type dependencies
are distinct edges. Follow `Item.links` as recorded, not a second Markdown guess at its resolved
meaning. Missing definitions, omitted blanket implementations and generic applicability remain
unknown. Query only selected items/adjacency within bounded demand; share artifact loading and
local lookup indexes through the existing owner, without introducing index synchronization.

Extend `TypePattern` using rustdoc `GenericArgs`, `AssocItemConstraint`, function-pointer/array
types, `WherePredicate`, `GenericBound` and `Term`, with syn parsing the requested structure.
Preserve lifetime/type/const argument kinds, associated-item arguments and HRTB binder scopes,
including GAT constraints and associated equality versus bounds. This supplies query shape,
not trait solving. A rare shape
may still be best answered by original declaration inspection and an explicit compiler probe.
Actual contextual compatibility belongs to B03, not a second implementation here.

## Work packages

### Authorized execution refinement — 2026-11-01

Execute C01/C02 from `926655517`. Artifact-qualified adjacency remains offline and separate
from native traversal/applicability. Reuse the existing pinned-producer coordinate test and
correct its misleading location comment rather than repeat feasibility work. Replace legacy
spelling input/output filters with the same structural matcher used by search. Generic
argument kinds and HRTB scopes remain explicit; textual unbraced identifiers retain type
meaning, while const parameters use braces or typed JSON. Opaque expressions are never
evaluated. One query-local catalog shares artifact lookups; no global index or new service.

<a id="c01"></a>
### C01 — API, documentation and local-use adjacency

Consumes the delivered Plan61 evidence foundation and existing acquired API/use/importability
machinery. The new addressed relationship export consumes A01's particular edge projection;
ordinary artifact inspection does not wait for A02 or every native B01 relation. Deliver a
library-fit question: select a signature/candidate, inspect associated/default methods, bounds
and documented conditions, follow an exact local integration, then request contextual
availability only when the question needs it. Reuse current commands and A03 expansion.
Extend the shared evidence contract for new relationships rather than concatenate artifacts.
Within one inquiry, share the acquired artifact and useful item/parent lookups; do not reload
or rescan the complete artifact for every adjacent edge. A retained global graph is unselected.

Before new source-link navigation, resolve the bounded producer-coordinate question noted in
O11: inspect an actual acquired artifact's `Span`/local `Location` conversion at an ASCII and
multibyte source location. Review comments disagree about column bases; no producer defect is
established. Make the actual producer convention and conversion explicit, with boundary tests,
without treating comment text as authority. Other artifact-only navigation need not wait for
this source-link slice. Plan62's completed native proto-byte origin work answers a different
coordinate question: reuse it for generated-contract navigation, but do not treat it as proof
of rustdoc Span conventions or schedule descriptor-column conversion again.

Acceptance covers a reexport with separate definition, two artifacts reusing a numeric item
ID, a compiler-resolved versus prose-only link, absent defining artifact, a provided default
absent from `Impl.items`, negative/synthetic/blanket records, a `Deref`-derived method and an
extension trait unavailable in the destination scope. A locally used API can still have
inconclusive generic applicability. Async/unsafe/const classification must not
be presented as proof of operational fit. Use known original fixture declarations/links to
challenge adapters, not their own rendered output as expected truth. Delete replaced duplicate
link/coordinate reconstruction and migrate callers when qualified navigation replaces it.

<a id="c02"></a>
### C02 — Rich structural API patterns

Consumes the existing validated request, typed API contracts and matcher; it does not wait for
new A01 structural edges, C01 adjacency or B01 native extraction. Add callback/function-pointer
inputs/outputs, parenthesized callable-trait shapes, arrays/const arguments, higher-ranked
binders, lifetime/type/const generic arguments, associated-item arguments and equality versus
bounds through the same query representation. Use syn `Type`, `GenericArgument`, path arguments
and bound-lifetime syntax to match the schema61 tree. Retain existing forms and one validator
across flags, typed JSON and saved queries; no token-substring or compatibility matcher.

Acceptance distinguishes `Iterator<Item = T>` from `Iterator<Item: Clone>`, same-spelled
lifetimes under distinct binders, GAT argument scopes, `[T; 3]` from `[T; 4]`, a callback shape
from an ordinary named type and repeated wildcards from variable binding. An unsupported pattern
reports its actual limitation rather
than matching broadly. Include a revealing negative counterpart to every new pattern family.
Retain known existing TypePattern forms and original declaration drill-through. Document when
a shape result should lead to B03 or an explicit probe rather than imply applicability.
Remove replaced special-case string checks and migrate query consumers/tests in the same
coherent change; do not preserve a compatibility matcher.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| C01 | planned | Acquired API graph/bridge owner | Bounded qualified adjacency/defaults, source-coordinate decision and useful local-use/contextual consumer |
| C02 | planned | API structural matcher owner | Rich input shapes/binders/constraints in the existing matcher and public request routes |

## Verification and handoff

**Proposed:** targeted API fixture/matcher/bridge tests plus an actual acquired-library question
through public search/inspection. Acquisition stays explicit; missing artifacts do not launch
builds. E01 can deliver current-feature guidance immediately; E03 later exercises these new
capabilities and their B03/D01 connections. Q01 owns scope-end qualification. No new dependency
or version bump is selected: verify exact pinned APIs through Context7 discovery followed by
primary/pinned source when implementation reaches consequential details. The review identifies
overview/pin drift; advertised releases are not evidence of checkout availability. The reviewed
schema is rustdoc-types0.61.0, while the syn overview targets3.0.9 and the inspected checkout
pins3.0.5. Verify exact interfaces before new use; no upgrade is selected. Concrete source leads
are rustdoc §§4,8/addendum §5 and rustdoc-types §§5–10/addenda B/E/F/G, plus syn's contextual
parsing/generic sections. These mechanisms support AP-04/AP-05 and DP-07/DP-15 by preserving
artifact and predicate meanings. Extra setup is justified only for the selected question.

No API package is newly Implemented or Tested by this revision. C01/C02 own targeted matcher,
fixture/bridge and public-consumer tests during implementation; E03 assembles the library-fit
journey and Q01 owns the later scope-end campaign.
