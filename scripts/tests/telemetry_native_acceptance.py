# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit acceptance of the linked public Python/Rust telemetry boundary.

Run after py-sync-native-correctness, through scripts/pse-env --native. This
finite loopback receiver is a test consumer, not a second production delivery path.
It neither starts a canonical store nor claims private-child/database continuity.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TRACE_A = "1234567890abcdef1234567890abcdef"
TRACE_B = "abcdef1234567890abcdef1234567890"
PARENT = "1234567890abcdef"


def require(value: object, code: str) -> None:
    if not value:
        raise RuntimeError(code)


def producer() -> dict[str, object]:
    import tempfile  # noqa: PLC0415
    from unittest.mock import patch  # noqa: PLC0415

    from opentelemetry import context, trace  # noqa: PLC0415
    from opentelemetry.trace.propagation.tracecontext import (  # noqa: PLC0415 -- SDK loads only in the manual producer.
        TraceContextTextMapPropagator,
    )

    import pse  # noqa: PLC0415
    from pse.contracts.enums import (  # noqa: PLC0415
        NativeBackend,
        NativeSolveIntent,
        PresolvePolicyKind,
    )
    from pse.tests.test_local_runtime import physical_documents  # noqa: PLC0415
    from pse.tests.test_native_workflow import package_documents  # noqa: PLC0415
    from scripts import telemetry_process  # noqa: PLC0415

    invalid_carrier = False
    current_carrier = telemetry_process.current_carrier

    def injector() -> dict[str, str]:
        return {"traceparent": "invalid"} if invalid_carrier else current_carrier()

    with tempfile.TemporaryDirectory(prefix="pse-telemetry-native-") as spill:
        settings = pse.EngineSettings(
            memory_limit_bytes=64 << 30,
            threads=2,
            spill_dir=spill,
            max_spill_bytes=1 << 30,
            batch_size=32,
        )
        with (
            patch.object(telemetry_process, "current_carrier", injector),
            telemetry_process.bootstrap(
                "pse-native-boundary-acceptance", native=True
            ) as lifetime,
        ):
            disabled = os.environ.get("OTEL_SDK_DISABLED") == "true"
            require(
                lifetime.status == ("disabled" if disabled else "initialized"),
                "native_fixture.bootstrap_failed",
            )
            if not disabled:
                require(
                    lifetime.native is not None
                    and lifetime.native.activation.get("status") == "initialized",
                    "native_fixture.bridge_not_initialized",
                )
            runtime = pse.Runtime(settings, ephemeral=True)
            try:
                physical = runtime.physical_from_documents(physical_documents())
                package = None
                for target, trace_id in zip((3, 4), (TRACE_A, TRACE_B), strict=True):
                    external = TraceContextTextMapPropagator().extract(
                        {"traceparent": f"00-{trace_id}-{PARENT}-01"}
                    )
                    token = context.attach(external)
                    try:
                        # A Python SDK record independently exposes its resource; the native
                        # call below still receives the explicitly selected external parent.
                        trace.get_tracer("pse.telemetry.acceptance").start_span(
                            "fixture.python"
                        ).end()
                        sources = package_documents(
                            "package telemetry { def Root { var x:Scalar; eq balance:x=="
                            + str(target)
                            + "; annotation start x(0); } }"
                        )
                        if package is None:
                            package = runtime.modeling_from_documents(
                                [sources], physical
                            )
                        else:
                            previous = package.revision_id
                            package = package.reload_from_parent([sources])
                            require(
                                package.revision_id != previous,
                                "native_fixture.edit_did_not_change_revision",
                            )
                        require(
                            package.reload_from_parent([sources]).revision_id
                            == package.revision_id,
                            "native_fixture.unchanged_source_not_reused",
                        )
                        root = next(
                            row.declaration_id
                            for row in package.declarations()
                            if row.name == "Root"
                        )
                        prepared = package.prepare_solve(
                            root,
                            pse.SolveSettings(
                                intent=NativeSolveIntent.ROOT,
                                backend=NativeBackend.KINSOL,
                                presolve=PresolvePolicyKind.OFF,
                            ),
                        )
                        require(
                            runtime.start([prepared, prepared]).wait().usable,
                            "native_fixture.solve_not_usable",
                        )
                        stream = pse.registry_table(
                            "reference.schema_relations", settings=settings
                        )
                        try:
                            require(
                                stream.next_batch() is not None,
                                "native_fixture.registry_pull_empty",
                            )
                        finally:
                            stream.close()
                    finally:
                        context.detach(token)
                # Invalid context is refused as diagnostic data, never scientific input.
                invalid_carrier = True
                stream = pse.registry_table(
                    "reference.schema_relations", settings=settings
                )
                try:
                    require(
                        stream.next_batch() is not None,
                        "native_fixture.unparented_pull_failed",
                    )
                finally:
                    stream.close()
            finally:
                runtime.close()
            instance = (
                dict(lifetime.settings.resource)["service.instance.id"]
                if lifetime.settings is not None
                else None
            )
        return {
            "native_result": "usable",
            "pid": os.getpid(),
            "service_instance_id": instance,
            "sdk_status": lifetime.status,
            "changed_source_targets": [3, 4],
            "unchanged_revision_reused": True,
        }


