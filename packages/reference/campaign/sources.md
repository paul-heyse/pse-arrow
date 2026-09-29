<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Integrated campaign

These flowsheets and studies exercise the modeling kernel through the reference
packages it depends on. Passing one authored fixture establishes only its named
properties and conditions; each fixture names its oracle.

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
