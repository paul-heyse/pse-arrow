---
title: Target design — thermodynamics and the property system
status: draft
date: 2026-09-26
parent: docs/plans/20-idaes-capability-target.md
review_sources: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md]
---

# Target design — thermodynamics and the property system

> **Superseded in part by [Plan 21](https://github.com/paul-heyse/pse-arrow/blob/0e725de269f18dd08331158a07b38a7d92ea0b5e/docs/plans/21-modeling-kernel.md) (2026-09-26).** Wherever this
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

**Evidence level: Proposed.** This is the target architecture for the property half of the
idaes-pse capability. It is a companion to [Plan 20](20-idaes-capability-target.md) and to the
[design review](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md)
that assesses it. Nothing here is implemented or authorized. The companion documents are:

- [modeling and flowsheets](20-target-modeling-and-flowsheets.md);
- [numerical strategies](20-target-numerical-strategies.md);
- [capability coverage map](20-idaes-coverage-map.md).

IDAES behaviour is cited from the pinned characterization in the `pyomo-and-solvers` skill:
idaes-pse 2.13.0 at `.codex/skills/pyomo-and-solvers/content/sources/idaes-pse-2.13.0/`, plus
its static records under `content/idaes/`. In this document the skill root is abbreviated
`skill:`. The characterization is read for behaviour only. No IDAES code, docstring or comment
is copied ([relationship to IDAES](../relationship-to-idaes.md)). Parameter values come from
primary literature. IDAES values serve only as test oracles.

## 1. What must be reproduced

The idaes-pse property capability, as characterized statically in 2.13.0, has four parts.

**Property framework.**

- The `PhysicalParameterBlock`/`StateBlock` contract.
- The property metadata catalogue: `StandardPropertySet` holds 73 kinds and
  `ElectrolytePropertySet` holds 9 (`skill:content/idaes/catalogs/property-sets.json`).
- On-demand property construction. `GenericStateBlock` alone has 99 on-demand builders
  (`show idaes-model:…generic_property.GenericStateBlock --view components`).
- The flow, density and enthalpy-flow term contracts that control volumes consume.
- Default balance types, the material flow basis, and reference states.

**Modular (generic) properties.** `GenericParameterBlock` takes a configuration dictionary with
these keys: `components`, `phases`, `state_definition`, `state_bounds`, `state_components`,
`pressure_ref`, `temperature_ref`, `phases_in_equilibrium`, `phase_equilibrium_state`,
`bubble_dew_method`, `parameter_data`, `base_units`, `include_enthalpy_of_formation`,
`reaction_basis`, `inherent_reactions` and `default_scaling_factors`. It selects from these
method families:

| Family | Members | Count |
|---|---|---|
| State definitions | FTPx, FcTP, FcPh, FPhx, FpcTP, FpTPxpc, electrolyte states | — |
| Pure-component methods | NIST, Perrys, RPP3, RPP4, RPP5, Constant, Eucken, ChungViscosityPure, ChapmanEnskogLennardJones, relative permittivity | 10 |
| Equations of state | Ideal; Cubic (PR/SRK, mixing rules A/B, `cubic_roots` external functions); ENRTL (symmetric/unsymmetric reference, constant alpha/tau) | 3 |
| Phase-equilibrium forms | fugacity, log_fugacity | 2 |
| Phase-equilibrium formulations | SmoothVLE (`_teq = smooth_min(smooth_max(T, T_bub, ε1), T_dew, ε2)`), CubicComplementarityVLE (temperature slacks and cubic-root complementarity via `smooth_min`) | 2 |
| Bubble/dew methods | IdealBubbleDew, LogBubbleDew | 2 |
| Henry | Henry forms (HenryType, ConstantH) | — |
| Reaction forms | arrhenius, power_law_rate, constant_dh_rxn, ConstantKeq, van_t_hoff, gibbs_energy, power_law_equil, log_power_law_equil, solubility_product, log_solubility_product | 10 |
| Transport | ViscosityWilke, ThermalConductivityWMS, NoMethod | — |
| CoolProp | CoolProp wrapper | — |

**Dedicated packages.**

- General Helmholtz multiparameter EoS for 11 fluids, through idaes-ext
  `general_helmholtz_external`. That library registers 238 ASL functions, and `show idaes-ext:…`
  gives each one.
- IAPWS-95 and SWCO2 wrappers.
- Activity-coefficient (NRTL/Wilson) packages.
- Example packages: BT_PR, ASU_PR, HC_PR, CO2_H2O ideal VLE, CO2-bmimPF6 PR, eNRTL brines.
- The MEA solvent and vapour packages, natural gas, flue gas, and the oxygen-carrier gas/solid
  packages in `models_extra`.

**Reference values.**

- 45 `PropertyTestHarness` subclasses.
- The approx oracles in property tests, part of 23,371 oracles
  (`skill:content/idaes/oracles/`).
- 2,559 documentation equations, many of them property derivations
  (`skill:content/idaes/docs/equations.json`).

The IDAES *design* has four features that the target deliberately does not reproduce:

- Each equation of state re-implements every property by hand. `Cubic` has 33 members, and
  each writes its own `cp_mol_phase`, `enth_mol_phase` and so on.
- Properties are constructed by mutation on first attribute access (`build_on_demand`).
- Methods are selected through a free-form configuration dictionary checked at construction.
- Opaque ASL external functions carry the physics that the solver cannot see
  (`cubic_roots`, `general_helmholtz_external`).

## 2. Design thesis

Four decisions shape the target. Each is argued against its alternative in the review (slot 9).

1. **Author a thermodynamic model once, as a potential; derive every property from it.** A
   residual Helmholtz energy `α_r(T, ρ, x)`, an ideal-gas contribution and an excess Gibbs
   energy `gᴱ(T, x)` are typed, authored expressions. Pressure, enthalpy, entropy, fugacity
   coefficients, heat capacities, speed of sound, activity coefficients and the rest follow from
   one registered set of thermodynamic identities, differentiated by Symbolica. A new equation
   of state is one potential declaration, not 30 property methods (DP-06, AP-03, §E extension
   locality).
2. **Keep models apart from closure algorithms.** Finding a density root, association site
   fractions or a phase split is an *implicit closure* of a model. It is selected by an explicit
   formulation policy:
   - equation-oriented equations the flowsheet solver sees; or
   - a nested closure algorithm that consumes the *compiled* model and returns implicit-function
     derivatives.

   Adding a model never touches an algorithm, and adding an algorithm never touches a model
   (AP-01, AP-02).
3. **Resolve property demand as an explicit, acyclic, incremental derivation.** A consumer names
   the property kinds it needs: a balance term, a port member or a unit equation. A Salsa-tracked
   resolver chooses, for each (property kind, index), the providing method from the property
   package and recursively resolves that method's requirements. It emits one inspectable property
   program with full lineage, and a cycle is a typed refusal. This replaces IDAES's mutable
   on-demand construction and the current "explicit provider call per use"
   ([blueprint §9.6](../authoritative_design/sections/physical-semantics.md#section-9-6)).
4. **The thermodynamic knowledge base is project-owned data on library-owned mathematics.**
   The project owns:
   - model forms, parameters, identities, reference states and envelopes;
   - the property packages that select them;
   - the closure algorithms, which are domain algorithms.

   Libraries own the generic parts: Symbolica/Numerica for symbolic algebra, derivatives and
   evaluators; faer for linear algebra; native solvers for iteration. FeOS stops being the
   property architecture. It becomes an independent reference oracle, and its algorithms may be
   adapted with attribution (§7).

## 3. Responsibilities and owners

| Responsibility | Decision hidden | Owner (target) | Consumes | Exposes |
|---|---|---|---|---|
| Physical quantity types, bases, reference states, conversion rules | Complete physical meaning of a value | `pse-quantity` (existing) | registry | `QuantityType`, conversion rules, inference |
| Material facts: species types (neutral, solvent, solute, ion, apparent), elements, charge, phases and phase types, dissociation, formation data | What the chemistry is | `pse-material` (extended) | reference data packages | typed material system |
| Property-kind catalogue | Which properties exist, with their physical type and index shape | registry (`pse-schema`), projected into `pse-properties` | quantity registry | `PropertyKind` with shape (none, component, phase, phase×component, phase-pair, reaction) |
| Thermodynamic model forms (potentials, correlations, transport, reaction forms) | The mathematics of a model family | reference packages (data) + `pse-properties` (typing and admission) | DSL, quantity inference | `ModelForm` declarations |
| Thermodynamic identities | How properties derive from potentials | `pse-properties` (registered identity rules, each written once) | Symbolica differentiation via `pse-math` | derived property expressions |
| Property packages | Which model and method provides which property for which phase and component, with which parameters, references, envelope and formulation policies | package data, admitted by `pse-properties` | material system, model forms, parameter data | `PropertyPackage` (immutable, identified) |
| State definitions | State variables, auxiliary variables, closure equations, port members and term contracts | package data (templates) | property package facts | state-block template |
| Property resolution | Which method provides each demanded property, and the derivation order | `pse-properties`, Salsa-tracked inside the compiler workspace | package, demand set | `PropertyProgram` with lineage |
| Closure formulations (equation-oriented) | Density-root selection, association, VLE/VLLE formulation, smoothing parameters | `pse-properties` (formulation policies → equations) | model program | equations and auxiliary variables |
| Closure algorithms (nested) | Density solve, cubic roots, TP/PH/PS flash, stability, saturation, pure-fluid inversion | `pse-thermo` (new crate: the target home of `pse-kernels`' thermodynamic role) | compiled model programs, faer, qualified root solvers | generalized provider contract (§4.9) |
| Parameter data | Values with units, sources and uncertainty | reference or user packages | — | versioned parameter sets with provenance |
| Reference validation | Agreement with independent references | test suites (`tests/conformance`, the thermo-reference group) | NIST, CoolProp, teqp, FeOS, IDAES oracles | conformance reports (evidence, not authority) |

Dependency direction:

- `pse-properties` depends on `pse-quantity`, `pse-material`, `pse-math` (Symbolica) and
  `pse-authoring` (DSL). It does not depend on runtime, Arrow or native solvers.
- `pse-thermo` depends on `pse-properties`, `pse-math` (compiled programs) and faer. It never
  depends on the flowsheet layer.
- The composition layer (`pse-modeling`, see the
  [modeling document](20-target-modeling-and-flowsheets.md)) consumes `PropertyProgram`s only
  through the resolver's typed demand interface.

## 4. Concepts and contracts

### 4.1 Material system

This extends [blueprint §9.1](../authoritative_design/sections/physical-semantics.md#section-9-1),
which today refuses charged or dissociating species.

- **Species.** A species declares its component role: `neutral`, `solvent`, `solute`,
  `cation`, `anion`, or `apparent` with a declared dissociation stoichiometry into true species.
  It also declares its charge, element composition, molecular weight, and valid phase types
  (`liquid`, `vapor`, `solid`, `aqueous`).
- **Phases.** A phase has a type and an equation-of-state or model assignment through the
  property package (§4.5). Phase-component membership is declared, never inferred. IDAES has the
  same rule through `valid_phase_types` and the `components_in_phase` sets.
- **Formation data.** Standard formation enthalpy, entropy and Gibbs energy at a declared
  reference state are parameter data. Whether a package includes formation enthalpy is a typed
  reference-state choice (§4.12), never a boolean scattered across methods.
- **Admission.** Admission checks electroneutrality of every apparent-species dissociation,
  element closure of every reaction and inherent reaction, and phase-type validity. Violations
  are refused with source identities (DP-03).

### 4.2 Property kinds

A registry-owned catalogue covers the IDAES standard and electrolyte property sets, preserving
IDAES names for parity (blueprint §6.14). It includes `flow_mol_phase_comp`, `enth_mol_phase`,
`fug_coeff_phase_comp`, `act_coeff_phase_comp`, `visc_d_phase`, `therm_cond_phase`,
`surf_tens_phase`, `pressure_sat_comp`, `temperature_bubble`, `dh_rxn`, `k_eq` and `molality`.
Each kind declares:

- its complete quantity type: kind, basis, reference state and scale (PS-01);
- its **index shape**: scalar, component, phase, phase×component, phase-pair, or reaction;
- whether it is a *state* coordinate, an *intensive derived* property or an *extensive term*.
  The last covers material, energy and density terms consumed by balances;
- for derived kinds, the canonical relation used by consistency checks. For example,
  `enth_mol = Σ_p phase_frac[p]·enth_mol_phase[p]` is recorded as a checkable identity, not as a
  method.

Kinds are typed identities. Suffix conventions such as `_phase_comp` are rendering only.
Consumers never infer shape from a name (DP-02).

### 4.3 Model forms

A **model form** is an authored, typed, parameterized mathematical definition in the expression
DSL. Its parameters are typed data axes (per component, per pair, per group). It is written from
primary literature into reference packages. The form kinds are:

| Form kind | Authored as | Examples (IDAES coverage and beyond) |
|---|---|---|
| **Residual Helmholtz** `α_r(T, ρ, n)` | Sum of contribution terms with mixing rules | Generalized cubic (PR, SRK, with IDAES mixing rules A/B and `k_ij`); PC-SAFT hard-chain + dispersion (+ polar); SAFT-VR Mie; multiparameter pure-fluid terms (polynomial, exponential, Gaussian and non-analytic terms for IAPWS-95, Span–Wagner and the other general-Helmholtz fluids); virial |
| **Ideal gas** | `cp_ig(T)` correlation with a declared datum | NIST Shomate, RPP3/4/5 polynomials, DIPPR 100/107/127, Joback group contributions, Perry's correlations; for multiparameter fluids, the ideal-gas Helmholtz part |
| **Excess Gibbs** `gᴱ(T, x)` | Typed expression with pair parameters | NRTL, Wilson, UNIQUAC, UNIFAC (group data), Margules, Van Laar; eNRTL as local NRTL + Pitzer–Debye–Hückel long-range + Born, with symmetric or unsymmetric references |
| **Pure-liquid / standard-state** | Correlation | Liquid density (Perry, Rackett), liquid cp, vapour pressure (Antoine, RPP, DIPPR 101), Henry constants |
| **Direct correlation** | Property expression | Transport: Chung and Chapman–Enskog viscosity, Wilke and WMS mixing, Eucken conductivity, surface tension; relative permittivity; entropy-scaling transport over a Helmholtz model |
| **Association** | Site-fraction mass-action equations over declared site types | SAFT association (2B, 3B, 4C and so on) |

**Admission** of a form checks four things:

- physical typing of every operation, using existing quantity inference
  ([§8.3](../authoritative_design/sections/physical-semantics.md#section-8-3));
- original-domain obligations, such as `ρ > 0`, `b·ρ < 1` and positive logarithm arguments;
- the declared validity envelope of the form, for example the temperature range of a
  correlation;
- parameter-axis completeness for the bound material system. A missing `k_ij` needs an explicit
  default policy (`zero` or `require`), the same rule FeOS binding enforces today.

### 4.4 Thermodynamic identities

The derivation rules are written once in `pse-properties`. Each maps a potential to a property
kind:

| Target | Identity (mixture of `n` moles, `α = α_ig + α_r`) |
|---|---|
| Pressure | `p = ρRT·(1 + ρ ∂α_r/∂ρ)` |
| Compressibility | `Z = p/(ρRT)` |
| Residual enthalpy, entropy, Gibbs energy | `h_r/RT = −T ∂α_r/∂T + Z − 1`; `s_r/R = −T ∂α_r/∂T − α_r + ln Z` (with a declared pressure-datum convention); `g_r = h_r − T s_r` |
| Fugacity coefficient | `ln φ_i = ∂(nα_r)/∂n_i − ln Z` |
| Heat capacities | `c_v = −T² ∂²α/∂T² · R`; `c_p = c_v + T (∂p/∂T)²_ρ / (ρ² ∂p/∂ρ)` |
| Speed of sound, Joule–Thomson, compressibilities | From the second derivatives of α |
| Activity coefficient (gᴱ models) | `ln γ_i = ∂(n gᴱ/RT)/∂n_i` |
| Phase property from component properties | Mixing rules declared per property kind, for example ideal-mixture enthalpy |

Identities are *registered rules with preconditions*: the potential kind, the available
arguments and the reference-state compatibility. They are expanded symbolically by Symbolica
(`pse-math`) into ordinary typed expressions. The existing preparation pipeline then compiles
them, with exact first and second derivatives for the solver
([§7](../authoritative_design/sections/mathematics-and-compilation.md#section-7)). The result:

- The derivative order the flowsheet solver needs is supplied by the ordinary pipeline. That is
  the third order of α for Hessians of pressure and fugacity constraints.
- No new AD mechanism is added (DP-13, PS-07).
- Every property that can be derived from a potential is derived from it. Hand-written
  per-model property methods, the IDAES `Cubic.enth_mol_phase` shape, do not exist. A missing
  identity is a registry addition.

**Explicit property formulations for parity.** Some IDAES formulations are not the exact
derivatives of their own potential. The IDAES `Cubic` fugacity uses a row-κ `δ_i`
(`ceos.py:441-449, 1381`), so for asymmetric κ its `ln φ` is not the composition derivative
of its `a_m`. A model form may therefore declare an **explicit property expression** that
overrides the identity for named kinds. The override is recorded in lineage. The shared
identity-consistency checks (§8) report it as a declared inconsistency rather than a failure.
Package authors choose, for example `PR (thermodynamically consistent)` versus
`PR (IDAES δ form)`. The default remains the identity-derived property.

**Expression size.** An identity is expanded per material-system specialization. Component-pair
sums grow as n², and third-order Hessian terms grow further. This is a cost risk, not a
correctness risk (review F-risk R3). The mitigations are:

1. `where`-shared subexpressions and Symbolica's common-subexpression optimization;
2. evaluating derivatives through Numerica jets (Taylor mode) rather than symbolic Hessian
   expansion;
3. the nested closure route of §4.8, which evaluates the potential as a compiled program with
   implicit derivatives;
4. measured specialization thresholds.

It stays *Proposed* until measured on PC-SAFT with 10–20 components and on IAPWS-95.

### 4.5 Property packages

A **property package** is an immutable, identified declaration. It replaces the IDAES
`GenericParameterBlock` configuration dictionary, and also the current authored
`property_packages`/`method_selections` storage that nothing consumes today. It contains:

| Field | Meaning | IDAES counterpart |
|---|---|---|
| Material system | Components, phases, types, membership, apparent/true relation | `components`, `phases` |
| Phase models | For each phase: Helmholtz model plus ideal gas, or ideal liquid plus gᴱ plus standard-state correlations, or an ideal model | per-phase `equation_of_state` |
| Method selections | For each (property kind, phase or component), the providing model form or correlation, with its parameter set | per-component method entries |
| Parameter sets | References to versioned parameter data with provenance and an explicit default policy for missing pair data | `parameter_data` |
| State definition | Reference to a state-definition template (§4.6) with state bounds | `state_definition`, `state_bounds` |
| Phase-equilibrium pairs | Declared pairs and the equilibrium form (fugacity or log-fugacity equality) | `phases_in_equilibrium`, `phase_equilibrium_form` |
| Formulation policies | Density-root route, association route, VLE formulation (smooth VLE with ε1 and ε2; complementarity with ε; nested flash), bubble/dew method | `phase_equilibrium_state`, `bubble_dew_method` |
| Reference state | Enthalpy and entropy datum; formation enthalpy included or not; pressure datum | `pressure_ref`, `temperature_ref`, `include_enthalpy_of_formation` |
| Material flow basis and default balance types | Typed defaults that control volumes consume when an instance selects `useDefault` | `get_material_flow_basis`, `default_material_balance_type` |
| Inherent reactions | Speciation reactions in the package's chemistry | `inherent_reactions` |
| Validity envelope | Composite envelope derived from the envelopes of the selected forms, narrowed by explicit package bounds | (none in IDAES) |
| Scaling and start hints | Nominal magnitudes per state variable and property kind; start estimators (§4.6) | `default_scaling_factors` |

**Package facts** are typed values that templates may guard on at specialization:

- phase list and phase types;
- component list;
- phase-equilibrium pairs;
- flow basis;
- whether inherent reactions exist;
- which property kinds are providable.

They are computed from the admitted package, never by introspecting objects. They are the
typed answer to IDAES's `hasattr(self.config.property_package, …)` checks, which the
characterization records as guards (`show idaes-function:… --view semantics`).

### 4.6 State definitions

A state definition is a *template* in the ordinary template language (see the
[modeling document §3](20-target-modeling-and-flowsheets.md)), parameterized by the bound
package facts. It declares:

- **State variables**, each with its quantity type and bounds. FTPx has `flow_mol`,
  `mole_frac_comp`, `temperature` and `pressure`. FcTP, FcPh, FPhx, FpcTP, FpTPxpc and the
  electrolyte true/apparent states are the other members.
- **Auxiliary variables and closure equations**, with guards over package facts. FTPx has
  `flow_mol_phase`, `mole_frac_phase_comp`, `phase_frac`, `total_flow_balance`,
  `component_flow_balances`, `sum_mole_frac`, `phase_fraction_constraint`, and the single-phase
  versus multi-phase variants recorded in `skill:content/idaes/index/components.tsv`.
- **`defined_state` semantics.** An inlet state whose composition is fully specified omits
  `sum_mole_frac`, as in IDAES. This is a declared template parameter, not a construction flag.
- **Port members and their extensive or intensive role.** Flow is extensive; temperature,
  pressure and mole fractions are intensive. This drives connection expansion and splitter and
  mixer formulations.
- **Term contracts.** The *material flow term* `[p, j]`, *enthalpy flow term* `[p]`, *material
  density term* `[p, j]` and *energy density term* `[p]`, with their basis. Control volumes
  consume these through the resolver, replacing `get_material_flow_terms` and its siblings.
- **Start estimators** (see the [numerical strategies document §2](20-target-numerical-strategies.md)).
  These are pure, typed expression rules that propose starting values: ideal-K Rachford–Rice
  phase split from Wilson K-values, bubble and dew temperature estimates, and outlet-equals-inlet
  propagation. They are start *inputs with provenance*, never fixes (PS-08).

### 4.7 Property resolution

The resolver has this input and output:

```
resolve(package, state_definition, context, demand: Set<(PropertyKind, IndexTuple)>)
  -> PropertyProgram { symbols, equations, auxiliary_variables, provider_calls,
                       lineage: (kind, index) -> (method, parameters, identity rule) }
```

- **Context.** The context carries the phase set present at this state (some states are
  single-phase), the formulation policies, and the time and space position for identity only.
- **Engine.** Salsa tracked functions do the work (`pse-compiler` workspace, extended). The
  query `property(package, state_def, kind, index)` resolves the providing rule and recursively
  requests its dependencies. Salsa memoizes shared sub-demands and detects cycles. A cycle is
  refused with the chain of `(kind, method)` pairs, not reported as a stack overflow (DP-12).
  Unchanged packages backdate, so an edit to one method re-resolves only its dependents (DP-09).
- **Selection rule.**
  1. An explicit method selection for that kind and index, if one exists.
  2. Otherwise, the identity rule that derives the kind from the phase's potential.
  3. Otherwise, a declared mixing rule from lower-index kinds.
  4. Otherwise, refusal: "property `enth_mol_phase[Liq]` has no providing method in package P".

  The order is fixed and documented, so a consumer never depends on hidden precedence
  (DP-15 selection by declared capability).
- **Materialization policy.** Each resolved property is either inlined as an expression or
  materialized as an auxiliary variable with its defining equation. IDAES makes some properties
  `Var`s with constraints and others `Expression`s. Here materialization is a declared
  package/state-definition choice, used for scaling, conditioning and user-visible reporting.
  Inlining is the default, and materialized variables carry nominal hints.
- **Output.** The output is an ordinary set of typed definitions and bindings consumed by
  template specialization. It enters the same `BodySpec` path as any other definition.
- **Lineage.** Lineage is a first-class output. Every result property can be traced to its
  method, parameters and identity rule (PS-02, DP-21).

The authority change: [blueprint §9.6](../authoritative_design/sections/physical-semantics.md#section-9-6)
states that "the global demand fixed point is not part of the current design". The resolver is
not a fixed point. It is an acyclic, demand-driven derivation closure with explicit refusal. D8
("property and reaction demand is bound explicitly") is preserved at the *consumer* boundary:
consumers name exactly what they need. It is reinterpreted inside the package, where
derivations are resolved by declared rules. That needs an ADR amending D8 (review
[F02](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f02)).

### 4.8 Implicit closures and formulation policies

These implicit relations are closures. Each has a declared formulation, chosen by the package
policy and recorded in the result (PS-06):

| Closure | Equation-oriented (EO) formulation | Nested formulation | IDAES parity route |
|---|---|---|---|
| Density or compressibility root at (T, p, x) | `ρ` (or `Z`) as an auxiliary variable with `p(T, ρ, x) = p`. Branch selection by bounds on packing fraction or `Z`, or for cubics by the complementarity formulation (`cubic_second_derivative`, `gp`/`gn` slacks) | Compiled α plus 1-D root solve plus implicit derivatives `∂ρ/∂θ = −(∂p/∂θ)/(∂p/∂ρ)` (second order by the same rule) | Cubic: nested (the `cubic_root_l`/`cubic_root_h` external functions, `show idaes-ext:cubic_root_l`) or complementarity |
| Association site fractions | Site-fraction variables plus mass-action equations | Inner fixed-point or Newton solve plus implicit derivatives | Not in IDAES; FeOS parity |
| Two-phase VLE | `smooth_vle(ε1, ε2)`: `_teq` via smooth max/min over bubble and dew temperatures, as in IDAES SmoothVLE; or `complementarity_vle(ε_t, ε_z)`: temperature slacks with `smooth_min` complementarity against phase flows | TP/PH/PS flash with stability analysis and implicit derivatives of phase fractions and compositions | SmoothVLE is IDAES's default for modular packages |
| VLLE / LLE | Multiphase MPCC with phase-presence complementarity | N-phase flash with a stability test | Not in IDAES modular properties; FeOS has binary VLLE only |
| Pure-fluid saturation and (p, h)/(p, s) inversion | Density and temperature as variables with saturation equations (Maxwell) and quality complementarity | Pure-fluid inversion algorithm with phase determination and implicit derivatives | General Helmholtz: nested, inside the external functions |
| Chemical equilibrium (Gibbs reactor) | Element balances plus stationarity of `Σ n_i μ_i` with element multipliers | — | IDAES GibbsReactor is equation-oriented |

Every formulation choice:

- declares its approximation (the smoothing parameter and the complementarity relaxation) and
  its smoothness order, and is reported with the result (PS-06, PS-12);
- has a post-solve verification that does not trust the formulation (PS-10):
  - EO VLE is checked for phase stability, a tangent-plane test at the solution;
  - nested flash results are checked for equilibrium residuals;
  - density roots are checked for mechanical stability `∂p/∂ρ > 0` on the declared branch;
- is identity-bearing. Changing ε or the route changes preparation keys (DP-09).

The smooth and complementarity formulations need **smoothing primitives in the DSL**:

- `smooth_max(a, b, ε)` and `smooth_min(a, b, ε)` with IDAES's form
  `½(a + b ± √((a−b)² + ε²))`;
- `smooth_abs`;
- a complementarity form `a ⟂ b` lowered to `smooth_min(a, b, ε) = 0`, or to a relaxation
  schedule policy.

Each takes an ε with a declared quantity type, which `pse-quantity::smoothing` already supports.
Today the DSL refuses these names on purpose (`pse-math/src/functions.rs:14-24`), and
[§7.2](../authoritative_design/sections/mathematics-and-compilation.md#section-7-2) lists them
as removed. Reinstating them as *declared formulation primitives* is an authority change (review
[F05](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f05)).

### 4.9 Closure algorithms and the generalized provider contract

Closure algorithms live in `pse-thermo`. They are domain algorithms that consume *compiled model
programs* from `pse-math`. They do not own model mathematics. The algorithm set is:

- **Cubic roots.** Analytic (Cardano/trigonometric) with explicit branch selection, and
  derivatives by implicit differentiation of `Z³ + c2 Z² + c1 Z + c0 = 0`. This is a small
  bespoke domain routine; §F reason: no library supplies branch-typed cubic roots with exact
  derivatives, and the `roots` crate gives values only.
- **Density solve.** For a general α: bracketed Newton on the compiled `p(ρ)` with branch
  initialization (liquid or vapour packing fraction). Convergence is checked by the residual,
  never by the iteration count. The FeOS 0.10.1 density iteration returns its last iterate as
  success after 50 iterations (`feos-core-0.10.1/src/density_iteration.rs`), and this is the
  error to avoid.
- **Flash.** TP, PH and PS; two-phase and N-phase with stability analysis. The algorithms are:
  - tangent-plane-distance minimization;
  - successive substitution with Rachford–Rice;
  - Newton on the Gibbs system, using faer for linear algebra;
  - implicit derivatives of phase fractions and compositions from the converged equilibrium
    Jacobian.

  The algorithms may be adapted from FeOS (MIT/Apache-2.0, with attribution) and generalized to
  N components with derivatives. At the pin, FeOS propagates implicit derivatives through
  bubble/dew points, PH/PS flash (two-phase only, needing a two-phase start temperature), pure
  saturation, critical points and `tp_flash_binary`. Only the N-component `tp_flash`
  (N > 2) is f64-only, and it returns `NoPhaseSplit` rather than a one-phase result (survey
  2026-09-26, `feos-core-0.10.1/src/phase_equilibria/`). Where an inner iteration is a generic
  nonlinear solve, the qualified solver owns it (PS-09); an in-worker KINSOL dense profile is one
  option. The successive-substitution and stability logic are domain algorithms.
- **Pure-fluid Helmholtz inversion.** Saturation with the Akasaka or Maxwell algorithm, and
  (p, h), (p, s) and (T, x) state inversion with phase determination and quality. This covers
  the general-Helmholtz function surface: the 238 registrations in `idaes-native-01`, all of which
  are *properties derivable from α*, so none needs a separate function.
- **Saturation pressure, bubble and dew points, critical points.**

The provider contract generalizes today's
[`ProviderSpec`](../authoritative_design/sections/physical-semantics.md#section-9-4), which is
strictly scalar in and out (`pse-kernels/src/lib.rs:190`), fixes one phase, and whose single
production thermodynamic implementation is FeOS:

| Aspect | Today | Target |
|---|---|---|
| Port shape | Scalar only; a vector is *n* rows with one `output` ordinal each | Typed index shapes (component, phase, phase×component) with declared order bound to the material system |
| Phase | One fixed phase per spec | Phase *set*, with typed per-phase outputs and a **regime** output: phases present and branch identity |
| Model | Hard-coded in the provider (FeOS parameters) | A reference to a compiled model program (potential plus package parameters). The provider is an algorithm |
| Derivatives | Declared order, from num-dual on explicit maps | Declared order with its source (`analytic`, `implicit-from-compiled`, `none`), and implicit derivatives from converged closures |
| Smoothness | One declared order | Piecewise: a declared order within a regime, with regime changes observable. A result that crossed a regime is reported, never silently differentiated across the boundary |
| Internal starts | One-trial cache | An explicit per-attempt warm-start state owned by the worker. Its effect on results is declared as a determinism class (DP-11), and it is never shared across attempts (DP-19) |
| Selection | String match `kind == "feos-pcsaft-dippr"` in `workflow::sources::factory` | Closure-algorithm registration by declared capability (closure kind, supported model forms, phases, derivative order). The package's formulation policy selects it, and preparation records the selection (DP-15) |

The failure classes (`Trial`, `OutsideEnvelope`, `Singular`, `Contract`, `Limit`, `Cancelled`,
`Terminal`) and their recoverability are kept. They are the right contract.

### 4.10 Reactions

A **reaction package** is admitted with a material system. It declares:

- **Stoichiometry** per reaction and phase, including multiphase and heterogeneous reactions,
  with element and charge closure checked at admission.
- **Rate forms**: Arrhenius, power law and user expressions. The concentration basis is a typed
  property-kind demand (`conc_mol_phase_comp`, partial pressure, activity, mole fraction)
  resolved through §4.7. Concentration forms are therefore no longer refused for lack of an
  implicit conversion, because the conversion is an explicit resolved property.
- **Equilibrium forms**: constant `K`, van't Hoff, Gibbs-energy based
  (`ln K = −ΔG°(T)/RT` from formation data), power-law and log-power-law, solubility product
  with complementarity for precipitate presence.
- **Heat of reaction**: `constant_dh_rxn` or derived from formation enthalpies. It is
  consistent with the package's enthalpy datum, and the existing rule against double counting
  formation energy is kept and generalized: a package whose enthalpy includes formation must not
  also add `dh_rxn`.
- **Rate basis**: molar or mass. This is a typed conversion, not a guess.

Rate and equilibrium *extents* enter balances as contributions through the law mechanism (see
the [modeling document §5](20-target-modeling-and-flowsheets.md)). Inherent reactions are part
of the property package and produce inherent extents in every control volume that uses it, as
in IDAES.

### 4.11 Electrolytes

- **True and apparent species.** The state is authored on either basis. The declared
  dissociation stoichiometry relates them; equilibrium speciation comes from inherent reactions.
  Properties are requestable on the true or apparent basis (`*_true`/`*_apparent` kinds).
- **eNRTL** is authored as a gᴱ model form (§4.3), with IDAES's symmetric and unsymmetric
  reference-state variants as a typed parameter.
- **Electroneutrality** is an admission invariant on declared ions and a per-phase equation
  where the state definition requires it.
- **ePC-SAFT** is a Helmholtz model form. It is not in IDAES; FeOS has it, and it is an
  independent oracle.

### 4.12 Reference states and conventions (PS-01)

- Every enthalpy-like and entropy-like property kind carries a reference-state type. The
  package selects one datum:
  - ideal-gas elements (with formation enthalpy); or
  - ideal-gas species at T0 and p0 (without formation enthalpy).

  Mixing different datums is refused by the existing additive rule
  ([§8.4](../authoritative_design/sections/physical-semantics.md#section-8-4)).
- The IDAES entropy pressure-datum convention and the FeOS entropy offset are explicit
  conversion rules, not constants hidden in a provider. Today `FeosPackage` has an
  `entropy_offset()`.
- **Basis.** Molar and mass bases are distinct quantity types. The conversion requires `mw` from
  the resolved properties and is an explicit operation in the program (§8.2), never a
  representation-unit edge.

### 4.13 Envelopes and validity (PS-02)

Validity has three layers:

1. **Form envelope.** The declared range of a correlation (T range of an RPP4 cp fit), or the
   documented range of an equation of state.
2. **Package envelope.** The intersection of the selected forms' envelopes, further narrowed
   by package bounds.
3. **Closure envelope.** The region where a closure algorithm is qualified, for example
   supercritical versus two-phase handling.

Evaluation outside an envelope is a recoverable `OutsideEnvelope` trial failure. That is the
current contract, extended from providers to authored forms by compiling the envelope as guard
obligations. Alternatively it is an **explicitly selected extrapolation policy** recorded with
the result (PS-02). Correlation validity intervals, today "data … not checked at execution time"
([§9.10](../authoritative_design/sections/physical-semantics.md#section-9-10)), become enforced
obligations.

### 4.14 Parameter data and provenance

- Parameter sets are versioned package data: values, units, source citation, and optional
  uncertainty. They are keyed by component identity (CAS or explicit species ID) and pair or
  group identity.
- Missing pair data uses the explicit default policy.
- Shipped datasets are sourced from primary literature: NIST, RPP, Perry's, the DIPPR public
  subset, and the original EoS publications. For multiparameter fluids, CoolProp's MIT-licensed
  fluid files are an admissible *data* source, recorded as such.
- IDAES parameter values are used only as oracle inputs in parity tests, with the IDAES test
  that asserted them cited by `idaes-oracle:` ID.
- Parameter estimation results (see the
  [numerical strategies document §6](20-target-numerical-strategies.md)) publish new parameter
  sets with lineage to the fit that produced them. A package binds a parameter set by identity.

## 5. Physical-semantics table (profile slot 3)

| Quantity or model element | Dimension and unit | Basis | Reference state / convention | Validity envelope | Authority |
|---|---|---|---|---|---|
| State temperature, pressure | K; Pa (absolute) | — | Absolute pressure; gauge only through the datum type | Package envelope | State-definition template; quantity registry |
| Flow, component flow | mol/s or kg/s | molar or mass (typed) | — | Nonnegative unless flow reversal is declared | State definition; property kind |
| Mole and mass fractions | dimensionless | phase or total composition subject | Declared component order | [0, 1], with IDAES-compatible bounds `[1e-20, 1.001]` as a declared state bound | Property kind |
| Enthalpy and entropy (molar, phase, component) | J/mol, J/mol/K | molar or mass | Package datum: element or species reference, T0 and p0, formation included or not | Form and package envelope | Identity rule on the potential; reference-state registry |
| Fugacity coefficient, activity coefficient | dimensionless (ln) | per component in phase | Standard state declared by the gᴱ model (symmetric or unsymmetric) | Form envelope | Identity rule |
| Density, compressibility | mol/m³; — | molar | Branch identity (liquid or vapour) | `∂p/∂ρ > 0` on the branch | Closure formulation |
| Phase fraction, phase flows | —; mol/s | molar | Phase-set regime | [0, 1] with complementarity at boundaries | VLE formulation policy |
| Reaction rate, extent | mol/s (per volume or per mass of catalyst, typed) | declared rate basis | Stoichiometric sign convention: products positive | Rate-form envelope | Reaction package |
| Heat of reaction | J/mol | per extent | Consistent with the enthalpy datum; no double counting | — | Reaction package and reference-state rule |
| Transport properties | Pa·s, W/m/K, m²/s, N/m | phase | — | Correlation envelope | Correlation form |

## 6. Change scenarios (thermodynamic subset)

These are refined into the review's scenario set (S-IDs there).

| Scenario | Target edit path | Current-baseline path |
|---|---|---|
| Add a new cp correlation (for example RPP5) | One model-form declaration with its parameter axes in a reference package, plus focused formula tests | The method package declares it, but nothing executes it. Every model that needs it hand-authors the same correlation as a computation definition: two authorities (review F03) |
| Add a new EoS family (for example SRK variant or SAFT-VR Mie) | One Helmholtz model form. Every property follows from identities, and the existing closures apply | A new Rust provider implementing each output by hand, plus a `sources.rs` string-match edit |
| Switch a package from SmoothVLE to complementarity VLE | Change the package formulation policy. No unit template changes | Not possible; phase equilibrium is refused |
| Replace nested cubic roots with the EO complementarity route | Change the package policy; the closure registry selects the formulation | Not possible |
| Test a property method in isolation | `pse-properties` unit test: resolve and compile one property for one package, evaluate against reference values. No runtime, DataFusion or native solver | Only by building a runtime model around a computation definition |
| Upgrade or replace a reference oracle library (FeOS, teqp) | Test-only dependency change | FeOS is the production provider, so an upgrade moves the num-dual family (register R-05) |

## 7. Library decision for thermodynamics

The options are compared on capability fit for the full IDAES scope, extension locality,
derivative integrity and ownership cost.

| Option | Fit | Gaps against IDAES scope | Consequence |
|---|---|---|---|
| **A. Keep FeOS as the property architecture, enabling more features** | PC-SAFT family, ePC-SAFT, PR, pure multiparameter (its term enums cover all 11 IDAES general-Helmholtz fluids), entropy-scaling transport for PC-SAFT, and derivative-capable bubble/dew, PH/PS flash, saturation and critical points | No activity-coefficient (excess-Gibbs) models, SRK or other cubics, modular method selection, correlation library, or phase-aware provider contract. N-component TP flash is f64-only. Transport exists only for PC-SAFT, whose `EntropyScaling` implementation is f64-only even though the trait is generic. Multiparameter is pure-only with no transport or surface tension; its `tc`/`rhoc` are reducing values (a data trap for R134a, R1234ze(E) and isobutane). The density iteration can report an unconverged root as success. FeOS fixes the num-dual family (R-05) | Every IDAES family outside SAFT becomes bespoke *beside* FeOS, giving two property architectures. **Rejected** as the architecture |
| **B. Fork FeOS and extend it** | Proven Helmholtz-trait and flash algorithms | Its models are hand-coded Rust `Residual` implementations with dual-number generics. Every new model is Rust code, and model data is not declarative. It duplicates Symbolica's role (AD by dual numbers versus library symbolic algebra) | Two math authorities: num-dual for thermodynamics, Symbolica for everything else. **Rejected** as a whole, but its *algorithms* are an input to option D |
| **C. CoolProp (via `coolprop-sys`/`rfluids`) or teqp for all EoS** | Broad pure-fluid coverage, transport and surface tension, analytic partial derivatives through the C API | C++ black box: model forms are not data. The C API has no composition derivatives, so mixtures cannot be equation-oriented. `coolprop-sys` serializes every call behind one process-wide lock with a global error string, which conflicts with attempt-owned workers (§18.8). teqp has no Rust binding. Weak on activity models and electrolytes | **Oracle only**, through the Python thermo-reference group (CoolProp next to teqp). teqp's bundled CoolProp-format fluid files (MIT) are an admissible *data* source for Helmholtz forms |
| **D. Native potential-based framework on Symbolica/Numerica (recommended)** | Models as authored potentials and correlations; identities derive every property; nested closures consume compiled programs; exact derivatives throughout | Must author every IDAES model family from literature and implement the closure algorithms. Expression-size cost for large SAFT mixtures needs measurement (§4.4) | One math authority. A model is data, and an algorithm is a registered closure. FeOS, CoolProp, teqp and NIST become **independent oracles** (DP-23) |

**Decision proposed: D**, with FeOS retained as:

- a **test-only oracle** for PC-SAFT and ePC-SAFT properties and for pure-fluid multiparameter
  models;
- an **algorithm reference** for the flash and stability routines adapted into `pse-thermo`.
  Where an algorithm is adapted from FeOS, FeOS stops being an independent oracle for *that
  algorithm*. Flash validation then uses teqp, CoolProp, NIST data and the IDAES oracles
  instead (DP-23: agreement through a shared transformation is not independent).

The production FeOS provider and its bindings
([§9.8](../authoritative_design/sections/physical-semantics.md#section-9-8)) are deleted once
the native PC-SAFT form and closures pass their conformance suite, in the same change (AGENTS.md
execution rhythm). Removing FeOS from production also removes the num-dual family coupling
recorded in R-05. Library candidates for the remaining generic pieces are in review slot 8.

## 8. Verification approach (PS-13)

Checks are shared by all models, so a new model form gains them without new test code:

- **Identity consistency.** Maxwell relations, Gibbs–Duhem across components, ideal-gas limit
  (`ρ → 0`), `Σ x ln γ` against `gᴱ/RT`, and derivatives against a finite-difference comparison
  at nontrivial points where doubt remains (PS-07).
- **Closure verification.**
  - Density: residual and mechanical stability.
  - Flash: equilibrium residuals, material balance, and tangent-plane stability of the
    reported phases.
  - Saturation: equal pressure and Gibbs energy.
- **Envelope rejection.** Each form rejects a point just outside its declared range.
- **Oracle independence.** Two implementations that read the same coefficient files agree by
  construction on the data. Examples are the native multiparameter forms and FeOS
  multiparameter, both reading CoolProp-format files. Data-independent oracles therefore
  anchor each fluid: the IAPWS-95 release verification values, and NIST.
- **Reference validation, with declared tolerances and conditions.**
  - NIST WebBook data. The characterization includes `pure-prop-nist-webbook.csv` among the
    IDAES test data files.
  - teqp (thermo-reference group).
  - CoolProp for multiparameter fluids.
  - FeOS for PC-SAFT and ePC-SAFT.
  - IDAES oracles: the property-harness values and the per-package `approx` assertions, keyed by
    `idaes-oracle:` ID. Same parameters, same reference states and same conditions are
    preconditions of the comparison (profile false positive: "It matches the reference
    simulator").

The conformance suite lives with its owner (`pse-properties`, `pse-thermo`). It runs without
the runtime (PSE-S05).

## 9. Risks and open questions

| Risk or question | Why it matters | How it is settled |
|---|---|---|
| R1. Expression size for multicomponent SAFT third derivatives | Preparation time and memory | Measure (PC-SAFT with 10/20 components, IAPWS-95); fall back to the nested route and Numerica jets |
| R2. EO VLE robustness versus nested flash | Convergence from poor starts | Both formulations exist; start estimators and the nested flash as an initializer; measured on the IDAES flash and column oracle cases |
| R3. Regime changes in nested closures inside NLP iterations | Nonsmoothness can stall Ipopt | Regime observation, post-solve stability check, and EO complementarity as the default for optimization |
| Q1. Which reference fluids to author first for general Helmholtz | Scope sequencing | Plan 20 phasing: water (IAPWS-95) and CO2 first, matching IDAES power-generation usage |
| Q2. UNIFAC group data licensing and sourcing | Data availability | Source-by-source provenance in the package; licensing does not constrain the design (dependency policy) |
