---
title: "Plan72e: Observation and derived state"
status: complete
date: 2026-11-09
adrs: [ADR-0267]
doc_role: plan
doc_owner: docs/plans/72-surrealdb-functional-improvements.md
doc_topics: [current-work, design, libraries, diagnostics, storage]
review_sources: [docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md, docs/functional_reviews/surrealdb-functional-improvements-2026-11-09/observation-and-derived-state.md]
---

# Observation and derived state

**Proposed.** A native caller follows selected study/run state with honest freshness and obtains
useful summaries without reconstructing all rows. This owns [O10/O11](../functional_reviews/surrealdb-functional-improvements-2026-11-09/observation-and-derived-state.md).
The [coordinator](72-surrealdb-functional-improvements.md) owns overall adoption. Plan66 retains
scientific meanings; Plan71 retains its active test/report infrastructure. No Plan68 work transfers.

## Foundation assessment and target

**Interface-checked** at `21f703887cc307f986975f9a2fba998f8bd2afa7` plus the concurrent tree:
the capture generator already increments a shared witness on relevant mutations, private knowledge
already uses read-only snapshots, and native result assembly rejects LIVE responses. Therefore
observation needs an explicitly owned subscription capability rather than `LIVE SELECT` text
passed through the request adapter. [Witness](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-codegen/src/codegen/surreal_study_capture_witness.rs) ·
[Results](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-operations/src/canonical_client/results.rs).
Core 3.4, efficient architecture 1.0 and process-simulator 1.5 apply through the selected standard;
DP-09/DP-18 and blueprint §20 preserve freshness and original effect authority.

The follow contract reports an initial identified snapshot, later refreshed snapshots and explicit
resynchronizing/unavailable state. Notifications invalidate; coalesced rereads establish current
state under the caller's authority. Register before reading or close the snapshot-registration
race with a selected read/recheck sequence. State exit and deletion must invalidate too. Missing
intermediate transitions are permitted by current-state follow; pretending a stale snapshot is
current is not. Neither a notification nor a summary acknowledges a write, renews a protection
or grants release authority.

