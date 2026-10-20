---
title: "Plan47b: Native diagnostics and causal attribution"
status: complete
date: 2026-10-18
adrs: []
review_sources: [docs/design_review/reviews/design_review_pipeline-diagnostics-and-review-artifacts_2026-10-18.md]
doc_role: plan
doc_topics: [design, diagnostics, libraries, numerical-execution, performance]
doc_owner: docs/plans/47-pipeline-diagnostics-and-review-artifacts.md
---

# Plan47b: Native diagnostics and causal attribution

## Target and foundation

This companion owns the native diagnostic catalogue and selective extension inquiries in
review §5 and §10. The [coordinator](47-pipeline-diagnostics-and-review-artifacts.md)
owns shared decisions and finding dispositions. This document owns B01–B06 progress;
it does not create additional findings or a second rule-decision table.

The intended capability is to distinguish extra executed work, changed numerical progress,
recovery/reconstruction work and increased inclusive cost using facts from the owners that
performed the work. Native state stays native-owned. A passive reader consumes copied facts;
it never calls a foreign getter, reconstructs an integration, estimates a condition number or
evaluates the physical model to fill an observational gap. Libraries retain their algorithms.
New observations are selected because they can discriminate a concrete unresolved cause,
not because every solver should emit the same population.

**Interface-checked baseline:** the review and its [native capability inquiry](../design_review/evidence/pipeline-diagnostics-and-review-artifacts-2026-10-18/native-capabilities.md)
describe the inspected production source at `e9d24396c`; the documentary checkpoint is
`6524e8591`. Relevant performing owners include native adapter/callback/return paths,
`pse-backend-native::observation`, POUNCE `observed`/`records`, dynamics `idas`/`linear`,
`pse-math::assembly_observation`, provider observations, and runtime source-map and
interpretation owners. Direct authoring inspection confirms the POUNCE unknown-inner-work
declarations, current Uno checkpoint semantics and the static source map's explicit missing
native dual/stationarity pullback. No new native runs, API probes or overhead measurements
were performed to create this document.

The foundations already separate issued/completed/failed work, cumulative and retained
populations, generations, availability, optional observation and scientific outcomes. Retain
that separation. Existing telemetry is the first input to analysis; the conditional candidates
below do not imply that existing class-specific owners are defective or need replacement.
The shared frozen/descriptive contract belongs to [47a](47a-frozen-evidence-and-review-context.md),
analysis/recording to [47c](47c-analysis-and-recording.md), publication to
[47d](47d-offline-review-pack-publication.md), CLI composition to
[47e](47e-cli-review-workflows.md), and assembled qualification to
[47f](47f-qualification-and-design-publication.md).

## Consumed meanings and preservation constraints

An observation retains its operation/native attempt, producer/generation/event identity,
backend/profile, family, actual phase, coordinate/norm convention, population/reset scope,
availability and collection coverage. Correlate through actual parent/path or source references.
Absent parentage, unsupported mappings and uncollected history remain explicit limitations.
Expose compact declarations sufficient for interpretation in the descriptive review context;
actual version/fork/options are useful recorded facts, not environment/provenance proof.

Native accepted states, termination-check trials, configured upper bounds, completed work
and original physical qualification are different meanings. Preserve ordinary **0.001** physical
engineering accuracy where selected by the scientific policy. Native tolerances, derivative
residual checks, local error weights, rank thresholds and KKT criteria retain their existing
independent meanings. Do not tighten stopping rules to make diagnostic trajectories richer.

Observe at documented owner-thread callbacks or safe returns while the native handle and
buffers are live. Keep callback storage stable, contain foreign-boundary exceptions/panics,
detach before destruction and retain stop/drain ownership. Selected capture failure/loss cannot
change a numerical outcome. Selection precedes expensive copying; no callback performs query,
export, blocking writer work or hidden numerical diagnosis. Admit vector scratch at its actual
owner and release it with that selection/attempt. A bounded retained prefix is not complete
history; count overflow/reset or absent sources cannot become a fabricated delta.

