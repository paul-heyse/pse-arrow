# Delegated child-cgroup containment probe (P1)

This is evidence for the comparative library-leverage review of CBO-004 (supervised child-process
drain), dated 2026-10-27, probe request P1. It characterizes the host's kernel and systemd user
manager in isolation, using a stdlib Python driver. It calls no pse-arrow code.

## Question

A supervisor runs in the root of a user scope created with `Delegate=yes`. Can it do the
following?

- (a) Create a leaf child cgroup without enabling controllers.
- (b) Move a freshly forked child into the leaf before exec, using a pre-exec write of `0` to
  `leaf/cgroup.procs`.
- (c) Kill the whole subtree with `cgroup.kill`. The subtree includes a grandchild that called
  `setsid` and a double-forked daemon.
- (d) Observe `populated 0` through `poll(POLLPRI)` on `leaf/cgroup.events`, then `rmdir` the
  leaf.

All of this must happen while the scope's limits stay enforced and the child is charged to the
scope.

The probe also asks:

- Does the no-internal-process rule refuse the move while the driver sits in the root?
- Is the `user.delegate` xattr present?
- What does `systemctl --user stop` do to a scope with a live, populated leaf?

## Environment

| Item | Value |
|---|---|
| Kernel | Linux 7.0.0-38-generic, cgroup2 mounted `nsdelegate,memory_recursiveprot` |
| systemd | 255 (255.4-1ubuntu8.18), system and user manager |
| Controllers delegated to `user@1000.service` | `cpu memory pids` (no `cpuset`, no `io`) |
| Interpreter | `/usr/bin/python3` 3.12.3, run with `-I` |
| Scope placement | `app.slice` (the default for `systemd-run --user --scope`). The `pse.slice` and `psebuild.slice` slices were not touched. |

The probe launches `systemd-run` directly, not through `scripts/pse-env --`. The probe's subject
is the transient scope itself and its properties (`Delegate=yes`, the limits, `OOMPolicy`). The
pse-env route would have added its own allocation scope around the command and registered it in
the host ledger. That adds nothing to the question, and it would have charged a probe to the
admission ledger.

## Commands

- `probe/driver.py` takes one of these modes: `main`, `oom`, `hold`, `nodelegate` and the
  internal `tree`.
- `probe/stop-with-live-leaf.sh` runs the stop experiment.

```sh
# main: steps (a)-(d), controller enabling, memory/pids/CPU charge and enforcement
systemd-run --user --scope --quiet --collect --unit=pse-probe-delegate-1 \
  -p Delegate=yes -p MemoryMax=1G -p MemorySwapMax=0 -p TasksMax=64 -p CPUQuota=50% \
  -- /usr/bin/python3 -I probe/driver.py main            > raw/main.json
# MemoryMax enforcement on a leaf child (768 MiB requested against 256 MiB)
systemd-run --user --scope --quiet --collect --unit=pse-probe-delegate-2 \
  -p Delegate=yes -p MemoryMax=256M -p MemorySwapMax=0 -p OOMPolicy=continue -p TasksMax=64 \
  -- /usr/bin/python3 -I probe/driver.py oom             > raw/oom.json
# comparison: the same scope without Delegate=yes
systemd-run --user --scope --quiet --collect --unit=pse-probe-delegate-3 \
  -p MemoryMax=256M -p TasksMax=16 -- /usr/bin/python3 -I probe/driver.py nodelegate > raw/nodelegate.json
# stop a scope whose leaf is populated (unit pse-probe-delegate-4)
probe/stop-with-live-leaf.sh <state-file>                > raw/stop-with-live-leaf.txt
# AllowedCPUs on a user scope (unit pse-probe-delegate-5)
systemd-run --user --scope -p Delegate=yes -p AllowedCPUs=0-3 -- python3 -c '<affinity/controllers>' > raw/allowedcpus.txt
```

The child tree in the leaf is built by `driver.py tree full <MiB>`. It contains:

- the direct child (Python);
- a forked grandchild that calls `setsid()` and execs `sleep 600`;
- a classic double-forked daemon: the intermediate process calls `setsid()`, forks and `_exit`s,
  and the daemon execs `sleep 600`;
- in the child itself, an allocation of N MiB with every page touched.

## Outcome per step

Every result below is *Measured* once, on this host, under the conditions above.

