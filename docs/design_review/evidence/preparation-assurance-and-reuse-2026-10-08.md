---
title: Preparation assurance and reuse incident evidence
date: 2026-10-08
---

# Preparation assurance and reuse incident evidence

Supporting evidence for the [principal review](../reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md).
This document records bounded observations and platform-source examination, not an
independent architectural verdict or a remediation ledger.

## Baseline and observation boundary

The observed process was pytest-xdist worker PID `591465` in the existing
`build/plan28-e3-parallel-pool-dev-20261008` assessment. Python was CPython 3.14.7;
py-spy was 0.4.2; perf was 7.0.14 on Linux `7.0.0-38-generic`.
The host reports Ubuntu glibc `2.39-0ubuntu8.9`. Exact distribution patches and
interposition were not independently qualified against the upstream source below.

The inspected checkout was dirty at HEAD `2410880efc5ab20e623006fde78e624044d3358d`.
That commit alone does not identify the executed source. The installed extension and
worker remained frozen while source and test repairs continued. Their independently
observed file identities were:

| Artifact | Bytes | SHA-256 | Modification time |
|---|---:|---|---|
| `python/pse/_native.abi3.so` | 759,951,849 | `13e2ae26763a78622c55a0c4a1cba1471f4a8870cc7879aa6d9ce4556c89ad04` | 2026-10-08 14:04:51.267517833 EDT |
| `target/debug/pse-worker` | 705,086,088 | `31d3339ea407ee26402ff49f7cf40553ee8c33756bd8a9fa793e34cba048c0a0` | 2026-10-08 12:16:27.805614821 EDT |

These are file associations for the diagnosis, not newly minted scientific qualification.
The profiler did not rebuild or replace either artifact, start another test, change
resource limits, signal/restart services, or inspect Python locals/environment values.

## Live preparation observations

**Measured — process read activity.** An initial process snapshot recorded
20,606,899,980,891 logical read bytes and 652,546,048 physical read bytes since this
worker started. These cumulative counters include its entire lifetime and are not
an exact cost attributable to one study or to hashing alone. A subsequent
24.271572453-second interval recorded another 55,119,663,342 logical read bytes,
885,990 read calls and zero additional physical read bytes.

A separate 61-snapshot sample, at 250 ms intervals from **22:12:51 to 22:13:06 UTC**,
lasted **15.028730846 seconds**:

| Observation | Result |
|---|---:|
| Logical read bytes (`rchar` delta) | 33,887,722,076 |
| Read calls (`syscr` delta) | 545,207 |
| Physical read bytes (`read_bytes` delta) | 0 |
| Physical write bytes (`write_bytes` delta) | 0 |
| Native `pse-math` thread CPU seconds | 8.83 + 6.18 |
| Native thread user/system CPU seconds | 12.53 / 2.48 |

FD 20 referenced the same installed extension in 38 snapshots, retaining inode
`31064179` and the recorded file size. Successive observations of that file included
21 decreasing offsets. The descriptor also cycled through NumPy OpenBLAS and PyArrow
libraries. This establishes repeated reading/reopening rather than a single once-only
scan. Sparse polling does not count complete passes or attribute all process reads
to this extension. Thread activity does not establish completed study-point count.

**Measured — sampled user-space CPU attribution.** A separate 15-second perf recording
at 49 Hz attributed **97.39% of weighted sampled `cycles:u` self overhead** to
`sha2::sha256::x86_sha::compress` in `_native.abi3.so`, on `pse-math` threads.
The report recorded zero lost samples and approximately 29,788,438,470 events.
This is a weighted user-cycle percentage, not a fraction of total wall time, all CPU,
or the complete test lifetime. Frame-pointer chains often terminated at compression;
the inline resolver warned that it could not read its first record. The resolved
function attribution remains usable; the perf report alone does not establish every
calling path.

The actual profiler commands were:

