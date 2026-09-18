# Selected PSE Delta source overlay

Interface-checked against the generated source selected by root Cargo metadata.
The upstream index remains pinned to delta-rs 58f07cd62bfbce3649a7e1c87c696288068ae184
and kernel 8ba063f8f84fec222000f66d40d70911d7c79675. The following additions exist only
in the reproducible `vendor/delta-rs` override. Do not describe them as upstream APIs.

| Selected source symbol | Consumer / purpose |
|---|---|
| `CdfLoadBuilder::with_file_metadata_cache` | `pse-catalog::delta::changes`: actual shared qualified native metadata cache |
| `DeltaTableBuilder::with_crc_replay_max_commits` / `DeltaTableConfig::crc_replay_max_commits` | Native kernel incremental replay bound on construction and update |
| `Snapshot::estimated_owned_heap_size_bytes` | Kernel/log/materialized batch extent estimate for pool-owned snapshot retention |
| `EagerSnapshot::snapshot_ref` | Retain the actual native Snapshot Arc; no serialization or private reconstruction |
| `Snapshot::write_checksum` | Native checksum outcome plus updated snapshot, separate from data commit success |
| `kernel::native` | Exact resolved kernel transaction API for real committed-transaction checksum seed qualification |
| `DeltaCdfTableProvider::supports_filters_pushdown` | CDF metadata predicates use native residual filtering; table-column predicates retain native pushdown |

The root `tooling/delta-native-seams.patch` owns the changes. `scripts/vendor-delta.py`
reconstructs files from immutable git blobs and records hashes in
`vendor/delta-rs/PROVENANCE.json`. Verify with `just delta-source <upstream-checkout>`;
edit the patch and regenerate with `--apply`. The original indexed corpus is not
silently replaced by this overlay. Cargo.lock owns the selected dependency universe.

CRC summaries do not contain the full Add set. Size is an estimate, not an allocator
hook. Native fallback on absent CRC is an acceleration miss. CDF injection preserves
Delta's own reader/file selection/change semantics. Integration acceptance remains
tracked in Plan 09, independently of callable interfaces.

The native kernel requires a checksum-producing committed transaction or an eligible
existing checksum to write/advance CRC. An ordinary delta-rs-loaded snapshot without
that seed returns `ChecksumWriteUnsupported`; configuring a replay limit does not
create a seed. The catalog unit qualifies a real kernel transaction, subsequent Delta
writes, stale CRC advancement and corrupt-CRC fallback. Preserve this limitation when
choosing defaults; do not manufacture CRC contents or imply ordinary writes bootstrap it.

The upstream skill's syntax-based write/commit-properties hint is not a deduplication
proof: at this pin SetTransaction is a durable witness; it does not by itself suppress
re-execution. PSE settles complete member/root receipts and never repeats a data write
merely to retry checkpoint or checksum acceleration.
