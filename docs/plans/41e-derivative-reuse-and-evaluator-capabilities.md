---
title: Derivative reuse and evaluator capabilities
status: in-progress
date: 2026-10-15
review_sources: [docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md]
scenario_sources: [docs/design_review/reviews/design_review_simulation-pipeline-efficiency-and-effectiveness_2026-10-15.md]
doc_owner: docs/plans/41-simulation-pipeline-efficiency-and-effectiveness.md
doc_topics: [design, libraries, preparation, validation]
adrs: [ADR-0191]
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
| E0 — demand/product lifetime foundations | Principal source baseline; current exact-point, worker, provider and native demand contracts | Specify point premises and completed-curvature ownership; trace actual fit/shooting demand orders and supplier products. Separate existing reuse from missing capability, completed products from checkpoints, and scientific residual weights from outer solver weights. Settle the minimum E1 contract without waiting for E2. Record material interface/rule questions with their dependent packets. | Completed |
| E1 — completed residual-curvature reuse | E0 point/product contract; current fitting exact mode | Retain successful compatible transient curvature at the existing Point owner and recombine native weights later. Migrate every exact Hessian path, retain failure/point eviction and independent final assessment, and remove unconditional recomputation plus obsolete repeat-work fixtures. Same-x changed native weights reuse; changed scientific context invalidates; failed upgrades cannot poison or advertise a complete product. | Completed |
| E2 — same-point integration frontier investigation/adoption | E0 actual demand/owner premises; 41d D2 only for adopted window-worker-dependent retention; 41d D1 when adopted IDAS history consumes revised layout accounting | Compare demand replay, eager requested sensitivities and retained eligible checkpoints against actual callback sequences, integrations/replay and combined retained state. Select an enforceable method/capability policy, implement adopted retention and migrate real fit/shooting consumers, or retain demand replay with bounded reasons. Qualify missing directions, changed IVPs, rejected/failed trials, affinity and cleanup; no zero-replay promise. | Completed |
| E3 — selected derivative actions investigation/adoption | E0 actual consumer demand; existing provider/IFT and sparse-factor contracts | Identify a real selected-action expansion gap and settle its supplier-to-consumer contract. Implement justified action composition with actual consumers and delete replaced full expansion there, or retain full evaluation for unsupported consumers with explicit evidence. Check independent perturbations, higher-order conventions, guards, dimensions and provider refusals. New kernel/provider contracts follow the required decision route first. | Completed |
| E4 — compiled/batched evaluator qualification/adoption | Exact installed library/source and feature inquiry; coordinator RC03 decision before changed capability; 41f F0 diagnostic vocabulary for qualification observations | Qualify semantic/FP/ISA/provider/cancellation and allocation/lane/tail contracts, compilation work and repeated evaluation fit. Adopt only a concrete supported route and migrate its consumers; otherwise record refusal/limitations and retain interpreted evaluation. No global feature/default change is inferred. Reconcile any exact dependency move through normal pinned-dependency rules. | In progress — scalar controls passed; remaining direct wire-version consumers migrating |
| E5 — derivative/evaluator migration and targeted closure | E1 complete; E2–E4 decisions and adopted implementation/migrations complete | Confirm every scoped Hessian/action/evaluator consumer uses its selected target and replaced code/tests/fixtures are deleted. Reconcile finding/investigation obligations at Plan 41 and hand package results, capability limits and final journeys to 41f. No investigation is silently left as optional unfinished completion scope. | In progress — targeted acceptance complete; shared generation/documentation handoff underway |

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

## Authoring verification and execution route

**Proposed at authoring:** all E packets and anticipated benefits. **Interface-checked:**
the focused source and review baseline above. Authoring ran no product tests, builds,
measurements or services. The investigation decisions and execution Outcome below own
the later delivery state. F14's repeated-product diagnosis is source-backed;
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


## Investigation decisions — 2026-10-15

**Interface-checked and Implemented — E0/E1:** the immutable prepared fit owner supplies
scientific context and observation/residual weights; the point is keyed by the existing
exact-bit x semantics. A complete successful vector of experiment residual-curvature
blocks is retained at that point. Outer objective/Lagrangian weights are recombined at
assembly, so repeated Hessian demand does not repeat curvature integration. Same-point
compatible demand upgrades can move completed blocks into the replacement without
cloning; failed upgrades publish neither old nor partial new products. Changed x,
scientific prepared owner, failure and cancellation release incompatible products.
Dense result/container extent is admitted at the fit job before callback effects.
Steady constraint curvature, Gauss–Newton, no-observation/no-free-parameter behavior
and independent original-rank assessment retain their contracts. Response demand now
also acquires the constraint Jacobian required by the steady implicit closure.

