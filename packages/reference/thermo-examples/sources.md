<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Thermodynamic demonstration data

**Implemented:** explicit species, phase membership, three single-phase property
packages and shared parameter rows. Compilation and numerical qualification are
separate test gates. These sources make no multiphase equilibrium claim.

The benzene and toluene formulas are C6H6 and C7H8; nitrogen is N2. Molecular weights
are derived from the admitted element counts and the selected CIAAW element table.
No independent molecular-weight constant shadows that calculation.

The liquid package uses Perry's *Chemical Engineers' Handbook*, seventh edition,
Table 2-196, equation 1, for liquid heat capacity and Table 2-30 for liquid density.
The [chapter supplied by Carnegie Mellon](https://schneider.cheme.cmu.edu/Files/Perry%27s%20Chemical%20Engineers%27%20Handbook%207th%20Edition/2.%20Physical%20and%20Chemical%20Data.pdf)
is the checked source. Coefficient values occur only in `materials/species-and-data.yaml`.
The heat-capacity coordinates are J/(kmol K), with temperature in K; density is
kmol/m3. Benzene heat capacity is valid over 278.68–353.24 K and toluene over
178.18–500 K. Density validity extends to 562.16 K and 591.8 K, respectively; the
logarithmic authored form excludes the upper critical endpoint. The demonstration
state bounds use the common interval 298.15–350 K.

The vapor benzene/toluene data are **derived cubic interpolation examples**, not
published RPP4 coefficients. We fit `A+B*T+C*T^2+D*T^3` to four NIST heat-capacity
anchors with exact rational Gaussian elimination, then round each resulting
coefficient once to binary64. The [benzene table](https://webbook.nist.gov/cgi/cbook.cgi?Mask=2681&Source=1933FER%2FMIL194&Units=SI)
gives 82.44, 113.52, 139.35 and 160.09 J/(mol K) at 298.15, 400, 500 and 600 K.
The [toluene table](https://webbook.nist.gov/cgi/cbook.cgi?ID=C108883&Mask=7)
gives 103.7, 139.9, 170.8 and 196.2 at those temperatures. The declared interpolation
domain is 298.15–600 K. Against the unused 300 K anchors, the predicted-minus-table
residuals are +0.024736854108667217 and −0.00543169744182137 J/(mol K), respectively.
These are interpolation checks, not an uncertainty model or an extrapolation claim.

The nitrogen package uses the [NIST Shomate 100–500 K coefficients](https://webbook.nist.gov/cgi/cbook.cgi?ID=C7727379&Table=on&Type=JANAFG),
Chase 1998 with the January 2009 fit. Its state bounds select 298.15–500 K.
The entropy reference is the [CODATA 1984 value at 1 bar](https://webbook.nist.gov/cgi/cbook.cgi?Mask=1A8F&Source=1966ART2),
including its reported uncertainty.

All packages share the explicitly declared 298.15 K, 100000 Pa quantity reference.
Formation enthalpy is excluded: the ideal-gas species enthalpy at that reference is
zero. Liquid reference enthalpy is the liquid-minus-gas formation-enthalpy difference,
using paired Prosen/Gilmont 1945 values in the NIST tables: benzene 49.04−82.93 and
toluene 12.0−50.00 kJ/mol. Liquid standard entropies come from
[NIST benzene](https://webbook.nist.gov/cgi/cbook.cgi?Mask=20F&Source=1974JAC49-52)
and [NIST toluene](https://webbook.nist.gov/cgi/cbook.cgi?Mask=78F&Source=1951RAM%2FBER212-214&Units=SI).
The gas constant is the exact product of the SI defining Avogadro and Boltzmann
constants from the [BIPM SI Brochure](https://www.bipm.org/en/publications/si-brochure).

Each parameter is owned by its actual property package and indexed by an explicit
species ID where required. The cp/h/s methods share exact declared coefficient
names. A prefix or a display name never chooses data. State bounds are authored
inputs; subsequent numerical execution must honor each selected dataset's domain.
