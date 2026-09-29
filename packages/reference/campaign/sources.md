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
