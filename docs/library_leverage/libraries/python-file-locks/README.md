# Python file-lock libraries: filelock, portalocker, fasteners, flufl.lock

Realizations considered: `filelock` 3.32.6 installed in the project venv (transitive via
`virtualenv`/`python-discovery`; not a declared dependency); `portalocker` 4.4.1 (2026-10-16),
`fasteners` 0.20 (2025-08-11), `flufl.lock` 10.0.0 (2026-10-12) from PyPI sdists, not installed.
Python 3.14.8 stdlib `fcntl.flock` as the baseline. Research date 2026-10-27. Sources: installed
`filelock/{_unix.py,_api.py,_descriptor.py,_read_write.py,_lease.py,__init__.py}`; sdists in
scratch: `portalocker-4.4.1/portalocker/portalocker.py`, `fasteners-0.20/fasteners/process_mechanism.py`,
`flufl_lock-10.0.0/src/flufl/lock/_lockfile.py`.

## Types

### python-file-locks/flock-with-timeout-poll (filelock)
- **Need it serves:** an exclusive inter-process lock on a path with a timeout, re-entrancy and a
  context manager.
- **Recognizable code shapes:** `fcntl.flock(fd, LOCK_EX | LOCK_NB)` in a `while` loop with
  `time.sleep`; deadline arithmetic around it.
- **Mechanisms and APIs:** `FileLock(path, timeout=…, mode=…, thread_local=…, blocking=…,
  is_singleton=…, poll_interval=0.05, lifetime=None)`; `acquire(timeout=…, poll_interval=…)`;
  `UnixFileLock` opens with `O_NOFOLLOW` (when available), defers truncation and `fchmod` until
  the lock is held, and detects a lock taken on an unlinked inode (`st_nlink == 0`) and retries;
  `lock_descriptor(fd, blocking=…, poll_interval=…)` locks an existing descriptor; `SoftFileLock`
  (existence-based) with owner markers and `SoftFileLease`; `ReadWriteLock` (shared/exclusive)
  implemented with SQLite, not `flock`.
- **Obligations discharged:** poll loop, timeout, re-entrancy counting, careful open/truncate
  ordering, release bookkeeping.
- **Residual caller obligations:** absolute deadlines shared across several locks (the API takes
  per-call relative timeouts), lock ordering, private-file ownership checks (`st_uid`,
  mode `0600` verification rather than just setting it), fd lifetime across phase changes.
- **Semantic conditions:** exclusive `flock` only — no shared `flock`; `ReadWriteLock` does not
  conflict with `flock` users. On a filesystem without `flock` it *falls back to `SoftFileLock`
  with a warning* by default. `UnixFileLock(fallback_to_soft=False)`, `preserve_lock_file=True`
  or `on_acquired` can instead fail closed on `ENOSYS`. `lock_descriptor(fd)` preserves the
  caller's descriptor ownership: no open/truncate/chmod/close/unlink or fallback. Waiting is
  still polling (default 50 ms).
- **Version / build conditions:** 3.32.6 (pure Python).
- **Limits and negative results:** no shared `flock`, no absolute-deadline API, polling only.
- **Evidence:** Interface-checked: installed 3.32.6 source.

### python-file-locks/alternatives
- **portalocker 4.4.1:** wraps `fcntl.flock` by default (or `fcntl.lockf` via `LockfLocker`),
  exclusive and shared flags, timeouts by polling, Redis/semaphore helpers. Semantically close to
  stdlib `flock` plus a poll loop.
- **fasteners 0.20:** `InterProcessLock`/`InterProcessReaderWriterLock` use `fcntl.lockf`
  (POSIX record locks), which do **not** conflict with `flock` holders on Linux.
- **flufl.lock 10.0.0:** NFS-safe lock built on hard-linking a claim file to the lock file, with a
  `lifetime` after which other processes may break it — a lease, not a kernel lock tied to fd
  lifetime.
- **Evidence:** Interface-checked: sdist sources named above (lock call sites only).

## Current applicability

### [CBO-047](../../../library_leverage/codebase/developer-tooling.md#cbo-047) — File-ledger primitives re-derived per script
- **Types:** `python-file-locks/flock-with-timeout-poll`, `python-file-locks/alternatives`
- **Realization:** filelock 3.32.6 is present transitively (would become a declared pin);
  the others are new dependencies.
- **Conclusion:** ruled out here — none expresses the ledger's rules better than the stdlib:
  filelock lacks shared `flock` and composed absolute deadlines; its default soft-lock fallback
  warns and can be disabled, so intrinsic fail-open behavior is not a rejection reason;
  fasteners uses a non-conflicting lock family; flufl.lock has lease semantics; portalocker is a
  thin `flock` wrapper. Stdlib-only cold-start paths also exclude all four. The consolidation the
  CBO seeks is a single in-repo module (the CBO's own counter-hypothesis), not a library.
- **What it would discharge / what remains local:** at most the poll loop in `lock_until`; lock
  order (invocation → identity → resource), absolute deadlines, `O_NOFOLLOW`/`0600`/owner checks,
  locks surviving phase changes and fail-closed handling stay local.
- **Semantic gaps against the CBO's preservation constraints:** shared native `flock`, one
  absolute clock across phases and stdlib-only cold start still remain local. The descriptor
  API and explicit fail-closed configuration address ownership/fallback concerns, but do not
  discharge those remaining needs.
- **Unsettled premises and what would settle them:** none decisive.
- **Evidence:** Interface-checked; Proposed (negative).
