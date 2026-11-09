# Derived state and change observation

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283` plus the current
dirty Plan66 tree. Representative source and selected supplier interfaces are
**Interface-checked**. Improvement directions are **Proposed**; no database probe ran.

## Current responsibilities and promising leverage

The existing system already delegates some mutation reactions to native events. The
[study capture witness generator](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-codegen/src/codegen/surreal_study_capture_witness.rs)
emits events for selected changes across study, point, run, attempt, result, project and
research tables. Each relevant write increments a shared publication witness. This protects
selected capture publication, including absent/future matches; it excludes incidental leases
and heartbeats. It is an actual consistency mechanism rather than generic observation.

[Result readers](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_results.rs) select current
manifests, terminal attempts and bounded block/batch pages under protected envelopes.
[Private knowledge](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/xtask/src/development/knowledge.rs) maintains a selected
generation and bounded source/subject/topic projections. Its v4 read-only snapshot is already
implemented. The derived development corpus has a separate lifetime from original scientific
records. Blueprint §20–§21 own their current meaning; Plan66/68 own remaining implementation
and qualification status.

The main opportunity is to let native derivation maintain an existing useful projection
or notification, eliminating repeated scans or application-managed update paths. A durable
receipt and a current observation have different promises, even when both describe one run.

## Capability map

| Native mechanism | Origin / availability | Appropriate research target |
| --- | --- | --- |
| `VALUE`, `DEFAULT`, `ASSERT`, `COMPUTED` | Pre-v4; v4 tightens effects | Write normalization versus current read projection; avoid storing/revalidating a cheap derived fact unnecessarily. |
| `DEFINE TABLE ... AS SELECT`, grouped views | Pre-v4; v4 fixes projected row identity | Repeated summaries over a declared source population, maintained by source mutations. |
| Synchronous events | Already used | Atomic projections or witnesses genuinely owned by a source write. |
| Asynchronous events and retry controls | Present in v4 reference; exact selected semantics need research | Work that can occur after commit without weakening the original acknowledgement. |
| Tracked/materialized references | Earlier tracking plus v4 materialization | Native inverse membership instead of repeatedly maintained back-reference lists. |
| Live queries | Pre-v4; v4 registration/permission changes | Current status observation replacing repeated polling where loss is recoverable by reread. |
| Changefeeds | Pre-v4, explicitly enabled | Incremental refresh of an existing derived corpus with retained catch-up. |
| Experimental live router | Separate opt-in in supplied v4 reference | Lower subscription fan-out cost only if an existing observed workload warrants it. |

See [reusable runtime contracts](../v4-runtime-contracts.md) for live/replay distinctions,
and the [graph brief](graph-representation-and-traversal.md) for relationship representation.
Do not infer repository-wide absence of these facilities from the selected source samples.

## Priority directions and useful compositions

**Investigate native summaries before another cache.** Select one repeatedly computed existing
workbench/result/evidence summary. Compare a maintained grouped view, a count/index-only query,
and the current selected query. A view could delete custom counter/projection-update code;
an indexed query could avoid the maintenance and freshness machinery entirely. Keep original
scientific data reconstructible. `DROP` ingestion is a poor default for original evidence
whose retention or independent interpretation matters.

**Reconsider witness granularity as part of capture design.** Native events already centralize
relevant mutations, but a single shared witness can create conflicts among unrelated captures.
Compare scope-specific witnesses, precise native predicate conflict tracking, and the current
conservative generation. The goal is complete dependency coverage with less unrelated invalidation
or contention. A materialized view is not itself a replacement for proof that an absent record,
membership change or indirectly referenced record could affect the captured result.

**Use notifications to invalidate an observation, then read the authoritative selected state.**
An existing status reader could subscribe to a narrow run/attempt selection and refresh its
bounded view when notified. This might remove periodic database work while retaining recovery
through ordinary reads. A notification should not grant release authority, extend a result
protection or settle an uncertain write. Account for the subscription's admission, permission
changes, cancellation and reconnect lifecycle; the current request-only client does not acquire
those semantics merely by accepting `LIVE SELECT` text.

**Consider changefeed-driven rebuildable projections only where repeated import has material cost.**
For the existing private knowledge/evidence corpus, retain source selection and mapping identity,
apply a feed batch and destination cursor together, and reconstruct on retention expiry.
Compare this with rebuilding the small selected corpus. `surreal-sync` Kafka ingestion is an
ecosystem lead, but no current Kafka participant was established in these inspected workflows;
adding a broker would require a concrete simplification over existing local operations.

## Counterconditions that can defeat value

Maintained views refresh from declared source writes, not arbitrary dependencies reached by
links or subqueries. A projected related name or result status can therefore become stale.
Import semantics can bypass ordinary maintenance and events; reconstruction needs an explicit
refresh path. The supplied v4 addendum describes stable `GROUP ALL` empty results and corrected
projected identity; neither establishes complete dependency inference.

Async reactions can outlive the originating commit and have separate retry/failure semantics.
External effects require their own idempotency and recovery. More durable observation is not
automatically better for disposable developer evidence: cursor storage, feed retention and
cleanup can exceed the cost of a bounded rebuild.

Live filters need to cover records leaving a selected state when the reader must remove them.
Committed notifications are permission-filtered and have no global writer order guarantee.
Changefeed cursors are inclusive in current documentation and can represent multiple mutations;
cursor progress, application event identity and exact source membership remain distinct.
[Live guide](https://surrealdb.com/docs/learn/querying/real-time/live-queries) and
[changefeed guide](https://surrealdb.com/docs/learn/querying/real-time/changefeeds).

## Next discriminating questions

1. Which present derived computation is repeated enough to justify maintenance, and what are
   all mutation, deletion, import and permission dependencies of its answer?
2. Can a scoped native representation replace the shared capture witness without missing an
   absent/future match? Compare conflict behavior for independent and overlapping captures.
3. Does the actual observer need every transition, or only a current refresh after reconnect?
   Choose subscription, feed or ordinary query from that requirement.
4. Can the source be reconstructed after feed expiry using its original selected membership,
   interpretation and payload owners, without silently switching to newer originals?

A later narrow probe should exercise source and indirectly referenced writes, final-row deletion,
import/rebuild, rollback, leaving a live predicate, disconnect and expired feed history.
No such campaign or throughput claim is made here.

## Evidence boundaries

The supplied [v4 overview and addendum](../../../../libraries/surrealDB/SurrealDB_v4.md)
and [Protobuf integration addendum](../../../../libraries/proto/Protobuf_SurrealDB_v4.md)
provide broad capability leads. Live/changefeed primary guides were inspected. The `DEFINE TABLE`
primary page timed out during this execution; detailed v4 view identity/import behavior remains
reference-backed and should be checked against tagged tests before selecting it. Historical
[SurrealQL type research](../surrealql-server-constructs.md) retains its own 3.3 evidence date.
