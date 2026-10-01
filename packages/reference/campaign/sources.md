<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Integrated campaign

These flowsheets and studies exercise the modeling kernel through the reference
packages it depends on. Passing one authored fixture establishes only its named
properties and conditions; each fixture names its oracle source and each dataset its
source and role (ADR-0123 Outcome 5).

`models/references.pse` (module `campaign_references`) declares the sources the
campaign cites beyond the seed: the IDAES 2.13 `test_pfr` TestInitializers and
`test_flash` TestInitializersCubicModularBTX oracle tests, and the analytic properties
the studies check (backward-difference and Radau convergence order, the steady-state
limit of a stable CSTR and the tangent-plane stability of a phase in equilibrium). The
IDAES 2.13 release, the `test_cstr` oracle and the SciPy CSTR/controller reference are
the seed's `references` entities. The IDAES inputs and printed results used as data are
`oracle_input`, so only test fixtures read them.

`models/cstr-dynamics.pse` solves one `reactors.CSTR` definition steady and dynamic.
The control volume generates its holdup rates over the instant at which a flowsheet
replicates it along its time domain; the flowsheet states only boundary conditions,
initial conditions and control. The controlled fixtures compare samples with an
independent reduced incompressible CSTR/controller ODE (SciPy 1.18.1 DOP853 and Radau,
rtol 1e-11, atol 1e-12). The open-loop fixture integrates from a feed-filled start
at zero duty for 40 residence times and compares the terminal state with the steady
solve of the same definition, whose IDAES 2.13 `test_cstr` comparison is the seed
fixture `cstr_saponification`.

`models/pfr-studies.pse` holds the saponification PFR at the IDAES 2.13 `test_pfr`
TestInitializers specification (1 m³/s, 100 mol/m³ of each reactant, no inlet products,
303.15 K, 0.5 m by 0.1 m², adiabatic and isobaric). `pfr_initializers_oracle` compares
20 backward elements with its outlet values at relative 1e-5 and with the
TestSaponification conservation and heat-of-reaction checks. The refinement studies solve
four meshes in one flowsheet and check the observed order of the outlet ethyl acetate
against declared bounds: backward differences over 5, 10, 20 and 40 elements (first
order), and three-point Radau over 1, 2, 4 and 8 elements (endpoint order five). The
Radau differences reach the solver tolerance by eight elements, so a finer Radau study
measures round-off rather than order.

`models/bt-pr-formulations.pse` solves one flowsheet, a Flash of the BT_PR binding at
the IDAES 2.13 `test_flash` TestInitializersCubicModularBTX conditions (1 mol/s
equimolar benzene/toluene, 368 K, 101325 Pa, zero duty and pressure change), under each
phase-equilibrium formulation the binding's `formulation` parameter selects: the smooth
bubble/dew formulation, the complementarity closure `pr_equilibrium.ComplementarityPR`
(equilibrium temperature slacks complementary to the phase fractions, smooth
realization), and the nested flash `nested_equilibrium.NestedPRFlash` (two-phase,
liquid-only and vapor-only regimes solved on compressibility roots and selected by
eligibility and the minimum Gibbs energy of the split). Each compares with the test's
outlet flows, compositions, temperature and pressure at its absolute 1e-3.

`models/exchanger-costing.pse` costs the IDAES 2.13 `test_heat_exchanger`
TestBT_Generic_cocurrent exchanger (BT_PR on both sides, 5 mol/s at 365 K against 1 mol/s
at 300 K, 100 W/(m² K)) from its solved area. The flowsheet binds the SSLW costing's area
and tube-side pressure to the exchanger's area and hot inlet pressure and aggregates its
capital cost. The fixture holds the cold outlet at the test's 596.9 °R and leaves the area
free, so the solve recovers the test's 1 m² within the 0.003 m² its outlet tolerance
allows. The expected costs are the SSLW U-tube, stainless/stainless, 12 ft correlation
evaluated by hand at 1 m² and 0 psig; 1 m² lies far below the exchanger sizes the
correlation is published for, so the fixture checks the costing's reading and arithmetic,
not a purchase price.

`models/recycle-flash.pse` holds `RecycleFlash`: Feed, Mixer, Heater, Flash and a liquid
Separator on the BTIdeal package, whose flash liquid is split between a purge and a recycle
to the mixer. The heater sets the flash temperature; the flash is adiabatic and isobaric.
The heater's runtime `target` parameter and splitter's runtime `purge` parameter
own those local specifications. `RecycleFlash` retains its `temperature` and
`purge_fraction` constructor inputs; the loop exposes their unit values as observations.
The mixer states no pressure relation (`MomentumMixingType.none`): a closed isobaric loop
has no minimum inlet pressure to select, and the loop runs at the feed pressure. The
recycle closure is one named material connection. The `tear` stage replaces its source
with a small positive recycle at the feed's state; the original connection then closes the loop.
`recycle_flash_conditional_handoff` owns the original 1 mol/s equimolar fresh-feed
specification for the typed request fixture, retaining the BTIdeal parameters' test-only
provenance. Its request selects the named recycle connection and four conditional unit
procedures; coupled execution remains a separate campaign qualification.
`recycle_flash_converges_from_defaults` solves four synthetic feeds (0.5 to 50 mol/s,
300 to 400 K, 0.45 to 0.6 benzene) with 90 % of the liquid recycled, from start annotations
and the tear stage only, and compares with the analytic steady state: the vapor product is
the equilibrium flash of the fresh feed at 368 K and 101325 Pa (the IDAES test_BTIdeal
phases, the lever rule for V/F), and the recycle is (1 - purge)(F - V)/purge.
`recycle_flash_failure_restores_specification` selects a tear stage whose 500 K recycle
estimate exceeds the BTIdeal temperature bound: the stage fails, nothing is committed, and
the unchanged specification solves from its own starts to the same steady state.

`models/flash-diagnostics.pse` holds the diagnostics scenarios. `flash_overspecified`
fixes the outlet temperature of the IDAES `test_flash` BTIdeal flash in addition to its
inlet, duty and pressure change: one equation too many. The root solve is refused
structurally before any route is chosen (`invalid_model`, `native.structural`), naming the
over-determined equations. `EquilibriumCascade` is a minimal cascade of ideal equilibrium
stages with a total condenser and a total reboiler under constant molar overflow; its stage
set and the links between adjacent stages are its arguments. Two stages with 100 mol/s
boilup, fed 1e-6 mol/s and drawing half of it as distillate, run near total reflux: the two
stages' balances of each component are near-parallel and the Jacobian is numerically rank
deficient under the `idaes-2.13` thresholds, and
`cascade_near_singular_diagnostics_name_members` expects both findings naming those
balances. Its state is compared with the analytic total-reflux limit (the reflux is the top
vapor, the boilup the bottom liquid, and y1 + x2 = 1 for benzene).
