---
title: Target design — models, control volumes and flowsheets
status: draft
date: 2026-09-26
parent: docs/plans/20-idaes-capability-target.md
review_sources: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md]
---

# Target design — models, control volumes and flowsheets

> **Superseded in part by [Plan 21](21-modeling-kernel.md) (2026-09-26).** Wherever this
> document assigns scientific concepts to code, the
> [modeling kernel architecture](21-modeling-kernel-architecture.md) governs. Those concepts
> are:
>
> - property kinds as a registry;
> - identity rules and closure algorithms in `pse-properties` or `pse-thermo`;
> - property-package, state-definition, costing-binding, preset or slot *types*.
>
> They are package data on the kernel's generic concepts: definitions, interfaces with default
> members, functions, sets and tables, accumulators, implicit blocks with realization policies,
> and annotations. The scientific content, coverage and scenarios here remain the target.

**Evidence level: Proposed.** This document is the target architecture for authoring and
composing process models at idaes-pse scope:

- process blocks and configuration;
- unit models, ports and arcs;
- control volumes 0D/1D and balances;
- reactions in units;
- distributed domains and dynamics;
- controllers;
- costing and surrogate embedding;
- the unit model libraries, including `models_extra`.

It is a companion to [Plan 20](20-idaes-capability-target.md). The property side is in the
[thermodynamics document](20-target-thermodynamics.md). IDAES behaviour is cited from the
`pyomo-and-solvers` skill's characterization of idaes-pse 2.13.0 (`skill:` =
`.codex/skills/pyomo-and-solvers/`). It is read for behaviour only and never copied.

## 1. What must be reproduced

- **Framework.**
  - `declare_process_block_class`, with 257 generated block classes in 2.13.0.
  - `ProcessBlockData` with `CONFIG` and imperative `build()`.
  - `FlowsheetBlock`: time domain, `dynamic` and `default_property_package` inheritance, and
    sub-flowsheets.
  - `UnitModelBlockData`: `add_inlet_port`/`add_outlet_port`, `_get_performance_contents` and
    stream tables.
  - Pyomo `Port`/`Arc`, with equality expansion and extensive splitting.
- **Control volumes** (`skill: show idaes-model:idaes.core.base.control_volume0d.ControlVolume0DBlock --view components`).
  - Material balances by `MaterialBalanceType`: componentPhase, componentTotal, elementTotal,
    total, none.
  - Energy balances by `EnergyBalanceType`: enthalpyTotal, enthalpyPhase, energyTotal,
    energyPhase, isothermal, none.
  - Momentum balances by `MomentumBalanceType`: pressureTotal, pressurePhase, momentumTotal,
    momentumPhase, none.
  - Terms: holdup and accumulation, rate/equilibrium/inherent reaction generation,
    phase-equilibrium generation, mass/heat/work/enthalpy transfer, pressure change, custom
    terms.
  - `ControlVolume1D` adds the area definition, flow direction and Pyomo DAE
    `transformation_method`/`transformation_scheme`/`finite_elements`/`collocation_points`.
  - Extended control volumes add enthalpy-transfer energy balances.
- **Core unit library.** 31 models in `idaes.models.unit_models`, each with CONFIG, a build
  trace, delegated control-volume construction, an initializer and a scaler, recorded as
  `idaes-model:` records.
- **Extended libraries.**
  - `models_extra`: 43 power-generation models (boilers, feedwater heaters, drums, the Helmholtz
    turbine/pump/valve family, SOC submodels), 9 column models, 4 gas-solid contactors plus their
    property and reaction packages, 3 gas-distribution models, the TSA fixed bed, and the CO2
    membrane.
  - Control: `PIDController`.
  - Costing: `FlowsheetCostingBlock`/`UnitModelCostingBlock`; SSLW with 13 `cost_*` methods
    and 18 enums; QGESS; power-plant costing; TSA costing.
