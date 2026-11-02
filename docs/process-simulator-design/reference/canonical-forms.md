---
title: "Canonical Forms"
---

# Canonical Forms

<a id="source-40a141b583698112becff2cd9add045c"></a>

## DAE initial-value problem

Source: [Notion](https://app.notion.com/p/40a141b583698112becff2cd9add045c).

**Usually produced by:** Authored dynamic process models, method-of-lines specialization, or a qualified index-reduction transformation

**Definition:** A declared differential-algebraic system, initial conditions and variable/derivative structure, together with the index/regularity assumptions and consistent-initialization evidence required by a selected integrator.

**Usually consumed by:** Integrators or transcriptions whose residual, mass-matrix, event and index contracts are satisfied

**Form:** DAE initial-value problem

<a id="source-40a141b5836981548787e0356c8be6c6"></a>

## Dynamic optimization

Source: [Notion](https://app.notion.com/p/40a141b5836981548787e0356c8be6c6).

**Usually produced by:** A dynamic Extremize task or admitted control/estimation composition

**Definition:** Criteria and constraints over a dynamic process model with declared controls, parameters, horizon and event/path semantics; the numerical control representation is explicit.

**Usually consumed by:** Qualified direct transcription, single/multiple shooting or another explicitly supported dynamic-optimization route

**Form:** Dynamic optimization

<a id="source-40a141b5836981e29bafea8c94f231cc"></a>

## Ensemble

Source: [Notion](https://app.notion.com/p/40a141b5836981e29bafea8c94f231cc).

**Usually produced by:** Expect, operating-case studies or other admitted batch profiles

**Definition:** A declared collection of task/form instances with scenario or sample identities and an explicit aggregation/failure policy. Conditional evaluations may run independently; statistical dependence and shared inputs remain part of the semantic sampling contract.

**Usually consumed by:** Bounded dispatch and aggregation retaining each occurrence's outcome, coverage and qualification

**Form:** Ensemble

<a id="source-40a141b5836981e6b199fee33484dccf"></a>

## Hybrid DAE

Source: [Notion](https://app.notion.com/p/40a141b5836981e6b199fee33484dccf).

**Usually produced by:** Dynamic process models with explicitly modeled operating events or mode changes

**Definition:** A differential-algebraic system with declared active modes, guards, resets, conservation/event transfers and initial consistency obligations. Fixed-layout, index and sensitivity/event support depend on the selected implementation.

**Usually consumed by:** Compatible hybrid integrators or explicit supported reformulations; unsupported variable-layout or event-derivative demands are refused

**Form:** Hybrid DAE

<a id="source-40a141b5836981c58abce3474b1ffe14"></a>

## Linear system and linear eigenproblem

Source: [Notion](https://app.notion.com/p/40a141b5836981c58abce3474b1ffe14).

**Usually produced by:** Linear process formulations, nonlinear steps or selected structural/local analyses

**Definition:** Linear algebra interfaces such as Ax=b or Av=λv with retained coordinate, scaling and source context; rank, conditioning and physical interpretation are separately assessed.

**Usually consumed by:** Compatible sparse/dense linear algebra or eigenanalysis with available numerical evidence

**Form:** Linear system and linear eigenproblem

<a id="source-40a141b58369818c944fd0165147feb4"></a>

## MCP

Source: [Notion](https://app.notion.com/p/40a141b58369818c944fd0165147feb4).

**Usually produced by:** Selected phase-appearance, inequality or other complementarity formulations

**Definition:** Complementarity pairs and associated equations, bounds and physical meanings, for example 0 ≤ a perpendicular to b ≥ 0. The complete original relation remains the acceptance target.

**Usually consumed by:** Compatible complementarity solvers or explicitly qualified smoothing/reformulation routes

**Form:** MCP

<a id="source-40a141b5836981d2a6fdf12b4b5a755d"></a>

## MINLP and GDP

Source: [Notion](https://app.notion.com/p/40a141b5836981d2a6fdf12b4b5a755d).

**Usually produced by:** Structural process decisions, supported integer domains or authored disjunctions

**Definition:** Distinct mixed-integer nonlinear and generalized disjunctive interfaces with declared domains, disjunctions and active physical relations. A selected realization states finite-bound, convexity or approximation prerequisites.

**Usually consumed by:** Compatible native solvers or qualified hull/big-M/other reformulations

**Form:** MINLP and GDP

<a id="source-40a141b5836981d282c3c4e9d7fc7968"></a>

## MPCC

Source: [Notion](https://app.notion.com/p/40a141b5836981d282c3c4e9d7fc7968).

**Usually produced by:** Explicit complementarity optimization or a qualified nested-optimality reformulation

**Definition:** An optimization problem with complementarity constraints. Replacing an inner optimization by KKT conditions is only a qualified transformation under stated regularity and optimality assumptions; it is not generally equivalent to a globally solved nonconvex inner problem.

**Usually consumed by:** Supported specialized solvers or declared regularization/reformulation with original-space assessment

**Form:** MPCC

<a id="source-40a141b5836981c9ab00dc570eb58086"></a>

## NLE

Source: [Notion](https://app.notion.com/p/40a141b5836981c9ab00dc570eb58086).

**Usually produced by:** Steady process specialization, fully discretized models or qualified algebraic reductions

**Definition:** A finite system F(x)=0 with physical/source mappings, declared domains and required residual/closure assessment. Square structure and local numerical rank are separate properties; count/matching alone does not establish solvability.

**Usually consumed by:** Contextually admissible root/continuation methods; a different feasibility formulation is explicit

**Form:** NLE

<a id="source-40a141b5836981f993b8e8f45a0cd6ac"></a>

## NLP

Source: [Notion](https://app.notion.com/p/40a141b5836981f993b8e8f45a0cd6ac).

**Usually produced by:** Process Extremize/Infer, dynamic transcription/shooting or an explicit constrained-feasibility formulation

**Definition:** A finite constrained optimization problem, minimize f(x) subject to c(x)=0, g(x)≤0 and declared domains, with source/physical mappings and derivative/regularity contracts. Feasibility formulations explicitly identify their nonphysical numerical objective, if any.

**Usually consumed by:** Compatible local or globally qualified solvers under their declared assumptions

**Form:** NLP

<a id="source-40a141b58369812cb5b2fc7aee9a66f3"></a>

## Nonlinear eigenproblem

Source: [Notion](https://app.notion.com/p/40a141b58369812cb5b2fc7aee9a66f3).

**Usually produced by:** An explicitly registered specialized profile, including a deferred electronic-structure adapter

**Definition:** A nonlinear eigenvalue or fixed-point/eigenpair problem with its normalization, admissible branches and target conditions. Electronic-structure SCF is a specialized example; it does not represent all post-HF methods or a current process-core commitment.

**Usually consumed by:** A compatible nonlinear eigen/SCF implementation with branch and convergence qualification

**Form:** Nonlinear eigenproblem

<a id="source-40a141b58369810b987ff5c8617554fd"></a>

## PDAE

Source: [Notion](https://app.notion.com/p/40a141b58369810b987ff5c8617554fd).

**Usually produced by:** Declared distributed-unit process formulations or an explicit specialized-domain adapter

**Definition:** Partial differential-algebraic relations over declared physical domains with boundary/initial conditions and closure assumptions. Distributed process-unit models are in process scope; an arbitrary CFD platform is a deferred specialized profile.

**Usually consumed by:** Qualified spatial discretization or a declared external implementation

**Form:** PDAE

<a id="source-40a141b583698186b243c900ff64011e"></a>

## Sampling problem

Source: [Notion](https://app.notion.com/p/40a141b583698186b243c900ff64011e).

**Usually produced by:** Expect, posterior Infer or an explicitly selected specialized probability profile

**Definition:** A declared probability target and requested statistics, with conditional/support/properness assumptions, evaluator contract and estimator/coverage demands. A density known up to a constant requires a supported interpretation; its normalization is not universally proved at admission.

**Usually consumed by:** Qualified quadrature, importance/Monte Carlo methods, MCMC or an explicitly supported sampler

**Form:** Sampling problem

<a id="source-40a141b583698173ba18f3c6f240d7b8"></a>

## Semantic problem

Source: [Notion](https://app.notion.com/p/40a141b583698173ba18f3c6f240d7b8).

**Usually produced by:** Process-domain authoring and typed task admission over an immutable package/model revision

**Definition:** The selected immutable task envelope: domain-model revision, effective case/specification, registered task profile and assessment/output demands. It references existing authored authority rather than creating one universal simulation AST or a closed variant per answer kind.

**Usually consumed by:** Explicit preparation/planning and supported task-profile compositions

**Form:** Semantic problem

<a id="source-40a141b583698129bbf5fd39568a46dc"></a>

## Stochastic process

Source: [Notion](https://app.notion.com/p/40a141b583698129bbf5fd39568a46dc).

**Usually produced by:** Registered stochastic relations/profiles

**Definition:** An explicitly selected SDE or jump-process formulation with stochastic-calculus/generator, initial-law, event and output semantics. Catalog presence is not a claim of a current execution route.

**Usually consumed by:** Contextually compatible integrators or simulation algorithms; specialized stochastic physics remains deferred unless separately admitted

**Form:** Stochastic process

<a id="source-40a141b58369816ca120c76dc02ed798"></a>

## Two-stage stochastic and semi-infinite programs

Source: [Notion](https://app.notion.com/p/40a141b58369816ca120c76dc02ed798).

**Usually produced by:** Registered stochastic, chance-constrained or robust optimization profiles

**Definition:** Distinct quantified optimization forms: scenario/recourse decisions with probability and nonanticipativity semantics, or constraints over a declared uncertainty set. Their relationship to a finite deterministic approximation is explicitly qualified.

**Usually consumed by:** Supported decomposition, scenario/constraint generation or deterministic reformulation under stated coverage conditions

**Form:** Two-stage stochastic and semi-infinite programs