```sh
scripts/pse-env -- perf record -B -e cycles:u -F 49 -g -p 591465 -o /tmp/plan28-preparation-perf-20261008.data -- sleep 15
scripts/pse-env -- perf report --stdio --no-children --sort comm,dso,symbol --percent-limit 1 -i /tmp/plan28-preparation-perf-20261008.data
/home/paul/.local/bin/py-spy dump --pid 591465 --native --full-filenames
```

The process sample used `scripts/pse-env -- python -` to read `/proc/591465/io`,
`task/*/stat`, `fd/*` and `fdinfo/*`; its exact payload is retained at
`/tmp/plan28-preparation-proc-sample.py`. It must not be rerun into the existing output.
Raw observations remain at `/tmp/plan28-preparation-proc-sample-20261008.jsonl`;
the perf data/report and Python dump remain under the same dated temporary prefix.
The selected measurements above are retained here independently of scratch-file lifetime.

**Interface-checked — calling context.** The Python dump taken at 22:13:29 UTC places
the main thread in `study`, called by `test_flash_sweep_prepares_structure_once` at
`test_studies.py:932`. The ordinary selection uses its `ephemeral-preparation` parameter.
The test submits 1,000 temperature points before inspecting their results. Py-spy's
Python-thread dump does not enumerate Rust-only math workers or report point progress.

The earlier read-only native snapshot
`/tmp/plan28-python-native-stack-20261008T2204.txt` identifies SHA compression through
`FileObservation::capture` on one math thread and `LocalReplay::validated_namespace`
cohort waiting on another, under modeling preparation. Together with current source,
these observations explain the repeated hashing path. Exact current-source pass counts
must not be attributed automatically to the older installed artifact.

## Platform-source examination

**Interface-checked — exact upstream release.** The official [glibc 2.39 archive](https://ftp.gnu.org/gnu/glibc/glibc-2.39.tar.xz)
was downloaded and independently checksum-checked:
`f77bd47cf8170c57365ae7bf86696c118adb3b120d3259c64c502d3dc1e2d926`.
The extracted source remains at `/tmp/glibc239-loader-review/glibc-2.39/`.
Current Context7 excerpts were not used to transfer guarantees to this release.

- `dl_iterate_phdr` holds a recursive `dl_load_write_lock` around enumeration and
  callbacks. The lock protects link-map list publication/removal; it does not freeze
  the complete loader transaction. [Iteration source](https://github.com/bminor/glibc/blob/glibc-2.39/elf/dl-iteratephdr.c),
  [lock definitions](https://github.com/bminor/glibc/blob/glibc-2.39/sysdeps/generic/ldsodefs.h).
- Loader counters describe publication/removal observations. Reopening an already
  loaded object can promote global scope or change other state without another
  publication; unchanged counters do not attest unchanged symbol lookup, provider
  state, file bytes or configuration. The derived removal count also requires care
  around secondary namespaces. [Open source](https://github.com/bminor/glibc/blob/glibc-2.39/elf/dl-open.c),
  [counter declaration](https://github.com/bminor/glibc/blob/glibc-2.39/elf/link.h).
- Unload destructors precede the removal write-lock interval. Owning-module TLS
  prewarming supports a narrower unchanged-ELF argument, not universal callback
  deadlock freedom or preparation of every dependent DSO's TLS.
  [Close source](https://github.com/bminor/glibc/blob/glibc-2.39/elf/dl-close.c),
  [TLS source](https://github.com/bminor/glibc/blob/glibc-2.39/elf/dl-tls.c).

These are limits of platform primitives. Whether a consumed reconstruction depends
on the uncovered state requires its own consumer-level argument. No wrong scientific
result, loader deadlock or arbitrary-host safety was demonstrated here.

## Limits and handoff

The measurements establish repeated hot-cache reads and dominant sampled SHA CPU;
they establish neither all preparation costs nor a remedy's speedup. There is no
new successful full test assessment, capacity claim, cold/warm scaling campaign,
or scientific/reference qualification. Related workspace, publication and lifecycle
findings are source-grounded in the principal review, not measured shares of this sample.
Current finding disposition belongs to the [Plan 28 coordinator](../../plans/28-surrealdb-unified-substrate.md).
