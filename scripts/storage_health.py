# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Original, bounded host storage observations; inspection confers no action authority."""

from __future__ import annotations

import hashlib
import json
import math
import os
import socket
import subprocess
import time
from pathlib import Path
from typing import Literal, TypedDict

import msgspec

from scripts import host_admission, native_operation
from scripts import surreal_server as server

SCHEMA = "storage-health-v1"
MAXIMUM_BYTES = 1 << 20
MAXIMUM_PROCESSES = 128
HOST_FIELDS = ("hostname", "MemTotal_bytes", "MemAvailable_bytes", "SwapTotal_bytes", "SwapFree_bytes", "filesystem_path", "filesystem_available_bytes", "filesystem_total_bytes")
CONFIGURATION_FIELDS = ("surrealmx_memory_limit_bytes", "surrealmx_max_transaction_age_seconds", "surrealmx_snapshot_interval_seconds", "surrealmx_max_async_queue_depth", "surrealmx_max_async_queue_bytes", "configured_endpoint", "pending_maintenance", "accepting_writes")
CGROUP_FIELDS = ("memory.current", "memory.max", "memory.peak", "memory.swap.current")
LIMITS = (
    "Shared service observations establish no operation attribution or causality.",
    "Approximate engine data bytes are not RSS, disk allocation or reclaimed bytes.",
    "Zero async queue depth does not establish synchronous durability or snapshot completion.",
    "Reader counts and pinned versions do not identify a stale transaction or its owner; absent oldest pin does not establish reader state.",
    "Configured persistence is not evidence of a successful flush, fsync or recovery.",
    "Filesystem availability is not application allocation or reclaimable space.",
    "An inspection or copied snapshot grants no protection, disposal or recovery authority.",
)


class Observation(msgspec.Struct, forbid_unknown_fields=True):
    status: Literal["available", "approximate", "unavailable"]
    value: int | str | bool | None = None
    reason: str | None = None

    def __post_init__(self) -> None:
        if (self.status != "unavailable") != (self.value is not None):
            raise ValueError("observation availability differs from its value")
        if self.status == "unavailable" and not self.reason:
            raise ValueError("unavailable observation lacks its reason")
        if type(self.value) is int and self.value < 0:
            raise ValueError("negative host/configuration observation")


class ProcessIdentity(msgspec.Struct, forbid_unknown_fields=True):
    boot: str
    pid: int
    start: str

    def __post_init__(self) -> None:
        if self.pid <= 0 or not self.boot or not self.start.isdecimal():
            raise ValueError("invalid original process identity")


class ServiceIdentity(msgspec.Struct, forbid_unknown_fields=True):
    state: str
    instance_id: str
    invocation: str
    port: int
    binary_sha256: str

    def __post_init__(self) -> None:
        if (
            not Path(self.state).is_absolute()
            or not self.instance_id
            or not 1 <= self.port <= 65535
            or len(self.invocation) != 32
            or len(self.binary_sha256) != 64
            or any(c not in "0123456789abcdef" for c in self.invocation + self.binary_sha256)
        ):
            raise ValueError("invalid selected service identity")


class Service(msgspec.Struct, forbid_unknown_fields=True):
    status: Literal["available", "unavailable"]
    identity: ServiceIdentity | None = None
    reason: str | None = None

    def __post_init__(self) -> None:
        if (self.status == "available") != (self.identity is not None):
            raise ValueError("service availability differs from original identity")
        if self.status == "unavailable" and not self.reason:
            raise ValueError("unavailable service lacks its reason")


class MetricSample(msgspec.Struct, forbid_unknown_fields=True):
    labels: str
    value: int

    def __post_init__(self) -> None:
        if self.value < 0:
            raise ValueError("MX gauges must be nonnegative")


class Metric(msgspec.Struct, forbid_unknown_fields=True):
    status: Literal["observed", "approximate", "unavailable"]
    samples: list[MetricSample]
    reason: str | None = None

    def __post_init__(self) -> None:
        if (self.status != "unavailable") != bool(self.samples):
            raise ValueError("metric availability differs from its original samples")
        if self.status == "unavailable" and not self.reason:
            raise ValueError("unavailable metric lacks its reason")


class ProcessObservation(msgspec.Struct, forbid_unknown_fields=True):
    identity: ProcessIdentity
    rss_bytes: Observation


