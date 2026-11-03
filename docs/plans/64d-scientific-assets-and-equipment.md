---
title: "Plan64d: Scientific assets and reusable equipment"
status: in-progress
date: 2026-11-02
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [current-work, design, validation]
review_sources: [docs/design_review/reviews/design_review_process_simulator_target_2026-11-02.md]
adrs: [ADR-0238]
---

# Scientific assets and reusable equipment

## Proto integration consumer obligations

Consume [64i](64i-engineering-contracts-and-validation.md) source-owned descriptors/validation
for selected property/reaction/equipment boundaries, including exact physical context and
declared versus prepared capability. [64k](64k-prepared-scientific-workers.md) delivers an
explicit local CoolProp water worker using the same scientific factory as the in-process
control; D02 retains physical science, datum, branch and derivative qualification. The worker
does not replace current native placement or establish additional fluid coverage. D04/D05
provide actual equipment consumers for the service/worker journeys. Migrate affected adapters
and tests when the new target replaces them; use the coordinator's Proto disposition owner,
not a second D-package status table. Existing closeout results retain their actual scope.

## Target and inspected foundation

**Proposed:** deliver P2/P3 scientific foundations and generic equipment by complete consumer closures and supported configurations. The [coordinator](64-process-simulator-target.md) owns combined scope and review disposition; this companion owns D01–D06. Source baseline is `1acd8c83c`, consuming the [integrated review](../design_review/reviews/design_review_process_simulator_target_2026-11-02.md), [scientific delivery contracts](../process-simulator-design/delivery/scientific-packages-and-equipment.md) and [finite IDAES scope](../process-simulator-design/delivery/idaes-scope-and-capability-coverage.md). The package table and current checkpoint own execution progress; document authoring established no scientific qualification.

**Interface-checked:** scientific definitions already live in authored packages. Relevant owners include [property selection](../../packages/reference/domain/models/properties.pse), [chemistry](../../packages/reference/physical/models/chemistry.pse), [equilibrium/state bindings](../../packages/reference/thermodynamics/models/equilibrium.pse), [reaction projection](../../packages/reference/thermodynamics/models/reactions.pse) and [process specialization](../../crates/pse-modeling/src/specialize/process.rs). Extend these consumers and their existing compiler/provider contracts; no second equipment graph, compulsory control-volume ancestry or new universal scientific AST is selected.

Current source is a starting implementation, not proof of complete configuration coverage. Ideal/PR, SRK-related records, NRTL, nonassociating PC-SAFT, caloric and existing state bindings must be assessed for their exact consumed behavior before adding duplicate science. Transport, general multiparameter properties, electrolyte/speciation, new state families and broader reactions remain selected scientific work, not capabilities granted by similar names.

## Scientific and equipment contract

Each increment delivers native definitions and exact source-attributed data; the supported component/phase/configuration population; demanded values, derivative order and validity; at least one consuming case; and independent evidence matching its claim. It preserves exact chemistry separately from empirical characterization, physical geometry separately from discretization, and scientific model substitution separately from implementation substitution.

State definitions own independent coordinates, supplied/receiving responsibility, reconstruction and transport observations. Equilibrium and numerical realization remain separate. Missing binary interaction data is not zero; a start does not select a requested stable branch; a potential evaluated at supplied density does not establish flash or global equilibrium. Retain support/refusal when derivatives, phase behavior or input records do not meet the consumer's contract.

A reusable unit exposes states, ports, optional balance regions/inventories, geometry, closure equations, supported configurations and specification freedoms. Reuse the physical definition across rating, sizing and optimization. A junction or boundary source need not contain a storage volume. Internal transfers cancel once under matching conventions; enclosing a subflowsheet must not count them twice. Material topology is not numerical solve order.

Reaction increments retain participating subjects, stoichiometry, extent/rate basis, kinetic/equilibrium law, data and formation/sensible-energy convention. Extend checked `reactions.Projection` where its exact chemistry applies; do not replace it with arbitrary per-component source callbacks. A weaker empirical conversion requires its own explicit accounting contract rather than an invented complete composition guarantee.

## Package state

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| D01 | in-progress | 64h H01 finite family/configuration population; 64a scope; 64b source policy | Scientific dependency groups, consuming cases, current capability evidence and missing contracts settled |
| D02 | in-progress | D01 selected state/property demands; 64b ordinary inputs | State/property/caloric/steam increments supply demanded outputs, derivatives, datum and branch/refusal behavior |
| D03 | in-progress | D01; selected transport/reaction dependencies | Transport, reactions and electrolyte/speciation increments satisfy their actual consuming contracts |
| D04 | planned | Relevant D02/D03 slices; 64c first-delivery composition | Generic equipment configurations and rating/design behavior delivered with initialization and original checks |
| D05 | planned | D04 selected equipment/property closures | HDA and methanol integration plus selected custom composition demonstrate complete steady workflows |
| D06 | planned | D02–D05 delivered populations; 64h correspondence | Selected scientific/equipment rows have independent configuration-specific evidence and truthful remaining coverage |

