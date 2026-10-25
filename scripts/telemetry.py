# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit telemetry service actions and passive diagnostic reads."""

from __future__ import annotations

import argparse
import http.client
import json
import math
import os
import time
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import (
    telemetry_alerts,
    telemetry_config,
    telemetry_process,
    telemetry_query,
    telemetry_service,
)

if TYPE_CHECKING:
    from collections.abc import Mapping


def _collector_samples(code: int, raw: bytes) -> list[dict[str, object]]:
    if code != 200:
        raise ValueError("Collector self metrics refused")
    from prometheus_client.parser import (  # noqa: PLC0415 -- load parser only for selected metrics reads
        text_string_to_metric_families,
    )

    samples: list[dict[str, object]] = []
    prefixes = (
        "otelcol_receiver_accepted_",
        "otelcol_receiver_refused_",
        "otelcol_exporter_sent_",
        "otelcol_exporter_send_failed_",
        "otelcol_exporter_enqueue_failed_",
        "otelcol_exporter_queue_",
    )
    for family in text_string_to_metric_families(raw.decode()):
        for sample in family.samples:
            if (
                sample.name.startswith(prefixes)
                and sample.value is not None
                and math.isfinite(sample.value)
            ):
                if len(samples) == 256:
                    raise ValueError("Collector self metrics exceed selected series")
                samples.append(
                    {
                        "name": sample.name,
                        "labels": sample.labels,
                        "value": sample.value,
                    }
                )
    return samples


def health(
    state: Path | str | None = None, *, environment: Mapping[str, str] | None = None
) -> dict[str, object]:
    """Observe independently sourced pipeline stages; never install/start/emit a probe."""
    observed = time.time_ns()
    try:
        service = telemetry_service.inspect(state, environment=environment)
    except (OSError, ValueError, telemetry_service.TelemetryServiceError):
        # A damaged deployment must not hide the independently available local reader.
        service = {
            "status": "unavailable",
            "reason": "selected_service_inspection_failed",
        }
    config = service.get("configuration")
    answer: dict[str, object] = {
        "observed_at_ns": observed,
        "service": service,
        "producer": telemetry_process.local_status(),
        "delivery": {
            "backend_acceptance": "unknown",
            "query_visibility": "not_exercised",
            "crash_survival": "not_established",
        },
        "source_coverage": "current local process and selected service observations; not all producers",
    }
    if not isinstance(config, dict):
        answer["pipeline"] = {
            "status": "unavailable",
            "reason": "telemetry_not_configured",
        }
        return answer
    selected_endpoints = service.get("endpoints")
    endpoints_view = selected_endpoints if isinstance(selected_endpoints, dict) else {}
    selected_services = service.get("services")
    services_view = selected_services if isinstance(selected_services, dict) else {}
    endpoints = {"backend": str(config.get("backend_http_endpoint", "")) + "/healthz"}
    if isinstance(endpoints_view.get("collector_health"), str):
        endpoints["collector"] = endpoints_view["collector_health"]
    stages: dict[str, object] = {}
    for name, endpoint in endpoints.items():
        owned = services_view.get("openobserve" if name == "backend" else "collector")
        if (
            not isinstance(owned, dict)
            or owned.get("ownership") != "verified"
            or not isinstance(owned.get("process"), dict)
        ):
            stages[name] = {
                "observed_at_ns": time.time_ns(),
                "status": "unavailable",
                "reason": "selected_service_not_active_or_owned",
            }
            continue
        try:
            code, _ = telemetry_query.request(endpoint)
            stages[name] = {
                "observed_at_ns": time.time_ns(),
                "status": "responding" if code == 200 else "refused",
                "http_status": code,
            }
        except (OSError, ValueError, http.client.HTTPException):
            stages[name] = {"observed_at_ns": time.time_ns(), "status": "unavailable"}
    collector = services_view.get("collector")
    if (
        isinstance(endpoints_view.get("collector_metrics"), str)
        and isinstance(collector, dict)
        and collector.get("ownership") == "verified"
        and isinstance(collector.get("process"), dict)
    ):
        try:
            code, raw = telemetry_query.request(endpoints_view["collector_metrics"])
            samples = _collector_samples(code, raw)
            stages["collector_delivery"] = {
                "status": "observed",
                "observed_at_ns": time.time_ns(),
                "source": "collector cumulative self metrics; not per-request acknowledgement",
                "samples": samples,
                "configuration_id": config.get("configuration_id"),
                "process": collector.get("process"),
                "query_visibility": "not_established_by_exporter_counters",
            }
        except (OSError, ValueError, UnicodeError, http.client.HTTPException):
            stages["collector_delivery"] = {
                "status": "unavailable",
                "observed_at_ns": time.time_ns(),
            }
    try:
        after = telemetry_service.inspect(state, environment=environment).get(
            "services"
        )
    except (OSError, ValueError, telemetry_service.TelemetryServiceError):
        after = None
    for name, selected in services_view.items():
        if (
            not isinstance(selected, dict)
            or selected.get("ownership") != "verified"
            or not isinstance(selected.get("process"), dict)
        ):
            continue
        current = after.get(name) if isinstance(after, dict) else None
        if (
            not isinstance(current, dict)
            or selected.get("process") != current.get("process")
            or current.get("ownership") != "verified"
        ):
            for stage in (
                ("backend",)
                if name == "openobserve"
                else ("collector", "collector_delivery")
            ):
                if stage in stages:
                    stages[stage] = {
                        "status": "stale",
                        "observed_at_ns": time.time_ns(),
                        "reason": "selected_process_changed_during_read",
                    }
    answer["pipeline"] = stages
    return answer


