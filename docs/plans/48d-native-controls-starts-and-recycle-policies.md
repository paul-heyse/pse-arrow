---
title: "Plan48d: Native controls, starts and recycle policies"
status: done
date: 2026-10-20
adrs: [ADR-0201, ADR-0204]
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
| D01 | Accepted RC01 target and contract amendment route; current policy/scaling | Correct control meaning; I12 decision and conditional native policy; independent shifted/large/small/stagnation cases and retirement of false step equivalence | done: implemented; focused native control tests passed; selected assembled acceptance recorded in48h |
| D02 | Current conic transport and original assessment; explicit finite work permission | I13 per-adapter policy and conditional refinement; independent SOC/PSD/magnitude/recovery checks, bounded failure and replacement deletion | done: implemented; five refinement controls and existing conic controls passed; selected assembled acceptance recorded in48h |
| D03 | Current initial/warm producer and every receipt consumer | Truthful submitted starts through adapter/runtime evidence; captured native-input cases and deletion of conflated flags/tests | done: implemented; focused actual native-input and receipt controls passed; selected assembled acceptance recorded in48h |
| D04 | Working D03; actual per-adapter lifecycle; selected design/reset routes under accepted RC07/RC10 | I14/I15/I16 outcomes and conditional restart/reset implementation; fresh-versus-retained and failure/option-removal controls, replaced-path retirement | done: adapter-specific eligibility settled; fixed-profile IDAS storage and actual multi-owner acceptance pass in48e/E01; selected assembled acceptance recorded in48h |
| D05 | Working 48c/C01 indexed contributor slice and complete linearization mapping | I17 outcome and conditional grouping/setup implementation; independent dense/sparse/singular/changed-premise checks and old scan/group retirement | done: bounded inquiry tested; retain existing explicit Direct/common-case and BTF owners, no automatic grouping policy; selected assembled acceptance recorded in48h |
| D06 | Current causal model, port/tear contracts; D01 control meaning; selected policy/design route under accepted RC08 before a default change | Indexed exchange plus I18/I19 decisions and conditional policies; semantic-port/domain/branch/failure checks, no duplicate iteration path | done: indexed exchange and retained candidate buffer implemented; focused native and repaired original-reference runtime controls passed; explicit acceleration settings retained; selected assembled acceptance recorded in48h |
| D07 | Actual backend eligibility; D05 for grouping comparisons; E05 only for coupled-preconditioned route; decision before new binding | I20 outcome and conditional portfolio; independent residual/solution and setup/fill/failure controls, replacement deletion | done: existing portfolio qualified in bounded24-solve inquiry; no universal linear/backend default selected; selected assembled acceptance recorded in48h |

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

## Execution checkpoint — 2026-10-20

D01's native-roundoff stagnation target and actual submitted-primal producers are implemented;
original physical budgets are unchanged. The start receipt now separately records whether its
selected prior/sparse seed was actually submitted, rather than treating a cold initial vector as
that receipt's warm seed. Its focused native regression passes; runtime continuation consumers
remain in the queued native scope.

The current native package selection ran57 controls against zero failures:55 passed and two
new exploratory controls failed. The failed cone trial demonstrated that reducing the configured
relative tolerance by the original violation ratio alone can repeat the same native trajectory;
the actual normalized residual and normalization must inform any narrower retry. The failed
Anderson fixture requested history3/5 for a one-coordinate map, correctly refused by the
existing history<=dimension contract. It is being replaced with a six-coordinate independent
map comparison, preserving that production admission rule. These failures are not hidden by the
passing production controls.

I14 retains the existing explicit restart profiles. Perturbed simplex controls cover cold,
good/poor richer seeds, primal-only seeds, changed bounds, seed-carried versus stated barrier,
actual native submission and independent original feasibility/objective. Good seeds took2 native
iterations against cold5/6, primal-only3 and poor richer seeds7–8 in this small case. Stating a
0.001 barrier did not establish a universal advantage. Existing removal/reset and persistent
limited-memory/full-Hessian controls remain required, rather than selecting a global small-barrier
rule from these observations.

I15 retains `warm_start_same_structure=no`: the actual Ipopt3.14.20 `OrigIpoptNLP::Initialize`
and `InitializeStructures` skip ordinary cache/space/value-dependent scaling construction when
same-structure is true. The current structural signature alone cannot establish those numerical
premises. Fresh-versus-retained classification/scaling and option-reset controls pass with the
existing rebuilding policy. A complete native cache/scaling reset and a setup-dominant workload
are the concrete reopening premises, not matching dimensions alone. Uno remains single-use in
the current binding, with required reuse explicitly refused; the additional IDAS reset inquiry
continues in48e.

