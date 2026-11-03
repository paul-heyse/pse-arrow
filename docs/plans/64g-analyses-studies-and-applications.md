---
title: "Plan64g: Analyses, studies and applications"
status: in-progress
date: 2026-11-02
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [current-work, design, validation]
review_sources: [docs/design_review/reviews/design_review_process_simulator_target_2026-11-02.md]
adrs: []
---

# Analyses, studies and applications

## Proto integration consumer obligations

[64j](64j-engineering-services-and-clients.md) delivers the local engineering service/client
over shared typed operation handlers; [64i](64i-engineering-contracts-and-validation.md) owns
its generated contracts, constraints and producer selection. G01 remains the scientific product
permission/applicability owner, and G08 the atomic capture/witness/protection/receipt owner.
Move affected inference, uncertainty, control and assessment consumers with their exact scope,
original operation identity, failed siblings and unavailable products. Progress is observational;
disconnect, timeout and cancel request are not native settlement or permission to reuse science.
J's persisted journeys consume G08's final versioned-drain and capture closeout rather than a
parallel capture path. G's existing evidence and pending failure fixes retain their own status.

## Purpose and foundations

**Proposed:** realize P6's engineering analyses over the same authored physical definitions and exact scientific revisions used by simulation. The [coordinator](64-process-simulator-target.md) owns the combined target, F01–F06 disposition and rule decisions; this companion owns G01–G08. The current checkpoint records execution authorization and progress. The source baseline is `1acd8c83ca8f9e6f6e9b5e9c9cbf98e299d574b8`; Core 3.4, efficiency heuristics 1.0, ProcessSimulator 1.5 and test-assessment 1.2 apply.

**Interface-checked:** `workflow/fitting.rs` and its `experiments`, `covariance`, `profile` and `residual_policy` modules already separate fitting preparation, experiments and derived results. `workflow/uncertainty.rs`, `horizon.rs`, `controls.rs` and `study.rs` provide existing consumers to extend. `completion.rs` distinguishes aggregate and step scientific permissions but its product lookup selects by derived quantity, while `derive_permission` consumes aggregate permission. This is a concrete boundary for F03, not evidence that every current fitting result is wrong.