def inspect_records(
    records: list[tuple[str, bytes]], produced: dict[str, object]
) -> dict[str, object]:
    from opentelemetry.proto.collector.logs.v1.logs_service_pb2 import (  # noqa: PLC0415 -- Decode only during explicit acceptance.
        ExportLogsServiceRequest,
    )
    from opentelemetry.proto.collector.trace.v1.trace_service_pb2 import (  # noqa: PLC0415 -- Decode only during explicit acceptance.
        ExportTraceServiceRequest,
    )

    spans = []
    logs = []
    resources = []
    for path, content in records:
        if path == "/v1/traces":
            request = ExportTraceServiceRequest.FromString(content)
            for resource in request.resource_spans:
                pid = next(
                    item.value
                    for item in resource.resource.attributes
                    if item.key == "process.pid"
                )
                require(
                    pid.HasField("int_value"), "native_fixture.process_pid_type_changed"
                )
                attributes = {
                    item.key: item.value.int_value
                    if item.key == "process.pid"
                    else item.value.string_value
                    for item in resource.resource.attributes
                }
                resources.append(attributes)
                spans.extend(
                    span for scope in resource.scope_spans for span in scope.spans
                )
        elif path == "/v1/logs":
            request = ExportLogsServiceRequest.FromString(content)
            for resource in request.resource_logs:
                pid = next(
                    item.value
                    for item in resource.resource.attributes
                    if item.key == "process.pid"
                )
                require(
                    pid.HasField("int_value"), "native_fixture.process_pid_type_changed"
                )
                resources.append(
                    {
                        item.key: item.value.int_value
                        if item.key == "process.pid"
                        else item.value.string_value
                        for item in resource.resource.attributes
                    }
                )
                logs.extend(
                    record
                    for scope in resource.scope_logs
                    for record in scope.log_records
                )
    native = [span for span in spans if span.name == "pse.native.operation"]
    for trace_id in (TRACE_A, TRACE_B):
        selected = [span for span in native if span.trace_id.hex() == trace_id]
        require(
            selected and all(span.parent_span_id.hex() == PARENT for span in selected),
            "native_fixture.parent_identity_changed",
        )
        pulls = {
            span.span_id.hex()
            for span in selected
            if any(
                item.key == "owner" and item.value.string_value == "python.native_pull"
                for item in span.attributes
            )
        }
        require(pulls, "native_fixture.native_pull_not_exported")
        require(
            any(
                log.trace_id.hex() == trace_id
                and log.span_id.hex() in pulls
                and any(
                    item.key == "code"
                    and item.value.string_value == "native.drain.settled"
                    for item in log.attributes
                )
                for log in logs
            ),
            "native_fixture.drain_log_not_correlated",
        )
    require(
        any(
            span.trace_id.hex() not in {TRACE_A, TRACE_B}
            and not any(span.parent_span_id)
            and any(
                item.key == "owner" and item.value.string_value == "python.native_pull"
                for item in span.attributes
            )
            for span in native
        ),
        "native_fixture.previous_parent_leaked",
    )
    require(
        {attributes.get("telemetry.sdk.language") for attributes in resources}
        >= {"python", "rust"},
        "native_fixture.language_resources_missing",
    )
    require(
        all(
            attributes.get("service.instance.id") == produced["service_instance_id"]
            and attributes.get("process.pid") == produced["pid"]
            for attributes in resources
        ),
        "native_fixture.process_identity_changed",
    )
    return {
        "native_spans": len(native),
        "log_records": len(logs),
        "independent_parents": [TRACE_A, TRACE_B],
        "shared_process_resource": True,
        "scope": "real ephemeral public solve and controlled table pull; no canonical store or private native-child claim",
    }


