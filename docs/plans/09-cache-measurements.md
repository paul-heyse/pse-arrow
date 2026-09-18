---
title: Native cache measurements and deployment defaults
status: done
date: 2026-09-17
adrs: [ADR-0070]
phase: 1
---

# Native cache measurements and deployment defaults

## Evidence boundary

**Measured:** `just bench-cache`, release profile, full matrix, baseline zero,
completed with 84 JSON records in
`build/plan09-acceptance-20260918-13/11-bench-cache.log`. This production-source
run includes shared support producers, completed native streaming readers and
explicit store-binding accounting. It supersedes the earlier cost receipt in
`/tmp/pse-plan09-cache-matrix4.log`; the IO controls and selected conservative
defaults remain unchanged. No full-model compilation speed claim is derived from
the small cache fixtures. The campaign captures exact source and dependency provenance.

The fixture uses DataFusion 55.1.0, Arrow/Parquet 59.3.0, object_store 0.13.2,
Rust 1.98.1, the pinned Delta revision and repository-owned overlay. Inputs and
commands are in `benches/benches/native_cache.rs` and its modules. Preparation,
execution, fixture construction, IO attempts and consumed bytes are separate
fields. Host OS caches were uncontrolled; separate-process reads are not cold-disk
measurements. Other qualification processes ran concurrently. Elapsed times are
observations, not isolated latency comparisons or optimal-setting evidence.

## Mechanisms and independent results

| Experiment | Observed result | Interpretation |
|---|---|---|
| Fixed-point graphs with 3/8/16 nodes | 5/10/18 rounds; 20 analyses, optimizations and physical plans and 2 SQL bindings in every case; 46/91/163 reusable executions | Planning does not grow with round count; two shared support producers are now explicit, and independent closure assertions remain in the fixture |
| Prepared rounds at 16/1,024/16,384 rows | One analysis, optimization and physical plan, 20 executions | Reset uses stable round inputs, without rebinding SQL or rebuilding the plan |
| 1,000 commits, interval 10 versus no checkpoints | 12 requests / 40,245 bytes versus 2,001 requests / 11,367,542 bytes; three repetitions agree on IO | Native checkpoints bound this log-tail work; this does not prove universal constant-time opening |
| 32 member roots at 1,000 commits | Fresh retained-cache fill: 384 requests / 1,287,840 bytes; warm: zero gets and 32 listings | Exact retained snapshots eliminate replay reads while preserving fresh native selection checks |
| Incremental snapshot advance | 2 log gets / 11,298 bytes | Native forward update reuses the snapshot seed |
| Four CDF image kinds | Identical counts: 16 inserts, 1 delete, 1 preimage, 1 postimage | Cache state preserves actual CDF semantics |
| Warm CDF metadata cache | 7 gets / 14,745 bytes versus 13 / 41,259 without cache | Injected native metadata reuse reaches the actual CDF reader |
| Resident capacity 0 / 64 KiB / 8 MiB, 16,384 rows | 0 / 14 / 4 resident loads; largest capacity produces 10 hits | An undersized cache can repeatedly bypass; bounded admission remains correct |
| Wide scan, selective/unselective, 1/4 readers, predicate cache 0/1 MiB | Each reader returns 8/4,096 rows respectively | Pushdown and residual correctness hold; no stable latency advantage establishes a nonzero default predicate cache |
| Actual CRC seed, metadata/file load classes | Metadata gets 3 without CRC versus 2 with CRC; file enumeration gets 5 versus 4 | CRC assists metadata but does not replace active-file replay; consumed bytes were slightly higher in this small CRC fixture |

Maximum observed retained entries were 59,484 bytes of metadata, 1,661,256 bytes
of native snapshots and 940,010 bytes of resident values. Store bindings are also
reported; the 32-root fixture retains 17,430 bytes of binding ownership. These are workload
observations, not maximum allocations for arbitrary models. Peak RSS includes
allocator history and native transient allocations. Unavailable native decoded-row
and replay-action counters remain NULL rather than being inferred from IO bytes.

## Complete engineering publications

**Tested and Measured:** all four current heater/mixer configurations complete
390-relation publication and independent Rust and separate-process Python inspection.
The [engineering receipt](../design_review/evidence/native-cache-engineering-2026-09-18.json)
preserves the actual relation counts, commands, ownership assertions and source
boundary. Rust uses the dev profile with force-validation, one worker, one partition
and a 32 GiB shared ceiling; Python uses the integration marker with one worker.
Baseline zero: four Rust cases and four Python cases pass, with no retries.

