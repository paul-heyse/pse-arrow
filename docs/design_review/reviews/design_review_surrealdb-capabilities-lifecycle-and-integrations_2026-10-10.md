---
title: SurrealDB capabilities, lifecycle and integrations
date: 2026-10-10
tier: design
purpose: target
status: review
standard: Core 3.4
heuristics: Efficient Architecture 1.0
profile: ProcessSimulator 1.5
baseline: 991fcea1984ea9cca0f281f050c87fad067a1431
---

# SurrealDB capabilities, lifecycle and integrations

**Architectural fitness: Revise. Behavioral adequacy: demonstrated control-clock failure and diagnostic fidelity defects; assembled cancellation and scientific qualification remain unresolved. Overall decision: Revise.**

SurrealDB remains a credible canonical persistence engine for this simulator. The inspected design uses it for capabilities that fit: typed records and relations, indexed selection, short atomic transitions, immutable operation receipts, retained scientific results and connected provenance. Mathematical semantics, preparation and native execution remain outside database transactions. Replacing the database would not, by itself, correct the principal defects found here.

The corrections belong chiefly at the boundaries between storage, deployment supervision and complete workflow operations. A supervisor lock can escape an operation's advertised deadline. Readiness and drain observations discard distinctions needed to explain or safely join an owned lifecycle. Independent immutable reads share client-side pacing with mutations. The storage client interprets native-worker deployment configuration that its read operations do not need. Python setup and close convert typed infrastructure failures into input-error strings. Terminal study consumers repeatedly hydrate the same occurrence inventory.

These are independent causes with interacting consequences. They do not justify a universal state manager, a second cache, an application WAL coordinator or moving numerical execution into SurrealDB. The preferred target preserves the existing scientific authorities and durable publication protocol, while narrowing coordination and passing complete, typed operations across the relevant boundaries.

## 1. Review contract, target and coverage

This is an independent **DESIGN / TARGET** subsystem review under **Core 3.4**, **Heuristics for Efficient Architecture 1.0**, **ProcessSimulator 1.5**, the selected review template and the [pse-arrow binding](../design_principles/binding/pse-arrow.md).

The functional target is a simulator supporting authoring, edit and re-solve, studies, dynamics, fitting and extension, with coherent durable sources and outcomes, truthful interruption, useful local iteration and credible execution over its supported workloads. This review assesses the SurrealDB integration serving that target; it does not treat existing deployment choices, plans or previous review conclusions as acceptance criteria.

**Baseline:** `main` at `991fcea1984ea9cca0f281f050c87fad067a1431`, 2026-10-10, with existing dirty Plan 28 documents, `docs/lifecycle.toml`, and two fixture edits in `scripts/tests/test_plan30_lifecycle.py` and `scripts/tests/test_surreal_mcp.py`. Those fixture edits remain unvalidated. They are not evidence that the stopped campaign has been repaired.

The principal reviewer inspected the decisive current source and relevant architecture owners. Bounded state mapping, library research and coordinator investigations supplied additional evidence. Supplier capabilities are scoped to SurrealDB **3.3.0**, its RocksDB backend and the repository's **locally patched** Rust SDK. The patch's guarantees are not attributed to unmodified upstream 3.3.0.

Inspected responsibilities include:

- Canonical transport, selection, staging, execution, results, analyses and retention in `pse-operations`.
- Runtime deployment composition, durable workflows, studies, physical admission, mathematical preparation, retained products and portable replay.
- Salsa workspace ownership and relevant identity/reuse contracts.
- Python runtime, study and retained-result lifetimes.
- Storage and receiver supervision, lifecycle reservations, readiness, native operation cancellation, fixture ownership and native MCP authorization.
- Relevant native database constraints, query planning, transaction, durability, cancellation, graph, changefeed and frontend capabilities.

The stopped Plan 28 campaign is retained as evidence. Its setup-test artifact records **754 tests, eight failures and three errors**, against a zero-failure baseline, before the enclosing campaign ended with exit 130. Ten terminal failures have source-supported stale-fixture explanations involving readiness, affinity or reservation expectations; the fixture corrections have not been validated. The remaining nested-observer drain failure has not been diagnosed.

The coordinator performed two narrowly scoped investigations during this review. One unchanged cancellation test passed in isolation. A separate controlled probe demonstrated the deadline escape described in F01. Neither establishes E3/E4/E5 closure.

No implementation, service mutation, test launch, cleanup, commit or push was performed by the principal reviewer. Repository publication and the two diagnostic probes are the coordinator's responsibility. The full campaign remained stopped, and no review recommendation was implemented.

**Disposition ownership:** these are new review findings awaiting maintainer adoption into one corrective plan or packet. They are not automatically added to Plan 28 or reopened as Plan 34 work. This review owns the historical judgment; the subsequently selected plan will own current dispositions.

## 2. Architecture worth preserving

The current architecture has several consequential strengths.

| Responsibility | Current owner and consumed contract | Preservation constraint |
|---|---|---|
| Scientific meaning | Authored relations, physical definitions, compiler admission and mathematical/native libraries | Database convenience must not redefine physical meaning or solver eligibility. |
| Durable source selection | `canonical_selection`, revisions, memberships and immutable versions | Preserve positive, absent-name and complete-set premises. |
| Private preparation and publication | Staging generations, closed manifests and guarded activation | A partially uploaded object must remain unavailable as a canonical source or result. |
| Active preparation | Salsa workspaces, immutable prepared products and owned allocations | Retain structural/value separation and actual allocation ownership. |
| Durable replay | Protected dependencies, eligible producers and current receiving-context qualification | A matching hash or old receipt must not mint current scientific authority. |
| Native attempts | Runtime/native owners and supervised workers | Cancellation requests do not release charges before actual completion or drain. |
| Results and analysis | Closed terminal descriptors, exact IPC blocks and explicit lineage | Preserve failure/partial classes, exact membership and selective hydration. |
| Deployment effects | Existing supervisor, host admission and authentic process/unit identities | Storage, receiver and observer lifetimes remain distinct. |