I13 treats POUNCE-convex separately: its exact pinned `hsde::relative_stop_permitted` permits a
relative arm only above `max_scale*epsilon > native_tol`; it is not Clarabel's always-relative
feasibility test. An actual SOC transport/native/independent-distance probe covers magnitudes1,
1e6/1e9, coefficients1e-6/1/1e6 and equilibration on/off. All18 returned physical distances were
within0.001 (largest approximately6.3e-5 at magnitude1), so these observations do not justify
copying Clarabel tuning into POUNCE. Wider cone/mixed-budget qualification and the Clarabel
working/refinement decision remain active.

The corrected25-test native selection passes against zero failures, including both repaired
inquiry fixtures. I18 retains explicit per-request native controls/current defaults. For six
independent tear coordinates, history0/1/3/5 used10/5/5/5 map calls on the easy contraction;
the slow near-unit map used40(limit)/21/12/10. The domain-limited expansive map failed without
acceleration and succeeded in11/9/8 calls with history1/3/5; damping0.5 sometimes helped the
alternating map and sometimes added work. Actual admitted history storage rose from0 to21408,
30016 and38752 bytes. This is a bounded native capability comparison, not a global policy winner.
The corrected fixture respects history<=dimension. Existing nonlinear/regime/truncation/domain
controls independently preserve raw map errors and original acceptance.

I19 retains native KINSOL map iteration/Anderson. `CausalUnit` supplies ordered port values and
independent original local state; it does not expose an admitted derivative of the complete
causal composition or permission to replace fixed-point branch selection with a different root
method. The exact Pyomo6.10.1 Network sequential decomposition offers Wegstein, but owns a
Python Port/Var/Arc graph with variable propagation, not a callback over this native map.
The current PETSc/diffsol Newton routes consume admitted equation/operator contracts. These
are integration/semantic boundaries, not claims that no other library implements Broyden or
Wegstein. No new graph bridge, fabricated derivative or project-owned iteration is adopted.
Reopen with a concrete whole-map native integration or complete composition derivative/root
permission and a demonstrated limiting case. The indexed port exchange is implemented and its
two focused controls passed independently of that no-adoption decision.

The indexed recycle owner now also retains one precharged tear-output candidate buffer.
`UnitWorker` retains its completed compact trial instead of constructing and cloning an
intermediate semantic map. Every sweep still checks reconstructed original values for
nonfinite/conflicting state; these checks are not merely optional final-report hydration.
The independent original observer remains separate. The retained-buffer control passed in the
native28-test selection. Current runtime consumers mostly passed; the full original-reference
fixture exceeded its old256MiB constructor allowance and now declares384MiB for its actual
overlapping preparation. Its repaired canonical rerun passes in the31-test runtime selection;
no production guard was lowered.

ADR-0204 and blueprint revision222 now specify one Clarabel-only observed-residual refinement
before production implementation. The qualified private probe uses the observed native primal
residual divided by8 times the failed original-quality ratio (and a stricter previous-control
cap), not a ratio adjustment of the requested tolerance alone. Million-scale original SOC
violation falls to approximately5.3e-8 in4+6 iterations; billion-scale native NumericalError
remains an error in6+11 even with a physically acceptable candidate. Production adoption and
five finite retry/failure controls now pass; POUNCE's current distinct policy is retained.

Independent source review also identified a late-stop gap after original-quality/report
publication. The public Clarabel owner now applies the original execution's stop veto on
both its ordinary and refined return, including cancellation from a durable progress tap.
It also cannot skip a native call and read preceding native counters/candidates through a
second conditional stop check. Deterministic ordinary/refined publication-cancellation and
reused-owner pre-stop controls pass in the same native selection.

**Tested, 2026-10-20:** `just unit-native-capability-package pse-backend-native
native-solvers solver,klu,isolation,uno,petsc` with the selected conic/refinement, grouping,
portfolio, recycle, stack, PETSc/SCIP and dynamic controls ran28 tests:28 passed,0 failed,
0 warnings (zero baseline), native-local without a canonical store. This includes five
production refinement controls and two grouping controls. It is not assembled qualification.

