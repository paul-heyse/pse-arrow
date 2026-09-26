# Process-simulator design principles

**Version 1.1 · 2026-09-25** · Domain profile for process simulation software:
steady-state and dynamic flowsheet simulation, optimization and parameter estimation.
Refines the [core design principles](../../core/design-principles.md) under their layering
rules (§B). It adds and tightens; it never relaxes a core principle. It names no specific
library — the repository binding does that.

## Functional target

A best-in-class process simulator lets an engineer author physically typed, reusable unit
and property models once. It composes them into flowsheets and case studies, and solves
them — square simulation, optimization, dynamics, parameter estimation — with results that
are physically meaningful, diagnosable when they fail, and fast enough for interactive edit
and re-solve loops, sweeps and recycles. The workloads a design must serve:

- the **edit → re-solve loop** on one flowsheet, where a value change must not rebuild structure;
- **studies** over many cases: sweeps, sensitivities, optimization, fitting;
- **recycles and tears**, where convergence behaviour and diagnostics dominate the user's experience;
- **dynamics** from the same model definitions as steady state;
- **extension** by adding a unit operation, a property or reaction model, or a solver.

For this profile, *correctness* in principles §1 explicitly includes physical consistency and
numerical integrity.

## Architectural application

Core AP-01–AP-06 apply to the organization of the simulator: separate authored physics,
provider integration, numerical policy, workflow composition and representation concerns;
trace the consumed contract when replacing a solver; compose studies from shared model
operations; keep physical meaning authoritative; expose lifecycle and capabilities; and
exercise policy and admission with their actual dependencies.

A new unit using existing physics is an ordinary extension. A new physical concept can
legitimately change the core contract. Neither case requires a universal plugin framework.
Numerical tests are necessary for numerical claims; they do not by themselves establish
modularity or local reasoning. This version retains PS-01–PS-13 and their gate semantics;
its review additions are routed to Core 3.0's architecture-first slots.

## Principle index

| ID | Principle | Level | Refines | Gate |
|---|---|---|---|---|
| PS-01 | Dimensioned quantities and explicit conventions | MUST | DP-02 | PS-G1 |
| PS-02 | Declared material systems and property validity | MUST | DP-02, DP-15 | PS-G1 |
| PS-03 | Conservation by construction, verified by closure | MUST | DP-06, DP-23 | PS-G1 |
| PS-04 | Well-posed before solving | MUST | DP-03 | PS-G2 |
| PS-05 | Topology, equation structure and solve order are distinct | MUST | DP-07 | PS-G2 |
| PS-06 | The formulation owns domain validity and smoothness | MUST | DP-08 | PS-G3 |
| PS-07 | Derivatives and scaling belong to the model contract | MUST | DP-11, DP-15 | PS-G3 |
| PS-08 | Initialization and convergence are declared strategies | MUST | DP-05, DP-12, DP-19 | PS-G3 |
| PS-09 | Solver selection follows problem class and capability | MUST | DP-13, DP-15 | G7 |
| PS-10 | Solve outcomes are truthful and independently verified | MUST | DP-19, DP-21 | PS-G3 |
| PS-11 | One model serves every analysis mode | SHOULD; MUST for warm-start and reuse dependencies | DP-06, DP-09, DP-10 | G6 |
| PS-12 | Results are physical and honestly scoped | MUST | DP-21, DP-22 | G7 |
| PS-13 | Shared model checks and reference validation | SHOULD | DP-16, DP-23 | — |

## Principles

### PS-01 — Dimensioned quantities and explicit conventions

**MUST · PS-G1 · refines DP-02.** Every physical quantity carries its dimension and unit. Basis
(mass, molar, volumetric; per unit time or not), reference state (for enthalpy, entropy and
Gibbs energy), gauge versus absolute pressure, and sign conventions (heat and work into or out
of a unit) are part of the type. Unit conversion happens only at declared boundaries and only
between compatible dimensions; a number never crosses an interface without its dimension and
basis. Dimensionless quantities state what they are ratios of.

**Audit.** Can a molar flow reach an equation expecting a mass flow, or a gauge pressure one
expecting absolute, without rejection or explicit conversion? Where do unit conversions happen,
and are they checked? Is the reference state of every energy quantity recoverable?

### PS-02 — Declared material systems and property validity

**MUST · PS-G1 · refines DP-02, DP-15.** Components, phases, the phase equilibrium assumption,
property methods and their parameters are declared, not implied by code paths. Every property
model states its validity envelope — temperature, pressure and composition ranges, phases and
the states it supports — and the derivatives it provides. Evaluation outside the envelope is a
diagnosed state: it is rejected, or it is an explicitly selected extrapolation policy recorded
with the result. It is never silent extrapolation. Property parameters are versioned data with
provenance.