Native coordinates require the actual transform to original variables/rows and physical
units. Submitted-NLP unscaled coordinates, Schur blocks, cone slots, presolved rows, integrated
states and authored occurrences are not interchangeable. Preserve supplied maps/scales and
explicit missing pullbacks; never match by nearest index or shared expression body alone.

## Work packages

| Package | Delivered capability | Prerequisites | State |
| --- | --- | --- | --- |
| B01 | Existing class-specific telemetry is consumed with truthful populations, coordinates and applicability | Existing native/evaluator/observation owners; review catalogue | complete |
| B02 | Bounded IDAS local-error inquiry and qualified selected observation if adoption criteria hold | B01 IDAS semantics; actual safe-return and state-map evidence | complete |
| B03 | Selected FERAL factor snapshot inquiry and qualified projection if adoption criteria hold | B01 factor/path identity and completed-factor lifecycle | complete |
| B04 | Genuine POUNCE inner-retry/refinement inquiry and qualified owner hooks if adoption criteria hold | B01 outer/path semantics; maintained fork loop inspection; B03 only if its fields are consumed | complete |
| B05 | Accurate Uno checkpoint interpretation and bounded Clarabel/Uno private-detail inquiry | B01 native checkpoints; concrete remaining question before any private-detail hook | complete |
| B06 | Selected native evidence reaches shared interpretation, frozen review and publication without changing science | Working relevant B01–B05 deliveries; 47a frozen/context contracts and 47c selected-analysis/recording contracts | complete |

The package order is not a blanket solver campaign. B02–B05 can investigate independently
after the relevant B01 facts are understood. B06 integrates each adopted population with its
actual consumers as it becomes usable; it must not postpone necessary producer-to-consumer
migration to a final inventory pass. The final assembled journeys belong to 47f.

<a id="b01"></a>

### B01 — Consume existing class-specific work before adding hooks

Start with the question, selected work/interval and retained source records. Resolve existing
producer declarations and actual applicability at the adapter/evaluator owner, then make them
available to the shared analysis in 47c and descriptive context in 47a. Reuse existing record
families and projections where they carry the required meaning. Do not add a universal solver
telemetry object or invent a new hook merely because a convenient query lacks a field.

