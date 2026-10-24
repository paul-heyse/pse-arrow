<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Modeling seed data

These declarations implement the Plan 21 seed. Passing one authored fixture
establishes only its named properties and conditions.

Every dataset names its source entity, role and lineage, and every fixture names the
source of its expected values as its oracle (ADR-0123 Outcome 5). The source entities
are declared in `models/references.pse`: the publications, software releases, release
files, IDAES tests and computed references the seed cites, and the seed's own analytic
and synthetic inputs. `oracle_input` and `synthetic` data are test-only: a root outside a
test fixture that reads them is refused. The notes below state what the citations do
not.

Historical property observations and solved or initialized fixture outputs use the
shared physical allowances in [`numerical_policy`](../domain/models/numerical-policy.pse).
Those same typed constants are marked as package engineering rules and supply
production engineering floors through full semantic quantity matching. The
production contextual relative fraction remains the default alongside those floors.
Where that policy has no absolute allowance for the semantic quantity, the fixture
retains its typed absolute floor and uses the shared dimensionless fraction for
relative comparison. The NRTL aggregate error uses that fraction of the reference
observations' aggregate magnitude. Expected values and model specifications are
unchanged. Exact source-bank retrieval, fixed-input aliases and mathematical
identities retain their independent precision.

`models/chemistry.pse` declares the seed's neutral chemical formula units and their
atom counts. The saponification reaction uses ethyl acetate and sodium hydroxide,
producing sodium acetate and ethanol; water is the solvent. Elemental closure is
checked independently for each admitted element. This formulation makes no ionic
speciation claim. The source atomic weights in g/mol are converted to kg/mol. All 21
previously shipped elements are retained as entities with authored masses; the
formula-weight fixture checks their use through the generic table contracts.

`models/caloric.pse` carries the nitrogen Shomate fit over 100–500 K. Its coefficients
use the published reduced coordinate T/(1000 K); heat capacity is J/(mol K). The
primitive functions produce enthalpy and entropy differences at the stock ideal-gas
datum. Reference subtraction removes arbitrary integration constants. No formation
enthalpy or entropy reference is inferred from those primitives. Tests use the rounded
published heat capacities and enthalpy increment, retaining absolute floors from the
printed resolution alongside the shared relative allowance. They check the enthalpy
derivative separately.

The reusable equations and row contracts live in `pse.methods`, in
`models/caloric.pse` and `models/pure-properties.pse`. Fit identities distinguish
liquid, vapor and alternative datasets for one species. The common polynomial
implements RPP4's cubic, Perry's quartic and DIPPR100 through declared coefficients
and a physical coefficient scale. Both enthalpy and entropy primitive derivatives
are checked; entropy uses the explicitly dimensionless Kelvin coordinate for its
chain-rule comparison. The caloric interface consumes those primitive evaluations
and owns reference subtraction. Reference values are supplied, never inferred.

Perry liquid heat capacity and density keep their natural units, J/(kmol K) and
kmol/m³. Benzene heat-capacity tests use the printed endpoint values with the retained
absolute floors and shared relative allowance. The density form excludes its critical
endpoint. Each declared interval belongs to that fit.

The IDAES oracle inputs (`rpp4_oracle`, `rpp4_pressure_oracles`,
`water_density_oracle`, `liquid_idaes_oracle` and the BT and saponification
configurations) are comparison inputs, not independently certified production
datasets. Their bounds identify the selected comparison interval, not a claim about the
publication's full empirical validity. The water checks use the original oracle
temperatures with shared concentration and pressure allowances: Perry density at
273.16 and 333.15 K; RPP4 saturation pressure at 298.15 and 373.15 K. Equations are
expressed from the polynomial,
definite-integral and reduced-temperature Wagner forms. No upstream implementation,
comments or docstrings are copied. The IDAES liquid oracle coefficients retain the
source's J/(kmol K) scale and are distinct from the published Perry coefficients.

The methane/ethane/propane DIPPR100 records keep the FeOS file's kmol coordinates. The
298.15–500 K interval is the retirement comparison scope; an independent empirical
envelope is not asserted. The frozen Decimal integration values of the Plan 14
reference provide a separate caloric oracle. The production provider records and
constructor have been retired; the authored datasets now supply the vessel and
thermodynamic definitions.

Existing benzene/toluene cubic interpolation coefficients remain demonstration
fits and are not relabeled as RPP4 source data.

The PR fixtures retain both the BT mixture and the fictitious-component inputs of the
IDAES cubic tests. The zero binary interaction rows follow the comparison convention;
they are not measured values. The nonassociating methane/ethane/propane PC-SAFT data
and five frozen teqp comparisons use the conventions documented in
[`pse.thermodynamics`](../thermodynamics/sources.md).

`bt-ideal.pse` binds the ideal log-fugacity, bubble/dew and caloric state to the
IDAES BTIdeal comparison inputs, including the small liquid heat capacities that
result from them. `oracle_h` carries the formation-inclusive 300 K reference; the
binding subtracts the supplied component formation values and accounts for the
298.15–300 K ideal-gas increment to obtain the stock sensible `h`. This shift depends
on composition and is applied in both state parameterizations. The original 47297 J/mol
FPhx specification is applied to `oracle_h`.

`bt-pr.pse` retains the 368 K, 100 kPa, equimolar BT_PR comparison. Its source enthalpy
and entropy carry an explicit formation-inclusive reference at 298.15 K and 101325 Pa.
Mixture conversion to that reference includes component formation values and the
ideal-gas pressure entropy shift from the stock 100 kPa datum. The homogeneous
potential supplies residual properties; separate phase instances prevent a smoothed
equilibrium temperature from being used as the caloric temperature. The printed oracle
rounding supplies the retained typed absolute floors alongside shared relative
allowances. Earlier focused native checks exercised the steady fixture's selected
oracle, derivative and envelope checks and the separate ideal-K initialization
fixture's warm-up and original-specification solves.

