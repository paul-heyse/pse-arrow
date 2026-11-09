# Observation and derived state

Supporting assessment for the [principal review](../surrealdb-functional-improvements-2026-11-09.md).
Inspected 2026-11-09 at `4f07766942ff3b21274fdfb686ad483031439462` with concurrent
dirty Plan66 changes preserved. Source/interface observations are **Interface-checked**;
recommended capabilities and benefits are **Proposed**. No subscription, replay or concurrency
probe ran. The principal review owns the integrated dispositions and priorities.

## Existing strengths and proposed interaction

The [capture witness generator](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-codegen/src/codegen/surreal_study_capture_witness.rs)
already uses synchronous events to change a shared publication witness for selected study,
point, run, attempt, manifest, project and research mutations. It excludes incidental heartbeats
and leases. The witness protects absence and future matches as well as existing rows; it is
an actual concurrency mechanism, not merely a counter for display.

The [result reader](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/src/canonical_results.rs)
and [knowledge reader](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/xtask/src/development/knowledge.rs)
provide bounded, selected current facts. Private knowledge already uses a read-only snapshot.
The [native result assembler](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/src/canonical_client/results.rs)
explicitly rejects an unsupported LIVE response. A persistent observer therefore requires an
owned subscription capability; issuing `LIVE SELECT` through the existing request adapter is
not an implementation.

The proposed workflow lets users follow a study's current state, refresh useful summaries and
understand whether a selected evidence view is current or needs rebuilding. Observation does
not grant operation acknowledgement, release authority or a protection renewal. Core AP-04,
DP-09 and DP-18 protect meaning, freshness and effects; AP-07/H13/H23–H27 judge the complete
cost and lifecycle. Scientific publication and outcomes retain blueprint §20–§21 meaning.

<a id="O10"></a>
## O10 — Follow current state without rebuilding a durable event system

**Recommend an owned reactive observation capability.** An operator can follow a selected
study/run or evidence selection and receive current bounded state after relevant changes,
with visible reconnect/resynchronization status. Provide a complete initial snapshot, then
native notification-driven invalidation and coalesced authoritative rereads. Register observation
before the snapshot or close the race with a proven subscribe/read/recheck sequence. A missed
intermediate transition is acceptable for this current-state capability; a stale visible state
presented as current is not.

Use `LIVE SELECT` on the smallest relevant tables/records under the observer's identity. Include
changes leaving the selected predicate, not just the new matching state. Keep subscription
admission, maximum retained metadata, cancellation, kill and connection settlement with a real
runtime owner. Notification bursts can coalesce into one pending refresh; they must not accumulate
full result payloads. On disconnect, mark continuity unknown and reconstruct the selected snapshot.
Recheck effective permissions; never trust notification contents as a new data-access grant.

Compared with polling, this removes unnecessary repeated current reads while providing a more
responsive interaction. Compared with turning notifications into a durable event log, it avoids
cursor/acknowledgement machinery that the chosen current-state contract does not need. Retain a
one-shot read operation and allow an explicit bounded polling realization if the selected native
subscription transport cannot meet custody; do not silently change continuity semantics.

