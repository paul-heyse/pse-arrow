<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Ordinary first-delivery native/reference correspondence

The [authored cases](models/delivery.pse) consume the independently selected
[ordinary BT inputs](../data/ordinary-bt/sources.md). That source package owns the
parameter values, source revisions, units, datums, missing-versus-predicted-zero
interaction policy and applicability statements. These cases are production `case`
roots; the `fixture` header is the shared case-specification grammar, not permission
to read oracle inputs. No IDAES formation constants or oracle-coordinate outputs
are adopted here.

## Exact selected cases

| Native case | Physical definition and specification | Native/reference obligation |
|---|---|---|
| `ordinary_delivery.ordinary_recycle` | Ideal benzene/toluene recycle, heater, adiabatic isobaric flash and liquid splitter; equimolar fresh feed, 1 mol/s, 368 K, 101325 Pa; purge fraction 0.25 | Run the `tear` stage, then the unchanged closed recycle. Retain all attempts, accepted original state, component/energy closure and physical reports. |
| `ordinary_delivery.ordinary_overheated` | Same original recycle specification; `overheated_tear` substitutes a 500 K tear port while preserving the defined tear component and energy transport; explicit Ipopt backend | Reach that actual stage and retain its attributed numerical stop, the independently checked 500 K versus 450 K temperature contradiction and attempt evidence. Commit no original state and leave the authored specification unchanged. A package/setup error or unrelated numerical failure does not satisfy the control. |
| `ordinary_delivery.ordinary_exchanger` | Co-current Peng–Robinson exchanger with relational density on all four states; equimolar sides at 101325 Pa, hot 5 mol/s at 365 K, cold 1 mol/s at 300 K, zero pressure drops, coefficient 100 W/(m² K); cold outlet fixed at 331.6111111111111 K and area free | Solve design area and the original side balances; evaluate SSLW costing at that actual area. The source-to-publication journey runs this case from a freshly reopened canonical source. |
| `ordinary_delivery.ordinary_rating` | Same exchanger physics and feeds; area fixed at 1 m² and cold outlet temperature free; explicit Ipopt | Use the public Rust design-to-rating composition below, then independently qualify the unchanged rating specification. It is a different question over the same physical definition. |
| `ordinary_delivery.ordinary_arithmetic_closure` | Same design specification, side balances and costing; replace the logarithmic mean temperature difference with the compatible arithmetic mean closure | Compare against the arithmetic-mean formulation, retaining the changed closure as part of scientific correspondence. |
| `ordinary_delivery.ordinary_feed_flash_isothermal` | Frozen ordinary liquid BT feed, equimolar, 1 mol/s, 368 K, 101325 Pa; equilibrium outlet at unchanged temperature and pressure | Calculate heat supplied to equilibrate the feed; retain both component flows, phase-stream identities and the original same-datum energy balance. Independently compare the vapor fraction with the binary Raoult/Rachford–Rice closed form using the selected Poling Wagner2.5,5 records. |
| `ordinary_delivery.ordinary_feed_flash_isenthalpic` | Same frozen liquid feed and pressure; zero heat supplied and free outlet temperature | Retain the original enthalpy balance and demonstrate cooling on equilibration, with distinct liquid/vapor output identities and original unit/source readback. |

The ideal recycle and PR exchanger are distinct thermodynamic realizations. Both
use the ordinary Wagner 2.5,5 vapor-pressure selection, gas heat-capacity integral
and explicit native energy datum. The recycle additionally uses published liquid
heat capacity and sourced vaporization enthalpy. The exchanger uses the ordinary
Yaws critical data and explicit predictive-zero unlike interaction. Read the exact
declarations and source package when constructing an independent reference.

## Formulation controls and permitted claims

All ordinary cases explicitly select Ipopt. This retains the controlled method
used for their native acceptance when additional solver backends are linked. The
three historical recycle preservation controls separately select POUNCE and require
the `solver-pounce` feature; their original physics, stage expectations and accuracy
checks remain unchanged. Automatic selection and other methods for these ordinary
cases remain unqualified by this delivery evidence; that limitation is not a claim
that those methods are unsupported elsewhere.

