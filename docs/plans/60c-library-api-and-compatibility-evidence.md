---
title: "Plan60c: Library API and compatibility evidence"
status: complete
doc_retention: while-dependent
doc_retention_reason: "Active Plan60 consumers and E03/Q01 qualification depend on the artifact contracts, matcher semantics and targeted acceptance."
date: 2026-11-01
doc_role: plan
doc_owner: docs/plans/60-engineering-code-intelligence.md
doc_topics: [current-work, design, libraries, diagnostics]
review_sources: [docs/functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md, docs/plan_improvement_reviews/plan60-engineering-intelligence-2026-11-01.md, docs/functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md]
adrs: [ADR-0235]
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
| C01 | complete | Acquired API graph/bridge owner | Artifact-qualified adjacency/defaults/bounds/impls/links with shared inquiry lookups and actual local-use/destination journeys |
| C02 | complete | API structural matcher owner | Rich structural inputs, binder scopes, constants/constraints and one validator/matcher across public request routes |

## Verification and handoff

**Tested:** the Outcome records API fixture/matcher/bridge tests and actual acquired-library
questions through public search/inspection. Acquisition stays explicit; missing artifacts do not launch
builds. E01 can deliver current-feature guidance immediately; E03 later exercises these new
capabilities and their B03/D01 connections. Q01 owns scope-end qualification. The existing
exact workspace `quote` dependency is inherited; no version bump was selected. Research uses
Context7 discovery followed by
primary/pinned source when implementation reaches consequential details. The review identifies
overview/pin drift; advertised releases are not evidence of checkout availability. The reviewed
schema is rustdoc-types0.61.0, while the syn overview targets3.0.9 and the inspected checkout
pins3.0.5. Verify exact interfaces before new use; no upgrade is selected. Concrete source leads
are rustdoc §§4,8/addendum §5 and rustdoc-types §§5–10/addenda B/E/F/G, plus syn's contextual
parsing/generic sections. These mechanisms support AP-04/AP-05 and DP-07/DP-15 by preserving
artifact and predicate meanings. Extra setup is justified only for the selected question.

C01/C02 own targeted matcher, fixture/bridge and public-consumer acceptance during
implementation; E03 assembles the wider library-fit journey and Q01 owns the later scope-end
campaign. Targeted capability acceptance does not establish those later series boundaries.

## Outcome

### What was built

**Implemented (2026-11-01, `11280f6e3`):** C01/C02 through the existing acquired-artifact,
definition/source bridge and request owners. Declared adjacency exposes containment, structured
bounds including associated-type bounds, implementation records, provided defaults, reexports
and compiler-resolved documentation links. Native and acquired graphs share the canonical
edge projection while keeping separate relation domains, guarded nodes and original witnesses.
Impl witnesses preserve negative/synthetic/blanket flags and declared trait/self types.
A provided default can be followed even when absent from `Impl.items`.

Each inquiry shares acquired definition lookup, artifact/context-qualified parent indexes and
contract projections. Missing retained definitions remain partial; inspection never acquires
them implicitly. Numeric item IDs are not identities across artifacts or contexts. Declared
membership does not invent Deref inheritance, destination importability or use-site applicability.
Existing qualified native consumers answer those separate questions.

One typed structural program handles flags, typed JSON, saved queries and ordinary API inspection.
It preserves callable/function-pointer forms, arrays, lifetime/type/const argument kinds,
higher-ranked scopes, GAT arguments, associated equality versus bounds and declared where
predicates. Recursive validation runs before providers. Source-addressed parse diagnostics
retain query coordinates. Repeated wildcards remain independent; opaque constants are not
evaluated and unsupported comparisons remain unknown. Textual unbraced identifiers retain
their type interpretation; const identifiers use braces or structured input. Legacy spelling
filters and type-only generic-argument filtering were removed with migrated callers/tests.
The existing exact workspace `quote` dependency was inherited without an upgrade.

**Tested:** the combined [60b Outcome](60b-native-relationships-and-lifetime-evidence.md#outcome)
command passed 38 tests against the zero failure baseline on local Linux, including the rich
matcher matrix, indexed contract projection, qualified external aliases, actual producer span
coordinates and shared request/schema regressions. The final native-hir all-target check had
zero errors and warnings; full generation published 10 configured targets. Additional public
checks used the native correctness route:

```bash
scripts/pse test pse-host:composed_investigations -E 'test(acquired_api_rich_shapes_) | test(actual_acquired_api_constraints_join_native_use_) | test(bounded_multi_seed_graph_)'
scripts/pse test pse-host:composed_investigations -E 'test(acquired_api_rich_shapes_) | test(actual_acquired_api_constraints_join_native_use_)'
```

The initial selection passed its graph test and failed both API fixtures during setup. After
repairing caller ownership, the final selection passed all 3 selected tests (two public journeys
and their matching guarded entry), with zero failures. The acquired fixture independently
expects callback/HRTB flag/JSON/saved-query equivalence and its negative counterpart, declared
where constraints, provided defaults absent from impl items, negative/synthetic/blanket impl
metadata, associated bounds and resolved versus prose-only links. It follows actual default
and Deref-selected calls through the existing native use bridge and refuses stale context.
The existing destination journey verifies native use and importability; contextual candidate
tests separately retain adjustments, required imports and obligation limits. These are targeted
capability checks, not a feature-adoption measurement or whole-series qualification.

### Mistakes corrected

Independent review found repeated whole-artifact parent scans per impl witness and omission
of associated-type bounds. A shared parent index removes the scans while retaining ambiguous
membership semantics; the acquired fixture asserts the previously omitted bound edge. A
generalized shared edge type removed duplicate canonical projection declarations without
collapsing identity domains.

The first public selection failed 2 of 3 tests against the zero target because the API journeys
attempted independent detachment inside an enclosing managed test workload. Both now use the
existing authenticated independent-caller/entry-gate harness. Production refusal remains in
force. The final rerun passed; failed setup runs receive no acceptance credit.

### Deliberate boundaries and handoff

**Implemented:** blueprint §21.7 owns enduring behavior. The actual pinned rustdoc producer
coordinate test was reused and its misleading comment corrected; no repeated feasibility
study or alternate source-link reconstruction was added. Patterns describe shape, declared
adjacency describes artifacts, and B03/native calls retain contextual predicate authority.
No trait solver, global index, synchronization service, implicit acquisition or new dependency
version was introduced. This plan remains retained while the parent series depends on it.

**Proposed / not_run:** E03's wider engineering-process journeys and Q01's final integrated/static
campaign remain separate. Formal ADR acceptance is unchanged, and the user-owned library
documentation remains outside this implementation's commit scope.
