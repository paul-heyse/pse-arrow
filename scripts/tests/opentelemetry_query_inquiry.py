# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Disposable OpenObserve OSS retrieval inquiry; no project services or data.

Downloads one exact native binary into TemporaryDirectory, sends synthetic OTLP
JSON, queries the stored representations, then waits for its exact child to exit.
This is a protocol fixture, not an application exporter or backend adapter.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import http.client
import json
import os
import secrets
import socket
import subprocess
import tarfile
import tempfile
import time
import urllib.parse
import urllib.request
from pathlib import Path

ARCHIVE = (
    "https://downloads.openobserve.ai/releases/openobserve/v1.1.2/"
    "openobserve-v1.1.2-linux-amd64.tar.gz"
)
TRACE = "a" * 32
SPAN = "0" * 15 + "1"


def attribute(key: str, value: str) -> dict[str, object]:
    return {"key": key, "value": {"stringValue": value}}


def ports() -> tuple[int, int]:
    with socket.socket() as first, socket.socket() as second:
        first.bind(("127.0.0.1", 0))
        second.bind(("127.0.0.1", 0))
        return first.getsockname()[1], second.getsockname()[1]


class Backend:
    def __init__(self, port: int, email: str, password: str) -> None:
        self.base = f"http://127.0.0.1:{port}"
        self.authorization = (
            "Basic " + base64.b64encode(f"{email}:{password}".encode()).decode()
        )

    def call(
        self,
        path: str,
        body: dict[str, object] | None = None,
        stream: str | None = None,
    ) -> dict[str, object]:
        headers = {"Authorization": self.authorization}
        if body is not None:
            headers["Content-Type"] = "application/json"
        if stream is not None:
            headers["stream-name"] = stream
        connection = http.client.HTTPConnection(
            urllib.parse.urlsplit(self.base).netloc, timeout=12
        )
        try:
            connection.request(
                "GET" if body is None else "POST",
                path,
                body=None if body is None else json.dumps(body).encode(),
                headers=headers,
            )
            response = connection.getresponse()
            raw, status = response.read(1 << 20), response.status
        finally:
            connection.close()
        try:
            parsed = json.loads(raw)
        except ValueError:
            parsed = raw.decode(errors="replace")[:4096]
        return {"status": status, "body": parsed}

    def sql(
        self, kind: str, sql: str, start: int, end: int, offset: int = 0, size: int = 2
    ) -> dict[str, object]:
        return self.call(
            f"/api/default/_search?type={kind}",
            {
                "query": {
                    "sql": sql,
                    "start_time": start,
                    "end_time": end,
                    "from": offset,
                    "size": size,
                },
                "timeout": 10,
            },
        )


def hits(reply: dict[str, object]) -> list[object]:
    body = reply.get("body")
    if not isinstance(body, dict):
        return []
    result = body.get("hits", [])
    return result if isinstance(result, list) else []


def mapping(value: object) -> dict:
    if not isinstance(value, dict):
        raise TypeError("Expected a fixture response object")
    return value


def sequence(value: object) -> list[object]:
    if not isinstance(value, list):
        raise TypeError("Expected a fixture response array")
    return value


