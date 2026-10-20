---
title: "Plan48e: Dynamics, fitting and preconditioning"
status: draft
date: 2026-10-20
adrs: []
doc_role: plan
doc_owner: docs/plans/48-execution-performance-and-numerical-efficiency.md
doc_topics: [design, numerical-execution, performance, preparation]
review_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md]
---

# Plan48e: Dynamics, fitting and preconditioning

## Purpose, authority and focused foundations

**Proposed:** this companion develops AF14/AF15, N01 and inquiries I16/I21–I27. The
[coordinator](48-execution-performance-and-numerical-efficiency.md) owns finding dispositions,
inquiry outcomes and the operator's 2026-10-20 acceptance of all RC01–RC12 recommendations
as written. RC05 and RC10 permit the stated target/investigation boundaries; acceptance
does not pick a preconditioner, reset policy, sensitivity method or optimizer winner.
Product execution is unstarted. This document owns only its E01–E08 packet states.
The [audit](../design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md)
and [recommendation register](../design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md)
remain source evidence rather than a second status ledger.

The focused foundation assessment applies Core 3.4, Efficient Architecture 1.0 and Process
Simulator 1.5 under the binding. It reuses the audit's settled conclusions and checks their
implementation boundaries, without restarting the numerical review. Relevant owners are
[workflows and results](../authoritative_design/sections/workflows-and-results.md), blueprint
§13.6 and its fitting contract, and [numerical execution](../authoritative_design/sections/numerical-execution.md),
blueprint §16.6, §18.6, §18.8 and §18.10. Source examined includes:

- `crates/pse-backend-native/src/dynamics/idas.rs`: a session owns native vectors,
  callback context, linear solver, sensitivities, quadratures and adjoint history. Existing
  schedule/segment reinitialization is not a complete arbitrary new-trial reset.
- `crates/pse-runtime/src/workflow/dynamics.rs`: existing pure-function cache and
  derivative-sufficiency handling after current binding. It is not an absent cache needing
  replacement by a weaker `(t,y)` key.
- `crates/pse-runtime/src/workflow/integrated.rs` and `workflow/shooting.rs`: shared
  integrated experiment, selected sensitivity/functional demand, one serial WindowWorker,
  fixed-trial anchored windows and a Point replaced only after a complete successful upgrade.
  Final original assessment has independent ownership.
- `crates/pse-runtime/src/workflow/fitting/oracle.rs`, `experiments.rs`, `profile.rs`,
  `covariance.rs` and `preparation.rs`: demand-specific products, existing experiment teams,
  exact/Gauss–Newton curvature and final/rank/profile consumers. Gauss–Newton already supplies
  `JᵀWJ` and includes the required nonlinear constraint curvature.
- `crates/pse-backend-native/src/dynamics/dae_preconditioner_tests.rs` and the IDAS
  preconditioner setup: direct KLU and None/Jacobi Krylov exist; BlockFactor is explicitly
  unsupported on this route. Jacobi currently requests full partials to select a diagonal.

These foundations favor extending current owners rather than adding a second dynamic
execution path, cache or fitting solver loop. The native-solver-libraries skill supplies
IDA/IDAS, root/NLP and sparse-factor routes. Source/feature/ABI fit must be checked for the
selected integration; a capability card does not establish this checkout's runtime support.
No new environment/provenance proof machinery or whole-system qualification is required
to create these documents.

## Target contracts and scientific boundaries

**Proposed:** compatible allocation/preparation may survive while new physical trials receive
fresh native IVP validity and independent original assessment. Changed state/rate, parameters,
mode, schedule/anchor, provider/branch, numerical realization, demand and execution scope
have distinct invalidation effects. Dimensions alone establish none of these compatibilities.
Each operation consumes one complete current premise set; product upgrades publish only after
all required work succeeds and retain an earlier usable product when stronger work fails.

Libraries continue to own integration, nonlinear correction, factorization and optimization
iteration/globalization. Workflows own scientific composition, demand, bounds, branch/start
permissions and final assessment. Native Limit, partial trajectory, derivative failure and
physical invalidity remain distinct; no failed/limited attempt is relabeled converged.
The practical physical 0.001 endpoint and original accuracy requirements remain unchanged.
There is no global tightening, implicit integration loosening or custom Newton polish.

