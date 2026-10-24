# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Cheap, immutable process telemetry selection; no SDK or scientific imports."""

from __future__ import annotations

import json
import math
import os
import uuid
from collections.abc import Mapping
from dataclasses import dataclass
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path
from urllib.parse import unquote, urlsplit, urlunsplit

_INSTANCE: tuple[int, str] | None = None


def process_incarnation() -> str:
    """A fork or exec is a new process; embedded SDKs share this incarnation."""
    global _INSTANCE
    pid = os.getpid()
    if _INSTANCE is None or _INSTANCE[0] != pid:
        _INSTANCE = (pid, str(uuid.uuid4()))
    return _INSTANCE[1]


def state_directory(environment: Mapping[str, str]) -> Path:
    if environment.get("PSE_TELEMETRY_STATE"):
        return Path(environment["PSE_TELEMETRY_STATE"])
    return Path(environment.get("XDG_STATE_HOME", str(Path.home() / ".local/state"))) / "pse/telemetry"


def _number(environment: Mapping[str, str], name: str, default: float, *, maximum: float = 60000) -> float:
    value = float(environment.get(name, str(default)))
    if not math.isfinite(value) or not 0 < value <= maximum:
        raise ValueError("invalid telemetry numeric setting: " + name)
    return value


def _integer(environment: Mapping[str, str], name: str, default: int, *, maximum: int = 60000) -> int:
    value = _number(environment, name, default, maximum=maximum)
    if not value.is_integer():
        raise ValueError("telemetry integer setting required: " + name)
    return int(value)


def _headers(value: str) -> tuple[tuple[str, str], ...]:
    result: dict[str, str] = {}
    for part in value.split(","):
        if not part.strip():
            continue
        name, separator, content = part.partition("=")
        name, content = unquote(name.strip()), unquote(content.strip())
        if not separator or not name or any(c in name + content for c in "\r\n"):
            raise ValueError("invalid telemetry header setting")
        result[name] = content
    if len(result) > 32 or sum(len(k) + len(v) for k, v in result.items()) > 8192:
        raise ValueError("telemetry headers exceed limit")
    return tuple(sorted(result.items()))


def _endpoint(value: str) -> str:
    parts = urlsplit(value)
    if parts.scheme not in {"http", "https"} or not parts.hostname or parts.username or parts.password or parts.query or parts.fragment:
        raise ValueError("telemetry endpoint must be an HTTP(S) URL without credentials, query or fragment")
    return value


@dataclass(frozen=True)
class SignalSettings:
    enabled: bool
    endpoint: str | None
    headers: tuple[tuple[str, str], ...]
    timeout_ms: int

    def native_settings(self) -> dict[str, object]:
        return {"enabled": self.enabled, "endpoint": self.endpoint, "headers": dict(self.headers), "timeout_ms": self.timeout_ms}

    def effective(self) -> dict[str, object]:
        endpoint = None
        if self.endpoint is not None:
            parts = urlsplit(self.endpoint)
            # Paths may contain tenant secrets; only the transport authority is public.
            endpoint = urlunsplit((parts.scheme, parts.netloc, "/<redacted-path>", "", ""))
        return {"enabled": self.enabled, "endpoint": endpoint, "header_names": [k for k, _ in self.headers], "timeout_ms": self.timeout_ms}


@dataclass(frozen=True)
class ResolvedSettings:
    disabled: bool
    selection: str
    traces: SignalSettings
    logs: SignalSettings
    metrics: SignalSettings
    resource: tuple[tuple[str, str], ...]
    max_queue_size: int
    max_export_batch_size: int
    schedule_delay_ms: int
    export_timeout_ms: int
    metric_export_interval_ms: int
    sampler: str
    sampler_argument: float
    configuration_id: str | None

    def native_settings(self) -> dict[str, object]:
        return {
            "schema_version": 1, "disabled": self.disabled,
            "traces": self.traces.native_settings(), "logs": self.logs.native_settings(), "metrics": self.metrics.native_settings(),
            "resource": dict(self.resource), "process_incarnation": dict(self.resource)["service.instance.id"],
            "max_queue_size": self.max_queue_size, "max_export_batch_size": self.max_export_batch_size,
            "schedule_delay_ms": self.schedule_delay_ms, "export_timeout_ms": self.export_timeout_ms,
            "metric_export_interval_ms": self.metric_export_interval_ms,
            "sampler": self.sampler, "sampler_argument": self.sampler_argument,
        }

    def effective(self) -> dict[str, object]:
        versions = {}
        for distribution in ("opentelemetry-api", "opentelemetry-sdk", "opentelemetry-exporter-otlp-proto-http", "opentelemetry-instrumentation-logging"):
            try:
                versions[distribution] = version(distribution)
            except PackageNotFoundError:
                versions[distribution] = None
        safe_resource = {key: value for key, value in self.resource if key in {"service.name", "service.version", "service.instance.id", "process.pid", "telemetry.sdk.language"}}
        return {
            "schema_version": 1, "disabled": self.disabled, "selection": self.selection,
            "signals": {"traces": self.traces.effective(), "logs": self.logs.effective(), "metrics": self.metrics.effective()},
            "transport": "http/protobuf", "resource": safe_resource, "sdk_versions": versions,
            "lifecycle_owner": "process host", "sampler": self.sampler, "sampler_argument": self.sampler_argument,
            "max_queue_size": self.max_queue_size, "max_export_batch_size": self.max_export_batch_size,
            "configuration_id": self.configuration_id, "delivery": "not_observed",
        }


