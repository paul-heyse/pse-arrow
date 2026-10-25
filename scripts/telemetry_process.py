# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Executable-owned SDK composition and W3C propagation, never scientific state."""

from __future__ import annotations

import importlib
import json
import logging
import os
import re
import sys
import threading
from contextlib import ExitStack, contextmanager, suppress
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Protocol, Self

from scripts.telemetry_config import ResolvedSettings, resolve_process

if TYPE_CHECKING:
    from collections.abc import Callable, Generator, Mapping
    from pathlib import Path

    from requests import PreparedRequest, Request, Session

CARRIER_ENV = "PSE_TELEMETRY_CARRIER"
_STATEKEY = re.compile(
    r"(?:[a-z][a-z0-9_*/-]{0,255}|[a-z0-9][a-z0-9_*/-]{0,240}@[a-z][a-z0-9_*/-]{0,13})\Z"
)
_TRACEPARENT = re.compile(r"00-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}\Z")
_LOCK = threading.Lock()


class Provider(Protocol):
    def shutdown(self) -> None: ...


class NativeProvider(Provider, Protocol):
    activation: dict[str, str | None]


def _diagnostic(code: str) -> None:
    # Never render settings, exception messages, endpoints or HTTP credentials.
    print("pse-telemetry: " + code, file=sys.stderr)


def validated_carrier(value: object) -> dict[str, str]:
    """Copy only bounded W3C fields; no baggage or arbitrary environment data."""
    if (
        not isinstance(value, dict)
        or not value
        or set(value) - {"traceparent", "tracestate"}
    ):
        return {}
    parent = value.get("traceparent")
    if not isinstance(parent, str) or not _TRACEPARENT.fullmatch(parent):
        return {}
    if parent[3:35] == "0" * 32 or parent[36:52] == "0" * 16:
        return {}
    result = {"traceparent": parent}
    state = value.get("tracestate")
    if state is not None:
        if (
            not isinstance(state, str)
            or len(state) > 512
            or not state.isascii()
            or any(ord(c) < 32 or ord(c) > 126 for c in state)
        ):
            return {}
        members = [item.strip() for item in state.split(",")]
        keys = set()
        if not state or len(members) > 32:
            return {}
        for member in members:
            key, separator, content = member.partition("=")
            if (
                not separator
                or not _STATEKEY.fullmatch(key)
                or key in keys
                or not content
                or content[-1] == " "
                or any(c in content for c in ",=")
            ):
                return {}
            keys.add(key)
        from opentelemetry.trace import TraceState  # noqa: PLC0415

        parsed = TraceState.from_header([state])
        if state and not parsed:
            return {}
        result["tracestate"] = parsed.to_header()
    return result


def current_carrier() -> dict[str, str]:
    """Read the current API context on every native ingress, never a cached span."""
    from opentelemetry.trace.propagation.tracecontext import (  # noqa: PLC0415 -- context API is loaded only on propagation
        TraceContextTextMapPropagator,
    )

    carrier: dict[str, str] = {}
    TraceContextTextMapPropagator().inject(carrier)
    return validated_carrier(carrier)


def helper_environment(environment: Mapping[str, str] | None = None) -> dict[str, str]:
    result = dict(os.environ if environment is None else environment)
    result.pop(CARRIER_ENV, None)
    carrier = current_carrier()
    if carrier:
        result[CARRIER_ENV] = json.dumps(carrier, separators=(",", ":"))
    return result


@contextmanager
def inherited_context(environment: Mapping[str, str] | None = None) -> Generator[None]:
    """Attach only for owned child work and always restore the prior context."""
    env = os.environ if environment is None else environment
    source = env.get(CARRIER_ENV)
    if not source:
        yield
        return
    carrier = {}
    if len(source) <= 1024:
        with suppress(ValueError, TypeError):
            carrier = validated_carrier(json.loads(source))
    if not carrier:
        _diagnostic("carrier_refused")
        yield
        return
    from opentelemetry import context  # noqa: PLC0415
    from opentelemetry.trace.propagation.tracecontext import (  # noqa: PLC0415 -- context API is loaded only on propagation
        TraceContextTextMapPropagator,
    )

    token = context.attach(
        TraceContextTextMapPropagator().extract(carrier, context=context.Context())
    )
    try:
        yield
    finally:
        context.detach(token)


_INTERNAL_LOG_NAMES = (
    "opentelemetry",
    "urllib3",
    "requests",
    "httpx",
    "httpcore",
    "grpc",
)


def _internal_log(name: str) -> bool:
    return any(
        name == prefix or name.startswith(prefix + ".")
        for prefix in _INTERNAL_LOG_NAMES
    )


