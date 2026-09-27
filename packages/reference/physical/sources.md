<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Physical data, version 1.0.0

The package documents are the declaration authority. The Arrow-free quantity and
material fixtures are generated from their admitted rows (blueprint §6.15.7). Scientific
checks use the sources below independently of regeneration equality.

The seven SI base dimensions and coherent products follow the
[BIPM SI definitions](https://www.bipm.org/en/measurement-units) and
[base-unit table](https://www.bipm.org/en/measurement-units/si-base-units). Currency
is an optional eighth model axis; this physical package assigns it no unit or
conversion. The separate fixture-currency package is synthetic test data.

Temperature coordinates follow [NIST SP 811, Appendix B.8](https://www.nist.gov/pml/special-publication-811/nist-guide-si-appendix-b-conversion-factors/nist-guide-si-appendix-b8).
Kelvin and Celsius increments have equal size; a Celsius point has a 273.15 K
offset. A Fahrenheit increment is 5/9 K, and a Fahrenheit point has the corresponding
273.15 − 32(5/9) K offset. An affine point and a temperature difference remain
different quantity contracts.

The psi multiplier follows the same NIST table. Its stored precision is qualified
against the defining pound, standard gravity and inch, using
0.45359237 × 9.80665 / 0.0254² Pa. The `psig` spelling restricts its datum to the
declared pressure reference. The gauge-to-absolute conversion adds that explicit
101325 Pa datum once; the representation-unit multiplier adds no datum.

The reference temperature 298.15 K and pressure 101325 Pa are explicit package
choices. They are not implicit universal standard conditions. Formation enthalpy
is excluded from this datum. Methods needing a different datum must declare it.

Stock thermochemistry has a separate explicit ideal-gas datum at 298.15 K and
100000 Pa. Its component sensible enthalpy is zero in the ideal gas at that
temperature; liquid component reference enthalpy must include the phase shift.
The gauge-pressure datum remains a distinct declared reference. Complete stock
enthalpy, entropy and transported-energy quantity types carry the selected
reference; an equal dimension or unit spelling cannot exchange the two.

Element masses use the selected engineering subset of the
[CIAAW abridged standard atomic weights, 2024](https://www.ciaaw.org/abridged-atomic-weights.htm).
The source values in g/mol are converted to kg/mol. These abridged values describe
normal terrestrial materials; isotope-enriched materials need their own declarations.
The selected table is not an isotope database or a complete periodic table. Source
uncertainties, propagated independently in molecular-weight tests, are not hidden
inside a hash or inferred from the number of decimal places.

Quantity kinds, bases, subjects, point/difference contracts and operation policies
are explicit modeling declarations under blueprint §6.2 and §8.3. Matching units
or dimensions cannot supply a missing physical kind. Operation preconditions must
be proved against the actual operands at application time.

Counts and indicators are dimensionless pure numbers with a declared category
(ADR-0103): `count` types an integer decision (units in operation, trays, modules)
and `indicator` a zero-or-one state. The category, not the dimension, is what admits
an integer or binary variable domain; the neutral scalar, mole fractions and other
dimensionless measured kinds carry none. Multiplying a quantity by a count or an
indicator keeps the quantity's complete contract (units in operation times a per-unit
capacity is a capacity), in the same way neutral scaling does.

Mixture normalization is explicit. The enthalpy expression is
`h0 + sum(x_i * (h_i - h0)) / sum(x_i)`, so only the enthalpy difference is
divided by the dimensionless mole-fraction weight. Its registered division preserves
the difference's basis, reference, scale and subject. Its prerequisite matches the
weighted operation's 1 bar thermochemistry datum and rejects points, component
subjects and foreign datums.
The harmonic density expression is `sum(x_i) / sum(x_i / rho_i)`; its division
requires the declared fraction and molar-volume contracts. A reduced fraction
retains its declared physical kind instead of silently becoming the neutral type.

Energy density is the product of molar density and molar enthalpy:
`(mol/m³) * (J/mol) = J/m³`. Its registered operation requires the actual common
molar basis, cancels that basis in the result, and retains the enthalpy datum.

The extensive component-flow `SumOver` contraction explicitly selects a species axis. A phase reduction is a different structural operation; failed physical preconditions cannot select another operation implicitly.

The stock caloric derivative contracts divide an enthalpy **difference** at the
declared 1 bar datum by a temperature difference to obtain molar heat capacity.
The operation checks both complete input contracts before removing the enthalpy
datum from the result. It does not authorize dividing an enthalpy point or a
foreign reference by a temperature interval.

Element projection records explicitly distinguish elemental amount flow from
species amount flow. The coefficient for a molar input is the actual atom count
(mol of element per mol of species); for a mass input it is that count divided by
the actual positive molecular weight declared in kg/mol, yielding mol/kg. These
are the elemental conservation identities in blueprint §10, not fitted scientific
coefficients. The registered multiplication operations check each input's full
quantity contract and produce the declared Element/molar result. Intermediate
indexed types preserve every explicit broadcast and Species/Phase reduction.

Caloric normalization is declared explicitly for the stock heat-capacity contract
and enthalpy/entropy differences. Equal dimensions alone do not authorize
these ratios. Enthalpy and entropy normalization accept differences with one common admitted
reference, retaining the prototype's kind, basis and subject constraints; it rejects
mixed references and points. This is not a datum conversion.
The enthalpy derivative operation likewise requires the stock enthalpy difference
and a temperature difference. Point enthalpy and a foreign datum are rejected.

The homogeneous-phase total molar density has no component subject. Component
amount, total density and absolute-pressure normalization each require their
selected complete contracts before producing a neutral mathematical coordinate.
These operations do not grant dimension-based relabeling to other quantities.

Temperature-scale, temperature-difference and mole-fraction ratios likewise require
their declared contracts. Smoothing normalizes a temperature difference by its
positive difference width, then restores that width before adding to an affine point.
It does not require an artificial squared-temperature quantity.

The BTIdeal oracle has a separate formation-inclusive enthalpy datum at 300 K and
1 bar. Its point and difference types cannot be used as stock sensible enthalpy.
The seed binding owns the explicit component-dependent reference shift using its
source formation values and ideal-gas caloric increments.

The BT_PR oracle separately declares formation-inclusive enthalpy and entropy at
298.15 K and 101325 Pa. Its source values are never labeled as stock sensible
properties. Authored conversion uses component reference values and the ideal-gas
entropy pressure correction.

Finite sums retain the consumed entity kind after scalar enumeration. The component
flow and concentration contractions consume `species` and produce a total with no
component subject. Their prerequisites compare physical axes independently of the
enumerated shape; the reduction itself checks the domain and remaining free indices.
Empty and fully filtered sets retain the same prototype checks.

Registered operations may have disjoint complete-contract prerequisites for the same
operand kinds. Exactly one proved declaration must apply; overlap and failure of all
prerequisites are refusals. This admits absolute-pressure and pressure-difference
normalization separately, and component versus total concentration separately, without
allowing cross-contract ratios. Reaction coefficients and volumetric rates have explicit
SI dimensions and reaction subjects. Signed reaction heat has a natural zero independent
of a species enthalpy datum.

`smooth_min_over` folds the existing two-value smooth minimum over a nonempty set of
ports in admitted membership order. The smoothing width is a positive difference
quantity. A singleton returns its value; an empty selection refuses because no physical
minimum exists. `fold(acc, value; i in members | input[i]; step)` binds `acc` and `value`
only in the step and checks the step's complete output type. Library differentiation
operates on the resulting shared expression bindings.
