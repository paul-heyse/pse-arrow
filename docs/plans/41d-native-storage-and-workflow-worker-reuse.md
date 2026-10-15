---
title: Native storage and workflow worker reuse
status: planned
date: 2026-10-15
review_sources: [docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md]
scenario_sources: [docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md]
doc_owner: docs/plans/41-simulation-pipeline-efficiency-and-effectiveness.md
doc_topics: [design, libraries, preparation, validation]
adrs: []
---

# Native storage and workflow worker reuse

This companion makes native admission follow the representation and method actually
consumed, and retains compatible mathematical/provider workers for the lifetime of a
shooting or horizon attempt. It owns delivery for [F12](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f12)
and [F13](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f13),
and the bounded O05/O06 investigations. [Plan 41](41-simulation-pipeline-efficiency-and-effectiveness.md)
owns all finding dispositions and rule decisions; this document owns D-packet progress.
Optional window parallelism is included as an explicit decision route, and is not
necessary to close F13.

## Basis and existing foundations

The baseline is HEAD `a08f5fa091db1840fc95b46c4d4f849ce4c5de62` plus the concurrent
dirty checkout inspected on 2026-10-15. **Interface-checked:** the principal review,
[numerical inquiry](../design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/numerical-pipeline.md),
[library inquiry](../design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/library-fit.md)
and [independent assessment](../design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/independent-assessment.md)
were reconciled with the focused source owners below. No new runtime measurement,
build or test establishes the proposed target. The interrupted cold preparation case
did not reach numerical execution and does not identify a numerical bottleneck.

| Foundation and source owner | Reuse and required correction |
| --- | --- |
| `crates/pse-backend-native/src/dynamics.rs`: `Profile::native_scatter_bytes`, checkpoint and second-order admission | Existing finite admission is useful. Its scatter allowance derives from `states²`, before the actual all-mode sparse support; migrate every consuming admission route, not only the initial native constructor. |
| `crates/pse-backend-native/src/dynamics/idas.rs`: `Session::prepare`, `PartialSlots`, consistent initialization | Existing sorted all-mode union, transpose/diagonal maps and out-of-pattern checks provide the real layout seam. Consistent initialization charges dense KLU fill even for SPGMR/SPFGMR; separate selected-method storage. |
| `crates/pse-runtime/src/workflow/modeling/dynamics.rs` | Base dynamic preparation already admits native scatter storage. It must consume the same prepared layout/demand as the native adapter, rather than retain a separate dense estimate. |
| Runtime `workflow/integrated.rs`, `workflow/shooting.rs`, `workflow/horizon/driver.rs` | Window integration/adjoint helpers construct fresh workers; shooting retains reports, and horizon initialization constructs an additional worker. Move actual callers to attempt-owned workers, including initial-condition evaluation. |
| Runtime `workflow/fitting/{oracle,experiments}.rs`; native `execution.rs`, `routing.rs` and retained adapters | Fitting already demonstrates owner-affine experiment workers; native adapters already contain meaningful reset/update/refactor facilities. Reuse those seams before adding another pool, cache or solver route. |
| Native `presolve.rs`; pinned POUNCE fork | `UNTRACKED_BOUND_DUALS` protects original reactions by disabling execution of LinearBounds/FBBT. Existing FBBT infeasibility prechecks remain useful; affine elimination recovery does not repair general tightening provenance. |

Blueprint §15–§18 own native eligibility, resources, recovery and original assessment;
§19 and ADR-0186 govern workflow attempts and changed-IVP behavior. The applicable
criteria are Core 3.4, efficiency heuristics and Process Simulator 1.5. The target
corrects execution within those consumed guarantees; it introduces no generic solver
engine or second model representation.

## Target contracts and alternatives

### Complete layout and selected native storage

The dynamic preparation owner supplies complete conservative support across every
admitted mode, including required diagonals, transposes and second-order mappings.
The adapter consumes this layout without rediscovering it independently. Storage
admission accounts for the chosen containers, temporary support construction and any
simultaneously live old/new layout. Support cannot be inferred from the current mode
or a successful derivative sample. A derivative outside declared support still fails.

D0 chooses between retained scatter destinations and compact column lookup using the
actual callback/access contract. It does not prescribe a new hash-map abstraction.
Known sparse map storage follows occupied structure and container capacity; genuinely
dense support still pays its dense extent. Native direct-factor fill is a separate
finite opaque allowance because input nonzeros do not bound fill. Krylov routes admit
their basis, vectors, headers and selected preconditioner; they do not pay for absent
KLU factors. Overflow and insufficient allowance refuse before the relevant allocation.
Charges survive callbacks and cancellation until native destruction/drain.

