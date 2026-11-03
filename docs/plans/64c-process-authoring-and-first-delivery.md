---
title: "Plan64c: Process authoring and first delivery"
status: in-progress
date: 2026-11-02
adrs: [ADR-0236, ADR-0237]
review_sources: [docs/design_review/reviews/design_review_process_simulator_target_2026-11-02.md, docs/plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md]
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [current-work, design, validation]
---

# Process authoring and first delivery

## Resulting engineering behavior

**Proposed:** one native path lets an engineer inspect selected process structure, edit an admitted definition/case, prepare and run supported operations, and inspect physical outputs or full failures. [64b](64b-native-scientific-intake.md) supplies Packet A's ordinary science; this companion supplies B/C and the native side of D, whose matched reference/closeout owner is [64h](64h-reference-coverage-and-qualification.md). The [coordinator](64-process-simulator-target.md) owns source baseline `1acd8c83c`, rule decisions and F01–F06 dispositions.

The first supported interface is a Rust domain-operation surface using existing `ModelingPackage` and workflow/result owners. A graphical application, new language binding or compulsory CLI wrapper is not needed to establish it. Typed declaration editing is the first edit representation; text documents can still be admitted as source, but editing declarations does not promise regenerated authoritative text. Preserve draft diagnostics without claiming an invalid draft can solve.

## Execution requirements from 64a