- **Configuration through Python callbacks and user expressions.** Examples are
  `delta_temperature_callback`, the valve function and `pressure_flow_callback`,
  `isentropic_performance_curves`, the MSContactor stream and interaction dictionaries, and
  custom material and energy terms.
- **Embedded surrogates.** `SurrogateBlock` with ALAMO, PySMO, Keras/ONNX (OMLT).

The IDAES *design* elements not reproduced are:

- **Imperative construction by mutation.** `build()` runs `add_*` calls under config
  branches. The characterization records 5,032 construction events, each with its guard chain.
- **Behaviour chosen by Python callbacks.**
- **Class inheritance for variants.** `Turbine`/`Compressor`/`Pump` subclass
  `PressureChanger` and change defaults.
- **Flag inheritance by parent traversal.** `useDefault` resolves by walking parent blocks at
  build time.
- **Time discretization as a model transformation that rewrites the model in place.**

## 2. Design thesis

1. **Typed templates are the model.** This is D2, kept. Everything IDAES expresses by
   imperative construction becomes a template declaration evaluated once at specialization:
   guards over typed parameters *and bound package facts*, indexed children, delegated ports,
   expression symbols, law instances over contributions, and interface-typed submodel slots
   instead of callbacks. No unit model contains code.
2. **Control volumes are ordinary templates over one indexed law engine.** A unit adds
   contributions, not balances (D7/ADR-0010, extended from scalar laws to indexed subjects).
3. **Analysis mode is not model structure.** The same template serves steady state, dynamics
   (integrated or simultaneous), optimization, estimation and studies (PS-11). Holdup and
   accumulation are generated by the law engine for the selected mode. Continuous domains are
   discretized by a declared transformation whose choice is part of the analysis.
4. **Composition semantics live in a pure modeling crate.** Template specialization, law
   expansion, reaction projection, discretization and costing aggregation move out of the
   effectful runtime into `pse-modeling`, a Salsa-integrated pure crate. Edits re-specialize
   incrementally, and the mechanisms can be tested without the runtime (AP-01, AP-06, DP-09).

## 3. The template language (authoring surface)

### 3.1 What exists and what is missing

The current template relations (§10.5, §11, §12) already give typed parameters, features and
feature rules, compile-time guards, finite and ragged domains, equations, scalar laws, typed
ports and connections, and single children. The reference `pse.units` and `states` packages
declare the IDAES shapes but stop at the first refusal in lowering. The refusal sites are in
`crates/pse-runtime/src/workflow/composition/lower.rs`:

| Refused construct | Site |
|---|---|
| Package-typed parameter | :340 |
| Material-derived domains | :507 |
| Reference and expression symbols | :124 |
| Submodel multiplicity | :1438 |
| Delegated ports | :1564, which finds no local members |
| Indexed laws and default balances | :1267 |
| Derivative symbols | :839 |
| Continuous domains | :471 |

The target admits each of these through the mechanisms below.

### 3.2 Mechanisms