class CgroupIdentity(msgspec.Struct, forbid_unknown_fields=True):
    boot: str
    group: str
    inode: int
    invocation: str

    def __post_init__(self) -> None:
        if (
            not self.boot or self.inode <= 0 or not self.group.startswith("/")
            or self.group == "/" or ".." in Path(self.group).parts
        ):
            raise ValueError("invalid selected cgroup identity")


class CgroupObservation(msgspec.Struct, forbid_unknown_fields=True):
    status: Literal["observed", "partial", "unavailable"]
    identity: CgroupIdentity | None
    fields: dict[str, Observation]
    processes: list[ProcessObservation]
    reason: str | None = None


class HealthSnapshot(msgspec.Struct, forbid_unknown_fields=True):
    schema: Literal["storage-health-v1"]
    requested_state: str
    started_unix_ns: int
    finished_unix_ns: int
    observer: ProcessIdentity | None
    host: dict[str, Observation]
    service: Service
    configuration: dict[str, Observation]
    metrics: dict[str, Metric]
    metric_started_unix_ns: int | None
    metric_finished_unix_ns: int | None
    cgroup: CgroupObservation
    tool_outcome: Literal["complete", "partial"]
    service_outcome: Literal["observed", "unavailable"]
    scientific_outcome: Literal["unknown"]
    correspondence: Literal["selected-service-cgroup", "unknown"]

    def __post_init__(self) -> None:
        if not Path(self.requested_state).is_absolute() or not (
            0 < self.started_unix_ns <= self.finished_unix_ns
        ):
            raise ValueError("invalid health capture context")
        if set(self.host) != set(HOST_FIELDS) or set(self.configuration) != set(CONFIGURATION_FIELDS) or set(self.cgroup.fields) != set(CGROUP_FIELDS):
            raise ValueError("health snapshot lacks declared host/configuration/cgroup coverage")
        if len(self.cgroup.processes) > MAXIMUM_PROCESSES or len({p.identity.pid for p in self.cgroup.processes}) != len(self.cgroup.processes):
            raise ValueError("health process population is duplicated or unbounded")
        if set(self.metrics) != set(server.MX_METRIC_FIELDS):
            raise ValueError("health snapshot lacks declared MX field coverage")
        for name, field in self.metrics.items():
            if field.status != "unavailable" and field.status != (
                "approximate" if name == "approx_data_bytes" else "observed"
            ):
                raise ValueError("health metric has incorrect precision")
        identity = self.service.identity
        if identity is None:
            if self.service_outcome != "unavailable" or any(
                field.status != "unavailable" for field in self.metrics.values()
            ) or self.metric_started_unix_ns is not None or self.metric_finished_unix_ns is not None:
                raise ValueError("unavailable service cannot authenticate engine observations")
        elif (
            identity.state != self.requested_state or self.service_outcome != "observed"
        ):
            raise ValueError("health identity differs from selected service capture")
        if identity is not None:
            first, last = self.metric_started_unix_ns, self.metric_finished_unix_ns
            if first is None and last is None:
                if any(field.status != "unavailable" for field in self.metrics.values()):
                    raise ValueError("health metrics lack original capture interval")
            elif first is None or last is None or not self.started_unix_ns <= first <= last <= self.finished_unix_ns:
                raise ValueError("health metrics differ from selected service capture")
        group = self.cgroup.identity
        if group is None:
            if self.correspondence != "unknown" or self.cgroup.processes or self.cgroup.status != "unavailable":
                raise ValueError("unavailable cgroup cannot establish process correspondence")
        elif (
            identity is None or group.invocation != identity.invocation
            or self.correspondence != "selected-service-cgroup"
            or self.cgroup.status == "unavailable"
            or (self.observer is not None and self.observer.boot != group.boot)
            or any(p.identity.boot != group.boot for p in self.cgroup.processes)
        ):
            raise ValueError("health cgroup differs from selected service incarnation")
        if self.tool_outcome == "complete" and (
            identity is None or self.observer is None or self.cgroup.status != "observed"
            or self.metric_started_unix_ns is None
            or any(f.status == "unavailable" for f in (*self.host.values(), *self.configuration.values()))
        ):
            raise ValueError("partial health capture cannot claim complete tool outcome")


def unavailable(reason: str) -> Observation:
    return Observation("unavailable", reason=reason)


