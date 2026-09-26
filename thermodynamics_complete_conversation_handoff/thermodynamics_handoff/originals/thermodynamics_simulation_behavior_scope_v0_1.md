# Thermodynamics simulation-behavior scope baseline

**Document:** THERMO-SCOPE-001  
**Version:** 0.1  
**Date:** 25 September 2026  
**Project:** Process simulator  
**Stage:** Step 1, scope establishment  
**Status:** Proposed working baseline, not an implemented or validated capability statement

## 0. Executive decision

The thermodynamics subsystem will be scoped around **materials at process-model locations and the operations required to describe, transform, transfer and evaluate them**, not around a list of equations of state or the interface of a selected numerical library.

The target is broad engineering flowsheet coverage at explicitly declared fidelity. Conventional fluid processing is the first numerical acceptance profile. Multiple liquids, petroleum representations, chemistry, solids, nonequilibrium contacting and partial-property materials remain part of the intended broad simulator scope, not optional afterthoughts. More specialized state and characterization patterns receive explicit representability requirements without an immediate execution commitment.

This document establishes 34 scenario requirements, 24 cross-cutting behavior requirements and 13 concept obligations. It does not choose backends, internal classes, storage schemas, solvers, numerical algorithms or Rust libraries. No scenario has been executed or numerically validated in this work.

### User-derived constraints

The guiding requirements are breadth over frontier fidelity; DWSIM as the initial flowsheet-oriented reference; reusable thermodynamic capabilities; and a maintainable, extensible language-independent design that precedes implementation. The coverage profiles below are recommendations, not additional user-approved release commitments.

### Evidence and design status

Source-backed observations are cited with [S1]–[S10]. All scenario choices, profiles, acceptance rules and conceptual distinctions below are proposed design requirements. Documentation was consulted to check the scope, not to certify any candidate package's support. Numerical reference systems, exact operating points and tolerances must be pinned during validation-fixture development.

## 1. Define what “coverage” means

| Level | Meaning | Required evidence | What it does not establish |
|---|---|---|---|
| **R: Representable** | The architecture can express the relevant material, phases, state, constraints, exchanges and result semantics without false physical assumptions. | A successful domain-model and action walkthrough, including failure and change behavior. | That data, a numerical provider or a working unit model exists. |
| **E: Executable** | A specified combination of models, parameters, provider and process calculation can perform the requested operation. | An actual end-to-end run for a pinned fixture, with required outputs and diagnostics. | That the result is physically accurate or covers other mixtures or conditions. |
| **V: Validated** | That executable combination meets declared conservation, thermodynamic, reference-data and operational checks over a bounded scope. | A validation record with independent references where available, applicable ranges, tolerances and limitations. | Universal accuracy, global equilibrium guarantees or fitness for every plant. |

These are evidence levels, not a single product-wide checkbox. A package might be executable for a temperature–pressure phase split but not for an energy-constrained flash, or validated for a nonreactive liquid while merely representable for an electrolyte.

A coverage statement must identify **material representation + component/species set + method and parameter versions + phase/reaction policy + requested operation + operating envelope + relevant property/derivative demands + process assumptions**. It must not say only “supports NRTL,” “supports electrolytes,” or “supports a reactor.”

For this stage, all entries are **scope specified**. R is an obligation to be demonstrated by the later blueprint; E and V are not yet assessed.

## 2. Coverage profiles and fidelity policy

| Profile | Coverage commitment | Intended interpretation |
|---|---|---|
| **P1: Conventional flowsheet foundation** | Target R, E and V for representative steady-state fixtures. | An initial usable foundation, not yet evidence of full engineering breadth. |
| **P2: Broad process-engineering coverage** | Target R, E and V for selected, bounded representative cases. | Required before describing the simulator as meeting the intended broad scope. Provider/data feasibility must still be checked. |
| **P3: Explicit extension patterns** | Target R with meaningful information and operation semantics. E/V deferred. | Avoid architectural exclusion without promising immediate numerical support. |

Profiles organize evidence and implementation sequencing; they are not schedules. P1 plus P2 is the targeted general-purpose simulator scope. If library research reveals a gap in P2, record and resolve the gap or revise scope explicitly; do not silently relabel it as a niche case.

### Operating-envelope policy

Do not define a universal temperature, pressure or composition range for the architecture. Each executable/validated profile receives its own model/data/numerical envelope. Test ordinary interior points, boundary approaches, phase changes and out-of-envelope requests separately.

The design must not assume every fluid lies below its pure-component critical point, nor assume every equilibrium request has a vapor fraction. Specialized critical-region research and novel high-fidelity models are not initial acceptance drivers. Ordinary gas-processing and single-fluid states must not be excluded by an artificial “subcritical only” type boundary. CoolProp's state interface explicitly recognizes single-phase and supercritical region labels; this is a reminder not to hard-code a vapor/liquid dichotomy [S5].

### Fidelity policy

A supported lower-fidelity engineering model is acceptable when its assumptions, required data and limitations are explicit and it passes the declared fixture. Missing information must not become invented molecular detail or zeros. A new numerical solver may be tried without changing physics; a new physical model or excluded phase is a different configuration and must be recorded as such.

## 3. System boundary

This scope uses “thermodynamics” broadly enough to include thermophysical properties, phase behavior, material/species descriptions, compatible caloric data and the interface to reaction/speciation properties. It does not make every reaction or equipment model part of an equation of state.

| Responsibility | Inside the scope | Neighboring responsibility, not owned by thermodynamics |
|---|---|---|
| Material definitions | Identity, composition meaning, characterization, basis and applicable phase/species descriptions. | User-interface editing and enterprise master-data systems. |
| Thermophysical properties | Phase and material properties; compatible transport/interfacial properties; declared bulk property closures. | Geometry-dependent heat/mass transfer, hydraulic correlations and equipment sizing. |
| Equilibrium/state operations | State resolution, phase/reaction equilibrium where supported, saturation/stability requests and their constraints. | Selecting process operating pressures, duties, residence times or production targets. |
| Reaction coupling | Stoichiometry, conserved quantities, state-dependent equilibrium/rate-law inputs and caloric compatibility. | Reactor geometry, residence-time equations, transport limitations and process-control logic. |
| Material state | Stream, internal stage/segment and inventory descriptions; partial, trial and accepted states. | Flowsheet topology, connection graphics, global solve ordering and time integration. |
| Numerical integration boundary | Declare needed inputs, supported properties, derivative meaning and result/failure behavior. | Selecting the global flowsheet solver, sparse representation or code-generation technology. |
| Information governance | Parameter/reference versions, provenance, validity, estimated-data policies and stale-result meaning. | Storage technology, database schema, distributed infrastructure and application access control. |

DWSIM's educational documentation illustrates the flowsheet-level division between stream calculations and the specific balance/rule imposed by a mixer, valve, compressor, separator or exchanger [S2]. IDAES explicitly uses local property states inside unit models as well as shared material parameter definitions [S3]. Those references motivate scope boundaries; they do not prescribe our implementation.

### Required property breadth

The request universe includes phase identity/amount/composition; density/volume; enthalpy, entropy and internal energy; heat capacities where meaningful; fugacity, activity, chemical potential or equivalent equilibrium relationships; viscosity, thermal conductivity and diffusivity where demanded; and phase-pair interfacial properties. No single material configuration must provide every item. Partial support is a first-class legitimate outcome, not a broken package.

### Calculation-request coverage

These are requested behaviors, not a universal promise that every property package can perform every operation. Composition/material information, units, bases, phase policy and data requirements accompany every applicable request.

| Request | Scope target | Important qualification |
|---|---|---|
| Single-phase property evaluation at a specified local state | P1 and P2 | Does not automatically equilibrate a multiphase process location. |
| Temperature–pressure phase equilibrium | P1, extended to P2 materials/phases | A saturated pure-fluid state can remain underdetermined without another valid condition. |
| Pressure–specific-enthalpy state resolution | P1 and applicable P2 scenarios | The mass/molar basis and caloric reference convention are explicit; chemistry is fixed or enabled explicitly. |
| Pressure–specific-entropy state resolution | P1 compression/expansion and selected P2 cases | A temperature–pressure provider lacking the required entropy calculation does not satisfy this request. |
| Pressure–phase-fraction or temperature–phase-fraction state resolution | P1 pure-fluid and selected P2 mixture cases | The designated phase, fraction basis and denominator are explicit; multiple branches or missing extra phase allocations may require more information. |
| Bubble/dew endpoint or saturation-property request | P1 and selected P2 fixtures | Distinguish endpoint properties from a fully allocated two-phase material state. |
| Phase stability and constrained-equilibrium queries | Applicable P1/P2; specialized metastability in P3 | Record the assessed candidate set and strength of stability evidence; no universal global-minimum proof is implied. |
| Chemical-equilibrium/speciation request under declared constraints | P1 simple reaction and P2 reactive/electrolyte cases | Identify conserved quantities, admissible species, thermal constraints and any external material exchange. |
| Phase-specific transport or phase-pair property request | P1/P2 when the selected process model requires it | Requires appropriate property data and a defined phase or phase pair; effective multiphase transport needs an additional declared closure. |
| Derivative or sensitivity request | Required to be expressible; execution depends on the selected calculation profile | Identify coordinates, held-fixed quantities, phase/chemical response and derivative method/limitations. |
| Inventory state from total internal energy, total volume and material amounts | P3 | Total quantities are not sufficient without amount/composition or other independent constraints. |

