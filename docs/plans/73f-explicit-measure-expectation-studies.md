---
title: "Plan73f: Explicit-measure expectation studies"
status: draft
date: 2026-11-09
doc_role: plan
doc_owner: docs/plans/73f-explicit-measure-expectation-studies.md
doc_topics: [current-work, design, numerical-execution, validation]
review_sources: [docs/functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md]
---

# Explicit-measure expectation studies

## Outcome and baseline

**Proposed.** This companion of [Plan73](73-symbolica-ecosystem-improvements.md) delivers O10: an engineer requests an expectation under a declared measure and receives the estimand, estimate, statistical uncertainty, convergence interpretation, original member outcomes and replayable provenance. It owns scientific operation/result declarations, protected publication and independent reading needed here; unfinished Plan66 work is not a prerequisite.

**Interface-checked.** Baseline `baadc7e32` supplies quantity-typed physical overlays in `workflow/sampling.rs::DesignRequest`, ordinary case operations, independent study lanes and retained occurrence results. Covariance propagation in `workflow/uncertainty.rs` establishes local response meaning. DOE and covariance do not specify an adaptive expectation. Reuse those owners without a second study service or probability framework.

## Selected scientific contract and first consumer

Deliver an ordinary authored constant-cp heater study: cp30 J/(mol K), inlet300 K, outlet uniformly310–330 K, and independent flow alternatives1/2 mol/s with probabilities1/4 and3/4. Requested outputs include duty and molar enthalpy difference. Independent mean duty is1050 W. Author the law with its validity interval and enthalpy datum; the operation accepts other lawful admitted cases rather than hard-coding this integrand. Existing authored unit and caloric vocabulary supply the first consumer.

The request names original source/context, scientific operation/settings, quantity-typed coordinates, measure/transform, selected outputs, estimator controls and replay policy. Initial measures are products of bounded continuous uniforms and finite categories with explicit normalized probabilities. Reject negative/nonfinite probabilities, zero total mass, invalid bounds, incompatible units and conflicting assignments before execution. Keep category identity and sampled occurrence separate even when realized values coincide.

An admitted transform maps the base unit cube/categories into physical coordinates, retaining its definition, domain and induced law. T=310 K+20 K·u² is a different law from uniform T; its independent mean duty is875 W under the same flow probabilities. Pushforward expectation applies no additional density Jacobian; integration against a declared physical density is a distinct meaning and is not silently inferred. Dependent inputs require an explicit joint transform, not a covariance-based independence guess.

Each output has its own estimand/availability. Retain failed scientific members. Default behavior withholds a completed expectation when a required positive-weight evaluation lacks that output; never drop and renormalize. A separately requested failure-probability estimand may explicitly classify scientific admission/domain/solve outcomes. Cancellation, infrastructure failure and unsupported interpretation are not physical failures. Statistical uncertainty is not an enclosure or truth claim about the model.

## Library composition, iteration and replay

Use Numerica3.0.1 `ContinuousGrid`, `DiscreteGrid` and nested `Grid` for adaptive samples, `Sample::get_weight` for sampling weight, and `StatisticsAccumulator` for scalar estimates/statistics. Apply the declared measure and library sampling weights exactly once. The [focused differentiation/integration account](../library_leverage/libraries/numerica/differentiation-and-integration.md#adaptive-integration-and-combinatorial-output) and [Numerica overview](../libraries/scientific_calcs/numerica.md) identify APIs; resolve selected-release weight/update conventions against installed source before implementing F03.

Workers receive compatible frozen iteration grids. `clone_without_samples`, `is_mergeable` and `merge` support collecting compatible observations; update the adaptive grid only at declared barriers. Do not independently adapt workers and interpret their weights as a common frozen distribution. Independent lanes own mutable scratch; compatible mathematical products share immutable backing.

Start with scalar ordinary case execution and fixed-weight Monte Carlo as a simple explicit realization/reference. Adaptive integration is selected as another realization, without a universal variance-improvement claim. General solves remain independent scientific members; C's qualified pure batch operation is integrated only for eligible arithmetic, preserving ordinary solve semantics. Initial pointwise derivatives use existing demands; derivatives of expectation need a separate estimator/interchange contract and are not implied by differentiating the sampler.

Controls specify iteration/sample limits and selected estimate/standard-error interpretation, with absolute/relative output-unit precision targets. At a limit report the actual estimate and unmet target. `MonteCarloRng::new(seed,stream_id)` plus state export/import support stream reproduction. Retain points, categories, weights, iteration identity and selected RNG/grid state through existing custody. Initial replay reproduces the sample population and its interpretation. Bitwise accumulation needs the same ordering and conditions; a seed alone is insufficient.

Cancellation stops issuance, settles live members and publishes incomplete original outcomes, not a completed expectation from a convenient subset. Restart uses the original retained iteration boundary or a distinct attempt, without silently mixing populations.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| F01 | open | Shared source/product contract; current case/overlay admission | Measure/transform/estimand/failure contracts and applicable decisions delivered with ordinary authored heater consumer. |
| F02 | open | F01 and original member execution | Fixed-weight scalar expectation applies weights once, retains all outcomes and produces correctly scoped estimates. |
| F03 | open | F02 and source-qualified grid/statistics contracts | Adaptive iterations use frozen compatible snapshots, merge/update barriers, controls, cancellation and stream/state replay. |
| F04 | open | F02/F03 products and protected custody; C's working batch contract for eligible integration | Native operation, generated boundaries, protected publication, independent read/export and eligible batch consumer delivered; replaced expectation machinery deleted. |
| F05 | open | F01–F04 functional delivery | Independent analytic, failed-member, replay and settled-cancellation journeys available to H; all expectation source leads dispositioned with consequences. |

F01/F02 extend existing scientific owners with measure identity, output coordinates/units, normalization, estimator, counts/weights, uncertainty meaning, stopping and member references. New authored measure/estimand and kernel/relation contracts follow the coordinator's decision route. Do not reconstruct a law from the seed at read time.

F03 verifies weight normalization and continuous/category nesting against a tiny known law before general case integration. Merge refusal and declared update barriers are correctness obligations, not optional instrumentation. F04 migrates the delivered user-facing request and independent reader together; readers consume original products without rerunning science. Delete provisional weight/statistics loops and direct in-memory result shortcuts when superseded; preserve DOE/covariance for their distinct responsibilities. Run codegen when declarations change.

F05 closes all supplied expectation leads: fixed versus adaptive methods, correlated transforms, RNG/state custody, statistics merge and pointwise derivatives. Additional distribution families, rare-event estimators and expectation derivatives are explicitly unselected unless a concrete scientific contract reopens them. Deterministic ODE/DAE integration remains with native solvers.

## Verification

**Proposed:** targeted tests use independent1050 W and875 W expectations, analytic normalization, malformed units/probabilities, categorical mixtures, unavailable outputs, compatible/incompatible merges, independent scratch, replayed populations and protected independent reading. Exercise cancellation and settlement through the actual study owner. H owns scope-end qualification; no full static campaign per package. No test, probe or performance measurement was run in authoring. Report actual commands, selection and zero-target failures when implementation is exercised.