def numeric_file(path: Path) -> Observation:
    try:
        raw = path.read_text().strip()
        if raw == "max":
            return Observation("available", "max")
        if not raw.isdecimal():
            return unavailable("kernel observation unavailable")
        return Observation("available", int(raw))
    except (OSError, ValueError):
        return unavailable("kernel observation unavailable")


def process_identity(pid: int) -> ProcessIdentity:
    return ProcessIdentity(host_admission.boot(), pid, native_operation.start_identity(pid))


def independent_host(state: Path) -> dict[str, Observation]:
    """Inspect kernel metadata and statvfs only; never open or enumerate engine files."""
    try:
        hostname = Observation("available", socket.gethostname())
    except OSError:
        hostname = unavailable("hostname unavailable")
    fields = {"hostname": hostname}
    for name in ("MemTotal", "MemAvailable", "SwapTotal", "SwapFree"):
        fields[name + "_bytes"] = unavailable("meminfo field unavailable")
    try:
        for line in Path("/proc/meminfo").read_text().splitlines():
            name, _, raw = line.partition(":")
            if name + "_bytes" in fields:
                parts = raw.split()
                if len(parts) == 2 and parts[0].isdecimal() and parts[1] == "kB":
                    fields[name + "_bytes"] = Observation("approximate" if name == "MemAvailable" else "available", int(parts[0]) * 1024)
    except OSError:
        pass
    # A missing service still has an independent host filesystem observation.
    path = state
    while not path.exists() and path != path.parent:
        path = path.parent
    fields["filesystem_path"] = Observation("available", str(path))
    try:
        usage = os.statvfs(path)
        fields["filesystem_available_bytes"] = Observation("available", usage.f_bavail * usage.f_frsize)
        fields["filesystem_total_bytes"] = Observation("available", usage.f_blocks * usage.f_frsize)
    except OSError:
        for name in ("filesystem_available_bytes", "filesystem_total_bytes"):
            fields[name] = unavailable("statvfs unavailable")
    return fields


def selected_process(pid: int, group: str, boot: str) -> ProcessObservation:
    """Read one original member only while its identity and membership remain stable."""
    process = process_identity(pid)
    if process.boot != boot or native_operation.process_group(pid) != group:
        raise ValueError("process membership changed")
    rss = unavailable("process RSS unavailable")
    for line in Path(f"/proc/{pid}/status").read_text().splitlines():
        if line.startswith("VmRSS:"):
            parts = line.split()
            if len(parts) == 3 and parts[1].isdecimal() and parts[2] == "kB":
                rss = Observation("approximate", int(parts[1]) * 1024, "kernel VmRSS estimate")
    if process_identity(pid) != process or native_operation.process_group(pid) != group:
        raise ValueError("process lifetime changed")
    return ProcessObservation(process, rss)


def observe_cgroup(state: Path, identity: ServiceIdentity, deadline: float) -> CgroupObservation:
    before = server.storage_unit_observation(state, deadline=deadline)
    if before.get("InvocationID") != identity.invocation:
        raise ValueError("service incarnation changed")
    group = before.get("ControlGroup", "")
    inode = host_admission.group_identity(group)
    if inode is None:
        raise ValueError("cgroup unavailable")
    boot = host_admission.boot()
    selected = CgroupIdentity(boot, group, inode, identity.invocation)
    directory = Path("/sys/fs/cgroup") / group.lstrip("/")
    fields = {name: numeric_file(directory / name) for name in CGROUP_FIELDS}
    processes = []
    partial = any(field.status == "unavailable" for field in fields.values())
    try:
        pids = (directory / "cgroup.procs").read_text().split()
        if len(pids) > MAXIMUM_PROCESSES:
            partial = True
        for raw in pids[:MAXIMUM_PROCESSES]:
            server.remaining(deadline)
            try:
                process = selected_process(int(raw), group, boot)
                processes.append(process)
                partial |= process.rss_bytes.status == "unavailable"
            except (OSError, ValueError):
                partial = True
    except OSError:
        partial = True
    after = server.storage_unit_observation(state, deadline=deadline)
    if after != before or host_admission.group_identity(group) != inode or host_admission.boot() != boot:
        raise ValueError("cgroup lifetime changed")
    return CgroupObservation("partial" if partial else "observed", selected, fields, processes, "some kernel fields or process members unavailable or bounded" if partial else None)


