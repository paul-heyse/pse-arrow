---
title: "Plan48d: Native controls, starts and recycle policies"
status: draft
date: 2026-10-20
adrs: []
doc_role: plan
doc_owner: docs/plans/48-execution-performance-and-numerical-efficiency.md
doc_topics: [design, numerical-execution, performance, preparation]
review_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md]
---

# Plan48d: Native controls, starts and recycle policies

## Purpose, authority and baseline

**Proposed:** this companion develops AF12/AF13 corrections and inquiries I12–I20.
The [coordinator](48-execution-performance-and-numerical-efficiency.md) alone owns finding
dispositions, inquiry decisions and RC01–RC12 operator decisions. The operator accepted all
twelve recommendations as written on 2026-10-20; the coordinator records that decision.
Accepted investigation boundaries do not select an unknown policy or authorize product execution.
Packet state is owned by this document's work table. The
[audit](../design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md#native-semantics-dynamic-reuse-and-fitting)
and its [recommendation register](../design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md)
are the source of recommendations, including rejected overgeneralizations.

The focused foundation assessment uses Core 3.4, Efficient Architecture 1.0 and
Process Simulator 1.5 under the repository binding. Existing semantic policy and original
assessment are useful foundations; their native realization must distinguish convergence,
stagnation, effort, submitted starts and retained allocations. This is a bounded source
assessment for planning, not another enclosing numerical review or runtime qualification.

Current source/contract owners examined:

- [Numerical execution](../authoritative_design/sections/numerical-execution.md), blueprint
  §16.6, §17.4, §17.6, §18.3, §18.6, §18.8 and §18.10. The KINSOL absolute-step equivalence
  written in §16.6 is the audit's identified defect, not a premise to reproduce.
- `crates/pse-backend-native/src/kinsol.rs`, `settings/kinsol.rs`, `kinsol/block.rs` and
  `kinsol/linear_work.rs`: policy-derived scaling, existing maximum-step call, native
  linear-method choices and session ownership. KINSOL already owns iteration/globalization.
- `crates/pse-backend-native/src/conic.rs`, `conic/lowering.rs`,
  `execution/pounce_convex.rs` and `settings/pounce_convex.rs`: distinct conic transports,
  native controls, original checks and update restrictions.
- `crates/pse-backend-native/src/execution/`, `ipopt/sequence.rs`, `ipopt/settings.rs`,
  `pounce/retained.rs`, `highs_lifecycle.rs` and
  `crates/pse-ipopt-sys/bridge/pse_ipopt_sequence.cpp`: separate initial primal and warm
  payloads, effective options and adapter-specific retained state. Existing Ipopt
  reoptimization deliberately rebuilds spaces with `warm_start_same_structure=no`.
- `crates/pse-backend-native/src/recycle.rs` and
  `crates/pse-runtime/src/workflow/modeling/recycle_tests.rs`: causal unit calls and semantic
  port exchange; Anderson is already an available native method, not a missing algorithm.

The native-solver-libraries capability routes for KINSOL, Ipopt, cones and sparse factors
inform the alternatives. They do not certify this checkout's linked ABI, features, resets
or numerical results. Candidate bindings must inspect the actual selected release before
claiming support. No source-pin or environment-proof project is a prerequisite to this plan.

## Resulting contracts and preserved meaning

**Proposed:** physical numerical policy continues to define original-coordinate acceptance,
including the practical 0.001 endpoint where it applies. Native working controls express
each method's actual residual, gap, stationarity, local-error or stagnation meaning. There
is no global tolerance tightening. A native Limit remains a Limit; independent physical
assessment may separately pass or fail and must not be replaced by that status. An effort
limit is not evidence that the physical model itself is invalid.

Adapters own their transport, native option application, restart and lifetime. Workflows
consume truthful submitted-start and result contracts without reconstructing solver rules.
Numerical refinement uses admitted library iteration/globalization under a finite explicit
policy, not a project-owned Newton polish. An extension to a new cone family changes its
transport/assessment owner; it does not copy tuning logic into workflow orchestration.

Reusable preparation, a supplied primal/dual seed and native algorithm history are distinct
products with different compatibility premises. Failure after uncertain native mutation
discards the affected owner. Cancellation/deadline checks, joined destruction and existing
thread/global exclusions remain mandatory; retries do not renew enclosing work limits.
No inquiry has a presumed winner or a universal performance ratio.

## D01 — KINSOL control meaning and native policy

**Proposed; AF12a, I12; RC01 accepted target.** The adapter consumes the resolved physical budgets,
normalization, shifted coordinates, native method and enclosing effort limits. Correct the
false absolute physical-step interpretation: KINSOL's scaled-step stagnation denominator
includes both inverse scale and current iterate magnitude. Residual convergence and final
original-coordinate validation remain separate. The maximum-step API already exists;
its norm and selected policy need evaluation rather than a duplicate API call.

The owned output is a truthful effective-control/status contract used by roots,
initialization, implicit solves and recycle runners, with matching diagnostic meaning.
Retire the false equivalence from settings commentary, policy tests and architecture via
the accepted decision route. Preserve sound residual-budget tests; replace the misleading
step assertion with tests of the actual formula and distinct outcome meanings.

I12 compares current conservative controls, the native roundoff-scaled step default and
explicit method-appropriate stagnation/max-step settings. Inspect the actual native formula
and compare shifted/nonzero-bound problems, near-zero versus large coordinates, badly
scaled residuals, accepted starts and stagnation with a physically unacceptable residual.
Use independently evaluated original equations, actual native termination and complete work,
not a check generated by `from_policy`. Stop the inquiry when these cases establish the
control meaning and a bounded supported policy, or expose a concrete unsupported premise.
Adopt a changed policy only with preserved original checks and consistent status evidence;
otherwise keep conservative tuning after correcting its meaning. Reopen on a demonstrated
false stagnation or native-release/scaling change. Any native refinement implementation is
conditional on that outcome and stays inside the same limits. The accepted RC01 correction
must follow its contract amendment route before implementation changes the governed contract.

## D02 — Per-adapter cone controls and bounded refinement

**Proposed; AF12b, I13.** Consume the admitted cone model, original physical budgets,
normalization/recovery maps, coefficient magnitudes and native route. Establish the relation
between Clarabel's relative feasibility control and the original `b−Ax` cone violation,
including SOC/PSD reconstruction and rounding. POUNCE-convex owns a separate inquiry into
its absolute stopping, gated relative arm and rechecks; assigning the same scalar tolerance
does not establish the same defect.

Output is an adapter-owned working-control policy with explicit finite native refinement,
if selected, plus every attempt's actual status and fresh original assessment. Migrate the
conic runners and evidence consumers together. A candidate already violating original
quality remains a failure. Retry admission must cover extra work before effects, retain
cancel/deadline/failure semantics and stop after its declared allowance. No one-retry success
promise or replacement of original independent assessment is permitted.

I13 compares magnitude-aware working controls, bounded native retries and the current route
on small/large coefficients, normalization changes, cone boundaries, ill-conditioned SOCs
and PSDs and mixed physical budgets. Independent analytic cone distances/eigenvalues and
original-unit residuals challenge transport and recovery; compare total attempts and work.
Stop after the transport/rounding domain and finite failure behavior are demonstrated or
boundedly refused. Adopt only the supported per-adapter policy; a no-adoption result keeps
the failure visible and records its trigger, such as a reproducible magnitude/recovery case
outside that domain. Retire superseded scalar mappings and retry callers/tests once replaced.

## D03 — Truthful submitted starts across adapters

**Proposed; AF13.** Given the pipeline's initial coordinates and optional compatible warm
payload, report source seed, transformed primal actually submitted, optional dual/barrier/
basis/working-set data, chosen restart and actual retained native state separately. An absent
optional warm payload does not prove that the initial primal was ignored. An adapter that
does not accept an external iterate must say so; do not manufacture multipliers or a warm
capability to make receipts uniform.

Migrate all adapter receipt producers and their runtime strategy/result/diagnostic consumers
in one coherent change, including KINSOL, Ipopt, POUNCE, Uno, HiGHS, Clarabel and other
admitted adapter paths that produce the shared evidence. Preserve available tangent predictor
behavior; a prediction's provenance, submitted seed and native reuse are separate facts.
Check before-effect refusal, native rejection and post-failure partial evidence. Retire flags
derived solely from optional payload presence and tests encoding that conflation.

Targeted acceptance captures the actual initial callback/native input independently for a
cold primal, predicted primal, compatible optional payload, rejected payload and unsupported
start route. Assert exact original-to-native mapping, no invented data and truthful reuse
after failure. Seeded iteration improvement is not an acceptance requirement.

## D04 — Restart, effective options and neutral resets

**Proposed; I14/I15/I16; RC07/RC10 accepted investigation boundaries.** D03 supplies truthful observations before
comparison. Each adapter consumes the complete current problem/layout, requested options,
seed quality, bounds/classifications, execution scope and prior native-owner state. Classify
an option by actual initialization/update semantics, not its numeric/string name. Produce
the effective current options after full application/reset; absent overrides must not inherit
stale values from the preceding attempt.

I14 compares cold, supplied-primal and supported richer restart strategies, including native
default versus barrier/push choices only where eligible. Perturb feasibility, bounds and
seed quality; include a good and bad seed, no optional payload, option removal and limited-memory
regressions. Stop after a bounded route-specific restart policy and option-reset behavior are
established or a concrete gap is identified. Adopt only supported explicit choices; otherwise
retain current strategies with truthful D03 evidence. Reopen on a demonstrated restart
regression or native option-lifecycle change. No small-barrier default follows from any primal.

I15 compares existing Ipopt space rebuilding with restricted same-structure reuse. Required
premises include reduced maps, fixed/free/equality classifications, bound classes, sparsity,
value-dependent scaling, options and callback lifetime, not just dimensions and patterns.
Changed numeric values can invalidate those premises. Compare with fresh solves, including
equal-dimensional incompatible cases; fail/refuse or rebuild before effects according to
the admitted reuse mode. Stop when a complete reusable subset is qualified or the native
contract prevents it. Adopt conditionally through accepted RC07's design route; otherwise retain `no`, revisiting when
a fitting native reset/scaling facility or demonstrated setup-dominant workload appears.

I16 treats Uno and every additional retained handle independently. Compare fresh construction,
compatible allocation-only reuse and a library-documented neutral reset. Reset option state,
primal/dual/history, counters, failure latches and callback borrows; separate thread-affine
ownership from storage compatibility. Success–failure–success, cancellation, removed options,
changed problem and destruction tests must agree with fresh owners. Stop per adapter at a
qualified reset or explicit unsupported reset. Adopt only through accepted RC10's selected reset/design route where needed;
otherwise reconstruct that adapter and revisit on an actual reset capability or repeated
setup cost worth investigating. IDAS's complete reset is developed in [48e/E01](48e-dynamics-fitting-and-preconditioning.md#e01--compatible-complete-idas-session-reset).
Retire replaced restart branches, stale option partitions and fresh-construction paths only
inside each qualified replacement scope; keep the necessary incompatible rebuild route.

## D05 — Physical grouping and linear setup shape

**Proposed; I17.** Consume [48c/C01](48c-mathematical-products-and-compiled-execution.md)'s
implemented indexed contributor contract: selected blocks reach their coefficients/derivative
contributors without rescanning every parent. That working producer/consumer slice, original
identity mapping and current linearization premises precede grouping comparisons. Semantic
BTF/unit boundaries do not require one native allocation or factorization each.

Compare sequential small blocks, physically grouped blocks and dense/sparse realizations
under valid causal/dependency order. Vary block sizes, density, coupling/fill, repeated
evaluations and setup refresh; no `n≈32` cutoff or universal interval is selected. The output
is a supported explicit grouping/linear-setup policy with bounded factor/scratch lifetime and
native failure recovery. A stale factor never crosses a changed linearization/provider/branch
premise; interval selection cannot waive required refresh.

Stop I17 after representative small/dense/sparse/skewed cases establish an eligible policy
or show that no new grouping pays for its machinery. Adopt only with independent original
solutions/residuals, measured complete work where a speed claim is made, and correct
singular/setup-failure behavior. Otherwise retain explicit existing routes; reopen on a
demonstrated shape or repeated-setup bottleneck. Conditional implementation migrates block
runners and setup consumers, then deletes their replaced grouping/scanning machinery.

## D06 — Indexed recycle exchange and explicit method policy

**Proposed; I18/I19; RC08 accepted investigation boundary.** Prepare semantic-port-to-slot mappings once, preserving
unit/quantity conversions, declared order, tear source/destination identity and diagnostic
spellings. Migrate causal unit inputs/outputs and tear propagation to those mappings, retaining
complete original-state reconstruction and independent final validation. Missing/duplicate
ports, invalid units and unsuccessful sweeps must not publish a partial complete state.
Retire per-sweep semantic map rebuilding/lookups where the replacement covers all consumers.

I18 compares current damping/Anderson choices with explicit history, delay, restart and damping
policies over valid contractions, near-divergence, regime/domain boundaries and poor starts.
Account for history memory, actual map evaluations and native failures; existing Anderson is
the starting capability. Stop with a qualified explicit supported policy or a no-change result.
Only that evidence and the selected policy/design route under accepted RC08 can change the default. No universal history 3–5,
delay one or same-root promise is selected. If no policy is adopted, keep existing defaults
and reopen on a demonstrated regime/quality regression or new acceleration capability.

I19 separately compares a fitting library-owned Wegstein/Broyden capability with existing
native acceleration and derivative-capable Newton tears. Newton requires an admitted causal
composition derivative and root/branch permissions; an opaque map does not acquire derivatives
by naming a method. Inspect actual source contracts, prototype only the missing integration,
and use independent original residual/branch checks on multiple-root and invalid-trial cases.
Stop at a qualified bounded composition or a precise capability/permission gap. Adopt and
migrate only the eligible route after its required decision; otherwise retain native KINSOL
and revisit when a fitting complete library operation or derivative-capable model exists.
Do not introduce project-owned generic fixed-point/quasi-Newton iteration. Indexed exchange
does not wait for either method inquiry to choose a winner.

## D07 — Eligible linear backend and direct/Krylov portfolio

**Proposed; I20.** Compare actual eligible native integrations rather than selecting a solver
name in isolation: current direct choices, candidate sparse-direct backends such as MUMPS/
SPRAL where actually integrable, and Krylov with qualified preconditioners. Consume D05's
indexed/grouped realization where relevant and [48e/E05](48e-dynamics-fitting-and-preconditioning.md#e05--coupled-shifted-dae-preconditioner)
only for coupled DAE preconditioned comparisons. Existing independent direct comparisons
need not wait for that optional capability.

Bind matrix structure/symmetry, pivoting/index width, scaling, linear accuracy, fill/memory,
factor reuse premises, native teams/exclusion and failure semantics. Compare complete
preparation, factor/setup and nonlinear work on eligible density/fill/conditioning shapes;
poor preconditioner quality may overwhelm a cheap apply. Original solutions and independent
linear residuals distinguish numerical improvement from weaker accuracy. Native capabilities
must pass the existing assurance and scheduling boundaries.

Stop I20 once each proposed route has an eligibility conclusion and representative complete
comparison, or a concrete unavailable integration is identified. Adopt only the qualified
portfolio/profile; otherwise retain current choices with a trigger such as a reproducible
fill/memory or convergence limitation and an available candidate integration. A new binding
uses the required decision route before implementation. Migrate selection, native owner,
admission and result consumers together and retire replaced custom dispatch/factor paths;
neither a new global automatic winner nor new threading permission follows from this inquiry.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| D01 | Accepted RC01 target and contract amendment route; current policy/scaling | Correct control meaning; I12 decision and conditional native policy; independent shifted/large/small/stagnation cases and retirement of false step equivalence | open |
| D02 | Current conic transport and original assessment; explicit finite work permission | I13 per-adapter policy and conditional refinement; independent SOC/PSD/magnitude/recovery checks, bounded failure and replacement deletion | open |
| D03 | Current initial/warm producer and every receipt consumer | Truthful submitted starts through adapter/runtime evidence; captured native-input cases and deletion of conflated flags/tests | open |
| D04 | Working D03; actual per-adapter lifecycle; selected design/reset routes under accepted RC07/RC10 | I14/I15/I16 outcomes and conditional restart/reset implementation; fresh-versus-retained and failure/option-removal controls, replaced-path retirement | open |
| D05 | Working 48c/C01 indexed contributor slice and complete linearization mapping | I17 outcome and conditional grouping/setup implementation; independent dense/sparse/singular/changed-premise checks and old scan/group retirement | open |
| D06 | Current causal model, port/tear contracts; D01 control meaning; selected policy/design route under accepted RC08 before a default change | Indexed exchange plus I18/I19 decisions and conditional policies; semantic-port/domain/branch/failure checks, no duplicate iteration path | open |
| D07 | Actual backend eligibility; D05 for grouping comparisons; E05 only for coupled-preconditioned route; decision before new binding | I20 outcome and conditional portfolio; independent residual/solution and setup/fill/failure controls, replacement deletion | open |

## Integration and verification

**Proposed:** implementation packets compile the affected package and run focused tests through
the repository's package/native routes with explicit force-validation where Arrow is consumed.
Existing control, conic, update, recycle and native profile tests are starting owners; add
the independent cases above in the same replacement change. Expected values derived solely
from the new scaling or mapping are insufficient. Each replacement closes only after its
consumers move and obsolete implementation/callers/tests/fixtures are deleted.

[48f](48f-scheduling-resources-and-result-completion.md) owns composed resource admission and
completion; D04/D07 supply actual reset/thread/backend requirements rather than introducing
a second pool. [48g](48g-diagnostics-and-review-publication.md) consumes truthful control,
start/reuse and work facts without changing scientific acceptance. Coordinate shared
`pse-backend-native` edits with [48e](48e-dynamics-fitting-and-preconditioning.md).
[48h](48h-qualification-and-design-publication.md) owns assembled integration, Python/native
journeys, measurement and final static/manual qualification after functional scope ends.
No universal speed ratio or iteration-count target is adopted here.

## Outcome (recorded after implementation)

### What was built

**Proposed; unimplemented.** This is a document-only plan. No implementation, product checks
or runtime measurements are established by its creation; operator acceptance is recorded
only by the coordinator. At closure, record what was built, a mistake made and corrected,
deliberate deviations and exact
qualified scope, with evidence links through the coordinator. Enduring amended meaning goes
to its architecture/decision owner through 48h before this plan is retired.

### A mistake made and corrected

**Proposed; unimplemented.** Record the implementation mistake and correction at closure.

### Deviations from the plan, deliberate

**Proposed; unimplemented.** Record deliberate implementation deviations at closure.
