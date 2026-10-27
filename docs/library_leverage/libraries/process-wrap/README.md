# process-wrap and command-group (Rust)

Realizations considered: `process-wrap` 10.0.1 (released 2026-09-23; not in the lockfile; Unix
paths depend on `nix`); `command-group` 5.0.1 (2023-11-18; the predecessor by the same project,
superseded by process-wrap). Research date 2026-10-27. Sources: Context7 `/websites/rs_process-wrap`
(docs.rs source pages `std/process_group.rs`, `tokio/process_group.rs`, `std/process_session.rs`,
`tokio/kill_on_drop.rs`); crate source 10.0.1 downloaded into scratch (`Cargo.toml` features,
`src/{std,tokio}/*.rs`).

## Types

### process-wrap/process-group-command-wrappers
- **Need it serves:** spawn a command as the leader of a new process group or session, signal
  the whole group, and reap group members, with std or tokio commands and composable wrappers.
- **Recognizable code shapes:** `Command::process_group(0)` / `pre_exec(setsid)` followed by
  `killpg` and `waitpid` loops; per-site kill-on-drop logic.
- **Mechanisms and APIs:** `CommandWrap::with_new(..)` / `.wrap(ProcessGroup::leader())`,
  `ProcessGroup::attach_to(pgid)`, `ProcessSession`, `KillOnDrop` (tokio), `ResetSigmask`,
  `JobObject`/`CreationFlags` (Windows); child wrapper `ProcessGroupChild` implements
  `start_kill` as `killpg(SIGKILL)`, `wait` as "reap the leader first, then `waitpid(-pgid)`
  until `ECHILD`" (std: one blocking loop; tokio: up to ten `WNOHANG` rounds, then the blocking
  loop on a blocking thread), and `try_wait`.
  Custom wrappers implement `CommandWrapper::{pre_spawn, post_spawn, wrap_child}`.
- **Obligations discharged:** group creation, group-wide signalling, reaping of the caller's own
  children in that group, cross-platform shape.
- **Residual caller obligations:** identity safety (it signals by numeric PGID after the leader is
  reaped), descendants that left the group, output bounding, deadlines/cancellation, drain proof.
- **Semantic conditions:** `waitpid(-pgid)` reaps only children *of the calling process*; it
  returns `ECHILD` once the caller has no children left in the group, even if grandchildren that
  were re-parented to init/a subreaper are still running. "All processes in the group are reaped"
  therefore means "all of *our* children in the group". After the leader has been reaped the
  numeric PGID can be reused, so a later `killpg` is not identity-safe. Members that `setsid` or
  `setpgid` escape.
- **Version / build conditions:** 10.0.1 features `std`/`tokio1` pull `nix`; default features
  include Windows-only items that compile away on Linux.
- **Limits and negative results:** no pidfd use, no `/proc` inventory, no cgroup containment
  (search scope: 10.0.1 `src/**` for `pidfd`, `cgroup`, `subreaper`, `PR_SET_CHILD`; the only
  matches are in the Windows job-object files' comments). command-group 5.0.1 has the same
  process-group design.
- **Evidence:** Interface-checked: docs.rs source via Context7 and downloaded 10.0.1 source.

## Current applicability

### [CBO-004](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-004) — Child-process and process-group supervision implemented per consumer
- **Types:** `process-wrap/process-group-command-wrappers`
- **Realization:** upgrade route 10.0.1 (new dependency; adds a direct `nix` edge — the lockfile
  has `nix` 0.31.3 transitively; process-wrap's `nix` requirement was not checked against it).
- **Conclusion:** ruled out here — its drain semantics are weaker than every preservation
  constraint of the two group-drain sites: no identity protection against PGID reuse, no proof
  that non-child descendants have exited, no `PIDFD_THREAD`-grade exit readiness.
- **What it would discharge / what remains local:** at most the `process_group(0)` + `killpg`
  lines that std/tokio already provide in one call each.
- **Semantic gaps against the CBO's preservation constraints:** "An unproved drain never releases
  the caller" and "start ticks against PID reuse" are violated by its `wait` (returns after
  `ECHILD`) and by signalling a numeric PGID.
- **Unsettled premises and what would settle them:** none; a custom `CommandWrapper` could host the
  repository's own drain logic, but then the library contributes only the wrapper trait.
- **Evidence:** Interface-checked; Proposed (negative).