The source makes important distinctions that a simplification must retain.

`SelectedDependencies` explicitly carries **no protection or write authority**. A dependency receipt can be reused only by adopting it under a newly acquired protected revision. Prepared mathematical products can therefore survive workspace changes without their old selection becoming a permanent publication capability.

`BasisKey` includes the selected request, complete dependencies, root, instance, bindings and limits. Its prehash is a bucket hint; full field equality remains authoritative. The inspected physical admission cache likewise distinguishes actual registry/session owners and obtains current protection separately from reusable physical products. Eviction does not invalidate escaped immutable owners.

Salsa remains a process-local semantic reuse mechanism, rather than an authority over database lifetime or scientific result validity. The inspected compiler workspace owns its mutable database, exposes cancellation and rebuilds bounded generations without persisting library-local handles.

The canonical transport also has useful guarantees:

- The original monotonic clock covers local queues, retries and response handling.
- Uncertain authorization transitions block subsequent context reuse.
- A successful RPC envelope is insufficient: statement errors and missing statement completion are checked.
- Definite conflicts authorize complete-decision retry; uncertain effects retain their original identity.
- The patched driver retains dispatched correlations through abandonment and physical teardown.
- `disconnect()` establishes the physical driver boundary, with an explicit limit: it does not prove server transaction or native scientific completion.

These mechanisms are not redundant merely because SurrealDB supplies transactions or WebSocket cancellation.

## 3. Domain, authority and lifecycle contracts

The integration must keep four different questions separate:

1. **What science was admitted?** Exact sources, physical context, preparation and selected policy.
2. **What durable effect occurred?** An original operation, its immutable inputs, acknowledged receipt or uncertain completion.
3. **What is executing or retained locally?** Tasks, native work, readers, prepared products and their allocation owners.
4. **What deployment can currently be used?** Storage invocation, credentials, protocol readiness, receiver association and admission.

One database does not make these one identity or lifetime. A closed socket can coexist with committed server effects. A cancelled study can coexist with draining native work. A cache entry can disappear while an escaped product remains valid and charged. A deployment receipt can become ineligible for new replay while old scientific history remains readable.

### Physical semantics and numerical scope

| Element crossing this subsystem | Required physical meaning | Authority and boundary obligation |
|---|---|---|
| Scientific scalar cells | Exact authoritative bits; declared numeric projection | Scientific owners define meaning; codecs preserve it, including signed zero and exceptional diagnostic domains. |
| Result relations and IPC blocks | Schema, units, basis, reference/convention metadata and semantic identities | Registry and scientific producers own declarations; framing and readers reject incompatible or incomplete representations. |
| Source/model revisions | Declared physical context, parameters, provider contracts and domain obligations | Compiler/scientific admission owns validity; storage retains exact source and dependency associations. |
| Solve completion | Actual outcome, achieved conditions and usable/partial/failed scope | Runtime scientific assessment owns classification; durable storage must preserve that classification. |
| Warm starts and reused preparations | Explicit compatibility and provenance | Existing semantic preparation and replay owners decide eligibility; storage presence alone is insufficient. |

The review does not invent new physical defaults, tolerances or validity envelopes.

Variable roles and structural well-posedness belong to compiler/model admission before native execution. Storage topology and provenance graphs are not substitutes for equation incidence or solve order. The inspected integration respects that responsibility direction. This review did not independently requalify the complete degree-of-freedom, derivative, conservation or numerical assessment implementations.

For formulation, evaluation and solve stages, the existing scientific owners retain formulation policy, domain guards, derivative source/order, scaling, solver-class selection, status mapping and independent post-solve assessment. The database stage stores and selects their admitted products; it does not supply a new numerical method or convergence oracle.

## 4. Revealing scenarios

| Scenario | Required behavior and change boundary | Current assessment |
|---|---|---|
| **S01 — Concurrent startup or maintenance join** | A compatible caller joins the exact owned lifecycle under its original clock. Unknown ownership refuses replacement. | F01 demonstrates a clock escape; F02 identifies insufficient observation distinctions. |
| **S02 — Interrupt nested observers while storage remains shared** | Cancel only owned foreground descendants, retain unresolved ownership, and preserve independently owned storage. | Source supports authentic targeting. The isolated rerun passed; the campaign failure remains undiagnosed. |
| **S03 — Sixteen same-problem readers alongside edits and reclamation** | Exact immutable selection survives concurrent change without ordering unrelated reads unnecessarily. | Guards preserve conflict premises; client-side per-problem pacing also serializes independent reads, F03. |
| **S04 — Read results without launching native science** | Storage selection needs the storage deployment contract; native execution configuration should be interpreted only when that capability is selected. | Current storage construction validates worker profiles and the shape of receiver associations, F04. |
| **S05 — Upgrade the SDK or substitute transport** | The integration owner absorbs API changes while preserving deadlines, uncertainty, replay and teardown guarantees. | The vendor patch identifies real missing upstream contracts; removing it requires equivalent evidence. |
| **S06 — Reopen or finish a large study** | Header polling stays small; required full occurrence results are hydrated once and projected for consumers. | Open-study polling is appropriately narrow. The terminal consumer repeats full hydration, F06. |
| **S07 — A storage failure during Python construction or close** | The caller receives a structured infrastructure cause rather than an invalid-input classification. | Current string conversion loses the cause and classification, F05. |
| **S08 — Add a model or change case values** | Extend scientific declarations and reuse stable preparation without changing storage or supervision policy. | Current semantic, compiler and cache boundaries largely support this. F04 increases unrelated configuration context. |
| **S09 — Add graph retrieval or notifications** | Use fitting native capabilities while preserving graph semantics, exactness and recovery ownership. | Native capabilities are eligible; selection depends on the complete consumed contract, discussed below. |
| **S10 — Server applies a transaction but completion is uncertain** | Retain the original operation and settle by exact identity; never infer rollback from a generic error. | Current immutable receipt direction is appropriate, including the RocksDB post-application error exposure. |

