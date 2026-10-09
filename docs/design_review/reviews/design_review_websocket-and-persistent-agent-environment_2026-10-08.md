---
title: WebSocket RPC and the persistent agent environment
date: 2026-10-08
status: review
tier: design
purpose: target
standard: Core 3.4; Heuristics 1.0; Process Simulator 1.5
---

# WebSocket RPC and the persistent agent environment

## 1. Decision, target and coverage

**Architectural fitness: Revise. Behavioral adequacy: unresolved for the proposed pivot. Overall decision: Revise the current environment; do not accept a direct transport substitution as the completed target design.**

**Candidate design decision: Not Accept.** Material lifecycle and capacity choices remain undecided. Design acceptance and later implementation qualification are separate obligations.

**Proposed direction:** use native binary WebSocket RPC through one Rust integration owner, backed by persistent, release-bound disk services. Give functional test invocations explicit database and session ownership; retain failed and incomplete evidence; coordinate heavy work across the host; reserve a truthful timing lane. Preserve the scientific authority, guarded publication, immutable acknowledgment and cancellation/drain contracts already present.

WebSocket is a credible transport choice because it preserves native values and request multiplexing while removing HTTP/2 from application RPC. That mechanism does not establish the cause or cure of the observed HTTP/2 failure. It also does not supply operation deadlines, safe cancellation, bounded accumulated work or uncertain-write settlement automatically. Those are the decisive contracts for adoption.

This is a **DESIGN-tier, TARGET-purpose** review under Core 3.4, the selected heuristic companion, Process Simulator 1.5 and the pse-arrow binding. The baseline is dirty `main` at `2410880efc5ab20e623006fde78e624044d3358d`, including concurrent Plan 28 I/J work. Findings describe the inspected composition, not a frozen release. An independent principal reviewer assessed the target; bounded repository mapping and exact-library research supplied additional evidence. The coordinator publishes the assessment without implementing its recommendations.

The functional target includes physically meaningful edit/re-solve, studies, recycles, dynamics and extension, with a practical environment for one operator whose code changes are made by Codex agents. The local host has sixteen physical cores, thirty-two logical CPUs and approximately 192 GB nominal RAM. Independent functional work should overlap; meaningful timing should have reserved capacity. Successful disposable test data should disappear automatically. Failure, interruption and incomplete evidence should remain pinned until explicitly released.

The review covers canonical application RPC, disk-service and receiver lifetimes, test isolation, evidence retention, host admission and timing interpretation. Numerical algorithms and all model families are not requalified here. The existing managed thousand-point failure repair remains separate. This review does not resume E4, E5 or any full campaign.

Evidence is **Interface-checked** for inspected library contracts and **Implemented** for the source paths described below. Historical measurements retain their original conditions. No new **Tested**, **Measured** workload or **Formally established** claim is made. Bounded read-only host observations are recorded separately in the [supporting evidence](../evidence/websocket-and-persistent-agent-environment-2026-10-08.md).

## 2. Responsibilities and the target boundary

The strongest existing boundary separates scientific preparation and native execution from short store transactions. Typed authored meaning and numerical outcome interpretation remain Rust responsibilities. The canonical store owns exact selection, roots, staging, fencing, publication and acknowledgment settlement. A persistent database does not become the numerical executor.

The target needs these coherent owners:

| Responsibility | Owned decision and contract |
|---|---|
| Scientific preparation/execution | Admitted physics, structural and value dependencies, solver capability, attempt-private workspaces and independently checked outcomes |
| Canonical operation adapter | Native wire values, complete statement results, bounded requests, effect identity and transport failure interpretation |
| Service generation | Exact server release, interpretation, persistent directory, authenticated endpoint and safe lifecycle |
| Receiver generation | Immutable worker artifact, admitted producer/runtime association, owned database context, capacity and drain |
| Test invocation | Unique resource identity, explicit terminal disposition and references to evidence |
| Host execution boundary | Admission of cooperating heavy work, aggregate resource limits and timing placement |
| Agent-facing commands | Select and explain those owned operations without recreating their policies |

These responsibilities need not become separate services or crates. Existing supervisor, environment and validation modules are suitable starting owners.