| Existing population to consume | Target use and limitation |
| --- | --- |
| Preparation, evaluator and provider demand/completion/evaluation, derivative achievement, cache hits/projections/miss causes, implicit stages and inclusive durations | Distinguish repeated preparation, broader derivative demand, actual evaluation and current provider effects. A cache hit cannot authorize skipping required effects. Shared contributors/aliases do not acquire repeated exclusive cost; derivative construction is not runtime evaluation. |
| Ipopt sequence/callback evaluations, iterations, native snapshots and enabled/running/available overall/initialization/search/trial/scaling/symbolic/numeric/backsolve timers | Join demand and task populations first. Nested timers are inclusive; submitted-NLP unscaled snapshots do not automatically map to original physical coordinates. Internal linear retry/refinement remains unavailable through these tasks. |
| POUNCE monolithic/Schur-F/Schur-S factors/backsolves, failure/interruption, ordering/fill/symbolic reuse and warm-start/crossover | Keep outer factor calls distinct from unknown inner numeric attempts and configured refinement bounds. Parent factor/path identity enables B03/B04. |
| KINSOL nonlinear/function/Jacobian/backtrack/beta/linear/preconditioner/Jv work and KLU issued/completed setup/symbolic/full-factor/refactor/rcond/condest/substitution | Explain fixed-pattern reuse, conditioning rejection and full-factor fallback without assuming direct KLU work is counted by Krylov iterations. Difference-quotient residual calls are a narrower population than all callbacks. |
| IDAS steps/residual/nonlinear/linear/setup/Jacobian/Jv/preconditioner/root/initialization/sensitivity/quadrature, order/step/time and checkpoint/replay budgets | Distinguish initialization, forward, sensitivity/quadrature, replay and backward owners. Internal accepted steps, returned samples and selected directions differ. B02 only adds a discriminating state-contribution view. |
| Diffsol setup causes and error-test/nonlinear work; faer/KLU symbolic/numeric/substitution and factor validity | Attribute checkpoint reconstruction versus numerical difficulty by segment/method. A setup is not necessarily symbolic analysis or fresh factor; LU completion is not conditioning qualification. BDF/RK/explicit applicability differs. |
| HiGHS typed integer/int64/double info, simplex/IPM/node/incumbent progress and existing explicit diagnostics | Preserve LP/QP/MIP applicability and presolve mappings. IIS, ranging, relaxation and copied-model inquiries can perform new work and retain their explicit diagnostic owner/deadline; inspection cannot trigger them. |
| Clarabel callback/final mu, sigma, step, primal/dual/infeasibility residuals, gaps, costs, kappa/tau, solve time and linear solver/nnz/thread metadata | Explain cone progress within its normalization and PSD `svec` map. Linear nnz is not a factor-call count; private timer breakdown is conditional B05 work. |
| SCIP selected stage/plugin presence/activation/reset, timing, incumbent/bound/gap/node and selected LP/NLPI work | Preserve restart/worker/merged cumulative populations and no-incumbent outcomes. `SCIPgetNLPs` counts LPs; `SCIPgetNLPIterations` counts LP iterations. No universal per-attempt delta or global-optimum claim. |
| PETSc SNES cumulative work/reason/norm, latest KSP and TS step/rejection/failure/time/dt, selected event activation/depth/entered/completed accounting | Distinguish setup, preconditioner, search and TS callback work. Inclusive event time/flops are not exclusive CPU; latest KSP differs from cumulative SNES linear work. Norm/preconditioning convention and configured MPIUNI/no-GPU limits travel with the evidence. |
| Uno iterations/evaluations/subproblems, objective/feasibility/stationarity/complementarity, CPU seconds, accepted/trial checkpoints, materialization and HiGHS feasibility work | Native checkpoint ordinals do not manufacture generic iteration numbers or original physical permission. CPU seconds differ from wall seconds; private subproblem iterations remain unavailable. B05 clarifies this wording. |

Targeted acceptance is copied-record interpretation/projection with absent, inapplicable,
unqueried, reset, interrupted and failed populations, plus one genuinely changed adapter's
small owner-level control when needed. Test direct-KLU versus Krylov meaning, inclusive timer
overlap and cumulative-versus-latest semantics with focused fixtures; do not rerun all solvers
merely to document existing signals. Remove any replaced duplicate interpreter/counting logic
when its consumers move; no legacy adapter mechanism is identified for deletion in advance.

<a id="b02"></a>

### B02 — Investigate weighted IDAS local-error contributions

**Coordinator inquiry I01. Question:** when existing rejection/setup/work counts leave a selected integration's
difficulty ambiguous, can safely sampled local-error contributions identify the relevant native
state coordinates? The reviewed SUNDIALS optional outputs `IDAGetErrWeights`,
`IDAGetEstLocalErrors` and `IDAGetTolScaleFactor` exist but are not consumed by the inspected
`dynamics/idas.rs`. Review the actual maintained implementation and documented return semantics
before using them; getter existence alone does not establish valid failed-return observations.

Use a short manufactured multi-scale differential/algebraic problem with independently known
state ordering and selected algebraic-error suppression. At eligible owner returns compare
copied weights/local estimates and the weighted contribution projection with the native
error-test convention. Include accepted/rejected-return provenance, pre-initialization absence,
restart and no-current-estimate cases; if a failed return does not expose a valid estimate,
report it unavailable instead of attaching an earlier estimate to that failure.

Adopt only if the observation settles a material question, stage validity/mapping are known,
and selected O(n) copies plus the necessary vector scan fit explicitly admitted owner buffers.
Emit bounded selected contributors with actual sample/state/step and weight/suppression
semantics; retain omitted extent and nonfinite/unavailable status. No fresh integration or
tolerance adjustment is performed to observe them. Local estimated error is not global physical
trajectory error, and an optional scalar tolerance-scale observation does not imply that the
vector population was available.

