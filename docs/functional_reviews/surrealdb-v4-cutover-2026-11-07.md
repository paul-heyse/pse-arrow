# SurrealDB 4 cutover and functional extensions

**Interface-checked** against main `9d80d9836` and the upstream `v4.0.0` tag.
The selected standard is core 3.4, efficient architecture 1.0, test assessment
1.2, process simulator 1.5, and the pse-arrow binding. Plan66 owns scientific
state, execution and independent interpretation. Plan67 owns history, database
tests, build reuse and retirement. Their existing end-to-end obligations apply
to the replacement.
[Plan66](../plans/66-integrated-simulation-target.md) ·
[Plan67](../plans/67-storage-lifecycle-and-build-reuse.md) ·
[SurrealDB 4 reference](../libraries/surrealDB/SurrealDB_v4.md) ·
[Protobuf and SurrealDB 4 reference](../libraries/proto/Protobuf_SurrealDB_v4.md)

## Deployment choice

**Proposed.** Build one exact v4 release and regenerate the catalog in a fresh
owned state directory. Recreate authored and reproducible products through their
native producers. Preserve only named original evidence with active consumers,
using portable readback first and one-time bounded import when the consumer
requires it. A regenerated scientific run receives a new identity. The alternative
in-place conversion preserves more v3 state but is experimental and one-way, and
would constrain the new schema around material the design phase can regenerate.
Keep the frozen v3 selection through the rehearsed pre-write rollback point.
After enabling v4 writes, use v4 forward recovery.
[Migration guide](https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x)

