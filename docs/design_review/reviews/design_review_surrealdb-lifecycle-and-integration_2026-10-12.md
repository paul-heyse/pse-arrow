---
title: SurrealDB lifecycle, system boundaries and integration target review
date: 2026-10-12
reviewer: Fresh independent principal design reviewer
tier: design
purpose: target
standard: core-3.4
profiles: [process-simulator-1.5]
baseline: 677394474a543c884a8a21d1a2626a6820ff834e
status: review
---

# SurrealDB lifecycle, system boundaries and integration target review

**Decision: Revise.** Retain the existing scientific authority, canonical source selection, immutable result publication and authentic operational ownership. Revise the scientific dependency/retention model and connection lifetime contracts before accepting this subsystem as the best target for concurrent local simulator work. SurrealDB remains a credible persistence engine; this assessment establishes neither a need to replace it nor an unconditional endorsement of every current integration choice.

The implementation already separates authored revisions, preparations, runs, attempts, scientific outcomes, readers and service generations. It has substantive conflict fences, explicit retirement, bounded cleanup and recovery. The papers' recommendation to introduce those concepts therefore overstates the missing machinery. The consequential gaps are narrower: selected external starts lack an independently expressible durable obligation; whole-run retirement and permanent occurrence history tie scientific preservation to broader lifetimes than consumers need; a shared physical connection can be closed without representing every dependent borrower; session restoration and exceptional admission completion have incomplete terminal contracts; and pure study policy still shares a broader integration compilation/test target. These are independent causes. Replacing all of them with a universal lifecycle controller would obscure their owners.

**Architectural fitness: not acceptable in the current target, G9 fails. Behavioral/semantic adequacy: incomplete and partly unresolved. Overall: Revise.** No fresh runtime tests, services, probes, measurements, installs or destructive operations were performed. Code paths are **Implemented** source evidence; inspected external APIs are **Interface-checked**; remedies and their benefits are **Proposed**. This review makes no new **Tested** or **Measured** claim.