No adoption is the explicit outcome if existing populations answer the question, the relevant
return cannot support a current estimate, mappings are unavailable, or the copy cost is not
justified for the selected route. Preserve that limit in analysis/context and keep the broader
unanswered physical attribution explicit. This route does not resolve any genuine unrelated
missing-evidence defect. B06 consumers depend on the adopted valid population, not on merely
completing the inquiry. Delete only any superseded duplicate snapshot path actually replaced.

<a id="b03"></a>

### B03 — Investigate selected FERAL factor snapshots

**Coordinator inquiry I02. Question:** does a changed ordering/scaling/front or perturbation path explain selected
factor work that existing nnz/fill/pivots/symbolic-reuse records do not distinguish? The reviewed
`last_factor_stats` contains additional scaling, escalation, supernode/front and static-pivot
perturbation detail. Inspect its lifetime and actual forwarding path first; a snapshot documented
as O(1) is not a measured observer-overhead claim.

Use small factor controls for successful completion, a genuinely exercised escalation/scaling
case, hard failure and retained application with no new factor. Adopt only the fields that
discriminate the question and can be read after their eligible factor at the performing owner.
Attach the actual factor/path/generation; unavailable or cleared snapshots cannot inherit a
prior application's successful statistics. Preserve requested versus actual ordering and
native matrix/normalization meaning. Inertia under static perturbation describes `A + Δ`;
pivot extrema are neither an original-model curvature certificate nor a condition number.

No adoption is appropriate if current records settle the cause or additional fields have no
valid/current snapshot on that path. Preserve unavailable detail, without changing factor
algorithms or running extra factors merely to populate a report. If adopted, extend the existing
native-factor projection and migrate B06 consumers; remove the superseded partial projection
only where the new projection actually replaces it, retaining all previously useful fields.

<a id="b04"></a>

### B04 — Investigate genuine POUNCE inner work

**Coordinator inquiry I03. Question:** does a single outer factor/backsolve operation conceal repeated native numeric
attempts or actual refinement steps material to the selected diagnosis? Existing
`Observation::record` explicitly marks these populations unknown. Inspect actual maintained
fork retry/refinement loops and the simplest existing library-owned observation seam before
choosing a small hook. Outer calls or configured upper bounds cannot be renamed as actual work.

A useful hook supplies parent factor/path/generation, inner issued/completed/failed/interrupted
attempts, and actual refinement steps/outcome. The actual native loop owns the event; no duplicated
solver controller, extra factorization, residual solve or altered fallback/stopping is allowed.
Qualify stable observer storage, exception containment, cancellation/drain and observer-off
behavior at the bridge as well as source loss at the consumer. Any fork/ABI revision remains
bounded to that capability with ordinary dependency change review.

Use selected small zero/nonzero refinement and successful/failed retry controls that actually
exercise the inner loop; an outer-call counting test is insufficient. Check that inner counts
differ where expected while original numerical acceptance/outcome remains unchanged. Adopt
only if real loop parentage and safe nonblocking observation can be established and the
additional population changes the next diagnostic action. If the loops cannot be exercised
or a safe useful hook is not established, record no adoption and keep counts unknown; no
claimed actual-work correction is complete on that basis. B06 needs the working hook and
qualified population before exposing actual inner counts. Retire replaced aggregate-only
labels only for newly qualified populations; preserve unknowns elsewhere.

<a id="b05"></a>

### B05 — Clarify Uno and investigate private phases selectively

Separate two obligations. First, correct shared interpreter wording where the broad Uno
poll/callback description obscures the already supplied structured accepted/trial checkpoints.
Inputs are the actual callback kind/ordinal/iterate semantics; outputs describe those precise
facts while retaining opaque poll intervals and unavailable private detail. Existing callbacks
deliberately supply no generic iteration number and cannot grant original convergence.
Use copied accepted versus termination-trial records, missing-kind and truncated-coordinate
controls; replace the overbroad explanation rather than adding a competing classifier.

