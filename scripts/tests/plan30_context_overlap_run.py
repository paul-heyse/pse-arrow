# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Run prepared C1 language fixtures together inside one admitted observer scope."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

from scripts import (
    host_admission,
    native_operation,
    native_tests,
    producer_deployment,
    test_resources,
    test_run,
)

ROOT = Path(__file__).resolve().parents[2]
FILTER = (
    "test(=testing::canonical_server_unit::plan30_rust_python_fixture_context_overlap)"
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "action", choices=("prepare", "run", "rust-child", "python-child")
    )
    parser.add_argument("directory", type=Path)
    arguments = parser.parse_args()
    directory = arguments.directory.resolve()
    if arguments.action == "prepare":
        directory.mkdir(mode=0o700, parents=True, exist_ok=False)
        python_native = producer_deployment.file_observation(
            native_tests.python_native_binary(os.environ)
        )
        native_operation.write_json(directory / "python-native.json", python_native)
        execution, worker = native_tests.ordinary_rust_capture(
            [
                "cargo",
                "nextest",
                "run",
                "-p",
                "pse-operations",
                "-p",
                "pse-relations",
                "--lib",
                "--features",
                "pse-operations/canonical-tests,pse-relations/force-validate",
                "--run-ignored",
                "only",
                "-E",
                FILTER,
                "--no-fail-fast",
            ],
            directory / "native.json",
            directory / "selected.json",
            os.environ,
        )
        native_operation.write_json(
            directory / "execution.json",
            {"arguments": execution, "worker": str(worker)},
        )
        return 0
    if (
        os.environ.get("PSE_TEST_EXECUTION_PROFILE") != "exclusive-observer"
        or host_admission.inherit(os.environ) is None
    ):
        raise RuntimeError(
            "Actual C1 execution requires its admitted observer owner and explicit profile"
        )
    os.environ["PSE_PLAN30_C1_CONTROL"] = str(directory)
    expected_python_native = json.loads((directory / "python-native.json").read_text())
    if arguments.action in {"run", "python-child"}:
        current_python_native = producer_deployment.file_observation(
            native_tests.python_native_binary(os.environ)
        )
        if current_python_native != expected_python_native:
            raise RuntimeError(
                "Prepared Python native artifact changed; preserve this control and prepare a new directory"
            )
    if arguments.action in {"rust-child", "python-child"}:
        # Each runner supplies its own selected test invocation; the one outer
        # observer owns physical capacity shared by these database-only clients.
        os.environ.pop(test_resources.MARKER, None)
        if arguments.action == "rust-child":
            execution = json.loads((directory / "execution.json").read_text())
            os.environ[test_run.RUST_OBSERVER] = "1"
            return native_tests.managed_rust_run(
                [
                    "--managed-primary-worker=" + execution["worker"],
                    *execution["arguments"],
                ],
                directory / "native.json",
            )
        return test_run.run_python(
            [
                sys.executable,
                "-m",
                "pytest",
                str(ROOT / "scripts/tests/test_plan30_context_overlap.py"),
                "-n",
                "0",
                "-q",
            ]
        )
    if any(directory.glob("*-ready.json")) or any(directory.glob("*-observed.json")):
        raise RuntimeError(
            "Rendezvous directory already contains a prior execution; preserve it and prepare a new directory"
        )
    deadline = time.monotonic() + 150
    processes = []
    streams = []
    codes = {}
    try:
        for language in ("rust", "python"):
            stream = (directory / (language + ".log")).open("x")
            streams.append(stream)
            processes.append(
                (
                    language,
                    subprocess.Popen(
                        [
                            sys.executable,
                            "-m",
                            "scripts.tests.plan30_context_overlap_run",
                            language + "-child",
                            str(directory),
                        ],
                        cwd=ROOT,
                        stdout=stream,
                        stderr=subprocess.STDOUT,
                    ),
                )
            )
        for language, process in processes:
            codes[language] = process.wait(
                timeout=max(0.001, deadline - time.monotonic())
            )
    except subprocess.TimeoutExpired:
        codes["deadline"] = 125
    finally:
        for _language, process in processes:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)
        for stream in streams:
            stream.close()
    records = {}
    for language in ("rust", "python"):
        ready = directory / (language + "-ready.json")
        observed = directory / (language + "-observed.json")
        if ready.exists() and observed.exists():
            declaration = json.loads(ready.read_text())
            records[language] = {
                "ready": declaration,
                "observed": json.loads(observed.read_text()),
                "resource": test_resources.resource_status(declaration["resource"]),
            }
    imported_path = directory / "python-native-imported.json"
    imported_python_native = (
        json.loads(imported_path.read_text()) if imported_path.exists() else None
    )
    passed = (
        codes == {"rust": 0, "python": 0}
        and len(records) == 2
        and imported_python_native == expected_python_native
    )
    if passed:
        passed = records["rust"]["ready"]["database"] != records["python"]["ready"][
            "database"
        ] and all(
            item["resource"]["disposition"] == "pass"
            and item["resource"]["cleanup"] == "removed"
            for item in records.values()
        )
    native_operation.write_json(
        directory / "result.json",
        {
            "status": "passed" if passed else "failed",
            "codes": codes,
            "records": records,
            "python_native": {
                "prepared": expected_python_native,
                "imported": imported_python_native,
            },
            "scope": "registered Rust/Python canonical fixture/schema ownership and same-name probe isolation; no native scientific execution",
        },
    )
    return int(not passed)


if __name__ == "__main__":
    raise SystemExit(main())
