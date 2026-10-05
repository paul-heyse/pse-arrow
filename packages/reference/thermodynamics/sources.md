<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Authored thermodynamic potentials

This package is under active Plan 21 construction. Homogeneous-phase values and
locally regular derivatives do not establish global phase stability.

`HelmholtzPhase` owns residual pressure, enthalpy, entropy, heat-capacity and fugacity
identities from a dimensionless molar residual Helmholtz potential. Physical temperature,
total molar density and component amounts are normalized at this boundary to K, mol/m³
and mol. The potential receives those explicit numerical coordinates and selected
component identities. Composition is normalized by the total supplied amount. Its
amount derivatives are taken at fixed density, so the chemical-potential identity
includes both the density term and the composition derivative times total amount.
Entropy is relative to an ideal gas at the same temperature and pressure. Residual
enthalpy/entropy carry the stock datum's difference contracts, not point values.

The gas constant is the exact SI product of Avogadro's and Boltzmann's defining
constants. One authored declaration supplies all methods. Heat-capacity defaults
use the pressure response at constant density and at constant temperature; a positive
mechanical response is checked separately. These local conditions do not imply a
global minimum in Gibbs energy.

`peng-robinson.pse` uses the original PR constants 0.45724 and 0.07780, its quadratic
acentric-factor alpha correction, and quadratic/linear mixing of attraction/covolume.
The residual potential follows the generalized cubic Helmholtz expression in
[Bell and Jäger, NIST publication 924058](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=924058).
Binary interaction rows are required in both orders. Dataset authors own the actual
values; a missing pair is not silently assumed zero. The ordinary thermodynamic
potential uses the derivative of the complete mixture expression.

The BT fixture supplies IDAES 2.13.0's PR inputs and compares the homogeneous vapor
at 450 K and 1 bar against its printed compressibility and fugacity coefficients.
Density is reconstructed from that printed compressibility. Its tolerance includes
the published rounding; this does not reproduce or certify the original flash solve.
The rational PR pressure equation separately checks the potential's density derivative.

`DensityRoot` owns one polynomial residual. The three concrete implementations select
inline, generic nested, or registered cubic-root execution. The caller supplies a start
and a strict density interval for the intended branch. Both the original pressure
relation and positive mechanical response remain checks; an interval is not a global
stability certificate. `IdaesDeltaPhase` explicitly overrides only the fugacity default
with IDAES's ordered-row delta convention. For symmetric interactions this agrees with
the potential derivative. With asymmetric inputs the convention can differ, so it is
an explicit parity variant rather than the default thermodynamic identity. Fictitious
components in `pr-oracle.pse` retain the published `test_ceos_PR` comparison inputs.
An additional synthetic asymmetric matrix control retains independently calculated
ordered-row and symmetric-derivative fugacity values. It checks that the explicit
override changes fugacity while preserving the shared pressure potential.

`pcsaft.pse` implements the nonassociating hard-sphere, chain and dispersion potential
of the publication it declares as `gross_sadowski_2001`, which its dispersion-constant
dataset names as its source. The equations and extended-precision dispersion constants
were cross-checked against the pinned FeOS and teqp releases the seed's references
declare. The segment table names its fixed diameter coordinate `sigma_angstrom`; epsilon/k
is a physical temperature scale. `molecular_density` converts the explicit mol/m³
coordinate to molecules/Å³ using the exact SI Avogadro constant. All mixture amounts
are normalized. The envelope requires positive temperature, density and selected
amounts, valid segment parameters and packing fraction below one. Association, polar
terms and the exact zero-density limit are outside this potential's contract.

The seed's alkane dataset retains the existing Gross-2001 methane/ethane/propane
parameters. The authored fixtures use all five frozen teqp states of the Plan 14
reference, including a dense homogeneous state.
Residual enthalpy is compared after subtracting the independently frozen ideal-gas
enthalpy. Pressure and every component fugacity coefficient use the stored tolerances.
These are comparisons at supplied densities, not a phase-equilibrium qualification.