| Mechanism | Contract | Replaces (IDAES) |
|---|---|---|
| **Package-typed parameters** | A parameter whose type is a property-package or reaction-package *capability* (for example "a property package providing `enth_mol_phase` and a molar or mass flow basis"). Bound per instance or through a declared **scope default**: flowsheet `default_property_package` resolved statically and recorded in the specialization, never by runtime parent traversal | `property_package=useDefault`, `default_property_package` |
| **Package facts in guards** | Guards may read typed facts of bound packages: phase list, phase types, component list, equilibrium pairs, flow basis, inherent reactions, providable kinds. The facts are values of the admitted package (see the [thermodynamics document §4.5](20-target-thermodynamics.md)) | `len(b.phase_list) == 2`, `hasattr(pkg, "inherent_reaction_idx")` |
| **Material-derived domains** | Template domains bound to the package's phase, component, phase×component or element sets, keeping identity (species ID, phase ID) | `phase_list`, `component_list`, `phase_component_set` |
| **State-block children** | A child instance of the package's state-definition template at a declared index (time, space, inlet or outlet), with `defined_state` and `has_phase_equilibrium` bindings. The child's property demand is the union of what its consumers request (thermodynamics document §4.7) | `build_state_block(...)` |
| **Indexed children (multiplicity)** | A child declared over a finite domain: mixer inlets, splitter outlets, MSContactor stages, trays, periods. Identity is `child(instance, name, member)` | `inlet_list`/`num_inlets`, stage loops |
| **Delegated and aggregate ports** | A port bound to a child's port or state (`control_volume.properties_in[t]`). Members come from the state definition's declared port members with their extensive or intensive role | `add_inlet_port(block=control_volume)`, `define_port_members` |
| **Expression and reference symbols** | Named derived quantities: inlined expressions, or aliases of a child symbol, with complete quantity types. They produce no rows | Pyomo `Expression`, `Reference` (117 references in the characterization) |
| **Interface-typed submodel slots** | A template parameter whose value is a *template implementing a declared interface*: typed inputs and outputs, required laws, allowed features. For example, a heat-exchanger ΔT slot is filled by `LMTD`, `LMTDSmooth`, `AMTD` or `Underwood`; a valve characteristic slot by `Linear`, `EqualPercentage` or `QuickOpening`; a pressure-flow relation slot; an isentropic performance-curve slot; a custom-term slot | Python callbacks: `delta_temperature_callback`, `valve_function_callback`, `pressure_flow_callback`, `isentropic_performance_curves`, `custom_molar_term` |
| **Presets** | A named, versioned partial binding of a template's parameters and slots. `Pump` is a preset of `PressureChanger` (`compressor=true`, `thermodynamic_assumption=pump`). A preset adds no equations and is resolved statically, with lineage | Subclasses `Turbine`, `Compressor`, `Pump`, `Heater1D` variants |
| **Continuous domains and derivative symbols** | Declared spatial or temporal domains with normalized coordinates, derivative symbols `d(x)/d(z)` and `d(x)/dt`, and boundary locations (`z.first`, `z.last`). Admitted as *pending discretization* and never passed to the compiler undiscretized (§6) | `ContinuousSet`, `DerivativeVar` |
| **Declared start estimators, stages and scaling hints** | Typed hints attached to templates and state definitions, consumed by the numerical strategies (see the [numerical strategies document](20-target-numerical-strategies.md)). They are hints and never mutate the specification | `default_initializer`, `default_scaler`, `initialize()` bodies |

Inheritance is not added. Composition, presets and interface slots cover every IDAES subclass
relation found in the characterization. The review's slot 9 argues why this beats inheritance
for authority and local reasoning. The one authority is the base template, and a preset is
binding data.

### 3.3 Authoring syntax

The durable representation stays the registry's authored relations (D1). A library of roughly
100 unit templates and 60 property-method forms authored as YAML rows would be verbose and hard
to review. The target therefore adds a **compact typed template syntax** in `pse-authoring`:

- one file per template;
- the same expression DSL;
- declarations for parameters, slots, children, ports, laws, equations and hints;
- parsing into the same generated declaration values.

