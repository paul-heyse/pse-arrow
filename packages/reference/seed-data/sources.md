<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Modeling seed data

These declarations implement the Plan 21 seed. Passing one authored fixture
establishes only its named properties and conditions.

`models/chemistry.pse` declares the seed's neutral chemical formula units and their
atom counts. The saponification reaction uses ethyl acetate and sodium hydroxide,
producing sodium acetate and ethanol; water is the solvent. Elemental closure is
checked independently for each admitted element. This formulation makes no ionic
speciation claim. Molecular weights are calculated from atom counts and the
[CIAAW 2024 abridged atomic weights](https://www.ciaaw.org/abridged-atomic-weights.htm).
All 21 previously shipped elements are retained as entities with authored masses;
the formula-weight fixture checks their use through the generic table contracts.

`models/caloric.pse` carries the nitrogen Shomate fit from the
[NIST Chemistry WebBook](https://webbook.nist.gov/cgi/cbook.cgi?ID=C7727379&Table=on&Type=JANAFG),
Chase 1998, January 2009 parameter fit, over 100–500 K. Its coefficients use the
published reduced coordinate T/(1000 K); heat capacity is J/(mol K). The primitive
functions produce enthalpy and entropy differences at the stock ideal-gas datum.
Reference subtraction removes arbitrary integration constants. No formation
enthalpy or entropy reference is inferred from those primitives. Tests use NIST's
rounded heat capacities and enthalpy increment, with tolerances matching the
printed resolution, and check the enthalpy derivative separately.

The reusable equations and row contracts live in `pse.methods`, in
`models/caloric.pse` and `models/pure-properties.pse`. Fit identities distinguish
liquid, vapor and alternative datasets for one species. The common polynomial
implements RPP4's cubic, Perry's quartic and DIPPR100 through declared coefficients
and a physical coefficient scale. Both enthalpy and entropy primitive derivatives
are checked; entropy uses the explicitly dimensionless Kelvin coordinate for its
chain-rule comparison. The caloric interface consumes those primitive evaluations
and owns reference subtraction. Reference values are supplied, never inferred.

Perry liquid heat capacity and density use the seventh-edition tables 2-196 and
2-30 from the [source chapter](https://schneider.cheme.cmu.edu/Files/Perry%27s%20Chemical%20Engineers%27%20Handbook%207th%20Edition/2.%20Physical%20and%20Chemical%20Data.pdf).
Their natural units are J/(kmol K) and kmol/m³. Benzene heat-capacity tests use the
printed endpoint values and their resolution. The density form excludes its
critical endpoint. Each declared interval belongs to that fit.

The entries named `rpp4_oracle`, `rpp4_pressure_oracles` and `water_density_oracle`
retain numerical inputs from the IDAES 2.13.0 `BT_ideal`, `test_RPP4` and
`test_Perrys` comparisons. They are oracle inputs, not independently certified
production datasets. Their bounds identify the selected comparison interval, not
a claim about the publication's full empirical validity. The water checks use
the original oracle temperatures and tolerances: Perry density at 273.16 and
333.15 K; RPP4 saturation pressure at 298.15 and 373.15 K. Equations are expressed
from the polynomial, definite-integral and reduced-temperature Wagner forms.
No upstream implementation, comments or docstrings are copied.

The methane/ethane/propane DIPPR100 records retain the values from FeOS 0.10.1
[`poling2000.json`](https://github.com/feos-org/feos/blob/c658aeab484f7a7096bfbf5425e40effd60da167/parameters/ideal_gas/poling2000.json).
The 298.15–500 K interval is the retirement comparison scope; an independent
empirical envelope is not asserted. The existing frozen Decimal integration
values in `tests/fixtures/plan14/thermo-reference.json` provide a separate caloric
oracle. FeOS attribution remains in the repository's data provenance. The production
provider records and constructor have been retired; the authored datasets now supply
the vessel and thermodynamic definitions.

Existing benzene/toluene cubic interpolation coefficients remain demonstration
fits and are not relabeled as RPP4 source data.

The PR fixtures retain both the BT mixture and the fictitious-component inputs in
IDAES 2.13 `test_ceos_PR`. The nonassociating methane/ethane/propane PC-SAFT data and
five frozen teqp comparisons use the sources and conventions documented in
[`pse.thermodynamics`](../thermodynamics/sources.md).

`bt-ideal.pse` binds the ideal log-fugacity, bubble/dew and caloric state to the
IDAES 2.13 `test_BTIdeal` and `test_BTIdeal_FPhx` comparison inputs. Its liquid
oracle coefficients retain the source's J/(kmol K) scale, including the small
heat capacities resulting from those inputs. They are distinct from published
Perry coefficients. `oracle_h` carries the formation-inclusive 300 K reference;
the binding subtracts the supplied component formation values and accounts for
the 298.15–300 K ideal-gas increment to obtain the stock sensible `h`. This shift
depends on composition and is applied in both state parameterizations. The
original 47297 J/mol FPhx specification is applied to `oracle_h`.

`bt-pr.pse` retains the 368 K, 100 kPa, equimolar BT_PR comparison from
IDAES 2.13 `test_BT_PR`. Its source enthalpy and entropy carry an explicit
formation-inclusive reference at 298.15 K and 101325 Pa. Mixture conversion to
that reference includes component formation values and the ideal-gas pressure
entropy shift from the stock 100 kPa datum. The homogeneous potential supplies
residual properties; separate phase instances prevent a smoothed equilibrium
temperature from being used as the caloric temperature. The printed oracle
rounding is reflected in explicit physical tolerances. The steady fixture passes
its selected oracle, derivative and envelope checks. The separate ideal-K
initialization fixture passes both the warm-up and original-specification solves
in a focused native check.

`saponification.pse` binds IDAES 2.13's supplied dilute-water and reaction inputs:
55388 mol/m³, 75.327 J/(mol K), a 298.15 K sensible reference, an Arrhenius factor
3.132e6 m³/(mol s), activation energy 43000 J/mol and reaction heat −49000 J/mol.
These are upstream demonstration parameters, not a new experimental fit. The seed
declares a 298.15–323.15 K envelope. Concentrations represent neutral formula units;
the reaction's stoichiometry preserves each declared element.
The 303.15 K reference values are independently evaluated from those inputs.
The energy comparison follows the upstream solvent-only approximation, whereas
total molar flow and mole fractions count every component. This distinction is
explicit in the shared-state binding. The packet records the scope of native checks
actually exercised; these source notes make no broader qualification claim.

The PR heater and cocurrent heat-exchanger fixtures retain their IDAES physical
specifications, 0.0001 K equilibrium smoothing widths and original oracle tolerances.
Their hot inlet bubble-point initial estimate is 364 K, separated from the fixed
365 K inlet temperature. This is a starting iterate, not a supplied solution or a
changed physical specification. It avoids centering the forward derivative sample
on the narrow smoothing transition; reducing the perturbation excessively instead
introduces floating-point cancellation. The default 1e-6 normalized perturbation and
1e-4 derivative tolerance remain in use.

`price-taker.pse` is a synthetic mixed-integer demonstration, not market data. A
generator commits an on/off indicator per period (`var on[t in periods]: Indicator in
binary`) and dispatches between 40 W and 100 W while on, under a 150 W summed budget;
each committed period costs 50 W of standby. Dimensionless price weights −0.5, 0.8 and
1.2 value the dispatched output. Enumerating the eight assignments gives the optimum:
only the third period runs, at 100 W, for a 70 W margin. The linear relaxation reaches
85 W with a half-committed second period, so the fixture checks that integrality is
enforced rather than relaxed. It runs only under an optimization intent; a root solve
refuses the free indicators (ADR-0103).

`gdp.pse` is a synthetic generalized disjunctive program, not equipment data. A supply
of at least 60 W is met by exactly one alternative: a small unit (up to 50 W, cost
10 W + 0.2·output), a large unit (up to 120 W, cost 30 W + 0.1·output) or staying idle.
Enumerating the alternatives gives the optimum: the large unit at 60 W, cost 36 W; the
small unit cannot meet the demand and idling produces nothing. The disjunction is
realized by the convex hull over the declared output and cost boxes (ADR-0104); the
fixture runs under an optimization intent.

`models/control-fixtures.pse` also declares `SaturatedProcess` and `SwitchedProcess`: a
first-order lag `lag·dx/dt = u − x` from rest, with a bounded or an on/off input tracking a
target over 2 s. Their expected values are closed-form optima of that analytic model (the
input stays at its bound until the target is reached, then holds it), not external data.
