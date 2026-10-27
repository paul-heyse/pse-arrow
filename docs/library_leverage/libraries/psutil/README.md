# psutil (Python)

Realizations considered: `psutil` 7.2.2 (latest on PyPI at research date, 2026-01-28; not
installed in the project venv; C extension). Research date 2026-10-27. Sources: Context7
`/giampaolo/psutil/v7.2.2` (docs: `Process`, `is_running`, `create_time`, `wait_procs`,
`memory_full_info`); sdist source in scratch: `psutil-7.2.2/psutil/_pslinux.py` (`create_time`,
`CLOCK_TICKS`, `boot_time`), `psutil/_psposix.py` (`wait_pid_pidfd_open`).

## Types

### psutil/process-snapshot-and-wait
- **Need it serves:** cross-platform process facts (status, memory, children, creation time) and
  waiting for process termination.
- **Recognizable code shapes:** `/proc/<pid>/stat` and `/proc/<pid>/status` parsing in Python;
  `os.kill(pid, 0)` liveness probes; tree-kill helpers.
- **Mechanisms and APIs:** `Process(pid)` with `oneshot()`, `status()`, `memory_info()`
  (`rss`, …), `memory_full_info()`, `children(recursive=True)`, `create_time(monotonic=False)`,
  `is_running()` (PID + creation-time check), `wait(timeout)` (pidfd + `poll` on Linux when
  available), `wait_procs(procs, timeout, callback)`, `send_signal`/`kill` with a pre-emptive
  reuse check.
- **Obligations discharged:** `/proc` parsing and units; PID-reuse checks inside psutil's own
  methods; pidfd-based waiting.
- **Residual caller obligations:** boot-ID binding of identities, namespace/executable facts,
  persistent identity records, any cgroup or pressure facts.
- **Semantic conditions:** `create_time()` is `boot_time + starttime/CLK_TCK` as a float based on
  the system clock (documented as affected by clock changes); `create_time(monotonic=True)`
  returns ticks/CLK_TCK as a float — usable for equality within a boot but not the raw tick
  integer. `children(recursive=True)` follows parent links, so re-parented descendants are missed.
  `VmHWM` (lifetime peak RSS) is not exposed as such (`memory_info` gives current RSS).
- **Version / build conditions:** C extension wheel; 7.2.2.
- **Limits and negative results:** no PSI and no cgroup API (search scope: `psutil-7.2.2/psutil/*.py`
  for `pressure`, `/cgroup`; no matches; Context7 v7.2.2 docs query returned none); no process
  group or cgroup drain.
- **Evidence:** Interface-checked: Context7 docs and sdist source.

## Current applicability

### [CBO-019](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-019) — Linux process, cgroup and pressure facts parsed by hand in Rust and Python
- **Types:** `psutil/process-snapshot-and-wait`
- **Realization:** upgrade route 7.2.2 (new compiled dependency).
- **Conclusion:** ruled out here — it covers none of the cgroup or PSI facts, its identity is a
  float derived from the same tick field without the boot-ID/namespace binding, and the
  stdlib-only cold-start rule excludes it from hook paths.
- **What it would discharge / what remains local:** the few-line `stat` split only.
- **Semantic gaps against the CBO's preservation constraints:** "start-tick identity against PID
  reuse" is weakened by float conversion and clock dependence of the non-monotonic variant;
  "absence and unavailable observation are distinct" must be rebuilt from `NoSuchProcess`/
  `AccessDenied`/`ZombieProcess`.
- **Unsettled premises and what would settle them:** none.
- **Evidence:** Interface-checked; Proposed (negative).

### [CBO-048](../../../library_leverage/codebase/developer-tooling.md#cbo-048) — Host workload admission and service supervision (non-SurrealDB parts)
- **Types:** `psutil/process-snapshot-and-wait`
- **Conclusion:** ruled out for the same reasons; the `/proc` and pressure readers it was asked to
  replace are mostly cgroup/PSI readers.
- **Evidence:** Interface-checked; Proposed (negative).
