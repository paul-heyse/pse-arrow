<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Authored process models

The process models use the generic checked language and shared indexed balance operations.
Focused Plan 25c controls establish its migrated composition and conservation contracts;
assembled native and Python journeys remain Plan 25k qualification. Earlier fixture evidence
retains its original scope and does not qualify every new configuration.

Control-volume material equations sum signed component molar transport. The energy balance
uses the state's declared molar-enthalpy reference, with heat and work positive into the
volume. Outlet pressure equals inlet pressure plus the declared pressure change. The
zero-dimensional contract selects component-total, phase-component, element-total or
total-molar balances. Element balances use declared atom counts and an element-flow
quantity; the selected basis determines closure, not an extra set of redundant equations.
`IndexedBalances` owns the indexed transport/source sums shared by control volumes,
Mixer and Separator. Optional holdup exposes phase-component amount and phase energy
inventories. A dynamic stored configuration relates these inventories to the original signed
flux through `conserve`; memoryless units keep algebraic balances during dynamics. The
current stored configuration requires a single-phase allocation and component-total or
component-phase inventory basis. A concrete unit owns its constitutive inventory equations;
element-total and total storage need an explicit inventory projection before admission.

Feed and Product expose one independent state specification through a material port. Heater specializes the
control volume with zero mechanical work. Mixer admits an indexed inlet set, sums component
and enthalpy transport, and sets its pressure with the authored finite smooth-min fold.
The smoothing parameter is an explicit pressure difference. The inlet states are defined;
the outlet property binding owns its reconstruction once.

Flash uses the same control-volume balances. Its outlet child refines the thermodynamic
interface to expose the equilibrium split. Liquid and vapor ports refer directly to that
state's independent phase component flows, temperature and pressure, with original energy
transport observations and without creating duplicate state models.

Separator partitions an inlet's phase-component inventory by total, component, phase or
phase-component fractions. Fractions sum to one over the declared outlets. Each outlet's
molar flow and composition are derived from the partition. Energy is the sum of
component enthalpy transport, supplied by an explicit additive phase-caloric function.
This prevents selective component splits from incorrectly retaining the inlet mixture's
molar enthalpy. The supplied function must reproduce the inlet energy on the selected
state; the energy accounting check rejects inconsistent bindings. BTIdeal supplies ideal
component calorics; Saponification supplies its solvent-only caloric approximation.
Nonadditive mixture calorics require a different explicit energy strategy and are not
qualified by this partition model. It preserves the inlet's phase inventory and thermal state; connecting the
result to another equilibrium package is an explicit subsequent model operation. Composition
is defined only at nonzero outlet flow. Shared material and energy observation accumulators require physical closure and verify the
partition independently without adding redundant conservation equations to the fraction
normalization. No holdup is claimed for this instantaneous partition.

Named numerical comparisons reside in `pse.seed-data/models/unit-fixtures.pse`. They retain
inputs and printed expectations from the IDAES tests each fixture names as its oracle
source. The Saponification separator's upstream volume-flow expectations are
converted to molar flow using its supplied total concentration. These are selected behavioral
comparisons, not a claim that every IDAES option or property package is reproduced.
Equations are independently expressed from conservation and phase partition contracts;
upstream code, comments and docstrings are not copied.

Pressure-change assumptions own their required state capabilities and the common energy
balance. The liquid pump form uses volume flow times pressure change. Compression divides
reversible work by efficiency; expansion multiplies it. The isentropic form requires an
entropy-capable reference state at outlet pressure and inlet entropy. Presets bind these
choices; no native dispatch recognizes unit names.

The spatial control volume uses a normalized axial coordinate. Its distributed sources
therefore represent the contribution across the full physical length. Backward difference
uses right-endpoint quadrature, consistent with its cell balances. Right Radau uses the
kernel's library-provided collocation and quadrature. PFR reaction sources use total volume,
local concentration and temperature. The independent continuum reference integrates
`dc/dV = -k(T)c²/q` with `T = Tin + (-ΔHr)(cin-c)/(rho_solvent cp_solvent)`; the seed's
`pfr_reference` computation agrees at V=0.05 m³, q=1 m³/s, cin=100 mol/m³ and
Tin=303.15 K: c=61.74340062 mol/m³ and T=303.599299658 K.
This reference is distinct from IDAES's 20-element backward-difference comparison.