Every trial evaluation retains its domain, branch, provider, finite-value and stop checks.
Only assurances whose complete premises are unchanged may be hoisted. Owner-thread native
state, callback lifetimes, private mutable scratch, panic containment, cancellation/drain,
checkpoint bounds and total accepted-step/work permissions survive all optimizations.
Retained capacity is not retained multistep/factor/adjoint history. A new mode or observation
channel should alter its own layout/demand preparation, not unrelated scheduling or solver
policy; unsupported native combinations still refuse before effects.

## E01 — Compatible complete IDAS session reset

**Proposed; AF14, I16; accepted RC10 investigation.** Split reusable native storage from
per-invocation borrowed oracle/scope binding without extending a borrow beyond its call.
Consume the prepared all-mode formulation/layout, native method and sensitivity/corrector
shape, quadrature/adjoint demand, admitted capacities and current physical trial. Produce
one owner-affine session rebound only when that complete storage compatibility holds.

The new-trial reset must establish time, state and rates; parameters and scheduled maps;
mode/event/root/sign state; consistent initial conditions and initial-sensitivity terms;
forward axes and corrector state; quadrature values/offsets and quadrature sensitivities;
adjoints/backward problems and checkpoints; linear setup validity; local-error control;
counters, progress generation, budgets and failure latches. Reset histories/counters without
renewing enclosing work permission or preserving prior failure flags. Settings removed by
the next request must not survive. If mutation fails or its result is uncertain, destroy
that owner before another trial; an incompatible shape uses the ordinary fresh path.

I16 compares fresh construction with compatible allocation-only reuse and any separately
qualified fixed-pattern mode. Generic `IDAReInit` does not establish KLU symbolic retention:
its generic linear initialization resets first-factorization. Do not label it retained
factorization or use it to skip necessary native setup. Inspect actual reset contracts and
run success–failure–success and cancellation sequences across changed parameters, mode,
anchors, forward/adjoint/quadrature selections and initialization failures. Compare to fresh
sessions with analytic ODE/index-1 DAE solutions and native lifecycle observations.

Stop after the supported complete reset subset and refusal/discard behavior are established,
or a concrete native lifecycle gap prevents safe reuse. Adopt only that subset through the
accepted RC10 route and any needed binding/design decision; otherwise retain fresh sessions
and local E02 improvements. A demonstrated fitting native reset or repeated allocation cost
is the no-adoption revisit trigger. Conditional implementation migrates public dynamics,
integrated experiment, fitting and shooting consumers of the same owner, without changing
their scientific route. Retire redundant compatible allocation paths and their obsolete
fixtures after replacement tests pass; incompatible rebuild remains necessary.

## E02 — Borrowed callback scratch, current products and anchored CSC

**Proposed; AF14.** Consume checked native vector extents, the current bound dynamic oracle,
function/demand and prepared all-mode support. Borrow native input slices for the synchronous
callback where alias/lifetime rules permit; retain private direction/product/output scratch
where mutation is required. Do not retain foreign slices, borrow across a native call that
can invalidate storage, or let two concurrent windows share mutable callback state.

Extend the existing pure-function cache and demand upgrades. Its premises include function,
time, state, full effective parameters/schedule, mode, formulation/realization, current
provider/branch binding, execution scope and the achieved derivative/product coverage. A
value-valid product need not be derivative-valid. A selected diagonal cannot impersonate a
full Jacobian, and provider/effectful functions cannot acquire purity from a cache label.
Failed upgrades leave earlier valid products intact without reusing partially mutated
scratch. No new parallel weak-key cache is introduced.

Prepare anchored-window/all-mode CSC support and canonical scatter with the immutable
formulation where it is invariant; refill only current coefficients. Include mass/shift,
initial and scheduled effects, transpose/second-order blocks and explicit structural zeros
where consumed. A changed mode uses the prepared conservative union or invalidates the
layout; a partial outside admitted support remains a contract failure.