class _SafeTelemetryDiagnostics(logging.Filter):
    """Replace selected internal records before any handler can render payloads."""

    def filter(self, record: logging.LogRecord) -> bool:
        if not _internal_log(record.name):
            return True
        try:
            message = record.getMessage().lower()
        except Exception:
            message = ""
        # These are safe classifications, never acknowledgements or delivery facts.
        if "connection refused" in message:
            classification = "Connection refused"
        elif "timeout" in message or "timed out" in message:
            classification = "Export timed out"
        elif record.levelno >= logging.ERROR:
            classification = "Failed to export"
        else:
            classification = "SDK diagnostic"
        component = (
            "exporter"
            if record.name.startswith("opentelemetry.exporter.")
            else "sdk"
            if record.name.startswith("opentelemetry")
            else "transport"
        )
        # Construct from an allowlist: no original args, traceback, stack, body,
        # headers, URL, source path or arbitrary extra fields survive.
        safe = logging.LogRecord(
            "pse.telemetry.diagnostic",
            record.levelno,
            "",
            0,
            "pse-telemetry: "
            + classification
            + "; component="
            + component
            + "; details=redacted",
            (),
            None,
        )
        safe.created = record.created
        record.__dict__.clear()
        record.__dict__.update(safe.__dict__)
        return True


def _protect_diagnostic_logging() -> None:
    """Process-host routes; application logger/handler/provider identities stay put.

    Origin filters protect already configured handlers before their first render.
    Namespace routes also protect later SDK module loggers before root handlers.
    The guards deliberately remain through process exit: a batch worker can log
    after its bounded shutdown join, and SDK globals themselves are one-shot.
    """
    guard = _SafeTelemetryDiagnostics()
    # Snapshot before adding our own handlers, preserving every caller handler.
    selected = [
        item
        for item in logging.Logger.manager.loggerDict.values()
        if isinstance(item, logging.Logger) and _internal_log(item.name)
    ]
    for logger in selected:
        logger.addFilter(guard)
        for handler in logger.handlers:
            handler.addFilter(guard)
    for name in _INTERNAL_LOG_NAMES:
        logger = logging.getLogger(name)
        if logger not in selected:
            logger.addFilter(guard)
            for handler in logger.handlers:
                handler.addFilter(guard)
        handler = logging.StreamHandler(sys.stderr)
        handler.setLevel(logging.WARNING)
        handler.addFilter(guard)
        logger.addHandler(handler)
        logger.propagate = False


class _ApplicationLogs(logging.Filter):
    def filter(self, record: logging.LogRecord) -> bool:
        # SDK/exporter/transport failure logging must remain stderr-only.
        return (
            not _internal_log(record.name) and record.name != "pse.telemetry.diagnostic"
        )


@dataclass
class ProcessLifetime:
    settings: ResolvedSettings | None
    status: str
    pid: int = field(default_factory=os.getpid)
    providers: list[Provider] = field(default_factory=list)
    handler: logging.Handler | None = None
    native: NativeProvider | None = None
    closed: bool = False

    def shutdown(self) -> None:
        """Called after native work settles. Flush completion is not delivery proof."""
        if self.closed or self.pid != os.getpid():
            return
        self.closed = True
        if self.native is not None:
            try:
                self.native.shutdown()
            except Exception:
                _diagnostic("native_shutdown_failed")
        if self.handler is not None:
            logging.getLogger().removeHandler(self.handler)
            self.handler.close()
        for provider in reversed(self.providers):
            try:
                provider.shutdown()
            except Exception:
                _diagnostic("provider_shutdown_failed")

    def __enter__(self) -> Self:
        return self

    def __exit__(self, *_exception: object) -> None:
        self.shutdown()


@dataclass
class _ProcessState:
    active: ProcessLifetime | None = None


_STATE = _ProcessState()


def _protocol_headers(headers: Mapping[str, str]) -> dict[str, str]:
    selected = {name.lower(): value for name, value in headers.items()}
    selected["content-type"] = "application/x-protobuf"
    return selected


def _export_session(headers: Mapping[str, str]) -> Session:
    """Use the public SDK Session seam to preserve resolved wire headers exactly.

    SDK 1.45.1 unconditionally overlays ambient OTEL headers before invoking its
    session. Requests prepares the request normally; this owned session then
    applies the resolved selection before send. No process environment changes.
    The SDK transport closes its supplied session during provider shutdown.
    """
    from requests import (  # noqa: PLC0415 -- transport loaded only for selected exporters
        Session,
    )

    selected = _protocol_headers(headers)

    class ResolvedSession(Session):
        def prepare_request(self, request: Request) -> PreparedRequest:
            prepared = super().prepare_request(request)
            prepared.headers.clear()
            prepared.headers.update(selected)
            prepared.prepare_content_length(prepared.body)
            return prepared

    session = ResolvedSession()
    session.trust_env = False
    return session


