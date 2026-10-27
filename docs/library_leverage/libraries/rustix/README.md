# rustix (Rust)

Realizations considered: installed `rustix` 1.1.5 (workspace pin `=1.1.5`, features `fs`, `time`,
`thread`, `process`, `event`); `rustix-linux-procfs` 0.1.1 exists in the local registry cache but
is not in the lockfile. Research date 2026-10-27. Sources: installed source `rustix-1.1.5/src/fs/at.rs`
(`linkat`, `renameat_with`), `src/fs/fd.rs` (`flock`), `src/fs/fcntl.rs` (`fcntl_lock`),
`src/process/{pidfd.rs,pidfd_getfd.rs,kill.rs,wait.rs,id.rs,prctl.rs}`; tempfile 3.27.0 and
cap-tempfile 4.0.3 as consumers showing the composition; Linux man-pages 6.19 `pidfd_open(2)`
(man7.org, fetched 2026-10-27).

## Types

### rustix/noreplace-and-anonymous-publication
- **Need it serves:** publish a fully written file under a final name atomically, without ever
  replacing an existing name, and without a named staging file that can be orphaned.
- **Recognizable code shapes:** `create_new(staging)` + `hard_link(staging, final)` +
  `remove_file(staging)`; recovery code that scans for leftover `*.pending` names; comments
  rejecting `rename` because it overwrites.
- **Mechanisms and APIs:** `rustix::fs::renameat_with(old_dir, old, new_dir, new,
  RenameFlags::NOREPLACE)` (Linux `renameat2`) is a single atomic no-clobber rename returning
  `EEXIST`; `openat(dir, ".", OFlags::TMPFILE | RDWR | CLOEXEC, mode)` creates an unnamed inode in
  the directory; `linkat(CWD, "/proc/self/fd/<n>", dir, name, AtFlags::SYMLINK_FOLLOW)` gives it a
  name and fails with `EEXIST` if the name exists (the `/proc/self/fd` form works unprivileged;
  `AtFlags::EMPTY_PATH` historically required `CAP_DAC_READ_SEARCH`). Directory fsync is
  `fsync` on an `open(dir, O_RDONLY|O_DIRECTORY)` fd. `tempfile` uses the first route,
  `cap-tempfile`/`atomic-write-file` the second.
- **Obligations discharged:** atomic no-clobber naming in one syscall (`renameat2`), or naming an
  anonymous inode so that a crash before naming leaves nothing to clean up (`O_TMPFILE`+`linkat`).
- **Residual caller obligations:** data fsync before naming; directory fsync after; content and
  link-count validation; fallback when the filesystem lacks `O_TMPFILE` (`EOPNOTSUPP`/`EISDIR`)
  or `RENAME_NOREPLACE` (`EINVAL`).
- **Semantic conditions:** `O_TMPFILE` + `linkat` publishes an inode that only ever has one name
  (link count 1 after link); `renameat2` moves the staging name (link count stays 1); both differ
  from `link`+`unlink`, where the inode has two names until the unlink. An `O_TMPFILE` staging
  cannot be resumed after a crash (it vanishes), which is a feature for cleanup and a loss for
  resumable writers. Both require same-filesystem directories.
- **Version / build conditions:** `renameat2` Linux ≥ 3.15; `O_TMPFILE` Linux ≥ 3.11 and
  filesystem support (ext4, xfs, btrfs, tmpfs).
- **Limits and negative results:** rustix does not fsync or verify; it is a syscall layer.
- **Evidence:** Interface-checked: rustix 1.1.5 `fs/at.rs`; composition read in tempfile 3.27.0
  `file/imp/unix.rs` and cap-tempfile 4.0.3 `src/tempfile.rs`.

### rustix/pidfd-process-handles
- **Need it serves:** signal, wait for and identify a process through a kernel handle instead of a
  recyclable numeric PID.
- **Recognizable code shapes:** `/proc/<pid>/stat` start-time checks before `kill(pid)`;
  `waitpid` loops; per-site PID-reuse comments.
- **Mechanisms and APIs:** `process::pidfd_open(pid, PidfdFlags::{empty,NONBLOCK,THREAD})`,
  `pidfd_send_signal(&fd, Signal)`, `waitid(WaitId::PidFd(fd), WaitIdOptions::{EXITED,NOWAIT,..})`,
  `event::poll` on the pidfd (`POLLIN` when the whole thread group has exited, unless
  `PIDFD_THREAD`), `pidfd_getfd`, `kill_process_group`, `getpgid`, `setsid`,
  `prctl::set_child_subreaper(Some(pid))` (makes orphaned descendants reparent to the caller so
  `waitid(P_PGID|P_ALL)` can reap and therefore observe them).
- **Obligations discharged:** race-free signalling and waiting once a pidfd is held; thread-group
  exit readiness through `poll`.
- **Residual caller obligations:** obtaining a pidfd without a race for processes that are not
  your unreaped children (open, then re-verify identity); discovering descendants (no
  "list members of a group/session" API — that remains `/proc` scanning or a cgroup);
  any persistent identity record.