### Numerical formulations

The domain must be compatible with sequential evaluation and with a coupled process formulation. It must not require all providers to emit symbolic equations, analytic derivatives, unrestricted flashes or the same internal primitives. Which providers can support which formulation belongs to later research.

## 4. Scenario catalog summary

A scenario is a behavior test, not a claimed plant design or universal numerical benchmark. “Candidate fixture” names indicate the kind of material needed; actual data, compositions, conditions and expected values remain to be pinned.

| ID | Family | Scenario | Profile | Target |
|---|---|---|---|---|
| SC-01 | F01 | Blending and splitting without reaction | P1 | R → E → V |
| SC-02 | F01 | Sensible heating and heat exchange | P1 | R → E → V |
| SC-03 | F01 | Liquid pumping and pressure-loss calculations | P1 | R → E → V |
| SC-04 | F01 | Gas compression, expansion and intercooling | P1 | R → E → V |
| SC-05 | F01 | Cooling and vapor–liquid separation | P1 | R → E → V |
| SC-06 | F01 | Pressure reduction with flashing | P1 | R → E → V |
| SC-07 | F02 | Conventional equilibrium-stage distillation | P1 | R → E → V |
| SC-08 | F02 | Nonideal-liquid separation and azeotropic behavior | P1 | R → E → V |
| SC-09 | F02 | Physical absorption, humidification and gas dissolution | P1 | R → E → V |
| SC-10 | F03 | Liquid–liquid extraction and decanting | P2 | R → E → V |
| SC-11 | F03 | Vapor–liquid–liquid separation | P2 | R → E → V |
| SC-12 | F04 | Water and steam through saturation | P1 | R → E → V |
| SC-13 | F04 | Pure-fluid refrigeration loop | P1 | R → E → V |
| SC-14 | F04 | Mixed-refrigerant phase change | P2 | R → E → V |
| SC-15 | F05 | Assay-derived petroleum pseudocomponents | P2 | R → E → V |
| SC-16 | F05 | Black-oil or other reduced petroleum representation | P2 | R → E → V |
| SC-17 | F06 | Specified-conversion reaction | P1 | R → E → V |
| SC-18 | F06 | Chemical-equilibrium reaction | P1 | R → E → V |
| SC-19 | F06 | Kinetically controlled reaction | P2 | R → E → V |
| SC-20 | F06 | Reactive separation | P2 | R → E → V |
| SC-21 | F07 | Electrolyte mixing and neutralization | P2 | R → E → V |
| SC-22 | F07 | Reactive gas absorption into aqueous liquid | P2 | R → E → V |
| SC-23 | F08 | Inert solids carried with fluid | P2 | R → E → V |
| SC-24 | F08 | Crystallization and precipitation | P2 | R → E → V |
| SC-25 | F08 | Gas–solid chemical transformation | P2 | R → E → V |
| SC-26 | F09 | Rate-based nonequilibrium contacting | P2 | R → E → V |
| SC-27 | F09 | Adsorption and membrane state extensions | P3 | R; E/V deferred |
| SC-28 | F10 | Heat exchange between different property packages | P1 | R → E → V |
| SC-29 | F10 | Material transfer across a property-package boundary | P2 | R → E → V |
| SC-30 | F10 | Translation between material representations | P2 | R → E → V |
| SC-31 | F11 | Mass-based empirical and nonconventional materials | P2 | R → E → V |
| SC-32 | F11 | Polymer distributions and material attributes | P3 | R; E/V deferred |
| SC-33 | F12 | Inventory-based state and dynamic compatibility | P3 | R; E/V deferred |
| SC-34 | F12 | Restricted-equilibrium and metastable-state requests | P3 | R; E/V deferred |

## 5. Detailed scenario cards

All acceptance statements below are obligations for later fixtures or architectural walkthroughs, not passed test results. Common numerical acceptance rules are in Section 8.

### F01. Conventional fluid processing

#### SC-01: Blending and splitting without reaction

**Profile:** P1  
**Flowsheet:** Two independently specified feeds → mixer → composition-preserving splitter.

**Engineering objective.** Combine and partition material without introducing hidden chemical or equilibrium assumptions.

**Specified information and assumptions.** Compatible material identities; independent feed states; explicit flow bases; outlet pressure rule and heat/work assumptions supplied by the unit model.

**Required behavior.** Resolve the outlet thermal state from the specified balance constraints. Distinguish a mechanical split from an equilibrium separation. Permit a property request independent of total flow.

**Acceptance witness.** Preserve each nonreacting component and the declared energy balance. An ideal mechanical splitter preserves intensive state and composition while changing flow. Recombining its products reproduces the original balance state within fixture tolerances.

**Required failure/limitation behavior.** Reject incompatible component mappings or enthalpy conventions rather than silently averaging temperatures or dropping components.

**Concept obligations.** Material identity, intensive state, extensive flow, state specification, unit-versus-property responsibility.  
**Cross-cutting tests:** X01, X03, X11, X14, X23.

#### SC-02: Sensible heating and heat exchange

**Profile:** P1  
**Flowsheet:** Single-phase liquid or gas → heater/cooler; repeat as one side of an exchanger.

**Engineering objective.** Support both a specified outlet temperature with computed duty and a specified duty with computed outlet state.

**Specified information and assumptions.** Feed state and amount flow; pressure or pressure-drop rule; one thermal specification; a compatible caloric method.

**Required behavior.** Provide enthalpy and heat capacity where required. For equipment rating, provide the requested conductivity and viscosity separately; the unit supplies geometry and heat-transfer correlations.

**Acceptance witness.** Temperature-specified and duty-specified forms agree on the same selected branch. Exchanger-side duties reconcile with the external heat-loss convention. A missing rating property does not invalidate an otherwise supported duty-only calculation.

**Required failure/limitation behavior.** Reject an over-specified thermal request, or identify the unsupported rating property without inventing it.

**Concept obligations.** Caloric properties, inverse requests, conditional transport demand, heat coupling.  
**Cross-cutting tests:** X02, X09, X19, X22.

#### SC-03: Liquid pumping and pressure-loss calculations

**Profile:** P1  
**Flowsheet:** Liquid feed → pump → pipe segment → outlet.

**Engineering objective.** Use liquid thermophysical properties in pressure-changing equipment without moving equipment physics into thermodynamics.

**Specified information and assumptions.** Liquid state; flow basis; pump efficiency or work relationship; pipe geometry and hydraulic closure supplied by the unit.

**Required behavior.** Supply density, enthalpy and viscosity; supply compressibility or additional properties only when the selected unit formulation requires them. Report the actual phase policy and validity.

**Acceptance witness.** The selected pump energy relation and pipe pressure-loss relation close using properties at declared evaluation locations. A phase change is detected or explicitly excluded under a declared approximation.

**Required failure/limitation behavior.** Do not continue a liquid-only hydraulic calculation as though vapor formation were impossible; return a policy-appropriate diagnostic.

**Concept obligations.** Single-phase evaluation, pressure dependence, property demand, phase admissibility.  
**Cross-cutting tests:** X07, X09, X10, X14.

#### SC-04: Gas compression, expansion and intercooling

**Profile:** P1  
**Flowsheet:** Gas feed → compressor/expander → intercooler → optional subsequent stage.

**Engineering objective.** Support ordinary pressure-changing gas equipment and its ideal reference state.

**Specified information and assumptions.** Feed state; outlet pressure; unit-selected efficiency/work relation; declared phase policy.

**Required behavior.** Support the pressure–entropy reference-state calculation and the actual energy-constrained outlet. Keep the ideal reference calculation distinct from the actual equipment result.

**Acceptance witness.** Reference and actual states remain separately identifiable; the actual material and energy balances close; a downstream phase change is handled by the appropriate request rather than hidden in a gas label.

**Required failure/limitation behavior.** A provider lacking a usable entropy method cannot claim this scenario merely because it supports temperature–pressure flashes.

**Concept obligations.** Entropy, caloric consistency, intermediate states, independent unit assumptions.  
**Cross-cutting tests:** X09, X10, X14, X19.

#### SC-05: Cooling and vapor–liquid separation

**Profile:** P1  
**Flowsheet:** Multicomponent hydrocarbon feed → cooler → equilibrium separator → vapor and liquid products.

