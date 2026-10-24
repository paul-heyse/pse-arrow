# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit real private-native-binary telemetry and outcome acceptance.

The native selector prepares the exact executable. Original 2/4/2 numerical,
reservation and process-reap assertions remain in their Rust fixture owner.
Short-child telemetry is best effort; missing records never become continuity proof.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import threading
import time
from http.server import ThreadingHTTPServer
from pathlib import Path

from scripts.tests.telemetry_native_acceptance import (
    PARENT,
    ROOT,
    TRACE_A,
    receiver_handler,
    require,
)


def inspect_records(records: list[tuple[str, bytes]]) -> dict[str, object]:
    from opentelemetry.proto.collector.trace.v1.trace_service_pb2 import (  # noqa: PLC0415 -- Decode only during explicit acceptance.
        ExportTraceServiceRequest,
    )

    observed = []
    for path, raw in records:
        if path != "/v1/traces":
            continue
        request = ExportTraceServiceRequest.FromString(raw)
        for resource in request.resource_spans:
            pid = next(
                item.value
                for item in resource.resource.attributes
                if item.key == "process.pid"
            )
            require(pid.HasField("int_value"), "child_fixture.process_pid_type_changed")
            attributes = {
                item.key: str(item.value.int_value)
                if item.key == "process.pid"
                else item.value.string_value
                for item in resource.resource.attributes
            }
            observed.extend(
                (attributes, span)
                for scope in resource.scope_spans
                for span in scope.spans
            )
    parents = [
        (attributes, span)
        for attributes, span in observed
        if attributes.get("service.name") == "pse-native-parent-acceptance"
    ]
    roots = [
        span
        for _, span in parents
        if span.name == "pse.native.operation"
        and any(
            item.key == "owner"
            and item.value.string_value == "native.child.actual_acceptance"
            for item in span.attributes
        )
    ]
    require(
        len(roots) == 1
        and roots[0].trace_id.hex() == TRACE_A
        and roots[0].parent_span_id.hex() == PARENT,
        "child_fixture.independent_parent_missing",
    )
    cases = [
        span for _, span in parents if span.name == "pse.native.child.acceptance.case"
    ]
    require(len(cases) == 3, "child_fixture.parent_cases_missing")
    case_pids: dict[str, str] = {}
    targets = []
    for case in cases:
        fields = {item.key: item.value for item in case.attributes}
        require(
            {"native.child.pid", "native.attempt", "acceptance.target"}
            <= fields.keys(),
            "child_fixture.result_references_missing",
        )
        # tracing-opentelemetry 0.34.0 sends unsigned tracing values through
        # its debug visitor as strings. Reading protobuf's unset int_value
        # silently returns zero; validate the actual wire variant instead.
        pid = fields["native.child.pid"]
        require(
            pid.HasField("string_value")
            and pid.string_value.isascii()
            and pid.string_value.isdecimal()
            and 0 < int(pid.string_value) < 2**32,
            "child_fixture.child_pid_wire_value_invalid",
        )
        case_pids[pid.string_value] = fields["native.attempt"].string_value
        targets.append(fields["acceptance.target"].double_value)
    require(
        sorted(targets) == [2.0, 2.0, 4.0] and len(case_pids) == 3,
        f"child_fixture.actual_result_references_changed: targets={targets!r}, child_attempts={case_pids!r}",
    )
    launch = {
        span.span_id.hex(): span
        for _, span in parents
        if span.name == "pse.native.child.launch"
    }
    require(len(launch) == 3, "child_fixture.launch_records_missing")
    parent_instances = {
        attributes.get("service.instance.id") for attributes, _ in parents
    }
    require(
        len(parent_instances) == 1 and None not in parent_instances,
        "child_fixture.parent_incarnation_missing",
    )
    child_resources: dict[str, str] = {}
    linked_pids = set()
    for attributes, span in observed:
        if attributes.get("service.name") != "pse-native-lane":
            continue
        pid = attributes.get("process.pid", "")
        instance = attributes.get("service.instance.id", "")
        require(
            pid in case_pids and instance and instance not in parent_instances,
            "child_fixture.child_process_identity_changed",
        )
        if pid in child_resources:
            require(
                child_resources[pid] == instance,
                "child_fixture.child_incarnation_changed",
            )
        child_resources[pid] = instance
        if span.name == "pse.native.operation" and any(
            item.key == "owner" and item.value.string_value == "native.child.owner"
            for item in span.attributes
        ):
            parent = launch.get(span.parent_span_id.hex())
            require(
                parent is not None and span.trace_id == parent.trace_id,
                "child_fixture.actual_child_parent_changed",
            )
            linked_pids.add(pid)
        if span.name == "pse.native.child.execute":
            attempt = next(
                (
                    item.value.string_value
                    for item in span.attributes
                    if item.key == "native.attempt"
                ),
                None,
            )
            require(attempt == case_pids[pid], "child_fixture.child_attempt_changed")
    require(
        len(set(child_resources.values())) == len(child_resources),
        "child_fixture.child_instances_reused",
    )
    return {
        "parent_cases": len(cases),
        "actual_child_pids": sorted(case_pids),
        "observed_child_resources": child_resources,
        "observed_child_parent_edges": sorted(linked_pids),
        "unobserved_child_parent_edges": sorted(case_pids.keys() - linked_pids),
        "child_parent_continuity": "observed_for_listed_children"
        if linked_pids
        else "not_established",
        "missing_child_records": "short-child process exit does not drain network export; absence is not nonexecution",
        "shared_producers": "independent roots may link the initiating case rather than share its trace",
    }