**Audit.** What happens when a solve iterate leaves a correlation's range? Can a result be
traced to the property method, its parameters and their source? Does a provider declare which
derivative orders it supplies?

### PS-03 — Conservation by construction, verified by closure

**MUST · PS-G1 · refines DP-06, DP-23.** Material, energy and — where modelled — momentum balances
are derived from declared contributions (streams, reactions, heat and work terms, accumulation),
not hand-written per unit model. Every converged result can be checked for balance closure, per
unit and for the flowsheet, independently of the equations the solver used. Element balances
hold across reactions.

**Audit.** Does a new unit model obtain its balances from contributions, or re-write them? Is
there a closure check on converged results, with a tolerance tied to the solve tolerances?

### PS-04 — Well-posed before solving

**MUST · PS-G2 · refines DP-03.** Each variable's role — fixed specification, free unknown,
initial guess, observed measurement, design decision — is explicit and never inferred from
whether a value happens to be present. Before a solver runs, degree-of-freedom analysis and
structural analysis (structural rank; decomposition into well-, over- and under-determined
parts; block triangularization) establish that the problem is structurally well-posed for its
analysis mode. Over- and under-specification are diagnosed with the offending variables and
equations named, in terms of the model's units, streams and ports.

**Audit.** Can a structurally singular problem reach a solver? When it is rejected, does the
diagnostic name the model elements responsible? Is an initial guess ever mistaken for a
specification?

### PS-05 — Topology, equation structure and solve order are distinct

**MUST · PS-G2 · refines DP-07.** Flowsheet topology (units, ports, streams, and information links
such as controllers), equation–variable incidence, and the solve order (blocks, tear sets,
sequential or simultaneous partitions) are three different structures. Each is derived
explicitly from its authority, keeps its own identities and mapping back to model elements, and
is never used in place of another — for example, a stream graph is not a dependency graph, and a
tear stream is a solve decision, not a model edit.

**Audit.** What does each graph's edge mean? Is tear selection derived from the topology, or
does it alter it? Can a solve-order block be mapped back to the units and equations it contains?

### PS-06 — The formulation owns domain validity and smoothness

**MUST · PS-G3 · refines DP-08.** Functions with restricted domains (logarithms, roots, divisions,
fractional powers) carry their domain obligations explicitly, and those obligations are recorded
from the authored model before any symbolic rewriting. Phase appearance and disappearance, flow
reversal, and discontinuous correlations are handled by a declared formulation policy —
complementarity, smoothing with a stated parameter, or explicit discrete modes — with its
approximation stated. The problem handed to a solver meets the smoothness that solver assumes,
or the violation is diagnosed.

**Audit.** Can simplification remove a domain guard? Which formulation policy governs each
discontinuity, and is its approximation reported with the result?

### PS-07 — Derivatives and scaling belong to the model contract

**MUST · PS-G3 · refines DP-11, DP-15.** Every derivative has a declared source (symbolic,
automatic, provider-supplied or finite-difference approximation). *Exact* is claimed only when
every link in the chain — equations, property providers, custom kernels — supplies exact
derivatives of the required order. Derivative correctness is established with the confidence
the use demands; comparison with finite differences is a common check where doubt remains. Nominal magnitudes, variable and constraint scaling, and conditioning diagnostics are
model attributes, not solver afterthoughts. Tolerances are reported as scaled or unscaled.

**Audit.** Where does each Jacobian and Hessian entry come from, and what establishes that it is
correct? Can a missing provider derivative silently become zero or a finite
difference? Are scaling factors declared with the model, and are conditioning problems diagnosed?

### PS-08 — Initialization and convergence are declared strategies

**MUST · PS-G3 · refines DP-05, DP-12, DP-19.** Initialization is a declared, staged strategy —
sequential passes, simplified submodels, continuation or homotopy — whose steps are inspectable,
and it never mutates the specification it initializes: temporary fixes and relaxations are
scoped and always restored, including on failure. Recycle convergence is a declared policy (tear
selection method, convergence method, tolerances, iteration limits, acceleration) with its
iteration history recorded. Initial guesses and warm starts are inputs with provenance.

**Audit.** After a failed initialization, is the model exactly as specified before? Which policy
chose the tear set and convergence method, and can the history be inspected?

### PS-09 — Solver selection follows problem class and capability

