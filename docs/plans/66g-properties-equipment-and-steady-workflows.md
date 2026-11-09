---
title: "Plan66g: Properties, equipment and steady workflows"
status: in-progress
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/66-integrated-simulation-target.md
doc_topics: [current-work, design, validation]
---

# Properties, equipment and steady workflows

## Responsibility and implemented baseline

Plan66 is the sole go-forward successor. This companion owns the remaining authored
property, reaction, transport and generic-equipment changes, their complete steady
consumers, and their receiving evidence. The [coordinator](66-integrated-simulation-target.md)
owns combined decisions; [66l](66l-required-population-and-assembled-acceptance.md)
owns the exact required population and assembled qualification. Prior package labels
are provenance, not prerequisites or instructions to reconstruct delivered mechanisms.

**Interface-checked baseline:** main `66341514d`. Existing authored thermodynamics and
process packages, scientific selection, physical specialization, native solvers and
scoped result owners remain the foundation. Selected ordinary native/reference controls
have previously run; they do not establish the entire configured population. Original
fixture inspection source is integrated. Its changed adapters/readers need receiving
evidence, not another implementation of original-input observation. Preserved FeedFlash
source and controls in the science worktree are candidates requiring a coherent source,
generation and acceptance decision before cutover; they are not an assembled delivery.

Relevant source owners are [thermodynamic states](../../packages/reference/thermodynamics/models/state-coordinates.pse),
[equilibrium](../../packages/reference/thermodynamics/models/equilibrium.pse),
[reactions](../../packages/reference/thermodynamics/models/reactions.pse),
[process models](../../packages/reference/process/models/), and
[selection closure](../../crates/pse-modeling/src/scientific_selection.rs).
The target follows review I01/I02/I03/I10 and scientific analysis S01–S04/S07 in the
[combined review](../plan_improvement_reviews/plan64-65-integrated-target-and-library-leverage-2026-11-06.md).
It implements physical operations, not a parallel science catalogue or generic unit dispatcher.

## Target and finite scientific scope

One admitted authored model should serve simulation, rating/design and downstream
analysis without changing material identity, reference state or interpretation at a
client boundary. State coordinates, equation topology, numerical solve order and
thermodynamic phase/root selection remain distinct. A local EOS root is not a stable
equilibrium certificate. An unavailable method or derivative is explicitly unavailable.

The following population must reach 66l's case/configuration/input closure. It is not
a Cartesian product: effective defaults, package capabilities and construction guards
determine admissible combinations. Unavailable required ordinary data remains open.

| Scientific population | Required distinctions and input closure |
|---|---|
| Modular state coordinates | FTPx, FcTP, FPhx, FcPh, FpcTP and FpTPxpc demands; defined/receiving states, material membership and phase allocation; zero flow/disappearing phases, bounds and reconstruction/advected enthalpy correspondence |
| Ideal, cubic and activity models | Selected BT/ASU/hydrocarbon/CO2-water/ionic-liquid mixtures; PR/SRK options and interaction data; Ideal/NRTL/Wilson activity demands and reference conventions; equilibrium, bubble/dew and Henry methods with actual validity and derivative products |
| Pure-fluid and caloric science | Selected water/steam, CO2 and general Helmholtz fluids; PH/PS/PU/TPX, liquid/vapor/mixed/saturation/supercritical regions and mass/mole conventions; exact ideal/caloric contributions, datum and registered scientific data |
| Component and mixture methods | Constant, NIST, Perry and RPP3/4/5 caloric inputs; selected Chapman–Enskog, Chung, Eucken, Wilke and WMS transport consumers; phase/mixture rules and explicit no-method behavior |
| Electrolytes and reactions | NaCl/KCl/mixed-solvent and MEA consumers; true/apparent mappings, charge/speciation and reference conventions; Arrhenius/power-law rates, constant/van't Hoff/Gibbs equilibrium, log/solubility forms, reaction extent/basis and formation-versus-sensible energy |
| Boundary/representation equipment | Feed/product, state junction, stream scaler, translator, skeleton/custom unit and isothermal/isenthalpic feed flash; independently selected inlet/outlet packages and declared preserved/lost quantities; supported steady/no-holdup restrictions |
| Mixing/separation/flash | Inlet selection, energy and pressure mixing alternatives; phase/inherent reactions; total/phase/component/phase-component or ideal splitting, energy split/equal-temperature/equal-enthalpy behavior, pressure/heat choices and property-dependent admissibility |
| Heating/pressure/valves | Heater modes; isothermal/isentropic/pump/adiabatic, compression/expansion and performance curves; linear/equal-percentage/quick-opening laws with curve provenance, pressure-flow closure and work sign |
| Exchangers | 0D co/counter/crossflow, distinct side packages, LMTD/smoothed/AMTD/Underwood methods, rating/design and NTU; 1D/shell-and-tube geometry, finite difference/collocation and side pressure/phase choices; wall storage and dynamic allocation continue in 66h |
| Reactors/contacting/solids | Stoichiometric, equilibrium, Gibbs/inert, kinetic CSTR/PFR, pipe and multistream contacting; per-stream direction, feeds/side streams, interacting map and heterogeneous reactions; solid/liquid separation and thickener settling laws, geometry, recovery and distinct solid/liquid bases |
| Complete steady workflows and extension | HDA modular/legacy, flash/distillation and simulation/design/costing roles; methanol with/without recycle; custom compressor/heater and aqueous/organic extraction; actual helper/data closures and original scientific coefficients |

