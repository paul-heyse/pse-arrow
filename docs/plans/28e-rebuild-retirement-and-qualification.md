---
title: Rebuild, retirement and integrated qualification
status: in-progress
date: 2026-10-05
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md, docs/design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md, docs/design_review/reviews/design_review_plan-28-completion_2026-10-06.md, docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md, docs/design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md, docs/design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md, docs/design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md, docs/design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md]
scenario_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#representative-journeys]
---

# 28e: Rebuild, retirement and integrated qualification

## Responsibility and adoption decision

This companion of [Plan 28](28-surrealdb-unified-substrate.md) owns clean regeneration,
cross-package retirement, build/codegen efficiency and the single assembled qualification
campaign for the pivot. Local mechanisms and targeted checks remain owned by A/B/C/D.
The coordinator owns the common target and US01–US05/EF01–EF08 dispositions. This document
owns E progress and final campaign evidence.

On 2026-10-05 the maintainer selected a **clean rebuild and hard pivot**. Replace the review's
preserving-import transition with regenerated controlled authored inputs, derived products
and scientific results. No legacy import, compatibility reader, dual-write period or old
PostgreSQL/Delta qualification prerequisite is in scope. Retire a replaced production path
with its last migrated caller and passing targeted controls; do not retain it as a comparison
oracle. A package can temporarily leave the unfinished application unavailable while building
the single target. It must not fill the gap with a second production implementation.

Controlled scientific fixtures, independent reference observations and clean-room IDAES
behavioral comparisons keep their scientific purposes. Regenerate storage-specific artifacts
through their owners rather than interpreting historical database bytes under a new codec.
Destructive removal of an actual application state directory is not an incidental shell
cleanup: execution identifies its target and requires that it be within the authorized rebuild
scope. The current plan-authoring turn does not delete data or mutate production.

## Qualification handoff from existing plans