Second, under **coordinator inquiry I04**, investigate Clarabel timer phases or Uno private subproblem/linear detail **only** when
existing progress/work and a useful CPU observation leave a consequential question unresolved.
Clarabel 0.11.1's public timer owner does not expose its nested map structurally; do not parse
printed text. Uno's current bridge declares private subproblem detail unsupported. Inspect a
fitting public API, bounded upstream access change or native owner hook, retaining actual
cone/subproblem/path identities and safe stages. No concurrent private-state reads or new
numerical work in inspection are permitted.

Adoption requires a selected short on/off and failure control, genuine phase/population
meaning, safe lifecycle and a demonstrated explanatory use. No adoption is the default if
existing records answer the question or no sound low-burden access is established. Keep the
limitation and next optional observation visible; this cannot substitute for the independent
Uno wording correction. If a hook is adopted, B06 migrates actual consumers and removes any
superseded opaque-only interpretation for that covered phase, without claiming global detail.

<a id="b06"></a>

### B06 — Integrate selected evidence and qualify causal limits

For each actual delivery, carry the copied population, semantic declaration, relevant mapping
and availability through existing observation/detail projection into 47c interpretation and
47a frozen descriptive inputs. The publication/CLI owners consume those working contracts;
they must not call native state to hydrate a pack. A selected unavailable mapping remains
explicit. New original physical evaluation, rank/condition or infeasibility analysis retains
the existing explicit diagnostic operation, finite work/admission and independent products;
opening an offline pack never executes it.

Coordinate **I05** with 47a/47c for the actual original-physical pullback scope. This package
supplies native coordinate/transform facts and their limits; it does not promise universal
dual, stationarity or executed physical-state reconstruction from the static source map.

Shared analysis can connect derivative-order demand to actual evaluations, factors to recovery
paths, and dynamic setup to forward/replay/backward/checkpoint purposes. Counts/order support
these explanations but do not alone prove elapsed benefit, intrinsic model difficulty or
exclusive cost. Inclusive duration-per-completion comparisons require matching populations
and compatible context; missing starts/selection/clock coverage blocks stronger conclusions.
The analysis owner retains source-linked facts, hypotheses, limitations and next observations.

Targeted tests cover source identities and full event grouping; producer reset/overflow/loss;
failed/cancelled work and optional observation failure; same source through live and frozen
consumers; omitted physical mappings; and selected field semantics after export/readback.
Use small off/on/selected-detail controls for each newly adopted observer, checking actual
work populations and ordinary scientific outputs before any overhead conclusion. No fixed
speedup or identical deadline-limited trajectory is promised. Remove replaced duplicate
projections/explanations and their obsolete tests once callers have moved and replacement
tests pass. Existing algorithms and scientific policies remain in place.

## Verification and handoff

For no-adoption outcomes, retain an observable reopening premise with the coordinator decision:
I01 reopens when error-test/rejection work remains decision-relevant and a valid mapped native
estimate is available; I02 when a changed scaling/order/front/perturbation path explains an
otherwise unresolved factor interval; I03 when one outer factor conceals material repeat work
and an exercisable safe loop seam exists; I04 when a concrete opaque subproblem/phase still
blocks the next action after existing work/progress and an available profile. These triggers
do not promise a winner, and cannot postpone the separate Uno wording correction.

**Implemented:** selected IDAS local-error capture, the FERAL factor extension, existing POUNCE refinement reuse with explicit parent/group identities, Uno wording correction and shared frozen interpretation. The Outcome records scoped native qualification and explicit no-adoption limits. During implementation use affected-package compile checks and meaningful targeted
unit/owner controls with explicit force-validation where required. Report the actual command,
mode, scope and result against the zero-error baseline. Source-backed API eligibility and a
successful copied-record test do not establish native runtime qualification.

