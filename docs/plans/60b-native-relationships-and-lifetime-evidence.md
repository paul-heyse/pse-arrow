---
title: "Plan60b: Native relationships and lifetime evidence"
status: active
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/60-engineering-code-intelligence.md
doc_topics: [current-work, design, libraries, diagnostics]
review_sources: [docs/functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md]
adrs: []
---

# Native relationships and lifetime evidence

## Purpose and foundation assessment

**Proposed:** expose source-qualified native structure, bounded lifetime observations and named
contextual predicates for [O01/O03/O11](60-engineering-code-intelligence.md#opportunity-dispositions).
B04 retains O10's conditional extraction decision. The existing immutable snapshot helper,
`crates/pse-host/src/semantic/{native,engine}.rs` and investigation adapters retain execution
and context ownership. Follow blueprint §21.7 and the review's exact-source qualifications.

The native HIR/IDE family is 0.0.358; standalone syntax 0.0.360 is intentionally separate.
The live provider already has supported call/predicate operations and typed snapshots already
return selected original/adjusted type facts. Extend through their actual capability boundaries,
not a claim of universal provider parity. Native handles are not arbitrary Send/Sync payloads;
one selected world/session retains them, exporting owned views through existing complete records.
No new service, universal semantic database or preparation side effect is needed.

## Target design and selected mechanisms

Use HIR objects and native relations instead of parsing rendered type strings. Review O01 and
[HIR §§3–7](../libraries/rust_analyzers/ra_ap_hir.md) identify:

- `Module::{declarations,impl_defs,scope}`, `HasContainer`/`HasSource` for containment and origin;
- `Function::{params_without_self,ret_type,async_ret_type}` and
  `Type::{walk,as_adt,fields,tuple_fields,generic_params}` for role-qualified type structure;
- `Trait::{direct_supertraits,all_supertraits,items,items_with_supertraits}` and
  `Impl::{all_in_crate,all_for_trait,self_ty,trait_,items}` for associated/implementation facts;
- `Semantics::{resolve_path,resolve_field,resolve_record_field_with_substitution}` for resolved
  uses, retaining substituted versus declared type meaning.

These are selected mechanism families, not a demand to extract everything for every request.
Return only demanded relations from the selected world. Field READ/WRITE is an original static
role, not complete mutation analysis through aliases, Deref or interior mutability. Pinned IDE
incoming-call hierarchy derives enclosing functions from reference usages; preserve that as
reference evidence unless an actual selected call is established. Explicit calls and implicit
language operations need their own witnessed classification. Neither implies runtime reachability.

Lifetime views combine lossless written structure with HIR observations. They can expose
captures, awaits, adjustments and local uses around a selected body. They do not infer physical
task settlement, transitive effects, complete ownership or observed destructor execution.
Native cancellation and actual CPU completion remain independently represented operations.

## Work packages

<a id="b01"></a>
### B01 — Native declaration, type and field relationships

Consumes A01's published semantics; working exports require implemented constructors/validation.
Implement a useful early slice: containing declaration, parameter/return/field type dependency
and resolved uses sufficient for A02's request/policy consumer. Then complete associated items,
supertraits, visibility and relevant impl relationships through selected queries. Use one native
context and request-local reuse, not a fresh analysis for each edge. Preserve macro/source
mapping ambiguity and unsupported portions. Syntax-only candidates remain available without
resolved-identity claims.

Test a declaration trace through a real public operation, same-spelled unrelated types/methods,
callback value versus selected invocation, field read/write versus interior mutation and a
substituted field type. Challenge call classification against exact pinned
[call hierarchy source](https://docs.rs/crate/ra_ap_ide/0.0.358/source/src/call_hierarchy.rs),
not the old display label. A02 can consume the early relation slice once its tests/public
adapter work; completion of B01 still requires the remaining selected relation families.
Migrate shared result consumers and delete replaced text-parsing/duplicate classification.
Do not delete complementary live-provider capabilities merely because snapshots lack parity.

<a id="b02"></a>
### B02 — Bounded capture, effect and lifetime evidence

Consumes B01 identity/context and relevant body facts. Add demand-selected
`Closure/Coroutine::captured_items`, `Closure::fn_trait`,
`Semantics::{expr_adjustments,locals_used,get_unsafe_ops}` and
`Type::{is_copy,drop_glue,future_output}` where the pinned API supports the question. Written
await/return/error paths come from AST, with native meaning joined only when established.
Show a body slice, actual observations and unresolved boundaries through A03 inspection.

Acceptance follows cancellation signal, CPU join owner, retained future/guard and sibling
request suppression/reset. A cancel call cannot imply finished CPU work; `drop_glue` cannot
imply a destructor ran; a shared reference cannot imply immutable state. Include overloaded
adjustment and unresolved macro/call boundaries. Tests should assert those evidence limits as
well as positive facts. Remove duplicated heuristic lifecycle summaries replaced by native
facts; keep original diagnostics/compiler evidence for questions these facts cannot settle.

Conditional compiler extension: only an actual decision requiring exact loans, move-path
initialization or liveness at a MIR location can trigger new extraction. First use retained
diagnostics/MIR and explicit probes. If insufficient, inspect exact pinned
`get_bodies_with_borrowck_facts`, native dataflow cursors and query timing before MIR is stolen:
[compiler consumer source](https://github.com/rust-lang/rust/blob/c1070d69382b8d2f2eb65119c738a77d9e324c9e/compiler/rustc_borrowck/src/consumers.rs).
Do not assume an after-analysis callback or newer overview API suffices. A selected component
requires its decision route, ownership and qualification; no driver/component installation or
prototype is a prerequisite to B02 completion while this trigger is absent.

<a id="b03"></a>
### B03 — Explicit contextual compatibility predicates

Consumes B01's qualified type identities and A01 refinement/unknown representation. Extend
the existing native predicate owner where a named question needs it. Distinguish exact type
equality, possible deep unification, coercion and trait applicability. HIR0.0.358 provides
`could_unify_with_deeply`, `could_coerce_to`, `impls_trait`, `has_any_impl` and supported
`try_rebase_into`/`try_rebase_into_owner` operations; their predicates are not interchangeable.
Use native origin/substitution checks before comparison. Never compare stringified types or
silently rebase arbitrary worlds.

Return the requested operation, original context/premises and result/unknown. Preserve
`evaluate_where_clause_at` Holds/NotProven/Invalid/Unsupported; textual
`get_failed_obligations` is explanation, not structured proof. `has_any_impl` existential success
does not establish a generic guarantee; `Impl::all_for_type` omits blanket impls and shallow
constructor matching/GAT normalization have limits. Test distinct same-rendered types,
compatible and refused origins, unresolved inference, coercion without equality and an
existential implementation that cannot discharge the requested generic obligation. C02 pattern
matches remain separate shape evidence. Migrate duplicate compatibility booleans only where
the qualified predicate replaces their actual callers.

<a id="b04"></a>
### B04 — Conditional native bulk extraction

**Deferred:** trigger only for repeated extraction or one genuinely global investigation whose
native batch route removes scalar coordination. Compare A02 request-local multi-seed and
resident Analysis reuse before selecting `StaticIndex::compute(analysis, vendored_libs_config,
include_hovers)`. Review O10 and exact
[static-index source](https://docs.rs/crate/ra_ap_ide/0.0.358/source/src/static_index.rs)
show file-once traversal and first/selected-crate choices: this is not all-world semantics.
An export must state its context and omitted interpretations. Monikers are not identity across
arbitrary revisions. Keep owner-confined handles and export compact owned records.

If triggered, assess work structure, memory/lifetime/cancellation and selected-context fidelity
against the actual consumer. Measure a claimed speedup with a baseline; no benchmark is needed
merely to recognize duplicated scalar work. Persisted indexing and remote transport have
separate [coordinator triggers](60-engineering-code-intelligence.md#conditional-investigations).
No unconditional feasibility task, prototype or adoption is required to close the series.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| B01 | planned | Native semantic relationship owner | Selected declaration/type/field/associated relations with honest calls/use classification |
| B02 | planned | Native body evidence owner | Public bounded capture/lifetime slice; compiler-private branch only on named need |
| B03 | planned | Native predicate owner | Named contextual predicates with origin checks and inconclusive cases |
| B04 | deferred | Native semantic owner | Observable bulk-work trigger and selected decision before any implementation |

## Verification and handoff

**Proposed:** targeted native-helper and host adapter tests exercise actual selected contexts,
positive facts and the counterexamples above. A live-provider pass does not qualify native
snapshot exports. Native preparation remains explicit, isolated from passive reads. E03 owns
actual engineering use; Q01 owns final integrated/static qualification. No new native,
compiler-private or performance claim follows from authoring this plan.
