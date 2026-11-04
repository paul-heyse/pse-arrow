---
title: "Plan64f: Specialized process families"
status: in-progress
date: 2026-11-02
adrs: [ADR-0238]
review_sources: [docs/design_review/reviews/design_review_process_simulator_target_2026-11-02.md]
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [current-work, design, validation]
---

# Specialized process families

## Proto integration consumer obligations

Family-specific clients consume [64i](64i-engineering-contracts-and-validation.md) projections
and [64j](64j-engineering-services-and-clients.md) typed operations without a parallel science
catalogue or execution engine. F02 supplies the selected steam/equipment consumer for
[64k](64k-prepared-scientific-workers.md) same-science worker comparison; retain exact
implementation/data, physical convention, demand and explicit unsupported capability.
Other F tracks depend only on the provider/interface slices they actually use. Existing
finite configuration coverage and scientific acceptance remain required; a generated schema
does not establish a delivered process family. Replace affected mappings/callers/tests together.

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
| F02 | in-progress | F01 power cases; 64d pure-fluid/pressure/heat/costing closure; required 64e physics | Steam, supercritical and integrated power cases have matched formulations, ordinary data and complete selected outputs |
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
family acceptance. Proposed ADR-0238 selects the first steam/power dependency: an explicitly
prepared native HEOS Water provider joined to authored state/equipment definitions, retaining
the ideal-only stock reference convention. Provider and pump/turbine source integration is in
progress. Supercritical property controls and wet-state limitations do not qualify a complete
power configuration or remove the remaining finite F02 obligations.

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

**Outcome: Partial implementation.** F01/F02 remain in progress and F03–F07 remain
planned. No configured specialist family, matched parity population or measured benefit
is established by this wave.

### Stopped implementation checkpoint

**2026-11-03 — stopped by instruction. Implemented:** explicitly prepared HEOS Water
registration, source-selected homogeneous branches and stock-reference translation,
authored liquid pump and two single-phase turbine controls. The registered native
document-ingress control passed; the broader specialist/power population remains open.

Latest actual command: `scripts/pse test pse-runtime --lib --features property-coolprop,solver-kinsol,solver-ipopt -E 'test(steam_tests)'`,
through the explicitly prepared native solver/linear-algebra environment and correctness
route on local Linux. **Failed against the zero baseline: 1 passed, 3 failed**, invocation
`01a1a380-1f0b-76d3-bfa9-cf8194f196a6`, with complete workflow settlement:

| Failing test | Actual failure and remaining obligation |
| --- | --- |
| `authored_liquid_pump_matches_independent_hydraulic_power_and_iapws_pressure` | Package admission: `rpp4_oracle: package data document is not supplied`; neither the added wrong-CAS refusal nor positive pump solve was exercised. |
| `authored_steam_turbine_matches_released_single_phase_reference` | Same missing package-data document before the native equipment solve. |
| `authored_hot_steam_turbine_matches_second_released_single_phase_reference` | Same missing package-data document before the second native equipment solve. |

Earlier predicate failures exposed unsupported conditional/typed-literal expression
syntax. The preserved repair declares the exact CAS once as mandatory typed data on
the existing source identity and compares supported field expressions. The latest
admission failure prevents claiming that the identity refusal or equipment physics is
qualified. Resolve the actual source/data closure before interpreting solver results;
retain the independent expectations and precise Water admission requirement.

The earlier five provider controls and their final-rerun limitation are owned by
[64d](64d-scientific-assets-and-equipment.md#stopped-implementation-checkpoint).
Backend rerun, xtask provider/caller controls and broader F02–F07 native/reference
acceptance were not run. Clippy and other follow-ups were explicitly skipped under the
[coordinator checkpoint](64-process-simulator-target.md#d-through-g-implementation-checkpoint).

### Current steam integration checkpoint — 2026-11-04

Execution has resumed under the coordinator's complete-Plan64 authorization. F01/F02
remain in progress, F03–F07 planned. The earlier stopped run above is historical evidence.
The latest selected equipment run was
`scripts/pse test pse-runtime --lib --features engineering-service,native-study-tests,product-reader,property-coolprop,solver-kinsol -E 'test(/^workflow::modeling::steam_tests::authored_/)'`
on prepared local Linux through the native correctness route: **0 passed, 6 failed against
zero**, all before equipment science during duplicate Water CAS package admission.
This includes three direct pump/turbine cases and their three prepared-worker counterparts.

Main `165faf093` integrates the source correction: one authoritative typed Water CAS fact,
separate provider-source provenance and the required wrong-CAS refusal; nominal molar
`Flow / Density -> VolumeFlow` typing with checked original operands/basis; and concrete
hydraulic/isentropic steam assumptions without the redundant PressureChanger wrapper.
Fixture paths follow the actual concrete unit. Equations, balances, guards, tolerances and
independent reference expectations remain intact. The generic PressureChanger remains useful
for its separate authored scope.

The selected equipment root route is the existing Ipopt bounded-root implementation.
Authored two-sided bounds cannot be represented by KINSOL's shifted sign constraints;
removing bounds to fit that backend would change the scientific problem. Ipopt's existing
C interface receives lower/upper variable bounds through `CreateIpoptProblem`; see the
[Ipopt C interface](https://coin-or.github.io/Ipopt/INTERFACES.html#INTERFACE_C)
and native routing in `pse-backend-native/src/routing.rs`. The finite diagnostic-cell
allowance covers the actual 11-by-12 and 17-by-18 derivative populations without changing
the numerical perturbation or accuracy criterion. Worker preparation also carries the
actual authored registry instead of assuming the standard registry.

Fresh captured scientific inputs and the final six-case rerun remain required. The final
pre-integration rebuild was interrupted by storage exhaustion before test execution; the
integrated main selection is now running. Source correction is not a tested equipment
repair. [64k's checkpoint](64k-prepared-scientific-workers.md#current-implementation-checkpoint--2026-11-04)
owns worker transport/process evidence and remaining matched-placement acceptance.
Neither the passing water-property transport controls nor passive Proto metadata establish
configured power-family or broader specialist acceptance.