def _compose(owner: ProcessLifetime) -> None:
    from opentelemetry import _logs, metrics, trace  # noqa: PLC0415
    from opentelemetry.sdk.resources import Resource  # noqa: PLC0415

    settings = owner.settings
    if settings is None:
        return
    wire = settings.wire
    # Public setters are one-shot. Never replace another application's providers.
    if (
        (
            wire.traces.enabled
            and not isinstance(trace.get_tracer_provider(), trace.ProxyTracerProvider)
        )
        or (
            wire.metrics.enabled
            and not isinstance(metrics.get_meter_provider(), metrics.NoOpMeterProvider)
            and type(metrics.get_meter_provider()).__name__ != "_ProxyMeterProvider"
        )
        or (
            wire.logs.enabled
            and type(_logs.get_logger_provider()).__name__ != "ProxyLoggerProvider"
        )
    ):
        owner.status = "initialization_refused"
        _diagnostic("existing_provider_preserved")
        return
    _protect_diagnostic_logging()
    from opentelemetry.exporter.otlp.proto.http import Compression  # noqa: PLC0415

    resource = Resource.create({**dict(wire.resource), "process.pid": os.getpid()})
    if wire.traces.enabled:
        from opentelemetry.exporter.otlp.proto.http.trace_exporter import (  # noqa: PLC0415 -- SDK loaded only for selected signal
            OTLPSpanExporter,
        )
        from opentelemetry.sdk.trace import TracerProvider  # noqa: PLC0415
        from opentelemetry.sdk.trace.export import BatchSpanProcessor  # noqa: PLC0415
        from opentelemetry.sdk.trace.sampling import (  # noqa: PLC0415
            ALWAYS_OFF,
            ALWAYS_ON,
            ParentBased,
            TraceIdRatioBased,
        )

        sampler = (
            ALWAYS_OFF
            if wire.sampler.endswith("always_off")
            else TraceIdRatioBased(wire.sampler_argument)
            if "traceidratio" in wire.sampler
            else ALWAYS_ON
        )
        if wire.sampler.startswith("parentbased"):
            sampler = ParentBased(sampler)
        provider = TracerProvider(
            resource=resource, sampler=sampler, shutdown_on_exit=False
        )
        owner.providers.append(provider)
        provider.add_span_processor(
            BatchSpanProcessor(
                OTLPSpanExporter(
                    endpoint=wire.traces.endpoint,
                    headers=_protocol_headers(dict(wire.traces.headers)),
                    timeout=wire.traces.timeout_ms / 1000,
                    compression=Compression.NoCompression,
                    session=_export_session(dict(wire.traces.headers)),
                ),
                max_queue_size=wire.max_queue_size,
                max_export_batch_size=wire.max_export_batch_size,
                schedule_delay_millis=wire.schedule_delay_ms,
                export_timeout_millis=wire.export_timeout_ms,
            )
        )
        trace.set_tracer_provider(provider)
    if wire.logs.enabled:
        from opentelemetry.exporter.otlp.proto.http._log_exporter import (  # noqa: PLC0415 -- SDK loaded only for selected signal
            OTLPLogExporter,
        )
        from opentelemetry.instrumentation.logging.handler import (  # noqa: PLC0415 -- SDK loaded only for selected signal
            LoggingHandler,
        )
        from opentelemetry.sdk._logs import LoggerProvider  # noqa: PLC0415
        from opentelemetry.sdk._logs.export import (  # noqa: PLC0415 -- SDK loaded only for selected signal
            BatchLogRecordProcessor,
        )

        logger_provider = LoggerProvider(resource=resource, shutdown_on_exit=False)
        owner.providers.append(logger_provider)
        logger_provider.add_log_record_processor(
            BatchLogRecordProcessor(
                OTLPLogExporter(
                    endpoint=wire.logs.endpoint,
                    headers=_protocol_headers(dict(wire.logs.headers)),
                    timeout=wire.logs.timeout_ms / 1000,
                    compression=Compression.NoCompression,
                    session=_export_session(dict(wire.logs.headers)),
                ),
                max_queue_size=wire.max_queue_size,
                max_export_batch_size=wire.max_export_batch_size,
                schedule_delay_millis=wire.schedule_delay_ms,
                export_timeout_millis=wire.export_timeout_ms,
            )
        )
        _logs.set_logger_provider(logger_provider)
        handler = LoggingHandler(logger_provider=logger_provider)
        handler.addFilter(_ApplicationLogs())
        # Retain an actual stderr fallback when there are no application handlers.
        if not logging.getLogger().handlers:
            logging.getLogger().addHandler(logging.StreamHandler(sys.stderr))
        logging.getLogger().addHandler(handler)
        owner.handler = handler
    if wire.metrics.enabled:
        from scripts.telemetry_metrics import (  # noqa: PLC0415
            cumulative_temporality,
            sdk_views,
        )
        from opentelemetry.exporter.otlp.proto.http.metric_exporter import (  # noqa: PLC0415 -- SDK loaded only for selected signal
            OTLPMetricExporter,
        )
        from opentelemetry.sdk.metrics import MeterProvider  # noqa: PLC0415
        from opentelemetry.sdk.metrics.export import (  # noqa: PLC0415 -- SDK loaded only for selected signal
            PeriodicExportingMetricReader,
        )

        reader = PeriodicExportingMetricReader(
            OTLPMetricExporter(
                endpoint=wire.metrics.endpoint,
                headers=_protocol_headers(dict(wire.metrics.headers)),
                timeout=wire.metrics.timeout_ms / 1000,
                compression=Compression.NoCompression,
                session=_export_session(dict(wire.metrics.headers)),
                preferred_temporality=cumulative_temporality(),
            ),
            export_interval_millis=wire.metric_export_interval_ms,
            export_timeout_millis=wire.metrics.timeout_ms,
        )
        meter_provider = MeterProvider(
            resource=resource, metric_readers=[reader], shutdown_on_exit=False,
            views=sdk_views(),
        )
        owner.providers.append(meter_provider)
        metrics.set_meter_provider(meter_provider)
    owner.status = "initialized"


