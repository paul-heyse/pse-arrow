# procfs / procfs-core (Rust)

Realizations considered: `procfs` 0.18.0 and `procfs-core` 0.18.0 (released 2025-08-30; latest
on crates.io at research date; not in the lockfile); `rustix-linux-procfs` 0.1.1 (registry cache
only) noted for hardened `/proc` opening. Research date 2026-10-27. Sources: crate sources 0.18.0
downloaded into scratch: `procfs-core/src/process/stat.rs` (`Stat::{pgrp, starttime}`),
`procfs-core/src/pressure.rs`, `procfs/src/{cgroups.rs, process/mod.rs, process/namespaces.rs,
sys/kernel/random.rs}`; `rustix-linux-procfs-0.1.1/src/lib.rs`.

## Types

### procfs/typed-proc-readers
- **Need it serves:** read `/proc` process and system facts through typed parsers instead of
  ad hoc string splitting.
- **Recognizable code shapes:** `rsplit_once(')')` on `/proc/<pid>/stat`; `strip_prefix("VmHWM:")`
  on `/proc/self/status`; reading `/proc/sys/kernel/random/boot_id`; parsing `0::` lines of
  `/proc/<pid>/cgroup`; parsing `/proc/pressure/*`.
- **Mechanisms and APIs:** `procfs::process::Process::{new(pid), myself(), new_with_root(path)}`
  → `.stat()` (`Stat { state, pgrp, starttime: u64, .. }`), `.status()` (`vmhwm` etc.),
  `.cgroups()` (`ProcessCGroups`), `.namespaces()`, `.exe()`; `procfs::process::all_processes()`
  / `all_processes_with_root(root)`; `procfs::sys::kernel::random::boot_id()`;
  `procfs_core::pressure::{CpuPressure, MemoryPressure, IoPressure}` and the generic
  `get_pressure<R: BufRead>` / `FromBufRead` parsers, which accept any reader (so they also parse
  cgroup `*.pressure` files); `procfs-core` is I/O-free and can parse supplied bytes.
- **Obligations discharged:** field positions and the `comm` parenthesis rule; unit handling for
  `status`; a root-override for fixtures (`new_with_root`).
- **Residual caller obligations:** identity composition (boot ID + start ticks + namespace),
  read-before/read-after consistency, distinguishing absent/unavailable/changed, the cgroup
  filesystem under `/sys/fs/cgroup` (procfs covers `/proc` only).
- **Semantic conditions:** `Stat::starttime` is the raw tick count (good for identity);
  `Stat::starttime()` the method converts to a local `chrono` time (lossy and not for identity).
  `CpuPressure.full` is a record even at system level, where the kernel defines it as undefined
  (reported as zero); callers must ignore it. Errors are `ProcError::{NotFound, PermissionDenied,
  Incomplete, Io, Other, InternalError}`, which can map onto absent/unavailable.
- **Version / build conditions:** 0.18.0; default features pull `chrono` and `flate2`
  (configurable).
- **Limits and negative results:** no pidfd-based process handle (search scope: `procfs-0.18.0/src`
  for `pidfd`); no `/sys/fs/cgroup` interface-file readers; no PSI triggers.
- **Evidence:** Interface-checked: sources listed.

### procfs/hardened-proc-open (rustix-linux-procfs)
- **Need it serves:** open `/proc/self/*` files while verifying that `/proc` is a genuine procfs
  mount (defence against an over-mounted `/proc`).
- **Mechanisms and APIs:** `proc_self_fd()`, `proc_self_status()`, `proc_self_maps()`,
  `proc_self_fdinfo_fd(fd)`, `proc_self_pagemap()`.
- **Limits:** opening only; no parsing; `/proc/self` scope only.
- **Evidence:** Interface-checked: `rustix-linux-procfs-0.1.1/src/lib.rs`.

## Current applicability

### [CBO-019](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-019) — Linux process, cgroup and pressure facts parsed by hand in Rust and Python
- **Types:** `procfs/typed-proc-readers`
- **Realization:** upgrade route 0.18.0 (new dependency; default features would add `chrono`
  and `flate2` unless disabled).
- **Conclusion:** fits with conditions for the Rust `/proc` parsers (stat start ticks, `VmHWM`,
  boot ID, `/proc/<pid>/cgroup`); ruled out for the cgroup limit walk and for the Python twin.
- **What it would discharge / what remains local:** the parsing of about eight `/proc/*/stat`
  readers and four `/proc/self/status` readers (counted at baseline; see README). The identity
  rule, read-before/after checks, the cgroup interface-file walk under `/sys/fs/cgroup`, and the
  Python readers (CBO-019's "format knowledge held twice") remain.
- **Prerequisites, configuration or extension:** `default-features = false`; mapping
  `ProcError` onto the absent/unavailable/changed vocabulary.
- **Semantic gaps against the CBO's preservation constraints:** "fail closed" holds if every
  `ProcError` maps to refusal; "start-tick identity" holds with `Stat::starttime` (the field, not
  the method). The two-language duplication is untouched — the stronger consolidation lever is a
  shared identity rule (see `linux-pidfd.md`), not a Rust parser crate.
- **Unsettled premises and what would settle them:** whether removing roughly a dozen small parsers
  justifies a dependency (comparative review; the CBO itself lists low priority).
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** The comparative review recommends this route as a minor change: one Rust process-fact reader on procfs-core 0.18 (`default-features = false`) in `pse-diagnostics` replaces about 12 hand parsers. The start-tick identity rule is retained rather than replaced by a pidfs inode (see `linux-pidfd`), and Python already has a single leaf (`process_observation.py`, used by 8 modules), so the two-language duplication is narrower than the CBO states. `lifecycle-owner.json` carries the start-tick identity across languages, which the reader must keep byte-compatible. See the [comparative review](../../reviews/host-processes-and-durable-files.md#cbo-019).
