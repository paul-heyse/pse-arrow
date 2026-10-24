# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Manual, effectful acceptance for disposable native telemetry services.

Run explicitly under scripts/pse-env; never discovered by unittest or pytest.
The default campaign owns one private service generation. Optional overflow and
startup-storage-failure cases each own a separate generation. No login enablement.
Receipts distinguish query visibility, observed replay and controlled expiry;
they establish neither exactly-once delivery nor general crash durability.
"""

from __future__ import annotations

import argparse
import hashlib
import http.client
import json
import logging
import os
import re
import shutil
import signal
import socket
import subprocess
import sys
import time
import uuid
from collections.abc import Callable, Iterator, Mapping
from contextlib import ExitStack, contextmanager
from pathlib import Path
from typing import TypeVar, cast
from unittest.mock import patch
from urllib.parse import urlsplit

from scripts import telemetry, telemetry_query, telemetry_service as service

ROOT = Path(__file__).resolve().parents[2]
PARENT_SPAN_ID = "1234567890abcdef"
CASES = ("signals_recovery_retention", "queue_overflow", "storage_startup_failure", "delivery_failures", "retry_expiry")
T = TypeVar("T")
RETENTION_OVERRIDES = {
    # Exact OpenObserve v1.1.2 config.rs: these are intervals/ingest admission,
    # not a shortened global retention policy (which remains seven days).
    "ZO_INGEST_ALLOWED_UPTO": "240",
    "ZO_COMPACT_DATA_RETENTION_INTERVAL": "2",
    "ZO_COMPACT_INTERVAL": "2",
    "ZO_MAX_FILE_RETENTION_TIME": "2",
}
SOURCES = {
    "backend_config": "https://github.com/openobserve/openobserve/blob/v1.1.2/src/config/src/config.rs",
    "stream_settings": "https://github.com/openobserve/openobserve/blob/v1.1.2/src/api/management/src/request/stream/mod.rs",
    "retention": "https://github.com/openobserve/openobserve/blob/v1.1.2/src/compaction/src/retention.rs",
    "collector_release_manifest": "https://github.com/open-telemetry/opentelemetry-collector-releases/blob/v0.163.2/distributions/otelcol-contrib/manifest.yaml",
    "file_storage_validation": "https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/v0.163.0/extension/storage/filestorage/config.go",
    "persistent_queue": "https://github.com/open-telemetry/opentelemetry-collector/blob/v0.163.0/exporter/exporterhelper/README.md",
}


class AcceptanceError(RuntimeError):
    """A safe fixture classification; backend response bodies are never rendered."""


def require(condition: object, code: str) -> None:
    if not condition:
        raise AcceptanceError(code)


def mapping(value: object) -> dict[str, object]:
    require(isinstance(value, dict), "fixture.object_expected")
    return cast("dict[str, object]", value)


def records(value: object) -> list[dict[str, object]]:
    require(isinstance(value, list), "fixture.rows_expected")
    return [mapping(row) for row in cast("list[object]", value)]


def wait_for(read: Callable[[], T], accepts: Callable[[T], bool], timeout: float, code: str) -> T:
    deadline = time.monotonic() + timeout
    while True:
        answer = read()
        if accepts(answer):
            return answer
        if time.monotonic() >= deadline:
            raise AcceptanceError(code)
        time.sleep(0.25)


def fresh_ports() -> dict[str, int]:
    """Hold all six sockets together until their distinct numbers are selected."""
    with ExitStack() as stack:
        answer = {}
        for name in ("otlp_http", "otlp_grpc", "backend_http", "backend_grpc", "metrics", "health"):
            listener = stack.enter_context(socket.socket())
            listener.bind(("127.0.0.1", 0))
            answer[name] = listener.getsockname()[1]
        return answer


def progress(phase: str) -> None:
    print("telemetry acceptance: " + phase, file=sys.stderr, flush=True)


def write_receipt(path: Path, receipt: dict[str, object]) -> None:
    temporary = path.with_suffix(".pending")
    temporary.write_text(json.dumps(receipt, indent=2, sort_keys=True, allow_nan=False) + "\n")
    temporary.chmod(0o600)
    temporary.replace(path)


@contextmanager
def fixture_configuration(*, queue_size: int | None = None, storage_failure: bool = False, bad_authorization: bool = False, retry_expiry: bool = False) -> Iterator[None]:
    """Patch declarations before setup seals their digests; never edit a generation."""
    original_write = service._write
    original_collector = service.collector_configuration

    def selected_write(path: Path, value: str, mode: int = 0o600) -> None:
        if path.name == "service.env":
            value += "".join(f"{key}={json.dumps(content)}\n" for key, content in RETENTION_OVERRIDES.items())
        original_write(path, value, mode)

    def selected_collector(state: Path, ports: Mapping[str, int], organization: str, streams: Mapping[str, str], *, configuration_id: str | None = None) -> dict[str, object]:
        result = original_collector(state, ports, organization, streams, configuration_id=configuration_id)
        if queue_size is not None:
            for exporter in mapping(result["exporters"]).values():
                queue = mapping(mapping(exporter)["sending_queue"])
                queue.update(queue_size=queue_size, num_consumers=1)
            mapping(result["processors"])["batch"] = {"timeout": "100ms", "send_batch_size": 1, "send_batch_max_size": 1}
        if bad_authorization:
            # This replaces only the Collector declaration before its digest is
            # sealed. service.env and the backend's private reader remain valid.
            for exporter in mapping(result["exporters"]).values():
                mapping(mapping(exporter)["headers"])["Authorization"] = "Basic Zml4dHVyZTppbnZhbGlk"
        if retry_expiry:
            # Same standard exporterhelper lifecycle with a short fixture budget,
            # declared before sealing. Ordinary service policy remains 300s.
            for exporter in mapping(result["exporters"]).values():
                mapping(mapping(exporter)["retry_on_failure"]).update(initial_interval="100ms", max_interval="200ms", max_elapsed_time="2s")
        if storage_failure:
            # The pinned extension validates that this exists and is a directory.
            # Seal a valid declaration, then change this owned data path before
            # service startup. The immutable configuration itself never changes.
            blocker = state / "queue-storage-blocker"
            blocker.mkdir(mode=0o700)
            mapping(mapping(result["extensions"])["file_storage"])["directory"] = str(blocker)
        return result

    with patch.object(service, "_write", selected_write), patch.object(service, "collector_configuration", selected_collector):
        yield


class Fixture:
    def __init__(self, state: Path, timeout: float) -> None:
        self.state, self.timeout = state, timeout
        self.state.mkdir(mode=0o700)
        info = self.state.stat()
        self.identity = (info.st_dev, info.st_ino, info.st_uid)
        self.configuration_id: str | None = None
        self.owner_id: str | None = None
        self.ports = fresh_ports()

    def configure(self, *, queue_size: int | None = None, storage_failure: bool = False, bad_authorization: bool = False, retry_expiry: bool = False) -> None:
        with fixture_configuration(queue_size=queue_size, storage_failure=storage_failure, bad_authorization=bad_authorization, retry_expiry=retry_expiry):
            image = service.setup(self.state, ports=self.ports, retention_days=7)
        self.configuration_id = str(image["configuration_id"])
        self.owner_id = str(image["owner_id"])
        if storage_failure:
            blocker = self.state / "queue-storage-blocker"
            require(blocker.is_dir() and not blocker.is_symlink(), "fixture.storage_fault_path_changed")
            blocker.rmdir()
            blocker.write_text("owned fixture: not a directory\n")
            blocker.chmod(0o600)

    def image(self) -> dict[str, object]:
        image = service.inspect(self.state)
        require(image.get("configuration_id") == self.configuration_id and image.get("owner_id") == self.owner_id, "fixture.service_generation_changed")
        return image

    def start(self) -> None:
        service.start(self.state)
        wait_for(lambda: telemetry.health(self.state), self.ready, self.timeout, "fixture.pipeline_not_ready")

    @staticmethod
    def ready(answer: dict[str, object]) -> bool:
        pipeline = mapping(answer.get("pipeline", {}))
        return all(mapping(pipeline.get(name, {})).get("status") == "responding" for name in ("collector", "backend"))

    def process(self, name: str) -> dict[str, object]:
        value = mapping(mapping(self.image()["services"])[name]).get("process")
        require(isinstance(value, dict), "fixture.process_identity_unavailable")
        return mapping(value)

    def unit_action(self, name: str, action: str) -> dict[str, object]:
        """Private fixture uses the service's exact ownership witness before effects."""
        require(name in {"collector", "openobserve"} and action in {"start", "stop"}, "fixture.unsupported_unit_action")
        config = service._configuration(self.state)
        unit = str(mapping(config["units"])[name])
        before = service._observe(unit, None)
        process = service._checked_process(config, name, before)
        if action == "stop":
            require(process is not None, "fixture.stop_requires_current_process")
        service._systemctl(action, unit)
        if action == "stop":
            after = service._observe(unit, None)
            require(after.get("MainPID", "0") == "0" and after.get("ActiveState") == "inactive", "fixture.unit_not_stopped")
            group = before.get("ControlGroup", "")
            events = Path("/sys/fs/cgroup") / group.lstrip("/") / "cgroup.events"
            require(not events.exists() or "populated 0" in events.read_text().splitlines(), "fixture.unit_children_remain")
            return process or {}
        return wait_for(lambda: self.process_or_none(name), bool, self.timeout, "fixture.unit_not_started")

    def process_or_none(self, name: str) -> dict[str, object]:
        try:
            return self.process(name)
        except (AcceptanceError, service.TelemetryServiceError, OSError):
            return {}

    def abrupt_collector_loss(self) -> tuple[dict[str, object], dict[str, object]]:
        config = service._configuration(self.state)
        unit = str(mapping(config["units"])["collector"])
        before = service._checked_process(config, "collector", service._observe(unit, None))
        require(before is not None, "fixture.kill_requires_current_collector")
        previous = mapping(before)
        # A pidfd pins the signalled process even if its numeric PID is recycled.
        descriptor = os.pidfd_open(int(cast("int", previous["pid"])))
        try:
            require(service._checked_process(config, "collector", service._observe(unit, None)) == previous, "fixture.collector_changed_before_signal")
            signal.pidfd_send_signal(descriptor, signal.SIGKILL)
        finally:
            os.close(descriptor)
        after = wait_for(lambda: self.process_or_none("collector"), lambda value: bool(value) and value != previous, self.timeout, "fixture.collector_not_restarted")
        wait_for(lambda: telemetry.health(self.state), lambda value: mapping(mapping(value["pipeline"]).get("collector", {})).get("status") == "responding", self.timeout, "fixture.restarted_collector_not_ready")
        return previous, after

    def journal_contains(self, name: str, expected: str) -> bool:
        config = service._configuration(self.state)
        require(name in {"collector", "openobserve"}, "fixture.journal_selection_invalid")
        unit = str(mapping(config["units"])[name])
        service._checked_process(config, name, service._observe(unit, None))
        result = subprocess.run(["journalctl", "--user", "--unit", unit, "--no-pager", "--output=json", "-n", "100"], text=True, capture_output=True, timeout=10, check=False)
        require(result.returncode == 0, "fixture.journal_unavailable")
        return any(expected in str(mapping(json.loads(line)).get("MESSAGE", "")) for line in result.stdout.splitlines())

    def cleanup(self, *, remove_state: bool) -> dict[str, object]:
        if not (self.state / "service.json").is_file():
            return {"services": "not_published", "state": "retained_unpublished"}
        # A setup exception may occur after publication but before inspect returns.
        # Re-establish only this minted private state's sealed service witness.
        config = service._configuration(self.state)
        if self.configuration_id is None and self.owner_id is None:
            self.configuration_id = str(config["configuration_id"])
            self.owner_id = str(config["owner_id"])
        self.image()
        service.stop(self.state)
        config = service._configuration(self.state)
        units = mapping(config["units"])
        directory = service._unit_directory(None)
        for unit in units.values():
            path = directory / str(unit)
            source = Path(str(config["generation"])) / str(unit)
            if path.exists():
                require(not path.is_symlink() and hashlib.sha256(path.read_bytes()).digest() == hashlib.sha256(source.read_bytes()).digest(), "fixture.unit_cleanup_identity_changed")
                path.unlink()
        service._systemctl("reset-failed", *(str(unit) for unit in units.values()), check=False)
        service._systemctl("daemon-reload")
        require(all(service._observe(str(unit), None).get("LoadState") == "not-found" for unit in units.values()), "fixture.unit_not_removed")
        info = self.state.lstat()
        require(not self.state.is_symlink() and (info.st_dev, info.st_ino, info.st_uid) == self.identity and info.st_uid == os.getuid(), "fixture.state_cleanup_identity_changed")
        if remove_state:
            shutil.rmtree(self.state)
        return {"services": "stopped_and_drained", "units": "authenticated_files_removed", "state": "removed" if remove_state else "retained_failure_diagnostics"}


