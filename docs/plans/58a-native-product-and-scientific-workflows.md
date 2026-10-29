---
title: Native product and scientific workflows
status: in-progress
date: 2026-10-28
adrs: []
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, architecture, libraries, validation]
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md, docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md, docs/functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md]
---

# Native product and scientific workflows

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
| [A01](#a01) Native application contract | in progress | Native workflow owner | Accepted shared contract and complete retained scientific caller scope |
| [A02](#a02) Incremental workspace and source workflows | in progress | Authoring/compiler owner | Native revision/edit/reuse behavior and source/history consumers |
| [A03](#a03) Prepared local execution and results | in progress | Runtime/math owner | Typed author→prepare→execute→inspect composition independent of Python |
| [A04](#a04) Complete scientific mode callers | in progress | Scientific workflow owners | Every supported mode constructible and failures honestly scoped |
| [A05](#a05) Native studies and replay | planned | Study/runtime owner | Generated studies, durable handoff and recorded-point replay |
| [A06](#a06) Adaptive study decision and retained strategies | planned | Study strategy owner | Bounded comparison decided; retained native strategies migrated |
| [A07](#a07) Consumer cutover and assurance handoff | planned | Native workflow integrator | Replaced consumers deleted and required H journeys ready |

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
