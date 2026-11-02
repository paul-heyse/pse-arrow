---
title: "Task Classes"
---

# Task Classes

<a id="source-40a141b583698141af4cea4a66892d25"></a>

## Bayesian calibration

Source: [Notion](https://app.notion.com/p/40a141b583698141af4cea4a66892d25).

**Canonical form:** An explicit posterior sampling/approximation profile. Properness and identifiability are established only where supported, with conditional evidence and unavailable products retained.

**Role pattern:** Selected quantities are estimands; nuisance uncertainty has declared conditioning/integration treatment.

**Extent and characteristic relations:** Process inference extension. Process observations and likelihood/noise model with explicit priors, dependence and posterior assumptions; current implementation is a separate capability question.

**Answer as stated:** Infer (posterior)

**Answer kind:** [Infer](answer-kinds.md#source-40a141b58369818f926dcf983d26cb3c)

**Task class:** Bayesian calibration

<a id="source-40a141b5836981fdb65ecee60bcb56d2"></a>

## CFD

Source: [Notion](https://app.notion.com/p/40a141b5836981fdb65ecee60bcb56d2).

**Canonical form:** Specialized PDE/discretized forms behind a qualified adapter; no current generic CFD capability is implied.

**Role pattern:** Owned by the specialized profile: physical geometry and boundary data, field unknowns and optional optimization inputs.

**Extent and characteristic relations:** Deferred specialized profile. A general multidimensional fluid-field domain, mesh/turbulence/multiphase closure ecosystem is outside the first-party flowsheet model. A future external unit may expose a typed process boundary and explicit conservative mappings.

**Answer as stated:** Future registered Solve/Analyze/optimization profiles, consumed through an explicit process interface when needed

**Answer kind:** [Solve](answer-kinds.md#source-40a141b5836981d28ffbc1a901cf1364)

**Task class:** CFD

<a id="source-40a141b5836981819f82dccb72ec4c44"></a>

## Continuation, bifurcation, operability

Source: [Notion](https://app.notion.com/p/40a141b5836981819f82dccb72ec4c44).

**Canonical form:** Supported continuation or augmented systems with local singularity/eigen analysis. A finite sweep is not automatically branch tracking or proof of all bifurcations.

**Role pattern:** Continuation parameter and branch variables are explicit; a numerical seed is separate from the requested branch.

**Extent and characteristic relations:** Process analysis/study profile. A declared parameter/path and branch-selection question over a process model, with supported continuation and local bifurcation/operability assessments.

**Answer as stated:** Study of Solve/Analyze tasks or an explicit registered continuation profile

**Answer kind:** [Solve](answer-kinds.md#source-40a141b5836981d28ffbc1a901cf1364); [Analyze](answer-kinds.md#source-40a141b58369810e8e69f617b4bda52f)

**Task class:** Continuation, bifurcation, operability

<a id="source-40a141b58369811dadd0d211c02af797"></a>

## Control analysis and tuning

Source: [Notion](https://app.notion.com/p/40a141b58369811dadd0d211c02af797).

**Canonical form:** Supported linearization/eigen/frequency analyses; tuning is a separate optimization composition. Structural diagnostics need no prior successful simulation.

**Role pattern:** Manipulated/controlled quantities and operating points are identified; tuning gains become decisions only in the tuning task.

**Extent and characteristic relations:** Process analysis profile. Dynamic process/controller definitions and the selected reference operating regime; control loops use explicit signal and physical actuator interfaces.

**Answer as stated:** Analyze; explicit Extremize composition for tuning

**Answer kind:** [Analyze](answer-kinds.md#source-40a141b58369810e8e69f617b4bda52f); [Extremize](answer-kinds.md#source-40a141b58369810e8c01df60ebe98800)

**Task class:** Control analysis and tuning

<a id="source-40a141b5836981eeadabdb97d7a8aef0"></a>

## Data reconciliation, gross error detection

Source: [Notion](https://app.notion.com/p/40a141b5836981eeadabdb97d7a8aef0).

**Canonical form:** Supported constrained statistical fitting and separate residual/gross-error assessment; uncertainty and detectability remain qualified.

**Role pattern:** Measured quantities are estimands only when the selected reconciliation model says so; measurements are not silently converted to hard inputs.

**Extent and characteristic relations:** Process analysis profile. Balance/constitutive models linked to measured data through observation, dependence and gross-error contracts.

**Answer as stated:** Infer

**Answer kind:** [Infer](answer-kinds.md#source-40a141b58369818f926dcf983d26cb3c)

**Task class:** Data reconciliation, gross error detection

<a id="source-40a141b5836981bab161e184223889da"></a>

## Distributed unit models (PFR, packed bed, membrane, rate-based column)

Source: [Notion](https://app.notion.com/p/40a141b5836981bab161e184223889da).

**Canonical form:** Distributed equations lowered through a declared discretization or qualified external unit implementation; dimensionality, time coupling and derivative support are explicit.

**Role pattern:** Boundary/operating data and selected geometry are specified; local states/fluxes are unknown. Dynamic use additionally declares inventories and temporal initial conditions.

**Extent and characteristic relations:** Process profile with formulation-specific coverage. A flowsheet unit with spatial or internal-coordinate transport, phase/interface closures and boundary contracts. Reduced-dimensional distributed units are included without requiring a general CFD domain.

**Answer as stated:** Solve

**Answer kind:** [Solve](answer-kinds.md#source-40a141b5836981d28ffbc1a901cf1364)

**Task class:** Distributed unit models (PFR, packed bed, membrane, rate-based column)

<a id="source-40a141b58369813380f0e2b75266eff0"></a>

## Dynamic optimization, optimal control

Source: [Notion](https://app.notion.com/p/40a141b58369813380f0e2b75266eff0).

**Canonical form:** Dynamic optimization realized by qualified transcription, single/multiple shooting or another supported route; physical control restrictions and numerical parameterization are distinct.

**Role pattern:** Admitted control trajectories, operating/design parameters and optionally horizon quantities are decisions; initial conditions and uncertainties have explicit semantics.

**Extent and characteristic relations:** Process analysis profile. The dynamic process/inventory model plus controls, horizon, path/terminal requirements and objectives.

**Answer as stated:** Extremize

**Answer kind:** [Extremize](answer-kinds.md#source-40a141b58369810e8c01df60ebe98800)

**Task class:** Dynamic optimization, optimal control

<a id="source-40a141b5836981aea83ce668148b9660"></a>

## Dynamic simulation (start-up, upset, batch)

Source: [Notion](https://app.notion.com/p/40a141b5836981aea83ce668148b9660).

**Canonical form:** Admitted ODE/index-1 or other explicitly supported DAE/hybrid profile, via integration or simultaneous discretization; event and sensitivity support are conditional.

**Role pattern:** Effective initial/boundary conditions and inputs are fixed; dynamic states and algebraic variables follow the selected consistent formulation.

**Extent and characteristic relations:** Process profile. The process model plus local inventories, temporal conservation, algebraic memoryless units, schedules, declared events/modes and operating/initial conditions. General recipe scheduling is a separate extension.

**Answer as stated:** Solve

**Answer kind:** [Solve](answer-kinds.md#source-40a141b5836981d28ffbc1a901cf1364)

**Task class:** Dynamic simulation (start-up, upset, batch)

<a id="source-40a141b58369816d9d32ecfc803e1cad"></a>

## Electronic structure (HF, DFT, post-HF)

Source: [Notion](https://app.notion.com/p/40a141b58369816d9d32ecfc803e1cad).

**Canonical form:** SCF/nonlinear-eigen or other method-specific forms; post-HF methods are not all represented by an SCF problem.

**Role pattern:** Specialized inputs/unknowns and scientific model choices; these do not define generic process components.

**Extent and characteristic relations:** Deferred specialized profile. Electronic/nuclear state, basis and quantum-chemistry model choices belong to a separate domain; process users may consume qualified derived parameters or property outputs.

**Answer as stated:** Future registered specialized profiles; no requirement to encode electronic structure in the flowsheet core

**Answer kind:** [Solve](answer-kinds.md#source-40a141b5836981d28ffbc1a901cf1364)

**Task class:** Electronic structure (HF, DFT, post-HF)

<a id="source-40a141b58369818da84cc6177b7f2b1c"></a>

## Geometry optimization, transition-state search

Source: [Notion](https://app.notion.com/p/40a141b58369818da84cc6177b7f2b1c).

**Canonical form:** Method-specific optimization or stationary-point search with its own Hessian-index/regularity evidence.

**Role pattern:** Specialized coordinates and branch/index target; a saddle search is not ordinary process-objective minimization.

**Extent and characteristic relations:** Deferred specialized profile. Molecular geometry and quantum/force-field energy evaluation belong to a separate microscopic domain.

**Answer as stated:** Future specialized Extremize for minima; a separately registered stationary/saddle-search profile for transition states

**Answer kind:** [Extremize](answer-kinds.md#source-40a141b58369810e8c01df60ebe98800)

**Task class:** Geometry optimization, transition-state search

<a id="source-40a141b583698132a9a0dd565f3893f2"></a>

## Local sensitivity

Source: [Notion](https://app.notion.com/p/40a141b583698132a9a0dd565f3893f2).

**Canonical form:** Qualified implicit, forward/adjoint or explicitly approximate derivatives; unavailable/nonsmooth cases retain reasons.

**Role pattern:** Input perturbation coordinates and response quantities are explicit; fixed inputs or decision/estimand coordinates may be analyzed under the chosen profile.

**Extent and characteristic relations:** Process analysis profile. A selected model/task/form and suitable reference candidate, with its branch, active-set and regularity context.

**Answer as stated:** Analyze

**Answer kind:** [Analyze](answer-kinds.md#source-40a141b58369810e8e69f617b4bda52f)

**Task class:** Local sensitivity

<a id="source-40a141b583698107b3decc6ccd46c9f5"></a>

## MPC and moving-horizon estimation

Source: [Notion](https://app.notion.com/p/40a141b583698107b3decc6ccd46c9f5).

**Canonical form:** A sequence of admitted optimization/inference forms with declared horizon, warm-start, consistency and failure policies.

**Role pattern:** Per-horizon controls/estimands, shared model parameters and fixed incoming data are declared; no implicit external actuation is authorized by the model.

**Extent and characteristic relations:** Process study profile. An explicitly owned receding sequence of dynamic optimization or inference tasks, with carried state/uncertainty, incoming observations and action/result transfers.

**Answer as stated:** Study composing Extremize, Infer, Solve and applicable Analyze profiles

**Answer kind:** [Extremize](answer-kinds.md#source-40a141b58369810e8c01df60ebe98800); [Infer](answer-kinds.md#source-40a141b58369818f926dcf983d26cb3c)

**Task class:** MPC and moving-horizon estimation

<a id="source-40a141b58369819488a4eef536bc9e40"></a>

## Model-based design of experiments

Source: [Notion](https://app.notion.com/p/40a141b58369819488a4eef536bc9e40).

**Canonical form:** Supported optimization over local information or another explicit design criterion. A local Fisher criterion does not prove global identifiability or predictive adequacy.

**Role pattern:** Experiment conditions are decisions; inferred parameters, prior/local point and observation design have distinct roles.

**Extent and characteristic relations:** Process study extension. Candidate experimental conditions and a declared information/uncertainty objective linked to a selected inference model and its validity assumptions.

**Answer as stated:** Extremize over Analyze

**Answer kind:** [Extremize](answer-kinds.md#source-40a141b58369810e8c01df60ebe98800); [Analyze](answer-kinds.md#source-40a141b58369810e8e69f617b4bda52f)

**Task class:** Model-based design of experiments

<a id="source-40a141b583698131a15cd7921be183e0"></a>

## Molecular Monte Carlo (NVT, GCMC, Gibbs ensemble)

Source: [Notion](https://app.notion.com/p/40a141b583698131a15cd7921be183e0).

**Canonical form:** A qualified specialized sampling problem with properness, convergence and coverage assumptions; no first-party delivery claim.

**Role pattern:** Specialized ensemble variables, probability target and model choices are explicit.

**Extent and characteristic relations:** Deferred specialized profile. Configuration/ensemble measures and molecular moves belong to a separate microscopic domain; qualified property/data outputs may later enter process models.

**Answer as stated:** Future specialized Expect profile

**Answer kind:** [Expect](answer-kinds.md#source-40a141b5836981f0a45ad165770120dd)

**Task class:** Molecular Monte Carlo (NVT, GCMC, Gibbs ensemble)

<a id="source-40a141b5836981fdba1fc800295b187b"></a>

## Molecular dynamics

Source: [Notion](https://app.notion.com/p/40a141b5836981fdba1fc800295b187b).

**Canonical form:** Specialized ODE/SDE/constraint dynamics and optional statistical aggregation with declared convergence evidence.

**Role pattern:** Specialized initial/ensemble data and force-field choices; their semantics are not imposed on flowsheet components.

**Extent and characteristic relations:** Deferred specialized profile. Particle configurations, force fields, ensembles and trajectories belong to a separate microscopic domain; the process model may consume qualified derived properties through an adapter or dataset.

**Answer as stated:** Future specialized Solve for trajectories and Expect for statistics; not universally Expect

**Answer kind:** [Expect](answer-kinds.md#source-40a141b5836981f0a45ad165770120dd)

**Task class:** Molecular dynamics

<a id="source-40a141b583698195bf99e5db2364569e"></a>

## Optimization under uncertainty (stochastic, robust, chance-constrained)

Source: [Notion](https://app.notion.com/p/40a141b583698195bf99e5db2364569e).

**Canonical form:** A supported stochastic/chance/robust optimization or explicit approximation; finite scenario coverage and certificates are qualified.

**Role pattern:** Design/control decisions, uncertain inputs and case-local unknowns are separate; robust sets are not probability distributions.

**Extent and characteristic relations:** Process analysis extension. Process cases with explicit probability or uncertainty-set semantics, recourse/nonanticipativity and declared objective/constraint quantifiers.

**Answer as stated:** Extremize with explicit quantified/composed uncertainty semantics; robust optimization need not contain Expect

**Answer kind:** [Extremize](answer-kinds.md#source-40a141b58369810e8c01df60ebe98800); [Expect](answer-kinds.md#source-40a141b5836981f0a45ad165770120dd)

**Task class:** Optimization under uncertainty (stochastic, robust, chance-constrained)

<a id="source-40a141b5836981c7a20eef0a8cee92e6"></a>

## Parameter estimation

Source: [Notion](https://app.notion.com/p/40a141b5836981c7a20eef0a8cee92e6).

**Canonical form:** Supported likelihood/loss fitting with optional local information/covariance analysis. Rank deficiency, active bounds and noise assumptions may restrict or withhold uncertainty products.

**Role pattern:** Selected parameters or other quantities are estimands; physical states remain unknown and effective hard inputs stay fixed.

**Extent and characteristic relations:** Process analysis profile. Selected steady/transient experiments, observations, shared/local parameters and a declared noise/loss model over the same process definitions.

**Answer as stated:** Infer (point)

**Answer kind:** [Infer](answer-kinds.md#source-40a141b58369818f926dcf983d26cb3c)

**Task class:** Parameter estimation

<a id="source-40a141b5836981c2ad25e466ff838ff5"></a>

## Phase and chemical equilibrium (flash, Gibbs reactor)

Source: [Notion](https://app.notion.com/p/40a141b5836981c2ad25e466ff838ff5).

**Canonical form:** NLE, complementarity or constrained thermodynamic minimization as explicitly selected; stationarity/isofugacity/local stability and globally certified equilibrium remain distinct claims.

**Role pattern:** Inputs depend on the selected state problem (for example TP, PH or specified inventories); phase amounts, compositions and equilibrium quantities are unknown.

**Extent and characteristic relations:** Process profile with staged scientific coverage. Material/phase identities, independent state constraints, conserved quantities and selected equilibrium/reaction formulation, including phase-presence and stability requirements.

**Answer as stated:** Solve or an explicitly scoped nested thermodynamic minimization; global stability only with its evidence

**Answer kind:** [Solve](answer-kinds.md#source-40a141b5836981d28ffbc1a901cf1364)

**Task class:** Phase and chemical equilibrium (flash, Gibbs reactor)

<a id="source-40a141b583698173aae3fc2178ee8ebd"></a>

## Population balances (crystallization, polymerization)

Source: [Notion](https://app.notion.com/p/40a141b583698173aae3fc2178ee8ebd).

**Canonical form:** A declared population-balance, sectional or moment formulation with a qualified discretization/closure; catalog presence does not establish implementation.

**Role pattern:** Selected kinetic/closure parameters are fixed or fitted; distribution/moment states and declared design quantities follow the task.

**Extent and characteristic relations:** Process extension profile. Unit-local particle-size/chain-length/age distributions or moments, birth/death/transfer laws and explicit material/energy coupling to process streams.

**Answer as stated:** Solve

**Answer kind:** [Solve](answer-kinds.md#source-40a141b5836981d28ffbc1a901cf1364)

**Task class:** Population balances (crystallization, polymerization)

<a id="source-40a141b5836981c78eabee8eea737656"></a>

## Process synthesis (superstructure)

Source: [Notion](https://app.notion.com/p/40a141b5836981c78eabee8eea737656).

**Canonical form:** Admitted MINLP/GDP or other explicit synthesis formulation with bound, realization and original-physics assessment requirements.

**Role pattern:** Supported structural alternatives and continuous operating/design quantities are decisions; model selection is not an opaque solver preference.

**Extent and characteristic relations:** Process extension profile. Explicit candidate units, configurations and typed interconnections with activation/logic constraints, material/property compatibility and alternative-specific physical assumptions.

**Answer as stated:** Extremize

**Answer kind:** [Extremize](answer-kinds.md#source-40a141b58369810e8c01df60ebe98800)

**Task class:** Process synthesis (superstructure)

<a id="source-40a141b5836981e0a8e2e47b40c50bf1"></a>

## Property evaluation

Source: [Notion](https://app.notion.com/p/40a141b5836981e0a8e2e47b40c50bf1).

**Canonical form:** Direct evaluation or a declared implicit/nested property solve; derivative and phase-branch capability is contextual.

**Role pattern:** Independent state inputs are fixed; requested properties are outputs. Derived/reconstructed quantities are not additional independent specifications.

**Extent and characteristic relations:** Process profile. A declared material/state contract plus selected physical model, parameterization, basis, reference conventions and validity/phase assumptions.

**Answer as stated:** Solve (evaluation)

**Answer kind:** [Solve](answer-kinds.md#source-40a141b5836981d28ffbc1a901cf1364)

**Task class:** Property evaluation

<a id="source-40a141b583698109a52ce2b399afdc04"></a>

## Reaction kinetics and microkinetics

Source: [Notion](https://app.notion.com/p/40a141b583698109a52ce2b399afdc04).

**Canonical form:** Algebraic, stiff ODE/DAE or other admitted reaction formulation; chemical composition and site/charge checks are qualified by available information.

**Role pattern:** Rate/thermodynamic parameters may be specified or fitted; unknowns and inventories depend on the selected reactor/phase model.

**Extent and characteristic relations:** Process profile with staged scientific coverage. Material/site identities, stoichiometry, phase/interface location, kinetics/equilibrium and explicit reaction-energy conventions; detailed microkinetics is a selected extension.

**Answer as stated:** Solve

**Answer kind:** [Solve](answer-kinds.md#source-40a141b5836981d28ffbc1a901cf1364)

**Task class:** Reaction kinetics and microkinetics

<a id="source-40a141b5836981d09fd6f348e06ea597"></a>

## Scheduling of batch plants

Source: [Notion](https://app.notion.com/p/40a141b5836981d09fd6f348e06ea597).

**Canonical form:** A supported scheduling MILP/MINLP/hybrid formulation or coupled study; interface presence does not establish delivery.

**Role pattern:** Assignments, sequencing and timing are decisions under a declared scheduling formulation; unit operation conditions retain separate physical contracts.

**Extent and characteristic relations:** Adjacent process extension profile. Tasks, equipment/resource capacity, recipes, timing and events linked explicitly to process models. This is not a prerequisite for the flowsheet physical domain or a general scheduling-platform commitment.

**Answer as stated:** Extremize

**Answer kind:** [Extremize](answer-kinds.md#source-40a141b58369810e8c01df60ebe98800)

**Task class:** Scheduling of batch plants

<a id="source-40a141b5836981e4b476efb26e9785a5"></a>

## Steady-state flowsheet

Source: [Notion](https://app.notion.com/p/40a141b5836981e4b476efb26e9785a5).

**Canonical form:** Qualified sparse algebraic or complementarity/feasibility form; structure and scientific closure are assessed independently of solver termination.

**Role pattern:** Effective feed/boundary and unit specifications are fixed; remaining variables are unknown; hard feasibility/design targets and released quantities are explicit.

**Extent and characteristic relations:** Process profile. Hierarchical units and subflowsheets, typed material/energy/signal couplings, material systems and conservation/constitutive closures. A selected steady regime suppresses temporal accumulation without removing internal spatial structure.

**Answer as stated:** Solve

**Answer kind:** [Solve](answer-kinds.md#source-40a141b5836981d28ffbc1a901cf1364)

**Task class:** Steady-state flowsheet

<a id="source-40a141b5836981fe8c56d3a459757706"></a>

## Steady-state process optimization

Source: [Notion](https://app.notion.com/p/40a141b5836981fe8c56d3a459757706).

**Canonical form:** Qualified NLP, mixed-integer, linear/quadratic/conic or other supported class; local/global evidence follows the selected route.

**Role pattern:** Design and operating decisions are declared separately from fixed inputs and physical unknowns.

**Extent and characteristic relations:** Process analysis profile. The selected flowsheet and operating cases plus explicit objectives, admissible decision domains and feasibility/design requirements.

**Answer as stated:** Extremize

**Answer kind:** [Extremize](answer-kinds.md#source-40a141b58369810e8c01df60ebe98800)

**Task class:** Steady-state process optimization

<a id="source-40a141b5836981879f2fc62d227fda9a"></a>

## Stochastic kinetics, kinetic Monte Carlo

Source: [Notion](https://app.notion.com/p/40a141b5836981879f2fc62d227fda9a).

**Canonical form:** A supported specialized Markov jump/sampling realization, not implied by catalog inclusion.

**Role pattern:** Specialized rate/generator inputs and random state evolution; process-facing outputs require a declared aggregation/uncertainty contract.

**Extent and characteristic relations:** Deferred specialized stochastic profile. Discrete microscopic configurations and jump generators are distinct from the process reactor rate/inventory models; a future reduced unit/property coupling must be explicit.

**Answer as stated:** Future registered stochastic Solve/Expect profile

**Answer kind:** [Expect](answer-kinds.md#source-40a141b5836981f0a45ad165770120dd)

**Task class:** Stochastic kinetics, kinetic Monte Carlo

<a id="source-40a141b58369811daacbf96d07b68090"></a>

## Surrogate and reduced-order model training

Source: [Notion](https://app.notion.com/p/40a141b58369811daacbf96d07b68090).

**Canonical form:** A registered fitting/training or reduction workflow plus independent validation; the resulting approximation is a selected scientific/formulation choice, not automatically equivalent physics.

**Role pattern:** Training coefficients may be estimands; architecture/model-class selection is separately declared.

**Extent and characteristic relations:** Process extension profile. Qualified process observations or simulations used to construct a surrogate/reduced model with an explicit input/output, physical-convention, validity and derivative contract.

**Answer as stated:** Infer

**Answer kind:** [Infer](answer-kinds.md#source-40a141b58369818f926dcf983d26cb3c)

**Task class:** Surrogate and reduced-order model training

<a id="source-40a141b5836981068a21df297df43afd"></a>

## Techno-economics and life-cycle assessment

Source: [Notion](https://app.notion.com/p/40a141b5836981068a21df297df43afd).

**Canonical form:** Additional authored calculations/accounting or task constraints as selected; implementation and empirical qualification are model specific.

**Role pattern:** Depends on whether the task evaluates, optimizes, fits or propagates uncertainty; economic/impact parameters retain their own provenance and units.

**Extent and characteristic relations:** Process extension profile. Process outputs coupled to explicit costing or impact models, with currency/base year, system boundary, allocations and data validity separated from the physical flowsheet.

**Answer as stated:** Solve or Extremize

**Answer kind:** [Solve](answer-kinds.md#source-40a141b5836981d28ffbc1a901cf1364); [Extremize](answer-kinds.md#source-40a141b58369810e8c01df60ebe98800)

**Task class:** Techno-economics and life-cycle assessment

<a id="source-40a141b58369810a8c0ce5b89fe1dc4d"></a>

## Uncertainty propagation, global sensitivity

Source: [Notion](https://app.notion.com/p/40a141b58369810a8c0ce5b89fe1dc4d).

**Canonical form:** Supported local propagation, sampling/quadrature or declared approximation/global-sensitivity estimator. Moment existence, sample failures and statistical evidence are reported rather than assumed.

**Role pattern:** Uncertainty treatment is explicit alongside the numerical input role; sampled/conditioned cases retain the source law.

**Extent and characteristic relations:** Process analysis extension. A declared probability model/dependence or qualified fitted covariance linked to selected process outputs and operating cases.

**Answer as stated:** Expect

**Answer kind:** [Expect](answer-kinds.md#source-40a141b5836981f0a45ad165770120dd)

**Task class:** Uncertainty propagation, global sensitivity

<a id="source-40a141b5836981d5a486da2894ab6499"></a>

## Unit rating vs design

Source: [Notion](https://app.notion.com/p/40a141b5836981d5a486da2894ab6499).

**Canonical form:** An admitted algebraic/feasibility task, or optimization when a criterion selects among feasible designs.

**Role pattern:** Rating fixes geometry and solves performance; design releases selected scalar/block design quantities and introduces specified targets. Closure requires applicable count/matching/rank checks, not a universal one-for-one swap.

**Extent and characteristic relations:** Process profile. The same unit/composite definition under different explicit operating specifications and design targets.

**Answer as stated:** Solve

**Answer kind:** [Solve](answer-kinds.md#source-40a141b5836981d28ffbc1a901cf1364)

**Task class:** Unit rating vs design