The composition root should select a service generation, an immutable receiver generation and an invocation-owned context. Consumers receive an already bound store/session handle. They should stop independently interpreting environment variables, switching a shared connection's database or deciding whether a prior receiver can be reused.

## 3. Meaning that must survive the pivot

The transport must preserve more than JSON-compatible result shape. `canonical_codec.rs` deliberately represents full-domain unsigned integers as checked decimals, distinguishes record IDs from text, uses native bytes, preserves diagnostic IEEE bits and checks finite projections against authoritative bits. Registry-admitted result IPC also retains schema, ordering, coordinates and completion framing.

Native WebSocket encoding is therefore preferable to a new JSON adapter. The inspected SDK WebSocket implementation uses `surrealdb_types::encode`. Preserve the existing codec and result-admission rules rather than translating their semantics into another wire vocabulary.

The operation contract must distinguish:

- rejection before submission;
- definite transaction rejection;
- acknowledged completion;
- deadline expiry with execution still possible;
- unknown acknowledgment requiring settlement;
- requested cancellation;
- confirmed drain.

A request ID routes a response; an immutable application operation ID settles an effect. Neither replaces the other.

**Physical-semantics boundary:**

| Element crossing storage | Required meaning | Authority |
|---|---|---|
| Physical source rows | Dimension, unit, basis, reference convention, validity and exact source identity | Authored/registry admission and physical owners |
| Numerical result IPC | Original checked values, schema, coordinates and scientific scope | Result admission |
| Completion | Terminal class, assessment, tolerances and original attempt lineage | Scientific completion owner |
| Prepared products | Complete consumed dependencies and eligible producer identity | Compiler/preparation owners |

No new physics is introduced by the pivot. The service and transport may carry or select these records, but may not infer their physical interpretation.

Well-posedness, derivative requirements, formulation guards, scaling, solver selection and independent post-solve checks remain neighboring scientific obligations. This review has not re-established them across all analysis modes.

## 4. Revealing scenarios

| ID | Stimulus and kind | Required response |
|---|---|---|
| S01 | Mechanism substitution: replace application gRPC with native WebSocket | Preserve exact codecs, statement completion and effect settlement; expose changed deadline, cancellation and streaming capabilities |
| S02 | Concurrent instances: two agents run Rust/Python functional tests | Each owns isolated mutable state; neither changes another invocation's database, receiver or evidence |
| S03 | Interruption: caller expires during disconnect, replay or commit | Expired work cannot silently acquire new authority; uncertain effects settle by original identity; ownership persists until its actual obligation ends |
| S04 | Code change: a worker rebuild occurs during another study | Existing admitted work drains under its original generation; a new artifact is admitted without reinterpreting or unnecessarily stopping compatible storage |
| S05 | Failure: solve succeeds but a later assertion fails | The test's scientific state and diagnostics remain pinned; operation success does not authorize test-data deletion |
| S06 | Growth: several agents build, test and solve while timing runs | Shared admission and placement constrain combined work; timing conditions and exclusions remain explicit |
| S07 | Recovery: agent dies holding a qualification lock | Its ownership can be established safely; stale exclusion can be recovered without deleting pinned evidence |
| S08 | Domain extension: add a model or compose a study | Existing semantic admission and execution primitives apply; persistence and agent tooling do not gain model-specific switches |
| S09 | Retention: a successful artifact becomes an input to another receipt | Cleanup protects evidence references and scientific roots before reclaiming disposable payloads |

S08 follows the existing semantic owners and does not justify moving physics into the database. S01 is a substantive mechanism substitution because the consumed lifecycle capabilities differ.

## 5. Transport and execution findings

### <a id="wp01"></a>WP01 — Caller timeout is not a complete RPC lifecycle contract

**High consequence; AP-02, AP-05, AP-07; DP-15, DP-19, DP-20; G5/G9; S01/S03.**

**Implemented evidence:** `canonical.rs::request` bounds waiting with `tokio::time::timeout`. `bounded_query` checks statement completion. Some owner-local staging/protection paths include queueing within one original clock. The free `protected_query` helper retries definite conflicts with fresh per-request waits; it does not itself carry an enclosing operation deadline.

