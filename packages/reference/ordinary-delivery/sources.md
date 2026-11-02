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
| `ordinary_delivery.ordinary_overheated` | Same original recycle specification; `overheated_tear` substitutes a 500 K tear port while preserving the defined tear component and energy transport | Reach that actual stage, retain its attributed `trial_rejected` failure and attempt evidence, commit no original state, and leave the authored specification unchanged. A package/setup error does not satisfy the control. |
| `ordinary_delivery.ordinary_exchanger` | Co-current Peng–Robinson exchanger with relational density on all four states; equimolar sides at 101325 Pa, hot 5 mol/s at 365 K, cold 1 mol/s at 300 K, zero pressure drops, coefficient 100 W/(m² K); cold outlet fixed at 331.6111111111111 K and area free | Solve design area and the original side balances; evaluate SSLW costing at that actual area. The source-to-publication journey runs this case from a freshly reopened canonical source. |
| `ordinary_delivery.ordinary_rating` | Same exchanger physics and feeds; area fixed at 1 m² and cold outlet temperature free | Match this fixed/free correspondence explicitly. It is a different specification over the same physical definition. |
| `ordinary_delivery.ordinary_arithmetic_closure` | Same design specification, side balances and costing; replace the logarithmic mean temperature difference with the compatible arithmetic mean closure | Compare against the arithmetic-mean formulation, retaining the changed closure as part of scientific correspondence. |

The ideal recycle and PR exchanger are distinct thermodynamic realizations. Both
use the ordinary Wagner 2.5,5 vapor-pressure selection, gas heat-capacity integral
and explicit native energy datum. The recycle additionally uses published liquid
heat capacity and sourced vaporization enthalpy. The exchanger uses the ordinary
Yaws critical data and explicit predictive-zero unlike interaction. Read the exact
declarations and source package when constructing an independent reference.

## Formulation controls and permitted claims

The ordinary ideal and PR states select 300–450 K consuming bounds, bubble/dew
starts of 365/370 K, and the inherited `SmoothVLE` parameters
`epsilon_bubble = 0.01 K`, `epsilon_dew = 0.0005 K`. These controls differ from the
historical exchanger witness's explicit 0.0001 K smoothing overrides. A matched
reference must use the current ordinary controls and parameter selections; this
package makes no historical default-fixture parity claim. Initial guesses remain
execution inputs, while the selected smoothing changes the formulation.

The case permissions retain unknown empirical applicability for FeOS gas heat
capacity and PR critical/prediction data. The exchanger explicitly permits unknown
SSLW area support and pressure-factor extrapolation below its published interval.
Cost reports retain `USD_CE500` meaning, the existing u-tube/stainless/twelve-foot
configuration, oversize factor and hot-side pressure selection. Arithmetic agreement
does not qualify a purchase price or validate unsupported equipment sizes.

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

The declarations and native tests implement the handoff; their current acceptance
evidence belongs to Plan64b/64c. Plan64h H02/H03 owns construction and execution of
the matched independent references, complete required-output comparisons and
assembled scientific qualification. Those reference executions have not been
performed by this package's implementation.
