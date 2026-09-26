<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Steady reusable unit declarations

**Implemented:** ordinary authored templates for a flowsheet, lumped control
volume, heater, feed, product, mixer, state junction and equality connection.
Compiler and numerical acceptance are separate named gates.

These are clean-room conservation declarations. A control volume receives and
sends molar species flow and transported enthalpy through its selected state
interfaces. Heat and work enter the signed energy balance only when their declared
guards are true. Pressure equality, pressure change and isothermal conditions are
explicit equations. No external implementation or model-class name is consulted.

Material `componentTotal`, `componentPhase` and `elementTotal` routes have separate
index contracts. Element conservation obtains element counts from the actual
`species_elements` rows. It is not a relabeling of a species quantity. Enthalpy
conservation has the declared energy subject and datum. The default balance choice
is resolved from the actual selected representative state. An absent law binding
is unsupported; notably, the control volume has no total-material binding.

The selected state interfaces must have exactly one phase and at least one species.
The templates preserve ordered species and inlet domains. A heater delegates its
ports and heat duty to its control-volume child. A mixer receives a finite explicit
inlet-domain ID parameter. Its pressure equation explicitly broadcasts the common
mixed-state pressure over that inlet domain. A state junction exposes the same selected state on both
ports. Equality connections compare corresponding typed state quantities and
composition members. Declared state port-member order is authoritative.

Steady holdup has an explicit positive volume and guarded material/energy inventory
expressions from the actual outlet density interfaces. Volume-times-density operations
prove complete input quantity contracts and retain the appropriate species, basis and
enthalpy datum. No time derivative is silently set to zero.

Dynamic behavior is excluded by an explicit feature rule. Numerical solving,
phase equilibrium, initialization and transport correlations are outside this
steady symbolic package. See blueprint §6.15 and §10.2 for the selected
scope and compilation obligations.

The law contracts name the exact representative child (`properties_out` for the control volume and `mixed_state` for the mixer) and its declared default feature. P8 uses these bindings only for `useDefault`; an explicit balance choice does not consult a default. A selected collection must agree on the same typed feature value.

The explicit `connection_bindings` row for template `b85e948565bb46db98cddceae395febc` selects equality expansion over its actual matched port members and complete physical types. The template name does not select runtime behavior.
