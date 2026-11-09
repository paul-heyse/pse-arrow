# Storage, durability and artifacts

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283`, preserving the
dirty Plan66 work. Current source/configuration and selected upstream interfaces are
**Interface-checked**; alternatives and benefits are **Proposed**. No recovery, allocation
or performance campaign ran. Engine and artifact-layout breaking changes remain eligible.

## What currently persists and who owns it

[Server preparation](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-host/src/server.rs) selects a patched source-built
v4 server with MX 1.1.0. [Source integration](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/scripts/surrealdb-source/source-build.md)
names the retained runtime, extension and compiler patches. The persistent MX profile uses
explicit durability/isolation controls; this is not an unconfigured ephemeral `mem` deployment.

[Native service](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/native_service.rs) constructs private
state, credentials and launch evidence, checks readiness, and owns actual shutdown/reopen/disposal.
[Sealed files](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_files.rs) store opaque payloads
with physical instance, digest, encoding, extent and interpretation. Only an attached catalog
receipt acknowledges durable publication. [Result readers](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_results.rs)
combine protected metadata with selected payload reads; [retention](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_retention.rs)
separates logical removal, protected roots and physical reclamation.

The existing workload thus includes transactional metadata, immutable/bounded scientific
payloads, portable products and disposable development corpora. Their different access and
retention needs provide a stronger basis for engine choice than one database-wide size label.
Blueprint §20–§21 and the [native substrate guide](../../../../dev/surreal-substrate.md)
own current contracts; changing engine does not require preserving incidental file layout.

## Capability families worth comparing

| Facility | Origin / realization | Relevance and limit |
| --- | --- | --- |
| MX MVCC, ordered/borrowed scans, persistence | Existing standalone capabilities; selected integrated MX 1.1 | Low-latency mutable metadata; whole working set and retained versions consume memory. |
| MX AOL/snapshots, cleanup and diagnostics | Existing, independently configured | Reduce bespoke persistence/maintenance where supplier controls cover the actual acknowledgement and recovery promise. |
| SurrealKV LSM, versioning, value-log separation | Standalone 1.2.0 in v4 dependency reference | Persistent capacity, historical binary payloads and less large-value compaction; integrated exposure needs checking. |
| SurrealKV encryption and integrity tools | Standalone capabilities | Protect/diagnose existing stored credentials or original artifacts if exposure is justified and keys have an owner. |
| Database historical `VERSION` reads | Backend-dependent | Potentially simplify some historical lookup, without substituting storage time for authored/effective scientific time. |
| Stable file/bucket storage and presigned access | Stabilized/expanded in v4 reference | Alternative to a separate sealed-file service for existing payloads; transfer atomicity and protection remain substantive. |
| SurrealDS | V4 enterprise distributed option | Existing continuity/read-scale needs only; new replication and local-log operational participants. |
| Earth/Cloud, separate TiKV, browser OPFS | Separate deployments/integrations | Lower priority without an existing availability, hosting or browser consumer. |

MX source documents borrowed inspection and caller-reused scan buffers; these are possible
building blocks for a small lookup-only derived store. A standalone engine would relinquish
SurrealDB's schema, graph, index and authorization services unless replaced. A second store
also introduces a new transaction/recovery boundary.
[MX 1.1.0](https://github.com/surrealdb/surrealmx/blob/v1.1.0/README.md).

SurrealKV's selected reference exposes historical reads, optional value-log separation and
distinct immediate/eventual durability. Those are meaningful alternatives for large immutable
or versioned bytes. Its standalone defaults must not be imported as the integrated backend's
behavior. Vendor benchmark rankings do not compare this repository's whole operation.
[KV 1.2.0](https://github.com/surrealdb/surrealkv/blob/v1.2.0/README.md).

## Prioritized directions

**Compare MX and integrated KV using the current metadata/payload split.** Determine whether
the real cost is retained metadata versions, payload duplication, recovery duration or query
work. If disk-backed canonical metadata removes a RAM-residency constraint without introducing
more coordination, it is eligible despite the existing MX decision. Conversely, keeping compact
hot metadata in MX while retaining large scientific payloads separately may remain the simpler
complete design. Serializable predicate protection differs by backend; identify which guarded
operations would need another realization before comparing performance.

**Investigate native buckets as a replacement for a complete file lifecycle.** Use one existing
protected result path to compare upload/seal, exact digest and original instance identity,
catalog attachment, interrupted transfer, reader protection, expiry/release and orphan recovery.
This could eliminate local file-routing and transfer machinery if native storage provides the
needed complete custody. A file value or presigned URL alone cannot replace publication and
collection authority. Compare direct selected reads and independent Arrow/Proto interpretation,
not only an upload succeeding.

**Reconsider historical representation at a real current consumer.** KV versioning might
replace an application-maintained sequence for a purely recorded-time lookup. Existing research
history also has authored/effective intervals, corrections, tombstones and immutable interpretation;
storage timestamps do not encode those meanings. Native history may still improve physical
retrieval beneath a declared semantic projection. Original message bytes plus an identified
descriptor can preserve interpretation without making those bytes graph-queryable.

**Delegate maintenance where the engine has the needed lifecycle.** Compare native cleanup,
compaction and snapshot APIs against current reclamation orchestration. Target removable work,
including redundant full scans and repeated serialization. Retain exact protection checks and
original-operation recovery wherever native maintenance cannot discharge them. Logical deletion,
version reclamation, allocator release and returned filesystem space are separate observations.

## Discriminating questions and failure scenarios

1. Which growing live set belongs to current readers, historical versions, cache, payloads or
   unreclaimed files? An approximate engine-byte gauge is insufficient to choose an engine.
2. Which integrated backend options are actually exposed by the selected source/features and
   its patches? Inspect the adapter, server declaration and recovery tests together.
3. Does a candidate preserve acknowledged-write recovery after restart and uncertain replies,
   plus physical reader/file custody, with less total machinery?
4. Is replication justified by existing recovery/availability needs? SurrealDS's single ordered
   write path and additional nodes/object storage need a whole-operation comparison; licensing
   is not a rejection reason.

Future focused evidence should distinguish process-crash recovery from device power loss,
exercise snapshot/AOL interruption and corruption refusal, and compare reclamation under a
held reader. No new fault-injection run was necessary to establish these research priorities.

## Evidence limits and linked work

The supplied [v4 storage overview](../../../../libraries/surrealDB/SurrealDB_v4.md) and
[standalone engine addendum](../../../../libraries/proto/Protobuf_SurrealDB_v4.md) were consumed
with exact engine READMEs. The public in-memory page fetched 3.x/2.x material and the SurrealDS
page was inaccessible. DS topology and managed-service claims remain supplied-reference leads,
requiring primary verification if promoted. Full feature/adapter eligibility and empirical
durability remain unsettled. [Transactions](transactions-and-guarded-state.md),
[native transport](native-values-transports-and-clients.md) and
[rollout migration](deployment-rollouts-and-migrations.md) own adjacent questions.