**Interface-checked library evidence:** the 3.3.0 native WebSocket router can defer routes during session replay. Its inspected dispatch path does not discard a route because its response receiver has closed or an application deadline has expired. Ordinary RPC requests have no deadline field. Remote WebSocket does not inherit the embedded meaning of `Config::query_timeout`. See the exact-release [WS router](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/src/engine/remote/ws/mod.rs), [native connection](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/src/engine/remote/ws/native.rs) and [RPC envelope](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/rpc/src/request.rs).

Consequently, dropping a future can end the caller's wait while queued or submitted work still executes. Retrying after that timeout is unsafe unless the operation's immutable acknowledgment contract authorizes settlement or replay. Releasing capacity merely because the caller returned can also undercount remaining work.

**Proposed correction:** the canonical adapter owns one original operation clock, submission/effect classification and settlement obligation. Distinguish local admission expiry from unknown submitted execution. Preserve server-side generation/expiry fences and immutable acknowledgments. If “expired queued work is never dispatched” is required, obtain an SDK route-lifecycle correction; an outer timeout alone cannot implement it.

Prefer a fitting correction through the typed SDK over implementing routing, replay and protocol framing independently. A weaker supported contract may explicitly return timed-out/unknown while retaining bounded ownership and settling the effect. It must not claim hard abort.

**Verification:** disconnect during queueing and replay, expire before dispatch, lose an acknowledgment after commit, contend through retries and confirm original clock ownership. Show that late work cannot publish under revoked authority and that no timeout authorizes a new scientific effect.

A legitimate long activation needs its longer owned clock; imposing the ordinary read clock on every operation would reject valid work.

### <a id="wp02"></a>WP02 — WebSocket multiplexing and message caps do not bound the complete workload

**High consequence; AP-05/AP-07; DP-20; G5/G9; S01/S06.**

The SDK supports finite WebSocket message and write-buffer configuration. Its defaults are a 64 MiB message limit and unlimited maximum write buffering. Connection capacity zero creates an unbounded route channel. A nonzero capacity bounds that channel, not the per-session pending map or replay-deferred collection. See the [WebSocket configuration](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/src/opt/websocket.rs).

The server's ordinary response path encodes a complete reply before outgoing queueing. A client receive limit therefore does not bound server materialization. The current 4 MiB gRPC profile cannot simply be renamed a WebSocket profile. See the [response owner](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/server/src/rpc/response.rs).

**Proposed correction:** retain application payload, result-block and descriptor bounds; configure finite SDK buffering; bound admitted in-flight operations and aggregate retained payloads at the adapter. Queries should select bounded results by their existing semantic contracts. Account for pending and replay obligations, not only channel occupancy.