The feed-flash cases compose the existing generic process flash through
`flashing.FeedFlash`; they do not duplicate phase-equilibrium or caloric equations.
Their frozen liquid input deliberately differs from an already equilibrated feed,
so temperature continuity requires latent heat while zero heat requires a changed
outlet temperature. Both states consume the ordinary published-data selections and
the same native enthalpy datum. They retain the exact Perry benzene liquid-Cp
extrapolation and FeOS unknown-path permissions described below. Native physical
and original-readback controls do not extend the five existing binary IDAES
interchange comparisons to these new cases.

The ordinary ideal and PR states select 300–450 K consuming bounds, bubble/dew
starts of 365/370 K, and the inherited `SmoothVLE` parameters
`epsilon_bubble = 0.01 K`, `epsilon_dew = 0.0005 K`. These controls differ from the
historical exchanger witness's explicit 0.0001 K smoothing overrides. A matched
reference must use the current ordinary controls and parameter selections; this
package makes no historical default-fixture parity claim. Initial guesses remain
execution inputs, while the selected smoothing changes the formulation.

Rating overrides only numerical starts: hot/cold outlet temperatures start at
360/330K, and the co-current first/second differences at65/30K. The inherited
state guess would put both free outlets at367.5K, equal to each other and above
the365K hot feed. The selected starts respect the physical direction of transfer
and the logarithmic driving-force domain. Area remains fixed at1m² and the cold
outlet temperature remains free; no equation, bound or acceptance budget changes.
The generic declared cold-start rating did not converge: Ipopt reported
`Restoration_Failed` with large original material/equilibrium violations. An
independent existing-method trial of POUNCE interior-point/FERAL reported
`Infeasible_Problem_Detected` with an unqualified candidate, large original
material/EOS violations and failed density-domain checks. Neither stop establishes
that the physical rating problem is impossible. The selected workflow uses Ipopt
for both questions and supplies a coherent accepted design state through existing
public native start operations; it does not guess every PR auxiliary coordinate.

The Rust operation composition selects the **same rating case/root/instance**
throughout. It clones the analysis, temporarily frees area and fixes the cold
outlet at the design temperature, and calls `prepare_analysis`/`solve_case`. Only an
originally accepted design point can supply the next input. It then prepares the
unchanged original rating analysis, verifies source/target quantity and unit
identity, and transfers exactly every original free target coordinate from the
accepted state. This includes the formerly fixed cold outlet and all PR auxiliary
states, while excluding the rating's fixed area. Incomplete coverage refuses.
`PreparedSolve::with_primal_start` constructs a new target-compatible primal-only
payload; `ModelingSolvePreparation::with_start` retains explicit-input attribution.
No native factors, duals, barrier parameter or mutable solver state transfers.
The final operation receives its own original assessment, permission and point
identity. The immutable authored source/specification remains unchanged.

This is explicit composition of the existing Rust operation interface. It does
not add authored fixed/free stage overlays or claim that the generic declared
cold-start case now converges, and it does not fabricate an initializer report.
Plan66g owns remaining delivery and acceptance; the five existing binary comparisons
retain their original scientific conditions and evidence scope.

The native composition reuses `recycle_flash.RecycleLoop` through the ordinary
`RecycleModel` interface. The existing `RecycleMixer`/`PhaseAwareMixer`
constructors accept an explicit recycle package; their historical default remains
the original BT liquid package. The existing exchanger equations, costing and
report source identities have one home in
`exchanger_costing.CostedExchangerModel`; both the historical `CostedExchanger`
constructor and this package's `Exchanger` consume that interface. No duplicated
side-balance or costing definition supplies the ordinary cases.

The case permissions retain unknown empirical applicability for FeOS gas heat
capacity and PR critical/prediction data. FeOS caloric increments use an explicit
unknown whole-integration-path claim with both physical endpoints retained; source
data bounds do not become an empirical interval proof. The common DIPPR100
mathematical integrals have one definition shared with the existing interval-gated
correlation functions. The exchanger explicitly permits unknown
SSLW area support and pressure-factor extrapolation below its published interval.
Cost reports retain `USD_CE500` meaning, the existing u-tube/stainless/twelve-foot
configuration, oversize factor and hot-side pressure selection. Arithmetic agreement
does not qualify a purchase price or validate unsupported equipment sizes.

