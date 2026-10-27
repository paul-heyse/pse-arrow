# Linux pidfd for threads (native capability, via rustix)

Realizations considered: Linux kernel ≥ 6.9 for `PIDFD_THREAD` (host reports 7.0); installed
`rustix` 1.1.5 (`process` and `thread` features declared: `pidfd_open`, `PidfdFlags` with
externally defined bits allowed, `thread::gettid`); `PidfdFlags` has no named `THREAD` constant at
1.1.5, so the flag is passed as `PidfdFlags::from_bits_retain(PIDFD_THREAD)` (value equals
`O_EXCL`). Awaited through `tokio::io::unix::AsyncFd` (tokio `net` feature).
Research date 2026-10-27. Sources: man7.org `pidfd_open(2)` (fetched 2026-10-27: `PIDFD_THREAD`
"since Linux 6.9"; with `PIDFD_THREAD` "the file descriptor becomes readable when the task exits and
becomes a zombie, even if the thread-group is not empty"); `rustix-1.1.5/src/process/pidfd.rs`;
`tokio-1.53.2/src/process/unix/pidfd_reaper.rs` (tokio's own pidfd use for children).

## Types

### linux-pidfd/thread-exit-readiness
- **Need it serves:** asynchronously learn that a specific OS thread has exited (after its
  thread-exit destructors), without dedicating a blocking thread to `join()`.
- **Recognizable code shapes:** `spawn_blocking(move || handle.join())`; polling
  `JoinHandle::is_finished()`; completion messages sent from inside the thread closure that do not
  witness thread-local destruction.
- **Mechanisms and APIs:** inside the new thread: `pidfd_open(gettid(), PIDFD_THREAD)` and hand
  the fd back before running work (non-leader threads are reaped at exit, so opening the fd after
  exit fails with `ESRCH`); outside: `AsyncFd::new(fd)?.readable().await`, then `handle.join()`
  for the result/panic payload.
- **Obligations discharged:** kernel-level exit notification integrated with the reactor.
- **Residual caller obligations:** obtaining the tid inside the thread, fallback for older kernels
  or `EINVAL`, ensuring the final `join()` cannot block, result and panic mapping.
- **Semantic conditions:** readiness means the kernel task exited, which happens after user-space
  thread-exit processing (C++ `thread_local`/Rust TLS destructors and pthread-key destructors run
  in the thread before the exit syscall). Whether `pthread_join` can still block briefly after
  readiness (the `CLONE_CHILD_CLEARTID` futex wake) is not established from documents.
- **Version / build conditions:** Linux ≥ 6.9; rustix `process` + `thread`; tokio `net`.
- **Limits and negative results:** Linux-only; not available in older kernels; no portable
  equivalent in std or tokio.
- **Evidence:** Interface-checked: man page and rustix source; the destructor-ordering and
  join-non-blocking claims are reasoning, not Tested.

## Current applicability

### [CBO-018](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-018) — Two executor threads per native job to wait for a join
- **Types:** linux-pidfd/thread-exit-readiness; tokio/fd-readiness-await.
- **Realization:** installed rustix 1.1.5 and tokio 1.53.2 (`net` present in the unified feature
  graph; declare it explicitly if adopted); kernel ≥ 6.9.
- **Conclusion:** unsettled — a credible route that removes the blocking-pool slot while
  preserving "completion signalled only after the thread exits", pending a probe of destructor
  ordering and join latency; the CBO itself rates the cost as probably negligible.
- **What it would discharge / what remains local:** discharges the waiting thread. Local: the
  spawn helper (stack size, name, fd hand-off), fallback, join, panic mapping, accounting.
- **Semantic gaps:** host-local kernel requirement (acceptable per local-over-portable stance, but a
  fallback is needed for robustness).
- **Evidence:** Interface-checked; Proposed. Probe request in README.
