# Linux pidfd and pidfs process identity (native capability)

Realizations considered: host kernel 7.0.0-38-generic (UAPI headers
`/usr/src/linux-headers-7.0.0-38-generic/include/uapi/linux/pidfd.h`, `sched.h`); glibc 2.39;
Rust bindings in `rustix` 1.1.5 (pidfd open/signal/wait) and `libc` 0.2.190 (`PIDFD_GET_INFO`,
`pidfd_info`, `clone_args`, `CLONE_INTO_CGROUP`); Python 3.14 stdlib (`os.pidfd_open`,
`signal.pidfd_send_signal`, `os.waitid(P_PIDFD, ..)`); systemd NEWS (main branch, fetched
2026-10-27) for the pidfd inode-ID contract. Research date 2026-10-27. Sources: Linux man-pages
6.19 `pidfd_open(2)`; the header files named; systemd NEWS entries for v255–v259; psutil 7.2.2
`_psposix.py` (`wait_pid_pidfd_open`). Kernel `fs/pidfs.c` could not be fetched (GitHub HTTP 429),
so the kernel version that introduced unique pidfs inode numbers is from general kernel history,
not re-verified here.

## Types

### linux-pidfd/handle-based-process-identity
- **Need it serves:** record and later re-check "is this still the same process" without
  composing boot ID, numeric PID and `/proc/<pid>/stat` field 22 (start time in clock ticks), and
  share one identity rule between languages.
- **Recognizable code shapes:** `rsplit_once(')')` + `split_whitespace()` + index 19 on
  `/proc/<pid>/stat`; tuples of `(boot_id, pid, starttime[, pid namespace])`; read-before /
  read-after identity checks around `pidfd_open` or `kill`.
- **Mechanisms and APIs:** a pidfd is a stable kernel handle; on pidfs kernels `fstat(pidfd).st_ino`
  is a 64-bit process ID that systemd documents as "unique for all processes of a specific Linux
  boot" and "does not recycle" (systemd v258/v259 NEWS: `$MAINPIDFDID`, `$LISTEN_PIDFDID`,
  `sd_pidfd_get_inode_id()`). `ioctl(pidfd, PIDFD_GET_INFO, struct pidfd_info)` returns pid, creds
  and `cgroupid` (always when available) and, when requested, exit status (`PIDFD_INFO_EXIT`) and
  coredump info. `/proc/self/fdinfo/<pidfd>` exposes `Pid:` / `NSpid:` (−1 after exit). Python
  can `os.fstat(os.pidfd_open(pid)).st_ino` with stdlib only.
- **Obligations discharged:** identity no longer depends on clock-tick resolution, boot-ID pairing
  or parsing; a held pidfd cannot be redirected to a recycled PID; `cgroupid` binds the process to
  a cgroup inode without reading `/proc/<pid>/cgroup` twice.
- **Residual caller obligations:** opening a pidfd by numeric PID is itself racy for a process
  that is not the caller's unreaped child — re-verify after opening (for example compare the
  recorded inode ID, or the old start time, once); pair the inode ID with the boot ID when it is
  persisted across reboots; decide what an unavailable pidfs means (fail closed).
- **Semantic conditions:** the inode ID is per boot, not per namespace; it identifies the thread
  group (or the thread with `PIDFD_THREAD`); it is meaningful only on pidfs kernels — on older
  kernels pidfds share an anonymous inode and `st_ino` is not an identity. `pidfd` readability
  (`POLLIN`) without `PIDFD_THREAD` means every thread of the group has exited, which is stronger
  than observing the leader's `Z` state.
  **Namespace hazard (added 2026-10-27):** a persisted pidfs identity that another process
  re-checks is unsafe across PID namespaces. The checker's `pidfd_open(pid)` resolves the number
  in its own namespace and can open an unrelated process, whose inode then differs, so the check
  wrongly concludes that the recorded process is gone. A PID-namespace check stays required
  beside the inode ID. Where boot- and namespace-bound start ticks are already effectively
  non-recyclable (a large `pid_max` with a coarse `CLK_TCK` still needs a full PID-space wrap
  within one tick), the inode ID adds no guarantee as a *persisted key*; its value is as a held
  handle for the process being acted on.
