---
title: Execution performance recommendations — independent assessment and integrated target
date: 2026-10-20
tier: design
purpose: target
decision: Revise
doc_role: review
doc_topics: [design, performance, preparation, numerical-execution, workflows, diagnostics]
standard:
  core: "3.4"
  efficient-architecture: "1.0"
  process-simulator: "1.5"
---

# Execution performance recommendations — independent assessment

## 1. Decision and scope

**Decision: Revise.** Action the confirmed work-removal directions and contract corrections,
with the revisions below. Do not adopt either external review as an unchanged implementation
plan. Their strongest proposals improve the existing separation of authored meaning,
prepared mathematics, native iteration, workflow composition and publication. Several
broader diagnoses overlook mechanisms already implemented; several proposed optimizations
would weaken scientific, ownership or failure semantics if applied literally.

**Architectural fitness:** the inspected implementation has AP-07 defects, especially
broad lexical-environment construction, repeated immutable preparation, dense reconstruction
inside sparse actions, parent scans and serial completion-side work. These remain material
without a new speed benchmark. **Behavioral adequacy:** not accepted in the scoped native
control/receipt contracts. KINSOL's documented step equivalence is false, and submitted
primal initialization can be reported as not submitted. Execution refuses the unqualified
root candidate, and original assessment rejects the cone-violating candidates. The root's
native Limit does not establish physical invalidity or completed original checks; this
audit does not establish publication of an invalid candidate as a valid solution.

The recommendation is a composed target with distinct structural, mathematical,
linearization, lane and result lifetimes. It does not require a universal solver,
one model-sized fused program, one work-stealing pool, an unconditional compiled backend,
or a new authority for validation and persistence.

### Inputs, baseline and ownership

The implementation inspected is `1ee81798a9b884f3054b685ee9629e2c8d124498`.
The pre-existing untracked external inputs were preserved:

- **R1:** [Execution performance target](../../design_review_execution-performance-target_2026-10-20.md).
- **R2 summary:** [End-to-end execution performance review](../../end-to-end-execution-performance-review-2026-10-20.md).
- **R2 principal:** [Measured diagnosis and target](design_review_end-to-end-execution-performance-target_2026-10-20.md),
  including its [measurement report](../evidence/end-to-end-execution-performance-2026-10-20/measurements.md).

The [recommendation register](../evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md)
contains a position on R1 F01–F14, C01–C03 and RC01–RC05; R2 P01–P19 and RC01–RC08;
and their compound remedies, portfolio choices, target numbers and unnumbered proposals.
Adopt, revise, omit and investigate are review recommendations, not operator decisions
or implementation acceptance. Follow-up disposition belongs to a future owning plan
created from this assessment. Existing plans, ADRs and their statuses are unchanged.

The root reviewer coordinated three fresh independent assessments of preparation,
numerical execution and workflows, then inspected decisive source and reconciled their
judgments with retained observations. No product code, dependency, solver default or
scientific tolerance changed. No new simulations, product builds, profilers or services ran.

### Criteria and functional target

This is a design-tier, target-purpose review under Core 3.4, Efficient Architecture
Heuristics 1.0, Process Simulator 1.5 and the pse-arrow binding. The criteria are those
standards and the best-in-class functional target: interactive edit/re-solve, studies,
recycles, dynamics, fitting, extension, and a development loop with actual dependencies.
ADRs, pins, plans and selected mechanisms describe current design; their agreement with
a proposal is not the criterion for accepting it.

The boundary is the authoring/admission → preparation → mathematical/native execution →
workflow/result/diagnostic pipeline, including required suppliers and consumers. It is
not a census of every unit/property model, an industrial-capacity qualification, a global
optimization comparison or a host-environment proof exercise.

## 2. Evidence corrections that change the recommendations

The [measurement assessment](../evidence/execution-performance-recommendations-audit-2026-10-20/measurement-assessment.md)
and [decoded observations](../evidence/execution-performance-recommendations-audit-2026-10-20/retained-observations.json)
give the source pointers and limits. The important corrections are:

1. **R2's reruns are not established diagnostics-off runs.** Its driver unsets
   `PSE_DIAGNOSTICS`; the runtime enables ordinary observation unless it is exactly
   `off`. Absence of selected detail/capture/export is a different condition.
2. **Debugger stack occupancy is not exact on-CPU attribution.** The sampler stops
   every thread; recognizable parked-frame filtering does not establish scheduling
   state, and phase-dependent debugger perturbation is substantial. `cpu-clock:u`
   perf self share is not hardware cycle share or allocator-only cost. The GDB output
   does contain one 0.937-second registry observation.
3. **The unaccounted interval outside a workflow invocation is not entirely preparation.**
   Registry, admission and preparation are strong leads, but startup/teardown and other
   enclosing work must retain their own scopes. A fitted fixed baseline and a few root
   sizes do not establish a universal exponent or a 1,000-variable duration.
4. **Starts already have observable numerical effects.** The ten accepted-start
   configurations have separable cold and seeded native records. Ipopt exact uses
   4 → 0 iterations and 33 → 13 evaluations; POUNCE exact uses 6 → 1 factors and
   36 → 16 evaluations; Uno uses 5 → 1 iterations despite failing its receipt assertion.
   POUNCE limited memory instead uses 6 → 7 factors and 37 → 39 evaluations. These
   same-problem solution seeds do not establish benefit on arbitrary perturbed problems.