Server `query_stream`/`query_cancel` support is not equivalent to the pinned Rust SDK providing incremental WebSocket query execution: its WebSocket item-stream path uses buffered fallback. Cancellation acknowledgment also does not establish drain. Avoid promising either capability until the actual consumed SDK path supports and qualifies it. See the [engine fallback](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/engine-api/src/lib.rs), [server streaming implementation](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/server/src/rpc/streaming.rs) and [official RPC contract](https://surrealdb.com/docs/reference/rest-api/rpc-protocol).

**Verification:** growth, disconnect/replay, oversized responses and cancellation must demonstrate bounded complete behavior, including server reply construction. A large legitimate exact result should continue through bounded blocks/pages, without truncation or an unjustified giant message.

## 6. Isolation, persistence and evidence findings

### <a id="wp03"></a>WP03 — Mutable test ownership differs between Rust and Python

**High consequence; AP-02, AP-04, AP-06; DP-19; G1/G5/G9; S02.**

`testing.rs::canonical_fixture_store` creates a UUID database on the selected server. Python `conftest.py::canonical_substrate` supplies the persistent state directory; `NativeRuntime::new` loads its configured database. Ordinary native Python execution defaults to sixteen pytest workers. Unique run IDs do not isolate shared problem heads, history or schema.

Managed Python runs serially through the one-primary route. That prevents particular overlap, but does not supply a general isolation contract for parallel agents or ordinary component tests.

**Proposed correction:** one invocation owner allocates an isolated context and passes it through the same typed production boundary. A database can be the isolation unit where schema and canonical history require it. Bound session/database context immutably; sharing transport is permissible only when supported session semantics preserve it. Do not mutate namespace/database selection on a shared handle.

Invocation identity owns resource accounting and terminal evidence disposition; mutable isolation follows the smallest independent test owner. Ordinary xdist workers—and independent tests within a worker—need separate database/problem/session state where their mutations can interfere. An invocation-wide database alone is insufficient when tests reuse names or history. Immutable fixtures may be shared. Tests intentionally exercising shared-state concurrency declare that sharing explicitly. Database-per-test is one realization, not a universal requirement; narrower isolation must cover every affected mutable namespace and lifecycle.

**Verification:** overlapping independent invocations reuse identical problem names, perform edits, retain reads and finish in opposite orders without interference. Exercise Rust and Python together. Genuine concurrency tests deliberately sharing a problem retain explicit shared ownership rather than being accidentally isolated.

### <a id="wp04"></a>WP04 — Ordinary failure teardown deletes the evidence the target requires

**High consequence; AP-04/AP-05; DP-19/DP-21; G5/G9; S05/S09.**

Rust `FixtureLifetime::drop` removes its fixture database even during panic unwinding; panicking changes cleanup-error reporting, not disposition. Python `ManagedStudyFixture.__exit__` ignores the exception information and calls `close`; successful drain then removes the qualification directory, including caller logs and controls.

These are useful teardown mechanisms, but they cannot decide that a test succeeded. Abrupt exit may retain data accidentally; ordinary assertion failure may erase it.

**Proposed correction:** the test invocation owns terminal disposition. Resources begin incomplete/pinned. Only an explicitly established successful terminal test outcome, completed drain and absence of retention obligations make disposable data eligible for automatic cleanup. Failure, interruption, missing outcomes and uncertain cleanup stay pinned until release.

Reuse canonical retirement for deliberately retained scientific results. Preserve its reader/study/analysis protections, tombstones and resumable pages. Whole disposable databases can have a simpler removal path once their owner proves they are disposable and drained.

Successful reports can still be evidence dependencies. `validation_receipts::reuse_checks` preserves original origin paths and digests; cleanup must protect these transitive references. A `latest` symlink is only a convenience.

**Verification:** assertion failure after successful science, partial collection, abrupt caller loss, cleanup interruption and referenced successful receipts. Releasing evidence must not bypass live readers or active native work.

### <a id="wp05"></a>WP05 — Persistent storage and changing receiver artifacts need different generations

**Material consequence; AP-01/AP-03/AP-07; DP-19; G9; S02/S04.**

The selected reference deployment binds one primary to one database and one observer. Changed worker/supervisor bytes require offline readmission; an active unmatched receiver refuses reuse. These checks protect real artifact association. The same topology creates a broad coordination point for agents continuously rebuilding different source states.

**Proposed correction:** keep disk-service generations stable across compatible client/worker rebuilds. Admit immutable receiver artifacts separately, with explicit producer, runtime, schema and capacity associations. A compatible server need not stop solely because application bytes changed. An incompatible interpretation or server release receives a distinct service generation and an explicit migration/rebuild decision.

Distinguish persistent disk state from service residency. The service generation should have a supervisor-owned lifetime independent of the Codex session that launched or borrowed it. Session exit releases that session's resources without stopping a shared service. Define readiness and recovery separately: process/listener health, authentication and interpretation compatibility, then admission of recovered application work. Current `Restart=no` provides deliberate failure containment, but does not establish continuous availability. Bounded automatic restart is a viable alternative only after crash recovery and uncertain acknowledgments are qualified; restart must neither reinstall schema nor replay scientific effects. Worker residency and cancellation/drain retain their own explicit ownership. This recommendation remains separate from the pending failing-test repair.

Begin with a shared functional server and isolated databases. Add a bounded pool only where catalog/commit contention, incompatible generations or destructive lifecycle tests require physical separation. The historical AE-26 results justify investigation, not universal sharding: the old DDL path has changed, and eight study tests were a counterexample to broad serialization.

Do not invent a “clone database” prerequisite. No checked 3.3.0 clone primitive supports that recommendation; import is not a neutral reset/clone operation.

**Verification:** old/new compatible receivers coexist under bounded capacity; changed artifacts cannot inherit old authority; incompatible interpretations refuse; failure of one owned fixture cannot stop another service generation.

### <a id="wp06"></a>WP06 — An empty exclusive file cannot establish abandoned ownership

**Material consequence; AP-05/AP-06; DP-19; G5/G9; S07.**

Both managed-study fixtures acquire `managed-study-qualification.lock` through exclusive file creation. The file contains no process identity or nonce. After owner loss, its existence cannot distinguish live ownership from abandoned exclusion.

**Proposed correction:** use the existing process-identity and lifecycle patterns, or kernel-held local exclusion where suitable. Separate recovery of exclusion from release of evidence. Verify associated descendants and drain before handing capacity over. Age alone must neither authorize stealing a live lock nor deletion.

**Verification:** abrupt owner exit, PID reuse, partial startup and surviving native descendants. Stale exclusion should be recoverable while evidence remains pinned.

## 7. Host capacity and timing findings

### <a id="wp07"></a>WP07 — Per-command ceilings do not coordinate the agent workload

**High consequence; AP-05/AP-07; DP-20; G5/G9; S06.**

`pse_env.py` creates per-command scopes, normally with a 120G ceiling. Plan 29 intentionally has no default aggregate memory cap. Nextest groups are invocation-local. Several independent invocations can therefore each admit their own nominal capacity.

The reference server, primary and observer have a coherent 160 GiB parent cap. Its sixteen physical-core selection covers every physical core on this host. Placement records correctly say that ceilings are not physical reservations. This profile does not coordinate unrelated builds, other test invocations or other projects.

**Proposed correction:** extend the existing environment/supervisor boundary with host-wide admission for cooperating heavy commands and persistent roles. Use finite workload classes and established systemd controls. Give ordinary functional work bounded concurrency and designate timing capacity. Avoid a new enterprise job broker.

A smaller timing lane can coexist with functional work, provided its actual profile and placement are reported. Full sixteen-core reference measurements need an admitted exclusive heavy-work window; SMT siblings do not provide physical-core isolation. Reading, editing and suitably light work can continue. Unmanaged other-project load remains an explicit limitation.

**Verification:** overlapping heavy invocations respect the shared envelope; cancelled native work retains its charge through drain; timing admission cannot silently inherit active competing heavy jobs. Exact practical allocations remain a design/measurement decision, not a result of this review.

### <a id="wp08"></a>WP08 — Persistent-service measurements need explicit cold/warm scope and service attribution

**Unresolved candidate obligation; AP-02/AP-07; DP-22; G7/G9; S04/S06.**

Existing measurement documentation distinguishes timed setup, runtime cold/warm state, pool reservations and process lifetime RSS. Persistence introduces additional independently warm state: server process, RocksDB cache, filesystem cache, receiver, compiler/prepared products and imported extension.

**Proposed correction:** name which state is cold or warm and which setup is timed. Report service lifetime/resource observations separately from client or case observations. Use a dedicated timing service/context so functional fixture creation and cleanup do not become hidden competing work.

“Fresh process per case” does not mean cold storage. Conversely, long-lived storage should not force every functional test to restart a server merely to satisfy a mislabeled cold condition.

**Verification:** representative edit/re-solve, study and reopen journeys under declared residency and contention conditions. Compare equal scientific scope and checks. No speedup or memory saving is established here.

## 8. Foundations and gates

Current-composition judgments:

| Foundation | Verdict | Reason |
|---|---|---|
| AP-01 Separation | Violated | Receiver readmission and deployment-wide lifetime couple independently changing application artifacts and storage availability: WP05 |
| AP-02 Contracts | Violated | Waiting, submitted work and context ownership are not sufficiently explicit for the proposed substitution and concurrent use: WP01/WP03 |
| AP-03 Composition | Unresolved | Bounded receiver/service generations and host admission have not been composed |
| AP-04 Domain model/authority | Violated | Test success, evidence pinning and disposable-resource disposition lack one adequate realized model: WP03/WP04 |
| AP-05 Explicit constraints | Violated | Accumulated transport work, abandoned exclusion and aggregate capacity lack sufficient boundaries: WP02/WP06/WP07 |
| AP-06 Local reasoning/testability | Violated | Ambient shared Python state and abandoned qualification ownership require unrelated global context: WP03/WP06 |
| AP-07 Execution fit | Violated | Independent invocations multiply capacity and broad receiver exclusion obstructs the stated parallel workflow: WP02/WP05/WP07 |

| Gate | Judgment and boundary |
|---|---|
| G1 Authority | Fail for test-resource disposition/context ownership; canonical scientific authority is a preservation strength |
| G2 Semantic fidelity | Unresolved for WebSocket round trips; native codecs supply a credible route |
| G3 Validity | Unresolved for the composed pivot; existing codec and interpretation refusals remain |
| G4 Hidden behavior | Unresolved for reconnect/expired queued work; retain explicit preparation/effect separation |
| G5 Consistency/recovery | Fail for aggregate ownership and failed-evidence disposition; guarded scientific publication remains valuable |
| G6 Transformation/reuse | Unresolved for changed service/receiver generations; preserve complete dependency and producer eligibility |
| G7 Truthful claims | Unresolved for candidate reliability, cancellation, streaming and performance claims |
| G8 Library leverage | Pass at Interface-checked direction: native SDK/RocksDB/systemd fit; no separate bespoke protocol stack is justified yet |
| G9 Architectural fitness | Fail; individual MUST gaps are not averaged |
| PS-G1 Physical consistency | Unresolved for pivot qualification; no new physical model is proposed |
| PS-G2 Well-posedness | Not applicable to service/transport formulation; neighboring scientific admission remains required |
| PS-G3 Numerical integrity | Unresolved for composed interruption/publication qualification; transport completion cannot replace scientific verification |

## 9. Alternatives and coherent recommendation

| Alternative | Assessment |
|---|---|
| Direct gRPC → WebSocket replacement | Removes HTTP/2, preserves native-value opportunity; leaves lifecycle and workflow defects. Insufficient |
| Existing SDK with finite admission/configuration and necessary lifecycle correction | Preferred candidate; concentrates transport ownership and preserves typed operations |
| Direct custom WebSocket protocol client | Possible only for a demonstrated SDK contract gap; adds routing, replay, framing, session and cancellation ownership |
| One persistent server with isolated contexts | Simplest functional starting point; shared physical contention remains possible |
| Bounded persistent server pool | Appropriate for demonstrated physical contention, incompatible generations and lifecycle tests; increases cache and supervision footprint |
| Fresh server for every ordinary test | Strong isolation, unnecessary normal startup/catalog machinery; useful for destructive server tests |
| One reference primary for all agent work | Useful dedicated scientific qualification lane; inadequate general parallel development topology |

The recommendation is one coherent boundary change: native RPC stays thin; storage and receiver generations separate; invocation ownership controls isolation and disposition; host admission controls concurrent heavy work. A small agent-facing CLI should compose existing owners and return selected generation/context, status, evidence location and release capability. It should not become another scientific retry, schema or retention authority.

The corrections intentionally remove ambient shared fixture selection, success inferred from destructor execution, indefinite stale exclusion and duplicated per-invocation capacity assumptions. They preserve exact values, short publication, original operation identity, honest partiality, native drain and retained scientific history.

## 10. Qualification needed before adoption

Qualification must cover native codec edge cases, inner statement errors, bounded results, disconnect/replay, unknown commit acknowledgment, expired queueing, cancellation versus drain, overlapping Rust/Python invocations, receiver-generation changes, ordinary failure pinning and reference-protected cleanup.

Scientific journeys should include edit/re-solve, multi-case study, truthful partial/interrupted completion and reopening retained results. Keep the original physical checks and explicit force-validation. Model-family conformance and numerical capacity remain outside this review's evidence.

The current HTTP/2 failure and the reported successful read-only stress observation have different scopes. The latter does not reproduce or diagnose the failure, and neither establishes WebSocket's assembled reliability. The existing failing managed identity remains with its current repair owner.

## 11. Rule impacts and disposition

These are proposed changes, not applied decisions:

| ID | Current rule/location | Proposed change; dependency; if retained |
|---|---|---|
| <a id="rc01"></a>RC01 | Local gRPC client/profile: blueprint §20.6, proposed ADR-0164, substrate guide | Native WebSocket application RPC; WP01/WP02. Retaining gRPC leaves transport substitution unadopted |
| <a id="rc02"></a>RC02 | One exact primary/observer reference topology: proposed ADR-0166, Plan 28h, §20.6 | Keep it as a qualification lane; admit separately bounded functional receiver generations. WP05/WP07. Retaining it limits overlapping managed invocation contexts |
| <a id="rc03"></a>RC03 | Plan 29 uncapped aggregate placement policy | Coordinate host admission and timing placement. WP07. Retaining it leaves shared-envelope safety unresolved |
| <a id="rc04"></a>RC04 | Current fixture Drop/context-manager cleanup; §24.1 temporary ownership | Terminal disposition plus failure/incomplete pins. WP04. Retaining it conflicts with the requested evidence lifecycle |
| <a id="rc05"></a>RC05 | Offline whole-profile worker readmission: substrate/environment guides | Separate compatible storage and receiver generations. WP05. Retaining it accepts deployment-wide interruption on artifact change |
| <a id="rc06"></a>RC06 | Broad failure-material preservation instruction | Prospectively permit automatic cleanup only for proven successful disposable resources, while keeping existing materials untouched and pinned references protected. WP04 |

Propose one dedicated follow-up plan as the sole disposition owner, linking Plan 28's canonical/publication and receiver contracts and Plan 29 AE-25/AE-26. Plan creation should settle these rule impacts and the unresolved transport/admission decisions. This review does not schedule that plan, change current packet status or create a competing backlog.

## 12. Final assessment and evidence limits

The best target is a persistent local environment with native WebSocket RPC, isolated invocation contexts, independently admitted artifact generations and bounded shared capacity. The existing scientific and canonical mechanisms provide a substantial foundation.

A direct transport swap is insufficient. Current architecture requires revision for context ownership, failed-evidence retention, generation lifetimes, abandoned exclusion and aggregate admission.

**Candidate design decision: Not Accept.** The operation lifecycle, bounded transport realization, service/receiver ownership and host-admission contracts remain materially undecided. Resolve those choices and establish a credible implementation route before accepting the proposed target design.

Runtime qualification is a separate implementation obligation. Acceptance of the proposed design would not establish a deployed pivot, passing recovery tests, scientific qualification or performance gains. The current architecture's decision remains **Revise**.

Inspected owners include `canonical.rs`, `canonical_codec.rs`, `canonical_result_retention.rs`, `testing.rs`, `pse-py::workflow::NativeRuntime::new`, the Rust/Python managed-study fixtures, `surreal_server.py`, `pse_env.py`, `native_tests.py`, `validation.py`, `validation_receipts.py`, Nextest configuration, blueprint §5/§20/§23/§24, Plan 28 I/J/E/H and Plan 29 AE-25/AE-26. Exact library evidence is scoped to SurrealDB/SDK 3.3.0; decisive sources include `engine/remote/ws/{native,mod}.rs`, `opt/websocket.rs`, RPC request/method definitions and server streaming/response paths.

The [supporting evidence](../evidence/websocket-and-persistent-agent-environment-2026-10-08.md) records the host inventory, existing disk persistence, source disagreements, exact-release capability limits and historical counterexamples. Existing multiple servers require ownership inventory, not automatic consolidation or deletion. SDK backup/import/export is not supported by the checked WS engine; an application RPC pivot must distinguish explicit provisioning/control-plane operations rather than silently promise an all-operations transport capability.

This is source-grounded design evidence. It establishes neither a deployed pivot nor newly passing tests, performance gains, power-loss durability or whole-simulator qualification.
