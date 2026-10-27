# tokio::process (Rust)

Realizations considered: installed `tokio` 1.53.2 (workspace pin `=1.53.2`). Research date
2026-10-27. Sources: installed source `tokio-1.53.2/src/process/mod.rs` (`Command::kill_on_drop`,
`Command::process_group`), `src/process/unix/{pidfd_reaper.rs,orphan.rs,reap.rs}`.

## Types

### tokio-process/async-child-with-pidfd-reaping
- **Need it serves:** spawn, bound and await a direct child from async code, with kill on drop.
- **Recognizable code shapes:** `tokio::process::Command` with `kill_on_drop(true)`, piped
  output read with `take(limit)`, `select!` between wait and cancellation.
- **Mechanisms and APIs:** `Command::process_group(pgid)`, `kill_on_drop`, `Child::{wait,
  try_wait, start_kill, kill, id}`; on Linux the reaper opens a `PIDFD_NONBLOCK` pidfd for the
  child and registers it with the reactor (`pidfd_reaper.rs`), falling back to `SIGCHLD`-driven
  reaping and an orphan queue (`orphan.rs`) that reaps dropped children later.
- **Obligations discharged:** async waiting on a direct child without a blocking thread;
  eventual reaping of dropped children.
- **Residual caller obligations:** descendants, group drain, identity of non-child processes;
  the internal pidfd is not exposed.
- **Semantic conditions:** `kill_on_drop` sends `SIGKILL` to the direct child only; the orphan
  queue reaps asynchronously after drop, so drop does not prove exit.
- **Limits and negative results:** no group/descendant drain API.
- **Evidence:** Interface-checked: installed 1.53.2 source.

## Current applicability

### [CBO-004](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-004) — Child-process and process-group supervision implemented per consumer
- **Types:** `tokio-process/async-child-with-pidfd-reaping`
- **Realization:** installed pin 1.53.2 (already used by the managed-primary control calls).
- **Conclusion:** fits (already adopted) for the direct-child control calls; ruled out as a
  group-drain mechanism.
- **What it would discharge / what remains local:** the managed-primary control path already
  relies on it plus an explicit kill-and-join; group-drain sites need a separate mechanism.
- **Semantic gaps against the CBO's preservation constraints:** if a control script starts
  descendants that outlive it, kill-on-drop does not reach them (observed baseline: that path
  kills and joins only the direct child).
- **Unsettled premises and what would settle them:** whether the control scripts can leave
  descendants (they invoke `systemctl`/`systemd-run`, whose units are owned by the manager, so
  probably not; not verified).
- **Evidence:** Interface-checked.