5. **Several mechanisms described as missing already exist.** These include granular
   expression inputs, schema-pointer/retained-validation paths, multi-output arithmetic
   regions, persistent native lanes and scoped teams, incremental durable readiness,
   in-process lease renewal, warm managed-primary reuse and Gauss–Newton fitting curvature.

Consequently preparation deserves high priority for cold/package-heavy paths, but
“numerics is the third lever” is not a universal ordering for warm prepared workloads.
The illustrative latency, iteration, exponent and core-scaling targets are not new SLAs.

## 3. Integrated target and preserved meanings

The existing semantic decomposition is a strength. Model/physical owners define
quantities, domains and roles; mathematical/library owners realize demanded operations;
native adapters own iteration and foreign state; workflow owners compose operations;
result/diagnostic owners expose evidence and effects. An optimization can change physical
execution units while preserving those semantic authorities.

| Lifetime / owner | Target behavior | Invalidation or completion boundary |
| --- | --- | --- |
| Authored source and checked package | Indexed lookup and dependency-aware checking; immutable package products may be reused across processes | Consumed source, imports/body/data, type/physical interpretation and complete membership dependencies |
| Registry and receiving admission | Reuse immutable declarations, schemas, domain environments and unbound templates | Actual registry/configuration/function bindings and invocation-sensitive eligibility remain current |
| Prepared mathematical structure | Indexed contributors/layouts and only demanded products; bounded joint arithmetic and named compiled modules | Body/layout/selected product/realization changes, including relevant provider and barrier premises |
| Current linearization | Canonical sparse coefficients and qualified factors/preconditioners reused for new actions | Current point, parameters, regime, provider context/visits, derivative sufficiency, scaling and method state |
| Execution lane / trial | Private compatible allocation and scratch, with adapter-specific reset and shared capacity admission | Neutral new-trial reset, failure discard, owner-thread destruction and actual child drain |
| Scientific result / detail / durable effect | Independent original assessment, then bounded selective projection and acknowledged publication | Each consumer retains its producers; queued publication is not completed durable effect |

No six-type hierarchy or universal cache is required. Ordinary domain functions and
existing owners can express these distinctions. Complete premises are scientific and
operational correctness dependencies, not a mandate to restore eliminated environment
or provenance attestation work.

### Physical semantics and well-posedness

| Element | Units / convention / validity | Semantic authority and preservation |
| --- | --- | --- |
| Variables, residuals, balances and property inputs | Authored dimensions, basis/reference conventions and property envelopes | Model/physical definitions and original assessment; native arrays never redefine them |
| Normalized coordinates and budgets | Declared coordinate/row transformations; physical target remains 0.001 in the selected engineering cases | Numerical policy plus transport; relative native norms and physical violation are distinct |
| Derivative and provider products | Declared source/order, selected branch, current context and achieved derivative capability | Mathematical/provider contract; Value-valid does not imply First-valid |
| Dynamic state and time | Differential/algebraic roles, parameter/mode segments, initial conditions and event/observation jumps | Dynamic formulation and native integration contract, including consistent initialization |
| Cone coordinates | Declared cone scaling and actual reconstructed `b−Ax` geometry | Original cone assessment, not native-reported slack alone |

Variable roles, structural matching, degree of freedom and decomposition remain prepared
model responsibilities; numerical rank/convergence is distinct. A structural block is
not proof of numeric invertibility or a selected nonlinear root. No change here authors
new physics or requalifies the entire seed bank. Existing shared model/reference checks
remain the appropriate PS-13 route for later affected scientific changes.

### Representative changes and failure scenarios

| Scenario | Required response and relevant boundary |
| --- | --- |
| S01 — value, source, import or physical-data edit | Rebind values without rebuilding unrelated structure; invalidate consumed bodies/data and negative lookups even if signatures/numbers appear unchanged |
| S02 — larger sparse model or many cheap cases | Work follows actual references, contributors and ready events; sparse actions do not require incidental dense storage or all-parent scans |
| S03 — provider, derivative order or branch changes | Current validity precedes reuse; domain/first-error behavior and unsupported derivative requests remain explicit |
| S04 — native adapter/method replacement or warm restart | Consumed norms, reset, maps, scaling, starts and assurance are explicit; the adapter absorbs native mechanics |
| S05 — skewed capture, cancellation or uncertain publication | Other legal work can progress; issued producers/native tasks drain, charges remain, and durable ACK is not inferred |
| S06 — new unit/workflow or focused test | Extend semantic definitions/composition; do not copy solver/validation policy or require unrelated storage/native machinery for a pure responsibility |

## 4. Findings and corrected implementation directions

Findings below describe current source, **Implemented/source-inspected**, except where
an existing measured observation is named. Their remedies and speed benefits are
**Proposed**. Each numbered obligation can close independently even when the paragraph
groups a coherent responsibility. Approximate affected surfaces identify semantic owners,
not a mandated new crate/service/API.

### Admission and immutable preparation