Migrate residual/JVP/Jacobian/sensitivity/quadrature/adjoint callback consumers with their
actual demand and ownership. Retire callback copies and repeated symbolic/scatter work only
where the new path covers their safety/meaning. Targeted acceptance independently checks
analytic derivatives and CSC matvecs, changed parameter/mode/branch/realization misses,
value-to-First upgrades and failed upgrades, selected/full products, sign/finite/trial guards
and repeated callbacks. Observe actual avoided allocation/symbolic work; numerical equivalence
and untouched failure order matter more than returned buffer shapes.

## E03 — Bounded independent fixed-trial anchored windows

**Proposed; AF15.** Consume actual private native workers with valid scratch/layout,
capacity, owner affinity and joined drain, coordinated with
[48f](48f-scheduling-resources-and-result-completion.md). Fresh per-window sessions satisfy
that contract; consume E01 retained allocation/E02 optimizations only if delivered and
qualified. Declining I16 reuse does not block parallel windows. Existing admission suffices;
F04/I28 immutable admission sharing and a new pool are not prerequisites.
At one complete multiple-shooting trial,
the optimizer supplies each window's anchor. Windows are independent integrations at that
trial; continuity rows couple their returned products. Ordinary single shooting and initial
anchor construction that chains the previous endpoint remain causal.

Replace the one serial trial WindowWorker with a finite admitted set of private owner-affine
workers. Preflight all required windows, derivative routes, live reports/checkpoints and
native exclusions before effects. Dispatch only independent admitted windows, bind the
current invocation/observation/scientific context explicitly, and reduce complete results in
declared window/sample/coordinate order. Do not multiply an outer experiment width by a new
unbounded window pool. Configured, admitted and active widths remain different facts.

Keep transactional Point replacement: failure, cancellation or derivative refusal drains
started native work, publishes no partially complete stronger point and leaves the previous
compatible weaker point usable. No automatic alternate route begins after effects merely
because another window failed. Existing memory/accepted-step permission is not renewed per
retry or reset. Final original assessment remains an independent operation.

Targeted acceptance compares a multi-window fixed trial against analytic trajectories and a
serial execution using the same anchors and demands, including continuity rows and selected
integral/path gradients. Exercise skewed windows, width one/two, a failed middle window,
cancel/deadline, preflight refusal and a failed First upgrade retaining values. Observe
actual overlap/private state and bounded live checkpoints; equality of final NLP optimum
alone does not establish correct window ordering. Delete the replaced serial-only trial
dispatch and its tests while retaining genuinely causal single-shooting/anchor construction.

## E04 — Selected Jacobi demand

**Proposed; N01.** Consume [48c/C02](48c-mathematical-products-and-compiled-execution.md)'s
selected mathematical-product contract and current-provider/guard validity, coordinated
with E02's product coverage. Request only the residual partial diagonal used by Jacobi when
no other current consumer already needs a full matrix. Form the actual shifted Newton
diagonal using differential/algebraic roles and current `cj`; preserve the current finite/
small-diagonal fallback semantics unless separately qualified.

Carry selected-entry/diagonal admission through the dynamic oracle, evaluator/provider and
native setup consumer rather than assembling a full Jacobian and projecting afterwards.
Providers without a fitting selective capability may use an explicit qualified full fallback;
never claim avoided work for that route. All-mode support, selected-entry order, provider
effects, shift updates and original trajectory checks remain intact. E05's stronger demand
must not force a Jacobi-only run to prepare its extra entries.

Targeted acceptance compares preconditioner actions against independently assembled shifted
diagonals for an ODE and coupled index-1 DAE, including changed `cj`, algebraic rows, absent/
tiny entries and fallback. Capture actual derivative requests and preparation on a wide
Jacobi-only model; full/selected cache upgrades and impure providers expose weak demand keys.
Migrate all Jacobi setup callers and delete replaced full-only assembly/column-search code
within the selective scope. No new preconditioner choice is needed to close N01.

## E05 — Coupled shifted DAE preconditioner

**Proposed; I21; accepted RC05 target.** Investigate a library-owned block/ILU/Schur
approximation to the actual coupled shifted operator. In this fixed-mass residual
formulation it is `cj M − f_y`; differential/algebraic cross-coupling remains in the exact
equations and Krylov action. A block-diagonal physical fiction is not a valid replacement
for those equations. Consume current coefficients/support, shift/scaling, differential/
algebraic partition and admissible setup/apply memory. Reuse E02 structure and request only
the product needed by the chosen preconditioner.

