# systemd manager D-Bus API (native capability, `org.freedesktop.systemd1`)

Realizations considered: host systemd 255 (255.4-1ubuntu8.18; user manager reachable on the user
bus and its private socket); feature availability for 256–262 recorded from systemd NEWS as
upgrade routes (a distribution upgrade, not a dependency pin). Research date 2026-10-27. Sources:
systemd NEWS (main branch, fetched 2026-10-27, entries for 254–262); Context7 `/systemd/systemd/v255`
(`docs/TRANSIENT-SETTINGS.md`, `docs/CGROUP_DELEGATION.md`); generated proxy signatures in
`zbus_systemd` 0.26201.0 `src/systemd1/generated.rs` (interface shape for systemd 262).

## Types

### systemd-manager-api/transient-units-and-jobs
- **Need it serves:** create and supervise transient services and scopes, wait for their start or
  stop jobs, and kill them, without parsing command-line tool output.
- **Recognizable code shapes:** `systemd-run --user --scope|--unit … --property=…` subprocesses;
  `systemctl --user show --property=… <unit>` parsing `Key=Value` text; `systemctl stop/start/
  reset-failed/is-active` shell-outs with timeouts; loops re-running `systemctl show` until a state
  changes.
- **Mechanisms and APIs:** `Manager.StartTransientUnit(name, mode, properties a(sv), aux)` →
  job object path; `Manager.Subscribe()` then the `JobRemoved(id, job, unit, result)` signal
  gives the job outcome (this is what `systemd-run` waits for); `StopUnit`, `KillUnit(name,
  whom, signal)`, `QueueSignalUnit` (≥ 254), `KillUnitSubgroup` (≥ 258), `ResetFailedUnit`,
  `GetUnitByPID`, `GetUnitByControlGroup`, `GetUnitByPIDFD` (interface present in 262 bindings;
  introduction version not checked), `GetUnitProcesses`, `AttachProcessesToUnit`; scope properties
  `PIDs` and, since 255, `PIDFDs` (create a scope from pidfds instead of PIDs); unit properties
  `ActiveState`, `SubState`, `InvocationID`, `ControlGroup`, `ControlGroupId`, `MemoryCurrent`,
  `MemoryPeak` (≥ 255), `EffectiveMemoryMax/High`, `EffectiveTasksMax` (≥ 256), `OOMKills`
  (≥ 259); `org.freedesktop.DBus.Properties.PropertiesChanged` for state changes; all resource
  settings (`MemoryMax`, `MemorySwapMax`, `TasksMax`, `CPUQuota`, `AllowedCPUs`, `Delegate`, …)
  and scope `RuntimeMaxSec`/`TimeoutStopSec` are accepted on transient units.
- **Obligations discharged:** authoritative unit state and invocation identity; job completion
  as an event; cgroup creation, limit application and teardown; no output parsing.
- **Residual caller obligations:** cross-unit admission (systemd has no semaphore across
  independent clients); ledger durability across processes; policy for what a failed job means;
  bus connection lifetime and timeouts; fallback when the manager is unreachable (observations
  must still be reportable).
- **Semantic conditions:** for a `--scope`, the scope's processes are the caller's own (the
  manager only registers them), so the caller's process must be alive to be moved; a scope whose
  last process exits goes inactive and is collected (`CollectMode`). Properties are typed on the
  bus (`InvocationID` is `ay`, limits are `t` with `UINT64_MAX` for infinity). systemd advises
  using `ControlGroup`/`GetUnitByControlGroup` instead of deriving names from cgroup paths.
  `Effective*` limits report "the most stringent limit systemd is aware of", not raw kernel files.
  Manager-written cgroup xattrs differ by manager (added 2026-10-27): in v255
  `cgroup_xattr_apply()` returns early for a non-system manager before applying
  `user.invocation_id`, `user.delegate`/`trusted.delegate` and the survive xattr, so units of a
  *user* manager carry none of them, while system-manager units (slices, `user@<uid>.service`) do.
  For a user unit the invocation ID must therefore come from the manager (`InvocationID`), not
  from the cgroup.
- **Version / build conditions:** host 255: `PIDFDs`, `MemoryPeak`, `QueueSignalUnit` available;
  `EffectiveMemoryMax`, `KillUnitSubgroup`, pidfd inode IDs (`$MAINPIDFDID`, ≥ 258), `OOMKills`
  not available. A client compiled against newer interface descriptions fails at runtime
  (unknown property/method) on 255, not at build time.
- **Limits and negative results:** no admission semaphore, no cross-process lease; readiness of an
  application protocol (for example a database accepting connections) is not a manager fact
  unless the service uses `Type=notify` and calls `sd_notify(READY=1)`.