**Measured:** the I17 private grouping control covers64 one-core tiny/dense/banded/skewed
separate/grouped populations, cold and replaced numeric data, KLU/dense/GMRES Jacobi/block
factor and setup intervals1/10. Synthetic complete KLU separate→grouped observations at
interval10 were771→133us (tiny),468→235us (dense),531→267us (banded),477→227us (skewed).
Grouping raises resident peak; a cumulative separate allocation sum is not its peak.
Current production already admits a whole-original Direct route and distinct BTF/block
owners. Arbitrary frontier grouping would need complete common branch/start/profile and
remaining-work mappings. Those permission and causal mappings, rather than marginal timing,
bound the retained route: the existing common whole-case Direct capability already permits
grouped execution, while distinct block owners cannot silently share root/start authority.

**Measured:** I20's24 one-core existing MUMPS/SPRAL/Pardiso/POUNCE-FERAL NLP solves cover
dense16/banded64/skewed32 and two numeric generations. Independent equality/stationarity,
inertia and common postsolve backsolve checks pass at unchanged0.001 engineering policy.
Cold ordering, different outer algorithms and unavailable private Ipopt factor RHS/solutions
prevent a universal linear winner. Retain the explicit portfolio and its present defaults.

### H01 scoped D05/D07 completeness handoff

The final bounded register/consumer pass found no additional production gap in these two
inquiries. D05's common original-case `Direct` and conditional `Block` mechanisms are actual
runtime consumers; independently admitted block boundaries retain their own branch, start,
profile and remaining-work meaning. The two `kinsol::grouping_inquiry` controls cover complete
grouping/setup costs, unchanged original roots and singular-attempt recovery. No replaced
production grouping path or unimplemented automatic cutoff is retained. New arbitrary-frontier
coalescing would require a concrete common-permission mapping; the inquiry does not claim
that faster grouped work is unavailable or reject it for small elapsed differences.

D07's `measurement_tests::portfolio_inquiry::bounded_existing_linear_portfolio_preserves_original_kkt_at_standard_accuracy`
exercises the existing MUMPS/SPRAL/Pardiso/POUNCE-FERAL integrations. Their capability/profile
owners remain usable explicitly; the inquiry did not withhold a proposed backend because it
was marginally slower. Complete solve and original residual/KKT checks support this bounded
portfolio, while unavailable private Ipopt factor RHS/solution access prevents a standalone
factor-level comparison claim. E05's newly adopted BBD route owns the dynamic coupled Krylov
consumer separately. No new library binding or universal default follows from these samples;
48h retains assembled qualification.

## Outcome (recorded after implementation)

### What was built

**Implemented and Tested, completed adopted scope (2026-10-20):** the package table records
all delivery/inquiry outcomes. [Plan48h](48h-qualification-and-design-publication.md#final-qualification-accounting)
records completed selected assembled acceptance and scope-end checks. The individual
observations below retain their original conditions and partial-run boundaries; they do
not assert a universal speedup or extend unsupported capabilities. There is no outstanding
authorized package work in this companion.

**Implemented, partial:** native stagnation semantics, truthful submitted-primal/selected-seed
receipts and indexed causal exchange are in source with the focused boundaries above. Bounded
Clarabel refinement is implemented under ADR-0204 and its five production controls pass.
D05/D07 inquiries qualify the existing explicit routes, including the 24 complete
MUMPS/SPRAL/Pardiso/POUNCE-FERAL solves described above. D04's additional IDAS reset
implementation and focused lifecycle evidence are owned by 48e/E01; its final runtime
population acceptance and enclosing qualification remain separate. The coordinator owns current
inquiry dispositions; enduring amended meaning is published through48h before retirement.

### A mistake made and corrected

The first Anderson comparison requested history3/5 for a one-coordinate oracle; existing
admission correctly refused it. The corrected six-coordinate fixture preserves the real
history<=dimension rule and exercises the intended settings. A requested-tolerance-ratio cone
retry also proved ineffective; the actual native residual is required for the selected bounded
working heuristic. Production behavior was not weakened to force either initial fixture to pass.

### Deviations from the plan, deliberate

The grouping inquiry retains the existing common-case Direct owner and independently
admitted BTF blocks rather than silently combining distinct branch/start/budget permissions.
The 24-solve linear portfolio qualifies actual existing backend integrations without selecting
a universal default or claiming access to private Ipopt factor RHS/solutions. These are
settled retained routes, not unimplemented portfolio promises or rejections based on marginal
timing. IDAS reset acceptance remains with 48e/E01 and assembled acceptance with 48h.