def validate_receipt(receipt: dict[str, object]) -> dict[str, object]:
    """Check fixture identity and query visibility, not general metric fidelity."""
    for name in ("traces_ingest", "logs_ingest"):
        assert mapping(receipt[name])["status"] == 200
    for reply in sequence(receipt["metrics_ingest"]):
        assert mapping(reply)["status"] == 200
    pages = sequence(receipt["trace_pages"])
    rows = [mapping(row) for page in pages for row in hits(mapping(page))]
    assert len(rows) == 5
    assert {row["span_id"] for row in rows} == {
        f"{index:016x}" for index in range(1, 6)
    }
    assert all(row["trace_id"] == TRACE for row in rows)
    assert all(row["pse_operation_id"] == "fixture-operation" for row in rows)
    assert all(row["service_service_instance_id"] == "fixture-one" for row in rows)
    assert all(
        row.get("reference_parent_span_id") == SPAN
        for row in rows
        if row["span_id"] != SPAN
    )
    aggregate = hits(mapping(receipt["trace_aggregate"]))
    assert len(aggregate) == 1
    assert mapping(aggregate[0])["span_count"] == 5
    logs = hits(mapping(receipt["logs_query"]))
    assert len(logs) == 1
    assert mapping(logs[0])["trace_id"] == TRACE
    assert mapping(logs[0])["span_id"] == f"{5:016x}"
    assert mapping(logs[0])["pse_diagnostic_code"] == "fixture.refused"
    counters = [mapping(row) for row in hits(mapping(receipt["counter_rows"]))]
    assert [row["value"] for row in counters] == [1.0, 4.0, 9.0, 2.0]
    assert len({row["start_time"] for row in counters[:3]}) == 1
    assert counters[-1]["start_time"] != counters[0]["start_time"]
    for response in mapping(receipt["promql"]).values():
        assert mapping(response)["status"] == 200
        body = mapping(mapping(response)["body"])
        assert body["status"] == "success"
        assert mapping(body["data"])["result"]
    exemplars = mapping(mapping(receipt["exemplars"])["body"])["data"]
    assert exemplars
    for series in exemplars:
        for exemplar in mapping(series)["exemplars"]:
            labels = mapping(mapping(exemplar)["labels"])
            assert labels["trace_id"] == TRACE
            assert labels["span_id"] == SPAN
    return {
        "baseline": 0,
        "assertion_failures": 0,
        "trace_fixture_spans": 5,
        "retrieved_spans": len(rows),
        "semantic_checks": [
            "all three signal ingests returned HTTP 200",
            "five unique span IDs, trace/operation/process association and four parent edges",
            "SQL aggregate agrees with independently constructed fixture",
            "correlated diagnostic log retains trace/span/code",
            "SQL cumulative counter values and changed reset start retained",
            "four PromQL requests returned nonempty successful results",
            "exemplar pointers retain submitted trace/span identities",
        ],
        "not_established": [
            "point-specific start_time through PromQL",
            "exemplar membership in each returned bucket",
            "general PromQL rate/reset/histogram correctness",
            "application or Collector propagation, durability or restart recovery",
        ],
    }


