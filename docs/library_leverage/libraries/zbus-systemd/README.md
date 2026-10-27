# zbus and zbus_systemd (Rust)

Realizations considered: `zbus` 6.1.0 (released 2026-10-23, crates.io) and `zbus_systemd`
0.26201.0 (2026-10-21; bindings generated from systemd 262.x interface descriptions, requires
`zbus` 6 with `proxy`); `systemd-zbus` 5.3.2 (2025-05-20) noted as an alternative binding; none in
the lockfile. zbus 5.x is the previous major (used, for example, by cgroups-rs 0.5.1). Research
date 2026-10-27. Sources: Context7 `/z-galaxy/zbus` (book `client.md`, `blocking.md`,
`upgrading-to-6.md`, README "Special tokio support"); `zbus_systemd-0.26201.0` source
(`Cargo.toml` features `systemd1` etc., `src/systemd1/generated.rs`).

## Types

### zbus-systemd/typed-systemd-manager-client
- **Need it serves:** call the systemd manager from Rust (async on tokio or blocking) with typed
  methods, properties and signal streams instead of spawning `systemctl`/`systemd-run`.
- **Recognizable code shapes:** `Command::new("systemctl")`, `Command::new("systemd-run")`,
  helper scripts invoked only to talk to the manager; text parsing of `show` output.
- **Mechanisms and APIs:** `zbus::Connection::session()` (the user manager is on the user bus) or
  `zbus::blocking::Connection`; `zbus_systemd::systemd1::ManagerProxy` with
  `start_transient_unit(name, mode, Vec<(String, OwnedVariant)>, aux)`, `subscribe()`,
  `receive_job_removed()` stream, `kill_unit`, `stop_unit`, `get_unit_by_control_group`,
  `get_unit_processes`; `UnitProxy`/`ScopeProxy`/`ServiceProxy` property getters (`active_state`,
  `invocation_id: Vec<u8>`, `control_group`, `control_group_id`, `memory_peak`,
  `effective_memory_max`, …) and `receive_*_changed` property streams. zbus 6 with tokio needs
  `default-features = false, features = ["tokio", "proxy"]` and then runs on the caller's tokio
  runtime "with no thread of zbus's own"; the blocking API uses tokio's `block_on` when the
  `tokio` feature is on. Unix fd passing uses `zvariant::OwnedFd` (needed for `PIDFDs`).
- **Obligations discharged:** wire protocol, authentication, typed (de)serialization, signal
  subscription and match rules, async integration.
- **Residual caller obligations:** choosing only properties the host manager has (bindings
  describe systemd 262; on an older manager calls fail at runtime); job-wait logic (subscribe
  before starting to avoid missing `JobRemoved`); timeouts and fail-closed mapping of errors;
  building the `a(sv)` property list for transient units.
- **Semantic conditions:** D-Bus calls are asynchronous messages with their own timeouts;
  signal streams can drop or reorder only according to D-Bus semantics (ordered per connection).
- **Version / build conditions:** `zbus_systemd` 0.26201.0 requires zbus 6; earlier
  `zbus_systemd` releases target zbus 5 (not enumerated here). Pure Rust, no libsystemd.
- **Limits and negative results:** no admission or ledger semantics; the binding generation
  version does not equal the host's systemd version.
- **Evidence:** Interface-checked: Context7 zbus book/README; zbus_systemd source.

## Current applicability

### [CBO-020](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-020) — Primary-receiver supervision split between the Rust runtime and a Python script
- **Types:** `zbus-systemd/typed-systemd-manager-client`, `systemd-manager-api/transient-units-and-jobs`
- **Realization:** upgrade route zbus 6.1.0 + zbus_systemd 0.26201.0 (new dependencies; zbus pulls
  its own zvariant/zbus_names family, `enumflags2`, `serde` etc.; tokio integration fits the
  pinned tokio 1.53.2 — exact version requirements not checked).
- **Conclusion:** fits with conditions — the runtime could observe the primary and worker units
  directly (state, invocation ID, control group, job results) and could own `StartTransientUnit`
  for them, removing the interpreter start and JSON exchange for observation; the script's
  policy (receipts, nonce, readiness of the database protocol, schema work) does not move with it.
- **What it would discharge / what remains local:** the "observe" round-trip through the script and
  its re-verification on both sides for manager-owned facts; the ownership identity, SHA-256
  pinning of deployed artifacts and readiness receipts remain.
- **Prerequisites, configuration or extension:** a single owner for unit lifecycle (otherwise the
  split simply moves); use only systemd-255 properties.
- **Semantic gaps against the CBO's preservation constraints:** "nothing is released without a
  drain" needs the job result plus a cgroup `populated 0` observation; the manager signal alone
  reports unit state, not process drain of a scope that leaked processes elsewhere (none can, by
  cgroup containment).
- **Unsettled premises and what would settle them:** which script functions are generic
  supervision (the CBO's own question; bodies not read); probe P2 for `StartTransientUnit` with
  `PIDFDs` on the user manager.
- **Evidence:** Interface-checked; application Proposed.

### [CBO-004](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-004) — Child-process and process-group supervision implemented per consumer
- **Types:** `zbus-systemd/typed-systemd-manager-client`
- **Realization:** upgrade route as above.
- **Conclusion:** unsettled — a transient scope per compiler child (created from its pidfd) gives
  containment and `KillUnit(whom=all)` without delegation, but adds a bus round-trip and a job wait
  to every compile; per-child delegated cgroups (`linux-cgroup-v2.md`) achieve the same with file
  writes.
- **Semantic gaps against the CBO's preservation constraints:** between spawn and scope creation
  the child is not yet contained (a fork in that window lands outside unless the child is stopped
  until the scope exists).
- **Unsettled premises and what would settle them:** compile frequency and latency tolerance (not
  measured); whether delegation is acceptable (if yes, D-Bus is unnecessary for this CBO).
- **Evidence:** Interface-checked; Proposed.