### Attempt-owned workers, fresh changed IVPs

A shooting attempt retains compatible mathematical/provider workers at its actual
window/lane owner. A horizon plant retains its compatible worker on the staged owner
thread across initial evaluation and periods. Each use binds the current values,
anchors, time/window, profile, providers and execution/diagnostic scope. Incompatible
structure/provider or unsupported profile reconstructs deliberately; changed numeric
state invalidates dependent caches and initializes the native IVP under its existing
contract. Worker construction reuse is distinct from integration reuse.

Do not move `!Send` native owners between arbitrary executor threads, keep a stale
cancellation scope, or continue multistep history across a changed IVP. Rebind/reset
must be supported by the actual worker/provider owner; retaining a handle while its
internals still reconstruct everything does not establish the intended benefit.
Fitting's existing lifecycle is a reference, not permission to broaden provider reuse.

Only independently anchored windows **within one shooting trial** are candidates for
parallel waves. Initial-point endpoint construction, rolling-horizon feedback and
ordered reductions retain their causal order. Independent optimizer trials are not
selected. A wave admits the combined native teams, workers, factors and checkpoints
before dispatch; cancellation retires and joins every actual owner. Serial retained
execution remains a valid selected result when parallel ownership or live state is
unfavorable.

### Class routes and presolve decisions

O05 compares proved affine square-root execution through existing library linear
algebra with current KINSOL, and separately traces existing native preparation across
real study lifetimes. Bounds, scaling, proof dependencies, seeds and original-state
assessment remain authoritative. A linear equation solution alone does not solve a
different bounded feasibility problem. Existing KINSOL routing is class-correct;
this inquiry must establish useful work removed before adding a route.