### D01 — Resolve the scientific dependency program

For each mandatory family/configuration selected by 64a, identify actual existing definitions and records, unresolved semantics, exact input source, supported numerical products and the smallest consuming case. Separate missing data, missing authored science, generic contract gaps, unimplemented integration and pending qualification. Package-name existence cannot settle any of those categories.

The investigation outputs consumer-specific scientific slices: state-coordinate families; cubic/activity/caloric extensions; steam/supercritical-fluid needs; equilibrium/stoichiometric chemistry; transport; and electrolyte/true-apparent representation. Each slice names its enabling contract and the downstream cases it unlocks. Unresolved phase/datum/derivative/representation choices block that slice's implementation design, not independent families. Do not require all of P2 before a usable equipment group proceeds.

### D02 — Deliver state and thermodynamic foundations

Extend demanded state-coordinate families through existing semantic slots and reconstruction. Verify state round trips and advected component/enthalpy correspondence, including selected zero-flow/disappearing-phase behavior. Preserve PR variant conventions, NRTL's potential-derived activity response and the distinction between nonassociating PC-SAFT and new association/polar science.

Deliver steam/supercritical-fluid capabilities with pump/turbine or heat-exchanger consumers. Compare authored and selected external realizations only for the complete demanded physical contract, including ideal/caloric contributions, datum, root/phase selection, derivatives and domain failures. No particular external dependency is adopted by this plan.