<a id="af01"></a>**AF01 — eager lexical environments multiply unrelated name resolution.**
For many declarations, `CheckedPackage::named_types_with` enumerates visible names and
repeatedly invokes owner/import/child resolution. Admission-time evaluation can rebuild
those environments again. R2's flat-package cubic mechanism is credible from the nested
work; exact growth for every package remains unmeasured. AP-07, DP-10 and S01/S02 apply.
Make lookup lazy over existing visibility/shadowing semantics, index missing lexical
access paths and retain qualified spellings. Cache immutable bases with current phase,
type/entity/extent overlays. A candidate-local owner-only memo can still become stale.
Enumeration consumers retain complete indexed populations.

<a id="af02"></a>**AF02 — structural environment/equality and retention units remain broad.**
The compiler already has granular Source/signature/expression queries, and
`Namespace::new` strips substantial non-lexical content. The defect is not “every edit
invalidates everything.” Broad structural environments and deep equality still amplify
some edits and compete with bounded query retention. AP-07, DP-09 and S01 apply.
Extend tracked consumed dependencies and structural sharing, including absent names and
ordered candidates. Tune the useful working set by bytes after reducing representation
cost. Imported signatures alone do not cover bodies, defaults, tables or physical meaning.
Persistent immutable checked artifacts are a worthwhile target, with complete premises,
current receiving shells and a fresh-check fallback; native Salsa persistence remains
an alternative to qualify, not a prerequisite. Enabling Salsa `rayon` is not what makes
parallel query execution possible. Parallel read-only phases must respect current
source-order and mutable candidate assembly.

<a id="af03"></a>**AF03 — immutable registry descriptions and bootstrap work are repeated.**
Registry build, ExpectedContract capture and closure descriptions repeat canonical
semantic work; native materialization forces an already-lazy bootstrap before checking
whether fallback is needed. AP-07/DP-10 apply. Reuse per-registry descriptions, normalized
SQL and indexed invariants; defer the actual bootstrap force. Generate immutable
production-catalog products from their existing declaration where useful, leaving custom
registries and live implementation caches intact. Hash-of-member-hashes would change
the current canonical graph preimage: preserve existing fingerprints by reusing/streaming
their existing representation, or treat a separately chosen identity change explicitly.

<a id="af04"></a>**AF04 — nested validation preparation repeatedly constructs domain environments.**
`occurrences::Node::prepare` reaches `field_local`, which reconstructs the enum-domain
map. AP-07/DP-10 apply. Share the resolved current environment through the nested tree
and extend existing immutable validation templates. Schema-pointer identity, contract
handles, checked batches and retained local contracts already establish different
properties; do not invent another generic trust handle. Registry schema identity does
not validate a newly supplied array. Receiving engine/function/configuration and
invocation-sensitive checks remain real. A typed loader is an investigation after these
smaller corrections, consuming the same registry rules and actual SQL obligations.

<a id="af18"></a>**AF18 — typed declaration masking performs a JSON round trip.**
`check/phases/expressions.rs::Environment::new` serializes, masks and restores typed rows.
This is a concrete internal representation crossing under AP-07/DP-10. Replace it with
a typed/generated projection preserving parser-owned field roles, signed zero and
unknown-field conservatism. This does not justify deleting external JSON payloads or
semantic witnesses with different contracts. Interned IDs help lookup but do not fix
eager scans; an allocator comparison follows material work removal rather than replacing it.

### Mathematical products and executable granularity

<a id="af05"></a>**AF05 — conditional block preparation repeatedly searches parent populations.**
`CasePlan` conditional capability/allocation and request ownership paths, plus runtime
block-boundary preparation, rescan parent instances/rows/columns. AP-07/DP-10 and S02
apply. Retain contributor/request-consumer/ordinal indexes once. Keep source order,
multiplicity, fixed boundary coordinates and all capability/allocation obligations.
Separately qualify physical grouping: BTF remains semantic evidence, while grouped
nonlinear execution must preserve actual per-block start/root/branch permissions.

<a id="af06"></a>**AF06 — sparse implicit and derived actions reconstruct columns and dense scratch.**
`implicit_products::demanded_derivatives` builds an n×n tangent through n unit directions
before sparse extraction; matrix-bit equality is checked after reconstruction.
Derived-family paths also materialize unit columns and rescan patterns. AP-07, DP-09,
DP-10 and PS-07 apply. Assemble canonical sparse entries directly or use qualified CPR
compression over conservative support. Retain a complete current linearization association
before skipping reconstruction; action weights/directions then rebind without becoming
an unbounded cache. Zero-seed traversal is not assumed to visit every consequential child.

<a id="af07"></a>**AF07 — sparse HVP construction is excluded by a complete-support gate.**
The flat direct weighted-Hessian construction route in `products.rs` admits directional
construction through a complete-triangle condition. AP-07/PS-07 apply where sparse
directional consumption is large. Make direct
actions eligible on sparse support using library mathematics; retain entry realization
for entry consumers or better preparation/evaluation economics. Existing reverse/multi-output
paths must not be described as universally absent.

<a id="af08"></a>**AF08 — demanded scalar compiled functions have separate compiler/linker lifecycles.**
`native_compiled::compile_prepared_with` creates per-evaluator compilation work. Exact
Symbolica 3.0.1 named exports and `load_new_function` support shared modules with independent
scratch. AP-07/DP-10 apply. Pack bounded co-used, already-demanded functions; preserve
child cancellation/drain and loaded-code retention. This does not require broad fusion,
JIT or compiling every order. Persistent scalar artifacts can be a derived local accelerator
under actual ABI/layout/compiler/ISA/FP compatibility. Default compiled execution remains
a cold/warm selection question.

