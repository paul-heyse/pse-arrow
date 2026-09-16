<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Stock methods, version 1.0.0

The expression documents are the formula authority. Coefficients are supplied by
the selected material package through typed parameter rows. They are not embedded
in these templates. Each coefficient is a dimensionless coordinate in the natural
unit system written explicitly in the expression. This representation preserves
the physical output kind, basis, subject, index shape and reference state.

NIST Shomate uses `t = T / 1000 K`. Its heat-capacity form is documented by the
[NIST Chemistry WebBook](https://webbook.nist.gov/cgi/cbook.cgi?ID=C7732185&Mask=1883&Units=SI).
The enthalpy and entropy expressions here are definite integrals of that heat
capacity from an explicit `T_ref`, plus explicit `h_ref` or `s_ref`. Constants of
integration in an indefinite published formula cancel in the definite difference;
they are not inferred or silently set to zero. The heat-capacity unit is J/(mol K),
while the Shomate enthalpy difference multiplier is kJ/mol.

The RPP4 form is the cubic heat-capacity polynomial in `T / 1 K` from Reid,
Prausnitz and Poling, *The Properties of Gases and Liquids*, fourth edition,
McGraw-Hill, 1987, ISBN 9780070517998. Its h and s forms follow directly from
integrating `cp dT` and `cp dT/T`. The method accepts declared coefficients; using
this algebra does not assert that a particular dataset came from that book.

The Perry liquid polynomial is equation 1 of table 2-196, and the liquid-density
correlation is equation 1 of table 2-30 in Perry and Green, *Perry's Chemical
Engineers' Handbook*, seventh edition, 1997. The
[source chapter](https://schneider.cheme.cmu.edu/Files/Perry%27s%20Chemical%20Engineers%27%20Handbook%207th%20Edition/2.%20Physical%20and%20Chemical%20Data.pdf)
uses J/(kmol K) and kmol/m³, respectively. Polynomial h and s are definite
integrals with explicit reference values. The density expression uses an
equivalent exponential/logarithmic form on the declared domain `B > 0`,
`T < C`; it does not extend the real-valued correlation through its critical
endpoint. Each coefficient dataset must supply its actual temperature interval.

Ideal gas density follows `P = rho R T`. Ideal liquid volume is mole-fraction
weighted molar volume. The enthalpy mixture expressions subtract an explicit
point of the common datum, average the resulting differences, and restore the
point. Dividing by the actual composition sum makes the normalization explicit.
The mixture has no excess or departure contribution. Different component datum
contracts require an explicit conversion before averaging.

The shared thermochemical datum is an ideal-gas enthalpy zero at 298.15 K and
1 bar, with formation enthalpy excluded. A liquid component's `h_ref` therefore
includes its actual phase shift. Entropy uses supplied standard component values;
the enthalpy zero does not determine entropy. Method output signatures carry the
full reference identity and natural unit. A representation conversion never
supplies a missing datum.

Implemented source declarations do not establish numerical solver execution,
derivative accuracy or IDAES equivalence. Formula tests must exercise actual
parsed expressions and independently sourced values or thermodynamic identities.

Each equation-template correlation explicitly binds its requesting state through `method_state_parameters`; `state` is the authored parameter key in these templates. Realization follows this relation rather than assigning meaning to the parameter spelling.
