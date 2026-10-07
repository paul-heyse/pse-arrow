---
title: Rebuild, retirement and integrated qualification
status: in-progress
date: 2026-10-05
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md, docs/design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md, docs/design_review/reviews/design_review_plan-28-completion_2026-10-06.md]
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
changes. The original 1,000-point journey must complete under the same capped profile;
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
Existing source and
deployment captures precede these repairs; E3 requires fresh captures and an unchanged,
zero-failure assembled rerun before E4 measurements or E5 closure.

The maintainer confirmed local validation is sufficient; no separate CI campaign is
required. Local native runs use the `local` Nextest profile, retaining resource groups,
zero retries and the finite whole-run bound while allowing the production task to report
its own deadline. A diagnostic recycle run stopped by the earlier 360-second harness
cutoff did not exercise its declared 600-second production limit and does not establish
a native convergence failure.

Refresh and associate actual runtime, worker and installed Python artifacts under B3's supported
producer profile before persisted reuse acceptance. The installed extension and previous
captures predate these repairs, so their refresh remains a real deployment prerequisite.
Set up the selected initialized
supervised state and eligible inspection publication through the owning recipes. A fixture
receipt, old installed binary or current git HEAD is insufficient. Reconcile manifest runner
prerequisites and obsolete recipe references at their owners; no new evidence wrapper is needed.

### E3 correctness, deployment and recovery

The audit's native attempt selected 2,741 tests; 2,421 completed (2,342 passed, 78 failed,
1 timed out), 2 interrupted and 320 not run. Source changed during the run. Linked Python
selected 199: 156 passed, 4 failed and 39 did not complete; producer admission, event queue,
version precedence and wire ID premises require current targeted evidence. Those runs were
interrupted and overlapped; neither qualifies the final tree or supplies comparable timings.
Old pre-pivot Plan 25/27 passes and three smoke cases do not replace this campaign.

Use `just assessment-list` to confirm the current gate surface, then the selected assembled
`just assessment <output> --python-profile producer` with its refreshed linked Python
and inspection prerequisites, `just seed-conformance` for the current reference manifest, and
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

Require zero failures, complete selected/executed accounting, and unchanged relevant
product-input/deployment association. Use the runner's declared input families; unrelated
concurrent prose edits remain contextual observations. A report's terminal-complete flag does not establish those conditions. If source
changes or a required scope fails, repair its owner and rerun affected qualification on the
stable final state; identify any repaired composite honestly. Do not introduce per-test accuracy
exceptions or drop workloads to manufacture acceptance. Record exclusions and actual conditions
in the Outcome rather than accumulating per-command checkpoint logs.

### E4 measurements after positive E3

Run the 22 applicable process/preparation selectors through
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
and `results-chain-1`: 22 selected cases rather than the whole case registry.

Use the existing `just build-frontend`, `just build-uncached`, `just build-cache-probe`
and `just build-storage` routes as applicable for default/native closure and turnaround.
Do not clean the whole cache, share target directories or compare concurrent/interrupted runs.
Where no comparable pre-pivot baseline survives, report target-only timings and structural
removal with no speedup claim. INLINE or batching hypotheses require complete-operation
measurement before claiming a gain. No invented latency threshold or new instrumentation
framework is required; raw samples and honest limits accompany **Measured** claims.

### E5 acceptance and retirement after E3/E4

Conduct the binding's bounded independent assembled review on the demonstrated final scope.
Reconcile US/EF/F dispositions at the coordinator and original scientific findings at their
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
| E3 — Assembled correctness and recovery | All functional A/B/C/D, B4's bounded decision and E1/E2 scope complete. Run one selected static/integration/scientific campaign against regenerated target. | Zero failures in named required scope; explain unsupported scientific/provider limits, repaired composite runs and actual crash/durability conditions. | Attempted; interrupted/nonqualifying; current correction handoff precedes fresh E3. |
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
tolerances; assembled qualification and the remaining scientific reruns are still pending.

## Outcome (recorded after implementation)

### What was built

**Implemented:** owner-generated canonical schemas/contracts/fixtures, target-only worker
and Python consumers, retirement of the PostgreSQL/Delta production mechanisms and crates,
scoped native capability setup, unchanged-output timestamp preservation and narrower
workspace-hack attachment. Actual DataFusion planning/predicate/provider consumers and
Arrow/Parquet scientific boundaries remain. Measurement fixtures drain their isolated owners.

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
