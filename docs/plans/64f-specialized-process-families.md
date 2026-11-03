---
title: "Plan64f: Specialized process families"
status: in-progress
date: 2026-11-02
adrs: []
review_sources: [docs/design_review/reviews/design_review_process_simulator_target_2026-11-02.md]
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [current-work, design, validation]
---

# Specialized process families

## Purpose and basis

**Proposed:** realize P5's finite released/configured process-family population as reusable authored science with complete consuming workflows. The [coordinator](64-process-simulator-target.md) owns combined scope, rule decisions and finding disposition; this companion owns F01–F07 package state. These local F package IDs are distinct from the source review's finding IDs.

**Interface-checked planning basis:** source `1acd8c83ca8f9e6f6e9b5e9c9cbf98e299d574b8`, the [integrated target review](../design_review/reviews/design_review_process_simulator_target_2026-11-02.md) and the [specialized-family roadmap](../process-simulator-design/delivery/implementation-sequence.md#p5--deliver-specialized-process-families-as-parallel-scientific-packages). Core 3.4, efficiency heuristics 1.0, ProcessSimulator 1.5 and test-assessment 1.2 govern the design. The full P5 scientific population still needs the finite P0 configuration crosswalk; documentation adoption is not execution evidence.

## Foundations and responsibility boundary

Retain existing authored process units under `packages/reference/process/models/`, scientific states/property/reaction definitions, native admission and `pse-modeling::specialize::process` state/port/inventory contracts. `control-volumes.pse`, `control-volume-1d.pse`, `reactors.pse`, `heat-exchanger.pse`, `pressure-changer.pse` and `costing.pse` are reusable starting surfaces, not proof that complete specialized families already exist.

Scientific and equipment meaning remains with authored package owners; numerical iteration stays with fitting native libraries. Use [models and composition](../authoritative_design/sections/models-and-composition.md), [physical semantics](../authoritative_design/sections/physical-semantics.md) and [workflows/results](../authoritative_design/sections/workflows-and-results.md) as enduring contract owners. A specialized family may require substantial equations and data without requiring a parallel object graph, universal plugin system or one generic Rust branch per equipment name.

F consumes exact ordinary source/data closures from 64b, selected P2/P3 properties/reactions/equipment from 64d, and only the required P4 storage/distributed/event slices from 64e. It supplies configured native model consumers and outputs to 64g application workflows and 64h reference closure. Independent tracks may proceed when their actual prerequisites are working; no whole-companion barrier is implied.

## Target and scope decisions

Every family declares its supported configurations, scientific assumptions, physical conventions, source/data revisions, derivative and validity contracts, initialization policy, outputs and attributable refusals. An equipment name is not a coverage unit: pressure models, phases, reactions, holdup, transfer options, discretization and selected property packages can change the problem.

Use the finite release-tree and independently pinned example/compatibility population owned by 64a/64h. Required upstream optional dependencies remain visible; absence is a prerequisite failure or a qualified substitute decision, not a reason to remove a mandatory case silently. Additional industry archetypes remain contrasting design cases until selected as finite increments; they do not automatically enlarge P5's denominator.

The [IDAES overview](../libraries/idaes-pse.md) and [released-scope inventory](../process-simulator-design/delivery/idaes-scope-and-capability-coverage.md) orient behavior and configuration discovery. Read exact 2.13.0 behavior and independent reference sources clean-room; reproduce scientific meaning, not Python implementation or class ancestry. A literal whole-distribution claim would also need supplemental non-process obligations owned outside this process-family companion.

## Package state

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| F01 | in-progress | P0 finite released/configuration population | Family/configuration and scientific-gap decisions assign each required variant to a concrete dependency and acceptance route |
| F02 | planned | F01 power cases; 64d pure-fluid/pressure/heat/costing closure; required 64e physics | Steam, supercritical and integrated power cases have matched formulations, ordinary data and complete selected outputs |
| F03 | planned | F01 gas-solid cases; separate states, heterogeneous reaction/transfer and required 64e storage | Selected fixed/moving/bubbling beds and chemical-looping configurations preserve basis, source and conservation meaning |
| F04 | planned | F01 capture cases; 64d speciation/transport/columns/membrane closure; required 64e distribution | Selected MEA, membrane and integrated separation cases pass their configured scientific contracts |
| F05 | planned | F01 TSA/SOC cases; 64d heat/mass/charge closure; 64e cycle/local-domain contracts | Selected adsorption/electrochemical cases retain state continuity, cross-region balances and supported provider products |
| F06 | planned | F01 network cases; gas-property/pressure coupling and required 64e inventory | Selected gas networks and upstream transient-linepack configurations have complete boundary and inventory evidence |
| F07 | planned | Selected F02–F06 and their actual consumer implementations | Finite configured family coverage, negative controls, retained readback and public claims reconcile with 64h |

## Work packages and consequential investigations

### Current checkpoint — 2026-11-03

All F scope is authorized with D/E/G. The H01 required-process crosswalk identifies distinct
steam/power, gas-solid, capture/membrane, TSA/SOC and network populations and their dependency
owners. Exact source inspection distinguishes required TSA adsorbent/steam/compressor variants
and bed direction/reaction variants. F02–F07 implementation and configuration-specific native/
reference evidence remain open; generic unit or solver availability supplies no specialist
family acceptance.

### F01 — Resolve configurations and scientific dependency closures

For each required variant, freeze the reference revision, physical model, data, active/fixed/free roles, boundaries, requested outputs and comparison assumptions. Distinguish an existing native implementation, incomplete authored science, missing data, generic semantic gap and unsupported numerical route. Record the decision and its affected consumer before selecting an adapter or language/schema change.

Do not aggregate all configurations into one ready/not-ready flag. Select useful vertical increments within each family, but preserve remaining mandatory variants at this owner. F01 produces the scientific prerequisites and branch decisions that make F02–F06 executable; it does not complete the scientific work by listing it.

### F02 — Power and steam systems

Deliver selected steam-cycle, supercritical-plant and NGCC configurations through existing pressure/heat/work and costing compositions. Confirm pure-fluid liquid/vapor/saturation/supercritical conventions, phase presentation, enthalpy/entropy datum, required derivatives and pressure/geometry closure. A steam property evaluator alone does not establish a turbine or complete plant.

Investigate whether each unit consumes ordinary property values, equilibrium/flash behavior or a detailed local heat-transfer model; that decision determines the P2/P4 prerequisites. Preserve turndown and operating-case conditions rather than validating only a nominal point. Match physical outputs, work/heat balances and assessment currency/period/source meaning; a different equation of state is separately evaluated changed science.

### F03 — Gas-solid and chemical looping

Author separate gas/solid states and heterogeneous reaction/extent bases for the selected fixed, moving and bubbling-bed variants. Retain phase-specific transport, interphase heat/mass exchange, solid inventory and reaction-energy conventions. Internal transfers cancel once in enclosing balances; apparent species or empirical solids cannot acquire exact elemental guarantees without the admitted composition evidence.

Resolve whether each selected spatial/transient formulation fits 64e's actual accumulation and numerical contracts. Transfer coefficients, hydrodynamics and closure correlations carry their own validity/data records. Qualify released oxidation/reduction configurations and limiting cases, not merely the generic bed shell.

### F04 — Capture and separation

Close electrolyte/true-apparent species, phase-equilibrium, reaction, transport and rate-based transfer dependencies for MEA absorber/stripper and selected integrated capture cases. Keep true/apparent mappings, charge/speciation and reaction heat from being counted twice. Direct state junctions and explicit translators retain different contracts; incompatible state/material conventions require the latter's declared preserved quantities and information loss.

Membrane cases retain their actual permeation laws, pressure coupling, species selection and configuration assumptions. Resolve equilibrium versus rate-based requirements before choosing a reusable contactor interface. Compare operating-region and configuration evidence, including missing directional parameters and unsupported phase combinations, rather than inferring full support from one converged column.

### F05 — Adsorption and electrochemistry

TSA delivery consumes the implemented 64e operating-cycle contract: retained inventories, active connections, heat input, pressurization, collection windows and cycle closure. Study orchestration cannot substitute for physical carryover. Initialization failure must retain its full report and original specification as required by the source review F01.

Solid oxide cell delivery may expose detailed local two-dimensional thermal/electrochemical physics through stable process-facing boundaries. Identify fuel/oxygen/material, electrical and thermal conventions and cross-region source balances. A reduced or external replacement is substitutable only for demanded outputs, derivatives, validity and effects; equal ports do not establish equal internal observables. Select detailed versus reduced reference correspondence explicitly.

### F06 — Gas networks and distribution

Deliver released compressor, node and pipeline configurations with exact pressure-flow, gas-property and boundary semantics. Preserve coupling across the network rather than treating material-flow direction as numerical solve order. Transient linepack requires actual inventory laws, consistent initial/boundary conditions and selected spatial/temporal representation from 64e.

Determine whether bidirectional flow is part of the chosen upstream case. Negative flow alone does not provide upstream composition/enthalpy or zero-flow mixing semantics; preserve the directed contract where sufficient and implement reversal only with its demanded transport meaning and controls. Retain steady versus transient and physical versus discretization identities in output comparison.

### F07 — Integrate configured family evidence

Exercise complete selected native/reference engineering cases through ordinary authoring, preparation, execution and retained result inspection. Preserve initialization and producer-owned result permission; diagnosis-only products remain available without manufactured physical candidates. Wrong-basis, datum, missing-output, ordering, transfer-sign and deliberate wrong-value controls must challenge the relevant comparison or boundary.

Retained evidence identifies all configured science and actual route products. A subset can enable a 64g consumer once accepted, while the remaining family obligations stay open. Remove replaced mechanisms and callers only after replacement checks and consumer migration; never retain a second production path as historical evidence.

## Verification and completion

**Proposed verification:** scientific owners supply independent property/reaction/constitutive checks, equipment owners supply original balances and specification controls, and configured flowsheets supply matched independent-reference evidence. Reference reproduction and intentional scientific improvement have separate evidence. No shared native calculation may manufacture both the claimed result and its independent expectation.

During implementation, use targeted owner compile/tests under repository execution rules. Join configured family/native/reference journeys at the selected integration boundary with 64h; consume applicable existing Plan53/58h obligations without changing their status. Performance measurements are needed only for stated benefit claims and must retain scientific-quality, configuration and workload conditions.

**Outcome: Proposed.** Authoring establishes no native family completion, solver result, parity pass or measured benefit. F01–F07 are planned. Completion requires every mandatory configured P5 row to have complete native behavior, supported products, matched scientific correspondence and retained exclusions; neither a unit-name inventory nor a single integration witness closes the family population.