class Backend:
    def __init__(self, fixture: Fixture) -> None:
        self.fixture = fixture

    def call(self, method: str, path: str, body: dict[str, object] | None = None) -> dict[str, object]:
        base, headers = service.backend_access(self.fixture.state)
        parsed = urlsplit(base)
        require(parsed.hostname == "127.0.0.1" and parsed.port == self.fixture.ports["backend_http"], "fixture.backend_identity_changed")
        connection = http.client.HTTPConnection(parsed.hostname, parsed.port, timeout=3)
        try:
            if body is not None:
                headers = {**headers, "Content-Type": "application/json"}
            connection.request(method, path, body=None if body is None else json.dumps(body, allow_nan=False).encode(), headers=headers)
            response = connection.getresponse()
            raw = response.read((1 << 20) + 1)
            require(response.status == 200, "fixture.backend_refused")
            require(len(raw) <= 1 << 20, "fixture.backend_response_bound")
            return mapping(json.loads(raw))
        finally:
            connection.close()

    def rows(self, signal_name: str, stream: str, sql: str, start: int, end: int) -> list[dict[str, object]]:
        require(signal_name in {"logs", "metrics"} and re.fullmatch(r"[a-z_][a-z0-9_]{0,127}", stream), "fixture.query_selection_invalid")
        try:
            answer = self.call("POST", f"/api/default/_search?type={signal_name}", {"query": {"sql": sql, "start_time": start, "end_time": end, "from": 0, "size": 100}, "timeout": 3})
            require(not answer.get("is_partial") and not answer.get("function_error"), "fixture.backend_partial")
            return records(answer.get("hits", []))
        except (AcceptanceError, ValueError, OSError, http.client.HTTPException):
            return []