Existing arithmetic regions and `evaluator_multiple` already perform joint work. Broader
fusion cannot move guards “afterwards”: finite arithmetic, Require, branches, applicability
and provider effects have ordered observable failures. `sqrt(0)` can be Value-valid but
not First-valid. Reuse or fusion must preserve the demanded operation, first refusal and
effect scope, rather than evaluating speculative invalid work because f64 does not trap.
Preserve the exponent-materialization workaround in `typed::unary_checked` unless its
replacement is qualified at actual Symbolica 3.0.1: normalization can discard a consumed
dependency. Unchanged fingerprint encoding retains byte identity; arithmetic regrouping
needs its declared numerical, guard and diagnostic equivalence, with bit identity only
where actually promised.

### Scheduling, capture and resources

<a id="af09"></a>**AF09 — outer batch grouping has less compatibility information than native dispatch.**
The workflow groups by backend, then the numerical owner checks complete profiles,
threads and composition before a serial fallback. AP-02/AP-07 and S02/S04 apply.
Expose conservative compatibility from that owner and partition before dispatch;
keep final native safety checks and member assessment. This source path does not by
itself explain homogeneous four/sixteen-point timing results.

<a id="af10"></a>**AF10 — the local frontier revisits ready-but-capacity-blocked occurrences.**
`study_execution` constructs pending candidates and checks readiness before capacity.
AP-07 and S02 apply. An ordered ready queue plus indexed waiting membership removes
unrelated repeated visits while preserving continuation priority, retries and cancellation.
This local defect must not be generalized to canonical readiness: durable notification
hints, bounded outgoing pages, repair cursors and exact immediate-premise claims already exist.

<a id="af11"></a>**AF11 — completion-side Compact capture suspends coordinator polling.**
`record_execution` awaits selected cursor exhaustion before processing/refilling other
completed lanes. AP-07/DP-20 and S05 apply. Move capture/close into the existing finite
lane future, retaining its RunResult/source/producer/allocation until terminal return.
Return mandatory completion and minimal continuation products; close or park independent
lanes while preserving an eligible private owner for unique-successor native reuse and
the existing fan-out seed semantics.
The last nonempty chunk is not terminal success. Long synchronous projection may still
need isolation, but spawning all lanes is not a prerequisite and does not automatically
inherit population, invocation or activity context.

Persistent native lanes, neutral resets and scoped teams already exist. `FuturesUnordered`
does not make separately dispatched native threads serial. Remaining preparation-job
thread churn, member pre/post work and immutable admission sharing deserve bounded
improvement; they do not establish the need for a generic work-stealing pool. Share the
immutable admitted basis, not mutable lane permissions or provider state. Thread-affine
foreign objects retain owner-thread construction/destruction and scoped capacity.

Computed class/coefficient/validation/rebind bounds and growth already cover several
paths. Extend actual unsupported producer families before reducing opaque prepayment.
Reserved stacks/capacity and RSS are different; no accounting-to-RSS ratio is adopted.
Memory waiting must not retain unrelated CPU/exclusion capacity. Width equal to all
cores is not selected without a useful accepted-throughput/memory comparison.

Durable workers refill productive lanes immediately; 500 ms sleep is an idle-discovery
choice. Lease heartbeat is an in-process task, distinct from periodic supervisor observation.
Warm primary reuse already exists. Investigate remaining discovery/claim crossings and
push wakeups as hints with resubscription/cleanup and authoritative fallback. Shared
local/durable bulk dispatch is a useful extension, preserving each occurrence's lease,
fence, start and ACK. A separate commit stage is conditional on observed blocking and
bounded pending products; no queued job is a completed durable result.

### Native semantics, dynamic reuse and fitting

<a id="af12"></a>**AF12 — native working controls are not consistently related to their claimed meaning.**
There are two separately closing obligations under AP-02/AP-05, G2 and S04:

- **AF12a, KINSOL:** `KINScSNorm` divides the step by `1/uscale + |u|`.
  The documented reduction to an absolute physical coordinate budget is false.
  The max-step API is already called. Correct the semantics and qualify a native
  stagnation/max-step policy; preserve residual convergence and independent original
  checks. Do not append a custom Newton polish or rename a Limit as convergence.
- **AF12b, cones:** Clarabel `tol_feas` is relative, while original `b−Ax` cone
  violation is physically checked. The observed SOC/PSD failures remain failures.
  Justify native working tolerances across normalization, magnitude, recovery and
  rounding, or use finite native retries with fresh checks. No one-retry guarantee
  follows. POUNCE-convex has different absolute/gated-relative stopping and rechecks;
  the same defect is not established there merely by its scalar tolerance setting.

Native convergence, stationarity, gaps, local-error controls, stagnation and effort limits
are different concepts. The target is their correct relation to original acceptance,
not describing all native stopping as stagnation or globally tightening 0.001.

