---
title: "25k: Integrated qualification and closure"
status: in-progress
date: 2026-09-30
adrs: [ADR-0153]
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md, docs/design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s01, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fs11]
---

# 25k: Integrated qualification and closure

## Purpose and ownership

This is the **only full qualification stage** of the [Plan 25 series](25-design-remediation.md).
Plans 25a–25j implement the target directly, use compilation and focused behavioral checks,
and delete replaced mechanisms as their callers move. They do not repeat this campaign at
section or document boundaries. This plan starts after all their functional packets are complete.
The later [25l functional follow-up](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25l-flowsheet-compilation-and-solver-routing.md) supplies
the corrections required at the full-case review boundary. The subsequent
[25m pipeline follow-up](25m-integrated-solve-pipeline.md) supplies the combined solver-review
functional handoff. The
[25n automatic-pipeline correction](25n-automatic-simulation-solve-pipeline.md) adds the corrected
functional prerequisite before K3 next resumes.

The coordinator owns its original finding dispositions; 25m owns its distinct adopted
solver-review findings; 25n owns AF-01–AF-08. This document owns the assembled system's eventual
qualification evidence. A passed packet, an accepted ADR and a historical Plan 23 result are
different evidence from a qualified Plan 25 implementation. The execution checkpoints below
record current readiness; the Outcome will record the completed campaign and its limits.

## Decisions

- Qualify behavior, architectural fitness and performance separately. A numerical pass does
  not establish change locality; a design review does not establish numerical accuracy.
- Use existing recipe-owned environments and independent reference data. Keep explicit Arrow
  force-validation for correctness runs. Native campaigns use the repository's memory-capped
  recipes; preserve the limit and record thread, timeout and feature conditions.
- Test the final target only. Do not retain deleted execution paths as comparison oracles.
  Preserve source/reference observations with independent scientific authority, regenerating
  only when their actual inputs or contracts change.
- Performance acceptance includes reuse/allocation counts and resource bounds. Report timings
  as measurements under named conditions, not invented speedup thresholds. Historical results
  are contextual unless a reproducible comparable baseline is available.
- Keep one qualification result per selected scope. Repair failures and rerun the failed scope;
  repeat other scopes only when the repair could invalidate their results. Report such a result
  as a composite campaign, with the zero-failure target, rather than as an initially clean run.

## Integrated journeys

These journeys combine the existing review scenarios; they are not a new parallel scenario
registry. Each needs positive controls and the indicated refusals, with physical tolerances
and fixture inputs authored alongside the implementation that supplies the behavior.

| Journey | Consumed plans | Required evidence |
|---|---|---|
| Typed scientific extension, S01/S09 and FS01–FS03 | 25a/25b/25h/25j | Lawful reduced-coordinate model and typed overlay round-trip; wrong coordinate/basis/datum refused; unknown composition cannot prove conservation; complete empty composition remains distinguishable; mixed pair selections respect fit-group constraints |
| Connected steady and dynamic flowsheets, S02 and FS04/FS05 | 25a/25b/25c/25e | Indexed mixer → holdup reactor → separator; executable recycle with conditional unit solves; independent boundary and temporal closure; required initial conditions; undeclared/wrong event transfer refused |
| Implicit substitution and square sensitivity, S03/S04 and FS06/FS07 | 25d/25e | Two-root and restricted-root cases, honest exact/relaxed labels, C1 first-order versus C2 second-order demand, rank/branch failure, qualified root response without a dummy objective |
| Event endpoints, shooting and incumbents, S04 and FS05 | 25c/25e/25f | Event-defined success evaluates its actual endpoint obligations; fixed-horizon coverage remains incomplete after an early event; shooting consumes the declared route and common qualification; accepted limit incumbent remains explicitly non-optimal |
| Repeated and interrupted durable experiments, S04/S06/S10 and FS09 | 25e/25f/25g/25i | Same binding in distinct occurrences, return-path continuation, compatible seed/fallback decisions, cancellation/retry and lease recovery, per-occurrence idempotency and typed failures without a fabricated result |
| Durable evolution, S05/S11 and FS10 | 25f/25g/25j | Old artifact read with its recorded domain; directional consumer/writer refusal; read-only open; explicit migration and lineage; additive vocabulary change preserves catalog references; bounded orphan reconciliation |
| Edit/rebind/reuse, S07/S09 and FS08/FS11 | 25a/25h/25i/25j | Clean/incremental agreement, fresh diagnostics over shared mathematics, old result attribution retained, A/B/A binding reuse, referenced-context invalidation and repeated durable-worker preparation reuse |
| Resource and failure lifecycle, S06/S10 and FS11 | 25f/25i | Abandoned staged work retains admission until native completion; failed dispatch releases it; escaped products retain reservations through eviction; worker panic is visible and never silently rerun |

## Execution packets

| Packet | Prerequisite | Responsibility and acceptance | Status |
|---|---|---|---|
| <a id="k1"></a>K1 Functional readiness | All functional packets in 25a–25j | Confirm target consumers are migrated, replacement/deletion obligations are complete, required decision routes are satisfied and fixtures exist for the journeys above. Resolve remaining functional work in its owning plan before starting K2 | done |
| <a id="k2"></a>K2 Format and static qualification | K1 | Run the single series-wide formatting/lint pass and relevant Rust/Python compilation, generation, family, governance and documentation checks; repair to zero | done |
| <a id="k3"></a>K3 Behavioral and scientific qualification | K2; 25l L6, 25m P11 and 25n N11 functional handoffs, with refreshed static evidence for affected source | Run the selected Rust, native, Python, conformance and compatibility journeys; independently assess physical closure, domains, original-space residuals, outcome truth and durable lifecycle | in progress |
| <a id="k4"></a>K4 Reuse and performance measurement | K3 | Measure cold/warm preparation, one-body edits, in-process/durable value studies, retention after eviction and worker admission; report counts, timing distributions, memory and all refusals/failures | planned |
| <a id="k5"></a>K5 Architectural assessment and closure | K3/K4 | Conduct one bounded target-design review of the assembled change, reconcile every finding with its evidence, update enduring owners and close only demonstrated scope | planned |

### Review boundary — full case compilation and solver execution