| Reference | Principal concern | Priority |
|---|---|---|
| [F01](#f01) | Explicit external seed: retained input versus late-bound availability | High target decision |
| [F02](#f02) | Independent artifact/reference lifetime versus whole-run retirement | Medium; growth-dependent |
| [F03](#f03) | Detailed scientific history versus minimal replay fences | Medium |
| [F04](#f04) | Borrower release versus authoritative physical connection close | High |
| [F05](#f05) | Finite acknowledged session restoration | Medium; availability-dependent |
| [F06](#f06) | Exceptional non-payload admission completion | Medium |
| [F07](#f07) | Actual production policy compilation/test dependency boundary | Medium |

## Review contract and evidence boundaries

This is a design-tier, target-purpose review of the scientific persistence/lifecycle subsystem and the operational and library integrations needed to understand it. Criteria are Core 3.4, its efficient-architecture heuristics/template, Process Simulator 1.5 and the functional target located through the pse-arrow binding. ADRs, plans, pins, blueprint mechanisms and working rules describe the subject; agreement with them is not an acceptance criterion. Their proposed impacts appear below.

The target is a local-first workstation with concurrent processes/workers: value edit → re-solve without structural rebuilding; studies including sweeps, sensitivity, optimization and fitting; recycles/tears; dynamics over the same definitions; unit/property/solver extension; responsibility-local compilation and tests. Model size, case count, trajectory extent, competing callers and interruption affect execution fit. No invented capacity SLA, mandatory numerical checkpoint feature or distributed control plane is assumed. The latest user steering selects the **SurrealKit 1.0 beta line** as target tooling; this review uses **1.0.0-beta.6** as the exact assessed release. Its 0.7 stable line is context, not the target fallback. The separate SurrealDB Rust SDK/server baseline remains 3.3.0. Remote/shared deployment is an extension scenario needing an explicit binding, not a currently supported profile.

Baseline is commit `677394474a543c884a8a21d1a2626a6820ff834e`, plus user-owned untracked `docs/surrealdb_white_paper_code_intelligence.md`. Both that paper and `docs/white_paper_on_surreal_db_in_simulation_state_management.md` were treated as leads. The untracked paper was preserved. Plan 36 remains **STOPPED**; this review does not restart qualification or authorize product, plan, policy or service changes.

Intended principal artifact: `docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md`.

The principal independently read decisive study/retention/schema, runtime/Python close, result-drain and vendored bounded-WS source, selected operational eligibility/backup/reservation source, the selected standards and both paper proposals. Supporting investigations cover broader details, reconciled here:

- [Scientific lifecycle and retention](../evidence/surrealdb-lifecycle-and-integration-2026-10-12/scientific-artifact-lifetimes.md): canonical operations, preparation reuse, studies, starts, results, analysis and retirement.
- [Connection and transport lifecycle](../evidence/surrealdb-lifecycle-and-integration-2026-10-12/connection-and-transport-lifetimes.md): exact patched SDK paths, borrowed connection ownership, restore, exceptional completion and local testing seams.
- [Library capabilities and tooling](../evidence/surrealdb-lifecycle-and-integration-2026-10-12/library-capabilities-and-tooling.md): exact-version native capabilities and both papers' proposals, including SurrealKit and testing alternatives.
- [Operational lifecycle and recovery](../evidence/surrealdb-lifecycle-and-integration-2026-10-12/operational-lifecycle-and-recovery.md): supervisor/resource metadata, fixture/evidence retention, backup/restore and physical-reclamation distinctions.

Read-only `git rev-parse HEAD` confirmed the baseline; `git status --short` exposed the user paper and concurrent review evidence, all preserved. Source searches/reads completed. Runtime tests/probes, compilation, services, installs and measurements were **not_run**, as assigned; no failure count or product pass is inferred.

These are supporting evidence, not competing overall decisions or disposition ledgers. No stale `docs/library-utilization.jsonl` or `docs/capability-maps/` was used. No unit/property model conformance suite, numerical algorithm, current store contents, physical power-loss recovery, remote deployment or representative end-to-end performance was qualified. Existing test bodies were read as possible controls; reading them is not an executed pass.

## The target should follow obligations and operations

The smallest sufficient taxonomy distinguishes what science means, what may execute, what must remain reachable and who may perform an effect. A payload is not intrinsically temporary, recoverable or permanent: several obligations can apply simultaneously. A terminal solve, sealed output, reusable start, numerical restart checkpoint, runner outcome and drained transport are different facts.

| Concept and owned operation | Authority and consumed contract | Lifetime and effects |
|---|---|---|
| Authored definition/revision: admit, edit, select | Modeling owner defines physics and immutable revision meaning; canonical source owner supplies exact selection | Authored edits create revisions; case binding does not mutate the definition; head/history/dependent roots protect needed closure |
| Case and analysis demand: bind, select mode/output | Scientific operation owns variable roles, values, mode, policy and requested extent | Values change numerical work/results without necessarily changing structural preparation |
| Prepared description: establish, rebind, reuse | Compiler/math/provider owners determine complete semantic dependencies and compatibility | Immutable mathematical validity, process-local residency and durable source reachability remain separate |
| Logical request and actual attempt: admit, claim, renew, cancel, close, settle | Workflow owns intended operation; execution owner owns actual mutable workspace and fenced authority | Retry creates an actual attempt; expiry revokes authority and records truthful loss, not inferred success |
| Result: stage, close membership, assess, publish, read | Scientific owner determines original assessment/usability; substrate enforces exact membership/visibility and current permission | Complete, partial, failed, absent and empty retain different meanings; copied immutable bytes may outlive DB permission |
| Start artifact: qualify, acquire, consume | Start/solver owner determines numerical role, original permission, compatibility and exact used values | Pending durable consumers need an explicit retained dependency or a deliberately late-bound availability contract |
| Restart checkpoint, **if selected**: capture, validate, resume | Relevant simulation/horizon owner defines sufficient restart state and equivalence | New attempt depends on the checkpoint; seed/trajectory/history/backup alone does not establish restart completeness |
| Analysis/reference: derive, activate, promote, release | Analysis/scientific artifact owner defines sufficient dependency closure; retention owner governs durable obligations | Promotion references immutable content or deliberately copies a self-contained product; it does not rerun or improve original science |
| Replay/effect fence: settle, refuse resurrection | Substrate transition owner defines original grant and irreversible closure | Minimal replay safety can outlive detailed scientific history without retaining every diagnostic/request forever |
| Test/evidence resource: register, borrow, reconcile, release, reclaim | Runner/producer owns terminal outcome, evidence roles and exact resource ownership | Passing, age or size alone does not authorize deletion; unresolved effects and failure evidence remain protected |
| Service/connection: select, borrow, release, restore, close | Deployment/transport owner governs authenticated incarnation, admission, actual driver drain and maintenance | Logical borrower release differs from authoritative physical shutdown; DB settlement and native drain remain independent |
| Physical storage: remove catalog, reap keys, compact | Engine/backend owns storage effects under the authorized higher-level operation | Logical disposal acknowledgment differs from physical key/file-space reclamation |

This taxonomy does not prescribe one table, crate, task, lease or service per row. Existing types, functions, immutable manifests and explicit references can realize most distinctions. Scientific semantics stay in Rust/model/native owners. SurrealDB can execute transactional selection/publication/reference operations without becoming numerical authority. Active trial arrays, evaluator workspaces and native factors stay process-local unless a specific scientific checkpoint contract requires otherwise.

### Responsibility boundaries and local tests

| Responsibility | Hidden decision; dependency direction | Local test context |
|---|---|---|
| Authored model/compiler/math/native solvers | Physical meaning, formulation, structural reuse and numerical method; workflows consume these operations | Model/provider/native tests for their actual contracts; storage need not be started for pure scientific policy |
| `pse-runtime::workflow` and pure `study_policy` | Scientific orchestration and fresh/continuation/retry decisions over explicit observations; adapters establish authority | Typed policy inputs independently; integration tests for actual canonical transitions |
| `pse-schema` → `pse-codegen` → `pse-operations` | One declaration of durable fields/relationships, generated DDL/codecs, typed operations and exact read/publication/retirement | Generated-schema and transactional fixture tests; consumers avoid parsing incidental SQL/storage state |
| Canonical transport and vendored SDK/engine-api | Original clocks, dispatch uncertainty, frame/byte owners, admission, ordered restoration and physical close | Existing local protocol-peer and lifetime-control seams; native science is unnecessary for transport failure policy |
| Supervisor/resource store/test-resource policy | Exact service/process identities, independent metadata durability, authenticated removal and recovery | Filesystem/process fixture controls with authentic ownership; no selected production-store mutation |
| SurrealDB/RocksDB and optional tooling | Transaction/query/index behavior, physical reclaim and suitable migration/test support | Exact released server/backend/profile integration; embedded memory tests cannot qualify WS/RocksDB durability |

Bootstrap/recovery metadata must remain available while the database is down or fenced. Moving the supervisor's state or test-removal authority into the DB it supervises creates a circular recovery dependency. Conversely, copying scientific facts into JSON operational records creates another scientific authority. A read-only inventory may compose owner projections; it should not become a second status or deletion owner.

### Physical semantics and numerical stages

The substrate's numerical responsibility is faithful preservation and dependency closure, not formulation or scientific certification.

| Quantity/model element | Dimension/unit and basis | Reference state/convention | Validity envelope | Authority and substrate obligation |
|---|---|---|---|---|
| Model variables, streams and result quantities | Authored admitted schemas/rows; mass/molar and rate bases remain explicit | Declared energy references, pressure/sign conventions | Model/provider declarations | Modeling/quantity/provider owner; preserve exact interpretation and identities |
| Warm/primal/dual/basis/prediction starts | Original typed start role/layout; some coordinates are scaled or dimensionless | Backend/scaling/branch/start conventions | Original assessment plus consumed compatibility | Scientific start owner; retain exact used values/provenance and dependency, not mere terminal flag |
| Dynamic state/trajectory | Physical time, states/rates and requested versus observed extent | Integration/event/discretization policy | Selected dynamic formulation and numerical assessment | Simulation owner; partial trajectory is not automatically a restart checkpoint |
| Sensitivities/covariance/derived analysis | Target and parameter dimensions/bases | Active-set/statistical/local-linearization conditions | Qualifications or explicit withholding | Analysis owner; substrate retains original conditions and exact inputs |

**Well-posedness:** variable roles, degrees of freedom, structural rank/decomposition and model-identity diagnostics belong to formulation/structural owners before solving. The store must preserve selected definitions and assessments; a hash, lease, manifest or successful commit cannot establish well-posedness. The end-to-end well-posedness and physical/numerical mechanisms were not audited here.

| Stage/owner | Structural versus value inputs; reuse/effects | Formulation, derivatives, scaling, solver class | Outcome/tolerance/post-solve obligation |
|---|---|---|---|
| Admission/preparation/compiler | Selected structure/provider/policy dependencies; bind changing values separately | Preserve authored guards and derivative/scaling contracts; no DB numerical method selection | Pre-solve admission and diagnostics remain with scientific owners; comprehensive numerical audit not performed |
| Evaluation/solve/native owner | Attempt-local values, workspaces, factors and selected starts; no DB transaction held over numerical work | Library-owned class-specific mathematics/solvers; derivative order and approximation remain explicit | Independent new-state residual/domain/bounds/closure checks remain required; this review does not certify them |
| Publication/read/analysis | Immutable exact blocks, closed manifests, current permission and original assessment; bounded pages | No new formulation/derivatives; preserve original physical/schema/qualification context | Successful transport/seal does not imply solution; local-analysis withholding is a positive source route |
| Recovery/restart | Exact operation/attempt identity and truthful loss today; checkpoint dependencies only if selected | Workload-specific restart equivalence cannot be inferred from VERSION or a seed | Original observations survive; reexecuted work and new attempts retain honest attribution |

## Representative changes and failure journeys

| Scenario | Expected boundary and observed impact | Evidence/acceptance |
|---|---|---|
| S01 Value edit → re-solve, including adjacent sweep cases | Rebind values without structural reconstruction. Study preparation lane retains structural target membership; compatible semantic bodies rebind source selections | Source route exists in `study_operations.rs` and `modeling/preparation.rs`; all-mode interactive latency/reuse remains unqualified |
| S02 Retained study selects an external seed, then producer retirement/retry/restart | Establish exact durable dependency or explicitly choose late-bound availability; do not let provenance imply protection | F01; both admission/removal orders and restart before dispatch distinguish the contracts |
| S03 Preserve a useful seed/final state/analysis while discarding unrelated trajectory/attempt detail | Scientific owner declares sufficient closure; multiple obligations survive independent releases | F02/F03; current whole-run pin is safe but coarse, permanent occurrence history remains policy-coupled |
| S04 Close one runtime/borrower during durable renewal or publication | Caller release cannot accidentally tear down another dependent; authoritative close must refuse or own cancel/join | F04; current protected-reader refusal is valuable but does not express every durable borrower |
| S05 Reconnect with missing setup ACK or malformed non-payload reply | Finite restore outcome; no partial-session dispatch; terminal admission independent of map reclamation | F05/F06; mock peers and held metadata can isolate the contracts |
| S06 New unit/property/solver or study mode, including recycles/dynamics and local policy compilation/tests | Extend scientific owner/binding; reuse shared model and operation composition. Recycle convergence is numerical strategy, not persistence graph traversal | Existing operation composition is positive; F07 limits pure-policy compile/test isolation; numerical families/conformance suites unexamined; no universal plugin framework required |
| S07 Replace transport/schema tool, or add remote/shared storage | Preserve complete consumed value, clock, uncertainty, drain and schema contracts. Remote needs explicit host/auth/deployment authority | Alternatives below; current loopback restriction is honest, not proof a remote binding is impossible |
| S08 Reclaim disposable fixtures, crash during maintenance, backup/restore | Authentic ownership/borrows/references/outcome/drain govern effects; unknown stays pinned/fenced | Operational source supports this contract; no fresh crash/recovery or physical-space qualification |

## Material findings

F02–F07 describe current source-established structural causes with **Proposed** remedies. F01 records a source-established availability distinction and unresolved target contract choice; it is not an asserted violation of a demonstrated early-binding promise. The future owning plan, if authorized, must hold dispositions; the identifiers here remain stable historical observations.

<a id="f01"></a>
### F01 — External selected starts have provenance but no independently governed durable dependency

**Priority:** High for retained seeded work and retry/restart. **Principles/gates/scenarios:** AP-04/AP-05, DP-07/DP-09, PS-08/PS-11; G6/G9 unresolved for the binding choice; S02.

**Evidence:** `workflow/study.rs::start_defined_study` admits binding/seed-role compatibility and persists `StartPolicy::Explicit`, but its `RunRequest.sources` retain modeling/physical revisions. Exact seed existence is not admitted there. `prepare_study_candidate_in_lane` acquires the seed later through `Operations::seed`. `surreal_retention.rs::forget_run` protects live readers, the producer run's studies and `canonical_analysis_inputs`; `collect_run` deletes original seed descriptors/completion. The schema and those operations contain no external selected-seed retention obligation. This matches the contrasting analysis-input protection handoff in `canonical_analyses.rs`/`surreal_analyses.rs`.

**Consequence:** a valid seed available when a retained study is requested can disappear before that study acquires it, retries or restarts, after the producer's own obligations are released. The consumer truthfully reports missing input; no silent fresh-start fallback or wrong numerical answer is alleged. The current API has not been shown to promise early-bound availability. The target gap is the missing explicit choice between a retained scientific dependency and deliberately late-bound availability. Provenance alone cannot support reproducible pending/retry work.

**Correction:** the scientific admission owner must choose and expose either (a) an exact admitted start with a durable dependency or self-contained copied artifact, or (b) a late-bound request whose availability can fail later and whose retry/restart promises reflect that choice. An exact admitted dependency is the preferred target for retained reproducible studies. Atomic admission/retirement must cover both commit orders. A conservative producer-run obligation can close loss first; artifact granularity is a separate improvement. Already-owned `AdmittedSeed` bytes remain usable without reacquiring producer permission.

**Remedy maturity/alternative:** Proposed. Late binding is legitimate for an exploratory request, so do not reject all such work or require needless copies per point. The public distinction and release boundary need design; the diagnosis does not depend on requiring a new seed store.

**Verification:** retain B's exact selected input, release A's original obligations, race admission/deletion both ways, restart before dispatch, retry after worker loss and finally release B. Either the exact original input survives or deletion-first causes explicit admission refusal; late-bound mode must instead disclose its availability semantics. Check original bits/permission/compatibility unchanged.

<a id="f02"></a>
### F02 — Whole-run retirement cannot express independent useful artifact lifetimes

**Priority:** Medium, rising with trajectory/case growth. **Principles/gates/scenarios:** AP-04/AP-07, DP-05/DP-19; G9; S03.

**Evidence:** `canonical_result_retirements` and `forget_run` withdraw a run; `collect_run` traverses all generations/result sets/seeds. Exact read/analysis selections exist but retaining one such analysis prevents whole-run retirement. `RetentionOwner::History` retains source revision history, not independently selected result content. No canonical selective promotion operation was found in inspected schema/retention/start/analysis owners.

**Consequence:** keeping one qualified seed or a small analysis requires preserving unrelated run outputs/attempts, or exporting them into a caller-owned lifecycle. The unit of retention is broader than the useful scientific dependency. This is structural retention amplification, not a measured disk/capacity failure and not a blanket requirement for deduplication. Upstream compact output demand helps new runs; it cannot independently release portions of an already retained full result.

**Correction:** permit an obligation/promotion operation over a scientific-owner-defined immutable selection and its sufficient closure, independent of the originating execution. Consumers receive stable reference identity and stop managing unrelated producer lifetime. Preserve original assessment, provenance, exact source/start dependencies and live-reader protection. Reference promotion need not copy payload; a deliberate self-contained copy may be simpler for a small start.

**Remedy maturity/alternative:** Proposed. Whole-run pins are a valid conservative initial implementation if the target explicitly accepts their scope/cost and does not claim selective canonical promotion. A scalar report, auditable sensitivity and restart state need different sufficient closures; arbitrary caller-selected byte subsets must not become certified artifacts. No generic content-addressed chunk store is mandated.

**Verification:** promote a seed/selected trajectory/analysis dependency, release the original run/study, retain independent borrowers, release them in different orders and restart mid-collection. Required content/assessment survives; unrelated payload can drain; invalid subset promotion is refused. Quantitative capacity benefit remains to measure if claimed.

<a id="f03"></a>
### F03 — Detailed scientific occurrence history is permanently coupled to replay-safety receipts

**Priority:** Medium for repeated exploratory studies. **Principles/gates/scenarios:** AP-01/AP-04/AP-07, DP-05/DP-19/DP-21; G9; S03/S08.

**Evidence:** `forget_study` writes a retirement marker. The schema/API preserve occurrence/scientific outcome receipts. `collect_run` removes payloads, append operations and original completion, but leaves run/attempt headers and non-append effect requests. Study points retain operation/binding descriptors, policy, facts, outcomes, start and attempt history; dependencies/readiness/cursors remain. Some of this is scientific history, not minimal effect identity. Existing settled protection/analysis collection and append-residue cleanup are positive counterevidence to a claim that every old tombstone still accumulates.

**Consequence:** even an explicitly disposable completed experiment leaves a scientific-history footprint proportional to occurrences and detailed requests, without an independent scientific/audit retention owner. Bounded collection pages bound one effect, not total permanent retention. Legitimate audit history should survive by selected obligation rather than as an incidental requirement of every replay fence.

**Correction:** separate deliberately retained scientific history from minimal irreversible effect identity/fence/terminal attribution. The owning user/project/scientific workflow releases its history obligation explicitly; cleanup acts only after grants, unknown effects and borrowers are settled. Old identity replay must neither resurrect content nor reinterpret unavailable history.

**Remedy maturity/alternative:** Proposed. A project requiring complete permanent audit may retain all current detail; that is a legitimate selected policy and counterexample cleanup must preserve. Compaction/archive or explicit release can be compared with deletion. Age-based destruction, a timer or a new audit service is unnecessary.

**Verification:** release a terminal exploratory study after exact settlement and inspect fixture-owned residue; deliberately audit-retained history and unknown effects remain, unselected details drain restartably, and old replay cannot resurrect them. Define minimal retained receipt semantics before deleting anything.

<a id="f04"></a>
### F04 — Caller close and authoritative physical close lack a complete durable-borrower contract

**Priority:** High for concurrent runtime use. **Principles/gates/scenarios:** AP-01/AP-02/AP-04/AP-05/AP-07, DP-19/DP-20; G9; S04.

**Evidence:** Python `NativeRuntime.close` clears caches then calls `CanonicalStore::disconnect`. That method drains `ResultReadDrain` and closes ordinary/activation clients. `Runtime::from_shared` and `Heartbeat::spawn` retain cloned `Operations`/store for renewal and publication. SDK disconnect closes shared admission and route drivers. Protected readers can cause a safe refusal; no equivalent durable-dependent borrower is represented at the close boundary.

**Consequence:** when no counted protection happens to remain, closing one handle can revoke a live dependent's renewal/publication transport; in other phases close refuses. Particular workflows may hold protections throughout and therefore avoid this manifestation. That is not a complete durable borrowing contract, especially for public store/Operations aliases. A physical force-close operation is legitimate, but a caller-visible runtime close needs explicit scope and authority. No inconsistent scientific result or reproduced native failure is asserted.

**Correction:** distinguish logical caller release from authoritative physical shutdown. The connection owner atomically closes new admission and either refuses with actual live dependents or owns their completion/cancel-and-join before transport teardown. State whether runtime `close` releases this caller, refuses active work or cancels its own work. Independent borrowers retain required capabilities through renewal/publication settlement. Native drain, local transport drain and DB effect settlement stay separate.

**Remedy maturity/alternative:** Proposed. A small activity guard or existing runtime ownership can suffice; `Arc::strong_count` cannot distinguish a cache alias from a real dependent. Preserve protected-reader refusal and explicit fixture/operator transport shutdown. Releasing one shared caller must not cancel another's work.

**Verification:** local close-versus-renewal/export and admission races; independent borrower release; refusal or final-dependent completion followed by actual driver close. One integrated durable journey checks wiring; the borrower policy itself needs no numerical solver.

<a id="f05"></a>
### F05 — Ordered session restoration has no finite acknowledged terminal contract

**Priority:** Medium, potentially high for post-restart interactive availability. **Principles/gates/scenarios:** AP-02/AP-05/AP-07, DP-12/DP-19/DP-20; G9; S05.

**Evidence:** in the production bounded WS path, `ReplayCursor` stores awaiting/index/refreshed without a deadline. `run_bounded_router` times out `replay_session`, but that call sends the first setup command and returns before the ACK chain completes. `maintain_request_lifetimes` handles ordinary pending/deferred requests, not restoration age. `send_replay_command` also returns the same boolean for exhausted replay and failed transmission; the ACK branch can treat `false` as permission to flush deferred work. The bounded connection report identifies exact source/test paths.

**Consequence:** with an open socket and missing setup ACK, callers expire correctly but future work repeatedly parks behind the same replay cursor. Caller bounds do not establish a recovered or terminally failed session. Ambiguous failed-send/exhausted state additionally relies on later IO failure rather than explicit successful restoration. No wrong-database write is claimed.

**Correction:** one finite restoration operation per session/socket incarnation, owning ordered acknowledged setup, an independent finite lifetime and ready/failed/disconnected outcomes. Only fully acknowledged exhaustion admits application routes. Missing ACK/send failure keeps them undispatched and reaches explicit failure/reopen or selected reconnect, preserving callers' original clocks and unknown application effects.

**Remedy maturity/alternative:** Proposed. Automatic retry is optional; terminal failure plus explicit reopen is simpler and valid. One caller's cancellation must not abort restoration still useful to others; slow valid setup may have its own finite budget. Do not replay uncertain application writes.

**Verification:** mock peer keeps socket/ping traffic while withholding one setup ACK, then acknowledges before bound or never acknowledges. Check readiness/failure, bounded recovery, no partial-session dispatch, original request expiry and a single setup-send failure distinguishable from successful exhaustion.

<a id="f06"></a>
### F06 — Malformed non-payload reply paths omit explicit finite-admission completion

**Priority:** Medium. **Principles/gates/scenarios:** AP-02/AP-05/AP-07, DP-20; G9; S05.

**Evidence:** `handle_response_with_id` and connection reset explicitly call `AdmissionPermit::release`. Non-payload `handle_parse_error` and `fail_all_pending_requests` take pending entries and return errors without that release. The exact locked `surrealdb-types==3.3.0` Papaya wrapper removes and clones values; pending clones share admission permits, so local clone drop is not independent of retired map/snapshot lifetime. Payload-owned malformed replies already force physical invalidation before these branches, correctly preserving their stronger drain boundary.

**Consequence:** finite admission slots can remain charged after errors are delivered and correlations removed, dependent on incidental epoch/snapshot reclamation; no reachable pending entry remains for reset to settle. Repeated errors can impede later work. This is source-established exceptional completion inconsistency, not a new claim that the fixed payload-owned late-response drain is broken.

**Correction:** terminal removal must own an idempotent explicit admission completion, separate from actual bytes/socket drain and DB settlement. Choose physical invalidation where authentic completion is uncertain; otherwise release after the relevant local handler ownership ends. Consolidation must preserve the distinctions rather than add premature blanket Drop release.

**Remedy maturity/alternative:** Proposed. No map replacement is necessary. A single-owner map is an alternative if broader simplification earns its integration cost. A malformed response never becomes successful DB/scientific completion merely because a slot is released.

**Verification:** retain removed/snapshot pending metadata; inject named malformed and undecodable errors for non-payload requests; assert application/control capacity becomes reusable without forced epoch collection. Preserve the owned-payload malformed-frame physical-drain behavior.

<a id="f07"></a>
### F07 — Pure study policy still shares the storage and columnar integration compilation/test boundary

**Priority:** Medium for ordinary study-policy changes and responsibility-local development. **Principles/gates/scenarios:** AP-01/AP-06/AP-07; G9; S06/S07.

**Evidence:** `pse-operations/src/study_policy.rs` implements effect-free policy from model types, diagnostics, petgraph and collections. Its production package unconditionally includes the canonical storage modules, SurrealDB SDK and Tokio. Its unit-test target includes `testing` under `cfg(test)` and declares `pse-codegen`, `pse-columnar` and Arrow dev dependencies. `pse-codegen` depends on `pse-columnar`; the latter consumes DataFusion common/execution and Arrow/Parquet/object-store capabilities. Filtering a unit-test name selects execution after the package/test target's source and dependencies are compiled; it does not create a pure-policy compilation unit. This follows the actual current manifests/source, not an inference from crate count. Current `pse-codegen` separation and tests moved by Plan 36 are already incorporated in this baseline.

**Consequence:** an ordinary pure study transition/admission change cannot be compiled and tested through its production owner without also compiling/checking unrelated storage and columnar/codegen integration source. An integration compile failure can block those policy tests although no DB/server/native solve is needed at runtime. The functional target explicitly includes responsibility-local compilation/tests. Runtime isolation alone does not satisfy that requirement. No cold-build duration, dependency dominance or measured productivity loss is claimed.

**Correction:** give the actual production policy implementation a coherent compilation/test boundary consuming its real semantic dependencies, and make adapters depend on that owner. Compare relocation into a fitting existing semantic owner, a genuine separated target with appropriate dependency boundaries, or a dedicated policy package only if warranted. Integration schema/transport tests retain their real SDK/codegen/Arrow dependencies. Removing a dev dependency without moving its consumers, filtering test names, or copying the policy into a surrogate test target does not repair the boundary.

**Remedy maturity/alternative:** Proposed. A shared package may be convenient and legitimately group coherent storage behavior, but this policy has an independent semantic variation axis and no storage contract to consume. Avoid moving transport into the model or requiring a new crate merely because the policy has a name. Selection should minimize changed ownership while preserving one production policy authority and all real integration controls.

**Verification:** inspect the selected production policy target and its reachable compile/test dependency closure: a real admission/transition change is exercised without the canonical SDK or unrelated columnar/codegen integration. Preserve adapters calling that same policy and necessary generated-schema/transport integration tests. Report closure separately from measured build latency; no timing threshold follows from this finding.

## Preserved operational guarantees and optional capabilities

Existing lifecycle mechanisms are substantial strengths, not requirements to rebuild under new terminology. `test_resources.py::eligible` requires reconciled pass or explicit release, real drain, known owner kind, no pins/creator borrow/references/borrows and verified producer scratch manifests where relevant. Reclaim claims exact cleanup under short metadata exclusion, performs effects outside it and settles exact outcomes; unknown/interrupted removal stays protective. A closed service can legitimately defer DB disposal rather than restarting someone else's service. `resource_store.py` has explicit migration and fenced-unactivated refusal/recovery controls; a refusal is not evidence a new authoritative metadata store is needed.

`surreal_server.py::lifecycle_reservation` uses exact nonce/process/start/boot ownership and short exclusion. Offline backup stops/drains the selected service and copies database, credentials/configuration, receiver contexts and retained executable generations with a hashed inventory. Restore/recovery preserve source and target identity and old-capability fencing. Logical export is a distinct recovery objective, not automatically equivalent to this complete operational backup. No online filesystem copying or broad namespace deletion was observed in these paths.

The selected RocksDB endpoint uses `sync=every&versioned=false`. VERSION/time travel is therefore not current behavior. `REMOVE DATABASE` acknowledgment is logical disposal; asynchronous key reaping and backend file-space recovery occur later. Neither missing physical-byte witness nor delayed reaping establishes a deletion correctness bug. If future capacity decisions require reclaimed space, observe physical recovery separately; a per-object byte ledger, per-run compaction or generic GC service does not follow from this review.

The current authenticated HTTP MCP inspection route already verifies the selected service identity and supplies existing VIEWER credentials. Starting a new owner-access embedded stdio store would change the owner/store being inspected. Future mutation tools should invoke domain operations with their permissions rather than reconstruct publication/retention through generic CRUD.

### Library fit and ownership cost

The [exact-version capability matrix](../evidence/surrealdb-lifecycle-and-integration-2026-10-12/library-capabilities-and-tooling.md) covers both papers' wider feature inventory and primary sources. The principal recommendations are selective:

| Capability / integration owner | Target choice and limits | Machinery and revisit condition |
|---|---|---|
| SurrealDB transactions/indexes/functions; canonical operation owner | Retain useful bounded server-side operations with checked statement outcomes and explicit conflict decisions. Rust owns scientific semantics | Avoid scalar caller choreography and blind client/server mirroring; investigate actual access patterns where selection scans or joins become material |
| Native typed references / REJECT; schema and retention owners | Adapt where an exact relationship's meaning fits. They cannot see dependencies hidden in strings/bytes, certify start/checkpoint sufficiency or replace creator/reclaimer settlement | Potentially remove redundant structural checks, only after both-order admission/deletion and lifecycle qualification at the resolved backend; do not convert attributed edges indiscriminately |
| Async events/changefeeds/live; workflow/notification owner | Optional post-commit work or refresh hints, not sole protection or durable scientific truth. Finite retry, feed retention gaps, duplicate effects and reconnect subscription loss need explicit semantics | Adopt only for a concrete consumer that removes polling/coordination. The 3.3.1 async retry-budget behavior must not be transferred to current 3.3.0 |
| Current patched WS SDK/engine-api; transport owner | Consumed original clocks, dispatch uncertainty, actual byte/drain facts justify integration below an outer timeout; F04–F06 remain | Retain library codecs/query builders and qualify repaired state machine. Upstream caller timeout/queue capacity alone is not an equivalent alternative |
| Native bytes and bounded Arrow blocks/pages; result owner | Already a useful integrated representation. Keep indexed queryable metadata separate from opaque IPC | Real gRPC incremental streaming is eligible for a demonstrated large-read need, but provisional-row finality/cancellation and the patch contract must survive; no current memory improvement measured |
| SurrealKit 1.0.0-beta.6; generated-schema/migration/fixture owners | **Selected target tooling, Proposed integration.** Use advantageous beta module/target, declarative test and rollout capabilities through the owned seam below; 0.7 is not the fallback | Consume generated DDL and preserve current digest/activation/fixture authority. Beta.6 cleanup discards statement checks and warns on errors; renewal loss warns without fencing SQL. Seed/test targeting differs from sync. These are integration obligations, not current product defects or rejection of the beta target |
| Testcontainers/embedded fixtures; test owner | Eligible where exact binary/network/isolation adds value; default module v2.2/memory/root/allow-all is not this production profile | Existing owned native fixtures are the comparator. Explicit image/backend/caps/readiness/removal and evidence policy remain; declarative DB tests complement scientific and protocol tests |
| Schema evolution; registry/codegen and explicit admin owner | One declaration → generated DDL/codecs; explicit compatibility/activation. Ordinary open must not mutate unknown schema | A rollout tool may replace some migration bookkeeping, not become a second schema authority or bypass admission/maintenance fencing; no rolling remote deployment need established |
| VERSION, buckets, Wasm/Surrealism, lightweight edges/inline caches, search/GQL/hosted APIs | Eligible capabilities, not an implied lifecycle roadmap. History is disabled; files/Wasm add experimental lifecycle/build obligations; lightweight edges lose attributes and receipt identity | Adopt only a consumed contract with meaningful work removed. Inline caches trade writes/overflow for narrow reads. Graph discovery/search capability does not justify database numerical iteration |
| CLI/MCP/Studio/OTel/metrics/timer | Retain current owned inspection/admin paths; optional selective diagnostics/scheduling | A timer is only a wake-up, never deletion authority. Studio is not schema authority. Metrics cannot certify checkpoint validity or reclaim. No mandatory collector, scheduler or plugin installation is proposed |

Full library eligibility remains; no license or current pin was used to reject a technically suitable candidate. The review does not establish end-to-end qualification of these prospective capabilities. Exact primary-source URLs and version limitations live beside the matrix claims.

### Selected SurrealKit beta integration

Target SurrealKit **1.0.0-beta.6** as repository tooling, with the registry/codegen owner supplying schema and the supervisor/fixture owners supplying effect authority. Prefer a standalone tool invocation initially so its independent SDK does not silently bypass the production patched transport contract. A deliberate Rust library integration is eligible if it materially removes adapter work and its lifecycle is qualified. Neither choice changes scientific authority.

Use beta modules and named targets to compose generated scientific/canonical schema, disposable fixtures and declarative database checks. Module paths/ownership must be stable and deliberate because sync tracks managed objects and prunes deletion; a file rename can mean removal/addition. The owning generation must produce the necessary schema, functions, assertions, permissions and enforced endpoints without a second handwritten canonical schema. Establish exact generated-DDL parser/entity-tracking compatibility before allowing sync pruning of persistent state. Beta.6's `schema_from` and schema-parity contracts are **Interface-checked**: suite selection overrides CLI, then defaults, with Sync fallback; Both schedules independent Sync/Rollouts jobs. Sync builds with pruning and the default module. Rollouts replays into an empty suite DB with `complete_newest=true` and refuses an absent rollout chain. If replay jobs exist, synchronization is enabled and global parity is enabled (default), parity runs first; fail-fast can suppress later jobs. Parity compares `describe_schema` definitions/catalog between separate databases, not behavior, data or preservation. These are useful generated-DDL versus rollout regression surfaces, not migration qualification. [Pinned beta.6 runner](https://github.com/surrealdb/surrealkit/blob/v1.0.0-beta.6/crates/surrealkit/src/tester/runner.rs).

Declarative SQL expectations, permissions, metadata, relationship behavior and HTTP/API checks can remove repetitive fixture assertions and exercise database contracts in isolation. They complement transport/cancellation/race/recovery and scientific tests. Fixtures must be registered to authentic ownership before destructive/setup effects where needed, preserve selected failure evidence and publish actual runner disposition/drain. Beta runner success and best-effort cleanup are different outcomes. If the beta runner cannot expose the necessary fixture identity/lifecycle seam, adapt that runner/tool boundary or keep the existing authenticated fixture owner around the selected beta contract tests; do not infer disposal from suite success. Exact seed/test target dispatch must be verified per command rather than inheriting sync's module/target matrix by assumption.

For persistent schema evolution, beta rollout progress and explicit rollback steps can replace suitable private migration bookkeeping. Beta.6 builds against SDK3.3 and requires Rust1.95. Its planned changed SQL is frozen beside each manifest with SHA256; ordered `rollout up` supports catch-up, with fork/hash/predecessor checks. This provides a reproducible generated-schema release artifact, not proof that application cutover or scientific data migration is safe. [Beta.6 migration guide, sections 8–9](https://github.com/surrealdb/surrealkit/blob/v1.0.0-beta.6/MIGRATING.md). The repository still owns closed admission, compatible application cutover, selected-store identity, borrower drain and recovery. `LockKeepAlive::spawn` renewal-loss warnings do not fence ongoing rollout SQL; a multi-writer rollout cannot rely on that stock behavior as exclusive-effect authority. Either an already exclusive owned maintenance lifetime supplies the real exclusion for all participating writers, or the tool integration must stop/fence further effects on lost authority. A lock TTL by itself and an outer caller timeout do not settle submitted effects. Do not demand remote rolling upgrade for the present workstation target.

The next design evidence should settle generated-DDL/parser compatibility, sync/seed/test target dispatch, production rollout resume/cutover/preservation and the actual fixture registration/cleanup seam at beta.6. The inspected `schema_from` and definition-parity contracts are now established at Interface-checked strength; actual integration remains untested. This can change the integration shape and amount of bespoke code removed. It does not change F01–F07 or authorize installation, migration or cleanup now. The selected beta line is an assumption for the proposed target, not a Tested tool integration.

### Alternatives and complete-operation tradeoffs

| Alternative | Fit and cost | Selection / revisit |
|---|---|---|
| Preserve current canonical operations, add explicit scientific obligations, repair close/restore/completion | Reuses exact publication and operational ownership; removes identified semantic gaps without replacing science or service control | Preferred target direction. Detailed artifact closure and borrower semantics still need design; not accepted implementation |
| Retain whole-run pins and permanent complete history | Simplest conservative preservation, easy scope; pays broader retention for small dependencies and every exploratory study | Legitimate materially narrower preservation product only if selected explicitly and no selective-release/promotion capability remains claimed. Does not settle connection findings |
| Copy a small self-contained start/reference product | Can decouple lifetime without introducing shared chunk reachability | Viable simpler F01/F02 remedy when copies are small/rare and scientific closure is complete; compare with references for large shared trajectories |
| Universal lifecycle controller/database for science, fixtures and service recovery | Superficial uniformity; mixes scientific sufficiency, runner outcomes, actual process drain and bootstrap authority | Reject this target direction: creates circular recovery or second authorities without removing actual obligations. A composed read-only inventory is sufficient for discoverability |
| Dedicated small WS actor or HTTP integration using existing codecs/types | Could reduce broad SDK adaptation or mutable replay state; still must own exact serialization, selection, correlation, clocks, cancellation and buffer drain | Credible mechanism-substitution exploration, not established simplification. No replacement is needed to repair F04–F06; compare complete consumed contract before deciding |
| Embedded core in one service, or another transactional local store with explicit relations/manifests | Can change transport/build/access tradeoffs but does not repair missing scientific dependencies or lifetime choices by itself | Eligible if concrete deployment/recovery/build burden warrants research. Multi-process direct RocksDB opens are not equivalent. No comparative replacement qualification was performed |
| External bulk artifacts plus transactional manifests | May suit very large trajectories but adds cross-store publication/reclamation/backup failure boundaries | Defer pending a demonstrated payload-scale benefit. Native bytes are the simpler current comparator; single-platform preference alone does not forbid a better future realization |

A local workstation still has concurrent writers, crashes and late effects, so local-first does not mean deleting authentic fences. Conversely, prospective remote support does not justify importing distributed leases and rolling-deployment machinery into every local operation. The chosen physical units should follow necessary atomicity, demand, reuse and recovery, not the number of taxonomy rows.

## Architectural foundations and independent gates

Foundation verdicts concern the inspected complete subsystem and its representative scenarios. Satisfied means source-reviewed architectural fit in that scope, not runtime qualification.

| Foundation | Verdict | Scenario, evidence and required action |
|---|---|---|
| AP-01 Separation of concerns | **Violated** | F03 couples scientific-history retention to effect fences; F04 lacks a boundary between caller release and physical ownership. Existing scientific/transport/service owners are strengths to preserve |
| AP-02 Stable contracts | **Violated** | F04/F05/F06 leave borrower close, restoration outcome and exceptional admission completion incomplete. Typed canonical scientific operations otherwise hide useful implementation details |
| AP-03 Composition | **Satisfied** in inspected boundary | Study operations compose declared-case/simulation/fit/horizon and pure policy; canonical adapters establish observations/authority. A numerical checkpoint is a possible new concept, not proof existing workflows are duplicated |
| AP-04 Domain model and semantic authority | **Violated** | F02/F03 underrepresent independent artifact/history obligations; F04/F05 underrepresent governing lifetimes/readiness. F01's early-/late-bound choice remains unresolved. Scientific model authority is not shown duplicated |
| AP-05 Explicit structure | **Violated** | F04–F06 miss enforceable lifetime/completion transitions; F01 lacks an explicit durable availability choice. Exact run/attempt/publication/read distinctions already govern behavior |
| AP-06 Local reasoning/testability | **Violated** for compilation/test locality | F07 makes filtered pure-policy compilation/tests depend on unrelated SDK and columnar/codegen integrations. Pure policy runtime setup and protocol-peer seams remain positive; future sufficiency policies must stay with scientific owners |
| AP-07 Execution fits workload | **Violated** | F02/F03 amplify retention, F05 can leave restoration permanently nonterminal, F06 ties finite slots to metadata reclamation; F04 can invalidate dependencies. Preparation reuse and bounded short RPCs are positive; quantitative capacity/latency remains unknown |

| Gate | Judgment | Evidence and limits |
|---|---|---|
| G1 Authority | **Pass, scoped source assessment** | Authored/scientific authority, generated declarations and operational owners remain distinct; no competing mutable science established. Inadequate lifecycle modeling fails AP-04, independently |
| G2 Semantic fidelity | **Unresolved for the full boundary** | Exact selections/typed outcomes and separate empty/partial states are substantive. New artifact/history scope and F01 binding semantics require decisions; all codecs/evolution paths not audited |
| G3 Validity | **Unresolved** | Existing protected guards and selection refusal are strengths; F05's restore-send outcome lacks explicit ready invariant. No full race, codec, numerical validity or recovery execution |
| G4 Hidden behavior | **Pass, scoped source assessment** | Pure policy and explicit effectful canonical/admin operations; ordinary open refuses unknown schema rather than installing it. No hidden scientific mutation established |
| G5 Consistency and recovery | **Unresolved for composed shutdown/recovery** | Exact immutable operation settlement and honest unknowns exist; F04/F05 expose incomplete composed lifecycle. No inconsistent publication is asserted; F01 alone does not establish G5 failure |
| G6 Transformation and reuse | **Unresolved across all supported modes** | Structural/value preparation and exact admitted starts provide routes. F01 must settle early-/late-bound availability; current truthful missing selection is not automatically invalid reuse. No comprehensive numerical equivalence audit |
| G7 Truthful capability claims | **Pass for scoped current contract; broad qualification unestablished** | Local-only profile, partial outcomes and receipt recovery are explicit. No measured capacity or numerical restart claim accepted. Future proposals remain Proposed |
| G8 Library leverage | **Pass, scoped source assessment** | Native solvers/math remain library-owned, existing server functions/bytes/MCP used. Patch addresses actual consumed gaps. Optional features need not be adopted merely because available; tooling alternatives still eligible |
| G9 Architectural fitness | **Fail** | F02–F07 violate applicable foundations; positive composition/testability cannot average them away. F01 independently leaves a consequential target choice unresolved |
| PS-G1 Physical consistency | **Unresolved for enclosing scientific behavior** | Physical meanings must survive persistence; full units/basis/envelope/closure mechanisms not audited |
| PS-G2 Well-posedness | **Unresolved for enclosing scientific behavior** | Structural/DOF admission remains upstream; no complete formulation-to-solver assessment |
| PS-G3 Numerical integrity | **Unresolved for enclosing scientific behavior** | Original completion/seed/analysis qualification is preserved in examined paths; full derivatives/scaling/status/post-solve chain and restart equivalence unqualified |

Unexamined scientific breadth is not an additional finding. The profile gates prevent this persistence review being presented as numerical acceptance. No unit/property model was changed, and no conformance/parity suite was run. Required per-state numerical checks, conservation and honest partiality remain preservation constraints on every remedy.

## Rule impacts and follow-up ownership

These are proposed impacts, **not applied changes**. Current rules can be overturned after operator confirmation during subsequent plan creation. Keeping a rule is not grounds to pretend its consequences disappeared. No new disposition ledger is created; future authorized owning plan/packet must link this review's IDs. Plan 36's stopped status supplies context, not authority to schedule these findings.

<a id="rc01"></a>
**RC01 — Scientific input obligation.** Current rule/location: blueprint §20.3/§20.4/§20.6 and ADR-0164 describe revision sources, seed acquisition and reader/study/analysis retirement protection; the current study request selects an explicit seed policy with acquisition later. Proposed change: expose early-bound retained scientific starts versus deliberately late-bound availability; preferred retained work admits exact input and installs its dependency/copy contract. Dependent finding: F01; artifact direction F02. If retained: preserve the late-bound request contract and disclose that producer retirement can make pending/retry acquisition unavailable; no early-bound durable availability promise.

<a id="rc02"></a>
**RC02 — Retention unit.** Current rule/location: blueprint §20.4/§20.6, `canonical_result_retirements`, current run-level APIs and Plan 36 P-B7's retained run-level lineage direction. Proposed change: permit independently retained artifact selections/promotion with scientific-owner-defined closure, alongside or replacing mandatory whole-run lifetime. Dependent finding: F02; useful for F01. If retained: conservative whole-run pins remain and their retention amplification is explicit; selective canonical promotion is outside that chosen scope. This review does not amend stopped Plan 36.

<a id="rc03"></a>
**RC03 — Permanent detailed scientific history.** Current rule/location: blueprint §20.4/§20.6, schema/API promise to retain immutable study occurrence/outcome receipts, and Plan 36 P-B7 lineage retention. Proposed change: separate minimal permanent replay fences/attribution from explicitly retained scientific definitions, diagnostics and occurrence history, with owned release. Dependent finding: F03. If retained: permanent full occurrence history is a deliberate audit policy; do not claim disposable exploratory history drains merely because payload retirement completes.

<a id="rc04"></a>
**RC04 — Runtime close semantics.** Current rule/location: Python `NativeRuntime.close` documentation/implementation and canonical disconnect contract, adjacent blueprint §20.5/§20.6 and ADR-0164 drain descriptions. Proposed change: explicitly separate logical borrower release, refusal/cancel-join of owned active work and authoritative physical close. Dependent finding: F04. If retained: expose current call as whole-connection destructive close with explicit affected dependents, not a safe caller-local release; ordinary concurrent runtime lifecycle remains unresolved. No change to exact native/service drain meaning is recommended.

<a id="rc05"></a>
**RC05 — Transport mechanism, conditional alternative only.** Current rule/location: ADR-0164, blueprint §20.5/§20.6 and Cargo's exact vendored SDK/engine-api choice. Proposed change only if future comparison selects dedicated WS/HTTP/embedded integration: replace that mechanism while preserving complete consumed clock/value/uncertainty/drain/selection contracts. Dependent recommendations: transport alternatives for F04–F06. If retained: repair F04–F06 within the current integration; these repairs themselves require no mechanism/pin rule change.

<a id="rc06"></a>
**RC06 — Remote binding, conditional extension only.** Current rule/location: ADR-0164 and `CanonicalStore::connect` loopback native WS plus local supervisor state. Proposed change only if remote/shared support is selected: explicit TLS/host/auth/admission/deployment capability binding and remote lifecycle contract. Dependent recommendation: S07 extension. If retained: remote deployment remains unsupported, which is consistent with present local-first scope. Removing a host check alone is insufficient.

<a id="rc07"></a>
**RC07 — Numerical restart, conditional product decision only.** Current rule/location: blueprint §13.5/§13.6 initial-state/in-process reinitialization and §20.6 truthful recovery; current supported scope does not establish durable numerical resume. Proposed change only if selected: workload-specific scientific checkpoint sufficiency/equivalence and exact new-attempt dependency. Dependent recommendation: dynamics/restart target uncertainty, not a diagnosed current checkpoint defect. If retained: truthful failure/retry or bounded rerun remains the promise; do not call seed, partial trajectory, VERSION history or backup numerical resume.

<a id="rc08"></a>
**RC08 — Policy compilation ownership.** Current rule/location: blueprint §3/§20.6 and the `pse-operations` package/module responsibility in the workspace-and-dependencies architecture section; current manifests place pure study policy with canonical storage. Proposed change: choose a coherent production semantic-policy compile/test owner independent of unrelated SDK/codegen/columnar integration; update ownership descriptions if relocation changes them. Dependent finding: F07. If retained: policy tests remain runtime-isolated but compilation-coupled; do not claim the target's responsibility-local compilation/test boundary is established. A new crate, if ultimately selected, uses the existing ADR/review route; it is not mandated here.

<a id="rc09"></a>
**RC09 — Selected beta schema/test/rollout tooling seam.** Current rule/location: registry/codegen schema authority and explicit activation in blueprint §4/§20.5/§20.6, current supervisor/fixture scripts and tool/dependency declarations. Proposed change: target exact SurrealKit 1.0 beta tooling, consuming generated DDL/modules and integrating declarative tests/rollouts under existing authentic operation owners. Preserve one schema authority and ordinary-open refusal; replace private bookkeeping only where beta contracts fit. Dependent recommendation: selected beta integration above, including its cleanup/locking/targeting obligations. If retained without a new seam: current generated/admin tooling remains the actual implementation and the user-selected beta target is unimplemented; stable0.7 is not an implied fallback. Exact pin/tool selection and any changed enduring ownership must be recorded during subsequent authorized implementation.

No recommendation mandates changing license policy, scientific model authority, toolchain, numerical solver ownership, generated-output ownership, schema declaration authority, authentication/drain guarantees or the zero failure baseline. SurrealKit beta is selected target tooling; REFERENCE and scheduler/history/transport alternatives remain selective comparisons; their adoption would require exact affected contracts and normal decision routing, not blanket edits to the rule corpus.

## Decision, priorities and settling evidence

The proposed target is coherent: scientific owners decide sufficient content and meaning; retention owners decide current obligations/reachability; execution/connection owners decide who may act and when dependencies drain; operational owners retain independent bootstrap and authenticated disposal authority; the engine supplies suitable transactions, queries and physical storage. It can reuse most current mechanisms. None of these responsibilities requires a universal lifecycle language or a new canonical store.

Consequence priority differs from prerequisite order:

| Priority | Decision or correction | Findings/scenarios | Distinguishing acceptance evidence |
|---|---|---|---|
| High | Clarify caller release versus authorized physical close and cover durable dependents | F04/S04 | Close/renew/export races, independent borrowers and actual final driver drain |
| High target decision | Choose retained exact versus deliberately late-bound external scientific inputs | F01/S02 | Both-order admission/removal plus pending restart/retry; exact copied/retained values and eventual release |
| Medium; availability can raise urgency | Finite acknowledged restoration and explicit exceptional admission completion | F05/F06/S05 | Missing ACK with live socket, failed setup send, held retired metadata and preserved owned-payload drain |
| Medium; growth can raise urgency | Independent artifact/history retention contracts | F02/F03/S03 | Promotion/borrow/release and restartable collection, minimal fences, deliberate audit preservation |
| Medium | Separate actual production policy compilation/test dependencies | F07/S06 | Policy target closure and same-authority adapter/integration tests |
| Selected target tooling | Settle SurrealKit beta generated-DDL, fixture and rollout seam | Beta integration/RC09/S07 | Exact beta.6 command/DDL and failure-lifetime compatibility; no disposal inferred from suite pass |
| Conditional | Choose restart equivalence, remote binding or alternate transport only for an actual requirement | S06/S07 | Scientific sufficiency and complete consumed integration contract; representative measurements only for quantitative claims |

For prerequisite order, F01 can first use a conservative producer-run obligation without waiting for F02's finer retention. F02 needs scientific sufficiency before physical reachability changes. F03 needs minimal replay/attribution semantics and complete borrower/unknown-effect closure before history collection. F04 needs the public close/borrower ownership decision before runtime/Python wiring. F07 can be settled independently by the semantic policy and integration owners; it need not change runtime/scientific behavior. F05 and F06 are independent transport corrections; fixing a small completion branch does not settle restoration or shutdown. None requires restarting Plan 36 or adding numerical checkpoints first.

Preserve exact selected source/provider/start identities and authoritative numeric bits; original assessment and partiality; immutable result membership and grant identity; creator/reclaimer conflict and no resurrection; independent new-state scientific checks; copied allocation versus renewable acquisition permission; acknowledged-result reuse and truthful uncertainty; finite queue/memory/native charge ownership through actual drain; offline historical interpretation and migration provenance; explicit runner outcome/evidence policy and authenticated removal. Do not silently migrate live stores, reinterpret old receipts, activate schema, restart services or delete production content under this review.

**Final scope decision: Revise the lifecycle/integration target; implementation and assembled scientific qualification remain unaccepted.** The strongest evidence is source-level missing distinctions, terminal-state paths and compilation dependencies, not historical tests or claimed performance. The next consequential design choices are the external-input binding/retention contract and caller-versus-physical close ownership. Artifact/history granularity should then be settled without weakening current publication/recovery. Numerical resume, physical reclaim latency, full workload capacity and remote support remain distinct decisions/evidence gaps. Follow-up status awaits future authorized plan creation; this review is complete evidence, not authorization to execute.