**Interface-checked — E2, select achieved-demand replay:** traced actual fit/shooting
value, response, forward, adjoint and exact-curvature callbacks against SUNDIALS 7.1.1
and Diffsol 0.16.2. Completed compatible products already reuse; E1 removes the repeated
exact-curvature operation. A value trial may be rejected before any derivative demand.
Eager sensitivities/checkpoints would perform and retain unnecessary work at those
points. IDAS primal history cannot retroactively supply tangents, `IDAAdjReInit`
destroys checkpoints, and Diffsol path cloning/interpolation can copy or replay native
work. No remaining compatible-history consumer earns a new owner/checkpoint policy.
Retain coherent achieved-demand caching and honest replay on missing capabilities,
fresh changed-IVPs, failed-point eviction and original final assessment. A zero-replay
promise or blanket eager derivative default is not selected.

**Interface-checked and Implemented — E3:** existing directional first products,
transpose solves and qualified multi-RHS use remain the supplier/consumer route.
`SampleEliminationPreparation` now retains only compatible canonical CSC pattern and
shared faer `SymbolicLu` at the actual IDAS context or Diffsol adjoint owner. Changed
samples refresh numeric factors; unchanged patterns reuse symbolic analysis. Pattern
changes replace it, unknown state support refuses, and failed numeric factorization
cannot publish a successful factor. Within one elimination, multiple RHS still share
its one actual numeric factor. Known sparse construction/pattern storage and finite
opaque algebraic factor allowance are included in checkpoint/second-order admission;
no parallel sample cache or universal numeric-factor reuse is added.

The pinned POUNCE fork's low-rank augmented-system consumer already checks
`multi_solve_matches_single_solve(nrhs)` before its batching replacement; retain that
eligibility. Full implicit second-order suppliers currently deliver full residual jets
and Hessians. No actual selected second-contraction supplier contract across opaque
providers is established, so retain full demanded expansion and its honest result
extent. This bounded no-adoption does not advertise unsupported higher-order actions.

**Interface-checked and Implemented — E4, explicit scalar selection:** exact installed
Symbolica 3.0.1 exports provider-free scalar value blocks. Adopt optional
`pse-math/compiled-evaluator` with typed `Optimization.evaluator = CompiledScalar` and
finite `module_bytes`; interpreted evaluation remains the default. The selected Linux
x86-64 g++ profile disables assembly, fast/unsafe math, contraction and native ISA
selection. Guards/branches/coordinate mapping/error checks remain at their original
owner; provider and first/second-order regions retain supported interpreted evaluation.
Stock repeated-scalar batch loops do not establish SIMD or a useful cross-callback
lane-packing consumer, so no SIMD/JIT/batched production route is selected.

ADR-0191 and blueprint revision 177 preceded production adoption under accepted RC03.
Evaluator selection participates in new artifact/view frame identities without changing
historical frame meanings; exact scalar preparation remains shareable. Study request
and operation version 7 and definition version 11 carry the typed required selector
without an omission-default compatibility route. Scientific replay and lazy study variants version 2 cover their directly embedded preparation; unchanged outer metadata envelopes retain nested-version ownership. Compiler work is cancellation-owned,
source/output overlap admitted, stderr bounded, and owned descendants drained before
release on every exit. Shared executable/template lifetime is separate from admitted
private worker scratch; no global executable cache or shared mutable evaluation lock
is introduced. Independent review exposed and corrected failed-driver descendant
release and post-effect allocation checks before acceptance. Targeted qualification
and any remaining limitations are recorded in the Outcome; no elapsed speedup is claimed.

E5 hands targeted migration and numerical/refusal evidence to 41f; whole-series
acceptance, full hygiene and representative assembled journeys remain there.


## Outcome — 2026-10-16

**Implemented:** E0/E1 retain only complete compatible residual-curvature products at
the existing fit point. E2 selects achieved-demand replay; E3 adopts owner-local
symbolic sample preparation and retains honest numeric/full-jet demands; E4 adds the
explicit supported scalar compiler selection. The inquiry decisions above own exact
versions, alternatives and limits. Worker private evaluators/frames now destroy before
shared templates, directories and accounting owners. Enduring contracts are at blueprint
§13.6/§14.5/§19.4/§25 and proposed ADR-0191. The default interpreter and existing
scientific/FP guarantees remain.

**Tested — targeted scope, zero-failure baseline, Linux, 2026-10-15–16:** root commands
used `PSE_TEST_RESOURCE_STATE=/tmp/pse41de-resources.99IJpf`, serial Nextest threads and
`scripts/pse-env`. Runtime feature selection was
`pse-runtime/solver-diffsol,pse-runtime/solver-idas,pse-runtime/solver-ipopt,pse-runtime/solver-kinsol,pse-runtime/solver-pounce,pse-math/compiled-evaluator`,
with linked `solver,klu` setup and explicit runtime `force-validate`.

