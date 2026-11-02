---
title: "Answer-kind contracts"
notion_source: "https://app.notion.com/p/40a141b58369813499d4ffdc7ad2bafa"
notion_last_edited: "2026-11-01T20:36:12.347Z"
---

# Answer-kind contracts

Source: [Notion](https://app.notion.com/p/40a141b58369813499d4ffdc7ad2bafa). Imported 2026-11-02.

**Version 3** · 2026-11-01

Solve, Extremize, Expect, Infer, and Analyze are the five supported **task profiles** of the current formulation contract. Infer remains first class. These profiles describe intent and expected results; they are not mutually exclusive categories for every piece of knowledge carried by a problem, nor a claim that all future tasks must be forced into a closed taxonomy.

A task may compose these profiles through a Study or nested realization when that composition has clear semantics. A new profile can be added when a real use case exposes a distinct contract; adding one requires defining its semantics, validation, outputs, and execution capabilities.

## Profile contracts

| Profile | Core request | Relevant roles and data | Typical answer and qualification |
| --- | --- | --- | --- |
| Solve | Find a feasible state, trajectory, or explicitly selected solution set satisfying the physical formulation and active case requirements | Specified and Unknown; free design choices are permitted for feasibility when their admissible domain and selection/output semantics are clear | Feasible representative, requested branch/member set, infeasibility evidence, or unresolved/failed outcome |
| Extremize | Optimize one or more declared criteria subject to physical and case constraints | Decision roles; optionally scenarios, uncertainty, observations used in the criterion, or nested analyses | Candidate optimum or Pareto set with local/global status, constraint evidence, and supported duals |
| Expect | Evaluate requested statistics under a declared input or intrinsic probability measure | Active Uncertain roles or an intrinsic measure; the underlying deterministic problem may require solves or optimizations | Estimates/samples with estimator uncertainty and stated sampling assumptions |
| Infer | Estimate quantities from observations and a declared statistical or loss model | Estimands, observations, noise/loss assumptions, optional priors, nuisance states and constraints | Point estimates or posterior representation, residuals, and qualified uncertainty/identifiability evidence |
| Analyze | Inspect a model, a state, or another result | Reference state/result when needed; structural questions may require no solved point | Structural diagnostics, derivatives, stability/spectral results, sensitivity, or identifiability evidence |

A feasibility problem need not have an objective. An underdetermined feasible set need not be rejected merely because its structural degree-of-freedom count is positive: the Query must say whether it requests a representative, a constrained selection, a continuation branch, or another supported description. A square-state solve may still require zero structural degrees of freedom as its particular contract.

## Constraints and data are usable across profiles

| Feature | Contract |
| --- | --- |
| Operating inequalities and path/terminal limits | Permitted wherever meaningful, including Solve; each condition declares whether it is enforced, checked afterward, or advisory |
| Design specifications | Equality or inequality targets with explicit released/replaced bindings or degrees of freedom; consistency is checked after index expansion |
| Datasets as input sources | Permitted in any profile; data-backed feed trajectories do not turn a Solve into Infer |
| Observations for validation | A Solve may compare predictions with observations without fitting the model; the resulting residuals are assessments |
| Observations for fitting | Infer or an explicitly declared optimization/statistical objective changes the estimation task |
| Distributions retained as knowledge | May remain attached to case knowledge without being active in the current computation; active use must be unambiguous |
| Active uncertainty in optimization | Criterion and constraints declare scenario, expectation, risk, chance, or robust semantics as appropriate |
| Derivative or diagnostic output | Available in any profile that can give it a well-defined meaning; Analyze is useful when analysis is itself the task |
| Dual output | Available only where a mathematical dual interpretation survives the formulation and transformations; a connection multiplier is not automatically an economic stream price |

A chance constraint, robust feasibility condition, or nested inference can be represented when its mathematical semantics and required execution capabilities are declared. The profile name alone neither grants support nor prohibits a coherent formulation.

## Roles and active knowledge

A binding normally supplies a Specified element and an unbound element defaults to Unknown. Decision, Uncertain, and Estimand describe additional active intent. The active assignment must avoid contradictory use of the same scalar element. A correlated distribution, a measured value, a fixed nominal value, and a prior can coexist as retained knowledge, but the Query must select their active interpretation.

A parameter that was estimated yesterday may be bound at its estimate for today's deterministic simulation, propagated as uncertain in a reliability study, or estimated again against new data. Its physical identity does not change.

The roles are not a substitute for model capabilities, nor a complete ontology of uncertainty, causality, or physical state. Model quantities still declare their structural nature and admissible physical domain.

## Design specifications and bounds

“One target releases one handle” is a helpful authoring convention. It is not a proof of balance: a slice can contain many scalar quantities, constraints can be dependent, and vector targets can release several degrees of freedom. Validate the expanded equations and active bindings, then report numerical issues separately.

Physical support belongs to the model; package validity belongs to the selected realization; equipment ratings and operating requirements carry their engineering source; decision domains and enforcement choices belong to the active task. An unbounded decision domain is not automatically invalid if the problem is otherwise meaningful. Bounds or a coercive objective may be sufficient conditions for certain analyses, but are not universal requirements for all solvable optimization problems.

## Regime and solution selection

Steady, transient, and periodic are requestable regimes, supported only when the selected model and transformation can provide them. Steady reduction may remove accumulation terms under declared assumptions. Periodic operation requires consistent periodic conditions and, where relevant, a phase condition or other way to resolve time-shift symmetry.

A requested root, phase-stable equilibrium, continuation branch, or saddle type is semantic intent. It cannot be smuggled into a Guess and then omitted from the semantic problem. Guesses guide numerical search; they do not define which answers count as correct.

## Evidence requirements

Every profile may request stronger evidence. The response distinguishes a structural check, a numerical diagnostic, an estimate, and a certificate. “No violation found,” “passed a local check,” and “globally established” are different results.

Lack of a general structural-identifiability proof does not automatically invalidate every inference task. The system reports available evidence, non-identifiable directions when found, and limitations; a strict query may explicitly require stronger analysis.

## Existing profile records

The database below retains one record per currently supported profile. The detailed records and this contract should be revised together when capabilities or semantics change.

[Catalog](../reference/answer-kinds.md)