`equilibrium.pse` separates material-state and phase-split equations from selected
property methods. `SmoothVLE` computes ideal bubble/dew temperatures and smooths
the equilibrium temperature between them; caloric properties use the actual state
temperature. Log-fugacity equality is admitted for strictly positive selected
compositions. Four bounded Rachford–Rice Newton estimates provide starts, not a
separate accepted flash result. The original equations and common checks qualify
the final solution. Smoothing widths, temperature interval, guesses and caloric
functions are explicit package inputs.

`ThermoPackage` supplies the shared material/caloric contract and energy flow.
`EquilibriumThermoPackage` adds one enthalpy equation to the two-phase state, permitting FTPx or
FPhx specifications. Mixture enthalpy is constructed from differences about an
explicit common datum; affine point enthalpies are not summed. Bindings own any
conversion from an external formation-inclusive convention to the stock sensible
reference. The BTIdeal binding and oracle scope are described in the seed sources.

`aqueous.pse` composes the shared contract with a dilute-solvent approximation.
Component concentrations determine total molar flow and normalized mole fractions;
the supplied solvent density and heat capacity determine energy transport. The
mixture molar enthalpy accounts explicitly for this distinction. An undefined state
constrains the solvent concentration, while a defined inlet can specify it.
`reactions.pse` supplies generic Arrhenius and second-order forms and a reaction
contract whose sets, stoichiometry, heat and method are binding inputs.

`nested-equilibrium.pse` supplies the alternative ideal phase split. It solves
two-phase, liquid-only and vapor-only residual sets and selects an eligible branch
by the declared dimensionless Gibbs criterion. The BT binding restricts its local
comparison to 355–380 K, 100–103 kPa and component feed fractions 0.4–0.6.
The single-phase alternatives normalize the incipient composition.
A failed alternative, a tied selection or a singular derivative remains a refusal;
the branch-local derivative contract does not establish smoothness across phase
appearance. The fixture comparisons include 358, 368 and 378 K.

`pr-equilibrium.pse` composes the same state contract with PR phase closures.
Bubble/dew calculations solve coupled composition and log-fugacity equations.
Separate closures evaluate equilibrium at the smoothed temperature and caloric
properties at the actual temperature. Density bounds select local liquid/vapor
branches; positive compressibility and mechanical response remain obligations.
The BT_PR steady fixture passes its selected oracle, derivative and envelope
checks. This local sample does not establish all states within the declared
temperature and pressure interval. The `ideal-K` stage is inherited; final fixture expectations are
evaluated only after solving the restored original specification.

`phase-stability.pse` authors Michelsen's tangent-plane distance of a homogeneous
feed at fixed temperature and pressure, over any Helmholtz law (ideal, PR or PC-SAFT):
`tpd(w) = Σ w_j (ln w_j + ln φ_j(w) − ln z_j − ln φ_j(z))`, with the trial
composition on the simplex and its density at the feed pressure. The reference
density bounds select the feed root. A local minimum proves nothing: global stability
is a claim only under the explicit certify intent, whose factorable export and global
bound decide it within the recorded box, tolerances and export fidelity. The check
`tpd > −tolerance` then holds at the certified minimizer, and a certified minimum below
it, or any original-qualified trial point with negative distance, shows instability.
The check itself is a predicate over the solved point; it does not run a solve.

Physical reconstruction, material/energy transport and ordinary thermodynamic output
checks consume the shared typed `numerical_policy` constants from the domain package.
Composition normalization keeps its equation and independently checks the same global
dimensionless allowance. Pressure reconstruction and its independent pressure check
share one physical pressure allowance. Strict positivity, admissible physical domains,
phase-selection assurance, smoothing parameters and mathematical identity tests retain
their own meanings. A tighter explicit analysis can override the ordinary design values.