| Case | Compose (s) | Execute/publication (s) | Rust reopen (s) | Python open/stream (s) | Peak compile reservation (bytes) |
|---|---:|---:|---:|---:|---:|
| Heater FTPx | 29.319 | 398.611 | 7.804 | 8.963 | 15,499,050,922 |
| Heater FcTP | 31.199 | 428.066 | 8.271 | 8.310 | 15,414,364,169 |
| Mixer FTPx | 31.307 | 444.548 | 8.353 | 7.391 | 19,600,194,958 |
| Mixer FcTP | 31.249 | 448.458 | 8.644 | 7.031 | 19,598,007,207 |

The Python checks retain exported buffers across handle closure, verify unchanged
persisted-file hashes, and require that final reservations equal reported cache
ownership with no outstanding pins, loads or in-flight bytes. These are complete
publication costs, not numerical solver times. Other qualification ran concurrently;
OS caches were uncontrolled. There is no predecessor comparison or universal speedup
claim. The Rust publications predate final construction/admission repairs; the Rust ledger
records affected qualification. All four independent Python readers were repeated
using the final rebuilt extension at source snapshot 25, with the same ownership
and unchanged-file assertions.

## Final kernel journey

**Tested and Measured:** `/tmp/pse-plan09-kernel98.log`, CI nextest with
force-validation, passes in **309.658 seconds** under the unchanged 360-second
deadline. Composition takes 31.321 seconds and execution 231.847 seconds; the same
case then publishes and reopens all 390 outputs, checking exact kernel natural units
and conversions. One worker/partition, the existing 32 GiB ceiling, concurrent checks,
uncontrolled OS caches and one 0.52-second debugger sample bound this observation.
Earlier timeouts remain in the Rust ledger. No controlled percentage improvement is
claimed. The final repair borrows the registry's existing lossless declaration;
external candidates still undergo independent complete comparison.

## Selected conservative defaults

The numerical callback boundary has a separate **Measured** receipt:
`/tmp/pse-plan09-callback61.log`, `just test-package pse-numerics --test native_expressions
-E '"test(repeated_callback_vectors_keep_contiguous_values_and_release_their_reservations)"'
--success-output immediate`, default profile with force-validation, one pass and zero
failures (baseline zero). Over 256 callbacks the fixture observes 32,111 prepared
reserved bytes and 32,415 maximum reserved bytes, returns to the prepared reservation
after every output drop and to zero after program drop. The loop takes 22,507
microseconds on this host. This measures reservation ownership and numerical results;
it does not count every upstream allocation or establish a solver throughput claim.

**Implemented selection:** retain the bounded `CacheBudget::for_memory` policy,
with `p = floor(deployment_memory_bytes / 32)`. These are capacity limits within
one shared pool, not separate budgets or optimal working-set predictions.

| Setting | Selected value | Basis |
|---|---|---|
| Native file metadata | `min(p, 50 MiB)` | Actual Parquet/CDF reuse is exercised; capacity remains deployment-scaled |
| Native snapshots | `2p` | Warm exact-selection replay elimination is measured; preserve room for larger native action sets |
| Resident decoded values | `4p` | The pressure control demonstrates both useful hits and undersized bypass; caller may disable or resize |
| Concurrent fills / staging | 1 / `2p` | Bound simultaneous unaccounted upstream construction; throughput tuning needs a real deployment workload |
| Entry-detail inspection | `min(p, 4 MiB)` | Bounded diagnostics must not compete without limit with model execution |
| Query working reserve | `max(floor(memory / 2), 1)` | Operators keep a declared share of the same pool; total policy must validate |
| Native statistics / directory listings | 0 / 0 | These fixtures supply no demonstrated benefit requiring their retention |
| Native predicate-result cache | 0 | Opt-in accounted capability is available; the scan matrix does not establish a default benefit |
| Checkpoint interval | 10 commits | Native interval-10 versus disabled IO control establishes practical benefit for control/member histories |
| CRC advancement / checksum cadence | 0 / 0 | The current native kernel requires an eligible transaction seed; ordinary unseeded snapshots cannot bootstrap CRC publication |
| Resident partitioning | Actual native source partitioning | No fabricated partition count or unconditional repartition is justified by these results |

Predicate pushdown remains enabled through the native reader. Other native scan
settings retain their explicitly captured caller configuration. These defaults
are not a library restriction: caller policies can use the supported native
capabilities subject to their actual resource and semantic contracts.

## Limits and follow-up triggers

Cache-on execution is not uniformly faster, especially for small, freshly opened
fixtures. Do not advertise a universal speedup. Revisit capacities when a real
model's retained working set approaches a cap; revisit fill concurrency and
partitioning with that model's measured IO/CPU pressure; enable CRC policy only
when its actual seed/commit route is eligible. A real remote backend requires its
own validator, conditional-write and maintenance qualification. These triggers do
not defer any current local provider/cache correctness obligation.
