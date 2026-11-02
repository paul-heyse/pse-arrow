---
title: "6. Nonlinear MPC with moving-horizon estimation"
notion_source: "https://app.notion.com/p/40a141b58369810990bbfeb39236ef31"
notion_last_edited: "2026-11-01T20:36:26.004Z"
---

# 6. Nonlinear MPC with moving-horizon estimation

Source: [Notion](https://app.notion.com/p/40a141b58369810990bbfeb39236ef31). Imported 2026-11-02.

**Version 3** · 2026-11-01

A dynamic CSTR with jacket physics is used by a moving-horizon estimator and a nonlinear predictive controller. The same suitable process model supports both tasks, while their horizons, data, roles, constraints, and objectives differ.

**Profiles:** a Study of Infer and Extremize. **Scope:** later process/control workflow. This is a design illustration; it does not authorize a connection to live equipment.

## Shared process model

The template contains concentration and thermal inventories, material and energy balances, reaction rates, jacket heat transfer, manipulated coolant flow, and feed disturbances. It declares its dynamic/derivative capabilities and supported operating domain.

The model includes relevant actuator behavior or explicitly treats commanded moves as applied inputs. If actuator dynamics or rate limits materially affect the process, they are modeled rather than hidden in the optimizer.

## Study cadence

| Element | Contract |
| --- | --- |
| Data window | Historian observations, for example every 5 s, aligned to timestamps and units |
| Control update | Every 30 s |
| Estimation horizon | A declared trailing window, for example 10 min |
| Prediction horizon | A declared future window, for example 20 min |
| Estimator | Infer initial state and any declared disturbances/parameters using an observation/noise model |
| Controller | Extremize a control objective subject to dynamics, initial estimate, and operating/actuator constraints |
| Execution request | A required solve deadline, for example 10 s, with explicit deadline handling |
| External output | A requested move sent only through the authorized deployment's validation and acceptance contract |

## Moving-horizon estimation

Applied input history, rather than merely previously commanded inputs, is aligned with the sensor window. The observation model relates concentrations and temperatures to their sensor measurements and includes justified noise/correlation assumptions.

The arrival term carries the previous estimator's information into the new window. A mean/covariance Gaussian approximation is explicitly an approximation; arbitrary posterior information cannot be reduced to it without acknowledging the loss.

The result provides estimated current state and relevant disturbances, with consistency, residual, rank/conditioning, and uncertainty evidence as supported.

## Predictive control formulation

Coolant-flow decisions lie in a declared admissible interval, for example 0–5 m³/h, and are held between 30 s update opportunities. Path temperature requirements, such as T ≤ 360 K, are enforced or treated through a specifically declared soft/robust formulation.

The objective can combine a time integral of scaled temperature deviation and a **sum of scaled discrete move changes** Δu. A squared continuous derivative penalty is not interchangeable with discrete changes in a piecewise-constant control signal.

Weights and scales give the objective a defined numerical and engineering meaning. A nominal controller binds the estimated state/disturbance at selected values. A robust or stochastic controller explicitly activates their uncertainty.

## Data edges and reuse

- Estimator state/disturbance outputs become controller bindings or uncertainty descriptions.
- Retained estimator information becomes the next arrival term with its approximation declared.
- Authorized and actually applied moves become future estimator input history.
- A shifted prior controller solution becomes a Guess; it does not secretly define which future branch is acceptable.

Compiled plans may be reused when structure and evaluator dependencies permit, even though the semantic problem changes with new observations and initial conditions. Execution still handles physical time inside each dynamic problem; Study handles the sequence of windows, updates, synchronization, and external interaction.

## Failure and evidence

The workflow declares behavior for missing or stale observations, inconsistent estimates, infeasible optimization, deadline expiry, and failed application of a command. A returned local candidate is assessed against current constraints before any authorized deployment uses it.

A deadline does not imply that a valid optimum will always be produced. The Run can return partial or failed status, and the Study applies its explicit fallback/acceptance policy. Computation provenance identifies which data, estimates, assumptions, and move history supported every proposal.

This preserves the useful separation between reusable process physics and orchestration while removing unsupported claims that the same semantic hash is reused every cycle or that all time semantics live only in Study.