Package completion requires its working consumer integration or its explicitly supported
no-adoption outcome, unresolved limits and dependent-route update. The latter cannot close
a material defect simply by declining a remedy. B05's wording correction remains required
independently of conditional private hooks. No candidate remains silently promised or receives
invented zero/complete coverage after inquiry.

47f selects the relevant assembled journeys and once-only scope-end static/integration checks
after functional integration. It does not acquire an all-solvers, long benchmark, full wheel,
reproduction or environment-proof campaign from this catalogue. Enduring native observation
meaning goes to the existing architecture/diagnostic documentation through the coordinator's
decision/publication route, not through independent edits during this plan-creation task.

## Outcome

The [completion audit](#completion-audit--2026-10-18) qualifies the implementation claims
below. Earlier commands/results retain their original tested scope; they do not establish
complete delivery of the reopened packages. The Work packages table owns current progress.

**Implemented — 2026-10-18:** B01–B06 deliver selected owner-safe IDAS local-error capture,
the existing FERAL factor event's additional snapshot/MC64 facts, actual POUNCE refinement
reuse with call parentage and bounded multipart groups, corrected Uno checkpoint wording,
and the shared source-linked live/frozen interpretation. Existing class-specific work,
coordinate, availability and inclusive-cost meanings remain authoritative. I01–I03 adopted
these bounded populations; I04 adopted no private Clarabel/Uno hook because no concrete
remaining phase question justified one. A specific unresolved phase after existing progress
and a useful profile is the reopening premise. The coordinator owns inquiry dispositions.

The dependency change is the coherent POUNCE/FERAL source-family move to
[`7f5e2e9d78555bd53f780c1af186c8fd22c922f5`](https://github.com/paul-heyse/pounce/commit/7f5e2e9d78555bd53f780c1af186c8fd22c922f5),
published with explicit maintainer approval on `plan47-native-diagnostics`. Other locked
versions and edges remain unchanged. The extension changes observation only, not algorithms
or defaults. The source uses one shared `last_factor_stats` read; it traverses supernodes
and may clone an External ordering vector. No universal O(1), zero-allocation or measured
overhead claim is made.

**Tested — 2026-10-18, baseline zero:** the isolated source checkout at that revision ran
`cargo test -p pounce-feral selected_ --lib` with **5 passed, 0 failed**, and
`cargo test -p feral -p pounce-common passive_ --lib` with **3 passed, 0 failed**, using the
linked solver environment. Controls exercise real MC64 retry/fallback deltas, successful to
fatal snapshot clearing, no-new-factor state, zero/nonzero and multi-RHS refinement,
observer-off output equivalence and optional-sink failures.

**Tested — 2026-10-18, baseline zero:** repository native-local controls used the private
resource state `/tmp/pse-diagnostic-review-resources-20261018`, pinned toolchain, explicit
force-validation and this prepared command prefix:

```text
PSE_TEST_RESOURCE_STATE=/tmp/pse-diagnostic-review-resources-20261018 scripts/pse-env --recipe-default-class exclusive --native=solver,klu -- env BASH_ENV= .venv/bin/python scripts/select.py --execution-effects native-local --features idas,pounce unit pse-backend-native
```

Appending `local_error` passed **4/4**, with 378 unselected tests. It checks native masked
full-state WRMS, actual fresh-return validity, negative/repeated/zero-step/restart absence,
bounded contributors/nonfinite/omitted coordinates, off/refusal/release output preservation
and optional progress failure. Appending
`'test(pounce::observed::tests::) | test(selected_native_factor_and_refinement_records) | test(solve::record::tests::)'`
passed **8/8**, with 374 unselected tests. It checks actual Progress multipart retention,
call/group/source identity, optional publication failures, and compact record ownership,
serde and extent. No compiler warnings remained in either final run.

**Tested:** the [47a Outcome](47a-frozen-evidence-and-review-context.md#outcome) records the
44-test shared runtime group, including copied native event readback and accepted/trial/
missing-kind Uno controls. Those checks establish selected consumer semantics, not every
solver's runtime qualification. I05's broader physical attribution and I07's assembled
membership/export obligations remain with 47c–47f. Full static/integration/overhead campaigns
are **not_run** here and remain with 47f; no speedup is asserted.

**Mistakes made and corrected:** the original unknown-inner-work assessment overlooked
already implemented refinement/RHS/residual facts; those were reused rather than adding
another controller. A larger factor event exceeded the existing Progress record size, so
the adapter now emits bounded version-2 parts with explicit group/part coordinates instead
of silently losing detail. An initial native preparation failed before tests when shell
initialization removed `SCCACHE_DIR` in dependency directories; clearing `BASH_ENV` at the
final prepared command fixed that run without a tooling edit or disabling the cache.

**Deliberate deviations:** no private Clarabel/Uno hook, no broad claim about all internal
retry/refinement work, no numerical hydration or automatic tolerance-scale application,
and no tightened physical accuracy. The IDAS selection owns a private SUNDIALS context and
two admitted scratch vectors to keep optional allocation errors out of the scientific
context. Only qualified current source facts are projected. The replaced partial factor
projection and overbroad Uno/unknown wording were updated in place; no compatibility
execution path or duplicate diagnostic store was retained. Enduring meaning is published
in blueprint §23.1 through revision 212; formal proposed ADR status is unchanged.

## Completion audit — 2026-10-18

Historical assessment at audit time. Current package states and the
[corrective Outcome](47-pipeline-diagnostics-and-review-artifacts.md#corrective-outcome--2026-10-18)
supersede the remaining-work statements in this section.


**Interface-checked:** independent production-source inspection and coordinator integration
review against the package targets. The [coordinator's completion audit](47-pipeline-diagnostics-and-review-artifacts.md#implementation-completeness-audit--2026-10-18)
owns CA01–CA12, their source basis, priority and closure conditions. This assessment explains
package-table corrections; it is not another finding-disposition ledger. No product repair or
47f qualification was performed. Historical focused passes above remain valid under their
stated conditions; a passing local fixture is not complete consumer integration.

| Package | Audit assessment | Basis / remaining boundary |
| --- | --- | --- |
| B01 | Partial | CA04: existing relevant native populations are filtered out and valid nonadditive latest values become unavailable cumulative values. |
| B02 | Aligned | IDAS selection, admitted scratch, safe-return freshness and actual copied local-error vectors preserve explicit unavailable cases. |
| B03 | Aligned | The pinned FERAL extension reuses the existing successful snapshot; selected extras and fatal clearing do not add a factor operation. |
| B04 | Aligned | Actual refinement/MC64 work and attempt-local parent/path/group identities reach validated multipart analysis; other inner work remains unknown. |
| B05 | Aligned | Uno accepted/trial wording follows actual callbacks. The bounded no-private-hook decision has an explicit reopening premise. |
| B06 | Partial | CA04: adopted families are integrated, but the broader existing-population consumption obligation is incomplete. No new hook is required by this finding. |

Next execution closes the cited supplying gaps and focused acceptance before composed completion
is claimed. Existing aligned mechanisms and the distinct live/scientific boundaries are retained;
missing broad qualification is not itself an implementation gap.


## Completeness repair — 2026-10-18

**Implemented:** The existing native catalogue now reaches analysis using source-native cumulative counters and
latest nonadditive gauges, including Diffsol checkpoint/recovery setup causes, IDAS shift and
checkpoint gauges, and KLU conditioning-rejected recovery. Scalar references retain their actual
population, units, generation/reset and availability. Retained maps follow original findings and
dynamic state contributors without inventing native dual/stationarity pullbacks. No native hook,
solver algorithm or stopping accuracy was added or changed by this repair.

The coordinated runtime selection exercised actual-shaped native checkpoints without final
segment summaries, original map correspondence, independent clocks and exact references. The
coordinator's [corrective Outcome](47-pipeline-diagnostics-and-review-artifacts.md#corrective-outcome--2026-10-18) owns command/results; broader all-context qualification remains
47f.