O06 compares library-owned original-reaction provenance/recovery with an explicitly
primal-only tightening route. Current restrictions stay until the coordinator records
the concrete [RC04](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#rc04)
decision and any required architecture/ADR change is available. Primal-only support
must represent dual/KKT/sensitivity unavailability honestly, including downstream
analysis and publication; it cannot silently satisfy a full-reaction request. SCIP
global bounds and affine-elimination recovery do not establish general FBBT reactions.

For recycle maps, preserve explicitly frozen inner starts and branches. Rebinding or
retaining a neutral/native worker must not carry hidden continuation state into the
next map evaluation and change the function being solved. Existing reset/update
facilities are useful only when they honor that actual workflow policy.

## Execution packets

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| D0 — layout/method and owner-lifetime foundations | Principal source baseline; current dynamic contract and actual admission consumers | Settle all-mode layout ownership and representation, selected-method storage, construction overlap and worker/provider rebind/reset contract. Inventory base/checkpoint/second-order/consistent-initialization consumers. Use source and small support/ownership controls; no large numerical rerun. Record selected alternative and any discovered rule impact at Plan 41 before dependent implementation. | Planned |
| D1 — sparse method-aware native admission | D0 complete layout/storage contract; implemented cancellation scope from 41b B1 before long native controls | Migrate runtime preparation and every IDAS admission/constructor to one complete prepared layout. Remove state-square scatter estimates and unconditional KLU fill for Krylov. Preserve all-mode/transpose/diagonal/second-order maps and finite real-drain ownership. Targeted sparse-growth, dense-refusal, changed-support and selected-method controls pass. | Planned |
| D2 — retained shooting/horizon workers | D0 settled rebind/reset contract; 41b B1 cancellation propagation for composed cancellation controls | Implement the rebind/reset boundary and retain workers at actual shooting attempt/window and horizon staged-thread owners; migrate integration, window adjoint, initial-state and period consumers. Delete reconstruct-per-call helpers and their obsolete fixtures once migrated. Repeated compatible calls reduce construction counts; changed scope/provider/IVP cannot reuse stale state; fresh-reference numerical/derivative controls and teardown pass. | Planned |
| D3 — affine routing and study lifetime inquiry/adoption | Current class proof, original assessment and retained-adapter contracts; 41c C1 local admission only for public local-study acceptance | Trace one actual sweep/study owner into existing native reset/update/refactor paths. Compare direct affine-root routing with current KINSOL using independently specified bounded/scaled/singular cases and actual construction/factor/callback work. Adopt a useful compatible route and migrate its real consumers, or retain the existing route with a bounded technical rationale. Any new backend/kernel contract follows the decision route first. | Planned |
| D4 — reaction provenance or primal-only investigation/adoption | Current exact POUNCE fork and original reaction contract; coordinator RC04 decision before changed-policy implementation | Establish whether original tightening reactions can be recovered through library-owned provenance, or define a concrete demanded primal-only contract. Test original feasibility and positive/negative dual-availability controls. Implement and migrate only an accepted, qualified route; otherwise retain restrictions/prechecks and record the limiting evidence. Do not enable general passes by option changes alone. | Planned |
| D5 — independent window-wave investigation/adoption | D2 retained owner/reset delivery; finite combined native admission; 41f F0 diagnostic vocabulary | Compare serial retained windows with finite admitted waves for one trial. Establish dependency, affinity, combined live state, cancellation/join and ordered reduction. Adopt and qualify an eligible wave or explicitly select serial execution with reasons. No independent trial parallelism or hard timing threshold is required. | Planned |
| D6 — numerical migration and targeted closure | D1/D2 delivery; D3–D5 decisions and every adopted consumer migration complete | Confirm all actual native-storage/window consumers migrated and replaced mechanisms/fixtures deleted. Reconcile source findings at Plan 41; hand targeted results and remaining assembled journeys to 41f. No package completion is inferred from a declaration or agreeing an interface. | Planned |

The D numbers are stable references, not a total ordering: D3 and D4 can investigate
independently of D1/D2, and D5 is never a prerequisite for closing worker reconstruction.
41f F0/F1 supplies shared diagnostic meaning and incremental producer integration;
this companion supplies the actual layout, factor, worker-construction and owner facts.
Do not make observation a prerequisite for pure layout inspection or invent a second
resource controller. D/E packages may edit the same integrated/native owners and
must coordinate file ownership despite independent logical dependencies.

## Investigation exits and preservation controls

Every D0/D3/D4/D5 inquiry records its question, inspected versions and consumed
contract, settling evidence, selected alternative and dependent scope in this owner.
An adopted alternative includes its implementation and targeted migration in the
packet. A refused alternative states why and what useful existing route remains.
Unsettled material semantics return to the coordinator for a concrete decision;
they do not become an implicitly completed or indefinitely deferred opportunity.

The selected source baseline includes vendored SUNDIALS 7.1.1 through sys 0.6.2,
Diffsol 0.16.2, faer 0.24.4 and POUNCE 0.12.0 at
`d02beafd370fd3815d6c0541fab1630ff839a739`. Recheck exact consumed source/features
when executing. Use the native/math library skills and Context7 for unresolved API
questions; do not transfer old skill probes into current assembled qualification.
Licensing does not exclude an otherwise sound alternative.

For S06, use diagonal and banded growth cases, a genuinely dense refusal, support
introduced only in another mode, transpose/second-order mappings and deliberate
out-of-pattern output. Exercise KLU and selected SPGMR/SPFGMR including consistent
initialization; distinguish known containers from opaque factor allowance. Failed
construction, cancellation and normal completion must release only after real drain.

For S07/S08, compare retained and fresh independently constructed workers for value,
forward response and supported adjoint demands; change values, anchors, window time,
scope, provider and profile deliberately. Repeated compatible windows must retain
construction while changed IVPs still integrate freshly. Exercise failed windows,
mid-wave cancellation and owner destruction. Keep causal initial-point and horizon
feedback assertions. Extend the actual diagnostics with construction/reset/integration,
selected storage/factor and active-owner facts without extra scientific evaluation.

Retain S09 initialization/recycle failures, original feasibility, scaling/branch and
typed unsupported boundaries where touched. S10 extension acceptance ensures a new
method supplies its own storage/reset contract without modifying unrelated workflow
policy. New support for high-index DAEs, hybrid derivatives or portable native history
is not implied.

## Verification and current state

**Proposed:** all D packets and benefits. **Interface-checked:** the focused source
foundation above. No product change, runtime test or measurement was performed while
authoring this companion. No numerical speedup, convergence superiority or capacity
claim is made.

During execution, use scope-aware `just check-package` and targeted `just unit-package`
for `pse-backend-native` and `pse-runtime`, with the selected native features/environment
and explicit Arrow force-validation for correctness closures. Record command, mode,
scope and result against a zero-failure baseline. Do not run whole hygiene/integration
after every packet. 41f owns assembled native/workflow/Python acceptance and final
static checks after all functional scope. Construction/work counts test the mechanism;
wall time is descriptive and has no hard threshold or repeated-sampling requirement.

No existing plan, architecture or ADR is changed by authoring this document. Plan 41
owns accepted decision routes and later transfer of enduring meaning. D0 is the first
local dependency-ready step; D3/D4 source inquiries may start in parallel when assigned.