**Engineering objective.** Establish the basic multicomponent phase-split behavior used by a flowsheet.

**Specified information and assumptions.** Feed composition and flow; separator pressure and thermal specification; permitted phases and a pinned property configuration.

**Required behavior.** Determine phase amounts and phase compositions; evaluate properties required for balances; distinguish material redistribution from chemical reaction.

**Acceptance witness.** Products reconstruct the feed component totals, satisfy the specified thermal condition, and satisfy equilibrium conditions for permitted transferable components. Stability status is reported relative to the assessed candidate phase set.

**Required failure/limitation behavior.** Do not report an unverified unstable or restricted-phase solution as unrestricted stable equilibrium.

**Concept obligations.** Overall versus phase composition, candidate versus present phases, equilibrium scope, stability evidence.  
**Cross-cutting tests:** X04, X05, X06, X07.

#### SC-06: Pressure reduction with flashing

**Profile:** P1  
**Flowsheet:** Pressurized feed → adiabatic valve → downstream equilibrium state.

**Engineering objective.** Support energy-constrained state resolution when pressure changes and phases may appear.

**Specified information and assumptions.** Feed material and state; outlet pressure; valve assumptions excluding shaft work and material kinetic/potential energy changes for the reference fixture.

**Required behavior.** Resolve a pressure–enthalpy outlet request with an explicit enthalpy basis and reference convention. Preserve the input accepted state during unsuccessful trials.

**Acceptance witness.** Component totals and the specified enthalpy condition are satisfied. Re-evaluation at the solved state reproduces the specified enthalpy within its fixture tolerance.

**Required failure/limitation behavior.** An infeasible request, unsupported specification, ambiguous branch or numerical failure must remain distinguishable.

**Concept obligations.** Inverse state request, phase transition, caloric basis, structured failure.  
**Cross-cutting tests:** X02, X05, X06, X23.

### F02. Mixture behavior and staged separation

#### SC-07: Conventional equilibrium-stage distillation

**Profile:** P1  
**Flowsheet:** Feed → multistage column with condenser and reboiler → products.

**Engineering objective.** Demonstrate that thermodynamic states exist inside equipment, not only on flowsheet connections.

**Specified information and assumptions.** Material definition; stage locations; pressure profile and column specifications owned by the unit; compatible phase and caloric methods.

**Required behavior.** Evaluate separate vapor/liquid states and equilibrium relations at each stage. Provide the same physical relationships to repeated numerical evaluation or a coupled process formulation without prescribing the global solver.

**Acceptance witness.** Stage and whole-column balances close; phase relations are satisfied at accepted stages; trial states are not exposed as converged products. Removing a required property identifies the affected stage or operation.

**Required failure/limitation behavior.** Local property success must not imply stage, column or flowsheet convergence.

**Concept obligations.** Internal material locations, shared definitions, equilibrium constraints, nested convergence.  
**Cross-cutting tests:** X14, X15, X19, X23.

#### SC-08: Nonideal-liquid separation and azeotropic behavior

**Profile:** P1  
**Flowsheet:** Polar liquid feed → flash or equilibrium-stage separation; alcohol/water is a candidate fixture family.

**Engineering objective.** Prevent a hydrocarbon-only model architecture while keeping phase behavior and caloric behavior explicitly compatible.

**Specified information and assumptions.** Liquid nonideality method, vapor-phase convention, compatible standard states, required interaction data, and a fixture with independently characterized phase behavior.

**Required behavior.** Represent a configured vapor/liquid method combination without treating every method as the same kind of equation of state. Resolve the declared equilibrium and heat duties.

**Acceptance witness.** The chosen fixture reproduces its reference phase topology and separation limitation within stated data tolerances. A converged but qualitatively wrong phase diagram does not pass.

**Required failure/limitation behavior.** Missing interaction parameters, inappropriate method combinations and unsupported extrapolation remain visible; a numerical fallback must not silently replace the physical model.

**Concept obligations.** Model composition, standard states, parameter completeness, qualitative phase behavior.  
**Cross-cutting tests:** X06, X08, X10, X11.

#### SC-09: Physical absorption, humidification and gas dissolution

**Profile:** P1  
**Flowsheet:** Gas and liquid feeds → nonreactive contact stage → gas and liquid outlets.

**Engineering objective.** Cover dissolved gases and water transfer without making every gas–liquid operation chemically reactive.

**Specified information and assumptions.** Distinct phase compositions; declared transferable components; appropriate phase partition convention; gas reporting basis such as wet gas, dry gas or per unit dry carrier where used.

**Required behavior.** Evaluate physical partitioning and phase-specific caloric properties. Preserve phase eligibility and any nontransferring carrier assumptions.

**Acceptance witness.** Transferred amounts and energy reconcile across the unit. Wet/dry reporting conversions preserve actual material quantities. No reaction occurs without a declared reaction model.

**Required failure/limitation behavior.** An undefined Henry-law/reference convention or missing gas–solvent data must not be hidden as ideal partitioning.

**Concept obligations.** Transfer eligibility, dilute-solute conventions, phase composition, reporting basis.  
**Cross-cutting tests:** X01, X08, X11, X21.

### F03. Multiple liquid phases

#### SC-10: Liquid–liquid extraction and decanting

**Profile:** P2  
**Flowsheet:** Feed and solvent → contacting stage → decanter → two liquid products.

**Engineering objective.** Require multiple instances of one phase category and phase-pair-specific behavior.

**Specified information and assumptions.** A documented partially miscible system; compatible parameters; permitted liquid phases; temperature, pressure and material amounts.

**Required behavior.** Determine distinct liquid phase compositions and amounts. Associate an interfacial property with a particular phase pair when requested. Do not use output-array position as physical identity.

**Acceptance witness.** Both liquid products have distinct meaningful identities, reconstruct the feed, and satisfy the selected equilibrium checks. Reordering provider outputs does not change physical results.

**Required failure/limitation behavior.** A method unable to represent the required liquid split must be rejected or declared unsupported, not patched by duplicating one liquid phase.

**Concept obligations.** Phase category versus phase instance, phase matching, pair-specific properties.  
**Cross-cutting tests:** X05, X06, X07, X22.

#### SC-11: Vapor–liquid–liquid separation

**Profile:** P2  
**Flowsheet:** Gas/organic/aqueous feed → three-phase separator → gas and two liquid outlets.

**Engineering objective.** Exercise simultaneous competing phase allocations rather than separate unrelated binary flashes.

**Specified information and assumptions.** A documented three-phase fixture; candidate phase set; material composition and thermal/mechanical specifications.

**Required behavior.** Allocate all components over a shared equilibrium problem or a validated equivalent solution procedure. Identify absent and present phases explicitly.

**Acceptance witness.** All three products reconcile material and energy. The appropriate interphase equilibrium conditions hold for the permitted transfers; phase appearance/disappearance is well defined at adjacent test points.

**Required failure/limitation behavior.** A two-phase-only provider cannot claim success by ignoring an allowed aqueous or organic liquid phase.

**Concept obligations.** Multiphase scope, global component accounting, phase-state identity.  
**Cross-cutting tests:** X04, X05, X06, X07.

### F04. Water, steam and refrigeration

#### SC-12: Water and steam through saturation

**Profile:** P1  
**Flowsheet:** Condensate → pump → boiler → expansion or heat delivery → condenser.

**Engineering objective.** Support a specialized pure-fluid configuration while exercising saturation and thermal balances.

**Specified information and assumptions.** Pure water identity; compatible package; subcooled, saturated, two-phase and superheated fixture states; explicit phase-fraction basis.

**Required behavior.** Distinguish saturated-liquid/vapor endpoint queries, a complete two-phase state and an underdetermined request. Retain the same reference convention around the loop.

**Acceptance witness.** Pressure and saturation temperature alone do not invent an intermediate phase fraction. An additional valid specification resolves it. Loop material and energy balances close.

**Required failure/limitation behavior.** Ambiguous saturation inputs receive an underdetermination diagnostic, not an arbitrary phase selection presented as a unique solution.

**Concept obligations.** Pure-fluid saturation, specification independence, quality basis, specialized packages.  
**Cross-cutting tests:** X02, X05, X11, X19.

#### SC-13: Pure-fluid refrigeration loop

**Profile:** P1  
**Flowsheet:** Evaporator → compressor → condenser → expansion valve → evaporator.

**Engineering objective.** Combine specialized fluid properties with the same flowsheet request semantics used elsewhere.

**Specified information and assumptions.** Selected pure working fluid; pressure levels; thermal boundary conditions and compressor relation; consistent reference conventions.

**Required behavior.** Support evaporation/condensation, pressure–enthalpy and pressure–entropy requests, and the separate unit energy accounting.

**Acceptance witness.** Every link conserves material, individual unit balances close, and loop heat/work terms reconcile. No provider-specific fluid name becomes the authoritative material identity.