The inherited source census at `git:66341514d:docs/plans/64h-required-process-population.md`
provides selectors and detailed input roles to transfer into 66l. General Helmholtz fluid
registration includes the required butane, CO2, water, isobutane, ammonia, propane and
selected refrigerant cases; a parameter JSON alone does not provide executable fluid or
transport support. Keep conditional supplemental source decisions at 66l rather than
silently expanding or shrinking this target.

## Native realization and shared contracts

[66a](66a-scientific-operations-and-product-semantics.md) owns operation/product meaning,
original coordinate and current permission. [66d](66d-mathematical-representation-and-numerical-execution.md)
owns numerical representation and provider capability. Consume only the slice a case
needs; independent ready property/equipment work does not wait for all science.
The selected CoolProp 8 HEOS Water realization remains the existing ADR-0238 route:
the full `coolprop-sys` C ABI with attempt-owned state, explicit loaded-library/data
admission, coherent enthalpy/entropy and demanded homogeneous partials. Follow the
[primary low-level contract](https://coolprop.org/coolprop/LowLevelAPI.html); do not
equate native properties with a qualified pump/turbine or silently extend wet-state
derivative claims. Compare teqp/FeOS or authored alternatives only where a demanded
remaining envelope needs them, including ideal/caloric completeness and phase selection.

Symbolica 3.0.1 multi-output evaluators, coefficient precision remapping and exported
instructions can share value/derivative preparation under 66d. Exact elimination must
retain domain/nonzero guards and original coordinates; it does not establish global
phase stability. Field-granular [Salsa preparation](../../crates/pse-compiler/src/workspace/modeling/granular.rs)
already exists: separate source/structure/value/precision invalidation instead of
creating another property cache. Native array batches may use ndarray 0.16.1 views/Zip
and faer 0.24.4 borrowed matrices; neither array position nor a numeric batch replaces
physical identity. Specific mechanisms and limits are in
[S02–S04](../plan_improvement_reviews/plan64-65-scientific-extensibility-analysis-2026-11-06.md).

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| G01 | planned | 66l's selected scientific/configuration/input identity; existing source admission | Every required property/equipment row has a concrete existing/missing science, ordinary-data, numerical-product or qualification disposition and a consuming case |
| G02 | in-progress | G01 selected state/property closure; 66a coordinate/product and 66d provider slice where changed | The prepared CoolProp ABI passed selected endpoint and bounded-refusal controls on main. The pure-Water saturation registration adds bounded pressure endpoint values and analytic first/second pressure derivatives through the same image and datum; receiving evidence and other state/phase scope remain open. |
| G03 | in-progress | G01 selected chemistry/transport inputs; native physical membership and basis | The pure ethane Chapman–Enskog producer passed its authored compiler control. Required mixture transport, reaction and electrolyte/speciation increments still need conserved quantities, validity, derivatives and independent consuming expectations. |
| G04 | in-progress | Relevant G02/G03 slices; generated/adopted operations from 66b | Bounded FeedFlash source, complete-data reference fixtures and two published ordinary-BT consumers are integrated. Original readback verifies separate unknown-Cp/Perry permissions and all four actual-flow exposure contexts, including an isolated original over-capacity refusal. Three homogeneous HEOS Water cases reach protected/portable consumers. The authored wet turbine selects the saturation registration for its actual and isentropic outlets; native receiving, wet delivery, other equipment, independent reference-binary and typed-client acceptance remain open. |
| G05 | planned | Working selected equipment/recycle/initialization closure | HDA, methanol and custom extension consumers deliver their distinct simulation/design outcomes with original scientific inputs and complete linked outputs |
| G06 | planned | Changed producers and 66b/66e/66f receiving consumers | Selected declarations, native clients, diagnostics, retained/exported readers and source/reference mappings migrate together; replaced adapters and fixtures are deleted |
| G07 | planned | Selected G02–G06 deliveries and 66l independent expectations | Configuration-specific missing acceptance is complete, required failures remain attributable, scope claims reconcile with 66l and enduring contracts are published |

### Selected pure ethane transport

The first G03 transport producer adds the nominal dynamic-viscosity quantity and
pascal-second unit at their existing physical owner. The authored Chapman–Enskog
method uses the existing component/phase property selection and `phased_parameter`
records, the independently retained species molar mass, and the selected positive
reduced-temperature law. The selected dilute-vapor permission is record-scoped;
unknown applicability does not imply general gas or dense-fluid validity.
The [authored control](../../packages/reference/seed-data/models/transport-fixtures.pse)
checks three handbook observations cited by the pinned IDAES ethane test and independently
compares first and second symbolic jets with temperature differences. **Tested, selected:**
default-profile, force-validated `scripts/pse test pse-compiler --lib -F force-validate`
passed this one authored control with zero failures and complete settlement
(`01a1bf6f-3bc9-7738-b192-3ade693c8d9d`). The [independent ethane driver](../../tests/reference/ethane-viscosity-reference.py)
evaluated the pinned IDAES 2.13.0 Chapman–Enskog/Neufeld expression at 320.15, 390.15 and
520.15 K. The separate native authored comparator passed under the default force-validated
`scripts/pse test pse-compiler pse-math pse-backend-native --lib` selection of the ethane and
math controls, with zero failures/warnings and complete settlement
(`01a1c12d-b299-73f5-82d6-b881a7f8a58f`). Negative receiving, typed result and actual
equipment consumers remain separate. No Wilke, Chung, Eucken or WMS claim follows.

G01 produces usable per-consumer closure decisions, not a serial barrier over the
entire table. Start with ready steady equipment and the HEOS Water equipment consumer
while independent chemistry and family inputs are resolved. G04 reviews candidate
FeedFlash source for the actual selected BT and water cases; completing an already
authored candidate means integrating and testing it, not rewriting it for a new ID.
G05 can proceed when its actual child equations and inputs work; it need not wait for
all electrolyte, solids or pure-fluid configurations.

### Selected Water protected-result delivery — November6

The original liquid-pump, vapor-turbine and hot-vapor-turbine fixtures now execute through
explicitly prepared HEOS Water registrations, retain native Step0/reference-point permission,
and preserve all ten original report rows through protected typed reading, bounded IPC export
and independent Arrow/Protobuf-context interpretation. The full population includes inherited
temperature, enthalpy, energy-flow and flow reports as well as work; filtering to the two work
rows would discard original evidence. Foreign revision selection refuses. See the shared
[`protected_steam_case`](../../crates/pse-runtime/src/workflow/modeling/steam_tests.rs) consumer.
This reuses the existing CoolProp low-level state/phase owner and native equipment equations;
it adds no provider factory, inferred EOS-name restoration or alternate production path.

**Tested, targeted in the owning checkout:** native `scripts/pse test pse-runtime --lib` with
the declared scientific/provider fixture closure selected the pump consumer (1/1) and both
vapor consumers (2/2); zero failures/build warnings and complete capture/settlement. Original
invocations are `01a1b33f-de92-710a-9bc0-7f8669ae8d40` and
`01a1b345-6dba-74c5-bd8a-4e1cc57bf7de`. Subsequent main receiving with `property-coolprop`
passed all three consumers in a selected nine-control Water/costing/worker run, zero failures,
pending lifetimes and compiler warnings, with complete capture/settlement. This is selected
main receiving, not assembled G acceptance. Static IAPWS/released-case expectations are independent stored
expectations, not fresh reference execution. The pump is the selected hydraulic approximation;
the existing homogeneous phase/datum/derivative limits remain. Wet, saturated, critical and
mixed-phase equipment are outside those three controls.

**Implemented, receiving pending:** the authored wet turbine explicitly selects a vapor inlet
and wet actual and reference outlets. The wet state uses pressure and molar quality, exposes
both Water phases, mixes enthalpy and entropy from endpoint differences on the common datum,
and computes volume flow from both endpoint densities. The existing turbine and pump equations
remain authoritative. The added fixture checks the released 400 K, 200 kPa inlet to 140 kPa case
against stored temperature, shaft work and wet quality, plus original entropy and phase,
component and caloric balances. The quality check distinguishes the wet outlet from quality one.
The rounded shaft-work observation uses the pinned reference test's stated one-percent
acceptance. The original-coordinate whole-turbine energy balance has its separate 0.01 W
allowance, and per-state phase energy projection uses the repository's nominal Power
resolution (1 W). The reference comparison and the local conservation checks answer
different questions.

[ADR-0238](../adr/0238-native-water-property-provider.md) supplies the binding decision.
The [CoolProp 8 adapter](../../crates/pse-backend-native/src/coolprop.rs) uses
`AbstractState_update` with `PQ_INPUTS` at quality zero and one, then
`AbstractState_first_saturation_deriv` and the selected patched
`AbstractState_second_saturation_deriv` on requested endpoint fields with pressure as the
coordinate. The [pinned HEOS source](https://github.com/CoolProp/CoolProp/tree/ae81610e7d23efc57f9d051c8e70a4d66e87537f)
and [maintained C ABI patch](../../vendor/coolprop-sys/pse-patch.md) identify the selected contract.
The adapter checks the 10 kPa–10 MPa interval, ordered densities, positive latent heat and
the common stock translation before releasing values or jets. The same authenticated image
and ideal-only stock reference serve homogeneous and saturated calls. The focused native and
authored controls do not establish wet protected/native-view/portable receiving or independent
reference execution.

**Tested, selected:** The three native ABI/registration controls and three authored dry
Water cases passed in the default-profile force-validated
[November 9 runtime selection](66-integrated-simulation-target.md#66ah-execution-checkpoint--november-9).
The wet fixture observes the original vapor fraction through the common outlet flow
contract. Its selected native comparison passed with zero failures and build warnings
and complete settlement (`01a1bf8c-3aa3-741b-ac71-14785594a48f`); this does not qualify
the separate protected or portable wet result.

The [typed engineering admission](66c-preparation-execution-and-service-lifecycles.md#packages)
now accepts explicit startup-authorized native registrations. The launched engineering client
still needs receiving evidence for an actual selected Water configuration, independently of
the authored fixture and native provider controls.

The ordinary FeedFlash consumer now also checks the existing source-owned applicability policy
explicitly: unknown-Cp family permission remains distinct from Perry record-scoped extrapolation,
and all four caloric/energy contexts retain their original source, actual flow dependency, unit and
frozen bound. A 1.0015 mol/s input exceeds the original 1.001 mol/s exposure bound while its
equimolar component deviation remains inside the unchanged 0.001 mol/s physical allowance.
All four exposure checks refuse, all other original checks pass, and failed typed readback retains
that exact outcome. This exercises the existing native assessment owner; no source coefficient,
permission, bound, comparator or production path was replaced. The original 2 mol/s negative also
failed legitimate authored component expectations; its failure is retained and the sharper input
isolates the intended guard. The positive two-mode consumer passed in that mixed selection, and
the corrected isolated negative passed 1/1 with zero failures/warnings and complete settlement
(`01a1b35a-f9b1-74b4-b8b7-5b7467b8821e`). Main's selected 23-control host/runtime receive
subsequently passed both ordinary FeedFlash controls with zero failures/compiler warnings and
complete capture/settlement. It does not supply independent reference execution or the complete
equipment/client population.

Each increment includes an operation request with exact selected definition,
specifications, scientific source/data revisions and requested products; its native
result must reach protected inspection, typed client and independent export through
[66b](66b-contracts-generation-and-interpretation.md),
[66e](66e-protected-data-and-result-delivery.md) and
[66f](66f-developer-operations-clients-and-evidence.md). A schema or scalar property
control alone is not that consumer closure. Retire a replaced mapping, generic numeric
fallback or fixture only after all its selected callers move and revealing controls pass.

## Verification and completion boundary

When G02/G04 consume [D's selected products or precision](66d-mathematical-representation-and-numerical-execution.md#symbolica-precision),
keep Water saturation endpoint jets on the pressure axis at fixed quality endpoints, separate
from homogeneous Water's temperature and molar-density coordinates. Test the composed wet
result against independent endpoint/Clapeyron, component, energy and entropy expectations;
finish the protected wet receiver. Ordinary BT keeps the selected charts, ordered provider
ports, regime-crossing refusal and original attempt. A protected BT read proves provenance,
not resolution of the historical derivative discrepancies. Use G03 pure-ethane viscosity as
a contrasting provider-free law; do not infer a native provider or broaden its validity.

**Planned verification:** use owner-local native correctness controls with explicit
Arrow force-validation and matching explicitly prepared provider profiles. Check state
round trips and original units/datum; physical component/element/energy closure; zero
flow/disappearing phase; wrong species ordering; bad interaction data; unsupported
method/derivative; infeasible initialization and validity boundaries. Match independent
expectations to selected physical inputs and formulations, retaining reference-only
data's role. Reference execution stays isolated at IDAES 2.13.0 and never supplies copied
code or product coefficients without separate ordinary-source admission.

Qualify equipment outputs and connected steady workflows, including rating/design freedoms,
not only isolated properties. A changed datum, phase convention or selected root must
be detected by the comparator. Generated/readback fidelity, current permission and
independent scientific agreement are separate assertions. 66l schedules assembled
campaigns at functional scope end. This document runs none and records no new Tested
or Measured claim. Completion requires all selected remaining cases and migrations,
not the existence of package names or passing previously delivered controls.