## 5. Findings

### <a id="f01"></a>F01 — Blocking supervisor exclusion escapes the original operation clock

**Principles:** AP-02, AP-05, AP-07; DP-19, DP-20.  
**Scenario:** S01.  
**Priority:** High.

**Implemented evidence.** `scripts/surreal_server.py::state_lock` acquires `fcntl.flock(..., LOCK_EX)` without a deadline. `lifecycle_reservation`, `wait_reservation` and `context_admission` use it while their enclosing operations advertise bounded admission clocks. `wait_reservation` checks `remaining(deadline)` before acquiring the lock, then can return after acquiring it without checking the clock again.

**Controlled counterexample.** The coordinator held `.supervisor.lock` in a child process for approximately 250 ms, supplied a 40 ms deadline and called the unchanged `wait_reservation` with an absent reservation path. It returned success after **0.251378623 s**. The retained record is `build/surreal-review-lock-clock-20261010/result.json`.

This is a demonstrated functional deadline violation, not a passing acceptance test or a throughput benchmark. The first harness failed before invoking the product because of Python stdin/forkserver bootstrap; the corrected subprocess harness produced the counterexample.

**Consequence.** A busy or stopped lock owner can delay a joining caller beyond its original budget. A caller can then receive apparent successful admission after expiry. Outer timeout checks do not bound an unconditional blocking acquisition inside them.

**Proposed correction.** Make the existing supervisor's admission/control clock reach lock acquisition and every relevant control wait. Refuse expired admission before effects and recheck after waiting. Keep cleanup obligations distinct: an expired caller cannot abandon owned effects and declare drain.

Use existing OS/library primitives; a new scheduling framework is unnecessary. A narrowly scoped nonblocking acquisition loop under the original deadline is one viable realization.

**Verification.** Hold the metadata lock across a short admission deadline and assert timely typed expiry with no later admission effect. Separately show that cancellation during an already owned lifecycle leaves its cleanup authority and resource charge intact. Increasing timeouts does not correct this finding.

### <a id="f02"></a>F02 — Lifecycle observations discard the distinctions required by their consumers

**Principles:** AP-02, AP-04, AP-05, AP-06; DP-02, DP-19, DP-21.  
**Scenarios:** S01, S02.  
**Priority:** High.

**Implemented evidence.** `service_readiness` returns broad enum values after discarding observation causes. Any live lifecycle reservation can produce `STARTING`, including an owner that is stopping or maintaining. An externally observed `activating` unit can instead become `UNAVAILABLE`; `start` immediately refuses that value. `_start` treats `STARTING` through an active-listener branch that can report "active but unhealthy."

`native_operation.drained` safely returns false for live population, identity mismatch and unavailable observations, but does not return which premise failed. `test_resources.eligible` reduces these cases to "supervised descendant not drained." The failed nested-observer fixture did not retain the observations needed to distinguish these cases.

The isolated unchanged cancellation test passed once, so these facts do **not** establish a leaked observer or an incorrect kill target.

**Consequence.** Callers cannot reliably distinguish authenticated progress they may join, lifecycle exclusion they must await, an actual mismatch they must refuse, and an observation failure they must report. Operators and tests reconstruct the cause through separate systemd/cgroup queries, potentially observing a later lifetime. Conservative refusal protects safety but does not supply an adequate operational contract.

**Proposed correction.** Let the existing observation owner return the decision-relevant state and its evidence: selected lifecycle/identity, authenticated progress or mismatch, and the failing observation when unavailable. Keep policy at the existing caller. Startup may join only compatible owned progress; uncertainty must not authorize restart. Drain may remain conservative while exposing why it is inconclusive.

This does not require a universal lifecycle service or a transcript per poll. A small typed observation and structured terminal diagnostic are sufficient.

**Verification.** Exercise owned startup, owned stop/maintenance, replaced invocation, unavailable manager, populated descendant and completed drain. Consumers must choose their action from the returned contract without reimplementing identity interpretation. On a genuine failure, retain the exact observation that caused refusal.

### <a id="f03"></a>F03 — Client-side pacing orders independent immutable reads with mutations

**Principles:** AP-03, AP-07; DP-10, DP-20; heuristics H22–H24.  
**Scenario:** S03.  
**Priority:** Medium; material for complete concurrent operations.

**Implemented evidence.** `CanonicalStore::protected_query` and `staging_query` both enter `StagingTurns::run(problem)`. Its per-problem mutex covers the complete RPC and associated retry operation. Result index pages, payload reads, result blocks and selected-source pages use this path.

Consequently, cloned same-store readers of independent immutable blocks cannot overlap these RPCs with one another or with same-problem staging. A slow request consumes the waiting readers' original clocks.

The SQL retention guard is a separate mechanism. SurrealDB 3.3.0 `FOR UPDATE` enrolls exact-record reads in optimistic conflict detection; it is not evidence that read-only transactions inherently require a client-side read/read mutex. Guard writes remain important for reclamation and phantom protection.

**Contrary evidence.** The source contains `sixteen_same_problem_protected_readers_share_short_turns_with_staging_and_exact_ack`; Plan 34 records its functional pass. That supports the current protocol's mixed-operation safety. It does not establish that ordering all immutable reads is necessary or that complete result retrieval avoids consequential queueing.

**Proposed correction.** Preserve exact protection, expiry, reclamation and complete-decision conflict checks. Narrow local pacing to operations that actually need it, allowing independent immutable reads to use the already bounded transport concurrently where their contract permits.

Do not simply delete guards. A fitting alternative may retain writer pacing or fair coordination during mutation-heavy reclamation. The target obligation is to remove unjustified local ordering, not to maximize request concurrency.