`saponification.pse` binds the IDAES dilute-water and reaction inputs: 55388 mol/m³,
75.327 J/(mol K), a 298.15 K sensible reference, an Arrhenius factor 3.132e6 m³/(mol s),
activation energy 43000 J/mol and reaction heat −49000 J/mol. These are upstream
demonstration parameters, not a new experimental fit. The seed declares a
298.15–323.15 K envelope. Concentrations represent neutral formula units; the
reaction's stoichiometry preserves each declared element. The 303.15 K reference values
are independently evaluated from those inputs. The energy comparison follows the
upstream solvent-only approximation, whereas total molar flow and mole fractions count
every component. This distinction is explicit in the shared-state binding. The packet
records the scope of native checks actually exercised; these source notes make no
broader qualification claim.

The PR heater and cocurrent heat-exchanger fixtures retain their IDAES physical
specifications and 0.0001 K equilibrium smoothing widths. Their output comparisons
use the shared physical allowances. The heat exchanger's cold-side degrees Rankine
are converted to kelvin. Their hot inlet
bubble-point initial estimate is 364 K, separated from the fixed 365 K inlet
temperature. This is a starting iterate, not a supplied solution or a changed physical
specification. It avoids centering the forward derivative sample on the narrow
smoothing transition; reducing the perturbation excessively instead introduces
floating-point cancellation. The default 1e-6 normalized perturbation and 1e-4
derivative tolerance remain in use.

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

## Independent numerical reference reproduction

The committed `scripts/pfr_reference.py`, `scripts/cstr_controller_reference.py` and
`scripts/decay_fit_reference.py` retain reduced equations and explicit inputs independently
of the production implementation. They import neither `pse` nor IDAES. The saponification
parameters above and the CODATA gas constant 8.31446261815324 J/(mol K) determine the
Arrhenius rate. The PFR's equal-reactant conservation reduction and constant solvent
calorics determine concentration and adiabatic temperature. The CSTR independently
integrates reactant concentration, temperature, controller integral and derivative-filter
state; its `Td=0.1 s` action, smooth limits (`eps=1e-6`) and back-calculation remain active.
The source equations and normalized signal conventions are described in
[`process/sources.md`](../process/sources.md). The CSTR initial state is supplied data,
not recomputed by the reference generator.

Reproduce in the exact locked test-group environment from the checkout root:

```bash
scripts/pse-env -- env UV_PROJECT_ENVIRONMENT=build/plan53a-reference uv sync --locked --python 3.12 --no-default-groups --group test --no-install-project
scripts/pse-env -- build/plan53a-reference/bin/python -m scripts.pfr_reference
scripts/pse-env -- build/plan53a-reference/bin/python -m scripts.cstr_controller_reference
scripts/pse-env -- build/plan53a-reference/bin/python -m scripts.decay_fit_reference
scripts/pse-env -- build/plan53a-reference/bin/python -m unittest scripts.tests.test_scientific_references
```

The generators require CPython 3.12, SciPy 1.18.1 and NumPy 2.5.3. Each records the
actual interpreter version, installed distributions, platform, lockfile and upstream API
source hashes. Full binary64 outputs, exact inputs, equations, methods, solver status,
source-file hashes and comparisons are written to ignored
`build/plan53a-reference-results/{pfr_reference,cstr_controller_reference,decay_fit_reference}.json`.
These are reproduction diagnostics; the authored fixture values remain the single frozen
comparison store. A failed comparison returns a nonzero exit code and retains diagnostics.
No generator writes an authored fixture.

The PFR uses DOP853 and Radau at `rtol=atol=1e-12`. Both independently reproduce the
`unit-fixtures.pse::pfr_radau` outlet. The CSTR uses both algorithms at `rtol=1e-11`,
`atol=1e-12` and samples 0, 1, 5 and 20 s, corresponding to
`campaign/models/cstr-dynamics.pse::ControlledCSTR`'s integrated and matching simultaneous
mesh checks. Reproduction compares with half a last printed digit plus an explicit
numerical allowance: 1e-7 mol/m³ for PFR concentration, 5e-10 K for PFR temperature
and 1e-8 for CSTR sampled concentration/temperature. The PFR concentration differs
from its frozen literal by approximately 3e-9 mol/m³, an immaterial last-digit difference;
the frozen literal and the much larger process acceptance allowance are retained.
Separate cross-method absolute checks use 1e-9 and 1e-8,
respectively. These are numerical reproduction premises, not revised process acceptance
policy, certified integration error bounds, or evidence for untested mesh coordinates.

The decay generator minimizes `sum((a exp(-k t)-y)^2)` for `t=[0,1,2,3,4]` and
`y=[2.02,1.19,0.75,0.43,0.28]` with bounds `a=[0.1,10]`, `k=[0.01,5]`. It uses SciPy's
bounded trust-region reflective least squares with an analytic residual Jacobian,
`ftol=xtol=gtol=1e-14`, `max_nfev=10000`, unit parameter scale and linear loss.
Explicit seeds `(1,1)`, `(0.1,0.01)`, `(0.1,5)`, `(10,0.01)`, `(10,5)` and `(2,0.5)`
must reproduce `regression-fixtures.pse::first_order_decay_certified`'s rounded `a,k`
within half a last printed digit, agree within 1e-8, and have an interior SSE-gradient
infinity norm at most 1e-8. Multistart agreement and stationarity support this selected
numerical solution; they do not establish global optimality or the product's certification.