SSLW heat-exchanger purchase costing represents one exchanger; indexed costing children
compose multiple purchases through the capital accumulator. The seed's coefficient
datasets name the correlation's publication as their source. The campaign costs the BT_PR
co-current exchanger from its solved area and tube-side pressure (`hx_costing_solved_area`);
the seed evaluates at a stated 1000 m² area. The source supplies no area applicability
range, so the seed and campaign name permission for that unknown evidence. The pressure
factor's reported range is 100–2000 psig. The campaign exchanger at 0 psig lies below it and
its flowsheet names a separate extrapolation permission. Applicability observations retain
the unknown/outside outcomes and the selected permissions. USD_CE500 and
USD_2018 use CEPCI 500 and 603.1 in the physical unit registry; year conversion is not a
second correlation multiplier in the model.

The dilute-liquid CSTR's optional inventories are `N[j] = V c[j]` and
`U = V rho_total (h - h_datum) - V (P - P_reference)`, with the stock 1 bar
reference. The existing solvent caloric state owns the enthalpy calculation. Its dynamic
fixture indexes this same CSTR definition over a single time domain and selects integrated
or simultaneous realization; neither unit equations nor kinetics are repeated by mode.
For the fixed-volume incompressible fixture, solvent accumulation is zero; the four solute
inventories and energy are differential states with explicit inventory initial conditions.

The Controller contract exposes normalized measured, target and manipulated signals.
PID uses proportional and integral action, filtered derivative on measurement, smooth
lower/upper limits and back-calculation anti-windup. In normalized coordinates:
`raw = bias + Kp (error + I/Ti - (Td measurement - F)/Tf)`,
`dF/dt = (Td measurement - F)/Tf`, and
`dI/dt = error + (output - raw) Ti/(Kp Taw)`.
The integral/back-calculation form corresponds to IDAES 2.13's gains
`gain_i = Kp/Ti` and `gain_b = 1/Taw`. The positive derivative filter is an explicit
addition; the upstream unfiltered derivative is not claimed as the same model.
Initialization values, smoothing and all time constants are supplied data.

The saturation fixture uses a constant unit error, `Kp=2`, `Ti=1 s`, `Taw=0.5 s`,
zero derivative action and `I(0)=1 s`. With the upper limit active the analytic recovery is
`I(t)=exp(-2t) s`; its tolerance includes the declared smoothing. The controlled CSTR
reference, the seed's `cstr_controller_reference` computation, independently integrates
concentration, temperature, integral and filter states. For V=0.0015 m³,
q=0.001 m³/s, inlet T=303.15 K and concentration 100 mol/m³, the reduced equations are
`dc/dt = (q/V)(cin-c) - k(T)c²` and
`dT/dt = (q/V)(Tin-T) + (Q - V ΔHr k(T)c²)/(V rho_solvent cp_solvent)`.
The initial state is the zero-duty steady CSTR fixture; controller target is 305 K,
heat is 10000 W times normalized output and the filter starts at `Td (T(0)-300 K)/(1 K)`.
Frozen comparisons at 0, 1, 5 and 20 seconds live in the authored fixture. Simultaneous
checks compare only matching mesh coordinates; they do not assert an interpolation oracle.
The selected integrated and simultaneous CSTR and both PID realizations pass their comparisons.
The simultaneous CSTR uses a 12-element, order-three Radau mesh. Integration tolerances are tighter than the
comparison tolerances so the original algebraic residual checks also meet their budgets.

The homogeneous vessel composes an explicitly supplied residual potential and component
ideal-gas enthalpy function. Total amount has its own physical contract, distinct from a
component inventory. For fixed composition, `N = rho V`, `U = N h - P V`,
`dN/dt = Fin - Fout`, and `dU/dt = Fin hin - Fout h + Q`. The selected DIPPR primitives
use the sensible 298.15 K datum; initial values retain the frozen homogeneous PC-SAFT
comparison. The directional valve uses the shared verified C² characteristic with a
declared pressure width and flow at that width. Its parameters are data, not a native
factory. Authored conservation descriptors compare the original amount and energy changes
to independently accumulated original fluxes in integrated and simultaneous realizations.
Focused original-space controls exercise the replacement; the bespoke vessel closure copies
are removed. The accumulated amount/energy observations remain for their continuing authored
fixture comparisons. Complete native scientific journeys remain Plan 25k qualification.