Plan 27 has implemented its A/B/C functional scope and immediate deletions, with a focused
77-control native accuracy selection and generator/Python transport evidence recorded at its
[checkpoint](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/27-contextual-engineering-accuracy.md#current-checkpoint).
[25k](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/25k-integrated-qualification-and-closure.md#current-execution-checkpoint) still has
incomplete K3/K4/K5 assembled work. Those facts are not completion evidence for this pivot.

Transfer the remaining applicable scientific campaign obligations to E3/E4/E5 for execution
against the rebuilt target. Keep 25k's CA finding dispositions and prior receipts at their
owner, linking the new evidence rather than duplicating it. Preserve scientific requirements:
physical closure, contextual accuracy, derivative/response validity, branch/rank refusals,
partiality, event endpoints, dynamics, fitting, continuation and native failure truth. Replace
their old PG/Delta/SQL storage journeys with A/C/D's canonical selectors, codecs and fences.
Do not finish the old storage stack's campaign before starting this pivot, or mark K3/K4/K5
passed merely because their requirements have moved.

The 14 selected native measurements and three additional scalar controls described in 25k
are prepared, not measured. E4 retains their applicable scientific questions and records the
actual target conditions. Historical Plan 22/23 qualification remains historical; final
enduring qualification documentation is amended only for demonstrated scope through the
decision/design route. E5 reconciles findings at their existing owners.

## Retirement and build foundations

Repurpose `pse-operations`; retire the generated `pse-operations-queries` and `pse-catalog`
crates after their actual consumers move. Remove PostgreSQL pool/client/COPY/DDL/query
generation, mandatory Delta result publication, cross-store settlement, publication windows
and member-prefix reconciliation. A3/C4 retain the necessary target read protection,
retention, cancellation and recovery obligations.

The SQL convenience retirement includes `Runtime.query`, native `query_session`,
`TableReader::query` and `ModelingKnowledge.query`, plus their Python wrappers, examples and
tests. Preserve their necessary user operations with D's native selectors/queries and Arrow
export. The PyArrow C-stream adapter and generated scientific Arrow contracts still serve
typed result and model consumers. Deleting a SQL method does not by itself remove DataFusion.

`pse-engine` owns DataFusion execution/planning and `pse-relations` uses DataFusion expression
and optimizer APIs. For each remaining mechanism, retain it only when it supplies a necessary
operation, or replace it with an existing suitable native capability within that operation's
owner. Do not rebuild a bespoke relational optimizer merely to remove a dependency. Remove
unconsumed DataFusion/Delta/object-store/cloud features and crates from manifests, codegen and
recipes once the caller inventory proves replacement. E2 records the final residual consumer
decision; it may legitimately leave DataFusion for a necessary capability, with no SQL API.
Preserve the single resolved Arrow/Parquet/object_store/DataFusion type universe for families
that remain.

Narrow `.config/hakari.toml` membership to actual consumers after those migrations. Do not
attach the remote database client or embedded engine to all semantic roots. Regenerate the
workspace hack through its owner. One feature set for real shared dependencies remains;
force-validation stays opt-in and explicit in correctness tests.

Independently fix `bench-builds` capability setup and remaining generation. Select the
measurement target before preparing its native environment; a relations/compiler/default
build must not initialize KLU, root isolation or unrelated Uno/PETSc discovery. Native
qualification still prepares its required capabilities. Hashing or discovery that is needed
must not execute unnecessarily under a shared lock or trigger nested Cargo before a usable
cache lookup.

Retire obsolete generated outputs at their generator. Remaining `write_tree`, Python stub
and ABI/document generation preserve unchanged bytes and mtimes while deleting stale outputs.
Use `just codegen` as part of the change; never edit protected generated files. Changed
declaration generation must still update its real consumers. Build-key changes are owned by
B3, not duplicated here.

## Efficiency extension integration

The [repository-wide extension](28-surrealdb-unified-substrate.md#repository-wide-efficiency-extension)
adds [28f](28f-shared-numerical-preparation.md), [28g](28g-bulk-data-operations.md) and
[28h](28h-native-setup-and-artifact-identity.md). N4/T5/L4 reconcile scientific/data/tooling
consumers after their adopted shared mechanisms and package-local deletion are implemented.
E2 remains integration owner for removed dependencies/recipes/generated outputs and actual
deployment adoption; it does not duplicate N/T/L package status or defer their necessary cleanup.

Preserve current qualification evidence at its original scope. E3 stays parked until the
existing A–E obligations and expanded functional scope are complete. The 2026-10-07 review,
source inventory and authored plans are not product acceptance. L0 records the accepted RC01
identity route before L3 changes capture/admission; fresh actual runtime/worker/imported-Python
association must establish the new role-specific contract before it supports assessment reuse.

## Remaining assembled execution

This is the sole assembled E3–E5 execution owner. F01–F05 dispositions are linked at the
[coordinator](28-surrealdb-unified-substrate.md#finding-dispositions). The dated
[completion audit](../design_review/reviews/design_review_plan-28-completion_2026-10-06.md)
is a bounded **Revise** assessment, not final E5 acceptance. The
[capability investigation](../design_review/evidence/plan-28-surrealdb-capabilities-2026-10-06/README.md)
is library evidence, not product or durability qualification.

### E1/E2 acceptance prerequisites

Before E3, integrate current A/B/C/D corrections and obtain their targeted receipts. Preserve
concurrent source, its authored workloads and contextual/global accuracy policy. The current
fixture includes `numerical-policy.pse`; quantity tests project native declarations into the
physical Arrow relation. Version-first study readmission and wire-ID assertions have changed
in source. Execute their affected controls rather than reusing the earlier failed XML.
Numerical sensitivity, restart and dynamic controls must exercise the intended base point,
constraint/scaling premises and relevant response under admitted production accuracy. A
verification solver budget is a test premise, not evidence of a derivative defect or a reason
to tune each test. Theory-order comparisons cannot substitute for production decision accuracy.

The current correction continues that production basis across ordinary sensitivity,
continuation, study, dynamic and optimizer journeys. Assertions consume the actual admitted
physical allowances, coordinate scales and action/backward-error policy. Objective stopping
criteria do not promise forward-coordinate accuracy; independent objective and original-row
oracles retain the relevant decision meaning. Tiny finite differences through a
tolerance-limited nested solve were replaced by checks of the defining implicit
derivative equations at the actual qualified inner point and material response. Outer
derivative assembly at an authored start does not qualify an outer steady solution. Exact manufactured
algebra, transport, rank conditions and deliberately small-scale refinement controls remain
separate from ordinary native accuracy. Coarse fold localization retains its actual rank
classification; it does not establish an automatic SimpleFold certificate by assumption.
Ordinary fits retain their authored integration controls and inspect the same frozen
numerical policy as production. Their material response actions use production's
weighted parameter scales and the relevant physical output allowances, with sample,
experiment and unit identity checked independently. The supplier-action budget used
by implicit reconstruction is not a fitting acceptance rule. Native sensitivity
local error controls are not endpoint-error certificates; independent analytic action
checks must retain that distinction. Identifiability thresholds retain their separate role.
Transient-fit controls also remove private KKT and integration precision. Independent
analytic First and Second actions exercise the production basis at the evaluated and
accepted candidates; differences between integrated physical predictions compose their
own physical allowances instead of borrowing a derivative-action budget.
Derivative-only forward, adjoint and Second observations now use the production job
admission path, including the prepared extent, threads, deadline and configured foreign
allowance. Their evaluators stay inside that owner until worker teardown and join;
ordinary test observations no longer grant a private checkpoint-memory allowance.
Fitting covariance reuses the native KKT owner's bound-activity classification, including
strong interior-point multipliers whose remaining slack exceeds the physical allowance.
The fit's qualified objective and original physical checks do not imply a particular
forward distance from the analytic parameter optimum. Short vessel Energy responses below
the shared design resolution retain their identity/transport checks without claiming
meaningful derivative accuracy; the Power meter and independent material-response controls
carry that comparison scope.

The earlier diagnostic assembled native and installed Python attempts were nonqualifying;
later source corrections and scoped validation are recorded in Outcome. The Python study admission lost transport
when the supervised canonical server hit its 2 GiB memory limit. Admission repeatedly
hydrated the same case and overlay demands, requesting equivalent body publication before
numerical assignments differed. Exact retained products already settle before redundant
staging in the current source. The correction retains one scoped structural basis, validates every
physical binding against the original operation closure and drops that basis on structural
changes. The original 1,000-point journey must complete under the selected workstation profile below;
source inspection does not identify the server allocator or establish a performance gain.
Cancellation settlement, staging close replay and competing identical source publishers
also exposed lifecycle defects requiring owning pipeline corrections. Source publishers
now wait for an exact immutable receipt instead of stealing an unexpired writer's
generation; explicit fenced recovery stays separate. The focused source receipt,
lease-fencing and cancellation controls pass; assembled qualification remains pending.
Source diagnosis after the PC-SAFT deadline found an automatic-driver deadline path
that could discard an already completed native stop and its assessment trace. Its correction preserves only
the current consistently assessed native time stop, without granting candidate use or
starting further work after expiry. The provider deadline alone does not identify an
Ipopt defect, nested implicit solve or dominant preparation cost; the retained execution
evidence must inform the next pipeline diagnosis.
Existing source and deployment captures retain their original scope. E3 requires positive
affected functional evidence before E4 measurements or E5 closure; the development-phase
policy below replaces the earlier blanket requirement for fresh deployment captures.

The maintainer confirmed local validation is sufficient; no separate CI campaign is
required. Local native runs use the `local` Nextest profile, retaining resource groups,
zero retries and the finite whole-run bound while allowing the production task to report
its own deadline. A diagnostic recycle run stopped by the earlier 360-second harness
cutoff did not exercise its declared 600-second production limit and does not establish
a native convergence failure.

Set up the selected initialized supervised state through the owning recipes. Use linked
code containing the behavior under test; ordinary compilation/install when needed to execute
edited code is separate from artifact requalification. B3's strict persisted-reuse controls
retain their actual artifact association requirements when that scope is explicitly selected.
A fixture receipt or current git HEAD cannot establish strict deployment qualification.
Reconcile manifest runner
prerequisites and obsolete recipe references at their owners; no new evidence wrapper is needed.

### Development-phase evidence and optional artifact requalification

The maintainer clarified on **2026-10-07** that dependency, compiler, environment or artifact
changes must not automatically require an artifact rebuild, producer recapture, or behavior
requalification. This policy governs the remaining E3/E4/E5 execution and supersedes earlier
checkpoint instructions that made fresh three-role capture a blanket prerequisite.

Keep prior results, tests and measurements with their original provenance and scope. A changed
fingerprint describes a different context; it does not demonstrate that a result is wrong.
Neither claim an old test exercised new code nor relabel an old capture as a current strict
artifact association. Test affected behavior when implementing a change, and repair actual
failures. Do not replay unrelated successful suites solely because a fingerprint changed.

Use the existing development assessment by default. Do not copy or freeze the Cargo environment
to retain evidence, restore an old host environment for capture eligibility, or add a nuanced
change-impact classifier at this stage. Cargo still builds stale units when executing changed
code. Exact cache/reconstruction eligibility, corruption checks and actual ABI requirements
remain execution-safety contracts; refusal of cache reuse is not invalidation of a retained
scientific outcome or a demand to requalify that outcome.

Retain producer capture/import/start association tooling and its focused positive/refusal
controls. Run the full strict artifact campaign only when the maintainer explicitly requests
it, including a future deployment/release qualification. Report that campaign's date, artifacts
and scope separately. Its absence does not block the current development campaign or plan
closure; it limits claims about current strict deployment qualification.

The strict controls have explicit selection owners: `assessment --python-profile producer`
captures actual roles, opts into Python association with `--producer-deployment`, and selects
only the ignored native association control with an exact filter. Ordinary Rust/Python selection
does not require strict captures and retains the finite producer-fixture scientific controls.
Selection inventories distinguish both routes; no ordinary result substitutes for strict
association evidence.

Current campaign sequencing keeps reference conformance and the full native/Python store-heavy
suites separate. The overlapped native attempt exposed typed canonical initialization conflicts
and was interrupted; it supplies no complete native acceptance. First-order implicit scratch
now follows its compiled capability under unchanged entry/byte limits. Affected static
prerequisites are complete, while original reference fixtures, both full native suites, applicable
E4 measurements and final E5 acceptance retain their obligations. Build locality instrumentation
includes unrelated snapshot-only documentation edits. The initial launch refused insufficient
space before taking samples; sufficient storage is now available under the unchanged 50 GiB
launch floor. Measurements remain queued behind functional qualification.

### Enhancement acceptance and affected diagnostics

The [remaining-design enhancement review](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md)
adds C5, B5, N5–N7, T6 and L5/L6 to existing functional scope. The coordinator alone holds its
review-qualified dispositions; this owner supplies diagnostics, composed evidence and measurements.
Earlier targeted N/T/L passes are foundations, not acceptance of these new corrections.

E1/E2 first make the affected route executable: L6 repairs the immutable-receipt/Cargo output
boundary; ordinary test-module placement defects remain owning static corrections. Obtain terminal
diagnostics for the actual failed Python selections before another complete campaign. Use the
existing selected Python/native recipes and real premises; do not infer a numerical cause or
dominant phase from failure markers or elapsed time. No broad recapture, CI or provider rebuild
is needed solely to repeat that diagnosis. Existing positive entries retain their scope.

| Working prerequisite and E3 journey | Acceptance distinction |
|---|---|
| C5 default stored/durable study, including Python | Many value bindings share declared structure but every physical binding/seed role is checked; distinct occurrences and changed route/layout/source controls. Cancellation during admission/ingestion and around issued/lost-ack/acknowledged activation settles truthful durable state. Include the original thousand-point workload. |
| L5 then B5 default worker/Python restart | Demonstrate durable reconstruction rather than fresh admission without a producer receipt or producer-profile rebuild. Same supported effective deployment reuses; changed role/artifact/native/configuration, unknown context, corruption and protection/dependency changes refuse safely. Retained outcomes and fresh execution remain available. |
| L6 repeated native build | Unchanged/changed/read-only derived output controls preserve sealed source, then execute the affected native route. Compilation alone establishes no numerical pass. |
| N5 dynamic layouts | Actual integration/gradient/Hessian/shooting consumers share immutable preparation with independent concurrent state; changed scales/order/constants/support, allocation refusal, events and original physical assessment remain. |
| N6 IDAS jumps | One current sample factor with correct multipliers/directions; changed matrices and singular/nonfinite recovery; independent First/Second checks under production policy. |
| N7 transient fit demand | Gradient-first one combined forward/backward, objective-only no backward, repeated-gradient reuse and coherent value-first upgrade; actual observation/weight/sample/unit correspondence plus admitted checkpoint memory, deadlines and cancellation. |
| T6 adopted access paths or retained decision | Exact selective/empty/skewed source/result/analysis selection, protection/retirement, byte admission and final-statement failure. No small-result/one-RPC inference of indexed scaling. |

Package checks are compile/targeted evidence during implementation; static hygiene and assembled
integration follow complete functional scope under the repository rhythm. Close N4/T5/L4 with the
new actual consumers before the sole composed local E3 campaign. The default replay journey is
required once B5 is adopted; broader strict producer qualification stays explicit optional scope.
The separate guarantee cannot be exercised by a fixture receipt or by treating historical capture
as a current association.

E4 measures complete operations only after their corresponding positive functional evidence:
stored creation/dispatch, repeated dynamic preparation, sample-direction elimination, fitting demand,
default restart reconstruction, selected acquisition and repeated build/setup. Reuse existing case
owners where they exercise the behavior; add a bounded selector only for a genuinely uncovered
operation. Preserve scientific demand, production physical accuracy and admitted resource conditions.
Report removed construction/crossings separately from elapsed-time gain and forced-validation overhead.
Where no comparable baseline survives, report target-only timings, not an invented speedup.

E5 needs resolved confirmed corrections, recorded investigation outcomes and positive applicable
E3/E4 evidence. ADR adoption and document consistency cannot substitute for those claims. Incomplete
evaluation of unrelated breadth is not another defect or a new qualification campaign.

### E3 correctness, deployment and recovery

The audit's native attempt selected 2,741 tests; 2,421 completed (2,342 passed, 78 failed,
1 timed out), 2 interrupted and 320 not run. Source changed during the run. Linked Python
selected 199: 156 passed, 4 failed and 39 did not complete; producer admission, event queue,
version precedence and wire ID premises require current targeted evidence. Those runs were
interrupted and overlapped; neither qualifies the final tree or supplies comparable timings.
Old pre-pivot Plan 25/27 passes and three smoke cases do not replace this campaign.

Prerequisites include completed N4/T5/L4 migration reconciliation, settled N0/T0/L0 decisions
and working adopted mechanisms. Confirm the full capability coverage at the coordinator;
the original PE example consumers are not the expanded completion boundary. Source-staging,
analysis activation and native installation reuse retain their distinct recovery/trust units.

Use `just assessment-list` to confirm the current gate surface, then the selected assembled
`just assessment --output <output>` with its development linked Python, `just seed-conformance` for the current reference manifest, and
`just parity` scope named in the verification section. A standalone
`just native-python <output>` is for a deliberately separate selected Python campaign
or an affected rerun; do not execute the same integrated scope twice.
The narrower `--functional-scope native` selects Rust only and cannot qualify Python. Complete
`just hygiene` and the relevant `just governance`, `just docs`, `just lint-native-contracts`,
`just lint-native-data` and `just features-powerset` manual checks. Preserve force-validation,
memory caps, pinned toolchain and exact selected identities. Recipe availability and current
prerequisites govern execution; add a bounded recipe only when no existing one owns a required
journey. No full release/wheel or CI campaign is implied.

Run on one stable reconciled source state with regenerated inputs and exact deployed artifacts.
Run the comprehensive development assessment from the admitted isolated RocksDB/WebSocket
`plan28-reference` profile and its 140 GiB primary host allocation. The ordinary native gate
derives its existing exclusive-observer partition from that parent; it does not execute the
ordinary suite in the 4 GiB managed observer. Managed gates use the reference receiver.
For the 34 ordinary E4 selectors, also capture the standalone native functional prerequisite
on the wide receiver under its matching 144 GiB parent, retaining the authenticated producer
fixture selector from the comprehensive assessment. Do not transfer evidence between these
different receiving contexts. The reference profile retains one primary process
with the original 128 GiB shared pool, 16 GiB per-worker capacity, sixteen CPU lanes and
thirty-two population tickets. Its worker ceiling is 140 GiB, the server ceiling 16 GiB
and the separate observer ceiling 4 GiB, under one finite 160 GiB parent. These are
ceilings rather than physical reservations; record available memory and actual readback.
Ordinary Rust tests use the configured sixteen outer slots and scientific groups; ordinary
Python uses the current four-worker grouped runner. Managed-primary journeys use the
separate observer partition, with native work exclusively in the primary. Record actual
configured/used conditions. No uncontrolled concurrent heavy campaign should compete with it.
Exercise A/C/D's composed read/retirement/root handoff, committed lost-ack claim recovery,
worker interruption/drain, ordinary retained failure/partial/cancellation and actual WebSocket
completion/export refusal. Abrupt acknowledged-write kill/reopen and quiesced offline
backup/restore must validate exact IDs, interpretation and replay in that same supported
profile. Restored scientific content identities survive, but the restored physical creation
authority is fresh and stale creation credentials must refuse. Include scientific reconstruction
and solve/reopen, beyond the maintenance driver's schema/hash controls. Logical export/import
alone cannot establish these guarantees or physical power-loss survival.

Require zero failures and complete selected/executed accounting for the selected functional
scope. Record the actual code/artifacts exercised. Use the runner's declared input families; unrelated
concurrent prose edits remain contextual observations. A report's terminal-complete flag does not establish those conditions. If source
changes or a required scope fails, repair its owner and rerun affected qualification on the
stable final state; identify any repaired composite honestly. Do not introduce per-test accuracy
exceptions or drop workloads to manufacture acceptance. Record exclusions and actual conditions
in the Outcome rather than accumulating per-command checkpoint logs.

### E4 measurements after positive E3

Execute all 37 explicitly selected process/preparation/admission selectors, including the four growth/edit controls below; extend measurement coverage where
the confirmed new consumer scope is not exercised by them. Run the applicable selectors through
`just case-measure <new-output> --functional-from <qualified-assessment>` with explicit
`--case` selections and their current
`.config/process-cases.json` / `.config/preparation-cases.json` owners. Each selector needs
positive corresponding functional evidence, the declared workload/profile and fresh-process
samples; unsupported scope needs an explicit owner/disposition, not a silently omitted case.
Run two profile-homogeneous batches: the thirty-four ordinary selectors retain their declared
64 GiB runtime ceiling and wide receiver declaration. Ordinary qualification uses
the existing exclusive-observer route, so qualification and measurements share the
same 144 GiB enclosing parent allocation on that wide store; this is not a claim
that a 4 GiB wide observer ran the ordinary test suite. `k4-study-in-process-16`,
`k4-study-in-process-serial-16` and `k4-study-managed-durable-16` retain the exact reference
allocation. Timing's 40 GiB pool is not a substitute for either declaration. Label actual
wide-runtime/exclusive-parent and reference observations and competing load,
without a timing-profile equivalence claim.
Each batch consumes a functional report matching its actual store, allocation and native
environment. Expose the existing managed-native route as a named functional prerequisite
where needed; preserve exact terminal selection and identity checks. `--transfer` is not
a workaround for mismatching measurement inputs. Reuse E3 evidence only where it matches.
Include selected compilation/frontier and hydration growth, cold/warm and A/B/A reuse,
value/structural edits, complete study dispatch/recovery and long exact result/analysis
selection. Preserve source/deployment identity, checkout, toolchain and cache conditions.

The transferred selection is `cold-small-1`, `warm-small-1`, `cold-small-4`,
`warm-small-4`, `structure-medium-1`, `specialization-medium-1`,
`k4-study-in-process-8`, `k4-study-durable-8`, `k4-demand-orders-selected-output`,
`k4-worker-permit-lifetime`, `k4-scalar-preparation`, `k4-retention-pressure`,
`k4-retention-bypass`, `document-admission`, `k4-accuracy-ordinary-1`,
`k4-accuracy-separated-decision-1` and `k4-accuracy-refinement-1`. Add the five
pivot selectors `cold-medium-1`, `cold-large-1`, `warm-medium-1`, `warm-large-1`
and `results-chain-1`. Complete publication adds `results-publish-chain-1`,
`results-publish-source-growth-1` and `results-publish-small-chain-1`.
The last case supplies a joint coordinate/sample growth comparison; source growth
changes physical comment bytes without changing scientific declarations.
This historical selection contains 25 cases. Add `k4-study-in-process-16` for independent
nonbatch Ipopt dispatch using the exact reference pool, CPU and population settings.
The eight-point in-process selector explicitly uses the ephemeral adapter; the durable
eight-point selector retains its explicit standalone worker and continuation meaning.
Neither selector establishes managed-primary overlap; the public managed controls own
that obligation. Added enhancement selectors below retain their separate coverage.

The remaining explicit selection adds `k4-study-in-process-serial-16` as the identical
science and resource comparator with one active CPU lane, and `k4-study-managed-durable-16`
for public dispatch into the exact managed primary. The latter measures fresh receiving
reconstruction, dispatch, execution and persistence; it does not measure interrupted
restart recovery. `shooting-single-idas-cold-1` supplies the complete one-shot IDAS shooting
operation, including trajectory publication, original objective decision assessment and
result projection. Local integration controls do not establish a forward accuracy bound
for every published state; the original shooting oracle keeps that distinction.
Together with `dynamic-rebind-4-1`, `fit-gradient-cold-1` and
`fit-curved-second-cold-1`, the explicit selection contains 32 cases. Registry membership
alone does not select additional measurements or establish functional qualification.

The execution audit adds `k4-restart-local-1` to cover ordinary persisted reconstruction,
bringing the explicit selection to 33 cases. It releases the original Rust runtime before
reopening the exact canonical revision in a distinct runtime, and distinguishes actual
body reconstruction from a separate fresh-admission control. Its calling-process operation
includes reopen, preparation, the original native solve, retained history reads and teardown;
initial seeding and the fresh control are separate. The controlled immutable native benchmark
composition does not measure managed process/Python startup, server restart or interruption.
An untimed original native solve in a separate database initializes the observed context
before seed admission; first native initialization is excluded. Receiving eligibility remains
independently observed and must match, with no retry or forced hit. Weak-runtime release
does not establish database-client/task quiescence; asynchronous protection release can
continue into untimed cleanup. The separate fresh control has a different timing boundary
and supplies no speedup comparison. Compilation and smoke acceptance have completed;
the 33-case measurement campaign remains pending positive assembled qualification.

The publication cases separate physical stage/reopen, document admission, modeling
publication, simulation preparation/execution, result reading and analysis activation.
The durable study case exercises the private worker physical-source and selected admission
route; public publication uses protected source reopening followed by one public document
admission. Standalone fitting/shooting remain one-shot consumers rather than a manufactured
repeat session. Empty/skewed selection and interrupted recovery have correctness controls;
these cases do not measure their timing. Native installation and deployment admission use
complete-operation observations from the existing build and assessment routes, whose Cargo
sample durations alone do not include setup. The build owner now observes initial
launch/admission, native setup and final authenticated drain separately from target timing.

Four bounded additions bring the selected campaign to **37 cases**.
`results-publish-state-growth-1` (128 states, 15 samples) and
`results-publish-sample-growth-1` (80 states, 31 samples) isolate each extent against
the existing 80-by-15 fixture. Complete preparation includes policy resolution;
publication and protected reads include production IPC preflight and trajectory
projection. Reports name actual resolved policy-target counts and the fixed registry
schema width; they do not attribute an isolated lookup/preflight speedup.
`source-edit-copy-small-1` and `source-edit-copy-large-1` copy and parse the exact
authored bytes and perform a complete unchanged canonical declaration edit at 80
and 128 units. Exact byte/declaration checks and unchanged revision/sequence checks
are untimed. These typed-declaration ingress cases do not claim document reimport or
scientific solver acceleration. All four additions use the ordinary wide batch.

Reconcile added coverage explicitly rather than silently rerun the whole registry or omit a
new mechanism. N4/T5/L4 supply affected workload conditions and existing case mapping. Add
bounded cases for policy/preflight/trajectory projection growth and unchanged-byte authored
edit/copy work. Existing 80-by-15 and 128-by-31 publication fixtures cover coordinate/sample
extent and complete physical stage/reopen, append/read and analysis activation; the 1 MiB
comment-padding variant covers physical source growth. None establishes differing payload
schema width. Record genuinely distinct supported width variants at their writer owner, or
retain the fixed-width limitation explicitly. Standalone fit/shooting have no supported
repeated session and remain one-shot; do not manufacture a repeat loop. Actual native setup
and artifact-edit locality use existing build measurement routes, extended to observe initial
setup/admission and final drain separately from target/Cargo intervals. Keep benchmark
declarations in their existing registry/script owners; no parallel measurement service.

Separate build/native installation and schema/fixture setup, selected preparation, numerical
execution, durable append/read and teardown. The interrupted whole-test observations cannot
serve as isolated solve or database latency baselines. Use cold/warm, value-only/structural and
relevant/unrelated edits, plus sample/coordinate/source growth, selective/empty/skewed reads
and complete recovery where applicable. Keep production accuracy and sensitivity basis and
identify force-validation overhead separately. Exact transported rows are compared with their
own original payload; independently solved outcomes use the production decision allowance.

Use the existing `just build-frontend`, `just build-uncached`, `just build-cache-probe`
and `just build-storage` routes as applicable for default/native closure and turnaround.
Do not clean the whole cache, share target directories or compare concurrent/interrupted runs.
Where no comparable pre-pivot baseline survives, report target-only timings and structural
removal with no speedup claim. INLINE or batching hypotheses require complete-operation
measurement before claiming a gain. No invented latency threshold or new instrumentation
framework is required; raw samples and honest limits accompany **Measured** claims.

### E5 acceptance and retirement after E3/E4

Conduct the binding's bounded independent assembled review on the demonstrated final scope.
Reconcile US/EF/F/PE dispositions at the coordinator and original scientific findings at their
existing owners. The earlier Revise audit does not satisfy final acceptance. Route enduring
contracts, supported deployment/accuracy limits and measurements through the existing
decision/design owners with a blueprint revision row; follow the required ADR adoption route
without treating maintainer authorization as accepted ADR status. Retire completed plans,
resolved reviews and displaced references only after surviving meaning has its owner. Leave
explicit scientific/provider or measurement exclusions with their real disposition and trigger.
No retirement, ADR acceptance or final qualification was performed by this documentation work.

## Parallel extension acceptance and campaign restart

The [coordinator's parallel target](28-surrealdb-unified-substrate.md#parallel-execution-integration)
adds A4/B6/T7/N8–N10/C6/C7/L7 and affected D1/D2 to E3/E4/E5. The maintainer accepted all
four Parallel RC decisions on 2026-10-08. Directions are confirmed; actual design amendments,
implementation, targeted correction evidence and qualification are not supplied by plan adoption.
The baseline is committed `84a1caf17656f00f38b22e703c7cbc2b63a44d2d`; no new product campaign ran for this authoring extension.

### Targeted readiness before restarting

Keep the reference campaign stopped until the selected route has its working contracts,
consumer migrations and targeted controls. A4 ends its attribution/guard decision; N10/L7 end
the native strategy and deployment decision; changed contracts follow R0. N8/N9 supply actual
temporary admission and scope ownership, B6/T7 integrate protection/publication, and C6/C7
compose ordinary studies. Extend N4/T5/L4 reconciliation to all these consumers, including
Rust public APIs, managed receivers and the installed Python route. An investigation can end
with a supported retained mechanism, but unresolved required concurrency remains a finding.

The existing native `conformance_parallel_sixteen_` selection keeps its original scientific
checks and force-validation. Its prior two failures are a zero-baseline failed result, not a
native concurrency ceiling, memory diagnosis or acceptance transfer. Preserve the distinction
between its 2 GiB fixture pool and the broader 128 GiB reference pool/16 GiB worker capacity;
L7 must establish the selected aggregate resource placement rather than assuming either pool
is an RSS cap. A reader-only pass does not settle complete-case operation.

| Required targeted behavior | Package evidence supplied to E3 |
|---|---|
| Sixteen complete authored cases and staged initialization | A4/B6/T7 plus N8; actual native entries, full inventory, discovery-order report and original assessment under unchanged production science. |
| Ordinary independent study execution | C6: sixteen fresh non-batching points through the public ephemeral route; actual overlapping workers, distinct repeated bindings and exact result placement. Keep continuation/seed order, failed-point isolation, finite admission and cancel/drain. A specialized native batch or independent conformance fixtures alone are insufficient. |
| Durable study deployment | C7/L7: the selected ordinary Rust/Python route and finite managed group complete ready points concurrently; no hidden extra local worker. Exercise claim races, stale generation, loss/reopen, uncertain claim/publication and cancel-after-start with drain. |
| Temporary preparation admission | N8/B6: generous capacity does not multiply immediate demand; real retained pressure yields bounded waiting or truthful typed refusal, with original deadline/cancellation and oversized refusal. |
| Native resource/coexistence | N9/N10: all actual persistent teams charged through idle scope/teardown, admission refusal before team creation, supported mixed/native profiles and cancellation/exclusion wait without resource cycles. |
| Protected publication and serving | A4/T7/D1/D2: concurrent pin/root/result publication and retirement, changed/uncertain replay, final-statement errors and retained escaped Arrow buffers preserve exact history and partiality. |

Use existing touched-package compile and narrowly selected unit/native recipes while implementing;
correctness invocations retain explicit force-validation. Test names/selectors added with each
mechanism must expose actual operation concurrency, not merely configured worker counts or timing
speedup thresholds. Do not run a full campaign per package. The relevant integration/solver/Python
journeys and static bundles remain E3 at functional scope end under the existing recipes.

### E3, E4 and E5 completion

E3 combines review S01–S10 with the existing scientific/recovery acceptance; no previously required
journey is silently removed. Build/Cargo jobs, Nextest/xdist placement, application cases and native
teams have separate selected limits. Parallel tests containing sixteen workers retain exclusive
outer slots; memory-heavy suites and the server use L7's aggregate placement. Qualification must
name the executed recipe/profile, selected inventory, zero-failure result and unsupported limits.
No power-loss, distributed deployment or universal sixteen-native-solves guarantee is inferred.

After positive corresponding functional evidence, E4 measures preparation/store waiting, numerical
execution and publication/serving separately, with complete wall time for the operation. Include
independent case/frontier growth, one versus sixteen workers where scientifically comparable,
cold/warm and changed-premise runs, selected internal teams and retained-session pressure. Use
existing case/build-measurement owners; add a selector only where the declared workload is missing.
Keep original tolerance/start/assessment/resource conditions explicit. Preserve comparability or
report target-only measurements; counts of jobs, RPCs, factors or candidates are mechanism evidence,
not elapsed-time proof. No sixteen-fold speedup or arbitrary latency target is required.

E5 reconciles Parallel F01–F05 at the sole coordinator, the accepted decisions and their actual
routes, and separate architectural/scientific conclusions. Local package completion is not
whole-finding closure. Carry enduring contracts through their architecture owners with a revision
row; retire plans/reviews only after their references and remaining obligations are resolved.

## Preparation acceptance and investigation handoff

The [preparation integration](28-surrealdb-unified-substrate.md#preparation-assurance-and-reuse-review)
adds I/J and A5/B7/C8/N11 to future affected acceptance. I0 owns U01's receiving premise
decision; J0 owns the complete basis, fresh attribution and explicit description/effect
decision. [28i](28i-runtime-validity-and-interruption.md) and
[28j](28j-pure-preparation-and-publication.md) own those packages. This document retains the
single assembled evidence owner and U02 route; it does not duplicate their implementation status.

| Boundary | Revealing acceptance after the working correction |
|---|---|
| Receiving validity | I0-supported roots, initial qualification versus unchanged sequential uses, relevant mutation/new receiving context, unsupported fresh preparation, actual artifact/role and corrupt-payload refusal. Independently valid admitted products require their consumed-premise argument, not only a sealed type. |
| Pure/effect composition | Pure basis/body preparation without store/host infrastructure; warmed memos still expose and publish/settle all exact required descriptions. No cached protection or acknowledgment permission. |
| Canonical recovery | Identical exact rooted offers can reuse prior acknowledgment; expiry before new admission refuses while committed acknowledgment recovers after expiry. Released roots, dependency changes, corruption, concurrency and lost responses preserve current visibility/recovery. |
| Stable preparation/current meaning | Fresh versus reused unchanged/value/structural requests, demand/order/physical/provider/absence changes and A/B/A. Current revision/instance/spans and numerical values remain accurate; escaped owners survive eviction/clear. |
| Study composition | The existing thousand-point Python flash journey and actual ephemeral/durable ready/managed routes distinguish pre-view work from view hits while retaining every occurrence/start/result and physical assessment. Small targeted controls expose the mechanism before scope-end complete journeys. |
| Analysis composition | Representative ordinary, initialization/recycle, dynamic mode and fit experiment/transient consumers preserve their actual distinct products, derivatives/layouts, domains and original physical checks. Do not infer every step/trial rebuilt from the reviewed entries. |
| Interruption/resources | Caller stop/original clock during capture and waits, surviving follower, last-consumer settlement, panic/refusal and actual drain/charges. State remaining foreign lock/call limitations; no fixed stop-latency assertion follows from polling. |

Select the smallest existing or new focused controls that expose each changed mechanism,
then relevant complete journeys once the new functional scope is implemented. Numerical
assessment must remain independent of native return status. The same cached transformation
is not its own correctness oracle; use independent expected physical/layout facts where
necessary. Existing untouched model-conformance receipts keep their original conditions,
not a new whole-product equivalence claim.

I0/J0 decisions and necessary authority amendments precede dependent implementation.
A5/J1 supply working explicit publication before J2/B7 reuse; C8/N11 and applicable I1/I2
must supply tested consumer integration before asserting assembled correction. A plan or
agreed interface is not readiness. Report named commands, mode/scope, zero baseline and actual
results; scheduling this acceptance is not newly executed evidence or closure of PA/U findings.

I/J and their consumer correction are implemented with scoped evidence in Outcome. The
original failed-identity Python continuation subsequently completed in Plan 30. The maintainer
authorized all remaining Plan 28 and full E3/E4/E5 execution on 2026-10-10; the current checkpoint
owns its sequence. Corrected-composition and historical repaired-composite evidence retain
their original conditions and do not replace the final assembled campaign.

Quantitative benefit stays Proposed until later authorized paired complete-operation
measurement under recorded source/artifact/profile/store and receiving conditions. Separate
receiving observation, pure preparation, native execution and publication/result serving;
the existing SHA sample is not a complete test cost or a remedy's speedup. E4 retains its
existing obligations, without a new mandatory matrix, SLA or instrumentation framework.
Historical-result assessment remains ±10%; solver stopping and physical checks are unchanged.

## Graph and hashing extension acceptance

The [coordinator's selected extension](28-surrealdb-unified-substrate.md#graph-and-hashing-review-integration)
adds J3/B8/N12/N13, selected RC04 V2 and bounded GH investigations. This document remains the sole assembled
evidence owner; correction progress and investigation decisions stay at their companions.
The completed original qualification continuation, historical Outcomes and remaining E4/E5
obligations retain their current authority. Planning the following acceptance does not run
or authorize a new campaign.

| Boundary | Proposed revealing acceptance after working correction |
|---|---|
| J3 inventory/settlement | Warm many-body primary/original views expose complete exact descriptions without repeated discovery. Changed reads, acknowledgment errors/absence, expiry, released roots and new admission preserve current outcomes; fresh lineage and escaped-owner charges remain. |
| N12 physical flow | Reordered decision/connection declarations and parallel occurrences retain physical bindings, policies, typed refusals and an independently expected forbidden-cycle witness. N12 preserves identity bytes; separately selected V2 adds explicit role/count framing under ADR-0167. |
| B8/N13 supplier composition | Selected versus full scope, an unrequested cycle, nested/sibling providers and missing unselected observations preserve scope. Actual derivative upgrades, starts/policy and new mutable case state remain current across reused topology. |
| Ownership/interruption | New retained/transient inventories and adjacency are admitted/accounted before allocation; aliases survive eviction/clear and charges last through drain. Caller cancellation and current effect clocks remain unchanged. |
| Adopted later GH alternative | Only a separately selected working realization adds its relevant encoding/collision, complete negative/membership invalidation, topology, receiving or restart controls. An investigation decision alone is not product qualification. |

Use targeted compile/unit controls during correction work, with force-validation and relevant
native environment. Once the selected functional extension and its affected callers/deletions
are complete, choose the relevant existing Rust/Python solve, initialization/recycle,
nested-provider, analysis/mode and fitting/study journeys. Integrate them once under the
authorized scope-end route with applicable static/manual checks; do not rerun whole campaigns
per package or equate a focused mechanism pass with whole-product scientific acceptance.
Independent expected identity/graph/physical facts must expose a wrong index or selected
closure; the retained graph is not its own oracle. Keep case physical/derivative assessment
and ordinary solver tolerances intact.

Source reasoning can establish removal of nested scans and repeated immutable construction.
Quantitative benefit remains **Proposed** until authorized paired complete-operation
measurement: include cold index/topology construction, warm lookup/selection/current settlement,
retained/transient state and recovery. Separate pure preparation, numerical execution and
publication/serving; no smaller workload, changed accuracy threshold or bulk hash throughput
stands in for the affected operation. Record actual source/artifact/profile/store, modes,
commands, zero baseline and results. E4 keeps its existing measurement route; these proposals
add no mandatory benchmark framework, SLA or unrequested campaign.

Closure of each Graph/hash finding stays at the coordinator and consumes linked package plus
affected acceptance evidence. Existing PA/U02 and earlier scientific findings keep their own
obligations; the new review/plan neither resolves them nor reopens them without new evidence.

### Graph/hash scoped acceptance — 2026-10-09

**Implemented:** J3/B8/N12/N13, FlowProjectionV2 and the selected local interner/prehash/key
reuse changes are integrated, with displaced discovery/lookup paths and temporary inquiry
hooks removed. [28k's followup outcome](28k-graph-kernels-and-hashing-investigations.md#followup-verification-and-outcome-2026-10-09)
owns the implementation account, commands, diagnostic measurements and corrected mistakes.
The companion packages own their mechanism-specific evidence; this section owns affected
acceptance and its limits.

**Tested:** compiler 6/6, structural flow 6/6, identity 61/61 and backend tears 3/3 controls
passed. After the maintainer cleared target artifacts, current-source native factorable
controls passed 47/47 and runtime controls 24/24, followed by all four selected native Python
journeys. All used the pinned checkout environment, explicit force-validation where applicable
and a zero-failure baseline. The native Python extension was rebuilt before those journeys.
The exact commands and test conditions are linked in 28k; local and public consumers exercise
indexed settlement, exact/colliding key reuse, selected/nested derivative demands, flows/recycle,
warm starts, retention and public preparation/conformance limits. Tooling repairs passed
539/539 controls, including 85 supervisor tests, and actual native observer drain restored
the original 144 GiB allocation cap.

**Tested:** final production `just bench-case-smoke build/graph-hash-followups-20261009-restart-final --case k4-restart-local-1`
passed with native-process, force-validation, one native thread and `measured: false`.
It demonstrates actual portable reuse after runtime-owner/pool release, fresh reconstruction
without eligibility, original historical rows and exact floating-point bits. The benchmark
now waits for both actual owner release and zero pool reservations, preserving original
assertions; one scheduler yield was a racy teardown assumption. This remains a same-executable,
warmed-loader control rather than managed-worker, process/Python startup, crash, server restart
or strict deployment qualification.

The PC-SAFT smoke harness completed with four typed refusals at case resolution, zero prepared
products. [28f's numerical-fact boundary](28f-shared-numerical-preparation.md#pc-saft-numerical-fact-investigation-boundary)
retains that observation; correction and current targeted acceptance now belong to
[Plan 33/EFF06](33-efficiency-principles-remediation.md#eff06). The later codebase-efficiency
review's F01–F11, investigations and affected qualification belong wholly to Plan 33, not E3/E4/E5.
The feature matrix was cancelled at the maintainer's request; interrupted coverage is not a pass.
These selected journeys do not establish full library/scientific qualification,
an end-to-end speedup, managed-primary deployment identity or complete Plan 28 E3 acceptance.
The full E3/E4/E5 campaign is now authorized; the current checkpoint owns execution. RC02 durable fast hashes and RC03 production
persisted incremental runtime remain unselected.

## WebSocket environment qualification handoff

[Plan 30](30-websocket-and-persistent-agent-environment.md) owns the new WP01–WP08
environment correction and the prospective AE-25/AE-26 follow-ups. Its
[30d D3](30d-host-admission-and-timing-qualification.md#packages-and-scope-end-acceptance)
qualifies the composed RPC, service/receiver, fixture/evidence and host-admission scope.
This packet remains the broader substrate/scientific qualification and existing failed-
identity owner. Evidence may satisfy a shared named journey only with its original
source/artifact/profile/residency conditions; copied links are not new executed tests.

Before future affected assembled qualification, consume the actual working WS complete-
page/settlement adapter, explicit isolated test contexts, separately admitted receiver
generations and declared timing profile. Preserve the exact reference declaration,
physical checks and engineering solver stopping budgets; historical comparisons remain
±10%. A transport change does not establish the observed HTTP/2 failure's cause or cure.

The completed Plan 30 implementation owns its local and assembled controls. Its
[Outcome](30-websocket-and-persistent-agent-environment.md#outcome-recorded-after-implementation)
records the repaired composite evidence and exclusions. The full remaining Plan 28 campaign
is now separately authorized; its current checkpoint owns execution and benefit measurements.

**Tested, failed-identity handoff, 2026-10-09:** Plan 30 completed the original
`test_flash_sweep_prepares_structure_once[managed-durable-science]` identity under
unchanged reference160 conditions: 1 pass, 0 failures in 1,832.49 seconds, one selected
identity, passed terminal reconciliation and no report errors. Its original 1,000
occurrences, native scientific and retained-result/physical assertions remain intact.
The command and artifacts are at the linked Plan 30 Outcome. This closes that named
failure-only continuation; it neither closes broader E3/E4/E5 nor establishes the
historical HTTP/2 failure's root cause. Solver stopping budgets are unchanged.

## Work packages

| Package | Prerequisite and delivered behavior | Completion boundary | Status |
|---|---|---|---|
| E1 — Controlled target regeneration | R0; A2 schema/selection and B1 admission slice. Replace storage-specific fixture setup, regenerate scientific source corpus and run configurations; add recipe-owned target DB setup. | Target inputs are admitted with exact interpretation; old test setup does not silently start PG/Delta. No migration importer or compatibility fixture service remains. | Implemented; focused declaration/schema and linked scientific/generated consumers passed. |
| E2 — Retirement and build locality | Joins each migrated A/B/C/D consumer as it becomes ready; tooling corrections can begin after R0 independently. Remove displaced crates/codegen/dependencies/recipes and implement scoped native setup/unchanged generation. | All target consumers work; remaining dependency consumers are explicit and justified; replaced mechanisms/tests/fixtures are gone. This does not postpone package-local deletions. | Implemented; focused native and linked consumers passed. Current development Rust/Python acceptance is selected by E3; optional strict producer association is outside this campaign. |
| E3 — Assembled correctness and recovery | All functional A/B/C/D and adopted N/T/L scope, including [parallel readiness](#parallel-extension-acceptance-and-campaign-restart), N4/T5/L4 reconciliation, B4's bounded decision and E1/E2 integration complete. Run one selected static/integration/scientific campaign against regenerated target. | Zero failures in named required scope; explain unsupported scientific/provider limits, repaired composite runs and actual crash/durability conditions. | Attempted; interrupted/nonqualifying; expanded functional handoff precedes fresh E3. |
| E4 — Preparation, build and operation measurement | E3 for scientifically comparable runtime measurements; targeted build instrumentation can be prepared earlier. Execute selected scientific and pivot efficiency measurements under recorded conditions. | Report measured scope and limitations without invented speedup thresholds or comparison to incomparable receipts. | Scheduled |
| E5 — Assembled review and closure | E3/E4. Conduct the binding's bounded assembled design assessment, reconcile finding owners and migrate enduring meaning through the decision/design route. | Demonstrated architectural/scientific scope has owners; qualification is published honestly; plans/reviews retire only when their references have moved. | Completion audit delivered Revise; final assembled acceptance/closure pending. |

Root integrates shared manifests, registry/generators, recipes and qualification selection.
Separate workers may own bounded package consumers, but must not concurrently rewrite these
shared surfaces. E2 is a continuing integration responsibility; it is not a late permission
to keep obsolete code after a replacement is proved.

## Verification and assembled acceptance

**Proposed:** during functional execution use `just check-package`, `just check` where needed,
`just unit-package` for isolated units and `just unit-native-package` for selected adapter units.
Add a recipe for a targeted server mechanism when no existing recipe fits. Correctness recipes
retain explicit force-validation; heavy native/linked Python runs retain memory caps. Product
integration and static hygiene run once at E3 after all functional scope, unless requested
earlier by the maintainer. `just turn-end` remains the root's end-of-turn formatting bundle.

E3 selects actual journeys from S01–S08 and the applicable 25k scientific controls:

| Complete operation | Required evidence beyond local units |
|---|---|
| Edit, select, compile, solve and reopen | Exact new revision/value attribution, preserved compatible reuse and physical correctness; structural/name/deletion/provider changes invalidate the right eligibility. No full ancestry/bundle retention. |
| Ordinary success, failure and cancellation | Scientific observations automatically queryable after restart, honest terminal/usable distinctions, actual start retained, no raw staged prefix admitted as success. |
| Concurrent study and interrupted completion | Scoped readiness, distinct occurrences, two-worker fencing, negative-premise conflicts, frozen result membership/late-write refusal, stale generation refusal, uncertain acknowledgment settlement and cancellation/drain. |
| Connected query, trajectory and analysis | Exact selection and method lineage; bounded streams/blocks; failed statement/export cannot become a complete artifact; retention cannot tear protected reads or delete active compilation inputs before product-root admission. |
| Scientific workflows and refusals | Steady/dynamic flowsheets, implicit roots and response, contextual accuracy, fitting, event/shooting/continuation and partial outcomes retain independently specified meaning under native Surreal/Arrow boundaries. |
| Rebuild, crash and restore | Controlled inputs regenerate; acknowledged writes survive abrupt termination/reopen in the selected profile; restore preserves exact identities/interpretation; stored descriptions reconstruct numerical products without full semantic recompilation. |

Use the command surface's relevant `just native-test`, `just native-python`, component,
conformance and `just parity` selections, after target fixtures and linked Python are refreshed.
Run `just hygiene`, fix failures and rerun the failed recipes, followed by relevant manual
`just governance`, `just docs`, native and powerset checks. Confirm final commands against
`just --list` at execution time. No full wheel/release campaign is implied. Name each actual
command, mode, features, memory/concurrency conditions, zero baseline and result in the Outcome.
Do not report a repaired composite as a clean initial pass.

E4 measures supported complete operations: cold and warm selected compilation/preparation,
value and structural edits, A/B/A reuse, durable studies, long result selection and default
versus native build turnaround. Preserve the checkout/toolchain/cache conditions; use the
owning build-measurement recipe without broad `cargo clean` or a shared target directory.
Capture a comparable prior baseline before deleting its mechanism if feasible; otherwise
report target-only measurements and structural changes without a speedup claim. Retain the
17 prepared scientific controls only with their actual valid target conditions. Counts that
show removal of repeated work are mechanism evidence, not a replacement for timing.

Completion requires executing and reporting the selected campaigns, not meeting an invented
latency number. If an operation exposes a concrete resource/correctness failure, repair its
owning package and rerun the affected scope. Extra SIMD/JIT/WASM/server-distribution or
generic search/MCP work needs a concrete new requirement; it is not hidden acceptance scope.

## Checkpoint and next step

The latest maintainer instruction interrupted the campaign and pivoted to the
[SurrealDB review](../design_review/reviews/design_review_surrealdb-capabilities-lifecycle-and-integrations_2026-10-10.md).
[Plan 35](35-surrealdb-lifecycle-and-integration-remediation.md#existing-plan-coordination)
owns its corrective scope separately. The campaign remains stopped. The sequence below is
a conditional continuation route, not an instruction to relaunch during plan authoring.
After later authorization, consume the applicable Plan 35 lifecycle/clock, storage composition,
diagnostic, read-coordination and terminal-study slices wherever a covering invocation uses
those contracts. Shared matching evidence may serve both owners; an unrelated inquiry does
not become a blanket campaign gate.

The latest attempted `plan28-e3-unified-state-qualified-20261010` setup capture recorded
754 tests, eight failures and three errors against baseline zero; the enclosing campaign
was interrupted with exit 130. The review distinguishes ten source-supported stale-fixture
explanations from one unresolved nested-observer drain failure. The two existing fixture
edits remain unvalidated. The review's isolated cancellation pass and lock-clock
counterexample retain their narrower scope and do not qualify E3. Future failure evidence
must retain the decisive owned-lifecycle observation rather than infer a leak from Boolean
drain refusal. This checkpoint owns the campaign's current status; the review retains the
original observations.

The earlier full remaining Plan 28 execution authorization on 2026-10-10 superseded
failed-identity-only and sequential correction instructions. Its starting tree was
`991fcea1984ea9cca0f281f050c87fad067a1431`; historical attempts and
scoped corrected results retain their original Outcomes and cannot qualify the final tree.

[Plan 34](34-unified-state-management-and-persistence.md) independently owns the new
state-management corrections, inquiries and finding dispositions. Its
[handoff](34-unified-state-management-and-persistence.md#existing-plan-coordination-and-qualification-handoff)
supersedes the unconditional restart instructions from earlier continuation attempts below.
Before relaunching an invalidated covering invocation, consume the applicable implemented
and targeted-tested slices: SM06/SM09 for durable cancellation, SM07/SM09 for recovery,
SM09 for phase/owner-correct preparation and claim controls, and SM01–SM05/SM08 where the
journey consumes allocation, retrieval or concurrent-startup changes. The integrated Plan 34
corrections and its actual native, cross-process, Python and scientific restore/rebuild
acceptance are available at its [Outcome](34-unified-state-management-and-persistence.md#outcome-recorded-after-implementation)
and [evidence](34-unified-state-management-and-persistence.md#acceptance-evidence).
Its owning checkpoint retains final qualification status. Reconcile the exact current
source/profile/tool closure before relaunch; scoped Plan 34 passes do not qualify this campaign.
E3/E4/E5 remain owned here. Share exact matching evidence rather than duplicate campaigns;
no completed Plan 30/33 Outcome is reopened and no existing process is stopped by this handoff.

The retained `plan28-e3-development-creator-completion-listener-fixed-20261010` native gate
terminated with exit 100: 3,114 tests run, 3,105 passed, nine failed and eight skipped
against a zero-failure target (native-test summary 5,159.997 s). The enclosing assessment
has not established E3 completion. Its cancellation, conformance, caller-drop and claim-expiry
failures inform Plan 34's corrections; the source review retains its original observation date
and incomplete-capture scope. Reconcile final assessment receipts before selecting a relaunch.

All nine exact identities have subsequent passing Plan 34 receipts in the shared invocation
owner: `445f26fed65c49a686dc4db449704d52` covers derivative scope, fatal-fixture drain,
slot reuse and the original conditional-reference fixture; `11df74a1da444a0aa236f3289742b367`
covers cancellation and sixteen-worker results; `ffe2835a31e14e08a24b7809d45942d1` covers
generic documents and expired claims; `67e6859036c04325837bab4c71e65b0b` covers cross-process
SCIP cancellation. Their XML hashes match retained terminal artifacts. The conditional
fixture passed in 108.787 seconds with its unchanged 4 GiB inner pool and scientific
assertions; the enclosing twenty-case invocation was a repaired composite, not a clean pass.
That narrower feature/context evidence does not replace the fresh full-native E3 campaign.

**Tested, original conditional-reference control, 2026-10-10:**
`PSE_SURREAL_STATE=<reference-state> PSE_WORKER_BINARY=target/debug/pse-worker
PSE_MEMORY_MAX=140G scripts/pse-env --resource-class reference
--native=solver,klu,isolation,uno,petsc -- just native-test --profile local -p pse-runtime
-E 'test(conditional_unit_reference_request_declares_original_inventories_and_root_refusal)'`
passed one selected test, zero failed, 781 skipped in 52.560 seconds (Nextest
`342dfb0a-f3e0-4f67-8372-ff0e95ef145e`; baseline zero). The selected reference service was
explicitly resumed before invocation. The fixture retains its original 4 GiB pool,
inventories and refusal oracle; this pass does not establish the historical failure's sole
cause or enclosing E3 acceptance. Full command output is retained at
`build/plan28-conditional-original-20261010.log`.

1. Reconcile live plan/index/command instructions before implementation. Implemented packet
   mechanisms remain implemented; N4/T5/L4 and E3–E5 acceptance remain open. Plan 33's completed
   corrections and decisions keep their separate owner.
2. Consume completed functional preparation and Plan 34's exact matching evidence:
   scientific rebuild/reopen/solve after maintenance, authenticated measurement prerequisite
   composition, build-owner setup/admission/drain instrumentation, and all four added growth
   and edit/copy fixtures. These mechanisms are implemented, not a fresh implementation backlog.
   The nine original failed identities are mapped above; exercise any invalidated controls
   without treating a narrower receipt as full-native qualification. Refresh the stale wide
   receiver only through stopped/drained reconfiguration/readmission, preserving state and
   credentials. Reconcile N4/T5/L4 consumers and delete any proved displaced mechanism.
   Repairs use targeted compilation and functional tests before integrated qualification.
3. At functional scope end, run comprehensive development `just assessment`, current
   `just seed-conformance`, pinned `just parity`, actual canonical recovery at the original
   reference allocation, `just hygiene`, governance/docs/native lints and full powerset.
   The assessment already selects ordinary and managed Rust/Python separately. Preserve the
   original thousand-point and sixteen-case workloads, actual concurrency/drain, production
   physical accuracy, zero retries and explicit force-validation. Repair failures at their
   owners and rerun invalidated covering invocations on the stable final inputs.
4. After positive corresponding functional evidence, measure all 37 explicit E4 selectors.
   The 34 ordinary selectors retain their 64 GiB runtime ceiling and wide receiver
   declaration under the same existing 144 GiB parent used for ordinary qualification;
   the three sixteen-point study selectors retain exact
   reference capacities. Each batch consumes a matching functional report, including actual
   store/allocation/native environment. No transfer flag or reduced fixture may bypass a
   mismatch. These are wide-runtime/exclusive-parent and reference observations,
   not timing-profile equivalence claims. The reference host route retains its
   140 GiB primary plus 16 GiB server and 4 GiB observer envelope.
   Build/locality measurements include separately observed native setup/admission/drain.
5. Conduct the independent bounded E5 review on demonstrated final scope, repair material
   findings, reconcile all dispositions and required ADR/design routes, move enduring meaning,
   and retire eligible plans/reviews. Do not mark optional strict producer capture required,
   manufacture repeat fit/shoot sessions, or infer physical power-loss durability.

Root owns campaign selection, shared runner/registry changes and final acceptance. Checkpoints
record state, decisions and next steps; commands and comprehensive evidence belong in Outcome.
Preparation mechanisms and targeted controls are implemented, and all four added fixtures
pass their targeted smoke journeys. The growth smoke exposed
an unconditional second-order inner-construction estimate; its correction now separates
source-bounded capability analysis from actual-order factory construction. Independent
review also corrected full-workspace admission for small bodies. The declared fixtures
now retain actual requested derivative orders. The same smoke exposed a benchmark caller that
ignored paged analysis retirement; teardown now drains analysis membership, inputs and
roots before requiring run retirement, outside the timed operation and under the original
request deadline. The private fixture owner removes any header whose immutable creation
window remains open; this does not claim durable analysis settlement. Explicit stop now
withdraws only that service's queued resume;
coordinated parking consumes the current service lifetime under lifecycle ownership,
including partial-stop recovery. Historical dirty worktrees remain preserved.
The two inner-provider jobs now preserve a deadline explicitly carried by their driver
through the existing scoped admission/join owner. Current staged callers already own an
enclosing deadline and drain; this corrects local clock propagation without claiming a
demonstrated scientific failure in those callers. Exact comprehensive measurement
prerequisites authenticate the producer fixture and compose only its owned selector;
standalone prerequisites retain exact context matching.
Functional preparation and independent bounded correction reviews are complete. Scope-end
formatting, affected static repairs, manual checks, full feature powerset and artifact refresh
are complete before the stable assembled capture. The operator restored
host inotify capacity, and the unchanged real lifecycle controls and complete affected tooling
selection now pass. Actual systemd lifetime termination remains required for native drain;
an empty cgroup alone does not discharge that obligation. Both campaign profiles were
refreshed before those captures; the foreground-cancellation correction below required
another refresh of their frozen Python closures for its subsequent capture. Current launch
prerequisites follow the handoff above. E3 acceptance remains pending;
E4 is not measured; E5 is not accepted. Keep these inputs stable through E4.

The continuation account below explains earlier repairs and launch premises. Its historical
successes retain their stated scope; the next launch follows the Plan 34 prerequisite handoff
above and this checkpoint's campaign sequence, rather than an earlier corrected-tree instruction.

The first assembled attempt exposed four stale tooling fixtures before the scientific
gates. They now exercise the current validation-composer entry, independent native
fixture allocation, admitted observer observation and supported standalone restore,
with explicit refusal of shared-context restore. The complete setup suite passes under
a real native owner. That correction supplied the next attempted capture's premise;
the interrupted attempt supplies no assembled acceptance.
The subsequent launch exposed a failed-resume state transition: ordinary-lane admission
correctly refused the reference service, but its parked flag had already been cleared,
allowing reconciliation to discard the pending resume. Admission and materialization now
precede that transition; failed startup restores closed parked state while retaining the
actual allocation and restored-state validation requirement. A successful retry must
establish listener and protocol readiness. Affected lifecycle/admission controls and
independent source review cover the correction. Both frozen supervisor/worker closures
required refresh before its subsequent capture; interrupted captures remain nonqualifying.
The next capture reached the ordinary Rust inventory and exposed one omitted approved
frame spelling for the implemented structural-body index. The fixture now declares that
existing versioned frame; production hashing and scientific behavior are unchanged.
Its subsequent capture required the corrected fixture's targeted integration control.
Ordinary Rust execution then exposed a cancellation fixture that applied its deliberately
short transaction budget to authentication before the tested operation. The two related
controls now establish selected connections under the normal setup clock and impose the
original short clock only on delayed submission, retaining cancellation, expiry and
no-success/rollback assertions. Their focused native acceptance and independent review
precede the next stable assembled capture.
That capture exposed a fixture teardown cycle in the separate short ordinary-client
control. A retained replacement WebSocket driver belonged to the current-thread test
runtime, while fallback fixture destruction blocked that runtime joining its disconnect.
Replacement transports now use the existing retained fixture executor; retaining an Arc
alone does not transfer runtime ownership. Normal operation and caught-failure teardown
pass their focused controls, including exact resource drain. The exposed body failure came
from immediately reusing the deliberately short ordinary session after an uncertain delayed
probe. The atomic-clock fixture now starts schema setup on a fresh retained ordinary client
with the same short budget, while preserving the separate activation clock and one creation
attempt. It does not claim immediate reuse of the canceled session. Independent source review
found no material defect. No production deadline or actual-drain requirement was relaxed;
that corrected tree supplied a subsequent capture's premise. Interrupted captures remain nonqualifying.
The next ordinary execution control retained its historical result-read owner through
explicit fixture removal. It now releases that owner after the final assertion; cleanup
retains its refusal to remove a database with live result buffers and waits for queued
protection release. The exact settlement and parallel-retention controls pass after this
fixture correction. Stopping that assessment also exposed an observer runner that survived
the gate process-group interruption and continued after its enclosing admission closed.
Those downstream admission failures are interruption artifacts, not independent scientific
findings. All processes from that attempt have now stopped. Foreground observer handoffs
now belong explicitly to their launching native operation: cancellation fences pending
launches, authenticates bound lifetimes, settles valid siblings despite another child's
uncertainty and still attempts direct payload termination after manager failure. Record
exclusion defers handled signals until lock release. Independent worker/primary lifetimes
retain their separate owners. The focused real controls demonstrate nested drain, late-bind
payload refusal and independent worker survival; independent source review finds no material
defect. The full production observer route remains an E3 obligation. Affected controls,
scope-end type/lint repairs, independent re-review and formatting are complete. Both frozen
Python closures have been refreshed through stopped reconfiguration/readmission; the existing
databases, credentials and Rust worker bytes are preserved. The canonical fixture group now
passes together. Those inputs supplied the next attempted capture's premise.
That expanded capture passed setup and prerequisite gates but encountered a transient
service lifecycle reservation during parallel fixture registration; the service retained
its PID/invocation and open admission. Its first reservation owner was not retained, so
the trigger is not established. Listener ownership now tolerates an enumerated descriptor
disappearing while another live descriptor proves the selected LISTEN inode; other access
errors and all process/unit/cgroup checks still refuse. The parallel canonical selection
passes under the configured sixteen slots.
Interruption of that capture also exposed a distinct creator-completion gap: validation
returned interruption without signalling the native creator, and its final group kill could
preempt the observer supervisor's drain handler. Creator retirement now settles its own
foreground handoffs, and parent-owned observer supervisors occupy separate sessions.
Nested borrowers and independent worker/primary owners remain unchanged. The actual
production-observer interruption regression and complete affected/static checks pass.
Final formatting and both stopped-profile frozen-tooling refreshes were complete for the
subsequent attempted capture. Independently killed observer
supervisors still require demonstrated drain and may conservatively refuse completion.

## Outcome (recorded after implementation)

### Assembled continuation corrections — 2026-10-10

**Implemented:** the reference profile's failed resume preserves parked, closed admission
until listener and protocol readiness succeed. Canonical test fixtures release their result
owners before explicit removal and retain replacement WebSocket drivers on the fixture
executor. Deliberately short transaction probes authenticate under the normal setup clock;
their operation deadline and negative assertions remain unchanged. The approved frame catalog
includes the existing structural-body index spelling. These corrections do not reduce the
scientific workloads, tolerances or drain obligations.

**Implemented:** foreground observer handoffs participate in their launching native operation's
cancellation. Pending launches consume a cancellation fence before payload; bound scopes require
matching unit, invocation, cgroup and inode before actuation. Unknown children stay pinned while
other authentic children settle. Metadata exclusion defers handled signals until lock release,
and manager errors still permit direct payload termination. Durable workers and persistent
primary services retain their independent lifetime owners.

**Tested (baseline zero, affected tooling):**
`scripts/pse-env --resource-class light -- .venv/bin/python -m unittest scripts.tests.test_native_operation scripts.tests.test_surreal_server scripts.tests.test_execution_contracts -v`
passed 202 tests in 58.562 seconds. Three revealing real controls passed in 10.845 seconds:
assessment interruption drained nested foreground scopes and a runner in a new session;
canceled late binding refused payload and drained; an independent worker survived its launcher
before eventual reclamation. Four final callsite and malformed-child controls passed in
0.251 seconds. The nested regression composes production handoff/scope construction;
full production `observer()` route acceptance remains with E3. Independent source review found
no remaining material finding. Unit survival is not scientific database durability.
Scope-end `just typecheck` initially reported 13 diagnostics (two suppressed) and
`just lint-py` reported 32 findings. After narrow type/validation/helper repairs, both
recipes passed with no new suppressions. Seven affected functional controls, including
real nested interruption and pending binding, passed in 4.658 seconds on the repaired code;
independent re-review found no material semantic change.

**Tested (baseline zero, native/local/force-validation):** the corrected exact result-settlement
and sixteen-retention-roundtrip controls passed 2/2, with 143 skipped, in 10.879 seconds through
`NEXTEST_TEST_THREADS=1 just unit-native-package pse-operations pse-operations/canonical-tests 'test(canonical_execution_exact_batches_private_closure_and_settlement) | test(canonical_execution_sixteen_retention_roundtrips_overlap_source_staging_and_private_pins)' --profile local`
under the reference allocation. Cleanup still refuses live result-buffer ownership.
The assembled canonical selection
`NEXTEST_TEST_THREADS=1 just unit-native-package pse-operations pse-operations/canonical-tests 'test(canonical_)' --profile local`
then passed 113/113, with 32 skipped, in 598.210 seconds on that reference allocation and
the refreshed frozen tooling. Its launcher exited successfully after the run.

**E3 remains incomplete:** the latest interrupted comprehensive capture passed all 39
prerequisite/static gates, including 701 setup tests, then selected 3,114 ordinary Rust tests.
The result-owner fixture failed before interruption. Nextest subsequently continued beyond its
enclosing admission, exposing the cancellation defect corrected above. Its final interrupted
summary was 3,103 executed, 2,608 passed, 495 failed, 11 not run and eight skipped;
the post-interruption admission failures are artifacts of that invalidated lifetime,
not independent scientific findings. Neither that capture nor the focused repairs establish
assembled acceptance. Refresh frozen tooling and rerun on stable inputs before E4/E5 closure.

**E3 follow-up remains incomplete:** the next capture passed 709 setup tests and all
prerequisite/static gates, then selected the same 3,114 ordinary Rust tests. A transient
lifecycle reservation refused fixture registration while the selected service retained
its process/invocation. The capture was interrupted; its final native summary was 2,968
executed, 2,483 passed, 485 failed, 146 not run and eight skipped in 224.625 seconds.
The grouped admission failures and post-interruption failures do not establish independent
scientific defects or assembled acceptance. Existing evidence does not identify the first
reservation owner.

**Implemented; Tested (baseline zero):** listener observation now ignores disappearance
of an individual enumerated descriptor while still requiring a live descriptor for a
selected LISTEN inode. Two controls passed in 0.004 seconds, including refusal to ignore
access errors. This fixes a concrete observation race without establishing that it triggered
the capture. Creator completion now settles foreground ownership even after an unsignalled
child interruption, while parent-owned observers use separate sessions to protect their
supervisors from gate-group kills. Six focused controls passed in 2.429 seconds, including
`test_real_validation_term_finishes_creator_and_production_observer`, nested borrower
preservation, failure restoration and independent worker survival. The actual observer
regression narrows unrelated scientific placement and uses `profile=None`; scientific
placement and exclusive observer cap borrowing remain E3 obligations. Already-dead
supervisor recovery remains a conservative refusal/pin boundary. Independent source review
found no material defect within this supported interruption scope.
The affected native-operation, supervisor and build-measurement modules passed 203 tests
in 38.351 seconds; execution-contract controls passed 30 in 1.685 seconds. `just typecheck`
reported zero diagnostics (two existing suppressed), and `just lint-py` passed. The same
canonical selection with `NEXTEST_TEST_THREADS=16` passed 113/113, with one slow notice and
32 skipped, in 672.805 seconds. Three minutes of concurrent lifecycle observation found
no reservation; that observation is limited to its interval and does not establish the
original trigger. This remains targeted evidence before a fresh E3.
The default `scripts/pse-env -- just turn-end` launch returned 125 because available host
memory remained below the startup pressure guard. The appropriate existing light route,
`scripts/pse-env --resource-class light -- just turn-end`, passed ADR indexing and formatting.
Both stopped campaign profiles were refreshed through official quiesce/stop/reconfigure/readmit
without starting scientific execution; their databases, credentials and Rust worker bytes
remain preserved. The next assessment uses the original reference allocation after ordinary
heavy-workload admission; the light tooling result does not qualify that allocation.

### Preparation correction — 28i/28j (2026-10-08)

**Implemented:** controlled receiving roots now qualify actual trust transitions, while
independent immutable mathematics and complete exact preparation bases retain their real
allocation owners. Pure compiler frontiers expose all portable descriptions; current
acquisition and rooted acknowledgment/publication run outside tracked computation on both
hits and misses. Current source attribution and case values bind separately. The displaced
namespace cohort, effectful retention callbacks and canonical compiler workspace are removed.
Blueprint §14.3–§14.4 and §5.3 own the enduring contracts; proposed ADR-0164 records the
selected decision without changing its decision-PR status.

**Tested (baseline zero, targeted native/force-validation):** the compiler frontier/reuse
selection passed 11 controls; the Symbolica pool/slot/order selection passed six, and the
callback/attribute negative-control selection passed three tests covering 54 inadmissible
registrations. Build-information cancellation/expiry passed two controls. Runtime preparation,
portable body and retained view controls were repaired through their failing identities;
the revised-source attribution control and original reconnect fixture each passed separately.
The actual-store exact rooted acknowledgment control passed after revision change and pin
expiry (`just unit-package pse-operations` with `canonical-tests` and force-validation).
The controls exercise refused new admission, retained provenance and real store settlement;
remembered cache state does not substitute for acknowledgment.

**Tested (baseline zero):** `just unit-native-package pse-runtime
pse-runtime/native-solvers,pse-runtime/canonical-tests` with the selected scoped-clock,
private receiving-drain and acquired-absence controls passed nine tests (run
`cd1d4ef0`). Eight startup/deadline/cancellation and private-drain controls passed (run
`76773d84`), preserving owners and charges through actual native thread/TLS join.
The final `test(kernel_conformance_attaches_shared_checks_to_authored_model_fixtures)`
selection passed its composed control (run `bb369f97`). Five selected native fitting/experiment/mode controls passed (run
`ca42c714`), including independent experiment profiles, inline sensitivities, smooth,
scheduled and state-reset response contracts. The post-dispatch cancellation control's
obsolete fixture expectation was corrected; its failure-only rerun passed (run
`9b3a197c`), with pre-dispatch refusal and parent isolation unchanged.

**Tested (baseline zero, supervisor tooling):** `just surreal-test` passed 68 controls.
Actual offline quiesce, drained stop, worker association renewal and restart also succeeded
after replacing the worker image. Serving admission still checks current disk artifacts;
drain identifies the previously admitted live executable and launch/cgroup association,
including an atomic replacement's kernel deleted-link spelling. No store materials or
compiler cache were deleted. Native worker and Python extension rebuilds passed without
warnings or errors; the latter included the generated stub check. `just docs` built 272
chapters and search successfully. `just ready` completed with its separately reported
optional solver-container warning; installed native solvers supplied these actual runs.

**Tested (baseline zero, native store controls with force-validation):**
`just unit-package pse-operations 'test(staging_turn_unit) |
test(protected_decision_retries_only_definite_conflicts_and_preserves_final_error) |
test(same_actual_revision_receipt_requires_live_pin_and_preserves_changed_inventory_refusal) |
test(sixteen_same_problem_protected_readers_share_short_turns_with_staging_and_exact_ack)'
--features pse-operations/canonical-tests`, under the installed solver/KLU/isolation/Uno/PETSc
environment and qualification state, passed 6/6 (109 skipped) in 13.905 seconds; run
`90fcd2dd-fbf3-4610-968b-156b05cfa696`. Sixteen clone readers share bounded short turns
with staging, adoption, full requalification, body acquisition, exact acknowledgment and
release while a concurrent source writer runs. Cancelled/expired local queue waiters do not
start new work or block another problem. A live identical actual revision may adopt complete
premises; released/expired pins refuse and changed inventory still requires qualification.

**Tested (baseline zero, native runtime receipt integration):**
`just unit-native-package pse-runtime pse-runtime/native-solvers,pse-runtime/canonical-tests
 'test(checked_selected_admission_) |
 test(ordinary_preparation_rechecks_additional_acquisition_absence_before_basis_reuse)'
 --profile local` passed 7/7 (692 skipped) in 15.495 seconds; run
`9c3d635e-8485-4a4f-9c03-e12ee88cef0c`. This exercises current independent protection,
A/B/A and unrelated-source reuse, absent inventory and nearer-name invalidation,
provider/root separation, byte-bounded eviction, cancellation before native entry and
additional acquired absence. Actual revision qualification is separate from scientific
basis identity; case values still bind independently. Fresh cloned premise allocations
retain their reservation with the selected read through its actual use and release.

**Tested (baseline zero, affected initialization/recycle):**
`just unit-native-package pse-runtime pse-runtime/native-solvers,pse-runtime/canonical-tests
 'test(initialization_restores_original_specification) |
 test(reference_recycle_initialization_preserves_scientific_obligations)'` passed the
initialization control, but the default Nextest profile killed recycle at its 120-second
harness limit. Failure-only rerun of
`test(reference_recycle_initialization_preserves_scientific_obligations)` with the existing
`--profile local` passed 1/1 (698 skipped) in 588.460 seconds; run
`e6b7f046-4c2f-410a-bb32-4e9b938bb087`. Both runs retained native/force-validation and the
original 600-second task limit and physical assertions. This is a repaired harness-profile
result, not a change to the solver's stopping policy.

**Correction:** the first rebuilt ephemeral sweep exposed a generic SDK deadline incorrectly
used as scientific preparation lifetime, plus redundant rechecks of already checked
selection premises. Scoped children now preserve the original absolute driver clock;
unscoped preparation has a separately named finite ten-minute omission policy. Whole
preparation expiry is a typed time resource limit; an actual canonical RPC keeps its own
30-second request bound. Additional receiving/acquisition premises still recheck, as the
actual acquired-absence-to-present control verifies. Private receiving work now drains before
source protection releases. No solver stopping or physical acceptance budget changed.

The subsequent managed sweep exposed same-process canonical contention: its worker
exited with a definite transaction conflict after the existing 32 complete protected-query
attempts, leaving five settled and 995 unassigned points. The observer later returned an
HTTP/2 driver failure at 659.12 seconds. Its job was cancelled through the existing study
policy, preserving all records. The ephemeral sweep advanced through changing live pins,
but repeated full immutable-revision qualification made progress slow; it was interrupted
at 939.37 seconds before the revised artifact rerun. Neither run supplies a positive sweep
receipt. Protected selection/acquisition/acknowledgment now share the existing weak
per-problem short-transaction pacing owner with staging. A guarded live identical-revision
receipt adoption avoids repeated full qualification; changed revisions and extra acquisition
premises still qualify. The conflict budget and distributed guarded predicates were preserved.

The rebuilt managed sweep subsequently failed at 560.76 seconds with another HTTP/2
driver error, before any of its 1,000 points was assigned. Its worker remained alive and
continued canonical I/O; memory/task limits and OOM counters did not identify exhaustion.
The exact failed study was preserved and cancelled through the existing study policy.
Source investigation exposed two further work-amplification paths: sixteen cold consumers
privately hydrated the same inventory before reaching the pure admission flight, while
every observer poll read all 1,000 occurrence rows before checking for a final result.
The correction paces cold qualification by exact request and actual immutable revision,
with each consumer retaining its own fresh protection, cancellation and qualification.
Completion polling reads the study header at a bounded 100 ms interval until terminality, then retains the original
complete result checks. The specific transport-level cause remains unproven; these source
corrections do not justify blanket replay of uncertain operations.

**Tested (baseline zero, native cold qualification and completion):**
`just unit-package pse-runtime 'test(selected_qualification_turn_) |
test(checked_selected_admission_) |
test(ordinary_preparation_rechecks_additional_acquisition_absence_before_basis_reuse) |
test(canonical_study_summary_live_owner_and_expired_writer_rebuild_without_science) |
test(canonical_study_cancellation_after_summary_close_fences_success)'
--features pse-runtime/native-solvers,pse-runtime/canonical-tests --profile local`,
under `scripts/pse-env --native=solver,klu,isolation,uno,petsc` and the qualification
state, passed 12/12 with 690 excluded in 51.310 seconds (run
`15cdea3e-ddb8-415c-a861-cb1bc112e1ab`; compile 1m55s). Sixteen cold consumers
hydrate once but retain independent current pins and mutable read premises. Cancelled
or expired waiters leave no allocation residue, changed actual revisions proceed
independently, and failed qualification is retryable without cached failure. The two
study controls preserve open-result absence, terminal exact attempt/occurrence handles,
summary recovery and cancellation fencing. Focused source reinspection found no remaining
material correctness finding; it does not prove the specific HTTP/2 cause.

**Deliberate deviation:** the receiving guarantee is limited to controlled worker/imported
Python roots and callback-free append-only mathematical state. Arbitrary embedding,
interposition, live symbol reset and executable-map mutation remain unqualified. The exact
instance-qualified basis uses existing bounded retention; no persistent/global Salsa
accelerator or second generic cache was added. Frozen historical measurements retain their
original conditions. A repaired composite continuation, rather than another full assessment
or a new performance campaign, follows the maintainer's explicit scope.

**Tested (baseline zero, repaired scope-end checks):** `just hygiene --live` initially
reported four unsuccessful recipes: the temporary design-edit marker, one Python style
finding, and the two Clippy modes. The marker was removed after the architecture amendment;
the source findings were repaired. Failure-only `just lint-license`, `just lint-py`,
`just clippy-default` and `just clippy-no-default` all passed. The two final Clippy modes
completed with zero findings in 10.16 and 0.71 seconds. The remaining hygiene recipes,
including generated-output checks, passed in the original bundle; this is a repaired
composite rather than a clean initial hygiene run. `just lint-solver-contracts` initially
reported five findings; its repaired rerun passed with zero findings in 45.69 seconds.
`just governance-tests` passed 102/102, zero skipped, with explicit force-validation
(run `c5a70327-c30e-4b40-918e-8a2612b528c4`, 16.721 seconds). These checks used the
qualification state and installed native environment; they do not replace scientific
journey outcomes. Subsequent source-only lint repairs preserved the functional behavior
of the worker and Python artifacts used by the final sweeps.

**Observed host constraint:** managed worker startup reported an inotify limit warning,
but the service started successfully. The host had 65,252 visible watch entries against
`fs.inotify.max_user_watches=65536`, with `max_user_instances=128`. An authorized live
increase to 1,048,576 watches and 1,024 instances could not be applied because
`sudo -n sysctl -w` required the operator's password. The limits remain unchanged;
no materials were removed to address the warning.

**Tested (baseline zero, after cold qualification/polling correction):** the affected
`just clippy-default`, `just clippy-no-default` and `just lint-solver-contracts` checks
all passed with zero findings, in 30.44, 0.63 and 46.19 seconds respectively.
`just turn-end` also passed ADR indexing and formatting. Earlier passed hygiene/governance
recipes retain their original source boundary; the complete bundle was not repeated.

**Tested (baseline zero, rebuilt ephemeral scientific sweep):**
`PSE_SURREAL_STATE=/home/paul/.local/state/pse-arrow/plan28-qualification-20261008
PSE_WORKER_BINARY=/home/paul/pse-arrow/target/debug/pse-worker scripts/pse-env --
just native-python build/plan28-ij-immutable-pin-sweep-closure-20261008
'python/pse/tests/test_studies.py::test_flash_sweep_prepares_structure_once[ephemeral-preparation]'
-n0 -v --tb=short` passed 1/1, zero failures/errors/skips, in 1,798.78 seconds
(call 1,798.75 seconds). All 1,000 original scientific outcomes and the one-view reuse
assertion passed. Its artifact includes guarded same-revision receipt adoption and short
transaction pacing; it predates the final cold-qualification/polling correction, which
has the separate twelve-control result above. This is the successful failure-only
ephemeral continuation, not a fresh complete Python suite or paired speedup measurement.

The next rebuilt managed sweep completed cold preparation and retained thirteen scientific
points before its worker exited on a definite result-ingestion transaction conflict.
Three other assigned attempts drained to failed terminal receipts; 984 points remained
unassigned. The observer did not produce a normal test result: its native wait deferred
SIGINT, so the ownership-verified observer was terminated after preserving its state.
Only this exact failed study was cancelled through existing policy. Its full state and
worker journal are preserved in
`/tmp/plan28-managed-sweep-failure-01a11e4907dd76a096b7964573a54795.json` and the adjacent
journal; no historical or unrelated study was cancelled.

Source investigation found that execution ingestion and study mutations bypassed the
short per-problem transaction pacing already used by preparation. They contend on the
same actual problem/study guards. The correction includes those short mutations under
the same pacing owner, preserving server authority, original request clocks, complete
definite-conflict retries and uncertain-response settlement outside the turn. Result
retention also preserves its original typed infrastructure cause instead of projecting
it as invalid model input. The managed scientific identity remains pending qualification.

**Tested (baseline zero, execution/study pacing):**
`just unit-package pse-operations 'test(canonical_execution_server_unit) |
test(study_committed_claim_start_and_summary_lost_ack_settle_exact_identity) |
test(study_start_lost_ack_cannot_redispatch_after_cancellation) |
test(study_retry_new_attempt_has_its_own_native_start_boundary) |
test(staging_turn_unit) | test(sixteen_same_problem_protected_readers_)'
--features pse-operations/canonical-tests --profile local`, under
`scripts/pse-env --native=solver,klu,isolation,uno,petsc` and the qualification state,
passed 16/16, zero failures, 100 excluded, in 22.744 seconds (compile 10.43 seconds;
run `a9ded4cc-ec5d-4135-990d-c6e4e5c49fe1`). The new control retains sixteen independent
executions concurrently with multi-block source staging and private protected readers;
existing controls preserve expiry, cancellation, exact replay and lost-ack settlement.
Actual sibling-study contention and the managed scientific identity remain separate
qualification obligations.

**Tested (baseline zero, typed retention failure and runtime composition):**
`just unit-package pse-runtime 'test(shared_study_retention_failure_preserves_infrastructure_cause) |
test(operation_context_preserves_typed_error_and_diagnostic) |
test(canonical_study_summary_live_owner_and_expired_writer_rebuild_without_science) |
test(canonical_study_cancellation_after_summary_close_fences_success) |
test(durable_worker_sixteen_cancelled_occurrences_obey_action_bound_and_drain)'
--features pse-runtime/native-solvers,pse-runtime/canonical-tests --profile local`, under
the same native environment and qualification state, passed 5/5, zero failures,
698 excluded, in 33.630 seconds (run `74eb77f1-02cc-4ea0-967b-e42fde5d9e80`;
compile 2m26s). The diagnostic control preserves the exact shared infrastructure cause
and complete public projection. The first compile exposed a test-only comparison of a
non-`PartialEq` diagnostic; comparison now uses the complete serialized projection.
Summary/cancellation controls and sixteen actual cancelled occurrences preserve bounded
action accounting and native drain.

**Tested (baseline zero, actual shared-study retention guard):**
`just unit-package pse-operations
canonical_studies_sixteen_sibling_retention_roundtrips_share_actual_study_guard
--features pse-operations/canonical-tests --profile local`, under the same native
environment and qualification state, passed 1/1, zero failures, 116 excluded, in
5.332 seconds (run `a7206966-d81c-453e-8c70-cf9c2b44cf4a`; compile 7.97 seconds).
Sixteen sibling occurrences claim/start and append original observations, synchronize
before closing ingestion, reconcile manifests, seal and settle against the actual shared
study guard, and retain exact independent data and attempts. The final summary seals the
actual study. This directly covers the observed contention boundary; complete managed
scientific qualification still requires its failed identity to pass.

**Tested (baseline zero, final retention-repair checks):** `just turn-end` passed
ADR indexing and formatting. The affected `just clippy-default`,
`just clippy-no-default` and `just lint-solver-contracts` reruns passed with zero findings
in 21.71, 1.36 and 30.39 seconds respectively. The native worker rebuilt with
`xtask/native-solvers,pse-relations/force-validate`; `just py-sync-native dev` rebuilt
the editable native extension and verified stubs against its actual compiled API.
Explicit quiesce, drained stop, offline `plan28-reference` receiver readmission and
start succeeded with the existing database and resource allocation preserved.

The final paced-retention managed attempt progressed through cold preparation and
settled 27 points, then its worker exited on `runtime::infrastructure`: an HTTP/2
protocol error with no query-domain error. This differs from the preceding definite
transaction conflict; a particular transport cause is not established. Two assigned
attempts closed ingestion but remained nonterminal, with 971 other points unassigned.
The exact study, all 1,000 point rows, 29 attempt rows and worker journal are preserved at
`/tmp/plan28-managed-sweep-failure-01a11e5e853e7043871a7acf487f831b.json` and the adjacent
journal. Its ownership-verified observer was interrupted after worker exit and returned
241, with no normal test result. Only that study is cancelled through normal policy;
closed nonterminal attempts retain their existing recovery obligations. The managed
scientific identity remains pending. A bounded read-only transport investigation tests
the pinned stack separately from scientific execution, without blanket replay of
uncertain writes.


**Tested (2026-10-08 resource-fixture repairs, baseline zero):**
`just unit-native-package pse-runtime
'pse-runtime/native-solvers,pse-runtime/canonical-tests'
'test(refused_allocation_leaves_the_same_admitted_package_retryable) |
test(occurrence_execution_tests::) |
test(grouped_source_precharges_resources_and_preserves_cancellation) |
test(trajectory_transport_shares_completion_retries_budget_and_retains_escaped_batches) |
test(complete_original_seed_reserves_before_allocation_and_preserves_retry_attempt) |
test(explicit_cone_source_demand_preserves_original_result_under_retained_pressure)'
--profile local` passed 13/13 in 17.947 seconds, with 674 tests skipped and
force-validation. Run `5ea38655-6a3f-4e9a-87b1-2d2479657120` is recorded in
`build/plan28-e3-final-resource-failures-20261008.log` and its separate JUnit copy.
The refusal fixture derives its held reservation from the actual admitted pool
instead of the former 512 MiB constant. The simultaneous-study fixture has an
8 GiB pool for its overlapping 1 GiB compiler workspaces. Refusal/retry, cancellation,
resource release and exact occurrence/preparation assertions remain exercised.

**Tested (2026-10-08 assembled native execution, baseline zero, failed):**
`just assessment build/plan28-e3-parallel-pool-dev-20261008 --python-profile dev --live`
completed the `native-test --profile local` selection with 2,977 passed and nine
failed out of 2,986 executed tests across 86 binaries in 4,083.673 seconds; seven
tests were excluded by selection/profile. Run
`cc90cf49-1ea4-4641-962d-d545fbb8819d` retains its original log and JUnit artifacts.
The failures comprise two resource fixtures, six worker fixtures with stale
supervised-profile premises, and the historical complementarity flash comparison.
The first two are resolved by the targeted result above; the flash targeted result
below resolves its failure. All six worker journeys now have positive composite
follow-up results; the managed and Python selections remain pending.
Source repairs made during the pending Python execution mean this assessment cannot establish a fresh successful
assembled E3 qualification. Its raw observations remain under their original binary
and source conditions; targeted repairs supply separate composite evidence.

**Tested (2026-10-08 worker-fixture execution, baseline zero, failed):**
`scripts/pse-env --native=solver,klu,isolation,uno,petsc -- just test-package
pse-runtime --test worker --features pse-runtime/native-solvers,pse-runtime/canonical-tests
--profile local` ran all six worker tests with force-validation against a separate
`plan28-reference` supervisor state, preserving the installed worker and the pending
Python deployment. Four passed and two failed in 358.758 seconds; run
`e786ca5d-451c-4de3-9302-6ed95d6e3c63` is recorded in
`build/plan28-e3-worker-fixtures-isolated-20261008.log` and its separate JUnit copy.
Stale thread overrides were removed; the two-worker claim fixture uses its own finite
20 GiB deployment and owned database, with worker drain and server stop on teardown.
The two SCIP interruption controls timed out before assignment. A read-only live
stack identified full loaded-file SHA-256 verification during semantic-body retention
inside candidate preparation. The probe observes assignment, not actual SCIP entry.
A test-only 120-second setup watchdog also failed. Live offset samples showed forward
progress and restarted full passes through the 705 MB worker image; the original
40-second watchdog did not accommodate this cold preparation. The finite setup
watchdog is now 600 seconds, separate from production solver/lease/cancellation clocks.
The one-selector extended diagnostic passed `cross_process_cancel_stops_scip` in
273.300 seconds with all original assertions; run
`91e2edb8-3d86-4083-a38c-819a226507d4` is retained in
`build/plan28-e3-worker-scip-preparation-extended-diagnostics-20261008.log` and its
separate JUnit copy. Read-only debugger samples occurred during setup; this is
functional evidence under the revised fixture watchdog, not proof that preparation
met the former 40-second window. The remaining selector
`test(killed_worker_freezes_truthful_observations_and_retries_only_by_authored_policy)`
passed 1/1 in 656.386 seconds, with five tests skipped, in the same native test-package
mode with force-validation. Run `4043dd27-45ee-4998-a192-818deb1af9c1` is recorded in
`build/plan28-e3-worker-kill-recovery-setup-repaired-20261008.log` and its separate JUnit
copy. All six original worker failures now have positive composite follow-up results.
Solver limits, leases, heartbeat, cancellation bounds, frozen facts and retry/scientific
assertions remain unchanged. No production replay optimization is claimed.

**Tested (2026-10-08 managed native selection, baseline zero):**
`just native-test --profile local --managed-primary-route` passed 4/4 in 219.099
seconds, with 718 tests excluded and force-validation, against the separate
`plan28-reference` state. `PSE_WORKER_BINARY` supplied the unchanged installed worker;
the runner compiled test binaries before observer placement and did not rebuild that
worker. The selected controls cover sixteen entered native owners and original
results, cancellation/drain with an untouched tail, reopened secant continuation,
and partial-history/escaped-Arrow retirement. Run
`e3e29b21-ccb3-4cd2-aed8-ebd0bfff3e7e` is recorded in
`build/plan28-e3-managed-native-repaired-20261008.log`; its selection, binary metadata,
native provenance and separate JUnit copy are in
`build/plan28-e3-managed-native-repaired-20261008/`. This separately executes the
managed selection blocked by the original failed native gate; it does not overwrite
that gate or establish a fresh successful full assessment. Python remains pending.

**Implemented (2026-10-08 historical-result assessment):** the maintainer selected
±10% relative allowance for final historical IDAES/reference comparisons. The six
nonzero complementarity beta expectations consume the single authored
`numerical_policy.historical_reference_relative` constant; three zero-reference cases
retain their ordinary absolute allowance. Duplicate Rust historical beta expectations
were removed. Native feasibility, material closure, finite outputs, temperature
agreement and all nine provider/feed realizations remain required. Ordinary solver
stopping budgets remain unchanged. A proposed test-only 1e-8 KKT override was removed
after the maintainer clarified this distinction; its interrupted run establishes no
acceptance. The first historical-allowance rerun exposed a missing required provenance
on the new constant; dedicated comparison provenance now repairs that authoring error.
**Tested:** `just unit-native-package pse-tests-conformance
pse-tests-conformance/native-acceptance
'test(acceptance::complementarity_flash::flash_phase_disappearance_agrees_across_realizations)'
--profile local` passed 1/1 in 496.042 seconds, with 16 tests skipped and
force-validation. This single test exercises all nine feed/provider realizations.
Run `05a8aa7a-a7a6-42cc-bec8-bf2fe65c4e77` is recorded in
`build/plan28-e3-flash-historical-relative-parser-repaired-20261008.log` and its separate
JUnit copy. The observed beta 0.39222538608302865 differs from the historical 0.3961
by approximately 0.98%, within the selected development allowance; the passing test
does not establish that either value is more scientifically accurate.

**Tested (2026-10-08 parallel conformance allocation, baseline zero):**
`just unit-native-package pse-runtime
'pse-runtime/native-solvers,pse-runtime/canonical-tests' 'test(conformance_parallel_)'
--profile local` passed 7/7 in 136.634 seconds, with 680 tests skipped and
force-validation. Run `646b892f-c574-4b1f-8114-0ab0a29d297f` is recorded in
`build/plan28-e3-conformance-pool-expanded-20261008.log`. The sixteen-lane ordinary
helper now has a 64 GiB common pool, proportional to its inherited per-lane capacity;
the serial comparator has 4 GiB. Worker/workspace, foreign-library, CPU/job, scientific
and deadline policies are unchanged. A scoped caller audit found no other inherited
small-pool override; explicit refusal fixtures remain separate. The preceding lifetimes
assessment passed all 39 pre-suite checks, then exposed typed pool refusals in the
cancellation, fatal-fixture and global-cap controls and was interrupted. Its later
managed/Python scopes were not run; it is incomplete and supplies no E3 acceptance.

**Tested (2026-10-08 fixture lifetimes, baseline zero):**
`just unit-package pse-operations 'test(canonical_staging::canonical_server_unit) |
test(canonical_retention::canonical_server_unit)' --features pse-operations/canonical-tests`
passed 23/23 in 38.920 seconds, with 85 tests skipped and force-validation. Run
`3248ea51-664f-4084-8d6e-793da8c12202` is recorded in
`build/plan28-e3-staging-lifetime-repair-20261008.log`.
`just unit-native-package pse-runtime
'pse-runtime/native-solvers,pse-runtime/canonical-tests' 'test(math::solves::paths::tests::)'`
passed 9/9 in 13.909 seconds, with 678 tests skipped and force-validation. Run
`3e96153c-58b8-4d40-81d2-9f1bec6a5330` is recorded in
`build/plan28-e3-path-lifetime-repair-20261008.log`. These test-only repairs preserve
production deadlines, scientific oracles and resource policies. The preceding resourced
assessment passed all 39 pre-suite checks, then exposed these two native-suite failures and
was interrupted; managed/Python scopes were not run. Its receipt is incomplete and these
focused repaired passes do not establish E3.

**Tested (2026-10-08 ordinary fixture resources, baseline zero):**
`just unit-native-package pse-runtime
'pse-runtime/native-solvers,pse-runtime/canonical-tests' <selected-filter>` passed 25/25
in 51.554 seconds, with 662 tests skipped, force-validation and a 4 GiB ordinary pool.
The selection covers the forms/global modeling controls, authored MILP, nested worker and
queue deadlines, goal/no-goal selected-root preparation, public occurrence scheduling,
four-ready native adapter batching, exact seed reservation/retry and cone retained pressure.
Run `b500b553-f4ba-487c-ba1e-8cd904dd3c94` and
`build/plan28-e3-ordinary-resources-pool-expanded-20261008.log` retain the exact filter and
command. The first compile failed on the new test's occurrence wrapper type and was
corrected by requiring its explicit ephemeral owner. The subsequent 2 GiB pool selection
passed 24/25: overlapping compiler reservations caused one typed pool refusal. Raising
that pool to 4 GiB preserves scientific policy, deadlines, foreign-library allowances and
deliberate refusal semantics. The interrupted assembled attempt is incomplete and supplies
no full qualification; its 39 pre-suite checks passed and its later managed/Python scopes
were not run. This focused repaired result does not establish E3.

**Tested (2026-10-08 parallel execution extension, baseline zero):**
`just unit-package pse-operations 'test(/_server_unit::/)' --features
pse-operations/canonical-tests --profile local` passed 73/73 in 186.188 seconds with
sixteen ordinary store-test slots. The subsequent `initialization_` selection passed
8/8 in 29.321 seconds, including sixteen concurrent isolated database installations,
typed rejection recovery/exhaustion, fresh inventory after another creator, committed
lost acknowledgment, cancellation and acknowledged-install readback refusal through
the production loop. The changed native runtime compiled with `just check-package
pse-runtime --features pse-runtime/native-solvers,pse-runtime/canonical-tests`;
the original native `workflow::modeling::conformance::` selection passed 27/27 in
66.497 seconds on the first scheduler revision. Subsequent continuous slot reuse and
additional controls require their own execution. `python -m unittest
scripts.tests.test_native_tests.PythonSelectionTests` passed 3/3, including sixteen
overlapping pytest worker processes and exact controller inventory/JUnit identities.
All ran through `scripts/pse-env`, with native checks using the linked environment
and correctness Rust checks retaining relation force-validation. These focused
observations do not establish restarted reference conformance or assembled E3/E4/E5.

**Tested (2026-10-07 current continuation, baseline zero):** the serial local
`just unit-native-package pse-tests-conformance pse-tests-conformance/native-acceptance`
selection for flash, finite dynamic optimization, PR/PC-SAFT instability, native recovery,
physical NLP and vessel fitting completed 7/7: four passed and three failed. All nine flash
realizations, PR instability, native recovery and vessel fitting passed. The failed dynamic
assertion compared a finite Radau optimum to a continuous-control optimum; PC-SAFT retained
an unqualified restoration failure; physical admission exposed the missing selected-source
engineering rule. The current assertion and frontend repairs postdate that binary and need
their own execution. This completed diagnostic run does not establish E3.

**Tested (2026-10-07 frontend correction, baseline zero):** serial local
`just unit-native-package` controls passed 4/4 in `pse-math` for independent affine rows,
stale assumptions, cancellation, proof work and retained class evidence; 1/1 in
`pse-compiler` for opaque neighbors and original guarded execution; and 2/2 in
`pse-modeling` for selected engineering-rule constants and provenance. The native runtime
controls for explicit Ipopt Auto-versus-Off affine elimination, old-context refusal and
existing selected-growth bounds passed. The equality-bound nominal characterization
control passed its corrected standalone selection. All used explicit relation
force-validation and recipe-owned native environment; compiler free-value, constant
provenance, square-system and requested-path fixture premises were corrected before
the corresponding positive executions. A further `pse-modeling` selection passed 2/2
legal self-difference controls, including full quantity identity, origin-sensitive refusal,
positivity, uncertainty and value/type mismatch. The corrected canonical rule-reuse runtime
control passed 1/1, including admitted typed rules, exact marker absence/presence, source-span
invalidation and unrelated authoritative-row reuse. Affected scientific journeys remain
in execution. These controls do not establish E3.

**Interface-checked (2026-10-07 bounded independent source review):** the 14 changed
historical fixture files, shared engineering-rule markers, legal self-difference admission
and resolved physical-floor helper had no material findings. The review checked distinct
flow/density/basis and point/difference contracts, exact rule/unit identity, and remaining
fixed-value, synthetic and analytic-identity comparisons. It performed no scientific
execution and does not establish E3 or E5.

**Tested (2026-10-07 current scientific correction, baseline zero):** the serial local
`just unit-native-package pse-runtime pse-runtime/native-solvers,pse-runtime/canonical-tests`
selection completed 3/3 with explicit relation force-validation: MILP routing passed
(18.803 s), original PR production-basis derivative conditions passed (377.234 s), and
original recycle initialization failed (290.042 s). Recycle retained native Success,
Stationary qualification and feasible physical quality, but six original connection
closure checks refused its final candidate; no stage was committed. Closure-policy
composition is under source diagnosis. This is focused evidence, not E3.

### What was built

**Tested (2026-10-08 UTC, controlled Linux/glibc development artifacts, baseline zero):**
the actual imported-Python quiet receiver reopened the persisted original revision and
scientific outcome with exact historical rows, with zero fresh-retention hardware breakpoint
hits and unchanged executable bytes. Its deliberate miss reached that breakpoint with expected
exit 42. The actual managed-worker test ran the same authored case in two distinct fresh processes
without producer receipts: the local-profile positive passed 1/1 in 225.472 s, and its deliberate
miss control passed 1/1 in 15.387 s. Hardware observations verified file-backed executable mapped
bytes before receiving admission; they are diagnostic branch evidence, not timing samples.
Original probe artifacts and logs remain under `/tmp/plan28-python-replay-*-fixed.log` and
`/tmp/plan28-worker-*-test.log`, with tested artifact hashes at
`/tmp/plan28-actual-replay-artifacts.sha256`. These finite cases establish quiet default
reconstruction for their observed effective contexts. Concurrent loader changes may correctly
fall back to fresh preparation; no arbitrary model or deployment portability is claimed.

**Tested (2026-10-08 UTC, affected qualification repairs, baseline zero):**
`just check-package pse-math` and `just unit-package pse-math implicit` passed, with 27/27
implicit tests under explicit force-validation and unchanged resource limits. The route/build
script selection passed 104/104 after the runner-ownership correction;
`just unit-consolidation-tools` passed 82/82.
Actual pytest collection deselects the one strict association by default and collects it with
`--producer-deployment`. Actual pinned full-feature Nextest collection selects exactly the
ignored native association with its explicit opt-in; the same exact filter without the opt-in
selects none. The native runner owns its list/run pair and fresh selected inventory; the outer
collector does not execute a second enumeration. Full workspace all-targets Clippy under the native/canonical-test/
native-acceptance feature graph passed with zero warnings. The scoped static assessment at
`build/assessment/plan28-remaining-static-owned-20261008` completed 39/39 gates with zero
unsuccessful checks and unchanged source; transferred positives retain their original provenance.
It deliberately excludes full native Rust/Python suites and supplies no assembled E3 acceptance.
The current configured-profile `just ready` passed without warnings. Build edit-locality controls
are implemented and tested; their actual E4 measurements remain unmeasured.

**Implemented:** owner-generated canonical schemas/contracts/fixtures, target-only worker
and Python consumers, retirement of the PostgreSQL/Delta production mechanisms and crates,
scoped native capability setup, unchanged-output timestamp preservation and narrower
workspace-hack attachment. Actual DataFusion planning/predicate/provider consumers and
Arrow/Parquet scientific boundaries remain. Measurement fixtures drain their isolated owners.

**Tested (2026-10-06 current continuation, baseline zero):**
`just unit-native-package pse-operations
'pse-operations/canonical-tests,pse-operations/test-support'
'test(canonical_results_server_unit) | test(testing::canonical_server_unit)' --profile local`
passed 8/8, including awaited fixture removal after cancelled, panicking and failed
release. `just unit-native-package pse-runtime
'pse-runtime/native-solvers,pse-runtime/canonical-tests'` with the derived, multistart,
PETSc and two canonical durable-result selectors passed 37/37 in 376.665 seconds.
Both used the selected 32 GiB supervised profile, serial Nextest scheduling and explicit
relation force-validation. `.venv/bin/python -m unittest
scripts.tests.test_surreal_server scripts.tests.test_validation
scripts.tests.test_execution_contracts scripts.tests.test_producer_deployment
scripts.tests.test_native_tests` passed 108/108, including two real installed-pytest
collection controls in disposable directories. Supervisor and capture unit controls
use mocked process/build observations; they do not qualify actual server reconfiguration
or installed deployment association. The original PR/recycle selection failed 2/2:
PR stopped at a scratch limit before derivative qualification, and recycle retained an
infeasible tear candidate. PR's policy/resource-lifetime correction requires a rerun;
recycle diagnosis remains open. These initial resource corrections did not change production
accuracy. The later source correction activates the existing shared physical thresholds as
engineering floors; their numerical values remain unchanged.

**Tested (2026-10-06 current continuation, baseline zero):**
`just unit-native-package pse-runtime
'pse-runtime/native-solvers,pse-runtime/canonical-tests'
'test(modeling::forms_tests::) | test(modeling::global_tests::) |
test(pr_jacobian_tests::) | test(recycle_tests::)' --profile local`
completed 21/21 selected controls in 1,331.193 seconds: 19 passed and two failed.
The original PR case reached its Second derivative diagnostic, exposing the
cancellation-sensitive reversed-pair assertion corrected in current source. The
positive recycle failed original-stage qualification; the overheated refusal control
passed. The retained initial numerical convergence and subsequent infeasible recovery
are mechanisms within the same tear-stage attempt, not an accepted preceding stage.
The changed PR assertion and recycle pipeline still require positive execution.
`just unit-native-package pse-compiler pse-relations/force-validate
'test(scientific_costing_)' --profile local` passed 2/2, exercising the corrected
partial-package policy import. These runs used serial local scheduling and explicit
force-validation; the ordinary compiler launcher had first failed to load the
required licensed native environment and is not recorded as a pass.

**Tested (2026-10-06 current continuation, baseline zero):**
`just python-runner-unit-test` passed 2/2 under the default Nextest profile with
explicit relation force-validation. `just canonical-recovery-test --profile-state
<selected-state>` passed the isolated native journey using the selected 32 GiB joint
allocation (16 GiB server, two 8 GiB worker slots). It exercised acknowledged-write
SIGKILL/reopen, stopped/drained server-cap reconfiguration from 16 to 8 GiB and back,
observed actual cgroup memory caps, unchanged identity/credentials, exact retained
source/product bytes, large staging, and gated offline backup/restore. The selected
deployment itself was only read as the fixture's profile source. The comparison repairs
compile through `PSE_NEXTEST_ACTION=list just unit-native-package
pse-tests-conformance pse-tests-conformance/native-acceptance` with the complementarity,
dynamic-optimization and instability selectors; compilation is not a positive test result.
Affected scientific reruns and assembled E3 qualification remain pending.

**Tested (2026-10-06):** targeted engine cache-family controls passed 8/8 through
`just unit-package pse-engine 'test(cache_service::tests::)'` with explicit force-validation;
`just producer-identity-test` passed 31/31 controls for actual Cargo artifact association,
the cdylib manifest route and bounded package-family refresh. Native graph-selection tooling
controls passed 14/14. The baseline was zero failures. Earlier runtime/worker captures retain
their reviewed observed scope; replacement build outputs require current artifact association.
E3 assembled qualification, E4 measurements and E5 assessment are not established here.

**Tested (2026-10-06 completion-audit correction, baseline zero):** the native runtime
`just unit-native-package pse-runtime 'pse-runtime/native-solvers,pse-runtime/canonical-tests'`
selection for version admission, progress, supplier hints, analytical native paths, fitting
and grouped scalar preparation passed 26/26 under the default Nextest profile with the
shared store budget and explicit relation force-validation. The quantity-relations integration
target passed 7/7 through the native launcher. Compiler reference/policy and closure selections
passed after repairing their authored input premises; seven native backend analytical/version
controls passed. Worker runner prerequisite controls passed as part of the 58-control
`just unit-consolidation-tools` selection; frozen frame admission passed its golden-vector
control. Production accuracy defaults and original scientific assertion thresholds remain
unchanged. These focused results precede the final namespace/publication fixes and do not
qualify the assembled tree or replacement Python deployment.

**Tested (2026-10-06 final selected-acquisition correction, baseline zero):** the current initialization/selection/staging native selection passed 20/20; composed study lost-ack/cancellation and multipage result-read/retirement controls passed 4/4. The actual producer-fixture-backed runtime selection passed 22/22, including persisted scalar replay, exact target-root refusal, supplier closure and reopened secant continuation. An independent bounded implementation review identified logical lookup metadata allocated before reservation; the correction reserves before the RPC and releases before fallback acquisition. Its actual low-pool `selected_source` entrypoint and scalar closure/replay controls passed 2/2. `just unit-package pse-modeling 'all()'` passed 252/252 with explicit relation force-validation after correcting the inventory assertion to the actual index-set prefix. These controls establish their named current behaviors, not assembled qualification.

**Implemented:** the unconsumed Delta vendor override, regeneration seams and obsolete caching map are removed, alongside their owning REUSE annotation. Surviving DataFusion/Arrow capabilities keep actual consumers. Optional absent vendor directories are observed without refusing a valid fresh checkout; a newly added vendor file changes executable source identity. **Tested:** the affected buildinfo foundation selection passed 4/4. The scalar/pressure preparation benchmark compiles through `native_exec.sh cargo check -p pse-benches --bench modeling_preparation --locked --features pse-benches/native-process,pse-relations/force-validate`; its current namespace acquisition retains the pre-RPC budget correction. The result-chain selector is being completed as an actual long trajectory and paged result/analysis journey before measurement.

**Tested (2026-10-06 production sensitivity basis, baseline zero):** configured
`just parity` passed 9/9 against IDAES 2.13.0 under linked native solvers and explicit
force-validation. The default-profile native selection
`just unit-native-package pse-backend-native pse-backend-native/native-solvers
'test(kkt::sensitivity::tests) | test(kkt::tests) | test(restart_tests::)'`
passed 28/28 after repairing the restart comparison's forward-error premise.
`just unit-native-package pse-runtime pse-runtime/native-solvers,pse-runtime/canonical-tests
'test(sensitivities_published_with_local_validity) | test(sensitivity_withheld_without_local_analysis)'`
passed 2/2. The selected supervised Surreal state, bounded Nextest concurrency and
force-validation were retained. These controls exercise the production policy, actual
qualified candidates, frozen physical scales/allowances, regularity and refusal conditions,
and material native parameter response. Exact matrix/transport mechanics remain separate.
KKT stopping controls residuals; the restart's empirical decision quantity is the original
objective, rather than a promised forward-coordinate bound. No production tolerance was
tightened and no per-test accuracy exception was introduced. This is focused evidence,
not assembled E3 qualification or E4 measurement.

**Tested (2026-10-06 deployment and measurement handoff, baseline zero):** the linked
`just py-test -n 0 -m integration
python/pse/tests/test_canonical_results.py::test_canonical_eligible_deployment_receipt_reopens_original_scalar`
passed 1/1 on the selected supervised state with the eligible current Python producer
capture and independently observed loaded-extension run header. The test reopens the
original canonical revision and checks exact retained result bytes, including fixed
parameter bindings; its empirical scalar allowance comes from the actual published
production tolerance. Earlier runs exposed a missing linked-library environment and
incorrect test assumptions about parameter rows and document re-import identity; these
were corrected before the positive rerun. The actual worker/Python captures have the
same outer attestation; exact Maturin editable RPATH replay reproduces the installed
binary byte for byte. Tooling controls passed 54/54 through
`.venv/bin/python -m unittest scripts.tests.test_validation scripts.tests.test_execution_contracts -q`,
including admission of the exact assembled native gate and refusal of arbitrary
dependency/selection edits. Full E3 and E4 remain pending.

### A mistake made and corrected

Local replay admission originally preceded mathematical initialization. License startup then
loaded NSS code and invalidated that admission, so an actual Python receiver silently took fresh
preparation. Initialization now precedes common local observation, outside the loader lock;
the corrected quiet receiver and deliberate miss distinguish both branches without modifying
executable bytes. First-order implicit admission also reserved triangular second-order factor
scratch. It now reserves only its compiled order, preserving the original resource ceilings.

The composed native assessment was launched while reference conformance was still writing to
the same physical canonical server. Sixty completed fixtures failed during initialization with
typed transaction conflicts; the run was interrupted and retained as nonqualifying. Sequenced
qualification must determine whether a production initialization correction is also necessary.
The optional strict route initially requested outer enumeration although its native runner
already owns list/run. That would execute the strict test during enumeration and lose selected
accounting. The corrected route consumes one runner invocation and its fresh inventory; a new
outer-collector control also confirms that synthetic inventory alone cannot qualify science.

Package-wide numerical meaning was missing from both bounded source acquisition and the
subsequent checked-package projection. Seeing constants in historical output checks did
not establish production rule admission. Both frontiers now use one typed marker classifier;
actual rule identity, complete quantity, provenance and reuse invalidation are tested at
the consumed boundary.

Conventional library capture did not model Maturin's actual Cargo rustc selection or editable
RPATH transformation. The tool now selects the observed manifest/lib route; installed-artifact
association also accounts for the source-backed metadata transformation. Cargo's cleanup
selectors operate on package-name families, so refresh now states that real boundary and
retains successful cleanup diagnostics rather than claiming exact-version cleanup.

### Deviations from the plan, deliberate

Retain DataFusion only for its actual native relational consumers; persistence and public
queries use the canonical substrate. Producer capture uses its separate inheriting profile
and direct owning launcher without moving the checkout target directory. No measured speedup
or full-series acceptance follows from these structural and focused observations.


**Tested (2026-10-07 fixed-bound recovery, baseline zero):**
`NEXTEST_TEST_THREADS=1 just unit-native-package pse-backend-native pse-backend-native/native-solvers 'test(fixed_reactions)' --profile local`
passed 4/4 controls (537 skipped), including actual native Ipopt original KKT qualification.
Manufactured controls exercise signs, maximization, normalization, disabled preprocessing,
missing/invalid duals, unchanged free-coordinate residuals and derivative refusal.
This focused result does not establish the affected PC-SAFT regression or assembled E3.

**Tested (2026-10-07 affected conformance selection, baseline zero):** the local native
selection of physical NLP, PC-SAFT, finite dynamic QP and discrete dynamic MIQP completed
4 tests: 2 passed and 2 failed (63 skipped). Both finite dynamic controls passed. Physical
NLP failed its preparation policy-admission assertion; PC-SAFT reached native success but
failed original stationarity because fixed-coordinate reactions were absent. These failures
drive the closure-target and fixed-bound recovery corrections, respectively. E3 remains open.


**Tested (2026-10-07 closure intent, baseline zero):**
`NEXTEST_TEST_THREADS=1 just unit-native-package pse-modeling pse-relations/force-validate 'test(process_contract_contextual_closure_)' --profile local`
passed 6/6 controls (261 skipped) after repairing two new fixture setup errors: provenance
injection spelling and a material state missing its required transport observation. The
controls retain both endpoints, distinguish marked rules from identical explicit literals,
preserve accumulator/reconstruction/inventory intent and refuse missing or invalid resolved
assessment budgets. No production admission was weakened for those fixture repairs.

**Implemented:** the authored recycle study now supplies optional frozen transport
characteristics from feed capacity, its existing purge and tear fraction, and per-species
liquid/vapor enthalpy differences over the shared property validity interval. The largest
within-species endpoint difference includes phase change without using a solved composition
or a datum-dependent enthalpy magnitude. Actual transported symbols receive the existing
reference-difference annotations; disabled context emits no scale. Actual contextual
preparation is positive; native scientific execution remains pending.

**Interface-checked:** bounded independent source reviews found no material defect in
fixed-bound reaction recovery or the authored recycle characteristics. They performed no
execution and establish neither the affected scientific regressions nor assembled E5.


**Tested (2026-10-07 contextual runtime controls, baseline zero):** the corrected local
native `contextual_closure_` selection passed 5/5 synthetic controls. A sixth actual
reference-preparation control failed before solving: multiplying Flow by a molar difference
produced a Power point under the authoritative physical multiplication contract, while the
scale requires a Power difference. The authored supplier now forms the difference of two
same-datum Power points through one reusable helper; no physical admission rule is relaxed.
That receipt predates the additional acquisition and indexed-target corrections below.

**Interface-checked:** bounded independent closure integration review identified two
material defects. Direct relative requirements could be added again after endpoint-budget
composition, and contextual inventory finalization could overwrite an explicit caller's
quadrature controls. Automatic fitting also dropped the factory's provisional inventory
identities. The corrected common-resolver composition and factory-origin tracking now pass
their focused regressions, including automatic and explicit fitting profiles. No new
assembled acceptance claim follows from these controls.

**Tested (2026-10-07 repaired contextual integration, baseline zero):**
`NEXTEST_TEST_THREADS=1 just unit-native-package pse-runtime pse-runtime/native-solvers,pse-runtime/canonical-tests 'test(contextual_closure_) | test(reference_recycle_context_reaches_original_transport_obligations)' --profile local`
passed 9/9 (652 skipped) in 278.873 seconds. The full reference preparation resolves all
20 original energy connection closures from separately admitted endpoint context, full
quantity/unit identity, original scale provenance and the tighter endpoint budget.
The preceding attempt passed seven synthetic controls but failed fitting acquisition and
actual reference specialization. Their owners now load missing import-owner metadata before
inspecting package rules and retain the named owner of an indexed computed annotation target
before rewriting its expression. Bounded independent source review found no material defect
in either repair. The modeling all-target compile check passed without warnings or errors.
These are preparation and control-origin results, not native recycle or E3 acceptance.

**Tested (2026-10-07 native recycle, baseline zero):**
`NEXTEST_TEST_THREADS=1 just unit-native-package pse-runtime pse-runtime/native-solvers,pse-runtime/canonical-tests 'test(reference_recycle_initialization_)' --profile local`
passed 2/2 (659 skipped) in 716.619 seconds: the original ordinary study satisfies its
scientific obligations, and the overheated initialization refuses without committing.
The earlier original closure refusal is corrected through typed shared policy and frozen
context; neither original acceptance nor explicit physical tolerances were bypassed.
This affected scientific pass remains distinct from the pending assembled E3.

**Tested (2026-10-07 process contracts, baseline zero):**
`NEXTEST_TEST_THREADS=1 just unit-native-package pse-modeling pse-relations/force-validate 'test(process_contract_)' --profile local`
passed 33/33 (235 skipped), including the new indexed computed annotation-owner regression.
Its first two fixture versions incorrectly used an Integer coordinate as a physical operand
and a numerically untyped membership selector; the corrected fixture uses declared entity
members and retains the computed Power expression. The existing 32 process contracts remained
positive throughout those fixture corrections.

**Interface-checked:** the bounded E1/E2 source assessment found no remaining displaced
PostgreSQL/Delta production implementation or retired public SQL convenience consumer in its
inspected scope. Remaining DataFusion/object-store consumers supply real relational and data
boundary operations. One pure-generation check unnecessarily initialized native solvers;
`codegen-python-check` now follows the existing plain-Cargo contract-generation route.
Scope-end generation and assembled qualification remain separate execution obligations.

**Tested (2026-10-07 scope-end tooling, baseline zero):** `just codegen` completed
through its owners. A Python document generator emitted separate Literal alternatives
that safe lint fixes merged, causing avoidable regenerated-output drift; the generator now
emits the merged alternatives itself. `just unit-package pse-codegen 'test(python_documents_)'`
passed 4/4 (69 skipped), and regeneration retained the unchanged workspace hack.
`just hygiene` executed all 23 recipes: its first pass failed Python lint, Python typing
and both Clippy modes. After owner fixes, `just lint-py`, `just typecheck`,
`just clippy-default` and `just clippy-no-default` each passed. The repaired composite is
not a clean initial hygiene pass. The selected pure tooling command
`.venv/bin/python -m unittest scripts.tests.test_native_tests scripts.tests.test_producer_deployment scripts.tests.test_surreal_server scripts.tests.test_validation`
passed 89/89, including producer refusal/association, invocation selection, supervisor
resource policy and receipt reuse/transfer controls. The final `just turn-end` completed
with no remaining Python lint findings or formatting changes. These checks do not qualify
fresh installed artifacts or the assembled scientific campaign.

**Tested (2026-10-07 affected native conformance, baseline zero):**
`NEXTEST_TEST_THREADS=1 just unit-native-package pse-tests-conformance pse-tests-conformance/native-acceptance 'test(authored_physical_nlp_preserves_native_routes_and_original_qualification) | test(pcsaft_tpd_fits_the_formal_pool)' --profile local`
passed 2/2 (65 skipped) in 1,443.403 seconds. The physical NLP control exercises the
authored multi-model native routes and original qualification; PC-SAFT retains original
qualification after eliminated fixed-variable reaction recovery. These repaired affected
controls do not establish assembled E3. `just lint-native-contracts` and
`just lint-native-data` also passed; local feature-combination execution remains pending.

**Implemented:** the local Nextest whole-campaign bound is four hours. The retained
assembled native run consumed 5,566 seconds while failed scientific cases terminated
early; replacing those cases with current complete positive journeys projects beyond
the previous two-hour bound under serialized server ownership. Four hours supplies
finite orchestration headroom, without changing production job deadlines, retry policy
or scientific admission. This scheduling estimate is not a measured current full-run
duration or assembled qualification.

**Tested (2026-10-07 worker journeys, baseline zero):**
`NEXTEST_TEST_THREADS=1 just worker-test --profile local` passed 5/5 in 100.545
seconds, against the selected managed substrate. Authored execution, package/data
round-trip, killed-worker recovery, cross-process cancellation and durable parallel
study dispatch retain their original obligations. This recipe builds the debug worker;
actual producer deployment association is a separately selectable strict qualification scope.

**Tested (2026-10-07 local parity, baseline zero):** `just parity` passed 9/9 in
134.10 seconds, including the linked preflight, covariance, degeneracy, PID and sensitivity
comparisons against the pinned IDAES environment. The recipe installed a development
extension. The development-phase policy above removes mandatory producer-profile reinstall,
capture and import association. This parity scope does not establish the assembled scientific campaign.