Compare direct KLU, current None/Jacobi and eligible coupled block-factor/Schur/ILU routes.
Examine the actual library factor/apply contract, invertibility assumptions and refresh
policy when `cj`, state, mode, parameters, provider or pattern changes. Singular/poorly
conditioned blocks need explicit refusal, admitted fallback or native failure behavior;
never silently invert an invalid factor. Do not infer factor retention from matching sizes
or generic ReInit. Native left/right side, linear accuracy and cancellation remain explicit.

Use an independently specified coupled index-1 DAE with nonzero cross-block terms and known
solution, plus singular algebraic blocks, varying shifts, sparse/fill-heavy cases and changed
premises. Check shifted matvecs and true residuals independently of the approximate factors,
original trajectories/derivatives, complete setup+Krylov+nonlinear work and live memory.
Stop I21 with a qualified coupled capability and its supported premises or a precise
capability/invertibility gap. Adopt conditionally through the RC05 binding/design route;
otherwise retain direct/None/Jacobi and record the observable trigger, such as an eligible
library factorization or reproducible coupled convergence/fill limitation. Conditional
implementation migrates profile/admission/setup/apply/report consumers and removes the
unsupported branch only for the newly qualified capability; E04 remains independently useful.

## E06 — Corrector, initial step, statistics and backward reset

**Proposed; I22/I23.** Preserve native integration and current scientific work budgets.
I22 compares simultaneous/staggered sensitivity correction, explicit/native-selected initial
step and coalesced statistics acquisition for concrete routes. Sensitivity error control,
consistent initialization and complete original accuracy remain part of every comparison;
fewer residual calls alone do not choose a method. Statistics may be collected/serialized
less often only while required intermediate progress, work-limit decisions, partial/failure
evidence and final facts remain available. Ordinary progress semantics follow
[48g](48g-diagnostics-and-review-publication.md), not a new native-local cadence policy.

Vary parameter/function axes, stiffness, initialization difficulty, schedules and rejected
trials. Compare analytic sensitivities/trajectories, native failure and actual total work;
separately verify intermediate cancellation/limit visibility and final counters. Stop I22
when route-specific choices and evidence cadence are qualified or no improvement is found.
Adopt only the supported explicit policy and its consumer migration; otherwise keep current
choices, reopening on a reproducible corrector/setup/statistics bottleneck or native change.
Retire superseded redundant getter/serialization work without deleting consumed progress.

I23 is narrower: inspect whether a pure-ODE backward route can avoid a particular backward
reset/consistent-initialization operation. Compare to current reset handling on smooth ODEs,
observation impulses, scheduled changes and index-1 DAEs. Adjoint jumps and forward segment/
checkpoint history must be correct at every observation; algebraic adjoints and rates may
require consistency even when the forward trajectory is known. A pure-ODE result grants
no permission to remove DAE reset/IC calls or replay history. Stop with an established eligible
subset tested against analytic gradients and an independent finite-difference control, or
an explicit no-adoption result. Adopt only that subset; otherwise retain resets and revisit
on an actual native continuation facility or demonstrated unnecessary ODE reset. Delete
only the replaced subset's calls/tests after the replacement's jump/history tests pass.

## E07 — Demand-specific fitting, curvature and integration accuracy

**Proposed; I24/I25/I26.** Extend existing FitOracle and integrated-experiment products;
do not recreate Gauss–Newton or remove nonlinear constraint curvature. Value-only trial
work prepares no derivatives merely because a later accepted trial might need them.
Upgrade compatible complete trial products transactionally by consumed demand. Complete
trial identity includes free/fixed/tied/scaled coordinates, parameter values, experiment/
revision, mode/schedule/initial mapping, realization/provider/branch, numerical policy,
execution scope and achieved product coverage. Profile fixing can change free-coordinate
and rank structure even when some physical values coincide.

Curvature remains an explicit portfolio: exact residual-objective plus constraint curvature,
Gauss–Newton residual-objective plus required constraint curvature, or admitted native
limited-memory/other approximation. Exact curvature may serve optimization as well as
uncertainty; covariance/rank consumers need their own assumptions and sufficient products.
Reuse compatible assessment allocation/preparation, never the cached producer result in
place of independent final original checks.