def resolve_process(service_name: str, *, root: Path | None = None, environment: Mapping[str, str] | None = None) -> ResolvedSettings:
    """Resolve once: explicit OTEL signal settings override base and local selection."""
    del root  # The local service selection is user state, independent of a checkout.
    env = os.environ if environment is None else environment
    configuration: dict[str, object] = {}
    path = state_directory(env) / "configuration.json"
    if path.is_file():
        if path.stat().st_size > 65536:
            raise ValueError("telemetry configuration exceeds limit")
        loaded = json.loads(path.read_text())
        if not isinstance(loaded, dict) or loaded.get("schema_version") != 1:
            raise ValueError("unsupported telemetry configuration schema")
        configuration = loaded
    configured = configuration.get("otlp_http_endpoint")
    base = env.get("OTEL_EXPORTER_OTLP_ENDPOINT", configured if isinstance(configured, str) else "")
    selected = bool(base or any(env.get(f"OTEL_EXPORTER_OTLP_{name}_ENDPOINT") for name in ("TRACES", "LOGS", "METRICS")))
    disabled = env.get("OTEL_SDK_DISABLED", "false").lower() == "true" or not selected
    base_headers = env.get("OTEL_EXPORTER_OTLP_HEADERS", "")
    signals: dict[str, SignalSettings] = {}
    for name in ("TRACES", "LOGS", "METRICS"):
        exporter = env.get(f"OTEL_{name}_EXPORTER", "otlp")
        if exporter not in {"otlp", "none"}:
            raise ValueError("unsupported telemetry exporter: OTEL_" + name + "_EXPORTER")
        enabled = not disabled and exporter != "none"
        protocol = env.get(f"OTEL_EXPORTER_OTLP_{name}_PROTOCOL", env.get("OTEL_EXPORTER_OTLP_PROTOCOL", "http/protobuf"))
        if enabled and protocol != "http/protobuf":
            raise ValueError("selected telemetry transport requires http/protobuf")
        endpoint = env.get(f"OTEL_EXPORTER_OTLP_{name}_ENDPOINT")
        if endpoint is None and base:
            endpoint = base.rstrip("/") + "/v1/" + name.lower()
        timeout = _number(env, f"OTEL_EXPORTER_OTLP_{name}_TIMEOUT", _number(env, "OTEL_EXPORTER_OTLP_TIMEOUT", 5, maximum=60), maximum=60)
        signals[name] = SignalSettings(enabled and bool(endpoint), _endpoint(endpoint) if endpoint and enabled else None, _headers(env.get(f"OTEL_EXPORTER_OTLP_{name}_HEADERS", base_headers)) if enabled else (), max(1, int(timeout * 1000)))
    resource = dict(_headers(env.get("OTEL_RESOURCE_ATTRIBUTES", "")))
    try:
        package_version = version("pse-arrow")
    except PackageNotFoundError:
        package_version = "source"
    resource.update({"service.name": env.get("OTEL_SERVICE_NAME", service_name), "service.version": package_version, "service.instance.id": process_incarnation(), "process.pid": str(os.getpid())})
    queue = _integer(env, "OTEL_BSP_MAX_QUEUE_SIZE", 2048, maximum=65536)
    batch = _integer(env, "OTEL_BSP_MAX_EXPORT_BATCH_SIZE", 512, maximum=65536)
    if batch > queue:
        raise ValueError("telemetry export batch exceeds bounded queue")
    sampler = env.get("OTEL_TRACES_SAMPLER", "parentbased_always_on")
    if sampler not in {"always_on", "always_off", "parentbased_always_on", "parentbased_always_off", "traceidratio", "parentbased_traceidratio"}:
        raise ValueError("unsupported telemetry sampler")
    argument = float(env.get("OTEL_TRACES_SAMPLER_ARG", "1"))
    if not math.isfinite(argument) or not 0 <= argument <= 1:
        raise ValueError("invalid telemetry sampler argument")
    configuration_id = configuration.get("configuration_id")
    return ResolvedSettings(disabled, "disabled" if disabled else "environment" if any(k.startswith("OTEL_EXPORTER_OTLP") for k in env) else "configured_service", signals["TRACES"], signals["LOGS"], signals["METRICS"], tuple(sorted(resource.items())), queue, batch, _integer(env, "OTEL_BSP_SCHEDULE_DELAY", 5000), _integer(env, "OTEL_BSP_EXPORT_TIMEOUT", 5000), _integer(env, "OTEL_METRIC_EXPORT_INTERVAL", 60000, maximum=3600000), sampler, argument, configuration_id if isinstance(configuration_id, str) else None)