**Required failure/limitation behavior.** A supported pure-fluid loop must not be advertised as support for arbitrary mixtures of that fluid.

**Concept obligations.** Specialized package, fluid identity mapping, loop integration.  
**Cross-cutting tests:** X01, X11, X15, X17.

#### SC-14: Mixed-refrigerant phase change

**Profile:** P2  
**Flowsheet:** Specified refrigerant blend → evaporating/condensing loop or exchanger section.

**Engineering objective.** Prevent pure-fluid quality or saturation assumptions from defining all mixture behavior.

**Specified information and assumptions.** Blend composition, actual component identities or explicitly declared pseudo-pure approximation, candidate phases, and mixture-capable data.

**Required behavior.** Represent different liquid and vapor compositions and distinct saturation endpoints where the selected fixture requires them. Distinguish mole-based from mass-based phase fraction.

**Acceptance witness.** The mixture fixture preserves composition through the loop and matches reference endpoint/temperature-glide behavior within its validation scope. A pseudo-pure approximation is identified as a separate configuration.

**Required failure/limitation behavior.** Do not interpret a mixture as a pure fluid merely to obtain a convenient flash call.

**Concept obligations.** Pure versus pseudo-pure versus true mixture, saturation endpoints, phase-fraction basis.  
**Cross-cutting tests:** X01, X02, X05, X11.

### F05. Petroleum characterization and reduced models

#### SC-15: Assay-derived petroleum pseudocomponents

**Profile:** P2  
**Flowsheet:** Assay or boiling-curve description → characterization → blend/heating/separation flowsheet.

**Engineering objective.** Treat characterized materials as reproducible model inputs rather than unidentified chemicals.

**Specified information and assumptions.** Original assay, stated test/basis conventions, cut definitions, characterization methods, estimated properties and provenance.

**Required behavior.** Create a reproducible component representation and use it in selected thermal/separation calculations. Retain the link to the originating assay and all estimation assumptions.

**Acceptance witness.** The characterization reconciles the declared assay quantities within agreed tolerances. Saving/reloading preserves the cut definitions and property values. Changing characterization creates a distinguishable configuration and invalidates dependent results.

**Required failure/limitation behavior.** Do not invent a registry identity, atom balance or exact molecular structure for a cut; do not claim support for an uncharacterized assay.

**Concept obligations.** Characterization, pseudocomponent identity, estimation, versioning.  
**Cross-cutting tests:** X08, X10, X16, X17, X21.

#### SC-16: Black-oil or other reduced petroleum representation

**Profile:** P2  
**Flowsheet:** Bulk oil/gas/water description → pressure/temperature changes → supported surface-separation calculation.

**Engineering objective.** Permit a useful engineering representation without falsely promising molecular-resolution equilibrium.

**Specified information and assumptions.** Model-specific bulk quantities; standard-condition conventions; empirical correlations; declared supported outputs and operating envelope.

**Required behavior.** Keep this representation distinct from an assay-characterized compositional mixture. Expose only calculations justified by its data and correlations.

**Acceptance witness.** Reported standard/actual quantities and phase partitions follow the selected empirical model. Unsupported chemical potentials or detailed-species requests remain unsupported.

**Required failure/limitation behavior.** No silent conversion from bulk oil characterization to an invented detailed molecular composition.

**Concept obligations.** Reduced representation, limited capability, reference conditions, explicit translation.  
**Cross-cutting tests:** X09, X10, X13, X21.

### F06. Reaction-coupled processing

#### SC-17: Specified-conversion reaction

**Profile:** P1  
**Flowsheet:** Reactant feeds → conversion reactor → cooling/separation.

**Engineering objective.** Separate process-imposed reaction extent from equilibrium prediction.

**Specified information and assumptions.** Stoichiometry, selected conversion or extent, phase participation, feed composition and an explicit heat-accounting convention.

**Required behavior.** Calculate resulting amounts and thermal requirements under the unit-imposed reaction extent. Treat conversion as an input assumption, not a computed equilibrium state.

**Acceptance witness.** Elemental and total mass balances close when defined; stoichiometric consumption is feasible; reaction heat is included exactly once under the chosen energy formulation. Total molecular moles are not incorrectly required to be conserved.

**Required failure/limitation behavior.** Reject impossible conversion or unbalanced specified chemistry rather than clipping negative product quantities.

**Concept obligations.** Reaction definition, extent, conservation basis, formation-property conventions.  
**Cross-cutting tests:** X04, X08, X11, X12.

#### SC-18: Chemical-equilibrium reaction

**Profile:** P1  
**Flowsheet:** Specified reactant mixture → isothermal or energy-constrained equilibrium reactor → outlet.

**Engineering objective.** Support changes in chemical species in addition to redistribution among phases.

**Specified information and assumptions.** Admissible species, conserved quantities, reaction-equilibrium or chemical-potential formulation, phase policy, and thermal constraints.

**Required behavior.** Resolve equilibrium under declared closed/open-system conditions. Keep phase equilibrium, chemical equilibrium and reaction inhibition separately identifiable.

**Acceptance witness.** Conserved elements and charge, where applicable, reconcile; admissible equilibrium conditions hold; the thermal constraint is satisfied. Restricting the species or phase set changes the declared problem rather than silently changing the solver.

**Required failure/limitation behavior.** A vapor–liquid flash with fixed chemical species totals is not accepted as chemical-equilibrium capability.

**Concept obligations.** Species versus components, reaction constraints, conserved quantities, thermal equilibrium coupling.  
**Cross-cutting tests:** X07, X11, X12, X20.

#### SC-19: Kinetically controlled reaction

**Profile:** P2  
**Flowsheet:** Feed → stirred or distributed reactor → outlet.

**Engineering objective.** Allow finite-rate chemistry to consume thermophysical properties without the property service imposing equilibrium.

**Specified information and assumptions.** Rate law, basis and units, participating phases, material state, and unit-owned residence time/geometry/transport assumptions.

**Required behavior.** Supply activities, concentrations or other declared rate-law inputs, as well as compatible thermal properties. Evaluate local states at required internal locations.

**Acceptance witness.** Rate and property conventions agree; the accepted reactor solution closes the relevant balances. Changing residence time changes the unit solution without changing the identity of the property package.

**Required failure/limitation behavior.** Do not replace a finite-rate reaction by equilibrium because that is the only supported backend operation.

**Concept obligations.** Kinetic properties, reaction–state coupling, local states, unit ownership.  
**Cross-cutting tests:** X09, X12, X14, X19.

#### SC-20: Reactive separation

**Profile:** P2  
**Flowsheet:** Reacting liquid/vapor system → reactive contacting or equilibrium-stage separation → products.

**Engineering objective.** Test that chemical and phase transformations can coexist in one local material description.

**Specified information and assumptions.** A documented compatible reacting-mixture fixture; declared reaction mode, stage assumptions, phase-transfer eligibility and energy conventions.

**Required behavior.** Represent simultaneous or explicitly approximated reaction/phase-equilibrium coupling. Keep the closure choice visible and compatible with the unit formulation.

**Acceptance witness.** Local and overall balances and the selected reaction/phase conditions hold together. Separate standalone successes are insufficient when their coupled result violates either set of conditions.

**Required failure/limitation behavior.** Do not silently sequence incompatible chemistry and flash models and label the result fully coupled equilibrium.

**Concept obligations.** Coupled chemistry and phase behavior, local closure, compatible methods.  
**Cross-cutting tests:** X07, X11, X12, X14.

### F07. Electrolytes and aqueous chemistry

#### SC-21: Electrolyte mixing and neutralization

**Profile:** P2  
**Flowsheet:** Acid/base/salt feeds → mixer or neutralizer → aqueous outlet.

**Engineering objective.** Support reporting composition and internal species composition as different legitimate representations.

**Specified information and assumptions.** Apparent feed description, true species definition, charge and elemental information, thermodynamic data and reference conventions.

**Required behavior.** Resolve speciation under the declared constraints; expose reporting quantities without summing apparent and true species as separate physical inventories.

**Acceptance witness.** Conserved quantities and electroneutrality, where imposed, hold. Apparent/true conversions do not double count mass. Reported pH and other quantities retain their activity/convention definitions.

**Required failure/limitation behavior.** Inconsistent charge or incomplete reaction/species data produces a specific diagnostic rather than an unexplained normalization.

**Concept obligations.** Apparent versus true species, charge, speciation, reporting maps.  
**Cross-cutting tests:** X01, X08, X11, X12, X13.

#### SC-22: Reactive gas absorption into aqueous liquid

**Profile:** P2  
**Flowsheet:** Gas feed plus aqueous solvent → absorber → gas and loaded-liquid products.

**Engineering objective.** Combine gas transfer, dissolved-species chemistry and heat effects in a flowsheet context.

