# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Cheap, immutable process telemetry selection; no SDK or scientific imports."""

from __future__ import annotations

import json
import math
import os
import uuid
from dataclasses import dataclass, field
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path
from typing import TYPE_CHECKING
from urllib.parse import unquote, urlsplit, urlunsplit

import msgspec

from scripts.generated.telemetry_settings import Settings, SignalSettings

if TYPE_CHECKING:
    from collections.abc import Mapping


@dataclass
class _IdentityState:
    incarnation: tuple[int, str] | None = None


_IDENTITY = _IdentityState()


def process_incarnation() -> str:
    """A fork or exec is a new process; embedded SDKs share this incarnation."""
    pid = os.getpid()
    if _IDENTITY.incarnation is None or _IDENTITY.incarnation[0] != pid:
        _IDENTITY.incarnation = (pid, str(uuid.uuid4()))
    return _IDENTITY.incarnation[1]


def state_directory(environment: Mapping[str, str]) -> Path:
    if environment.get("PSE_TELEMETRY_STATE"):
        return Path(environment["PSE_TELEMETRY_STATE"])
    return (
        Path(environment.get("XDG_STATE_HOME", str(Path.home() / ".local/state")))
        / "pse/telemetry"
    )


def _number(
    environment: Mapping[str, str], name: str, default: float, *, maximum: float = 60000
) -> float:
    try:
        value = float(environment.get(name, str(default)))
    except ValueError:
        raise ValueError("invalid telemetry numeric setting: " + name) from None
    if not math.isfinite(value) or not 0 < value <= maximum:
        raise ValueError("invalid telemetry numeric setting: " + name)
    return value


def _integer(
    environment: Mapping[str, str], name: str, default: int, *, maximum: int = 60000
) -> int:
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
    try:
        parts = urlsplit(value)
        # Accessing port also rejects malformed and out-of-range transport ports.
        _ = parts.port
    except ValueError:
        raise ValueError("invalid telemetry endpoint") from None
    if (
        parts.scheme not in {"http", "https"}
        or not parts.hostname
        or parts.username
        or parts.password
        or parts.query
        or parts.fragment
    ):
        raise ValueError(
            "telemetry endpoint must be an HTTP(S) URL without credentials, query or fragment"
        )
    return value


def _effective_signal(signal: SignalSettings) -> dict[str, object]:
    endpoint = None
    if signal.endpoint is not None:
        parts = urlsplit(signal.endpoint)
        # Paths may contain tenant secrets; only the transport authority is public.
        endpoint = urlunsplit((parts.scheme, parts.netloc, "/<redacted-path>", "", ""))
    return {
        "enabled": signal.enabled,
        "endpoint": endpoint,
        "header_names": sorted(signal.headers),
        "timeout_ms": signal.timeout_ms,
    }


@dataclass(frozen=True)
class ResolvedSettings:
    """Private generated wire settings plus host-only selection provenance."""

    _document: str = field(repr=False)
    selection: str
    configuration_id: str | None

    @classmethod
    def from_wire(
        cls, *, wire: Settings, selection: str, configuration_id: str | None
    ) -> ResolvedSettings:
        """Freeze the generated wire once, without retaining mutable maps."""
        try:
            document = msgspec.json.encode(wire)
        except (TypeError, ValueError):
            raise ValueError("invalid private telemetry settings") from None
        if len(document) > 65536:
            raise ValueError("telemetry settings exceed 64 KiB")
        return cls(document.decode(), selection, configuration_id)

    @property
    def wire(self) -> Settings:
        """A fresh typed projection cannot mutate the process selection."""
        try:
            return msgspec.json.decode(self._document, type=Settings)
        except (msgspec.DecodeError, ValueError, TypeError):
            raise ValueError("invalid private telemetry settings") from None

    def native_json(self) -> str:
        """Return only the canonical immutable Rust-owned private contract."""
        return self._document

    def effective(self) -> dict[str, object]:
        wire = self.wire
        versions: dict[str, str | None] = {}
        for distribution in (
            "opentelemetry-api",
            "opentelemetry-sdk",
            "opentelemetry-exporter-otlp-proto-http",
            "opentelemetry-instrumentation-logging",
        ):
            try:
                versions[distribution] = version(distribution)
            except PackageNotFoundError:
                versions[distribution] = None
        safe_resource = {
            key: value
            for key, value in wire.resource.items()
            if key
            in {
                "service.name",
                "service.version",
                "service.instance.id",
                "process.pid",
                "telemetry.sdk.language",
            }
        }
        return {
            "schema_version": wire.schema_version,
            "disabled": wire.disabled,
            "selection": self.selection,
            "signals": {
                "traces": _effective_signal(wire.traces),
                "logs": _effective_signal(wire.logs),
                "metrics": _effective_signal(wire.metrics),
            },
            "transport": "http/protobuf",
            "resource": safe_resource,
            "sdk_versions": versions,
            "lifecycle_owner": "process host",
            "sampler": wire.sampler,
            "sampler_argument": wire.sampler_argument,
            "max_queue_size": wire.max_queue_size,
            "max_export_batch_size": wire.max_export_batch_size,
            "configuration_id": self.configuration_id,
            "delivery": "not_observed",
        }