Consume [64a's operation decisions](64a-scope-and-shared-contracts.md#a03--selected-library-backed-operation-design) at these owners:

| Package | Selected mechanism and boundary |
|---|---|
| C01/C02 | Typed declaration edits and existing flow owners; Salsa derivation is separate from runtime custody/pressure. Use consumed dependencies, not whole-submission invalidation or a second cache. |
| C02 | Symbolica parameterized multi-output/function DAG and demanded Numerica realizations; faer symbolic analysis and numeric factors have distinct dependencies. Preserve solver-owned POUNCE/FERAL factors; reject aborted optimization before retention. |
| C05 | Compiler-bound incidence through POUNCE HK/DM/BTF independently of candidate values, numerical Jacobians and solver eligibility; point diagnostics keep their separate owner. |
| C03/C06 | Completion-driven finite fan-out where useful, explicit admission closure/cancellation settlement and native joins for foreign/TLS teardown. No scheduler rewrite. |
| C04/C05 | Scoped producer facts and registry-generated boundary adapters; retain Buffa/ConnectRPC and protected lineage. No quantity-only permission or caller-authoritative export label. |

C03 preserves Solve versus full Initialization report, including unsuccessful reports. C04/C05 migrate local/durable/inspection/fitting/uncertainty/portability consumers together under proposed ADR-0236. Version changed interpretations explicitly; historical missing scope cannot strengthen permission. C07 covers final initialization, ready-state and conditional ordinary free-variable acceptance without relaxing hard domains or fixed specifications.

C06 completes native journeys and the exact H02/H03 handoff, not matched-reference execution or full A–D acceptance. G08, broader analyses and the collective upgrade retain their owners.

### Recycle improvement integration — 2026-11-02

Consume the [performance and meaningful-convergence review](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md),
including its follow-up at `1e8ce99b0`. The maintainer selected all I01–I12 for staged
integration and directed implementation of plausible optimizations without a separate
benefit-justification phase. Measurements guide refinement and expose regressions; they do
not gate beginning a source-grounded correction. Investigate genuine correctness, contract
or competing-configuration questions within the implementing package. The
[coordinator](64-process-simulator-target.md#recycle-improvement-dispositions) owns each
finding's disposition; package progress lives here and in 64b.

**Interface-checked:** field fixed-point selection and `PreludePremises::select` still
materialize before tracked reuse; `declaration_inventory` still calls `delta_parent`.
The existing function-selection fix is implemented. The review's four temporary source
edits were restored: its faster observed field-prototype journeys, increased RSS and
incomplete recordings establish neither adopted implementation nor a stable speedup.
Core3.4, efficiency heuristics1.0, test-assessment1.2 and ProcessSimulator1.5 apply.

Keep first-party Rust at optimization level **2**. A 2→3 change is excluded; existing
dependency profile settings remain intact. Preserve force-validation, ordinary scientific
scope, original physical checks and unbounded host placement. Larger timeouts, host caps,
cache-cap increases and a universal enthalpy floor are not selected remedies. No additional
rule change is required by this target; actual durable-contract or architectural changes
follow the existing decision route before implementation.

## Package state

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| C01 | in-progress | A02/A03 meaning; B01 selected definitions | Selected source/declaration/instantiated process views and typed edits over existing immutable identities |
| C02 | in-progress | C01; RC03 contract slice | Label/numeric/topology edits preserve or invalidate the appropriate artifacts and match clean admitted reconstruction |
| C03 | in-progress | B ordinary-input path; admitted initialization/solve | Complete success and intended failure reports survive public native operations and restoration/readback |
| C04 | in-progress | Protected result/producer completion contract | Export selection derives exact scientific usability; affected readers/writers migrate together |
| C05 | in-progress | RC04; completion and local-analysis contracts | Exposed product applicability and exact governing scope are consistent across native/recorded consumers |
| C06 | in-progress | C01–C05 selected behavior; B delivery; C08 precision and C09 evidence slices; H02 matched reference contracts and inputs | Recycle/exchanger native journeys, efficiency refinements and independent-reference handoff retain complete scoped acceptance |
| C07 | in-progress | 64a A04 ordinary-interval disposition | Quality-owned tolerance semantics integrated into affected ordinary/conditional/initialization acceptance without relaxing hard domains or fixed specifications |
| C08 | planned | B01/B02 scientific premises as needed; B04 exact case context; existing numerical policy | Shared meaningful precision covers governing first-case targets, with attributable fallback decisions and original physical acceptance |
| C09 | planned | Existing source recording/result owners; C03–C05 report and permission contracts | Real journey capture, retained outcomes and bounded offline analysis preserve truthful coverage and proportionate diagnostic work |

## C01 — Native inspection and editing

Use `modeling.rs::with_declarations` for additions/replacements/removals against an explicit parent revision. Use selected semantic instance/member IDs rather than list ordinals or displayed labels. Return units, state/port contracts, connection occurrences, scientific selections, case roles, diagnostics and their actual inspected scope. Keep drafts, admitted declarations, selected instantiated semantics and numerical artifacts distinguishable.

Use `prepare_flow` for selected topology; do not infer solve order from material-flow direction or require a numerical square solve for a property/structure panel. Preserve parallel edges, isolated units, component identity/order independence and source-attributable incompatible-connection errors. Reuse existing source-delta/canonical publication for recorded edits. No second editable equipment graph or independent readiness state machine is introduced.

Acceptance uses a label rename without semantic identity change, an illegal state/port connection attributed to its original declaration, a reordered component set with equivalent meaning, two parallel connections and an isolate. Invalid edit admission leaves the parent revision intact. Ordinary operation consumers, not only a fixture-specific adapter, use the new surface.

### Ordered reads without edit preparation — I08

Make selected callers use existing declaration windows/pages or known semantic identities.
For legitimate complete inventory reads, factor the authenticated ordered-read owner so it
hydrates the requested rows once without constructing delta/dependency `Products` or copying
an edit parent. Keep `declaration_inventory`'s full ordering/completeness contract; do not
implement it by repeatedly calling public pages that decode the same ingress frame.
Construct editing products only at the actual edit boundary. Migrate C01 and 64b B03–B05
consumers together, including ordinary case discovery helpers.

Carry typed immutable owners and leases through read-only consumers where this removes
copies. Distinguish actual declaration/dataset copies, carried metadata and fallback decoding
before correcting their extent reservations; the generic 32× allowance is not universal.
Preserve refusal before allocation and conservative treatment of unknown extents.
Acceptance checks exact IDs/order/provenance, absent/corrupt/duplicate ingress, cancellation,
source disposal and leases, scarce-capacity reads, and a subsequent real edit/CAS/publication.
Count actual hydration/edit-product work; a short returned page alone proves no saving.

## C02 — Case binding and artifact reuse

Change feed temperature as a case value without changing physical definition or reconstructing compatible symbolic structure. Change fixed/free roles, topology, contractual state slots or scientific data through their actual admitted structural/scientific paths. Rating and design select different specifications over the same exchanger physics; releasing area and specifying outlet temperature is not a mere cosmetic value edit.

Carry exact consumed evaluator/method dependencies through existing prepared owners. A guess is an execution input, not the semantic branch requirement. Reuse starts/factors only while their own premises hold. Compare admitted semantic meaning after incremental and clean construction; equality of incidental output order or source bytes is not the claim. Read existing declared-diagnostic reuse tests as preservation evidence, not newly passing receipts.

### Selection, discovery and retained working set — I01/I02/I09/I11

Move each exact `StaticPackageSelection::field` request through the existing `Fragments`
boundary to `select_package`/`read_static_package` before materialization. Preserve the
monotonic dependency fixed point and standalone derivation of the same selection. Likewise
send the exact prelude selector and complete `PreludeBindings` to the tracked owner before
building `PreludePremises`, member maps or conditional maps. Existing Salsa derivation owns
reuse; runtime custody and pressure remain separate. No second selector/cache is introduced.

Use `CheckedPackage::occurrences_of` and actual definition/member-to-instance relationships
instead of scanning every expression against every state. Factor discovery membership and
final owned projection through the same selection traversal, allowing shared/borrowed views
to avoid repeated intermediate packages. Share immutable recipe/capture premises rather
than rebuilding equivalent state before lookup. Keep actual instance contexts, inherited
environments, forwarded sources, temporal closure, inactive aliases, negative namespace
reads, permissions and member limits. A view must not accidentally prolong a huge parent;
complete equality remains authoritative even with a stronger once-computed hash hint.

Retain cause, generation, actual bounds and bounded ingredient distribution at the existing
exclusive retention census before reset. Correct duplicated ownership/extent accounting
where demonstrated, and improve key/admission-history lookup while preserving newest-first
selection and positive/absent premises. A displayed row limit must not disguise a full
candidate scan. Preserve escaped products across eviction/rotation and existing service-body
retention. Distinguish reservations, known/unknown retained heaps and process RSS.

Address the field prototype's increased working set as part of these changes: remove
redundant intermediate/key ownership rather than compensating with larger caps. Count field
and prelude construction separately; aggregate projection counts may still grow for valid
new selections. Reuse existing exact dependency/error owners, preserving cancellation,
typed provenance/resource refusals and current receiver qualification.

### Preparation lifecycle and carried qualification — I03/I12

Preserve `CheckedSelection` through adjacent operations while its existing `carries`
source/physical/provider/registry and same-invocation premises hold. Re-establish receiving
qualification when they change; retain explicit Arrow validation and cross-invocation
checks. Trace and remove repeated qualification/cloning in the selected B/C consumers,
using existing single-flight preparation rather than a global writable fixture cache.

Reuse compatible frontiers, bodies and evaluators across value changes. Treat fixed/free
roles, tear overlays, original A/B/A restoration and design-to-rating as distinct requests
whose unchanged subproducts may survive. Numerical points, factors, starts and original
acceptance keep their own dependencies. Preserve independent waiter cancellation/deadlines
and producer drain. C03/C06 may consolidate duplicate setup/assertions only with the same
scientific claim; keep a preparation-only control, four-feed historical coverage, the
ordinary one-feed case and explicit accepted-design rating lineage.

**Planned acceptance:** extend the existing compiler reuse/physical-premise controls for
tracked/direct values and derivatives, repeated exact selectors, same definition with
different bindings, shadowing/absent members, changed imports/data, forwarded and temporal
instances, A/B/A history, eviction, cancellation and receiving-budget refusal. Compile
touched packages and run targeted controls with force-validation. Capture complete work
counts where available and truthful partiality otherwise; time and memory are separate
outcomes, not additional pre-implementation approval gates.

## C03 — Preserve full initialization outcomes

The reviewed `modeling/declared.rs::execute_declared` converted an incomplete initialization report to an input error and returned only the last result on success. The process-facing initialization operation now returns the full typed report, including stage/attempt evidence, candidate availability, original assessment and restored specification. Keep numerical failure distinct from invalid input or unavailable capability. The selected recorded route is direct native canonical execution; persisted study initialization remains explicitly unsupported until its own versioned protocol supplies the actual policy (ADR-0237).

Retain success, failure and partial-evidence meaning through the selected local and recorded route; inspect existing canonical result representation and extend its owner only for actually missing report fields. No second initializer, rollback engine or retry mechanism is permitted. Migrate each process-facing initialization consumer and its meaningful tests together; convenience solve operations may retain their narrower documented contract.

The failure control must reach the intended initialization stage and preserve the user's immutable specification. A setup error does not satisfy it. A successful multi-attempt initialization must retain earlier attempts and qualifiers, not just the final result. Independently exercise original residual/domain/bounds and closure checks; a native success code is insufficient.

C09 retains these actual reports while their source/attempt owners are alive, including
successful reports. Its diagnostic collection failure cannot replace the scientific outcome.

## C04 — Producer-owned export usability

The reviewed `connected_results.rs::pack_selection` copied caller-supplied usability, and validation reconstructed using that same supplied value. Resolve usability from the exact protected run/attempt/member/coverage permission owner when creating and validating an export selection. A caller may request content or apply stricter eligibility; it cannot strengthen unknown or unusable science.

Migrate affected pack-selection/export/readback consumers and durable representations together, using the existing decision route if a durable/public contract changes. Preserve schema, field/row/predicate coordinates, source identity, withheld/unknown states and diagnostic-only access. Retain bounded export staging and interruption/no-final-file behavior. Delete the replaced assertion path after all callers move; do not keep a compatibility entry point that accepts an authoritative caller label.

Acceptance changes only the supplied label and verifies scientific usability cannot improve. Also check missing producer permission, a qualified producer, changed protected coordinates, incomplete output coverage and interrupted output. This correction does not claim an incorrect published physical result was reproduced during review.

## C05 — Scoped products and analysis-only results

First delivery exposes structural diagnostics and the supported products actually requested by the recycle/exchanger workflows; it does not add every sensitivity/covariance family to P1. Inspect all public/recorded consumers before fixing the product identity. Extend `Completion::derive_permission`, `step_permission`, local-analysis and result-inspection responsibilities rather than introduce a service.

A derived fact identifies its actual step/member/reference point, quantity, relevant coverage and analysis assumptions. A valid point-local derivative at an unusable physical candidate remains diagnostically visible but withheld from scientific reuse. An independently accepted step is not erased by a failed sibling, and cannot upgrade that sibling or aggregate coverage. Quantity-only lookup is insufficient when the same quantity occurs at several points.

Structural Analyze success follows the inspected model and analysis evidence; it requires no fabricated physical member or accepted-state predicate. Point-derived sensitivity/covariance/stability still requires its actual point and mathematical prerequisites. Lazy products receive a scoped fact when produced; an earlier immutable invocation can truthfully retain unavailable status. 64g expands product families only with the same contract and migrated consumers.

Acceptance pairs zero-state underdetermined structural inspection with a point-derived request missing its point, plus two same-quantity products at different scopes and one failed independent sibling. Unknown capability remains unavailable; mandatory unsupported evidence refuses rather than silently downgrades.

## C06 — Assemble A–D's native side

Use the actual recycle-flash and exchanger-costing authored definitions in `packages/reference/campaign/models/`. B supplies ordinary eligible records. Preserve the ideal benzene/toluene recycle separately from BT_PR/relational-density exchanger assumptions. Execute default recycle success, its intended initialization failure/restoration and solved-area exchanger design. Include rating with its explicit fixed/free correspondence and one compatible closure substitution without rewriting side balances; changed science has a separate claim.

Retain exchanger unknown-area/pressure-extrapolation permissions and costing currency/unit conventions; matched arithmetic is not a qualified equipment purchase price. Inspect semantic physical outputs and exact retained readback through explicit local/recorded routes. H's reference construction can overlap after scientific correspondence settles; integrated D closeout needs the completed native side.

**Proposed verification:** revealing owner tests and affected compile checks during implementation; assembled native/reference journeys only when their selected scope is integrated. Preserve independent analytic balance expectations and wrong/missing-output comparator controls. A–D completion is bounded first-delivery acceptance, not P0–P7 equivalence. The [native/reference correspondence](../../packages/reference/ordinary-delivery/sources.md) owns the exact ordinary input/configuration/output handoff; H02/H03 independently constructs and executes its matched references.

### Numerical configuration and complete lifecycle — I04

Optimize evaluator realization and numerical configuration using actual attempt work,
consuming C08's meaningful requirements and C09's retained facts. Compare interpreted and
qualified compiled-scalar realization, symbolic optimization effort, exact/limited-memory
curvature and compatible linear settings where the consumed model supports them. Implement
plausible improvements directly; short comparisons choose between mutually exclusive settings
and expose preparation/evaluation tradeoffs. Keep library-owned iteration/globalization and
POUNCE/FERAL factors; no application solver or Rust opt-level increase is selected.

Retain effective settings, starts and exact original/tear/design/rating identities, native
terminal statistics, callback/evaluator demand, KINSOL inner-root work and available factor
call/parent-call/refinement/multipart detail. Opaque MUMPS internals remain unavailable.
Do not count original and transported boundaries as independent solves, reset-per-batch
demands as cumulative counters, or compiler work inside `native.worker` as native solve time.
Preserve existing explicit methods until a replacement passes the same scientific controls.

Separate build/fixture preparation, boundary/provider/admission wait, preparation, realization,
native execution, original assessment, publication/export and teardown. Trace unexplained
launch waits through existing host owners; reuse the captured executable/profile within a
campaign and account for artifact growth. Consume Plans40/53/54/58 rather than create a
runner or automatic cleanup mechanism. Scheduling changes require actual harmful-overlap
evidence and preserve independent work's host capacity.

At functional scope end, exercise default recycle success, intended rejection/restoration,
exchanger design/rating, arithmetic closure and exact publication/reopen/readback. Recover
the historical exchanger's exact inconclusive conformance check through existing report
export and repair any scope-owned failure; aggregate status cannot stand in for its cause.
Record original and refined conditions, elapsed/RSS, diagnostic perturbation and all failed
alternatives. No numeric speedup threshold is invented. Integrated native/component/solver
campaigns, performance comparisons and scope-end hygiene/manual checks run after assembly
under AGENTS.md; H02/H03 and wider reference/deployment qualification remain separate.

## C07 — Shared ordinary-interval acceptance

64a's source assessment assigns the DA-062 correction here. Reuse `pse-backend-native::quality` interval/tolerance computation in the affected initialization and conditional/causal consumers, retaining original source IDs and physical-coordinate assessment. Trace each caller to distinguish ordinary free-variable tolerance from hard domain/sign guards and fixed specifications before replacing its comparison. No global relaxation or interval-arithmetic subsystem is selected.

Exercise a finite representable deviation inside ordinary tolerance, one outside, and strict-domain/fixed-value countercontrols across the affected routes. Migrate all selected consumers and delete their replaced ordinary comparisons; keep the stronger checks where their meaning requires them. C03/C06 and E/G/H acceptance consume the correction only for routes implicated by this discrepancy. Passing this slice does not independently close every DA-062 manifestation. This is a scheduled production package, not work executed by 64a.

## C08 — Shared engineering precision — I05

Complete `NumericalPolicy`/`ResolvedNumericalPolicy` and the existing shared authored
engineering rules, consuming 64b's exact scientific/flow context. Inventory governing
variables, rows and closure targets for historical and ordinary original/tear/design/rating
requests with semantic instance IDs, full quantity/unit, allowance source, characteristic,
fallback, coordinate scale, physical budget and resulting native controls. Cover canonical
fallbacks beyond the printed enthalpy rows; target counts include roles other than constraints.

Resolve useful caloric differences against independently admitted heat-capacity/thermal
resolution and consuming energy/flow requirements, with relevant output/decision goals.
Freeze meaningful characteristics independently of iterates and arbitrary absolute enthalpy
datums. The review's 6–9 microkelvin local equivalence exposes a gap, but its flow calculation
rules out applying an 11–17 J/mol allowance globally. Use actual consumers and coupled
physical acceptance to choose requirements at the existing owner, not convenient constants
or per-test solver knobs. B01/B02 closes genuine missing source context; B04 supplies exact
formulation, validity, units/basis, flow premises and provenance. Extrapolation remains explicit.

Retain per-row physical-budget transport and independent stationarity/complementarity.
Correct unnecessarily coupled coordinate stopping at its existing owner if the resolved
inventory reveals it, preserving original bound/residual assessment. Require meaningful
context for governing ordinary first-case engineering targets; classify every remaining
fallback by role and reason. Enable the existing strict engineering-context mode for the
selected cases once their required coverage is complete; do not change the global default.
Output/decision goals apply where the selected analysis demands them, without compulsory
paired certification for every ordinary solve. Hard domains, fixed values, derivative steps,
smoothing and exact identities keep their separate meanings. C07 remains the DA-062 interval
comparison correction; this package changes the context it consumes, not its ownership.

**Planned acceptance:** independently computed caloric/energy-flow cases at different flow
scales; unit and affine-datum invariance; changed characteristic/goal invalidation; missing
required context refusal; heterogeneous physical budgets and native transport; intended
out-of-budget and strict-domain/fixed-value failures. C06 then checks original balances,
bounds, domain and applicability under the new shared policy. Re-admit/recompile changed
authored defaults and publish a new scientific revision where needed; retained old revisions
keep their original interpretation and numerical layout identity.

## C09 — Journey diagnostics and retained evidence — I06/I07/I10

Integrate opt-in `source_recording_from_environment` at actual local and selected recorded
journey roots before Runtime/source/workspace construction. Propagate the context over the
whole async future with `observation::scope`; an enter guard must not cross await. Retain
successful and unsuccessful original reports/settings/physical checks before disposal.
`execute_declared` outcomes and `Arc<RunResult>` differ: preserve typed direct-outcome facts
at their existing owner, while full RunResult handoff uses the supported
`prepare_operation -> start -> RunHandle` route. Name experiments separately when their
execution routes differ; do not fabricate a RunResult to satisfy the handoff API.

Keep original science and diagnostic admission/finalization outcomes independent. Reconcile
actual activation, issued/observed/queued/acknowledged records, loss, valid prefix, writer
settlement and end reason. Add distinct oversize/full/closed rejection counts at the existing
collector; inspect upstream compiler dropped updates separately. Complete applies to the
selected acknowledged interval/families, not arbitrary later work. Recent hub history,
20-second live samples, longer source recording and bounded explanations are separate scopes.
Preserve raw records and use cursor pages/selected operation-attempt windows so handoff
output can be read within frozen aggregate admission; do not copy a complete large archive
into one oversized frozen context or equate the explanation cap with recording completeness.

Use bounded reusable JSON buffers, batch complete-frame flushes with truthful published
readable prefixes, and channel wakeups instead of idle polling. Preserve nonblocking native
callbacks, finite queues, deadline/stop/drain and final successful flush. Retain/export once
and reuse existing typed tables/analysis indexes for bounded questions. Improved throughput
must retain explicit refusal/partiality under overload rather than block science or promise
an unbounded lossless stream. Increasing the queue alone did not fix the reviewed loss.

For managed children, preserve the separate one-time group grant, per-attempt share/slots,
child references and refusals, original monotonic expiry and scientific drain. A local ACK
cannot establish managed coverage. Correlate process-instance operation, producer,
generation, source/family sequence and field ordinal with actual run/attempt/step identities;
preserve POUNCE parent-call and multipart joins. Reject conflicting duplicates and distinguish
batch resets, overlapping boundaries and unavailable observations.

**Planned acceptance:** a successful preparation journey and initialized journey with actual
selected activation, retained original outcome and offline-readable evidence; intended
scientific failure with retained attempts; deterministic queue/metadata/output exhaustion,
stop/expiry, partial write, readable complete prefixes and records beyond explanation limits;
managed child admission/refusal/expiry and cancellation/drain. Require complete collection
for the representative bounded selections declared complete, and truthful incomplete/refused
results for curtailed controls. Measure ordinary observation, selected detail and optional
thread-CPU conditions separately to refine overhead without a prerequisite justification phase.
Migrate affected C03–C06 readers/writers together; version actual changed durable/public
representations through the existing registry/decision route. Diagnostic visibility never
strengthens scientific permission.

## Remaining execution order

Start narrow C09 activation/loss attribution and C02 pre-reset observations alongside field
and prelude corrections. C01/B inventory work and B/C08 scientific context can proceed
independently. Continue C02 miss-side/lifecycle/qualification optimizations as integrated
changes, using observed retained ownership to refine them rather than waiting for a broad
benefit study. C06 numerical comparisons consume the relevant C08 policy and C09 report
slices; structural corrections do not wait for all diagnostic features. Finish functional
scope before the assembled C06 qualification. Preserve package state and existing receipts;
new planning scope does not reopen or erase previously verified narrower behavior.

## Current checkpoint — 2026-11-02

Full initialization outcomes, actual parent projection coordinates, protected scoped product
permission, compiler-bound structural case-variable inspection and ordinary-bound acceptance
are implemented. Their selected authoring/initialization/permission/budget controls pass, as
do the workspace and selected native-provider all-target compile checks. Ordinary design,
arithmetic closure, positive recycle and the canonical source-to-consuming-case/readback
journey pass their focused controls. Rating now passes an explicit same-case accepted-design
to original-rating composition through the public Rust operations, transferring complete
typed primal inputs and requiring fresh final permission. Generic cold starts under Ipopt
and POUNCE remain unqualified; no restored initializer policy or solver factors are invented.
Its long preparation exposed duplicate static function-source materialization before the
existing tracked selector cache. C02 now sends the exact selection request to that owner
before materialization. Focused controls retain source/physical metadata and executable
values/derivatives while raw projection counts stay equal for one versus eight calls.
Ordinary cases explicitly select Ipopt; historical recycle preservation controls explicitly
select POUNCE and declare that feature prerequisite. These are qualified method selections,
not claims that all linked algorithms converge on every case. The coordinated 36-control
native rerun now passes with both declared methods, and all six public native-diagnostics
integration controls pass. Scope-end qualification remains open. The requested
[performance and meaningful-convergence review](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md)
records remaining eager field projection, confirmed canonical enthalpy fallback and restored
disposable experiments. I01–I12 are now scheduled through the coordinator and the packages
above, including new planned C08/C09; their production changes and benefit qualification
remain outstanding. No shared precision default was changed by the review or this plan revision.
The overheated
stage retains an actual Ipopt numerical stop and the original50K connection contradiction,
not a mathematical infeasibility certificate. Its case-local expectation and independent
contradiction control preserve that distinction without changing central classification.
The exact overheated contradiction and historical recycle preservation controls pass.
The Perry benzene liquid-Cp allowance preserves its published envelope and makes the
368K recycle use explicit extrapolation. The selected native delivery and affected
preservation controls pass; scope-end qualification remains open, and H02/H03 reference
execution remains separate.