The recycle cases separately authorize the exact published Perry benzene liquid-Cp
record's extrapolation: their 368K operating temperature is14.76K above its unchanged
353.24K source maximum. This record permission denies unknown evidence and does not
cover other DIPPR100 records or any exchanger case. The overheated control retains
the same ordinary-input allowance, but its 500K tear still violates the original
300–450K physical specification/transport obligations and must fail without commit.
An independent reference must reproduce the explicit liquid-Cp extrapolation, while
retaining its limited evidence claim.

With the shared physical-budget stopping projection, the selected Ipopt attempt
terminates with `Infeasible_Problem_Detected` (2), infeasible category and an
unqualified candidate. The failure
control requires that actual native outcome, an executed `overheated_tear` result,
the compiled 500K source coordinate, the receiver's unchanged 450K upper bound,
the resulting approximately50K temperature-connection violation and failed recycle
transport closure. This is an attributed local native stop at the contradictory
physical stage, not a mathematical infeasibility certificate. The original oracle
witness retains its separate method and `trial_rejected` expectation.

The selected native profiles require strict engineering context. Enthalpy variables
and their physical binding rows use the shared power requirement and independently
frozen flow-exposure bounds. Each final contribution includes the selected
`numerical_policy.molar_flow` margin before the allowance is derived; the original
candidate must satisfy that expanded bound. This margin neither changes physical
circulation capacities nor relaxes the consuming power budget. Shared consumers
account for all distinct contributions and use their actual resolved power budget,
including characteristic magnitudes and stricter requirements.

Single-phase liquid targets also use the sourced interval-average caloric response
and shared thermal resolution. The 300–450 K response endpoints require the same
explicit Perry-record extrapolation permission; this local estimate is neither an
empirical interval proof nor a propagated output-error certificate. The PR states
use energy-only context because no independently qualified frozen EOS density branch
supplies a thermal response. Temperature keeps its separate shared resolution.
The exchanger's selected 1 m² rating reference and fixed 100 W/(m² K) coefficient
supply its area/area-equality and coefficient characteristics. These choices are
local numerical context, not equipment-size or purchase-price qualification.

## Required outputs and independent controls

The native tests require each named physical report and reject missing, ambiguous
or nonfinite values. Recycle reports retain both species' fresh/vapor/purge rates,
fresh/product/purge energy and heater duty, in addition to recycle and vapor flow.
The comparator independently cancels internal recycle transport and checks the
external component and energy balances. Its summed allowances derive from the
native 0.001 mol/s component and 1 W closure resolutions.

Exchanger reports retain both sides' inlet/outlet temperature, flow, enthalpy and
energy flow, both duties, coefficient, solved area and cost factors/results. The
comparator independently checks side flow, isolated energy exchange, the selected
co-current temperature-difference formula, duty and dimensional SSLW arithmetic at
the actual area. It uses the native 0.1 K temperature and 1 W duty resolutions for
the compounded heat-transfer check. Missing energy output and deliberately wrong
outlet flow must fail the same comparator. Canonical readback compares the entire
retained `runtime.modeling_reports` batch and its coordinates after releasing the
completed run handle.

**Tested on November 6, 2026 at `1b2240537`:** the force-validated local-profile
`scripts/pse test pse-runtime pse-backend-native pse-host pse-math --lib` selection
passed 314 tests with zero failures against a zero baseline, zero build warnings
and complete settlement. The original invocation `01a1b123-7fa6-76cd-81dc-7125e1cb2ef2`
includes all five `ordinary_reference_*_binary_comparison` controls against pinned
IDAES 2.13.0 and the protected intake, publication and exact-source readback control.
The [immutable receiving record](https://github.com/paul-heyse/pse-arrow/blob/66341514d81b2035dc0f59b3206c0a8c35800ba0/docs/plans/65h-integration-and-qualification.md#remaining-scope-receiving--november-6)
specifies the features, native profiles and selected populations. That run establishes
the named native and independent reference journeys at its recorded revision.
[Plan66g](../../../docs/plans/66g-properties-equipment-and-steady-workflows.md) owns current
receiving, including the separate typed engineering-client journey.
