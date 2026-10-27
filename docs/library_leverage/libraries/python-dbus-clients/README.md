# Python D-Bus clients for systemd: jeepney, dbus-fast, pystemd

Realizations considered: `jeepney` 0.9.0 installed in the project venv (transitive, via `keyring`
and `secretstorage`; not a declared dependency); `dbus-fast` 5.2.0 (2026-10-02, PyPI; not
installed); `pystemd` 0.15.3 (2026-01-15, PyPI; not installed; Cython extension linking
libsystemd). Python 3.14.8. Research date 2026-10-27. Sources: installed
`.venv/lib/python3.14/site-packages/jeepney/{io/blocking.py, low_level.py, fds.py}`; sdists
downloaded into scratch: `dbus_fast-5.2.0/src/dbus_fast/{aio,glib}/message_bus.py`,
`pystemd-0.15.3/{setup.py, pystemd/run.py, pystemd/systemd1/}`.

## Types

### python-dbus-clients/pure-python-blocking-client (jeepney)
- **Need it serves:** talk to the systemd user manager from short-lived Python tools without
  spawning `systemctl`, with deadlines, and without compiled dependencies.
- **Recognizable code shapes:** `subprocess.run(["systemctl", "--user", "show", "--property=…"])`
  plus `Key=Value` parsing; `systemd-run` invocations used only to create a scope.
- **Mechanisms and APIs:** `jeepney.io.blocking.open_dbus_connection(bus="SESSION",
  enable_fds=True)`; `DBusAddress`/`new_method_call` to build `org.freedesktop.systemd1.Manager`
  and `org.freedesktop.DBus.Properties.Get/GetAll` calls; `conn.send_and_get_reply(msg,
  timeout=…)`; `MatchRule` + `conn.filter(rule)` and `recv_until_filtered(queue, timeout=…)` for
  `JobRemoved`/`PropertiesChanged`; Unix fd passing (`'h'` type, `FileDescriptor`) when
  `enable_fds=True`. Pure Python, no third-party dependencies.
- **Obligations discharged:** wire protocol, auth, marshalling, timeouts, fd passing.
- **Residual caller obligations:** hand-written method signatures (no typed systemd binding);
  error-to-policy mapping.
- **Semantic conditions:** a blocking connection per process; connection setup costs one socket
  handshake (no interpreter spawn).
- **Version / build conditions:** 0.9.0; requires `DBUS_SESSION_BUS_ADDRESS` or the default
  `$XDG_RUNTIME_DIR/bus`.
- **Limits and negative results:** not stdlib; importing it in a hook that must be stdlib-only
  would violate that rule unless the module is vendored.
- **Evidence:** Interface-checked: installed 0.9.0 source.

### python-dbus-clients/asyncio-client (dbus-fast)
- **Mechanisms and APIs:** `dbus_fast.aio.MessageBus(bus_type=BusType.SESSION,
  negotiate_unix_fd=True).connect()`, introspection-driven proxies, signals; a GLib loop variant.
- **Semantic conditions / limits:** asyncio-first; optional Cython acceleration; heavier import
  than jeepney; a poor fit for synchronous CLI tools.
- **Evidence:** Interface-checked: dbus-fast 5.2.0 source (message bus modules).

### python-dbus-clients/libsystemd-binding (pystemd)
- **Mechanisms and APIs:** `pystemd.systemd1.Unit`/`Manager` objects, `pystemd.run(cmd,
  user_mode=True, wait=…, remain_after_exit=…, extra={…})` mirroring `systemd-run`.
- **Semantic conditions / limits:** Cython extension built against libsystemd
  (`pkg-config libsystemd`); a build-time system dependency.
- **Evidence:** Interface-checked: pystemd 0.15.3 `setup.py`, `run.py`.

## Current applicability

### [CBO-048](../../../library_leverage/codebase/developer-tooling.md#cbo-048) — Host workload admission and service supervision rebuilt above systemd (non-SurrealDB parts)
- **Types:** `python-dbus-clients/pure-python-blocking-client`, `python-dbus-clients/asyncio-client`,
  `python-dbus-clients/libsystemd-binding`
- **Realization:** jeepney 0.9.0 is already in the environment transitively (declaring it would
  be a new direct pin); dbus-fast and pystemd are new dependencies.
- **Conclusion:** jeepney fits with conditions for the non-cold-start tooling (admission, lifetime,
  supervisor, telemetry service) — it replaces `systemctl` spawns and text parsing with timed
  bus calls and `JobRemoved` waits; ruled out for stdlib-only cold-start hook paths unless
  vendored; dbus-fast is a poor fit (asyncio-first tools are not the shape here); pystemd adds a
  native build dependency for no capability jeepney lacks for these calls.
- **What it would discharge / what remains local:** process spawning per manager query, parsing of
  `show` output, polling for unit state; the ledger, lane capacity, identity, deadlines and
  release rules remain.
- **Prerequisites, configuration or extension:** a small local module with the handful of
  systemd method signatures used; error mapping to fail-closed outcomes; manager-unreachable
  fallback to kernel cgroup observation.
- **Semantic gaps against the CBO's preservation constraints:** "privileged clients cannot bypass
  admission" is unaffected (bus access is per-user); "drain observations stay reportable when the
  manager is unavailable" requires the cgroup-file path to stay.
- **Unsettled premises and what would settle them:** which modules are on the stdlib-only cold
  path (the hooks guide owns that list); whether replacing `systemd-run --scope` (which runs the
  workload as the scope's own process) with bus calls changes the process tree contract (probe P2).
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** The stdlib-only cold-start constraint covers more than
hooks: `pse_env.py` and `host_admission.py` run bootstrap admission under the system Python
fallback, so a non-stdlib client in admission would create a second route rather than replace one.
The comparative review retains the current paths, which already combine manager facts, kernel
cgroup evidence and the ledger. See the [comparative review](../../reviews/host-processes-and-durable-files.md#cbo-048).

### [CBO-020](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-020) — Primary-receiver supervision split between the Rust runtime and a Python script
- **Types:** `python-dbus-clients/pure-python-blocking-client`
- **Conclusion:** fits with conditions on the script side only; it does not remove the split
  (that requires choosing one owner; see `zbus-systemd.md`).
- **Evidence:** Interface-checked; Proposed.