def selected_cgroup(state: Path, identity: ServiceIdentity, deadline: float) -> CgroupObservation:
    """Link only members of the observed incarnation, rechecking kernel lifetimes."""
    try:
        return observe_cgroup(state, identity, deadline)
    except (OSError, ValueError, subprocess.SubprocessError, server.SupervisorError, host_admission.AdmissionError):
        return CgroupObservation("unavailable", None, {name: unavailable("selected cgroup unavailable") for name in CGROUP_FIELDS}, [], "selected service/kernel correspondence unavailable")


def metric_fields(snapshot: dict[str, object]) -> dict[str, Metric]:
    families = server.object_mapping(snapshot["families"])
    fields = {}
    for name in server.MX_METRIC_FIELDS:
        source = server.object_mapping(families.get(server.MX_METRIC_PREFIX + name, {}))
        samples = []
        population = server.object_inventory(source.get("samples", []))
        if len(population) > 64 or sum(len(str(sample.get("labels", ""))) for sample in population) > 64 << 10:
            fields[name] = Metric("unavailable", [], "original series exceeds bounded health selection")
            continue
        for sample in population:
            value = sample.get("value")
            labels = sample.get("labels")
            if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0 or int(value) != value or not isinstance(labels, str):
                raise ValueError("invalid original MX sample")
            samples.append(MetricSample(labels, int(value)))
        fields[name] = Metric(
            ("approximate" if name == "approx_data_bytes" else "observed") if samples else "unavailable",
            samples,
            None if samples else str(source.get("availability", "unregistered")),
        )
    return fields


def selected_metrics(
    state: Path, config: dict[str, object] | None, deadline: float, started: int,
) -> tuple[ServiceIdentity, dict[str, Metric], int, int]:
    """Validate original selected-service authority and its capture interval."""
    original = server.metric_snapshot(state, deadline)
    if original.get("schema") != "selected-native-metrics-v1":
        raise ValueError("unsupported original metric codec")
    identity = msgspec.convert(original["identity"], ServiceIdentity, strict=True)
    if identity.state != str(state) or (config is not None and identity.instance_id != config["instance_id"]):
        raise ValueError("metrics differ from requested service authority")
    decoded = metric_fields(original)
    first, last = original["started_unix_ns"], original["finished_unix_ns"]
    if type(first) is not int or type(last) is not int or not started <= first <= last <= time.time_ns():
        raise ValueError("metrics lack original capture interval")
    if config is not None and config != server.config_for(state, deadline=deadline):
        raise ValueError("owner configuration changed during capture")
    return identity, decoded, first, last