def run(output: Path) -> None:
    output.mkdir(parents=True, exist_ok=False)
    receipt: dict[str, object] = {
        "scope": "synthetic OTLP JSON to disposable OpenObserve OSS 1.1.2",
        "archive_url": ARCHIVE,
        "application_continuity": "not exercised",
        "collector": "not exercised",
        "checks": {"baseline": 0},
    }
    with tempfile.TemporaryDirectory(prefix="pse-otel-query-review-") as directory:
        scratch = Path(directory)
        archive = scratch / "backend.tar.gz"
        with (
            urllib.request.urlopen(ARCHIVE, timeout=60) as download,
            archive.open("wb") as dest,
        ):
            while block := download.read(1 << 20):
                dest.write(block)
        with archive.open("rb") as downloaded:
            receipt["archive_sha256"] = hashlib.file_digest(
                downloaded, "sha256"
            ).hexdigest()
        binary = scratch / "openobserve"
        with tarfile.open(archive) as packed:
            candidates = [
                m for m in packed if m.isfile() and Path(m.name).name == "openobserve"
            ]
            if len(candidates) != 1:
                raise ValueError("Expected exactly one native OpenObserve binary")
            source = packed.extractfile(candidates[0])
            if source is None:
                raise ValueError("Missing binary archive body")
            with binary.open("wb") as dest:
                while block := source.read(1 << 20):
                    dest.write(block)
        binary.chmod(0o700)
        http_port, grpc_port = ports()
        email, password = "review@example.invalid", secrets.token_urlsafe(32)
        env = {
            "PATH": os.defpath,
            "HOME": directory,
            "LANG": "C.UTF-8",
            "ZO_LOCAL_MODE": "true",
            "ZO_LOCAL_MODE_STORAGE": "disk",
            "ZO_META_STORE": "sqlite",
            "ZO_QUEUE_STORE": "memory",
            "ZO_DATA_DIR": str(scratch / "data") + "/",
            "ZO_HTTP_ADDR": "127.0.0.1",
            "ZO_HTTP_PORT": str(http_port),
            "ZO_GRPC_ADDR": "127.0.0.1",
            "ZO_GRPC_PORT": str(grpc_port),
            "ZO_WEB_URL": f"http://127.0.0.1:{http_port}",
            "ZO_TELEMETRY": "false",
            "ZO_ROOT_USER_EMAIL": email,
            "ZO_ROOT_USER_PASSWORD": password,
            "ZO_HTTP_WORKER_NUM": "2",
            "ZO_QUERY_THREAD_NUM": "2",
            "ZO_GRPC_RUNTIME_WORKER_NUM": "2",
        }
        backend = Backend(http_port, email, password)
        log_path = scratch / "backend.log"
        with log_path.open("wb") as log:
            child = subprocess.Popen(
                [str(binary)], cwd=scratch, env=env, stdout=log, stderr=log
            )
            receipt["child_pid"] = child.pid
            try:
                deadline = time.monotonic() + 45
                while True:
                    assert child.poll() is None, (
                        f"Backend exited at startup: {child.returncode}"
                    )
                    try:
                        ready = backend.call("/healthz")
                        if ready["status"] == 200:
                            break
                    except OSError:
                        pass
                    assert time.monotonic() < deadline, (
                        "Backend readiness exceeded 45 seconds"
                    )
                    time.sleep(0.1)
                now = time.time_ns()
                start, end = now // 1000 - 60_000_000, now // 1000 + 60_000_000
                resource = {
                    "attributes": [
                        attribute("service.name", "review-fixture"),
                        attribute("service.instance.id", "fixture-one"),
                    ]
                }
                spans: list[dict[str, object]] = []
                for index in range(1, 6):
                    span: dict[str, object] = {
                        "traceId": TRACE,
                        "spanId": f"{index:016x}",
                        "name": "fixture-root" if index == 1 else "fixture-child",
                        "kind": 1,
                        "startTimeUnixNano": str(now - 1_000_000 + index * 10_000),
                        "endTimeUnixNano": str(now + index * 10_000),
                        "attributes": [
                            attribute("pse.operation.id", "fixture-operation"),
                            attribute(
                                "pse.stage", "prepare" if index < 3 else "execute"
                            ),
                        ],
                        "status": {"code": 2 if index == 5 else 0},
                    }
                    if index > 1:
                        span["parentSpanId"] = SPAN
                    spans.append(span)
                receipt["traces_ingest"] = backend.call(
                    "/api/default/v1/traces",
                    {
                        "resourceSpans": [
                            {
                                "resource": resource,
                                "scopeSpans": [
                                    {
                                        "scope": {"name": "review-fixture"},
                                        "spans": spans,
                                    }
                                ],
                            }
                        ]
                    },
                    "review_traces",
                )
                receipt["logs_ingest"] = backend.call(
                    "/api/default/v1/logs",
                    {
                        "resourceLogs": [
                            {
                                "resource": resource,
                                "scopeLogs": [
                                    {
                                        "scope": {"name": "review-fixture"},
                                        "logRecords": [
                                            {
                                                "timeUnixNano": str(now),
                                                "severityNumber": 17,
                                                "severityText": "ERROR",
                                                "body": {
                                                    "stringValue": "synthetic classified refusal"
                                                },
                                                "traceId": TRACE,
                                                "spanId": f"{5:016x}",
                                                "attributes": [
                                                    attribute(
                                                        "pse.diagnostic.code",
                                                        "fixture.refused",
                                                    )
                                                ],
                                            }
                                        ],
                                    }
                                ],
                            }
                        ]
                    },
                    "review_logs",
                )
                metric_receipts: list[object] = []
                for offset, value, reset in (
                    (-30, 1, False),
                    (-20, 4, False),
                    (-10, 9, False),
                    (-5, 2, True),
                ):
                    timestamp = now + offset * 1_000_000_000
                    started = now - (6 if reset else 40) * 1_000_000_000
                    base: dict[str, object] = {
                        "startTimeUnixNano": str(started),
                        "timeUnixNano": str(timestamp),
                        "attributes": [attribute("stage", "execute")],
                    }
                    counter = {
                        "name": "review_counter",
                        "unit": "1",
                        "sum": {
                            "aggregationTemporality": 2,
                            "isMonotonic": True,
                            "dataPoints": [dict(base, asInt=str(value))],
                        },
                    }
                    histogram = {
                        "name": "review_duration",
                        "unit": "s",
                        "histogram": {
                            "aggregationTemporality": 2,
                            "dataPoints": [
                                dict(
                                    base,
                                    count=str(value),
                                    sum=value * 0.2,
                                    explicitBounds=[0.1, 1.0],
                                    bucketCounts=["0", str(value), "0"],
                                    min=0.2,
                                    max=0.2,
                                    exemplars=[
                                        {
                                            "timeUnixNano": str(timestamp),
                                            "asDouble": 0.2,
                                            "traceId": TRACE,
                                            "spanId": SPAN,
                                        }
                                    ],
                                )
                            ],
                        },
                    }
                    metric_receipts.append(
                        backend.call(
                            "/api/default/v1/metrics",
                            {
                                "resourceMetrics": [
                                    {
                                        "resource": resource,
                                        "scopeMetrics": [
                                            {
                                                "scope": {"name": "review-fixture"},
                                                "metrics": [counter, histogram],
                                            }
                                        ],
                                    }
                                ]
                            },
                        )
                    )
                receipt["metrics_ingest"] = metric_receipts
                query = f"SELECT * FROM review_traces WHERE trace_id = '{TRACE}' ORDER BY start_time"  # noqa: S608 -- TRACE is the fixed synthetic fixture constant, never caller input.
                deadline = time.monotonic() + 40
                pages: list[dict[str, object]] = []
                while True:
                    pages = [
                        backend.sql("traces", query, start, end, offset)
                        for offset in (0, 2, 4)
                    ]
                    if sum(len(hits(page)) for page in pages) == 5:
                        break
                    assert time.monotonic() < deadline, (
                        "Five trace records did not become query-visible"
                    )
                    time.sleep(0.25)
                receipt["trace_pages"] = pages
                receipt["trace_aggregate"] = backend.sql(
                    "traces",
                    f"SELECT COUNT(*) AS span_count FROM review_traces WHERE trace_id = '{TRACE}'",  # noqa: S608 -- fixed synthetic fixture identity only.
                    start,
                    end,
                )
                receipt["logs_query"] = backend.sql(
                    "logs",
                    f"SELECT * FROM review_logs WHERE trace_id = '{TRACE}'",  # noqa: S608 -- fixed synthetic fixture identity only.
                    start,
                    end,
                )
                receipt["counter_rows"] = backend.sql(
                    "metrics",
                    "SELECT * FROM review_counter ORDER BY _timestamp",
                    start,
                    end,
                    size=10,
                )
                receipt["histogram_rows"] = backend.sql(
                    "metrics",
                    "SELECT * FROM review_duration_bucket ORDER BY _timestamp",
                    start,
                    end,
                    size=16,
                )
                metric_queries = (
                    "review_counter",
                    "increase(review_counter[1m])",
                    "review_duration_count",
                    "histogram_quantile(0.95,sum by(le)(rate(review_duration_bucket[1m])))",
                )
                prom: dict[str, object] = {}
                for query_text in metric_queries:
                    query_args = urllib.parse.urlencode(
                        {
                            "query": query_text,
                            "start": now / 1e9 - 35,
                            "end": now / 1e9 + 1,
                            "step": "1s",
                        }
                    )
                    prom[query_text] = backend.call(
                        "/api/default/prometheus/api/v1/query_range?" + query_args
                    )
                receipt["promql"] = prom
                exemplar_args = urllib.parse.urlencode(
                    {
                        "query": "review_duration_bucket",
                        "start": now / 1e9 - 35,
                        "end": now / 1e9 + 1,
                    }
                )
                receipt["exemplars"] = backend.call(
                    "/api/default/prometheus/api/v1/query_exemplars?" + exemplar_args
                )
                receipt["checks"] = validate_receipt(receipt)
            except Exception as error:
                receipt["error"] = f"{type(error).__name__}: {error}"
                raise
            finally:
                child.terminate()
                try:
                    child.wait(timeout=15)
                    receipt["cleanup"] = (
                        "exact child terminated and reaped; temporary state removed"
                    )
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=5)
                    receipt["cleanup"] = (
                        "exact child killed and reaped; temporary state removed"
                    )
                receipt["child_returncode"] = child.returncode
                with log_path.open("rb") as saved:
                    saved.seek(max(0, log_path.stat().st_size - 8192))
                    receipt["backend_log_tail"] = (
                        saved.read()
                        .decode(errors="replace")
                        .replace(password, "<fixture-password>")
                    )
                (output / "results.json").write_text(
                    json.dumps(receipt, indent=2, sort_keys=True) + "\n"
                )
    print(output / "results.json")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    run(parser.parse_args().output)
