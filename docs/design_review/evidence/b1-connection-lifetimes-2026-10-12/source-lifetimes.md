---
title: B1 source lifetime evidence
date: 2026-10-12
doc_owner: docs/design_review/reviews/design_review_b1-connection-lifetimes_2026-10-12.md
doc_topics: [design, storage]
doc_retention: while-dependent
---

# B1 source lifetime evidence

Supporting evidence for the [bounded B1 target review](../../reviews/design_review_b1-connection-lifetimes_2026-10-12.md), not an independent verdict or disposition ledger. Baseline: `a6aedd31c51688e3686723306d7deac14bd4072c`, plus proposed ADR-0181 in the current worktree. Unrelated concurrent tooling/resource/investigation edits were preserved.

## Inspected ownership and consequences

- `crates/pse-py/src/workflow.rs`, `NativeRuntime::close`: clears program caches and calls the inner canonical store's physical disconnect. `NativeRuntime` derives Clone; its inner workflow runtime also derives Clone. `initialize_database` is a distinct explicit creation/administration path that drains its temporary store. Constructor `new` connects/opens a canonical store and composes `Runtime::from_shared`. These paths need different target authorities rather than a single replacement close call.
- `crates/pse-runtime/src/workflow/mod.rs`, `Runtime`, `from_shared`, `with_durability`, `canonical`: cloneable runtime retains shared native services, canonical deployment, durability operations and optional physical cache. Independent construction and ordinary cloning are distinguishable operations. `clear_program_cache` is already separately named. Public capabilities expose selected lower-level owners.
- `crates/pse-runtime/src/workflow/canonical.rs`, `CanonicalDeployment`, `Runtime::canonical_store`: exposes the actual canonical store. `crates/pse-runtime/src/workflow/durable.rs`, `Operations::from_store`, `Operations::store`: clones/exposes that store. Counting runtime wrappers alone cannot cover these paths.
- `crates/pse-runtime/src/workflow/run.rs`, `Lease::drop`, `RunHandle`, simulation `start_owned`: dropping the last public lease requests cancellation while the spawned supervisor retains the native handle and joins it. `RunResult` retains a Runtime; a completed result's incidental alias is different from a still-running supervisor.
- `crates/pse-runtime/src/workflow/durable.rs`, `DurableAttempt::{new,claimed,start,finish,abandon,terminate,reject_export}`, `Operations::settle`: the durable attempt spans registration, claim, native admission, progress ingestion, result/seed export and terminal settlement. Heartbeat stops before the final acknowledged seal on one path. Rejection can close/recover ingestion after heartbeat stops. A heartbeat-scoped guard ends too early.
- Same source, `Heartbeat::{spawn,stop}`: task clones Operations and renews its fence until stop/join. `Streamer::{new,finish,drop}`: the writer task clones Operations, owns bounded append work, and can outlive a dropped Streamer; Drop closes its queue but does not join the writer. Each child needs actual continuation ownership through its last operation.
- `crates/pse-runtime/src/workflow/connected_results.rs`, `CanonicalResultReader`: retains Runtime and ResultRead, pages bounded immutable content and closes unread work separately from escaped arrays. `workflow/progress.rs`, `ProgressStream`: retains a selected read and runtime while paging, releases read on close/exhaustion, and preserves copied coverage without future-read authority. Reader activity and copied allocation ownership must remain distinct.
- `crates/pse-runtime/src/workflow/study.rs`, `StudyHandle` operation surfaces were searched to identify status/cancel/wait as affected entrypoints; their complete transaction/workflow bodies were not audited in this bounded review. Fit/horizon and analysis are required migration consumers named by B1; their full lifetime paths are not freshly qualified here.
- `crates/pse-operations/src/canonical.rs`, `ProtectedSelection::finish`, `ProtectionOwner::{acknowledge_release,drop}`: exact release may move into an executor task. Drop obtains a tracked release token while decrementing live readers, then spawns the release RPC. The new activity handoff must cover this transition. Explicit finish retains the owner through its spawned cleanup task.
- `crates/pse-operations/src/canonical_results.rs`, `ResultReadDrain::{retain,release,close_and_drain}`, `ResultReleaseTask`: current reader admission/refusal uses a short mutex and live count. Close refuses live readers without sealing admission; otherwise it seals and joins tracked releases. Completion/failure is recorded before the tracker token drops. A new physical gate must preserve the cleanup continuation and avoid a partial refusal that has already sealed another gate.
- Same source, `ReadOwner::renew`, `CanonicalStore::{result_payload,read_payload}`: renewed DB permission and exact issuing-client identity remain distinct from returned immutable bytes.
- `crates/pse-operations/src/canonical.rs`, `CanonicalStore::disconnect`: reader drain precedes sequential ordinary and activation client disconnects. Partial/cancelled teardown therefore needs explicit Closing ownership and authentic continuation; successful completion of only the first driver is insufficient.
- `crates/pse-operations/src/canonical_transport.rs`, `CanonicalClient::{context,disconnect}`: the existing client owns request-context clocks/admission cancellation and delegates physical disconnect to the patched SDK. The caller/activity domain distinction belongs above that driver boundary, preserving the existing transport contract.
- `crates/pse-operations/src/testing.rs`, `FixtureLifetime::{drain_connections,drop}`, and `canonical.rs::remove_isolated_fixture`: authentic fixture teardown drains the store and retained peers before publishing resource drain. Some existing fallback logic observes fixture aliases; B1 must not substitute those aliases for actual connection activities or silently weaken authenticated fixture removal.

## Standard and current-design context

Read the shared worker/design-reviewer contracts, design-review skill, selected standard/Core 3.4 foundations and relevant execution/failure gates, efficiency heuristics, Process Simulator 1.5 principles/review additions and pse-arrow binding. Current context owners read: blueprint §20.5/§20.6 in `identity-and-publication.md`, relevant §21 workflow/result ownership material, Plan 37 and Plan 37b, principal lifecycle review F04, and proposed ADR-0181.

These owners describe the existing subject and transition; architectural judgment uses the selected standard and functional target. Historical compatibility/cutover text is not a newly imposed preservation requirement for this review; Plan 37 OP01 remains the selected future cutover direction outside B1.

## Limits and performed checks

Read-only `git status --short`, targeted `rg` searches and source/document reads established these observations. Search scope was the named runtime/Python/canonical/transport/fixture owners and adjacent public accessors, not every repository callable. No exhaustive absence claim, numerical certification or completed implementation coverage follows from that search.

Tests, compilation, runtime probes, services, installations and measurements: **not_run**. No product pass, failure count or B1/F04 implementation resolution is inferred. No product/ADR/plan files were edited by this reviewer. The root owns publication registration and its normal turn-end handling.