| Step | Result | Evidence (`raw/`) |
|---|---|---|
| (a) `mkdir` leaf, no controllers | **Succeeds.** The leaf is `domain`, with `cgroup.controllers` empty. It has only core files: `cgroup.*`, `cpu.stat`, and `*.pressure`. It has no `memory.current` and no `pids.current`. | `main.json` `a_*` |
| (b) Pre-exec move (`write "0"` from the forked child) | **Succeeds** while the driver stays in the scope root and the scope's `cgroup.subtree_control` is empty. All three members end up in `leaf-a`: the child, the setsid grandchild (new SID) and the daemon (new SID, reparented to `systemd --user`, pid 2381). The kernel does not refuse it: the no-internal-process rule does not apply while `subtree_control` holds no domain controller. | `main.json` `b_member_cgroups`, `b_member_sid`, `b_member_ppid` |
| Memory charge | The scope's `memory.current` rose by 264.6 MiB for a 256 MiB child (child VmRSS 276 MB). Memory used in the leaf is charged to the scope, the nearest ancestor with `memory` enabled. | `main.json` `b_scope_*` |
| `rmdir` while populated | `EBUSY` | `main.json` `rmdir_while_populated` |
| (c) `cgroup.kill` | **Kills all three members**, including the setsid grandchild and the reparented daemon. The pidfd of each member became readable. The direct child returned −9. | `main.json` `c_d_kill` |
| (d) `POLLPRI` on `cgroup.events` | With the poll armed before the kill, one wakeup arrived. `populated 0` came after 12.7 ms, while the direct child was still an **unreaped zombie** (`State: Z`). Zombies do not hold `populated`. `rmdir` succeeded after the reap. | `main.json` `c_d_kill`, `c_child_state_before_reap`, `d_rmdir` |
| Zombie pids charge | After the kill, the scope showed `pids.current` = 2 (the driver plus the zombie) while the leaf showed `populated 0`. A zombie keeps its pids charge until it is reaped. | `main.json` `c_scope_after_kill` |
| TasksMax=64 | A leaf child forked 62 times, then got `EAGAIN`. The scope showed `pids.current` = 64 and `pids.events max 1`. The leaf's tasks count against the scope's limit. The leaf then drained through `cgroup.kill` in 83 ms. | `main.json` `tasks` |
| CPUQuota=50% | A leaf child spun for 2.0 s of wall time and used 1.038 s of CPU. The scope's `nr_throttled` rose by 20, and the leaf's `cpu.stat` shows the same usage. | `main.json` `cpu` |
| MemoryMax=256M | A leaf child that grew toward 768 MiB was OOM-killed (−9). The scope's `memory.events` changed to `max 37, oom 1, oom_kill 1`. The driver in the scope root survived because of `OOMPolicy=continue` and `memory.oom.group 0`. | `oom.json` |
| Enable controllers while the driver is in the root | `+memory` gave **EBUSY**. `+pids` and `+cpu` were **accepted** (both are threaded-capable) and were reverted at once. Domain controllers, and so per-leaf memory limits, are unavailable while a supervisor is in the root. | `main.json` `enable_controllers_with_procs_in_root` |
| `user.delegate` xattr | **Absent** on the delegated scope, read from inside and from outside. `systemctl --user show` reports `Delegate=yes` and `DelegateControllers=cpu cpuset io memory pids`. | `main.json` `scope_xattrs`, `stop-with-live-leaf.txt` |
| Without `Delegate=yes` | `mkdir` and `rmdir` of a leaf **also succeed**: the scope directory is owned by uid 1000. Kernel permission therefore does not show delegation. Writing there breaks systemd's contract, but nothing enforces the contract. | `nodelegate.json` |
| `systemctl --user stop` with a populated leaf | The stop took 36 ms. The driver, the child, the setsid grandchild and the daemon are all gone, and both the leaf and the scope cgroup were removed (`KillMode=control-group`). | `stop-with-live-leaf.txt` |
| AllowedCPUs on a user scope | The scope has no `cpuset` controller (`cgroup.controllers` is `cpu memory pids`). Its affinity stayed at 32 CPUs, so `AllowedCPUs=0-3` was **not enforced** through the cgroup. Delegation and leaves make no difference to this. A cgroup move without `cpuset` leaves the inherited affinity unchanged: the leaf child's affinity matched the driver's. | `allowedcpus.txt`, `main.json` `cpu.child_affinity_count` |

### Why `user.delegate` is missing (source, *Interface-checked*)

In systemd v255 `src/core/cgroup.c`, `cgroup_xattr_apply()` applies `user.oomd_*`, the log
xattrs and the coredump xattrs for every manager. It then returns early
`if (!MANAGER_IS_SYSTEM(u->manager))`, before
`cgroup_invocation_id_xattr_apply`, `cgroup_delegate_xattr_apply` (`trusted.delegate` /
`user.delegate`) and `cgroup_survive_xattr_apply`. The same guard is on `main` as of 2026-10-27.
Units under a user manager therefore never carry `user.delegate` or `user.invocation_id`. The
`user.delegate=1` on `user@1000.service` is set by the *system* manager.

## Limits

- Each case ran once, on one host and kernel. No timing distribution was collected. The 12.7 ms
  and 83 ms figures show order of magnitude only.
- The move used a pre-exec `write("0")` from Python's `preexec_fn`. `clone3(CLONE_INTO_CGROUP)`
  was not exercised. Rust's `CommandExt::pre_exec` performs the same syscall sequence, but it was
  not run.
- Fork races during `cgroup.kill` (a member forking at the instant of the kill) were not
  stressed. Kernel handling of that case is inherited from the kernel documentation, not tested
  here.
- The OOM case used `OOMPolicy=continue`. With the user manager's default
  (`DefaultOOMPolicy=stop`), systemd would stop the whole scope, supervisor included, after a
  leaf OOM kill. This was not run.
- The probe scopes lived in `app.slice`, not in the pse allocation slices. Nested slice limits
  were not exercised.
- The host disk was full near the end of the session. All raw outputs were written before that
  point and are complete. A final cleanup check confirmed that no `pse-probe-delegate-*` unit,
  cgroup or driver process remained, and that `app.slice` `cgroup.subtree_control` was
  unchanged (`memory pids`).
