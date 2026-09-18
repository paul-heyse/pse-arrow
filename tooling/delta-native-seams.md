---
title: Reproducible Delta native integration interfaces
status: in-progress
date: 2026-09-17
---

# Delta native integration interfaces

**Implemented; interface-checked by `just check` (workspace, all targets).** ADR-0070 and Plan 09 C04 select a
repository-owned source override of delta-rs at the full revision declared in
`Cargo.toml`'s `workspace.metadata.pse.delta-source-revision`. Its Arrow/DataFusion
and kernel dependency universe stays in the root Cargo lockfile.

`vendor/delta-rs` is generated third-party source. Edit
`tooling/delta-native-seams.patch`, then regenerate with
`just delta-source <read-only-upstream-checkout> --apply`. The default command
verifies every output against immutable git blobs and the patch. No source cache,
external checkout, upstream branch or private Delta identity is modified.
`PROVENANCE.json` records upstream and resulting file SHA-256 values and patch SHA-256.
The library-only manifests exclude upstream development harnesses and benchmarks.

The narrow additions are:

- `CdfLoadBuilder::with_file_metadata_cache`: inject the caller's actual namespaced
  native cache; native file selection, CDC semantics and reader factory remain Delta's.
- `Snapshot::estimated_owned_heap_size_bytes` and `EagerSnapshot::snapshot_ref`:
  retain native snapshots and account kernel/log metadata plus materialized action and
  statistics arrays. Extents are estimates, shared buffers are charged conservatively,
  and upstream replay allocations remain bounded staging exposure rather than a hard
  allocator guarantee. No serialized snapshot or copied private identity algorithm.
- `DeltaTableBuilder::with_crc_replay_max_commits`: bound the kernel's incremental
  CRC replay on new snapshots and forward updates. Default zero disables advancement.
- `Snapshot::write_checksum`: native post-commit CRC writing with a separate result
  and updated native snapshot. Callers must retain a successful data commit when this
  acceleration fails; never repeat the effectful input. CRCs do not contain all Add
  actions and do not replace native active-file replay.
- `kernel::native`: re-export the exact resolved kernel for the native transaction
  fixture that creates a real `CommittedTransaction` checksum seed. This introduces
  no second kernel dependency or application checksum format.
- `DeltaCdfTableProvider::supports_filters_pushdown`: predicates referencing CDF
  metadata stay in DataFusion's residual filter. The pinned native file/action
  pruning schema contains only table columns, so advertising Exact support for
  `_commit_version`, `_commit_timestamp` or `_change_type` fails planning. Ordinary
  table-column predicates keep the native pushdown path.

**Tested:** `native_kernel_crc_seed_advances_through_delta_writes_and_corruption_falls_back`
passes under force-validation (baseline zero). At this pin an ordinary Delta snapshot
without a checksum cannot bootstrap one: the kernel returns
`ChecksumWriteUnsupported`. A native committed-transaction seed can advance across
Delta writes. Missing/unsupported CRC remains a replay fallback; no data write is
repeated, and CRC defaults remain disabled pending the cost matrix. A checksum does
not establish full active-file enumeration. The qualification fixture uses the actual
kernel transaction, never fabricated checksum JSON.

The local Delta skill's original pin remains the upstream API reference. These
additions are an explicit source overlay and must not be represented as upstream
APIs until upstream publishes equivalent interfaces.
