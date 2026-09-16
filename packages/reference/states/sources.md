<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Steady single-phase states, version 1.0.0

FTPx declares total molar flow, temperature, absolute pressure and component mole
fractions. FcTP declares component molar flows, temperature and absolute pressure;
its total flow is their contraction, with mole fractions related by component
flow equations. The package carries the exact one-phase/nonempty-species
constraint on the actual selected material system, not a caller-supplied count.

These are clean-room state-coordinate choices following blueprint §9.2. Their
algebra is material accounting: component flow is total flow times composition,
and an undefined state requires composition closure. The `defined_state` guard
retains the inlet/outlet distinction. Mole-fraction defaults are the declared
compatibility interval [1e-20, 1.001]. Positive temperature and pressure bounds
exclude nonphysical absolute coordinates. Initial values are suggestions, not
equations or property data.

Every state exposes material flow by phase/species, enthalpy flow by phase,
material density by phase/species and internal-energy density by phase. The
internal-energy identity is `u = h - P/rho`; transported enthalpy and internal
energy retain the explicit component reference convention. Property methods
must provide the actual enthalpy and density requested by these expressions.
Scalar flow and pressure, and quantities with fewer axes, are explicitly broadcast
over each missing phase/species binder. The same declarations therefore support
indexed law and inlet-collection contexts as well as fixed state coordinates.

The selected property package is an explicit required parameter. A relative
`self` scope is realized per actual state instance; a global set of all states
cannot stand in for it. Defaults state molar flow basis, component-total material
balances and total-enthalpy energy balances. No multiphase flash or transient
execution is claimed by these declarations.
