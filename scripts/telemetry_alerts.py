# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed local diagnostic alerts evaluated and delivered by OpenObserve OSS.

There is no host polling scheduler. Each control is explicit; backend history,
source freshness and local delivery are separate observations, never intervention.
"""
from __future__ import annotations

import json
import re
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import telemetry_query, telemetry_service

if TYPE_CHECKING:
    from collections.abc import Mapping

ALERT_STREAM = "pse_alerts"
TEMPLATE = "pse_telemetry_local_v1"
DESTINATION = "pse_telemetry_local_v1"
DESCRIPTION = "PSE local telemetry v1; diagnostic only; no automatic intervention"
INTERVAL_SECONDS = 60
WINDOW_SECONDS = 300
STALE_SECONDS = 120
MAX_INTERVAL_US = 7 * 86400 * 1_000_000
QUEUE_WARNING_FRACTION = 0.8
QUEUE_CAPACITY = telemetry_service.COLLECTOR_LIMITS["queue_requests_per_signal"]


@dataclass(frozen=True)
class Condition:
    name: str
    metric: str
    threshold: float
    interpretation: str
    increase: bool = False

    def sql(self) -> str:
        # Every identifier is a closed declaration, never caller supplied SQL.
        if self.increase:
            delta = "CASE WHEN previous_value IS NULL THEN 0 WHEN value >= previous_value THEN value-previous_value ELSE value END"
            return (
                f"SELECT service_instance_id,exporter,SUM({delta}) AS increase "
                f"FROM (SELECT service_instance_id,exporter,value,LAG(value) OVER "
                f"(PARTITION BY service_instance_id,exporter ORDER BY _timestamp) AS previous_value FROM {self.metric}) "
                f"GROUP BY service_instance_id,exporter HAVING SUM({delta}) >= {self.threshold}"
            )
        return f"SELECT _timestamp,value,service_instance_id FROM {self.metric} WHERE value >= {self.threshold}"

    def public(self) -> dict[str, object]:
        return {
            "name": self.name, "metric": self.metric, "threshold": self.threshold,
            "interpretation": self.interpretation, "interval_seconds": INTERVAL_SECONDS,
            "window_seconds": WINDOW_SECONDS, "counter_resets": "reset-aware within each process/exporter series; first sample has no preceding observation" if self.increase else None,
        }


CONDITIONS = (
    Condition("queue_pressure", "otelcol_exporter_queue_size", QUEUE_WARNING_FRACTION * QUEUE_CAPACITY, f"At least {QUEUE_WARNING_FRACTION:.0%} of the selected {QUEUE_CAPACITY} request queue capacity per signal"),
    Condition("export_failures", "otelcol_exporter_send_failed_metric_points", 1, "At least one increase in failed metric point exports per exporter and Collector incarnation over five minutes", True),
    Condition("mx_poison", "surrealdb_storage_surrealmx_poisoned", 1, "Native MX poisoned gauge is set; no failure cause or automatic repair inferred"),
    Condition("persistence_pressure", "surrealdb_storage_surrealmx_async_queue_depth", 26, "At least 26 asynchronous persistence batches; explicit absolute warning threshold"),
)


class AlertError(RuntimeError):
    """Credential-free native alert failure."""


@dataclass
class _Selection:
    state: Path | str | None
    environment: Mapping[str, str] | None
    image: dict[str, object]
    base: str
    headers: dict[str, str] = field(repr=False)

    @classmethod
    def open(cls, state: Path | str | None, environment: Mapping[str, str] | None) -> _Selection:
        image = telemetry_service.inspect(state, environment=environment)
        versions = telemetry_service._record(image.get("versions"), "versions")
        if versions.get("openobserve") != "1.1.2":
            raise AlertError("Native telemetry alerts require the qualified OpenObserve OSS 1.1.2 release")
        backend = telemetry_service._record(telemetry_service._record(image.get("services"), "services").get("openobserve"), "backend")
        if backend.get("ownership") != "verified" or not isinstance(backend.get("process"), dict):
            raise AlertError("Selected telemetry backend is not active and owned")
        base, headers = telemetry_service.backend_access(state, environment=environment)
        return cls(state, environment, image, base, headers)

    @property
    def organization(self) -> str:
        value = telemetry_service._record(self.image["configuration"], "configuration").get("organization")
        if not isinstance(value, str) or not re.fullmatch(r"[a-zA-Z_][a-zA-Z0-9_]{0,127}", value):
            raise AlertError("Invalid selected telemetry organization")
        return value

    def recheck(self) -> None:
        image = telemetry_service.inspect(self.state, environment=self.environment)
        before = telemetry_service._record(self.image["services"], "services")["openobserve"]
        after = telemetry_service._record(image["services"], "services")["openobserve"]
        if image.get("configuration_id") != self.image.get("configuration_id") or after != before:
            raise AlertError("Telemetry backend incarnation changed during alert operation")

    def api(self, path: str, body: dict[str, object] | None = None, *, method: str | None = None, missing: bool = False) -> dict[str, object] | None:
        self.recheck()
        code, raw = telemetry_query.request(self.base + path, headers=self.headers, body=body, method=method)
        self.recheck()
        if missing and code == 404:
            return None
        if not 200 <= code < 300:
            raise AlertError(f"Native telemetry alert request failed (HTTP {code})")
        try:
            value = json.loads(raw)
        except (ValueError, TypeError):
            raise AlertError("Native telemetry alert response is invalid") from None
        if not isinstance(value, dict):
            raise AlertError("Native telemetry alert response is not an object")
        return value


def _list(value: dict[str, object] | None, key: str) -> list[dict[str, object]]:
    items = value.get(key) if value is not None else None
    if not isinstance(items, list) or any(not isinstance(item, dict) for item in items):
        raise AlertError("Native telemetry alert list is invalid")
    return items


def _name(condition: Condition) -> str:
    return "pse_telemetry_" + condition.name


def _payload(condition: Condition, org: str, enabled: bool) -> dict[str, object]:
    row = {"service_instance_id": "{service_instance_id}"}
    if condition.increase:
        row.update(exporter="{exporter}", increase="{increase}")
    else:
        row.update(value="{value}", timestamp_us="{_timestamp}")
    return {
        "name": _name(condition), "org_id": org, "stream_type": "metrics",
        "stream_name": condition.metric, "is_real_time": False,
        "query_condition": {"type": "sql", "sql": condition.sql(), "conditions": [], "search_event_type": "alerts"},
        "trigger_condition": {"period": 5, "operator": ">=", "threshold": 1, "frequency": 1,
                              "frequency_type": "minutes", "silence": 1, "timezone": "UTC", "cron": ""},
        "destinations": [DESTINATION], "template": TEMPLATE, "row_template": json.dumps(row),
        "context_attributes": {}, "description": DESCRIPTION, "enabled": enabled,
        "folderId": "default", "tz_offset": 0,
    }


def configure(state: Path | str | None = None, *, enabled: bool = True, environment: Mapping[str, str] | None = None) -> dict[str, object]:
    """Explicitly create/update only the fixed native rules and local HTTP destination.

    A source without an established metric schema remains missing, rather than
    receiving synthetic zero observations. Re-run this explicit control once
    its producer has published. Existing foreign rules with our names refuse.
    """
    if type(enabled) is not bool:
        raise ValueError("Alert enable selection must be boolean")
    selected = _Selection.open(state, environment)
    state_path = telemetry_service.state_directory(state, environment)
    config = telemetry_service._configuration(state_path)
    env_path = str(config.get("environment_file", state_path / "service.env"))
    values = dict(line.split("=", 1) for line in Path(env_path).read_text().splitlines() if "=" in line)
    if json.loads(values.get("ZO_SSRF_ALLOWED_CIDRS", '""')) != "127.0.0.1/32":
        raise AlertError("Explicit service reconfiguration is required for narrow loopback alert delivery")
    org = selected.organization
    existing = _list(selected.api(f"/api/v2/{org}/alerts"), "list")
    by_name = {item.get("name"): item for item in existing}
    for condition in CONDITIONS:
        item = by_name.get(_name(condition))
        if item is not None and item.get("description") != DESCRIPTION:
            raise AlertError("An independently owned native alert occupies a selected name")
    template = {
        "name": TEMPLATE, "type": "http", "body": json.dumps([{
            "alert_name": "{alert_name}", "stream_name": "{stream_name}",
            "org_name": "{org_name}", "alert_type": "{alert_type}",
            "matched_rows_start": "{alert_start_time}", "matched_rows_end": "{alert_end_time}",
            "triggered_at_us": "{alert_trigger_time}", "observations": "{rows:10}",
            "condition": "scheduled diagnostic threshold satisfied", "intervention": "none",
        }]),
    }
    existing_template = selected.api(f"/api/{org}/alerts/templates/{TEMPLATE}", missing=True)
    if existing_template is not None and (existing_template.get("name") != TEMPLATE or existing_template.get("body") != template["body"]):
        raise AlertError("An independently owned notification template occupies a selected name")
    existing_destination = selected.api(f"/api/{org}/alerts/destinations/{DESTINATION}", missing=True)
    if existing_destination is not None and (existing_destination.get("url") != selected.base + f"/api/{org}/{ALERT_STREAM}/_json" or existing_destination.get("template") != TEMPLATE):
        raise AlertError("An independently owned notification destination occupies a selected name")
    if existing_template is None:
        selected.api(f"/api/{org}/alerts/templates", template)
    selected.api(f"/api/{org}/alerts/destinations" + (f"/{DESTINATION}" if existing_destination is not None else ""), {
        "name": DESTINATION, "url": selected.base + f"/api/{org}/{ALERT_STREAM}/_json",
        "method": "post", "skip_tls_verify": False, "template": TEMPLATE,
        "headers": selected.headers | {"Content-Type": "application/json"},
    }, method="PUT" if existing_destination is not None else "POST")
    rules = []
    for condition in CONDITIONS:
        item = by_name.get(_name(condition))
        if item is not None and not enabled:
            alert_id = item.get("alert_id")
            if not isinstance(alert_id, str) or not re.fullmatch(r"[a-zA-Z0-9]+", alert_id):
                raise AlertError("Native alert identifier is invalid")
            selected.api(f"/api/v2/{org}/alerts/{alert_id}/enable?value=false", method="PATCH")
            rules.append(condition.public() | {"status": "disabled"})
            continue
        source = selected.api(f"/api/{org}/streams/{condition.metric}/schema?type=metrics", missing=True)
        if source is None:
            rules.append(condition.public() | {"status": "missing", "reason": "source_metric_not_published"})
            continue
        payload = _payload(condition, org, enabled)
        if item is None:
            selected.api(f"/api/v2/{org}/alerts", payload)
        else:
            alert_id = item.get("alert_id")
            if not isinstance(alert_id, str) or not re.fullmatch(r"[a-zA-Z0-9]+", alert_id):
                raise AlertError("Native alert identifier is invalid")
            selected.api(f"/api/v2/{org}/alerts/{alert_id}", payload | {"id": alert_id}, method="PUT")
        rules.append(condition.public() | {"status": "enabled" if enabled else "disabled"})
    selected.recheck()
    return {"status": "configured", "rules": rules, "delivery_stream": ALERT_STREAM,
            "configuration_id": selected.image["configuration_id"], "scheduler": "openobserve-native",
            "automatic_intervention": False}


def evaluation_status(rule: dict[str, object] | None, latest_source_us: int | None, history: list[dict[str, object]], now_us: int) -> str:
    """A quiet, disabled or overdue evaluation never reports healthy by omission."""
    if rule is None:
        return "missing"
    if rule.get("enabled") is not True:
        return "disabled"
    if latest_source_us is None:
        return "missing_data"
    if now_us - latest_source_us > STALE_SECONDS * 1_000_000:
        return "stale"
    native_evaluation = rule.get("last_triggered_at")
    if type(native_evaluation) is not int:
        native_evaluation = None
    if not history:
        if native_evaluation is not None:
            return "backlogged" if now_us - native_evaluation > STALE_SECONDS * 1_000_000 else "evaluated_history_pending"
        return "evaluation_unavailable"
    last = max(history, key=lambda row: int(row.get("timestamp", 0)))
    if native_evaluation is not None and native_evaluation > int(last.get("timestamp", 0)):
        return "backlogged" if now_us - native_evaluation > STALE_SECONDS * 1_000_000 else "evaluated_history_pending"
    if now_us - int(last.get("timestamp", 0)) > STALE_SECONDS * 1_000_000:
        return "backlogged"
    outcome = str(last.get("status", "")).lower()
    if outcome in {"error", "failed"}:
        return "evaluation_failed"
    if outcome in {"skipped", "pending"}:
        return outcome
    return "evaluated"


def read(state: Path | str | None = None, *, start_time_us: int, end_time_us: int, limit: int = 100, environment: Mapping[str, str] | None = None) -> dict[str, object]:
    """Read native schedule/history, source freshness and delivered local outcomes."""
    if type(start_time_us) is not int or type(end_time_us) is not int or not 0 <= start_time_us < end_time_us or end_time_us - start_time_us > MAX_INTERVAL_US:
        raise ValueError("Alert reads require an ordered interval of at most seven days")
    if type(limit) is not int or not 1 <= limit <= 1000:
        raise ValueError("Alert reads require from one to 1000 rows")
    try:
        selected = _Selection.open(state, environment)
    except (AlertError, telemetry_service.TelemetryServiceError, OSError, ValueError):
        return {"status": "unavailable", "reason": "selected_backend_not_active_or_owned", "records": [], "rules": [], "complete": False}
    org = selected.organization
    now_us = time.time_ns() // 1000
    rules_native = _list(selected.api(f"/api/v2/{org}/alerts"), "list")
    history_answer = selected.api(f"/api/v2/{org}/alerts/history?start_time={start_time_us}&end_time={end_time_us}&size={limit}&sort_by=timestamp&sort_order=desc", missing=True)
    history = _list(history_answer, "hits") if history_answer is not None else []
    rules = []
    for condition in CONDITIONS:
        rule = next((item for item in rules_native if item.get("name") == _name(condition)), None)
        source_exists = selected.api(f"/api/{org}/streams/{condition.metric}/schema?type=metrics", missing=True)
        source = None if source_exists is None else selected.api(f"/api/{org}/_search?type=metrics", {"query": {
            "sql": f"SELECT MAX(_timestamp) AS latest FROM {condition.metric}",
            "start_time": max(0, now_us - WINDOW_SECONDS * 1_000_000), "end_time": now_us, "from": 0, "size": 1,
        }}, missing=True)
        hits = _list(source, "hits") if source is not None else []
        latest = hits[0].get("latest") if hits else None
        latest = latest if type(latest) is int else None
        owned_history = [item for item in history if item.get("alert_name") == _name(condition)]
        rules.append(condition.public() | {"status": evaluation_status(rule, latest, owned_history, now_us),
            "native_alert_id": rule.get("alert_id") if rule else None, "latest_source_us": latest,
            "last_native_evaluation_us": rule.get("last_triggered_at") if rule else None,
            "last_native_satisfied_us": rule.get("last_satisfied_at") if rule else None,
            "history": owned_history, "source_interval_start_us": max(0, now_us - WINDOW_SECONDS * 1_000_000), "source_interval_end_us": now_us})
    delivery_exists = selected.api(f"/api/{org}/streams/{ALERT_STREAM}/schema?type=logs", missing=True)
    delivery = None if delivery_exists is None else selected.api(f"/api/{org}/_search?type=logs", {"query": {
        "sql": f"SELECT * FROM {ALERT_STREAM} ORDER BY _timestamp DESC", "start_time": start_time_us,
        "end_time": end_time_us, "from": 0, "size": limit,
    }}, missing=True)
    records = _list(delivery, "hits") if delivery is not None else []
    if len(records) > limit or len(history) > limit:
        raise AlertError("Native alert result exceeds selected row bound")
    selected.recheck()
    services = telemetry_service._record(selected.image["services"], "services")
    return {"status": "available", "rules": rules, "records": records, "history": history,
            "complete": False, "source": {"backend": "openobserve", "backend_version": "1.1.2",
            "configuration_id": selected.image["configuration_id"], "backend_process": services["openobserve"],
            "start_time_us": start_time_us, "end_time_us": end_time_us, "retrieved_at_us": now_us,
            "source_cut": "bounded live read; no atomic snapshot", "limit": limit},
            "delivery_stream": ALERT_STREAM, "automatic_intervention": False}
