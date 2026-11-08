# SurrealDB v4 fresh-generation decision

**Accept-scoped.** Independent change assessment against
main `9e301a03d` and core 3.4, efficient architecture 1.0, test assessment
1.2 and process simulator 1.5. The initial design assessment covered the
private candidate. The receiving assessment extends the decision to the
default source and fresh catalogs, with historical v3 state and receipts
held by their original owners. [ADR-0257](../../adr/0257-rebuild-canonical-storage-on-surrealdb-v4.md)

Plan67's existing regeneration and named-original custody separate durable
scientific meaning from replaceable catalog layout (AP-03, AP-04, PS-13).
A fresh level-4 catalog reuses the generated declaration, preparation,
activation and settlement owners. Staying on patched 3.3.4 preserves the
current baseline and its workarounds. The vendor's experimental in-place
path is irreversible, can carry forward stale indexes and supplies no
PSE-specific conversion contract for patched persistent MX. A parallel
operating route would duplicate write and custody ownership (AP-07).
[Vendor migration guide](https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x)

The proposed candidate retains the existing MX and client boundaries.
Writable transaction paths must keep SSI, guarded replay and honest
unknown effects. v4 gRPC permits session reattachment across connections,
while PSE charges a connection-bound session through actual cleanup.
Unary status text alone cannot preserve typed conflict details. API stage
savepoints cannot establish whole-transaction rollback. The extension's
new invocation limits must preserve PSE's fail-fast admission and original
deadline (DP-09, DP-15). These are implementation and receiving questions
for the exact candidate. [Tagged gRPC server](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/server/src/rpc/grpc.rs) ·
[Tagged API executor](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/core/src/exec/api.rs) ·
[Extension runtime](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/core/src/surrealism/runner.rs)

Use existing tests where their assertions distinguish the required outcome.
Add a case where a changed consumed contract has a specific coverage gap.
Actual installed semantics, peer behavior and protected scientific readback
need the native receiving path. Plan68a/b own candidate results and patch
dispositions. Plan68e owns assembled recovery and selection. Its native
receiving results exercised the exact private runtime, restart and activation,
bounded query and Study behavior, and developer fixture disposal. The aqueous
case completed the scientific and portable readback assertions and then timed
out in its fixture drain. Plan66 retains that lifecycle obligation. Keep the
scope to fresh v4 catalogs, with one selected writer and a frozen pre-write
return route. The existing observations establish no comparative performance
claim.
[Plan68](../../plans/68-surrealdb-v4-cutover.md)
