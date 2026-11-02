---
title: "Execution"
notion_source: "https://app.notion.com/p/40a141b5836981bd9fe1cb07c9ce9a3f"
notion_last_edited: "2026-11-01T20:36:13.277Z"
---

# Execution

Source: [Notion](https://app.notion.com/p/40a141b5836981bd9fe1cb07c9ce9a3f). Imported 2026-11-02.

**Version 3** · 2026-11-01

Execution remains a collection of modular capabilities that transform or evaluate declared problem forms. Plans are explicit artifacts, and the atomic SimulationProblem remains the submission boundary. The flowsheet domain can use the repository's existing compilation and numerical foundation without first implementing a universal planner across every simulation discipline.

## Kernels and declarations

A kernel declares the form and capabilities it accepts, the outputs it produces, its settings, and the provenance/reconstruction information it records.

| Declaration area | Required meaning |
| --- | --- |
| Accepted input | Form/version, size or structural limits, supported relation kinds and external evaluators |
| Assumptions | Smoothness, convexity, derivative availability, physical or mathematical assumptions needed by the method |
| Produced output | Form/result/version and what information may be lost or approximated |
| Numerical settings | Tolerances, discretization, sampling, linear/nonlinear solver options |
| Evidence | Convergence criteria; local/global scope; optional estimators or certificates; unsupported guarantees |
| Reconstruction | Available mappings for values, sensitivities, duals, diagnostics, and generated artifacts |
| Runtime | Required dependencies, determinism, resource constraints, cancellation/checkpoint support |

Some conditions are structurally checkable. Others depend on declared assumptions or numerical behavior and must be reported accordingly. A declaration must not label all prerequisites decidable merely because the interface lists them.

The existing kernel kinds—lowering, solver, evaluation, analysis, and planner—remain useful. A practical plan can be handwritten or selected by a policy; a general search over all kinds and forms is optional.

## Canonical forms

Canonical forms describe mathematical interfaces between kernels. Shared names alone are insufficient for chaining: versions, variable/domain conventions, derivative conventions, solution meaning, and transformation assumptions must agree.

The forms catalog is a capability vocabulary. A listed form is not a claim that a production implementation, certified solver, or lossless translator already exists. Full CFD and electronic-structure forms are deferred integration capabilities for the present flowsheet work.

## Plan, run, and reuse

1. **Resolve requirements.** Determine the selected physical model, task profile, active constraints, required output/evidence, and available evaluation capabilities.
2. **Choose a plan.** Select a compatible chain through policy, explicit configuration, or a planner. Required MethodRequest settings constrain it; advisory settings guide it.
3. **Record transformations.** Every elimination, scaling, discretization, relaxation, decomposition, or surrogate use records its assumptions and origin map. A relaxation or surrogate cannot silently count as exact execution of the original request.
4. **Execute a Run.** Bind an immutable submission, record runtime versions and seeds, emit progress and diagnostics, and preserve failure or partial results.
5. **Reconstruct and assess.** Produce engineering-level values and evidence where reconstructable, retain numerical artifacts for inspection, and mark unavailable mappings.
6. **Refine when warranted.** Retry or refine using declared policies and evidence; distinguish refinement of one model from changing physical assumptions.

A structural compiled plan may be reusable across new case values if its dependency contract permits. Cache keys must include the relevant structural configuration and evaluator versions. The semantic hash of a complete case changes with meaningful data, while its reusable compiled structure may remain the same. Every actual Run is separately identified.

## Reconstruction is not a universal inverse

A Lift records a reconstruction/provenance operation. Exact inverses exist for some transformations; others allow only partial reconstruction or interpolation.

| Transformation | Typical reconstruction issue |
| --- | --- |
| Variable elimination | Recover eliminated values only when the elimination rule and needed values are available |
| Scaling | Values and duals require the recorded scaling and conventions |
| Discretization | Reconstruct a field over declared coverage using a recorded basis; discontinuities and boundary behavior remain explicit |
| Aggregation or reduction | Lost fine-scale detail is not recoverable without additional assumptions |
| Relaxation | A relaxed solution may need feasibility restoration and cannot automatically be labeled a solution of the original problem |
| Surrogate substitution | Model discrepancy and validity must be disclosed; original-model derivatives or guarantees do not appear automatically |
| External nested solve | Derivatives depend on solution selection, regularity, and the evaluator's declared behavior |

Generated solver objects may have several model origins and a transformation origin. Numerical failures must be traced as far as the evidence permits; an unknown cause remains unknown rather than being assigned an invented engineering explanation.

## Accuracy and guarantees

Kernel accuracy models and a posteriori estimators are valuable where available. Their absence does not forbid ordinary convergence-based process simulation. It means the kernel cannot claim an unsupported error guarantee.

A plan reports separately:

- Solver convergence and feasibility diagnostics.
- Estimated numerical error, including estimator assumptions.
- Certified numerical bounds, when actually established.
- Uncertainty propagated from inputs or estimated parameters.
- Model-form discrepancy and comparison across physical alternatives.

An output accuracy request is matched to the appropriate evidence. If the request requires a bound the available methods cannot establish, proposal or execution reports that limitation. An approximate estimate can be accepted only when the query permits that qualification.

## Extension without delaying the process frontend

The modular contracts allow an external CFD unit, property evaluator, surrogate, or specialized solver to be integrated later. The immediate priority is compiling and evaluating validated flowsheet models through the numerical capabilities already selected for the project. No universal cost model, globally optimal planner, or complete catalog of quantum forms is a prerequisite.

## Kernel kinds and forms

[Catalog](../reference/kernel-kinds.md)

[Catalog](../reference/canonical-forms.md)

[Catalog](../catalog/simulation-concepts.md)