def emit(state: Path, trace_id: str, phase: str, *, count: int = 1, age_days: int = 0, three_signals: bool = False) -> dict[str, object]:
    """Fresh SDK process; no handwritten OTLP or direct ingest qualification."""
    environment = {key: value for key, value in os.environ.items() if not key.startswith(("OTEL_", "PSE_TELEMETRY_"))}
    environment.update({"PSE_TELEMETRY_STATE": str(state), "OTEL_EXPORTER_OTLP_TIMEOUT": "2", "OTEL_BSP_MAX_QUEUE_SIZE": "64", "OTEL_BSP_MAX_EXPORT_BATCH_SIZE": "16", "OTEL_BSP_SCHEDULE_DELAY": "60000", "OTEL_METRIC_EXPORT_INTERVAL": "3600000"})
    if not three_signals:
        environment.update(OTEL_LOGS_EXPORTER="none", OTEL_METRICS_EXPORTER="none")
    command = [sys.executable, "-m", "scripts.tests.telemetry_pipeline_acceptance", "--emit", "--state", str(state), "--trace-id", trace_id, "--phase", phase, "--count", str(count), "--age-days", str(age_days)]
    result = subprocess.run(command, cwd=ROOT, env=environment, text=True, capture_output=True, timeout=30, check=False)
    require(result.returncode == 0, "fixture.sdk_producer_failed")
    answer = mapping(json.loads(result.stdout))
    require(answer.get("trace_id") == trace_id and answer.get("expected_parent_span_id") == PARENT_SPAN_ID, "fixture.producer_identity_changed")
    answer["producer_exit_code"] = result.returncode
    # Selected classification only: retain neither stderr text nor HTTP bodies.
    answer["sdk_stderr_evidence"] = {"connection_refused": "Connection refused" in result.stderr, "export_failure": "Exception while exporting" in result.stderr or "Failed to export" in result.stderr}
    return answer


