---
title: Numerical execution
status: current
---

# Numerical execution

This area turns an admitted, prepared case into native numerical work and truthful
outcomes. It owns structural admission, the single resolved numerical policy,
initialization and recycle strategies, class-specific adapter selection, attempt-owned
native execution and the result envelope. Libraries own every iterative algorithm; the
project owns contracts, coordinate transport, eligibility and independent original-space
assessment. The main owners are `crates/pse-structural`, `crates/pse-backend-native`,
`crates/pse-runtime/src/math/` and `crates/pse-runtime/src/workflow/`, with numerical
vocabulary in `pse-model::numerics` and `pse-math::{numerics,normalization}`.

## 15. Structural analysis and diagnostics

Structural analysis answers whether the selected equations can determine the selected
unknowns before any value is computed, and gives initialization and recycle strategies
their block and cycle structure. Numerical diagnostics answer what a completed attempt
actually achieved in original coordinates. Both are consumers of the complete admitted
case from the compiler ([§14](mathematics-and-compilation.md#section-14)); neither
evaluates a partial inventory as if it were complete.

IDAES `DiagnosticsToolbox` parity is not a current capability: there is no
threshold relation, no named structural/numerical check catalogue and no
`report_*`/`assert_*` view family. The checks below are the ones the native lifecycle
actually performs.

### 15.2 Structural checks without numerical values

Structural admission is class-aware and consumes the compiler's existing analysis
rather than repeating matching (`pse-backend-native::structural`):

| Mode | Admission rule | Refusal |
|---|---|---|
| Roots (square equations, declared fixed point) | Every original equality row and free variable participates in the maximum matching | `ProblemError::Structural` naming overdetermined rows and underdetermined columns by semantic ID |
| NLP | Every equality row matches; inequalities and genuine optimization degrees of freedom remain admissible | Overdetermined equality rows by semantic ID |
| Native feasibility | The capability admits redundant affine/conic rows to its native feasibility algorithm; the original scope and matching observations remain retained | Unsupported scope or capability |
| Point evaluation | No free variable; evaluate every original obligation directly | Failed original checks; no matching or rank claim |

[ADR-0145](../../adr/0145-declared-execution-and-composed-qualification.md) (proposed; authorized implementation) makes Root intent require complete original square equality matching even for an explicitly selected NLP or factorable adapter. Optimization uses the selected capability's structural policy. `StructuralAssessment` retains scope, original variable/equation roles, bounds, matching and unmatched identities; `RouteDecision` retains intent, requested selection, eligibility, representation, lexicographic realization and refusals. Diagnose, conformance and solve preparation consume these facts. Generated `runtime.route_decisions` and `runtime.structural_assessments` retain them without a native attempt.

A scope marked partial is refused; structure over a subset cannot certify the case.
Fixed-point and Picard admission observe the original residual support, not the
iteration map. A reused matching witness is checked linearly against the current row
inventory, equality membership and incidence edges before it is trusted; opaque
callback oracles receive the same analysis from their original residual Jacobian.
Physical unit consistency is established earlier by physical admission
([§8](physical-semantics.md#section-8)), so it is not a structural check here.

Structural matching never certifies numerical rank. A structurally square system can
still be singular at a point; that is a numerical observation (§15.4, §15.5).

### 15.3 Structural algorithms

`pse-structural::incidence` is a checked semantic boundary around pounce-presolve:

- **Incidence.** Complete selected-case equality rows (including rows with no
  incidence), free columns, conservative all-branch support of guarded expressions and
  objective/inequality coupling. Original contribution provenance (instance, output
  ordinal) survives even where matching deduplicates pairs.
- **Matching, Dulmage–Mendelsohn and BTF.** pounce-presolve's Hopcroft–Karp matching,
  DM partition (over-, under- and square-determined parts) and block triangular form.
  Library results are validated against the declared dimensions and equality coverage
  before they are converted back to semantic IDs.
- **Deterministic order.** rustworkx lexicographical topological sort breaks ties by
  semantic ID; petgraph and rustworkx supply components, condensation and cycle finding
  for flow and dependency projections (`projection`, `flowsheet`).
- **Block identity.** A block's `BlockId` hashes its scope and canonical membership, so
  identity is independent of execution position. A different valid topological order is
  not wrong; positional interpretation without the identity mapping is.
- **Coupling.** Blocks record conservative auxiliary coupling. Coupling and conditional
  partitions do not establish independent eliminability or independent subproblems.

The recursive matching search is qualified to 100,000 rows on a dedicated 32 MiB
thread stack; larger inputs are refused, not truncated. Graph admission checks node and
edge limits before allocation. Cancellation is checked between library stages.

### 15.4 Numerical checks that require values

After any native attempt, the adapter re-evaluates the original model at the returned
candidate through the original oracle (`pse-backend-native::quality`). The observation
never reuses a solver's possibly stale row buffer. It records:

- raw constraint values, original row bounds and signed equality residuals for finite
  equality rows only;
- per-side row and variable-bound violations in each source's declared physical unit,
  each paired with its resolved physical tolerance (§16.6);
- integrality violations for discrete domains;
- a dimensionless maximum of violation divided by its tolerance, the only aggregated
  quantity, because unlike units are never combined;
- original-coordinate stationarity and complementarity under the minimization
  convention `L = sense*f + lambda*g - z_lower*x + z_upper*x`, or an explicit dual
  error when multipliers or derivatives are missing or invalid.

Physical conservation checks (`runtime.physical_checks`) are computed separately from
mathematical feasibility by the workflow completion owner; balance closure is owned by
[§10](models-and-composition.md#section-10). Presolve infeasibility proofs are
confirmed on per-bound tolerance-expanded original normalized facts and retain original
row, column and contribution identities. Missing evidence carries a typed unavailable
reason; it is never zero or a fabricated NaN.

### 15.5 Advanced analyses

> Decision: [ADR-0118](../../adr/0118-one-kkt-point-analysis.md) — parametric
> sensitivity, reduced Hessian, covariance and uncertainty propagation with PS-12 validity,
> through one KKT-point analysis at the qualified candidate, in original coordinates (a
> FERAL factor with inertia and `pounce-sens-core`), for every NLP and QP route. Implemented
> for the NLP routes (Ipopt, POUNCE and SCIP's fixed-assignment re-solve): the analysis
> (Plan 22 S0), parametric sensitivity and the reduced Hessian (S1,
> [§15.5.1](#section-15-5-1)), covariance and intervals (S3,
> [§19.4](workflows-and-results.md#section-19-4)) and propagation (S4,
> [§19.8](workflows-and-results.md#section-19-8)). A quadratic program reaches the analysis
> by routing (Plan 22 N5, implemented): automatic routing of a sensitivity request prefers
> an adapter whose candidate the analysis differentiates ([§18.9](#section-18-9)), so the
> program runs on an NLP route; an explicitly selected coefficient or cone adapter solves
> it and withholds the quantities as `no_local_analysis`. POUNCE-convex's own QP
> sensitivity is excluded as a second mechanism. ADR-0118 supersedes ADR-0107, whose
> two-route mechanism (POUNCE `SensSolve` and an Ipopt barrier replica) and presolve
> restriction no longer apply.
>
> Decision: [ADR-0109](../../adr/0109-pounce-l1-and-convex-methods.md) — whole-model
> infeasibility explanation through the explicit POUNCE ℓ1 route (Plan 22 N3,
> implemented, [§18.3](#section-18-3)).

Each analysis is opt-in or bounded, and none replaces the original candidate:

| Analysis | Owner and mechanism | Limit |
|---|---|---|
| Infeasibility rays, IIS, basis ranging, feasibility relaxation; fixed-commitment LP duals, basis-inverse rows, the presolved LP and the root cut pool | HiGHS native diagnostics (`highs::diagnostics`) on copied models ([§18.10](#section-18-10)), restored to original coordinates except the basis-inverse and presolve views, which stay in the native normalized model | Diagnostic data only; a relaxed point is never a primal solution. MIP IIS concerns the LP relaxation, and fixed-commitment duals are conditional on the MIP's discrete solution. Penalties are explicit physical declarations, never inferred from units |
| Presolve rank diagnostics | pounce-presolve equality-rank pass | No objective-changing remedy |
| Numerical PSD qualification | Resource-bounded faer eigenanalysis with residual qualification | Explicit opt-in; never repairs the matrix (§18.10) |
| KKT-point analysis | For an optimizing NLP candidate, through the one NLP runner (Ipopt, POUNCE and SCIP's fixed-assignment re-solve): one active-set KKT system in original coordinates, factored by FERAL with certified inertia and a Hager–Higham condition estimate (`kkt`, `Evidence::local`, [§15.5.1](#section-15-5-1)) | A local verdict on activity, LICQ and curvature (`sufficient`, `negative_curvature`, `singular`, `undecided`); an analysis that cannot run is a typed `Unavailable`; coefficient, cone and root routes have none |
| Parametric sensitivity and reduced Hessian | On request (`SolveSettings.sensitivity`), read from the same analysis over the model's parametric program with the parameters pinned ([§15.5.1](#section-15-5-1)) | Local and first-order, valid while the active set holds; each quantity is certified or withheld with its reason (`runtime.local_validity`) |
| Verified infeasibility certificates | Clarabel's rays, and HiGHS's when its diagnostics request them, recovered to original coordinates and verified against the original data (`runtime.infeasibility_certificates`, [§18.10](#section-18-10)) | Only a verified ray at full native accuracy carries the `certificate` assurance; a ray is never a candidate |
| Sum-of-squares bound | `ModelingPackage::sos_bound`: the moment relaxation of `pounce-convex` over the polynomial projection of the case's factorable program (`execution::sos`, Plan 22 N5), at a stated order | A floating-point SDP value labelled `sos_bound_nonrigorous`: never a certified global bound and never the basis of an objective-bound check; a program that is not an exact polynomial, or whose moment basis exceeds 256 monomials, is refused (`sos_bound_labelled_nonrigorous`, runtime units) |
| Jacobian conditioning | FERAL sparse LU of a square Jacobian in a stated scaling, with a Hager–Higham 1-norm estimate (`conditioning::jacobian_condition`), reported as the `jacobian.condition_estimate` modeling finding | An estimate and a lower bound; none when the LU is singular |
| Fit response rank and conditioning | faer pivoted LU for implicit response, bounded SVD for scaled observation rank; the right singular vectors in `runtime.response_directions` | Rank alone implies no covariance, which has its own rule ([§19.4](workflows-and-results.md#section-19-4)); global identifiability is not claimed |

The Jacobian conditioning row is Plan 22 N4 (*Tested* by
`jacobian_condition_estimate_matches_dense_reference`, native backend units). The KKT-point
analysis (Plan 22 S0) replaced N4's post-solve curvature check, its two-pass assembly and
its `second_order.unavailable` metric.

Authored diagnostic profiles also drive bounded faer Jacobian SVD, selected linear and
Jacobian optimization analyses, and nonlinear elastic/deletion explanations. The nonlinear
explanation (`ModelingPackage::explain_nonlinear`) is a deletion filter over the outer
equations in which every attempt solves the remaining original rows on the explicit ℓ1
route ([§18.3](#section-18-3)); no elastic reformulation is built, and a least-infeasible
stop is a local obstruction, never a proof of infeasibility. A global conclusion is a
separate, explicit analysis (`ModelingPackage::certify_infeasibility`,
[§18.10.1](#section-18-10-1)) whose result sits beside the local explanation; the local
explanation never falls back to it. Reports distinguish observations, candidates,
inconclusive limits and unattempted work; a deletion heuristic is not a proof of a minimum
infeasible subsystem. Finite studies retain per-case outcomes and explicit predecessor
dependence. The generic profile/knowledge contract is proposed
[ADR-0101](../../adr/0101-modeling-analysis-knowledge.md).

#### 15.5.1 KKT-point analysis and parametric sensitivity

> Supplement: [ADR-0144](../../adr/0144-selected-mathematical-realizations-and-square-response.md) (proposed; authorized implementation).

Regular-square Root response uses complete original equality matching and `F_x X_p = -F_p`, independently of optimizing KKT analysis. Existing sensitivity requests admit Root parameter derivatives, while Root reduced-Hessian and covariance-propagation requests refuse. Publication supplies physical primal state derivatives and explicit rank/cutoff/backward-error/neighborhood evidence; no objective, multiplier or KKT-only fields are fabricated. An unqualified local response is withheld independently of the base root.

> Decision: [ADR-0118](../../adr/0118-one-kkt-point-analysis.md) — one KKT-point analysis
> in original coordinates (Plan 22 S0, S1 and the advanced step Y5c2, implemented for the
> NLP routes, which automatic routing prefers for a sensitivity request, N5).

**The analysis** (Plan 22 S0, `pse-backend-native::kkt`). The one NLP runner
([§18.7](#section-18-7)) analyses the candidate against the original model when the step's
`kkt::Analysis` asks for it. `Analysis::for_intent` asks under the `optimize` intent only,
and an analysis requested under another intent is a contract error. A report without a
candidate or an original observation gets no analysis; a candidate whose multipliers could
not be recovered, or that is infeasible in original coordinates, gets a typed `Unavailable`
(`Multipliers`, `Infeasible`). Otherwise one active-set KKT system is assembled
from the original model's derivative programs at the recovered candidate, in the layout
`[x; active rows; active bound rows]`: every variable keeps its row, and an active bound,
including an `l = u` pin, is a row of its own (`−eⱼ` for a lower bound, `+eⱼ` for an upper
one), so each bound multiplier names its variable. The derivatives, the candidate and its
recovered multipliers are original values. FERAL factors the normalized congruence
`K̃ = P·K·P/S_f`, with `P` built from the declared normalization of variables, rows and
objective: pivoting, zero-pivot detection and the condition estimate are scale dependent and
use the matrix the numerical policy declares well scaled, while inertia is invariant under
the congruence, so every verdict is coordinate free. `KktFactor` answers in original
coordinates through the explicit back-map and implements `pounce-sens-core`'s
`SensBacksolver`.

- *Activity.* A row or bound is active when its slack is within its acceptance budget, or
  when its normalized multiplier exceeds the resolved stationarity budget and its
  normalized slack: an interior-point candidate keeps an active limit's slack at `μ/z`,
  which may exceed the budget, and an inactive limit's multiplier at `μ/s`. An active
  constraint is strongly active when its normalized multiplier exceeds the stationarity
  budget and weakly active otherwise; equalities and pins are always strongly active.
- *LICQ* is read from the inertia of `[I Aᵀ; A 0]` over the active gradients `A`
  (`Licq::Independent`, or `Dependent` with its deficiency), so a dependent active set and a
  singular reduced Hessian are distinct verdicts.
- *Curvature* (`Curvature`) is the reduced Hessian's inertia: the KKT inertia less that of
  the constraint block. `negative_curvature` along a direction satisfying every active
  constraint refutes a local minimizer; `singular` is a zero eigenvalue on the null space
  of the active gradients; `sufficient` is positive definiteness on the null space of the
  strongly active constraints, which is second-order sufficiency; `undecided` means weakly
  active constraints exist and neither test decides. With weakly active constraints the
  same assembly with their rows released tests the null space of the strongly active ones.
- *Result.* `Evidence::local` holds a `KktPoint` (the activities, LICQ, curvature, inertia,
  the Hager–Higham 1-norm condition estimate and the normwise backward error of a refined
  backsolve) or a typed `Unavailable` (`Multipliers`, `Infeasible`, `Hessian`, `Limit` or
  `Failed`). It is typed evidence, never a metric.

SCIP's fixed-assignment re-solve ([§18.10.1](#section-18-10-1)) runs through the same
runner, so an adopted candidate keeps its analysis, conditional on the assignment. The
coefficient, cone and root runners have no local analysis. *Tested* by
`kkt_inertia_certifies_second_order` (Ipopt and POUNCE),
`second_order_verdicts_follow_the_inertia`, `licq_failure_distinct_from_singular_curvature`,
`kkt_factor_backsolve_matches_dense_reference`, `local_analysis_unavailable_is_typed`,
`interior_point_active_bound_beyond_its_tolerance_is_active` and
`fixed_assignment_resolve_keeps_local_analysis` (native backend units).

**Parametric sensitivity** (Plan 22 S1). `SolveSettings.sensitivity` requests it as a
`SensitivityRequest`: declared parameters of the solved case by identity (at least one, none
repeated), whether to add the reduced Hessian, and an optional propagation
([§19.8](workflows-and-results.md#section-19-8)). It is admitted under the `optimize` intent
only and enters the solver profile identity. The solve keeps the parameters fixed; they are
pinned only in the analysis. Preparation builds the view's parametric program once
(`CasePlan::parametric`: the plan's columns followed by the parameters, at second order,
cached with the view; `pse.compiler.modeling-parametric.v1`), and after qualification the
analysis pins the parameter columns at their values (`transform::Pinned`, sIPOPT's pin
formulation). A pin's multiplier is not read from any solver: it is computed as the
original stationarity of the parameter column, `∂f/∂p + (∂g/∂p)ᵀλ`, which by the envelope
theorem is also `df*/dp`. `pounce-sens-core` then reads the parametric step over the pin rows
from the factor: `dx/dp`, the derivatives of the row and bound multipliers and `df*/dp`, in
original physical units per parameter unit whatever presolve removed, and on request the
reduced Hessian `H_R = d²f*/dp²` with its eigen-decomposition. Over the pin rows the Schur
reduction returns `−H_R`, and the analysis negates it. The eigen-decomposition is taken on the
normalized factor, where the dimensionless `S_p·H_R·S_p/S_f` does not depend on the choice
of units; both the physical and the normalized matrix are published. `df*/dp` and the
reduced Hessian follow the authored objective sense. A request whose parametric callbacks
cannot be built withholds its quantities and never refuses the solve.

The quantities are certified only at a candidate qualified `Stationary` or better, with a
recovered multiplier for every original row and bound that passes original-coordinate
complementarity (presolve is never switched off to recover one), and only when the pinned
parametric KKT point has independent active gradients, no weakly active constraint and
second-order sufficiency. Otherwise each requested quantity is withheld.
`runtime.local_validity` publishes one `LocalValidity` row per requested quantity: whether
it is certified or the `WithheldReason` (`no_candidate`; `no_local_analysis` on a
coefficient, cone or root route; `multipliers_unrecovered`, `complementarity_failed`,
`not_stationary`, `analysis_unavailable`, `licq_failed`, `weakly_active`,
`second_order_failed` or `backsolve_failed`; the fitting and propagation reasons of
[§19.4](workflows-and-results.md#section-19-4) and
[§19.8](workflows-and-results.md#section-19-8)) with its typed detail, whether the quantity
is conditional on a discrete assignment, and the LICQ, strict-complementarity and
second-order verdicts, the weakly active count, the condition estimate and the backsolve
residual of the point it was read from. Certified data are in
`runtime.parametric_sensitivities` and `runtime.reduced_hessians`; a withheld quantity has
no data rows. The duals of a step whose sensitivities are certified are
`sensitivity_certified` (`DualQualification`); otherwise they stay
`evaluated_kkt_not_sensitivity_certified`. Ipopt, POUNCE and the SCIP re-solve give the same
quantities.

Missing or incorrect recovered multipliers withhold sensitivity under the original KKT
criteria. Native presolve admits only transformations with supported multiplier transport
([§18.5](#section-18-5)); supported affine elimination recovers the singleton row's
multiplier and agrees with the unreduced analytic response. *Tested* by
`sensitivity_matches_analytic_nlp` (Ipopt and POUNCE, including the first-order prediction
of a re-solve at a perturbed parameter), `reduced_hessian_sign_pinned`,
`sensitivity_withheld_when_sosc_fails`, `sensitivity_withheld_when_weakly_active`,
`sensitivity_withheld_when_licq_fails`,
`sensitivity_withheld_when_multiplier_fails_complementarity`,
`sensitivity_backend_independent`, `sensitivity_survives_presolve` and
`automatic_presolve_preserves_original_multiplier_and_sensitivity` (linked native backend
units, explicit force-validation, 2026-10-04), and
`sensitivities_published_with_local_validity` and
`sensitivity_withheld_without_local_analysis` (runtime units). The comparison against
Ipopt's sIPOPT that ADR-0118 names agrees within 1e-6 on a nondegenerate program with an
active bound (`test_sensitivity_agrees_with_ipopt_sens`, `just parity`). A quadratic program with a request
routes to an NLP adapter and receives the same analytic quantities
(`qp_sensitivity_through_kkt_analysis`, runtime units).

**Advanced step** (Plan 22 Y5c2, `kkt::advance`). A request may keep its pinned
parametric factor (`Sensitivity::retain`, set by `PreparedSolve::retaining_factor`). When
the step's sensitivities are certified, the runner keeps an `Advance` (the factor, its pin
rows, the candidate with its multipliers, row values and bounds, and the parametric
Jacobian) in the worker's `Retained` state beside, not inside, the adapter's session; the
worker charges its bytes to the job's allowance before the next step and releases it when
the allowance cannot hold it, recording none as kept (`Parametric::retained`). `predict`
answers at new parameter values by one `parametric_step` backsolve, `Δw = K⁻¹·(−e_pin)·Δp`,
without evaluating or factoring anything. It refuses, with a typed `Fallback`, when no
factor is retained, the parameters differ, the backsolve fails, or the step changes the
active set: an active multiplier changes sign, or an inactive row (on its linearization
through the parameter columns) or bound is driven beyond its budget. While the active set
holds, the prediction of a problem whose KKT conditions are linear in the parameters is its
solution. The rolling horizon's advanced-step controller is its consumer
([§13.6](workflows-and-results.md#section-13-6)). *Tested* by
`advanced_step_predicts_within_the_active_set` and
`advanced_step_needs_certified_sensitivities` (native backend units).

### 15.6 Reports

Structural refusals and native failures reach callers as structured values, not prose.
Every public error carries a `pse-diagnostics` code and failure class
([§23.2](operations-and-validation.md#section-23-2)); workflow boundary diagnostics
(`pse-model::diagnostic::BoundaryDiagnostic`) add the stage, violated rule, affected
semantic IDs, typed observations and authored source locations enriched from the model
revision (`pse-runtime/src/workflow/diagnostics.rs`). Diagnostic capture is bounded and
cannot change a scientific or publication outcome.

Declared relational invariants write `runtime.diagnostics_findings` through
`pse-rules::invariants` for inspection and fixture validation. Solve outcomes, metrics
and assessments are result relations ([§19](workflows-and-results.md#section-19)); see
the [generated runtime relations](../../generated/relations/runtime.md) for columns.

## 16. Numerical policy and scaling

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> resolved
> numerical requirements and candidate assessment extend the shared vocabulary.

Numerical meaning is resolved once per admitted analysis, before presolve or any
adapter runs. The single resolved interpretation then feeds model normalization, native
stopping controls and original-space acceptance. Adapters never reinterpret raw
tolerances or infer magnitudes from trial values.

### 16.1 Resolved numerical policy and normalization

> Decision: ADR-0163 (proposed; authorized implementation).

The contextual engineering target separates conditioning from useful physical accuracy.
An inherited allowance resolves as `max(F,r*S)`, using the applicable shared physical
allowance, shared fraction and an independently admitted engineering characteristic scale.
Explicit requirements retain their existing absolute-plus-relative-times-nominal meaning.
The scale never follows an iterate or an untagged conditioning nominal.

`pse-model::numerics::NumericalPolicy` holds library-neutral controls: ID-keyed
analysis requirements, `strict_nominals`, `native_scaling`, independent KKT budgets
(stationarity, complementarity), an optional separately qualified acceptable-stop
budget, integrality, continuous and MIP gaps, and independent closure/incumbent policies.
`pse-math::numerics::resolve` combines it with the selected targets and sourced
declarations into an immutable `ResolvedNumericalPolicy`.


Ordinary engineering design starts with a normalized accuracy of `1e-3` (0.1% of a
meaningful frozen characteristic scale). The model-owned default supplies physical
fallback `absolute = 1e-3 * nominal`, independent stationarity/complementarity and
continuous/MIP gap budgets; explicit sourced requirements retain precedence. Integrality
keeps its separate `1e-8` discrete-feasibility requirement. These budgets do not establish
an output-error bound or make the same raw number meaningful in different units.

The reference packages use one shared physical accuracy policy in
`domain/models/numerical-policy.pse`. Individual reconstruction, transport, balance,
inventory and output-check slots reference its typed defaults. A distinct explicit value
remains an override; defaults are changed at the shared owner rather than tuned per fixture.
The initial temperature and power allowances are 0.1 K and 1 W. A 1 kg process-mass
allowance is the maintainer's guideline for mass-based analyses; the selected reference
models currently use molar quantities and have no mass-inventory policy slot. Other typed
magnitudes account for the small reference simulations; these provisional
settings are not an output-error certificate or a completed process qualification.
Authored integration `relative(global)` and `normalized_absolute(global)` select the
model-owned normalized default at source parsing; explicit numeric controls retain their
own values. Canonical declarations store the resolved numbers: changing a default requires
reparsing/recompiling authored sources, not mutating already-persisted declaration rows.
Generated conservation quadratures initially share the tightest consuming physical
inventory allowance, without an additional automatic tightening factor.

An analysis can later choose a distinct physical resolution for a decision threshold,
trace quantity, sensitivity or other named requirement. Integration local error, cumulative
closure, reference comparisons and native stopping retain independent meanings even when
their default values share an owner. Fixture output expectations are independent original
checks; they do not themselves become solver accuracy requests. A needed tighter solve uses
explicit numerical requirements.
Production tests check the implemented conditions and declared behavior: numerical stencil
coefficients and physical mesh spacing, assembly/boundaries, supported derivative/domain
conditions, termination and original-space acceptance. A process mesh-refinement journey
checks sampled output stability at a declared decision resolution, not empirical reproduction
of a theoretical convergence order. Focused numerical verification can use explicit finer
precision to distinguish implementation errors; it does not set ordinary simulation defaults.
Finite-difference step size, exact identities and interval-proof controls are not engineering
output tolerances. Estimated acceptance does not become certified evidence by changing a number.

A target is a variable, row, objective, observable or closure coordinate with its full
quantity type and unit. Each resolved target records:

- a nominal, converted into the target's own unit, frozen for relative acceptance;
- a positive coordinate scale; integer coordinates keep scale one so the lattice
  survives;
- an absolute tolerance in physical units and a dimensionless relative tolerance;
- the frozen budget `absolute + relative * nominal`;
- selected and overridden provenance for every field.

`pse-math::normalization::Normalization` is positive diagonal coordinate transport:
`x = Sx z`, normalized rows `= rows / Sr`, normalized minimization objective `= f / Sf`.
It carries derivatives, sparse coefficients, bounds, starts, duals and FBBT tapes in
both directions; it is not a unit conversion and not a new evaluator. Native
algorithmic scaling (for example Ipopt's gradient-based scaling) is a separate,
explicitly permitted step after normalization. Nonpolyhedral cone blocks share one
positive row scale. Hard guards are never relaxed by scaling.

For native NLP execution, one resolved feasibility budget τ is transported into
per-row coordinates `Srᵢ = εᵢ / τ`, where εᵢ is the original physical row budget.
Thus the native test `|violationᵢ / Srᵢ| ≤ τ` enforces that row's own budget instead
of imposing the tightest budget on every row. The map carries residuals, row bounds,
Jacobians, weighted Hessians and row multipliers together; variable and objective
coordinates retain their declared scales. Original KKT and sensitivity analysis use
the declared normalization after recovery. The consumed native map enters the
presolve/native layout identity, so a changed budget cannot reuse incompatible native
state. Both maps belong to the admitted attempt and are released before an Auto
fallback rebuilds them. *Tested* by
`heterogeneous_row_budgets_transport_callbacks_bounds_and_original_duals`,
`changed_row_budget_invalidates_native_layout_without_changing_declared_nominals`,
`native_row_maps_refuse_an_insufficient_admitted_attempt_allowance` and
`native_pressure_like_root_stops_with_heterogeneous_original_row_budgets`
(native backend controls, all native features and explicit force-validation, 2026-10-04).

### 16.2 Sources and precedence

> Decision: ADR-0163 (proposed; authorized implementation).

Engineering scale selection has its own precedence: analysis, case, model, applicable
provider, explicitly tagged range/capacity/reference difference, tagged quantity magnitude.
Equal-precedence conflicts refuse admission. Affine points use differences/ranges; cancelling
outputs use their own scale. Zero scale is valid; missing meaningful context uses a recorded
canonical fallback unless strict engineering completeness is requested.

| Rank | `NumericalSource` | Origin |
|---|---|---|
| 7 | `Analysis` | Requirements in the request's `NumericalPolicy` |
| 6 | `Case` | Selected case declarations |
| 5 | `Model` | Explicit model target declarations (`authored.numerical_requirements`) |
| 4 | `ModelHint` | Bound authored nominal/scale/tolerance annotations |
| 3 | `PropertyDefault` | Explicitly bound provider-output/property defaults |
| 2 | `DerivedNominal` | Qualified characteristic magnitude from original additive terms |
| 1 | `QuantityNominal` | Quantity registry nominal |
| 0 | `CanonicalFallback` | Recorded canonical-unit nominal of one |

Higher rank wins per field. Equal-rank conflicts, duplicate requirement identities,
requirements naming unselected coordinates, nonpositive or nonfinite magnitudes, a
unit on a normalized-coordinate requirement and a nominal that disagrees with its
scaling factor all fail resolution. A declaration cannot promote itself to an analysis
override. With `strict_nominals`, the canonical fallback is refused instead of
recorded. Authored nominal, scale and initialization annotations resolve into this same
policy. Block and causal/recycle projections carry original units, physical bounds,
integrality and provenance; they cannot manufacture a second default interpretation.

### 16.5 Identity, provenance and persistence

> Decision: ADR-0163 (proposed; authorized implementation).

Admitted goals, physical conversions, selected engineering context and provenance enter the
frozen acceptance identity. Refinement demands have a distinct consumed-work identity and
cannot loosen admission. Study members may bind a new context; iterations cannot. Generated
boundaries carry retained goal assessments and preserve supported older no-goal interpretation
without rewriting stored bytes; unsupported contract versions refuse explicitly.

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> completion records the actual request, effective settings and submitted start. Plan 22
> A4 (implemented) derives settings identity from serde and frames the start a step used,
> and any reuse of retained native state, into its lineage identity.

`ResolvedNumericalPolicy::key` frames the complete interpretation, including
provenance, and enters prepared-request identity. Changing a nominal, tolerance or source
therefore changes reuse identity even when a native library fingerprint could not tell
the difference. The native stopping budgets are derived from that interpretation
([§16.6](#section-16-6)), so the policy key already covers them; `ResolvedAccuracy::key`
frames the budgets themselves where a stamp needs them without the policy, as for the
session stamp of a declared root ([§17.4](#section-17-4)).

Execution settings are identified through serde, never through a hand-written field list:
`Controls::identity` (option values keep their native type and exact float bits),
`BackendSettings::identity` and `ResolvedAccuracy::key`. A new field therefore enters
identity without an edit to a hashing function. The solver profile identity
(`pse.solver.profile.v4`) frames the native session profile ([§17.6](#section-17-6)), every
control, the linked native build (library versions, the solver-image manifest and the
numerical contract, `pse.native.build.v1`; [§18.3](#section-18-3)) and the selection by its
registry spelling.

The lineage request identity of a completed algebraic step (`pse.completed.request.v2`,
published in `runtime.run_lineage`) frames the revision, instance, preparation, selected
request, profile and resolved policy together with the start the step actually used:
whether it reused retained native state, its predecessor attempt, whether the start was
submitted, the submitted seed by content and any partial start. Seed content
(`WarmStart::content_key`, `pse.native.seed.v3`, which frames an NLP seed's barrier value
and working set) excludes the run that produced the seed. The same request
seeded differently is therefore a different lineage, and the same seed produced by another
run is the same lineage ([§20.3](identity-and-publication.md#section-20-3)).

The resolved interpretation, candidate assessments and physical checks are published
as `runtime.resolved_numerics`, `runtime.candidate_assessments` and
`runtime.physical_checks`; public serialization consumes the completed assessment and
performs no second evaluation. Authored scaling schemes and diagnostics consume the
same resolved policy; comparing policies is a sequence of explicitly prepared analyses.

### 16.6 Derived native controls and original-space acceptance

> Decision: ADR-0163 (proposed; authorized implementation).

Declared scalar goals independently request output resolution, an inclusive quantitative
criterion, or both. Estimated evidence is the default; Certified needs an actual valid
certificate. Criterion-only goals add no unrelated digit requirement. Native feasibility and
root-selection obligations remain independent of output evidence. Compatible goals share
residual/factor preparation and output actions. Reserve fixed error before allocating
reducible contributions; conservative deterministic composition does not use statistical RSS.

Execution selectively refines within existing finite grants, stopping on resolution (including
a resolved violation), unavailable capability, nonprogress, precision limits or exhausted
permission. Completion consumes the final assessment once. Assess permits resolved violation
subject to other obligations; RequireSatisfied additionally refuses it. Unresolved goals retain
diagnostics and refuse the requested goal-qualified result. No goals means NotRequested and
adds no accuracy-only factors, proof searches or comparator solves.

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> `CandidateUse` gains `seed_only` and `diagnostic_only` and becomes the only acceptance
> rule every workflow consumes (Plan 22 A1, implemented). The incumbent of a relaxed global
> export, which the decision also makes `diagnostic_only`, arrived with the SCIP route (Plan
> 22 G3, implemented).
> Stopping budgets are resolved from the policy and are not a user control (Plan 22 A4,
> implemented).

From the resolved policy, `pse-backend-native::solve::ResolvedAccuracy::resolve` derives
normalized native stopping budgets: feasibility is the minimum of the comparable normalized
variable/row budgets; stationarity, complementarity, integrality and gaps come from the
policy. Neither the raw maximum nor the raw minimum of differently dimensioned
tolerances is ever used. `ResolvedAccuracy` is not a user control. The caller's `Controls`
choose time and iteration limits, threads, history, the Hessian mode (`HessianMode`:
`exact`, `limited_memory`, or `gauss_newton` for a least-squares fit objective only,
[§19.4](workflows-and-results.md#section-19-4)), reuse, the start policy and native
options, and carry no accuracy, so nothing a caller supplies can stand in for the resolved
value. Preparation resolves it once (`PreparedSolve::accuracy`; per block
for initialization; from the map's own policy for a causal map), and every adapter, runner
and qualification receives that value. A nested library solve resolves its own from its
budgets (`ResolvedAccuracy::from_policy`). Derived library options:

| Adapter | Derived from the policy |
|---|---|
| Ipopt and POUNCE | `tol`, `constr_viol_tol`, `dual_inf_tol`, `compl_inf_tol`; `bound_relax_factor = 0` and `honor_original_bounds`; acceptable-level options only when an acceptable budget is declared |
| KINSOL | `kinsol::Settings::from_policy`: each variable and residual scale is `feasibility * s / t` and the scaled-step tolerance is `feasibility`, so KINSOL's residual and step tests reduce to the original per-row and per-coordinate budgets; sign constraints from one-sided bounds on shifted coordinates and from guard signs ([§18.10](#section-18-10)). Callers choose only the method (`kinsol::Method`) |
| HiGHS | Feasibility, integrality and MIP gap controls |
| Clarabel | Primal/dual residual and gap controls, with cone-block adjustments retained in provenance |

Ipopt separately enforces the three component thresholds; its overall scaled
`tol` uses their maximum so it does not impose the tightest component on unrelated
errors. POUNCE retains its conservative minimum because its scaled dual-error floor
uses the overall tolerance. Acceptable-stop components remain separately declared,
and original qualification applies after either native result.

Options that encode these semantic controls are reserved; a caller-supplied native
option that conflicts with them is refused before the library sees it.

`kinsol::Settings::from_policy` is the one KINSOL configuration. The root runner,
conditional initialization blocks ([§17.1](#section-17-1)) and the nested implicit solve
inside an evaluation all derive from it; the nested solve takes the budgets and nominals of
its unknowns and rows from the resolved policy's targets, and its normalized feasibility
budget is the smallest tolerance-to-nominal ratio of its unknowns; nested solves on one
worker reuse a cached KINSOL session per problem layout ([§18.10](#section-18-10)). The
derived controls reach every adapter through the shared representation runners
([§18.7](#section-18-7)). One NLP runner, `execution::nlp`, serves staged-sequence steps
(single solves, authored sequences, studies and initialization blocks), fitting and the
factorable runner's fixed-assignment re-solve ([§18.10.1](#section-18-10-1)): presolve
pipeline (or its terminal report), native solve, library recovery with an independent
original observation, KKT evidence and qualification. No workflow derives
native controls or acceptance of its own.

Acceptance is judged in original physical coordinates, not in the solver's normalized
space. `quality::Tolerances::from_policy` projects the frozen per-ID budgets onto one
oracle's order. Integration accuracy, integrated observables, instantaneous balances
and cumulative closure are distinct requirements.

The workflow completion owner (`pse-runtime/src/workflow/numerics.rs`) is the single
owner of candidate use. It retains native termination, original validation and qualification, required model checks, closure, applicability and coverage independently. Final permission and every applicable qualifier/refusal are typed; the reason text is a derived presentation and is never parsed ([ADR-0145](../../adr/0145-declared-execution-and-composed-qualification.md), proposed; authorized implementation).

Default incumbent `Refuse` keeps independently validated original-feasible limited candidates seed-only where lawful. `AcceptFeasible` may grant result permission at typed time, node, iteration or solution limits. `AcceptWithinGap` additionally requires a valid bound in the original objective convention meeting the authored absolute or valid same-sign relative criterion. A relaxed export may supply a qualified bound but its incumbent remains diagnostic. Missing validation, failure, cancellation, panic, least-infeasible and relaxed-only points cannot be upgraded.

Closure defaults to `RequireClosed`. `AllowUnclosed` qualifies completed outside-budget closure while retaining its residuals; missing required closure remains unavailable and refuses. Applicability opt-ins retain their individual qualifiers alongside incumbent and closure qualifiers. Missing checks or required observation coverage refuse permission. All-fixed point evaluation grants permission only through complete original checks. A trajectory qualifies only when its admitted actual endpoint and required prefix obligations are satisfied (§13.5).

`CandidateUse` remains the permission projection (`usable`, `qualified_unclosed`, `seed_only`, `diagnostic_only`, `unusable`); `runtime.candidate_assessments` additionally records independent evidence, incumbent policy, bound/gaps, qualifiers, refusals and result/seed permissions. Consumers retain the composed decision and never reconstruct it from a success bit or enum spelling.

Every workflow consumes this decision; none re-derives acceptance:

- a result (a commit, an advance, a publication) needs `usable` or `qualified_unclosed`.
  This covers staged-sequence steps, block-initialization commits (which add the block's
  own coordinate and finiteness checks), authored initialization stages and homotopy
  advances, fitting estimates, the feasibility witness of a nonlinear explanation and
  `RunResult::usable`;
- a study's dependent point may be seeded from a predecessor that is a result or
  `seed_only`; any other predecessor refuses the dependent point with a `conflict`
  diagnostic;
- `StartPolicy::PreviousAccepted` ([§17.6](#section-17-6)) seeds only from results; a
  `seed_only` candidate never becomes a sequence warm start or a committed block value.

Fitting keeps the native decision when it forbids use and otherwise decides from the
fresh original-model quality of the final evaluation. `runtime.candidate_assessments`
publishes each step's native termination, numerical feasibility, closure, policy, use
and reason as distinct columns.

## 17. Initialization, starts and recycles

> Decision: [ADR-0083](../../adr/0083-class-specific-native-execution.md) — native
> heuristic/exact tear selection and sequential initialization;
> [ADR-0093](../../adr/0093-qualified-native-strategies.md) — transactional stages and
> explicit starts.

Initialization produces a start; it never claims an optimum and never rewrites the
case. Strategies are derived from distinct structural projections and authored
policy; libraries own every iteration.

### 17.1 Structural initialization model

> Decision: [ADR-0114](../../adr/0114-typed-operational-store.md) —
> restates D13: initialization stages, including discrete fixing, are immutable overlays;
> [ADR-0093](../../adr/0093-qualified-native-strategies.md) — transactional stages. Plan 22
> A6 (implemented) runs every block as a solve step of one staged sequence on a retained
> native session; [ADR-0116](../../adr/0116-typed-boundary-documents.md) — initialization
> admission in Rust and typed attempts across the Python boundary (Plan 22 A5,
> implemented).

`pse-structural::initialization::Plan` converts a complete, structurally sound square
analysis into predecessor-first conditional blocks, each with its original rows, solved
columns and explicit predecessor inputs. Deficient, partial or non-square structure is
refused before any factorization. `pse-runtime::math::initialization::PreparedInitialization`
compiles each block as an ordinary library artifact (unselected variables bound as
fixed) and resolves every block's route before worker acquisition
([§18.7](#section-18-7)); a failed attempt does not silently change its selected route; declared numerical recovery follows §18.6. Typed backend
settings must belong to every block's route. Each block runs through the shared runner of
its route's representation, the same runners a solve uses: `execution::roots` for KINSOL,
with scales derived per block by `kinsol::Settings::from_policy`
([§16.6](#section-16-6)), or the one NLP runner `execution::nlp` for Ipopt and POUNCE, with
library presolve off.

**Admission.** `PreparedInitialization::validate_profile` is the one admission of an
initialization request, in Rust, and every caller reaches it before native work. The intent
must be `root` or `initialize`. The request carries no explicit presolve policy or numerical
convexity strategy, since blocks run without either, and it may require neither an explicit
start nor native reuse. Typed backend settings must belong to every block's route, checked
by that route's `BackendExecution::admit_settings`. The schedule is serial and finite: one
thread, at least one stage and at most 4096 stage-block steps. A stage overlay may replace
only declared, finite fixed or parameter inputs, never a solved column, and every block
boundary value must be present. Admission returns every block's route and the resolved
numerical policy. Each block attempt keeps its stage, route, boundary, result or typed
failure, its shared numerical trace, and whether it committed; the Python boundary exposes the routes and the ordered
attempts as typed values (`NativeStrategyAttempt`, exactly one of a native report and a
typed failure; [§18.7](#section-18-7)). *Tested* by `initialization_admission_in_rust`
(runtime units) and `test_route_and_eligibility_are_typed` (Python unit test).

Execution is transactional over immutable case bindings:

- a block commits only coordinates that passed independent original-quality checks;
  a failed block commits nothing;
- `InitializationReport` keeps the immutable original specification, the committed
  solved unknowns only (never fixed inputs or overlays), every stage overlay and stage
  candidate (including failed or cancelled stages), every block attempt with its route,
  and whether the final stage used the original bindings;
- a failed or cancelled stage retains its overlay as evidence and cannot return it as
  the authoritative specification.

Initialization accuracy comes from the numerical policy, and the schedule is finite and
bounded. `MathService::initialize` admits the whole schedule once and stamps one absolute
task deadline before queueing. Binding, rebinding, derivative/support preparation and all
block attempts consume that scope; neither a preparation wait nor a later block renews it.
The schedule runs as one staged sequence on one native session ([§18.8](#section-18-8)): each stage composes its
overlay over the immutable original values, and each block is an ordinary solve step
(`MathService::prepare_conditional`, run by the shared step executor
`MathService::execute`) bound to those values. Each block's view is bound once
(`PreparedBlock::bind`) and rebound to later stages' values
([§14.4](mathematics-and-compilation.md#section-14-4)). Blocks run in predecessor order on
the session's thread, and a block's solved coordinates become its successors' inputs only
after its candidate is a result. Retained native state serves a later step with the same
backend, coordinates and profile (the same block in a later stage) when that step's
`ReusePolicy` allows it; the default, `Fresh`, rebuilds. A step whose candidate is not a
result drops the retained state.

A case resolved directly for the initialize intent refuses a free discrete variable
(`modeling.domain`, [§6.8](schema-and-relations.md#section-6-8)). Authored staged initialization
implements ADR-0103’s `FixAtStart` policy: it validates integral discrete starts, fixes them
within the stage overlay, and restores the original free discrete specification for final
assessment and on every exit. The public entry is the package-bound block initialization strategy
(`workflow/strategies/conditional.rs`). Authored stage and homotopy initialization
(`ModelingPackage::initialize_model`, `workflow/modeling/engines.rs`) runs on the same
staged-sequence primitive and retains original-specification acceptance
([§17.5](#section-17-5)).

### 17.4 Flowsheet recycles and dynamic starts

> Proposed amendment: [ADR-0155](../../adr/0155-numerical-derived-families.md), Plan 25n N1/N4.

Control-sensitive schedules use execution dependencies without fictitious derivative incidence.
The accepted final unit sweep retains all local unknowns, including non-port state, for complete
original assembly/assessment. A connection-map root report alone remains a scoped map product.

> Decision: ADR-0142 (proposed; maintainer-authorized implementation).

A scheduled unit is admitted as an explicit map or a conditional equation problem with
supplied boundary inputs, owned residuals, local unknowns and required outputs. Admission
checks dependency locality, mode-qualified structure and selected solver capability before
iteration; topology only supplies tear candidates. A conditional solve fixes boundary inputs
under scoped ownership and returns independently qualified outputs or typed refusal.
A simultaneously solved initialization is an explicit alternative, never a failed-unit fallback.
`CausalUnitRequest.realization` selects `ExplicitMap` or a `Conditional` problem carrying
owned residual/unknown identities and one `SolveSettings` profile. Aggregate material ports
resolve to their authoritative independent coordinate ports. A derived input fixes its
original member observation through a typed boundary residual and solves its constituent
unknowns; fixing a fabricated value symbol would not constrain that state. The compiled
residual is the admitted difference of two observations: temperature inputs and outputs
remain points while their residual is an interval. Resolved numerical magnitudes and
provenance project through that subtraction without affine offsets or renewed defaults;
characteristic normalization magnitudes scale with the target representation. Admission retains
the complete structural witness and selected derivative demand. Native constraint handlers
whose locality cannot be proved, undeclared parent coupling and bounds outside the root
adapter capability refuse before iteration. The affected workflow diagnostic retains the
unit/source identities and original mathematical cause. Each evaluation uses an immutable
original-case overlay, so temporary inputs and starts cannot escape into another call.

`pse-structural::flowsheet::FlowGraph` is the admitted physical flow projection:
explicit connection occurrences (parallel occurrences are never merged), node isolates,
declared decision groups with finite nonnegative costs and `Free`/`Mandatory`/`Forbidden`
policy. Tear selection (`pse-backend-native::tears`) has two explicit routes:

- **Exact.** A HiGHS MILP with order variables, `r_u + 1 <= r_v + |V| t_group` per
  occurrence; model size is linear and cycles are not enumerated. MIP incumbent, bound,
  gap and optimality remain distinct observations.
- **Heuristic.** An explicit unweighted petgraph greedy feedback-arc route. Its computed
  authored cost is not an optimality claim.

Both results are independently checked: the remaining graph must be acyclic under
petgraph, forbidden groups are respected, and a cycle that the allowed policy cannot
break is reported concretely. A selected tear set feeds `RecycleRequest`, which names
each unit's causal direction port by port. `CausalMap` evaluates declared causal units
over the witnessed order with physical port conversions; KINSOL owns the fixed-point
iteration and Anderson acceleration. Input bounds cannot be enforced by fixed-point
iteration, so bounded causal inputs are refused with a request for a constrained
simultaneous strategy. No arbitrary residual is reinterpreted as a causal map.

Consistent initial conditions for dynamics belong to the integrators
([§13](workflows-and-results.md#section-13)). The starts they consume carry typed
provenance. Case resolution records a `StartSource` for every resolved input
(`workflow/modeling/cases.rs`): `ModelDefault`, `Case` (with its authored path),
`Annotation` (with the start annotation's declaration), `Predecessor` (the solved values
of an earlier staged step, [§17.6](#section-17-6)), `Continuation` (a continuation
override) or `Stored`, a seed read from the operational store's solutions by
coordinate-compatibility stamp and preparation identity, or named explicitly
(`with_stored_start`; a study point's predecessor, a resumed job's incumbent,
[§20.6](identity-and-publication.md#section-20-6)). Workflows branch on this type, never on
a label. An integrated state without an isolated initial equation
takes its lower-endpoint start from its start annotation, evaluated through the model at the
endpoint, when its source is `Annotation`; every other source supplies the resolved
constant. An annotation source whose model output is missing is refused, not frozen.

### 17.5 Continuation

> Decision: [ADR-0093](../../adr/0093-qualified-native-strategies.md) — transactional stages
> and explicit starts; [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> candidate use decides every advance. Plan 22 A6 (implemented) runs stages and homotopy as
> steps of one staged sequence.

Finite supplied continuation replaces declared fixed/parameter inputs stage by stage and
executes the structural block sequence transactionally ([§17.1](#section-17-1)).

Authored initialization (`ModelingPackage::initialize_model`) runs the named stages, then
bounded adaptive homotopy over the declared continuation endpoints, then the unchanged
original specification, as the steps of one staged sequence (`workflow::staged::Staged`).
Every attempt composes an overlay over the immutable original specification
([§19.1](workflows-and-results.md#section-19-1)): a stage attempt selects `stage.<name>`,
and a homotopy attempt replaces the continuation parameters by their values at the current
fraction of the path. Homotopy steps therefore change values only. They share one prepared
view ([§14.4](mathematics-and-compilation.md#section-14-4)) and, when the step's reuse
policy allows, the retained native session (`homotopy_steps_reuse_session`). The policy
bounds the initial and minimum step, the growth after an accepted step, the attempts and the
wall time, which covers binding, native execution and assessment of every step.

An attempt after an accepted one starts from the solved values of the last accepted attempt
(`Start::Accepted`, [§17.6](#section-17-6)); before any acceptance an attempt starts from the
specification. A failed attempt keeps its overlay as evidence and seeds nothing. A failed
stage, a failed first homotopy point or a non-retryable failure stops initialization with its
cause; a retryable homotopy failure halves the step, down to the minimum step. Stage and
homotopy attempts answer to the model's checks, not to final-fixture expectations; only a
separate solve of the unchanged original specification, answering to every obligation, may
commit. This is local initialization evidence, not a convergence guarantee.

### 17.6 Explicit starts and allocation reuse

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> completion records the submitted start. Plan 22 A4 (implemented) separates the
> coordinate and profile stamps and types the seed transformations; Plan 22 A6
> (implemented) types where a staged step starts (`Start`) and runs every step through one
> executor.

Numerical start policy (`StartPolicy`: `NoPriorStart` by default, `PreviousAccepted`,
`Explicit`) is independent of native allocation reuse (`ReusePolicy`: `Fresh`,
`AllowRebuild`, `RequireReuse`). Reusing a retained native model never implies a warm
start, and a disallowed start clears retained native start state, including HiGHS basis
state.

Each prepared native step carries a `Compatibility` stamp in four parts:

| Part | Frames | Governs |
|---|---|---|
| `layout` | Seed coordinates: backend, objective sense, free variables with their domains and physical maps, rows, body keys and sparsity patterns (`pse.solver.coordinates.v1`) | Whether a seed fits |
| `profile` | The native session: presolve policy, numerical policy, convexity policy, intent, the session-relevant controls and typed backend settings (`pse.solver.session.v1`) | Whether retained native state fits |
| `data` | Parameters, fixed values, coefficients, bounds and provider configurations of this attempt | Nothing; it may differ |
| `backend` | The native backend | Both |

A seed needs only `layout` and `backend` to match (`WarmStart::validate`), so a step that
changes a native option or setting keeps its predecessor's seed. Retained native state
needs `layout`, `profile` and `backend` to match (`Compatibility::same_session`); a changed
profile rebuilds the session, or `RequireReuse` refuses. The profile leaves out the
per-attempt budgets (time and iteration limits, history) and the sequencing policies
(reuse and start), because every attempt re-applies them. An explicit cone request
normalizes its coordinates at preparation, so its layout also frames the resolved policy.

A `WarmStart` is an owned, typed payload (root primal; NLP primal with optional bound
and row multipliers; HiGHS primal/dual/basis; POUNCE SQP iterate and working set) with its
compatibility stamp and an optional origin run/attempt. `StartReceipt` records what was
actually submitted, distinct from the output seed a report offers for later use: the
predecessor attempt, the seed, an explicit partial MIP seed, whether the native API
received it and the typed `SeedTransformation` path the seed took. That path is
`normalization` (by its identity), followed by `presolve` (its transformation identity and
the passes the library applied) when a pass was applied. It is recorded from what ran,
only when a seed or partial seed was supplied, never as a constant label. Its published
form is owned by [§20.3](identity-and-publication.md#section-20-3). Mutable native state
never crosses a worker boundary. Fitting starts are refused; declared parameter guesses
are its input.

**Starts in staged sequences.** Where an initialization or homotopy step starts is
`workflow::staged::Start`: `Specification`, the specification's own values and start
annotations; or `Accepted(k)`, the solved values of an earlier step whose composed
candidate decision permits use. A missing or unpermitted predecessor refuses the step.
Seeded physical values enter case resolution as `StartSource::Predecessor` for every
variable the case leaves free ([§17.4](#section-17-4)).

The numerical strategy admits an entry start separately from recovery proposals and
native allocation reuse. `NoPriorStart` excludes inherited proposals; `Explicit` consumes
the declared seed without silently substituting a predecessor; `PreviousAccepted` consumes
only a predecessor whose original completion permits use. Prediction, auxiliary and
surrogate proposals retain their own provenance, physical coordinate binding and branch
policy, and must pass original-model screening before use. Screening a start grants no
result permission ([§18.6](#section-18-6)).

Studies use the study policy's explicit seed edge and operation-owned seed role
([§19.3](workflows-and-results.md#section-19-3)). A declared-case continuation submits the
compatible predecessor's complete typed `WarmStart`, including any native dual or basis
payload, through the prepared operation's start admission. It does not merely copy values
or use a staged `Seed(k)` variant. Availability, scientific permission and allowed fallback
remain decisions of the study policy. Compatibility and transformation receipts describe
what the native step actually received; mutable foreign state stays on its owning worker.

Initialization distinguishes a start from a seed. A block starts from its staged values,
and that initial point is not a warm start. Only under `PreviousAccepted`, from the second
stage on, do the block's committed values from an earlier stage make its start a seed, with
that attempt as its origin and its transformation path recorded. Otherwise the receipt
records no seed and nothing submitted.

## 18. Native class-specific execution

> Decision: [ADR-0083](../../adr/0083-class-specific-native-execution.md) — native
> class-specific execution; [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) —
> shared execution vocabulary; [ADR-0093](../../adr/0093-qualified-native-strategies.md)
> — contextual eligibility and qualification.

Each mathematical class has its own representation and native adapter under one
runtime lifecycle. There is no universal problem object, no solver-neutral file format
and no hidden fallback engine. Declared bounded composition follows ADR-0154/0156, with library-owned fitting iteration and original permission. [ADR-0083](../../adr/0083-class-specific-native-execution.md)
owns the rationale; [ADR-0082](../../adr/0082-library-owned-process-mathematics.md) owns
library ownership of the mathematics.

### 18.1 Problem representations

> Decision: [ADR-0103](../../adr/0103-variable-domain-facet.md) — authored variable
> domains reach `CoefficientProblem` and routing (Plan 22 M1, implemented);
> [ADR-0104](../../adr/0104-discrete-constraint-forms-and-realizations.md) — lowered
> constraint forms and native-form metadata (Plan 22 M3 and M4, implemented);
> [ADR-0105](../../adr/0105-scip-factorable-backend.md) — the factorable representation
> (Plan 22 G2 and G3, implemented).

The compiler's immutable case products are projected into class-specific views
(`pse-backend-native` crate root, `assembled`):

| Representation | Meaning | Consumed by |
|---|---|---|
| `OracleContract` | Content identity, source variables with bounds in native column order, row IDs, available derivative order and admitted smoothness (derivatives and smoothness are separate claims) | All callback adapters |
| `NlpOracle` | Objective, constraints, gradient, Jacobian and weighted Lagrangian Hessian as separate demands, with sparse patterns, normalization, presolve facts and structural analysis | Ipopt, POUNCE |
| `NleOracle` | Square residuals with an assembled Jacobian or Jacobian-vector product | KINSOL |
| KINSOL `Function` | Residual equations, explicit Picard splitting `F(x) = Lx - N(x)` with a declared constant `L`, or a declared causal map with an original residual validator | KINSOL |
| `CoefficientProblem` | Sparse LP/MILP/QP coefficients with objective sense and constant, the declared registry variable domains (`ModelingVariableDomain`, including `semicontinuous` and `semiinteger`) and consumed parameter assumptions | HiGHS |
| `ConicProblem` | Cone data in pse-owned sparse-matrix and cone types (`conic::SparseMatrix`, `conic::Cone`): declared by an explicit cone request, or lowered from a `CoefficientProblem` when a linear or convex quadratic program is routed to Clarabel (`ConicProblem::from_coefficients`, [§18.10](#section-18-10)) | Clarabel |
| `FactorableProgram` | Library-neutral factorable DAG over original case columns with per-row fidelity, projected under the step's values only for a factorable route ([§7.5](mathematics-and-compilation.md#section-7-5)) | SCIP |

Coefficient views are derived only through admitted affine/degree-two proofs, and cones
are declared by an explicit request; neither is inferred from NLP rows. All-fixed
models are evaluated directly as a constant route, except that the `certify` intent and a
native constraint form are refused there ([§18.7](#section-18-7)). Dynamic representations are owned
by [§13](workflows-and-results.md#section-13).

The compiler carries each free variable's declared domain
([§6.8](schema-and-relations.md#section-6-8)) into `CaseStructure`, `ProblemFacts`, the
coefficient view and the factorable program; nothing between authoring and a backend
converts it. A free discrete column removes `smooth_nlp` and `square_root`; it makes a
coefficient-eligible model `mixed_linear`, or `mixed_integer_quadratic` with a quadratic
objective, and any other model `mixed_integer_nonlinear` ([§18.7](#section-18-7)). An
authored MILP therefore routes to HiGHS under an optimization intent, while MIQP and MINLP
route to SCIP ([§18.10.1](#section-18-10-1)), whose export lowers semicontinuous and
semiinteger columns by `semi(indicator)`. Integrality is never relaxed to reach a route.

Constraint forms and disjunctions ([§19.7](workflows-and-results.md#section-19-7)) reach
every representation as the ordinary rows and variables of their lowering. A form realized
for a native handler also travels as `CaseStructure` metadata
([§7.5](mathematics-and-compilation.md#section-7-5)) that only an adapter consuming the
handler may execute ([§18.7](#section-18-7)).

### 18.2 Evaluation programs and callback boundary

Value and derivative programs are Symbolica/Numerica artifacts prepared by the
compiler ([§7](mathematics-and-compilation.md#section-7)); scientific functions and generic external capabilities are supplied through the
authored modeling contract ([§9](physical-semantics.md#section-9)). Each attempt owns cloned
evaluators, provider workers and scratch; shared programs stay immutable.

`pse-backend-native::callback::CallbackState` is the single trial-evaluation boundary
for all callback adapters:

- work runs under `catch_unwind`; a Rust panic becomes a `Panic` termination and never
  unwinds across a C or library callback;
- typed causes (never diagnostic strings) classify failures: domain violations and
  provider trial, envelope, singular and regime-crossing failures are recoverable trials
  (`ProviderError::recoverable`); cancellation is cancellation; everything else is fatal
  evaluation failure;
- outputs are published only after a successful evaluation, so a failed trial cannot
  leave partial buffers or return the previous trial's values;
- only terminal failures latch; later successful trials preserve native success;
- per-demand call counts and wall time, including rejected trials, become metrics.

**Regime crossings.** A nested regime selection binds its derivatives to the regime chosen
at the first derivative request. A later trial that selects another regime is refused as
the typed, recoverable `ProviderError::RegimeCrossing`, naming the selector, the bound
regime and the selected one, so the outer method shortens or changes its step instead of
continuing across the switch; runtime diagnostics classify it as a rejected trial and keep
the three identities as its sources. The callback boundary counts these refusals. A solve
report's `evidence.callback.regime_crossings` and its metric `callback.regime_crossings`
hold the attempt's total, a subset of its trial rejections, and each Ipopt iteration event
carries `regime.crossings` for that outer iteration, bounded by the progress history like
every event. *Tested* by `nested_stage_reports_regime_crossings_per_outer_iteration`
(native runtime units: Ipopt steps from the bound regime towards the other, with no
crossing in iteration 0 and crossings from iteration 1, and never accepts).

### 18.3 In-process NLP: Ipopt C and POUNCE

> Decision: [ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md) — Ipopt
> linear solvers MUMPS+METIS, SPRAL SSIDS and oneMKL Pardiso, typed and explicitly
> selected over one BLAS/LAPACK provider, with HSL excluded (Plan 22 N1, implemented);
> [ADR-0109](../../adr/0109-pounce-l1-and-convex-methods.md) — POUNCE's ℓ1 options are
> reserved (Plan 22 A3), and the typed `L1ExactPenalty` method is the explicit ℓ1 route
> (Plan 22 N3, implemented). Plan 22 N2 (implemented) adds typed warm restarts and carries
> the SQP working set through presolve.

`pse-ipopt-sys` holds generated, committed bindgen output for the Ipopt 3.14.20 C
interface (`just codegen --only bindgen`; see [ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md)
for the digest-pinned solver image). ABI tests assert 32-bit indices, `f64` numbers,
the version string and real symbol relocations. `pse-backend-native::ipopt` is the safe
driver: RAII problem ownership on the owning worker, direct callbacks over `NlpOracle`,
exact or limited-memory Hessians, primal/dual starts, reserved options and an
intermediate callback that checks cancellation and deadline.

**Ipopt linear solvers.** The adapter's pse-owned settings (`settings::ipopt::Settings`)
select one linked symmetric-indefinite factorization, each kind with its own typed
parameters and spelled as Ipopt's `linear_solver`: `mumps`, sequential MUMPS 5.9 with a
stated ordering (AMD, AMF, PORD, METIS or QAMD; METIS by default, and MUMPS's automatic
choice is not representable); `spral`, SPRAL SSIDS on OpenMP threads, with its ordering,
scaling and pivot method; or `pardisomkl`, oneMKL Pardiso on MKL threads, with its ordering
and weighted matching. The same settings state the barrier update (`mu_strategy`, monotone
by default), the cold-start pushes `bound_push` and `bound_frac`, and the warm restart
below, so no library default decides them. HSL solvers and the runtime-loaded Pardiso are
not representable, and their loaders `hsllib` and `pardisolib` stay reserved. Admission
(`ipopt::admit`) runs when the route's settings are admitted and again on the owning worker
before any native construction. It reads the linked library and the process state
(`ipopt::Runtime::observe`) and refuses with a typed `Unsupported`, never substituting
another solver: a solver the linked Ipopt was not built with; more than one thread for
MUMPS; SPRAL without `OMP_CANCELLATION=TRUE` or with `OMP_PROC_BIND` false; and oneMKL
Pardiso outside the image's pinned `MKL_CBWR` branch (`COMPATIBLE`) or with `MKL_DYNAMIC`
on ([§18.8](#section-18-8)). The solver image links one BLAS/LAPACK provider, oneMKL (LP64,
GNU threading), and one OpenMP runtime, libgomp; the BLAS calls of every native component
resolve into it. The linked build (Ipopt and oneMKL versions, the linked solvers, the CBWR
branch in force and the image manifest, `pse.native.ipopt.build.v1`) enters every request
profile identity ([§16.5](#section-16-5)), and each report's provenance names the Ipopt and
oneMKL versions, the typed linear-solver settings and the CBWR branch in force, published in
`runtime.solve_metrics` (namespace `native_provenance`) beside `linear.threads`. Under the
pinned branch only numerical reproducibility is claimed, never bitwise equality between
solvers. *Tested* by `ipopt_mumps_metis_ordering_selectable`,
`spral_refused_without_omp_cancellation`, `ipopt_pardisomkl_selectable_under_cbwr` and
`single_blas_provider_in_process` (native backend units in the solver image, merge
`7c216968`).

POUNCE (`pounce`, `tnlp`) shares the same `NlpOracle`, callback policy and presolve
pipeline, and adds interior-point and active-set SQP methods, FERAL linear algebra with
admission-bounded threads ([§18.8](#section-18-8)), restoration statistics and working-set
starts. Its method is the typed `pounce::Method` (registry `PounceMethod`): `interior_point`
by default, `active_set_sqp`, or `l1_exact_penalty` (below). Continuous NLP for both
adapters passes through pounce-presolve's qualified wrappers (`presolve::pipeline`): policy
`Off`, `Auto` (qualified source-backed passes only) or `Explicit` with required passes that
must qualify. The library owns reductions, derivative transport and recovery; the project
records effects and inverse source attribution, not a second transformation IR. Auxiliary
reduction is never automatic.

The pinned binding cannot recover general original-bound multipliers from `LinearBounds`
or `Fbbt` tightening. Both passes are therefore ineligible for executable NLP transformation:
`Auto` records the reason and required explicit passes refuse before dispatch. Native
execution retains the authored bounds and required rows; affine elimination retains its
separate library-owned source-bound recovery. Bound warm-start hints from untracked
tightening are disabled. A separate library-owned interval analysis can establish source
infeasibility only after confirmation against the acceptance-padded original bounds and
rows, under the same execution scope. Its geometry, row removals and dual hints are
discarded before native construction. *Tested* by `presolve::tests::bounds` and
`presolve_certificate_respects_each_bound_budget` (linked native backend units, explicit
force-validation, 2026-10-04). Transformation identity uses the bounds and recovery maps
actually consumed by execution.

**Warm restarts.** A complete primal-dual seed restarts under a typed profile
(`solve::WarmRestart`, part of both adapters' settings), because with the cold-start pushes
a seeded interior point is pushed back towards the analytic centre and loses most of its
benefit. The profile states the initial barrier parameter (`mu_init`): the final barrier
value the producing interior-point solve recorded with the seed, or a stated positive
value. It is set only under the monotone barrier update, the only one that reads it; a
seed without a barrier value, such as an authored one, leaves the native default, and the
receipt says so. The profile also states the primal, slack and multiplier pushes
(`warm_start_bound_push`, `warm_start_bound_frac`, `warm_start_slack_bound_push`,
`warm_start_slack_bound_frac`, `warm_start_mult_bound_push`), 10⁻⁹ by default so that the
restart stays near the seed. These options are reserved, and the restart that ran
(`AppliedRestart`) is a typed seed transformation in the start receipt
([§17.6](#section-17-6)). The barrier value travels in authored objective units and is
scaled like the objective natively; a seed's content identity frames it together with the
working set (`pse.native.seed.v3`).

POUNCE's active-set SQP method also accepts a working set with the primal-dual iterate. A
working set indexes native rows and bounds, so it passes through presolve only under the
transformation that produced it: the pipeline compares the seed's transformation identity
with the attempt's and records the transfer either way (`WorkingSetTransfer`). An
interior-point method records a working set as not submitted, and the ℓ1 method accepts
only the primal seed, since its problem has its own slack multipliers. *Tested* by
`interior_point_warm_profile_recorded`, `warm_restart_reduces_iterations_on_perturbed_case`
and `sqp_working_set_restart_reaches_runtime` (native backend units, merge `7c216968`).

**The ℓ1 route.** `l1_exact_penalty` is the Thierry–Biegler ℓ1 exact penalty-barrier method
(`pounce-l1penalty`). It is explicit only: never selected automatically and never a retry.
It relaxes every row, inequalities through bounded slacks, so a feasible model returns a
point the penalty makes exact and an infeasible one stops as locally infeasible at a
least-infeasible point. That point carries a `LeastInfeasible` label naming the original
rows it leaves violated beyond their budgets, and candidate use makes it `diagnostic_only`
([§16.6](#section-16-6)). Every presolve pass assumes that the rows hold, so under this
method `Auto` presolve resolves to `Off` and the presolve report records the resolution
(`presolve::Resolution::RelaxedRows`), while an explicit policy with passes is refused.
The bounded nonlinear infeasibility explanation runs on this route
([§15.5](#section-15-5)). *Tested* by `l1_route_returns_labelled_least_infeasible_point` and
`l1_auto_presolve_resolves_off_and_is_recorded` (native backend units, merge `7c216968`).

**Option hygiene.** Both adapters reserve the options that encode the derived controls
([§16.6](#section-16-6)), derivative constancy, iteration and time limits, bound
infinities, native scaling, the linear solver and its parameters, the barrier update and
pushes, and the warm-restart options; a caller-supplied value for one of them is refused.
A retained session never carries an earlier step's option into a later one, and the two
libraries reach that differently. POUNCE can clear its option table, so a reused
application starts from an empty table. Ipopt retains the actual C++ application and
TNLP and uses `ReOptimizeTNLP` only under the complete immutable effective-option,
coordinate, objective-sense, profile, sparsity and bound signature. Changed structural
options rebuild that state, or under `RequireReuse` return `ReuseRefusal::Structure`; an
option subset cannot establish same-structure reuse. Seed barrier and restart pushes are
per-attempt data: the adapter clears them before applying the current seed, including
warm-to-cold reuse. They cannot force a structural rebuild or survive into an unseeded
attempt. Ipopt owns adjustment of a finite supplied start into its native bounds, including
bounds transferred by qualified affine elimination; final qualification still checks the
original problem. The borrowed numeric oracle is rebound for each solve and remains
valid until native execution and cleanup finish. The adapter, rather than a caller option,
owns same-structure admission. *Tested* by
`persistent_tnlp_reuses_real_application_with_fresh_borrowed_numeric_oracle` and
`persistent_profile_change_refuses_required_reuse_and_discards_native_state`
(native backend units, explicit force-validation). POUNCE's hidden second
solves, `mu_strategy_fallback` and
`dual_divergence_retry`, are pinned off, as is its automatic ℓ1 retry after restoration
failure (`l1_fallback_on_restoration_failure`); the exact-penalty switch
(`l1_exact_penalty_barrier`) is set only by the typed ℓ1 method. A result therefore never
comes from an undeclared attempt beyond the admitted iteration budget. After each POUNCE
solve the adapter reads the complete effective option table back from the application:
every registered option at its current value, plus explicitly set prefixed options, with
the registered defaults beside it. That snapshot, not the adapter's request, is the report's effective options.

**Scaling and metric names.** Model coordinates are normalized before either adapter runs
([§16.1](#section-16-1)); no user scaling reaches the libraries, and the former native
`Scaling` path is removed. `nlp_scaling_method` is `gradient-based` when the policy permits
native algorithmic scaling and `none` otherwise. Ipopt progress metrics name their
coordinates: `objective.normalized`, `stationarity.normalized` and
`iterate.normalized.infinity_norm` are values of the normalized model (Ipopt's unscaled
readbacks undo only its own scaling), while `primal.native` and `dual.native` are the
infeasibilities exactly as Ipopt reports them. None of them is a physical value.

Original complementarity remains mandatory independently of native success or presolve
admission; a failing recovered multiplier grants feasibility at most. HiGHS's QP
regularization is derived from the requested gap
([§18.10](#section-18-10)).

### 18.6 Truthful outcomes

> Proposed amendment: [ADR-0154](../../adr/0154-declared-numerical-strategy.md), Plan 25n N2/N3.

Original assessment transports its scientific conclusion, typed cause, composed permission,
independent artifact retention and disjoint work alongside native termination. Actual produced
evidence enters the same state used by the next operation and trace. Original correction cannot
recursively select the auxiliary family being corrected or manufacture an unavailable guarantee.

> Decision: [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) — typed
> adapter evidence (Plan 22 A1, implemented), which qualification reads by evidence type,
> never by backend (Plan 22 A2, implemented); new assurances, each with stated conditions:
> `global_bound` and `proven_infeasible` (Plan 22 G3, implemented for SCIP),
> `exact_certificate` (Plan 22 G7, implemented for SCIP's exact rational MILP) and
> `sos_bound_nonrigorous` (Plan 22 N5; declared in the registry vocabulary and produced by
> no adapter yet).
>
> Decision: [ADR-0121](../../adr/0121-convexity-compiler-facts.md) — one typed
> `InfeasibilityCertificate`, verified in original coordinates for Clarabel and HiGHS rays
> alike, replaces the string-keyed certificate metrics (Plan 22 C4, implemented).
>
> Decision: [ADR-0118](../../adr/0118-one-kkt-point-analysis.md) — the KKT-point analysis
> is typed evidence of the step (Plan 22 S0, implemented, [§15.5.1](#section-15-5-1)).

`pse-backend-native::solve::SolveReport` is one envelope per attempt, including attempts
without a usable candidate. Its facts are independent:

| Fact | Representation |
|---|---|
| Native termination | Raw code, native name and message, plus a shared `NativeTermination` category (success, acceptable, feasible-only, infeasible/unbounded variants, limit variants, resource exhaustion, inconclusive, cancelled, numerical, evaluation, panic, invalid) |
| Candidate kind | `FinalIterate`, `BestIterate`, `FeasiblePoint` or `ConstantEvaluation`: what the API actually supplied |
| Feasibility | Original-space `Quality` (§15.4) |
| Stationarity / optimum / gap | `Qualification`: `Unqualified` < `Feasible` < `Stationary`, `OptimalWithinTolerance`, `GapQualified` |
| Certificates | One typed `InfeasibilityCertificate`: a primal-infeasible (Farkas) or dual-infeasible (recession) ray at `full` or `reduced` native accuracy, recovered to original coordinates with its verification ([§18.10](#section-18-10)); never exposed as a primal solution |
| Local analysis | `Evidence::local`: the KKT point of an optimizing NLP candidate, or why it is unavailable ([§15.5.1](#section-15-5-1)); requested sensitivities and their validity beside it |
| Completeness | Unattempted sequence steps are counted; partial trajectories and dropped events are explicit |
| Evidence gaps | `EvidenceUnavailableReason` on metrics and observations; absent means unavailable, never zero |

`quality::qualify` grants only what original observations support. A validation error,
missing candidate or infeasible quality yields `Unqualified`. Feasibility yields
`Feasible`. Beyond that, the kind of typed evidence the adapter recorded selects the rule,
and every rule but the global and certificate ones needs a success or acceptable native
stop. No backend is named, so a new adapter qualifies through the evidence it produces:

- `GlobalEvidence`, recorded by a certifying adapter over a factorable export, is read first
  and follows its own rule ([§18.10.1](#section-18-10-1)): an original-feasible candidate is
  `Feasible`; only a read-back-equivalent export transfers anything global; an infeasible
  stop the backend concluded grants the assurance `proven_infeasible`, and a successful stop
  with a finite dual bound grants `global_bound`; `GapQualified` additionally needs an
  original-feasible candidate from a result source whose fresh original objective lies
  within the requested absolute gap of the bound, or within the relative gap when both share
  a sign. A conclusion reached in rational arithmetic over an exact export (the backend
  reports the solve exact and the dual bound's source is `ExactExport`) grants
  `exact_certificate` instead, the only rigorous assurance, and a gap closed that way
  qualifies as `OptimalWithinTolerance` rather than `GapQualified`;
- otherwise an infeasibility certificate decides alone: a ray is never a candidate, so the
  report stays `Unqualified`, and it carries the `certificate` assurance only when the ray is
  at full native accuracy and verified against the original data
  (`InfeasibilityCertificate::certified`); a reduced-accuracy or unverified ray, or one whose
  original data could not be evaluated, grants no assurance;
- otherwise `CoefficientEvidence` needs a verified upload-equivalence readback plus either a MIP gap
  within budget (`GapQualified`, or `OptimalWithinTolerance` at a zero gap) or LP/QP dual
  feasibility and primal-dual error within budget (`OptimalWithinTolerance`);
- otherwise `ConicEvidence` needs residuals and a gap within budget
  (`OptimalWithinTolerance`);
- otherwise `KktEvidence` with original stationarity and complementarity within budget
  grants `Stationary`; an acceptable stop additionally needs a declared acceptable budget.

A root-system adapter supplies no multipliers, so the root runner tops out at `Feasible`.
A postsolve or validator failure preserves the native outcome and clears assurance;
optional diagnostic failure never replaces the original solve.

**Evidence and metrics.** Adapters record typed `solve::Evidence` on the report:
callback trial history (`CallbackEvidence`: recoverable trial rejections and whether a
terminal failure latched), whether a start was submitted through the native API, whether
the attempt reused retained native state, the interior-point restart applied to a
primal-dual seed and whether an active-set working set reached the solver
([§18.3](#section-18-3)), original-coordinate KKT acceptance (`KktEvidence`, recorded by
`quality::record_kkt`), the KKT-point analysis of an optimizing NLP candidate
(`Evidence::local`, [§15.5.1](#section-15-5-1)) with any requested sensitivities, HiGHS
coefficient-model evidence
(`CoefficientEvidence`: upload equivalence, discreteness, objective, MIP gap and dual
bound, primal and dual solution status, dual infeasibility and primal-dual objective error),
Clarabel conic residuals (`ConicEvidence`) and a certifying adapter's `GlobalEvidence`
(export fidelity, box identity, objective sense, feasibility tolerance, requested gaps,
native primal and dual bounds, gap and nodes, readback, the sources of the dual bound and of
the candidate, the infeasibility conclusion and whether it was reached exactly).
Qualification (`quality::qualify`), evaluation retry (`callback::retryable_evaluation`: at
least one recoverable trial rejection and no latched terminal failure), start receipts (the
submitted flag) and lineage identity (both start flags, [§16.5](#section-16-5)) read only
this evidence. The
string-keyed `metrics` are observations for reporting and publication and are never an
input to a decision. A local infeasibility stop with a candidate also carries a
`LeastInfeasible` label naming the original rows it leaves violated beyond their budgets
([§18.3](#section-18-3)); it is diagnostic evidence, never a result.

The report also retains effective options and queried native defaults, provenance,
bounded events, complete native statistics where the library exposes them, the start
receipt and an output seed, and from a certifying adapter a `GlobalRecord`: the declared
box, the ranked solution pool, the infeasible subsystem, the exact rational objective and
whether a retained search tree was reused ([§18.10.1](#section-18-10-1)). Fit response
derivatives remain candidate data distinct from estimator qualification. The shared tags are registry-owned and projected into Rust,
Arrow and Python ([ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md)); candidate
use (§16.6) combines these facts with physical closure.

> Decision: [ADR-0154](../../adr/0154-declared-numerical-strategy.md) and
> [ADR-0156](../../adr/0156-library-owned-numerical-composition.md) (proposed;
> independently reviewed target, implementation owned by Plan 25m).

Declared numerical strategy composes preparation, prediction, native profiles and bounded
recovery through existing math/runtime owners. Pure planning consumes explicit problem,
capability and retained-product facts; execution and original assessment are injected effects.
One enclosing scope covers all work; mechanism/attempt caps do not renew its deadline.
Capability refusals differ from failed trajectories. Task exhaustion, cancellation,
required contract failure, infrastructure failure and panic are terminal. A lawful numerical
failure may take a declared applicable recovery, including another profile on the same
backend. A backend change requires named selected alternatives. Optional refusal may leave
the permitted base route available. Original result, auxiliary-use and artifact retention
permissions are separate; trace transport derives from the registry. Scientific statistical,
control, integrator and durable-effect policies retain their owners.

Prepared public operations expose the registry-derived numerical declaration and delegate
composition and validation to Rust. Direct and public run projections retain the same
actual strategy events with their run occurrence and step. Standalone cone and authored
recycle reports retain their submitted numerical run identity; conditional initialization
retains one schedule identity and each block trace with its attempt ordinal. Structural
tear selection retains its structural result without manufacturing numerical strategy events.
Typed nested causes preserve original source attribution, cancellation, deadline expiry,
resource refusal and domain trial meaning across provider/native wrappers. PETSc native
errors retain their original status and header-defined resource, contract, infrastructure
or numerical category before any scientific post-solve callback; a callback terminal cause
takes precedence over its native wrapper status. Work counters identify their charging owner;
unknown observations remain absent and inclusive native/probe counts are not added twice.
Path events carry actual localization and rank observations; a nondegeneracy conclusion
requires genuine compatible compiler Second support, separately from First preparation.

### 18.7 Capability, eligibility and selection

> Proposed amendment: [ADR-0154](../../adr/0154-declared-numerical-strategy.md),
> [ADR-0158](../../adr/0158-observed-pounce-execution.md), Plan 25n N5/N6/N8.

Current-version Auto will resolve one next preparation, execution or assessment decision from
conditional adapter capabilities, actual products and observations. Explicit constraints and
request-owned replacement-start permissions remain authoritative. Native second opinions and
partitioned/finite-difference Hessians require acting typed configurations. Library-owned Schur
receives semantic separators after actual classification and the same effective FERAL profile.

> Supplement: [ADR-0152](../../adr/0152-demand-driven-compilation-and-contextual-routing.md) (proposed; Plan 25l functional implementation complete).

Contextual candidate assessment composes shared class/intent/rank policy with adapter-owned settings, representation and structural requirements, consuming an explicit immutable build/runtime snapshot. Scientific or representation evidence still needed, supported mathematics with artifacts not yet prepared, final readiness and runtime absence remain distinct. Obtain relevant pending class evidence before falling through to a lower-preference class. Capability reconsideration is limited to newly established scientific/representation incompatibility. Numerical trajectory recovery follows the declared strategy above; task resource exhaustion, cancellation, required contract and infrastructure failures remain terminal. An explicit selection is never silently replaced.

> Decision: [ADR-0105](../../adr/0105-scip-factorable-backend.md),
> [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) — the
> backend-execution adapter table, the explicit `certify` intent and SCIP routing for MIQP
> and MINLP (Plan 22 A2 and G3, implemented);
> [ADR-0104](../../adr/0104-discrete-constraint-forms-and-realizations.md) — native
> realizations only on adapters with the handler (Plan 22 M3 and M4, implemented; SCIP
> consumes every handler since Plan 22 G7, implemented);
> [ADR-0116](../../adr/0116-typed-boundary-documents.md) — typed routes, eligibility rows
> and registry reason codes across the Python boundary (Plan 22 A5, implemented);
> [ADR-0111](../../adr/0111-multi-objective-optimization.md) — lexicographic and weighted
> multi-objective routes (Plan 22 C3, implemented: the authored members and levels and
> the native and staged lexicographic routes, [§6.8](schema-and-relations.md#section-6-8)).
>
> Decision: [ADR-0121](../../adr/0121-convexity-compiler-facts.md) — automatic ownership per
> class (`Capability.automatic_classes`, Plan 22 C4, implemented); convexity classes from
> the compiler fact, and a numerical PSD qualification only for the request whose explicit
> policy asks for it (`Requirements::numerical_psd`, Plan 22 C5, implemented).
>
> Decision: [ADR-0104](../../adr/0104-discrete-constraint-forms-and-realizations.md) §5 — an
> authored realization's structural requirement selects the method that honours it (the
> ℓ1 exact penalty of `penalty(l1)`, Plan 22 M5b, implemented); ADR-0118 — automatic
> routing of a sensitivity request prefers an adapter whose candidate the KKT-point
> analysis differentiates (Plan 22 N5, implemented).

Capability is five distinct facts:

| Fact | Owner |
|---|---|
| Requested | `SolverProfile`: the registry intent `NativeSolveIntent` (`optimize`, `root`, `feasible_point`, `initialize`, `certify`), `SolverSelection::{Auto, Explicit}`, controls and typed backend settings |
| Available | `BackendExecution::linked`: the adapter is linked by feature; `runtime.solver_capabilities` publishes one row per linked adapter (§18.9) |
| Admitted | Compiler facts (`pse-math::facts::ProblemFacts`) and structural admission (§15.2): domains, bound shapes, prepared derivative order, coefficient eligibility, convexity evidence, native constraint forms |
| Eligible | `routing::admit`: every applicable typed `Ineligible` reason per adapter for this model, profile and thread count, each with its registry reason code (`NativeIneligibility`) |
| Selected | `routing::Requirements::select` returns `Constant` or `Native(backend)` |

**The adapter seam.** `pse-backend-native::execution` owns one `BackendExecution` adapter
per registry `Backend`. The static table (`execution::adapter`) maps every `Backend` value
to its adapter through an exhaustive match, so a new registry value cannot compile without
one; there is no string lookup. Each adapter declares one representation (`Nlp`, `Roots`,
`Coefficients`, `Cone`, `Factorable` or `Trajectory`); the algebraic router assesses every
adapter except the trajectory ones. Each adapter owns its pse-owned settings type, one
`Capability` record (classes, the subset of them it serves automatically
(`automatic_classes`), derivative representation, warm-start support, general bounds,
one-sided bounds as shifted sign constraints (`sign_bounds`), parallelism, whether it
certifies global bounds (`certifies`), the native constraint handlers it consumes
(`native_forms`), the structural requirements its settings' methods honour
(`requirements`), the classes it optimizes lexicographically in one native solve
(`lexicographic`), whether it solves a study's independent points as one batch (`batch`)
and whether its candidate carries the multipliers the KKT-point analysis differentiates
(`sensitivities`), reuse, cancellation and diagnostics), admission of its settings
and model contract, its native session on the owning worker, its warm-start payload and
its typed evidence. The capability record remains the source of generic eligibility and the published static
inventory row. Contextual assessment additionally composes adapter-owned settings,
representation and structural admission through explicit inputs; a second central table must
not independently copy those conditions. Settings identity
(`BackendSettings::identity`) is derived from serde, never from a hand-written field list,
and enters the request identity and the native profile stamp ([§17.6](#section-17-6)).
Native state retained between the finite steps of a sequence is an opaque, worker-owned
`execution::Retained`: an adapter reuses only its own session, when the coordinate and
profile stamps match (`Compatibility::same_session`) and `ReusePolicy` allows, and
otherwise tears it down before building a replacement. `RequireReuse` refuses instead, with
a typed `ReuseRefusal`: `Foreign` when another backend holds the retained state,
`Structure` when coordinates, profile, sparsity, bounds or the complete settings/options
signature differ ([§18.3](#section-18-3)); the boundary rule is `native.reuse`, class `incompatible`
([§23.2](operations-and-validation.md#section-23-2)). A reused session never inherits an
earlier step's native options ([§18.3](#section-18-3)). *Tested* by
`stub_backend_routes_through_adapter_table` (native backend units).
The retained state lives on the native session of a staged sequence
([§18.8](#section-18-8)). Retention requires both composed original candidate permission
and independent admission of the compatible session artifact; either refusal drops the
session. Separately admitted response products retain their own validity and owner.

**Shared runners.** Workflows choose a runner by representation and the adapter by table
lookup; neither step names a backend. The runners `execution::nlp`, `execution::roots`,
`execution::coefficients`, `execution::cone` and `execution::factorable` build the adapter's
native representation from original-coordinate inputs, execute it, recover original
coordinates and qualify the report (§18.6). The one NLP runner serves staged-sequence steps,
initialization blocks, fitting and the factorable runner's fixed-assignment re-solve
([§16.6](#section-16-6)); the factorable runner is described with its adapter
([§18.10.1](#section-18-10-1)). The coefficient runner re-evaluates the original compiled
model at the candidate (`execution::OriginalModel`): a projection that disagrees with it
becomes a validation failure, and the fresh original values replace the projected
observation before qualification.

**Selection.** The classes an adapter is assessed against follow from facts and intent
(`routing::problem_classes`): a root intent makes a square continuous problem
`square_root`; a continuous problem is `smooth_nlp`. For other intents a coefficient-eligible
problem adds `linear` or `mixed_linear` when it is affine, and when it is quadratic
`convex_quadratic` with convexity evidence, `nonconvex_quadratic` without it, or
`mixed_integer_quadratic` with a discrete column; a discrete problem that is not
coefficient-eligible is `mixed_integer_nonlinear`. Explicit cones and trajectories are never
inferred from algebraic facts. The classes are ordered most specific first: `square_root`,
then the coefficient or discrete class, then `smooth_nlp`. Automatic selection walks them in
that order and, for the first class that has one, takes the eligible adapter whose record
lists that class among its `automatic_classes` with the lowest automatic rank (KINSOL,
HiGHS, Ipopt, POUNCE, Clarabel, then SCIP). A rank orders the owners of one class and never
grants eligibility; Diffsol and IDAS are trajectory adapters that the algebraic router never
assesses. Roots and initialization therefore reach KINSOL, then Ipopt, then POUNCE:
one-sided bounds stay with KINSOL ([§18.10](#section-18-10)), and roots with two-sided boxes
route to a constrained NLP adapter rather than dropping bounds. Optimization reaches HiGHS
for admitted linear, mixed-linear and convex quadratic programs, then Ipopt, then POUNCE.
SCIP is automatic for mixed-linear, mixed-integer quadratic and nonlinear programs and
smooth NLP, behind HiGHS, Ipopt and POUNCE, so it is selected automatically for MIQP and
MINLP, which no other adapter represents, and wherever the others are ineligible, as for
native constraint forms; a continuous nonconvex QP stays with the local NLP adapters unless
certification is requested. Clarabel serves `linear`, `convex_quadratic` and
`continuous_cone` but is automatic only for cones, so an LP or convex QP reaches it only by
explicit selection, and a cone only through an explicit cone request, since cones are never
inferred. *Tested* by `problem_classes_follow_facts_and_intent`,
`explicit_only_classes_never_automatic` and `miqp_routes_to_scip` (native backend units).

An explicit selection is never substituted. An unlinked choice fails with `Unavailable`,
whose alternatives are the other linked adapters that the capability rule finds eligible
for this request; choosing one still passes structural, settings and contract admission.
Refusals outside routing, such as tear selection without HiGHS, list none. An ineligible
choice fails with every reason. The `certify` intent is only ever requested explicitly and
is served only by an adapter whose record `certifies` (today SCIP); every other adapter
carries the `Certification` reason, and without an eligible certifying adapter selection
refuses with a typed `Unsupported` before any route. An all-fixed model is refused under
`certify` rather than evaluated, because constant evaluation proves no bound.

A structure that leaves constraint forms to native handlers
([§7.5](mathematics-and-compilation.md#section-7-5)) is eligible only on an adapter whose
record lists every required `NativeConstraintForm`; every other adapter carries
`Ineligible::NativeForms` naming the missing handlers. No form is converted silently, and an
all-fixed model with native forms is refused rather than evaluated, since constant
evaluation enforces no handler. SCIP's record lists all seven handlers
([§18.10.1](#section-18-10-1)) and no other record lists any, so a structure with native
realizations routes to SCIP, automatically when it is linked, and is refused on every other
route (`gdp_indicator_matches_hull`, runtime units). A refusal lists every adapter's
assessment by registry spelling.

**Typed routes and eligibility.** A prepared operation keeps its selected `routing::Route`
(`Constant` or `Native(backend)`) and one `routing::Eligibility` row per assessed adapter:
the backend and every applicable `Ineligible` reason, empty when eligible. Each reason has
one registry code, `NativeIneligibility` (`not_linked`, `serial`, `not_square_root`,
`no_objective`, `certification`, `class`, `derivatives`, `bounds`, `native_forms`,
`least_squares`, `method`, `lexicographic`; `Ineligible::code`), and keeps its typed
detail: the problem's classes, the required derivative order, whether sign bounds were
representable, the missing handlers, or the unmet structural requirements. `method` marks a
formulation whose structural requirement (ADR-0104 §5) the adapter's record or the request's
settings do not honour: the ℓ1 exact penalty an authored `penalty(l1)` states is honoured
only by POUNCE's `L1ExactPenalty`, which native defaults on POUNCE then select as the
author's choice, never an automatic one, and the report records that the penalty relaxes
every row (`authored_l1_realization_selects_route`, runtime units; `l1_never_automatic`,
native backend units). `lexicographic` marks several objectives in a class the adapter does
not optimize lexicographically in one solve (`several_objectives_route_only_to_a_lexicographic_record`).
`least_squares` marks a Gauss–Newton Hessian requested for anything but a least-squares fit
objective, which only fitting declares ([§19.4](workflows-and-results.md#section-19-4)); every
adapter refuses it before native work (`gauss_newton_requires_least_squares_objective`). The
Python boundary exposes them as `NativeRoute`, `NativeEligibility` and `NativeIneligible`
values in registry spellings, never as Rust `Debug` text. No runtime relation publishes
them. *Tested* by `test_route_and_eligibility_are_typed` (Python unit test).

Unsupported bound, derivative, nonsmooth and thread combinations
fail during preparation, before worker acquisition. Typed backend settings must belong to
the selected route (`BackendExecution::admit_settings`), and the route's model contract is
admitted before any worker exists (`admit_contract`, for example KINSOL's one-sided bounds
and guard signs).

Strategies are derived from distinct projections rather than topology alone: square
root blocks, declared causal fixed-point and Picard maps (§17.4), simultaneous
constrained NLP blocks and supplied continuation (§17.5). An SCC does not by itself
select KINSOL. A finite sequence of prepared steps runs on one native session as one staged
sequence ([§19.2](workflows-and-results.md#section-19-2)) and continues after an unaccepted
step only when the steps are declared independent.

### 18.8 Threading, cancellation and resource ownership

> Proposed amendment: [ADR-0154](../../adr/0154-declared-numerical-strategy.md),
> [ADR-0158](../../adr/0158-observed-pounce-execution.md), Plan 25n N2/N6.

Hard counters require pre-operation admission or complete conservative reservation covering
preparation, native final validation and independent assessment. Unknown measured work remains
unknown; a complete unreconciled reservation retains its allowance. Native observation/control
reports actual failed work and reserves complete simultaneously live storage through teardown.
Resource/cancellation/contract abort is distinct from numerical fallback.

The source-owned bounded FERAL profile covers complete linear storage for the actual serial AMD
factor geometry, including MC64 ordering/scaling, Schur coupling and refinement, monolithic fallback
and restoration/low-rank wrapper storage. Its optional dimension constraint is a caller limit;
absence derives the actual classified native geometry rather than selecting another numerical path.
Application heap storage remains opaque and deployment-owned. Retained storage transfers to its
resource owner; completing a task releases its ledger and foreign allowance while resource tokens
remain until teardown. The
[25n Outcome](../../plans/25n-automatic-simulation-solve-pipeline.md#outcome)
owns functional integration evidence and the qualification handoff.

> Decision: [ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md) — SPRAL
> (OpenMP) and oneMKL threads become admitted resources under this owner, with one
> BLAS/LAPACK provider, one OpenMP runtime and a pinned `MKL_CBWR` (Plan 22 N1,
> implemented); [ADR-0105](../../adr/0105-scip-factorable-backend.md) — SCIP's concurrent
> solving only in deterministic mode with the admitted permits as its threads (Plan 22 G7,
> implemented).

`MathService` (`pse-runtime/src/math.rs`, `math/jobs.rs`) is the single execution
owner. Its `MathPolicy` draws finite allowances from the deployment memory pool:
artifact retention, per-job foreign allowance, worker storage, workspace generations,
native stack, live jobs and flights.

> Decision: [ADR-0166](../../adr/0166-bounded-parallel-scientific-execution.md)
> (proposed, under Plan 28 implementation). Temporary demand is distinct from worker
> capacity. One population ticket bounds pending requests, running jobs and retained
> sessions together. Failed entry releases CPU and temporary dispatch resources before
> waiting on the common pool's release notification; notification grants no bytes.
> Retries retain cancellation and the original finite deadline. Without such a clock,
> the configurable admission-only default is thirty seconds and ends at dispatch.
> Native team stacks are reserved before actual team creation and held through idle
> scope and destruction/join. Actual native exclusion releases compute while waiting;
> guard acquisition and native destruction remain on the owning thread. These changes
> do not lower scientific demand or change native execution clocks. The owning plan
> distinguishes implemented slices and their evidence from this target.

- A job acquires a job slot, CPU permits for its admitted cores and a pool reservation
  covering stacks (one per worker plus a coordinator for parallel teams), numeric bytes
  and the foreign allowance; parallelism cannot exceed the effective process CPU count.
  A compiler job (preparation, rebinding, flow or function generation) runs inside its
  workspace's lease, which already covers the compiler's working set, so it charges only
  its thread up front; a fit preparation charges its sparse-layout bound (`max_cells`),
  not the workspace allowance.
  A library that enforces its own memory limit receives the job's foreign allowance as
  `Execution::memory`, set by `MathService::execute`; SCIP turns it into `limits/memory`
  and refuses to run without it ([§18.10.1](#section-18-10-1)).
- Mutable native state is constructed, used and destroyed on the owning `pse-math`
  thread. Only owned `Send` inputs and results cross; native objects may be `!Send`.
  Salsa never holds mutable solver state.
- Dropping a `SolveHandle` requests cancellation; awaiting `finish` observes completion
  only after native teardown, thread-local destruction and join. Permits and
  reservations survive caller cancellation until that join. Retained results transfer a
  split of the job reservation to an `AllocationLease` that lives with the result; a
  product larger than the job's working allowance first grows the reservation to its
  extent, and one the pool cannot admit is refused as a pool limit.
  Conditional block execution reserves the complete original and component result
  allowance before starting. After all components and the original assessment finish,
  its unique lease may be partitioned before any report escapes: known report envelopes
  count actual owned vector/string capacities and conservative map storage, together
  with conservative source-derived dimension headroom, bounded by the prior complete
  admission after checking the visible envelope. Excess capacity and arithmetic overflow
  refuse with a typed memory failure; an opaque extension keeps its complete admission.
  The partition releases only unused reporting capacity and attaches the retained owner
  in place. Native scratch, session and analysis owners retain their separate lifetimes.
  Extracted managed component reports share that owner through teardown. A caller's
  explicit deep clone copies its payload outside automatic engine admission; sharing
  the producer's owner does not separately charge or bound those caller allocations.
- A staged sequence runs on one `NativeSession` (`math/staged.rs`, Plan 22 A6). Opening it
  takes one job slot and a pool reservation for its thread's stack, the foreign allowance
  and the inner-session cache, held until the session closes. Its evaluators draw their
  numeric storage from the pool as they are built, up to one worker share, so an idle
  session holds no worker storage. Each step acquires CPU permits for
  its own admitted cores only, so the driver prepares and rebinds between steps without
  holding permits for an idle session. Every step, a single solve included
  (`MathService::solve` opens a one-step session), runs through `MathService::execute` on
  the session thread with the sequence's retained native state. The thread enters the
  scopes of the adapters its steps use (`execution::scoped`); a step that needs another
  adapter scope or thread count ends the current scope, dropping retained state, and
  re-enters. Cancelling a step sets that step's stop flag and leaves the session usable;
  closing the session joins its thread, which witnesses native and thread-local
  destruction.
- Cancellation and deadlines are checked only at library-supported checkpoints
  (Ipopt intermediate callback, POUNCE TNLP callbacks, KINSOL evaluations, HiGHS
  interrupt callbacks for simplex, IPM and MIP, Clarabel's termination callback, SCIP's
  event handler, integrator step boundaries). HiGHS' QP solver and PDLP never poll the interrupt
  callback, so they stop only at their native time limit. A long native factorization
  completes before teardown.
- Profile-chain worker panic and scheduling failure are typed outcomes, with actual
  parallelism recorded. A started chain is never silently replayed; partial spawn recovery
  assigns only unstarted chain indices to the surviving workers. Registered accelerators
  survive immutable package edits. [ADR-0150](../../adr/0150-checked-admission-and-owned-reuse.md)
  records this implemented ownership change; decision status remains proposed.
- KINSOL is serial. Clarabel is serial with QDLDL and runs its MKL Pardiso KKT solver on
  admitted MKL threads (`conic::admit_threads`), so its record is `parallel` only in a build
  with `clarabel-pardiso`; the owning worker's MKL-local thread count is set for the solve
  and restored afterwards (`mkl::Threads`). Ipopt, POUNCE, HiGHS and SCIP may use admitted
  threads. A count that a record cannot use is ineligible (`Ineligible::Serial`), and one
  its settings cannot use is refused when the settings are admitted
  (`BackendExecution::admit_settings`), both during preparation. FERAL factorizes on its own
  Rayon pool, which cannot be injected into the admitted local pool. It therefore runs
  parallel only when the admitted thread count is greater than one and covers that pool's
  whole size (`RAYON_NUM_THREADS`, otherwise the available parallelism), and serially
  otherwise; the report records the effective count as `linear.threads`.
- Ipopt's threads belong to its linear solver ([§18.3](#section-18-3)). MUMPS is sequential
  and refuses more than one; SPRAL runs on OpenMP threads and oneMKL Pardiso on MKL threads.
  For one solve the owning worker's OpenMP and MKL-local thread counts are set to the
  admitted count and restored afterwards (`ipopt::runtime::Threads`), and the report records
  it as `linear.threads`. The process environment these solvers depend on is fixed when the
  OpenMP runtime starts, so admission reads it back instead of setting it: SPRAL needs
  `OMP_CANCELLATION=TRUE` and a true `OMP_PROC_BIND`, and Pardiso needs the image's pinned
  `MKL_CBWR` branch and `MKL_DYNAMIC` off. The solver image sets `MKL_CBWR=COMPATIBLE`,
  `OMP_CANCELLATION=TRUE`, `OMP_PROC_BIND=TRUE` and `OMP_PLACES=sockets`; `pse-worker`
  re-executes itself with the OpenMP variables and a device-free `HWLOC_COMPONENTS` when
  any is absent, keeping an operator's explicit value. Any other foreign BLAS threading is
  environment configuration.
- SCIP solves concurrently when more than one thread is admitted: `SCIPsolveConcurrent` in
  deterministic mode (`parallel/mode` 1) with exactly the admitted count
  (`parallel/minnthreads` and `parallel/maxnthreads`), its LP solver kept at one thread
  (`lp/threads`) and presolve left to the concurrent solvers (`concurrent/presolvebefore`
  false). The record says `parallel`, and `scip::Settings::admit` refuses more than one
  thread only together with exact solving or reoptimization. Copies of the event handler in
  the concurrent solvers poll for cancellation and stream their improving solutions as
  incumbents ([§18.10.1](#section-18-10-1)). The admitted count is recorded as
  `scip.threads`. *Tested* by `concurrent_mode_under_admitted_permits`, a two-thread solve
  that checks the native options and a gap-qualified result, and
  `scip_concurrent_streams_incumbents` and `scip_concurrent_solve_cancels`, whose flag,
  raised at the first streamed incumbent, stops a two-thread solve as a cancellation that
  claims no gap (native backend units).
- Each attempt evaluator is built by one path, `MathService::worker`. Its provider workers
  are scoped to the attempt's cooperative cancel flag, so a nested native provider, such as
  an implicit inner solve, polls that same flag. Its numeric storage is charged to a
  per-job `WorkerBudget`, the worker share of the job's reservation (a session's budget
  draws each charge from the pool), for as long as the evaluator lives. Staged-sequence steps (the step, the original re-evaluation and the
  step's original-model assessment, all on the session's worker), block initialization,
  causal-map units and owned-worker jobs build their
  evaluators this way, so the reservation covers all of a job's live evaluators together;
  one that does not fit is refused as a worker-storage limit.

Reservations are conservative admission policy, not allocator interception or a
process RSS ceiling. The workstation sizing rationale and measured behavior are
qualification matters ([§24.2](operations-and-validation.md#section-24-2)).

### 18.9 Capability matrix

> Decision: [ADR-0102](../../adr/0102-discrete-and-global-design-target.md),
> [ADR-0105](../../adr/0105-scip-factorable-backend.md),
> [ADR-0109](../../adr/0109-pounce-l1-and-convex-methods.md),
> [ADR-0110](../../adr/0110-dynamics-profile-extensions.md) — the target adds SCIP,
> POUNCE-convex and extended dynamics; rows change only as Plan 22 packets land. Since
> Plan 22 A2 (implemented) each row is published from its adapter's capability record.
> Implemented: SCIP (G1, G3–G7 and the semi lowering, with G4 and G6 partial as stated in
> [§18.10.1](#section-18-10-1)), the Ipopt linear solvers and their threads (N1), the IDAS
> and Diffsol extensions (Y1, Y2), scheduled inputs (Y0c), adjoint sensitivities (Y3),
> Gauss–Newton and exact transient Hessians (Y4), shooting (Y5b), the KINSOL extensions
> (Y6, with the byte-bounded session cache) and POUNCE-convex (N5).
>
> Decision: [ADR-0121](../../adr/0121-convexity-compiler-facts.md) — the records publish
> `automatic_classes`; Clarabel serves `linear`, `convex_quadratic` and `continuous_cone`,
> automatically only for cones (Plan 22 C4, implemented), including the continuous cone
> programs preparation recognizes from the compiler's convexity fact (C5, implemented,
> [§18.10](#section-18-10)).

The linked inventory is the static adapter table `execution::LINKED`
(`pse-backend-native/src/execution.rs`). `runtime.solver_capabilities` (version 3: version 2
added `automatic_classes`; version 3 adds `requirements`, `lexicographic_classes`, `batch`
and `sensitivities`) publishes one row per linked adapter from the capability record
routing reads ([§18.7](#section-18-7)); the
test `published_capabilities_equal_routing_rules` rebuilds each adapter from its published
row alone and checks that routing assesses it identically. Feature
`pse-runtime/native-solvers` links the full profile, and Clarabel's non-SDP route is always
present. Diffsol and IDAS publish their records, but their trajectory representation
belongs to the integrator workflows ([§13.6](workflows-and-results.md#section-13-6)) and
the algebraic router never assesses them.

| Backend | Classes | Bounds | Derivatives | Starts | Threads |
|---|---|---|---|---|---|
| Ipopt | Smooth NLP | General | Exact Hessian or limited memory | Primal/dual with a typed restart | Admitted for SPRAL and oneMKL Pardiso; MUMPS serial |
| POUNCE | Smooth NLP | General | Exact Hessian or limited memory | Primal/dual with a typed restart; working set | Admitted pool |
| KINSOL | Square root, declared fixed point | One-sided, as shifted signs | Jacobian or JVP | Primal | Serial |
| HiGHS | LP, MILP, convex QP; LP and MILP levels lexicographically in one solve | General, semi domains | Coefficients | Primal/dual, basis, partial MIP start; QP hot start | Admitted |
| Clarabel | Explicit cones, automatic (SDP with `solver-sdp`, on oneMKL); LP and convex QP by explicit selection, lowered to cone form | General | Coefficients | None | QDLDL serial; MKL Pardiso on admitted MKL threads (`clarabel-pardiso`) |
| POUNCE-convex | LP, convex QP and continuous cones by explicit selection only, lowered to cone form; batches a study's independent points | General | Coefficients | The previous solutions of the same layout | Admitted pool: one instance's factorization, or a batch's instances each factored serially |
| SCIP | LP, MILP, convex and nonconvex QP, MIQP, MINLP, smooth NLP; certifies; all seven native constraint forms | General, semi domains through `semi(indicator)`; finite boxes inside nonlinear terms | Factorable | Primal incumbent | Admitted: deterministic concurrent mode, except with exact solving or reoptimization |
| Diffsol | ODE, semi-explicit index-1 | None | Forward sensitivities and adjoint gradients (`forward_and_adjoint_sensitivities`) | None | Serial |
| IDAS | ODE, semi-explicit index-1 | None | Forward sensitivities, adjoint gradients and the second-order adjoint (`second_order_adjoint_sensitivities`) | None | Serial |

HiGHS is linked at 1.15. Its record states that simplex, IPM and MIP honour the interrupt
callback while its QP solver and PDLP stop only at the native time limit
([§18.8](#section-18-8)); a QP start is a hot start that needs the exported basis
([§18.10](#section-18-10)). SCIP is linked at 10.0.2 (`scip-sys` 0.1.28, feature `scip`,
part of `native-solvers`); its record is the only one that `certifies` and the only one that
lists native constraint handlers ([§18.10.1](#section-18-10-1)). The published row carries the
`certifies` and `native_forms` columns. `sign_bounds` means one-sided bounds of any value,
represented as sign constraints on shifted coordinates (KINSOL, [§18.10](#section-18-10)).
The Diffsol and IDAS records distinguish their derivative routes
([§13.6](workflows-and-results.md#section-13-6)): Diffsol forward sensitivities and adjoint
gradients, IDAS in addition the second-order adjoint of an exact transient Hessian.
`batch` marks POUNCE-convex, the one adapter that solves a study's independent points as
one parallel batch ([§19.3](workflows-and-results.md#section-19-3)); its record states its
cancellation granularity, a solve-wide deadline with no interrupt, so a stop request is
honoured before a solve or batch starts and a running solve stops at the deadline.
`sensitivities` marks Ipopt, POUNCE and SCIP for optimizing KKT analysis, whose candidates
carry the required multipliers, and KINSOL for qualified regular-square Root response
under ADR-0144. Automatic routing of a sensitivity request prefers contextual support
([§15.5](#section-15-5)). *Tested* by `pounce_convex_never_automatic` and
`sensitivity_requests_route_to_multiplier_adapters` (native backend units).

Outside the matrix today: global solving where a provider output without a declared
envelope enters a nonlinear term (no production provider declares one yet); exact rational
solving of anything but a linear program without native forms, conditional rows or
auxiliaries; native handlers on any adapter but SCIP; a local analysis, sensitivities or
covariance read on the coefficient and cone routes themselves (a sensitivity request routes
automatically to an NLP adapter instead); exact transient Hessians on Diffsol; general or
higher-index DAE; finite-difference derivatives; GPU and distributed execution. Durable
multi-process execution through the
operational store ([ADR-0114](../../adr/0114-typed-operational-store.md),
[§20.6](identity-and-publication.md#section-20-6)) is implemented: jobs across worker
processes, studies, durable incumbents and resumption from them. General or higher-index
DAE, finite-difference derivatives, GPU execution and distributing one solve remain outside
the target.

### 18.10 Root, coefficient and cone adapters

> Decision: [ADR-0108](../../adr/0108-ipopt-linear-solvers-and-solver-image.md) — Clarabel's
> SDP profile runs on the image's single oneMKL BLAS/LAPACK provider (Plan 22 N1,
> implemented); [ADR-0116](../../adr/0116-typed-boundary-documents.md) — pse-owned boundary
> types, including Clarabel's (Plan 22 A5, implemented);
> [ADR-0083](../../adr/0083-class-specific-native-execution.md) — class-specific adapters.
> Plan 22 Y6 (implemented) types the KINSOL method controls, shifts one-sided bounds to sign
> constraints and caches nested sessions per worker. Plan 22 C2 (implemented) adds the HiGHS
> node budget, incumbents, fixed-commitment duals, the presolve and basis views and the
> gap-derived QP regularization.
>
> Decision: [ADR-0121](../../adr/0121-convexity-compiler-facts.md) — Clarabel serves LP and
> convex QP besides cones, its direct solver is QDLDL or MKL Pardiso with `faer-sparse`
> excluded, and its and HiGHS's rays become one verified certificate (Plan 22 C4,
> implemented); convexity is a compiler fact that HiGHS's convex-QP admission and cone
> routing read, and preparation recognizes continuous cone programs (Plan 22 C5,
> implemented).
>
> Decision: [ADR-0109](../../adr/0109-pounce-l1-and-convex-methods.md) — POUNCE-convex as an
> explicit method for linear, quadratic and cone programs, batched studies and
> sum-of-squares bounds (Plan 22 N5, implemented; its own QP sensitivity is excluded,
> ADR-0118).

**KINSOL** (`kinsol`) owns Newton, line search, Picard and fixed-point iteration with
Anderson acceleration. Callers choose only the typed `kinsol::Method`, the adapter's
pse-owned settings, identified through serde: the strategy; the linear solver, which is
vendored KLU over the analytic CSC Jacobian, bounded dense, or matrix-free SPGMR, SPFGMR,
SPBCGS or SPTFQMR over the analytic Jacobian-vector product with a Krylov dimension; the
Anderson history, damping and setup interval; an optional Newton-step cap
(`KINSetMaxNewtonStep`, at least one scaled unit); for the Krylov routes, the inexact-Newton
forcing term (`KINSetEtaForm`: Eisenstat–Walker choice 1 by default, choice 2 with its
safeguard and power, or a constant), optional right Jacobi preconditioning from the
analytic Jacobian diagonal, or library block-factor preconditioning from complete original
equality BTF diagonal blocks. Block factors require an assembled analytic Jacobian and
finite admitted storage; FERAL owns each sparse LU factor and solve. The full-system Krylov
action retains every off-block coupling through the actual demanded JVP. Factors refresh
under KINSOL setup control and compatible source/pattern ownership; they do not establish
fresh response factors. The adapter reports actual block setup, factor and solve counts
separately from unknown native-internal factorization work. The unpreconditioned route can
consume directional-only support without assembling a Jacobian. Every Krylov route admits
its complete basis, Hessenberg and native header storage before allocation, including routes
without preconditioning or Anderson acceleration. Anderson admits its peak vectors, QR,
pointer/scalar arrays and headers under the same finite foreign allowance; combining it with
Krylov or block factors admits their combined peak rather than independent subtotals.
Retained ownership supplements queried native workspace words with storage those queries
omit. Conditional initialization reserves that foreign allowance together with worker
storage under the original task scope. With Anderson acceleration, callers may also select
its orthogonalization and delay. A control that does not apply to the chosen route is refused, not ignored. Scales and
the step tolerance come from `Settings::from_policy` ([§16.6](#section-16-6)).

KINSOL represents one-sided bounds exactly as sign constraints on shifted coordinates: it
iterates `u = x − l ≥ 0` for `x ≥ l` and `u = x − h ≤ 0` for `x ≤ h`, which leaves the
Jacobian and the step tests unchanged, and a guard sign at least as strong as a bound is
enforced unshifted instead. This is what a capability record's `sign_bounds` means
([§18.7](#section-18-7)): one-sided bounds of any value route to KINSOL, which keeps every
iterate inside them. Two-sided boxes, a bound facing its guard sign and constrained
fixed-point or Picard iteration are refused. Compatible layouts reuse SUNDIALS/KLU
allocations on the owning worker; a reused session keeps the controls fixed at allocation,
refreshes the others, and never carries an earlier step's Newton-step, forcing-term or
Anderson controls.

A nested implicit solve (`implicit.rs`) reuses KINSOL sessions through a per-worker-thread
cache keyed by problem identity and bounded in bytes. The solve takes a compatible session
out of the cache, so a solve nested inside it cannot alias it; refreshes it with the call's
function and parameters, keeping the SUNDIALS context, vectors and KLU analysis; and returns
it afterwards, counted at its retained bytes (`kinsol::Session::retained_bytes`). The
cache's budget is `MathPolicy.inner_session_bytes` (16 MiB by default), reserved in the job
or native session lease ([§18.8](#section-18-8)) and handed to the owning thread
(`implicit::budget_sessions`); sessions beyond it are released least recently used first, a
session larger than the whole budget is not kept, and a thread that received no budget
keeps none, so no cache outlives an admitted job. The trial problem is lent to the session
for one solve only. An inner unknown's one-sided bound is represented exactly, while a
two-sided interval keeps only its sign information and is rechecked afterwards
(`Problem::verify`). *Tested* by `inner_session_cache_bounded_by_bytes` (native backend
units).

**HiGHS** (`highs`, HiGHS 1.15) receives a checked native upload whose full readback must
match the coefficient view before any bound or optimality claim transfers. It supports LP
(choose, simplex, IPM, PDLP), MILP with integer, binary and semi domains, and convex QP.
A mixed-integer model or a quadratic objective requires the `choose` method. HiGHS solves
a continuous QP with its active-set QP solver whatever method is named, and its QP interior
point (HiPO) is not built, so an explicit simplex, IPM or PDLP request with a nonzero
Hessian is refused as `Unsupported` rather than silently ignored. PDLP stops only at its
native time limit ([§18.8](#section-18-8)). Convex QP requires evidence: an exact rational
`GramCertificate` (`sign*Q = Rᵀ diag(w) R` with nonnegative weights, checked through
Symbolica/Numerica against the unchanged matrix and objective orientation) by default, or
an explicitly requested numerical PSD qualification with tolerances. An indefinite or
inconclusive matrix is never repaired. A submitted QP start is consumed: HiGHS 1.15 makes
the QP hot start opt-in, so the adapter sets `qp_allow_hot_start` unless the caller
supplies it, and the effective options record the value that ran. The active-set solver
returns a valid basis with every QP solution, and the output seed exports that basis with
the primal, because the hot start needs both. Scheduler teardown is exclusive and blocking.

The HiGHS settings (`settings::highs::Settings`) name the method, an optional MIP node
budget, opt-in diagnostics and a partial, source-attributed MIP start. The node budget
(`nodes`, native `mip_max_nodes`) is separate from the iteration budget; without one the
native default, no node limit, is restored explicitly, because a retained model keeps its
previous options, and the option is reserved. An improving MIP solution (callback kind 4)
becomes a typed `IncumbentEvent` in original units and coordinates, its primal captured
under a one-second throttle; a merely feasible one (kind 3) is not an incumbent. Durable
attempts stream and store incumbents ([§20.6](identity-and-publication.md#section-20-6)).
For a quadratic objective, the regularization δ that HiGHS's active-set QP solver adds to
the Hessian diagonal (`qp_regularization_value`, reserved) is derived from the requested
gap: it changes the objective by `δ/2·‖x‖²`, so δ is twice the absolute gap budget divided
by the largest `‖x‖²` over the variables' bounding box (an unbounded coordinate counts as
one normalized unit), and never exceeds the native default 10⁻⁷.

Opt-in diagnostics (`highs::diagnostics`) run on copies and never replace the original
solve: rays, IIS (for a MIP, of its LP relaxation), basis ranging and a feasibility
relaxation, and, with Plan 22 C2, four views. The fixed-commitment LP (`Highs_getFixedLp`,
solved separately) prices the constraints of a MIP with its discrete columns fixed at the
solution; its duals are conditional on that commitment, never duals of the MIP, and it is
refused for a model without discrete columns, without a feasible solution or with a
non-integral commitment. The other three are rows of the basis inverse of an optimal
continuous LP, the native presolved LP of a copied model with its postsolved solution, and
the pool of cuts after root cut generation. A view that cannot run records its reason.
The runtime publishes the rays, IIS, ranging and relaxation as metrics, and the rays of a
continuous LP also become the typed, verified certificate described with Clarabel below; the
four views are returned on the report only. The Jacobian degeneracy analysis
(`jacobian_diagnostics`) runs
its LP certificates and minimum-support MILPs with one HiGHS session per problem family,
updated from anchor to anchor. *Tested* by `mip_node_budget_independent`,
`highs_incumbents_streamed`, `qp_regularization_within_gap_budget`,
`fixed_lp_duals_conditional_on_commitment`, `basis_inverse_and_presolve_views`,
`cut_pool_captured_on_request` and `degeneracy_hunter_reuses_session` (native backend
units).

**Clarabel** (`conic`) receives explicit cones through a pse-owned boundary vocabulary:
`conic::SparseMatrix` (CSC with its dimensions), `conic::Cone` (zero, nonnegative,
second-order, exponential, power, generalized power and PSD triangle, tagged by `kind`) and
`conic::Settings`, the mode, the KKT direct solver and every admitted native control;
iteration and time budgets, stopping tolerances, equilibration and threads belong to the
shared controls and the resolved accuracy. These types are the Python wire format and
the request identity (`pse.cone.layout.v3`) and map to Clarabel's own types only inside the
adapter, so a Clarabel upgrade cannot change a Python contract or an identity; Clarabel's
spellings are refused. Matrix, dimension and parameter checks precede the mapping. SDP uses
packed PSD-triangle cones under the backend feature `sdp` (`pse-runtime/solver-sdp`), whose
BLAS/LAPACK calls resolve into the image's one oneMKL provider (LP64, GNU threading)
through Clarabel's provider-less `blas-src` and `lapack-src`; a build without it refuses a
PSD cone. `SingleSolve` permits native presolve/chordal
preprocessing; `ReusableData` disables them to use native data updates. Results are
post-processed into source space. *Tested* by `clarabel_boundary_types_are_pse_owned`
(native backend units).

The KKT direct solver is the registry vocabulary `ClarabelDirect`: `qdldl` (the default,
serial) or `mkl_pardiso`, oneMKL Pardiso under the `clarabel-pardiso` profile, which admits
more than one thread; a build without the profile refuses it, and QDLDL refuses more than
one thread (`conic::admit_threads`). Clarabel's Pardiso loader opens a file named
`libmkl_rt.so`, which the solver image does not ship; the adapter therefore checks once per
process that `pardiso_` resolves into the linked LP64 interface library, publishes a
`libmkl_rt.so` alias of that library in a private temporary directory through the loader's
own `MKL_PARDISO_PATH` (the process's one environment write for it, made before the loader
first reads it), and verifies that loading the alias returned the object already in the
process and mapped no second oneMKL or Intel OpenMP runtime (`mkl::pardiso`); a loader that
would find another `libmkl_rt` is refused. This runtime alias is the supported mechanism;
the solver image is not changed for it (maintainer decision, 2026-09-29).
`clarabel/faer-sparse` is not used: it would bring a second faer beside the pinned one
(register R-45).

A linear or convex quadratic program explicitly routed to Clarabel is lowered to cone form
(`ConicProblem::from_coefficients`): equality rows form one zero cone, each finite side of
the other rows one nonnegative-cone row (upper before lower; a lower side's row takes its own
identity under `pse.cone.lowered-row.v1`), a maximization is negated, the quadratic keeps
only its upper triangle with evidence for exactly that matrix, and discrete columns are
refused. Variable bounds stay on the contract and are appended as cone rows by the adapter.
The coefficient runner lowers, solves and restates the result on the coefficient rows
(authored-sense row duals, reduced costs and certificate coordinates) before the original
model re-check.

**Convexity as a compiler fact** (Plan 22 C5, ADR-0121). Preparation decides convexity once
and carries it with the view's value products (`ProblemFacts.convexity`,
`pse.math.convexity-fact.v1`), so a value-only rebind re-decides it with the values and
never reuses a stale verdict. A degree-two coefficient objective is convex by an exact
rational LDLᵀ with symmetric pivoting of its quadratic form, which keeps its Gram factors
(`GramCertificate`, `pse.math.gram.v2`, non-diagonal forms included) or finds a direction of
negative curvature; `ConvexityPolicy::Numerical` stays an explicit per-request qualification
(`Requirements::numerical_psd`) and never becomes a fact. A continuous nonlinear program is
recognized as a cone program by the curvature pass over the factorable program preparation
already builds (`pse-math::curvature`): DCP composition rules with signs from the
outward-rounded interval box prove each node's curvature and record its atom (exponential,
logarithm, entropy and relative entropy to the exponential cone; powers to the power cone;
absolute values to linear rows; Euclidean norms and certified quadratic forms to
second-order cones). Only exact rows and objective over continuous columns, without
auxiliaries, implicit residuals or native forms, and with every retained obligation
following from the box, are recognized. The recognized program's epigraph form
(`pse.cone.recognized.v1`, `conic::recognized`) is what a cone adapter lowers, raised back
to the program and re-checked against the original case; routing classes the program
`continuous_cone`, which Clarabel owns automatically. The SCIP curvature detection is a
differential test oracle only. *Tested* by `exact_ldlt_certifies_nondiagonal_psd`,
`gram_certificate_yields_soc` and `curvature_sound_against_scip_oracle` (math units),
`unrecognized_problem_not_routed` (native backend units), `convexity_fact_rebinds_with_values`
(compiler units) and `recognized_exp_cone_routes_to_clarabel` and
`numerical_psd_only_under_explicit_policy` (runtime units).

**POUNCE-convex** (`execution::pounce_convex`, Plan 22 N5) is explicit only. It receives the
pse cone form, including a coefficient program the runner lowered, and maps it to POUNCE's
standard form: zero-cone rows become equalities, every other block inequality rows
`G x ⪯_K h` with its `ConeSpec`, and the variable box stays first class. POUNCE stores the
power cone norm-first and PSD blocks as the lower triangle, so both are row permutations
mapped back for the multipliers; the generalized power cone is refused. A quadratic
program runs through `solve_qp_ipm_warm`, a cone program through `solve_socp_ipm_warm`, to a
tolerance a decade inside every budget the report is qualified against. The report carries
the primal, dual and gap residuals of the cone form, a Farkas or recession certificate in
the certificate layout above, and warm starts come from the previous solutions of the same
layout (`settings::pounce_convex`: self-dual embedding or the direct method, equilibration,
crossover and FERAL). Batches are described in
[§19.3](workflows-and-results.md#section-19-3). *Tested* by
`pounce_convex_qp_matches_highs` (a convex QP and a mixed LP against HiGHS),
`pounce_convex_farkas_certificate` and `psd_rows_map_lower_to_upper_triangle` (native
backend units).

**Infeasibility certificates.** A primal-infeasible (Farkas) ray or a dual-infeasible
(recession) direction from Clarabel, or from HiGHS when its diagnostics request rays
(`diagnostics.rays`; fetching them on every infeasible LP is register R-39), becomes
one typed `InfeasibilityCertificate` with its native accuracy (`full`, or `reduced` for
Clarabel's almost-infeasible statuses). `transport::recover_certificate` restores it to
original coordinates, and `certificate::verify` recomputes it against the original data
with the feasibility budget as tolerance: for a Farkas ray the residual `|Aᵀy|` bounded over
each finite box, the dual-cone violation and the margin by which `−bᵀy` exceeds the rows' and
bounds' acceptance budgets weighted by `|y|`, which must be positive, so a problem
infeasible by less than its budgets is never certified; for a recession direction an
objective `qᵀx` below minus the tolerance relative to the direction's magnitude, with the
worst residual and the cone violation within tolerance.
`runtime.infeasibility_certificates` publishes the ray over rows and finite variable bounds
with its verification, and only a verified ray at full accuracy carries the `certificate`
assurance ([§18.6](#section-18-6)). The string-keyed `certificate.*` metrics are removed.
*Tested* by `clarabel_lp_matches_highs`, `clarabel_qp_farkas_certificate`,
`farkas_certificate_verified_in_original_coordinates`,
`almost_infeasible_is_not_certified`, `clarabel_mkl_pardiso_matches_qdldl` (a feasible LP
and a convex QP) and `clarabel_mkl_pardiso_refused_without_profile` (native backend units),
and `clarabel_serves_explicit_linear_program` and `infeasibility_certificate_published`
(runtime units).

Every normalized-coordinate adapter recovers candidates, duals and certificates to original
coordinates through `transport` before quality is assessed; the factorable adapter works in
original case columns ([§18.10.1](#section-18-10-1)).

#### 18.10.1 Factorable adapter: SCIP

> Decision: [ADR-0105](../../adr/0105-scip-factorable-backend.md) — SCIP 10.0.2 through the
> factorable projection and a pse-owned `scip-sys` binding (Plan 22 G1 and G3, implemented;
> implicit definitions and provider envelopes in the export, G4, implemented, with no
> production provider declaring an envelope yet, and the certified heater, G4r; IIS, G5;
> native handlers, solution pool, reoptimization, concurrent and exact modes, G7, with
> concurrent incumbent streaming since `518c3108`; the certified tangent-plane stability
> model, G6, partial, with the fixture intent and the stability tests of G6r; durable
> incumbents, G8, and their origin and pruning, G8f, implemented);
> [ADR-0103](../../adr/0103-variable-domain-facet.md) — semi domains lowered by
> `semi(indicator)` for SCIP (implemented);
> [ADR-0106](../../adr/0106-execution-vocabulary-discrete-and-global.md) — `global_bound`,
> `proven_infeasible`, `exact_certificate` and `GapQualified` with their conditions;
> [ADR-0120](../../adr/0120-provider-envelope-contract.md) — the provider envelope contract.

**Binding and lifecycle.** `pse-backend-native::scip` (feature `scip`) binds the raw SCIP
10.0.2 C API through `scip-sys` =0.1.28 against the solver image's `SCIPOPTDIR` headers;
neither `russcip` nor the `bundled` or `from-source` profiles is used. SCIP has no run-time
API-version query, so the build asserts `SCIP_APIVERSION` 156, the `SCIP_Real = f64`
signatures and a 32-bit `int`; at run time `scip::abi` checks `SCIPmajorVersion`,
`SCIPminorVersion` and `SCIPtechVersion` against 10.0.2 and refuses a mismatch as
`Unsupported`. One SCIP instance per attempt is created, used and freed on the owning
worker, on every path including unwinding, and no SCIP type leaves the module. Nothing
native is retained between attempts, except that a reoptimizing finite MIP sequence keeps
one instance and its search tree (below).

**Settings.** `scip::Settings` is pse-owned and identified through serde: the nested
Ipopt's linear solver (`nlp_linear_solver`, `mumps` by default), the random seed shift
(`seed`), a total node budget including restarts (`nodes`; absent leaves the deadline as
the only budget), the number of ranked stored solutions to report (`pool`, none by
default), and the switches `iis`, `exact` and `reoptimize`, all off by default. Mutually
exclusive combinations are refused with a typed `Unsupported` before any native work
(`Settings::admit`, when the route's settings are admitted and again when the instance is
configured): exact solving with reoptimization or with IIS generation, more than one thread
with exact solving or reoptimization, and SPRAL in the nested Ipopt without
`OMP_CANCELLATION=TRUE`.

**Representation and export.** The adapter's representation is `Factorable` and its
derivative capability `factorable`, so routing requires no derivative order of it.
Preparation projects the case under the step's values (`CasePlan::factorable_program`,
[§7.5](mathematics-and-compilation.md#section-7-5)) only for a factorable route, in a
bounded preparation job, with the case's implicit definitions and the declared output
envelope of every registered provider that has one
([§9.4](physical-semantics.md#section-9-4)). It admits the export there
(`execution::admit_program`), before the step reaches a native session, refusing with every
typed reason: an objective without a projection under an optimization or `certify` intent;
a variable or auxiliary inside a nonlinear term without a finite box, which spatial
branching needs; a semi column whose active interval is not `0 < l ≤ u < ∞`
(`SemiInterval`); or a nonfinite constant or exponent. The export plan collects the selected
rows that
have a projection, the constraints of unconditional obligations, implicit residuals (`= 0`)
and their declared bounds. A strict obligation bound is closed with a relative margin:
`x > b` exports as `x ≥ b + 10⁻⁹·max(1, |b|)`, and `x < b` symmetrically. A row without a
projection is dropped, which keeps the export a relaxation and makes it `Relaxed`. A binary
column's box is `[0, 1]` intersected with its declaration. Affine functions become SCIP
linear constraints and every other function a nonlinear constraint; an affine objective
becomes variable objective coefficients and a nonlinear one an epigraph variable.

**Semi domains.** SCIP 10.0.2 has no semicontinuous variable type, so the export lowers every
semicontinuous or semiinteger column by `semi(indicator)`, the one semi lowering of
ADR-0104 (HiGHS takes semi domains natively): a binary indicator `z` per column, the links
`x − u·z ≤ 0` and `x − l·z ≥ 0`, and the column's box `[0, u]`, integer for a semiinteger
column. The lowering is exact. Each is recorded on the report
(`ExportTransformation::SemiIndicator`, metric `export.lowered.semi_indicator`) while the
declared box stays the global domain, and an IIS maps the links back to the column's
declared bounds. *Tested* by `semi_indicator_lowering_matches_highs_native`,
`semiinteger_lowering_keeps_integrality`, `semi_transformation_recorded` and
`exact_mode_accepts_semi_lowering` (native backend units in the solver image).

**Native constraint forms.** The export consumes every form the structure leaves to a
native handler ([§7.5](mathematics-and-compilation.md#section-7-5)), each constraint named
after its native ordinal, and a fixed operand becomes a fixed variable. An indicator on a
linear row becomes one SCIP indicator constraint per finite side, on the literal that
activates it (the negated variable for an indicator active at zero). An indicator on a
nonlinear row is lifted exactly through slacks, `f − s ≤ u` and `f + t ≥ l` with
`s, t ≥ 0` that the literal forces to zero, because SCIP 10.0.2's superindicator over a
nonlinear constraint crashes during solving. SOS1 and SOS2 sets keep their weights; `and`,
`or` and `xor` become SCIP's logic constraints, `xor` as the parity constraint
`z ⊕ a ⊕ b = 0`, and SCIP presolve upgrades an asserted `or` to `logicor`. A cardinality
bound is given explicit weights, because SCIP 10.0.2 copies a cardinality constraint into
sub-SCIPs by duplicating its weights, which are null when none are given. A fixed inactive
indicator removes its row, more than one indicator on a row is refused, and a logic or
indicator operand must be binary. *Tested* by `native_forms_consumed_by_scip`,
`indicator_on_nonlinear_row_unenforced_in_fixed_assignment_resolve` and
`scip_export_readback_equivalent` (native backend units in the solver image) and
`gdp_indicator_matches_hull` (runtime units).

**Readback.** Before solving, the adapter reads the native model back and evaluates every
exported constraint and the objective against `FactorableProgram::evaluate` at the start
clamped into the box, with auxiliaries at their box midpoints. The largest relative
deviation must not exceed 10⁻⁹ for any global claim to transfer (`GlobalEvidence::readback`).

**Reserved options.** `misc/catchctrlc`, `limits/time`, `limits/memory`, `limits/gap`,
`limits/absgap`, `limits/totalnodes`, `numerics/feastol`, `randomization/randomseedshift`,
`lp/threads`, `nlpi/ipopt/linear_solver`, `nlpi/ipopt/hsllib`, `nlpi/ipopt/pardisolib` and
every `parallel/`, `concurrent/`, `iis/`, `reoptimization/` and `exact/` option are set only
from typed settings and controls; a caller-supplied value for one of them is refused. `catchctrlc` is false, because signal
handling belongs to the Python boundary; the time limit is the attempt's remaining deadline;
the memory limit is the job's foreign allowance in MiB (`Execution::memory`,
[§18.8](#section-18-8)); the relative and absolute gaps come from `ResolvedAccuracy`, the
absolute one converted to original objective units; the feasibility tolerance is the
resolved normalized budget clamped to [10⁻⁹, 10⁻⁶]; the seed shift and the total node
budget come from `scip::Settings` (`seed`, `nodes`); `lp/threads` is one; and the nested
Ipopt's linear solver is the typed `IpoptLinearSolver` (`mumps` by default, `spral`, refused
without `OMP_CANCELLATION=TRUE`, or `pardisomkl`), while the HSL and Pardiso-project loaders
are never set. After configuration every reserved key and every admitted free-form option is
read back from the instance in its native type, and that snapshot is the report's effective
options. More than one admitted thread runs SCIP's deterministic concurrent mode
([§18.8](#section-18-8)).

**Cancellation and progress.** An event handler on the instance catches presolve rounds and
node, LP, best-solution and dual-bound events. On each it polls the attempt's cancel flag
and calls `SCIPinterruptSolve` once, on the owning thread; on every new best solution or
improved dual bound it pushes a `scip.bound` progress event with both bounds. Between the
INITSOLVE and SOLVED stages, a new best solution also becomes a typed `IncumbentEvent`: the
objective's value, dual bound, gap, nodes, native seconds and, throttled to one capture per
second (the first and the last always kept), SCIP's best solution over the export's program
columns. A durable attempt streams and stores it, and a resumed job injects it
([§20.6](identity-and-publication.md#section-20-6)). The reported objective is always the
objective function's value, never SCIP's epigraph variable: an affine objective is SCIP's
original objective with the export offset applied, and a nonlinear one is the objective
expression evaluated at the solution (`scip::objective_value`), for incumbents, the
candidate and pooled solutions alike (`epigraph_incumbent_reports_function_value`). A panic
in the handler is contained as a SCIP error. The handler is copied into sub-SCIPs, the
concurrent solvers and the IIS sub-problem. A copy polls for cancellation, and its bounds
are not the attempt's. A concurrent solver also reports its new best solution when it
improves on every incumbent reported so far, whichever solver found it, so the stream stays
monotone: concurrent solving starts without central presolving, each solver's transformed
variables keep the export's names, and its solution maps back to the program's columns by
name, where the neutral program evaluates the objective; such an incumbent carries no dual
bound or gap (`scip_concurrent_streams_incumbents`).

**Status map.** `scip::termination` maps every raw `SCIPgetStatus` value of the 10.0.2 ABI
without a wildcard: `OPTIMAL` and `GAPLIMIT` are success with the native assurance
`global_bound`, because the requested gap limit is the successful stop; `INFEASIBLE` is
infeasible with `proven_infeasible`; `UNBOUNDED` and `INFORUNBD` keep their categories;
`USERINTERRUPT` and `TERMINATE` are cancelled; the node, total-node, stall-node and restart
limits are a limit; `TIMELIMIT` is the time limit; `MEMLIMIT` is resource exhaustion;
`PRIMALLIMIT` and `DUALLIMIT` are an objective limit; `SOLLIMIT` and `BESTSOLLIMIT` are a
solution limit; and `UNKNOWN` is inconclusive. Every stop but the first two leaves the bound
unestablished, and a raw value outside the ABI is an internal error. Qualification then
constrains the native assurance (below).

**Incumbents and the solution pool.** A compatible primal seed ([§17.6](#section-17-6)) is
submitted through `SCIPaddSolFree`, as a partial solution when auxiliaries or an epigraph
exist, and the report records whether SCIP stored it; exact solving certifies its own
solutions and is given no floating-point seed. SCIP's best solution in program columns is
the reported candidate and the output seed. With `pool` greater than zero, up to that many
stored solutions are reported beside it in SCIP's order, best first (for an epigraph
export, the epigraph variable's order), each with its objective's value, and the factorable
runner re-qualifies each in original coordinates like the candidate. `runtime.solution_pool`
publishes them, one row per rank and free variable, with
the objective and the original feasibility, absent when the point could not be evaluated.
A pooled solution is an observation: only the qualified candidate is a result or a seed.
*Tested* by `solution_pool_ranked` (native backend units) and `solution_pool_published`
(runtime units).

**Reoptimization.** With `reoptimize`, a finite MIP sequence keeps one SCIP instance and its
search tree in the sequence's retained native state ([§18.7](#section-18-7)). A later step
reuses it only when its constraint system is unchanged: the identity
`pse.scip.reoptimization.system.v1` frames the program's structure, each column's identity,
domain and box with the auxiliary and indicator boxes, each exported constraint's affine
form, sides and condition, and the native forms with their operands and weights, and no
value key, so only the linear objective, including its sense, may change. The step frees the
previous reoptimization solve, changes
the objective (`SCIPchgReoptObjective`) and refreshes the time limit. Constraints and
objective must be linear. A different system tears the instance down and builds another, or
is refused under `RequireReuse`; without `reoptimize` nothing is retained. The report
records the reuse (`reused_native_state`, `scip.reoptimized`). *Tested* by
`reoptimized_sequence_matches_cold_solves` and
`reoptimization_session_follows_the_constraint_system` (native backend units).

**Exact rational MILP.** With `exact`, SCIP solves in rational arithmetic
(`SCIPenableExactSolving`, set before the problem exists). Variables and linear constraints
receive exact data; every finite binary64 value is an exact dyadic rational, so the
conversion is exact. The export must be a linear program without native forms, conditional
rows or auxiliaries, and anything else is refused; the `semi(indicator)` lowering adds only
binaries and linear rows and is admitted (`exact_mode_accepts_semi_lowering`). A solve
counts as exact only when SCIP
reports it exact and its status is optimal or infeasible; qualification then grants
`exact_certificate` ([§18.6](#section-18-6)), and the report keeps SCIP's exact rational
objective as text. *Tested* by `exact_mode_on_delicate_milp` (native backend units), which
also shows that a floating-point solve never claims `exact_certificate` and that a
nonlinear program is refused.

**Irreducible infeasible subsystems.** With `iis`, a proof of infeasibility (status
`INFEASIBLE`, not `INFORUNBD`) is followed by SCIP's IIS finder within the remaining time
(`SCIPgenerateIIS`). The subsystem SCIP leaves in its sub-problem is attributed by native
name to exported functions (selected rows, obligations, implicit residuals and implicit
bounds), native forms and the finite declared bounds of program variables and auxiliaries.
Constraints are minimized, and the declared bounds of the variables the kept constraints
use stay members. `irreducible` records SCIP's flag: removing any function member leaves
the kept system feasible, within its tolerances. Linear and nonlinear programs are both
served. Two SCIP 10.0.2 behaviours are avoided. With bound removal, its post-processing
deletes any linear constraint whose single-use variables lost both bounds, which can leave
a feasible "subsystem", so bounds stay in the subsystem (`iis/removebounds` false). And
constraints re-added by the greedy finder's additive phase are skipped by its deletion
phase while the result is still reported irreducible, so only the deletion filter runs
(`iis/greedy/additive` false). `ModelingPackage::certify_infeasibility` runs the `certify`
intent on SCIP with `iis` and returns the assurance, the export fidelity, the subsystem's
rows and members and the irreducibility flag, beside the local ℓ1 explanation
([§15.5](#section-15-5)); no runtime relation publishes the subsystem yet. *Tested* by
`nonlinear_iis_irreducible_flag`, `mip_iis_on_true_mip`,
`iis_irreducible_whatever_the_row_order` and `global_infeasibility_proof` (native backend
units) and `certified_infeasibility_beside_local_explanation` (runtime units).

**Metrics.** Every SCIP report records the conditions its global claims hold under:
`global.feasibility` (the native feasibility tolerance), `global.gap_relative` and
`global.gap_absolute` (the requested gaps, the absolute one in original objective units)
and `global.domain` (the identity of the declared box), beside `export.fidelity`,
`scip.threads`, `scip.exact` and `scip.reoptimized`.

**Auxiliary callbacks.** Factorable preparation retains Value support for the primary SCIP
program. When its admitted export requires a fixed-assignment re-solve, the compiler prepares
a separate genuine callback product: Second for the requested exact local Hessian or First
for limited memory. Provider preparation consumes that actual order. Compiler-issued artifact
keys and order enter preparation identity, and the product retains its allocation owners.
Unavailable source support remains a local refusal; it never upgrades the primary program's
capability or converts an unpriced incumbent into an original result.

**The factorable runner and the candidate rule.** `execution::factorable` re-observes the
candidate against the original compiled model with fresh values, the declared boxes and
integrality ([§15.4](#section-15-4)), then decides where the candidate comes from (ADR-0105
§2):

- an original-feasible incumbent of an exact export is the candidate
  (`PrimalSource::Backend`), unless the program combines integer columns with a non-affine
  function (MIQP and MINLP);
- otherwise the incumbent is an assignment proposal, committed as closed boxes
  (`AlgebraicOracle::with_fixed_assignment`): every integer column is fixed at its rounded
  incumbent value; a semi column on its zero branch is fixed at zero, a semicontinuous one
  on its active branch keeps its declared `[l, u]`, and a semiinteger one is fixed at its
  rounded value within it. The continuous problem is re-solved through the one NLP runner by
  the automatic NLP route, seeded explicitly at the incumbent, with its adapter's default
  settings and a coordinate and profile stamp of its own that frames both ends of every
  committed box
  (`pse.factorable.fixed-assignment.v1`). An original-feasible re-solve candidate is adopted
  (`PrimalSource::FixedAssignment`) with its KKT evidence and its local analysis and any
  requested sensitivities ([§15.5.1](#section-15-5-1)), all conditional on the assignment,
  and it replaces the incumbent as the output seed; the incumbent's objective stays a metric
  (`semi_minlp_fixed_assignment_resolve`, `fixed_assignment_resolve_keeps_local_analysis`);
- without an adopted re-solve, the incumbent of a relaxed export remains an observation
  (`PrimalSource::RelaxedIncumbent`), which candidate use makes `diagnostic_only`
  ([§16.6](#section-16-6)).

The dual bound's source is recorded as `ExactExport` or `RelaxedExport`, and the box the
backend branched over as `pse.factorable.domain.v1`.

**Qualification** ([§18.6](#section-18-6)). With `GlobalEvidence`, an original-feasible
candidate is `Feasible`. Nothing global transfers unless the export read back equivalent.
An infeasible stop whose backend concluded infeasibility then grants `proven_infeasible`,
and a successful stop with a finite dual bound grants `global_bound`: a dual bound for the
exported program over the declared box, within the recorded tolerances and export
fidelity, never interval-rigorous. `GapQualified` additionally needs an original-feasible
candidate from a result source, never a relaxed incumbent, whose fresh original objective
lies within the requested absolute gap of the bound, or within the relative gap when both
share a sign. A `Relaxed` export's own incumbent therefore supports only the bound and the
infeasibility conclusion; a gap may combine its dual bound with an adopted fixed-assignment
candidate, and both sources are recorded. A conclusion SCIP reached exactly over an exact
export grants `exact_certificate` in place of either assurance, and a gap closed that way
qualifies as `OptimalWithinTolerance`.

**Routing** ([§18.7](#section-18-7)). The record's classes are `linear`, `mixed_linear`,
`convex_quadratic`, `nonconvex_quadratic`, `mixed_integer_quadratic`,
`mixed_integer_nonlinear` and `smooth_nlp`, with general bounds, primal starts and
`certifies` and the seven native constraint handlers. Its automatic rank is last, so it is
selected automatically only where no other adapter is eligible: mixed-integer quadratic and
nonlinear programs, and structures that leave a form to a native handler. It is the one
adapter that serves the `certify` intent, and every other class reaches it only by explicit
selection. Certification and global solving apply to factorable problems
over finite boxes. Implicit blocks enter through their original residuals, and a provider
output inside a nonlinear term needs a declared envelope
([§7.5](mathematics-and-compilation.md#section-7-5)); no production provider declares one
yet, so outside tests such a term is still refused. *Tested* by
`certify_exports_implicit_residuals_exactly` (runtime units) and `relaxed_export_bound_only`
(native backend units).

**Certified heater** (Plan 22 G4r). `heater_optimization_certified` (native acceptance
conformance) certifies the authored PC-SAFT heater optimization, declared `intent certify;`
over a vapor-branch box, with SCIP's optimal status, `GapQualified` and `global_bound` and a
read-back-equivalent export. The search needs a foreign allowance of 2 GiB, which becomes
SCIP's `limits/memory`. Measured on 2026-09-28, 512 MiB and 1 GiB stop at the memory limit
before the first node, while 2 GiB proves optimality in 91 nodes and the whole run takes
18 s. The allowance is declared on the solve that needs it: `SolveControls.foreign_bytes`,
authored in the fixture as `policy { limits foreign_bytes(2147483648); }`. That solve's own
reservation charges it (DP-20). An unset control keeps the deployment default
(`MathPolicy.foreign_bytes`, 64 MiB) and leaves request identities unchanged. Shooting and
fitting solves follow the same rule (Plan 23 H1f).

**Phase stability** (Plan 22 G6, partial). The authored tangent-plane-distance model
(`TangentPlaneStability` in the reference thermodynamics package) minimizes the
tangent-plane distance of a trial phase, and its authored check reads the result; an
objective-bound check reads the certified dual bound when the step carries one
([§19.2](workflows-and-results.md#section-19-2)). A fixture selects the `certify` intent by
declaring it (`intent certify;`, [§6.10](schema-and-relations.md#section-6-10)). Native
acceptance conformance covers four cases:

- `tpd_certifies_stable_feed` certifies an ideal mixture stable: automatic selection reaches
  SCIP, the result is `GapQualified` with `global_bound`, the dual bound and the distance are
  zero within tolerance, and the check's basis is `global_bound`;
- `fixture_intent_selects_certify` runs the same fixture under an automatic `optimize`
  policy: the declared intent takes it to SCIP with a `global_bound` check basis, while the
  control without the intent stays on a local route with a `point` basis;
- `tpd_detects_known_instability` finds the known Peng–Robinson instability of an
  equimolar benzene–toluene feed at 368 K and 101,325 Pa on a local Ipopt route: the
  authored distance lies below −0.1,
  agrees with teqp 0.23.1's `canonical_PR` reference within 10⁻³ and with the authored
  reference within 10⁻⁸, and the stability check fails with a `point` basis;
- `pcsaft_tpd_fits_the_formal_pool` prepares the three-component PC-SAFT distance, which
  needs 5,510 formal slots, within the default slot allowance of 8,192 (one pool chunk;
  `pse_modeling::Limits::body_slots` unset,
  [§7.1](mathematics-and-compilation.md#section-7-1)); an allowance of 4,096 refuses it
  with `MathError::SlotLimit`, and it solves locally with Ipopt.

The campaign's `bt_pr_liquid_tpd_reference` is a separate local liquid reference at
368 K and 101,325 Pa. It retains the authored Helmholtz model, original domains and
TPD = 0 oracle within 10⁻⁶, and explicitly uses Ipopt with original feasibility,
stationarity and complementarity. It replaces the historical campaign failure expectation
whose success depended on SCIP's upstream convex-handler defect. The saved CIP reproducer
and dedicated contradiction controls retain that defect under R-52; local reference
agreement establishes no global stability bound.

Certification is established for the ideal feed only. SCIP does not close the gap of the
Peng–Robinson instability case in bounded time (after 10 minutes and 42,413 nodes its dual
bound was −2.05 beside an incumbent at −0.1104), and the PC-SAFT distance is solved locally,
not certified. Detecting an instability is therefore a local result, and certified
stability beyond the ideal feed remains open.

## Additional and retired section identities

#### 15.1 Thresholds (defaults preserved from IDAES) — retired

No IDAES diagnostic threshold set exists; acceptance tolerances come from the resolved
numerical policy in [§16.1](#section-16-1) and [§16.6](#section-16-6).

#### 16.3 Nominal value algebra

Authored nominal and scale annotations, including composed scheme defaults, resolve through
[§16.2](#section-16-2). They do not infer correctness from the current numerical trial.

#### 16.4 Scaling schemes

Scaling is package knowledge interpreted by the generic annotation/compiler mechanism.
The resulting targets enter the single numerical resolver; see [§16.1](#section-16-1).

#### 17.2 Authored initialization stages

Inherited stages and `when` variants specialize the same definition with explicit stage
bindings. Original model checks remain active, while final fixture expectations apply only
to the final original specification. See [§17.1](#section-17-1).

#### 17.3 Initialization order

Demanded children, effective stages and structural blocks determine execution from admitted
contracts. Failure and cancellation retain attempted/unattempted states; no caller mutation
or hidden plugin order supplies authority. See [§17.5](#section-17-5).

#### 18.4 NL backend — retired

Production NL/Pyomo routes were removed in favor of native class-specific adapters
([§18.1](#section-18-1), [ADR-0083](../../adr/0083-class-specific-native-execution.md)).

#### 18.5 Kernel adapters generated from `KernelSpec` — retired

Library-owned evaluation programs and provider workers ([§18.2](#section-18-2),
[§7](mathematics-and-compilation.md#section-7)) replaced generated kernel adapters.