def _operation(args: argparse.Namespace) -> Mapping[str, object]:
    if args.command == "configuration":
        environment = (
            None
            if args.state is None
            else {**os.environ, "PSE_TELEMETRY_STATE": str(args.state)}
        )
        result = telemetry_config.resolve_process(
            "pse-telemetry-observer", environment=environment
        ).effective()
    elif args.command == "health":
        result = health(args.state)
    elif args.command == "install":
        result = telemetry_service.install()
    elif args.command == "setup":
        ports = json.loads(args.ports) if args.ports else None
        if ports is not None and not isinstance(ports, dict):
            raise ValueError("Listener selection must be an object")
        result = telemetry_service.setup(args.state, ports=ports)
    elif args.command == "lookup-trace":
        if args.trace_id is None or args.start_us is None or args.end_us is None:
            raise ValueError(
                "Trace lookup requires --trace-id, --start-us and --end-us"
            )
        result = telemetry_query.lookup_trace(
            args.state,
            args.trace_id,
            start_time_us=args.start_us,
            end_time_us=args.end_us,
        )
    elif args.command in {"watch-configure", "watch-disable", "watch-read"}:
        if args.command == "watch-read":
            if args.start_us is None or args.end_us is None:
                raise ValueError("Watch history requires --start-us and --end-us")
            result = telemetry_alerts.read(
                args.state, start_time_us=args.start_us, end_time_us=args.end_us
            )
        else:
            result = telemetry_alerts.configure(
                args.state, enabled=args.command == "watch-configure"
            )
    else:
        action = {
            "start": telemetry_service.start,
            "inspect": telemetry_service.inspect,
            "restart": telemetry_service.restart,
            "stop": telemetry_service.stop,
            "enable": telemetry_service.enable,
            "disable": telemetry_service.disable,
            "reconfigure": telemetry_service.reconfigure,
            "recover-configuration": telemetry_service.recover_previous_configuration,
        }[args.command]
        result = action(args.state)
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "command",
        choices=(
            "install",
            "setup",
            "start",
            "inspect",
            "health",
            "restart",
            "stop",
            "enable",
            "disable",
            "configuration",
            "lookup-trace",
            "reconfigure",
            "recover-configuration",
            "watch-configure",
            "watch-disable",
            "watch-read",
        ),
    )
    parser.add_argument("--state", type=Path)
    parser.add_argument(
        "--ports", help="Explicit JSON listener-port selection for owned fixtures"
    )
    parser.add_argument("--trace-id")
    parser.add_argument("--start-us", type=int)
    parser.add_argument("--end-us", type=int)
    args = parser.parse_args(argv)
    if args.ports is not None and args.command != "setup":
        parser.error("--ports is supported only by setup")
    if any(
        value is not None for value in (args.trace_id, args.start_us, args.end_us)
    ) and args.command not in {"lookup-trace", "watch-read"}:
        parser.error("trace filters are supported only by lookup-trace")
    try:
        result = _operation(args)
        print(json.dumps(result, indent=2, sort_keys=True, allow_nan=False))
    except (OSError, ValueError, RuntimeError, http.client.HTTPException) as error:
        print(
            json.dumps(
                {
                    "status": "unavailable",
                    "code": "pse.telemetry.operation_refused",
                    "operation": args.command,
                    "message": str(error)
                    if isinstance(
                        error, (telemetry_service.TelemetryServiceError, ValueError)
                    )
                    else "Selected telemetry operation is unavailable",
                    "remedy": "Inspect the selected telemetry state and owned service configuration; native results remain available through their original readers.",
                }
            )
        )
        return 1

    else:
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