<a id="af13"></a>**AF13 — start receipts conflate submitted initial primal and optional warm payload.**
`Pipeline::new` retains the primal in `initial` while optional `warm` can be absent.
Several adapter flags derive submission from that optional value. AP-04/G7 and PS-11
apply. Report source seed, transformed primal submission, optional dual/barrier/working
state, selected restart and actual internal state reuse distinctly. Do not manufacture
duals or diagnose ignored initialization. Same-structure Ipopt reuse requires retained
maps and value-dependent scaling qualification; numeric-option mutability is an actual
native contract, not a name-based partition. A retained Uno handle needs its own reset.

<a id="af14"></a>**AF14 — compatible dynamic trials reconstruct native allocation and callback intermediates.**
Shooting retains a DynamicWorker but fresh IDAS sessions and callback vectors can still
be created for each window/trial. AP-07/DP-10/PS-11 and S03/S04 apply. Retain compatible
capacity and checked borrowed inputs/scratch, with a complete new-trial reset for state,
rate, parameters, mode, consistent initialization, sensitivity/corrector, quadrature,
adjoints/checkpoints, counters and failure latches. Failed mutation discards the owner.
Generic `IDAReInit` does not promise symbolic KLU retention: native initialization forces
first-factorization outside the explicit fixed-pattern mode. Qualify that mode separately.

The dynamic owner already has a pure-function time/state/parameter cache with derivative
sufficiency after current guard binding. Improve its upgrades rather than introducing
a weaker `(t,y)` cache. Trial-point guards remain; accepted-step-only checks are unsafe.
Hoist invariant CSC/scatter structure and coalesce statistics without losing needed
progress/work/partial evidence. Backward observation impulses require correct adjoint
jump/history and DAE consistency; pure-ODE improvements do not justify removing every reset.

<a id="af15"></a>**AF15 — fixed-trial anchored shooting windows use one serial worker.**
The trial supplies independent window anchors; continuity constraints couple returned
products rather than requiring sequential integration at that trial. AP-07/DP-20 and
S02/S03 apply. Use bounded private lanes and ordered reduction, preserving valid previous
cache state on failure and charging live checkpoints. Ordinary single shooting and
initial-guess anchor construction can remain sequential.

Fitting already supplies Gauss–Newton `JᵀWJ` and preserves nonlinear constraint curvature;
exact curvature serves optimization as well as selected uncertainty analysis. Keep the
curvature portfolio and demand-specific trajectory upgrades. Always integrating sensitivities
on value-only rejected trials can add work or reject otherwise eligible work. A residual
LM/TR route is an investigation for genuinely eligible fits, compared with current Ipopt-GN.
Forward/adjoint selection considers demanded axes, observations, events, checkpoints and
derivative products. Reuse assessment allocations without substituting cached producer
results for independent original checks.

Anderson is already an available native recycle capability. A new default, history size,
Wegstein/Broyden route or Newton tear formulation needs a qualified map/derivative/root
contract and complete work comparison. No universal 3–5 history or identical-root promise
is adopted. Indexed port exchange is independently useful. Sparse-direct/Krylov, dense tiny
groups and native linear backends remain workload choices rather than hard size cutoffs.

### Observation, representation and additional improvements

<a id="af16"></a>**AF16 — ordinary callbacks repeat shared observation updates and formatting.**
Prepared facts and detailed boundary gating already exist, but ordinary phase/count/time
updates still incur shared-state/clock work; KINSOL rich residual events have other progress
consumers. AP-07/DP-21 apply. Keep exact work admission and panic reconciliation separate,
publish coherent aggregates with an explicit cadence/final flush, and compact rich payloads
according to all consumers. Pre-work and post-success stop checks protect different cases;
do not remove the late-result veto or panic boundary to meet a microsecond target.

<a id="af17"></a>**AF17 — quality-row recovery performs repeated linear ID lookup.**
`transport::report` searches `contract.rows` for each returned quality row. AP-07/DP-10
apply. Retain an ordinal map or validate complete positional correspondence once, preserving
subsets/reordering and unknown IDs. This is independent of the larger numerical work and
does not require claiming it dominates today's small cases.

The following additional design items refine the target beyond the external remedies.
N02/N03 extend defects already observed by the reviewers; they are not counted as new
independent causes of those same losses.

<a id="n01"></a>**N01 — Jacobi setup requests a full Jacobian to consume its diagonal.**
`dynamics/idas.rs::precondition_setup` calls `partials` then searches each column for the
diagonal. Under AP-07/DP-10/PS-07, a qualified diagonal/selected-entry demand can avoid
unconsumed products and scans when no other current consumer needs the full matrix.
The target is consumer-specific product admission, preserving all-mode support, provider
effects, actual shifted diagonal and existing fallback semantics. A stronger block
preconditioner may need more entries; it must not force the cheap Jacobi route to do so.
Verify identical admitted preconditioner actions and original trajectories with fewer
constructed entries, including absent diagonals and parameter/mode changes.

<a id="n02"></a>**N02 — admitted phase retirement is coupled to lossy optional updates.**
`PhaseGuard::drop` uses the same `Node::update` try-lock route as optional history.
Losing closure can leave an active phase until terminal operation and consume the finite
active-phase population. AP-05/DP-21 and S05 apply. Reliably retire admitted current state
independently from optional history retention; represent lost history explicitly. A finite
SPSC buffer alone cannot promise losslessness under a stalled consumer. Verify contention,
overflow, thread exit and operation termination without false current activity or leaked
phase capacity. Do not make optional pressure block callbacks indefinitely or reject science.