def receiver_handler(
    received: list[tuple[str, bytes]], status: int, delay: float
) -> type[BaseHTTPRequestHandler]:
    """Bind a receiver's case values for the lifetime of its server threads."""

    class Handler(BaseHTTPRequestHandler):
        def do_POST(self) -> None:
            received.append(
                (self.path, self.rfile.read(int(self.headers["Content-Length"])))
            )
            if delay:
                time.sleep(delay)
            self.send_response(status)
            self.send_header("Content-Length", "0")
            self.end_headers()

        def log_message(self, format: str, *args: object) -> None:
            del format, args

    return Handler


def run(output: Path) -> int:
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    receipt: dict[str, object] = {
        "baseline": 0,
        "scope": "linked public Python/PyO3/Rust telemetry",
        "cases": [],
    }
    cases: list[dict[str, object]] = []
    receipt["cases"] = cases
    failures = 0
    for name, status, delay in (
        ("enabled", 200, 0.0),
        ("disabled", 200, 0.0),
        ("refused", 503, 0.0),
        ("delayed", 200, 0.6),
    ):
        received: list[tuple[str, bytes]] = []

        server = ThreadingHTTPServer(
            ("127.0.0.1", 0), receiver_handler(received, status, delay)
        )
        server.daemon_threads = True
        thread = threading.Thread(
            target=server.serve_forever, kwargs={"poll_interval": 0.05}, daemon=True
        )
        thread.start()
        item: dict[str, object] = {"case": name}
        cases.append(item)
        try:
            environment = {
                key: value
                for key, value in os.environ.items()
                if not key.startswith(("OTEL_", "PSE_TELEMETRY_"))
            }
            environment.update(
                {
                    "PSE_TELEMETRY_STATE": str(output / "unconfigured"),
                    "OTEL_EXPORTER_OTLP_ENDPOINT": f"http://127.0.0.1:{server.server_port}",
                    "OTEL_EXPORTER_OTLP_TIMEOUT": "0.15",
                    "OTEL_BSP_EXPORT_TIMEOUT": "1000",
                    "OTEL_BSP_SCHEDULE_DELAY": "60000",
                    "OTEL_METRICS_EXPORTER": "none",
                }
            )
            if name == "disabled":
                environment["OTEL_SDK_DISABLED"] = "true"
            started = time.monotonic()
            child = subprocess.run(
                [
                    sys.executable,
                    "-m",
                    "scripts.tests.telemetry_native_acceptance",
                    "--produce",
                ],
                cwd=ROOT,
                env=environment,
                capture_output=True,
                text=True,
                timeout=90,
                check=False,
            )
            (output / (name + "-stderr.txt")).write_text(child.stderr)
            require(child.returncode == 0, "native_fixture.public_operation_failed")
            produced = json.loads(child.stdout)
            item.update(produced=produced, elapsed_seconds=time.monotonic() - started)
            require(
                produced["native_result"] == "usable", "native_fixture.outcome_changed"
            )
            if name == "enabled":
                item["telemetry"] = inspect_records(received, produced)
            elif name == "disabled":
                require(not received, "native_fixture.disabled_exported")
            else:
                require(received, "native_fixture.failure_transport_not_exercised")
                item["delivery"] = "not_established; native result preserved"
            item["status"] = "passed"
        except Exception as error:
            failures += 1
            item.update(
                status="failed",
                error=str(error)
                if isinstance(error, RuntimeError)
                else type(error).__name__,
            )
        finally:
            server.shutdown()
            server.server_close()
            thread.join()
        print(
            "native telemetry acceptance: " + name + " " + str(item["status"]),
            file=sys.stderr,
            flush=True,
        )
    receipt.update(
        status="passed" if failures == 0 else "failed", failure_count=failures
    )
    (output / "acceptance.json").write_text(
        json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    )
    print(
        json.dumps(
            {
                "status": receipt["status"],
                "failure_count": failures,
                "receipt": str(output / "acceptance.json"),
            }
        )
    )
    return 0 if failures == 0 else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--produce", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.produce:
        print(json.dumps(producer()))
        return 0
    if args.output is None:
        parser.error("select a fresh private --output directory")
    return run(args.output.absolute())


if __name__ == "__main__":
    raise SystemExit(main())
