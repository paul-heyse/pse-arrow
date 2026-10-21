---
title: "Plan48e: Dynamics, fitting and preconditioning"
status: done
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
Implementation and focused qualification are underway. This document owns only its E01–E08 packet states.
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

**Implemented / focused Tested; adopted I21 route:** ADR-0208 selected SUNDIALS 7.1.1's
existing BBD approximate preconditioner through explicit `SpgmrBbd` settings. Difference and
retained bands refer to the original state order, with checked widths and retained bands no
wider than difference bands. This is a library-owned band approximation to the coupled shifted
operator, not a physical-block decomposition or custom factor loop. SPGMR uses left
preconditioning and the existing exact JVP; residuals, mass roles and off-band couplings stay
exact. Setup/native singular failures remain failures without a hidden retry.

The wrapper binding, profile/layout admission, native install/statistics and runtime Value/JVP
producer are implemented. [CA02](48-execution-performance-and-numerical-efficiency.md#ca02)
corrected a public sequencing gap: an owned checked admission now charges the finite
replacement population once before Initial source effects and transfers the original
Execution into session preparation. Direct entry still admits; configuration/extent changes
refuse. Public denied-owner/absent/insufficient-memory tests require zero Initial effects and
unchanged caller output, and pass alongside current cancellation and reset controls.
Segment ReInit installs fresh BBD workspace. Independent
retained E01 BBD storage is intentionally excluded until its complete reset is qualified.
Built-in opaque/dependent providers retain early full preparation when exact selected support
is unavailable; genuine full derivative consumers retain their existing complete products.

The original private FERAL full-LU factor probe remains a comparison oracle, not the adopted
approximation. Its factor-level true residual and changed/singular/cancel controls supplement
the actual BBD coupled DAE, exact-action, native-failure, admission and reset controls. The
native checks and authored BBD receiver evidence below qualify the exercised route; they
establish neither a universal faster method nor global trajectory accuracy from local controls.

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

### I25/I26 bounded inquiry decisions

**Tested:** `short_forward_adjoint_checkpoint_inquiry_preserves_the_same_sampled_functional`
compares the same scheduled nonlinear index-1 DAE and weighted sampled functional using
forward responses over four actual integration axes and adjoints over selected one/all four
axes, each at checkpoint spacing 3, 25 and 250. Ordinary local tolerances remain 0.001;
the derivative comparison checks the predicted effect of a 0.1 parameter change against the
same practical 0.001 functional allowance. The source owner's native-local/store-free
`unit-native-capability-package pse-backend-native` inquiry selection passed this control
against the zero-failure baseline (the complete 12-test selection had ten passes and two
separately owned policy-inquiry failures). It retains actual replay/checkpoint/constructor
and evaluation observations; its cold/order-dependent wall times are no crossover policy.

I25 retains explicit `FitDerivatives::{Responses, Gradient}`. Residual LM, response rank,
response covariance and supplied Gauss–Newton/exact products consume response Jacobians;
a scalar adjoint cannot supply those rows. An explicitly requested gradient-only NLP retains
its supported limited-memory route and fresh final forward response/rank assessment.
The comparison establishes compatible derivatives, not a generic axis-only selection law.
A future automatic selector must include the actual functional/output and observation demand,
checkpoint/replay storage and experiment-team envelope, and expose its selection before
effects. Reopen on an actual demand/checkpoint crossover with those complete premises.

**Interface-checked:** I26 keeps the current integration accuracy. The examined IDAS
`local_error` return evidence describes current native state-error weights/estimates and
availability; it does not enclose the weighted FitOracle objective or response-gradient
error. Existing fit demand/upgrade comparison checks preserve complete current products,
but supply no optimizer acceptance forcing function or coupled objective/gradient uncertainty
witness near flat optima or active constraints. An isolated tolerance change would therefore
lack the consistency authority required by this package. No inexact optimizer loop is adopted,
and no unsupported loose-tolerance experiment is treated as scientific evidence. Reopen when
an actual producer supplies a weighted objective/gradient-consistent error estimator with
explicit forcing/upgrade points and unchanged final physical requirements. Value-only trials
and transactional derivative upgrades remain the supported reduction in requested work.

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

**Tested, bounded I27 outcome:** `shooting_primal_start_inquiry_preserves_fresh_owner_and_current_bounds`
and `shooting_equal_size_anchor_maps_are_distinct_start_contexts` pass cold/good/poor anchors,
changed bounds/options, cancellation/refusal followed by fresh success and same-sized distinct
semantic anchor maps at the original engineering policy. Explicit submitted primal data is
truthful; the optimizer remains fresh. The actual shooting owner exposes neither a compatible
richer seed map nor a complete neutral native reset/ownership handoff, so dimensions alone
cannot qualify retained optimizer state. This capability gap, rather than small timing
comparisons, settles the supported subset; reopen on a concrete compatible map/reset owner.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| E01 | Current native ownership; accepted RC10 boundary; selected reset/binding decision before implementation | I16 IDAS reset outcome and conditional storage reuse; independent fresh/reset ODE/DAE plus changed-mode/demand/failure controls; redundant compatible allocation retirement | done: ADR-0205 actual detached job-foreign lease, fixed-profile value/forward storage and nullable callback slot implemented; focused Automatic/Profile3 workspace/current-cancel/initial-boundary controls pass in native26; scheduled DAE/forward-quadrature and fresh current-adjoint/Second controls pass in native8; actual nested profile-chain, parent/team population, finite-pressure fallback, immediate native retirement and exact final job-drain controls pass; supported value/forward subset complete; selected assembled acceptance recorded in48h |
| E02 | Current callback/oracle purity and all-mode support; shared selected-product contract where consumed | Borrowed inputs/private scratch, complete current products and anchored CSC; derivative/scatter/guard/cache-upgrade controls and replaced copy/pattern work deletion | done: callback/cache/prepared CSC implemented; native callback/Anchored/adjoint and runtime cache/shooting controls pass; corrected actual-child Second allowance-refusal and caller-output atomicity control passes its clean exact rerun; selected assembled acceptance recorded in48h |
| E03 | Actual private workers/scratch, existing shared capacity/affinity/joined drain and current anchors; fresh sessions valid, E01/E02 optimization and F04/new pool not required | Independent fixed-trial windows with ordered transactional reduction; serial/analytic/skew/failure/cancel/preflight tests; serial-only trial dispatcher retirement | done: bounded current-owner/persistent-child dispatcher and distinct prepaid per-window coverage implemented; serial/width-two runtime controls and initial-worker optional-native retirement correction pass in the actual current runtime selection; selected assembled acceptance recorded in48h |
| E04 | Delivered 48c/C02 selected-product slice; E02 current-product coverage | Selected Jacobi demand through actual providers/setup; independent shifted-action/trajectory and request-width tests; replaced full-only work deletion | done: compiler/runtime/native selected demand, coupled-owner/opaque fallback, ordinary evidence refusal and wide-model matched physical classification/actual setup-population controls pass; selected assembled acceptance recorded in48h |
| E05 | Exact shifted operator and actual library contracts; E02 support/product slice; accepted RC05 route | I21 decision and conditional coupled preconditioner; independent coupled/singular/shift-change residual/trajectory tests and capability-consumer migration | done: CA02 repaired; public pre-effect BBD admission/refusal, exactly-once charge and current reset/adjoint/authored controls pass in the 27-test repair selection; 48h records selected qualification |
| E06 | Current sensitivity, adjoint jump/history, work and progress contracts | I22/I23 outcomes and conditional corrector/h0/statistics/pure-ODE changes; analytic sensitivity/jump/limit controls, obsolete subset retirement | done: batched statistics, current pure-ODE backward rates and Automatic h0/Profile3 implemented under ADR-0206/0207; focused policy/callback/adjoint/current-document controls pass in native26; current initial-step statistic/reset lifecycle passes in native8; actual runtime integration consumers pass their focused selections; selected assembled acceptance recorded in48h |
| E07 | Current FitOracle products/constraints; E01/E02 where dynamic reuse is consumed; selected binding decisions before new methods | I24/I25/I26 decisions and conditional fitting portfolio/demand/accuracy changes; independent value/gradient/constraint-curvature/final-assessment controls and old path retirement | done: ADR-0209 fitting-owned residual route and actual owner-local neutral handoff implemented; current native/canonical all-target compile is clean; corrected matched operations and real IDAS profile-chain controls pass; persisted-result receipt decoding and known dense-workspace eligibility controls pass; I25/I26 decisions settled below; selected assembled acceptance recorded in48h |
| E08 | Working D03/D04 start/reset subset; shooting maps; E01/E02 for retained dynamic workers | I27 outcome and conditional starts/optimizer reuse; independent perturbed-optimum/map/failure/options tests; qualified old restart path retirement | done: I27 settled on tested explicit primal starts and a fresh optimizer owner; cold/good/poor, changed bounds/options, failure/cancel recovery and equal-sized distinct anchor maps pass; no compatible richer optimizer reset/map contract is exposed, so no retained optimizer is claimed; selected assembled acceptance recorded in48h |

## Implementation checkpoint

E01's narrowed value/forward storage target was published in proposed ADR-0205 and
blueprint §18.8/§18.10 before production changes. The implemented retained owner contains
compatible native allocations, immutable layout/identity and the actual detached job-foreign
lease obtained before allocation. Each call uses a fresh callback context, current parameters
and original execution. Stable nullable user-data indirection covers IDAS's copied callback
pointers; roots, removable options and numerical modules are reapplied. Errors destroy the
owner while the current callback box remains alive through native return/teardown.
Adjoint/backward owners remain fresh, and no symbolic/numeric factor retention is claimed.
Focused native controls passed for current Automatic/Profile3 value/forward reuse, prior
context cancellation and lease replacement, roots/directions including zeros, and stop before
or during Initial source evaluation preventing native construction. The additional
`retained_scheduled_dae_and_forward_quadratures_match_fresh_current_products` control passed:
a scheduled index-one DAE's forward state/output and quadrature derivatives agree with fresh
current products, analytic states meet the unchanged 0.001 empirical envelope, repeated requests
retain the same native pointers, and detached coverage releases on owner drop. The focused
current adjoint/Second control also passes original deadline/work delegation, refusal before
Initial on occupied coverage, Initial cancellation preventing construction, analytic sampled
products and child-only report history. These fresh backward calls do not expand retention.
The single runtime job-to-task-to-native passive-lease lifetime control and corrected
multi-owner runtime population both pass their actual canonical selections.

A subsequent actual-lifetime review found that a refused second retained claimant could take
the fresh path while a different transient native owner still occupied the same foreign
allowance. The single-workspace native passes do not qualify that multi-owner population.
The correction now prepares distinct finite coverage for every actual parent/team IDAS
worker before provider/native effects, including Exact's additional parent population.
Required concurrent active envelopes precede the optional complete resident grant; pressure
shrinks team width before effects or disables retention for the whole population while
preserving distinct active-lane coverage. The immutable preparation's opaque library-metadata
allowance remains separate. The fallible existing coverage query distinguishes unsupported
retention from an allowance occupied by a different resident owner; the latter refuses before
Initial effects instead of allocating a fresh session against occupied capacity. Existing E03 fixed-window coverage already
uses distinct per-window foreign envelopes. Its initial causal worker now discards completed
optional IDAS storage before a wider team claims the original allowance, while preserving the
mathematical worker for serial fallback. Parallel profile dispatch likewise prepares one root
envelope per actual active chain lane before dispatch; nested experiment populations preserve
opaque passive coverage instead of assuming a concrete allocation-token type. Required width
shrinks before effects when coverage cannot fit. Fresh adjoint and second-order consumers now
forward the actual current work owner and original deadline through the same native cores;
these populations remain fresh and their report history remains child-local. Runtime multi-experiment, IDAS profile-chain and parallel-team controls pass. The final
population control verifies twelve nested unique passive owners, required active coverage,
atomic optional-pressure fallback, parent/team Exact populations, immediate native claim
retirement and exact final job-drain equality while retaining the same immutable preparation.
Its forwarding recorder identifies accepted constructor growth as the enclosing job's separate
live admission; that conservative reservation ends at job drain, not at each oracle Drop.


E02's callback-only IDAS inputs now use checked synchronous native borrows that end
before publication. Escaping checkpoints, result samples and session snapshots keep
owned reads. Residual output reuses the owned function values, Jacobi solve reuses the
already admitted Krylov product, and sensitivity output uses its completed product
matrix after all input rates have been read. The numerical guards, callback control
boundaries and accepted scientific work remain owned by their existing paths.

The runtime pure function cache computes candidate values and commits stronger
coverage only after successful derivative work and the post-scope check. An authored
model control forces derivative-snapshot refusal and checks retained compatible Value
coverage, successful retry and changed-point invalidation. This control passes in the actual
canonical runtime selection; the selected-entry coverage extension remains part of E04.

Anchored full-matrix derivatives now require finite preparation through the existing
`prepare_layout` / `use_prepared_layout` route. Production window owners cache both
plain and observed all-mode/function layouts. The removed callback-time First
triplet/sort path is replaced by canonical numeric refill and separately charged,
immutable escaped snapshots; drained snapshots can reuse their owned numeric storage.
Action-shaped products retain their independently prepared inner product paths.
Weighted-Hessian assembly is a separate unchanged product. Native direct derivative
callers migrated; profile/native-support inspection alone needs no matrix preparation.

The coordinator's native combined run exercised 30 IDAS, corrected initial-history
and selected-parameter controls with zero failures, including the three new callback
borrow/alias/refusal controls. The complete 57-test run had two failures in separately
owned conic and KINSOL inquiry probes. A mistaken `anchored_tests` filter omitted the
actual `dynamics::anchored::tests` module. The corrected coordinator run
(`unit-native-capability-package pse-backend-native`, linked KINSOL/Diffsol scope)
subsequently exercised that module, `dynamics::tests::adjoint` and the corrected
root inquiry probes: 25 passed, zero failed against the zero baseline. This
includes prepared Anchored all-mode refill, admission/refusal and snapshot reuse,
and first/second-order adjoint regressions. Actual runtime cache, shooting worker retention,
failed-upgrade and fixed-window controls subsequently pass. The assembled exact-Hessian
control exposed an obsolete failure injection into the coordinator's copied memory field;
the corrected control targets the actual transient execution and checks atomic output refusal.
Its exact rerun passes with zero compiler warnings/errors and zero test failures.
These focused results do not qualify unexercised products.

E03's fixed-trial dispatcher is now in source. Production callback teams are prepared
before the NLP receives its oracle. Width one keeps the current owner and its admitted
scratch; wider teams create one private worker per fixed window on persistent child
threads. Both use one window operation and ordered reduction. A candidate team prepays
resident stack/native/metadata and existing known or opaque constructor owners before
child effects; refused optional widening releases the candidate and selects a smaller
width. Actual `MathService::quiescent_phase` admission authorizes active width, independently
of the outer KKT width, and every started reply drains before Point publication. Each
wave transfers current execution, invocation and observation context. Initial causal
anchors and independent final assessment keep their current-owner lane. New opaque lease and actual width-two/skewed-window/analytic/refused-upgrade controls pass,
including width two while the outer KKT grant is one. Initial native storage is retired before
the wider team claims its original foreign envelope; serial mathematical scratch remains valid.

E04's compiler-issued semantic entry plan and provider-bound numerical path are
implemented by the mathematical/compiler owner. The dynamic preparation bridge uses
that plan inside the existing admitted compiler job, before artifact preparation,
and charges escaping artifact descriptors. A constrained value-only IDAS
Krylov/Jacobi population keeps complete static First support while requesting Value,
RHS JVP and exact RHS row/state entries. Full sensitivity, adjoint, shooting and
stronger transient consumers retain their ordinary full preparation. Actual required
provider sources conservatively choose the original full plan before numeric artifact
compilation when exact product eligibility is not established; no numerical retry
switches routes after effects. The native raw diagonal consumer now forms the unchanged
shifted inverse using existing private scratch, while the runtime records selected,
Value and full coverage independently. Selected-only runtime layouts retain static
support without a full numeric matrix template. Borrowed workers and Anchored wrappers
forward raw diagonal provenance. Full fallback carries its existing source Evaluation
so the native owner preserves prepared-support refusal before using the diagonal.
Observed quadrature-state diagonals are structural zeros, with current flux Value
checks retained in RHS-to-flux order through one separately charged, reused candidate.
Native raw diagonal/full-fallback/observed-flux controls passed their focused selection.
The actual authored 16-state selected ODE pipeline reached Value plus 16 entries, complete
off-diagonal static support and separate numerical coverage. The coupled childless implicit
owner versus opaque-owner preflight control passed in the canonical runtime selection.
The wide-model raw native run completed but exceeded the independent physical 0.001 budget
at one of 48 sample/coordinate points. A same-source/profile full-KLU reference reproduced
that classification: at rate16 and time0.1, selected error was0.0010280995101038734 and full
KLU error was0.001028256629203493. The other47 points met the original budget. No tolerance,
rate or physical acceptance budget was changed. The corrected control compares each original
physical classification and selected/full agreement under that allowance, explicitly retaining
the original-invalid observation. Those assertions passed in the subsequent canonical32-test
selection, which had29 passes and3 failures against zero. The wide-model control then failed
its observation assertion: the default fixture's128 aggregate metric slots could not retain
the actual performing population. Its correction reads actual producer rows and
linked operations under an explicit fixture diagnostic budget, requires positive selected
setup observations to equal the independently reported complete native setup population, and
only then accepts absent/zero full-setup work. Positive selected-entry work and no full numeric
worker snapshots remain required. **Tested:** the corrected control passed in the subsequent
canonical runtime six-control selection (5 passed,1 separately owned population-control failure,
zero compiler warnings/errors; run `59360525-7375-404a-b7f7-fda825f6577f`). Its actual
`native.callback` producer emitted10 selected setups, exactly matching all10 native setup
evaluations, with zero full setups and positive selected-entry work. The operation retained240
aggregate metrics with no curtailed aggregate coverage, refused items or dropped updates;
optional detail-ring loss remained explicitly separate. Only the fixture diagnostic policy
changed; no production metric budget or counter emission was altered.

A separate ordinary-run control requests a passive Time-valued view of the same relaxation
state, an explicit 0.001s Estimated/RequireSatisfied goal and refinement. It preserves the
initial native controls and independently checks the original analytic error. Current point
arithmetic evidence deliberately refuses state-derived outputs supplied by generated implicit
rate providers: selected First capability is not arithmetic-uncertainty evidence. The control
therefore expects Unresolved/EvaluatorUncertainty, no accepted result and no guessed refinement.
It passed in that canonical selection with zero compiler warnings. This is a current accuracy-evidence support limit, not an
unimplemented E04 requirement or a global-error certificate inferred from native completion.

E05's authored BBD receiver initially exposed a production layout assumption: every Value-only
RHS was treated as a diagonal consumer. Empty exact-entry plans are now valid Value/JVP
requests; only a nonempty diagonal request must satisfy the complete row/state extent and
identity checks. The canonical four-control diagnostic selection passed
`kernel_bbd_prepares_value_and_exact_jvp_without_full_numeric_jacobian`; its three separately
owned/diagnostic failures remain failures against zero. It executes actual prepared simulation,
exact off-band action and native BBD statistics without full numerical Jacobian preparation.
The subsequently strengthened empty-plan diagonal refusal/output-atomicity assertion passes
in the actual 32-control runtime selection; no extra numerical derivative coverage is inferred.

**Measured, E05 bounded complete-operation inquiry:** one native-local one-core process
evaluated68 coupled populations at2/8/16 states, coefficient/scaling changes, sparse/dense
support and explicit Configured initial-step policy. Each complete observation includes oracle
construction, checked preparation, consistent initialization, four samples, original checks,
statistics and native detach/drop. Relative and differential absolute controls remain0.001;
algebraic absolute controls remain0.001 times the declared scaling coefficient. The independent analytic
trajectory envelope is0.01 and normalized algebraic closure0.001. These are distinct tests,
not a claimed global0.001 certificate. Complete elapsed ranges in microseconds were:

| States | KLU | None | Jacobi | Private full LU | BBD narrow | BBD complete support |
| --- | --- | --- | --- | --- | --- | --- |
| 2 | 44.53–155.87 | 45.81–75.31 | 51.67–60.93 | 78.55–142.87 | 50.25–65.16 | same supported band as narrow |
| 8 | 52.57–69.85 | 48.56–54.04 | 59.02–63.55 | 113.09–181.05 | 81.98–84.29 | 74.24–77.67 |
| 16 | 86.84–141.29 | 58.48–88.25 | 80.04–111.78 | 272.79–282.14 | 126.95–136.95 | 121.06–128.21 |

Complete-support BBD used5 linear iterations,15 setups and27 applications versus None's34–35
iterations and Jacobi's18–21. Its source calls were68/158/278; narrow larger bands used18–21
iterations and156–165/268–285 calls. Maximum complete-support analytic error was about0.000461;
the narrow larger cases reached about0.002758 within that inquiry's original0.01 envelope.
The first-principles coupling/work reduction supports the explicit adopted route despite tiny
elapsed overhead; no universal default, automatic ordering or factor-retention permission is
selected from these observations.

The final bounded E05 register/consumer pass found no additional missing production binding
or superseded implementation path: profile, layout/precharge, exact residual/JVP, BBD install/
refresh, metrics and ordinary simulation producer all consume the new explicit route. Native
singular/admission/shift/reset controls and the authored receiver exercise those boundaries.
The private full-LU probe remains a meaningful independent comparison, not a second production
preconditioner. The current support limits and tested strengthened receiver assertions above
retain their own evidence boundaries; 48h owns assembled qualification.

E06 uses the native `IDAGetIntegratorStats` batch for its exact overlapping scalar
fields while retaining separate sensitivity, quadrature and other counters. Safe-return
publication, lossless scalar encoding, source generations and accepted-step meanings
remain unchanged. The batch-versus-individual control passed at initial, consistent-IC,
advanced, restarted, limited-return and null-memory error boundaries in the coordinator's
native-local/store-free `unit-native-capability-package pse-backend-native` selected
native-solvers run (28 passed, zero failed against the zero baseline). That same focused
selection passed all five new E04/E06 native controls, including raw selected setup,
explicit full fallback/prepared support and current observed-flux validity. The actual
runtime selected preparation and fitting/shooting boundaries remain separate. Current native
policy controls now qualify the bounded pure-ODE first-order backward rate initialization,
Automatic native initial-step selection, explicit Configured alternative and Profile3 receiving
contract. DAE and second-order adjoints retain native consistent initialization. Source-bound
inquiry conditions and broader runtime qualification remain distinct from these focused passes.

E07 uses the actual fitting-owned Automatic/NativeNlp/ResidualLm route and literal residual
termination/work document. Its single dense matrix extent authority now also gates actual
method selection and early First-only preparation against the effective known foreign
allowance; Automatic retains the NLP route when the dense workspace cannot fit, while
explicit LM refuses before source preparation. Execution rechecks its current actual grant.
The same final original assessment handles both alternatives.
Neutral scratch lives inside the actual adapter owner or one actual profile-chain child:
callback Drop destroys providers and detaches controls before CaseScratch is returned to a
finite charged slot; reset and new providers/current scope precede the next evaluation.
Compatibility uses the exact consumed CaseAssembly and experiment role, not a whole FitProblem
or a prior free-map/scope identity. Dynamic frames, product caches, wrapper layouts and native
workspaces are fresh on the new oracle. Parallel experiment children remain fresh because no
complete neutral child-return contract is available. Current source compiles in the runtime
native all-target check. The first actual canonical runtime selection passed real LM method,
original quality/covariance/profile output, callback refusal, neutral steady/transient scratch,
changed pin maps, derivative refusal and the single submitted-lease lifetime control. Its two
LM comparison controls failed on the same extreme-scaled native LimitedMemory objective-gap
assumption. Independent centered weighted normal equations show that normalized .001
stationarity does not promise a .001 objective gap in those coordinates. The corrected controls
retain LM's independent original optimum check and explicitly check each native candidate's
physical/scaled gradient and exact quadratic objective-gap identity. These corrections, the persisted/reopened public LM result control and the multi-owner
coverage control required two fixture corrections in the actual 32-control runtime selection.
The corrected complete matched-operation controls pass for ordinary and extreme scales,
near-collinearity and a positive-domain nonlinear response at unchanged accuracy. Real IDAS
parallel profile chains and current initial-worker retirement also pass. The stored-result
control must decode the canonical receipt wrapper before its inner runtime strategy; the
pressure control must establish its baseline after normal idle compiler/native reclamation.
Both corrections were exercised by the next six-control selection: persisted method/status/
profile receiving and known dense-workspace eligibility passed, as did matched complete LM
operations and the real-method/fresh-assessment control. The population control passed nested
12-owner uniqueness/charge/lifetime and forced-pressure fresh fallback, then found a 33,984-byte
post-evaluation global-pool delta. The actual forwarding recorder establishes that increment
as accepted constructor reservation growth retained by the current job. Immediate native-claim
retirement and exact original post-job baseline both pass with the same preparation alive;
no byte tolerance, rebased baseline or numerical-policy change is introduced.

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

**Implemented and Tested, completed adopted scope (2026-10-21):** the independent
[Completion audit](48-execution-performance-and-numerical-efficiency.md#completion-audit)
identified CA02 in E05; the repair and settling controls now pass with adjacent checks
in the 27-test selection. [48h](48h-qualification-and-design-publication.md#completion-audit-repair-qualification)
records actual repair acceptance and all three full continuing Clippy modes. Earlier
observations retain their original conditions and partial-run boundaries; no outstanding
accepted package obligation remains in this companion.

**Implemented:** the scoped E01–E08 targets and supported inquiry outcomes are recorded above.
Fixed-profile IDAS value/forward storage has separate current contexts and actual detached
foreign coverage; backward/Second/BBD populations remain fresh. Actual fitting experiment,
Exact parent/team and profile-chain populations use distinct admitted coverage. Callback
borrows/private scratch, prepared Anchored refill, transactional current products, persistent
fixed-window child owners, selected Jacobi and explicit BBD demand remove the replaced work.
Fitting consumes its qualified library residual method and owner-local neutral mathematical
scratch, with honest method/termination/result identity and unchanged independent final checks.
I25/I26 retain explicit derivative selection and fixed current accuracy for the concrete
meaning gaps below. I27 retains tested explicit primal starts and fresh optimizer ownership.

**Tested, focused:** native-local/store-free
`unit-native-capability-package pse-backend-native pse-backend-native/native-solvers
solver,klu,isolation,uno,petsc` with workspace/BBD/policy/callback/adjoint/current-document
filters passed 26/26 against zero (run 664ca1fa-431c-4241-869f-9f589e129c08). The later
workspace/current-adjoint/statistics/h0 selection passed 8/8 with zero compiler warnings/errors
and zero test failures (run a8bec5e4-4e88-4b08-b072-0bc8a238dcc9). It includes scheduled
DAE/forward-quadrature retained/fresh products and current fresh adjoint/Second work, deadline,
occupied-coverage and Initial cancellation boundaries. The existing zero-root replacement
control emits IDAS's expected identically-zero-root diagnostic; compiler-clean does not mean
warning-free native output.

**Tested, focused:** canonical-runtime
`unit-native-package pse-runtime` in the selected native-solvers/canonical-tests closure passed
the complete matched LM, real method/fresh assessment, persisted/reopened LM receipt,
known dense-workspace eligibility and selected-Jacobi controls in the six-control selection
(run 59360525-7375-404a-b7f7-fda825f6577f; five passed, one population assertion failed against
zero). The subsequent final twelve-control selection ran 11 passes and one failure against
zero (run ea5cc2d3-e00e-4e0a-9ec6-f8d0ee59107d;
`/tmp/pse48-runtime-final-twelve-controls.log`). Its passing scope includes actual native
population ownership/pressure/drain, shooting/simultaneous equivalence and continuity,
weighted loss/curvature, regression covariance, profile likelihood, one fresh independent
assessment, parent-cancel supervisor drain and both actual same-worker publication controls.
The population forwarding recorder observes exact cumulative accepted constructor bytes
33984/67968/101952/169920 for serial-GN/serial-Exact/team-GN/team-Exact, while native claims
release immediately and the final original job baseline matches with identical preparation
still alive. The one failure is an obsolete test injection into the coordinator's memory
field instead of the actual transient owner's finite allowance. That test-only correction
also checks caller-output atomicity. Its exact canonical rerun passes 1/1 in 0.907s with zero
compiler warnings/errors (run 62d8b84d-5aa2-4db6-9b56-96d3c8a5e1dd;
`/tmp/pse48-runtime-curvature-final-control.log`). The preceding twelve-control run compiled
with one test-only deprecated atomic-method warning; the exact method rename is applied and
this rerun compiles clean. Final static qualification remains with 48h. No warning baseline
is accepted.

**Measured, bounded complete operations:**
`fitting_residual_production_matches_native_complete_operations_at_original_accuracy`
compares actual LM and Ipopt Gauss–Newton/exact/limited-memory preparation, solve, fresh
quality, covariance/Wald and prediction work for ordinary/extreme scales, near-collinearity
and a positive-domain nonlinear response, in both route orders at unchanged 0.001. Actual LM
complete times are 2.058–3.690ms across those cases, with literal residual/Jacobian counts
3/2 for linear/domain and 10/9 near-collinear. This supports the exercised route and removed
work; it is neither a general speedup nor an optimizer-iteration comparison. Native scaled
stationarity alone does not promise a physical objective-gap bound in extreme coordinates;
independent analytic optimum, gradient scaling and exact quadratic-gap controls retain that
boundary without tightening the policy.

**Tested, bounded inquiry:** I25's scheduled nonlinear DAE sampled-functional control compares
forward four-axis responses with selected one/all-axis adjoints across checkpoint spacing
3/25/250 at unchanged 0.001; all seven actual routes passed. **Interface-checked:** I26 keeps
fixed accuracy because the examined producers expose no weighted objective/response-gradient
consistent estimator or forcing witness. These settle their inquiry questions without a
generic automatic selector or inexact optimizer. All evidence remains scoped; 48h owns the
assembled Python/native/static/governance qualification and aggregate result.

### A mistake made and corrected

The first authored coupled-provider fixture assumed every provider must take the full
Jacobian fallback. The actual childless implicit owner now qualifies exact selected First
products, so that fixture is corrected to assert selected execution and separately tests an
actual opaque owner that cannot qualify preflight. The width-two dispatcher fixture also
underestimated retained per-window opaque storage; its finite pool now covers the real
population while the production transactional shrink/refusal rule remains unchanged.
Those corrected runtime controls pass. The final population fixture also originally expected
oracle Drop to release the enclosing job's conservative constructor reservation. Forwarding
the actual constructor admission establishes its exact growth; the corrected boundary is
immediate native retirement plus exact final job drain with the original preparation alive.
This changes no production accounting or leak allowance.

### Deviations from the plan, deliberate

I25 keeps explicit derivative selection rather than introducing an axis-only automatic
selector; I26 adopts no inexact optimizer policy without a consistent objective/gradient
error producer. These are concrete capability/meaning limits, not timing-based rejections.
Sound demand-specific value/derivative work removal continues within those boundaries.