Expose one subscription owner under the existing client/service lifetime, with selected metadata
limits, cancellation, kill and physical disconnect. Fulfilling subscription semantics can require
a distinct physical channel, but never a second semantic source or hidden transport fallback.
[Live semantics](https://surrealdb.com/docs/learn/querying/real-time/live-queries).

Compute summaries by grouped/indexed native query first. Use `COMPUTED` for cheap read-time
presentation, `VALUE` for stored normalization and a native maintained view only when declared
source writes cover its dependencies. Synchronous events maintain required write-owned effects;
async events are repeatable derived work with visible freshness. Source/interpretation identity
travels with every retained projection. A display summary is never a capture concurrency fence.

## Packages

### Execution checkpoint — 2026-11-10

Execution is authorized under proposed ADR-0267. The selected server implements native gRPC
Subscribe with at-most-once, nonresumable delivery. Use a dedicated fixed-Viewer generation,
SubscribeBegin before snapshot, one pending refresh and explicit kill/drain. Permission and
session skips can be silent: report as-of snapshots and next_revalidation_at within 30 seconds,
not an unsupported validity lease. Native grouped summaries and explicit reconstruction are
selected; maintained views/changefeed replay are not added. Plan73 scientific categories and
original result authority remain their owners; capture SSI is not replaced by summaries.

The current source provides native fixed-principal registrations, current study/run grouped
summaries and evidence/knowledge generation following. The ordinary connection-only project
reader observes one finite subscription window with bounded retained updates. The old delayed
telemetry follow shortcut is removed; explicit one-shot per-attempt telemetry remains.
Fresh principal registration, rather than same-session cached roles, revalidates authority
within 30 seconds; dirty notifications cannot postpone that deadline. Summary run-header
dependencies retain exact linked-run invalidation. Native loss, rollback, revocation and
capture receiving now pass the selected controls linked in the Outcome.

Query-first grouped summaries preserve original lifecycle, settlement and usable-result facts.
Existing 72c native predicate protection remains the capture owner; summaries grant no capture
fence. No maintained view, feed cursor or generic broker is selected. Reopen retained aggregate
maintenance for demonstrated repeated costly summary reads or a cross-process retained
projection owner; arbitrary source-file edits still require original-cut reconstruction.

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| E01 | Complete | Current native client/service owners; O10 snapshot/continuity contract | Select and implement an owned native subscription surface with snapshot alignment, invalidation and settled cancellation. |
| E02 | Complete | E01 owned subscriptions and current selected-state readers | Native study/run/evidence follow returns current snapshots and explicit resynchronization while preserving permission and custody. |
| E03 | Complete | 72c chosen predicate/scoped guard contract; 72d composed selection | Shared query-first summaries and precise invalidation replace broad reconstruction/conflict work without missing dependencies. |
| E04 | Complete | E03 explicit projection dependency/source identity | Select maintained/replayable projections where useful; otherwise retain on-demand/reconstruction and close each branch with its reason/trigger. |

<a id="E01"></a>
### E01 — Own the subscription and snapshot boundary

Inspect the selected supplier RPC/server session path and choose one explicit subscription
channel under the canonical client owner. Prefer the existing native protocol where supported;
if it cannot supply live delivery, select a native notification channel with declared auth/session
and teardown rather than pretending current request assembly is streaming. Record the required
feature/preparation changes and route interface/lifecycle changes through the normal decision
path before implementation. This bounded contract decision protects against lost initial changes
and orphaned physical work; it is not a performance experiment.

Implement register/read/recheck, generation binding and bounded coalescing with no accumulated
payload queue. Return an owner that closes admission, kills its subscription and settles the
channel under the original deadline. Targeted controls cover a change during registration/read,
rollback, disconnect, permission loss and cancellation with an outstanding notification.

<a id="E02"></a>
### E02 — Deliver current-state follow

Migrate selected native observation consumers to identified snapshot plus invalidation-driven
refresh. A single pending refresh absorbs bursts; rereads apply current permissions. Mark state
unknown on loss and resnapshot before reporting current. Preserve one-shot reads as a useful
operation, not a compatibility duplicate. Remove periodic polling/repeated hydration only in
consumers migrated to the new lifecycle. No frontend is introduced.

Targeted acceptance: create/update/delete, leaving a filter, rapid changes, reconnect during
snapshot, denied data and kill/drain. A follow client must remain unable to settle an uncertain
canonical effect. Any complete-transition-history contract remains distinct from follow.

<a id="E03"></a>
### E03 — Query summaries and narrow invalidation

Provide selected point counts, terminal-state distributions and evidence freshness from shared
native grouped/count operations. Migrate workbench/result/evidence callers and remove their
row-by-row aggregation where replaced. Use the precise dependency contract selected by 72c
for capture validity: native serializable predicate/range tracking within a transaction, and
scope-specific witnesses only where validity extends outside it. Delete broad witness writes
only for predicates completely covered by that replacement; retain semantic generations,
authority fences and original attempt identity.

Positive reads, absent/future matches, membership changes, deletions and indirect dependencies
must be explicit. Focused independent cases exercise disjoint versus overlapping captures and
phantoms. This is a correctness-sensitive replacement, so proving the absence coverage is
required; benchmarking the obvious removal of unrelated invalidation is not.

<a id="E04"></a>
### E04 — Choose maintenance and replay only for a complete useful projection

For a source-local frequently reused summary, select `DEFINE TABLE ... AS SELECT` or synchronous
event maintenance when it removes repeated useful work. Keep mutable indirectly referenced
details in current queries unless their invalidation is also declared. Check source updates,
empty/final-row behavior, import/rebuild and migration; these are stale-result risks.

For a long-lived derived knowledge/evidence view, use `CHANGEFEED`/`SHOW CHANGES` only when
its actual database source supplies every needed change. Apply changes idempotently and advance
the destination cursor atomically; reconstruction uses original selected membership and mapping
identity on expiry/change. Inspect inclusive cursors, filtered empty batches and snapshot/feed
alignment because missed changes would corrupt freshness. A feed cannot observe arbitrary source
file edits. Choose full rebuild for small disposable or non-feed selections; no generic broker
is introduced. [Changefeeds](https://surrealdb.com/docs/learn/querying/real-time/changefeeds).

Do not use async work to acknowledge an effect before required state exists. Native async events
are at-least-once separate transactions. The experimental retained live router is deferred until
an actual subscription fan-out/resumption requirement makes its extra retained state useful.
`DROP` ingestion is excluded for retained scientific originals; Kafka/surreal-sync is excluded
for local projection refresh unless source/destination participants independently require it.

## Coverage and acceptance

O10 live refresh, reconnect/state exit and snapshot races belong to E01/E02. Durable transition
delivery is an explicit E04 contract, never an implicit upgrade to follow. O11 derivation clauses,
views/events, scope witnesses, feeds/cursors and rebuild belong to E03/E04. Materialized inverse
membership belongs to 72d's relationship choice, not a second summary authority.

**Proposed verification.** Use targeted native policy/query/subscription tests during implementation,
with explicit force-validation where Arrow is consumed. 72i joins current-state, projection and
capture receiving at scope end. No blanket load or replay campaign is needed; only serious
unresolved correctness/performance risk warrants additional proof. No tests ran during authoring.

## Outcome

**Implemented**, 2026-11-10. E01–E04 deliver owned native gRPC Subscribe, fixed-Viewer
generations, Begin-before-snapshot alignment, continuously drained coalesced invalidations,
explicit loss/resynchronization and kill/detach settlement. Snapshot construction and
publication respect both the original window and the bounded principal recheck deadline.
An as-of snapshot and `next_revalidation_at` confer no validity lease or write authority.

Study/run summaries use native grouped queries preserving lifecycle, assigned/settled and
scientifically usable categories. Study follow includes the exact linked-run dependency.
Evidence/knowledge follow observes imported cut/pointer/generation changes; arbitrary source
file edits still require reconstruction. The connection-only project reader exposes a finite
follow window. The delayed `StudyObserver::follow` polling shortcut and `delay_ms` action
branch are deleted; useful one-shot detailed telemetry remains.

**Tested** under the selected Linux/native conditions in the
[shared receiving record](72-surrealdb-functional-improvements.md#observation-and-custody-receiving):
the joined 80-case population and the ordinary reader passed against zero failures. Native
summary/follow exercises rollback, indirect linked-run changes, authority recheck crossing
the snapshot seam, channel loss before publication and fresh resynchronization. The
[7-case private discovery selection](72-surrealdb-functional-improvements.md#discovery-receiving)
also exercises invalidation and Viewer password revoke/refuse/restore. These are current-state
controls, not durable transition-delivery evidence.

**Mistake corrected.** The supplier caches actor roles in a session. Reusing that session
would not revalidate authority. A fresh signin/register occurs within 30 seconds of the
original registration, and dirty traffic cannot postpone it. Linked-run changes are observed
explicitly rather than inferred from study-point writes.

**Deliberate deviations.** Query-first reconstruction is the complete selected realization;
no maintained view, feed cursor, durable replay broker or concurrency witness replacement is
added. Existing 72c native capture protection remains authoritative. Its parked qualification
obligations retain that owner. Blueprint §20.5/§21.7 and proposed ADR-0267 retain the enduring
contract; 72i owns series acceptance and retirement. E01–E04 implementation is complete.