**Revealing legitimate case.** Frequent reclamation can make optimistic retries expensive. The correction must preserve writer progress and avoid a read/retry storm; separate read concurrency from the actual mutation conflict domain.

**Verification.** Retain mixed-reader/edit/reclaim safety controls. Show that two independent protected payload reads can progress concurrently without weakening exactness, original clocks or resource ownership. Assess complete-operation queueing where it could change the selected coordination policy. No numerical speedup is claimed.

### <a id="f04"></a>F04 — Storage construction interprets native-worker deployment policy

**Principles:** AP-01, AP-02, AP-06; DP-16, DP-17.  
**Scenarios:** S04, S05, S08.  
**Priority:** Medium.

**Implemented evidence.** `CanonicalOptions` carries `NativeAllocation` and `ManagedPrimaryReceiver`. `from_state_for_database` validates native workers, pool/case/thread populations and process headroom. When a primary receiver is configured, it also requires an execution profile, absolute executable/script paths and digest-shaped association fields before returning storage connection options. This is configuration admission, not proof of the current executable bytes. `CanonicalStore::native_allocation` and `managed_primary_receiver` reload those contracts through the database handle.

These facts are needed when composing a managed scientific receiver. They are not needed to decode or select a canonical result.

**Consequence.** A native execution-profile change affects storage construction and its fixtures. A valid storage-only operation can be refused because an unrelated worker profile or configured executable association is malformed. Storage callers and tests must know native deployment details, and the database integration becomes a route for execution policy. An absent optional primary receiver does not itself prevent construction.

**Proposed correction.** Keep the supervisor's authoritative deployment configuration, but project storage connection/admission facts to the storage owner and native execution facts to runtime/deployment composition. Validate each consumed capability at its selection boundary. The runtime can pass an explicitly validated receiver association to the operation that launches it.

One physical configuration file is compatible with this correction. It does not require another crate, a neutral storage trait or duplicated configuration authorities.

**Revealing legitimate case.** A storage service and receiver can share a deployment identity while having different lifetimes. The projection must preserve their exact association and stale-generation refusal; moving fields must not turn a path string into authority.

**Verification.** A canonical storage read should construct from valid storage facts without a native receiver profile. Managed execution must still refuse a missing or incompatible execution association. Changing receiver configuration should remain local to deployment composition.

### <a id="f05"></a>F05 — Python setup and close convert infrastructure failures into input errors

**Principles:** AP-02, AP-05; DP-02, DP-21.  
**Scenario:** S07.  
**Priority:** Medium.

**Implemented evidence.** `crates/pse-py/src/workflow.rs::NativeRuntime::new` converts canonical connect/open failures using `WorkflowError::Input(e.to_string())`. `NativeRuntime::close` performs the same conversion for disconnect failure.

`WorkflowError::Canonical` already exists. `workflow/diagnostics.rs` projects that variant through the typed diagnostic route, while `Input` is classified as `DiagnosticRule::WorkflowInput` and retains only a text detail.

**Consequence.** A connection, interpretation, timeout or transport-drain failure can be reported as invalid API input. Python consumers lose the typed cause and must parse prose to distinguish infrastructure from authored-input refusal.

This finding does not claim that Python lacks protected-reader drain. `pse-py` normally enables operations' `test-support` feature, which includes that drain mechanism.

**Proposed correction.** Preserve `WorkflowError::Canonical` through the existing Python diagnostic projection. Add contextual stage attribution without discarding the underlying cause. Apply the same rule to adjacent conversions found while migrating these callers.

**Verification.** A controlled canonical timeout, interpretation mismatch and disconnect failure must retain their infrastructure diagnostic and cause through Python. Invalid user input must remain a distinct classification.

### <a id="f06"></a>F06 — Terminal study consumption repeats full occurrence hydration

**Principles:** AP-03, AP-07; DP-10, DP-20; heuristics H6, H8, H16–H20.  
**Scenario:** S06.  
**Priority:** Medium; grows with supported study size.

**Implemented evidence.** `StudyHandle::result` appropriately polls only the study header while the study is open. Once terminal, it calls `status`, which pages and materializes the complete occurrence inventory.

The durable execution observer in `workflow/study_execution.rs` calls `handle.result()` until terminal and discards the returned full result. It then reserves retained summaries, calls `handle.status()` again, and subsequently pages the complete facts inventory again. The operations boundary admits studies up to 100,000 points and pages occurrences in groups of 64.

Explicitly requesting all point results legitimately requires processing all points. The finding is the repeated terminal retrieval introduced by consumer boundaries, not that full results exist.

**Consequence.** Completing a study causes repeated database crossings, decoding and temporary collections over already terminal occurrence state. The first terminal result collection also precedes the execution observer's later summary reservation. Per-page bounds do not establish bounded combined live state or proportionate total work.

**Proposed correction.** Consume one coherent terminal occurrence read and project the needed result keys, status and facts for the existing consumers. Reuse that immutable realization through its actual lifetime. Preserve the narrow open-study header polling.

A new snapshot certificate or durable duplicate summary is unnecessary. An existing paged operation with a complete internal projection may suffice. Public paged status is an optional interface decision, not a prerequisite to eliminating the repeated internal scans.

**Revealing legitimate case.** An open study can change during observation. Reuse must rely on terminal immutability or an equivalent existing protected contract; it must not turn a live status page into an invented coherent snapshot.

**Verification.** Trace terminal consumption and show that the occurrence inventory is hydrated once for the returned views. Preserve point ordering, failure/partial classes, exact run/attempt associations and truthful retained ownership. Exercise a multi-page study and corruption on a later page.

## 6. SurrealDB and library-owned simplification

### The SDK patch addresses real contract gaps

`vendor/surrealdb/PATCH.md` records an optional finite native-WebSocket profile: request-local deadlines/cancellation, dispatch state, application and reserved control admissions, bounded queue/replay state, ordered acknowledged setup replay, physical teardown and selection checkpoints.