Spans map back to the source, and render → parse round-trips exactly. This is an authoring
projection with one authority, like documents and builders today
([§22](../authoritative_design/sections/models-and-composition.md#section-22)). It is a second
*surface*, not a second definition. Rust and Python builders remain for programmatic
generation.

```text
template Heater : Unit {
  param property_package : PropertyPackage<provides: enth_mol_phase, flow_basis>
  param material_balance_type : MaterialBalanceType = useDefault
  param energy_balance_type   : EnergyBalanceType   = useDefault
  param has_pressure_change   : bool = false
  child control_volume : ControlVolume0D {
    property_package = property_package, has_heat_transfer = true,
    has_pressure_change = has_pressure_change,
    material_balance_type = material_balance_type, energy_balance_type = energy_balance_type }
  port inlet  = control_volume.inlet
  port outlet = control_volume.outlet
  ref heat_duty = control_volume.heat
  ref deltaP    = control_volume.deltaP  when has_pressure_change
}
```

The example is illustrative syntax, not a committed grammar.

## 4. Control volumes

### 4.1 ControlVolume0D

This is a reference template in `pse.core`, parameterized by the IDAES option set: balance
types, the `has_*` flags, reaction package, phase equilibrium and geometry. It declares:

- **State-block children**: `properties_in` and `properties_out` at time `t`. The outlet has
  phase equilibrium when requested. Optionally a `reactions` child from the reaction package.
- **Law instances.** One per selected balance, by enum:
  - `MaterialBalanceType.componentPhase` gives a phase×component material law.
  - `componentTotal` gives a component law with phase-summed terms.
  - `elementTotal` gives an element law projected through the element-composition matrix.
  - `total` gives a total-mass law.
  - `useDefault` resolves from the package's typed default.
- **Contributions.** The material-flow terms of inlet and outlet from the state definitions'
  term contracts; kinetic, equilibrium and inherent reaction generation (extent ×
  stoichiometry); phase-equilibrium generation as an internal transfer between the two phases
  of a pair, with one identity and opposite signs; mass transfer; custom terms through a slot.
- **Energy.** Enthalpy flow terms, `heat`, `work`, enthalpy transfer and heat of reaction.
  Isothermal selects the explicit `T_out = T_in` equation instead of a law.
- **Momentum.** Pressure balance with an optional `deltaP` term.
- **Holdup.** `volume`, `phase_fraction`, and material and energy holdup from the density term
  contracts. The holdup symbols are active when `has_holdup`. Accumulation is generated by the
  law engine in dynamic modes (§7).

Each IDAES construction event for `ControlVolume0DBlockData` in the characterization maps to a
template declaration. The coverage map lists the events. The IDAES `ConfigurationError` checks
become admission invariants:

- "reaction block required for reaction terms";
- "phase equilibrium requires ≥ 2 phases";
- "holdup requires geometry".

Each is refused at specialization with the instance's source identity, not raised during
construction (DP-03).

### 4.2 Indexed law engine

The law engine extends [§10.1](../authoritative_design/sections/models-and-composition.md#section-10-1),
where the executable boundary is scalar conservation over total, energy or momentum:

| Aspect | Target |
|---|---|
| Subjects | total mass or amount, component, phase×component, element, charge, total energy, phase energy, momentum or pressure, cost (§9), and user accounting subjects |
| Index domains | Material-derived domains from bound packages. Laws expand per member, and identity includes subject members |
| Contribution index maps | A contribution declares its index map into the law's subject domain: identity; stoichiometric (reaction extent `[r]` × `ν[r, p, j]` into `[p, j]`); element projection (`[p, j]` × `a[e, j]` into `[e]`); phase-pair transfer (`[pair]` into `[p, j]` with a sign per side); phase sum. Maps are typed and admitted; a map between incompatible quantity types is refused |
| Expansions | `conservation` (equality over contributions); `isothermal` and `pressure_total` expansions as declared equation templates (not special cases in lowering); `accumulation` generated per analysis mode |
| Default balance resolution | `useDefault` reads the bound package's typed default at specialization, recorded in the resolved balance |
| Tolerance | The law's closure tolerance stays explicit. IDAES has none; the requirement is kept from §10.1 |
| Closure checks | Unchanged in kind: independent recomputation from the same declared contributions, now per subject member (PS-03) |

Reactions enter only as contributions with stoichiometric maps. The current "project one
authored rate into species balances" logic (`workflow::reactions`) becomes one index map kind.
It is not a separate projection mechanism.

### 4.3 ControlVolume1D and distributed models

ControlVolume1D is a template over a continuous spatial domain `x ∈ [0, 1]`, normalized as in
IDAES, with length `L` and area `A`. It carries a declared `area_definition` (constant or
distributed) and a `FlowDirection` (forward or backward), which determines the inlet boundary
and upwinding. Laws carry an axial flux-derivative contribution `−(1/L)·∂F/∂x`. The law engine
also generates the per-element and per-point holdups. The `material_flow_dx`/`enthalpy_flow_dx`
style derivative variables are derived symbols, not authored ones.

Boundary conditions are explicit equations at `x.first` and `x.last`, never hidden in port
expansion. Heat, mass and momentum transfer terms are indexed by `[t, x]`. Discretization is
§6.

## 5. Units, reactions and the libraries

### 5.1 Core library (`pse.units.core`)

All 31 IDAES core unit models are reference templates built from the mechanisms of §3–§4. The
table groups them by the mechanism that distinguishes them; the coverage map lists each model.

| Group | Models | Distinguishing mechanism |
|---|---|---|
| Single-CV units | Heater, Flash, CSTR, StoichiometricReactor, EquilibriumReactor, PressureChanger (with Turbine, Compressor and Pump presets), Valve, Pipe (0D) | CV0D child plus unit equations. Flash uses the package's VLE formulation and a split-by-phase outlet pair. PressureChanger uses an interface slot for its thermodynamic assumption (isothermal, adiabatic, isentropic, pump) and an optional performance-curve slot |
| Mixing and splitting | Mixer, Separator | Indexed inlet or outlet children. Mixing and splitting laws are explicit (no implicit port mixing, §12.2 kept). Split basis (total, phase, component, phaseComponent) and energy split basis are enum parameters. Pressure equality versus minimization is an enum parameter; minimization is a smooth-min formulation with a declared ε |
| Two-sided exchangers | HeatExchanger, HeatExchangerNTU, HeatExchangerLumpedCapacitance | Two CV children. The ΔT interface slot (LMTD, smooth LMTD, AMTD, Underwood with `cbrt`), flow pattern, and the NTU/effectiveness form |
| Distributed units | PFR, HeatExchanger1D, ShellAndTube1D | CV1D children plus the discretization choice (§6). Co- or counter-current is a flow-direction binding |
| Equilibrium reactors | GibbsReactor | Element law plus Gibbs stationarity with element multipliers, from the resolved chemical potentials (thermodynamics document §4.8) |
| Multi-stream | MSContactor | Indexed children per stream and stage, a declared stream-interaction map, and heterogeneous reactions |
| Boundary and translation units | Feed, FeedFlash, Product, StateJunction, Translator, StreamScaler | State-block children with explicit translation equations (§12.6 kept) |
| Solid-liquid | Thickener0D, SLSeparator | Separator variants over solid and liquid packages |
| User extension | SkeletonUnitModel | A template with user-declared ports and a body from an equation slot or a surrogate (§10) |
| Performance curves | IsentropicPerformanceCurve | Interface slot implementation |

New DSL functions needed by the library are `cbrt` (Underwood ΔT), `smooth_max`/`smooth_min`/
`smooth_abs`, complementarity, `tanh`/`sigmoid`/`softplus` (surrogates and smooth switching)
and `asinh` (Butler–Volmer). Each is added to the function enum with its physical rule,
domain obligation and derivative claim
([§7.2](../authoritative_design/sections/mathematics-and-compilation.md#section-7-2)).

### 5.2 Extended libraries

`models_extra` is in scope. This is an authority change to §0.2 and to the relationship
document (review slot 11). Each family is a reference package built on the same mechanisms:

| Package | Models | New requirements beyond core |
|---|---|---|
| `pse.units.power` | Boiler heat exchangers (2D cross-flow header), fireside, waterwall, drum and 1D drum, downcomer, feedwater heaters (condensing, dynamic), steam heater, water pipe and tank, 3-stream HX, cross-flow HX 1D, balance block, CPU; the Helmholtz unit family (mixer, splitter, valve, pump, isentropic compressor and turbine, turbine inlet, outlet, stage and multistage, phase separator, NTU condenser); the SOC stack (cell, module, channel, conductive and porous slabs, triple-phase boundary, contact resistor); SoecDesign | Pure-fluid Helmholtz packages (thermodynamics document §4.9). 2D distributed domains (two continuous domains). Electrochemistry equations (Nernst, Butler–Volmer with `asinh`). Multistage turbines as indexed children |
| `pse.units.columns` | Tray, TrayColumn, Condenser, Reboiler, PackedColumn, MEAColumn, SolventCondenser, SolventReboiler, PlateHeatExchanger | Indexed tray children (TrayColumn). Packed column as CV1D pairs with interfacial mass transfer and an enhancement-factor slot. MEA electrolyte packages |
| `pse.units.gas_solid` | FixedBed0D/1D, MBR (moving bed), BubblingFluidizedBed, plus the oxygen-carrier gas, solid and heterogeneous-reaction packages | Two material systems in one unit (gas and solid packages); heterogeneous reactions across phases; multi-region 1D (bubble, cloud-wake, emulsion) |
| `pse.units.gas_distribution` | GasPipeline, PipelineNode, IsothermalCompressor, natural gas package | 1D momentum with friction; network composition |
| `pse.units.adsorption` | FixedBedTSA0D (with costing) | Adsorption isotherms as model forms; a cyclic-process formulation |
| `pse.units.membrane` | Membrane1D | Counter-current 1D with permeance |
| `pse.control` | PIDController | Signal ports; integral state; derivative filter; anti-windup as a smooth formulation policy; dynamic mode only (§7) |

## 6. Discretization

A **discretization transformation** in `pse-modeling` lowers continuous domains to finite ones.
It is a DP-08 transformation with a declared contract:

| Aspect | Contract |
|---|---|
| Inputs | Specialized templates with continuous domains, derivative symbols and boundary equations; a **discretization policy** per domain: scheme (finite difference backward, forward or central; orthogonal collocation Lagrange–Radau or Lagrange–Legendre), number of finite elements, collocation points, and element-boundary placement |
| Outputs | Finite indexed domains (element, point) with stable member identities; discretization equations (FD stencils, collocation derivative matrices); continuity equations between elements; the mapping from every generated row and variable back to `(template symbol, continuous coordinate)` (PS-05) |
| Collocation data | Radau and Legendre points as certified real roots of Jacobi polynomials (Symbolica `UnivariatePolynomial::isolate_real_roots` and `refine_root_interval`, pinned 3.0.0), and derivative weights from Symbolica polynomial arithmetic. Not tabulated by hand. Pyomo itself uses `numpy.roots` |
| Equivalence promised | Consistency of order *k* for the scheme on smooth solutions. No claim of accuracy for a given mesh; refinement studies are a study workflow |
| Where applied | Space: always, at specialization. Time: only for the **simultaneous** dynamic route (§7); the integrated route keeps time continuous and passes it to Diffsol/IDAS |
| Failure modes | Unsupported scheme and domain combination, a derivative without a boundary condition where the scheme needs one, or an index problem after discretization. Each is refused with source identities |

The transformation is identity-bearing. A policy change re-specializes only the affected
instances (DP-09). This reinstates, as a typed transformation, what
[blueprint §13.4](../authoritative_design/sections/workflows-and-results.md#section-13) retired.
Retiring it was right for the NL/Pyomo-era pass. The target needs it for distributed units and
for simultaneous dynamic optimization, and neither can be expressed today.

## 7. Analysis modes and dynamics

| Mode | Model effect (generated, not authored) | Execution |
|---|---|---|
| Steady | Laws without accumulation; holdup symbols active only if `has_holdup` | Root (KINSOL) or NLP |
| Dynamic, integrated | Accumulation `d(holdup)/dt` generated for each law with holdup; time continuous; space discretized | Diffsol / IDAS DAE route with consistent initialization and events ([§13](../authoritative_design/sections/workflows-and-results.md#section-13)) |
| Dynamic, simultaneous | As above, then time discretized (§6); initial conditions and path constraints over time points | NLP (Ipopt / POUNCE); dynamic optimization, NMPC and MHE ([numerical strategies document §6](20-target-numerical-strategies.md)) |
| Optimization, estimation, sensitivity | Objective, estimation and parameter declarations on the same model | NLP; fitting ([§19.4](../authoritative_design/sections/workflows-and-results.md#section-19-4)) |

- `dynamic` and `has_holdup` stop being flowsheet flags inherited by traversal. `dynamic` is the
  analysis mode of a case. `has_holdup` is an instance feature with a declared default, and
  dynamic mode requires it where a law has accumulation.
- The current separate `authored.dynamic_cases` state/RHS-row authoring and the hand-written
  `workflow::vessel` recipe (with fixed CAS numbers) are replaced by the generated route. The
  dynamic *case* keeps horizon, events, inputs and initial conditions.
- Differential index is checked structurally after generation, using the existing index-1
  matching of the algebraic partition. A higher-index model is refused, naming the offending
  equations, unless the author declares a reduced formulation; automatic index reduction is not
  a target.
- The PID controller and other signal-flow units connect through signal ports, which are
  information links and not flow edges (§12.5 kept, PS-05).

## 8. Flowsheets and connectivity

- **Flowsheets.** A flowsheet is a template: units, connections and sub-flowsheets as
  children. It carries scope defaults (default property package, time domain) resolved
  statically, not a runtime block with inherited flags.
- **Ports and connections.** Connections keep today's typed admission
  ([§12.2](../authoritative_design/sections/models-and-composition.md#section-12-2)). Members now
  come from state definitions with extensive or intensive roles. Extensive fan-out still
  requires an explicit splitter.
- **Stream tables and unit reports.** These are *result projections*, not model structure.
  A stream table is a query over port members at a time point. A unit performance report is
  declared report symbols (expression symbols) evaluated from the result. Both are Arrow
  relations derived from the completion
  ([§19.2](../authoritative_design/sections/workflows-and-results.md#section-19-2)). This covers
  IDAES `tables.py`, `_get_performance_contents`, `_get_stream_table_contents` and tags. Display
  tags become result-projection declarations.
- **Topology, incidence and solve order stay distinct structures** (PS-05; §12.5, §15 kept).

## 9. Costing

Costing reuses the same mechanisms:

- **Cost correlations as model forms.** The 13 SSLW methods — heat exchanger, vessel,
  fired heater, pressure changer (pump, compressor, fan, blower, turbine), cyclone,
  electrostatic precipitator, dryer, evaporator, … — and their material and type factors are
  authored forms. The 18 SSLW enums become typed parameters. QGESS and power-plant account
  correlations are authored the same way.
- **Unit costing.** A costing *binding* attaches a costing template to a unit instance and
  reads declared unit symbols such as area, work and volume. This replaces
  `UnitModelCostingBlock(flowsheet_costing_block=…, costing_method=…)`.
- **Flowsheet aggregation.** Capital, fixed operating and variable operating cost, and
  annualization, are *accounting laws over cost contributions* (ADR-0010 names cost laws;
  §4.2 subjects). Aggregating a new unit's cost is adding its contribution, not editing a
  flowsheet costing routine.
- **Currency and time.** Currency is a quantity axis. Year-specific currency units convert
  through registered cost-index conversion rules backed by CEPCI (or other index) reference
  data with provenance. This covers IDAES `register_idaes_currency_units`. Conversion between
  years is a registered conversion, never a hidden constant (PS-01).
- **Utility minimization.** Pinch analysis (IDAES `utility_minimization`) is an optional
  reference template: a heat-cascade formulation over declared hot and cold stream symbols. It
  is not framework code.

## 10. Surrogate embedding

| Surrogate kind | Embedding | Derivatives |
|---|---|---|
| Polynomial, ALAMO-form, RBF, kriging mean predictor, MLP with smooth activations | Authored expression forms generated from a trained-artifact *data package* (weights, centres, kernel parameters with provenance) | Exact, via the ordinary pipeline |
| Opaque networks (ONNX with ReLU and the like) | A provider over an inference library, with a declared derivative order and smoothness. Value-only surrogates are refused where second derivatives are required, unless a limited-memory Hessian route is selected | Declared by the provider |

Training and sampling (PySMO, ALAMO, Keras) are *study workflows* outside the model core (see
the [numerical strategies document §6](20-target-numerical-strategies.md)). A trained surrogate
enters the model only as a published, identified data package. `SurrogateBlock` becomes an
ordinary template whose body is the surrogate form, with input and output ports and a declared
validity envelope, the training domain. Evaluation outside it is an envelope failure or an
explicit extrapolation policy (PS-02).

## 11. Ownership and crates

| Crate | Responsibility (target) | Change |
|---|---|---|
| `pse-authoring` | DSL and **template syntax**; documents; identities; edits | Extended |
| `pse-modeling` (new) | Template specialization; presets and slots; package-fact guards; indexed law engine; reaction and costing contribution maps; discretization transformation; analysis-mode generation; report projections declarations. Pure, no Arrow/Tokio, consumed by compiler queries | New crate (ADR). Absorbs `pse-runtime::workflow::{composition, balances, reactions, vessel}` and the dynamic-declaration lowering, deleting them there |
| `pse-properties` (new) | Property kinds, packages, model forms, identities, property resolution, closure formulations | New crate (ADR); see the [thermodynamics document](20-target-thermodynamics.md) |
| `pse-thermo` (new or renamed `pse-kernels`) | Closure algorithms (providers) | See the thermodynamics document |
| `pse-compiler` | Salsa workspace. Specialization and resolution become tracked queries, so edits are incremental from template to body | Extended |
| `pse-runtime` | Admission boundary, effects, jobs, results and publication. No model semantics | Reduced |

The present concentration of semantics in `pse-runtime` is the baseline this corrects:

- `composition/lower.rs` (1,726 lines) mixes parameter typing, guard evaluation, domain
  binding, symbol expansion, case targeting, equation rewriting, law lowering, submodel
  validation and connection and tear construction.
- `model.rs::freeze` mixes selection, classification, provider registration, a valve-law
  dimension check, lowering, projection and admission.

Review [F07](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f07)
covers this.

## 12. Change scenarios (modeling subset)

| Scenario | Target edit path | Baseline |
|---|---|---|
| Add a unit model from existing physics (for example IDAES `Flash`) | One template file: a CV0D child with the phase-equilibrium binding, two outlet ports split by phase, and a preset. Conformance cases from `idaes-oracle:` values | Not expressible. Needs a package child, phase equilibrium, indexed laws and delegated ports |
| Add a ΔT formulation to HeatExchanger | One template implementing the ΔT interface | IDAES: a Python callback. Baseline: not expressible |
| Switch a flowsheet from steady to dynamic | Change the case's analysis mode; add the horizon and initial conditions | Re-author the dynamic states and RHS rows, or use a hand-written recipe |
| PFR with collocation instead of backward FD | Change the discretization policy of the instance or case | Not expressible |
| Add costing to an existing flowsheet | Costing bindings on units plus one flowsheet costing template | Not implemented |
| Test the law engine or discretization in isolation | `pse-modeling` unit tests over declarations. No runtime, DataFusion or native solvers | Only through `pse-runtime` |