**Specified information and assumptions.** Gas and aqueous definitions, speciation/reaction rules, transfer conventions, thermal properties and a chosen equilibrium-stage or rate-based unit closure.

**Required behavior.** Account for gas entering or leaving the liquid and for changes in dissolved species. Keep gas reservoirs or added titrants explicit when constrained chemistry requires them.

**Acceptance witness.** Element and energy transfers reconcile across gas and aqueous descriptions. Any imposed pH/fugacity condition that requires external material exchange reports that exchange.

**Required failure/limitation behavior.** A speciation-only result with no defensible caloric method does not qualify as an energy-balanced absorber calculation.

**Concept obligations.** Reactive phase transfer, open-system constraints, energy completeness.  
**Cross-cutting tests:** X09, X11, X12, X13, X20.

### F08. Solids-containing processing

#### SC-23: Inert solids carried with fluid

**Profile:** P2  
**Flowsheet:** Liquid/gas plus inert solids → heating/transport → mechanical separation.

**Engineering objective.** Represent a solid-bearing material without forcing the solids into a chemical-equilibrium calculation.

**Specified information and assumptions.** Solid identities or empirical material descriptors, amounts, phase-specific caloric/density data, participation policy and optional particle attributes.

**Required behavior.** Carry the solids through balances; include their thermal contribution when required; distinguish mechanical separation from precipitation. Let equipment models own settling/filtering closures.

**Acceptance witness.** Solid and fluid quantities reconcile. Equilibrium-inactive solids do not disappear. A mixture viscosity or other effective property is returned only under a declared bulk closure.

**Required failure/limitation behavior.** Do not assign a fictitious vapor pressure or molar basis to an empirical solid merely to fit a fluid API.

**Concept obligations.** Phase participation, non-equilibrium solids, particle attributes, bulk closures.  
**Cross-cutting tests:** X01, X07, X09, X22.

#### SC-24: Crystallization and precipitation

**Profile:** P2  
**Flowsheet:** Solution → cooling/evaporation or reagent addition → crystal-bearing material → separation.

**Engineering objective.** Distinguish equilibrium solid formation from inert-solid transport and from crystal growth kinetics.

**Specified information and assumptions.** Dissolved species/components, admissible solid identities, compatible solution/solid data, thermal constraints and phase policy.

**Required behavior.** Determine equilibrium solid quantities or report the selected constrained calculation. Keep particle-size distribution and nucleation/growth modeling external unless explicitly added.

**Acceptance witness.** Conserved quantities reconcile; present phases satisfy the relevant equilibrium tests; absent allowed phases satisfy the declared stability/saturation checks or are marked unchecked. Thermal quantities include the selected phase-change/chemical convention once.

**Required failure/limitation behavior.** Do not treat a saturation calculation alone as a prediction of crystal size or kinetic yield.

**Concept obligations.** Equilibrium solids, phase onset, identity of solid forms, optional kinetic attributes.  
**Cross-cutting tests:** X05, X07, X11, X12, X22.

#### SC-25: Gas–solid chemical transformation

**Profile:** P2  
**Flowsheet:** Gas and reactive solid → contactor/reactor → gas and solid products.

**Engineering objective.** Ensure reactive solids are not represented as inert carrier mass or dissolved fluid by default.

**Specified information and assumptions.** Solid reactant/product identities, gas species, reaction mode, compatible solid/gas caloric data and unit-owned contact/transport assumptions.

**Required behavior.** Evaluate the requested reaction-equilibrium or kinetic property relationships for the declared phases. Permit unreacted and product solids to coexist as distinct entities.

**Acceptance witness.** Conserved quantities and energy reconcile across gas and solids. The calculation distinguishes equilibrium restrictions from finite-rate limits.

**Required failure/limitation behavior.** Unsupported solid-solution or polymorph behavior remains explicit rather than being approximated as an ideal liquid without authorization.

**Concept obligations.** Reactive solids, multiple solid identities, heterogeneous chemistry.  
**Cross-cutting tests:** X07, X08, X11, X12.

### F09. Nonequilibrium contacting and surface extensions

#### SC-26: Rate-based nonequilibrium contacting

**Profile:** P2  
**Flowsheet:** Gas/liquid feeds → spatially resolved contactor → separate outlets.

**Engineering objective.** Support phase-specific bulk states and local interface relationships without automatically equilibrating the entire unit.

**Specified information and assumptions.** Separate phase temperatures and compositions where the chosen unit requires them; pressure/mechanical convention; transfer and interface assumptions.

**Required behavior.** Evaluate each bulk phase independently, plus interfacial equilibrium or driving-force properties as requested. The unit supplies interfacial area and transfer coefficients or correlations.

**Acceptance witness.** A nonequilibrium bulk input remains nonequilibrium unless the unit explicitly imposes a closure. Interphase mass/energy fluxes reconcile across the unit. Required diffusivity and interfacial properties are capability-checked.

**Required failure/limitation behavior.** An equilibrium flash is not substituted for a rate-based contactor or used to overwrite its phase states.

**Concept obligations.** Multiple local states, interface versus bulk, transport properties versus transfer closures.  
**Cross-cutting tests:** X07, X09, X14, X19, X22.

#### SC-27: Adsorption and membrane state extensions

**Profile:** P3  
**Flowsheet:** Bulk feed → adsorber or membrane contacting location → separate material domains.

**Engineering objective.** Reserve a concrete extension path for surface-bound material and selective transfer.

**Specified information and assumptions.** Bulk material states; optional adsorbed/surface inventory, loading basis and site definition; membrane-side identities and applicable chemical-potential conventions.

**Required behavior.** Represent relevant bulk and nonbulk states without labeling every surface inventory as an ordinary liquid phase. Keep isotherms, permeation laws and equipment transport relationships explicitly assigned.

**Acceptance witness.** A conceptual walkthrough preserves material between bulk, permeate/retentate and optional surface storage. Unsupported surface or electrochemical requests are expressible and explicitly rejected.

**Required failure/limitation behavior.** A generic untyped metadata attachment is insufficient evidence that this scenario is representable.

**Concept obligations.** Surface inventory, generalized material domains, selective transfer extension.  
**Cross-cutting tests:** X07, X13, X14, X22.

### F10. Heterogeneous thermodynamic regions

#### SC-28: Heat exchange between different property packages

**Profile:** P1  
**Flowsheet:** Process-fluid circuit ↔ heat exchanger ↔ independent water/refrigerant circuit.

**Engineering objective.** Allow different thermodynamic configurations in one flowsheet without implying material translation.

**Specified information and assumptions.** Independent material definitions and packages per circuit; separate reference conventions; declared heat-loss/work conventions.

**Required behavior.** Compute each circuit’s enthalpy changes independently and couple them through heat transfer. Do not require common component sets or equal numerical enthalpy zeroes.

**Acceptance witness.** Energy transferred from one side reconciles with energy received by the other and any stated losses. Components never cross the wall in this scenario.

**Required failure/limitation behavior.** Do not invoke a chemical component translator merely because two connected equipment sides use different packages.

**Concept obligations.** Thermodynamic regions, heat versus material coupling, local reference invariance.  
**Cross-cutting tests:** X01, X11, X14, X18.

#### SC-29: Material transfer across a property-package boundary

**Profile:** P2  
**Flowsheet:** Material stream described by package A → explicit boundary treatment → region using package B.

**Engineering objective.** Handle different models for the same physical material without a hidden energy or identity discontinuity.

**Specified information and assumptions.** Common or mapped material identities; both package conventions; chosen quantities to preserve; known reference offsets and explicit discrepancy policy.

**Required behavior.** Separate a pure reference-state conversion from disagreement between physical models. Define which state is recomputed and under which constraints.

**Acceptance witness.** Conserved quantities and selected boundary constraints hold. Holding temperature and pressure reports any residual enthalpy-model mismatch; holding pressure and corrected enthalpy may require a recomputed temperature. No unexplained heat source closes the gap.

**Required failure/limitation behavior.** Do not promise simultaneous preservation of incompatible temperature, pressure, composition and enthalpy predictions from different models.

**Concept obligations.** Reference translation, model discrepancy, constraint selection, boundary ownership.  
**Cross-cutting tests:** X02, X11, X13, X20.

#### SC-30: Translation between material representations

**Profile:** P2  
**Flowsheet:** Detailed/apparent material description → explicit representation map → lumped/true-species description.

**Engineering objective.** Permit heterogeneous modeling fidelity while making information loss and conserved quantities explicit.

**Specified information and assumptions.** Source and target representations; mapping rules; feasible conserved quantities; assumptions needed for any reverse reconstruction.

**Required behavior.** Translate without treating an irreversible aggregation as a bijection. Preserve provenance and distinguish reporting conversion from physical re-equilibration.

**Acceptance witness.** The selected mass/element/charge or defined empirical balances reconcile. Round-trip equality is required only for reversible mappings; information lost by lumping is documented.

