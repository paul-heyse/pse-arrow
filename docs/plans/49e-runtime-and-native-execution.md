---
title: Runtime and native execution remediation
status: in-progress
date: 2026-10-21
adrs: []
doc_role: plan
doc_owner: docs/plans/49-library-native-remediation.md
doc_topics: [design, numerical-execution, validation]
---

# Runtime and native execution remediation

Execution state and pause handoff: [the coordinator](49-library-native-remediation.md#pause-handoff--requested-closeout-2026-10-21).

Current proposed revisions: [the 2026-10-22 reassessment](49-library-native-remediation.md#remaining-scope-reassessment--2026-10-22).
Production execution remains paused; original authoring baselines and receipts retain their dates.

This companion develops R10–R16, retained-pool drain F04 and the fitting termination candidate F05 from the
[integrated review](../design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md)
and its [recommendation audit](../design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md).
The [coordinator](49-library-native-remediation.md) owns scope, rule decisions,
package status, finding disposition and combined qualification. Everything proposed here
is governed by that owner's recorded decisions; review judgments alone are not acceptance.
F01 and F02 have independently executable correction boundaries before the larger redesign.

The result is one runtime preparation and run lifecycle, adapter-owned numerical capability,
and shared scientific policies whose consumers stop restating them. The target preserves all
16 shared durable lanes, multiple processes, caller-independent durable work, native drain,
fences, explicit starts and independent original-space qualification. It introduces no execution
daemon. Store replacement, mathematical representation, preparation retention and schema/tooling
implementation belong to the adjacent companions; this document identifies their consumed contracts.

## Current runtime foundation and proposed operation composition

**Implemented/source-inspected, 2026-10-22:** shared supervision, typed requests/paired outcomes,
physical preparation, parameter-valued Simulation/Fit points, native capability/lifetime/status
work and fixed independent LM criteria are integrated with bounded evidence at the coordinator.
Current persisted operation/request is Version8 with four study operations and stored definition12;
Version7 descriptions below are the historical authoring baseline. Remaining consumer migration,
scientific/native/Python acceptance and publication stay open; the reassessment does not rebuild
these mechanisms from scratch or turn earlier focused evidence into package completion.

[Review O01](../functional_reviews/plan49-remaining-scope-and-library-integration-2026-10-22.md#O01)
adds STO09 scope-page/guarded-repair consumers to RUN03/RUN05: worker discovery, rotating readiness
recovery and dependent notification advancement use complete bounded database operations. Keep
`study_policy` as the pure semantic owner and preserve `Unknown => Reconcile` before cancellation,
pending scientific admission, exact seed permission and candidate/immediate predecessor read sets.
A repaired readiness hint cannot grant claim or native dispatch permission.

Bound aggregate premise bytes and distinct dependencies. Keep item-level complete revision fences,
notification source/before-cursor expectations and explicit accepted/stale/refused outcomes. A
changing candidate must not stall independent progress; advance only certified cursor work or
persist unfinished items. Unknown page effects reconcile immutable receipts before retry. Native
work and its flat stop flag remain under the existing supervisor outside database transactions.

Migrate all three worker routes and remove their per-point scope/repair loop when new targeted
controls preserve recovery and both notification/cursor orders. Existing request, map, start,
numerical outcome and drain contracts remain. This change depends on working STO09 contracts,
not a complete storage-engine rerun or an optional Surrealism module (I26/RC22).

Review O06/PRE09 may propose a separate finite configuration/explanation operation. Until RC23
is explicitly selected, do not expand `StudyOperation`, host request enums or public settings.
Any selected Z3 execution receives the original clock and finite native owner, returns owned
attributed results with Unknown distinct from Unsat, and joins physical work before releasing
resources. It remains separate from pure Salsa encoding and ordinary point dispatch.

## Basis and focused foundation assessment

Static inspection used checkout HEAD `e08b1f1dd` on 2026-10-21, including the current review
documents. It did not execute builds, tests, numerical probes or performance experiments.
Applicable criteria are Core 3.4, Efficient Architecture 1.0 and Process Simulator 1.5 from
the repository standard. Current contracts remain blueprint §13.5, §16.2, §17, §18 and §19.2–§19.3
in the [numerical](../authoritative_design/sections/numerical-execution.md) and
[workflow](../authoritative_design/sections/workflows-and-results.md) owners.

At the original authoring baseline, the inspected [cancellation wrapper](../../crates/pse-runtime/src/cancel.rs) already supplies
`child_with_deadline`, with downward-only cancellation and minimum inherited deadline tests.
The [horizon driver](../../crates/pse-runtime/src/workflow/horizon/driver.rs) still creates a
fresh source after admission. The [run owner](../../crates/pse-runtime/src/workflow/run.rs)
contains separate operation skeletons and independent `RunRequest`/`RunReport` enums. Native
jobs already join their worker before releasing compute; that is the foundation to preserve,
rather than replacing it with abort-on-drop task machinery.

At that baseline, [StudyOperation](../../crates/pse-runtime/src/workflow/study_operations.rs) was a strict
`Version<7>` persisted occurrence containing `PreparationSettings` and serialized
`OperationRequest`. It is not a generic host command. Its current four operations are
DeclaredCase, Simulation, Fit and Horizon; feature-gated host Shooting is a distinct capability.
[Binding admission](../../crates/pse-runtime/src/workflow/bindings.rs) refuses nonempty overlays
outside the admitted steady route. [IntegratedExperiment](../../crates/pse-runtime/src/workflow/integrated.rs)
already converts parameter-valued bindings into integration columns, but plain Simulation
does not construct that binding product. Reusing this primitive requires a Simulation builder.

The [Uno adapter](../../crates/pse-backend-native/src/execution/uno.rs) advertises one static
curvature capability while admission accepts limited memory only. Runtime initialization and
conditional strategies construct KINSOL map sessions outside the seam. The reviewed native
inventory and challenge establish the consequences and the map-specific replacement:
[inventory](../design_review/evidence/in-use-library-remediation-2026-10-21/inventory-native-solvers.md),
[numerical challenge](../design_review/evidence/in-use-library-remediation-2026-10-21/challenge-numerical.md).
No new library is needed to centralize these responsibilities. An adapter trait is justified by
actual method/settings variation; project start permissions and path premises keep their owners.

## Target contracts

### Cancellation authority, leases and native drain

A run receives a child of the token belonging to its execution owner, narrowed by the original
clock. For immediate work that owner is the caller; for a durable occurrence it is the worker
lane under `stop → drain → run`. Dropping a durable caller's handle does not become a stop
request for that lane. A caller must still be able to explicitly request a durable cancellation
through its existing authenticated/fenced operation, independently of handle lifetime.

One supervisor owns minting, attempt registration, dispatch, admission wait, join, completion
and publication. Operation implementations supply the admitted work and typed finish behavior.
The lease uses a cancellation drop guard while preserving the observable `run.stop_requested`
fact. [tokio-util 0.7.19 CancellationToken](https://docs.rs/tokio-util/0.7.19/tokio_util/sync/struct.CancellationToken.html)
provides `child_token` and `drop_guard`; the existing `CancelSource` carries the clock.
The library tree does not replace the native callback's flat atomic flag: polling the tree
can lock, while native callbacks require the current cheap leaf.

Exactly one supervisor bridge waits for cancellation, sets that leaf, awaits native destruction
and join, and only then releases resources and returns the typed stop outcome. Cancellation
wins over a candidate completing during the drain. Deadline expiry retains `TimeLimit`, rather
than being flattened into an undifferentiated cancellation. Partial horizons keep the completed
trajectory prefix and explicit partiality; no uncomputed endpoint is published as complete.
The same contract applies to failed admission, panic conversion and shutdown. An unpolled bridge
may delay stopping, but cannot permit early native resource release.

Worker lanes retain in-task `FuturesUnordered` draining. Aborting lane futures or returning
while owned native work remains would violate the durable execution contract. Conic, recycle
and block-initialization strategies currently return `SolveHandle` without durable run records.
The proposed RC13 choice is to state that existing exception. Moving them into durable supervision
is a separate functionality decision, not an implicit consequence of sharing a skeleton.

### Runtime requests and preparation

Runtime owns reusable operation request structs, preparation controls, validated document
admission and declared-procedure dispatch. Hosts adapt their transport into those meanings.
A host request superset wraps shared requests plus host-only operations, with a typed projection
into the study-admitted set. An unsupported projection fails before durable effects. This leaves
`StudyOperation` study-owned and preserves its admitted operation membership. Physical typed
requests advance StudyOperation/StudyRequest to `Version<8>` and stored definitions to 12;
the maintainer's fresh-data selection removes old readers rather than reinterpreting Version7.
If Shooting is later selected for studies, change the version and specify replay/regeneration
explicitly; never add a serialized Shooting variant as a side effect of removing a Python enum.

The runtime preparation entry consumes source/context, explicit `PreparationSettings`, operation
and admitted value bindings. It composes declared execution once, returning the operation-specific
prepared product or a typed refusal. Python stops selecting default profiles, choosing KINSOL,
renewing deadlines, synthesizing progress and composing admission chains. New external documents
still receive ingress validation: sharing runtime semantics does not erase that trust transition.
One deployment sequence owner serves both Python and worker startup, preserving ADR-0164's
per-host deployment assertion as a call into that owner.

One outcome enum pairs each admitted request with its `Result<Report>` by construction, eliminating
cross-variant matches and `_ => None` mismatches. `strum::EnumDiscriminants` and
`enum_map::EnumMap` can derive exhaustive operation-kind tables from that declaration; they do
not decide which operations a study admits. `PositiveCount` and a validated `LeasePolicy`
constructor own single-value and heartbeat/lease invariants. Serde's `try_from` route and its
schemars projection must expose those same admitted values to the boundary catalogue owner.

### Parameter-valued studies and shared scientific policy

Fit and Simulation study points admit parameter-valued overlays through the common request.
Preparation builds identity-to-integration-column bindings with physical conversions once.
Each occurrence binds new immutable values, retaining unchanged programs and artifacts while
re-establishing every changed numerical premise. Simulation gains the binding builder that
Fit already needs; calling an absent `PreparedSimulation::rebind` is not the implementation.
Each point uses its declared start or an explicitly recorded seed dependency, and recomputes
consistent DAE initial conditions. A previous point's successful native state is not an implicit
start for the next point, including retries and lane reassignment.

Initial-state, schedule and fit-data overlays remain bounded investigation leads. They require
new binding meaning beyond the parameter-column primitive and cannot be announced as delivered
by this package. Retained IDAS sessions through `IDAReInit`/`IDASensReInit` are also a lead: fresh
sessions are the selected baseline. A retained route must be identical under the relevant fresh
contract or expose its history as a recorded dependency before adoption.

The [preparation companion's PRE05](49c-resolved-model-and-incremental-preparation.md#one-compiler-policy-product-per-mode)
defines the pure `pse-model::study_policy` contract for
predicted-start eligibility and fallback, taking explicit facts rather than runtime service
handles. PRE05 also defines the compiler's numerical-annotation lowering for each analysis
mode and its view choice, hosted as tracked immutable products. RUN05 owns executor adoption.
Deliberate steady/implicit/dynamic differences remain explicit at this owner. Executors consume
these outputs instead of rebuilding annotation/native strings or repeating promotion predicates.
Adding an annotation changes its semantic lowering owner and relevant mode tests, rather than
forcing three orchestration implementations to acquire scientific policy.

### Native capabilities and compositions

The execution seam answers `capability_for(settings)` and publishes truthful union and
per-profile rows. Admission, routing, diagnostics and Python inspection consume those facts.
Uno SQP-LBFGS and SLP describe their actual curvature and support; no row promises an exact
Hessian route that settings admission refuses. A capability declaration is checked against
the route actually reached, including unlinked builds and explicit unsupported settings.

Add explicit `Problem::FixedPoint`/`Problem::Picard` inputs and a map-specific execution route.
It carries the map, variable scaling, original residual, declared restart schedule and authorized
starts. It must not inherit the square-root runner's normalization, Jacobian pattern checks,
warm transport or reuse assumptions blindly. Fixed-point/Picard demand no derivatives, and the
Picard map is never labelled a Jacobian. KINSOL remains the iterator through its existing
`Function` variants. Structural admission and final qualification use original residual meaning;
each application has a fresh session. A neutral reset is eligible only after separate evidence
establishes fresh equivalence or recorded warm dependence.

`BackendExecution::recovery_profiles(.., permission)` exposes bounded, separately admitted
recovery candidates. POUNCE lowers its existing `second_opinion_rungs`; the semantic owner passes
replacement-start permission, and perturbed starts become result dependencies. Do not use the
restoration driver that hides an `IpoptApplication` ladder inside one native call. Each rung
keeps its own outcome and original-space qualification. Separator consumption and JVP demand
become adapter facts. Only corrector backend suitability moves across the arclength seam;
explicit physical start, equal numerics key, bit-equal scale and presolve-off premises remain
with the path owner. PETSc NASM/pseudo-time stays composition-only without a decomposition input.

Adapter-owned diagnostics preserve bounded owned error text and panic containment at FFI.
For SUNDIALS, clear default context handlers with `SUNContext_ClearErrHandlers` before pushing
the selected bounded handler; pushing alone retains the default stderr path. Keep the existing
KLU observers, fixed-pattern application, IDAS checkpoint/replay budgets and original-space
checks when map/session code moves. A seam migration cannot delete integration hooks as cleanup.

### Uno binding and memory support

The bridge exports named `UNO_*` constants guarded by C/C++ `static_assert`s against its source
header, and Rust matches those names. `UNO_INFEASIBLE_SMALL_STEP` (iterate code 5) maps to
`Numerical`; code 3's stationary constraint-violation meaning remains distinct. Callback error
and cancellation latches still override native status. Original-space qualification never
turns a stall into an infeasibility conclusion merely because a candidate exists.

Retain legitimate integration hunks in `docker/solvers/uno-pse.patch`: deadlines, cancellation,
memory admission, bounded observation, serial HiGHS settings and readback. Evaluate and declare
four numerical hunks separately: outer-to-QP tolerance forwarding, zero QP regularization,
feasibility-LP seeding, and explicit lower-triangle materialization of the L-BFGS operator.
The selected target preserves SQP-LBFGS, rather than deleting its capability to simplify a patch.
Materialization has quadratic storage; derive/refuse its supported size under the actual admitted
allowance. The review's roughly 1,450-variable default-envelope arithmetic is not a measured
capacity promise. Moving to Uno IPM is a major-method addition and remains outside this target.

For direct FERAL KKT factors, use `SymbolicFactorization::work_estimate().peak_bytes` after
symbolic analysis and before numeric factorization as a priori admission. Keep post-factor
`KktFactor::bytes()` accounting and the complete hard envelope: delayed pivots can change real
storage. POUNCE-internal FERAL retains its bounded profile until that path has its own coverage.
An earlier refusal may reduce admitted cases and must be assessed explicitly, not sold as a free win.

PETSc `PetscMallocSet` and SuiteSparse allocator setters are conditional integration candidates,
not automatic replacements for the full foreign allowance. Hooks cover named allocator domains;
BLAS/MKL and bypasses remain outside them. Installation precedes global initialization. Every
allocation retains original owner and size across later free/realloc, another current lane or
thread, and process lifetime. Initialization allocations have a process owner. The current
PETSc process serialization is preserved. No thread-local current-attempt counter can alone
establish ownership, and partial domain coverage cannot authorize early charge release.

### Retained surrogate workers and shared parallelism

F04's [retained surrogate](../../crates/pse-runtime/src/math/surrogate/library.rs) waits for a
worker's solver/state destruction acknowledgement, then drops a custom `rayon::ThreadPool`.
That acknowledgement does not join the actual Rayon worker threads: pool drop requests termination.
Retain the real OS `JoinHandle`s through `ThreadPoolBuilder::spawn_handler`, terminate/drop the
pool after solver-state destruction, and join every owned worker before releasing its stack and
foreign-memory charge. Admission or worker-construction failure follows the same drain, including
partial thread creation. Preserve library-owned retained non-Send solver/state custody.
The [Rayon builder contract](https://docs.rs/rayon-core/1.13.0/rayon_core/struct.ThreadPoolBuilder.html)
also offers a scoped pool; that is a valid alternative only if it preserves this retained owner's
lifetime without rebuilding the library state for every proposal.

Global-pool-capable dependencies are a lead, not evidence that the global pool is reached here.
Identify the actual call path, its existing local `install` context and the embedding host's
initialization ownership before selecting global controls. Eager per-deployment `build_global`
can fail against an already initialized host or add workers without settling ownership; it is
not the selected universal correction. A reached escape needs a bounded local/admitted route
or an explicit host-owned global contract that accounts for the shared process. If no such
consumer is established within the inspected route, record that bounded result and retain the
inquiry's trigger rather than claiming the entire workspace has no global consumers.

## Execution packages and consumer cutover

The coordinator records RC07/RC08/RC10/RC12/RC13/RC15/RC16 with their consequences,
the operator's explicit decisions and routes. This companion's recommendation is settings-aware
capability, a declared Uno binding, parameter-valued Fit/Simulation points, host-only request
superset with current study membership, the stated explicit-strategy exception, direct FERAL
pre-admission and compiler-owned per-mode policy. RC12 does not recommend silently admitting
Shooting into studies. A rejected rule change preserves its current rule and requires the
coordinator to revise the dependent package before execution. ADR and architecture amendments
are prerequisites for the affected contract changes, not proof that their implementation passed.

These package definitions describe work and acceptance, not mutable package status. The
coordinator records decisions and readiness. Shared files require explicit executor ownership
when work starts even where logical dependencies permit parallel work.

| Package | Required input | Delivered boundary and deletion |
|---|---|---|
| RUN01 — horizon stop correction | Existing child-token/clock contract; no supervisor prerequisite | Replace the fresh horizon source with a child; caller cancellation and inherited deadline reach native drain; remove the shadow authority. |
| RUN02 — one run supervisor | RUN01 controls; selected RC13 lifecycle boundary | Simulation, fitting, modeling, shooting and horizon share lifecycle and one native bridge; worker drain is a stop child; delete per-kind skeleton/forwarding copies. |
| RUN03 — requests and runtime admission | RUN02 paired finish interface; coordinator RC12 decision; catalogue input contract; working STO09 scope/apply contract for worker cutover | Shared request/preparation structs and paired outcomes; finish actual host migration and bounded worker operation adoption; delete replaced chains/defaults/per-point coordination. |
| RUN04 — parameter-valued points | RUN03 working preparation/binding entry; stable parameter and physical-context identities | Simulation binding builder and Fit/Simulation overlays with explicit starts/fresh ICs; replace unsupported-only branches for exactly these admitted inputs. |
| RUN05 — scientific policy consumers | PRE05's working pure start-policy/compiler lowering/view products; STO09 scoped study facts | Migrate both predicted-start/fallback copies and all steady/implicit/dynamic annotation/view consumers onto PRE05; delete executor policy copies. |
| RUN06 — truthful execution seam | Selected RC07; original residual/scale/start contracts | Settings-dependent capability, fixed-point/Picard route and recovery profiles; migrate runtime initialization/conditional paths; only then remove Python's KINSOL choice in RUN03. |
| RUN07 — Uno outcome and binding | F02 mapping controls; selected RC08 for declaration | Named constants and truthful small-step outcome independently first; declare four numerical hunks and dense limit, preserving integration hooks and SQP-LBFGS. |
| RUN08 — native memory refinement | Selected RC15; existing complete admission/drain envelope | Direct FERAL estimate plus post-factor enforcement; adopt PETSc/KLU domain tracking only with original-owner/lifetime coverage, otherwise retain full envelope and explicit candidate disposition. |
| RUN09 — LM stopping semantics | Explicit fitting accuracy/eligibility inputs; existing stationarity/covariance withholding | Assess separate `with_ftol`/`with_xtol` and declared small `with_gtol`; adopt only a justified cosine-policy mapping, keeping the original-space stationarity gate. |
| RUN10 — retained parallel-worker drain | Current surrogate lifetime and admitted thread/stack/foreign allowance | Join actual retained pool worker handles before charge release; establish reached global-pool consumers before selecting any host/global controls; remove pool-drop-as-join assumptions. |

RUN01 and the mapping/constants part of RUN07 can land independently. RUN02 provides the
paired finish lifetime before RUN03 migrates hosts. RUN06's working map route is the specific
prerequisite for deleting Python's direct KINSOL selection; other RUN03 host work can proceed
earlier. RUN04 needs an implemented binding entry, not merely agreed request types. RUN05 can
start consumer design once PRE05 outputs are agreed, but migration completion needs the working
products and every consumer. RUN08
does not block request/policy work and cannot weaken existing memory admission while unresolved.

RUN09 addresses F05 as a real policy question, not an identity conversion. The pinned
[LM builder](https://docs.rs/levenberg-marquardt/) documents separate tolerances; the reviewed
git source `e67066c` shows `with_tol` disables `gtol` and makes `ftol=xtol`. Its `gtol` is
MINPACK's scale-free cosine test, not the repository's scaled absolute gradient. Compare existing
`lm_inquiry.rs` cases using the declared mapping, typed termination and evaluation counts;
require no newly unqualified fits or withheld covariance. If that condition fails, retain the
existing behavior with an explicit coordinator disposition rather than inventing a false mapping.

**Selected during execution, 2026-10-21:** I18 selects the pinned non-minpack library's
three independent default criteria explicitly: `ftol=xtol=gtol=30*f64::EPSILON`.
This is a library-relative cost/step/cosine policy, not a conversion from physical
stationarity. Preserve the original-space qualification and covariance gates. The
implementation must record the actual policy, effective work caps and exact profile/
replay identity, and remove the production `with_tol(stationarity)` mapping. Actual
baseline comparisons, a deliberate cosine/stationarity disagreement, low-cap and
cancellation controls remain required before RUN09 closes. This may change bounded
work; no speed benefit or new public tuning surface is selected.

**Interface-checked, I22:** no additional non-parameter overlay, wider history reuse or
deeper structural companion is selected. Current typed point replacement has no declared
meaning for replacing IC constraints, schedule topology or observation provenance.
Authored parameters, schedules and immutable source revisions retain their actual roles.
Existing IDAS restart and bounded neutral-storage paths already consume `IDAReInit` and
`IDASensReInit`; fresh construction remains required outside their exact admitted key.
The [SUNDIALS reset contract](https://github.com/llnl/sundials/blob/main/doc/idas/guide/source/Usage/SIM.rst)
retains options while resetting solution history; sensitivity count and adjoint history
have additional requirements. Existing whole/independent/conditional structural scopes
already serve the examined consumers. Revisit a named missing input meaning or measured
compatible setup burden, with explicit input/identity/history decisions and fresh state,
sensitivity, callback, error, cancellation and custody controls. This static disposition
does not add tests or claim universal bit equality or a performance improvement.

## Verification and integration contract

All acceptance below is **Proposed**. Historical probe C tests F01's caller-cancellation
divergence and probe D tests the synthetic Uno mapping; neither qualifies this implementation
or its inherited-deadline/native journeys. During implementation use package compile checks
and focused unit tests through the repository recipes, with explicit force-validation on actual
Arrow-consuming closures. Failure baseline is zero. The coordinator selects assembled journeys
and static/manual checks once the functional scope is implemented.

| Boundary | Independent targeted acceptance |
|---|---|
| RUN01/RUN02 stop and lifetime | Caller stop mid-horizon returns cancellation with a partial prefix; inherited deadline returns typed TimeLimit; child drop leaves caller/sibling live; a blocked native destructor retains compute/lane/view charge until joined. |
| Durable supervision | Caller disconnect leaves durable work running; explicit fenced cancel stops it; worker stop drains all 16 shared lanes; simultaneous processes cannot seal another attempt; no return drops active lanes. |
| RUN03 request semantics | Equivalent host/study requests produce equivalent admitted preparation; host-only Shooting projection refuses before effects; Version8 physical requests and stored definition12 are explicit with no historical reader; wrong request/report pairing is unrepresentable. |
| Admission and deployment | Malformed documents and invalid counts/lease relations fail at the semantic constructor; external ingress is revalidated; both hosts exercise the same deployment sequence with per-host assertion. |
| RUN04 scientific binding | Unit-converted Fit/Simulation parameter points equal independently prepared fresh points; parameter changes requiring new DAE ICs recompute them; retry/lane order cannot leak a preceding point's start. |
| RUN05 ownership | Table-driven mode cases exercise annotation differences and view promotion at the compiler owner; predicted-start/fallback tests need explicit policy facts without Runtime/native startup. |
| RUN06 capabilities | Per-settings advertised classes/derivatives equal actual admission/routes; fixed-point and Picard use shared scaling, authorized restarts and original-residual qualification with fresh sessions; recovery candidates remain separately admitted. |
| RUN07 Uno | Every named stop code has independently specified category; code 5 is Numerical; callback/cancel overrides survive; SQP-LBFGS and SLP retain original checks, serial controls and pre-materialization refusal. |
| RUN08 memory | FERAL estimate is consulted before numeric factorization and post-factor overrun still refuses; hook tests transfer current lane/thread before free/realloc and retain original/process owners; bypasses retain full-envelope coverage. |
| RUN09 fitting | Existing inquiry cases retain qualified results/covariance under any adopted stopping policy; cosine and physical stationarity tests deliberately disagree in a revealing case and publication obeys the latter. |
| RUN10 retained pool | Block a worker after solver-state acknowledgement and prove the resource lease remains held until the actual handle joins; constructor failure and cancellation join every created worker; any adopted global control exercises an actual reached consumer and preinitialized embedding host. |

Cross-plan integration consumes immutable prepared products from the math/preparation companions,
the store companion's guarded attempt/fence/maintenance semantics, and the boundary companion's
owned physical-input snapshot. Runtime supplies semantic request constructors, pairing and native
lifetime facts to those consumers. The schema catalogue owner must include actual compiled roots
and direction; it does not infer a schema change from non-serialized host enums. Tooling consumes
runtime admission/deployment/diagnostic launch owners rather than recreating them, through
[49g](49g-contracts-maintenance-and-tooling.md). Shared scientific lowering/start/view contracts
come from PRE05; RUN05 owns their orchestration consumers. Assembled evidence and durable design
publication are owned by [49h](49h-qualification-and-design-publication.md).

Measurements are scoped follow-up evidence, not universal feature gates. If claiming better
throughput, native capacity or preparation reuse, compare full admitted runs under named model
size, sparsity, concurrency, 16-lane/multi-process load, allowance and cancellation conditions.
Static removal of lifecycle/policy copies supports a locality claim only. Enduring contract and
ADR amendments follow the coordinator's accepted rule routes before dependent implementation.
