---
title: "Plan73a: Shared mathematical preparation"
status: draft
date: 2026-11-09
doc_role: plan
doc_owner: docs/plans/73a-shared-mathematical-preparation.md
doc_topics: [current-work, design, libraries, numerical-execution, validation]
review_sources: [docs/functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md]
---

# Shared mathematical preparation

## Outcome and foundations

**Proposed.** Deliver O01 and mapped O08 in [Plan73](73-symbolica-ecosystem-improvements.md): one admitted definition supplies useful selected values, partials and actions while different physical instances retain their original guards, coordinates, provider premises and results. Own the remaining structural integration and deletion here, including Plan66 overlaps; no unfinished Plan66 package gates delivery.

**Interface-checked.** Baseline `baadc7e32`. [Current library construction](../../crates/pse-math/src/library.rs) uses `FunctionMap::add_aliases`, `Atom::evaluator_multiple` and direct translation to retain shared definitions in an instruction DAG. [ScalarCache/ScalarProducts](../../crates/pse-math/src/library/admission.rs) already share immutable exact preparations across interpreted/native Value selections. [LocalPartials](../../crates/pse-math/src/execution.rs), [typed partials](../../crates/pse-math/src/typed_partial.rs) and [dynamic layouts](../../crates/pse-math/src/jets.rs) already supply selected/action derivative structure. Extend these owners, not a second DAG, AD engine or global shape/cache service.

The source [preparation assessment](../functional_reviews/symbolica-ecosystem/preparation-and-lifecycle.md) and [transformation account](../library_leverage/libraries/symbolica/symbolic-transformations.md) explain the remaining opportunity: avoid repeated flatten/differentiate/reconstruct/optimize work where an actual consumer still performs it. Current functionality is a foundation, not a missing feature inferred from a review heading.

## Selected product and binding contract

A request identifies the admitted source/definition set, original ordered coordinates, selected outputs and derivative demand. Runtime values, directions, cotangents and weights bind at execution when they do not change structure. The existing product owner retains compatible exact mathematics and demand-specific derivatives once; numerical and analytical consumers obtain compact views appropriate to their lifetime.

Qualification includes consumed function semantics, guards, physical/provider support, layouts and derivative order. The low-level scalar key need not duplicate unrelated scientific identity, but the enclosing admitted product must qualify all premises consumed by a result. Changed values can reuse preparation; changed definitions, guard/provider meaning or selected axes invalidate affected structure. Share immutable backing and private workers, never transient refusal or mutable operation control.

Keep named local definitions and partials through requested products, using native replacement/collection or a reusable `Transformer` only for a real complete repeated pass. Thin callbacks need no wrapper. Use joint preparation for outputs requested together; retain independently useful products when demand or invalidation differs. Global fusion that prepares unrequested derivatives is not the target.

[Current direct products](../../crates/pse-math/src/products.rs) remain necessary where a finite weighted square-root result would encounter overflowing intermediate Second derivatives under a staged realization. Do not globally delete flattening/direct evaluation. Replace only a migrated product's redundant construction after its full behavior is preserved; original domain and derivative obligations remain mandatory. No silent fallback follows a selected route's construction refusal.

Mapped definition-led reuse transports ordered parameters, inputs, outputs, selected derivative axes/pairs and both Hessian coordinates. Retain quantity/datum compatibility, occurrence provenance, original guards and provider eligibility per physical instance. Preparation reuse is not result deduplication. Reordered equivalent bindings must interpret the same original coordinates; incompatible roles or axes refuse before execution. Compiler physical formals and current runtime preparation supply this authority, rather than a canonical graph replacing model identity.

## Selective native composition and additional correspondence

