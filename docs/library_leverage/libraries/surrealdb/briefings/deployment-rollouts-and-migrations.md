# Deployment, rollouts and migrations

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283` plus the
concurrent Plan66 working tree. Source behavior is **Interface-checked**;
compositions and benefits below are **Proposed**. No migration or deployment ran.

## Existing responsibility and useful supplier mechanisms

[FrozenRollout](../../../../../crates/pse-operations/src/canonical_maintenance/frozen_rollout.rs)
reads bounded, hashed schema copies and explicit removals. It deliberately excludes
arbitrary SQL, variables, rename hints and data migrations. Field/table removals
require an independent data witness. [The owned preview](../../../../../crates/pse-operations/src/native_service/rollout_preview.rs)
binds the manifest to the original listener, authenticated catalog, installed study
extension and verified tool. A retained preview cannot authorize a later effect.

This is an existing deployment-inspection boundary, not evidence of an unrestricted
recurring configured migration executor. [The operating guide](../../../../dev/surreal-substrate.md)
and blueprint §20 own lifecycle and custody. [Plan68](../../../../plans/68-surrealdb-v4-cutover.md)
owns cutover status; fresh-default compatibility does not settle the best future
schema/data lifecycle.

| Mechanism | Origin/status | Potential delegation |
| --- | --- | --- |
| Sync, embedded schema, source tracking | Pre-v4 foundations; selected rc.1 tool | Desired-state reconciliation and packaging |
| Modules, targets and tracked seeds | 1.0 prerelease, retained in rc.1 | Independent reconciliation partitions and initialization |
| Frozen SQL, chain lint, ordered catch-up | Earlier prerelease, retained | Historical execution independent of current schema files |
| Incremental catalog writes and repair | Later prerelease | Recovery of schema-management metadata |
| Named field transformations and cleanup | v4 engine + rc.1 adapter | Stored-value conversion with tracked migration identity |
| `await_migrations`, reverse steps, compatibility operations | v4-aware rc.1 | Explicit conformance barriers and transition management |

See [the migration coordination contract](../v4-tooling-contracts.md#surrealkitnative-migration-coordination)
for exact supplier conditions and links. The complete [supplied reference](../../../../libraries/surrealDB/SurrealKit.md)
original §§3–6 and addendum §§5–8/11–12 distinguishes definition identity,
historical execution and record state.

## Highest-value inquiry: delegate a complete selected transition

Choose an actual generated field representation change, such as a future native
unsigned representation replacing the current Decimal guard encoding. Compare
three complete routes: fresh reconstruction, a supplier-managed named conversion,
and an explicit application backfill. Include consumers, protected originals,
indexes, publication and recovery. A breaking change is acceptable; overlapping
old readers are a requirement only if an actual participant needs them.

The native route could remove application-side scan/convert/write loops and
separate rewrite progress machinery. Its value depends on preserving scientific
meaning rather than choosing a convenient fallback. Index removal creates an
interval requiring a deliberate writer/uniqueness policy. Read-time converted
values can become visible before the background rewrite ends. A successful
statement or a matching final schema therefore does not establish safe completion.

The next discriminating question is whether this selected transition fits native
migration restrictions and what original-data witness proves its intended effect.
If fresh local reconstruction already provides the superior complete operation,
retaining a universal migration history would add needless machinery.

## Other directions worth investigating

**Use supplier history/recovery instead of parallel local bookkeeping.** Compare
the native maintenance owner's effect contract with frozen SQL, step identity,
chain lint and state-aware repair. Determine which local structures establish
listener/issuer/operation authority and which merely reinterpret supplier state.
Only the latter are plausible deletion targets. Repair of tracking metadata is
different from replaying uncertain data-changing SQL.

**Make reconciliation identity match effective inputs.** Source hashes and
text-substituted SQL need not agree. Existing frozen input rejects substitution;
that may remain simpler than adding variables. If a real current deployment needs
variables, bind effective bytes and provenance rather than treating unchanged source
as proof that desired state is unchanged. Stable module keys matter when pruning.

**Use native conformance barriers for destructive cleanup.** Compare rc.1's
completion barrier and compatibility inspection with current field/table removal
witnesses. The useful result is a smaller complete transition, not an additional
gate duplicating the same uncertainty. Sequence values, access secrets, retained
payloads and physical disk reclamation remain distinct from schema definitions.

## Reasons to retain the present approach

The current design-phase fresh-store route avoids recurring in-place orchestration
where no long-lived deployment needs it. Its narrowly admitted manifests make
possible effects reviewable. Native ownership checks retain information that a
generic target name and rollout hash do not establish. A more capable supplier
executor is not automatically a better application contract.

Definition reversal can fail to recover discarded information; arbitrary
cross-field transformations still need explicit data logic. Multi-target execution
does not introduce cross-database atomicity. Neither current acceptance nor a
schema parity test proves historical data effects.

## Discriminating work and evidence limits

First inspect one actual representation transition with its readers and index
dependencies. Then, if worthwhile, test forward conversion, interrupted completion,
unknown acknowledgement, rejected duplicate values, reverse conversion and cleanup
against independent pre/post data witnesses. Preserve original operation identity.
These are proposed focused assessments, not tests run in this discovery.

Reusable release facts were checked in tagged rc.1 source; broader rollback,
device durability and every historical application combination remain unqualified.
Link [generation](schema-compilation-and-generation.md),
[contract tests](contract-testing-and-developer-workflows.md) and
[storage](storage-durability-and-artifacts.md) when developing a complete option.