- **Semantic conditions:** a pidfd opened for your own unreaped child is race-free; for other
  PIDs it is race-free only after a post-open re-check. Subreaper status is process-wide and
  changes reparenting for every descendant of every thread, which interacts with other reapers in
  the same process (for example an async runtime's child reaper).
- **Version / build conditions:** `pidfd_open` Linux ≥ 5.3, `pidfd_send_signal` ≥ 5.1,
  `waitid(P_PIDFD)` ≥ 5.4, `PIDFD_NONBLOCK` ≥ 5.10, `PIDFD_THREAD` ≥ 6.9 (man-pages 6.19).
- **Limits and negative results:** rustix 1.1.5 has no wrapper for the `PIDFD_GET_INFO` ioctl,
  `clone3`/`CLONE_INTO_CGROUP` or `pidfd_spawn`; those need `libc` 0.2.190 constants
  (`PIDFD_GET_INFO`, `clone_args`, `CLONE_INTO_CGROUP` are present there) or raw `ioctl`.
  Search scope: `src/process/**` and `src/backend/*/process/**` of rustix 1.1.5.
- **Evidence:** Interface-checked: rustix 1.1.5 source files above; man-pages 6.19 `pidfd_open(2)`.

### rustix/flock-and-ofd-locks
- **Need it serves:** advisory whole-file locks on a descriptor.
- **Mechanisms and APIs:** `fs::flock(fd, FlockOperation::{LockShared, LockExclusive,
  NonBlockingLock*, Unlock})` (BSD `flock`), `fs::fcntl_lock(fd, ..)` (open-file-description
  locks).
- **Semantic conditions:** `flock` and OFD/POSIX record locks are independent lock families on
  Linux; a lock taken in one does not conflict with the other. No timed variant exists for either.
- **Limits:** no deadline-bounded acquire; same as std `File::lock`.
- **Evidence:** Interface-checked: rustix 1.1.5 `fs/fd.rs`, `fs/fcntl.rs`.

## Current applicability

### [CBO-003](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-003) — Durable file publication and the sealed-file byte layer on `std::fs`
- **Types:** `rustix/noreplace-and-anonymous-publication`, `rustix/flock-and-ofd-locks`
- **Realization:** installed pin 1.1.5 with the already-enabled `fs` feature; no dependency move.
- **Conclusion:** fits with conditions — `renameat_with(NOREPLACE)` can replace the
  `hard_link`+`remove_file` pair at the simple publishers; `O_TMPFILE` staging fits only where a
  crash-orphaned staging file has no resume value.
- **What it would discharge / what remains local:** one atomic no-clobber publication call and,
  with `O_TMPFILE`, no staging to clean up. The fsync order, single-link and alias checks, digest
  verification and lock policy remain local (a shared local helper would hold them).
- **Prerequisites, configuration or extension:** none beyond the pinned feature set.
- **Semantic gaps against the CBO's preservation constraints:** the canonical store's resumable
  deterministic staging and its "two links until unlink, then exactly one" check are tied to
  `link`+`unlink`; `renameat2` keeps resumable staging but changes the link-count reasoning;
  `O_TMPFILE` removes resumability. Hard-link rejection on recovery is unaffected (it is a read-side
  check).
- **Unsettled premises and what would settle them:** whether resume-by-name is required (design
  owner); whether every target filesystem supports `RENAME_NOREPLACE`/`O_TMPFILE` (host fact; a
  one-line probe would settle).
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** The repository already uses the `O_TMPFILE` + `linkat` form, in the offline helper's `anonymous_copy_file`, so three publication routes coexist (link plus unlink, `O_TMPFILE` + `linkat`, and tempfile's `persist_noclobber`). `renameat_with(NOREPLACE)` keeps the deterministic, resumable `.pending`/`.incomplete` staging and removes the two-name state; legacy two-name aliases still need their settlement paths (`settle_published_alias` and offline `reconcile`) until they are shown absent. The comparative review recommends one small publication helper on `renameat_with(.., RenameFlags::NOREPLACE)` with an explicit `Durable`/`Visible` level, replacing `hard_link` + `remove_file` at 6 sites while keeping deterministic staging and the offline `O_TMPFILE` route. See the [comparative review](../../reviews/host-processes-and-durable-files.md#cbo-003).

### [CBO-004](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-004) — Child-process and process-group supervision implemented per consumer
- **Types:** `rustix/pidfd-process-handles`
- **Realization:** installed pin 1.1.5 (already used by both group-drain sites).
- **Conclusion:** fits (already adopted); the residual is descendant discovery, which rustix
  cannot provide — see `linux-cgroup-v2.md` for the containment route and `linux-pidfd.md` for
  identity.
- **What it would discharge / what remains local:** signalling and waiting are already delegated
  to rustix pidfds; the `/proc` group inventory, re-inventory loop and the "unproved drain parks"
  policy remain local and are duplicated in two crates.
- **Prerequisites, configuration or extension:** `set_child_subreaper` would be the only rustix-level
  addition that changes drain semantics (orphans become reapable by the supervisor); it is
  process-wide and conflicts with an async runtime's own reaping in the same process unless the
  supervisor runs in a dedicated process.
- **Semantic gaps against the CBO's preservation constraints:** a subreaper still relies on
  process-group or parent linkage, not containment; a member that calls `setsid` still reparents
  to the subreaper (good) but leaves the process group (so `kill_process_group` misses it).
- **Unsettled premises and what would settle them:** whether a single shared supervised-child
  type should live in a low-level crate both consumers depend on (repository structure question).
- **Evidence:** Interface-checked; application Proposed.