def run(output: Path) -> int:
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    cases: list[dict[str, object]] = []
    failures = 0
    for name, status, delay in (
        ("disabled", 200, 0.0),
        ("enabled", 200, 0.0),
        ("refused", 503, 0.0),
        ("delayed", 200, 0.6),
    ):
        records: list[tuple[str, bytes]] = []

        server = ThreadingHTTPServer(
            ("127.0.0.1", 0), receiver_handler(records, status, delay)
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
                if not key.startswith(
                    ("OTEL_", "PSE_TELEMETRY_", "PSE_NATIVE_TELEMETRY_")
                )
            }
            environment.update(
                {
                    "PSE_TELEMETRY_STATE": str(output / "unconfigured"),
                    "OTEL_EXPORTER_OTLP_ENDPOINT": f"http://127.0.0.1:{server.server_port}",
                    "OTEL_TRACES_SAMPLER": "always_on",
                    "OTEL_BSP_MAX_EXPORT_BATCH_SIZE": "1",
                    "OTEL_BSP_SCHEDULE_DELAY": "10",
                    "OTEL_BSP_EXPORT_TIMEOUT": "1000",
                    "OTEL_EXPORTER_OTLP_TIMEOUT": "0.15",
                    "OTEL_METRICS_EXPORTER": "none",
                    "OTEL_LOGS_EXPORTER": "none",
                }
            )
            if name == "disabled":
                environment["OTEL_SDK_DISABLED"] = "true"
            else:
                environment.update(
                    {
                        "PSE_NATIVE_TELEMETRY_ACCEPTANCE": "1",
                        "PSE_NATIVE_TELEMETRY_ACCEPTANCE_PARENT": json.dumps(
                            {"traceparent": f"00-{TRACE_A}-{PARENT}-01"}
                        ),
                    }
                )
            command = [
                "just",
                "unit-native-capability-package",
                "pse-runtime",
                "pse-runtime/solver-kinsol",
                "solver,klu,isolation,uno,petsc",
                "actual_portable_native_root_child_reconstructs_and_parent_screens",
            ]
            started = time.monotonic()
            child = subprocess.run(
                command,
                cwd=ROOT,
                env=environment,
                capture_output=True,
                text=True,
                timeout=900,
                check=False,
            )
            (output / (name + "-stdout.txt")).write_text(child.stdout)
            (output / (name + "-stderr.txt")).write_text(child.stderr)
            require(
                child.returncode == 0,
                "child_fixture.native_selector_or_assertion_failed",
            )
            require(
                "1 passed" in child.stdout + child.stderr,
                "child_fixture.selected_test_not_observed",
            )
            item.update(
                elapsed_seconds=time.monotonic() - started,
                native_result="original 2/4/2, process reap and reservation assertions passed",
            )
            if name == "enabled":
                item["telemetry"] = inspect_records(records)
            elif name == "disabled":
                require(not records, "child_fixture.disabled_exported")
            else:
                require(records, "child_fixture.failed_transport_not_exercised")
                item["delivery"] = "not_established; original native assertions passed"
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
            "native child telemetry acceptance: " + name + " " + str(item["status"]),
            flush=True,
        )
    receipt = {
        "baseline": 0,
        "scope": "actual private native executable via native-local selector; not an assembled Python-to-child journey",
        "cases": cases,
        "status": "passed" if failures == 0 else "failed",
        "failure_count": failures,
    }
    (output / "acceptance.json").write_text(
        json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    )
    return 0 if failures == 0 else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    return run(parser.parse_args().output.absolute())


if __name__ == "__main__":
    raise SystemExit(main())