This is meaningful integration machinery. Upstream server cancellation does not supply the complete client contract.

The pinned server cooperatively cancels normal queries on socket teardown and drains handlers so transaction cleanup can run. That means disconnect can initiate server cancellation; it is still not an acknowledgement that server cleanup completed. The streaming `query_cancel` path requests stop for a streaming query and acknowledges before terminal completion. Provisional rows and later failure remain possible. Native scientific workers are a separate lifetime in either case. See the pinned [WebSocket RPC implementation](https://github.com/surrealdb/surrealdb/blob/238bfeb11f5725bebed370167656748df8067595/surrealdb/server/src/rpc/websocket.rs).

**Proposed direction:** preserve the finite profile's consumed guarantees. Compare an upstream upgrade or streaming-query integration against those guarantees before removing patch code. Streaming cancellation could avoid closing a whole connection for one abandoned request, but requires driver admission, framing, finality and drain integration. It is a candidate mechanism, not a qualified drop-in replacement.

Tokio's existing `JoinSet`, `TaskTracker` and `CancellationToken` can remove generic in-process wakeup or task-lifetime bookkeeping where they fit. They do not replace dispatch uncertainty, immutable operation identities or native join. `JoinSet::shutdown` discards task failures; a drain that needs failure evidence must collect outcomes. Already-running `spawn_blocking` work cannot be stopped merely by aborting its handle. [JoinSet](https://docs.rs/tokio/1.53.2/tokio/task/struct.JoinSet.html), [TaskTracker](https://docs.rs/tokio-util/0.7.19/tokio_util/task/task_tracker/struct.TaskTracker.html), [spawn_blocking](https://docs.rs/tokio/1.53.2/tokio/task/fn.spawn_blocking.html).

### Transactions and durability retain application obligations

Short native transaction blocks and registry-generated functions fit atomic canonical transitions. They can remove repeated round trips and repeated store-local transition construction while leaving scientific interpretation in Rust.

Interactive SDK transactions also exist, but introduce a session and idle lifetime. They are not automatically simpler than the current bounded transaction blocks. Retry behavior documented for another SDK must not be transferred to Rust 3.3.0.

`FOR UPDATE` protects exact record-read premises. Selecting a matching population and then enrolling its existing IDs does not protect new matching records. Named guard records remain justified for absent/set premises and reclamation. See the pinned [statement implementation](https://github.com/surrealdb/surrealdb/blob/238bfeb11f5725bebed370167656748df8067595/surrealdb/core/src/dbs/statement.rs).

The RocksDB adapter applies `tx.commit()` before awaiting the commit coordinator's sync completion. Thus an error can follow application of the transaction, even apart from a lost network acknowledgement. A successful `sync=every` completion awaits grouped fsync, but a generic failure does not prove rollback. Original operation receipts and exact reconciliation remain necessary. The native coordinator already groups WAL sync; adding another application WAL batching layer would duplicate an existing capability. [RocksDB commit implementation](https://github.com/surrealdb/surrealdb/blob/238bfeb11f5725bebed370167656748df8067595/surrealdb/kvs-rocksdb/src/lib.rs), [commit coordinator](https://github.com/surrealdb/surrealdb/blob/238bfeb11f5725bebed370167656748df8067595/surrealdb/kvs-rocksdb/src/commit_coordinator.rs).

This review makes no new power-loss durability claim.

### Native selection is already bulk and indexed

`next_membership_pages` already combines multiple exact inventory selectors in one protected transaction and returns bounded arrays. The registry declares composite indexes for scoped selection, paging, logical lookup, results and study dispatch. "Add batching" or "add indexes" would be an inadequate diagnosis.

Remaining opportunities concern the complete operation:

- Preserve multiple selector intents through callers.
- Avoid repeated terminal retrieval and decoding.
- Inspect actual planner behavior for interval predicates and `out.kind`/`out.closed` dereferences.
- Compare exact payload access and projection with complete object hydration.
- Retain server-side filtering and ordering where the planner can use them.

`LIMIT 64` bounds returned rows, not examined records or dereferences. Actual `EXPLAIN`/execution evidence should settle a disputed access path; no new generic planner or permanent benchmark framework is required. [Pinned explain implementation](https://github.com/surrealdb/surrealdb/blob/238bfeb11f5725bebed370167656748df8067595/surrealdb/core/src/exec/operators/explain.rs).

### Constraints and graph features are eligible, with semantic limits

Native `ENFORCED` relations can supply endpoint-existence admission where staging order permits. Current relation generation does not emit that constraint. This is a useful simplification candidate, not proof that current guarded writers admit dangling authoritative state. Import bypasses relevant admission checks, so it cannot replace post-restore integrity verification.

`REFERENCE` can maintain inverse relationships and deletion policy. It may remove unconditional inverse/cascade bookkeeping when its semantics fit. It does not prove current selection, expiry, retention eligibility or native drain. Cascading deletion is inappropriate where evidence must survive independently of its originating record. [Pinned edge enforcement](https://github.com/surrealdb/surrealdb/blob/238bfeb11f5725bebed370167656748df8067595/surrealdb/core/src/doc/edges.rs), [reference handling](https://github.com/surrealdb/surrealdb/blob/238bfeb11f5725bebed370167656748df8067595/surrealdb/core/src/doc/field.rs).

LIGHTWEIGHT relations fit derived pair connectivity without edge payloads. They do not fit canonical edges whose metadata, ordering or multiplicity is meaningful. Native INLINE EDGES/REFERENCES can maintain adjacency caches and batch starting-record lookups, but add write amplification and have overflow or query-shape bypasses. Their suitability depends on a fitting traversal operation, not on the mere presence of graph data.

Surreal traversal is not a replacement for equation-structure analysis, numerical graph algorithms or solver libraries. Moving a generic graph operation to a fitting library remains eligible; moving scientific meaning into a database query is a separate decision.

### Notifications and asynchronous events are observations

LIVE can wake a UI or invalidate a read projection. Historical pinned Rust capability probes SB101 and SB135 record stream termination on reconnect and a subscription remaining after stream drop. These are historical probe results, not new runtime qualification. Current JavaScript managed-subscription documentation does not change those Rust 3.3.0 facts.

CHANGEFEED plus `SHOW CHANGES` can support catch-up, with explicit cursor, retention-gap recovery and idempotence. Neither LIVE nor a changefeed is scientific completion or current Salsa dependency authority.

Synchronous database events execute in the writer transaction. Asynchronous events have maintenance/retry lifetimes and can exhaust retries. They may own fitting database-local projections; they do not establish exactly-once native execution or external effects. [Pinned event implementation](https://github.com/surrealdb/surrealdb/blob/238bfeb11f5725bebed370167656748df8067595/surrealdb/core/src/doc/event.rs).

### Frontends and maintenance should remove actual adapters

The repository already uses native MCP. It should not build a bespoke MCP SQL proxy to reproduce that capability. Existing VIEWER authorization checks private records, the expected endpoint, current service ownership and readiness identity without starting a service or exposing secrets.

Postgres wire is a SurrealQL frontend, not a promise of ANSI PostgreSQL semantics. GraphQL can generate useful typed CRUD, but its identity and mutation semantics must match the consumer. DEFINE API can remove fitting database-local handlers, with explicit authorization and body-shape qualification. These are eligible capabilities; they do not justify another public interface without a concrete operation or bounded exploration purpose.

Logical export can provide a database-consistent transfer representation. It does not include the whole supervisor, credential, worker and installation envelope. No fitting public hot-checkpoint operation was established in the inspected RocksDB adapter; that is a bounded search result, not a universal absence claim.

Embedding can eliminate IPC and parts of service supervision, but moves the native engine, build closure, memory and maintenance ownership into application processes. It is not automatically simpler for multiple Python processes and detached workers sharing one durable authority. Experimental modules, buckets or WASM capabilities have no established fit here as replacements for native scientific execution.

### Bounded avenues for further investigation

These questions preserve the distinction between a demonstrated defect and an eligible capability. An investigation should adopt a technically fitting simplification even when an isolated timing difference is small or noisy; non-inclusion needs a concrete contract failure, ineffective mechanism or significant performance impairment. Numerical improvement claims still require measurements.

| Inquiry | Decision to resolve | Decisive evidence and preservation boundary |
|---|---|---|
| **Q01 — Complete selection access paths** | Which selectors or payload projections eliminate examined work and round trips? | Actual plans and complete operations with interval/dereference predicates, representative skew and multiple selector intents. Preserve exact membership and later-page corruption refusal. |
| **Q02 — Native integrity and inverse ownership** | Which registry families can use ENFORCED or REFERENCE instead of equivalent custom bookkeeping? | Staging/publish order, import bypass, retention and independent evidence lifetime. Endpoint existence is only one part of canonical eligibility. |
| **Q03 — Fitting graph representations** | Where do LIGHTWEIGHT or native adjacency caches replace useful derived connectivity work? | A concrete traversal, payload/multiplicity requirements, high-degree behavior and write amplification. Preserve scientific graph authority and measure complete retrieval. |
| **Q04 — Streaming cancellation and patch retirement** | Can upstream or streaming facilities replace individual vendor mechanisms with equal guarantees? | Dispatched versus undispatched work, provisional versus terminal frames, replay, original clock, reserved control progress and physical drain. A cancellation ACK alone is insufficient. |
| **Q05 — In-process task primitives** | Can JoinSet/TaskTracker/token composition simplify existing local lifetime bookkeeping? | Identify the actual task owner; collect required outcomes and prove join. Do not transfer remote/native completion guarantees to task cancellation. |
| **Q06 — Notifications and local projections** | Can LIVE/changefeed/events remove polling or fitting projection code? | Reconnect, explicit unsubscribe, retention gaps, duplicate delivery and retry exhaustion. Preserve authoritative reread and exact publication/claim protocols. |
| **Q07 — Maintenance and deployment alternatives** | Can native export, a fitting checkpoint or embedding remove more lifecycle work? | Separate database consistency from the full deployment envelope; compare multi-process ownership, rebuild/recovery and engine resource costs. No hidden fallback or second production authority. |
| **Q08 — Remaining active-state lookup and result interfaces** | Would indexed compiler-request lookup or public paged study status materially simplify supported operations? | Current exact-equality lookup and retained-byte bounds; concrete consumer needs. F06's internal repeated hydration can be corrected without first adding either mechanism. |

## 7. Alternatives and target selection

| Alternative | Capability retained or removed | Main tradeoff | Judgment |
|---|---|---|---|
| Current supervised remote SurrealDB | Shared durable authority, process isolation, detached receivers and native querying | Current clock, observation, composition and complete-operation defects remain | Revise. |
| Corrected existing composition | Preserves durable semantics and scientific owners; narrows coordination and redundant work | Requires coordinated boundary corrections, without a database migration | Preferred target. |
| Embedded SurrealDB | Removes network/service boundaries for a fitting single-process owner | Moves engine/build/resource ownership into each application; shared multiprocess authority still needs design | Eligible alternative; insufficient evidence to select it for the enclosing deployment. |
| Replace with another established database | May offer different transaction, query and operational capabilities | Requalification of exact codec, selection, uncertainty, retention and shared deployment contracts | No present evidence that replacement is necessary. |
| Active-only scientific execution with optional persistence | Can remove storage availability from a deliberately local workflow | Changes canonical-source and durable workflow scope; cannot silently satisfy current durability claims | A separate product/architecture decision, not an implicit fallback. |
| Universal state/recovery service | Centralizes many APIs | Combines unrelated meanings and lifetimes; risks duplicating existing owners | Reject as the default remedy. |

The simplest viable correction is the existing architecture with better boundaries, rather than a new subsystem. The remote server earns its cost through shared multiprocess state and failure isolation. Those benefits do not justify interpreting native-worker policy inside every storage connection or serializing every same-problem immutable read.

The mandatory canonical-source route for explicit ephemeral execution is a consequential current product choice. This review does not establish an offline execution requirement or select a second production backend. If local execution without storage becomes a target requirement, reconsider that rule explicitly; do not add a hidden durable-to-ephemeral fallback.

## 8. Architectural foundations and gates

### Foundation verdicts

| Foundation | Verdict | Basis |
|---|---|---|
| **AP-01 Separation of concerns** | **Violated** | Storage construction interprets native receiver and execution-allocation policy, F04. Scientific mathematics and durable publication otherwise have useful separation. |
| **AP-02 Stable contracts** | **Violated** | Original-clock semantics escape at a blocking boundary; lifecycle observations and Python error conversions lose decision-relevant meaning, F01/F02/F05. |
| **AP-03 Composition** | **Violated** | Existing math/storage/native composition is sound in direction. Shared read/mutation pacing and repeated terminal projections entangle independent operations, F03/F06. |
| **AP-04 Domain model and semantic authority** | **Violated in operational lifecycle modeling** | Scientific sources, dependencies and outcomes have meaningful authorities. Readiness/drain observations lack distinctions their consumers require, F02. |
| **AP-05 Explicit structure** | **Violated** | The advertised clock does not govern all waits, and failure classification is lost at boundaries, F01/F02/F05. |
| **AP-06 Local reasoning/testability** | **Violated** | Storage-only construction requires unrelated native configuration; lifecycle refusal requires reconstructing discarded observations, F02/F04. |
| **AP-07 Execution fits workload** | **Violated** | Demonstrated deadline escape, unnecessary read ordering and repeated full terminal hydration, F01/F03/F06. No numerical performance improvement is inferred. |

### Independent gate verdicts

| Gate | Verdict | Evidence and limit |
|---|---|---|
| **G1 Authority** | **Pass, Interface-checked within inspected contracts** | Registry-generated contracts, separate semantic/storage identities, explicit selected dependencies and publication owners. Multiple representations are not competing authorities. |
| **G2 Semantic fidelity** | **Fail** | Operational distinctions and typed infrastructure causes are lost, F02/F05. No scientific-bit corruption is established. |
| **G3 Validity** | **Unresolved for the enclosing integration** | Checked schemas, interpretation, fences and exact decoding are substantial protections. The stopped campaign and unvalidated fixture edits do not establish current assembled acceptance. |
| **G4 Hidden behavior** | **Pass, Interface-checked within inspected paths** | Storage I/O, creation, execution and replay qualification are explicit. Ordinary open does not silently provision unknown schema. |
| **G5 Consistency and recovery** | **Unresolved** | Immutable operation settlement, closed manifests and generations provide a credible protocol. The nested-observer campaign failure remains undiagnosed; one passing isolated rerun cannot qualify assembled drain. |
| **G6 Transformation and reuse** | **Pass, Interface-checked for inspected reuse contracts** | Complete selected premises, current replay admission, full cache equality and separate structural/value preparation. This is not comprehensive equivalence qualification. |
| **G7 Truthful capability claims** | **Fail at the inspected diagnostic boundary; broader qualification unresolved** | F05 reports infrastructure failures as workflow input errors. Supplier and campaign limits must remain explicit. |
| **G8 Library leverage** | **Pass at reviewed design strength** | Existing libraries own database, incremental, cache and numerical machinery. The SDK patch has stated contract gaps. Additional native capabilities are eligible candidates, not proven mandatory replacements. |
| **G9 Architectural fitness** | **Fail** | Individual foundation violations stand independently of current happy-path correctness. |

### Process-simulator gates

| Gate | Verdict | Scope |
|---|---|---|
| **PS-G1 Physical consistency** | **Unresolved for scientific adequacy** | Exact cells, schema and lineage preservation were inspected. Physical balances, provider envelopes and conventions were not comprehensively requalified. |
| **PS-G2 Well-posedness** | **Unresolved for scientific adequacy** | Responsibility remains before solving under compiler/structural owners. This review did not execute the complete structural admission journeys. |
| **PS-G3 Numerical integrity** | **Unresolved for scientific adequacy** | Durable outcomes preserve scientific classification and do not infer success from storage or cancellation. Solver, derivative, domain and post-solve checks were not comprehensively requalified. |

These unresolved scientific gates do not manufacture a physics defect. They prevent this subsystem review from being read as whole-simulator acceptance. Conversely, correct physical results would not waive the architectural findings.

## 9. Evidence, verification and limits

| Claim or risk | Evidence level | Current result |
|---|---|---|
| Supervisor original-clock guarantee | **Implemented; Measured operational counterexample** | Unchanged `wait_reservation` returned after its 40 ms deadline while blocked on a 250 ms lock holder. F01 is established. |
| Nested observer cancellation | **Tested, narrowly scoped** | Coordinator ran unchanged `test_real_assessment_cancellation_drains_nested_observers_preserves_storage_unit`; one passed, zero failures/errors, baseline zero, approximately 0.973 s. |
| Original campaign drain failure | **Tested, historical failed campaign; cause unresolved** | Retained in `build/plan28-e3-unified-state-qualified-20261010/setup-test.xml`. Isolated contrary evidence does not resolve it. |
| Ten stale fixture failures | **Implemented source evidence supporting the explanation** | Fixture edits remain unvalidated. No campaign repair claim. |
| Protected reader/staging correctness | **Implemented; historical targeted evidence** | Exact 16-reader mixed-operation control exists and Plan 34 records a pass. No current latency or whole-topology result inferred. |
| Typed Python cause loss | **Implemented source evidence** | Existing `Canonical` variant is replaced with `Input(string)` at setup/close. |
| Repeated terminal study hydration | **Implemented source evidence** | Terminal result, status and facts paths repeat the occurrence read. |
| SurrealDB supplier behavior | **Interface-checked/source-backed; named historical probes where stated** | Scoped to 3.3.0 and specific backend/SDK. No new capability benchmarks run. |
| Corrective architecture | **Proposed** | None of F01–F06 is closed by this review. |

The cancellation diagnostic retained its temporary directory and observations under `build/surreal-review-cancellation-20261010`. It preserved the original test assertions and deadlines. Its passing result establishes that the failure is not reproduced by that isolated run; it does not prove behavior under the stopped campaign's load or topology.

Both coordinator probes used `scripts/pse-env -- python3` with inline diagnostic harnesses. The cancellation harness invoked the named unchanged `unittest` case, wrapping temporary-directory cleanup and `await_drain` only to retain/print observations. The contention harness used a handshake with a subprocess holding the existing metadata lock, then called unchanged `wait_reservation`. Their structured `result.json` records retain outcomes and conditions. Neither invocation resumed the full campaign.

The supplier review used exact upstream tag `238bfeb11f5725bebed370167656748df8067595`, pinned skill material and the local patch, with Context7 and primary-source GitHub research. Current documentation and open issue reports are leads where release behavior is not independently established. An unreported or unexamined capability is not evidence of absence.

No new numerical benchmark, capacity claim, full setup-test rerun, integration campaign, parity campaign or E3/E4/E5 qualification was performed. No recommendation changes the required independent physical and external IDAES assertions.

Internal tests should preserve intended contracts rather than freeze an immature implementation. When an adopted correction replaces internal machinery, migrate or remove its internal vectors and fixtures with that machinery. Do not retain a compatibility path merely to preserve an old internal golden, and do not weaken physical or external reference assertions to make the replacement pass.

## 10. Rule impacts and disposition route

The following rule impacts are recommendations for subsequent plan creation. They are not applied by this review.

### <a id="rc01"></a>RC01 — Replace the current presumption favoring broad per-problem pacing

**Current rule/design:** Plan 34's SI05 and deliberate limits retain per-problem pacing until guard topology changes or complete-operation evidence identifies consequential queueing, stating that a visible mutex is insufficient.

**Proposed change:** permit target correction of demonstrated unnecessary local ordering between independent immutable reads while retaining distributed guard semantics. The structural proof is the shared `StagingTurns` route and absence of a read/read atomicity dependency, rather than an assumption that all same-problem reads require serialization.

**Depends on:** F03, S03.

**If retained:** F03 remains an explicit unresolved target disagreement. The follow-up owner must establish that the broader pacing buys a necessary complete-operation benefit or supply the existing rule's requested queueing evidence. The historical safety pass alone does not settle that question.

### <a id="rc02"></a>RC02 — Separate storage configuration consumption from native execution configuration consumption

**Current rule/design:** `CanonicalOptions::from_state_for_database` requires and validates `NativeAllocation` and validates configuration fields of an optional `ManagedPrimaryReceiver`; `CanonicalStore` exposes their reload operations. These interfaces embody a combined deployment contract.

**Proposed change:** let the composition owner consume and validate native execution associations, while the storage owner consumes its connection, interpretation, admission and identity projection. Update affected architecture/API contracts where they currently assign the combined behavior to the storage handle.

**Depends on:** F04, S04/S05/S08.

**If retained:** storage-only clients remain deliberately coupled to native deployment configuration. The target must state and justify that supported limitation; it cannot claim independent storage construction.

No rule change is needed to preserve original clocks, typed errors, exact operation settlement, physical assertions or existing allocation ownership. F01, F02, F05 and F06 correct behavior within those guarantees.

No recommendation currently changes the selected database, supported RocksDB deployment, mandatory canonical sources, scientific identity framing, numerical tolerances or external parity contract. Any later choice to replace those mechanisms requires its own explicit rule impact and decision.

Findings adopted for work should have one current disposition owner using the binding's existing route. Previous reviews remain evidence with their original scopes. This review neither overwrites their findings nor creates a second independently maintained status ledger.

## 11. Decision and correction boundaries

**Overall decision: Revise.** Retain SurrealDB as a credible canonical substrate while correcting the integration boundaries.

| Priority | Correction | Findings | Acceptance boundary |
|---|---|---|---|
| High | Make the original control clock govern actual waits and admission | F01 | Controlled contention expires without later admission; cleanup remains owned. |
| High | Preserve authenticated lifecycle progress, mismatch and observation failure | F02 | Startup/drain consumers act from one typed observation and retain decisive refusal evidence. |
| Medium | Preserve canonical typed failures through Python | F05 | Infrastructure and input errors remain distinguishable without text parsing. |
| Medium | Narrow local pacing while retaining protection/reclamation semantics | F03 | Independent immutable reads overlap; mixed mutation/reclaim safety and writer progress remain intact. |
| Medium | Move native deployment interpretation to composition | F04 | Storage-only construction does not require receiver policy; managed execution still validates exact association. |
| Medium | Reuse one coherent terminal occurrence read | F06 | Terminal consumers stop repeating full hydration; ordering, outcome scope and ownership remain truthful. |

Consequence priority and implementation order differ. F02's observation contract helps diagnose cancellation and verify F01 without inventing another observer. F04 supplies cleaner inputs for local integration tests. F03 must preserve existing distributed safety before removing local coordination. F06 can be corrected independently at the study consumer boundary.

The next decision is whether to adopt these correction boundaries and RC01–RC02 into one new, explicitly authorized corrective scope. The stopped scientific campaign should not be restarted merely because this review is published or because an isolated cancellation test passed.

The current evidence establishes real architectural and operational defects, a viable preservation-first target, and useful native capability alternatives. It does not establish a production observer leak, a required database replacement, a speedup, or comprehensive scientific qualification.
