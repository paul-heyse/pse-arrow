---
title: Knowledge placement and porting guide
status: draft
date: 2026-09-26
parent: docs/plans/21-modeling-kernel.md
---

# Knowledge placement and porting guide

**Evidence level: Proposed.** This is where every kind of idaes-pse scientific knowledge sits
on the [modeling kernel](21-modeling-kernel-architecture.md), and how it is ported. It is the
porter's contract: if a construct cannot be placed as described here, that is a kernel gap to
raise, not a reason to write model code. The syntax below is **illustrative**. Plan 21 packet
K1 fixes the grammar.

## 1. Placement map

| IDAES knowledge | Kernel construct | Package (proposed layout) | What the porter writes |
|---|---|---|---|
| Component, phase and element classes; `valid_phase_types`; charge; molecular weight | Entity kinds `species`, `phase`, `element` with attributes (K2) | `pse.chem` (kinds) + dataset packages (members) | Kind declarations once; entities and attribute rows per chemical |
| `component_list`, `phase_list`, `phase_component_set`, `phase_equilibrium_idx` | Sets derived from entities and tables (K3) | Property-package definitions | Set expressions (filters and relation images) |
| `parameter_data`, JSON parameter files, SSLW coefficient dictionaries, CEPCI values | Tables and datasets with provenance and units (K4) | `pse.data.<source>` | Table schema + rows with citation; a missing-value policy for pair data |
| `StandardPropertySet`, `ElectrolytePropertySet` names | Members of prelude interfaces (`MaterialState`, `PhaseThermo`, `ElectrolyteThermo`) (K7) | `pse.prelude.thermo` | Member declarations with complete quantity type and index shape (once) |
| Pure-component methods (NIST, Perrys, RPP3/4/5, Constant, Chung, Chapman–Enskog, Eucken, …) | Functions implementing interface members (`IdealGasCp`, `LiquidCp`, `VaporPressure`, `LiquidDensity`, `PureViscosity`, …) with `@valid` (K5, K7) | `pse.thermo.methods.<source>` | One function per property form; a coefficient table; validity range; tests |
| Equations of state (Ideal, Cubic PR/SRK, ENRTL, and beyond: PC-SAFT, multiparameter) | `fn alpha_r(T, rho, n)` implementing `HelmholtzPhase`, or `fn gE_RT(T, x)` implementing `ExcessGibbsPhase`. Properties come from interface **defaults** (identities as data) | `pse.thermo.eos.<name>` | The potential function and its mixing rules; explicit overrides only where IDAES differs from the identity (for example the Cubic δ) |
| Thermodynamic identities (p from α, ln φ from ∂(nα)/∂n, h, s, cp, speed of sound; ln γ from gᴱ) | Default members of `HelmholtzPhase`, `ExcessGibbsPhase` and `IdealGasPhase` (K7, K10 ∂) | `pse.prelude.thermo` | Written **once** |
| Density root, cubic root branch, association, bubble/dew temperature, phase split, (p, h) inversion | Implicit blocks with branch bounds, starts, optional regime selection and a realization policy (K11) | Inside the EoS or formulation definitions | Equations + starts + bounds; the policy default |
| `cubic_roots` external functions | `accelerated(cubic_roots)` realization of the cubic implicit block (optional) | Kernel accelerator registry | Nothing in the package; the accelerator is validated against `nested` |
| State definitions (FTPx, FcTP, FPhx, FcPh, FpcTP, electrolyte) | Definitions implementing `MaterialState`: state vars, auxiliary vars, closure equations, port members with roles, term members (flow, density, enthalpy flow) | `pse.thermo.states` | The definition; start estimators; nominal hints |
| Phase-equilibrium forms and formulations (fugacity and log-fugacity equality, SmoothVLE, complementarity VLE), bubble/dew methods, Henry | Definitions implementing `PhaseEquilibrium` or `SaturationEstimate`; smoothing functions as data (K5) | `pse.thermo.equilibrium` | Equations; ε as typed parameters |
| `GenericParameterBlock` configuration (a property package) | A definition implementing `ThermoPackage` that binds phase models, dispatch tables per component, state definition, VLE formulation, reference state and datasets (K6–K8) | `pse.thermo.packages.<name>` | A binding definition. No new equations |
| `default_material_balance_type`, flow basis, `include_enthalpy_of_formation`, reference state | Package members (typed enums, quantity types) read by control volumes through the interface | Package definition | Values |
| Reaction packages, rate forms (Arrhenius, power law), equilibrium forms (Keq, van 't Hoff, Gibbs, solubility), `dh_rxn` | Functions and definitions implementing `RateLaw`, `EquilibriumConstant`, `ReactionHeat`; stoichiometry as a table; element-closure and electroneutrality as requirements | `pse.reactions.*` | Forms, tables, requirements |
| Control volumes (CV0D, CV1D, extended), balance types | Definitions with accumulators, contributions, `when` variants over balance-type enums and analysis mode | `pse.core.control_volumes` | The definitions (once) |
| Unit models (31 core + 57 extra) | Definitions composing CV children, ports, unit equations, slots, presets | `pse.units.<family>` | One definition per unit, presets for subclasses |
| Config callbacks (ΔT, valve characteristic, pressure-flow, performance curves, custom terms) | Interface-typed slots with implementations (K7) | Next to the unit | An interface + one definition per callback |
| Subclasses (Turbine, Compressor, Pump, …) | Presets | Next to the unit | One line each |
| `ConfigurationError` checks | Requirements (K12) | In the definition | Predicate + message |
| Initializers and legacy `initialize()` sequences, `ModularPropertiesInitializer` estimates | `@start` annotations and named `stage`s (K12) | In the definitions | Start expressions; stage variants |
| Scalers, `default_scaling_factors`, constraint scaling schemes | `@nominal` and `@scale` annotations | In the definitions | Nominal expressions; scheme names |
| `_get_performance_contents`, stream tables, tags | `@report` annotations | In the definitions | Labels |
| Costing base, SSLW, QGESS, power-plant and TSA costing | Functions + tables + contributions to accounting accumulators declared by a flowsheet costing definition | `pse.costing.*` | Correlations, factor tables, contributions |
| Currency units, CEPCI | Units and conversion rules with a cost-index table (K1, K4) | `pse.costing.currency` | Rows |
| PID controller | Definition with signal ports, integral state `d(i)/dt` under `when analysis.dynamic` | `pse.control` | The definition |
| Surrogates (ALAMO, PySMO polynomial, RBF, kriging, OMLT) | Generated function definitions + data tables; opaque networks as external functions | Published surrogate packages | Emitted by a training study, not hand-written |
| Discretization schemes (FD, Lagrange–Radau, Lagrange–Legendre) | Scheme definitions (stencils; collocation via `jacobi_roots`) | `pse.numerics.discretization` | The schemes (once) |
| Smooth math (`smooth_max`, `safe_log`, `cbrt`, `x_over_exp_x_minus_one`) | Functions (K5) with obligations or verified `piecewise` | `pse.prelude.math` | The functions (once) |
| DiagnosticsToolbox thresholds, `idaes.cfg` solver defaults | Named policy profiles (data) | `pse.profiles.idaes` | Values |
| IDAES test assertions (23,371 oracles) | Conformance tests referencing `idaes-oracle:` IDs (K12) | Next to each ported definition | Case bindings + expectations + tolerances |
| Flowsheet examples | Definitions + cases | `pse.examples.*` | Composition only |

**Not placed as knowledge**, because they are workflow or infrastructure:

- PETSc wrappers: covered by the native solvers.
- CLI, UI, DMF: out of the target.
- Prescient co-simulation, and the market bidding and tracking loops of `grid_integration`:
  studies or workflows at the Python boundary over kernel analyses.

## 2. Porting procedure (per IDAES entity)

The skill at `.codex/skills/pyomo-and-solvers/` supplies everything needed. The porter reads
for behaviour; nothing is copied or machine-translated from IDAES source (clean room).
Equations are re-expressed from the behaviour and the cited literature. Parameter values come
from primary sources, and IDAES values are used only as oracle inputs.

1. **Read the record.** `show idaes-model:<class> --view config|components|equations|init|scaling|oracles|docs`,
   or `show idaes-method:<class> --view equations`.
2. **Map each record section to its kernel construct:**

| Record section | Kernel construct |
|---|---|
| `config.entries` | parameters (enum, bool, typed); callbacks → slots; `useDefault` → scope default |
| `components` with guard chains | `var` / `param` / `let` / equations inside `when` blocks with the same guards, re-expressed over package facts |
| `delegated_components` | Already provided by the child definition (a CV or state definition); **write nothing** |
| `method_components` (on-demand builders) | Lazy members of the implementation (definitional `let` or `var … defined by`) |
| `equations[].forms` (including `Constraint.Skip` guards) | Equations with `for … where` filters |
| `routines` (initialize, scaling) | `@start`, `stage`, `@nominal`, `@scale` |
| `oracles` | Conformance tests with the same parameters, reference states and tolerances |
| `docs` equations | Cross-check against the written equations; disagreements are recorded, and the code behaviour is what IDAES executes |

3. **Author** the package file(s), and the dataset rows with citations.
4. **Test.** Run the conformance tests for the ported entity, plus the kernel's shared checks:
   DoF, closure, derivative consistency sample, envelope rejection, start-to-solve.
5. **Done** when the tests pass with **no Rust change**. Otherwise file a kernel gap (Plan 21
   *Kernel gaps* table).

## 3. Worked examples

These are illustrative sketches only. Units use the existing `value{unit}` literal form. Data
values are placeholders.

### 3.1 Chemistry kinds and data

```text
package pse.chem
entity kind element { atomic_mass: MolarMass }
entity kind species { mw: MolarMass, charge: Integer = 0, cas: Text?, formula: Text?,
                      valid_phases: Set<PhaseType> }
entity kind phase   { type: PhaseType }
enum PhaseType { liquid, vapor, solid, aqueous }
table composition[e: element, j: species] : Count

package pse.data.bt
use pse.chem.*
entity species benzene { mw = 78.11{g/mol}, cas = "71-43-2", valid_phases = {liquid, vapor} }
entity species toluene { mw = 92.14{g/mol}, cas = "108-88-3", valid_phases = {liquid, vapor} }
dataset perry_liquid_cp source "Perry's Chemical Engineers' Handbook, 8th ed., Table 2-153"
  : pse.thermo.methods.perry.cp_liq_coeffs { benzene: [...], toluene: [...] }
```

### 3.2 A correlation is a function

```text
package pse.thermo.methods.perry
table cp_liq_coeffs[j: species, k: 1..5] : ...          // column units declared per k
fn cp_liq(T: Temperature, c: Row<cp_liq_coeffs>) -> MolarHeatCapacity
   @valid(T in c.T_range, reject)
   = c[1] + c[2]*T + c[3]*T^2 + c[4]*T^3 + c[5]*T^4
impl LiquidCp for species using cp_liq(T, cp_liq_coeffs[self])
test perry_benzene_298 { expect cp_liq(298.15{K}, cp_liq_coeffs[benzene]) == <ref> ± rel 1e-9
                         source "Perry 8th ed." }
```

The `LiquidCp` interface consumes explicit primitive functions for `cp` and `cp/T`,
plus the package's reference temperature, enthalpy and entropy. Its defaults subtract
each primitive at the reference temperature and add the declared reference value.
Package tests check both primitive derivatives against `cp`; the continuous-axis
integral operator does not imply an arbitrary symbolic antiderivative capability.

### 3.3 Identities are interface defaults

```text
package pse.prelude.thermo
interface HelmholtzPhase {
  param components: Set<species>
  fn alpha_r(T: Temperature, rho: MolarDensity, n: Amount[components]) -> Dimensionless
  // inputs bound by the phase state:
  sym T: Temperature;  sym p: Pressure;  sym x: MoleFraction[components]
  var rho: MolarDensity defined by implicit { p == rho*R*T*(1 + rho*∂alpha_r/∂rho(T, rho, x)) }
          @start(p/(R*T)) @bounds(0, rho_max) realize policy.density
  let Z        = p/(rho*R*T)
  let h_res    = R*T*(-T*∂alpha_r/∂T(T, rho, x) + Z - 1)
  let ln_phi[j in components] = ∂(Σn·alpha_r)/∂n[j](T, rho, x) - log(Z)
  let enth_mol = ideal.enth_mol(T) + h_res
  …                                                     // s, g, cp, cv, speed of sound
}
```

### 3.4 An equation of state is one function

```text
package pse.thermo.eos.cubic
def PengRobinson(components, crit: CritTable, kappa: PairTable) : HelmholtzPhase {
  fn alpha_r(T, rho, n) = …                             // generalized cubic α_r with PR (u, w)
}
def PengRobinsonIdaesDelta(...) : HelmholtzPhase {      // parity variant (row-κ δ)
  use PengRobinson(...) for alpha_r
  override let ln_phi[j] = …                            // IDAES form; recorded as an override
}
```

The density implicit block inherits the interface's `realize policy.density`. A package
chooses `inline` (equation-oriented), `nested`, or `accelerated(cubic_roots)`.

### 3.5 A property package is a binding

```text
package pse.thermo.packages.bt_ideal
def BTIdeal : ThermoPackage {
  components = {benzene, toluene};  phases = {Liq: liquid, Vap: vapor}
  phase_model[Vap] = IdealGasPhase(cp = cp_ig_method)
  phase_model[Liq] = IdealLiquidPhase(cp = perry.cp_liq, dens = perry.dens_liq, psat = rpp4.psat)
  cp_ig_method[j in components] = rpp4.cp_ig                // dispatch table (per member)
  state = FTPx(bounds = …);  equilibrium = SmoothVLE(pairs = {(Vap, Liq)}, eps1 = 0.01{K}, eps2 = 0.0005{K})
  reference = ReferenceState(T = 298.15{K}, p = 101325{Pa}, formation = included)
  default_material_balance = componentTotal;  flow_basis = molar
}
```

### 3.6 A control volume holds accumulators

```text
package pse.core.control_volumes
def ControlVolume0D(pkg: ThermoPackage, material_balance: MaterialBalanceType = useDefault,
                    has_heat_transfer = false, has_pressure_change = false,
                    has_phase_equilibrium = false, has_holdup = false, reactions: RateReactions?) {
  require not has_phase_equilibrium or size(pkg.phases) >= 2 : "phase equilibrium needs two phases"
  child inlet[t in scope.time]  : pkg.state(defined = true)
  child outlet[t in scope.time] : pkg.state(defined = false, equilibrium = has_phase_equilibrium)
  let mbt = if material_balance == useDefault then pkg.default_material_balance else material_balance
  when mbt == componentPhase {
    accumulate material[t in scope.time, (p, j) in pkg.phase_components] conservation (tolerance = policy.closure)
    contribute material[t, p, j] inflow  inlet[t].flow_term[p, j]
    contribute material[t, p, j] outflow outlet[t].flow_term[p, j]
    when has_phase_equilibrium { contribute material[t, p, j] transfer(eq[t, pair, j], side(pair, p)) … }
    when reactions? { contribute material[t, p, j] generation Σ(r in reactions.set | reactions.stoich[r, p, j]*extent[t, r]) }
    when has_holdup and analysis.dynamic {
      var holdup[t, p, j] : Amount defined by eq holdup == volume[t]*phase_frac[t, p]*outlet[t].dens_term[p, j]
      contribute material[t, p, j] accumulation d(holdup[t, p, j])/d(t)
    }
  }
  when mbt == componentTotal { … }   when mbt == elementTotal { … composition[e, j] … }
  accumulate energy[t] conservation …   when has_heat_transfer { var heat[t]: Power; contribute energy[t] heat_in heat[t] }
  port inlet = inlet[*].port;  port outlet = outlet[*].port
}
```

### 3.7 Units compose

```text
package pse.units.core
def Heater(pkg: ThermoPackage = scope.default_property_package, has_pressure_change = false, …) {
  child cv : ControlVolume0D(pkg = pkg, has_heat_transfer = true, has_pressure_change = has_pressure_change, …)
  port inlet = cv.inlet;  port outlet = cv.outlet
  alias heat_duty = cv.heat  @report("Heat Duty")
}

def Mixer(pkg = scope.default_property_package, inlets: Set<Label> = {1, 2},
          pressure: MixerPressure = minimize, eps = 1e-3{Pa}) {
  child in_state[i in inlets, t in scope.time] : pkg.state(defined = true)
  child mixed[t in scope.time] : pkg.state(defined = false)
  accumulate material[t, (p, j) in pkg.phase_components] conservation …
  contribute material[t, p, j] inflow in_state[i, t].flow_term[p, j] for i in inlets
  contribute material[t, p, j] outflow mixed[t].flow_term[p, j]
  when pressure == minimize { eq mixed[t].pressure == smooth_min_over(i in inlets | in_state[i, t].pressure, eps) }
  when pressure == equality { eq mixed[t].pressure == in_state[i, t].pressure for i in inlets }
  port inlet[i in inlets] = in_state[i, *].port;  port outlet = mixed[*].port
}

interface DeltaT { sym dT1: TemperatureDifference; sym dT2: TemperatureDifference; sym dT: TemperatureDifference }
def LMTD : DeltaT { let dT = (dT1 - dT2)/log(dT1/dT2) }
def Underwood : DeltaT { let dT = ((cbrt(dT1) + cbrt(dT2))/2)^3 }
def HeatExchanger(hot_pkg, cold_pkg, delta_t: DeltaT = Underwood, flow: FlowPattern = countercurrent) { … }

preset Pump = PressureChanger(compressor = true, assumption = PumpAssumption)
```

### 3.8 Reactions, costing and control

```text
package pse.reactions.saponification
table stoich[r: reaction, p: phase, j: species] : Dimensionless = { R1: { Liq: { NaOH: -1, EthylAcetate: -1, SodiumAcetate: 1, Ethanol: 1 } } }
require for r in reactions, e in elements: Σ(j | composition[e, j]*stoich[r, Liq, j]) == 0 : "element closure"
def ArrheniusRate(k0: RateConstant, E: MolarEnergy) : RateLaw { let rate = k0*exp(-E/(R*T))*conc["NaOH"]*conc["EthylAcetate"] }

package pse.costing.sslw
fn hx_base_cost(area: Area, kind: HXType) -> Currency@CE500
   @valid(area in hx_area_range[kind], reject) = exp(hx_coeff[kind].a + hx_coeff[kind].b*log(area/1{ft^2}) + …)
def HXCosting(unit: HasArea, kind: HXType, material: HXMaterial) : UnitCosting {
  contribute scope.costing.capital accounting convert(hx_base_cost(unit.area, kind)*material_factor[material], scope.costing.base_year)
}

package pse.control
def PID(kp, ki, kd, form: PIDForm = velocity) {
  require analysis.dynamic : "a PID controller needs a dynamic analysis"
  port setpoint: signal in;  port measured: signal in;  port output: signal out
  var integral[t in scope.time] @start(0)
  eq d(integral[t])/d(t) == error[t]
  let output[t] = bias + kp*error[t] + ki*integral[t] + kd*d(error[t])/d(t)
}
```

### 3.9 Tests are data

```text
test bt_ideal_state_solution source idaes-oracle:<test id>::<n> {
  root = StateOnly(pkg = BTIdeal)
  fix root.state.flow_mol = 100{mol/s};  fix root.state.temperature = 368{K};  fix root.state.pressure = 101325{Pa}
  fix root.state.mole_frac_comp[benzene] = 0.5
  solve square
  expect root.state.phase_frac[Vap] == <oracle value> ± rel 1e-5
}
```