Execution checkpoint, 2026-10-02: the maintainer paused further qualification to assess
full flowsheet/case compilation, capability-driven mathematical execution and the design
issues exposed by integrated failures. The
[target review](../design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md)
records the current implementation's Revise judgment and proposed direction. It identifies
consumer demand arriving after executable/support admission, incomplete composition of
preparation with contextual solver eligibility, and misleading public interpretation of
unavailable feasibility evidence/resource failure. It also preserves the existing physical,
structural, implicit-meaning, identity and lifecycle contracts.

K3 remains incomplete; K4 and closure remain pending. This requested review is broader in
its compilation/routing inquiry than the planned final K5 evidence assessment and does not
complete that packet. Its findings are now scheduled in the
[coordinator's existing disposition owner](25-design-remediation.md#full-case-compilation-and-solver-routing-follow-up)
through the [25l implementation plan](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25l-flowsheet-compilation-and-solver-routing.md).
**Implemented/Tested, 2026-10-02:** [25l L0–L6 functional scope is complete](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25l-flowsheet-compilation-and-solver-routing.md#outcome-recorded-after-implementation),
with selective support, contextual readiness, completion and generated/recorded boundaries,
124 focused Rust tests, five Python codec tests and final default/linked compilation.
Its four runtime preparation controls start no native attempt and use no storage.
Static refresh checkpoint, 2026-10-02: the requested `just hygiene` refresh is complete
after source repairs and regeneration of the route-decision invariant fixtures. Default,
no-default and linked native Clippy all pass with `--keep-going` and `-D warnings`;
`just lint-solver-contracts` covers the linked graph outside hygiene, and
`just fmt-rust-check` passes. The final checks meet the zero-failure target after iterative
repair. Targeted diagnostics, math assembly, native preparation and contextual routing
controls pass with explicit force-validation and one test thread; the five Python codec
controls also pass. The repairs preserve shared immutable facts, inline admitted metadata,
retained routing evidence and the distinct exact-Hessian inputs.

The next work here is the affected K3 scientific and integrated journeys, K4 measurements
and K5 assessment. Earlier K1/K2 results retain their historical conditions; this static
refresh and focused readiness do not qualify whole-case solutions or performance.
The selected dev-profile measurement choice remains in force.

The resumed campaign includes 25l's demand separation, pending class/capability evidence,
contextual method/build/representation controls and original failure-meaning matrix alongside
these existing journeys. K4 extends its current workload owners to measure actual selected
output and Value/First/Second support construction, reuse and resource lifetimes. This remains
one composite qualification campaign; 25l adds no second full gate.

### K1 — Readiness without a new governance framework

Execution checkpoint, 2026-10-02: the completed 25a–25j working tree is the preserved
baseline. Existing controls cover much of the required storage, response and lifetime
matrix. Readiness work adds actual connected process/recycle and mixed durable journeys,
and prepares measurement workloads before the comprehensive functional receipt. K4 uses
the existing dev profile, as selected by the maintainer. Native campaigns remain capped
at 120 GiB, with bounded test scheduling and one BLAS/OpenMP thread. Heavy campaigns and
timing runs are serialized. Proposed ADRs retain their decision-PR status; no commit,
push or publication is part of this execution.

Readiness is complete. Actual indexed conditional recycle and dynamic inventory controls,
mixed durable recovery/continuation/cancellation, and nonoptimal incumbent acceptance are
registered in their existing harnesses. The mixed durable journey corrected nullable retry
classification and premature refusal before explicitly permitted fallback acquisition in
their existing store/policy owners. The five K4 workloads now have untimed controls through
`just bench-case-smoke`; this mode creates no measurement receipt and does not relax
`just case-measure`'s comprehensive functional prerequisite. Measurement instrumentation
counts actual body admissions, value rebinds and Delta write attempts. The next work is the
format/static pass, then the comprehensive functional receipt and selected dev measurements.

Use the existing plan ledger, source inspection and packet evidence. Do not create a source seal,
mandatory symbol manifest or automated architecture score. An intentional retained component
must still have a current target responsibility; deleted-path tests are not historical artifacts
to preserve. Legitimate independent oracles and native-library adapters remain.

Execution checkpoint, 2026-10-03: the resumed scientific campaign exposed two additional
consumer/resource issues after the scoped support analysis correction: derivative sampling
consumed a Value-only case, and repeated occurrences multiplied mutable evaluator scratch.
The correction requests First only at the sampling consumer and shares scratch sequentially
inside one attempt while retaining occurrence-specific caches and attribution. Required
factorable projection exhaustion now refuses with a typed resource failure; §7.5 is reconciled
with that behavior. The maintainer authorized larger finite admission ceilings for the 192 GB workstation:
512 MiB local numeric scratch, 2 GiB complete case-worker storage and an 8 GiB runtime worker
allowance. Explicit small-budget refusal controls remain bounded; scientific tolerances,
reference expectations and physical bounds remain the acceptance inputs. The runtime allowance
also raises its existing derived factorable/cone/SOS capacities fourfold. A successful focused
control does not close K3. The PR smooth fixture now reaches Ipopt, but its candidate remains
infeasible and fails physical closure and seven independent expectations; the larger allowance
is not scientific acceptance. A same-model limited-memory diagnostic also fails. The smooth
fixture now declares its inherited `ideal-K` initialization stage followed by the original PR
specification, with a finite 1800-second initialization allowance; the staged probe satisfied
all original model checks and reference expectations. This does not qualify the other
formulations or enclosing campaign. The affected
multi-fixture run exposed a separate conformance assumption about preliminary route decisions:
no selected candidate means its structural assessment may be absent while preparation remains
pending. Conformance now distinguishes that state from refusal and continues independent
fixtures after a resource interruption. A genuine common original witness survives refusal;
ambiguous full candidate assessments remain Rust-owned, with mode/admission summaries in
serialized route records until a common or selected flat assessment exists. Publication errors
retain their diagnostics and incompleteness without reinterpreting a completed native result.
Finish the affected scientific journeys, then collect the comprehensive qualification and
measurements.

The retained affected run passed the smooth and complementarity PR formulations. Its PFR
failures exposed a distinction lost at completion: nonzero accounting totals were counted as
required zero-closure observations. The specialized domain model now owns that distinction,
and steady, transient and fitting completion consume its required-check count. Accounting
totals retain evaluation and finiteness checks. The finite case-specialization ceiling is now
8,192 distinct bodies; repeated occurrences still count once and explicit small ceilings refuse.

The maintainer authorized the nonlinear selection extension on 2026-10-03.
ADR-0153 records its new kernel binding before integration. Constant authored
physical boxes and an exact residual/eligibility/guard DAG now admit conditional
First/Second preparation when a validated verifier is present. Numerical KINSOL
execution remains separate: a direct uniform root chart and native closed-slab
exclusion must establish one eligible root throughout a parameter neighborhood,
or complete exclusion must establish an empty alternative. Unresolved coverage,
multiple roots, strict boundaries and opaque projection never confer derivatives.
IBEX owns interval arithmetic, contraction and covering; its supported bundled
SoPlex route is guarded before Taylor contraction. Proof state is consumed in its
originating request, with finite shared time/cells and reserved transient storage.

Projection, isolated adapter and compiler controls pass. The full linked profile
exposed a SoPlex ABI collision: IBEX's bundled library calls resolved to incompatible
SCIP exports and crashed during LP contraction. The selected correction is source-built
static PIC libraries with hidden C++ visibility, preserving the one proof implementation.
The corrected fully linked adapter controls pass; the original nested PR journey still
requires successful execution. Independent
inspection identified IBEX's partial-coordinate ACID mask edge case and unnecessary
parameter shaving. HC4 with guarded Taylor/LP alone exhausted the covering allowance.
The adapter now adds library-owned parametric interval Newton behind the same full-box
C1 guard admission, with the same native covering and complete-exclusion requirements.
The linked controls pass. Exact request replay proves the first liquid-only alternative
empty; whole-case qualification remains pending. Refusals now preserve chart, boundary
and coverage reasons rather than collapsing them.
Exact two-phase replay establishes its uniform chart and excludes the lower-fraction
complement slab, but exhausts 131,072 cells in the upper slab. Identical native predicates
now share guard functions while the source retains attribution. The finite covering ceiling
is raised to 1,048,576 cells, with its conservative reservation derived from that ceiling;
the outer deadline, worker admission and OS memory cap remain enforced.
The expanded exact replay completes two-phase exclusion in 322,286 cells. Whole-case
execution then reveals a different issue: the vapor-only domain contains an eligible
middle cubic root although the numerical stable-liquid-root proposal is ineligible.
Empty coverage would be false. The authorized capability now establishes a unique
minimum across the entire eligible-root union: a regular winning chart plus complete
score/tolerance competitive-root exclusion. Losing regimes may contain roots without
being unique or empty. Exact criteria and tolerances enter the shared projection, and
one worker-owned certificate may be reused only within its exact source, physical-domain,
parameter, winner and derivative scopes. This replaces the stronger per-alternative proof.
The first production whole-selection run reached Ipopt but exhausted its case
time allowance without establishing physical closure or the reference expectations.
Exact replay of its full three-alternative request establishes complete competitive
exclusion with IBEX's largest-first bisector; this changes covering strategy without
changing the original domains, predicates or proof requirements. The run also exposed
a deadline ownership defect: nested numerical and proof calls could start a fresh
allowance instead of consuming the parent attempt's remaining time. Scoped provider
construction now carries the original absolute deadline and cancellation together;
the repair must reject late evidence without relabeling time exhaustion as cancellation.
Fitting, shooting and dynamic final checks consume the same enclosing scope, including
empty observation programs. Targeted causal-map controls also exposed a separate
structural admission defect: upgrading a Value function's empty numeric coordinate map
could erase its original free dependencies. Structural incidence now derives those
coordinates from the original free inventory, retains the selected outputs and unspent
construction allowance, and keeps control dependencies separate from numerical Jacobian
support. Boundary admission checks those controls without promoting a Value evaluator.
The shared-deadline, structural-incidence and linked boundary controls pass, and the
refreshed hygiene, default/no-default Clippy and linked-native Clippy checks meet the
zero-failure target. Independent implementation review identified a final operational
branch binding committed before a deadline check; the check now precedes that commit.
The unchanged nested PR case still exhausts its 600-second attempt allowance. A focused
run uses the maintainer-authorized larger finite allowance of 3,600 seconds, preserving
the fixture's authored inputs, original bounds, procedure, expectations and tolerances.
That run also exhausts the shared execution deadline. Its completed competitive proofs
have no unresolved boxes, while outer Ipopt progress remains slow with large dual residuals
and heavily reduced trial steps. A focused original-case diagnostic now compares the
complete Jacobian against independent Value differences and examines normalized numerical
rank before selecting a repair; more wall time alone is not an established solution.
The initial Jacobian agrees on stable material entries, including selected-root links,
and has numerical rank 120 at the recorded cutoff. A separate weighted Hessian comparison
finds no stable material selected-root disagreement. Its combined-row check exposes
finite-difference resolution limits when tiny non-selector steps are subtracted from
gradients dominated by affine terms. Those columns now use the central-difference
truncation/roundoff step scale; actual selector inputs retain chart-sized steps, and
the original bounds and comparison tolerances remain unchanged. The rerun has no stable
Jacobian or weighted Hessian disagreements. Twelve apparent differences resolve within
the unchanged tolerance; the remaining direction is noisy and its symmetric counterpart
agrees. The original-start probe retains its other noisy entries and two-combination
Hessian coverage limits. A separately reported explicit POUNCE probe is next: the published
capability admits this unchanged bounded problem, but the automatic Ipopt timeout remains
an unsuccessful receipt rather than becoming a pass through another profile.
Whole-case acceptance remains pending. The next dependencies are successful production nested PR execution,
full campaign/static assessment, K4 measurements and bounded K5 closure. K3, K4 and K5
remain open.

The ten historical 25a/25b/25d/25e worktrees have been assessed against current source owners,
registered tests and completed packet outcomes. No missing aligned work was identified; their
changes are integrated or superseded, and merge debris has no product value. Remove those
worktrees after qualification passes, without retaining differences merely because they exist.

### Review boundary — integrated solve pipeline

Checkpoint, 2026-10-03: the maintainer paused further same-scope solve probes to assess the
integrated numerical pipeline. The
[solver acceleration review](../design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md)
and [current-tree refinement](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md)
retain the full applicable method target and specify corrected contracts. Neither design
review qualifies the original nested PR case or completes K5.

[25m](25m-integrated-solve-pipeline.md) now owns implementation of those findings and the
functional prerequisite. Its P11 handoff supplies migrated consumers, deletion obligations
and registered fixtures. K3 remains incomplete and paused pending that handoff; historical
K1/K2 evidence retains its scope. At functional completion, refresh the affected static
scope and run one composite qualification campaign here. Do not resume larger-limit or
alternate-profile probes as a substitute for the planned pipeline changes.

Extend the existing K3 journeys with actual declared strategy execution, same-backend
profile recovery, derived-to-original assessment, sheet continuity, accuracy propagation,
report-less/terminal failures and generated/direct/Python trace projection. Keep original
PR/CSTR fixtures, bounds and independent expectations. Extend K4's existing dev-profile
workloads with actual proof/evaluation/setup/factor reuse and strategy work/resource
accounting; method inclusion requires no exhaustive combinations or benchmark ranking.
K5 assesses the assembled corrected target and adopted decisions. Worktree cleanup remains
the already recorded post-qualification action.

Functional handoff, 2026-10-04: Plan 25m P0–P11 is implemented, with migrated consumers,
generated public trace projection and the bounded assembled architecture review. K3 resumes
with formatting and static repair, then one assembled functional/scientific campaign against
the original scientific criteria. The PR liquid tangent-plane fixture now has a distinct
local-reference identity and explicit Ipopt intent: its original laws, bounds, checks and
TPD oracle are unchanged, while the old backend-defect-dependent failure expectation is
retired. Dedicated SCIP reproducer and contradiction tests remain; global PR certification
is still unqualified under R-52. K4 follows the complete functional receipt in the current
dev profile. K5 consumes that evidence before closure and obsolete-worktree removal.

Execution pause, 2026-10-04, requested by the maintainer: Plan 25m's active agent repairs
are integrated in the existing main working tree; the 44 affected targeted native tests
pass against zero. Its [execution checkpoint](25m-integrated-solve-pipeline.md#execution-checkpoint)
owns their current state and targeted verification. The superseded assembled campaign
completed 37 initial checks, doctests and the inspection fixture successfully; default Rust
reported 2,629 passes, two failures and one timeout, and native Rust reported 3,119 passes
and eight failures, all against a zero-failure target. Its linked Python run and the refreshed
original PR campaign were stopped before completion. Source changed during repair, so this
is partial historical evidence and cannot support K4. The frozen complete qualification,
full seed/domain scope, required dev measurements and final closure remain pending. At that historical checkpoint, no obsolete worktree had been removed. The maintainer later
requested their assessment/removal during the 25n current-task closeout; the 25n checkpoint owns
the resulting cleanup.

Plan-creation checkpoint, 2026-10-04: the
[automatic-pipeline review](../design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md)
is organized in [25n](25n-automatic-simulation-solve-pipeline.md), which owns its functional
corrections and the missing native bindings. Its N11 handoff is now a prerequisite to resuming
this campaign. The original scientific criteria, full seed/domain scope, current dev-profile K4
measurements and post-qualification worktree assessment/removal remain unchanged. Creating that
plan does not resume execution or revise the historical receipts above.

Current-task closeout, 2026-10-04: 25n is partially implemented and paused for a review whose
structure the maintainer will provide. Its checkpoint owns landed work, focused verification and
remaining scope. No 25n N11 handoff or resumed full K3/K4/K5 campaign is claimed. The latest
maintainer instruction moves stale-worktree cleanup into that closeout.

Continuation, 2026-10-04: the maintainer waived the preliminary review and resumed 25n
implementation. Its checkpoint owns the remaining functional work and eventual N11 handoff.
This instruction does not claim completed K3/K4/K5 qualification.

The 25n continuation registers its functional fixtures in the existing native and Python
harnesses for the resumed K3 campaign:

| Journey | Existing harness/fixture | Qualification obligation retained here |
|---|---|---|
| Connected process topology and coupled original completion | Runtime `workflow::strategies::conditional::tests::automatic_workflow_causal_` and `math::solves::derived::tests::automatic_blocks_`; seed `campaign/models/recycle-flash.pse` | Successful and failed local sweeps, all non-port states, original residuals/bounds, no partial commit; topology does not establish connected-sheet transport |
| Stiff/ill-conditioned reconstruction and supplier chains | Runtime `compiled_ill_conditioned_nested_relation_chain_refines_against_tighter_native_reference`, `actual_multiple_root_suppliers_consume_nonzero_authored_offsets_and_chain_actions`, and full Root reconstruction controls | Actual consumed accuracy, chain actions, finite proof workspace and independent original comparison |
| Original PR/CSTR science | `campaign/models/bt-pr-formulations.pse`, `recycle-flash.pse`, `cstr-dynamics.pse` under `packages/reference/conformance.toml` | Original physical laws, bounds and independent values remain binding; decision-relevant criteria follow the authorized convergence review; targeted mechanism controls do not qualify these fixtures |
| Changed active set and related targets | Runtime `advanced_step_falls_back_on_active_set_change`, `advanced_step_activity_start_is_corrected_before_move_authorization`, `related_case_study_uses_secant_then_original_correction` | Proposal permission, original correction and separate control-move authorization |
| Resource, terminal cause and accuracy refusals | Runtime `math::strategy::admission`, `workflow::modeling::work_admission_tests`, derived original/refinement controls; native `pounce::schur_tests` | Pre-operation admission, retained allocation lifetimes, truthful unknown composed work, terminal no-retry and wrong-point/class refusal |
| Public/current recorded interpretation | Python `test_native_workflow.py`, `test_native_boundary_contracts.py`; runtime `workflow::worker_tests` and historical readmission controls | Inspection/request/result transport; full durable restart and mixed-operation lifecycle remain K3 journeys |

These tests are automatically discovered by `just native-test` and the linked Python recipes;
no parallel fixture registry is introduced. K4 still measures dev-profile reuse and resource
lifetimes, and K5 still owns the assembled assessment and decision/architecture closure.

### K2/K3 — Recipe selection and reporting

Execution checkpoint, 2026-10-02: static repairs are complete. Registry-owned enum defaults,
generated tuple-key projection, explicit store fences and receipt groups, current Python/native
consumers and invariant fixtures now agree with the target. The feature recipes retain workspace
unification and the complete declared combinations while supplying native math paths and qualifying
member feature names. Native helper compile conditions follow their actual consumers.

The imported future-compatibility repair uses the published validator/derive releases through the
existing generated Delta override, with one root version pin and the workspace lock as build
authority. A provisional Cargo patch was rejected by governance and removed; no governance rule
or warning was suppressed. Imported metadata boundary controls preserve schema, length and Unicode
semantics. The remaining work is the full behavioral/scientific campaign, then dev measurements
and the bounded architectural verdict. Source inputs stay unchanged from the comprehensive receipt
through measurement; closure documentation follows afterward. Parity runs before the ordinary
linked extension is refreshed for that receipt.

The first complete assessment exposed scientific lowering, initialization ownership and
recorded-contract defects, as well as fixtures still expecting replaced contracts. Repairs
retain authored temporal occurrence positions and resolved axis ownership, actual dependency
closure through validity/evidence, datum-point subtraction before reference translation,
and the admitted physical types of static conditional branches. Block binding now retains
existing shared parents while charging its new products. Recorded contracts select exact
versions and provenance; fault-proxy shutdown awaits owned relays and backend completion.
The local operational store has completed its preserving V1–V8 transition without reset, including append-only Uno/PETSc
native-backend enumeration values.
Targeted checks precede the repaired integrated campaign. The default Rust and doctest scopes
will inherit the existing native execution environment explicitly, including the Symbolica
licence, rather than relying on an interactive shell's direnv activation. K4 and the final
review remain pending successful qualification; the campaign will be reported as composite.

The full authored seed closure exposed additional campaign consumers beyond the earlier
focused roots. Their migration preserves nominal reduced quantities, directed transfer
owners, explicit indexed arguments and the original independent oracles. Concrete analyses
must declare narrow permission for selected records with unknown applicability; the evidence
remains unknown and extrapolation remains forbidden. Qualification also checks fixture
binding against the actual refined physical target and bounds actual sparse support
construction rather than refusing an otherwise lawful body solely for its support width.
Repeated PFR instances also exposed occurrence-specific validity targets in mathematical
identity. The ADR-0150 correction separates body-local checked members from each instance's
total attribution binding, retains complete ranges and physical context, and versions the
changed current frames without altering historical bytes. Typed diagnostic composition must
apply that binding before wrappers contribute independent source identities.

Positive Peng–Robinson flowsheets now explicitly select the original density relation in the
outer problem. ADR-0144 distinguishes this authored relational formulation from the retained
operational selector; an unproved nonlinear selector remains value-only and its derivative
refusal retains a typed capability cause. Intensive fixtures declare finite construction/support
allowances; the 2026-10-03 maintainer-authorized workstation adjustment also raises the memory
defaults recorded in the checkpoint above. Original independent expectations remain.
The complete seed campaign and assembled receipt must pass before measurement begins.

Use the current `just --list` contract at execution time. The initial selection is `just fmt`,
`just ci-fast`, `just quality`, `just governance`, `just adr-lint`, and `just docs`, with
`just docs-test` for any publisher/citation changes. Avoid repeating aggregate checks already
covered by a successful enclosing recipe without a reason.

Refresh the linked extension through `just py-sync-native`; exercise `just native-test` and
`just native-python <output>` for the selected native and Python scopes, `just seed-conformance`
for the full authored seed/domain campaign, and `just parity` for the affected IDAES compatibility
journeys. Use filters to select actual tests, and record those filters and omitted scopes. Missing
required native or parity prerequisites are failures, not skips or passing exclusions.

The final campaign must include real storage and solver composition, in addition to the isolated
policy checks used during implementation. New cross-plan fixtures should be registered in the
existing harnesses by their functional owners. Update a recipe if the target changes its required
feature/environment selection; do not work around it with an unrecorded long command line.

The [25g handoff](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25g-durable-contract-evolution.md#verification) supplies authored, unexecuted
storage controls in `pse-catalog/tests/native_artifact_migration.rs` and the operations migration/
retirement harnesses. K3 must exercise exact frozen-25f preservation, interrupted committed prefixes,
malformed history, reset rollback/lost acknowledgement, prior unresolved inventory, export expiry,
foreign overlapping intent protection and queued lease renewal, plus native migration lineage,
changed-map recovery refusal and actual restart/discovery/reclaim. The existing `just native-test`
workspace harness discovers these targets; `just db-test` selects the isolated operations journeys.
Do not count their earlier all-target compilation as execution.

The integrated 25h/25i/25j handoff adds the frozen `test-fixtures/plan25g/` operational
source and the appended V7 operational identity/frame transition. Focused preserving migration
controls have run against isolated PostgreSQL; K3 still owns actual restart, mixed-operation
and recovery composition. Historical digests retain their bytes and nullable frame provenance;
an unknown historical frame cannot become a current operational cache key.

Generated native-boundary fixtures under
`python/pse/tests/fixtures/generated-native-boundaries/` cover partial/cancelled outcomes,
pre-result refusal, failed attempts, incumbents and event-ended trajectories. Their isolated
Python decoding checks establish transport distinctions, not solver or storage outcomes. K3
must exercise those distinctions through the actual assembled workflows.

The 25i handoff supplies complete-context body/package reuse, nested escaped-allocation owners
and injected profile dispatch/panic controls. K4 must measure these mechanisms under actual
in-process and durable workloads; focused reuse counts are not timing or RSS evidence. K2
also owns the recorded imported `proc-macro-error2` future-compatibility advisory alongside
the zero source-warning target.

Each executed result names command, mode/features, fixture scope, conditions, failure count
against zero, and remaining exclusions. K2/K3 commands are a proposed selection, not claims
that any current command has run or passed.

### K4 — Measurements that distinguish the design

Execution refinement, 2026-10-04: untimed controls show that medium structural
preparation alone takes several minutes, while the 32-block warm-up has exceeded
fifteen minutes before its first observation. Measure the required mechanisms with
cold/warm small cases at one and four executor threads, the medium structural-edit
and specialization-edit controls, the two eight-point study controls, all five scalar
preparation/retention/worker-lifetime controls and document admission. This selected
dev-profile campaign covers reuse, invalidation, occurrence execution and ownership;
the broader size-by-thread grid remains outside its measured scope. Report this limit
explicitly and make no general scaling or production-release performance claim.

Count actual body admissions and prepared products: an unchanged body with a retained warm entry
and unchanged complete admission context must reuse its admitted product; changing one equation
invalidates its dependency closure; a span-only edit changes attribution without unnecessary
arithmetic rebuilding. Measure resident reuse separately from lawful recomputation after eviction
or denied retention. Compare an N-point durable study on one worker with equivalent in-process
execution, separating structural preparation from binding and
attempt-owned solver work. Count repeated occurrences as separate experiments even when they
share preparation.

Use the existing cache/case measurement recipes and add the necessary workload selectors to
their owners. Account for unique live allocations and escaped owners, not merely cache size or
RSS. A deterministic gated worker establishes permit lifetime; timing alone is insufficient.
Record pool limits, concurrency, cache state, source revision and force-validation mode. Performance
mode must not be substituted for correctness qualification.

### K5 — Final review and retention

Apply the selected core and process-simulator design standard to the changed boundaries. Check
ordinary scientific extension, mechanism substitution and isolated policy testing, alongside
physical/numerical gates. Judge the new target itself; neither historical review verdict proves it.

The coordinator changes a finding to resolved only when all its child obligations and required
evidence are complete. Preserve any actual residual gap explicitly rather than averaging it away.
Move enduring contracts and rationale into their architecture/ADR owners through the existing
routes; then retire completed plans and resolved reviews only when their readers are redirected.
No commit, push or publication is implied by this campaign description.

## Current K5 finding dispositions

The scheduled closeout review owns the diagnosis; this table owns its execution status.
Its original assessment remains distinct from subsequent remedy acceptance.

| Finding | Status | Correction and acceptance obligation |
|---|---|---|
| K5-F01: original-point publication assumes a native candidate | resolved | **Implemented/Tested:** authoritative original coordinates and rebound values now reach joined result tables without fabricated native fields. The three modeling-result controls and full `just native-python` pass, including the original heater/recycle and flash journeys; independent remedy assessment confirms these public consequences. |
| K5-F02: complete block admission remains retained after execution | resolved | **Implemented/Tested:** complete pre-operation admission survives all work; checked completed capacity partitions its unique owner before escape. Short-pool, excess-capacity, failed-block, panic and extracted-owner controls pass. Full `just native-python` passes the unchanged 1,000-point sweep and following capped study. Opaque payloads retain complete admission; native scratch/session ownership is unchanged. |
| K5-F03: multistart uses the source grant instead of the dispatched declaration | resolved | **Implemented/Tested:** binding consumes the enclosing admitted start rules, then the frozen explicit native profile; immutable source policy and exact identity checks survive. Actual multistart recovery and absent-source/outer-grant controls pass in the eight-control native selection; independent remedy assessment accepts this focused correction. |
| K5-F04: repeated original assessments collide in work ownership | resolved | **Implemented/Tested:** the dispatched mechanism occurrence enters the assessment charging identity. The actual chain retains three distinct assessment owners without inventing callback counts; the same-owner duplicate remains a terminal internal failure. Both controls pass in the eight-control native selection; independent remedy assessment accepts this focused correction. |

## Contextual accuracy review dispositions

The maintainer requested and authorized the broader accuracy design review on 2026-10-05.
The maintainer then authorized authoring the full CA-F01–CA-F04 follow-up in
[Plan 27](27-contextual-engineering-accuracy.md), and subsequently authorized implementation.
The findings remain scheduled until the corrective packages and assembled acceptance meet
their obligations. Implementation does not establish measured optimal tolerance values.
This table remains the sole disposition owner. The review's independent
judgment and inspected baseline remain in its principal document.

| Finding reference | Scenario | Disposition | Decision/work owner | Evidence or settling obligation |
|---|---|---|---|---|
| [CA-F01](../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md#ca-f01) | S01, S03, S06 | scheduled | [27a A0/A1/A3](27a-accuracy-intent-and-contextual-policy.md#packages-and-local-acceptance), [27c C1/C4](27c-selective-refinement-and-completion.md#packages-and-qualification-handoff); analysis/policy and completion owners | Working output/decision contract, strength and lawful no-goal operation, then independent integrated acceptance; a proposed contract is not a landed correction |
| [CA-F02](../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md#ca-f02) | S01, S02 | scheduled | [27a A2/A3](27a-accuracy-intent-and-contextual-policy.md#packages-and-local-acceptance); numerical resolution/shared policy owners | Contextual scale/default rationale independent of conditioning, inherited defaults versus explicit overrides, unit/datum/cancellation controls; provisional constants are not measured optima |
| [CA-F03](../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md#ca-f03) | S04 | scheduled | [27b B1–B4](27b-error-allocation-and-numerical-evidence.md#packages-and-local-acceptance); math/implicit/reconstruction/dynamics owners | Goal-derived allocation and valid actual evidence within the stated producer-support matrix; preserve certification and original checks; qualify assembled consumers |
| [CA-F04](../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md#ca-f04) | S03, S04 | scheduled | [27c C1–C4](27c-selective-refinement-and-completion.md#packages-and-qualification-handoff); strategy/completion/boundary owners | Quantitative outcomes, finite permitted refinement and immutable retained completion; actual threshold/resource/nonprogress/refusal controls and assembled acceptance |

## Outcome (recorded after implementation)

### Current execution checkpoint

Review checkpoint, 2026-10-05: the maintainer paused implementation/qualification for the broader
[contextual-accuracy design review](../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md).
The independent review is complete: **Revise** the architecture for the requested target; its
contextual-accuracy behavioral claim is **Not Accept** at source/literature evidence strength.
Current physical-feasibility support remains a preservation constraint. The recommendations
are now developed in the authorized [Plan 27 series](27-contextual-engineering-accuracy.md),
which is being implemented at the maintainer's request. Its
[checkpoint](27-contextual-engineering-accuracy.md#current-checkpoint) owns the current
functional state; producer integration and real native/transport validation precede
the full campaign. K3/K4/K5 remain incomplete. The
[disposition table](#contextual-accuracy-review-dispositions) owns this follow-up. Next authorized
production work follows 27's A0/C0 decision route and working prerequisites; the full campaign
remains here after its functional handoff.

Execution resumed on 2026-10-05 at the maintainer's request from clean main
`ad665a0222551196b1160e426f5242361215a6a0`, after Plan 26's completed testing-architecture
handoff. The committed conditional-support correction is present; 25n's N11 acceptance
and finding reconciliation precede the assembled campaign. PostgreSQL 18.6 reports the
current schema ready. Stale-worktree cleanup is already complete in 25n and is not pending.

Run one composite campaign using the existing recipe owners: scope-end hygiene and its
uncovered leaves, a version-5 full-native functional assessment, focused default-feature
absence and doctests, complete linked Python, IDAES parity, and the complete seed/domain
manifest. Serialize heavy execution and preserve original criteria and budgets. Repair
failures in their owners and refresh affected evidence before the required dev-profile K4
measurements. K4 selects cold/warm small cases at one/four threads, medium structure and
specialization edits at one thread, both eight-point studies, the five scalar K4 cases and
document admission (14 cases). Three additional scalar accuracy controls use the same model
with no goal, a separated decision, and active output refinement. Their existing case-measure
owner records actual work, memory, retained outcomes and independent root differences; the
selectors are prepared, not measured. K5 then assesses the stable assembled target and closes the
owning findings and records. Earlier partial receipts retain their historical scope.

The resumed public campaign now succeeds after original-point publication and completed
block-report accounting corrections. The unchanged 1,000-point flash sweep and following
capped related-root study both pass. The independent closeout assessment confirms the public
consequences of K5-F01/F02. The real automatic POUNCE second-opinion control now passes.
The enclosing native run exposed multistart grant/profile binding and repeated-assessment
charging-identity defects; their actual positive, refusal and duplicate-charge controls now
pass after repair. Callback, document-version, nested-provider and limited-native fixtures
now express their intended current paths without changing scientific criteria or budgets.
Affected static checks and the two selected public strategy/study consumers now pass.
The subsequent full native graph exercised every selected functional control; only stale
study/job codec-version assertions failed. Those controls now pass with current-version
round trips and prior-version refusal; the fresh exact-source functional graph has passed
completely. The full authored manifest completed with 113 passed seed fixtures, three failed
and two inconclusive; all six domain fixtures passed. Remaining repairs concern nested
selection proof deadlines and recycle initialization scope/provenance. The maintainer
requested a decision-relevant convergence review before further changes or probes. The
provisional edit of two steady CSTR concentration comparisons from 1e-6 to 5e-6 mol/m³
is not an accepted or qualified engineering accuracy policy. Their observed deviation
was 1.9592909978882744e-6. An independent reduction of
the authored material, caloric and Arrhenius laws reproduces the reference concentration
20.316088036339778 mol/m³ and temperature 304.085837345337 K. The
[bounded convergence review](../design_review/reviews/design_review_convergence-criteria_2026-10-05.md)
findings F01–F03 are adopted for this packet. Shared physical fallback, KKT and gap defaults
use the model-owned normalized 1e-3 engineering accuracy; discrete integrality retains its
separate meaning. The maintainer rejected per-fixture convergence tuning and selected shared
physical defaults, initially 0.1 K and 1 W, with 1 kg as a guideline for process mass analyses.
Individual authored policy slots retain their meanings and reference one shared typed policy;
distinct values remain explicit overrides. The selected models use molar inventories rather
than a mass-inventory contract. Integration global aliases inherit the normalized default,
and generated conservation quadratures share their consuming inventory allowance without
an additional automatic tightening factor. Earlier internal allocations and their selected
receipts retain their prior scope; they do not qualify this revised shared policy.
Four-mesh PFR theoretical-order ratios and their oracle declarations are deleted; two-mesh
output-stability checks replace them while focused coefficient, spacing, boundary, continuity
and quadrature premise controls remain. The provisional contractor-order experiment is removed.
Fixture expectations are independent checks, not implicit solver accuracy requests. Selected
shared-policy unit controls pass. The representative steady and controlled dynamic CSTR
journeys pass; both selected PFR journeys refuse with a conditional-block usable-original-
coordinates error, rather than a theoretical-order assertion. Production refusal evidence now
identifies local Ipopt infeasibility with valid candidate coordinates and violated original rows.
Pipeline analysis found that an optional block's numerical refusal was classified as a terminal
capability refusal at the automatic handoff, stopping its already-catalogued Direct operation.
That classification is repaired and the production event-to-selection regression passes. The
maintainer requested whole-pipeline debugging, including characterization, scoping, presolve
and admitted solver alternatives. Cross-backend Auto recovery remains a concrete analysis gap:
current native profile recovery is POUNCE-specific. Keep explicit solver choices, original
acceptance, shared thresholds and enclosing budgets intact while resolving it. No speedup or
enclosing campaign closure is established. Refresh affected conformance and exact-source qualification before the selected
K4 measurements and assembled K5 assessment.
Earlier failed receipts retain their actual scope and do not establish enclosing qualification.

### What was built

**Implemented:** original result publication consumes the actual rebound preparation and
complete reconstructed coordinates. Block completion reserves every report before execution,
then partitions its unique owner using checked capacities and conservative headroom; unknown
extensions retain their full admission. The actual unavailable cause survives workflow
assessment. Replaced assumptions and duplicate ownership are removed.

**Implemented, convergence-policy correction:** normalized ordinary design stopping defaults
use 1e-3 of a meaningful frozen characteristic scale. Authored production policy slots now
reference shared physical defaults rather than separately tuned literals. The policy's initial
temperature/duty allowances are 0.1 K and 1 W; molar flow, concentration and inventory values
account for the smaller reference models. These provisional defaults and retained overrides
are distinct from claims of process qualification. Integration global aliases reference the
same normalized default, with numeric overrides preserved; generated flux quadrature uses
its consuming closure allowance without an extra precision multiplier. Synthetic native
contract tests use explicit verification budgets. Blueprint §16.1 and §13.3 own the meanings.

**Tested, 2026-10-05, zero-test-failure baseline:** `just unit-native-selected` with CI profile,
one test thread, native-solvers and explicit force-validation passed all 23 selected tests:
model engineering defaults, math numerical resolution, native stopping/original acceptance,
implicit nominal precedence, collocation coefficients, backward physical spacing and compiled
collocation assembly/continuity/quadrature. The initially reported unused qualification in a
new test is corrected; a fresh `just unit-native-selected` selection of the nine native
numerical tests passed with zero failures and no warnings (CI profile, one test thread). This selected
pass does not qualify the enclosing scientific campaign or establish faster execution.


**Tested, shared-policy controls, 2026-10-05, zero-failure baseline:**
`just unit-native-selected` with CI profile, one test thread, native-solvers and explicit
force-validation passed all 13 selected tests: global integration inheritance with independent
overrides and canonical round trips; conservation quadrature inheritance; conservation and
state-reconstruction physical budget propagation; and the nine native numerical controls.
`just codegen --only schemas` regenerated the seven affected numerical-policy JSON schema
consumers, and `just py-sync-native` rebuilt the dev native extension successfully. These
controls establish policy wiring, not whole-campaign scientific adequacy or improved timing.


**Tested, selected shared-policy production journeys, 2026-10-05, zero-failure baseline:**
`just modeling-conformance --manifest packages/reference/conformance.toml --run seed` with
fixtures `01a0ee3de15373109faad8251c7d3424`, `7b4665c39e1748cb91118b24f3638f64`,
`f341a18f0f284413b6b20f1309e47e6d` and `8f755c37ae0341349c43cfcc02a62331` completed
732 checks: two fixtures passed and two failed. The steady and controlled integrated CSTR
journeys passed. Both PFR mesh journeys failed during solve with `unsupported: conditional
block did not establish usable original coordinates`; their physical deviations are unavailable
in this receipt. This is unresolved production execution, not evidence that the physical
thresholds should be tuned. Local reports are under
`build/plan25k-20261005/global-accuracy-conformance`; dev native extension, explicit
force-validation and native-solvers. The final authored admission was refreshed with the direct ideal-identity fixture
`01a0e169482c760bbd985bb44116c72e`: `just modeling-conformance` completed all 13 checks
and its single fixture passed. Other scientific and performance qualification is pending.

**Tested, additional shared-policy production journeys, 2026-10-05, zero-failure baseline:**
`just modeling-conformance --manifest packages/reference/conformance.toml --run seed` with
fixtures `29dd6a1a3e444acfbf14992087f9d32c`, `3bf1cbaf35734fba8b97994a915d8032`,
`01a0e17a4587742dba3702b20fe8c77f`, `01a0e17a4587742dba3702b641de44be`,
`01a0e17a4587742dba3702bab24efca9` and `a76db9a26b384861b47436d9ff4aebb7` passed
all six fixtures and 1,830 checks. These exercise the closed integrated/simultaneous vessel
trajectories and inline, nested, accelerated and bounded-cubic density solves with the shared
policy. Conditions are the dev native extension, native-solvers and explicit force-validation;
local reports are under `build/plan25k-20261005/global-policy-additional`. This selected
receipt does not resolve the PFR refusals or qualify the full manifest.

**Implemented/Tested, conditional candidate refusal evidence, 2026-10-05:** the single
block commit gate now retains the actual refusal cause, including native use policy,
termination/qualification, coordinate shape/order/finiteness and bounded original residual
witnesses. Shared callback and validation causes retain their identity. No acceptance or
tolerance rule changed. `just unit-native-selected` with CI profile, one test thread,
native-solvers and explicit force-validation passed both
`block_commit_is_atomic_and_requires_native_success_plus_original_quality` and
`conditional_failure_preserves_shared_terminal_causes_and_actual_native_status` against
the zero-failure baseline. A refreshed selection passed without compiler warnings after
correcting an unnecessary qualification in the new test.

**Implemented/Tested, automatic numerical handoff repair, 2026-10-05, zero-failure baseline:**
rejected native block candidates/quality retain a numerical cause; malformed coordinates or
advertised report metadata retain a contract cause. Callback and validation causes remain
shared. The actual event projection is used by both production dispatch and its regression,
so an abandoned optional numerical Block operation can select the existing Direct operation
without accepting or retaining its refused coordinates. `just unit-native-selected` with
CI profile, one test thread, native-solvers and explicit force-validation passed all five
selected controls: both block-refusal controls above,
`abandoned_block_native_refusal_reaches_direct_through_production_projection`,
`optional_component_numerical_failure_retains_observation_and_charges_before_direct` and
`automatic_next_preserves_empty_start_grants_and_terminal_scientific_refusal`. There were
no compiler warnings. Selected PFR and full campaign refresh remain pending this repair.

**Tested, 2026-10-05, zero-failure baseline:** `just native-python
build/plan25k-20261005/python-qualified`, after `just py-sync-native`, passed all 210 selected
unit/component/integration tests with zero failures, errors or skips in 971.03 seconds.
The original 1,000-point flash control took 695.893 seconds; the following capped related-root
control passed in 0.869 seconds. Conditions were the dev native extension, Python 3.14.7,
explicit force-validation, native-solvers and one BLAS/OpenMP thread. Current JUnit, selection,
terminal and actual extension/provider provenance are retained locally in that output.
This follows two earlier failed full runs (206/210 and 208/210 passes) and is composite
remedy evidence, not an initially clean campaign. The current full native graph, authored
campaign, K4 measurements and final assessment remain pending; these public results alone
do not establish their completion. Only the main checkout remains after the previously
authorized stale-worktree retirement.

### A mistake made and corrected

The result exporter assumed that usable original coordinates must come from a native
candidate. Complete reconstruction instead supplies a `ConstantReport`; its actual original
coordinates and rebound preparation now own publication. The repeated flash journey also
exposed retaining full construction allowances after all block reporting was complete.
The correction keeps complete admission during work and refines only the unique completed
report owner, rather than raising the pool or weakening the scientific checks.


The first shared-profile source admission found a tolerance with the correct units but the
wrong complete physical kind: heat-release `EnergyTransferRate` requires its own typed
magnitude rather than `Power`. Its shared 1 W policy is now declared in the proper kind.
The earlier runtime parameters used for structural tolerance selection were also removed in
favor of admitted shared constants. Neither repair changes the chosen convergence levels.

### Deviations from the plan, deliberate

The maintainer selected 14 dev-profile measurement cases after earlier large warm-up costs;
the broader size-by-thread grid remains a stated measurement limit. Full native workspace
selection covers the database, worker and publication recipe subsets, so those controls
will be executed once in that graph. Proposed ADR acceptance remains separate from source
conformance and functional qualification; no commit or push is part of this execution.
On 2026-10-05 the maintainer redirected work to reviewing accuracy against analysis decisions,
then authorized the correction and clarified that individual policy slots should initially
share global values, with later explicit overrides. Per-fixture accuracy tuning and automatic
inner tightening are removed from this approach. Production numerical tests check the implementation's
premises and declared behavior; they do not require a full nonlinear process simulation to
reproduce theoretical order ratios obscured by its selected solver tolerance. The provisional
5e-6 mol/m³ CSTR comparison is replaced by the coherent physical design allowance and the
contractor-order experiment is removed. Earlier manifest evidence retains its original criteria;
accuracy-policy follow-up and refreshed qualification remain owned by this packet.