Migrate final fit, profile-likelihood, covariance/rank and original-assessment consumers
that reconstruct compatible immutable preparation or allocations. Rebind changed free/fixed
coordinates, ranks, bounds and interpretation before reuse. Targeted controls fix a profile
parameter, change its free-coordinate map without changing vector length, and introduce a
failed derivative upgrade: valid preparation can survive, but final physical assessment
still performs its independent numerical evaluation and never accepts a cached producer result.

I24 compares current constrained Ipopt-GN/exact/limited-memory routes with a fitting
library-owned LM/trust-region residual solver only for actually eligible unconstrained or
bound-only fits. Compare full parameter/constraint/scaling meaning, robust residuals where
supported, derivative demands, termination and final assessment on nonzero-residual and
nonlinear-constraint cases. Native optimization owns iteration/globalization; no custom LM
loop is planned. Stop at a supported portfolio with complete work/quality evidence or a
precise residual-method capability gap. Adopt only the fitting route through its decision
path and migrate fitting/profile/result consumers; otherwise keep current routes and reopen
when an eligible workload/library capability appears. Delete superseded curvature/demand
paths rather than retaining two fitting pipelines.

I25 compares explicit forward, adjoint and any automatic selection policy over parameter,
functional/output and observation axes, events, demanded Jacobian/rank/curvature products,
checkpoint/replay storage and existing experiment-team budget. Parameter count alone is
insufficient. Preserve full physical values, selected derivative identity, direct/initial/
tied/scaled/scheduled/anchor terms and unsupported hybrid refusals. Test analytic gradients
and response Jacobians plus finite differences at nontrivial points. Stop at a bounded
qualified policy or keep explicit selection; a no-adoption result reopens on a demonstrated
axis/checkpoint crossover. Conditional policy implementation must expose actual selection
and refusal, not quietly switch after effects or turn a failed gradient into zeros.

I26 compares fixed current integration accuracy with an explicit inexact trial policy.
State objective/gradient consistency, error-estimation premises, forcing/upgrade points and
required final physical accuracy before evaluating a candidate. Rejected value-only trials
may not need sensitivities; inaccurate values cannot be mixed with an unrelated gradient.
Compare independently refined trajectories/objectives/gradients, optimizer acceptance and
final fresh physical checks near difficult/flat optima and active constraints. Stop with a
bounded policy that preserves final requirements or a no-adoption finding. Adopt only that
explicit supported policy; otherwise retain current accuracy, revisiting when a concrete
consistent error-control capability exists. The physical 0.001 endpoint is neither loosened
nor globally tightened. No inexactness is inferred from native success.

## E08 — Shooting starts and optimizer neutral reuse

**Proposed; I27.** Consume [48d/D03–D04](48d-native-controls-starts-and-recycle-policies.md)'s
truthful submitted-start, effective-option and adapter reset contracts plus current shooting
variable/anchor/bound/constraint maps. Distinguish caller-supplied primal anchors/controls,
compatible optional duals and native algorithm history. Repeated horizon or profile problems
may change free coordinates, node structure, scaling, active bounds or schedule maps; matching
vector dimensions does not qualify a seed or retained optimizer.

Compare fresh optimization, compatible supplied primal and adapter-qualified richer restart/
neutral reuse on perturbed horizons, anchor quality and bounds. Include structure-compatible
numeric changes, equal-sized incompatible maps, option removal and success–failure–success.
Use analytic shooting optima/continuity and independent original trajectories; record actual
submitted data, native state/reset and complete optimizer+integration work. No small barrier
or favorable iteration ratio is assumed.