**Required failure/limitation behavior.** Missing information for unlumping is not synthesized as a unique composition.

**Concept obligations.** Representation mapping, conservation mapping, information loss, reversibility.  
**Cross-cutting tests:** X01, X12, X13, X17.

### F11. Empirical and distribution-described materials

#### SC-31: Mass-based empirical and nonconventional materials

**Profile:** P2  
**Flowsheet:** Assay-described biomass, sludge, polymer granules or other engineering material → heating/mixing/separation.

**Engineering objective.** Support useful limited-property process models without pretending every material is a known molecular mixture.

**Specified information and assumptions.** Declared mass basis; empirical density/heat-capacity/enthalpy relationships as available; composition or assay descriptors and their validity.

**Required behavior.** Answer only supported requests. Permit mass/energy calculations where justified; require additional characterization before molar, elemental or chemical-potential calculations.

**Acceptance witness.** A reference heat/mass-balance case runs without an invented molecular weight. A request requiring missing molecular information fails specifically. Optional empirical mixture rules are explicit.

**Required failure/limitation behavior.** Do not use arbitrary molecular weight, critical constants or zero-valued missing properties to satisfy a universal fluid contract.

**Concept obligations.** Partial material definition, mass-only basis, conditional properties, empirical closure.  
**Cross-cutting tests:** X01, X08, X09, X13, X22.

#### SC-32: Polymer distributions and material attributes

**Profile:** P3  
**Flowsheet:** Polymer/solvent feed → mixing/devolatilization location → material with retained distribution descriptors.

**Engineering objective.** Reserve support for materials whose characterization is not exhausted by one molecular identity and mole fraction.

**Specified information and assumptions.** A documented distribution or moment representation; basis; polymer/solvent descriptors; model-specific parameters and permitted reductions.

**Required behavior.** Associate distributions and derived averages with their material definition. Distinguish repeat units, molecules, segments and mass bases when relevant.

**Acceptance witness.** A conceptual state-and-transfer walkthrough preserves the declared distribution information or records an explicit reduction. Unsupported detailed rheology or polymer equilibrium is not implied.

**Required failure/limitation behavior.** A single arbitrary mean molecular weight is not accepted as a lossless representation of an entire distribution.

**Concept obligations.** Distribution-valued characterization, basis semantics, model-specific reduction.  
**Cross-cutting tests:** X01, X09, X13, X22.

### F12. Inventory and constrained-state extensions

#### SC-33: Inventory-based state and dynamic compatibility

**Profile:** P3  
**Flowsheet:** Vessel inventory at an identified time/location → material-state evaluation under volume/energy constraints.

**Engineering objective.** Keep future dynamics possible without including a time integrator in the present thermodynamics scope.

**Specified information and assumptions.** Component/species inventories or another declared material amount basis; total internal energy and total volume, or another admissible independent specification.

**Required behavior.** Represent inventory separately from flow. Permit a state request based on total quantities with the required amount information; the process model owns accumulation balances, temporal evolution and events.

**Acceptance witness.** A nonempty inventory does not require a fictitious flow. Total versus specific quantities are unambiguous. A zero inventory has an explicit policy rather than undefined intensive values disguised as a physical state.

**Required failure/limitation behavior.** Do not infer density from total volume alone without material amount or assume a steady-flow enthalpy request is always sufficient.

**Concept obligations.** Holdup, amount versus flow, internal energy, time/location identity.  
**Cross-cutting tests:** X01, X02, X03, X14, X24.

#### SC-34: Restricted-equilibrium and metastable-state requests

**Profile:** P3  
**Flowsheet:** Process location with an explicit phase or reaction restriction → constrained property/state evaluation.

**Engineering objective.** Separate user-imposed physical approximations from solver heuristics and global-equilibrium claims.

**Specified information and assumptions.** Named excluded phases/reactions or requested branch; rationale; validity restrictions; expected diagnostics.

**Required behavior.** Represent frozen chemistry, suppressed solids or a metastable branch where a provider supports it. Keep this distinct from an initial guess and from equilibrium over all allowed candidates.

**Acceptance witness.** The result retains its restriction label and cannot be reported as unrestricted equilibrium. Removing a restriction invalidates the prior result for the changed problem.

**Required failure/limitation behavior.** A fallback that disables a phase or reaction without changing the declared problem is forbidden.

**Concept obligations.** Constraint policy, metastability, model assumptions versus numerical guesses.  
**Cross-cutting tests:** X05, X06, X07, X16.

## 6. Cross-cutting simulation behavior

The scenarios do not define an unchecked Cartesian product of every fluid, model, phase and operation. These tests probe important combinations across scenarios. Each applies wherever its prerequisites are meaningful, and each fixture must list the applicable tests.

| ID | Behavior | Test stimulus | Required outcome |
|---|---|---|---|
| X01 | Units and material bases | Change units; convert mass/mole amounts only when molecular information exists; distinguish phase fraction, flow, inventory and composition bases. | Physical results are invariant under valid conversions; impossible conversions are rejected. Fractions always identify their denominator. |
| X02 | Specification completeness and independence | Test fully specified, under-specified, over-specified and mutually inconsistent requests, including pure-fluid saturation. | A completed state is not fabricated from dependent or missing information. A coupled process model may retain unresolved variables without labeling them a completed state. |
| X03 | Zero flow and empty inventory | Test zero stream flow with a retained specified composition, undefined composition at zero component flow, and an empty vessel separately. | No division by zero or arbitrary normalization occurs. Zero flow is not confused with zero material inventory. Retained intensives are marked as conventions when not physically determined. |
| X04 | Trace and absent components | Use zero and trace species; include products absent from a reactive feed; probe illegal negative accepted amounts. | Nonreacting missing components are not created; permitted reaction products may form. Numerical regularization is disclosed and does not change accepted conservation. |
| X05 | Phase appearance and identity | Move through phase boundaries; reorder returned phases; let a phase disappear. | Phase identity is not a positional array index. No composition is reported as an actual present-phase composition for an absent phase; any incipient/trial composition is labeled. |
| X06 | Branches and initial guesses | Repeat a request from different guesses; construct a known multiple-solution case. | Differences receive branch/stability diagnostics. Reproducibility is defined for a stated policy and tolerance, not an unjustified promise of universal uniqueness. |
| X07 | Equilibrium scope | Compare unrestricted candidate sets, phase-limited calculations, frozen reactions and independent bulk phases. | Reported equilibrium/stability claims match the actual assessed set and constraints. A numerical retry never silently changes physical assumptions. |
| X08 | Missing or changed data | Remove interaction/correlation data; substitute an approved estimate; replace a fitted parameter set. | Required gaps block affected operations or use an authorized visible estimate. Sources, assumptions and resulting invalidations are recorded. No invented universal zero parameter. |
| X09 | Partial property capability | Request a mass/energy calculation, then an entropy, diffusivity or chemical-potential calculation not supported by the same configuration. | A supported limited task remains usable. An unsupported task fails specifically. A value-only provider does not automatically claim derivative or caloric completeness. |
| X10 | Validity and extrapolation | Probe the interior, boundary and exterior of model, data and numerical operating envelopes separately. | Out-of-envelope use follows an explicit policy. Convergence does not imply physical validity; missing validity information is itself visible. |
| X11 | Reference and energy conventions | Change a pure enthalpy reference consistently; couple different packages; compare reaction heat accounting conventions. | Physical heat/work predictions remain consistent under legitimate reference changes. Physical model disagreement is not mislabeled as a reference offset. |
| X12 | Conservation and reaction accounting | Exercise nonreactive components, reactive elements/charge, empirical balance quantities and heat of reaction. | Conservation checks use the correct quantities; total molecular moles are not universally conserved. No double counting of reaction heat or apparent/true inventories. |
| X13 | Representation mapping | Rename/reorder compounds; aggregate detailed components; request a reverse mapping. | Identity changes are distinguished from physical conversion. Conserved quantities and information loss are specified; irreversible aggregation does not acquire a fictitious inverse. |
| X14 | Internal states and locations | Evaluate a stream, column stage, exchanger segment, reactor location and inventory using shared material definitions. | Thermodynamics is not restricted to top-level streams. Independent local states do not overwrite one another or require a flowsheet graphic. |
| X15 | Recycle and nested convergence | Close a recycle with phase changes; fail one unit after other units have converged. | Property, equilibrium, unit and flowsheet convergence are distinct. Intermediate iterates are not published as accepted whole-flowsheet results. |
| X16 | Change and invalidation | Edit composition, parameters, phase policy, model selection or characterization. | All affected results become stale; unaffected independent configurations remain isolated. Old accepted results can be retained with their original provenance, not labeled current. |
| X17 | Reproduction and saved cases | Save/reload, repeat a run, compare a changed provider or data version. | Required definitions, parameter values, versions and policies are recoverable. Numerical reproducibility uses explicit tolerances; historical results remain attributable to their configuration. |
| X18 | Independent calculations | Interleave two cases with different compositions, parameters and reference conventions. | One case cannot contaminate another through hidden mutable configuration. Isolation can be achieved by the implementation without assuming a provider is inherently thread-safe. |
| X19 | Derivative meaning and solution integration | Request a phase derivative, a saturation-path derivative, a derivative after re-equilibration, and a parameter derivative where needed. | Each request specifies coordinates, held-fixed quantities and validity. Unsupported or nonsmooth derivatives are reported, not invented; all providers need not emit equations or analytic derivatives. |
| X20 | Inverse design and constrained chemistry | Let a host model solve for a duty/pressure or impose a chemical constraint requiring material exchange. | The division between specified inputs and unknowns is explicit. Required reservoirs, titrants and energy exchanges are accounted for; trial infeasibility does not corrupt an accepted state. |
| X21 | Reporting and reference volumes | Switch actual/standard gas volume, wet/dry composition, assay fraction basis or carrier-normalized concentration. | Reference temperature, pressure, basis and conversion convention accompany derived quantities. “Standard volume” is never a free-standing physical amount. |
| X22 | Effective multiphase properties | Ask for slurry viscosity, mixed-phase density/heat capacity, interfacial tension or effective conductivity. | An effective bulk property requires a declared physical/mixing closure. A phase-pair property identifies its pair; geometry-dependent transport is not disguised as equilibrium thermodynamics. |
| X23 | Failure, interruption and partial results | Force an unsupported request, nonconvergence, cancellation or a failed optional property after a successful primary calculation. | Failure categories differ. Results are qualified per operation/property; primary accepted results are not destroyed by an optional-property failure or replaced by trial values. |
| X24 | Temporal and signed-flow compatibility | Represent state at a time point, inventory accumulation context and a reversed connection direction. | Direction belongs to the flow/connection convention; physical amounts and compositions remain meaningful. Time integration and flow-network solution remain outside thermodynamics. |

