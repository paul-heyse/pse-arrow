---
title: "Plan60b: Native relationships and lifetime evidence"
status: active
date: 2026-11-01
doc_role: plan
doc_owner: docs/plans/60-engineering-code-intelligence.md
doc_topics: [current-work, design, libraries, diagnostics]
review_sources: [docs/functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md, docs/plan_improvement_reviews/plan60-engineering-intelligence-2026-11-01.md, docs/functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md]
adrs: []
---

# Native relationships and lifetime evidence

## Purpose and foundation assessment

**Proposed:** deliver structural relationships, explained format/derive origins, and bounded
lifetime and contextual-predicate investigations. Original
[O01/O03/O11](60-engineering-code-intelligence.md#opportunity-dispositions) remain at the
coordinator. This document integrates the improvement review's
[I01/I02](../plan_improvement_reviews/plan60-engineering-intelligence-2026-11-01.md#I01),
[I04](../plan_improvement_reviews/plan60-engineering-intelligence-2026-11-01.md#I04), and the
functional review's [O01](../functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md#O01).
These review-local references do not rename original opportunities. The operator retained
explicit triggers for B04 and deeper compiler alternatives on 2026-11-01.

The authoring baseline is `ccf1b42da` plus relevant concurrent Plan62 native/proto/compiler
changes. Plan61 delivers typed constructors, generated RPC, validation and original context
guards. Plan62 source projects semantic roles/operations, captures, callable context, layout,
predicates and selected compiler capture/suspension/cleanup facts, while its latest checkpoint
leaves native/composed/compiler journeys and Q01 open. Reuse those producers with their limits;
this plan neither rebuilds them nor grants qualification credit to source existence. Blueprint
§21.7 and `semantic::{native,engine}` retain authority.

The native HIR/IDE family is 0.0.358; standalone syntax 0.0.360 is intentionally separate.
The live provider already has supported call/predicate operations and typed snapshots already
return selected original/adjusted type facts. Extend through their actual capability boundaries,
not a claim of universal provider parity. Native handles are not arbitrary Send/Sync payloads;
one selected world/session retains them, exporting owned views through existing complete records.
No new service, universal semantic database or preparation side effect is needed.

## Target design and selected mechanisms

Use HIR objects and native relations instead of parsing rendered type strings. The original
[review O01](../functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md#O01) and
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

Lifetime views combine lossless source, existing HIR observations, original acquired compiler
facts and actual managed-work outcomes around a selected body. Each hop states its provider and
unresolved source/body/phase correspondence. Source versus stored capture types, initialization,
saved storage, active loans, static cleanup and observed destruction remain distinct. Neither
cancellation nor dropping a delivery future establishes CPU settlement. No complete effect model
is selected. Demand-selected observations share request-local native reuse without exporting
handles or reloading an analysis for every edge. B02/B03 consume existing Plan61/62 contexts
independently of the full new B01 structural-relation set.

## Work packages

<a id="b01"></a>
### B01 — Native declaration, type and field relationships

Consumes A01's agreed typed-edge semantics; working exports require the consumed constructor/
validation slice and the existing native world owner.
Implement a useful early slice: containing declaration, parameter/return/field type dependency
and resolved uses sufficient for A02's request/policy consumer. Retain both qualified endpoints,
role/parameter position, original occurrence witnesses and parallel edges: two parameters using
one type are not one fact. Never derive identities from rendered type strings. Then complete
associated items,
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

Consumes existing Plan61/62 qualified context/body facts and original artifact/invocation owners,
not completion of B01. The consumer starts with a selected guard/cancellation/await concern,
inspects source operations, expands captures/stored fields and phase-qualified cleanup, then
follows the original managed outcome when physical settlement matters. Join only established
body/source/context correspondence and retain original addresses for deeper inspection.

Reuse projected `Closure/Coroutine::captured_items`, `Closure::fn_trait`,
`Semantics::expr_adjustments` and `compiler_investigation::facts::{Capture,SavedField,
SuspensionVariant,Cleanup,Body,Report}`. Demand-selected `locals_used`, unsafe-operation or
`drop_glue` observations may enrich a missing consumed fact through the same owner. Written
await/return/error paths remain AST observations. A03 owns the question-shaped view and F01 its
safe reference handoff. No second capture producer, borrow checker or lifecycle engine is planned.

Acceptance follows cancellation signal, CPU join owner, retained future/guard and sibling
request suppression/reset. A cancel call cannot imply finished CPU work; `drop_glue` cannot
imply a destructor ran; a shared reference cannot imply immutable state. Include overloaded
adjustment and unresolved macro/call boundaries. Tests should assert those evidence limits as
well as positive facts. Remove duplicated heuristic lifecycle summaries replaced by native
facts; keep original diagnostics/compiler evidence for questions these facts cannot settle.

Conditional compiler extension: only an actual decision requiring exact loans, regions or a
phase-specific fact not supplied by existing initialization/liveness observations can trigger
new extraction. First use acquired diagnostics/MIR/facts; probes remain separate explicit effects.
If insufficient, inspect exact pinned
`get_bodies_with_borrowck_facts`, native dataflow cursors and query timing before MIR is stolen:
[compiler consumer source](https://github.com/rust-lang/rust/blob/c1070d69382b8d2f2eb65119c738a77d9e324c9e/compiler/rustc_borrowck/src/consumers.rs).
Do not assume an after-analysis callback or newer overview API suffices. A selected component
requires its decision route, ownership and qualification; no driver/component installation or
prototype is a prerequisite to B02 completion while this trigger is absent.

The functional review's [O08](../functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md#O08)
adds a separate trigger: an implicit drop, shim or vtable dependency that current source
operations/diagnostics/artifacts cannot explain. The compiler owner then assesses pinned
`MentionedItems` or shim projection, accessibility and query-consumption order. Existing
`mir_promoted`, `mir_drops_elaborated_and_const_checked`, `optimized_mir` and
`mir_coroutine_witnesses` explain current phases; they do not authorize a general pass framework.
Decision evidence distinguishes generated bodies without unique source, normal/unwind cleanup,
C/C-unwind and incompatible phases. Only that deeper consumer waits. A reasoned rejection or
defer closes the investigation, not the base lifetime delivery; dormant alternatives never
require mandatory feasibility studies to complete this series.

<a id="b03"></a>
### B03 — Explicit contextual compatibility predicates

Consumes existing Plan61/62 qualified type identities, predicate facts and refinement/unknown
representation. New B01 edges are needed only for a consumer that traverses them. Expose the
requested operation/premises through the existing owner, extending only missing consumed facts.
Distinguish exact type equality, possible deep unification, coercion and trait applicability.
HIR0.0.358 provides
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
include_hovers)`. The original
[review O10](../functional_reviews/engineering-workflow-code-intelligence-2026-10-31.md#O10) and exact
[static-index source](https://docs.rs/crate/ra_ap_ide/0.0.358/source/src/static_index.rs)
show file-once traversal and first/selected-crate choices: this is not all-world semantics.
An export must state its context and omitted interpretations. Monikers are not identity across
arbitrary revisions. Keep owner-confined handles and export compact owned records.

If triggered, assess work structure, memory/lifetime/cancellation and selected-context fidelity
against the actual consumer. Measure a claimed speedup with a baseline; no benchmark is needed
merely to recognize duplicated scalar work. Persisted indexing and remote transport have
separate [coordinator triggers](60-engineering-code-intelligence.md#conditional-investigations).
No unconditional feasibility task, prototype or adoption is required to close the series.

<a id="b05"></a>
### B05 — Explained format and derive origins

Consumes B01's qualified structural-edge slice and A01's witness/unknown semantics. Deliver two
bounded questions: why a captured format-string substring refers to a selected local, and why
a selected implementation exists through a derive. Return original occurrences, established
targets and optional/plural source origins. An originating ADT is not exact inverse macro history.

Use pinned .358 `Semantics::{as_format_args_parts,check_for_format_args_template_with_file,
derive_helper,impl_generated_from_derive,descend_node_into_attributes,original_range_opt}`;
the [HIR addendum §§1–2](../libraries/rust_analyzers/ra_ap_hir.md) and
[exact semantic source](https://docs.rs/ra_ap_hir/0.0.358/src/ra_ap_hir/semantics.rs.html) explain
the boundaries. Formatting parts may be assembly operands, not global definitions. Procedural
derives require the relevant explicitly prepared extended world. Standalone syntax .360 cannot
replace the native .358 type universe. No inverse macro database is needed.

Acceptance follows a format occurrence or impl through typed inspection to original source.
Include escaped braces, captured identifiers, an unresolved custom macro, assembly operand,
multiple attribute origins, unsupported expansion and same-spelled unrelated local. Missing
preparation/origin stays unavailable. Known native fixture meaning and original bytes challenge
mapping; two renderings of one projection are insufficient. Migrate and delete replaced heuristic
provenance assembly after the typed consumer passes, retaining cheap text/AST nomination.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| B01 | planned | Native semantic relationship owner | Selected declaration/type/field/associated relations with honest calls/use classification |
| B02 | planned | Native body evidence owner | Joined existing capture/compiler/original-work evidence; deeper extraction conditional |
| B03 | planned | Native predicate owner | Existing contextual predicates consumed with origin checks and inconclusive cases |
| B04 | deferred | Native semantic owner | Observable bulk-work trigger and selected decision before any implementation |
| B05 | planned | Native provenance owner | Typed format-reference and derive-origin questions with optional/plural source correspondence |

## Verification and handoff

**Proposed:** targeted native-helper and host adapter tests exercise actual selected contexts,
positive facts and the counterexamples above. A live-provider pass does not qualify native
snapshot exports. Native preparation remains explicit, isolated from passive reads. E03 owns
actual engineering use; Q01 owns final integrated/static qualification. No new native,
compiler-private or performance claim follows from authoring this plan.
