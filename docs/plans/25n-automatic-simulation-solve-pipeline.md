---
title: "25n: Automatic simulation and solve pipeline"
status: in-progress
date: 2026-10-04
adrs: [ADR-0154, ADR-0155, ADR-0156, ADR-0157, ADR-0158]
review_sources: [docs/design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md, docs/design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md, docs/design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md]
scenario_sources: [docs/design_review/design_principles/binding/pse-arrow.md#pse-s01, docs/design_review/design_principles/binding/pse-arrow.md#pse-s02, docs/design_review/design_principles/binding/pse-arrow.md#pse-s03, docs/design_review/design_principles/binding/pse-arrow.md#pse-s05]
---

# 25n: Automatic simulation and solve pipeline

## Purpose, baseline and ownership

Build one pipeline that derives admissible mathematical decompositions, demands their products,
resolves native profiles and performs bounded numerical execution from a simulation problem.
**The maintainer selected automatic composition as the normal behavior for `Auto`.** Explicit
backend, profile, scientific branch, tear, mechanism and start requirements remain constraints.
Direct simultaneous execution remains a valid resolved composition, not a separate legacy path.

This plan organizes the work proposed by the
[automatic-pipeline review](../design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md).
It owns AF-01–AF-08 dispositions, the target decisions and packet progress below.
[25m](25m-integrated-solve-pipeline.md) retains its earlier finding/evidence ownership and links
the missing method obligations to their corrections here. [25k](25k-integrated-qualification-and-closure.md)
remains the only full qualification/measurement/closure owner. The series
[coordinator](25-design-remediation.md) links these owners rather than maintaining their statuses.

The production baseline is committed `main` at
`58b700bc36d59517d77886496c6ad4072ed4a560`. At authoring, the review and its navigation/checkpoint
edits were the only uncommitted changes. They are preserved. Core 3.3 and ProcessSimulator 1.4
govern this plan. The source review and focused compiler/runtime/native advice support the
foundation assessment below; no production source, dependency pin or scientific receipt changed
during authoring. The maintainer authorized implementation after approving the detailed execution approach.
Production changes proceed in N0–N11; assembled qualification remains owned by 25k.

Retain the applicable T1–T10 and M1–M18 target, including M1b, from
[25m's contracts and method placement](25m-integrated-solve-pipeline.md#combined-target-and-contracts).
This plan supplies the missing consumers and automatic composition, not a new numerical engine.
No exhaustive solver tournament, universal-fastest claim, generic workflow language, new
mathematical IR or new workspace crate is required. Libraries continue to own numerical
iteration, globalization, factors, integration controllers and surrogate model management.
Scientific workflows retain statistical, control, time/event, occurrence and durable policy.

### Existing foundations and required changes

| Foundation | Current source basis | Decision for this plan |
|---|---|---|
| Authored science and original case | Compiler `CasePlan`, physical rows/roles, provider demand, immutable binding and original assessment | Preserve equation authority and original meaning; derive complete numerical, execution/validity and coupling projections |
| Contextual native routing | Original/derived class evidence, pending readiness and adapter-owned profile requirements | Reuse these operations beneath composition; numerical history never becomes a capability blacklist |
| Strategy execution | `PreparedSolve::with_strategy`, finite driver, injected assessment, scopes and retained workers | Replace caller/direct-only dispatch with one next-decision binder; preserve explicit declaration as a constraint |
| Accuracy | Actual reduced/root refinement and `ReconstructionAccuracy`; preparation has no point certificate | Replace global rung admission demands with operation input/output/consumption contracts |
| Reconstruction | `ReconstructionOracle`, `DerivedFamily::bind_reduced`, selected residual binding and admitted implicit factories | Add a compiler-derived composite supplier; do not force arbitrary blocks into one `RegimeFactory` |
| Native methods and reuse | Actual KINSOL JVP/block factors, FERAL retention, Ipopt sequences, Uno, PETSc and egobox state | Preserve existing objects and scopes; complete M8/M17/M18 routes and actual observation |
| Public/durable boundaries | Rust-owned generation, version-first readmission and distinct source/request/preparation identities | Version changed execution meaning; preserve historical bytes and refuse unsupported readmission explicitly |

The remaining reconstruction work is concrete: existing math interfaces admit a generic supplier,
but the runtime reduced request currently assumes one factory covering every eliminated row and
coordinate. The selected replacement composes existing authored `Root`/`Regimes` factories and
`CasePlan` projections through that generic interface. Structural matching alone never produces
a selected regular sheet. Ineligible equality regions remain simultaneous or supply only an
initialization proposal followed by original correction.

Focused implementation entry points follow. These are starting owners, not a frozen file inventory;
follow actual consumers when a contract changes.

| Boundary | Starting source |
|---|---|
| Dependency projections, block completion and local preparation | [compiler workspace](../../crates/pse-compiler/src/workspace.rs), [math execution](../../crates/pse-math/src/execution.rs), [initialization](../../crates/pse-runtime/src/math/initialization.rs), [conditional strategies](../../crates/pse-runtime/src/workflow/strategies/conditional.rs) |
| Composite reconstruction and requested alternatives | [math derived contracts](../../crates/pse-math/src/derived.rs), [runtime derived preparation](../../crates/pse-runtime/src/math/solves/derived.rs) |
| Shared request, resolution, execution and scientific assessment | [strategy declarations](../../crates/pse-model/src/strategy.rs), [runtime strategy](../../crates/pse-runtime/src/math/strategy.rs), [prepared solves](../../crates/pse-runtime/src/math/solves.rs), [staged execution](../../crates/pse-runtime/src/math/staged.rs), [modeling assessor](../../crates/pse-runtime/src/workflow/modeling/assessment.rs), [opaque binder](../../crates/pse-runtime/src/math/opaque_strategy.rs) |
| Native completion and typed public generation | [POUNCE adapter](../../crates/pse-backend-native/src/pounce.rs), [settings catalog](../../crates/pse-schema/src/catalog/solve_settings.rs), [strategy catalog](../../crates/pse-schema/src/catalog/native_strategy.rs) |

## Target operations and preserved semantics

The names below describe consumed meanings; implement them in the existing owners with ordinary
domain functions/types. They do not prescribe a new service or one struct per concept.

### One request, conditional resolution and bounded dispatch

The composition request contains original problem/intent and required guarantees, backend/profile
constraints, allowed or required mechanisms, branch/start rules, product demands and finite task
controls. `Auto` supplies the versioned automatic policy. An explicit declaration pins its
composition; an explicit backend restricts native selection unless named alternatives are allowed.
Both pass through the same admission and execution operations. Python must not construct prepared
rung lists to obtain automatic execution.

The pure resolver consumes cheap mathematical facts, adapter-issued conditional capabilities,
actual retained products, prior observations and remaining allowances. It returns one next
decision: prepare a named product/representation, execute an admitted operation, assess a fully
reconstructed original candidate, finish with original permission, or stop with a typed cause.
The effectful driver binds that decision, performs it under the existing task scope and updates
the owned product/observation state. Expensive proofs, exact tear optimization and native work
are visible preparation/execution operations, not effects hidden inside inspection.

Capabilities distinguish ready support, conditional support with named prerequisites, unavailable
implementation and incompatibility. A prerequisite names the responsible producer and what it
must establish. A hash or label is not its evidence. Provider/class/derivative readiness may be
unresolved without being impossible. Failed trajectories remain observations and may permit a
different admitted profile of the same backend.

The automatic policy is deterministic and versioned. Use complete structural reduction and
existing valid products where their guarantees fit; otherwise retain simultaneous execution.
Select class-specific profiles using the existing routing authority and their actual derivative,
bounds/domain, sparse algebra and accounting requirements. Prefer already admitted products over
speculative expensive preparation. Request a fresh response/factor only for a declared related-target
consumer, requested sensitivity or another concrete product demand. Do not compute every method's
artifacts merely because that method exists.

The default permits lossless admitted representations, demanded products and same-entry library/
profile recovery. Replacement recovery origins are empty unless the request explicitly authorizes
them; fresh auxiliary, predicted, surrogate or displaced candidates need that permission. Repeating
an entry and replacing its starting point are different operations. Existing class/profile ordering
breaks ties only after guarantees, hard constraints, actual products and preparation demand are
satisfied. An automatic curvature choice resolves to a concrete supported Hessian mode before
adapter admission; an explicitly requested Exact mode remains binding. N0 declares this choice
in the new current request rather than reinterpreting serialized Exact settings.

Before dispatch, derive a finite candidate/preparation allowance from the admitted catalogs and
declared work/path controls. The default does not repeat an identical mechanism/profile/binding/
start/accuracy decision after unchanged refusal. Further refinement/subdivision requires its own
finite admitted producer/controller contract and actual progress. The enclosing absolute deadline,
shared pool and task work ledger never reset. Optional local unavailability may preserve a base
route; required incompatibility and terminal contract/resource/cancel/panic/infrastructure causes
cannot turn into recovery. Strict numerical caps require pre-operation admission through a producer/
native hook or a conservative complete reservation, including final validation and original
assessment. Reconcile actual successful and failed work against that reservation. A producer with
neither a hook nor a complete bound is incompatible with a strict cap; post-call counting alone
cannot enforce it. If actual work remains unknown under a complete conservative bound, retain the
full reserved allowance and report unknown actual work; never report reservation as measured work.
Deadline checks before/after indivisible work may observe an overrun, but may not admit late
evidence as successful completion.

Entry identity and recovery permission are separate. `Explicit` never silently substitutes its
entry point. `NoPriorStart` excludes inherited starts, not unchanged structural programs. Fresh
task-produced replacement recovery is allowed only by the admitted request's recovery-origin rules;
the resolver may narrow these permissions, never create them.
A workflow predecessor or a matching layout is not start permission. Connected-branch requests
require actual sheet/transport/orientation witnesses, including partial-path coverage; they do not
accept unrelated secants or displaced starts merely because a regime name matches.

### Structure, decompositions and complete reconstruction

Keep three projections distinct:

- Numerical incidence for matching, sparse derivatives and structural BTF.
- Execution/validity dependencies for branch choices, domain/applicability checks and provider inputs.
- Objective, inequality and other coupling that constrains independent solves or elimination.

Use library matching/BTF/SCC operations. Merge or invalidate sequential blocks when control/validity
dependencies require it, without creating fictitious Jacobian entries. A fixed-parameter guard
remains a validity input. Control cycles do not prove numerical rank. Objective/inequality coupling
remains simultaneous unless a valid complete reconstruction carries it.

Derive unit/port/row/unknown inventories from the authored model. Topology supplies causal/tear
candidates only. Preserve mandatory/forbidden groups, authored costs and required tear guarantees.
When no weighted optimum is requested, the existing explicitly identified unweighted heuristic
is admissible; required minimum-cost selection uses the existing HiGHS decision problem under
visible preparation accounting. A heuristic may not satisfy an optimality requirement.

Construct explicit maps only where the authored realization supplies the needed causal meaning.
Otherwise derive admissible conditional equation problems from complete owned inventories and
actual local solver demands. For a bounded input incompatible with fixed-point execution, choose
constrained simultaneous execution only when alternatives are permitted; a required map receives
an explained refusal. No arbitrary residual becomes a causal map. Resolve outer Value demand
first and propagate genuine local First/Second/action requirements afterward.

For reduction, compose admitted local factories/projections as a `ReconstructionOracle` supplier
with complete original coordinate/row maps, RHS realization, validity, actual chain actions and
propagated accuracy. Preserve original objective, inequalities and eliminated-coordinate bounds.
Do not invent regime rankings or a Cartesian product of regimes to manufacture eligibility.
Permit empty retained original rows when reconstruction is valid; target/native admission decides
whether Root, Feasibility or Optimize guarantees fit. Objective-free feasible sets and objective-only
optimization are legitimate; an underdetermined qualified-root request is different.
Zero retained rows and zero retained coordinates are distinct. A reconstruction that supplies every
original unknown produces a candidate for original assessment directly; it needs no zero-variable
native dispatch. A positive-dimensional underdetermined Root still receives an explained refusal.

At an accepted tear point, retain or repeat the final unit sweep and collect **all** local unknowns,
including non-port states. Local unit values remain transactional until complete original assembly.
Independently evaluate the complete original point before claiming simultaneous satisfaction.
Existing connection-map reports remain scoped map products, not full flowsheet results.

A terminal realization may proceed directly to original assessment only when its existing
correspondence contract and consumed witnesses establish the requested original guarantees.
Approximate curvature on the unchanged original problem needs no additional solve when its original
assessment already establishes the requested guarantees. Auxiliary or incompletely reconstructed
products require correction only when an admitted producer can establish a missing guarantee.
The original-correction role binds the frozen original target through the same executor and excludes
recursive selection of the auxiliary/reduced family being corrected. Native duals, global
certificates and stopping statuses cannot be copied from a different family without an admitted
original-space correspondence. Correction cannot manufacture global coverage, certified
infeasibility, connected-branch or dual correspondence; otherwise return a typed unmet-guarantee
refusal.
This avoids mandatory redundant solves without manufacturing original qualification.

### Assessment, work and accuracy are consumed products

The injected assessor returns the scientific product, original conclusion and typed cause,
composed candidate permission, artifact/session retention and disjoint work. Preserve native
termination separately. Modeling, fitting and shooting owners still perform and classify their
own scientific checks; the driver does not inspect statistical reports to infer meaning.

Distinguish local native infeasibility from original-model infeasibility proof, candidate scientific
rejection from assessment inability, and numerical failure from terminal operation failure.
Recovery requires the specific permitted observation and an admitted remedy. A failed scientific
expectation does not automatically authorize another trajectory.

Charge preparation, prediction, native callbacks, native final residual validation, independent
assessment and failed/abandoned work once under their actual owners. KINSOL's validation after
callback statistics and the modeling assessor's fresh `constraints(values)` call belong to this
inclusive boundary. Use disjoint phase charges or a producer-owned inclusive total, never both.
Unknown composite counts cannot be replaced by known native subtotals. Poll scope before and
after indivisible work and before accepting evidence; time and allocation safety remain distinct
from exact numerical work accounting. Reserve/admit native validation and independent assessment
before their next counted work unit so neither silently crosses a hard cap.

Replace the strategy-global accuracy list with operation-specific input prerequisites, output
obligations and before-use consumer demands. Admit an evidence producer from its support and
bounded production/refinement capability, not its own future certificate. Publish actual evidence
bound to original/source, coordinates/point or covered region, derivative/action order, normalization,
branch, allowance and evidence class. Estimated evidence cannot satisfy Certified demand. Support
or exact derivative provenance cannot manufacture zero numerical error.

Keep one actual product state shared by resolution and trace. Refine/promote products through their
producer under the same scope and invalidate only affected numerical products. Retain structure,
sheet/exclusion proofs, point/jet evidence, symbolic setups, numeric factors and escaping allocations
as distinct products with their consumed dependencies and lifetime owners. Trace storage remains
allocation-owned; failure to allocate a trace must retain the original terminal cause.

### Native completion and source-owned observation

POUNCE's pure `second_opinion_rungs` supplies recovery descriptions. The native adapter derives
availability from the **actual typed FERAL configuration**, effective barrier/start settings and
observed quality escalation, then lowers each description into an immutable per-attempt profile.
MC64 and quality changes must configure rebuilt main/restoration factories, not merely option
strings. Reconstruct each noncumulative attempt from the original typed settings and explicit
option set-ness. Hidden library retries stay off. Perturbation is a screened start operation with
actual displacement and branch permission. Each attempt retains its own status/candidate/work and
undergoes original assessment; native success does not grant scientific permission.

Add registry-owned Partitioned and FiniteDifference Hessian modes and POUNCE-specific typed
settings. Both consume analytic First support, not supplied Second. Bind conservative objective/
constraint support after presolve, including `TNLP::get_objective_variables_linearity`; never infer
objective support from a potentially zero first gradient. Preserve unset versus explicit partition
update formulas and record oversized-element diagonal degradation. FD uses conservative structural
patterns, explicit coloring/reuse policy and finite probe work, retaining primal and multiplier
reuse dependencies. Approximation stays labeled approximate and preserves final original accuracy.
Terminal callback failures cannot become backward-probe recovery. Other adapters act or refuse
these modes explicitly; reserved raw options cannot compete with typed lowering.

For solve Schur, use POUNCE-owned arithmetic and add a narrow source-owned observation/control
seam. Produce or acquire an immutable upstream/fork commit based on the characterized source;
pin the complete resolved POUNCE family, including internal workspace/path dependencies, to one
coherent source/revision through workspace dependencies and the lockfile. Preserve QP/convex
features and FERAL 0.18.0 unless separately authorized scope changes it. Existing dependency policy
permits this. The native source/build identity records the immutable revision, not only the unchanged
0.12 version string. Do not edit registry caches, use `[patch]`/`[replace]`, wait for an unconfirmed
release, or copy Schur iteration into project code.
The packet owns producing/acquiring the immutable source artifact, not an external release promise.

The source seam supplies the actual post-classification layout and effective fixed-variable treatment,
requested/admitted separator, actual path/fallback reason, primitive work and fallible pre-allocation/
pre-operation admission. Map semantic separators after project presolve **and** actual native
classification into `x,s,c,d` offsets; reuse library classification rather than reproducing it.
This is a bidirectional binding: return the mapped separator after actual TNLP classification and
before the algorithm builder consumes it. A notification-only observer after construction is too
late. Add source-owned typed Schur configuration injection using the same effective FERAL profile
as the main/restoration factories; `set_kkt_schur_block` alone supplies only indices and cannot
establish those settings. Remove the independent option-string derivation for the configured route.
Observe actual factor/backsolve attempts, including failure before monolithic fallback. Poll/admit
work before operations and distinguish contract/resource/cancel abort from numerical fallback.

Reserve both Schur factors, coupling/action storage, dense separator buffers, maps, packed vectors,
refinement matrices and simultaneously live monolithic fallback objects. Keep allowances through
teardown. Separator fraction is not memory admission. Limited-memory remains its Woodbury path;
assembled exact/partitioned/FD combinations require source-backed admission and actual controls.
Sensitivity Schur remains a different product. Partition submission alone is not actual-use evidence.

### Mechanism selection and actual consumer obligations

| Mechanisms | Admissible role in automatic resolution | Required integration boundary |
|---|---|---|
| Direct class-specific solving | Minimal original execution when decomposition/proposals do not fit | Existing routing supplies actual profiles; constant/LP/QP/conic/discrete routes remain lawful |
| M1/M1b/M3 prediction and secants | Related target and admitted available products; actual regularity/start/branch facts | Common selector consumes real KKT/activity/QP/root/secant products; screen original target and correct nonlinear proposals |
| M2/M7 bounded correctors/restoration | A proposal or specifically recoverable numerical observation; bounds/domain support | Original frozen specification and auxiliary permission; stationary non-root is not final Root success |
| M4/M5/M6/M8 native methods | Actual directional/setup/map/globalization capabilities, typed profiles and observed failures | Prepare JVP independently from preconditioner Jacobian; reuse actual setups/factors; visible second opinions |
| M9/M10 continuation/homotopy | Admitted anchor/path/family, orientation, domains and terminal correspondence | Existing bounded library correctors/path controllers; no invented anchor or merely overlapping sheet proof |
| M11/M12 artificial flow/IC recovery | Declared admissible flow/mass or authored differential/algebraic roles | Library controller and original consistent-state assessment; no reinterpretation of scientific time/events |
| M13/M14/M15 block/reduced/preconditioned forms | Complete dependencies, coupling, actual reconstruction/action/regularity contracts | Full original reconstruction; scoped PETSc NASM limits; initialization-only fallback when elimination is unproved |
| M16/M17 surrogate/multistart/batch/Schur | Declared fidelity/finite domain, start/branch policy, independent targets or mapped separators | Retained egobox state and coordinated workers; no automatic multiplication of every failed multistart into a retry ladder |
| M18 evaluation reduction | Demanded outputs/actions and explicit approximation contracts | Retain selected-root/proof/jet products separately; typed Hessians and actual preparation/probe work |

Availability is not a requirement to execute every family. Each family supplies admissibility and
production contracts to the common resolver; unsupported geometry, unknown regularity, insufficient
guarantees or unavailable strict accounting produce visible reasons. Preserve global-coverage and
branch guarantees: a locally restricted sheet cannot satisfy a global request merely because it
converges. Configuration is selected from fitting capabilities, not an exhaustive benchmark grid.

The next-decision design avoids an eagerly prepared graph whose optional proofs/factors can dominate
a simple direct solve. Its cost is explicit product invalidation, admission and observation handling;
those meanings belong to existing operation owners. Composite reconstruction adds mapping/action
obligations but avoids synthetic regime selection and duplicate authored inventories. The source-owned
native seam introduces a maintained immutable dependency revision; that cost is justified by actual
configuration, memory and work control that an opaque wrapper cannot supply. Reconsider the custom
source revision when an upstream release supplies the same consumed contracts, preserving its tests.

Three future changes illustrate the boundaries without adding scope. A new native backend supplies
conditional profiles and an operation binder, reusing resolver/assessment rather than workflow retries.
A new implicit provider supplies reconstruction/actions/evidence, reusing original-coordinate
composition rather than a flowsheet-specific numerical engine. A new scientific workflow supplies
its mathematical target, original assessor and requested guarantees while retaining its scientific
loop. A genuinely new guarantee or unaccountable native operation reopens the relevant contract;
it cannot enter through an adapter exception. Reduced preparation/reuse costs are expected only
where structure or qualified products survive across targets; 25k K4 measures that benefit and the
retained-memory cost rather than assuming a speedup for every problem.

## Public contracts, identity and decision route

Current settings/documents must not reinterpret stored omitted-backend defaults. The coordinated
cutover uses these next versions, with version checks preceding nested current decoding:

| Current owner | Planned current contract | Required meaning |
|---|---|---|
| SolveSettings v1 | v2 | Versioned composition policy, constraints and resolved automatic default; existing backend/settings/start remain authoritative |
| Unversioned NumericalStrategy JSON; strategy/request/preparation frames v1 | NumericalStrategyDocument v2 envelope; v2 strategy/request/preparation frames plus distinct resolution identity | Check document version before body decoding; operation-bound evidence and actual decisions; retain historical frame bytes |
| StudyOperation/StudyRequest v2 | v3 | Reconstruct the same requested numerical policy under the new settings |
| StudyDefinition v4; JobPayload v6 | v5; v7 | Stored execution interpretation admitted before nested inputs |
| DurableJobRequestV3 / DurableModelingRequestV1 / DurableStudyRequestV2 | V4 / V2 / V3 | Changed requested execution semantics have new identity, not rewritten historical hashes |

Expose requested composition and inspectable resolved decisions for each applicable prepared
numerical target in Rust/Python. Before execution, inspection may show conditional prerequisites;
after execution it records actual products/profiles/observations and work. Preserve current explicit
prepared-declaration bindings through the same binder; automatic users never assemble factories or
native rung lists. Stable scientific identities are separate from request, decision, preparation,
binding and native-build identity. Opportunistically available products do not invalidate unrelated
immutable evaluator artifacts.

Generate vocabulary, documents, relation projections and stubs from the Rust/registry declarations
with `just codegen` as each declaration changes. Version recorded event/descriptor interpretation
where added observation/work/evidence meanings affect it. Use 25g's directional compatibility and
explicit readmission; unsupported historical execution is typed refusal, not a parallel old solver
path. Historical result readability and original payload bytes remain intact. SQL readiness alone
does not migrate opaque JSON semantics.
The strategy envelope contains a version and strategy body; the direct Rust/Python JSON codec
checks the header before current-body decoding. Historical unversioned declarations require explicit
current readmission or typed execution refusal, never implicit reinterpretation by that codec.

N0 owns the precise decision changes before their production implementation: amend proposed
ADR-0154 for automatic resolution/assessment/constraints, ADR-0155 for composite reconstruction,
operation-bound evidence and changed identity/readmission, and extend the applicable native-profile
decision for the observation binding. Preserve ADR-0156's library ownership and ADR-0157's existing
Uno/PETSc scope; use a short new ADR for a distinct POUNCE binding contract rather than relabeling
Uno/PETSc. Accepted records are immutable; supersede affected accepted clauses when required.
The required architecture owners are blueprint §14, §17.4, §18.6–§18.8, §19.3 and §20.5. Use the
decision/design route and revision row for actual owner amendments. ADR status acceptance remains
its own prescribed operation; this plan does not grant it.

The default-policy/public-boundary changes require a bounded decision review of the chosen target
contracts. Reuse the current diagnostic review rather than repeating a whole-product review. New
dependency pins alone need no ADR or governance relaxation. No library/license is rejected.

## Execution packets

Execution was authorized and is now being closed out at the maintainer’s request before a separately instructed review. The table distinguishes landed slices from remaining packet obligations. Early contract readiness is
not consumer acceptance; packages close only after their actual targeted consumers and deletion
obligations pass. Static checks and assembled integration wait until all functional scope lands.

| Packet | Actual prerequisite | Result and responsible boundary | Closeout state |
|---|---|---|---|
| <a id="n0"></a>N0 Decisions and shared contracts | Current review and confirmed Auto default | Decision proposals/owner amendment route; exact request, dependency, assessment, product and observation meanings. Coordinator owns shared contracts and identities. | Proposed decisions amended and bounded target review Accept; ADR acceptance remains separate. |
| <a id="n1"></a>N1 Mathematical fidelity and demand | N0 dependency/reconstruction contracts | Compiler/math projections, valid block scheduling, original block completion, local consumer demand and empty-row reduction. | Dependency/schedule/demand source and focused compiler controls landed; automatic block execution remains. |
| <a id="n2"></a>N2 Assessment and inclusive work | N0 assessment/observation contract | Runtime/native/workflow assessor transport; actual original cause and disjoint accounting, including failed/final checks. | Original conclusion/cause and disjoint work transport landed; inclusive native/producer hooks and consumer coverage remain. |
| <a id="n3"></a>N3 Conditional capabilities and actual products | N0 product/request contract; N2 charges | Common producer binder, operation-bound accuracy and one owned execution product state; existing real reduced/root producer consumed. | Owned actual product state and operation-bound consumption landed; broader producer/driver acceptance remains. |
| <a id="n4"></a>N4 Derived decomposition and reconstruction | N1 complete projections; N3 producer admission | Compiler-derived causal/block/reduced candidates, composite supplier and full non-port original-state reconstruction. | Composite reconstruction, chain uncertainty and zero-retained candidate source landed; automatic causal/block dispatch and complete compiled-case coverage remain. |
| <a id="n5"></a>N5 Native profiles and visible recovery | N2 original cause/work; N3 demand contract | Native POUNCE second-opinion profiles, typed partitioned/FD support and actual compiler/oracle consumers. | Typed profiles and actual partitioned/FD consumers passed 13 focused controls; full packet acceptance remains. |
| <a id="n6"></a>N6 Observed solve Schur | N0 binding contract; N2/N3 admission; N4 semantic separator slice | Source-owned observer artifact, coherent pins, actual native layout/lowering/use/fallback and complete storage/work. Source seam can proceed before final partition consumer. | Coherent public POUNCE/FERAL source and adapter hooks landed; runtime admission/retention bridge and full requested controls remain. |
| <a id="n7"></a>N7 Qualified proposal selection | N3 product state; existing predictor/transport interfaces | Common KKT/activity/QP/root/secant selector and real related-target/horizon consumers, with source/branch/start screening. | Shared Root/KKT selector and root/study/horizon consumers landed; Activity/QP/secant selector integration and broader product coverage remain. |
| <a id="n8"></a>N8 Automatic resolution and driver | N2/N3 working semantics; N4–N7 applicability/production contracts | Pure deterministic next decisions, demand binding, observation-conditioned finite execution and explicit declaration binding on one path. | Lazy same-session direct/reduced/POUNCE selection slice landed; complete alternatives, strict accounting and packet acceptance remain. |
| <a id="n9"></a>N9 Scientific consumer migration | N8 real target execution; relevant N4/N7 products | Modeling/init/recycle, studies, fitting, shooting, horizons and dynamic initialization use the common target contract while retaining scientific policy. | Modeling assessment and selected prediction consumers changed; opaque fit/shooting Auto, dynamic/init/recycle migration remain. |
| <a id="n10"></a>N10 Public and durable cutover | N8/N9 actual target binders; N0 version contract | Generated Rust/Python requests/inspection/trace, current versions/identity/readmission and coordinated activation of Auto default. | Current versions/frames and generated boundaries landed; compile and 3 public controls passed; full public/durable round-trip scope remains. |
| <a id="n11"></a>N11 Functional handoff and retention | N1–N10 integrated and targeted checks complete | Delete replaced paths, reconcile evidence/status, register assembled fixtures and hand readiness/static refresh to 25k. | Not complete; this checkpoint is a paused partial implementation, not a 25k handoff. |

N1/N2 and native source work can overlap after their shared contract slices are settled. Logical
independence does not grant simultaneous edits to strategy declarations, registry, settings, IDs or
the driver. Name one editor for shared surfaces; assign producer/adapter/consumer scopes explicitly.
Use existing main for ordinary work, isolated checkouts only when genuine concurrent editing needs
them. No plan-writing worktree is required.

### N0–N3: establish meanings before policy relies on them

N0 records public defaults, hard versus permitted alternatives, requested guarantees, actual
operation evidence/work, nonrecursive original-correction roles, strict pre-operation admission,
strategy document versioning and decision routes. Its acceptance is coherent contract/source inspection
and the required bounded decision review, not a scientific test claim. Allocate any new ADR through
the existing numbering route. No duplicated profile defaults or second capability registry.

N1 retains numerical support and adds separate execution/validity/coupling projections. Derive
predecessor/merge policy from them and assess the complete assembled original point. Migrate
block initialization and causal preparation immediately; remove derivative-only scheduling,
blanket whole-case First preparation and mathematical nonempty-row rejection. Preserve actual
First demand in conditional solvers and selected residual minima.

N2 introduces the common assessor result and transports it through native and opaque consumers.
Scientific owners supply their conclusions. Count final native validation and independent
assessment; failure charges survive terminal errors. Remove private transport/classification only
after the common seam carries the same scientific meaning. Unknown inclusive counters without a
complete conservative reservation refuse strict caps; with one, retain the full allowance and keep
actual work unknown. Native convergence remains visible even when original permission refuses it.

N3 binds prerequisite/produced/consumed evidence to actual operations. Use existing actual reduced
reconstruction and root/point-action refinement as the first real producers, publish their evidence
into the same state the next consumer and trace use, and remove preparation-only empty snapshots
and global demands. Producer support never certifies its own future result. Preserve allocation
owners, original deadline/cancellation and observed unsuccessful work.

Targeted acceptance: `just check-package`/`just unit-package` for touched compiler/math/model
packages and selected `just unit-native-package pse-runtime`/backend controls with the relevant
adapter features. Tests must cover the branch-controlled `x/y` counterexample, fixed-parameter
guard, native convergence with original refusal, terminal assessor failures, refusal before the next
native-validation/assessment work unit crosses a strict cap, unknown total without a complete bound
under a strict cap, unknown actual total retaining a complete conservative reservation,
actual nonempty accuracy production,
wrong-point/class/normalization refusal, Value-only explicit map and genuine conditional First demand,
objective-only reduction, eliminated bounds and underdetermined Root refusal. No integration suite
is substituted for these revealing local controls.

### N4: derive usable mathematical alternatives

Construct complete candidate inventories and admissibility through existing compiler/math/structural
owners. Lift the single-factory restriction by composing admitted reconstruction suppliers, not by
inventing selection semantics. Preserve RHS, selected sheet, objectives, inequalities and bounds.
Use block solving as an initialization proposal where regular reduction is unestablished. Gather
non-port local state at final tear evaluation and independently assess original completion. A failed
unit sweep does not partially commit the original state.

Targeted actual compiled-case controls: a control-dependency cycle that merges or refuses without
invented rank; cross-unit objective/inequality coupling; multiple admitted implicit suppliers with
chain actions and consumed accuracy; a legitimate zero-row reduced NLP/feasibility target; complete
reconstruction with no retained coordinates proceeding directly to original assessment; missing
sheet regularity retaining simultaneous execution; complete non-port recycle reconstruction; and
bounded required-map/forbidden-tear refusal. Compare reconstruction/actions with independent
original evaluation. Delete caller-required duplicate residual/unknown inventory assembly where
automatic alternatives replace it; keep intentional explicit constraints as legitimate bindings.

### N5–N6: library-owned methods with actual configuration and observation

N5 lowers library second opinions into actual typed profiles, rebuilds factories, preserves option
set-ness and original assessment, and binds partitioned/FD modes with conservative support and
actual First demand. Register exact effective settings and approximation/degradation observations.
Reserve raw competing controls. Targeted native controls must demonstrate a real acting rung,
noncumulative baseline restoration, forbidden perturbation, terminal no-retry, objective support
at a stationary start, partitioned degradation, FD probe failures/reuse and unsupported-mode refusal.
Use actual compiler/TNLP consumers, not only pure rung descriptions.

N6 owns the immutable POUNCE source addition and acquisition. The observer/control hook is synchronous,
worker-local, reset per attempt and exception/panic shielded. It reports actual layout/path/work and
admits extents before allocation/operation. Preserve library matrix construction, Schur arithmetic,
inertia, quality escalation and numerical fallback. Bind separators before algorithm construction and
inject typed Schur settings from the effective FERAL profile. Update the complete resolved family
together and verify source/revision coherence, including internal dependencies, features and the
unchanged FERAL pin. The new source/build identity invalidates affected
native products, not unrelated evaluator mathematics.

Targeted native controls: successful Schur execution with actual typed MC64/quality second-opinion
settings on its factors; fixed-variable removal/classification and
effective RelaxBounds treatment; unsuitable separator fallback; failed Schur work followed by
monolithic work counted separately once; pool refusal before allocation; cancellation/contract abort
without fallback; full allowance retained through teardown; and admitted assembled Hessian profiles.
Actual-use telemetry is required. A wrapper counting opaque invocations or assumed two-factor work
does not close this packet. Remove temporary diagnostic/source-acquisition substitutes after the
source-owned route is consumed; never modify the registry cache or generated outputs manually.

### N7–N8: product selection and automatic composition

N7 supplies one proposal selection operation consuming qualified products and the target's
constraints. Prefer admitted exact/covered transport when required; select root/KKT/activity/QP/
secant/zeroth-order starts only under their actual contracts. A partial path remains partial; a
secant across unrelated sheets is unavailable. Prepare/materialize dense responses only if demanded.
Use at least one real related-case study and the existing advanced-step horizon as consumers.
Keep horizon control application distinct from start-only permission. Delete consumer-private
numerical selection when the common operation is tested and consumed.

N8 replaces vector-only automatic execution with pure next-decision resolution and the existing
bounded effect driver. An explicit declaration binds its requested operations through the same
mechanism; its declared recovery/start constraints do not change. Resolution includes demanded
preparation and retains rejection reasons. Optional unavailable products can leave direct execution
valid. A same-backend different-profile attempt is not blocked by prior trajectory history.
Actual products, causes, work and remaining limits determine the next admissible decision.

Targeted resolver/driver controls: minimal direct resolution; no eager Second/proof preparation;
conditional support discharged by a real producer; no identical no-progress loop; same-backend
recovery; required/optional refusal; strict unknown/unbounded-work refusal; enclosing exhaustion; explicit
backend/profile/entry identity; `NoPriorStart` and separately admitted recovery; connected-sheet
constraints; constant/rejected/report-less paths; auxiliary-to-original correction without recursive
family selection; direct assessment after approximate curvature on the original problem; and typed
refusal for missing guarantees that correction cannot establish. Pair scripted pure decisions with
actual native consumers. N8/N9 migrate internal binding while preserving admitted current request
interpretation; N10 activates the versioned Auto default and removes its predecessor. Delete duplicate
dispatch after replacement consumers pass; do not retain a compatibility executor.

### N9–N10: consumers and recorded interpretation

Migrate each applicable numerical target, not merely its trace wrapper:

| Consumer | Common operation consumed | Scientific responsibility retained |
|---|---|---|
| Modeling/init/recycle | Decomposition, demanded preparation, numerical execution and complete reconstruction/assessment | Authored procedures, overlays and original physical completion |
| Study | The same target operation and qualified product/start admission | Sampling, occurrence/dependency policy, publication and durable effects |
| Fitting | Actual fit oracle/support, bounded numerical composition and assessment transport | Experiments, identifiability/rank, statistical brackets and uncertainty |
| Shooting | Actual shooting oracle/First support and admitted limited-memory or other supported profile | Windows, integration/endpoint policy and experiment meaning |
| Horizon | Shared proposals/correction targets and actual evidence | Estimation/control sequencing and authorization to apply a move |
| Dynamics | Algebraic/consistent-initialization preparation where applicable | Library integration controller, differential/algebraic roles, scientific time/events and trajectory assessment |

Opaque targets participate through actual callable contracts; they do not claim symbolic expressions
or unprovided Hessians. Migrate numerical retry/preparation only, not whole scientific loops.
Remove `opaque_strategy::direct` as a separate numerical binder after fit/shooting callers use the
common binder; preserve their original check producers. Preserve incompatible/unavailable target
results as typed observations rather than silently dispatching another scientific procedure.

N10 activates new current versions and the Auto default with all affected callers and generated
boundaries in one coordinated cutover. Earlier private resolver development must not reinterpret
stored v1 settings. Preserve historical bytes/readability; refuse unsupported historical execution
before decoding current nested settings. Update content identities and public preparation/inspection/
trace together, including non-modeling targets. Run `just codegen` for each declaration change and
`just python-stubs` against the appropriate compiled boundary when required by its source change.
No handwritten parallel Python schema or old execution shim.

Targeted acceptance uses actual workflow units and generated/direct Python controls: fit/shooting
first-order restrictions, statistical/control policy unchanged under different numerical attempts,
study/horizon product adoption, dynamic-role/IC refusal, version-first historical readmission,
unchanged historical hash bytes, changed current execution request identity, independent operation
and occurrence identity, request/inspection/actual-trace round trips and unsupported native modes.
Use isolated operational-store units only where these changed recorded contracts require them;
full restart/mixed-operation qualification remains 25k.

### N11: handoff, qualification and deletion

Integrate targeted results against zero, ensure all replaced mechanisms/callers/fixtures are deleted
and record remaining supported limits accurately. Do not port tests for deleted mechanisms as if
they remain production scope. Register the actual automatic connected/stiff/coupled case, original
PR/CSTR cases, changed-active-set and resource/accuracy failures for 25k.

At all-functional-scope completion, run the relevant `just hygiene`/manual checks under 25k's
existing static refresh and the selected assembled journeys once. Repair to zero and rerun failed
or affected scopes, preserving composite receipt history. K4 measures current dev-profile cold/warm
preparation, value/structural edits, actual proof/evaluation/setup/factor counts and retained-resource
lifetimes after functional qualification. No release-profile timing campaign or fabricated speedup
threshold. K5 assesses the assembled target and moves enduring contracts/rationale to their owners.
25n handoff does not itself mark K3/K4/K5 complete.

Worktree inventory/judgment/removal remains the already authorized post-qualification action.
Assess whether code is valuable and aligned, integrate worthwhile changes, and retire obsolete
worktrees; differences alone never justify indefinite retention. No worktree cleanup occurs during
plan authoring or substitutes for tests passing.

## Finding dispositions and verification

These findings remain **open**: landed slices and targeted controls do not yet satisfy every
packet acceptance obligation. This table owns adoption and subsequent disposition; older 25m findings keep their existing owner and link the relevant corrections.

| Finding | Disposition | Planned owner | Evidence needed for resolution |
|---|---|---|---|
| [AF-01](../design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md#af-01) | open | N1/N4 | Control-sensitive original block completion, distinct dependency projection and full reconstruction |
| [AF-02](../design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md#af-02) | open | N2/N8/N9 | Actual original refusal/cause reaches transition and public trace without false permission or terminal retry |
| [AF-03](../design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md#af-03) | open | N2/N6 | Inclusive/disjoint work, pre-operation hard-cap admission, unknown/unbounded-total refusal and conservative reservation without a false measured count |
| [AF-04](../design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md#af-04) | open | N3/N8 | Actual producer evidence admitted and consumed; wrong point/source/class/normalization remains refused |
| [AF-05](../design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md#af-05) | open | N4/N7/N8/N9/N10 | Automatic full-case/related-target composition and actual scientific consumers with explicit constraints preserved |
| [AF-06](../design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md#af-06) | open | N5/N6 | Actual visible second opinions, solve Schur use/fallback/accounting, typed partitioned/FD execution and truthful profile limits |
| [AF-07](../design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md#af-07) | open | N1/N4 | Actual Value-only map preparation and genuine conditional derivative demand |
| [AF-08](../design_review/reviews/design_review_automatic-simulation-solve-pipeline_2026-10-04.md#af-08) | open | N1/N4 | Objective-only and objective-free feasibility reductions; original bounds/assessment and underdetermined Root refusal |

**Proposed:** the target operations, source-owned observer addition, current document versions,
packet acceptance controls and benefits above. **Interface-checked:** the review/focused authoring
assessment inspected the existing generic reconstruction seam, native APIs, conditional routing,
version-first boundaries and admitted immutable Git dependency route. **Implemented:** the baseline
25m foundations exist with their source limitations and scoped evidence. The closeout checkpoint records new **Implemented** and scoped **Tested** evidence. No 25n
assembled qualification, performance measurement or formal proof is claimed. Existing 25m receipts remain scoped to their
original named commands/conditions and do not close these findings.

The original PR scientific runs, unfinished linked Python/assembled campaign and publication timeout
retain their existing interpretation in 25m/25k. Do not infer a solver smoothness limitation from
an attempt requesting a different derivative order, weaken original bounds/criteria, or increase
budgets as a substitute for implementing this target. Use the repository's recipe-owned memory
scope and the maintainer's workstation capacity for admitted runs; no arbitrary lower resource
ceiling is added here.

Completion requires every N0–N11 packet's supported functional scope and targeted deletion/control
acceptance, coordinated current public/durable interpretation, coherent decision routes and a clear
25k handoff. Scientific full-series qualification and measurement remain separate. Record actual
implementation mistakes/corrections and deliberate deviations in the Outcome at implementation
closure; do not prefill them with hypothetical success.

## Authoring checkpoint

The review findings and confirmed Auto-default choice are incorporated. The composite reconstruction
and native observer routes were assessed with focused read-only advice and reconciled here. Source
inspection cannot establish their execution; their actual producer/consumer controls are packet
acceptance. No implementation, campaigns, pin changes or worktree cleanup were performed during
plan creation. The next executable work, after production authorization, is N0 shared decision
contracts, then N1/N2 fidelity and assessment foundations and N3 actual evidence admission.

## Execution checkpoint

Maintainer-directed pause, 2026-10-04.

The maintainer requested current tasks conclude, integration into main, compile and recent-change
focused test repair, worktree removal and documentation reconciliation, followed by a pause.
The next review’s structure will come from the maintainer; no review or comprehensive campaign
is started by this closeout. All worker source changes are integrated directly in the existing main
checkout and remain uncommitted for review. This is not completion of N0–N11 or a 25k functional handoff.

**Implemented:** proposed ADR-0154/0155 amendments, ADR-0158 and corresponding proposed architecture
contracts; a bounded target-decision review returned Accept, distinct from implementation acceptance.
Compiler numerical/dependency/coupling projections, retained-environment block preparation,
composite selected-provider reconstruction and incoming-neighborhood checks are integrated.
Automatic supplier discovery currently admits zero-offset equality realizations where the retained
factory’s value interpretation is unambiguous; nonzero offsets need actual producer-issued realization
metadata before automatic admission. Explicit admitted supplier realization remains supported.
Complete reconstruction with no retained coordinates yields an original candidate without a fictitious
zero-variable native attempt. Original scientific conclusions/cause, disjoint charges and an owned
product state are transported independently of native termination.

**Implemented:** the public maintainer POUNCE fork supplies typed, source-owned post-classification
separator/configuration, attempted work and fallible storage/operation hooks. The complete POUNCE
family and FERAL 0.18 come from one immutable artifact; native build identity includes its revision.
The bounded serial dense profile supports dimensions 1–7 and separately labels complete linear
storage versus opaque application storage. Larger ordinary routes remain supported under deployment
memory scope; a complete strict bound for those factors is unavailable. The runtime work-admission
bridge and retained reservation lifetime have **not** been integrated, so source/adapter controls do
not establish N6 task-level strict-cap closure.

**Implemented:** typed Hessian modes and immutable noncumulative POUNCE recovery profiles; shared
qualified Root/KKT prediction selection in root/study/horizon source; lazy automatic direct/reduced/POUNCE
binding on the retained native session with a shared task scope, ledger, product state and no-repeat
binding identity. Initialization/causal inventories are inspectable/preparable, but they are not yet
automatic dispatched operations. Opaque fitting/shooting still use their direct binder; its removal,
complete scientific consumer migration and non-modeling strategy inspection remain open. Shared
Activity/QP/secant selection is not implemented; unused selector scaffolding was removed during
closeout, while existing standalone library/product capabilities remain.

**Implemented:** SolveSettings v2, NumericalStrategyDocument v2, StudyOperation/StudyRequest v3,
StudyDefinition v5, JobPayload v7 and new request/preparation/profile/durable identity frames.
Header-first decoding refuses historical execution before current nested interpretation. Historical
frames and bytes remain retained. Registry declarations include independent original conclusion and
actual decision identity. These source changes and selected controls do not establish durable restart or complete public Python
round-trip qualification.

### Focused closeout verification

**Interface-checked:** `just check` (default workspace, all targets), `just check-solver-contracts`
(linked native library/test source graph with force-validation) and `just check-native-python`
completed without compile errors or warnings. **Implemented:** `just codegen` completed all six
schema targets, Ipopt bindings and Hakari generation/dependency management; `just py-sync-native`
built the dev-profile editable extension and regenerated the actual compiled Python API stub.
The complete resolved POUNCE family plus FERAL comprises 16 packages from the same
[corrective source artifact](https://github.com/paul-heyse/pounce/commit/1c3ad80911e3d94702c54f7259aa9659adee4706).

**Tested:** the following focused controls passed against a zero-failure baseline on the pinned
dev/test profile. Rust test recipes explicitly enable `pse-relations/force-validate`; native
recipes use the memory cap. Symbolic/native runs load the repository Symbolica license.

| Actual command and selected scope | Final result |
|---|---|
| `just unit-package pse-model 'test(version_admission_precedes_current_body)'` | 1/1 passed; historical header admission precedes current nested decoding |
| `just unit-package pse-compiler 'test(conditional_source_branch_cycle_is_execution_coupling_without_numerical_rank) \| test(fixed_parameter_branch_is_retained_in_execution_projection) \| test(conditional_locality_refuses_objective_coupling)'` | 3/3 passed; execution/validity/coupling projections and retained-environment automatic block preparation |
| `just unit-package pse-math 'test(composite_) \| test(reduced_zero_row)'` | 4/4 passed; disjoint/chain reconstruction, uncertainty refusal/amplification and empty retained rows |
| `just unit-native-package pse-backend-native kinsol,root-isolation 'package(pse-backend-native) & (test(actual_ibex_incoming_neighborhood) \| test(causal))' --test-threads 1` | 2/2 passed; actual IBEX incoming uncertainty and complete non-port causal state |
| `just unit-native-package pse-backend-native pse-backend-native/pounce 'test(pounce::tests)'` | 13/13 passed on the corrective artifact; actual typed profiles, stationary objective support, Schur use/admission and retained factories |
| `just unit-native-package pse-runtime pse-runtime/native-solvers` with the exact filter below | 46/46 passed; strategy/assessment/work/product controls, actual Root/KKT/occurrence consumers, codecs, fitting curvature and zero-native original reconstruction |
| `just py-unit` with the three exact node IDs below, after `just py-sync-native` | 3/3 passed; declared envelope/request inspection, original permission and unique charges, duplicate-ID refusal and document-key semantics |

The runtime filter was:

```text
test(math::strategy::tests) | test(math::opaque_strategy::tests) | test(math::prediction::tests) | test(advanced_step) | test(retained_root_action_screens_target_then_original_corrector_qualifies) | test(occurrence_execution_tests) | test(actual_full_reconstruction_uses_direct_original_outcome_without_native_attempt) | test(current_request_admits_version) | test(solve_settings_enum_types) | test(study_definition_historical_readmission_codec_unit) | test(job_request_identity_independent_of_key_order) | test(study_job_) | test(gauss_newton_hessian_matches_jtwj) | test(compiled_weighted_loss_gradient_and_exact_hessian)
```

The Python node IDs were:

```text
python/pse/tests/test_native_workflow.py::test_declared_numerical_strategy_uses_original_permission_and_stops_unused_rung
python/pse/tests/test_native_boundary_contracts.py::test_strategy_document_constructor_and_decoder_refuse_duplicate_selected_ids
python/pse/tests/test_native_boundary_contracts.py::test_generated_document_keys_preserve_mapping_and_signed_zero_equality
```

This is composite focused evidence. Earlier native 12/13, runtime 41/46 and Python 2/3 results
were repaired and their selected scopes rerun. Compile repairs added required fields/imports and
correct feature gates. A bounded original fixture now uses a lawful Ipopt initialization route;
its actual KINSOL/IBEX reconstruction still bypasses outer native execution. Strategy controls now
retain scientific refusal without inventing numerical recovery and test strict-cap refusal before
dispatch. The Schur source bug rebuilt its sparsity pattern when finite-difference support changed
at unchanged dimensions. Generation now validates documents with the same candidate registry enums;
Hakari exclusions match the typed Git revision request. These repairs preserve the requested
contracts, rather than weakening original bounds or scientific criteria.

The source artifact's corrective bounded-factor regressions (2/2) and observed Schur application
corpus (3/3) also passed. They are source-level controls, not runtime admission-bridge acceptance.
No comprehensive hygiene, integration, performance campaign, full durable restart, 25k K3/K4/K5
or new design review was run during closeout. No assembled qualification or measurement is claimed.

### Maintainer-requested failure repair

After the pause, the maintainer authorized repair of the six reported non-timeout failures and
workspace Clippy findings. This bounded repair does not resume the remaining implementation or
start the proposed review.

**Implemented:** the identity control freezes the historical typed POUNCE document under V4,
preserving its golden, and captures the expanded current settings under V5. Each new control
remains identity-bearing. The approved frame additions retain all historical spellings. The
closed Python document probe consumes the registry-owned Hessian vocabulary, and the invariant
fixture recipe regenerates the two strategy-event row fixtures with the declared conclusion and
decision columns. The shared acceptance profile supplies the current composition/reconstruction
fields.

**Implemented:** the declared-rung control separately exercises unsupported strict-work refusal
before dispatch and actual Clarabel execution under its unchanged native iteration controls.
Transition-bearing completion events remain separate from assessment charges. Clippy repairs use
owned indirection for retained proposal/binding payloads, preserve move and lease ownership, and
simplify conditions and test helpers without lint suppressions or a second execution path.

**Implemented:** an adjacent selected-supplier control exposed a genuine producer bug:
composite admission checked extents without establishing its selected sheets, so the first
evaluation changed realization identity after consumer demand capture. Reconstruction admission
now returns a typed, unqualified actual point. Composite admission transports these points in
dependency order and establishes every supplier before capture. The actual selected-supplier
readout owns this transport. Certified evaluation and the strict consumed-parameter/sheet guard
remain required; the admission point supplies no accuracy certificate.

**Tested:** `just native-test` with the focused selection and `--test-threads 4` passed 27/27 against a
zero-failure baseline with native solver features, explicit force-validation and the memory cap.
The selection includes all six reported failures; actual reconstruction/refinement and original
correction; disjoint/chained admission and uncertainty controls; initialization; Root/KKT prediction;
Ipopt/POUNCE/SCIP root responses; strict declared-rung admission; and the weighted Jacobian control.
`just unit-package pse-math 'test(derived::tests)'` separately passed 19/19 with force-validation:
retained identity, consumed-product accuracy, missing-action refusal, ill-conditioning, zero-row
reconstruction and terminal original obligations. This is composite focused evidence after the
adjacent producer failure was repaired, not a full-suite rerun. The two reported publication
timeouts remain outside this task. The maintainer requested closeout followed by a stop; remaining
25n implementation and the proposed review remain paused.

**Interface-checked:** final `just clippy` completed both workspace/all-target modes (default and
no-default-features), and `just lint-solver-contracts` completed the linked native/all-target graph,
including conformance and benchmark consumers with force-validation. All use keep-going and
`-D warnings`; final results are zero errors and warnings against the zero baseline. Earlier
blocked consumers were rechecked after their prerequisite repairs. No comprehensive hygiene or
full scientific/durable qualification is claimed by this bounded closeout.

### Remaining implementation and validation

Resume only after the maintainer’s review instructions. Resolve the outstanding packet obligations
in the table: common runtime native work/storage admission and teardown ownership; automatic
initialization/causal binding and full original correction/product contracts; complete strict-work,
branch and optional/required controls; opaque scientific target migration; generated/public/durable
round trips; and N11 deletion/fixture handoff. Then 25k owns the affected static refresh, assembled
scientific/durable journeys, dev-profile reuse/resource measurements and final assessment. All AF-01–08
remain open pending their complete stated evidence. ADR status changes retain their decision route.

The maintainer’s latest instruction explicitly moves stale-worktree assessment/removal into this
closeout, superseding the earlier post-qualification timing. The assessment found ten old
25a/25b/25d/25e secondary trees with no commits outside main. Their product changes are integrated
or superseded; old helpers, changed authored IDs and patch fragments do not warrant transplantation
or indefinite retention. All ten obsolete worktrees and their two obsolete local branch refs were removed.
`git worktree list` now contains only the main checkout; no source transplant was needed.
