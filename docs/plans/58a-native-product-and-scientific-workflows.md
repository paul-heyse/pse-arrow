---
title: Native product and scientific workflows
status: done
date: 2026-10-28
adrs: []
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, architecture, libraries, validation]
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md, docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md, docs/functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md]
---

# Native product and scientific workflows

A01–A07 are complete for the selected native scope. The
[Outcome](#outcome-recorded-after-implementation) owns final acceptance, corrections and the
handoff to H; earlier checkpoints retain their historical conditions. Remaining F/G consumers,
paused B/C/D and full H qualification keep their separate owners.

This supporting plan develops the scientific application target of [Plan58](58-rust-owned-system.md). It is a new decomposition of the
Rust-owned target, superseding Plan57's delivery scope rather than copying its packages. The package table and checkpoints distinguish selected implementation from remaining work; authoring this
document authorizes no implementation. The source [functional
review](../functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md) supplies O01, O10 and O12, with a bounded O13
decision. Its observations are planning inputs, not evidence that a native application journey has already passed.

The [hard-cutover review](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I01),
published at `c3b7f136d`, additionally supplies I01/I02/I07/I09/I10/I13/I14. The coordinator owns
their dispositions. This amendment retains scientific outcomes while removing unnecessary Python delivery
requirements; no package inherits unfinished Plan57 work.

The 2026-10-29 amendment integrates the [database-boundary review](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md),
published at `a6a144bce`, against the subsequent implemented checkpoints below. References to that
review are source-qualified here; its O01–O10 are distinct from the original functional review's
O01–O15. The coordinator owns their dispositions. This plan owns the native repeated-analysis
workflow for [database-boundary O05](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O05),
consuming D's canonical history and E's protected-result composition. Document adoption establishes
no new runtime or qualification evidence.

## Target and foundation assessment

The product root is a callable Rust library: admitted authored definitions and explicit physical context enter existing
authoring/compiler/runtime owners; typed prepared operations execute through the original native supervisor and return immutable results
or an owned operation handle. Ordinary in-process calls do not serialize through JSON, an MCP catalogue, Python objects or a universal
dispatcher. Small executable commands may expose useful operations when inexpensive, but a product CLI is neither the architecture root
nor a completion requirement. A future frontend consumes the same capabilities without implementing scientific admission, workflow
policy or result interpretation.

Normal linked Rust APIs are the immediate public capability. Do not first construct public pyclasses,
frozen Python document wrappers, a replacement codec or a foreign ABI. Explicit build/development generation
may publish declarations needed by retained consumers; ordinary execution, reads and help do not regenerate
code. Validate external files/requests for finiteness, versions, unknown fields and physical context at their
actual ingress, then carry established premises through Rust construction and borrowing.

Current foundations include `pse-runtime/src/workflow/mod.rs`, `operation_preparation.rs`, `sampling.rs`, `strategies.rs`,
`result_snapshot.rs` and `pse-compiler/src/workspace.rs`. `RuntimeOperationRequest`, `ModelingPackage::prepare_operation`,
`PreparedOperation::start` and `RunHandle` already connect shared native capabilities. Existing `pse-project` and `pse-worker` are
binaries under `crates/pse-runtime/src/bin/`; their existence does not establish native caller coverage of every supported workflow.
Python wrappers in `crates/pse-py/src/workflow/` identify retained user outcomes to migrate, rather than an object graph to recreate.

The focused assessment preserves blueprint §14.1/§14.4 preparation and §18 scientific execution: Salsa tracks semantic dependencies;
native compilation is an explicit effect outside queries; immutable plans/programs are reusable while mutable evaluators, numerical
factors and solver state belong to an attempt. Prepared operations and durable study descriptors have deliberately different
eligibility: host-only shooting must not acquire a persisted projection simply to make APIs uniform. Native callers need clear
construction and composition, not another scientific engine.

Keep maintained Symbolica 3.0.1 and faer 0.24.4 realizations and their vendor deltas. Symbolica multi-output evaluation already shares
expression work; faer separates sparse symbolic structure, numeric factors and explicit scratch. Reuse structure only under compatible
sparsity and numeric factors only under unchanged coefficients. Borrow native views where lifetimes fit; avoid frontend-shaped arrays
between native stages. No new JIT/SIMD/CUDA, exact-root, interval, partial spectrum or matrix-free capability is required by migration.
Sources are the review's O10 and mechanism table, `pse-math/src/{library,implicit,native_compiled}.rs`, and the existing vendor notes.

Core 3.4, heuristics 1.0, test assessment 1.2 and process-simulator 1.5 govern this target. The coordinator's
[T01](58-rust-owned-system.md#t01) and [T02](58-rust-owned-system.md#t02) own early shared contracts and the ADR/formal-review route
before contract-changing code, including changes to D12/blueprint §21. This document proposes no independent rule acceptance. Optional
PyO3 implementation is deferred; its absence cannot block native scientific completion.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [A01](#a01) Native application contract | done | Native workflow owner | Accepted shared contract and complete retained scientific caller scope |
| [A02](#a02) Incremental workspace and source workflows | done | Authoring/compiler owner | Native revision/edit/reuse behavior and source/history consumers |
| [A03](#a03) Prepared local execution and results | done | Runtime/math owner | Typed author→prepare→execute→inspect composition independent of Python |
| [A04](#a04) Complete scientific mode callers | done | Scientific workflow owners | Every supported mode constructible and failures honestly scoped |
| [A05](#a05) Native studies and replay | done | Study/runtime owner | Generated studies, durable handoff and recorded-point replay |
| [A06](#a06) Adaptive study decision and retained strategies | done | Study strategy owner | Retained actor and native strategy callers tested; external batch ask-and-tell extension deferred to an actual consumer |
| [A07](#a07) Consumer cutover and assurance handoff | done | Native workflow integrator | Replaced consumers deleted and required H journeys ready |

<a id="a01"></a>
## A01 — Establish the native application contract

Start after T01's shared-contract decision and T02's consumer scope are available. Inspect native producers, Python scientific entry
points and representative users to settle the smallest complete Rust caller surface for admission, revision/edit, preparation, local
run, study, diagnostics and selective reads. Record useful operations beside their existing owners, with actual input/context, output
ownership, effects, unsupported cases and cancellation policy; no catalogue framework or operation registry is an implementation
deliverable. Expose missing construction operations at those owners rather than teaching callers to reconstruct private layouts. Keep
pure/local work store-free where supported. The decision exits with agreed callable boundaries and consumer responsibilities, including
the host-only/durable distinction. Required ADR/formal review precedes incompatible boundary changes. Targeted acceptance exercises
native request construction and refusal without Python or unrelated storage/solver setup; remove superseded private construction paths
with their migrated callers.

<a id="a02"></a>
## A02 — Make the incremental workspace independently useful

Consume A01's source/revision contract and extend existing `CompilerWorkspace` inputs and tracked derivations, not a second workspace
cache. Native users admit a package, edit values or structure, prepare demanded products and inspect dependencies/diagnostics tied to
authored identities. Value edits preserve unaffected structural products; membership, deletion, absent lookup, provider,
physical-context and interpretation changes invalidate everything they can affect. A new unit using existing physics extends
declarations; adding a physical concept changes its semantic owner. Use existing graph projections for explanations; SCCs and
deterministic traversal must not confuse physical recycles with acyclic dependency order or persist local graph indices as identities.
A02 owns migration of native source/project/history callers through the existing runtime/project and
operations revision/publication owners. D supplies connection/authentication and canonical transition policy
where persistence is selected; A owns the scientific source/submission callers and their preparation.
Shared source/configuration revision facts belong to D's canonical operations when persistence or
cross-caller agreement is selected. Their native typed/compiled representations remain revision-bound
execution products, not another writable source authority. Database graph access may support project
inspection; native equation incidence, SCCs, formulation and numerical preparation remain with their
scientific owners. A database schema does not replace physical admission or the existing registry.
Transactional shared drafts are a possible [database-boundary O09](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O09)
consumer for F05's bounded inquiry, not permission to move single-owner workspaces or Salsa into a
second engine. Select that extension only with an actual shared update/query contract and identified
coordination retired; ordinary local edit/re-solve remains database-free.
Salsa persistence is deferred pending a concrete consumer: if selected later, compatibility/input namespaces and safe
recomputation are mandatory. Targeted compiler tests distinguish value-only, structural and deletion edits, concurrent read/write drain
and unavailable-cache recomputation. Delete Python-only workspace caching after caller migration.

<a id="a03"></a>
## A03 — Compose preparation, local execution and inspection

Consume A01 and A02's usable preparation slice, plus [C02](58c-runtime-lifecycles.md#c02)'s implemented start/stop/join contract. Use
`PreparationServices`, `ExecutionServices`, complete source/context identity, admitted bindings and `PreparedOperation` to execute a
local scientific operation without Python or a database when the operation is local. Share immutable evaluator/structural products
across compatible requests; create fresh controls, mutable evaluators, factors and provider workers for each attempt. Preserve current
physical/domain/derivative admission and independent post-solve checks; library success is not a verified scientific result. Expose
original typed diagnostics and partial/refused outcomes. Consume the results owner's protected snapshots/selective readers and explicit
retention rather than materialize all results into a facade. Returned handles preserve output lifetime independently of preparation
lookup eviction. Targeted runtime tests cover a local successful case, invalid physical context, unsupported route, canceled observer
and retained result after producer closure. Remove native-to-Python-to-native assembly and copied wrapper policy as each actual consumer
moves.

<a id="a04"></a>
## A04 — Preserve all supported scientific modes through native callers

After A03, migrate square/root, optimization, initialization/recycles, dynamics/events, fitting, sensitivity, generated-design/study and
diagnostic callers, retaining existing class-specific scope. Include current horizon/shooting and strategy operations where supported;
do not widen their durable or solver eligibility by implication. Inputs retain explicit starts, bounds, scaling, physical envelopes,
derivative order and source provenance. Numerical iteration/globalization remains with qualified existing libraries. Shared model
definitions feed modes rather than parallel formulations. For each moved mode, pair a native successful call with a revealing
failure/partial case and retain the independent scientific expectation or oracle selected by its existing assessment owner. Targeted
tests run at the scientific owner; full multi-mode journeys wait for H. Remove wrapper-only tests and fixtures only after the retained
scientific claim has a native test or explicit H owner. No permission to lose peripheral interfaces authorizes loss of one of these
scientific modes.

Coordinate F04's independent-reference cutover: the existing parity candidate harness imports `pse.Runtime`
and generated Python types. Native candidate operations must exchange explicit cases/results with isolated
IDAES/Pyomo, SciPy or teqp references instead. A public CLI or a required Python extension is unnecessary.
Retain original tolerances and physical interpretation; a deliberately wrong candidate must fail comparison.

<a id="a05"></a>
## A05 — Compose native designs, studies and reproducible results

Consume A03/A04, [D01](58d-canonical-control-and-connectivity.md#d01)'s connection/authentication,
[D03](58d-canonical-control-and-connectivity.md#d03)'s bounded canonical progression and
[C03](58c-runtime-lifecycles.md#c03)'s native preparation/resource handoff. A05 owns scientific
recording/submission and the repeated-analysis consumer from
[database-boundary O05](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O05).
D owns persisted submission, readiness, revisions, claim and settlement meaning; E02 owns protected
numeric relations. These are early usable capability slices, not a requirement to finish every
package in A/C/D/E before composing one study.

Native generated designs produce owned realized overlays; local studies remain local until explicit
recording. Submit exact original source/context/operation identity, policies, points and dependencies
through the canonical owner. Consume D's bounded applicable action and exact preparation premises,
including continuation rather than interpreting an unproductive page as global idle. Native workers
prepare admitted immutable inputs, resolve seeds and acquire actual launch resources outside database
transactions, then return prepared evidence to D for a current revision/generation/dependency/cancellation
recheck before execution claim. A preparation candidate grants no execution authority. Keep the original
issued claim/operation identity for uncertain acknowledgements; do not add a preparation-ticket protocol
merely to mirror the existing native owner.

The worker retains its effect loop and native handle/token/resource custody; it must not retain a second
authoritative readiness or transition interpretation. Migrate callers with D03/D04 and C03–C05 and remove
subsumed paging, action interpretation and frontend retry/assembly code only when the complete operation
covers their retained meaning. Local duplicate-preparation suppression is not a canonical claim ledger;
its retirement requires showing that the replacement removes its useful wasted-work role.

Retain recorded-realized-point replay, warm-start dependencies, actual failures and original result
provenance. Cancellation/restart must not reinterpret imputed values as observations or regenerate
recorded points from a seed. Install dependency edges before activation; this scope does not introduce
live graph mutation. Notifications wake consumers; canonical catch-up resolves missed events. Record
useful calculation/progress observations at scientific granularity, not every native runtime event.

A repeated-analysis workflow selects yesterday's and today's accepted attempts by authored revision,
physical/effective context, realized point and scientific qualification, then uses E02's protected IPC
relations for comparison. Explicit domain revisions, effective time and frozen cohort identity are not
interchangeable with database MVCC or a future KV storage timestamp. The current persistent MX path
can supply this domain history without storage-level time travel or a separate execution-history index.

Targeted acceptance covers deterministic point identity under reordered completion, generation failure,
recorded replay after producer closure, sparse bounded pages with continuation, two competing workers,
stale prepared evidence/cancellation before claim, and lost publication acknowledgement under original
identity. The repeated-analysis case includes worker replacement and a later context correction while
preserving the earlier frozen selection. A stale native completion must not overwrite a current accepted
attempt. Expected science remains independent of the producer; full joined journeys belong to H.
Delete Python study assembly, duplicated history reconstruction and redundant conversion with the
migrated retained callers, keeping independent scientific/reference evidence and E's read custody.

<a id="a06"></a>
## A06 — Decide adaptive composition without inventing another optimizer

Preserve current native adaptive strategies and their thread-affine `EgorSolver`/`EgorState` owner. Before any new ask-and-tell surface,
compare that realization with pinned EGObox 0.38.1 `EgorServiceFactory`/`EgorServiceApi::suggest` against actual
suggest→evaluate→observe→resume needs. Research exits with one bounded choice: reuse the actor; adopt service ask-and-tell for a named
consumer; or defer the extension with that consumer/trigger recorded. Check exact release APIs, state serialization/reconstruction,
failure/imputation semantics, batching and thread/lifetime requirements. A source/API check suffices for rejection; unresolved execution
premises require a small targeted experiment during authorized implementation, not an open-ended benchmark campaign. Do not mix newer
overview APIs into current ego/moe/gp/doe 0.38.1/0.35.5/0.36.4/0.35.1, ndarray 0.16.1 or rand_xoshiro 0.6.0. Stable study/point identity
assigns random streams. If adopted, the same package migrates the concrete consumer, tests truthful observations and resume/cancel
behavior, and deletes its replaced optimization loop. Deferral does not remove A05.

<a id="a07"></a>
## A07 — Complete native consumer cutover and hand off qualification

Finish with every retained scientific workflow callable through Rust APIs while Python is absent. Migrate actual documentation/examples,
worker roots and non-Python consumers alongside each owner; retain only optional binding obligations explicitly selected elsewhere.
Coordinate deletion of the retiring Python facade/MCP consumers with their owning plan; never remove a shared guarantee merely because
its old wrapper disappeared. A future frontend adaptation must consume these operations without copying their scientific policy. Product
command additions remain discretionary.

Use integrated cutover sets: owner, real consumers, configuration, generator declarations, targeted tests and
obsolete implementation deletion belong together. Temporary development incompleteness earns no acceptance;
do not add compatibility DTOs, a FastMCP proxy or a Python scientific fallback to keep intermediate commits usable.
Retire Python projections, stubs, manifests and self-check expectations with their last retained consumer in F/G.
Keep genuine external scientific contracts, not mechanism-specific drift guards.

## Verification and current checkpoint

**Proposed:** use `just check-package pse-compiler`/`pse-runtime` and narrowly selected `just unit-package <pkg> <filter>` for changed
behavior, with explicit force-validation on actual Arrow correctness closures. Package descriptions specify discriminating cases, not
executed tests. [H](58h-qualification-and-publication.md) owns assembled native local/durable scientific journeys, independent
oracle/reference selection, scope-end integration/solver/component checks, `just hygiene` and manual governance/docs/native/powerset
checks. Measurements need a separate decision purpose; no speedup is claimed here. At authoring, implementation and qualification were **not_run**; the checkpoint below records subsequent authorized execution. Completion requires the coordinator's combined acceptance and truthful zero-baseline
reporting, not this document's package count or a local targeted pass.

## Native local operation checkpoint — 2026-10-28

The selected A01/A03 wave uses existing public Rust constructors and a database-free KINSOL
operation, with only source-admission A02, local C01/C02, F02 and the selected G retirement.
It introduces no application facade, executor, database dependency or CLI. SharedRuntime,
its registry-bound factory, Runtime::from_shared_local, original physical/model document
admission and PreparedOperation/RunHandle compose the downstream caller. The package and
execution services retain the same actual bindings; invalid physical input and incompatible
receivers remain refusals. Module documentation records the public composition.

The agreed Python cut removes asyncio run waiting and CancelWait; shared synchronous
adapters and unrelated asynchronous cursor interfaces remain with their existing owners.
Scientific Python assertions are retained through synchronous joins, with live-cursor joins
after asynchronous consumption/detachment. No replacement future adapter is introduced.
F03 is limited to making the actual compiled-metadata stub owner callable with an explicit
artifact, without the retired broad native profile or installed-module prerequisite.

A01–A03 remain in progress for the series: this selection does not deliver incremental edits,
all scientific modes, durable studies or all source/history consumers. C01's long-lived root
comparison and other C packages retain their own obligations. Existing local last-owner stop,
passive observation, repeatable waits and original native-drain semantics remain authoritative.

### Selected verification and outcome

**Implemented:** the downstream `local_operation` target uses only production constructors,
source-backed physical/model admission and PreparedOperation/RunHandle. Rust module docs
record construction and lifetime semantics. Existing native owners needed no new lifecycle
behavior, public type, scheduler or facade. Python asyncio run waiting and its cancellation
guard are deleted together with their dedicated bridge test and callers. Retained scientific
assertions use synchronous joins; shared synchronous and asynchronous cursor adapters remain.
The actual compiled-metadata generator updates only the removed method and its provenance.

**Tested:** five selected local/native lifecycle controls pass against zero failures:

```sh
scripts/pse-env --native=compiler,klu -- cargo t -p pse-runtime --lib \
  --features pse-runtime/solver-kinsol \
  -E 'test(local_waiter_and_observer_detach_without_cancelling_original_handle) | test(caller_departure_cancels_only_immediate_child_and_keeps_durable_owner_live) | test(abandoned_caller_holds_pool_and_cpu_through_tls_destructor) | test(compiler_authority_and_workspace_remain_owned_through_tls_destructor_and_abandonment) | test(cancelled_serial_preparation_keeps_loan_and_storage_until_child_drain)'
```

The native Cargo-restoration setup passes; 1249 unselected inventory tests receive no
acceptance credit. The new handle control establishes waiter/lease policy; existing actual
TLS/preparation controls establish lower native resource retention and drain. This is
composed coverage, not an experiment interrupting a live KINSOL callback.

The validated PyO3 construction passes through
`scripts/pse-env --native=compiler,klu -- scripts/cargo-bin/cargo-pse-correctness build --locked
-p pse-py --lib --features pse-py/solver-kinsol --message-format=json`. Cargo's actual artifact
is supplied to `just python-stubs --extension target/debug/lib_native.so` and
`just python-stubs-check --extension target/debug/lib_native.so`; both pass without extension
installation. The generated API diff removes only wait_async; independent review confirms
its provenance hashes. The recipe transport control passes one test with publish/check
subcases, literal spaced/metacharacter paths and original exit preservation:
`scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_recipe_arguments.RecipeArgumentTests.test_python_stubs_accept_explicit_artifact_without_installed_extension -v`.

Initial public-test construction exposed accessor/type and downstream recursion-limit issues;
these were corrected without changing production guards. Its first executable success case
incorrectly expected one solve-variable row: the original table records both parameter p and
variable x. The correction checks their distinct roles, the authored parameter, actual KINSOL
selection and the original equation under propagated physical tolerance; it does not discard
parameter output. Review also replaced a feature-dependent Ipopt-unavailable assertion with
the intrinsically unsupported HiGHS nonlinear-root route. The negative case had passed before
the positive row assertion was corrected. All failure baselines remain zero.

No full Python solver, canonical-store, other solver, feature-matrix, parity or performance
campaign ran. Broad H qualification and the remaining A/C/F/G packages stay open. Scientific
and construction evidence remain distinct; no quantitative speed or memory benefit is claimed.

**Tested:** the final public caller command
`scripts/pse-env --native=compiler,klu -- cargo t -p pse-runtime --test local_operation
--features pse-runtime/solver-kinsol` passes two tests, zero failures/skips, and its native
Cargo-restoration setup passes. It covers actual source admission, KINSOL/equation assessments,
repeatable shared results, retained rows after caller/cache/native-lane closure, invalid
physical sources, wrong bindings, incompatible route, prestart cancellation and sealed caller.
Fixture bodies settle before propagating errors or assertion panics. Independent bounded source
and generated-output review has no remaining material findings. The selected local wave is
complete at this acceptance level; enclosing packages retain their broader obligations.

The root's `just turn-end` passes ADR-index generation and formatting against zero failures.
Final independent user-manager readback finds no owned pse-host/pse-foreground units, and
all selected construction/test runs have finished. Formal ADR status is unchanged. The
architecture amendment used the authorized PSE_DESIGN_EDIT=1 route; the pre-existing tracked
.design-edit content was preserved. No full-series qualification or new speed claim follows.

## Complete native workflow execution selection — 2026-10-29

The maintainer authorized the detailed A01–A07 execution approach, including the necessary
C lifecycle, E01/E02 retained-data and F/G consumer-retirement slices. Earlier local/native
worker and reference receipts retain their original scopes; this selection does not restart
paused unrelated B/C/D work or count historical library fixtures as qualified.

Execution first separates mathematical preparation/execution from protected retained-data
construction, and establishes a public local immutable edit/prepare/resolve consumer. The
read capability must contain actual configured reader/model/pool/admission resources, not a
renamed SourceServices/PreparationServices or full Runtime. Frozen cohorts and snapshots
consume it. A02 extends existing SourceDelta/with_delta and Salsa products; no second cache
or source authority is introduced. A04 moves complete scientific caller sets and their
independent assertions; missing API construction belongs to the scientific owner rather than
another facade. Local capability and persisted study eligibility remain distinct.

A05 then composes exact realized designs, canonical submission/progression, native workers,
protected replay and frozen repeated analysis. Historical context-based study fixtures must
move to actual native capabilities before their acceptance. A06 retains the complete current
EgorSolver/EgorState actor; the dataset-driven service suggest API is not an equivalent
retained state/lifecycle replacement. A new externally scheduled adaptive batch interface is
deferred until an actual consumer needs it; native strategy behavior remains required here.
A07 deletes migrated Python scientific orchestration and reconciles actual remaining consumers,
generation and declarations with F/G. Independent external Python oracles remain isolated.

Library selection follows the reviewed pinned release contracts. In particular ndarray 0.16.1,
rand_xoshiro 0.6.0, tonic 0.14.6 and the current EGObox family do not inherit newer overview APIs.
Use ndarray views/Zip for useful native transformations, Bytes ownership for protected buffers,
existing graph projections and Salsa dependency tracking, typed boundary Serde/Schemars/Prost,
thiserror/miette failures, and calculation-level tracing/OTel. No new RPC hop, collector,
sd-notify or hierarchical shutdown framework is selected. Native thread/foreign/TLS joins and
canonical effect settlement are not replaced by task cancellation. Workstation parallelism,
generous pool defaults and existing finite scientific contracts retain their owners.

Current assignments use main with explicit disjoint source ownership. The coordinator owns
shared exports/manifests, builds, integration and status; no source worker runs Cargo or
formatters. Targeted public/native tests establish each cut. Scope-end qualification and H
handoff follow all selected functional work; no performance benefit is inferred from deletion.

### Native data and public caller evidence

**Implemented:** public native callers now exercise immutable value and structural source edits,
root re-solving with preserved prior results, Clarabel cone optimization, initialization and
explicit/conditional recycle strategies. The E01/E02 prerequisite extraction removes retained
SourceServices/Runtime/math ownership from native snapshots, readers and frozen cohorts and
adds native protected cohort relations. Existing Python wrappers remain retirement work; their
full runtime captures were identified by independent source review and are not covered by a
claim that every consumer is math-free.

**Tested:** the compiler invalidation control passes one test, zero failures, 319 excluded,
using `scripts/pse-env -- just unit-package pse-compiler
namespace_add_delete_shadow_import_and_absence_match_fresh_complete_admission`.
The native selected run passes 19 tests, zero failures, 1458 excluded, against the zero baseline:

```sh
scripts/pse-env --native=compiler,solver,klu -- /bin/sh -c '
  PSE_NATIVE_FIXTURE_MODULE_RECEIPT=$(cat /tmp/plan58-native-module-record-path)
  export PSE_NATIVE_FIXTURE_MODULE_RECEIPT
  export PSE_NATIVE_FIXTURE_SERVICE_HELPER="$PWD/target/debug/pse-canonical-service"
  exec scripts/cargo-bin/cargo-pse-correctness nextest run --locked -p pse-runtime \
    --test native_study --test native_conic --test native_workspace --test native_strategies --lib \
    --features native-study-tests,product-operators,solver-kinsol,solver-ipopt,solver-scip \
    -E "test(retained_native_admission_preserves_scope_and_close_without_math) | test(native_recorded_study_two_workers_preserve_science_protection_and_lineage_after_producer_close) | binary(native_conic) | binary(native_workspace) | binary(native_strategies) | test(math::surrogate::library::tests::)" \
    --no-fail-fast
'
```

The explicit prepared production module and private native-service fixture avoid the configured
development database. The two-worker study retains its original lineage and independent root
oracle; after scientific shutdown, a separately constructed read capability verifies the original
unknown and equation. Existing EGO state controls retain the actor decision's bounded evidence.
These targeted checks do not close A02's full incremental coverage, all A04 modes, A05 repeated
analysis, A07 retirement, or E/H qualification. No scope-end static or performance campaign has
run for this selection.

**Interface-checked:** the adjacent Python bridge compiles with zero failures using
`scripts/pse-env --native=compiler,solver,klu -- scripts/cargo-bin/cargo-pse-correctness check
--locked -p pse-py --features solver-kinsol,solver-ipopt,solver-scip`. This checks its retained-data
type migration and does not qualify Python execution or remove its remaining runtime captures.

### Native repeated analysis and scientific caller cut

**Implemented:** `native_repeated_analysis` now uses generated realized overlays, explicit
source/context recording, competing workers, frozen protected cohorts and native DataFusion
relations. It replaces producer/worker callers, records a later calibration correction with
unchanged physical definitions, preserves the earlier domain-time cut and frozen selection,
and replays the original recorded points under their original context. Native aggregate
assertions retain unavailable/empty distinctions, exact attempt attribution, quantity/unit
identity and independent count/minimum/maximum/mean expectations. A wrong-pool query refuses.
Four-relation joins compare actual original/corrected roots twice after all scientific lanes
close; the receiving read capability never constructs mathematics.

**Tested:** one repeated-analysis test passes, zero failures/skips, through the same explicit
private native-service/module fixture and correctness route used above, selecting
`--test native_repeated_analysis` with
`--features native-study-tests,product-operators,solver-kinsol,solver-ipopt,solver-scip`.
Its first executable failure exposed an invalid assumption that the configured native state
has a mutable default catalog. The corrected caller uses DataFusion55.1.0's direct
`read_table`/`alias`/`join_on` composition, preserving its configured state and pool.

**Implemented:** public local callers now exercise hybrid events/reset/sensitivity and terminal
prefixes, single/multiple shooting and sampled continuity/path constraints, rolling-horizon
applied moves with independent plant evolution, smooth nonlinear optimization, weighted
fit/covariance, active-bound sensitivity and Jacobian degeneracy. Their adverse cases retain
refusal, stopped candidates and incomplete evidence rather than treating them as qualified.
`ShootingProblem::numerics` exposes the existing immutable resolved policy so a native consumer
can assess the original physical accuracy; no new scientific engine or durable operation is added.

**Tested:** `--test native_dynamic_workflows` passes three tests, zero failures/skips; the final
`--test native_scientific_modes --test native_adaptive_strategy` selection passes six tests,
zero failures/skips. Both use:

```sh
scripts/pse-env --native=compiler,solver,klu -- \
  env PSE_SCIENTIFIC_INPUTS=/tmp/plan58-reference-native-inputs.json \
  scripts/cargo-bin/cargo-pse-correctness nextest run --locked -p pse-runtime \
  --test native_scientific_modes --test native_adaptive_strategy \
  --features native-study-tests,product-operators,solver-kinsol,solver-ipopt,solver-scip,solver-highs,solver-diffsol \
  --no-fail-fast
```

These use the previously captured immutable physical-source capsule, current production public
constructors and original native supervisors. Initial test construction omitted demanded NLP
outputs and requested tighter analytic agreement than the default engineering budget. The
corrected caller demands those outputs and requests explicit physical accuracy; no acceptance
tolerance was loosened. The adaptive root fixture initially selected arbitrary finite bounds
unsupported by KINSOL; it now preserves the original square-root class and keeps statistical
search bounds with the surrogate owner.

A06 retains `EgorSolver`/`EgorState`: the public actor test advances the same retained history
and RNG, cancels a snapshot independently, screens a statistical proposal and obtains permission
only from the original root corrector. A failed callback retains its typed cause and actual work
without imputation or silent retry. Snapshotting is an owned observation, not serialized actor
regeneration. The service ask-and-tell extension remains deferred until a named externally
scheduled adaptive batch consumer requires it; no second optimizer is selected.

A07 deletes the now-replaced Python aggregate projection while retaining native aggregation,
export and the separately consumed frozen browsing/observer interfaces. The reduced PyO3 module
build and actual compiled-stub publication pass; independent source review found no material
actionable issues. [G's consumer cut](58g-boundary-retirement-and-optional-consumers.md#native-repeated-analysis-consumer-cut--2026-10-29)
owns that integration and its remaining Python boundaries. Full A/E/G and H acceptance,
remaining scientific/frontend consumers, residual-limit and payload-custody controls keep their
own obligations; no complete scope-end qualification or measured speed claim follows.

### Native generation and prerequisite acceptance

**Tested:** the selected compiler prerequisite run passes eight tests, zero failures, 312
excluded, using `scripts/pse-env -- just unit-package pse-compiler` with the filter union for
`kernel_queries_reuse_math_across_values_unrelated_edits_and_clean_rebuild`,
`immutable_revision_refuses_provider_context_substitution_before_publication`,
`namespace_add_delete_shadow_import_and_absence_match_fresh_complete_admission`,
`cloned_readers_drain_success_error_and_local_cancellation_before_repair`,
`context_only_same_quantity_id_with_changed_meaning_invalidates_consumed_body`,
`document_reuse_decoded_interpretation_changes_match_clean_admission`,
`physical_context_and_matching_source_publish_as_one_candidate` and
`delayed_completion_refuses_changed_reached_provider_registration`. This is focused A02
prerequisite evidence, paired with the public immutable-edit caller; it is not a new
compiler-wide qualification or performance measurement.

**Tested:** the explicit private native-service correctness route selecting `--test native_study`
passes all four tests, zero failures/skips, with
`--features native-study-tests,product-operators,solver-kinsol,solver-ipopt,solver-scip,solver-highs,solver-diffsol`.
It exercises original science/protection after producer closure, cancellation before claim,
active cancellation with actual drain, and killed-process replacement with preserved unknown
effect meaning. No configured development database is used.

**Implemented and Tested:** the store-free public `native_design` target passes two tests,
zero failures/skips, through the same prepared correctness route and selected feature set.
All six design kinds retain physical coordinate meaning, intervals, Latin/centered strata or
exact Cartesian endpoints/order. Duplicate integral-level coordinates remain distinct
occurrences. Realized designs and exported content survive producer/cache/native-lane/root
disposal. Invalid targets, physics, ranges, counts, work, grid shape and cancelled requests
refuse without poisoning subsequent valid generation. The target itself requires no solver
feature or canonical service; the run reuses the current broader native build selection.

The strengthened repeated-analysis caller uses authored occurrence keys 13/29 independently
of vector/page positions. Its initial added assertion incorrectly treated a cohort member's
ordinal as that authored key. The corrected assertions retain the ordinal and match the
exact canonical point key to the authored identity. The full replay/correction/comparison
test passes again, zero failures/skips, with `--test native_repeated_analysis` under the
same private fixture and feature selection.

A07 deletes Python `GeneratedDesign`/`generate_design`, its complete PyO3 sampling module,
registration/aliases/exports and dedicated wrapper/replay tests. Native design geometry and
the recorded-point replay consumer retain the scientific claims. The reduced PyO3 build and
compiled-stub publication pass; independent bounded source review found no material issues,
with the ordinal correction established by execution. Generic conformance/manifest execution
and the thermodynamic evidence campaign remain separate F consumers; the per-package native
conformance caller supplies the next A04 coverage cut below. These results do not close A07/G or H.

### Public native conformance and remaining handoff

**Implemented and Tested:** `native_conformance` passes two public-caller tests, zero
failures/skips, using the prepared native correctness route with `--test native_conformance`
and the same selected feature set above. The target itself requires no linked solver or
scientific capsule. Direct per-package and independently budgeted standalone pure requests
preserve an exact arithmetic expectation, original typed trial-rejection/form/variable lineage,
selected-versus-package coverage, unknown-selection refusal, incomplete/unattempted inventory,
and report-owned tables after producer closure. One-byte evaluator scratch refuses with the
original typed resource code/rule/class/detail on both native and pure request paths.

Initial test construction needed an explicit async error type and incorrectly selected an
inapplicable numeric expectation from the fixture whose expected outcome is rejection. The
corrected assertion examines the sole evaluated numeric comparison while separately requiring
the declared original rejection and its passed expected-failure check. No scientific requirement
or tolerance was relaxed. The corresponding Python scientific/preparation assertions retire;
their still-consumed CLI export subcase remains a developer-adapter test. No CLI/report protocol,
thermodynamic campaign or shared document helper is removed by this A04 scientific cut.

A06's bounded decision and retained-strategy caller acceptance are complete: keep the full
actor, with eight existing library-state controls and two public construction/continuation/
failure controls, plus native initialization/recycle callers. A newly named externally
scheduled adaptive batch consumer is the observable trigger for revisiting the deferred
ask-and-tell extension; snapshot persistence or a second optimizer is not selected.

Remaining A01–A05/A07 acceptance must reconcile all actual scientific/source consumers and
frontend retirement with their F/G owners before whole-plan closure. F05 owns generic
conformance manifest/run/CLI/report orchestration and the distinct thermodynamic evidence
campaign; native per-mode callers do not replace those developer semantics by implication.
E01/E02 still own residual-limit and payload-custody/reclamation controls. H owns composed
scientific/reference and scope-end static/manual qualification; it has not run for this scope.
Unrelated paused B/C/D work is not reopened. Native capabilities and selected passing cuts
must not be reported as complete first-party Python retirement or full-series acceptance.

### Native read and observation follow-up

**Implemented:** passive `StudyObserver` retains the same narrow read capability as snapshots
and cohorts; independent read callers can construct/reconnect it without a mathematical runtime.
Three protected-result unit controls use private native service ownership instead of the old
Python fixture registry. Their original scientific/protection assertions remain, and the
fixture explicitly selects the existing engine Contract-observation policy. Its module gate
is `native-study-tests`, matching the actual private-service prerequisite.

**Tested:** four tests pass, zero failures, 1617 excluded, with the same explicit prepared
production module/private service helper and:

```sh
scripts/pse-env --native=compiler,solver,klu -- \
  scripts/cargo-bin/cargo-pse-correctness nextest run --locked -p pse-runtime \
  --test native_repeated_analysis --lib \
  --features native-study-tests,product-operators,solver-kinsol,solver-ipopt,solver-scip,solver-highs,solver-diffsol \
  -E 'binary(native_repeated_analysis) | test(result_pruning_tests::)' --no-fail-fast
```

The repeated-analysis case now checks passive observation before execution and reconnect,
foreign issuer/context refusal, observation-only cancellation, bounded cohort export, exact
protected detail export and typed aggregate export after scientific shutdown. The three library
controls cover inexact residuals before limit/projection, the complete selected universe after
stream exhaustion, configured pool/state, physical predicate reuse and protected-schema pruning
without native batch decode. All use authenticated private state, not the configured database.

The enabled library-test feature combination exposed a horizon test-cleanup type mismatch:
successful abandonment returns its original row, while settlement consumes unit success.
Mapping that success to unit preserves its original errors. Export receiver assertions also
needed the actual wire contract: canonical export metadata is distinct from a registry relation,
and its manifest field carries a digest rather than the storage key. The corrected assertions
check original export attribution and exact generated fields/arrays without weakening registry
admission. No production wire format or scientific tolerance changed.

The following A07 cut removes the Python frozen-cohort/passive-observer projection and its
polling state machine, both facade method declarations, registration/exports, generator family
and adapter-only tests. The existing native APIs and five typed study ProjectAction variants
remain. Their actual executable consumer now runs within the private repeated-analysis journey,
checking bounded workbench paging, cohort NDJSON/export refusal, exact result IPC metadata and
arrays, typed analysis, and terminal observation/history coverage. Core Rust construction does
not require product-operators; the executable consumer is explicitly feature-gated.

**Tested:** the same explicit private fixture/correctness route selecting
`--test native_repeated_analysis` with
`--features native-study-tests,product-operators,solver-kinsol,solver-ipopt,solver-scip,solver-highs,solver-diffsol`
passes one test, zero failures/skips (run `9198e1e6-2722-450d-974c-2e292ca3c266`).
The projection generator's remaining project/research interface, effects, defaults and refusal
controls pass five tests, zero failures, using `scripts/pse-env -- .venv/bin/python -m unittest
scripts.tests.test_project_operation_projection`. Its missing-document control now targets
the retained research declaration after the study declaration's deletion. Remaining scientific
consumer reconciliation, E delivery custody and H qualification stay open.


### Native source reload, local studies and diagnostic callers

**Implemented:** the public `native_source_reload` caller uses exact-parent complete document
inventories, compares all declarations and scientific source identity against independent fresh
ingress, restores removed documents and preserves both immutable histories. Malformed syntax
and another parent's delta refuse atomically. Three duplicate Python parsed-reload tests retire;
the Python method still has separate local/adoption and telemetry acceptance consumers, so this
cut does not remove it or claim complete source-facade retirement.

**Implemented:** four `native_local_study` callers preserve generated equal inputs as distinct
occurrences with original keys, admission identity across equivalent bindings, complete physical
refusal envelopes, unusable predecessor refusal without dispatch, independent successful members,
lazy selected-source construction, and compact exact/declared-empty detail after producer close.
They are store-free and use the actual admitted root/equation physical requirements.

**Implemented:** six `native_diagnostics` callers preserve singular/missing-value diagnostics,
source maps, owned SVD evidence, physically converted sparse samples and bounded partial campaigns,
affine IIS/ranging/relaxation, shared authored fixture checks/initialization, local nonlinear
explanation with the original POUNCE attempts, and typed admitted knowledge/provenance. The
sample receiver owns immutable decoded Arrow input independently of the producer. Existing native
Jacobian callers retain the separate complete/incomplete degeneracy search. External Python
mutation hooks retire with their frontend; this does not weaken native physical admission.

**Tested:** using the prepared private-service/module environment above, the correctness selection
below runs 13 controls: 11 initially pass and two fixture-setup cases fail, with 1647 excluded.
The affected two then pass, zero failures, using the same flags and selecting only their names
(run `8f9a28e1-eaa9-4426-bbaa-d889109d1538`). This is composite passing evidence, not a single
all-pass campaign or H qualification:

```sh
scripts/pse-env --native=compiler,solver,klu -- \
  scripts/cargo-bin/cargo-pse-correctness nextest run --locked -p pse-runtime \
  --test native_local_study --test native_source_reload --test native_diagnostics \
  --test native_repeated_analysis --lib \
  --features native-study-tests,product-operators,solver-kinsol,solver-ipopt,solver-scip,solver-highs,solver-diffsol,solver-pounce \
  -E 'binary(native_local_study) | binary(native_source_reload) | binary(native_diagnostics) | binary(native_repeated_analysis) | test(project_operator_wire_defaults_effects_and_public_projection_set_remain_owned)' \
  --no-fail-fast
```

The diagnostic sample caller initially assumed bare variable paths; its receiving coordinates now use
actual prepared source-instance lineage and require unique selected diagnostic columns. The
two-point design initially allowed 100 work units while the generator requires 160; a finite
256-unit fixture allowance admits the intended geometry without changing scientific assertions
or the independent local point-limit refusal. Earlier test compilation also corrected borrowed
source-map scope and optional diagnostic point values. No scientific tolerance changed.

Independent bounded source review finds no material local-study/reload issues. The previous
cohort-cut review identified root-level helper autodiscovery: both new support modules now live
under directories, so only their owning targets include them and the study executable helper
retains its explicit `product-operators` gate. The post-relocation repeated-analysis and
retained-action declaration controls pass in the selection above. This correction changes no
production interface. Diagnostic projection retirement follows this passing native evidence;
shared F05 settings, campaign admission/handoff types and source/telemetry consumers keep their
owners. The project-source follow-up below supplies the next A02 native consumer slice.


### Diagnostic retirement and native project-source follow-up

**Implemented:** the diagnostic Python cut is integrated across its five methods, four result
classes, singular-mode tuple, registration/exports and compiled stub. Shared diagnostic settings,
native-analysis output, source maps, knowledge and process diagnostics remain for actual
consumers. [G's diagnostic cut](58g-boundary-retirement-and-optional-consumers.md#native-diagnostic-consumer-cut)
owns that boundary. The native singular caller now also exercises public Jacobian requests and
full/attempt-limited exports, including supplied origin, original source/numerical identity,
exact row/variable nominals, source-attributed conditioning and irreducible sets. Source fields
must cover both original equations before their document/spans are checked. Independent review
identified those two missing receiving assertions; the corrections preserve the original
scientific contract rather than relying only on a lower-level optimization control.

**Implemented:** the repeated-analysis journey now invokes a public native project-source
consumer within its existing private service. A separate project/branch/problem retains
parameter values 1→2→3 through native document admission, source fork, opaque draft custody,
typed draft admission/publication, immutable versions and branch/bookmark selections. Raw
malformed bytes never become scientific publication. Accepted publication replay settles its
original intent before consulting a later mutable draft; selected source remains exact after
new heads and rename. Issuer, parent and physical-premise refusals preserve the original head
and bytes. No additional runtime, executor or canonical service is created. Broader project
operator/export and delivery-custody assertions keep their existing G/E/F owners.

**Tested:** two focused tests pass, zero failures/skips, five excluded (run
`fa6c1f4f-be70-4c5d-881f-aafeb5e28504`). The same explicit private-service/module correctness
route selects `--test native_diagnostics --test native_repeated_analysis`, the feature set
above including `solver-pounce`, and
`-E 'test(singular_diagnostics_preserve_missing_values_source_map_and_owned_modes) | binary(native_repeated_analysis)'`.
This establishes the added public Jacobian/export/source-field controls and the composed native
project-source/repeated-analysis consumer. It does not rerun unmodified scientific modes or H.

**Interface-checked:** the reduced `pse-py` build and actual compiled-stub publication pass with
the preceding prepared build/stub commands. The retained project/research facade and diagnostic
settings-envelope controls pass eight cases, zero failures, using the explicit compiled image,
private `--basetemp`, `python/pse/tests/test_project_operation_facade.py` and
`python/pse/tests/test_standalone_document_wire.py::test_standalone_native_byte_ingress_uses_the_same_envelope`.
No extension installation or broad Python component qualification is claimed.

Before the final focused native run, `cargo clean -p pse-runtime` reclaimed 103.5 GiB of
package-owned outputs under the maintainer's existing cleanup authorization. Filesystem available
space increased by about 100.4 GiB; prepared host tools, the canonical-service helper, extension
image, caches outside that package and databases remained. The subsequent run includes its cold
runtime compile cost. This is storage housekeeping, not a measured execution-speed benefit.


### Complete fitting caller coverage and functional handoff

A01–A06's native functional scope has passed its selected acceptance. At this checkpoint A07
remains in progress for scope-end integration, static checks and the handoff to H; the later
Outcome records its settlement. This is not completion of G's
whole Python retirement or H's assembled series qualification. Formal ADR statuses remain
proposed. The public native producers/callers require neither Python nor a product CLI.

**Implemented and Tested:** the distinct residual-LM, all-fixed evaluation and transient-fit
callers pass three tests, zero failures, seven excluded (run
`60b915aa-df4e-4860-a8ad-a5fa1ee371c9`). Using
`PSE_SCIENTIFIC_INPUTS=/tmp/plan58-reference-native-inputs.json`, the existing prepared native
correctness route selects `--test native_scientific_modes --test native_dynamic_workflows`,
the feature set above, and the three names `native_residual_lm_reports_actual_stopping_and_original_fit_qualification`,
`native_all_fixed_fit_evaluates_original_sources_through_shared_completion_and_owned_tables`
and `public_integrated_fit_preserves_analytic_rate_response_and_observation_provenance`.
Residual-LM retains original stopping criteria, actual work and qualification; all-fixed fitting
runs evaluation with no fabricated optimizer or counts, preserves source declarations and
shared completion/table custody. Both reject an actual unsupported requested policy. The
transient fit recovers rate 3 in `total(t)=2+rate*t` at its frozen parameter allowance and
preserves scaled response/observation/unit/run/experiment provenance and qualified exports.
Expected results remain analytic rather than copied from production responses.

The corresponding Python scientific assertions retire. Their shared measurement fixture remains
for the actual research-source consumer; the earlier dynamic simulation/storage/export case
also remains. The generic retained Python strategy adapter still needs its non-algebraic
receiver refusal, which has a narrow store-free control rather than another scientific test.
That control passes one case, zero failures, with `--basetemp`, the explicit current compiled
image and `test_native_workflow.py::test_retained_generic_strategy_adapter_refuses_fitting_preparation`.
Its initial inherited canonical fixture refused at configuration read before provisioning;
the final control constructs an ephemeral caller and forbids canonical allocation. No configured
database is mutated. Native `PreparedFit` does not expose the algebraic-only method.

Initial fitting test compilation attempted to propagate borrowed report/completion errors out
of their result owner. Expected-success assertions now inspect those borrowed results while the
owner is alive, preserving their original errors on assertion failure. No production lifetime
contract or scientific tolerance changed. The reduced Python module build and compiled-stub
publication also pass after deleting the redundant package-level local-study forwarding method.
The retained flash-sweep consumer uses `LocalStudy.execute`; the native Rust core study operation,
opaque F05 recording handoff and reports remain.

The native guide now teaches Rust simulation/physical-input/fitting construction. G's
[remaining consumer dispositions](58g-boundary-retirement-and-optional-consumers.md#remaining-scientific-consumer-dispositions)
classify the actual F04/F05/E/G dependencies instead of implying complete facade retirement.
A05 consumes D's already recorded sparse progression, generation/issuer/seed fencing and
uncertain-acknowledgement controls at
[D's native checkpoint](58d-canonical-control-and-connectivity.md#native-progression-migration-and-control-checkpoint--2026-10-29).
Those owner tests retain their original conditions; A does not reopen paused D work or promote
that component evidence to an unrun assembled journey. Independent bounded implementation review
found no remaining native scientific implementation gap after these fitting additions.


## Outcome (recorded after implementation)

### What was built

**Implemented · 2026-10-29:** A01–A06 deliver the callable native source/workspace,
preparation, local scientific execution, study/design, retained-data and diagnostic scope
recorded above. A07 integrates its actual consumers, deletes the replaced Python scientific
projections and supplies H with independently constructible native callers. The Rust
application needs neither Python nor a product CLI. Durable policy/history stays canonical;
mathematics, attempt-local mutable state and original physical resource ownership stay with
their Rust owners. Architecture revisions 308–310 and proposed ADR-0233 retain the enduring
construction/boundary contracts. The native workflow and operational-store guides describe
actual construction and existing-state limits.

**Implemented:** G's [remaining consumer dispositions](58g-boundary-retirement-and-optional-consumers.md#remaining-scientific-consumer-dispositions)
keep the real F04/F05/E04/G03 obligations explicit. Retained thermodynamic campaign,
reference/adoption/telemetry, stored-result and optional adapter consumers do not become
unqualified deletions. This completes A's native consumer selection and handoff, not all
first-party Python retirement, paused B/C/D, or full Plan58/H qualification. A's completed
record remains while those live readers consume its handoff.

**Tested:** zero-failure baseline; the integrated native selection passes **56 tests across
15 binaries, zero failures, 1631 excluded**, with 16 nextest lanes, explicit force-validation,
the prepared compiler/solver/KLU profile, private native service/module ownership and the
explicit scientific-input capsule. Run `94234d59-fdd3-4c51-b66e-327bdd94c345` uses:

```sh
scripts/pse-env --native=compiler,solver,klu -- \
  scripts/cargo-bin/cargo-pse-correctness nextest run --locked -p pse-runtime --lib \
  --test local_operation --test native_workspace --test native_conic --test native_strategies \
  --test native_design --test native_conformance --test native_adaptive_strategy \
  --test native_scientific_modes --test native_dynamic_workflows --test native_local_study \
  --test native_source_reload --test native_study --test native_repeated_analysis --test native_diagnostics \
  --features native-study-tests,product-operators,solver-kinsol,solver-ipopt,solver-scip,solver-highs,solver-diffsol,solver-pounce \
  -E 'binary(native_workspace) | binary(native_conic) | binary(native_strategies) | binary(native_design) | binary(native_conformance) | binary(native_adaptive_strategy) | binary(native_scientific_modes) | binary(native_dynamic_workflows) | binary(native_local_study) | binary(native_source_reload) | binary(native_study) | binary(native_repeated_analysis) | binary(native_diagnostics) | binary(local_operation) | test(result_pruning_tests::) | test(retained_native_admission_preserves_scope_and_close_without_math) | test(math::surrogate::library::tests::) | test(project_operator_wire_defaults_effects_and_public_projection_set_remain_owned)' \
  --no-fail-fast
```

The command inherits `PSE_NATIVE_FIXTURE_MODULE_RECEIPT`,
`PSE_NATIVE_FIXTURE_SERVICE_HELPER` and `PSE_SCIENTIFIC_INPUTS` from the explicit private
fixture setup recorded above. It includes source/edit/reload, all retained scientific-mode
callers, active cancellation/process replacement, recorded-point replay, later context
correction, protected reads/pruning, data-only observation and actual project consumers.
The D-owned uncertain-acknowledgement/fence tests keep their earlier conditions; they were
not silently counted in these 56. No configured user store was provisioned or changed.

**Tested:** legacy existing-state controller/refusal controls pass **362 tests and 422
subtests, zero failures**, using the same native profile and
`.venv/bin/python -m pytest -n16 --dist load scripts/tests/test_surreal_server.py scripts/tests/test_plan30_lifecycle.py scripts/tests/test_surrealism_module.py --basetemp=<private-temporary-directory> --tb=short -q`.
Their fixtures use current typed registered-context and immutable generation declarations;
new Python construction refuses before network/filesystem effects. A backup drain refusal
may leave its intentionally reserved destination empty, but never copied payloads or a ready
backup. Existing closed-admission and original-context refusals remain asserted.

**Tested:** `scripts/pse-env --native=compiler,solver,klu -- just governance-tests`
passes **25 tests, zero failures/skips**, run `fa89757e-7e88-4c23-a5ae-2b5383c413c2`.
The focused schema-v2 checkpoint test passes one case, zero failures, 505 excluded, using
`just unit-package pse-operations schema_v2_checkpoint_binds_original_request_and_closed_intermediate_identity --features canonical-tests,native-service,canonical-maintenance,test-support`
under that profile. The retained store-free Python adapter refusal, eight facade/envelope
controls and five projection-generator controls keep the focused results above.

**Interface-checked:** the initial `just hygiene --live` has seven unsuccessful components
against zero and remains a failed aggregate. Subsequent affected reruns pass Python
lint/types/imports, default and no-default workspace Clippy, dependency analysis, and
`just python-stubs-check --extension target/debug/lib_native.so` against the actual newly
built image. The installed older image was not represented as current. Other hygiene
components, including generated equivalence, contracts, family checks and rustdoc, passed
in the original aggregate. `just ready` passes after removing an unused runtime dependency
edge; no dependency versions changed. Documentation publication passes 904 chapters.
`just turn-end` passes formatting and ADR-index checks. The final comprehensive Clippy
selection follows the maintainer's explicit scope-end request below.

**Proposed / not_run:** full H assembled/reference/performance and persistent-production
qualification remain open. `just features-powerset` is unsuccessful in both
`features-combinations` and `features-no-default` (exit 125 before compilation): native
admission reports no completed isolation provider. No matrix campaign or feature combination
ran. Subsequent comprehensive Clippy preparation admits matching pinned isolation/UNO/PETSc
installations through the existing native snapshot owner, removing that observed prerequisite;
the matrix campaigns were not rerun. H owns their actual later execution. No smaller matrix,
Python fallback or unprepared native environment was substituted. There is no measured
speedup claim or claim that retained optional Python adapters are fully qualified.

### Final comprehensive Clippy and receiving controls

**Interface-checked · 2026-10-29:** every following workspace/all-targets mode passes against
zero, with **zero errors and zero warnings**. All use `--keep-going`, the pinned toolchain,
`--locked`, explicit Arrow correctness admission and `-D warnings`:

| Mode | Command | Result |
| --- | --- | --- |
| Default | `scripts/pse-env -- just clippy-default` | passed |
| No default features | `scripts/pse-env -- just clippy-no-default` | passed |
| Ordinary linked solver workspace/conformance/benchmarks | `just lint-solver-contracts`, with the prepared native provider inputs | passed |
| All workspace features, including taskdump | Command below, with the same provider inputs | passed |

```sh
RUSTFLAGS='-C link-arg=-fuse-ld=mold --cfg tokio_unstable' \
  scripts/pse-env --native=compiler,solver,klu,isolation,uno,petsc -- \
  scripts/cargo-bin/cargo-pse-correctness clippy --keep-going --workspace \
  --all-targets --all-features --profile diagnostic-taskdump --locked -- -D warnings
```

The explicit adopted `PSE_ROOT_ISOLATION_DIR`, `UNO_DIR` and `PETSC_DIR` inputs are those
recorded in the selected native provider receipts. The pinned UNO qualification matches
Cargo's current HiGHS archive; retained input libraries/patches match their receipts and
BLAS selection. The ordinary mode remains separate from unstable diagnostic compilation.
Clippy establishes static/interface coverage, not execution of optional scientific features.

**Tested:** all **13 correctness-adapter process controls pass**, zero failures/skips, with
`scripts/pse-env -- scripts/cargo-bin/cargo-pse-correctness nextest run --locked -p pse-host --test correctness --no-fail-fast`
(run `ee467f48-fc6e-4ccc-9827-595ba8eaa9b6`). The native solver recipe initially refuses because
the adapter forwards Clippy's `--no-deps` into `cargo check --unit-graph`. The graph translation
now removes that linter-only flag while preserving actual Clippy arguments, continuation,
selected target/dependency coverage and original stdout/stderr/exit. The process regression
requires `--no-deps` exactly once at the actual linter, and `--keep-going`/`--all-targets` at
both graph and linter. The host pair is explicitly refreshed; `just ready` and `just turn-end`
pass afterward.

**Tested:** **10 affected canonical-service controls pass**, zero failures, 515 excluded,
at 16 nextest lanes (run `fdad6f2d-0f39-4537-b2af-84cf6a8f88b1`). The explicit private-service
module/helper setup above supplies
`nextest run --locked -p pse-operations --lib --test native_service --features canonical-tests,native-service,canonical-maintenance,test-support --no-fail-fast`
through the correctness adapter, selecting the six `canonical_studies_` controls in that binary,
`native_control_http_actor_grants_original_receipts_and_raw_envelopes`,
`native_service_current_migration_joins_actual_ingress_and_reconstructs_ready`,
`native_service_closed_admission_preserves_original_attempt_settlement_only`, and
`native_old_publisher_settles_while_migration_is_closed_before_successor_activation` in the
library. This rerun exercises the changed HTTP/error-propagation, progression, exact schema
refusal and borrowed publisher-drain helpers using actual private native service owners.
No existing deployment is opened or mutated.

### A mistake made and corrected

The initial script-control command omitted the explicit native library profile and therefore
failed fixture setup; the corrected parallel run supplies compiler/solver/KLU and passes.
Earlier properly configured controls then exposed stale persisted-generation/context fixtures;
those now consume actual typed current declarations rather than weakening the assertions.
Some backup assertions incorrectly demanded destination absence after reservation; independent
source/contract review confirmed that an empty reserved directory is permitted. Assertions
now require emptiness and absence of ready backup content while preserving the drain refusal.

Scope-end lint also found a stale benchmark data-service argument, overly broad test-method
feature gates, redundant borrows, unused dependency/code and non-diagnostic unreachable test
arms. Those are corrected at their owners. An expanded workspace selection additionally finds
11 canonical-service test-helper diagnostics; original errors now propagate through the
existing fixture owner, and exact status/body/refusal assertions remain. The all-feature
selection finds and corrects a second retained-data benchmark argument. Test fixture failure
custody now retains its original
undrained owners in a typed process-lifetime holder rather than forgetting them; it reports
original diagnostic paths and makes no asynchronous-cleanup or success claim. A diagnostic
print cannot precede establishing that custody. The earlier scientific fixture/borrowed-result
mistakes and corrected acceptance remain recorded above; no tolerance was relaxed.

The first all-features Clippy invocation incorrectly uses the ordinary profile for Tokio's
explicit unstable taskdump feature; it refuses compilation for missing `tokio_unstable`.
The corrected command uses the existing `diagnostic-taskdump` profile with the required
opt-in and preserves the repository's mold linker flag. Native provider adoption also exposes
a real copy-permission bug: immutable input metadata stays read-only in a private stage and
cannot be relocated. The strengthened existing regression reproduces PermissionDenied before
the fix; `just unit-package pse-host providers_external` then passes two controls, zero
failures, 132 excluded (run `567a9ac6-7a25-4c3e-8c66-c97b91a237c4`). Only private staged copies
gain owner-write permission; original bytes/modes and final sealed publication are asserted.
The host tool pair is explicitly refreshed and `just ready` passes after preparation.

### Deviations from the plan, deliberate

A06 retains the existing library-owned adaptive actor; an external batch ask-and-tell extension
remains conditional on a real consumer. No second optimizer, mandatory CLI, foreign ABI,
general execution registry or new resource cap was introduced. Existing legacy state controls
remain only for their actual F/G consumers; their unreachable fresh Python constructor was
deleted. The native fixture owner is not advertised as a general operator provisioning CLI.

The raw colleague-note bodies remain unchanged; required title metadata and Reference search
classification were added solely to permit publication without promoting them to architectural
authority. Concurrent test-audit edits are preserved outside this implementation's commits.
The unavailable feature matrix is handed to H instead of reopening paused B/C/D. All local
passing results retain their selected scope and do not close the series.