- **Version / build conditions:** `pidfd_open` ≥ 5.3; pidfs-backed unique inode IDs on 64-bit
  kernels from the pidfs era (6.9 per kernel history, not re-verified); `PIDFD_GET_INFO` present
  in the host 7.0 UAPI (introduction version not verified here); systemd consumers ≥ 258.
- **Limits and negative results:** no pidfd enumerates descendants or a process group; pidfs IDs
  do not survive reboot; no Rust safe wrapper for `PIDFD_GET_INFO` in rustix 1.1.5 (raw ioctl with
  `libc` constants needed); psutil 7.2.2 uses pidfds only internally for waiting.
- **Evidence:** Interface-checked: host UAPI headers, man-pages 6.19, systemd NEWS text, psutil
  7.2.2 source. The uniqueness claim is a documented systemd contract about the kernel facility,
  not a kernel-source reading.

### linux-pidfd/spawn-into-cgroup
- **Need it serves:** start a child already inside a chosen cgroup and receive its pidfd in the
  same step, so no descendant can be created outside the containment boundary.
- **Recognizable code shapes:** spawn, then write the PID to `cgroup.procs`; `pre_exec` hooks that
  move the child; `systemd-run --scope` wrappers used only for containment.
- **Mechanisms and APIs:** `clone3(struct clone_args { flags: CLONE_INTO_CGROUP | CLONE_PIDFD,
  cgroup: <dirfd of target cgroup>, pidfd: &out })`; glibc 2.39 `pidfd_spawn`/`pidfd_spawnp` with
  `posix_spawnattr_setcgroup_np` (systemd ≥ 256 uses this route); portable fallback: a
  `pre_exec` closure in the child writing `0`/its PID into the target `cgroup.procs` before
  `exec` (async-signal-safe `open`/`write`).
- **Obligations discharged:** containment from the first instruction of the child; with
  `CLONE_PIDFD`, a race-free handle.
- **Residual caller obligations:** the target cgroup must exist and be writable (delegation);
  Rust `std::process::Command` exposes neither `clone3` nor a pidfd on stable (its
  `linux_pidfd` API is unstable), so the clean route needs FFI; the `pre_exec` route works with std.
- **Semantic conditions:** cgroup migration permissions (common-ancestor write access) apply to
  the spawner.
- **Version / build conditions:** `CLONE_INTO_CGROUP` Linux ≥ 5.7; glibc ≥ 2.39 for
  `pidfd_spawn`; `libc` 0.2.190 defines `CLONE_INTO_CGROUP` and `clone_args` but no
  `pidfd_spawn` binding (search scope: `libc-0.2.190/src/**`).
- **Evidence:** Interface-checked: host `sched.h`, `libc` 0.2.190 source, systemd v256 NEWS
  ("pidfd_spawn() ... target cgroup and get a pidfd back in a single operation"); nightly
  `std/src/os/linux/process.rs` (`#![unstable(feature = "linux_pidfd")]`).

## Current applicability

### [CBO-019](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-019) — Linux process, cgroup and pressure facts parsed by hand in Rust and Python
- **Types:** `linux-pidfd/handle-based-process-identity`
- **Realization:** host kernel 7.0 + rustix 1.1.5/libc 0.2.190 (Rust) and Python 3.14 stdlib; no
  new dependency.
- **Conclusion:** fits with conditions — a pidfs inode ID (paired with the boot ID) is a
  candidate single identity rule for both languages, replacing start-tick parsing where the
  identity is created while the process is live.
- **What it would discharge / what remains local:** the stat-field-19 parsers and their tick
  semantics; the before/after re-read around `pidfd_open` reduces to one post-open check. The
  boot-ID binding, namespace and executable facts, and fail-closed handling of unavailable
  observations remain local.
- **Prerequisites, configuration or extension:** a raw `ioctl`/`fstat` helper in Rust; the
  existing records that persist start ticks would change format.