<a id="n03"></a>**N03 — stable progress identifiers are formatted inside repeated residual callbacks.**
KINSOL formats semantic coordinate IDs into String-keyed events at each residual.
These events can feed current/history/durable progress even without selected detail.
AP-07/DP-10/DP-21 apply. Prepare stable IDs/slots once and update compact values; materialize
rich views only for actual consumers. Also avoid republishing invariant unavailable-CPU
status every time. Verify history-zero current progress, durable taps and selected detail
retain their meanings while repeated formatting/shared updates disappear.

<a id="n04"></a>**N04 — aggregate review publication retains every extracted scalar row.**
`scripts/pipeline_review.py::publish` builds a Python dictionary list before writing CSV.
The existing index records **6,550,481 rows**, and the CSV is **1,176,406,434 bytes**.
These are decoded output facts, not newly measured peak memory. Under AP-07/DP-10, stream
source-addressed rows to a temporary output with a running count, then publish the coherent
index/output; keep explicit errors and nulls. Preserve distinct original samples and
label duplicate projections rather than merging them as independent observations.
Optional partitioned/typed analytical views can complement the unchanged raw source facts.
Verify stable rows/pointers/counts and failure publication with working memory bounded by
the current decoded input plus writer buffers. This is an offline review-tool improvement,
not a claim about native solve latency, and has lower product priority.

## 5. Library fit and credible alternatives

Exact consumer contracts were checked, including Salsa 0.28.4, DataFusion 55.1.0,
Arrow 59.3.0, Symbolica 3.0.1, faer 0.24.4, vendored SUNDIALS 7.1.1 via sundials-sys
0.6.2, Clarabel 0.11.1, the selected POUNCE/FERAL fork, and Ipopt's current sequence
bridge/source. The shared Symbolica capability reference is 3.0.0; relevant named
export/load behavior was checked at actual 3.0.1. Upstream-family PETSc facilities
do not establish that the selected project build/PC vocabulary already exposes them.

| Question | Simplest credible route and alternative |
| --- | --- |
| Lexical/validation reuse | Existing indexed semantic owners/templates first; avoid another resolver or trust hierarchy |
| Persistent preparation | Complete immutable artifacts versus whole Salsa persistence; keep receiving ownership and format compatibility explicit |
| Sparse linearization | Library selected entries versus compressed directions; faer/native sparse factors own algebra |
| Compiled functions | Symbolica named exports/loader and bounded modules; no bespoke loader or requirement for JIT |
| Fitting | Existing constrained Ipopt-GN/exact/limited-memory portfolio versus a qualified residual LM/TR method |
| Native state | Existing adapter-specific retained lanes and neutral resets versus newly qualified update/reset or isolated-process route |
| Strong DAE preconditioning | Existing native callbacks/library factors, with direct KLU as comparison; field-split alternatives require integration and qualification |
| Scheduling/telemetry | Existing finite lane/coordinator and explicit lifecycle state before another framework, queue or generic pool |

Library eligibility is not restricted by an existing consumer or license category.
Adoption still needs semantic fit, complete ownership and a credible operation. No
dependency upgrade or new crate is required merely to recommend these directions.

## 6. Priorities, prerequisites and settling work

Consequence priority differs from order. Native control interpretation and receipt truth
are early corrections because comparisons and user-visible outcomes depend on them.
Cold/package-heavy execution strongly motivates AF01/AF03/AF04/AF18. Warm numerical
work motivates AF05–AF07 and AF14–AF15 independently; those should not wait for a
whole persistent compiler or universal pool. AF09/AF11/AF17 are bounded independent
corrections. Reliable phase retirement helps future attribution before increasing span volume.

| Direction | Prerequisite that matters | Minimum distinguishing evidence for later implementation |
| --- | --- | --- |
| Lazy lookup / granular edits | Exact lexical/import and changing phase semantics | Independent shadowing/absent/import/quoted-name outcomes; value/source/data edits versus clean recomputation; examined-reference counts |
| Registry/domain/template reuse | Existing fingerprints, actual receiving/function/invocation binding | Same canonical identities, custom bootstrap fallback, malformed/nested/null inputs and different receiving scopes |
| Sparse products / factor reuse | Conservative support; complete current child validity before reconstruction elision | Direct entries versus actions, changed provider/regime and zero/subset requests; sparse memory and factor/construction counts |
| Block grouping / compiled modules | Branch/start policies; demanded co-used functions and barriers | Same original checks and permitted roots, fewer parent visits/setups/compiler children; cold/warm total cost and bounded retention |
| Study compatibility / frontier / capture | Numerical-owner legality; exact producer completion | Compatible/incompatible pair; ready-visit counts; skewed producer held after last chunk with terminal error/cancellation |
| Native tolerance / starts | Correct native norm, maps/scaling/reset and original assessment | Short shifted/scaled roots, magnitude-varied cones, perturbed NLPs; actual attempts/status/work, no relaxed physical endpoint |
| Dynamic reuse / preconditioning | Complete new-trial reset and actual shifted operator | Failure→retry, sensitivity/mode changes, generic/fixed-pattern KLU counts, coupled/singular-block cases and original trajectory checks |
| Fitting / shooting | Actual constraints/loss/product demand and fixed-trial independence | Few/many selected axes, nonzero residual curvature, two skewed anchored windows with failure; integrations and admitted checkpoints |
| Resource/diagnostic/publication changes | Valid producer bounds and current-state/ACK meanings | Safe admission, coherent final/current snapshots, overflow/terminal loss, finite uncommitted products and streamed output equivalence |

