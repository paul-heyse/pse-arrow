---
title: Derivative reuse and evaluator capabilities
status: planned
date: 2026-10-15
review_sources: [docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md]
scenario_sources: [docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md]
doc_owner: docs/plans/41-simulation-pipeline-efficiency-and-effectiveness.md
doc_topics: [design, libraries, preparation, validation]
adrs: []
---

# Derivative reuse and evaluator capabilities

This companion removes repeated completed exact transient curvature at the existing
point owner, then resolves the distinct opportunities for native same-point history,
selected derivative actions and compiled/batched evaluation. It owns delivery for
[F14](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#f14)
and the O01/O03/O04 investigation routes. [Plan 41](41-simulation-pipeline-efficiency-and-effectiveness.md)
owns their dispositions and rule decisions; this document owns E-packet progress.
Completed curvature reuse is required independently of optional native checkpoints.

## Basis and foundations

The baseline is HEAD `a08f5fa091db1840fc95b46c4d4f849ce4c5de62` plus the concurrent
dirty checkout inspected on 2026-10-15. **Interface-checked:** the principal review,
[numerical inquiry](../design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/numerical-pipeline.md),
[library inquiry](../design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/library-fit.md)
and [independent assessment](../design_review/evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/independent-assessment.md)
were reconciled against the focused owners below. No authoring-time native execution,
new library probe or numerical measurement establishes the proposed routes.

| Existing owner | Established boundary and intended use |
| --- | --- |
| `crates/pse-runtime/src/workflow/fitting/oracle.rs`: `Demand`, `Point`, `evaluate`, `hessian`, `transient_curvatures` | Exact-point achieved-demand caching already retains predictions/responses/Jacobians/adjoint products. Every exact Hessian callback nevertheless computes transient residual curvature again. Extend this owner rather than create a parallel cache. |
| Runtime `workflow/{integrated,shooting,dynamics}.rs`, `workflow/fitting/{experiments,modeling}.rs` | Worker ownership, demand upgrades and original assessment already exist. Reports are observations, not native checkpoints; optional history must compose with actual worker lifetimes and changing scope. |
| Native `dynamics/idas.rs`, vendored SUNDIALS 7.1.1 and Diffsol 0.16.2 checkpoint/adjoint paths | IDAS sessions borrow oracle/context and have owner-affinity restrictions. Primal-only history cannot retroactively supply missing forward-sensitivity history. Diffsol interpolation may replay intervals and path cloning copies retained data. These constraints shape E2. |
| `crates/pse-math/src/implicit.rs`, `library.rs`, provider/assembly/jet owners | First directional products, sparse factors and transpose/multiple-RHS solves already exist. Implicit second-order evaluation expands residual jets before output projection; E3 must establish actual selected-action supplier and consumer contracts. |
| `Cargo.toml`; installed Symbolica 3.0.1 evaluation source | Multi-output CSE/Horner preparation and directional/dual evaluation are already used. The current feature profile omits `native_code_generation`. Shared math skill 3.0.0 is discovery context, not exact 3.0.1 qualification. |

Blueprint §7/§14 owns mathematical/derivative preparation, §17/§18 owns native
execution and independent assessment, §19 owns workflow responses and attempts,
and §25 records supported/excluded capabilities. Core 3.4, efficiency heuristics
and Process Simulator 1.5 require the consumed mathematical meaning and physical
execution to agree. They do not require one universal evaluator or eager derivatives.

## Target and consequential distinctions

### Completed exact curvature at the existing point

Fitting retains the completed transient residual-curvature block
`Σ rᵢ wᵢ ∇²yᵢ` alongside the exact point's compatible products. Its premises include
the experiment and included observations, scientific residual weights, bindings and
directions, provider/profile, mode/schedule, starts and accuracy. Premises immutable
in the prepared owner may be represented by that owner; do not duplicate all facts
into a new global key. Exact point equality retains the existing semantics.

The outer native objective weight and Lagrangian multipliers are applied after the
underlying transient curvature product. Changing those solver weights does not
invalidate that product. Changing observation/residual weighting does. This is not
an unweighted response Hessian tensor: choosing a broader retained tensor would need
its own useful consumer and storage justification. Preserve steady-experiment
constraint curvature and Gauss–Newton behavior.

Publish a complete successful product coherently with its point; an error or cancelled
upgrade cannot advertise a completed curvature capability. Failed/old points release
their owned products. The independent final original-state assessment remains a
distinct assessment mode, and may deliberately request fresh original sensitivities.
No native checkpoint redesign is needed to deliver this correction.

### Same-point integration capability frontier

E2 compares demanded replay, checkpointed primal and eager requested first-order
capability using actual fit/shooting callback demand. Existing exact-point caches
already avoid many repeated integrations; value-to-first-order replay is a legitimate
tradeoff when trial points are rejected before derivatives are requested.

An adopted frontier owns the compatible equation/provider, point, profile, modes,
schedule/events, starts, accuracy, sampled output meaning, scope and finite leases.
It knows which primal, tangent, adjoint and curvature capabilities were actually
recorded. A missing direction/history must be built or honestly replayed. IDAS
primal-only checkpoints cannot invent tangent history; `IDAAdjReInit` deletes
checkpoints. Reports cannot substitute for those native products. Diffsol path
interpolation is mutable and may replay intervals; avoid deep cloning to conceal
ownership problems. Keep native borrowers and `!Send` owners on their actual thread.

Changed IVPs still initialize freshly under ADR-0186. Same-point retention does not
authorize changed-trial continuation or portable checkpoints. Compare retained
working set, failed-trial eviction and integration/upgrade/replay counts; a blanket
checkpoint policy can worsen work and admission. The simplest sound demand replay
policy is an acceptable investigation result if alternatives do not earn adoption.

### Selected derivative actions

E3 starts with actual consumers requesting a direction, cotangent or selected second
contraction, rather than inventing a generic action API without a user. Establish
complete supplier capability, derivative order, support/guards, dimensions and output
meaning through providers, IFT assembly and factor solves. Preserve Taylor/factorial
conventions, symmetry, original coordinates/scaling and provider failure propagation.

Use existing first directional evaluation, transpose solves and multi-RHS factors.
The library inquiry's `SampleElimination` lead belongs to E3: inspect whether repeated
work at an unchanged sample point can share algebraic elimination and its compatible
numeric factor. Across changed samples, retain only compatible symbolic/pattern
preparation and refresh numeric factors. Do not make numeric-factor reuse universal
or add a second sample cache without an actual consumed lifetime.

Where opportunistic multiple-RHS batching can alter nonlinear trajectories, retain
the pinned POUNCE `multi_solve_matches_single_solve(nrhs)` eligibility condition.
Agreement only within an output tolerance does not establish that batching is an
eligible replacement for the consumed nonlinear operation.

Narrow second-order residual expansion only when suppliers can honor the consumed
action; opaque full-jet suppliers cannot be assumed narrowable. A requested full dense
Hessian still pays its result extent. Do not introduce another solver or a competing
cache to remove an intermediate. Independently perturbed original responses provide
a useful control against two mutually consistent but wrong derivative transforms.

### Compiled/batched evaluator qualification

E4 compares exact-version Symbolica compiled/JIT/SIMD/batched paths with existing
interpreted multi-output/directional evaluation only after avoidable whole work is
understood. The selected profile must preserve no-FMA/floating-point and ISA policy,
domain/branch guards, nonfinite/error meaning, provider callbacks, cancellation,
coordinate ordering and independent assessment. Include compilation amortization,
executable/scratch allocation ownership, retained lifetime, lane packing and tails.
Do not enable native code generation globally, change tolerances or silently select
a new evaluator default to complete this investigation.

Any adopted capability expansion waits for the coordinator's concrete
[RC03](../design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md#rc03)
decision and required supported-profile/design route. An unresolved floating-point,
provider or lifecycle mismatch prevents adoption. A narrowly supported explicit
route is eligible if its limitation is enforceable and honest; otherwise retain
the interpreter. Optional batching must define failures per actual batch operation
and cannot reinterpret lane padding as a scientific input or result.

## Execution packets

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| E0 — demand/product lifetime foundations | Principal source baseline; current exact-point, worker, provider and native demand contracts | Specify point premises and completed-curvature ownership; trace actual fit/shooting demand orders and supplier products. Separate existing reuse from missing capability, completed products from checkpoints, and scientific residual weights from outer solver weights. Settle the minimum E1 contract without waiting for E2. Record material interface/rule questions with their dependent packets. | Planned |
| E1 — completed residual-curvature reuse | E0 point/product contract; current fitting exact mode | Retain successful compatible transient curvature at the existing Point owner and recombine native weights later. Migrate every exact Hessian path, retain failure/point eviction and independent final assessment, and remove unconditional recomputation plus obsolete repeat-work fixtures. Same-x changed native weights reuse; changed scientific context invalidates; failed upgrades cannot poison or advertise a complete product. | Planned |
| E2 — same-point integration frontier investigation/adoption | E0 actual demand/owner premises; 41d D2 only for adopted window-worker-dependent retention; 41d D1 when adopted IDAS history consumes revised layout accounting | Compare demand replay, eager requested sensitivities and retained eligible checkpoints against actual callback sequences, integrations/replay and combined retained state. Select an enforceable method/capability policy, implement adopted retention and migrate real fit/shooting consumers, or retain demand replay with bounded reasons. Qualify missing directions, changed IVPs, rejected/failed trials, affinity and cleanup; no zero-replay promise. | Planned |
| E3 — selected derivative actions investigation/adoption | E0 actual consumer demand; existing provider/IFT and sparse-factor contracts | Identify a real selected-action expansion gap and settle its supplier-to-consumer contract. Implement justified action composition with actual consumers and delete replaced full expansion there, or retain full evaluation for unsupported consumers with explicit evidence. Check independent perturbations, higher-order conventions, guards, dimensions and provider refusals. New kernel/provider contracts follow the required decision route first. | Planned |
| E4 — compiled/batched evaluator qualification/adoption | Exact installed library/source and feature inquiry; coordinator RC03 decision before changed capability; 41f F0 diagnostic vocabulary for qualification observations | Qualify semantic/FP/ISA/provider/cancellation and allocation/lane/tail contracts, compilation work and repeated evaluation fit. Adopt only a concrete supported route and migrate its consumers; otherwise record refusal/limitations and retain interpreted evaluation. No global feature/default change is inferred. Reconcile any exact dependency move through normal pinned-dependency rules. | Planned |
| E5 — derivative/evaluator migration and targeted closure | E1 complete; E2–E4 decisions and adopted implementation/migrations complete | Confirm every scoped Hessian/action/evaluator consumer uses its selected target and replaced code/tests/fixtures are deleted. Reconcile finding/investigation obligations at Plan 41 and hand package results, capability limits and final journeys to 41f. No investigation is silently left as optional unfinished completion scope. | Planned |

E1 can close F14 without E2–E4. E2/E3 can inspect source independently; implementation
dependencies arise only from the capability adopted. 41a's selected preparation can
improve shared mathematical foundations without blocking E1. 41f F0/F1 supplies
diagnostic vocabulary and incremental integrations; this companion supplies actual
demand, completed-product, upgrade, integration, replay and evaluator construction
facts. Observations cannot trigger a new scientific evaluation or govern cache validity.
Coordinate shared edits to integrated/native/provider owners with 41d rather than
mistaking logical independence for disjoint files.

## Investigation exits and meaningful controls

For E0/E2/E3/E4, record the question, exact source/features inspected, settling
evidence, selected alternative, implementation/migration consequence and remaining
support limits in this owner. Adopted alternatives include functional delivery in
their packet. A refusal identifies the technical mismatch or absent useful consumer
and the retained working route. If a material premise remains unresolved, return
the concrete decision to Plan 41; do not treat a future investigation as completed
work or an ownerless backlog. No numerical benefit is assumed from an API's existence.

E1 controls use repeated exact Hessian callbacks at bitwise-identical x with changed
outer objective weights and multipliers; compare against an independently assembled
weighted result and assert no additional transient curvature integration. Change an
included observation/weight, binding/direction, provider/profile, start, schedule or
accuracy premise at its supported owner and require fresh compatible work. Include
unchanged repeated demand, changed x, Gauss–Newton, no-free-parameter/observation
experiments, failed/cancelled curvature and point replacement/release. Keep final
assessment mode's independent numerical/rank checks.

For S07, E2 exercises value→value, value→forward, forward→adjoint, repeated adjoint
with changed cotangent, missing second-order tangent directions and changed point.
Count actual completed products, integrations/upgrades and replay; compare independent
fresh outputs/derivatives. Test rejected trials and held resources through native
destruction. E3 exercises directional/transpose/multi-RHS equivalence, asymmetric
input/order mistakes, branch/support changes and an opaque provider that must refuse
unsupported action narrowing. Full dense demands retain their honest result size.

E4 uses scalar/multi-output/domain-boundary/provider-error cases, cancellation during
preparation/evaluation, partial SIMD batches and tails, scratch/executable teardown
and supported-target FP behavior. One complete relevant observation may describe
amortization/work removed; exact comparative timing, statistical sampling and hard
performance criteria are not acceptance. S05 response meanings and S09 unsupported,
branch/scaling and original acceptance stay intact where these changes touch them.
S10 requires a new provider/evaluator to implement the consumed capability locally
without changing workflow policy or claiming unsupported derivative orders.

Use native-solver-libraries and symbolica-faer-oximo skills, Context7 and the exact
selected release/vendor source when an API question remains. Current source baselines
are Symbolica 3.0.1, faer 0.24.4, vendored Diffsol 0.16.2 and SUNDIALS 7.1.1 through
sys 0.6.2. Do not use stale utilization/capability catalogs, transfer 3.0.0 evaluator
assumptions to 3.0.1, or treat shared skill probes as current composed acceptance.

## Verification and current state

**Proposed:** all E packets and anticipated benefits. **Interface-checked:** the
focused current source and review baseline above. Authoring ran no product tests,
builds, measurements or services. F14's repeated-product diagnosis is source-backed;
callback frequency and optimizer-level benefit remain unmeasured.

Execution uses scope-aware `just check-package` and targeted `just unit-package`
for touched `pse-runtime`, `pse-math` and `pse-backend-native` closures, with selected
native features/environment and explicit Arrow force-validation where required.
Report command, mode, scope and actual zero-baseline result. 41f owns assembled
native/workflow/Python qualification and full hygiene/manual checks after functional
scope; no per-packet whole-product campaign or environment/provenance-proof work
is introduced. Numerical assertions and actual cancellation/drain remain mandatory.

No existing plan, architecture or accepted ADR is changed by authoring this companion.
Plan 41 owns decisions and eventual enduring documentation transfer. E0 is the first
local step; E1 then delivers the compulsory correction while conditional inquiries
proceed through their explicit dependencies.