def resolve_process(
    service_name: str,
    *,
    root: Path | None = None,
    environment: Mapping[str, str] | None = None,
) -> ResolvedSettings:
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
    base = env.get(
        "OTEL_EXPORTER_OTLP_ENDPOINT", configured if isinstance(configured, str) else ""
    )
    selected = bool(
        base
        or any(
            env.get(f"OTEL_EXPORTER_OTLP_{name}_ENDPOINT")
            for name in ("TRACES", "LOGS", "METRICS")
        )
    )
    disabled = env.get("OTEL_SDK_DISABLED", "false").lower() == "true" or not selected
    base_headers = env.get("OTEL_EXPORTER_OTLP_HEADERS", "")
    signals: dict[str, SignalSettings] = {}
    for name in ("TRACES", "LOGS", "METRICS"):
        exporter = env.get(f"OTEL_{name}_EXPORTER", "otlp")
        if exporter not in {"otlp", "none"}:
            raise ValueError(
                "unsupported telemetry exporter: OTEL_" + name + "_EXPORTER"
            )
        enabled = not disabled and exporter != "none"
        protocol = env.get(
            f"OTEL_EXPORTER_OTLP_{name}_PROTOCOL",
            env.get("OTEL_EXPORTER_OTLP_PROTOCOL", "http/protobuf"),
        )
        if enabled and protocol != "http/protobuf":
            raise ValueError("selected telemetry transport requires http/protobuf")
        endpoint = env.get(f"OTEL_EXPORTER_OTLP_{name}_ENDPOINT")
        if endpoint is None and base:
            endpoint = base.rstrip("/") + "/v1/" + name.lower()
        timeout = _number(
            env,
            f"OTEL_EXPORTER_OTLP_{name}_TIMEOUT",
            _number(env, "OTEL_EXPORTER_OTLP_TIMEOUT", 5, maximum=60),
            maximum=60,
        )
        signals[name] = SignalSettings(
            enabled=enabled and bool(endpoint),
            endpoint=_endpoint(endpoint) if endpoint and enabled else None,
            headers=dict(
                _headers(env.get(f"OTEL_EXPORTER_OTLP_{name}_HEADERS", base_headers))
            )
            if enabled
            else {},
            timeout_ms=max(1, int(timeout * 1000)),
        )
    resource = dict(_headers(env.get("OTEL_RESOURCE_ATTRIBUTES", "")))
    try:
        package_version = version("pse-arrow")
    except PackageNotFoundError:
        package_version = "source"
    resource.update(
        {
            "service.name": env.get("OTEL_SERVICE_NAME", service_name),
            "service.version": package_version,
            "service.instance.id": process_incarnation(),
            "process.pid": str(os.getpid()),
        }
    )
    queue = _integer(env, "OTEL_BSP_MAX_QUEUE_SIZE", 2048, maximum=65536)
    batch = _integer(env, "OTEL_BSP_MAX_EXPORT_BATCH_SIZE", 512, maximum=65536)
    if batch > queue:
        raise ValueError("telemetry export batch exceeds bounded queue")
    sampler = env.get("OTEL_TRACES_SAMPLER", "parentbased_always_on")
    if sampler not in {
        "always_on",
        "always_off",
        "parentbased_always_on",
        "parentbased_always_off",
        "traceidratio",
        "parentbased_traceidratio",
    }:
        raise ValueError("unsupported telemetry sampler")
    try:
        argument = float(env.get("OTEL_TRACES_SAMPLER_ARG", "1"))
    except ValueError:
        raise ValueError("invalid telemetry sampler argument") from None
    if not math.isfinite(argument) or not 0 <= argument <= 1:
        raise ValueError("invalid telemetry sampler argument")
    configuration_id = configuration.get("configuration_id")
    return ResolvedSettings.from_wire(
        wire=Settings(
            disabled=disabled,
            traces=signals["TRACES"],
            logs=signals["LOGS"],
            metrics=signals["METRICS"],
            resource=resource,
            process_incarnation=resource["service.instance.id"],
            max_queue_size=queue,
            max_export_batch_size=batch,
            schedule_delay_ms=_integer(env, "OTEL_BSP_SCHEDULE_DELAY", 5000),
            export_timeout_ms=_integer(env, "OTEL_BSP_EXPORT_TIMEOUT", 5000),
            metric_export_interval_ms=_integer(
                env, "OTEL_METRIC_EXPORT_INTERVAL", 60000, maximum=3600000
            ),
            sampler=sampler,
            sampler_argument=argument,
        ),
        selection="disabled"
        if disabled
        else "environment"
        if any(k.startswith("OTEL_EXPORTER_OTLP") for k in env)
        else "configured_service",
        configuration_id=configuration_id
        if isinstance(configuration_id, str)
        else None,
    )