The selected first native increment is [ADR-0238](../adr/0238-native-water-property-provider.md):
CoolProp 8 HEOS Water through the existing full `coolprop-sys` C ABI and typed provider joins,
with coherent pressure/enthalpy/entropy, homogeneous first/second partials, explicit ideal-only
stock reference translation and actual pump/turbine consumers. It requires exact loaded-library
and scientific data admission; native properties alone do not qualify the configured consumers.
Wet-state derivatives and broader fluid/configuration obligations remain separate. The
[primary API](https://coolprop.org/coolprop/LowLevelAPI.html) explains attempt-owned state reuse.
teqp remains an alternative for residual Helmholtz/phase demands, with ideal/caloric completeness
and ancillary-versus-equilibrium meaning assessed separately ([multifluid](https://teqp.readthedocs.io/en/latest/models/multifluid.html), [derivatives](https://teqp.readthedocs.io/en/latest/derivs/derivs.html)).

### D03 — Deliver transport, reaction and electrolyte closures

Transport increments deliver phase/mixture rules and selected viscosity, conductivity or diffusion consumers. Reaction increments supply rate/equilibrium methods, extent normalization and reaction energy without double-counting formation enthalpy. Electrolyte work supplies actual speciation and true/apparent mappings, charge/composition and equilibrium equations, activity/transport requirements and independent checks; generic charge metadata is insufficient.

Use an equilibrium/stoichiometric reactor as a reaction witness and the selected rate-based capture/contacting consumer as the electrolyte/transport witness. Preserve the same scientific input and tolerance mapping for reference reproduction. Capability questions about missing sparse derivatives or nonsmooth phase transitions produce explicit refusals or qualified products; do not silently approximate an exact derivative claim.

### D04 — Deliver reusable generic equipment configurations

Expand the finite selected configurations of feeds/feed flashes, junctions/scalers/translators, mixers/separators, pressure changers, valves/pipes, flash, stoichiometric/equilibrium/Gibbs reactors, CSTR/PFR, solid-liquid separation, multi-stream contactors and exchanger families. Tray/column and custom-unit composition enter where mandatory examples demand them. Distributed/transient variants consume the physical contracts of 64e rather than removing current refusals early.

Each configuration owns its pressure/energy/material assumptions and effective fixed/free roles. Exercise rating and design over the same exchanger definition and a compatible property/closure substitution without rewriting side balances. Gibbs stationarity is not a stable-equilibrium certificate. Initialization uses existing staged/tear/continuation overlays and 64c's complete-report contract; final acceptance assesses the restored original problem.

IDAES `PropertyInterrogatorBlock`/`ReactionInterrogatorBlock` and `list_required_properties` illustrate demand discovery, not executable science or complete optional-configuration coverage ([2.13 documentation](https://idaes-pse.readthedocs.io/en/2.13.0/reference_guides/model_libraries/generic/property_models/interrogator.html)). Native specialization explains its actual demanded closure. [Tagged MSContactor behavior](https://github.com/IDAES/idaes-pse/blob/2.13.0/idaes/models/unit_models/mscontactor.py) supports composition without compulsory control-volume ancestry; do not translate its Python hierarchy.

### D05 — Integrate steady process and assessment consumers

Use the selected HDA simulation/optimization/costing and methanol-synthesis flowsheets as joined milestones. Match property/reaction assumptions, pressure closure, recycles and explicit design specifications before numerical comparison. Missing science remains attributed to its D02/D03/D04 owner; a setup failure cannot satisfy an intended recycle or domain-failure test.

Costing/assessment packages consume identified physical outputs and preserve currency/base date, period, utilization, factor provenance, allocation and system boundary. Keep capital, operating cost, emission and characterized impact distinct. The P1 exchanger correlation is a scoped witness, not evidence for all equipment purchase prices. Broader assessment inputs enter with their actual P3/P6 consumers rather than a comprehensive finance database.

### D06 — Close the selected scientific configurations

For each delivered configuration retain independent value/derivative, convention, original residual/bounds/domain, material/energy/element/charge and branch evidence appropriate to its claim. Compare a declared operating region and selected boundaries, not only one nominal point. Shared production calculations cannot generate the expected scientific answer being tested. Source/readback identity and numerical acceptance are separate checks.

Hand 64h exact native/reference configurations, output mappings, comparison allowances and unavailable/unattempted scope. A supported unit configuration can complete while another configuration remains open; a nominal heater pass cannot qualify the generic library. Independent specialized families in 64f consume the actual accepted slices, not the completion label of this whole document.

## Execution and evidence boundary

### Current checkpoint — 2026-11-03

The entire D–G scope and necessary enabling slices are authorized. H01's
[required-process crosswalk](64h-required-process-population.md) is being expanded from exact
sources; it is not a complete or numerically qualified denominator. The first D02 slice extends
the existing state/port/reconstruction owners with six coordinate families and explicit
zero-flow/phase limits. The first D03 slice supplies source-attributed constant/van't Hoff
log-equilibrium laws and an admitted reaction-activity projection. Their actual authored-source
controls now have the bounded passing evidence recorded below; no new family completion
or assembled scientific qualification is claimed.
Steam, transport, electrolyte, broader reaction/equipment and integrated process obligations
remain open under D02–D06.

Keep extraction/database access outside scalar property callbacks. Prepare immutable science once while its premises hold; attempt-owned mutable provider state is not shared between concurrent solves. Prefer native composed evaluation and sparse products where they meet the actual contract; retain necessary independent physical checks for every new numerical state. Repeated assembly, per-scalar crossings and nested pools need justified workload fit, but no cost model or universal streaming requirement follows.

**Planned verification:** targeted owner tests during implementation, selected real native/property/unit/reference integrations at scope end, and 64h assembled finite-population acceptance. Preserve original failed, unavailable and not-run evidence. Performance/robustness gains remain unmeasured until a matched-quality campaign establishes them. No current new scientific conformance status follows from this plan.

Next work is D01's dependency investigation alongside ready 64b/64c first-delivery work. New generic contracts use the existing decision route and migrate their actual consumers together; replaced mechanisms, callers, tests and fixtures retire once their target replacements pass. This companion neither reopens completed solver machinery nor absorbs existing Plan53/58h qualification.

### Stopped implementation checkpoint

**2026-11-03 — partial implementation; work stopped by instruction.** D01–D03
remain in progress; D04–D06 and the remaining scientific population are not complete.

**Tested:** `scripts/pse test pse-compiler --lib -E 'test(scientific_state_) | test(scientific_equilibrium_) | test(scientific_inventory_)'`
passed 17/17 on local Linux with correctness-route Arrow validation: seven state-coordinate,
five equilibrium-law/projection and five inventory controls; zero failures. This includes
the corrected independently supplied equilibrium point, original residual perturbation,
scope/extent/domain refusals and exact inventory input coverage. Invocation:
`01a1a355-1d25-70a0-81ab-50f17afb83b2`. It is compiler/model evidence, not the pending
native state/inventory solve population or independent IDAES qualification.

**Tested at the earlier provider revision:** `scripts/pse test pse-backend-native --lib --features coolprop -E 'test(coolprop::tests)'`
passed five native Water controls against the declared released reference values,
datum, derivatives and branch behavior on Linux x86_64. An equivalent iterator cleanup
followed; the final backend rerun was not run. The optional dependency is pinned to
`coolprop-sys =8.0.0`; preparation is explicit, and source names never load a provider.

**Failing:** the latest steam-only runtime selector ran four tests: the registered
document-ingress control passed and all three pump/turbine cases failed during package
admission with `rpp4_oracle: package data document is not supplied`. Invocation:
`01a1a380-1f0b-76d3-bfa9-cf8194f196a6`. The typed CAS fact/source predicate correction
is preserved, but the positive equipment solves and wrong-CAS refusal discriminator
did not reach execution. Resolve the actual package/data closure; do not classify this
as a native solver or property-value result. [64f](64f-specialized-process-families.md#stopped-implementation-checkpoint)
owns those configured consumer failures.

Native state/inventory ten-fixture acceptance, final provider and xtask selection
controls, broader D02/D03 science and D04–D06 acceptance remain unrun/open. The
[coordinator checkpoint](64-process-simulator-target.md#d-through-g-implementation-checkpoint)
records explicitly skipped Clippy and other follow-ups. No speedup, full family
completion or comprehensive qualification is claimed.