Use [the review](../design_review/reviews/design_review_process_simulator_target_2026-11-02.md), the [application delivery contract](../process-simulator-design/delivery/implementation-sequence.md#p6--complete-engineering-analysis-and-application-workflows) and [answer-kind contracts](../process-simulator-design/simulation/answer-kind-contracts.md). Existing operations remain the implementation owners; public profile names do not establish supported combinations. P6 can consume accepted steady models before every P4/P5 family is delivered.

## Target contracts and integration

An analysis request selects its physical model/case, active statistical or optimization interpretation, requested products and required evidence. Admission returns the actual supported route or an attributable refusal. Datasets can supply a feed or validation comparison without becoming an inference task. A stored distribution or estimated parameter does not become active uncertainty merely because it is present.

Extend completion/product permission for the actually exposed consumers. The product's identity includes its originating step/member/reference point and relevant coverage, quantity and mathematical assumptions. Same-quantity products at different points must not exchange permission. Preserve aggregate coverage and failed siblings, while retaining independently qualified step products. Diagnostic visibility remains separate from permission to reuse a numerical result scientifically.

Structural Analyze operations can produce valid evidence with zero numerical members. Their qualification follows the inspected subject and analysis operation; a physical-state permission predicate is inapplicable. Point-based sensitivities, curvature and covariance retain their actual point permission and local regularity assumptions. This implements F03/F06 together rather than correcting containment and reintroducing the phantom-state prerequisite through permission.

Shared parameters remain one estimand across experiments; nuisance quantities and initial states are experiment-scoped when justified. Observation identity includes subject, units/basis, sampling or averaging interval, time alignment, missingness and noise/dependence interpretation. A small residual does not prove identifiability. Likelihood, posterior approximation and local covariance are different evidence products.

Uncertainty execution retains the selected probability measure, joint dependence, realized inputs and failed-case policy. Dropping failed samples changes the population unless explicitly justified. Comparing deterministic outputs for the same realized inputs differs from comparing estimators or distributions. Seeds identify a selected realization only under the recorded generator/implementation contract; do not promise cross-version bit regeneration.

Physical operating steps and computational Study occurrences stay distinct. Horizon control consumes the actual dynamics/event capabilities of its selected plant, retains state estimate and horizon provenance, and states what happens on infeasibility or interruption. Multiperiod coupling, prices and commitment are authored study/decision contracts, not substitutes for time integration. Released Caprese NMPC behavior supplies a comparison; MHE remains an explicitly identified native extension rather than fabricated IDAES 2.13 equivalence.

Surrogate artifact import/evaluation, optimization embedding and trainer reproduction are separate capabilities. Preserve scaling, training domain, model discrepancy, derivative order and artifact identity. Smooth and ReLU embeddings demand different mathematical routes. HiGHS is a candidate for fitting linear/MILP subproblems; SCIP algebraic nonlinear constraints are a candidate for factorable nonlinear/discrete problems. Neither an opaque provider nor a library name establishes global capability. Carry the [review's concrete mechanisms and limits](../design_review/reviews/design_review_process_simulator_target_2026-11-02.md#8-library-fit-and-alternatives) into a selected design before implementation; candidates are not dependency adoption.

Engineering assessments retain physical contributor, boundary, currency/base period, utilization, allocation and factor provenance. Costing, environmental accounting and utility targeting are not conserved quantities merely because they aggregate. Reuse research/publication owners for calibrated coefficients and data; do not create a second result or life-cycle database.

## Package state

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| G01 | in-progress | 64a task/outcome contracts; C05 shared product contract; selected exposed products | P6 demands and route decisions feed the shared owner; integration consumes implemented scoped permission without quantity-only aliasing |
| G02 | in-progress | G01; ordinary experimental inputs and accepted models from 64b/64d | Joint inference/reconciliation retains shared parameters, observation meaning, identifiability limits and qualified uncertainty |
| G03 | planned | G01; accepted deterministic operation; selected distributions | Uncertainty execution retains measure/dependence, actual samples, failed-case treatment and statistically appropriate comparisons |
| G04 | planned | Selected reference applications from 64h; G01; demanded scientific capability | Surrogate/discrete realization decision settled, selected artifact/training/embedding paths integrated and unsupported demands refused |
| G05 | planned | Selected dynamic route from 64e or accepted steady period model; G01 | Control/horizon and multiperiod/grid workflows retain physical-time/coupling meaning and intentional failure behavior |
| G06 | planned | G01; selected physical results and assessment inputs | Cost/environment/utility products and useful stream/result/visualization views preserve identities, assumptions, provenance and permission |
| G07 | planned | Selected G02–G06 integrations and their actual consumers | Joined workflow acceptance, consumer migration and enduring contract publication completed within declared scope |
| G08 | in-progress | 64a A04 publication-witness target; exact selector/writer closure | Atomic capture/protections/receipt replaces pending global gate, preserving future-member completeness, unrelated liveness and recovery |

## Investigations and execution route

### Current checkpoint — 2026-11-03

All G scope, including G08, is authorized with D/E/F and required enabling slices. G01 consumes
64c's implemented scoped product-permission owner instead of recreating it from this document's
historical baseline description. Remaining P6 consumers and finite configurations are being
mapped. Proposed ADR-0230 and blueprint344 record G08's atomic witness target before production
change. The dedicated pinned-engine writable-module probe passed actual multi-write rollback
under typed error, module timeout and caller timeout with terminal drain before ownership release.
The selected implementation is therefore the typed SQL-only whole-operation export. Source
migration now joins complete selector/fact writers, protection predicates in both commit orders,
exact-request receipt replay and the single client call; full production capture, race and
10,000-member acceptance remain in progress. No new G completion is claimed.

### G01 — Product scope before additional consumers

Inventory P6 non-fit products in `local_analysis.rs`, `result_inspection.rs` and fitting/results. Feed their point/step/coverage demands to C05, the shared completion/product-contract design and migration owner. Demand discovery can proceed before C05 implementation; P6 consumer integration requires its implemented contract or a coordinated extension at that same owner, including generated declarations and historical readback implications. G01 owns P6 route selection and consumer migration, not a second permission design. Use existing immutable result semantics; do not rewrite an earlier invocation to claim a later product existed.

### G02/G03 — Select scientific/statistical contracts

Begin with shared-parameter exchanger fitting and the selected NRTL/PR reference experiments identified by 64h. Resolve parameter identifiability, bounds, noise model and observation correspondence before choosing covariance/profile or posterior evidence. A deliberate confounding example must remain inconclusive rather than yield invented parameter recovery. For uncertainty, choose the input measure/dependence and failure interpretation before selecting sampling or propagation machinery. Record whether comparisons concern recorded points, estimator uncertainty or distributional agreement. These decisions gate statistical qualification, not ordinary deterministic case evaluation.

### G04 — Preserve the requested surrogate and discrete behavior

For each required ALAMO/PySMO/ONNX/OMLT-style reference pipeline, identify the actual trainer, persisted format, evaluation interface, scaling, embedding and optimization claim. Determine which behavior belongs to the finite parity target and which is an intentional native alternative. Inspect the selected native backend's complete class and representation support before designing an adapter. Output an implement/reuse/refuse decision for each demanded stage with explicit prerequisites; unavailable required external trainers remain visible obligations in 64h. Do not assume standalone evaluation or global optimization from an import method.

### G05/G06 — Couple existing owners through selected applications

Use the finite reference cases to select initial NMPC, multiperiod/grid and assessment deliveries. Compare the model's actual index/layout/events and derivative support with the controller's requirements. Resolve horizon handoff, state/observation time, warm starts, period links, discrete commitments and failure policy at their semantic owners. External time advance requires state custody, interval/hold/event and restart/restore contracts where consumed; a residual callback does not advance physical time. Engineering views consume the same projections and edit contracts as 64c. Native visualization/reporting is an eventual workflow obligation; a graphical editor is not a P1 prerequisite.

## Acceptance and completion

**Proposed verification:** target owner-local tests and matched scientific cases during implementation; 64h owns assembled scope-end qualification. Exercise two same-quantity products at different steps, a failed sibling with a qualified independent product, and structural analysis with no state member. Change only a caller label and establish that permission cannot strengthen. Retain unknown/withheld products and diagnostic access.

For joint fitting, independently check observation residuals, physical closure and known shared-parameter behavior; use rank/identifiability and uncertainty evidence appropriate to the claim. Sampling acceptance checks full selected population and the declared failure treatment. For controllers and grid studies, test infeasible horizons, incomplete physical coverage and period-coupling violations, not just successful optimization. Compare assessment units/periods and surrogate outputs/domain limits independently of the adapter being tested.

Prepare immutable science once where compatible, isolate mutable attempt state, avoid repeating physical solves for readers, and bound retained trajectories/cases to actual products. Coordinate case/native pools through existing execution owners; apply cancellation/drain and durable selection corrections from their owning plans when consumed. These are qualitative execution-fit requirements; speed, memory and capacity improvements remain unmeasured until representative campaigns justify them.

Migration moves every selected producer and reader of a changed contract together, including generated schemas and exact retained readback. Delete replaced interpretation paths after consumers move and revealing tests pass. Existing Plans28/40/53/54/58h keep their status and qualification duties. G07 completes only the selected P6 population and linked obligations, not unselected industry or supplemental whole-distribution claims.

**Outcome:** Partial implementation. G01/G02/G08 remain open; selected passing
controls do not qualify the complete P6 population or atomic-capture replacement.

## G08 — Atomic capture and unrelated liveness

64a assigns the DA-063 production correction here. Replace the cross-RPC open/pending/commit protocol with one bounded transaction that registers a publication witness, evaluates complete selector membership, protects captured results/manifests and records an exact-request receipt atomically. Every membership or captured-metadata writer updates the witness in its own transaction, including insertion/removal and project/branch/research/outcome dependencies. Lease-only renewal does not; cancellation never waits for a pending-capture gate but updates the witness when it changes captured facts. Start conservatively with a global witness; narrowing it requires evidence that all selector dependencies remain covered.

Use commit-time conflict detection and bounded retry rather than durable exclusion. Idempotent replay after lost acknowledgement resolves the recorded receipt; same identity with changed request refuses. Drain/settle legacy pending captures before cutover. Remove their RPC, busy guard and obsolete recovery-incarnation path only after all writers/consumers migrate, preserving protected-read ownership and authenticated abandoned-owner cleanup.

Writable Surrealism is the preferred typed whole-operation prototype if its inherited-transaction failure probe passes and it removes procedural SQL/client choreography. First prove multi-write rollback after typed failure/timeout and actual completion before ownership release on the selected engine. Its current purity attestation must be explicitly amended through the decision route; network/module-KV side effects remain outside the selected transaction contract. If that probe fails, use the same atomic witness design in the existing authored transaction owner; do not keep two production implementations. This fallback selects an established mechanism for the same contract, not weaker capture semantics.

Discriminate concurrent future-matching inserts and selector-metadata changes from unrelated renewal/cancellation; verify either a retry or one complete protected population, lost-ACK replay and stale-owner refusal. Record actual contention/retry behavior before claiming improved throughput. H05's persisted-study claims consume implemented G08 and applicable original-owner qualification; A04 source reasoning alone does not repair DA-063.

### Stopped implementation checkpoint

**2026-11-03 — stopped by instruction.** Corrective edits are preserved, including
uncommitted changes. No further build, test or debugger work is running. Failure
baseline is zero; the following failures remain open.

**G02 — Tested, selected consumer:** the two
`scientific_equipment_joint_exchanger_fit_` controls passed in the combined native
runtime selector with `property-coolprop,solver-kinsol,solver-ipopt`, invocation
`01a1a36d-f404-7795-a7c8-24ab90cf0294`. They exercise two experiments over the actual
authored exchanger, shared-coefficient recovery, independent ODE predictions/information,
original material/energy closures and the free-coefficient/area rank-one case with
covariance and parameter-publication refusal. The overall six-test invocation was
3 passed/3 steam failures, not a full pass.

The initial fitting failures exposed dropped engineering-rule context. Commit
`2550fa665` retains exact rule/scale/caloric provenance in the existing fit numerical
policy, aliases experiment targets and refuses conflicts. Two later unnecessary test
qualifications were removed; Clippy has not verified those final edits. Broader G02
applications and affected policy/caloric regressions remain open.

**G08 — Implemented, incomplete:** schema v4, typed writable capture, publication
witness and exact-request receipts replace the normal open/pending/commit client path.
The native service now explicitly selects a cache of 100 modules and derives physical
controller capacity as twice the cache capacity times the admitted module pool size
(800 at pool four), unless a caller explicitly selects controller capacity. Explicit
positive cache/total selections are preserved; invalid/range-overflow selections refuse.
This removes the observed eight-controller default mismatch without changing guest
memory, execution-time or pool limits. Broader resource/throughput qualification is
not established.

The broader ordinary batch with `canonical-tests,canonical-maintenance,native-service`
ran **30 tests: 25 passed, 5 failed**, invocation
`01a1a371-761d-7719-bee5-450b1f3abade`. Controller-capacity refusals were absent;
legacy-owner cleanup, static migration, capacity selection and selected liveness,
receipt and input controls passed. The latest focused selector over small lost-ACK
capture and the three stack controls ran **4 tests: 1 passed, 3 failed**, invocation
`01a1a37a-f40b-74cb-9194-2930bd5abc24`, using the optimized module. Remaining failures:

| Test / discriminator | Latest evidence and next obligation |
| --- | --- |
| `capture_all_unavailable_members_retain_the_named_parent_summary` | Ordinary execution still aborts with stack overflow after claim/finalization future boxing. A debugger child passed the same captured case, but did not yield an overflow stack; its wrapper failed after `bt` on normal exit. Compare actual launch environments and capture the failing stack; do not treat the debugger run as acceptance. |
| `capture_opaque_public_ids_and_protection_tuple_keys_do_not_alias` | Ordinary execution still aborts with stack overflow. |
| `capture_and_absent_protection_retirement_are_fenced_in_both_commit_orders` | Ordinary execution still aborts with stack overflow. Both commit-order acceptance obligations remain open. |
| `capture_future_member_and_selector_metadata_commit_before_capture_force_complete_retry` | Broader batch failed during fixture initialization with an h2 body/connection error. Cause and actual future-match acceptance remain unresolved. |
| `capture_native_10000_members_preserves_complete_bounded_pages_and_receipt` | Pre-optimization run hit the strict one-second module epoch timeout, then unresolved absent receipt/drain timeout. The optimized 10,000-member rerun never executed: its pending launcher was canceled before fixture creation, exit 143. |

The preserved optimization keeps all captured arrays and bulk inserts inside one fixed
host-SQL transaction block; only the typed request and bounded receipt/refusal cross
the WASM boundary. The optimized small lost-ACK/exact-replay control passed. This does
not establish that the large capture now meets its unchanged clock/resource contract.
Owner registration before the held transaction's BEGIN corrected an independent
race-test snapshot error and its stalled-capture control passed.

**Tested, bounded migration:**
`native_service_retained_v3_old_module_upgrades_to_explicit_prepared_target` passed
1/1, invocation `01a1a369-cfac-77a3-abe8-7d439578ae02`: an unchanged authenticated
readonly artifact in a newly owned frozen-v3 fixture, restart/quiescence, migration,
exact receipt replay, module promotion and current readiness. This is not a recovered
historical deployment or a full migration population. The legacy helper's latest
fixture correction restores two original count-seal events; that final overlay needs
a rerun. Obsolete Busy refusal/classifier/retry remnants still require retirement once
replacement controls pass. G08 and DA-063 are not closed.

Clippy, hygiene, final generated comparison and further regressions/campaigns were
explicitly skipped; the
[coordinator checkpoint](64-process-simulator-target.md#d-through-g-implementation-checkpoint)
owns common closeout state and the separate ongoing plan-document work.