def capture(state: Path, deadline: float) -> HealthSnapshot:
    """Collect available independent host facts even when selected service is unavailable."""
    state = server.lexical_absolute(state)
    started = time.time_ns()
    host = independent_host(state)
    try:
        observer = process_identity(os.getpid())
    except (OSError, ValueError):
        observer = None
    configuration = {name: unavailable("selected owner configuration unavailable") for name in CONFIGURATION_FIELDS}
    config = None
    try:
        config = server.config_for(state, deadline=deadline)
        resources = server.object_mapping(config["resources"])
        for name in CONFIGURATION_FIELDS[:5]:
            value = resources.get(name)
            configuration[name] = Observation("available", value) if type(value) is int else unavailable("owner configuration field unavailable")
        configuration["configured_endpoint"] = Observation("available", server.database_endpoint(state))
        pending = config.get("derived_rebuild_pending")
        configuration["pending_maintenance"] = Observation("available", f"{pending.get('kind')}:{pending.get('phase')}") if isinstance(pending, dict) else Observation("available", "none")
        accepting_writes = config["accepting_writes"]
        configuration["accepting_writes"] = Observation("available", accepting_writes) if isinstance(accepting_writes, bool) else unavailable("owner admission field unavailable")
    except (OSError, ValueError, KeyError, subprocess.SubprocessError, server.SupervisorError, host_admission.AdmissionError):
        configuration = {name: unavailable("selected owner configuration unavailable") for name in CONFIGURATION_FIELDS}
    metrics = {name: Metric("unavailable", [], "selected service unavailable") for name in server.MX_METRIC_FIELDS}
    service = Service("unavailable", reason="selected service or metric observation unavailable")
    group = CgroupObservation("unavailable", None, {name: unavailable("selected cgroup unavailable") for name in CGROUP_FIELDS}, [], "selected service unavailable")
    metric_started = metric_finished = None
    try:
        selected = server.selected_service(state, deadline)
        identity = msgspec.convert(selected.identity, ServiceIdentity, strict=True)
        if identity.state != str(state) or (config is not None and identity.instance_id != config["instance_id"]):
            service = Service("unavailable", reason="verified service differs from requested authority")
        else:
            service = Service("available", identity)
            try:
                observed, decoded, first, last = selected_metrics(state, config, deadline, started)
                if observed == identity:
                    metrics, metric_started, metric_finished = decoded, first, last
                else:
                    metrics = {name: Metric("unavailable", [], "metrics differ from verified service incarnation") for name in server.MX_METRIC_FIELDS}
            except (OSError, ValueError, KeyError, subprocess.SubprocessError, server.SupervisorError, host_admission.AdmissionError) as error:
                metrics = {name: Metric("unavailable", [], str(error)[:1024] or type(error).__name__) for name in server.MX_METRIC_FIELDS}
            # A failed metric response proves neither service death nor continuity.
            # Recheck the independent original owner before retaining its identity.
            selected.recheck(deadline)
            group = selected_cgroup(state, identity, deadline)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError, server.SupervisorError, host_admission.AdmissionError):
        service = Service("unavailable", reason="selected service identity or continuity unavailable")
        metrics = {name: Metric("unavailable", [], "selected service identity or continuity unavailable") for name in server.MX_METRIC_FIELDS}
        metric_started = metric_finished = None
    partial = service.status == "unavailable" or observer is None or group.status != "observed" or metric_started is None or any(f.status == "unavailable" for f in (*host.values(), *configuration.values()))
    return HealthSnapshot(
        SCHEMA, str(state), started, time.time_ns(), observer, host, service,
        configuration, metrics, metric_started, metric_finished, group,
        "partial" if partial else "complete",
        "observed" if service.status == "available" else "unavailable", "unknown",
        "selected-service-cgroup" if group.identity is not None else "unknown",
    )


def read(path: Path) -> HealthSnapshot:
    with path.open("rb") as stream:
        raw = stream.read(MAXIMUM_BYTES + 1)
    if len(raw) > MAXIMUM_BYTES:
        raise ValueError("storage health snapshot exceeds its bound")
    return msgspec.json.decode(raw, type=HealthSnapshot, strict=True)


class OwnerAction(TypedDict):
    action: Literal["inspect-service", "recover-maintenance"]
    command: list[str]
    effect: str
    preconditions: str


class HealthPresentation(TypedDict):
    snapshot: dict[str, object]
    reference: str
    limits: list[str]
    owner_actions: list[OwnerAction]


def owner_actions(snapshot: HealthSnapshot) -> list[OwnerAction]:
    """Inert finite command suggestions: the original owner rechecks every action."""
    base = ["just", "surreal"]
    state = ["--state", snapshot.requested_state]
    actions: list[OwnerAction] = [{
        "action": "inspect-service", "command": [*base, "status", *state],
        "effect": "inspect current owner status",
        "preconditions": "current owned configuration; snapshot is descriptive only",
    }]
    pending = snapshot.configuration.get("pending_maintenance")
    if pending is not None and pending.status == "available" and pending.value != "none":
        actions.append({
            "action": "recover-maintenance", "command": [*base, "recover-maintenance", *state],
            "effect": "explicit recovery through original maintenance owner and exact effect settlement",
            "preconditions": "owner reacquires lifecycle reservation, verifies current closed admission, exact pending kind/phase, drained processes and required initializer/verifier; changed or unsupported premises refuse",
        })
    return actions


def reference(snapshot: HealthSnapshot) -> str:
    """The original owner's immutable content reference, never a live protection grant."""
    raw = json.dumps(msgspec.to_builtins(snapshot), sort_keys=True, separators=(",", ":"), allow_nan=False).encode()
    return "storage-health:sha256:" + hashlib.sha256(raw).hexdigest()


def presentation(snapshot: HealthSnapshot) -> HealthPresentation:
    return {"snapshot": msgspec.to_builtins(snapshot), "reference": reference(snapshot), "limits": list(LIMITS), "owner_actions": owner_actions(snapshot)}