Keep persistent MX and the bounded gRPC client. v4 includes MX 1.1.0 and
explicit serializable transactions, but implicit statements still default to
snapshot isolation upstream. Its gRPC sessions are process-wide capabilities
that can outlive TCP connections, and unary errors are reduced to a code and
bounded text. Retire individual patches only when v4 native behavior and the
PSE controls establish equivalent conflict, durability, typed errors and
settlement. Port the pure study policy to an exact digest-bound v4 extension and
qualify the fresh-instance ABI and physical allocation limits.
[Release lock](https://github.com/surrealdb/surrealdb/blob/v4.0.0/Cargo.lock) ·
[Transactions](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions) ·
[Tagged gRPC implementation](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/server/src/rpc/grpc.rs) ·
[Extensions](https://surrealdb.com/docs/reference/query-language/statements/define/extension)

The exact candidate is server `v4.0.0` at
`284fe93afc851c3baec2c262c01289602021a70d`, SurrealMX 1.1.0,
`surrealdb-types` 4.0.0, `surrealdb-protocol` 0.14.1, and Surrealism 0.7.0.
Track the database executable and separate `surreal-ext` builder in the
same source provenance. The extension definition digest identifies its manifest,
while the prepared artifact receipt also identifies the complete archive. v4
already locks its new DiskANN dependency. Compare that locked source and the
pinned compiler before dropping the old archive and compiler patches.
[Tagged server manifest](https://github.com/surrealdb/surrealdb/blob/v4.0.0/Cargo.toml) ·
[Extension definition](https://surrealdb.com/docs/reference/query-language/statements/define/extension)

The old parser and AST crates have been replaced. Use the v4 syntax and
stored-definition entry points for the consumers that need them, with installed
definition readback from the selected server as the activation witness. Check
permissions, credential redaction, table-clause uniqueness, numeric comparison,
missing versus null, and result cardinality against actual generated inputs.
The tagged v4 API executor handles failures with per-stage savepoints and can
return an error response as a value. Exercise an enclosing earlier write before
considering retirement of the whole-transaction rollback patch.
[Tagged parser APIs](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/expr/src/parser/api.rs) ·
[Upgrade behavior changes](https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x) ·
[Existing patch purposes](../../scripts/surrealdb-source/source-build.md)

<a id="O01"></a>
## O01 — Share generated storage constraints

Use generated `DEFINE TYPE` for a repeated constraint consumed by stored fields
and guarded statements. A declared type is useful when its coercion, assertion
and permissions match the original native meaning. Keep domain descriptors and
historical interpretation independent. Preserve presence, occurrence identity
and exact numeric policies at the database boundary. v4's native protobuf codec
widens or degrades selected value types, so its accepted transport subset needs
independent receiving controls.
[Named types](https://surrealdb.com/docs/reference/query-language/statements/define/type) ·
[Tagged value conversion](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/value.rs)

<a id="O02"></a>
## O02 — Inspect protected results and dependencies

Use current read-only snapshots for an effect-free multi-statement inspection.
Keep ticket admission and publication in their guarded operations. Compare
structured `EXPLAIN ANALYZE` observations for protected metadata and immediate
dependency reads with the same fixtures and semantic results. The existing graph
already has inline caches. Tune them when the receiving operation benefits.
Recordless edges fit endpoint-only membership and require a demonstrated case
without occurrence identity or payload.
[Read-only transactions](https://surrealdb.com/docs/reference/query-language/statements/begin) ·
[Executed plans](https://surrealdb.com/docs/reference/query-language/statements/explain) ·
[Graph options](https://surrealdb.com/docs/reference/query-language/statements/define/table)

<a id="O03"></a>
## O03 — Triage history and retire resources by owner

Connect original failure to scoped readback and current-inventory rerun after the
old executable is released. Use ordered bounded expiry selection with a stable
tie-breaker, followed by owner rechecks of holds and custody. Compare v4 B-tree
and on-demand aggregates before adding maintained views. A view earns its
storage only with repeated demand and complete source-table invalidation.
Measure allocated footprint separately from logical expiry.
[Index execution](https://surrealdb.com/docs/reference/query-language/statements/define/indexes) ·
[View maintenance](https://surrealdb.com/docs/reference/query-language/statements/define/table)

The current history publisher and reader are pilot interfaces. Connect the
verified projection to the existing Plan66f resolver as a source of original
selection facts. Resolve against current inventory under the resolver's own
admission and preserve the original verdict, the rerun outcome and any expired
raw member as separate facts. Exercise unavailable history and a changed or
missing current target as well as the successful rerun.
[History pilot](../plans/67b-invocation-custody-and-developer-history.md) ·
[Current-inventory rerun](../plans/66f-developer-operations-clients-and-evidence.md)

**Tested, isolated query only.** A disposable memory session with the official
4.0.0 executable used four records with expiries 99, 100, 100 and 101.
`EXPLAIN ANALYZE FORMAT JSON` for the existing scalar expiry index with
`WHERE expires_unix_ms <= 100 ORDER BY expires_unix_ms ASC, id ASC LIMIT 2`
showed a forward range scan with the limit pushed into the index and no
separate sort. The result returned the 99 row and the lower record ID at
100. This supports retaining the scalar index for the proposed first slice.
Persistent reclamation and the receiving history workflow require separate
qualification.

<a id="O04"></a>
## O04 — Consolidate database tests on an owned v4 endpoint

Select SurrealKit 1.0.0-rc.1 for a disposable native fixture. Stage the generated
definitions and preserve original suite results and distinct setup, assertion,
and settlement outcomes. Retain native physical and scientific test owners.
Compare effective Cargo units and eligible cache reuse before claiming benefits.
[SurrealKit release lock](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/Cargo.lock)

## Alternatives and delivery evidence

Keep history on MX for the pilot. Reopen separate SurrealKV placement when
capacity or recovery isolation requires it. Reopen changefeeds only for a named
consumer with a continuity and recovery contract. Alternate transports,
additional retrieval modes and transactional extension duties require a concrete
receiving operation and evidence of better end-to-end fit.

The rehearsal pairs the protected scientific result journey with independent
readback, history publication and rerun, and SurrealKit failure and cleanup.
Verify database durability, typed conflicts, scope isolation and the selected
original evidence before switching. Source findings establish interface fit.
Local qualification and any measured benefit belong to the owning plans.

## Suggested delivery order

1. Qualify exact source preparation and the v4 client, parser and extension
   against a private generation. Run the native conflict, connection loss,
   saturation, lost acknowledgement, crash and restart controls before removing
   the corresponding v3 patches.
2. Regenerate the catalog from source owners. Inventory held evidence by
   consumer and verify each selected original through its receiving reader.
   Exercise the protected scientific result and history-to-rerun journeys,
   including refusal and interruption, before considering the new generation
   ready. Plan67's build comparison and incident retirement retain their own
   evidence and can advance independently where their inputs are ready.
3. Rehearse the pre-write rollback, then select the verified v4 generation as
   one unit. Enable writes after successful readback and admission checks.
   Release the frozen generation through the owning retirement path once its
   selected evidence is independently readable.
