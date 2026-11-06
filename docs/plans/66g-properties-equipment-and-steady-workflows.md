---
title: "Plan66g: Properties, equipment and steady workflows"
status: planned
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
| G02 | planned | G01 selected state/property closure; 66a coordinate/product and 66d provider slice where changed | Missing state/caloric/phase and pure-fluid outputs delivered through actual equipment consumers; existing implementations receive only missing evidence |
| G03 | planned | G01 selected chemistry/transport inputs; native physical membership and basis | Required transport, reaction and electrolyte/speciation increments operate with conserved quantities, validity, derivatives and independent consuming expectations |
| G04 | in-progress | Relevant G02/G03 slices; generated/adopted operations from 66b | Remaining generic equipment configurations and candidate FeedFlash closure delivered through admission, execution and exact protected read, preserving explicit unsupported cases |
| G05 | planned | Working selected equipment/recycle/initialization closure | HDA, methanol and custom extension consumers deliver their distinct simulation/design outcomes with original scientific inputs and complete linked outputs |
| G06 | planned | Changed producers and 66b/66e/66f receiving consumers | Selected declarations, native clients, diagnostics, retained/exported readers and source/reference mappings migrate together; replaced adapters and fixtures are deleted |
| G07 | planned | Selected G02–G06 deliveries and 66l independent expectations | Configuration-specific missing acceptance is complete, required failures remain attributable, scope claims reconcile with 66l and enduring contracts are published |

G01 produces usable per-consumer closure decisions, not a serial barrier over the
entire table. Start with ready steady equipment and the HEOS Water equipment consumer
while independent chemistry and family inputs are resolved. G04 reviews candidate
FeedFlash source for the actual selected BT and water cases; completing an already
authored candidate means integrating and testing it, not rewriting it for a new ID.
G05 can proceed when its actual child equations and inputs work; it need not wait for
all electrolyte, solids or pure-fluid configurations.

Each increment includes an operation request with exact selected definition,
specifications, scientific source/data revisions and requested products; its native
result must reach protected inspection, typed client and independent export through
[66b](66b-contracts-generation-and-interpretation.md),
[66e](66e-protected-data-and-result-delivery.md) and
[66f](66f-developer-operations-clients-and-evidence.md). A schema or scalar property
control alone is not that consumer closure. Retire a replaced mapping, generic numeric
fallback or fixture only after all its selected callers move and revealing controls pass.

## Verification and completion boundary

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
