---
title: Rebuild, retirement and integrated qualification
status: in-progress
date: 2026-10-05
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md, docs/design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md, docs/design_review/reviews/design_review_plan-28-completion_2026-10-06.md, docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md, docs/design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md]
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
[checkpoint](27-contextual-engineering-accuracy.md#current-checkpoint).
[25k](25k-integrated-qualification-and-closure.md#current-execution-checkpoint) still has
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
tolerance-limited nested solve are being replaced by checks of the defining implicit
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

The diagnostic assembled native and installed Python runs have finished; source repairs
and their targeted validation remain in progress. The Python study admission lost transport
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
`just assessment <output>` with its development linked Python, `just seed-conformance` for the current reference manifest, and
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
Select the admitted isolated RocksDB/gRPC profile: the prepared 32 GiB allocation is a 16 GiB
server plus two 8 GiB managed worker slots. Record actual configured/used conditions, not merely
the proposed allocation. No uncontrolled concurrent heavy campaign should compete with it.
Exercise A/C/D's composed read/retirement/root handoff, committed lost-ack claim recovery,
worker interruption/drain, ordinary retained failure/partial/cancellation and actual gRPC
completion/export refusal. Abrupt acknowledged-write kill/reopen and quiesced offline
backup/restore must validate exact IDs, interpretation and replay in that same supported
profile. Logical export/import alone cannot establish these guarantees.

Require zero failures and complete selected/executed accounting for the selected functional
scope. Record the actual code/artifacts exercised. Use the runner's declared input families; unrelated
concurrent prose edits remain contextual observations. A report's terminal-complete flag does not establish those conditions. If source
changes or a required scope fails, repair its owner and rerun affected qualification on the
stable final state; identify any repaired composite honestly. Do not introduce per-test accuracy
exceptions or drop workloads to manufacture acceptance. Record exclusions and actual conditions
in the Outcome rather than accumulating per-command checkpoint logs.

### E4 measurements after positive E3

Retain the 22 transferred process/preparation selectors and extend measurement coverage where
the confirmed new consumer scope is not exercised by them. Run the applicable selectors through
`just case-measure <new-output> --functional-from <qualified-assessment>` with explicit
`--case` selections and their current
`.config/process-cases.json` / `.config/preparation-cases.json` owners. Each selector needs
positive corresponding functional evidence, the declared workload/profile and fresh-process
samples; unsupported scope needs an explicit owner/disposition, not a silently omitted case.
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
This is 25 selected cases rather than the whole case registry.

The publication cases separate physical stage/reopen, document admission, modeling
publication, simulation preparation/execution, result reading and analysis activation.
The durable study case exercises the private worker physical-source and selected admission
route; public publication uses protected source reopening followed by one public document
admission. Standalone fitting/shooting remain one-shot consumers rather than a manufactured
repeat session. Empty/skewed selection and interrupted recovery have correctness controls;
these cases do not measure their timing. Authored unchanged-byte edit sharing has functional
controls but no isolated timing claim. Native installation and deployment admission use
complete-operation observations from the existing build and assessment routes, whose Cargo
sample durations alone do not include setup.

Reconcile added coverage explicitly rather than silently rerun the whole registry or omit a
new mechanism. N4/T5/L4 supply affected workload conditions and existing case mapping. Add
recipe-owned cases only for gaps: policy/preflight/trajectory projection growth; unchanged
selected admission and repeated fit/shooting owners; physical-document stage/reopen and authored
edit copying; narrow/wide result append/read and analysis activation. Actual native setup and
artifact-edit locality use the existing build measurement routes, extended at their owner if
phase timing currently excludes setup/admission. Keep benchmark declarations in their existing
registry/script owners; no parallel measurement service or per-helper benchmark quota.

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

## Work packages

| Package | Prerequisite and delivered behavior | Completion boundary | Status |
|---|---|---|---|
| E1 — Controlled target regeneration | R0; A2 schema/selection and B1 admission slice. Replace storage-specific fixture setup, regenerate scientific source corpus and run configurations; add recipe-owned target DB setup. | Target inputs are admitted with exact interpretation; old test setup does not silently start PG/Delta. No migration importer or compatibility fixture service remains. | Implemented; focused declaration/schema and linked scientific/generated consumers passed. |
| E2 — Retirement and build locality | Joins each migrated A/B/C/D consumer as it becomes ready; tooling corrections can begin after R0 independently. Remove displaced crates/codegen/dependencies/recipes and implement scoped native setup/unchanged generation. | All target consumers work; remaining dependency consumers are explicit and justified; replaced mechanisms/tests/fixtures are gone. This does not postpone package-local deletions. | Implemented; focused native and linked consumers passed; earlier runtime/worker producer qualification is scoped; current replacement artifact association and installed Python admission remain. |
| E3 — Assembled correctness and recovery | All functional A/B/C/D, adopted N/T/L scope with N4/T5/L4 reconciliation, B4's bounded decision and E1/E2 integration complete. Run one selected static/integration/scientific campaign against regenerated target. | Zero failures in named required scope; explain unsupported scientific/provider limits, repaired composite runs and actual crash/durability conditions. | Attempted; interrupted/nonqualifying; expanded functional handoff precedes fresh E3. |
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

The enhancement review is now scheduled on `dacc9c3`; it does not change product code or qualify
the target. The stable local assessment retained 37 passed entries (including reviewed transfers),
three failures and one incomplete Python entry against zero. It has no terminal Python traceback
or final reconciliation after interruption. Current next work is L6/affected diagnostics followed
by complete C5/B5/N5–N7/T6/L5 functional scope, then composed E3, applicable E4 and E5. Earlier
handoffs below retain their original evidence and do not mandate strict recapture/rebuilding.

### Efficiency review handoff, 2026-10-07

The maintainer redirected execution to the [production execution efficiency review](../design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md)
because of the long native run, then selected stopping and preserving the owned runner.
The selected SurrealDB server, resource profile and implementation remain preserved.
Commit `5260a3e9a` subsequently recorded that implementation and this handoff during review
publication; the reviewed production, configuration and test bytes remained unchanged.
E3 is interrupted and incomplete; E4 measurements and E5 assembled acceptance remain pending.
The review is a separate design assessment, not the E5 completion review.

**Tested, partial preserved campaign; baseline zero:**
`build/assessment/plan28-qualified-resumed-20261007/checks.json` records 39 positive gates,
one interrupted native gate and one not-run full Python gate. Thirty-seven positive gates
were explicitly transferred from the reviewed prior evidence; current runtime/worker producer
deployment and imported-Python association were executed against their actual captured artifacts.
The native command `just native-test --profile local --config-file <run>/native-test-nextest.toml`
selected 2,821 tests in native force-validation mode: 2,425 passed, one was aborted by the
requested SIGTERM, and 395 did not run. That abort is not a discovered scientific failure.
The report records unchanged source during execution. Its 1,793.689-second native-gate duration
includes the logged 4 minute 11 second build; individual test times include fixture and teardown.
The review's [preserved execution evidence](../design_review/evidence/production-execution-efficiency-2026-10-07/execution-baseline.md)
states the attribution limits. These are incomplete campaign observations, not a full pass or
new production performance qualification.

The subsequent [repository-wide plan extension](28-surrealdb-unified-substrate.md#repository-wide-efficiency-extension)
now schedules review findings, related variants and confirmed RC01 through 28f/28g/28h.
Production remedies, accuracy changes and qualification resumption were not performed by
the review or plan authoring. After the expanded functional handoff, resume affected correctness/deployment journeys and
E4/E5 under this existing owner. Keep valid earlier evidence at its original scope; a capture-only
repair does not itself justify repeating the completed series. The earlier continuation notes
below describe the work leading to this handoff.

The maintainer committed and pushed the correction baseline as `219338742` before this
continuation. Execution implements the bounded refinements agreed in the plan-execution
discussion: explicit awaited fixture finalization, stopped/drained resource-only supervisor
reconfiguration, and ordered producer capture and imported Python association after
installation. Ordinary scientific controls retain production accuracy; original PR,
recycle and conformance journeys precede the stable deployment handoff.
The derived/multistart/PETSc and durable-runtime selection is now positive. The PR
diagnostic dropped the admitted compiler policy in its independent primitive checks;
the correction restores that policy and keeps diagnostic admission through native
teardown and returned-buffer lifetime. The current original recycle now retains a feasible stationary native candidate without
commit because original connection closures still refuse it. The refused checks are raw Power/W differences of transported `flow*h`; their
folded 1 W allowances are not guaranteed by primitive-field native feasibility.
Source diagnosis found that lowering erased the distinction between a marked shared rule
and an explicit tolerance, and original closure assessment bypassed contextual resolution.
The correction preserves both endpoint obligations, resolves their full physical quantities
through the production policy, and retains the tighter endpoint budget. Meaningful frozen
engineering context is authored separately from solved trial values; explicit literals remain
explicit. Native success remains insufficient for original physical acceptance. Bounded runner review exposed option values being
mistaken for Python test selectors and deployment prerequisites being retained without
current artifact paths/input coverage. The wrappers now separate positional selectors;
deployment capture and imported association require fresh execution, while retained
fixture consumers use their verified origin. Pytest owns positional/default selection
through its `testpaths` setting; both wrappers remove their bespoke option parsers.
Reviewed declaration/list files enter
the deployment input identity. Conformance qualification also explicitly checks requested
native route identity and original-feasible stationary PC-SAFT outcomes rather than a
scalar alone. Its pressure-consistent intermediate solve preserves the original final
free composition; qualified negative local tangent-plane distance retains scientific
model-check refusal, rather than a false zero/stability premise.
The maintainer clarified that historical scientific output comparisons must tolerate
analytically insignificant variation. Empirical comparisons now consume shared authored
physical resolutions or their authored expectation checks; variable-bound feasibility
allowances do not establish forward output accuracy. Discretized optimizers retain
original physical checks and production optimality evidence, without asserting unrequested
exact optimizer coordinates. The correction aligns 236 historical expectations across 14
authored seed models with that basis, preserving their values and equations. Partial fixture
loaders now include the policy package. Ordinary Python candidate comparisons and parity
parameter estimates consume their execution policy; exact identity, storage, fixed-input
and mathematical-mechanism checks keep their distinct obligations. Source diagnosis also
found that the shared physical constants had no package-level engineering-rule markers:
output expectations used the physical floors, while production admission could still fall
back to 0.001 canonical units, including Pa. The 16 existing constants now declare those
markers, with no value changes or second policy. The actual physical admission control
then exposed a second defect: selected package acquisition omits anonymous engineering-rule
annotations, so the markers failed to reach the solver. The bounded package frontier
now acquires these semantic dependencies and their referenced constants,
without hydrating unrelated package members. Physical admission controls require the actual
temperature, density, pressure and power rules on the original solver targets.
The acquired markers were also being dropped by the checked-package projection; both
frontiers now retain the same typed package marker and its constant/provenance closure.
Actual source admission now reaches the shared rules. The legal self-difference guard
now admits additive `ComponentFlow` by its complete resolved quantity identity while
refusing origin-sensitive temperature and pressure points. The canonical control distinguishes
unchanged authoritative rows from parsed package edits: reparsing changes source provenance
and correctly invalidates exact reuse, even when the selected extent stays bounded. Marker
add/remove and unrelated typed-row extension preserve existing rows to isolate dependency-local
reuse and absence evidence. Their actual runtime validation is positive.
Canonical modeling context v2 explicitly refuses reopening revisions staged under the
older marker metadata; restaging the authored source supplies the current interpretation.
The store schema and retained result codec are unchanged. This clean-rebuild cutover has
one reader and does not reinterpret old selected-source dependencies as current ones.
The PR Second diagnostic now checks the canonical linear systems production assembles,
exact mirrored output and separately expanded defining equations with all contraction
terms in the backward-error denominator. It retains the production budget rather than
treating a differently re-summed, nearly cancelled RHS as an actual solved linear system.
The current production-basis PR execution is positive. Contextual closure preparation,
including actual reference transport obligations, is now positive; the affected native
recycle and conformance executions follow that handoff. The isolated selected-profile recovery journey is
positive, including drained resource reconfiguration and observed kernel memory caps.
The ongoing conformance selection passed all nine flash realizations, then exposed
another invalid continuous-optimum comparison in the finite Radau QP. Its correction
checks the actual submitted objective, original physical equations and retained
optimality evidence; the independent continuous integration checks remain. Flash
comparisons now read the resolved shared rule's physical floor with full quantity,
unit and rule identity checks, avoiding repeated scalar preparation. The penalty
route executes the same preparation whose structural requirement was inspected.
Bounded original-check values and closure inventory are retained in each refused
strategy cause before a later recovery candidate can replace it. The completed affected conformance selection passed both finite dynamic optimization
controls. Physical NLP stopped during preparation because its Power rule had no admitted
closure target; PC-SAFT reached native success in six iterations but original KKT refused
missing reactions on two eliminated fixed-temperature coordinates. These source-grounded
corrections precede another affected scientific execution and do not establish assembled
qualification.
The retained PC-SAFT restoration failure drives two further frontend corrections: nominal
row characterization must preserve equality-bound coordinates (280 K must not become a
fallback 1 K), and requested affine preprocessing needs independent row evidence even
when an explicitly selected nonlinear adapter does not request whole-problem coefficient
classification. Original feasibility, scientific checks and KKT remain required; the current PC-SAFT execution now provides native convergence evidence, while original
KKT qualification remains under correction. The pinned affine wrapper omits reactions for
eliminated fixed columns. Recovery now completes only those declared equality-bound
reactions in original physical units, using the existing Lagrangian derivative owner and
counted callbacks; it changes neither primal values nor surviving-coordinate multipliers.
Earlier successful capture/import controls retain their original artifact scope and do not
qualify the new binaries. E3 remains the sole assembled campaign, followed by all 22 E4
selectors and E5 acceptance. No old-storage or distinct CI campaign is added.

E1/E2 are active under the full-scope 2026-10-06 authorization. The native registry,
scientific physical fixture, public contract documents, Python contracts and exact native
schema have been regenerated through their owners. Pure contract generation avoids solver
discovery and extension execution. Unchanged output bytes retain their timestamps.

PostgreSQL operations, generated SQL/COPY/query output, Delta catalog/publication/settlement,
legacy artifact descriptors and physical layout adapters are removed. Their callers now use
canonical revisions/results and native engine ownership. The workspace hack and dependency
resolution have been regenerated. DataFusion remains for native relational preparation,
row predicates and optimizer/provider contracts; Arrow/Parquet remain typed scientific data
boundaries. Public SQL convenience and unconsumed Delta dependencies are retired. The orphaned
publication-only protobuf diagnostic codec and its DataFusion protobuf dependency are
removed; native row, ownership and receiving-session controls keep their useful assertions.
The workspace hack now attaches only to actual data-boundary and store consumers;
normal semantic, numerical and native-ABI roots no longer import the data stack through
feature stabilization. Supported nightly feature unification and the remaining consumers'
fallback preserve the shared type universe. Modeling, math and compiler library checks
compile this narrower graph; correctness tests continue to select the actual relation
owner explicitly for force-validation.

Worker, Python, engine-fixture and measurement consumers have moved to the target. Their
focused native boundaries, worker journeys and process benchmarks compile; native row,
ownership, pushdown, schema and declaration controls pass. Linked Python scientific controls
now pass for the process/recycle, dynamic/transient fitting, limited-incumbent and memory-stop
cases after separating canonical source IPC admission from the smaller result-block limit.
Generated-contract consumers use canonical manifest and scalar-index rows. Bounded exact study
source admission and IPC receipt reuse now pass their actual ingress, readiness, invalidation,
retirement and early/late cache-clear controls. Active selections remain protected through
run and analysis root admission. Escaped decoded Arrow arrays survive result reclamation and
release their allocation only on final drop. Qualified Python deployment receipt admission
remains before the functional handoff. Explicit Python test targets no longer also collect the entire test directory,
and recipes preserve each argument, including compound marker expressions.
Measurement fixture owners
now register isolated databases and explicitly drain result-release tasks before teardown,
preventing sample-to-sample storage accumulation. Scientific input values and independent
numerical assertions are preserved. No old fixture publication service is started by pytest.

Earlier actual runtime library and enclosing worker qualification produced eligible receipts without
refusal reasons under the reviewed producer profile. Current Python capture, import association
and public deployment admission remain pending. Captures distinguish compiler unit dep-info
from Cargo aggregate rerun hints, select a real library or binary target, and keep reviewed
outer build provenance separate from the scientific key. Source review and native/tool
closure remain required for the retained worker/Python profiles; successful fixture controls
do not grant production eligibility.

The implementation checkpoint is preserved in commit `06302af77`. The checkout's build
outputs were subsequently replaced outside this execution team. Earlier captures retain
their observed scope; they do not establish freshness or installed-artifact association for
the replacement outputs. Current deployment qualification must rebuild and associate actual
selected outputs under the retained reviewed premises. A separately launched native
assessment is using the checkout's Cargo lock; its results and prerequisites must be
reconciled before adopting any enclosing qualification claim. That independent assessment
finished with a Revise verdict and interrupted failing native/Python diagnostics against
the original commit; neither run qualifies this newer tree.

The actual editable Python link exposed a deployment association gap: Maturin uses its
Cargo rustc manifest/lib route, supplies the extension-module context and patches editable
library search paths. A conventional library build's output alias is insufficient to qualify
that installed artifact. The capture tool now selects the observed cdylib route; targeted
actual-Cargo controls pass. Python-only packaging inputs and immutable compiled/installed
association are being reconciled before the public deployment control. Executor refresh
also now uses bounded package-name families, matching Cargo's real cleanup semantics, while
retaining exact selected IDs and fresh-build observations in evidence. Final source cleanup
will require current root captures rather than relabeling these earlier immutable receipts.

Scope-end generation, formatting and hygiene ran after the initial functional handoff.
Hygiene exposed five failing recipes; instructions, spelling and Python checks were repaired,
and both Clippy variants subsequently passed. The independent completion audit exposed
additional functional corrections, so the assembled target campaign is held while those
corrections receive targeted controls. E4 measurements and final E5 acceptance/closure remain
pending. The previous A/B integrated testing is not being resumed.

The completion audit's F01–F05 dispositions live in the coordinator: F01 lifecycle-root ownership and F03
initialization completion have positive native correction controls; F02 grouped selected
preparation has positive guarded payload/frontier controls, including completed bounded namespace metadata
and identical concurrent publication settlement controls; F04
runner prerequisites and scientific fixture/request premises are being corrected with targeted
controls. Its numerical observations do not establish a derivative defect. F05 remains open
for current installed Python association, assembled qualification, measurements and closure.
The dated review retains its original commit and failed/interrupted evidence.

The assembled campaign's isolated server profile now uses a 16 GiB server and two
8 GiB managed worker slots within a 32 GiB allocation, following the maintainer's workstation
capacity correction. The previous 2 GiB server OOM observations retain their original scope;
they do not establish behavior under this larger allocation. The supervisor derives a 4 GiB
RocksDB cache and 12 GiB tracked-memory threshold from the new server allocation. This preserves the authored SCIP
interruption workload's existing worker admission rather than reducing its scientific
requirements to fit the smaller mechanism-test profile. Schema initialization completed through `just canonical-init`; assembled execution follows
the final functional handoff. Old application states are preserved.

Ordinary workflow fixtures now retain an explicitly owned temporary database until their
last store borrower is released; ordinary connections never acquire automatic database
ownership. Reliable reader finalization continues to use the existing awaited
`remove_isolated_fixture()` after returned read owners are dropped. Automatic cleanup is a
fallback: cancellation or cleanup failure in a detached reader task can retain the fixture
without failing the main test body. This does not establish guaranteed cleanup after abrupt
reader shutdown or explain the historical server OOM observations.

The ordinary horizon controls now assess original row and variable feasibility plus the
production stationarity and complementarity conditions. Their independent convex tracking
comparison follows the bound composed from those admitted conditions; the raw Objective
allowance is not treated as a certified optimum-gap promise. Shooting publication uses
the compiler's typed member-output identities. These corrections preserve production
tolerances. The affected contextual preparation, native recycle, PC-SAFT and physical NLP
controls are now positive, as are the current worker journeys and local parity. Remaining
execution is fresh actual producer captures and installed Python association, then the stable assembled
E3 campaign and current seed conformance. E4's 22 selectors and build measurements follow
positive E3; final independent E5 acceptance, finding reconciliation and decision/design
adoption remain afterward. The current scope-end checks are a repaired composite, recorded
in the Outcome, and do not replace those remaining obligations.

## Outcome (recorded after implementation)

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