def producer(args: argparse.Namespace) -> dict[str, object]:
    from opentelemetry import metrics, trace  # noqa: PLC0415
    from opentelemetry.trace.propagation.tracecontext import TraceContextTextMapPropagator  # noqa: PLC0415
    from scripts.telemetry_process import bootstrap  # noqa: PLC0415

    require(re.fullmatch(r"[0-9a-f]{32}", args.trace_id) and re.fullmatch(r"[a-z_]+", args.phase), "fixture.producer_selection_invalid")
    require(1 <= args.count <= 32 and args.age_days in {0, 4}, "fixture.producer_bound")
    start = time.time_ns() - args.age_days * 86400 * 1_000_000_000
    carrier = {"traceparent": f"00-{args.trace_id}-{PARENT_SPAN_ID}-01"}
    metric_name = "pse_pipeline_acceptance_" + args.trace_id
    span_ids = []
    with bootstrap("pse-telemetry-pipeline-fixture") as lifetime:
        require(lifetime.status == "initialized" and lifetime.settings is not None, "fixture.sdk_not_initialized")
        tracer = trace.get_tracer("pse.telemetry.acceptance", "1")
        for index in range(args.count):
            span = tracer.start_span("fixture." + args.phase, context=TraceContextTextMapPropagator().extract(carrier), start_time=start + index * 1000, attributes={"pse.fixture.phase": args.phase})
            with trace.use_span(span, end_on_exit=False, record_exception=False, set_status_on_exception=False):
                require(f"{span.get_span_context().trace_id:032x}" == args.trace_id, "fixture.producer_parent_trace_changed")
                span_ids.append(f"{span.get_span_context().span_id:016x}")
                if index == 0:
                    logging.getLogger("pse.telemetry.acceptance").warning("telemetry pipeline acceptance", extra={"pse.diagnostic.code": "fixture.pipeline", "pse.fixture.phase": args.phase})
                    metrics.get_meter("pse.telemetry.acceptance", "1").create_counter(metric_name, unit="1").add(3, {"stage": "execute", "pse.fixture.phase": args.phase})
            span.end(end_time=start + index * 1000 + 1_000_000)
        incarnation = dict(lifetime.settings.resource)["service.instance.id"]
    return {"trace_id": args.trace_id, "span_ids": span_ids, "expected_parent_span_id": PARENT_SPAN_ID, "service_instance_id": incarnation, "timestamp_us": start // 1000, "metric_name": metric_name, "phase": args.phase, "sdk_shutdown": "completed; backend delivery not inferred"}


def window(produced: dict[str, object]) -> tuple[int, int]:
    timestamp = int(cast("int", produced["timestamp_us"]))
    return max(0, timestamp - 60_000_000), timestamp + 60_000_000


def trace_read(fixture: Fixture, produced: dict[str, object]) -> dict[str, object]:
    start, end = window(produced)
    return telemetry_query.lookup_trace(fixture.state, str(produced["trace_id"]), start_time_us=start, end_time_us=end, limit=100)


def visible(fixture: Fixture, produced: dict[str, object]) -> dict[str, object]:
    expected = set(cast("list[str]", produced["span_ids"]))
    answer = wait_for(lambda: trace_read(fixture, produced), lambda value: value.get("status") == "available" and expected <= {row.get("span_id") for row in records(value.get("records", []))}, fixture.timeout, "fixture.known_trace_not_visible")
    matching = [row for row in records(answer["records"]) if row["span_id"] in expected]
    require(all(row.get("parent_span_id") == PARENT_SPAN_ID and row.get("service_instance_id") == produced["service_instance_id"] for row in matching), "fixture.backend_parent_or_incarnation_changed")
    return answer


def samples(health: dict[str, object], prefix: str, *, label: str = "exporter", selected: str = "otlp_http/traces") -> list[dict[str, object]]:
    delivery = mapping(mapping(health.get("pipeline", {})).get("collector_delivery", {}))
    return [item for item in records(delivery.get("samples", [])) if str(item.get("name", "")).startswith(prefix) and mapping(item.get("labels", {})).get(label) == selected]


def wait_counter(fixture: Fixture, prefix: str, minimum: float, *, label: str = "exporter", selected: str = "otlp_http/traces") -> dict[str, object]:
    return wait_for(lambda: telemetry.health(fixture.state), lambda value: sum(float(cast("float", item["value"])) for item in samples(value, prefix, label=label, selected=selected)) >= minimum, fixture.timeout, "fixture.collector_counter_not_observed")


def main_campaign(fixture: Fixture, receipt: dict[str, object]) -> None:
    progress("three SDK signals")
    baseline = emit(fixture.state, uuid.uuid4().hex, "baseline", three_signals=True)
    receipt["producer"] = baseline
    receipt["trace_visibility"] = visible(fixture, baseline)
    backend = Backend(fixture)
    start, end = window(baseline)
    trace_id = str(baseline["trace_id"])
    logs_sql = f"SELECT * FROM pse_logs WHERE trace_id = '{trace_id}'"
    logs = wait_for(lambda: backend.rows("logs", "pse_logs", logs_sql, start, end), bool, fixture.timeout, "fixture.sdk_log_not_visible")
    selected = next((row for row in logs if row.get("span_id") == cast("list[str]", baseline["span_ids"])[0]), None)
    require(selected is not None, "fixture.log_span_changed")
    log = mapping(selected)
    require(log.get("service_instance_id") == baseline["service_instance_id"] and log.get("pse_diagnostic_code") == "fixture.pipeline" and log.get("body") == "telemetry pipeline acceptance", "fixture.log_fields_changed")
    receipt["log_visibility"] = {key: log.get(key) for key in ("trace_id", "span_id", "service_instance_id", "pse_diagnostic_code", "body", "_timestamp")}
    metric = str(baseline["metric_name"])
    metric_sql = f"SELECT * FROM {metric} WHERE service_instance_id = '{baseline['service_instance_id']}'"
    points = wait_for(lambda: backend.rows("metrics", metric, metric_sql, start, end), bool, fixture.timeout, "fixture.sdk_metric_not_visible")
    require(any(row.get("value") == 3 and row.get("stage") == "execute" and row.get("service_instance_id") == baseline["service_instance_id"] for row in points), "fixture.metric_fields_changed")
    receipt["metric_visibility"] = {"stream": metric, "stream_rule": "backend metric name; stream-name header is not the metrics destination", "rows": [{key: row.get(key) for key in ("value", "start_time", "_timestamp", "service_instance_id", "stage")} for row in points], "general_metric_semantics": "not_established"}
    receipt["health_before_outage"] = telemetry.health(fixture.state)

    progress("backend outage and persistent queue")
    backend_before = fixture.unit_action("openobserve", "stop")
    # Occupy the two default exporter consumers before the target request, so a
    # queue-size observation can distinguish pending storage from in-flight retry.
    wakeups = []
    for phase in ("outage_first", "outage_second"):
        wakeups.append(emit(fixture.state, uuid.uuid4().hex, phase))
        time.sleep(1.25)
    graceful = emit(fixture.state, uuid.uuid4().hex, "graceful_queued")
    recovery: dict[str, object] = {"backend_before": backend_before, "graceful_target": graceful, "wakeups": wakeups}
    receipt["recovery"] = recovery
    receipt["queue_before_graceful"] = wait_counter(fixture, "otelcol_exporter_queue_size", 1)
    collector_before = fixture.unit_action("collector", "stop")
    collector_after = fixture.unit_action("collector", "start")
    wait_for(lambda: telemetry.health(fixture.state), lambda value: mapping(mapping(value["pipeline"]).get("collector", {})).get("status") == "responding", fixture.timeout, "fixture.graceful_collector_not_ready")
    require(collector_after != collector_before, "fixture.graceful_restart_incarnation_unchanged")
    recovery.update(collector_graceful_before=collector_before, collector_graceful_after=collector_after)
    # A positive gauge left by older records proves nothing about this target.
    # Let recovered consumers settle while the backend is still stopped, then
    # require the sole new producer to increase the persistent queue gauge.
    time.sleep(1.25)
    queue_before = telemetry.health(fixture.state)
    before_samples = samples(queue_before, "otelcol_exporter_queue_size")
    require(before_samples, "fixture.queue_baseline_unavailable")
    baseline_size = sum(float(cast("float", item["value"])) for item in before_samples)
    receipt["queue_before_abrupt_target"] = queue_before
    abrupt = emit(fixture.state, uuid.uuid4().hex, "abrupt_queued")
    recovery["abrupt_target"] = abrupt
    receipt["queue_before_abrupt"] = wait_counter(fixture, "otelcol_exporter_queue_size", baseline_size + 1)
    killed, restarted = fixture.abrupt_collector_loss()
    backend_after = fixture.unit_action("openobserve", "start")
    wait_for(lambda: telemetry.health(fixture.state), Fixture.ready, fixture.timeout, "fixture.recovery_not_ready")
    require(backend_after != backend_before, "fixture.backend_restart_incarnation_unchanged")
    recovery.update(collector_killed=killed, collector_restarted=restarted, backend_after=backend_after)
    recovery["graceful_visible"] = visible(fixture, graceful)
    recovery["abrupt_visible"] = visible(fixture, abrupt)
    recovery["previously_visible_after_restart"] = visible(fixture, baseline)
    recovery["claim"] = "these independently known queued and previously visible records survived these observed transitions; not exactly-once or universal durability"

    progress("controlled stream retention")
    aged = emit(fixture.state, uuid.uuid4().hex, "retention_aged", age_days=4)
    aged_before = visible(fixture, aged)
    retention: dict[str, object] = {"producer": aged, "before": aged_before, "global_retention_days": 7, "stream_retention_days": 3}
    receipt["retention"] = retention
    # A normal stream policy change, against only this disposable trace stream.
    backend.call("PUT", "/api/default/streams/pse_traces/settings?type=traces", {"data_retention": 3})
    expired = wait_for(lambda: trace_read(fixture, aged), lambda value: value.get("status") == "empty" and not value.get("records"), fixture.timeout, "fixture.controlled_retention_not_observed")
    retention.update(method="SDK-issued four-day-old span visible under seven-day global policy, then three-day stream retention with short fixture compactor intervals", after=expired, recent_record_still_visible=visible(fixture, baseline), physical_byte_erasure="not_established", claim="controlled query expiry after policy change; absence alone is not general execution evidence")


def overflow_campaign(fixture: Fixture, receipt: dict[str, object]) -> None:
    progress("separate queue size one fixture")
    fixture.unit_action("openobserve", "stop")
    produced = emit(fixture.state, uuid.uuid4().hex, "queue_overflow", count=16)
    receipt["producer"] = produced
    receipt["overflow_counter"] = wait_counter(fixture, "otelcol_exporter_enqueue_failed_", 1)
    fixture.unit_action("openobserve", "start")
    wait_for(lambda: telemetry.health(fixture.state), Fixture.ready, fixture.timeout, "fixture.overflow_recovery_not_ready")
    answer = wait_for(lambda: trace_read(fixture, produced), lambda value: bool(value.get("records")), fixture.timeout, "fixture.overflow_survivor_not_visible")
    observed = {row.get("span_id") for row in records(answer["records"])}
    expected = set(cast("list[str]", produced["span_ids"]))
    require(observed <= expected and 0 < len(observed) < len(expected), "fixture.overflow_not_discriminated")
    receipt["visibility"] = answer
    receipt["known_fixture_input_count"] = len(expected)
    receipt["observed_fixture_count"] = len(observed)
    receipt["claim"] = "finite standard queue rejected requests while backend unavailable; queue rejection and query visibility are separate observations"


def retry_expiry_campaign(fixture: Fixture, receipt: dict[str, object]) -> None:
    progress("bounded standard retry exhaustion")
    baseline = emit(fixture.state, uuid.uuid4().hex, "before_retry_expiry")
    receipt["baseline"] = visible(fixture, baseline)
    fixture.unit_action("openobserve", "stop")
    expired = emit(fixture.state, uuid.uuid4().hex, "retry_expired")
    receipt["expired_producer"] = expired
    receipt["receiver_acceptance"] = wait_counter(fixture, "otelcol_receiver_accepted_spans", 2, label="receiver", selected="otlp")
    wait_for(lambda: fixture.journal_contains("collector", "no more retries left"), bool, fixture.timeout, "fixture.retry_exhaustion_not_observed")
    wait_for(lambda: fixture.journal_contains("collector", "Exporting failed. Dropping data."), bool, fixture.timeout, "fixture.retry_drop_not_observed")
    receipt["exporter_failure"] = wait_counter(fixture, "otelcol_exporter_send_failed_spans", 1)

    def drained(value: dict[str, object]) -> bool:
        selected = samples(value, "otelcol_exporter_queue_size")
        return bool(selected) and all(float(cast("float", item["value"])) == 0 for item in selected)

    receipt["queue_after_drop"] = wait_for(lambda: telemetry.health(fixture.state), drained, fixture.timeout, "fixture.expired_queue_not_drained")
    fixture.unit_action("openobserve", "start")
    wait_for(lambda: telemetry.health(fixture.state), Fixture.ready, fixture.timeout, "fixture.expiry_recovery_not_ready")
    restored = emit(fixture.state, uuid.uuid4().hex, "after_retry_expiry")
    receipt["restored_visibility"] = visible(fixture, restored)
    receipt["baseline_after_restart"] = visible(fixture, baseline)
    absent = trace_read(fixture, expired)
    require(absent.get("status") == "empty" and not absent.get("records"), "fixture.expired_record_reappeared")
    receipt["expired_lookup"] = absent
    receipt["claim"] = "one known SDK batch exhausted the fixture's two-second standard retry budget and was explicitly dropped; restored ingestion and retained baseline were independently visible; this is retry exhaustion, not a queue-age TTL or universal interpretation of empty reads"


def delivery_failures_campaign(fixture: Fixture, receipt: dict[str, object]) -> None:
    progress("isolated invalid exporter credential")
    backend = Backend(fixture)
    backend.call("GET", "/api/default/streams?type=traces")
    receipt["backend_private_access"] = {"http_status": 200, "source": "authenticated selected backend stream-list read; body withheld"}
    produced = emit(fixture.state, uuid.uuid4().hex, "bad_exporter_credential")
    receipt["producer"] = produced
    receipt["receiver_acceptance"] = wait_counter(fixture, "otelcol_receiver_accepted_spans", 1, label="receiver", selected="otlp")
    receipt["exporter_failure"] = wait_counter(fixture, "otelcol_exporter_send_failed_spans", 1)
    wait_for(lambda: fixture.journal_contains("collector", "HTTP Status Code 401"), bool, fixture.timeout, "fixture.exporter_401_not_observed")
    receipt["exporter_http_evidence"] = {"http_status": 401, "source": "exact owned Collector journal contains pinned OTLP HTTP exporter status classification", "body_and_credentials": "withheld"}
    require(produced.get("producer_exit_code") == 0, "fixture.producer_result_changed_by_backend_refusal")

    progress("stopped Collector ingress refusal")
    receipt["collector_before_stop"] = fixture.unit_action("collector", "stop")
    refused = emit(fixture.state, uuid.uuid4().hex, "refused_collector_ingress")
    receipt["refused_ingress_producer"] = refused
    require(refused.get("producer_exit_code") == 0 and mapping(refused["sdk_stderr_evidence"]).get("connection_refused"), "fixture.sdk_refused_ingress_not_discriminated")
    health = telemetry.health(fixture.state)
    receipt["health_after_collector_stop"] = health
    pipeline = mapping(health["pipeline"])
    require(mapping(pipeline["backend"]).get("status") == "responding", "fixture.independent_backend_health_lost")
    require(mapping(pipeline["collector"]).get("status") == "unavailable", "fixture.stopped_collector_reported_available")
    require(mapping(health["delivery"]).get("backend_acceptance") == "unknown" and mapping(health["delivery"]).get("query_visibility") == "not_exercised", "fixture.failed_health_invented_delivery")
    backend.call("GET", "/api/default/streams?type=traces")
    receipt["claim"] = "real SDK producer completion is preserved under backend credential rejection and refused Collector ingress; receiver receipt, exporter HTTP failure and independent health remain distinct; native scientific result behavior is outside this fixture"


def run(output: Path, *, timeout: float, queue_overflow: bool, disk_failure: bool, selected_cases: tuple[str, ...] | None = None) -> int:
    output.mkdir(parents=True, mode=0o700, exist_ok=False)
    path = output / "acceptance.json"
    receipt: dict[str, object] = {"schema_version": 1, "scope": "actual Python SDK to pinned native Collector to OpenObserve; disposable owned user units", "baseline": 0, "status": "running", "started_at_ns": time.time_ns(), "sources": SOURCES, "fixture_backend_overrides": RETENTION_OVERRIDES, "cases": {}, "not_established": ["native PyO3/database continuity", "exactly-once delivery", "survival under arbitrary disk corruption or runtime ENOSPC", "scientific populations", "finite capture equivalence"]}
    cases = mapping(receipt["cases"])
    failures = 0
    chosen = selected_cases if selected_cases is not None else ("signals_recovery_retention", *(("queue_overflow",) if queue_overflow else ()), *(("storage_startup_failure",) if disk_failure else ()))
    receipt["selected_cases"] = list(chosen)
    receipt["not_run_cases"] = [name for name in CASES if name not in chosen]
    for name in chosen:
        item: dict[str, object] = {"status": "running"}
        cases[name] = item
        fixture = Fixture(output / (name + "-state"), timeout)
        passed = False
        cleanup_failed = False
        try:
            progress(name + ": configure")
            fixture.configure(queue_size=1 if name == "queue_overflow" else None, storage_failure=name == "storage_startup_failure", bad_authorization=name == "delivery_failures", retry_expiry=name == "retry_expiry")
            item["configuration_id"] = fixture.configuration_id
            item["ports"] = fixture.ports
            if name == "storage_startup_failure":
                # Process activation can precede component validation. Observe
                # this exact negative condition instead of waiting for normal
                # pipeline readiness through its entire timeout.
                try:
                    service.start(fixture.state)
                except service.TelemetryServiceError:
                    pass
                wait_for(lambda: fixture.journal_contains("collector", "is not a directory"), bool, timeout, "fixture.storage_failure_reason_not_observed")
                wait_for(lambda: fixture.process_or_none("collector"), lambda process: not process, timeout, "fixture.storage_failed_collector_still_running")
                image = fixture.image()
                item["observation"] = image
                item["claim"] = "persistent queue startup refuses an owned regular-file storage directory; running disk exhaustion/corruption not established"
            else:
                fixture.start()
                item["initial_service"] = fixture.image()
                if name == "queue_overflow":
                    overflow_campaign(fixture, item)
                elif name == "delivery_failures":
                    delivery_failures_campaign(fixture, item)
                elif name == "retry_expiry":
                    retry_expiry_campaign(fixture, item)
                else:
                    main_campaign(fixture, item)
            passed = True
            item["status"] = "passed"
        except Exception as error:
            failures += 1
            item["status"] = "failed"
            item["failure"] = str(error) if isinstance(error, AcceptanceError) else type(error).__name__
            progress(name + ": failed; retaining selected receipt and private state")
        finally:
            try:
                item["cleanup"] = fixture.cleanup(remove_state=passed)
            except Exception as error:
                failures += 1
                item["status"] = "failed"
                cleanup_failed = True
                item["cleanup"] = {"status": "failed", "cause": type(error).__name__, "remedy": "Inspect the exact retained private generation and authenticated unit owners before cleanup."}
            write_receipt(path, receipt)
        if cleanup_failed:
            receipt["remaining_optional_cases"] = "not_run_after_cleanup_failure"
            break
    receipt.update(status="passed" if failures == 0 else "failed", failure_count=failures, finished_at_ns=time.time_ns(), unselected_optional_cases=[name for name, selected in (("queue_overflow", queue_overflow), ("storage_startup_failure", disk_failure)) if not selected])
    write_receipt(path, receipt)
    print(json.dumps({"status": receipt["status"], "receipt": str(path), "baseline": 0, "failure_count": failures}))
    return 0 if failures == 0 else 1


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--timeout", type=float, default=90)
    parser.add_argument("--case", choices=CASES, action="append", dest="cases", help="Run exactly these selected cases; omit to retain the default campaign")
    parser.add_argument("--queue-overflow", action="store_true")
    parser.add_argument("--disk-failure", action="store_true", help="Owned startup storage-path failure; does not fill any disk")
    parser.add_argument("--emit", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--state", type=Path, help=argparse.SUPPRESS)
    parser.add_argument("--trace-id", default="", help=argparse.SUPPRESS)
    parser.add_argument("--phase", default="", help=argparse.SUPPRESS)
    parser.add_argument("--count", type=int, default=1, help=argparse.SUPPRESS)
    parser.add_argument("--age-days", type=int, default=0, help=argparse.SUPPRESS)
    args = parser.parse_args(argv)
    if args.emit:
        require(args.state is not None, "fixture.producer_state_missing")
        print(json.dumps(producer(args), allow_nan=False))
        return 0
    if args.output is None or not 10 <= args.timeout <= 180:
        parser.error("select a new --output directory and a timeout from 10 to 180 seconds")
    if args.cases is not None and (args.queue_overflow or args.disk_failure or len(args.cases) != len(set(args.cases))):
        parser.error("--case selections must be unique and cannot combine with the optional-case flags")
    return run(args.output.absolute(), timeout=args.timeout, queue_overflow=args.queue_overflow, disk_failure=args.disk_failure, selected_cases=tuple(args.cases) if args.cases is not None else None)


if __name__ == "__main__":
    raise SystemExit(main())