### Source checks informing these requirements

Pure-fluid temperature–pressure inputs on saturation do not, by themselves, select a unique liquid/vapor allocation; CoolProp documents saturation ambiguity and separate quality-based requests. Its documented derivative operations also have state-specific restrictions, and its enthalpy/entropy conventions are explicitly reference dependent [S5].

IDAES provides a translator framework between property packages and requires application-specific mapping constraints rather than supplying a universal automatic conversion [S4]. Reaktoro documents constrained chemical equilibrium and explicitly open systems, supporting the need to expose external substance exchange in a constrained chemistry request [S6].

ChEDL separates property data from algorithms and warns that its collected data are not a critically evaluated recommendation set. Accordingly, data availability and physical validation must remain separate evidence claims [S7].

## 7. Integrated flowsheet acceptance journeys

These are integration witnesses. They supplement, rather than replace, the individual scenario cards and cross-cutting tests.

| ID | Proposed integrated journey | Primary coverage | Distinctive acceptance question |
|---|---|---|---|
| J01 | Gas mixing → compression → cooling → flash → pressure reduction; add a declared recycle. | SC-01, SC-04, SC-05, SC-06 | Can inverse states and phase changes coexist with recycle convergence and shared configuration without state contamination? |
| J02 | Polar-mixture column plus a separately justified extraction/decanter section and solvent recycle. | SC-07–SC-11, SC-26 | Can staged states, nonideal liquids and multiple liquid instances be modeled without treating an entire unit as one equilibrium state? The fixture must establish that the selected chemistry supports each section. |
| J03 | Water/steam or refrigerant utility circuit exchanging heat with a different process material. | SC-02, SC-03, SC-12–SC-14, SC-28 | Can independent packages close a heat balance without exchanging components or forcing a shared numerical enthalpy zero? |
| J04 | Reactant mixing → conversion/equilibrium/kinetic reactor variants → cooler/separator → recycle. | SC-17–SC-20 | Are reaction assumptions, conserved quantities, reaction heat and downstream phase equilibrium handled consistently? |
| J05 | Aqueous reagent mixing → reactive gas contact → precipitation → filtration. | SC-21–SC-24, SC-26, SC-30 | Can species accounting, material exchange, solids and energy remain consistent across different local chemical descriptions? |
| J06 | Assay characterization → blending/heating → compositional separation, with a separately declared reduced-model boundary test. | SC-15, SC-16, SC-29, SC-30 | Are characterization and any information-losing translation reproducible and physically qualified? |
| J07 | Empirical solid-bearing feed → heating → mechanical separation → optional compatible reactive-solid section. | SC-23, SC-25, SC-31 | Can the supported mass/energy calculation proceed without invented molecular properties, while the reactive extension requires the additional chemistry it actually needs? |
| J08 | Conceptual internal inventory/surface/polymer state → permitted transfer/change operation → retained result. | SC-27, SC-32–SC-34 | Can the later information model express the extension and its limits without an untyped metadata escape hatch? Numerical execution is not required by P3. |

For P1/P2, eventual validation must include both thermodynamics-level witnesses and at least one actual process integration witness for each included family. A standalone property call is not sufficient proof of a unit-operation scenario.

## 8. Acceptance rules and evidence requirements

### 8.1 Representability acceptance

A scenario passes R only when a later blueprint walkthrough can identify the relevant material, state and phase entities; express independent and unresolved specifications; distinguish equilibrium from process constraints; account for the correct conserved quantities; describe valid outputs and failures; and explain configuration changes and invalidation.

A universal `custom_data` field, arbitrary label or generic “external package” placeholder does not establish R unless the operation and conservation semantics are also specified. Conversely, R does not require committing to a concrete database schema or class layout.

### 8.2 Executability acceptance

An E record must contain a selected provider/build, actual component and parameter data, material definition, operation, phase/reaction policy, explicit inputs and unit assumptions, an observed successful run, the required outputs and diagnostics, and a limitation record. A calculator lacking one property required for the claimed unit is not executable for that unit, even when its standalone equilibrium call succeeds.

P1 and P2 are targets for publicly obtainable dependencies and data. A capability dependent on commercial, restricted or separately licensed resources must be labeled accordingly and cannot silently satisfy that target. DWSIM's current guide itself distinguishes some paid-edition methods, illustrating why a product feature list is not proof of public-core availability [S1]. Exact software/data terms belong to the later provider research, not a legal conclusion in this scope.

### 8.3 Numerical and physical validation

Every V fixture must define absolute and relative tolerances before its result is assessed. For any balance residual, compare the magnitude with an absolute floor plus a relative allowance based on a declared physical scale. Do not use a zero or nearly cancelling net quantity as the only scale, and do not inherit a pass criterion merely from a provider's default stopping tolerance.

Validation must cover four distinct questions:

1. **Conservation and state constraints:** relevant material balances, thermal constraints, correct bases, positive admissible amounts, valid fractions for present phases, and no forbidden species/phase creation.
2. **Thermodynamic consistency:** phase/chemical equilibrium conditions for the declared participating species and phases, applicable stability information, and consistency between forward and inverse requests on the same selected branch.
3. **Reference agreement:** selected experimental/reference cases with compatible definitions and known uncertainty, or an explicitly weaker implementation-verification claim where independent evidence is unavailable.
4. **Operational behavior:** repeatability, save/reload, change invalidation, phase transitions, missing data and controlled failure.

Numerical error and model/data error receive separate tolerances. There is no proposed universal percentage accuracy across all properties, fluids and conditions. Cross-library agreement alone is not independent validation when the methods or data share an origin.

For nonreactive molecular cases, conserve component quantities. For reactive cases, conserve relevant elements, mass and charge, including explicit exchanges. For empirical/lumped cases, check only the declared balances that their information supports. Do not assert an elemental closure when the material has no justified elemental characterization.

A converged result is not automatically a stable equilibrium or a physically accurate result. Record the candidate-phase/reaction scope and the strength of the stability checks actually performed. No general proof of a global minimum is required or implied for every provider.

### 8.4 Fixture dossier to be produced later

Each fixture will pin an identifier and version; scenario and cross-cutting IDs; actual substance/assay/species data; method/parameter versions; external dependencies; allowed phases and reactions; feed/initial data; independent specifications and unresolved variables; process assumptions; required primary and optional outputs; operating envelope; reference evidence and its uncertainty; numerical and physical tolerances; initial-guess/branch policy; conservation rules; expected failure behavior; and observed results.

This is a semantic checklist, not a storage schema. The fixture dossier is deliberately not completed with invented benchmark values at this scoping stage.

## 9. Concept coverage: obligations imposed on the later blueprint

These are necessary conceptual distinctions, not a predetermined architecture decomposition. Later research can reorganize them provided the scenarios remain expressible.