- **Semantic gaps against the CBO's preservation constraints:** identity of a process observed
  only by PID (another user's ledger entry, a process that already exited) still requires a
  PID-based open; start ticks are observable from `/proc` without a handle, inode IDs are not
  observable for an exited process. Persisted ledgers that compare against identities written by
  an older kernel would need a migration rule.
- **Unsettled premises and what would settle them:** the pidfs inode-ID uniqueness on this host
  (a probe: two `pidfd_open` on the same PID give equal `st_ino`; different processes differ);
  whether the project accepts a kernel floor at the pidfs era as a host-local premise.
- **Evidence:** Interface-checked; application Proposed.
- **Missed manifestations (correction to the CBO footprint):** the same boot-ID + start-tick
  identity is also parsed in the canonical file store's owner identity
  (`pse-operations/src/canonical_files.rs`, `canonical.rs`, `canonical_files/offline.rs`), in
  `xtask/src/producer_preparation.rs`, and in `pse-runtime/src/tooling/capture/launch.rs`; there
  are at least eight Rust readers of `/proc/<pid>/stat` or `/proc/self/stat`, not "about six
  small parsers" overall.

**Reviewer correction (2026-10-27).** The pidfs inode is not adopted as the persisted identity.
On this host `pid_max` is 4194304 and `CLK_TCK` is 100, so the (boot, PID namespace, PID, start
tick) identity is already effectively non-recyclable, and a persisted inode ID would add the
namespace hazard above. The comparative review keeps that identity, uses pidfds where a process is
acted on, and consolidates the Rust parsers on a procfs-core reader instead. Python already has a
single leaf (`process_observation.py`, used by 8 modules), and `lifecycle-owner.json` carries the
start-tick identity across languages (Python writes it; Rust `require_initializer` reads it). See
the [comparative review](../../reviews/host-processes-and-durable-files.md#cbo-019).

### [CBO-004](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-004) — Child-process and process-group supervision implemented per consumer
- **Types:** `linux-pidfd/spawn-into-cgroup`, `linux-pidfd/handle-based-process-identity`
- **Realization:** host kernel 7.0, glibc 2.39; FFI through `libc` 0.2.190 or `pre_exec` with std.
- **Conclusion:** fits with conditions — spawning each supervised child into its own delegated
  child cgroup turns "drain" into the kernel's `populated` flag (see `linux-cgroup-v2.md`); the
  pidfd part is already adopted.
- **What it would discharge / what remains local:** the `/proc` inventory and re-inventory loop;
  the leader-held-unreaped trick to reserve the PGID. Output bounding, deadline/cancellation
  observation and the "unproved drain never releases" policy remain local.
- **Prerequisites, configuration or extension:** the supervising process must run in a cgroup
  subtree it may create children in (a systemd unit with `Delegate=yes`, or a delegated
  `user@.service` subtree) — this is a deployment prerequisite, not a library one.
- **Semantic gaps against the CBO's preservation constraints:** none in principle — containment
  is stricter than process-group membership (a `setsid`/`setpgid` member stays contained);
  outside a delegated cgroup (plain test runs, editors) the route is unavailable and a fallback
  would reintroduce a second mechanism.
- **Unsettled premises and what would settle them:** whether every runtime process that compiles
  or launches children runs inside a delegated unit (deployment/placement owner); probe request P1.
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** Probe P1 showed that a pre-exec write of `0` to a leaf's
`cgroup.procs` works in a `Delegate=yes` user scope with the supervisor in the scope root, and that
`cgroup.kill` then drains `setsid` and double-forked descendants (*Measured*; details and
conditions under [linux-cgroup-v2](../linux-cgroup-v2/README.md)). `CLONE_INTO_CGROUP` was not
exercised. `native_child` needs no group drain at all, since it runs the repository's own binary
with the interpreted evaluator, and the two group-drain sites differ deliberately in failure
policy (park versus refuse after 3 s). See the
[comparative review](../../reviews/host-processes-and-durable-files.md#cbo-004).
