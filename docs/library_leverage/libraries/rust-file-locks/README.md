# Rust advisory file locks: std `File::lock`, fs4, fd-lock

Realizations considered: std `File::{lock, lock_shared, try_lock, try_lock_shared, unlock}` on the
pinned toolchain `nightly-2026-09-29` (stable since 1.89.0); `fs4` 1.1.0 (released 2026-04-28,
crates.io; not in the lockfile) and 0.13.1/0.6.6 present only in the local registry cache;
`fd-lock` 4.0.4 (2025-03-10; registry cache only). Research date 2026-10-27. Sources: nightly
`library/std/src/sys/fs/unix.rs` (`flock(LOCK_EX|LOCK_NB)` etc.) and `library/std/src/fs.rs`
(`#[stable(feature = "file_lock", since = "1.89.0")]`); `fs4-1.1.0/src/unix.rs`,
`src/file_ext/async_impl.rs`; `fd-lock-4.0.4/src/sys/unix/mod.rs`.

## Types

### rust-file-locks/flock-whole-file
- **Need it serves:** exclusive or shared advisory lock on an open file, released on close.
- **Recognizable code shapes:** `libc::flock`/`rustix::fs::flock` calls; lock-file wrappers;
  polling `try_lock` with sleeps.
- **Mechanisms and APIs:** std `File::lock`/`lock_shared` (blocking), `try_lock`/`try_lock_shared`
  returning `TryLockError::{WouldBlock, Error}`, `unlock`; on Linux all map to `flock(2)`.
  `fs4::FileExt` offers the same operations (also via `rustix::fs::flock`) plus `allocate`,
  `allocated_size` and filesystem statistics, and "async" variants for tokio/async-std files;
  `fd_lock::RwLock<T: AsFd>` wraps a descriptor with RAII read/write guards (flock on Linux).
- **Obligations discharged:** the syscall and RAII release.
- **Residual caller obligations:** deadline-bounded or cancellable acquisition; lock-file
  creation policy (`O_NOFOLLOW`, mode, ownership, single link); fd inheritance policy
  (`O_CLOEXEC`); lock ordering.
- **Semantic conditions:** `flock` locks belong to the open file description — duplicated or
  inherited fds share them, a second `open` of the same file in the same process conflicts. They
  do not interact with POSIX/OFD `fcntl` locks on Linux, so every participant (including other
  languages) must use `flock` to conflict. No timed acquire exists in the kernel; a blocking
  `flock` can only be interrupted by a signal.
- **Version / build conditions:** std API needs Rust ≥ 1.89 (satisfied by any current toolchain);
  fs4's async methods call the same blocking `flock` on the calling thread (no `spawn_blocking`),
  so they do not make lock waiting asynchronous.
- **Limits and negative results:** none of the three provides a timeout, deadline or async-ready
  wait; a deadline needs either a `try_lock` poll loop or a blocking `lock` on a dedicated thread
  whose result is abandoned on timeout (the abandoned thread later acquires and must drop the
  lock; it occupies a blocking thread for the whole wait). Search scope: whole `src/` of fs4 1.1.0
  and fd-lock 4.0.4 for `timeout`, `spawn_blocking`, `deadline`.
- **Evidence:** Interface-checked: sources above.

## Current applicability

### [CBO-003](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-003) — Durable file publication and the sealed-file byte layer on `std::fs` (instance lock)
- **Types:** `rust-file-locks/flock-whole-file`
- **Realization:** std on the pinned toolchain (already used).
- **Conclusion:** ruled out here as a replacement for the 5 ms `try_lock` poll — neither std nor
  fs4/fd-lock offer a bounded wait; the poll is the deadline mechanism, and the current std usage
  is already the idiomatic library route.
- **What it would discharge / what remains local:** nothing further; a shared local
  `lock_until(deadline)` helper (Rust and the Python twin in CBO-047) is the residual consolidation.
- **Prerequisites, configuration or extension:** none.
- **Semantic gaps against the CBO's preservation constraints:** the "instance lock must conflict
  with online `flock` users" constraint rules out any `fcntl`/OFD-based lock crate; std, fs4 and
  fd-lock on Linux all use `flock`, so they would conflict correctly. A blocking `lock()` on a
  blocking thread cannot honour cancellation; the CBO's research question "can the polling
  `try_lock` become a blocking lock on a blocking thread?" has the answer "only by abandoning the
  waiter on timeout", which leaves a thread blocked until the holder releases.
- **Unsettled premises and what would settle them:** whether a 5 ms poll is a measurable cost at
  all (not claimed; no measurement).
- **Evidence:** Interface-checked.
