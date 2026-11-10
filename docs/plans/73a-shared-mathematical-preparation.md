---
title: "Plan73a: Shared mathematical preparation"
status: complete
date: 2026-11-09
doc_role: plan
doc_owner: docs/plans/73a-shared-mathematical-preparation.md
doc_topics: [current-work, design, libraries, numerical-execution, validation]
review_sources: [docs/functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md]
---

# Shared mathematical preparation

## What this document changes

- [A01](#A01) strengthens the existing compiler request → retained scalar family → demanded-product boundary; it does not create another cache.
- [A02](#A02) changes derivative construction to retain named shared mathematics and removes redundant reconstruction for migrated products.
- [A03](#A03) changes existing assembly entry/action preparation and execution to use one admitted instance-transport projection; it removes repeated interpretation of binding scales, aliases and destinations.
- [A04](#A04) resolves joint/retained/composed preparation for a separately useful caloric module and integrates the selected route.
- [A05](#A05) is a bounded investigation of additional reuse across independently authored motifs, beyond the existing definition/body reuse.
- [A06](#A06) completes production fitting/solver integration and removes the pathways displaced by those changes.

These packages include cross-cutting shared machinery as well as caller migrations. A04/A05 may conclude that the existing route is preferable; A01–A03/A06 have concrete consolidation and integration work, not an open-ended opportunity search.

## Outcome and foundations

**Implemented, Tested.** Delivered O01 and mapped O08 in [Plan73](73-symbolica-ecosystem-improvements.md): one admitted definition supplies useful selected values, partials and actions while different physical instances retain their original guards, coordinates, provider premises and results. Own the remaining structural integration and deletion here, including Plan66 overlaps; no unfinished Plan66 package gates delivery.

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
| [A01](#A01) | complete | Coordinator scope and current source | Product/demand/binding contract and actual remaining consumer migrations/deletions identified; function/profile/receiving implications reconciled with B/C/D/E. |
| [A02](#A02) | complete | A01 and working B controls where construction changes | Named definitions and selected products reach fitting/solver/exact consumers; replaced repeated construction removed. |
| [A03](#A03) | complete | A01/A02 binding/product contract | Definition-led repeated physical instances share preparation with complete axis/output transport and distinct original results. |
| [A04](#A04) | complete | A01 module scenario and B04 compatibility outcome | Joint/host/composed route selected and implemented in actual independent consumers, with replaced preparation removed. |
| [A05](#A05) | complete | A03 and current structural/physical interpretation | Independent-motif graph matching decision closed and selected result integrated; no unused canonical-cache framework. |
| [A06](#A06) | complete | A02–A05 and relevant numerical/exact consumers | Actual consumers and retirement reconciled; independent actions, invalidation and interruption journeys ready for H. |

## Implementation scope and package detail

This plan changes shared preparation and assembly, not the authored model's scientific meaning. A01 establishes the concrete shared product contract; A02 removes repeated symbolic preparation within it; A03 consolidates instance transport across existing derivative/entry consumers; A04 changes a separately reusable module's construction only when native composition earns it; A05 resolves graph-assisted reuse beyond definition identity; A06 closes the actual consumer migrations and deletions. A04/A05 contain bounded comparative decisions, not unconditional new cache projects.

The following sections own each package's scope. The table above is a navigation index, not the implementation specification. Current paths and tests are preservation evidence, not claims that all the improvements below are absent today.

<a id="A01"></a>
### A01 — Make the shared mathematical product contract explicit at the existing boundary

**Change and location.** Extend the existing compiler-issued `ArtifactRequest` and retained scalar-product route. The current path is compiler resolved region/formals → `PreparedSupport`/`LocalDemand` → `ArtifactRequest` → runtime `MathService::scalar_products` → `ScalarProducts` → demanded compiled product. The target keeps that path authoritative and makes its reuse boundary explicit for every selected product, instead of letting later callers reconstruct or independently qualify equivalent preparation.

The concrete owners are [ArtifactRequest and artifact_request](../../crates/pse-compiler/src/workspace.rs), [runtime artifact retention](../../crates/pse-runtime/src/math/artifacts.rs), [scalar preparation](../../crates/pse-math/src/library/admission.rs), and [product construction](../../crates/pse-math/src/products.rs). `ArtifactRequest::scalar_key` already separates exact scalar families from requested Taylor order/action; `build_scalars` and `build_reusing` already route through retained preparation. `MathService` already has reservation-owning scalar flights and invalidation epochs. Preserve these mechanisms; a second cache or caller-supplied independent key is outside scope.

**Target contract.** The compiler owns an ordered source/definition and support product, selected outputs/coordinates, function/provider premises and original source/guard attribution. Exact scalar identity includes the compatible ordered mathematics and consumed optimization/environment settings. Demanded numerical products additionally qualify product kind, derivative shape, layout and numerical profile. Runtime input values, directions, weights and cotangents remain bindings. Source/physical instance identity and mutable worker state do not become part of a reusable mathematical expression merely to prevent unsafe reuse elsewhere: they remain qualified by their existing enclosing admission/assembly owners.

A01 documents this distinction beside these actual types and updates the existing request/retained-product interface where it cannot represent the selected reuse. It produces an explicit migration list in this package checkpoint covering `products.rs`, regional partial setup, assembly entry/action preparation, compiler modeling formals and runtime artifact requests. For each, name the retained product consumed and the old construction removed, or establish that its distinct realization remains. Refining that list during implementation is allowed; discovery of A03's entire target is not deferred to it.

**Library composition.** `FunctionMap::add_aliases` expands named definitions in caller-local scope. `add_function` instead has explicit arguments that shadow matching globals. Preserve this distinction when passing definitions through `Atom::evaluator_multiple(...).add_aliases(...).direct_translation(true).build()`. This existing composition retains a shared instruction DAG and joint outputs. See [maintained function map/builder](../../vendor/symbolica/src/evaluate/function_map.rs) and [library construction](../../crates/pse-math/src/library.rs). A01 adds no application expression interpreter.

**Completion.** The selected contract must let B describe receiving without inventing source authority, C bind runtime batches without recompiling structure and D/E consume selected exact views without rebuilding source names. Check existing interpreted/native scalar-family reuse, different output/axis ordering, changed guards/environment and cancellation/eviction with live retained owners. Remove only redundant caller-side descriptors/keys displaced by this boundary; do not count a new type declaration alone as delivery. Joint contract design with B01/C01 is early work, not a circular requirement for all three implementations to be complete.

<a id="A02"></a>
### A02 — Preserve shared definitions through requested derivative construction

**Change and location.** Work in `products.rs::{compile_product_inner,compile_direct_product}`, `execution.rs::LocalPartials` and [typed partial lowering](../../crates/pse-math/src/typed_partial.rs). The current direct route creates temporary flattened expressions, then constructs a weighted expression and differentiates it. Regional routes already keep local first/second partial aliases. The target carries selected named definitions and partials through compatible demanded products and retains that exact preparation for reuse, avoiding reconstruction for each output/product where the operation can preserve sharing.

Use existing `ProductKind`, `JetLayout` and `compile_product_reusing` as the public mathematical entry. `LocalPartials::new` continues asking Symbolica for derivatives and assigning named aliases. `typed_partial` already lifts/rebinds shared function calls through native `replace_map`; retain its lexical/source semantics. For pure weighted products, weights remain formal runtime arguments. Differentiate the weighted/action expression with Symbolica and feed the needed aliases and outputs jointly into the existing evaluator builder. Do not implement derivative rules in application code or materialize a dense Hessian for an action.

**Preserved distinct route.** Product preflight currently chooses Direct, Reverse or Regional before construction. Keep that semantic choice. The direct square-root weighted-product case can have a finite final result while intermediate Second components overflow; a universal local-jet replacement is incorrect. Reuse named definitions where it preserves that result; retain bounded direct normalization/differentiation where required. No fallback is selected after a chosen route refuses or cancels.

**Migrations and deletion.** The exact callers move through compiler `ArtifactRequest::build_reusing` and runtime artifact retention, including the prepared products assembled for fitting/solver entry and action consumers. Delete repeated alias rebuilding, flattened copies and per-output optimization only for products now supplied by the shared route. Keep direct flattening for cases whose replacement has not established the same requested finite/error behavior. Remove superseded mechanism fixtures while retaining [product guard controls](../../crates/pse-math/src/product_guard_tests.rs) and independent action behavior.

**Acceptance.** Use a chain/diamond with common intermediate `u=x*y`, outputs `u+x` and `u*u`, independently derived Jv/Jᵀw/weighted-Hv, selected reordered outputs and changed runtime weights. Verify preparation reuse through the real artifact owner, not two library handles created by a test. Include first-only demand constructing no Second products, original derivative-order refusal, cancellation with no retained success and the existing finite weighted square-root countercase. B's working checked construction is required only for newly changed supplier operations; this package cannot silently weaken current bounds while waiting for B.

<a id="A03"></a>
### A03 — Consolidate existing instance and derivative transport across assembly pathways

**Existing capability and actual delta.** [InstanceBinding](../../crates/pse-math/src/binding.rs) already separates instance identity from reusable body identity, holds checked source/member attribution, ordered converted slots and output contributions. `CaseStructure` already adds bindings for known body shapes. Compiler `AdmittedModeling` already groups bodies by normalized identity. A03 does not introduce another template registry or claim that instance reuse is missing.

The remaining cross-cutting change is to make the existing binding transport a shared, demand-specific immutable projection used by ordinary selected entries and action assembly. Current [assembly planning/execution](../../crates/pse-math/src/assembly.rs) builds gradient coordinates, constraint/objective terms, sparse Jacobian/Hessian terms and action gathers/scatters in separate routes; [assembly_entries](../../crates/pse-math/src/assembly_entries.rs) separately builds selected entry destinations/scales. These pathways repeatedly interpret the same formal→case coordinate and contribution mapping. Consolidate that interpretation while preserving their compact result layouts and exact selected support.

**Target dataflow and interface.** Extend the existing bound-instance/assembly preparation owner with one immutable sparse transport projection derived from `InstanceBinding`, demanded formals/outputs and selected coordinate support. It contains admitted input gather indices/scales/offsets, output contribution destinations/scales, alias accumulation and source/member correspondence. Existing entry, gradient, Jacobian and action plans consume compact views of this projection instead of independently recomputing conversion and binding rules. Keep their specialized sparse storage and numerical worker kernels; do not replace them with dense mapping matrices or a generic graph engine.

For explanation, write local canonical coordinates as `z=P*x+b` and output contribution assembly as `y=R*f(z)`. P/R denote sparse gathers/scatters with repeated indices, not materialized dense matrices. Values include b; directions use P without b. Jacobian actions use `R*J*(P*v)`, transpose actions accumulate `Pᵀ*Jᵀ*(Rᵀ*w)`, and weighted Hessian actions use `Pᵀ*H_(Rᵀ*w)*(P*v)`. Repeated input aliases and output targets accumulate rather than overwrite. Selected Hessian entries must include both input conversion scales. If a stored off-diagonal local pair maps through aliased inputs to one global diagonal, include both symmetric contributions before accumulation; deduplicating the global destination must not discard one term. Offsets never multiply a derivative. Nonlinear/composition-dependent physical transformations are outside this admitted affine unit-conversion map and stay with their scientific owner.

**Library responsibility.** Symbolica owns the reusable local expression, its derivatives and evaluator; Numerica supplies the underlying numeric domains. The existing `FunctionMap::add_aliases`/joint evaluator composition from A01/A02 supplies that local program once. Physical gather/scatter and source attribution remain thin application responsibilities; no library canonicalizer can infer them. A03 reuses the local library product and consolidates its application transport, rather than inventing another differentiation engine to move derivatives between coordinates.

**Affected pathways and deletion.** Migrate `CaseAssembly` preparation, `case_patterns`, selected `EntryPlan` construction and worker `gradient`, `jacobian`, `hessian`, `jacobian_product`, `jacobian_transpose_product`, `hessian_product` transport to the shared projection. The runtime modeling assembly feeding fitting/solver oracles is the actual production integration path. Delete duplicate conversion-scale/destination interpretation in these routes once all consume the projection. Keep per-instance guard/provider execution, source errors, mutable trials and result provenance distinct; sharing does not permit executing one instance and copying its scientific outcome to another.

**Acceptance and readiness.** Start from existing `equal_normalized_gathers_reuse_arithmetic_but_keep_each_instance_scatter_and_bits` and alias/action controls in assembly tests as implemented baseline. Add two instances sharing a body with different source coordinates, reordered axes, nonzero affine offsets, unequal scales, repeated inputs and repeated row contributions. Verify independent value/entry/action expectations under the equations above, including global symmetric-pair collapse. Change one instance's physical support/guard and verify correct refusal or invalidation without changing the other's attribution. Check that a source error names the correct instance even when arithmetic is shared. A03 finishes with the real assembly/entry consumers migrated and repeated binding interpretation removed; a test showing two instances reference one body is not sufficient.

<a id="A04"></a>
### A04 — Resolve and integrate independent module composition

The selected comparison is the authored `properties.caloric_dh(s,T_reference,T)` call used by `CaloricReference.enthalpy`, with the same pure caloric definition consumed inside the ordinary heater energy residual. The module signature retains selected correlation parameters and both temperatures; the response additionally binds flow and inlet/outlet state. Use the compiler resolved callable/formal boundary, not a runtime provider callback. [CaloricReference](../../packages/reference/methods/models/caloric.pse) supplies the standalone property observation, and F's ordinary heater uses the energy response. This comparison may use the new study consumer alongside existing property/solver preparation; label that addition honestly. If current joint preparation already shares all relevant construction, select that route rather than inventing repeated work.

Compare three complete routes: one joint alias build, separately retained host-called programs, and tagged3.0.2 `EvaluatorComposer` connecting outputs to later inputs. The composer performs dead-work pruning and instruction CSE/CPE/stack optimization without repeating symbolic Horner; [the versioned contract](../library_leverage/libraries/symbolica/evaluator-contract.md#selected-composition-contracts-refreshed-november-9) supplies that mechanism. `merge` concatenates outputs and checks input count, so it is not a substitute for ordered semantic binding or chained composition.

Inputs include exact coefficients, ordered formal/output maps, selected derivative demand and independent invalidation premises. B04 supplies a compatible maintained composer backport/rebase only if this comparison selects it; both packages share the A01 scenario, so they do not depend on each other's completed implementation. Retained non-inline bodies can prevent `vectorize`; select inlining before derivative expansion and real flags after rebuilding. Mathematical equality cannot prune original guards or fallible effects.

Select native composition only when independently consumed modules would otherwise reconstruct/reoptimize symbolic mathematics and combined lifetime/invalidation is appropriate. Otherwise deliver the simpler joint or retained-module route. Integrate the selected route into the existing product owner, remove the migrated reconstruction/optimizer setup and any unused candidate adapter, and record the decision at this section. Validate module-alone/chain results, reordered incompatible inputs, independent module edit, requested derivatives and canceled composition. No timing campaign is required to establish removed construction; claims about its magnitude would require measurement.

<a id="A05"></a>
### A05 — Decide additional reuse for independently authored motifs

This finite investigation concerns independently authored equivalent motifs that cannot share a definition identity. It is separate from A03's existing instance bindings and E's genuine indexed canonicalization. Its selected case is the ordinary heater energy-balance region used by F and an independently authored equivalent of that region in G's candidate interpretation, with reordered local declarations and renamed occurrence paths. Retain identical physical heat/datum/port roles; include parallel connection occurrences in the structural control. Compare their existing body/scalar identities first. The input is the two compiler-selected pure regions with binding/guard roles; the output is a validated additional correspondence or a reasoned decision that current definition/body reuse already supplies it. A copied example with no additional production preparation is not evidence for graph-cache adoption.

Use Graphica3.0.1 `Graph::canonize` on a role-colored incidence representation only if this case exposes remaining useful repeated preparation. Represent units, ports, connection occurrences and binding/decision roles as vertices where transport requires them, because the returned map covers vertices, not edges. Compose original→canonical maps to propose region correspondence, then have the existing physical/mathematical owner validate exact definitions, original guards, provider premises and axis transport. Include isolated units, parallel roles and fixed physical identities. `HiddenData` excludes metadata from comparison; it does not canonically order source provenance or prove scientific equivalence. See [correspondence source account](../library_leverage/libraries/graphica/mathematical-template-reuse.md).

Do not add whole-flowsheet canonicalization to every preparation. If the candidate cannot remove work surviving existing definition/body/scalar reuse, close with no adoption, the examined case and the concrete condition that would reopen it. If selected, integrate correspondence into A03's existing transport and preparation owner, delete replaced matching/reconstruction, and verify renamed-equivalent, role-changed, guarded and parallel-edge countercases. Canonical graph equality alone is never acceptance.

<a id="A06"></a>
### A06 — Complete the shared preparation and assembly cutover

Inspect the actual routes changed by A02/A03/A04 and any adopted A05 result: compiler `ArtifactRequest`, runtime scalar/artifact retention, math product construction, assembly entry/action consumers and their fitting/solver uses. Each must consume the chosen retained product/transport through one authoritative route. This is integration and retirement work, not an excuse to discover the earlier package scopes at the end.

Remove replaced reconstruction passes, alias/partial setup, duplicate transport interpretation, adapters, settings and mechanism-specific fixtures. Preserve semantically distinct direct/regional/reverse realizations, per-instance guards, source attribution, selected sparse support and independent scientific tests. Confirm compatible numerical rebinding avoids the identified construction using existing observation hooks or source ownership evidence; no new utilization/benchmark framework is selected.

Exercise a repeated fitting/residual assembly through the runtime owner with entry, Jv/Jᵀw and Hessian-action demands, original solver coordinates and changed physical support. Independent analytic small models expose transport/sign/order mistakes. Include cancellation/eviction while another live worker retains shared backing, no partial artifact publication and separate mutable scratch. Pass the actual scope, removals and unresolved unsupported regions to H, without claiming all Plan66 physical populations complete.

## Verification

**Implemented, Tested.** Local acceptance is complete: the full compiled-evaluator math selection passed 640/640, selected compiler acceptance passed 8/8, and the shared runtime diamond/fitting controls passed their composite affected reruns, baseline zero. See the [Outcome](#outcome) and [73b's exact verification account](73b-supplier-controls-and-qualified-receiving.md#outcome). H retains assembled A–G, independent scientific and static qualification; no speedup magnitude is measured.

## Execution checkpoint — 2026-11-10

Execution of the integrated A/B/C scope began at `d2302f32a` with a clean checkout. The selected first assignments are shared transport consolidation, checked supplier construction and stable primitives. A01/B01/C01 are coordinated prerequisite contracts; A03 proceeds within existing binding semantics. Required new kernel/primitive decisions precede dependent implementation. All other scope remains open until its actual production cutover and targeted evidence; broader Plan73/Plan66 completion is not claimed.

**A03 — Implemented, Tested.** Commit `d3b6fefc3` replaces separate gradient coordinates,
entry source/scale interpretation and action destination interpretation with one immutable
`BindingTransport` per physical instance. Demands share admitted `EvalBindings` addresses with
compact ordered views. Selected entries, gradient/Jacobian/Hessian and all three action routes
use the projection. Original instance guards and source/member attribution remain separate.

`scripts/pse test pse-math --lib -E 'test(shared_transport_preserves_affine_aliases_reordered_entries_and_all_actions) | test(shared_body_guard_refusal_retains_the_actual_receiver_instance) | test(equal_normalized_gathers_reuse_arithmetic_but_keep_each_instance_scatter_and_bits) | test(receiver_context_guards_are_excluded_from_instance_donation_and_value_does_not_grant_first) | test(aliases_repeated_rows_isolates_and_stable_refill) | test(demanded_directional_case_actions) | test(weighted_case_hessian_actions) | test(constraint_cotangents_use_direct_products) | test(assembly::entries::tests) | test(compiled_gathers_refuse) | test(symbolic_definition_regions_compose_mixed_partials_without_flattening_lets)'`
passed 14/14 in the default Nextest profile, zero failures against baseline zero, on 2026-11-10.
The shared `scripts/pse check pse-math --all-targets` also passed with zero failures.
This establishes the scoped transport contract; it does not qualify an end-to-end fitting
journey or close A06/H.

A01/A02/A06 are integrating named local-partial retention into the existing scalar family,
artifact flights and leases. The obsolete per-compilation pointer cache is being removed.
The bounded Direct route remains required for finite weighted results with overflowing
unrequested intermediates. A04/A05 are comparing actual caloric and independently authored
heater consumers against existing joint aliases and canonical physical/formal correspondence;
no composer or graph cache has been adopted.


## Integrated shared-preparation checkpoint — 2026-11-10

**Implemented.** A01/A02 retain named `LocalPartials` in the existing `ScalarProducts`
family. Exact definition/parameter/dependency/formal/order meaning selects the retained
preparation; mutable workers and caller cancellation do not. Compiler requests now expose
transpose, weighted-Hessian action and selected-entry demands through the existing product
boundary. Runtime artifact completion publishes the newly demanded named partials back into
the existing scalar entry with its original allocation lease. The obsolete per-compilation
pointer cache is removed. Direct, Reverse and Regional remain distinct: Direct preserves the
finite weighted square-root countercase where unused local Second entries would overflow.

**Implemented.** A06 uses those same compiler requests and runtime flights for the assembly
that feeds fitting and solver oracles. A03 already removed repeated scale/destination
interpretation. No alternate derivative cache, dense transport matrix, compatibility shim or
result deduplication was introduced. Runtime cache eviction retires residency while live
workers retain the first allocation owner; interrupted construction cannot publish success.

**Tested.** `scripts/pse test pse-runtime --lib -E
 'test(artifact_products_retain_named_diamond_partials_across_runtime_actions_and_eviction)'`
passed 1/1, zero failures against baseline zero, in the default Nextest profile. The actual
artifact owner supplies analytic selected entries, Jv, transpose-Jacobian and weighted-Hessian
actions for a shared diamond, changed runtime weights, live-worker eviction, preparation
identity reuse and canceled nonpublication. This is the shared runtime/assembly acceptance;
H retains assembled scientific fitting/solver-population qualification.

**Implemented — bounded decisions.** A04/B04 retain joint aliases through
`FunctionMap::add_aliases` and the existing direct-translation evaluator. The actual
`caloric_dh`/`caloric_ds` module and heater energy response already lower through that shared
owner. No separately retained native module is repeatedly reconstructed in the selected
consumer; adding `EvaluatorComposer` would add wiring, derivative, invalidation and module
lifetime machinery without deleting that work. No composer backport or dependency move is
selected. Reopen when an independently retained native module feeds a later demand and
repeats symbolic/Horner construction under compatible guard/profile/invalidation premises.

A05's renamed and reordered independently authored heaters already have equal canonical body
keys. Changed original guards and repeated-term multiplicity remain different; parallel
physical connection occurrences and isolated units remain distinct. No graph matcher or
canonicalization cache is selected. Reopen if useful repeated preparation remains after the
existing body/scalar reuse and exact physical/guard/multiplicity correspondence can establish
its transport.

**Tested.** `scripts/pse test pse-compiler --lib -E
 'test(actual_caloric_module_and_energy_response_keep_joint_aliases_and_independent_invalidation)'`
passed 1/1, zero failures against baseline zero, in the default Nextest profile. Independently
expected caloric Value/First, coefficient-edit invalidation, surviving old products, physical
input refusal and cancellation exercise the real compiler consumer. The companion authored
heater/correspondence controls passed in the same scoped compiler investigation. These
bounded choices complete A04/A05; they do not imply a general absence of composition or graph
opportunities elsewhere in Plan73.

## Outcome

**Implemented, Tested — 2026-11-10.** A01–A06 are complete. Compiler-issued demands,
retained named partials in the existing scalar family, and one immutable instance transport
now serve selected entries and direct actions through runtime artifact retention and the
actual fitting/solver assembly. Replaced pointer-local preparation and duplicate destination,
scale and contribution interpretation are removed. Direct, regional and reverse products
retain their distinct scientific validity; shared preparation does not imply shared mutable
scratch or a weakened original guard. B's checked alias/build/Taylor/map cutover completes
the common construction path used by these consumers.

**Tested.** The final `scripts/pse test pse-math --lib --features compiled-evaluator` passed
640/640 with zero skips and zero compiler warnings, baseline zero. The selected compiler
acceptance passed 8/8, including actual caloric-module aliases/invalidation, original numerical
requests/binding and symbolic replay. The runtime shared-diamond control passes unchanged
under its original 8 MiB fixture allowance after the final supplier bound correction, preserving
independent entry/Jv/Jᵀw/Hessian-action answers, changed weights, cache eviction with a live
worker, retained named partials and canceled nonpublication. The two actual steady-fitting
population controls also pass through serial and existing assigned native lanes. The exact
commands, composite runtime result and exclusions are recorded in [73b's Outcome](73b-supplier-controls-and-qualified-receiving.md#outcome).

**A mistake made and corrected.** Initial-allocation estimates were still treated as final
program/worker bounds by two retained tests after adaptive supplier admission replaced the
private predictor. Those assertions now compare actual retained backing with the same
captured admission owner, preserving coordinate, axis, numerical and worker checks. The
request's earlier `binary64-rne` label also overstated the general coefficient conversion;
its frame now names the actual pinned conversion without changing that arithmetic policy.

**Deliberate choices.** Joint native aliases already supply the caloric module and heater
response, so no composer backport, new module owner or unused adapter was added. Renamed and
reordered heaters already share normalized body keys, while changed guards/multiplicity and
physical occurrences remain distinct; no additional graph matcher/cache was selected. The
concrete reopening conditions remain in A04/A05. Expected preparation savings follow from
removed work and retained ownership; no measured speedup or general graph equivalence claim
is made.

The enduring contracts are in blueprint §14.3.1–§14.3.3 and ADR-0263/0264. This completed
companion remains an evidence input to the active coordinator and 73h; retirement waits for
those consumers. H's assembled qualification and the remaining D–G capabilities are separate.
Plan66 has not been edited or marked complete.