| ID | Concept obligation | Motivating scenarios | Minimum implication |
|---|---|---|---|
| C01 | Material identity and alternative representations | SC-01, SC-15, SC-16, SC-21, SC-30, SC-31 | Separate identified species, engineering components, apparent species and empirical lumps. |
| C02 | Amount, flow, inventory and basis | SC-01, SC-14, SC-23, SC-31, SC-33 | No universal molar requirement; no inventory-as-flow workaround; explicit phase-fraction denominators. |
| C03 | Partially specified and resolved local state | SC-04, SC-06, SC-07, SC-12, SC-26, SC-33 | Support intermediate/unknown states and internal locations without equating them to completed stream results. |
| C04 | Phase identity and participation | SC-05, SC-10, SC-11, SC-23, SC-24, SC-34 | Separate phase type, phase instance, eligibility, presence, restrictions and equilibrium participation. |
| C05 | Coherent configured property methods | SC-02, SC-08, SC-12, SC-16, SC-26 | Separate method compatibility from provider choice; do not require all packages to implement every property. |
| C06 | Data, characterization and provenance | SC-08, SC-15, SC-16, SC-21, SC-32 | Preserve source and fitted/estimated/user-supplied status; retain characterization lineage. |
| C07 | Caloric and reaction-reference conventions | SC-02, SC-04, SC-17, SC-18, SC-28, SC-29 | Represent energy consistently, including formation and reaction conventions and cross-package boundaries. |
| C08 | Equilibrium and chemical constraint scope | SC-05, SC-11, SC-18, SC-20, SC-22, SC-24, SC-34 | Distinguish phase equilibrium, chemical equilibrium, constrained equilibrium, open reservoirs and stability evidence. |
| C09 | Transport and interface property demand | SC-03, SC-09, SC-10, SC-23, SC-26, SC-27 | Separate thermophysical properties from equipment transport/rheology/population closures. |
| C10 | Thermodynamic regions and boundary treatment | SC-28, SC-29, SC-30 | Heat coupling, material transfer, reference conversion and representation translation are different actions. |
| C11 | Capability, validity and result quality | SC-04, SC-08, SC-16, SC-22, SC-31, SC-34 | Supported request, available data, numerical convergence and physical validity must be reported independently. |
| C12 | Calculation lifecycle and reproducibility | SC-06, SC-07, SC-13, SC-15, SC-29, SC-33 | Separate configuration, trial state, accepted result, stale result and reproducible historical result. |
| C13 | Surface and distribution extension information | SC-23, SC-27, SC-32 | Preserve meaningful nonbulk/material-characterization information without claiming the associated solver exists. |

All 34 scenarios have local concept obligations in their cards. All 13 cross-scenario obligations above are motivated by explicit scenarios. The later blueprint must add the reverse trace from its eventual entities/actions back to these obligations; that architecture-level trace cannot be claimed completed before those entities/actions exist.

## 10. Explicit exclusions and deferred commitments

### Excluded from this thermodynamics subsystem

Flowsheet graphics and editing; equipment geometry and mechanical design; plant economics; process-control execution; global flowsheet/time-integration algorithms; computational fluid dynamics; detailed particle population balances; and detailed multiphase hydrodynamic/transport closures are not owned by the thermodynamics subsystem. They may be consumers of thermodynamic information and can remain in the broader process-simulator roadmap.

### Not initial numerical acceptance drivers

New equation-of-state development; molecular simulation; frontier near-critical accuracy; universal cryogenic/extreme-pressure coverage; arbitrary solid polymorph/solid-solution equilibria; full polymer-distribution equilibrium and rheology; surface/electrochemical multi-field models; and universally differentiable phase transitions are not P1/P2-wide obligations.

This does not ban any ordinary case merely because an existing provider uses a sophisticated model to calculate it. For example, a specialized pure-fluid formulation is allowed when it is the practical way to cover an ordinary utility. Mainstream membrane/adsorption bulk-property needs remain compatible with the scope; the full surface/electrochemical machinery is a P3 extension rather than a mandatory general-fluid feature.

### Not promised by representability

Arbitrary numbers of phases are not an execution promise; the architecture should avoid fixed semantic slots while providers declare actual limits. No guarantee is made of complete data for every compound pair, automatic model selection, automatic reference reconciliation between arbitrary packages, a unique answer for every specification, or full CAPE-OPEN runtime compliance. Standards are design references at this stage [S8].

## 11. Working assumptions and decisions left open

| Item | Baseline assumption | What later work must resolve |
|---|---|---|
| Initial operation mode | Steady-state process cases first; inventories and temporal states remain representable. | Which dynamic/holdup requests earn E/V support and with which providers. |
| Fidelity | Useful declared engineering fidelity, not uniform high accuracy. | Per-fixture data and property tolerances. |
| Provider strategy | Multiple compatible providers may be used; none selected here. | Safe integration granularity, derivatives, statefulness, licensing and available data. |
| Property-package assignment | Different regions and internal locations may use appropriate configurations. | Defaulting/inheritance and validation rules, including material-transfer boundaries. |
| Data completeness | Partial and estimated data are allowed with explicit limitations. | Approved estimation/override/fallback policies and provenance. |
| Chemistry | Reaction/property interoperability belongs in scope; reactor closure stays external. | Which chemistry/speciation models supply complete required energy and transport properties. |
| Global solve style | Both evaluation-based and coupled process formulations must remain possible. | Provider-specific formulation and derivative contracts. |
| Exact benchmark systems | Candidate families are specified, not numerical values. | Pinned reference sources, compositions, conditions and expected results. |
| Performance | Isolation, repeatability and controlled failure are in scope. | Throughput/latency targets informed by representative workloads. |

## 12. Step-1 completion and handoff

**Completed in this baseline:** the coverage vocabulary; intended P1/P2 breadth; explicit P3 extensions; a 34-scenario catalog with assumptions, required behavior, acceptance witnesses and failure expectations; 24 cross-cutting behaviors; system boundaries; integrated acceptance journeys; concept obligations; and exclusions/open assumptions.

**Not performed:** full DWSIM source reverse-engineering; comparative library capability certification; parameter-database assessment; executable probes; numerical validation; final domain entities/action contracts; storage schemas; or Rust design.

The next investigation can use this scope as a checklist: for each DWSIM workflow it traces, identify the scenario and cross-cutting behaviors it covers, where the behavior resides, which assumptions are implicit, and what must be retained or changed. It should not start by assigning a backend to every row.

The substantive success criterion is: **a future simulator can tell the user what material is being modeled, which physical assumptions govern it, which requested calculations are supported, what information is missing, and what the resulting numbers do and do not establish, across the intended breadth of flowsheet cases.**

## Sources consulted for scope grounding

Accessed 25 September 2026. These sources support the limited observations explicitly attributed above, not a capability certification of this proposed simulator. URLs are retained for reproducibility; the `stable` and other moving documentation paths should be pinned during detailed provider research.

[S1] DWSIM, *Property Packages Guide*. Conventional system categories and an explicit Plus-edition annotation. `https://dwsim.org/tutorials/en/reference/property-packages-guide.html`

[S2] DWSIM, *Streams, State and Balances*. Flowsheet/stream and unit-operation context. `https://dwsim.org/tutorials/en/fundamentals/01-streams-state-and-balances.html`

[S3] IDAES, *Property Package*. Local state blocks, shared physical parameters, reaction properties and conditional property metadata. `https://idaes-pse.readthedocs.io/en/stable/explanations/components/property_package/index.html`

[S4] IDAES, *Translator Block*. Application-specific mapping between different package states. `https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/translator.html`

[S5] CoolProp, *High-Level Interface*. State/saturation inputs, phase labels, derivative restrictions and reference-state conventions. `https://coolprop.org/coolprop/HighLevelAPI.html`

[S6] Reaktoro, *Chemical equilibrium with constraints*. General equilibrium constraints and open-system behavior. `https://reaktoro.org/tutorials/equilibrium/equilibrium-specifying-constraints.html`

[S7] ChEDL Thermo, *Introduction to ChemicalConstantsPackage and PropertyCorrelationsPackage*. Data/model separation and caveats about collected data. `https://thermo.readthedocs.io/chemical_package_tutorial.html`

[S8] CO-LaN, *Business interface specifications* and *Thermodynamics and Physical Properties interface specification v1.1*. Implementation-neutral reference specifications and scope of thermodynamic interoperability. `https://www.colan.org/category/specifications/businessinterfacespecifications/` and `https://www.colan.org/specifications/thermodynamics-and-physical-properties-interface-specification-v1-1/`

[S9] DWSIM, *Reaction Systems*. Distinct specified-conversion, equilibrium and Gibbs reactor examples; contextual grounding for SC-17/18 rather than a prescribed provider or model. `https://dwsim.org/tutorials/en/intermediate/03-reaction-systems.html`

[S10] CO-LaN, *Petroleum Fractions*. Contextual grounding for characterization as a distinct modeling concern. `https://www.colan.org/specifications/petroleum-fractions/`