- **Evidence:** Interface-checked: systemd NEWS text, v255 docs via Context7, generated
  interface signatures. No call was made against the host manager. The xattr condition is
  Interface-checked against systemd v255 `src/core/cgroup.c` and was observed read-only on a
  systemd 255 host (one app scope and two user services without `user.invocation_id`;
  `user@1000.service` and the slices with it) and in a delegated-scope
  [probe](../../../design_review/evidence/library-leverage-delegated-cgroup-containment-2026-10-27/README.md).

## Current applicability

### [CBO-020](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-020) — Primary-receiver supervision split between the Rust runtime and a Python script
- **Types:** `systemd-manager-api/transient-units-and-jobs`
- **Realization:** host systemd 255 manager; client via `zbus` (see `zbus-systemd.md`) or Python
  (see `python-dbus-clients.md`).
- **Conclusion:** fits with conditions — the manager owns the facts the two sides currently
  re-derive (unit active state, invocation ID, control group, job results, kills), so either side
  could observe them directly; it does not own the readiness receipts, nonce, script/worker hashes
  or ownership ledger, which remain policy.
- **What it would discharge / what remains local:** `systemctl show` parsing, start/stop job
  waiting, unit-by-cgroup lookup, kill-to-cgroup; the primary's protocol readiness, ownership
  identity (`PrimaryOwner`), receipts and drain policy remain.
- **Prerequisites, configuration or extension:** user-bus connectivity from the runtime and the
  script; deciding which side owns unit lifecycle (one writer).
- **Semantic gaps against the CBO's preservation constraints:** "systemd user units are the
  placement authority" is preserved and strengthened; "ownership fails closed" requires mapping
  bus errors and an unreachable manager to refusal; properties newer than 255 must not be relied on.
- **Unsettled premises and what would settle them:** which script operations are generic
  supervision vs policy (CBO-048 notes the 11.6k-line script bodies were not read; not read in this
  pass either); probe P2.
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** The runtime is the only consumer of `observe-primary`, and
`ensure-primary` runs once per study run while observation runs at every heartbeat. The comparative
review keeps the script as the lifecycle owner and moves observation to kernel evidence (the
cgroup `populated` flag plus a main-PID pidfd handed off by `ensure-primary`) rather than to a
manager client. See the [comparative review](../../reviews/host-processes-and-durable-files.md#cbo-020).

### [CBO-048](../../../library_leverage/codebase/developer-tooling.md#cbo-048) — Host workload admission and service supervision rebuilt above systemd (non-SurrealDB parts)
- **Types:** `systemd-manager-api/transient-units-and-jobs`
- **Realization:** host 255.
- **Conclusion:** fits with conditions for observation and lifecycle calls (unit state, invocation
  ID, control group, job completion, stop/kill); ruled out as an admission ledger (no cross-client
  semaphore, as the CBO's counterevidence states).
- **What it would discharge / what remains local:** the `systemctl` shell-outs and their
  `Key=Value` parsing (the helpers in the admission, lifetime, compiler-cache, telemetry and
  supervisor modules); polling for unit stop can become a `JobRemoved`/`PropertiesChanged` wait.
  Lane capacity, boot/start identity of owners, `unbounded` mode, release-never-inferred rules
  and drain reportability without the manager remain local.
- **Prerequisites, configuration or extension:** a D-Bus client in Python that respects the
  stdlib-only cold-start rule (only stdlib or a vendored pure-Python client qualifies).
- **Semantic gaps against the CBO's preservation constraints:** "drain observations stay
  reportable when the manager is unavailable" — the kernel cgroup files remain the fallback
  observation, so the manager API cannot be the only observation path.
- **Unsettled premises and what would settle them:** whether the `systemd-run --scope` placement
  (the caller's own process becomes the scope) can be replaced by `StartTransientUnit` with
  `PIDFDs` from the launcher without changing the process tree the workloads expect (probe P2).
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** The premise that the paths re-derive manager state is only
partly true: the sampled paths already combine manager facts (`systemctl show`), kernel cgroup
evidence and the ledger the premises require, and `systemd-run --scope` already contains from the
first instruction. Bootstrap admission runs under system Python, so a non-stdlib D-Bus client
would be a second route. The comparative review retains the current paths. See the
[comparative review](../../reviews/host-processes-and-durable-files.md#cbo-048).
- **Footprint correction:** `systemctl` appears far more than "about 25" times — 36 references in
  the supervisor script (through one `systemctl()` helper) plus 14 in the telemetry service, 7 in
  admission, 6 in operation lifetime and further sites in `pse_env.py`, `native_operation.py`,
  `activity.py`, `compiler_cache.py` and `llvm_system.py` (rg counts of the word, non-test
  modules; a call-site count would be lower).
