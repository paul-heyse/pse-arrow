---
title: "3. Best temperature profile for the same reactor"
notion_source: "https://app.notion.com/p/40a141b583698174bb22ef67109d4eb6"
notion_last_edited: "2026-11-01T20:36:23.499Z"
---

# 3. Best temperature profile for the same reactor

Source: [Notion](https://app.notion.com/p/40a141b583698174bb22ef67109d4eb6). Imported 2026-11-02.

**Version 3** · 2026-11-01

The batch reactor from [2. Kinetic parameters from four batch experiments](batch-parameter-inference.md) is operated to favor intermediate B. Its constant-volume imposed-temperature physical model is reused. The Specification and Query change, and the selected model must support the requested variable horizon and control representation.

**Profile:** Extremize. **Scope:** process dynamic optimization.

## Reused physics and new case

The material system, reaction definitions, component balances, and dynamic capabilities are the same as in Example 2. Initial concentrations are (2.0, 0, 0) mol/L. Fitted kinetic values are bound from the earlier Solution with their provenance.

This nominal optimization treats those parameter estimates as fixed. A robust or stochastic optimization would activate the retained parameter uncertainty and its correlations explicitly.

## Decisions and constraints

| Item | Contract |
| --- | --- |
| Temperature | Decision trajectory between 300 and 380 K |
| End time | Decision between 2 and 8 h |
| Actuator schedule | Temperature may change at integer clock hours measured from the batch start; it is held constant between allowed updates |
| Terminal quality | c\[C\](t_end) ≤ 0.05 mol/L, enforced |
| Objective | Maximize a declared revenue-minus-time expression with dimensionally consistent coefficients |
| Outputs | Selected temperature policy, concentration trajectories, end time, feasibility evidence, local/global status, and supported sensitivities or duals |

For example, let α have units of currency·L/mol and β have units of currency/h. The objective α·c\[B\](t_end) − β·t\_end then has a coherent currency basis. These coefficients describe the study's simplified economic value, not an intrinsic property of the reactor.

The hourly actuator rule is meaningful even when t\_end varies: the final interval may be shorter than one hour. A policy with six equal normalized-time segments is a different admissible set and must not also be described as hourly updates. The prior example conflated those two choices.

## Variable horizon

The domain extent is parameterized by t\_end. Execution can map physical time to a fixed normalized coordinate, provided derivative scaling, event locations, control-update times, and result reconstruction preserve the physical-time meaning.

If the selected backend cannot represent a variable horizon with the declared actuator schedule, it reports that capability gap or uses a separately declared formulation. It does not silently substitute six equal segments.

## Execution and result

A compatible shooting or collocation method produces an optimization problem. The result reports a candidate locally optimal solution when only local optimality is established. Constraint residuals and discretization evidence are separate from the solver's termination flag.

A terminal-constraint multiplier is returned only with its objective normalization, sign convention, scaling, and reconstruction meaning. It is not intrinsically a universal economic price.

If the admissible temperature were instead any continuous profile, a piecewise-constant control grid could be a numerical approximation. Here the allowed hourly changes are a control restriction, so they remain in the semantic formulation.

This illustrates model reuse with clearly changed case knowledge, task intent, and actuator constraints, rather than promising that every study differs only in its role labels.