Start with existing short cases and source counters. One exploratory observation can
settle a mechanism; only a timing-dependent selection warrants repeated matched trials,
such as five alternating observations with actual medians/ranges. Keep scientific work
around the existing 60-second cooperative budget with actual drain. Intrusive long sampling,
large 2,048-variable sweeps, a full 2,000–5,000-equation flowsheet and platform exploration
are follow-up work only when they answer a remaining decision. No exact SLA or significance
threshold is introduced.

## 7. Architectural and scientific judgments

Judgments are scoped to the inspected pipeline, not the full scientific model inventory.
Successful prior controls do not establish unexamined behavior or a proposed target's implementation.

| Foundation | Judgment and basis |
| --- | --- |
| AP-01 separation | Satisfied in the inspected semantic/math/native/workflow ownership; optimization should preserve it rather than create shared mutable services |
| AP-02 contracts | Violated narrowly by native-control meaning and start-submission evidence; coarse batch compatibility also hides a consumed capability |
| AP-03 composition | Satisfied in the inspected workflow/math/native composition; no evidence that a universal pool is necessary. New grouping/commit routes remain unqualified |
| AP-04 domain authority | Violated narrowly where primal submission and optional warm payload are conflated in reported behavior; physical/model authority otherwise remains explicit in inspected paths |
| AP-05 explicit constraints | Violated in the claimed native step interpretation; admitted phase completion also needs a deliberate reliable-current-state contract |
| AP-06 local reasoning/testability | Satisfied for inspected semantic, mathematical and native owners and bounded tests; compiled/persistent/reset extensions require explicit dependencies. Whole-workspace isolation was not requalified |
| AP-07 execution fit | Violated by AF01–AF11 and the other identified incidental work under the functional target; exact speed magnitudes remain Proposed |

| Gate | Scoped judgment |
| --- | --- |
| G1 authority | Pass in inspected paths: model/registry declarations retain authority; derived duplicate work is not automatically a second authority |
| G2 fidelity | Fails for the KINSOL promised step interpretation; reject fusion/guard proposals that would introduce further semantic loss |
| G3 validity | Unresolved for lifecycle-current-state validity because admitted phase closure can be lost (N02). Ingress/provider/original checks are source-inspected strengths; whole-model validation was not requalified |
| G4 hidden behavior | Pass in inspected paths: observation, native work and publication have explicit effect owners. Incomplete attribution is a diagnostic limitation, not by itself an undeclared effect |
| G5 consistency/recovery | Pass for inspected producer/ACK/drain contracts; broader pool/reset/commit alternatives remain unqualified, not presumed equivalent |
| G6 transformation/reuse | Pass for inspected conservative matrix/provider and receiving validation paths. Proposed reconstruction elision, caching and grouping need their listed premises before acceptance |
| G7 truthful claims | Fails for submitted-start reporting; experimental labels/attribution require the corrections in section 2 |
| G8 library leverage | Pass in inspected owners: mathematics/iteration/factors are library-owned; no reason to implement generic Newton polish or convergence algorithms locally |
| G9 architecture | Fails independently through the scoped foundation violations, especially AP-07 |
| PS-G1 / PS-G2 | Pass for inspected physical interpretation and structural-admission boundaries; full unit/property conformance and all well-posedness cases remain unexamined |
| PS-G3 | Unresolved at whole scoped-route qualification level; original root/cone rejection is safety evidence, not proof of an invalid solution publication or adequacy of a proposed replacement |

These qualified strength/limit statements do not convert missing scientific qualification
into passing gates. Behavioral adequacy is not accepted overall, and G9 independently
requires revision.

## 8. Rule impacts

Rules change only after operator confirmation in subsequent plan creation. These rows
distinguish a selected correction direction from a conditional capability/default
investigation; neither authorizes an unknown winner or product implementation.