For one independently reusable pure constitutive module consumed alone and in a residual/response chain, compare joint symbolic construction, retained host-called modules and supplier composition. [Tagged3.0.2 EvaluatorComposer](../library_leverage/libraries/symbolica/evaluator-contract.md#selected-composition-contracts-refreshed-november-9) wires outputs to inputs and finishes with pruning, CSE/CPE and stack optimization without repeating symbolic Horner search. `merge` concatenates outputs and checks parameter count, not meaning; `Eq + Hash` coefficient requirements also matter.

Select composition when a real independent consumer repeats reconstruction/Horner work and combined invalidation fits. Prefer joint construction without independent reuse; retain host-called modules when independent invalidation dominates. Select derivative demand before inlining: retained sub-evaluators can block `vectorize`. Apply real specialization after rebuilding. B04 owns scoped supplier compatibility/backport; A owns the consumer outcome. No wholesale upgrade or parallel application composer is assumed.

Definition-led mappings are selected first. For independently authored equivalent motifs beyond that reuse, perform a finite additional-reuse comparison using role-colored incidence vertices for units, ports, connections, bindings and decision groups. [Graphica correspondence contracts](../library_leverage/libraries/graphica/mathematical-template-reuse.md) provide vertex transport, not a canonical edge map or proof of mathematical/provider equivalence. Equal canonical objects are candidate indexes requiring owner validation. Adopt graph matching only if it removes meaningful preparation surviving definition reuse; otherwise retain definition transport and record that concrete reopen condition. Physical IDs and source metadata are not dummy indices. E owns native indexed canonicalization, which can avoid graph projection entirely for genuine contractions.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| A01 | open | Coordinator scope and current source | Product/demand/binding contract and actual remaining consumer migrations/deletions identified; function/profile/receiving implications reconciled with B/C/D/E. |
| A02 | open | A01 and working B controls where construction changes | Named definitions and selected products reach fitting/solver/exact consumers; replaced repeated construction removed. |
| A03 | open | A01/A02 binding/product contract | Definition-led repeated physical instances share preparation with complete axis/output transport and distinct original results. |
| A04 | open | A01 module scenario and B04 compatibility outcome | Joint/host/composed route selected and implemented in actual independent consumers, with replaced preparation removed. |
| A05 | open | A03 and current structural/physical interpretation | Independent-motif graph matching decision closed and selected result integrated; no unused canonical-cache framework. |
| A06 | open | A02–A05 and relevant numerical/exact consumers | Actual consumers and retirement reconciled; independent actions, invalidation and interruption journeys ready for H. |

A01 traces current math products, compiler formals, runtime artifact preparation and fitting/solver/response callers, rather than preserving a historical inventory as truth. Record actual changed scope in its checkpoint; no permanent utilization service is required. A02 includes existing reverse/regional and direct requested products, with the finite-answer countercase as an explicit preservation test. A03 moves a repeated authored physical instance consumer, including original source reporting, rather than only a math-unit rebinding example.

A04/B04 resolve composition as one joint decision: the module scenario is available after A01, so B04 does not depend on completion of A04. A05 similarly integrates its finite decision into the actual reuse path or records no adoption with the unmet consumer/value condition. A06 includes source inspection that retired flattening, reconstructed aliases and duplicate partial setup are gone only where truly superseded. Keep useful independent behavior tests against the replacement; delete mechanism-specific fixtures/configuration with removed paths.

Build closure, fixed dual specialization, context isolation, compaction and streaming decisions belong to H01. A supplies actual representation/lifetime evidence; it does not add a scheduler, capacity cap or cache to make those inquiries possible.

## Verification

**Proposed:** scoped `pse check` and focused `pse test` for changed math/compiler/runtime consumers, with force-validation in Arrow closures. Use repeated chains/diamonds, reordered outputs, independently expected Jv/Jᵀw/Hv, changed values versus guard/layout/provider invalidation, distinct mapped-instance provenance and incompatible axes. Include finite weighted output with overflowing unused intermediate, cancellation and no partial retention. Supplier-vs-shared self-comparison alone is insufficient. H owns relevant assembled fitting/solver/native/receiver qualification after functional scope. No build, test or performance measurement was run in authoring; expected less repeated preparation remains qualitative.