The [live-query guide](https://surrealdb.com/docs/learn/querying/real-time/live-queries)
documents committed, subscriber-permission-based notifications and no total cross-writer order.
That is enough for invalidation; it is insufficient for an exactly-once transition history.
The current request-only client must be extended or replaced at an explicit subscription boundary.
The selected surface remains conditional on transport/lifetime qualification, while the user
capability and invalidation/snapshot design are recommended. The experimental retained live router
is not the default: its added retention and failure model must purchase a demonstrated benefit.

For a consumer needing every transition, expose a distinct replayable-change contract backed by
retained `CHANGEFEED`/`SHOW CHANGES` or explicitly authored events. Preserve source identity,
mapping revision and per-change identity; a feed versionstamp does not alone identify each domain
event. This is not a hidden stronger promise of `follow`. Such a consumer can be a new audit or
incremental-analysis activity within current workflows, without forcing it on ordinary status.

Demonstrate create/update/delete, state exit, rollback, rapid changes, permission loss, disconnect
during initial snapshot, cancellation and retained-reader drain. The observer must report current,
resynchronizing and unavailable states honestly and remain unable to settle an uncertain write.
This changes the native observation/client contract and likely its service lifecycle, so the
required decision route accompanies implementation planning. It does not require a new frontend.

<a id="O11"></a>
## O11 — Derive useful summaries and invalidate only relevant work

**Recommend query-first summaries plus precise change scopes.** Let the workbench show selected
point counts, terminal-state distributions and evidence freshness without forcing clients to
reconstruct every row. Use one native grouped/indexed query for current summaries; a cheap pure
`COMPUTED` projection can express read-time presentation. Keep mathematical/scientific evaluation
with its owner. Add a maintained representation only when it removes useful repeated work or
supports a functional interaction that on-demand queries cannot serve well.

Compare four complete realizations: existing bounded reads, native grouped/count queries,
`DEFINE TABLE ... AS SELECT` materialized summaries, and explicit event-maintained projections.
Choose grouped queries as the general starting target because they carry no new freshness state.
Select native views for source-local aggregates with a complete declared dependency set; select
synchronous events where a write-owned projection cannot be expressed as a view. Use stored
`VALUE` normalization where derivation belongs to the stored value, and `COMPUTED` where current
read-time derivation avoids unnecessary persistence. These earlier mechanisms remain valuable in v4.

Do not store an indirectly referenced mutable label/status in a view and assume every dependency
will refresh it. View maintenance follows declared source mutations. Keep mutable related details
as current query joins, or declare and maintain their invalidation explicitly. Import/rebuild,
empty `GROUP ALL` summaries, projected identity changes and final-row deletion need independent
witnesses. `DROP` ingestion is not selected for original scientific evidence: discard would defeat
reconstruction. It remains eligible only for deliberately disposable input with a retained aggregate
as its complete product, not as an accidental storage optimization.

**Recommend removing unrelated capture conflicts as a target, without replacing all guards with
display summaries.** The current single publication witness serializes changes that can be
independent. Prefer exact serializable predicate/range protection within an appropriately scoped
capture transaction when it covers the actual decision. Where policy/freshness extends beyond
one database transaction, use scope-specific witnesses whose membership and absence coverage are
explicit. A native view count is not a concurrency fence. Retain semantic generations, original
attempt identity and authority fences where they convey more than conflict detection.

This composition lets independent studies progress without incidental global invalidation and
keeps captured summaries coherent. It can remove redundant broad event increments and conservative
recomputation. It can also introduce new missed-dependency failures if scopes are guessed from
existing rows. Positive references, absent/future matches, membership changes and deletions must
all be represented. Coordinate with [canonical operation design](canonical-operations-and-policy.md)
so there is one chosen guard strategy, not independent view and procedure solutions.

**Conditionally select changefeed refresh for a long-lived rebuildable projection.** A developer
may keep an evidence/knowledge view current rather than repeatedly importing unchanged sources.
Use the source owner's selected membership and interpretation, apply changes idempotently and
advance the destination cursor in the same destination transaction. Rebuild from the original
selected source when retention expires or source/mapping identity changes. A filesystem/source
selection needs its own acquired change producer; a database feed cannot observe arbitrary file
edits. Do not enable a database-wide feed and infer it covers every input to private knowledge.

For small disposable selections, choose full reconstruction instead: a cursor ledger, retained log
and recovery worker can cost more than rebuilding. The [changefeed guide](https://surrealdb.com/docs/learn/querying/real-time/changefeeds)
establishes retained catch-up as a separate facility. Exact inclusive/exclusive cursor handling,
filtered empty batches and initial snapshot/feed alignment require selected-source checks before
delivery; the guide's narrative is not a protocol proof. Native versionstamps remain distinct from
source coverage, event identity and scientific time.

Asynchronous events are suitable only for repeatable derived side work with visible freshness.
The [event reference](https://surrealdb.com/docs/reference/query-language/statements/define/event)
documents transactionally enqueued, separately executed, at-least-once behavior; external effects
are not rolled back. Do not use them to acknowledge scientific publication before its required
effects exist. Kafka/surreal-sync is not selected for these local projections: it adds a broker
participant without a demonstrated new transfer responsibility. Reopen when the actual source
and destination already need that integration, not merely when the tool can run.

Delivery means independent and overlapping captures, phantoms/absence, source and indirectly
referenced updates, import/rebuild, migration, feed duplication/expiry and interrupted destination
apply all have truthful current/partial/stale status. A failed refresh must preserve the prior
identified view rather than publish a mixed cut. Quantitative contention/rebuild savings remain
unmeasured; no throughput claim follows from deleting a broad witness in a design document.

## Discovery correspondence and integration limits

| Captured direction | Review disposition |
| --- | --- |
| Read-time versus stored derivation; summaries, views, empty aggregate identity | O11 selected query-first approach; views/events selected only for complete source-local maintenance |
| Synchronous versus asynchronous events | O11 preserve atomic required effects; repeatable asynchronous derived work only |
| Shared witness granularity, native predicate conflicts, scoped invalidation | O11 target with O04 transaction design; no deletion of semantic/authority fences by assumption |
| Live refresh, state exit, permissions and reconnect | O10 selected user capability with owned invalidation/snapshot realization |
| Live router and retained notification history | O10 not default; conditional on useful subscriber/reconnect workload and exact selected semantics |
| Durable feeds, destination cursor, source expiry and reconstruction | O10 separate transition contract; O11 conditional incremental projection, full rebuild retained for small selections |
| Materialized inverse membership | O08 graph representation; a summary is not a release/capture authority |
| `DROP`, Kafka/surreal-sync, general event broker | Not selected for retained scientific originals/local projections; explicit functional triggers above |

No runtime probe was needed to settle these design recommendations; delivery confidence in the
conditional boundaries remains limited. Exact tagged view/import behavior and feed reader edges
need focused verification when selected. `DEFINE TABLE` timed out during primary retrieval;
view details retain their supplied v4 reference/addendum evidence. Live, feed and async-event
primary pages were inspected on 2026-11-09. Historical tests were not rerun or promoted into fresh
qualification. Source-wide absence, safe global guard deletion and durable subscription delivery
are not claimed.
