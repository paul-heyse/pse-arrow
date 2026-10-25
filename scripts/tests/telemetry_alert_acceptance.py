# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded native OSS alert journey on a disposable, authenticated service pair.

Synthetic metric observations belong only to this fixture. Ordinary telemetry
services and their data are never stopped, reconfigured or injected into.
"""

from __future__ import annotations

import argparse
import json
import tempfile
import time
import uuid
from pathlib import Path

from scripts import telemetry_alerts as alerts
from scripts import telemetry_query, telemetry_service
from scripts.tests.telemetry_pipeline_acceptance import Fixture, records, require


def run() -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="pse-alert-acceptance-") as temporary:
        fixture = Fixture(Path(temporary) / "state", 30)
        receipt: dict[str, object] = {
            "baseline_failures": 0,
            "fixture": "synthetic isolated native metrics",
            "backend": "OpenObserve OSS 1.1.2",
            "collector": "0.163.2",
        }
        try:
            fixture.configure()
            fixture.start()
            now = time.time_ns()
            instance = uuid.uuid4().hex
            metrics = []
            for condition in alerts.CONDITIONS:
                # This stream exists but has no samples in the current five-minute
                # window. It must report missing data, never an inferred zero.
                timestamp = (
                    now - 600_000_000_000
                    if condition.name == "persistence_pressure"
                    else now
                )
                points = [
                    {
                        "timeUnixNano": str(timestamp),
                        "asInt": str(int(condition.threshold) + 1),
                        "attributes": [
                            {"key": "exporter", "value": {"stringValue": "fixture"}}
                        ],
                    }
                ]
                if condition.increase:
                    points.insert(
                        0,
                        {
                            "timeUnixNano": str(timestamp - 1_000_000_000),
                            "asInt": "0",
                            "attributes": [
                                {"key": "exporter", "value": {"stringValue": "fixture"}}
                            ],
                        },
                    )
                metrics.append(
                    {"name": condition.metric, "gauge": {"dataPoints": points}}
                )
            code, _ = telemetry_query.request(
                f"http://127.0.0.1:{fixture.ports['otlp_http']}/v1/metrics",
                headers={"Content-Type": "application/json"},
                body={
                    "resourceMetrics": [
                        {
                            "resource": {
                                "attributes": [
                                    {
                                        "key": "service.instance.id",
                                        "value": {"stringValue": instance},
                                    }
                                ]
                            },
                            "scopeMetrics": [
                                {
                                    "scope": {"name": "pse.alert.acceptance"},
                                    "metrics": metrics,
                                }
                            ],
                        }
                    ]
                },
            )
            require(code == 200, "alert.fixture_ingestion_failed")
            # Backend schema publication is asynchronous. This is a finite test
            # wait, not a production scheduler or a service lifecycle side effect.
            for _attempt in range(30):
                base, headers = telemetry_service.backend_access(fixture.state)
                if all(
                    telemetry_query.request(
                        base
                        + f"/api/default/streams/{condition.metric}/schema?type=metrics",
                        headers=headers,
                    )[0]
                    == 200
                    for condition in alerts.CONDITIONS
                ):
                    break
                time.sleep(1)
            else:
                raise RuntimeError("alert.fixture_source_schema_not_visible")
            receipt["configuration"] = alerts.configure(fixture.state)
            for attempt in range(12):
                time.sleep(15)
                end = time.time_ns() // 1000
                answer = alerts.read(
                    fixture.state, start_time_us=end - 600_000_000, end_time_us=end
                )
                print(
                    json.dumps(
                        {
                            "attempt": attempt,
                            "delivery_count": len(records(answer["records"])),
                            "history_count": len(records(answer["history"])),
                        }
                    ),
                    flush=True,
                )
                deliveries = records(answer["records"])
                timestamps = {
                    row.get("_timestamp")
                    for row in deliveries
                    if row.get("alert_name") == "pse_telemetry_queue_pressure"
                }
                if len(timestamps) >= 2 and answer["history"]:
                    receipt["scheduled"] = answer
                    break
            else:
                raise RuntimeError(
                    "alert.native_schedule_history_or_recurrence_not_observed"
                )
            require(
                any(
                    rule["name"] == "persistence_pressure"
                    and rule["status"] == "missing_data"
                    for rule in records(answer["rules"])
                ),
                "alert.missing_data_was_not_explicit",
            )
            require(
                any(
                    row.get("alert_name") == "pse_telemetry_export_failures"
                    for row in deliveries
                ),
                "alert.counter_increase_not_delivered",
            )
            require(
                any(row.get("observations") for row in deliveries),
                "alert.process_observations_not_delivered",
            )
            receipt["disable"] = alerts.configure(fixture.state, enabled=False)
            end = time.time_ns() // 1000
            disabled = alerts.read(
                fixture.state, start_time_us=end - 600_000_000, end_time_us=end
            )
            require(
                all(
                    rule["status"] == "disabled" for rule in records(disabled["rules"])
                ),
                "alert.disabled_state_not_explicit",
            )
            receipt["disabled_read"] = disabled
            receipt["automatic_intervention"] = False
        finally:
            receipt["cleanup"] = fixture.cleanup(remove_state=True)
        return receipt


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence", type=Path, required=True)
    arguments = parser.parse_args()
    if arguments.evidence.exists():
        parser.error("evidence destination must be fresh")
    result = run()
    arguments.evidence.parent.mkdir(parents=True, exist_ok=True)
    arguments.evidence.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"status": "passed", "evidence": str(arguments.evidence)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