| ID | Current rule / location | Proposed change and dependencies | If current rule is retained |
| --- | --- | --- | --- |
| RC01 | Numerical policy §16.6 KINSOL step equivalence | Correct relative stagnation versus physical acceptance; AF12a; R1-RC01/R2-RC03 | Correct the false promise even if native tuning remains conservative; no claimed absolute-step equivalence |
| RC02 | Mathematics §14.4 no persistent Salsa store / scoped admission reuse | Permit complete immutable checked-artifact acceleration; qualify storage granularity and receiving shells; AF02/R2-RC01 | Implement lazy/granular in-process work removal; fresh processes still check packages |
| RC03 | Ordinary per-return freshness/current-progress and evidence contract, blueprint §24.1 | Define coherent coalesced cadence, reliable admitted lifecycle retirement and final flush/loss meanings; AF16/N02/N03; R1-RC03/R2-RC08 | Preserve current freshness, limiting work to local formatting/update efficiency and reliable closure |
| RC04 | Mathematics §14.3 actual matrix identity before compatible factor/first-product upgrade | Permit pre-reconstruction reuse only with complete current linearization validity; AF06; R2-RC04 | Sparse construction/coloring still helps; reconstruct coefficients before factor reuse |
| RC05 | Numerical execution's explicit unsupported coupled index-1 DAE BlockFactor/Schur capability | Qualify an actual coupled approximate preconditioner; R1-C01/P15/R1-RC04/R2-RC07 | Keep direct KLU and existing None/Jacobi Krylov choices, including N01 demand improvement |
| RC06 | Mathematics §14.5 and ADR-0191/0193 scalar realization/platform scope | Investigate separately qualified vector/JIT/device realization and any selected default change | Retain scalar execution and bounded module packing; no claimed SIMD/device capability |
| RC07 | Numerical execution §18.3 Ipopt `warm_start_same_structure=no` | Investigate restricted reuse with complete maps/scaling/reset premises; P13/R2-RC05 | Refresh native spaces; still use truthful compatible primal/dual seeds |
| RC08 | Numerical execution §17.4 current damping/Anderson defaults | Qualify a changed explicit recycle policy/default; P14/R2-RC06 | Current defaults remain; existing acceleration can be selected and indexed exchange improved |
| RC09 | Native process/thread/global exclusion deployment contracts | Investigate process-isolated/concurrent capability for a concrete workload; R1-RC05 | Preserve MPIUNI, scheduler/TPI gates and actual native owner drain |
| RC10 | Native execution §18.8 thread-affine persistent lane and neutral-reset contract | Conditional extension for additional adapter resets or broader worker placement; AF14/P05/P17 | Reuse only currently qualified lanes/scopes and optimize local preparation work |
| RC11 | Schema/relations §4 and receiving admission bindings | Conditional generated typed authored-document path retaining identical registry/SQL/invocation obligations | Reuse existing templates/domain/retained-contract paths; no second validator |
| RC12 | Workflow/result §19, publication §20 and durable §24.1 joined completion | Conditional compute/commit lifetime separation with bounded pending products and unchanged ACK truth; C03/P07 | Keep joined publication and improve compatibility/dispatch and actual crossings first |

Two important external omissions are **not adopted** as target changes: hash-of-hashes
would change identity framing, and broad guard-after-fusion would change ordered barrier
semantics under mathematics §14.3/ADR-0193. The corrected target preserves both contracts.
Loading persisted artifacts likewise does not transfer an old receiving owner's eligibility.
Any later proposal changing those meanings needs a concrete additional rule decision.

## 9. Verification, coverage and disposition

**Implemented/source-inspected:** decisive consumer and exact-library paths described
above, including contrary evidence for existing incrementality, validation fast paths,
native pools, durable readiness/renewal, GN curvature and reset restrictions.
**Measured, historical named conditions:** the original short diagnostic campaign and
R2's single-observation replay/profiler evidence, interpreted within section 2's limits.
**Executed in this audit:** read-only decoding of ten accepted-start configurations,
seven root replay records, five debugger sample populations, registry breakpoint output
and the aggregate publication index. These are recomputed observations of existing runs,
not newly executed science or a before/after performance result.

The principal source routes for reproducing the judgments are:

| Responsibility | Decisive source owners |
| --- | --- |
| Lexical checking and masks | `crates/pse-modeling/src/check.rs`, `check/phases.rs` and `check/phases/expressions.rs` |
| Registry and validation | `crates/pse-schema/src/fingerprint.rs`; `crates/pse-relations/src/validate/occurrences.rs` and `predicates.rs` |
| Sparse products and compilation | `crates/pse-math/src/implicit_products.rs`, `products.rs` and `native_compiled.rs` |
| Local versus durable scheduling | `crates/pse-runtime/src/workflow/study_execution.rs`, `crates/pse-runtime/src/math/staged.rs`; `crates/pse-operations/src/canonical_studies.rs` |
| Native controls, storage and fitting | `crates/pse-backend-native/src/kinsol.rs`, `dynamics/idas.rs`; `crates/pse-runtime/src/workflow/fitting/oracle.rs` and `experiments.rs` |
| Observation and review publication | `crates/pse-runtime/src/observation.rs`, `crates/pse-diagnostics/src/observation.rs`; `scripts/pipeline_review.py` |

Paths abbreviated within a row share the immediately preceding crate's `src` directory.
These are navigation routes to the consumed contracts, not an exhaustive source manifest.

**Not run:** new solver simulations, prototype optimizations, dependency builds, full
hygiene/integration/parity/feature qualification, large industrial models and longer-horizon
sampling. Existing evidence/source settled the present diagnoses; remaining method,
default and lifecycle selections have explicit short settling questions in section 6.
No speedup from any proposed remedy is claimed. Scientific coverage is not inferred from
the original 41 controls or successful pack readback.

**Tested, documentation only:** numbered-recommendation coverage, finding anchors,
local review links, extracted JSON parsing and publication asset selection passed.
`just turn-end` and the documentation-mode `just docs` both exited zero against a zero
failure baseline; the latter published 521 chapters. These checks establish document
consistency/publication, not product or scientific qualification.

The external reviews, their measurements, original campaign and pre-existing plans
remain unchanged. The principal audit and supporting register are assessment records;
adopted finding status and operator decisions move to a future implementation plan.
This review's final position is **Revise** the execution design through the bounded
corrections and explicitly conditional investigations above.