**MUST · G7 · refines DP-13, DP-15.** The problem class — square nonlinear system, LP, QP,
conic, NLP, mixed-integer, DAE — is derived from the model's structure and analysis mode, not
assumed. The solver is selected by declared capability for that class (derivative orders,
bounds, sparsity, warm start, cancellation) and explicit policy, and the selection is recorded.
Established numerical solvers own iteration, globalization and factorization; the simulator does
not reimplement them. A class the configured solvers cannot handle is rejected before solving.

**Audit.** Is the class derived or assumed? Does any own code reimplement a Newton step, line
search or factorization that a qualified solver provides? What happens when no solver supports
the class?

### PS-10 — Solve outcomes are truthful and independently verified

**MUST · PS-G3 · refines DP-19, DP-21.** Every solver status maps to a typed outcome — for
example, converged with the tolerances achieved, locally infeasible, iteration or time limit,
restoration failure, evaluation error, user cancellation. Status strings are never parsed at
the point of use. A post-solve check, independent of the solver's own claim, verifies residuals,
bounds, domain obligations and balance closure in physical terms. A result that is not
converged is never published as a solution; a partial or best-iterate result is published only
as such. Failures name the units, streams, equations or variables involved.

**Audit.** Can an iteration-limit result be read as a solution? Is there a post-solve check
independent of the solver's status? Does a failure diagnostic point at model elements, or only
at solver indices?

### PS-11 — One model serves every analysis mode

**SHOULD, and MUST for warm-start and reuse dependencies · G6 · refines DP-06, DP-09, DP-10.**
Steady state, dynamics, optimization, sensitivity and parameter estimation are analysis modes
declared over the same model definitions plus a case, not separate models. Studies over many
cases reuse prepared structure and evaluators and bind only values per case. A warm start, an
initialization result or a reused factorization is a declared dependency of the result it
influences. Dynamic modes declare the differential index, how consistent initial conditions are
computed, event and discontinuity handling, and time discretization or integration policy.

**Audit.** Does switching mode require re-authoring the model? Does a value-only sweep rebuild
structure? Is a warm start recorded as an input of the result it changed?

### PS-12 — Results are physical and honestly scoped

**MUST · G7 · refines DP-21, DP-22.** Results are reported in physical units against model
identities (units, streams, ports, phases, components), with the basis and reference state
stated. Derived quantities — duals and shadow prices, sensitivities, parameter covariances and
confidence intervals — state the conditions under which they are valid (active set, second-order
conditions, local linearization, statistical assumptions) and are withheld or flagged when those
conditions are not met.

**Audit.** Can a user read any result without knowing solver indices? Are sensitivities or
covariances reported where their validity conditions fail?

### PS-13 — Shared model checks and reference validation

**SHOULD · refines DP-16, DP-23.** Correctness properties common to every unit and property
model — degrees of freedom, balance closure, derivative consistency, scaling sanity, envelope
rejection, initialization from default guesses — are checked by shared means where checks are
warranted, rather than re-authored for each model, so a new model gains them without new code.
Where reference data or reference simulators exist, physical behaviour is compared against them
with declared tolerances and conditions, and thermodynamic models are consistent (for example,
Gibbs–Duhem and fugacity equality at equilibrium). Agreement with a reference is evidence, not
proof.

**Audit.** Would a new model gain the common checks without re-authoring them? Where a claim of
physical fidelity is made, what reference supports it?

## Profile gates

These add to the core gates; G9 independently assesses architectural fitness.

| Gate | Fails when… | Principles |
|---|---|---|
| PS-G1 — Physical consistency | A dimension, basis, reference-state or convention mismatch, an unenforced property envelope, or a non-closing balance can reach a result. | PS-01, PS-02, PS-03 |
| PS-G2 — Well-posedness | A structurally ill-posed problem can reach a solver, or is rejected without naming the model elements responsible. | PS-04, PS-05 |
| PS-G3 — Numerical integrity | A result can be reported as a solution without meeting its declared convergence, tolerance, derivative or domain contract, or a solver status is misclassified. | PS-06, PS-07, PS-08, PS-10 |

## Common false positives in simulators

| Attractive claim | Hidden defect to check |
|---|---|
| "The solver reported success." | Scaled tolerances met while unscaled residuals, bounds or balance closure fail. |
| "Derivatives are exact." | One provider or kernel in the chain supplies finite differences or nothing. |
| "Units are checked." | Dimensions are checked but basis, reference state or gauge/absolute are not. |
| "It matches the reference simulator." | Different property parameters or reference states, or matching only at the reference's own convergence tolerance. |
| "Initialization is robust." | It succeeds by leaving temporary fixes or relaxations in the model. |
| "It is equation-oriented, so it is fast." | Structure is rebuilt per case, or per-iteration evaluation crosses a language boundary. |