def bootstrap(
    service_name: str,
    *,
    root: Path | None = None,
    environment: Mapping[str, str] | None = None,
    native: bool = False,
) -> ProcessLifetime:
    """An executable opts in once; library Runtime construction never calls this."""
    with _LOCK:
        if _STATE.active is not None and _STATE.active.pid == os.getpid():
            return ProcessLifetime(
                _STATE.active.settings,
                "already_initialized"
                if not _STATE.active.closed
                else "initialization_refused",
            )
        try:
            settings = resolve_process(service_name, root=root, environment=environment)
        except (ValueError, OSError, TypeError):
            _diagnostic("configuration_refused")
            _STATE.active = ProcessLifetime(None, "configuration_refused")
            return _STATE.active
        owner = ProcessLifetime(settings, "disabled")
        if settings.wire.disabled:
            _STATE.active = owner
            return owner
        try:
            _compose(owner)
            if native and owner.status == "initialized":
                facade = importlib.import_module("pse._telemetry")
                owner.native = facade.initialize(
                    settings.native_json(), current_carrier
                )
                if owner.native.activation.get("status") not in {
                    "initialized",
                    "already_initialized",
                    "disabled",
                }:
                    _diagnostic("native_initialization_refused")
        except Exception:
            # Telemetry is observational and cannot change a native operation outcome.
            _diagnostic("initialization_unavailable")
            owner.status = "initialization_unavailable"
        _STATE.active = owner
        return owner


def run_helper(
    service_name: str, main: Callable[[], int], *, native: bool = False
) -> int:
    """Fixed executable shell, outside the helper's native cancellation/drain owner."""
    from opentelemetry import trace  # noqa: PLC0415

    with (
        bootstrap(service_name, native=native),
        inherited_context(),
        ExitStack() as stack,
    ):
        stack.enter_context(
            trace.get_tracer(__name__, "1").start_as_current_span(
                "pse.helper",
                attributes={"pse.helper.service": service_name},
                record_exception=False,
                set_status_on_exception=False,
            )
        )
        return main()


def local_status() -> dict[str, object]:
    """Passive current-process state; configured endpoints never imply delivery."""
    owner = _STATE.active
    if owner is None or owner.pid != os.getpid():
        return {
            "status": "not_initialized",
            "lifecycle_owner": "process host",
            "export_attempts": {
                "status": "unavailable",
                "reason": "SDK counters not owned by this read",
            },
            "delivery": "not_observed",
            "queue_occupancy": "unavailable",
            "dropped_records": "unavailable",
        }
    return {
        "status": owner.status,
        "lifecycle_owner": "process host",
        "closed": owner.closed,
        "native": owner.native.activation if owner.native is not None else None,
        "configuration": owner.settings.effective()
        if owner.settings is not None
        else None,
        "export_attempts": {
            "status": "unavailable",
            "reason": "SDK counters not owned by this read",
        },
        "delivery": "not_observed",
        "queue_occupancy": "unavailable",
        "dropped_records": "unavailable",
    }
