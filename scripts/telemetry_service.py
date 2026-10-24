# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit native telemetry installation and user-systemd service ownership.

The installed binaries, immutable configurations and persistent diagnostic state
are independent of a checkout. Inspection never installs, starts or repairs a
service. Queue admission and retry are Collector responsibilities.
"""

from __future__ import annotations

import base64
import contextlib
import fcntl
import hashlib
import json
import os
import re
import secrets
import socket
import stat
import subprocess
import time
import uuid
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import diagnostic_trace

if TYPE_CHECKING:
    from collections.abc import Iterator, Mapping

OWNER = "pse-telemetry-v1"
DEFAULT_PORTS = {
    "otlp_http": 4318,
    "otlp_grpc": 4317,
    "backend_http": 5080,
    "backend_grpc": 5081,
    "metrics": 8888,
    "health": 13133,
}
DEFAULT_STREAMS = {"traces": "pse_traces", "metrics": "pse_metrics", "logs": "pse_logs"}
TOOLS = ("collector", "openobserve")
TOOL_DECLARATIONS = {"collector": "otelcol-contrib", "openobserve": "openobserve"}
STOP_SECONDS = 30
COLLECTOR_LIMITS = {
    "queue_requests_per_signal": 1000,
    "retry_elapsed_seconds": 300,
    "batch_items": 512,
    "batch_max_items": 1024,
    "ingress_body_bytes": 4 << 20,
    "stop_seconds": STOP_SECONDS,
}


class TelemetryServiceError(RuntimeError):
    """A credential-free, actionable service failure."""


def state_directory(
    state: Path | str | None = None, environment: Mapping[str, str] | None = None
) -> Path:
    env = os.environ if environment is None else environment
    home = Path(env.get("HOME", str(Path.home())))
    selected = state or env.get("PSE_TELEMETRY_STATE")
    path = Path(selected) if selected else Path(env.get("XDG_STATE_HOME") or home / ".local/state") / "pse/telemetry"
    return Path(os.path.abspath(path))


def _no_symlinks(path: Path) -> None:
    if any(item.is_symlink() for item in (path, *path.parents)):
        raise TelemetryServiceError("Telemetry paths must not traverse symlinks")


def _private_directory(path: Path, *, create: bool = False) -> None:
    _no_symlinks(path)
    if create:
        path.mkdir(parents=True, exist_ok=True, mode=0o700)
    info = path.stat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
        raise TelemetryServiceError("Telemetry state must be an owned private directory (0700)")


def _read(path: Path, *, private: bool = True) -> dict[str, object]:
    _no_symlinks(path)
    info = path.stat()
    if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or (private and info.st_mode & 0o077):
        raise TelemetryServiceError("Telemetry ownership record has unsafe permissions or owner")
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise TelemetryServiceError("Telemetry ownership record must be an object")
    return value


def _write(path: Path, value: str, mode: int = 0o600) -> None:
    _no_symlinks(path)
    temporary = path.with_name(f".{path.name}.{uuid.uuid4().hex}")
    try:
        descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
        with os.fdopen(descriptor, "w") as stream:
            stream.write(value)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def _write_json(path: Path, value: Mapping[str, object]) -> None:
    _write(path, json.dumps(value, sort_keys=True, indent=2) + "\n")


def _digest(path: Path) -> str:
    _no_symlinks(path)
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def _file_identity(path: Path) -> dict[str, int]:
    _no_symlinks(path)
    info = path.stat()
    if not stat.S_ISREG(info.st_mode):
        raise TelemetryServiceError("Telemetry executable is not a regular file")
    return {"device": info.st_dev, "inode": info.st_ino, "size": info.st_size, "mtime_ns": info.st_mtime_ns, "ctime_ns": info.st_ctime_ns, "mode": info.st_mode, "uid": info.st_uid}


@contextlib.contextmanager
def _lock(state: Path) -> Iterator[None]:
    _private_directory(state)
    lock = state / ".lifecycle.lock"
    _no_symlinks(lock)
    descriptor = os.open(lock, os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
    try:
        fcntl.flock(descriptor, fcntl.LOCK_EX)
        yield
    finally:
        os.close(descriptor)


def install(environment: Mapping[str, str] | None = None) -> dict[str, str]:
    """Explicitly install both pinned archives through the shared tool installer."""
    return {name: str(diagnostic_trace.install(TOOL_DECLARATIONS[name], environment)) for name in TOOLS}


def _ports(ports: Mapping[str, int] | None) -> dict[str, int]:
    if ports is not None and set(ports) - DEFAULT_PORTS.keys():
        raise TelemetryServiceError("Unknown telemetry listener selection")
    selected = DEFAULT_PORTS | dict(ports or {})
    if any(type(port) is not int or not 1 <= port <= 65535 for port in selected.values()) or len(set(selected.values())) != len(selected):
        raise TelemetryServiceError("Telemetry ports must be distinct integers from 1 through 65535")
    return selected


def collector_configuration(
    state: Path, ports: Mapping[str, int], organization: str, streams: Mapping[str, str],
    *, configuration_id: str | None = None,
) -> dict[str, object]:
    """JSON is also YAML; the installed Collector validates this exact declaration."""
    exporters = {}
    for signal, stream in streams.items():
        exporters[f"otlp_http/{signal}"] = {
            "endpoint": f"http://127.0.0.1:{ports['backend_http']}/api/{organization}",
            "headers": {"Authorization": "${env:PSE_TELEMETRY_AUTHORIZATION}", "stream-name": stream},
            "encoding": "json",
            "compression": "gzip",
            "timeout": "10s",
            "retry_on_failure": {"enabled": True, "initial_interval": "1s", "max_interval": "30s", "max_elapsed_time": f"{COLLECTOR_LIMITS['retry_elapsed_seconds']}s"},
            "sending_queue": {"enabled": True, "sizer": "requests", "queue_size": COLLECTOR_LIMITS["queue_requests_per_signal"], "num_consumers": 2, "block_on_overflow": False, "storage": "file_storage"},
        }
    return {
        "receivers": {"otlp": {"protocols": {"http": {"endpoint": f"127.0.0.1:{ports['otlp_http']}", "max_request_body_size": COLLECTOR_LIMITS["ingress_body_bytes"]}, "grpc": {"endpoint": f"127.0.0.1:{ports['otlp_grpc']}", "max_recv_msg_size_mib": COLLECTOR_LIMITS["ingress_body_bytes"] >> 20}}}},
        "processors": {"batch": {"timeout": "1s", "send_batch_size": COLLECTOR_LIMITS["batch_items"], "send_batch_max_size": COLLECTOR_LIMITS["batch_max_items"]}},
        "exporters": exporters,
        "extensions": {
            "file_storage": {"directory": str(state / "collector-queue"), "timeout": "10s", "fsync": True},
            "health_check": {"endpoint": f"127.0.0.1:{ports['health']}"},
        },
        "service": {
            "extensions": ["file_storage", "health_check"],
            "telemetry": {
                "resource": {"attributes": [{"name": "service.name", "value": "pse-telemetry-collector"}, *([{"name": "pse.configuration.id", "value": configuration_id}] if configuration_id else [])]},
                "logs": {"level": "info"},
                "metrics": {"level": "normal", "readers": [{"pull": {"exporter": {"prometheus": {"host": "127.0.0.1", "port": ports["metrics"]}}}}]},
            },
            "pipelines": {signal: {"receivers": ["otlp"], "processors": ["batch"], "exporters": [f"otlp_http/{signal}"]} for signal in streams},
        },
    }


def _unit_quote(value: str, *, execution: bool = False) -> str:
    if "\n" in value or "\r" in value or "\x00" in value:
        raise TelemetryServiceError("Invalid service path")
    value = value.replace("\\", "\\\\").replace('"', '\\"').replace("%", "%%")
    return '"' + (value.replace("$", "$$") if execution else value) + '"'


def _unit(
    name: str, binary: Path, generation: Path, state: Path, owner_id: str
) -> str:
    command = _unit_quote(str(binary), execution=True)
    if name == "collector":
        command += " --config=" + _unit_quote(str(generation / "collector.yaml"), execution=True)
    return (
        "[Unit]\n"
        f"Description=PSE telemetry {name} {owner_id}\n"
        "\n[Service]\nType=simple\n"
        # Path directives consume the complete value; literal quotes are a
        # path. ExecStart alone uses the command word parser.
        f"WorkingDirectory={str(state).replace('%', '%%')}\n"
        f"EnvironmentFile={str(state / 'service.env').replace('%', '%%')}\n"
        f"ExecStart={command}\n"
        "Restart=on-failure\nRestartSec=2s\n"
        f"TimeoutStopSec={STOP_SECONDS}s\n"
        "KillMode=control-group\nUMask=0077\nNoNewPrivileges=yes\n"
        "\n[Install]\nWantedBy=default.target\n"
    )


def setup(
    state: Path | str | None = None,
    *,
    environment: Mapping[str, str] | None = None,
    ports: Mapping[str, int] | None = None,
    organization: str = "default",
    streams: Mapping[str, str] | None = None,
    retention_days: int = 7,
    binaries: Mapping[str, Path | str] | None = None,
) -> dict[str, object]:
    """Publish an owned generation, without starting or enabling either service.

    An existing selection is never replaced implicitly. Reconfiguration/upgrades
    require a separately selected state pending explicit compatibility review.
    ``binaries`` supports isolated fixtures; ordinary setup verifies installed pins.
    """
    selected = state_directory(state, environment)
    selected_ports = _ports(ports)
    selected_streams = dict(DEFAULT_STREAMS if streams is None else streams)
    if set(selected_streams) != set(DEFAULT_STREAMS) or any(not re.fullmatch(r"[a-zA-Z_][a-zA-Z0-9_]{0,127}", item) for item in (organization, *selected_streams.values())):
        raise TelemetryServiceError("Organization and three signal streams require safe identifiers")
    if type(retention_days) is not int or retention_days < 3:
        raise TelemetryServiceError("OpenObserve retention must be at least three days")
    _private_directory(selected, create=True)
    with _lock(selected):
        if (selected / "service.json").exists():
            raise TelemetryServiceError("Telemetry state already configured; inspect or select a new state")
        if any(path.name != ".lifecycle.lock" for path in selected.iterdir()):
            raise TelemetryServiceError("Telemetry setup requires an empty owned state")
        tools = {name: Path(binaries[name]).absolute() if binaries is not None else diagnostic_trace.verify_binary(TOOL_DECLARATIONS[name], environment) for name in TOOLS}
        for path in tools.values():
            _no_symlinks(path)
            if not path.is_file() or not os.access(path, os.X_OK):
                raise TelemetryServiceError("Telemetry executable missing or not executable")
        owner_id = uuid.uuid4().hex
        generation_id = uuid.uuid4().hex
        generation = selected / "generations" / generation_id
        generation.mkdir(parents=True, mode=0o700)
        for directory in (selected / "collector-queue", selected / "openobserve-data"):
            directory.mkdir(mode=0o700)
        public: dict[str, object] = {
            "schema_version": 1,
            "otlp_http_endpoint": f"http://127.0.0.1:{selected_ports['otlp_http']}",
            "otlp_grpc_endpoint": f"http://127.0.0.1:{selected_ports['otlp_grpc']}",
            "backend_http_endpoint": f"http://127.0.0.1:{selected_ports['backend_http']}",
            "organization": organization, "streams": selected_streams,
            "configuration_id": generation_id,
        }
        # The pinned backend requires all four character classes. A random URL token
        # alone can omit one; retain its full entropy and guarantee the required prefix.
        email, password = "telemetry@localhost.local", "pSe9!" + secrets.token_urlsafe(32)
        authorization = "Basic " + base64.b64encode(f"{email}:{password}".encode()).decode()
        values = {
            "PSE_TELEMETRY_AUTHORIZATION": authorization,
            "ZO_ROOT_USER_EMAIL": email, "ZO_ROOT_USER_PASSWORD": password,
            "ZO_LOCAL_MODE": "true", "ZO_LOCAL_MODE_STORAGE": "disk", "ZO_META_STORE": "sqlite",
            "ZO_QUEUE_STORE": "memory", "ZO_DATA_DIR": str(selected / "openobserve-data") + "/",
            "ZO_HTTP_ADDR": "127.0.0.1", "ZO_HTTP_PORT": str(selected_ports["backend_http"]),
            "ZO_GRPC_ADDR": "127.0.0.1", "ZO_GRPC_PORT": str(selected_ports["backend_grpc"]),
            "ZO_WEB_URL": str(public["backend_http_endpoint"]), "ZO_TELEMETRY": "false",
            "ZO_COMPACT_DATA_RETENTION_DAYS": str(retention_days),
            "ZO_COMPACT_EXTENDED_DATA_RETENTION_DAYS": str(retention_days),
        }
        _write(selected / "service.env", "".join(f"{key}={json.dumps(value)}\n" for key, value in values.items()))
        _write_json(generation / "collector.yaml", collector_configuration(selected, selected_ports, organization, selected_streams, configuration_id=generation_id))
        validation = subprocess.run([str(tools["collector"]), "validate", "--config", str(generation / "collector.yaml")], env={**os.environ, **values}, capture_output=True, timeout=30, check=False)
        if validation.returncode:
            raise TelemetryServiceError("Collector rejected configuration; generation was not published")
        units = {}
        for name in TOOLS:
            unit = f"pse-telemetry-{owner_id}-{generation_id}-{name}.service"
            _write(generation / unit, _unit(name, tools[name], generation, selected, owner_id))
            units[name] = unit
        files = [generation / "collector.yaml", selected / "service.env", *(generation / unit for unit in units.values())]
        record: dict[str, object] = {
            "schema_version": 1, "owner": OWNER, "owner_id": owner_id,
            "state": str(selected), "configuration_id": generation_id,
            "generation": str(generation), "units": units, "ports": selected_ports,
            "binaries": {name: {"path": str(path), "sha256": _digest(path), "identity": _file_identity(path), "version": diagnostic_trace.tool_spec(TOOL_DECLARATIONS[name])["version"] if binaries is None else "fixture"} for name, path in tools.items()},
            "files": {str(path.relative_to(selected)): _digest(path) for path in files},
            "retention_days": retention_days,
            "public_configuration": public,
            "limits": COLLECTOR_LIMITS,
        }
        _write_json(selected / "service.json", record)
        # Presence is configured producer selection; publish it only after validation.
        _write_json(selected / "configuration.json", public)
    return inspect(selected, environment=environment)


def _configuration(state: Path) -> dict[str, object]:
    _private_directory(state)
    config = _read(state / "service.json")
    if config.get("owner") != OWNER or config.get("state") != str(state) or config.get("schema_version") != 1:
        raise TelemetryServiceError("Selected telemetry state ownership differs")
    owner_id, generation_id = config.get("owner_id"), config.get("configuration_id")
    if not all(isinstance(item, str) and re.fullmatch(r"[0-9a-f]{32}", item) for item in (owner_id, generation_id)):
        raise TelemetryServiceError("Telemetry generation identity is invalid")
    generation = state / "generations" / str(generation_id)
    units = {name: f"pse-telemetry-{owner_id}-{generation_id}-{name}.service" for name in TOOLS}
    if config.get("generation") != str(generation) or config.get("units") != units:
        raise TelemetryServiceError("Telemetry generation paths differ")
    files = config.get("files")
    required = {str(path.relative_to(state)) for path in (generation / "collector.yaml", state / "service.env", *(generation / unit for unit in units.values()))}
    if not isinstance(files, dict) or set(files) != required:
        raise TelemetryServiceError("Telemetry generation file inventory differs")
    for relative, expected in files.items():
        path = state / relative
        info = path.stat()
        if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
            raise TelemetryServiceError("Telemetry generation permissions or ownership changed")
        if _digest(path) != expected:
            raise TelemetryServiceError("Telemetry generation files changed")
    binaries = config.get("binaries")
    if not isinstance(binaries, dict) or set(binaries) != set(TOOLS):
        raise TelemetryServiceError("Telemetry binary inventory differs")
    for item in binaries.values():
        # Verify immutable installation identity without rereading hundreds of
        # megabytes on every passive health read. Setup authenticates bytes;
        # replacement, mutation or chmod changes this complete stat identity.
        if not isinstance(item, dict) or _file_identity(Path(str(item.get("path")))) != item.get("identity"):
            raise TelemetryServiceError("Telemetry binary changed")
    public = _read(state / "configuration.json")
    ports = config.get("ports")
    if not isinstance(ports, dict):
        raise TelemetryServiceError("Telemetry port declaration is invalid")
    _ports(ports)
    if public != config.get("public_configuration") or public.get("configuration_id") != generation_id or public.get("schema_version") != 1:
        raise TelemetryServiceError("Telemetry public configuration identity differs")
    return config


def backend_access(
    state: Path | str | None = None, *, environment: Mapping[str, str] | None = None
) -> tuple[str, dict[str, str]]:
    """Internal credential boundary for bounded backend requests; never expose it."""
    selected = state_directory(state, environment)
    config = _configuration(selected)
    values = dict(line.split("=", 1) for line in (selected / "service.env").read_text().splitlines() if "=" in line)
    authorization = json.loads(values["PSE_TELEMETRY_AUTHORIZATION"])
    if not isinstance(authorization, str) or not authorization.startswith("Basic "):
        raise TelemetryServiceError("Telemetry backend authorization is invalid")
    public = config["public_configuration"]
    assert isinstance(public, dict)
    return str(public["backend_http_endpoint"]), {"Authorization": authorization}


def _systemctl(*arguments: str, environment: Mapping[str, str] | None = None, check: bool = True) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ if environment is None else environment)
    runtime = Path(f"/run/user/{os.getuid()}")
    if (runtime / "bus").is_socket():
        env.setdefault("XDG_RUNTIME_DIR", str(runtime))
        env.setdefault("DBUS_SESSION_BUS_ADDRESS", f"unix:path={runtime}/bus")
    result = subprocess.run(["systemctl", "--user", *arguments], env=env, capture_output=True, text=True, timeout=STOP_SECONDS + 15, check=False)
    if check and result.returncode:
        unit = next((item for item in arguments if item.endswith(".service")), "selected telemetry units")
        operation = arguments[0] if arguments else "operation"
        raise TelemetryServiceError(f"User systemd {operation} failed for {unit}; inspect its user journal")
    return result


def _observe(unit: str, environment: Mapping[str, str] | None) -> dict[str, str]:
    result = _systemctl("show", "--property=LoadState,ActiveState,SubState,MainPID,InvocationID,ControlGroup,FragmentPath,Description,UnitFileState", unit, environment=environment, check=False)
    if result.returncode:
        return {"LoadState": "unavailable", "ActiveState": "unknown"}
    return dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)


def _unit_directory(environment: Mapping[str, str] | None) -> Path:
    env = os.environ if environment is None else environment
    home = Path(env.get("HOME", str(Path.home())))
    return Path(env.get("XDG_CONFIG_HOME") or home / ".config") / "systemd/user"


def _checked_process(config: Mapping[str, object], name: str, observed: Mapping[str, str]) -> dict[str, object] | None:
    """Bind current systemd invocation to executable, proc incarnation and cgroup."""
    if observed.get("LoadState") in {"not-found", "unavailable"}:
        if observed.get("LoadState") == "unavailable":
            raise TelemetryServiceError("User systemd observation unavailable")
        return None
    generation = Path(str(config["generation"]))
    units, binaries = config["units"], config["binaries"]
    if not isinstance(units, dict) or not isinstance(binaries, dict):
        raise TelemetryServiceError("Invalid service ownership record")
    expected_fragment = generation / str(units[name])
    fragment = Path(observed.get("FragmentPath", "/missing"))
    if observed.get("Description") != f"PSE telemetry {name} {config['owner_id']}" or not fragment.is_file() or _digest(fragment) != _digest(expected_fragment):
        raise TelemetryServiceError("Telemetry unit ownership differs")
    pid = int(observed.get("MainPID", "0"))
    if pid == 0:
        if observed.get("ActiveState") == "activating" and observed.get("SubState") in {"auto-restart", "auto-restart-queued"} and not observed.get("ControlGroup"):
            # The owned immutable unit is in its restart delay and has no child.
            # Stopping its timer does not confer authority over a stale PID.
            return None
        if observed.get("ActiveState") not in {"inactive", "failed"}:
            raise TelemetryServiceError("Telemetry process incarnation is unresolved")
        return None
    invocation, group = observed.get("InvocationID", ""), observed.get("ControlGroup", "")
    if not re.fullmatch(r"[0-9a-f]{32}", invocation) or not group.startswith("/") or group == "/" or ".." in Path(group).parts:
        raise TelemetryServiceError("Telemetry process systemd incarnation is invalid")
    proc = Path("/proc") / str(pid)
    try:
        start_ticks = int((proc / "stat").read_text().rsplit(")", 1)[1].split()[19])
        binary = binaries[name]
        executable = (proc / "exe").stat()
        if not isinstance(binary, dict) or str((proc / "exe").resolve()) != binary["path"] or f"0::{group}" not in (proc / "cgroup").read_text().splitlines():
            raise TelemetryServiceError("Telemetry process executable or cgroup differs")
        identity = binary.get("identity")
        if not isinstance(identity, dict) or (executable.st_dev, executable.st_ino) != (identity.get("device"), identity.get("inode")):
            raise TelemetryServiceError("Telemetry running executable installation differs")
    except (FileNotFoundError, ProcessLookupError) as error:
        raise TelemetryServiceError("Telemetry process changed during observation; retry inspection") from error
    return {"pid": pid, "start_ticks": start_ticks, "invocation_id": invocation, "control_group": group}


def inspect(state: Path | str | None = None, *, environment: Mapping[str, str] | None = None) -> dict[str, object]:
    """Read selected configuration and current process associations without effects."""
    selected = state_directory(state, environment)
    if not selected.exists():
        return {"state": str(selected), "status": "unconfigured", "services": {}}
    config = _configuration(selected)
    services = {}
    units = config["units"]
    assert isinstance(units, dict)
    for name, unit in units.items():
        observed = _observe(str(unit), environment)
        try:
            process = _checked_process(config, name, observed)
            ownership = "verified" if observed.get("LoadState") == "loaded" else "absent"
            problem = None
        except (TelemetryServiceError, OSError, ValueError) as error:
            process, ownership, problem = None, "unavailable", str(error)
        services[name] = {"unit": unit, "load_state": observed.get("LoadState", "unknown"), "active_state": observed.get("ActiveState", "unknown"), "sub_state": observed.get("SubState", "unknown"), "enabled": observed.get("UnitFileState", "unknown"), "ownership": ownership, "process": process, "problem": problem}
    pipeline = _read(Path(str(config["generation"])) / "collector.yaml")
    binaries = config["binaries"]
    assert isinstance(binaries, dict)
    versions: dict[str, str] = {}
    for name, item in binaries.items():
        if not isinstance(item, dict) or not isinstance(item.get("version"), str):
            raise TelemetryServiceError("Telemetry binary version record is invalid")
        versions[str(name)] = item["version"]
    components: dict[str, list[str]] = {}
    for kind in ("receivers", "processors", "exporters", "extensions"):
        declared = pipeline[kind]
        if not isinstance(declared, dict):
            raise TelemetryServiceError("Telemetry component declaration is invalid")
        components[kind] = sorted(str(name) for name in declared)
    return {
        "state": str(selected), "status": "configured", "configuration": _read(selected / "configuration.json"),
        "owner_id": config["owner_id"], "configuration_id": config["configuration_id"], "generation": config["generation"],
        "retention_days": config["retention_days"], "limits": config["limits"], "services": services,
        "endpoints": {"collector_health": f"http://127.0.0.1:{config['ports']['health']}/", "collector_metrics": f"http://127.0.0.1:{config['ports']['metrics']}/metrics"},
        "versions": versions, "components": components,
        "storage": {"collector_queue": str(selected / "collector-queue"), "backend": str(selected / "openobserve-data")},
        "declaration_sources": {"tools": ".config/diagnostic-tools.toml", "service": "scripts/telemetry_service.py", "collector": str(Path(str(config["generation"])) / "collector.yaml")},
    }


def _publish_units(config: Mapping[str, object], environment: Mapping[str, str] | None) -> None:
    directory = _unit_directory(environment)
    _no_symlinks(directory)
    directory.mkdir(parents=True, exist_ok=True)
    units = config["units"]
    assert isinstance(units, dict)
    for unit in units.values():
        target, source = directory / str(unit), Path(str(config["generation"])) / str(unit)
        _no_symlinks(target)
        if target.exists() and _digest(target) != _digest(source):
            raise TelemetryServiceError("A different telemetry unit already occupies the selected identity")
        if not target.exists():
            _write(target, source.read_text())
    _systemctl("daemon-reload", environment=environment)


def _start(config: Mapping[str, object], environment: Mapping[str, str] | None) -> None:
    units, ports = config["units"], config["ports"]
    assert isinstance(units, dict) and isinstance(ports, dict)
    observations = {name: _observe(str(unit), environment) for name, unit in units.items()}
    processes = {name: _checked_process(config, name, observed) for name, observed in observations.items()}
    # Refuse collisions before starting either child. Existing exact children own
    # their listeners and are not probed as if they were foreign processes.
    for key, port in ports.items():
        name = "openobserve" if key.startswith("backend_") else "collector"
        if processes[name] is None:
            try:
                with socket.socket() as listener:
                    listener.bind(("127.0.0.1", int(port)))
            except OSError as error:
                raise TelemetryServiceError(f"Telemetry listener collision on 127.0.0.1:{port}") from error
    for name in ("openobserve", "collector"):
        if processes[name] is None:
            _systemctl("start", str(units[name]), environment=environment)
            deadline = time.monotonic() + 10
            while True:
                observed = _observe(str(units[name]), environment)
                process = _checked_process(config, name, observed)
                if process is not None and observed.get("ActiveState") == "active":
                    break
                if observed.get("ActiveState") == "failed" or time.monotonic() >= deadline:
                    raise TelemetryServiceError(f"Telemetry {name} did not start; inspect its user journal")
                time.sleep(0.05)


def start(state: Path | str | None = None, *, environment: Mapping[str, str] | None = None) -> dict[str, object]:
    selected = state_directory(state, environment)
    with _lock(selected):
        config = _configuration(selected)
        _publish_units(config, environment)
        _start(config, environment)
    return inspect(selected, environment=environment)


def _stop(config: Mapping[str, object], environment: Mapping[str, str] | None) -> None:
    units = config["units"]
    assert isinstance(units, dict)
    # Verify *both* current owners before stopping either. A stale PID or unit
    # name alone is never authority to stop a process.
    observations = {name: _observe(str(unit), environment) for name, unit in units.items()}
    processes = {name: _checked_process(config, name, observed) for name, observed in observations.items()}
    for name in ("collector", "openobserve"):
        if observations[name].get("LoadState") == "not-found":
            continue
        current = _observe(str(units[name]), environment)
        process = _checked_process(config, name, current)
        if process != processes[name] or current.get("InvocationID") != observations[name].get("InvocationID") or current.get("MainPID") != observations[name].get("MainPID"):
            raise TelemetryServiceError("Telemetry incarnation changed before stop; retry with current observation")
        if process is None and current.get("ActiveState") == "inactive" and not current.get("ControlGroup"):
            continue
        _systemctl("stop", str(units[name]), environment=environment)
        after = _observe(str(units[name]), environment)
        if after.get("MainPID", "0") != "0" or after.get("ActiveState") not in {"inactive", "failed"}:
            raise TelemetryServiceError("Telemetry stop did not establish a drained service")
        group = observations[name].get("ControlGroup", "")
        if group:
            events = Path("/sys/fs/cgroup") / group.lstrip("/") / "cgroup.events"
            if events.exists() and "populated 0" not in events.read_text().splitlines():
                raise TelemetryServiceError("Telemetry children remain in the owned cgroup after stop")


def stop(state: Path | str | None = None, *, environment: Mapping[str, str] | None = None) -> dict[str, object]:
    selected = state_directory(state, environment)
    with _lock(selected):
        _stop(_configuration(selected), environment)
    return inspect(selected, environment=environment)


def restart(state: Path | str | None = None, *, environment: Mapping[str, str] | None = None) -> dict[str, object]:
    selected = state_directory(state, environment)
    with _lock(selected):
        config = _configuration(selected)
        _stop(config, environment)
        _publish_units(config, environment)
        _start(config, environment)
    return inspect(selected, environment=environment)


def enable(state: Path | str | None = None, *, environment: Mapping[str, str] | None = None) -> dict[str, object]:
    """Explicitly select start at user login; does not start services now."""
    selected = state_directory(state, environment)
    with _lock(selected):
        config = _configuration(selected)
        _publish_units(config, environment)
        units = config["units"]
        assert isinstance(units, dict)
        _systemctl("enable", *(str(unit) for unit in units.values()), environment=environment)
    return inspect(selected, environment=environment)


def disable(state: Path | str | None = None, *, environment: Mapping[str, str] | None = None) -> dict[str, object]:
    """Explicitly remove login activation; current serving lifetime is unchanged."""
    selected = state_directory(state, environment)
    with _lock(selected):
        config = _configuration(selected)
        units = config["units"]
        assert isinstance(units, dict)
        for name, unit in units.items():
            observed = _observe(str(unit), environment)
            _checked_process(config, name, observed)
        _systemctl("disable", *(str(unit) for unit in units.values()), environment=environment)
    return inspect(selected, environment=environment)
