# Linux cgroup v2 containment, events and PSI triggers (native capability)

Realizations considered: host kernel 7.0.0-38-generic with the unified hierarchy (observed files
under the user manager's cgroup: `cgroup.kill`, `cgroup.events`, `memory.events(.local)`,
`memory.peak`, `cgroup.pressure`, `{cpu,memory,io}.pressure`, `cgroup.stat.local`); host systemd 255
as the delegating manager. Research date 2026-10-27. Sources: kernel documentation
docs.kernel.org `admin-guide/cgroup-v2` and `accounting/psi` (fetched 2026-10-27); systemd v255
`docs/CGROUP_DELEGATION.md` (Context7 `/systemd/systemd/v255`); a read-only listing of the host
cgroup directory.

## Types

### linux-cgroup-v2/containment-kill-and-drain
- **Need it serves:** kill every process a supervised job created, including re-parented and
  re-grouped descendants, and know race-free when none remain.
- **Recognizable code shapes:** `/proc` scans filtering by process group; re-inventory loops
  "a member may have forked before its kill"; `killpg` followed by per-member waits; polling
  `cgroup.events` or `systemctl show` until a unit is inactive.
- **Mechanisms and APIs:** create a child cgroup (`mkdir`) inside a delegated subtree; place the
  child there at spawn (`CLONE_INTO_CGROUP`, or a pre-exec write to `cgroup.procs`); kill with
  `echo 1 > cgroup.kill` ("deals with concurrent forks appropriately and is protected against
  migrations"); wait for `populated 0` in `cgroup.events` — "poll and [i]notify events are
  triggered when the value changes" (`poll` with `POLLPRI`, or inotify `IN_MODIFY`); `rmdir` the
  empty cgroup. `cgroup.freeze`/`frozen` offers stop-the-world inspection. `memory.peak`
  (per-fd resettable by writing) and `memory.events` (`oom`, `oom_kill`, `max`, `high`;
  `.local` variant non-hierarchical) give per-job memory outcome without process sampling.
- **Obligations discharged:** descendant discovery; fork-race handling during kill; an
  authoritative, event-driven "drained" signal; per-job peak memory and OOM attribution.
- **Residual caller obligations:** arranging delegation; naming and removing child cgroups;
  converting the `populated` event into the caller's wait/deadline/cancellation model; handling
  an unavailable cgroup (fail closed).
- **Semantic conditions:** a delegatee cannot move processes into or out of its delegated
  sub-hierarchy, and moving needs write access on the common ancestor's `cgroup.procs`; a
  same-user process can still migrate itself elsewhere within a subtree the user owns, so
  containment holds against `setsid`/`setpgid`, not against deliberate self-migration.
  The "no internal process" rule restricts enabling *domain controllers* for children of a
  cgroup that has processes; pure containment (kill/populated/peak) does not need controllers
  enabled in `subtree_control`. `cgroup.kill` fails with `EOPNOTSUPP` in threaded cgroups.
  systemd's guidance: do not derive unit names from cgroup paths; ask the manager
  (`ControlGroup`, `GetUnitByControlGroup`).
- **Version / build conditions:** `cgroup.kill` Linux ≥ 5.14; `memory.peak` ≥ 5.19 (per-fd reset
  later; present on the 7.0 host); cgroup v2 unified hierarchy.
- **Limits and negative results:** without delegation (a process inside a scope or service whose
  cgroup systemd owns without `Delegate=yes`) the caller should not create sub-cgroups; systemd
  may also manage them. Not usable in environments where the process is not in a writable cgroup.
- **Evidence:** Interface-checked: kernel cgroup-v2 documentation text quoted above; systemd v255
  delegation document; host file listing. No probe.

### linux-cgroup-v2/limit-readback
- **Need it serves:** verify that a placement's effective memory and CPU limits admit a workload
  profile, walking ancestors.
- **Recognizable code shapes:** reading `/proc/self/cgroup` (`0::` line), then `memory.max`,
  `cpu.max`, `cgroup.controllers` in each ancestor directory up to the root.
- **Mechanisms and APIs:** files `memory.max` (`max` or bytes), `cpu.max` (`$MAX $PERIOD`, `max`
  allowed), `cgroup.controllers` (which controllers are available, hence whether an absent
  interface file means "inherited, not enabled here"). No kernel file reports the effective
  (minimum over ancestors) limit; the walk is the native mechanism.
- **Obligations discharged:** none beyond the files themselves.
- **Residual caller obligations:** the ancestor walk, distinguishing absent from inherited from
  unreadable, exact equality checks.
- **Limits and negative results:** systemd's `EffectiveMemoryMax`/`EffectiveTasksMax` unit
  properties (systemd ≥ 256) report "the most stringent limit systemd is aware of", i.e. only
  manager-configured limits, not raw kernel values written by others; there is no `EffectiveCPUQuota`.
- **Evidence:** Interface-checked: cgroup-v2 doc; systemd NEWS v256 text.

### linux-cgroup-v2/psi-triggers
- **Need it serves:** be notified when a population's stall time crosses a threshold, instead of
  periodically sampling and differencing pressure counters.
- **Recognizable code shapes:** periodic reads of `/proc/pressure/*` or `<cgroup>/*.pressure`,
  `total=` deltas, sleeps between samples, ad hoc thresholds.
- **Mechanisms and APIs:** open a pressure file `O_RDWR`, write `"<some|full> <stall µs> <window µs>"`,
  then `poll`/`epoll` for `POLLPRI`; one trigger per fd (`EBUSY` otherwise); windows 500 ms–10 s;
  notifications rate-limited to one per window. Per-cgroup files (`cpu.pressure`,
  `memory.pressure`, `io.pressure`) behave the same as the system files; `cgroup.pressure` = `0`
  disables PSI accounting for that cgroup (non-hierarchical).
- **Obligations discharged:** threshold detection with kernel-side windowing; event delivery
  without polling loops.
- **Residual caller obligations:** threshold policy; mapping an event to an action; sampling is
  still needed for reports that quote averages or totals.
- **Semantic conditions:** unprivileged triggers require the window to be a multiple of 2 s.
  "CPU full is undefined at the system level, but has been reported since 5.13" (reported as
  zero for compatibility). Pressure describes a population, not a specific operation's cause.
- **Version / build conditions:** PSI Linux ≥ 4.20; triggers ≥ 5.2; unprivileged triggers in
  later kernels (present by 7.0 per current documentation).
- **Limits and negative results:** no library is needed or materially helpful; Python and Rust
  can both write and poll the fd with stdlib/rustix.
- **Evidence:** Interface-checked: docs.kernel.org `accounting/psi`.

## Current applicability

### [CBO-004](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-004) — Child-process and process-group supervision implemented per consumer
- **Types:** `linux-cgroup-v2/containment-kill-and-drain`
- **Realization:** host kernel 7.0 + systemd 255 delegation; std/rustix only.
- **Conclusion:** fits with conditions — the strongest available drain proof, stricter than the
  current process-group inventory, but only where the supervising process runs in a delegated
  cgroup.
- **What it would discharge / what remains local:** `/proc` group scanning, start-tick identity
  checks of members, re-inventory after fork, the unreaped-leader PGID reservation; replaced by
  `cgroup.kill` + `populated 0`. Bounded output, cancellation/deadline, deterministic compiler
  flags and the park-on-unproved-drain policy remain.
- **Prerequisites, configuration or extension:** the runtime/worker unit needs `Delegate=yes`
  (or the work runs under a delegated subtree); a spawn path that places the child before exec.
  The repository currently launches whole commands with `systemd-run --user --scope` plus
  `MemoryMax`/`TasksMax` properties and no `Delegate=` (observed in the supervisor script's
  `systemd-run` argument lists).
- **Semantic gaps against the CBO's preservation constraints:** an unprivileged run outside any
  delegated unit (tests, ad hoc shells) cannot use it; keeping the `/proc` route as a fallback
  would keep two drain proofs.
- **Unsettled premises and what would settle them:** probe P1 (below in README); placement owner
  decision on delegating the worker scope.
- **Evidence:** Interface-checked; application Proposed.

### [CBO-019](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-019) — Linux process, cgroup and pressure facts parsed by hand in Rust and Python
- **Types:** `linux-cgroup-v2/limit-readback`, `linux-cgroup-v2/psi-triggers`
- **Realization:** native files; no library.
- **Conclusion:** ruled out here as a library route for the limit walk (the walk is the native
  mechanism and no crate removes it, see `cgroups-rs.md`, `procfs.md`); PSI triggers fit only if a
  threshold-notification consumer exists — the current pressure readers are report samplers.
- **What it would discharge / what remains local:** nothing for limit readback; for pressure, a
  trigger would replace polling only in a watcher, not in a report.
- **Semantic gaps against the CBO's preservation constraints:** none introduced; the
  "inherited vs absent `cpu.max`" distinction is exactly what `cgroup.controllers` encodes and must
  remain caller logic.
- **Unsettled premises and what would settle them:** whether any admission or drain loop would
  act on pressure thresholds (product owner).
- **Evidence:** Interface-checked.

### [CBO-048](../../../library_leverage/codebase/developer-tooling.md#cbo-048) — Host workload admission and service supervision rebuilt above systemd (non-SurrealDB parts)
- **Types:** `linux-cgroup-v2/containment-kill-and-drain`
- **Realization:** native; Python stdlib (`select.poll` with `POLLPRI` on `cgroup.events`, or
  `os.inotify` is not stdlib — `select.poll` is).
- **Conclusion:** fits with conditions — drain observation of a scope's cgroup can wait on the
  `populated` change event instead of re-reading `cgroup.events` in a loop, with stdlib only.
- **What it would discharge / what remains local:** polling cadence for drain; the inode-identity
  check around the read and ledger semantics remain.
- **Semantic gaps against the CBO's preservation constraints:** an absent cgroup directory (scope
  already collected) must still be distinguished from unavailable observation; `poll` gives no
  event for directory removal, so a deadline remains necessary.
- **Unsettled premises and what would settle them:** whether `POLLPRI` on `cgroup.events` fires
  for the user's scopes on this host (probe P1 covers it).
- **Evidence:** Interface-checked; application Proposed.
