# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual explicitly selected diagnostic Python extension and executor smoke."""

from __future__ import annotations

import hashlib
import json
import os
import signal
import sys
import tempfile
import threading
import time
import tomllib
from pathlib import Path

import msgspec

import pse
from pse import codec
from pse._build import compiled_features
from pse.contracts import documents
from pse.contracts.documents import ClosedDuration, TaskDumpRequest
from pse.contracts.enums import NativeSolveIntent
from pse.contracts.values import SemanticId

ROOT = Path(__file__).resolve().parents[2]


def diagnostic_request() -> documents.DiagnosticExecutionRequest:
    """Use existing physical fixtures and one bounded four-equation diagnostic."""
    fixture = ROOT / "tests/fixtures/packages/physical-primitives"
    physical = {
        str(path.relative_to(fixture)): path.read_text()
        for path in fixture.rglob("*")
        if path.is_file()
    }
    primitive = tomllib.loads(physical["package.toml"])["package"]
    major, minor, patch = map(int, primitive["version"].split("."))
    dependency = (
        f'dependencies = [{{ package_id = "{primitive["package_id"]}", '
        f'version_req = {{ operator = "exact", major = {major}, '
        f"minor = {minor}, patch = {patch} }} }}]"
    )
    manifest = (
        (ROOT / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
        .replace("dependencies = []", dependency)
    )
    members = " ".join(
        f"var x{i}:Scalar; eq e{i}:x{i}=={i + 1}; annotation start x{i}(0);"
        for i in range(4)
    )
    case = "ab" * 16
    source = (
        f"package taskdump_control {{ def Root {{ {members} }} "
        f'@id("{case}") case run {{child root:Root=Root();}} }}'
    )
    policy = codec.decode_json(
        (ROOT / "packages/reference/diagnostics/idaes-2.13.json").read_bytes(),
        documents.ModelingDiagnosticPolicy,
    )
    return documents.DiagnosticExecutionRequest(
        physical_documents=physical,
        model_documents=({"package.toml": manifest, "models/control.pse": source},),
        case_id=SemanticId.from_hex(case),
        settings=pse.SolveSettings(intent=NativeSolveIntent.ROOT),
        diagnostics=msgspec.convert(
            policy, type=documents.ModelingDiagnosticPolicyBody, from_attributes=True
        ),
        allowance=ClosedDuration(secs=30, nanos=0),
        maximum_source_bytes=4 << 20,
    )


def interrupted_observer(runtime: pse.Runtime) -> dict:
    """Signal only an observed issued request; distinguish a missed timing window."""

    def interrupt(
        stop: threading.Event,
        baseline: int | None,
        observed: dict,
    ) -> None:
        while not stop.is_set():
            current = runtime.task_dump_status()
            if (
                current is not None
                and current.request != baseline
                and current.target_state == "pending"
            ):
                observed.update(pending_observed=True, request=current.request)
                os.kill(os.getpid(), signal.SIGINT)
                return

    attempts = []
    for _ in range(20):
        before = runtime.task_dump_status()
        baseline = before.request if before is not None else None
        stop = threading.Event()
        observed: dict = {"pending_observed": False, "interrupted_in_call": False}
        began = time.monotonic()

        watcher = threading.Thread(
            target=interrupt, args=(stop, baseline, observed), daemon=True
        )
        watcher.start()
        try:
            try:
                runtime.task_dump(
                    TaskDumpRequest(
                        wait=ClosedDuration(secs=5, nanos=0),
                        maximum_tasks=64,
                        maximum_bytes=65536,
                    )
                )
            except KeyboardInterrupt:
                observed["interrupted_in_call"] = True
            finally:
                stop.set()
                watcher.join(timeout=1)
        except KeyboardInterrupt:
            observed["signal_after_call"] = True
            stop.set()
            watcher.join(timeout=1)
        observed["observer_elapsed_seconds"] = time.monotonic() - began
        retained = runtime.task_dump_status()
        deadline = time.monotonic() + 5
        while (
            retained is not None
            and retained.target_state == "pending"
            and time.monotonic() < deadline
        ):
            time.sleep(0.001)
            retained = runtime.task_dump_status()
        observed["settled_elapsed_seconds"] = time.monotonic() - began
        observed["retained_task_count"] = len(retained.tasks) if retained else None
        observed["retained_request"] = retained.request if retained else None
        observed["settled_state"] = retained.target_state if retained else None
        attempts.append(observed)
        if observed["interrupted_in_call"]:
            if (
                not observed["pending_observed"]
                or retained is None
                or retained.request != observed["request"]
                or retained.target_state != "completed"
            ):
                raise ValueError(
                    "interrupted issued dump was not retained to settlement"
                )
            return {"status": "tested", "attempts": attempts}
        if retained is not None and retained.target_state == "pending":
            break
    return {
        "status": "unavailable",
        "attempts": attempts,
        "reason": "cooperative dump completed before a Python signal window was observed",
    }


def main() -> int:
    features = compiled_features()
    if not {"force-validate", "diagnostic-taskdump"} <= set(features):
        raise ValueError(
            "select the separately built diagnostic-taskdump correctness extension"
        )
    module_path = sys.modules["pse._native"].__file__
    if module_path is None:
        raise ValueError("selected native extension has no file identity")
    binary = Path(module_path)
    if ".venv-diagnostic-taskdump" not in binary.parts:
        raise ValueError(
            "diagnostic extension must be installed separately from the editable checkout"
        )
    with tempfile.TemporaryDirectory(prefix="pse-taskdump-smoke-") as directory:
        runtime = pse.Runtime(
            pse.EngineSettings(
                memory_limit_bytes=64 << 30,
                threads=2,
                target_partitions=1,
                spill_dir=directory,
                max_spill_bytes=1 << 30,
                batch_size=7,
                cache=pse.CacheSettings(
                    working_bytes=128 << 20,
                    concurrent_loads=2,
                    inflight_bytes=32 << 20,
                    inspection_bytes=4 << 20,
                ),
            ),
            ephemeral=True,
        )
        try:
            if runtime.task_dump_status() is not None:
                raise ValueError("new process unexpectedly has an issued task dump")
            result = runtime.task_dump(
                TaskDumpRequest(
                    wait=ClosedDuration(secs=5, nanos=0),
                    maximum_tasks=64,
                    maximum_bytes=65536,
                )
            )
            status = runtime.task_dump_status()
            if (
                status is None
                or status.request != result.request
                or result.target_state != "completed"
            ):
                raise ValueError(
                    "actual executor cooperative request did not complete under independent observation"
                )
            request = diagnostic_request()

            def scientific_work(started: threading.Event, output: dict) -> None:
                began = time.monotonic()
                started.set()
                try:
                    physical = runtime.physical_from_documents(
                        request.physical_documents
                    )
                    package = runtime.modeling_from_documents(
                        request.model_documents, physical
                    )
                    output.update(
                        complete=True,
                        revision=package.revision_id,
                        failure=None,
                        route="public physical/model reconstruction; no mathematical execution",
                    )
                except Exception as error:
                    output["error"] = repr(error)
                finally:
                    output["elapsed_seconds"] = time.monotonic() - began

            baseline: dict = {}
            baseline_started = threading.Event()
            baseline_worker = threading.Thread(
                target=scientific_work,
                args=(baseline_started, baseline),
                daemon=True,
            )
            baseline_worker.start()
            baseline_worker.join(timeout=35)
            baseline["python_call_returned"] = not baseline_worker.is_alive()
            if (
                baseline_worker.is_alive()
                or not baseline.get("complete")
                or baseline.get("failure") is not None
            ):
                raise ValueError(f"no-dump reconstruction baseline failed: {baseline}")
            scientific: dict = {}
            started = threading.Event()
            worker = threading.Thread(
                target=scientific_work,
                args=(started, scientific),
                daemon=True,
            )
            worker.start()
            started.wait(timeout=1)
            interruption = interrupted_observer(runtime)
            worker.join(timeout=35)
            scientific["python_call_returned"] = not worker.is_alive()
            if (
                worker.is_alive()
                or "error" in scientific
                or "complete" not in scientific
            ):
                raise ValueError(
                    f"independent reconstruction has not returned: {scientific}"
                )
            interruption["independent_workflow"] = scientific
            interruption["no_dump_baseline"] = baseline
            interruption["thread_join_limit_seconds"] = 35
            interruption["meaning"] = (
                "observer interruption does not cancel its issued target dump; "
                "scientific execution is not qualified by the model reconstruction control"
            )
            if interruption["status"] != "tested":
                raise ValueError(
                    f"pending-workflow signal control unavailable: {interruption}"
                )
            with binary.open("rb") as source:
                binary_digest = hashlib.file_digest(source, "sha256").hexdigest()
            report: dict = {
                "passed": 1,
                "failed": 0,
                "binary": str(binary),
                "sha256": binary_digest,
                "features": features,
                "executor": result.executor,
                "request": result.request,
                "task_count": len(result.tasks),
                "python_interruption": interruption,
                "scope": "diagnostic action contracts only; scientific outcome is independent and no scientific qualification is claimed",
            }
        finally:
            close_started = time.monotonic()
            runtime.close()
        report["runtime_close_seconds"] = time.monotonic() - close_started
        report["runtime_close_returned"] = True
        print(json.dumps(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