Stop I27 with a qualified explicit start/reuse subset or an identified map/reset gap. Adopt
only within the adapter's selected reset contract and required design route; otherwise retain
truthful primal starts/fresh native owner and reopen on an actual compatible reset capability
or reproducible setup-dominant repeated-shooting workload. Conditional implementation moves
start preparation, optimizer runner, profile/assessment and result evidence together, retires
replaced cold/restart branches inside the qualified scope and preserves incompatible rebuild.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| E01 | Current native ownership; accepted RC10 boundary; selected reset/binding decision before implementation | I16 IDAS reset outcome and conditional storage reuse; independent fresh/reset ODE/DAE plus changed-mode/demand/failure controls; redundant compatible allocation retirement | open |
| E02 | Current callback/oracle purity and all-mode support; shared selected-product contract where consumed | Borrowed inputs/private scratch, complete current products and anchored CSC; derivative/scatter/guard/cache-upgrade controls and replaced copy/pattern work deletion | open |
| E03 | Actual private workers/scratch, existing shared capacity/affinity/joined drain and current anchors; fresh sessions valid, E01/E02 optimization and F04/new pool not required | Independent fixed-trial windows with ordered transactional reduction; serial/analytic/skew/failure/cancel/preflight tests; serial-only trial dispatcher retirement | open |
| E04 | Delivered 48c/C02 selected-product slice; E02 current-product coverage | Selected Jacobi demand through actual providers/setup; independent shifted-action/trajectory and request-width tests; replaced full-only work deletion | open |
| E05 | Exact shifted operator and actual library contracts; E02 support/product slice; accepted RC05 route | I21 decision and conditional coupled preconditioner; independent coupled/singular/shift-change residual/trajectory tests and capability-consumer migration | open |
| E06 | Current sensitivity, adjoint jump/history, work and progress contracts | I22/I23 outcomes and conditional corrector/h0/statistics/pure-ODE changes; analytic sensitivity/jump/limit controls, obsolete subset retirement | open |
| E07 | Current FitOracle products/constraints; E01/E02 where dynamic reuse is consumed; selected binding decisions before new methods | I24/I25/I26 decisions and conditional fitting portfolio/demand/accuracy changes; independent value/gradient/constraint-curvature/final-assessment controls and old path retirement | open |
| E08 | Working D03/D04 start/reset subset; shooting maps; E01/E02 for retained dynamic workers | I27 outcome and conditional starts/optimizer reuse; independent perturbed-optimum/map/failure/options tests; qualified old restart path retirement | open |

## Integration and verification

**Proposed:** implementers compile affected packages and run meaningful targeted tests through
the repository's native/package routes, with explicit force-validation for Arrow-consuming
closures. Existing IDAS initial/preconditioner tests, runtime shooting tests and fitting
curvature/covariance/profile tests are initial test owners, not evidence that this new scope
is already tested. New independent cases accompany the mechanism they challenge. Mathematical
and native support tests cannot substitute for an independent original candidate assessment.

Coordinate shared backend/oracle surfaces with [48c](48c-mathematical-products-and-compiled-execution.md)
and [48d](48d-native-controls-starts-and-recycle-policies.md). E03 supplies ready independent
work within existing adequate shared admission, owner affinity and joined drain;
[48f](48f-scheduling-resources-and-result-completion.md) owns changes to those common
boundaries. Optional admission sharing, new placement or new pools are not E03 barriers.
New progress
or method facts go through [48g](48g-diagnostics-and-review-publication.md). None of these
changes erase scientific pre/post-stop checks or joined native drain.

[48h](48h-qualification-and-design-publication.md) owns assembled qualification after
functional scope: relevant dynamic/fitting/shooting native and Python journeys, selected
performance comparisons, final hygiene/manual checks and architecture publication. Numeric
speedups require measured workloads and conditions; structural work removal alone does not
claim end-to-end speed. A supported no-adoption inquiry outcome may close its question while
leaving its conditional implementation unnecessary; the coordinator records that distinction.

## Outcome (recorded after implementation)

### What was built

**Proposed; unimplemented.** Document creation establishes no new runtime behavior, passed
product tests or measurements. The operator's recommendation acceptance lives in the
coordinator; unknown numerical winners remain unresolved. At closure record what was built,
a mistake made and corrected, deliberate deviations and exact tested/measured scope. Publish
enduring contracts through the architecture/decision owners and retire resolved planning
material through 48h.

### A mistake made and corrected

**Proposed; unimplemented.** Record the implementation mistake and correction at closure.

### Deviations from the plan, deliberate

**Proposed; unimplemented.** Record deliberate implementation deviations at closure.