| Command and scope | Actual result |
| --- | --- |
| `just check-native-package pse-runtime <runtime-features-above> solver,klu` | All targets compiled. One test-only observer warning was corrected by matching its actual consumer feature gates. Final source compile/regeneration status belongs to the shared closeout below. |
| `just unit-native-capability-package pse-runtime <runtime-features-above> solver,klu <selection-below> --ignore-default-filter` | **19 passed, 2 failed**, 842 excluded; Nextest `38e359d6-0236-4712-8622-6522759e3953`. Both failures were new worker-count observation controls, repaired and rerun below. Exact-curvature/outer-weight, failed retry/cancellation, steady implicit closure, Gauss–Newton, coherent demand, original-rank, study codec and owner reset controls passed. |
| Same runtime command, selection `test(shooting_lane_retains_worker) \| test(horizon_reuses_prepared_view) \| test(common_modeling_runs_park)` | **3 passed, 0 failed**, 860 excluded; Nextest `03bd923d-6553-4ac1-8e8a-a57a468be195`. Two repaired controls plus the shared actual-run consumer control; this is not three previously unexercised cases. |
| `just unit-package pse-math 'test(native_scalar)' --features pse-math/compiled-evaluator --ignore-default-filter` | **8 passed, 0 failed**, 324 excluded; Nextest `5ac109f7-d7ae-4bf2-b343-f6da244bf310`. Real provider-free production blocks, independent builtin arithmetic, no-FMA witness, coordinates, interpreted derivative regions, guards/nonfinite typed failures, shared code/private scratch, bounded output/logs, failed-driver drain and cancellation-at-return. One unnecessary qualification warning was removed. |
| Combined math/native command in [41d's Outcome](41d-native-storage-and-workflow-worker-reuse.md#outcome--2026-10-16) | **12 passed, 0 failed**, including the same eight evaluator controls on the corrected final teardown/source, with no compile warning. Four other cases are native layout controls. |
| `just unit-package pse-compiler 'test(directional_requests_append_distinct_keys_and_build_one_axis_with_full_formal_support)' --features pse-math/compiled-evaluator --ignore-default-filter` | **1 passed, 0 failed**, 260 excluded; Nextest `73d71d3b-aac6-480a-b50d-cb1128bbe2e5`. Evaluator choice changes new artifact/view keys while directional output, complete formal support and unchanged scalar sharing remain. |

The runtime selection was
`test(steady_response_solves_the_compiled_implicit_closure) | test(compiled_weighted_loss_gradient_and_exact_hessian) | test(gauss_newton_hessian_matches_jtwj) | test(exact_transient_fit_hessian_satisfies_production_action_basis) | test(transient_fit_demand_cache_and_coherent_upgrade) | test(prepared_fitting_relationships_preserve_mixed_gradients_and_rank_upgrade) | test(shooting_lane_retains_worker) | test(shooting_demand_upgrade) | test(shooting_order_policy) | test(horizon_reuses_prepared_view) | test(independent_completed_lane) | test(independent_failed_or_cancelled_lane) | test(common_modeling_runs_park) | test(study_request_codec_unit) | test(study_definition_historical) | test(study_operation_unit)`.
Do not sum overlapping reruns or transfer these results to unselected scientific scope.

**Interface-checked:** independent source review checked complete publication/invalidation,
worker affinity, actual consumer migration, compiled FP/ISA/guard contracts and native
resource lifetime. Standalone exact-library probes remain inquiry evidence; product
acceptance uses the controls above. No convergence superiority, timing speedup or
whole-series qualification is established.

### A mistake made and corrected

The initial compiled owner marked failed compiler drivers joined without draining their
children and checked module size after export/compiler effects. Independent review
caught both before acceptance. The corrected owner keeps its leader waitable until
owned descendants stop, joins stderr capture, preflights supported source channels and
bounds both compiler output phases before effects. It borrows already source-admitted
export inspection instead of cloning it before preflight. A final readback corrected
Rust field destruction order so private native state ends before its shared owners.
The strengthened failed-driver/cancellation, allocation and production ownership controls
passed. No environment/provenance proof or exact RSS claim was added.

### Deliberate deviations and remaining boundary

No general checkpoint frontier, narrowed opaque second-jet supplier, SIMD/JIT/batched
route or new default is selected. Each inquiry has an explicit supported exit above.
Only qualified Value arithmetic compiles; derivatives/providers retain their declared
interpreted region. Seven transitive lockfile additions accompany Symbolica's optional
feature, with no direct pin/family move or wholesale resolution. New frames and required
study/replay/variant versions carry changed profile meaning without a compatibility path. Final generated-surface inspection caught the two additional direct preparation owners; both move to version 2, without cascading unrelated envelope bumps. E5's final
regeneration/Python/documentation handoff remains in progress until the shared closeout
below is settled. Full hygiene and assembled acceptance remain at 41f.


### Shared closeout checkpoint — 2026-10-16

The selected numerical/reuse/evaluator controls above passed. `just codegen` completed
all five target groups before the final direct preparation-owner version review.
Scientific replay and lazy variants now use version 2 in source, with new codec/export
assertions. Their final five-test runtime selection failed during LLVM output with
ENOSPC before any test ran; those new assertions are **not_run**. Generated replay/variant
versions still need the next zero-argument `just codegen` publication. The filesystem
has less than 1 MiB free after all task worktrees/build outputs were integrated and
removed. No build is running. The maintainer has been asked to supply space outside
active build/evidence. No cache, database, evidence or unrelated allocation was deleted.

E4/E5 and D6 remain in progress for that final handoff; full 41f qualification remains
separately scoped. Once space is available, finish the five codec controls, regenerate,
record current generated/Python boundaries, run turn-end, and close these packet owners.
