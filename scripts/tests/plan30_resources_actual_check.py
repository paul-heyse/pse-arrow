# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Opt-in real pytest/xdist and filesystem retention controls; no database science."""

from __future__ import annotations

import ast
import json
import os
import subprocess
import sys
import time
import unittest
from pathlib import Path

from scripts import native_operation, surreal_server, test_resources, test_run

ROOT = Path(__file__).resolve().parents[2]


def bindings(directory: Path) -> None:
    """Only service configuration is a stand-in: these controls own no database."""
    test_resources.registry = lambda: directory / "registry"

    def configured(state: Path, *, deadline: float | None = None) -> dict[str, object]:
        _ = state, deadline
        return {
            "instance_id": "filesystem-control",
            "namespace": "pse",
            "admission": "open",
            "accepting_writes": True,
        }

    surreal_server.config_for = configured


def wait_file(path: Path, deadline: float) -> None:
    while not path.exists():
        if time.monotonic() >= deadline:
            raise TimeoutError(f"control file did not arrive: {path}")
        time.sleep(0.02)


PLUGIN = """
import contextlib, json, os, time, uuid
from pathlib import Path
import pytest, xdist
from scripts import native_operation, test_resources
from scripts.tests.plan30_resources_actual_check import bindings
directory = Path(os.environ["PSE_C_CONTROL"])
bindings(directory)

@contextlib.contextmanager
def owned(request):
    database = "canonical_test_" + uuid.uuid4().hex
    controls = directory / "state" / database
    resource = test_resources.register({"state": str(directory / "state"), "database": database,
        "kind": "controls", "directory": str(controls), "test": request.node.nodeid})
    controls.mkdir()
    (controls / "same-problem-head").write_text(database)
    native_operation.write_json(directory / "observed" / (resource + ".json"),
        {"resource": resource, "database": database, "controls": str(controls), "nodeid": request.node.nodeid,
         "invocation": test_resources.read_invocation(Path(os.environ[test_resources.MARKER]))["nonce"],
         "worker": os.environ.get("PYTEST_XDIST_WORKER"), "started": time.monotonic()})
    try:
        yield controls, resource
    finally:
        test_resources.record_drain(resource)
        (directory / "observed" / (resource + ".dropped")).write_text("drained before assertion")
"""

TESTS = """
import os, time
from pathlib import Path
import pytest
from conftest import owned

@pytest.mark.unit
@pytest.mark.parametrize("index", [0, 1])
def test_same_problem(request, index):
    directory = Path(os.environ["PSE_C_CONTROL"])
    with owned(request) as (controls, resource):
        (directory / "barrier" / resource).write_text("ready")
        deadline = time.monotonic() + 15
        while len(list((directory / "barrier").iterdir())) < 4:
            assert time.monotonic() < deadline, "overlap barrier expired"
            time.sleep(.02)
        assert (controls / "same-problem-head").read_text() == controls.name

@pytest.mark.unit
def test_failure_after_drop(request):
    with owned(request):
        pass
    assert False, "intentional assertion after context drain"

@pytest.mark.unit
def test_interrupted(request):
    with owned(request):
        os._exit(23)
"""


def child(directory: Path, selection: str) -> int:
    bindings(directory)
    os.environ.pop(test_resources.MARKER, None)
    return test_run.run_python(
        [
            sys.executable,
            "-m",
            "pytest",
            "-c",
            str(directory / "pytest.ini"),
            "--confcutdir",
            str(directory),
            str(directory / "test_controls.py"),
            "-k",
            selection,
            "-n",
            "2" if selection == "same_problem" else "0",
            "-q",
        ]
    )


class ActualResourceControls(unittest.TestCase):
    @unittest.skipUnless(
        os.environ.get("PSE_PLAN30_C_COLLECTION_ACTUAL"),
        "explicit nested collector subprocess control",
    )
    def test_nested_collection_preserves_parent_catalog_and_later_fixture_association(
        self,
    ) -> None:
        directory = Path(os.environ["PSE_PLAN30_C_COLLECTION_ACTUAL"]).resolve()
        directory.mkdir(mode=0o700, parents=True, exist_ok=False)
        for name in ("state", "observed"):
            (directory / name).mkdir(mode=0o700)
        (directory / "pytest.ini").write_text(
            "[pytest]\nmarkers = unit: bounded control\njunit_family = xunit1\n"
        )
        source = ast.parse((ROOT / "conftest.py").read_text())
        names = {
            "pytest_collection_finish",
            "_write_selected_inventory",
            "pytest_xdist_node_collection_finished",
        }
        hooks = ast.unparse(
            ast.Module(
                body=[
                    node
                    for node in source.body
                    if isinstance(node, ast.FunctionDef) and node.name in names
                ],
                type_ignores=[],
            )
        )
        (directory / "conftest.py").write_text(
            "from __future__ import annotations\n"
            + PLUGIN
            + "\n_parallel_inventory_written_key = pytest.StashKey[bool]()\n"
            + hooks
        )
        (directory / "test_controls.py").write_text("""
import json, os, subprocess, sys
from pathlib import Path
import pytest
from conftest import owned
from scripts import test_resources

@pytest.mark.unit
def test_a_nested(request):
    directory = Path(os.environ["PSE_C_CONTROL"])
    invocation = Path(os.environ["PSE_TEST_INVOCATION"])
    if os.environ.get("PSE_C_NESTED") != "1":
        catalog = invocation.read_bytes()
        enumeration = Path(os.environ["PSE_TEST_ENUMERATION"])
        selected = enumeration.read_bytes()
        result = subprocess.run([sys.executable, "-m", "pytest", "-c", str(directory / "pytest.ini"),
            "--confcutdir", str(directory), str(directory / "test_controls.py") + "::test_a_nested", "-n", "0", "-q"],
            env=dict(os.environ, PSE_C_NESTED="1"), capture_output=True, text=True, timeout=15)
        (directory / "nested.stdout").write_text(result.stdout)
        (directory / "nested.stderr").write_text(result.stderr)
        assert result.returncode == 0, result.stdout + result.stderr
        assert invocation.read_bytes() == catalog
        assert enumeration.read_bytes() == selected
    else:
        (directory / "nested-pid").write_text(str(os.getpid()))
    with owned(request):
        pass

@pytest.mark.unit
@pytest.mark.parametrize("index", [0, 1])
def test_b_later(request, index):
    with owned(request):
        pass
""")
        with (directory / "parent.log").open("x") as log:
            result = subprocess.run(
                [
                    sys.executable,
                    "-m",
                    "scripts.tests.plan30_resources_actual_check",
                    "--runner",
                    str(directory),
                    "nested or later",
                ],
                cwd=ROOT,
                env=dict(os.environ, PSE_C_CONTROL=str(directory)),
                stdout=log,
                stderr=subprocess.STDOUT,
                timeout=25,
                check=False,
            )
        assert result.returncode == 0, (directory / "parent.log").read_text()
        bindings(directory)
        observations = [
            json.loads(path.read_text())
            for path in (directory / "observed").glob("*.json")
        ]
        assert len(observations) == 4
        assert len({row["invocation"] for row in observations}) == 1
        records = [
            test_resources.resource_status(row["resource"]) for row in observations
        ]
        assert all(
            row["disposition"] == "pass" and row["cleanup"] == "removed"
            for row in records
        )
        invocation = next(
            path
            for path in (directory / "registry" / "invocations").glob("*.json")
            if test_resources.IDENTIFIER.fullmatch(path.stem)
        )
        catalog = test_resources.read_invocation(invocation)
        assert len(catalog["selected"]) == 3
        assert catalog["collection_owner"]["pid"] != int(
            (directory / "nested-pid").read_text()
        )
        native_operation.write_json(
            directory / "result.json",
            {
                "status": "passed",
                "scope": "actual parent/nested pytest collection authority and later filesystem fixture association; no native science",
                "catalog": catalog,
                "resources": records,
            },
        )

    @unittest.skipUnless(
        os.environ.get("PSE_PLAN30_C_ACTUAL"), "explicit bounded subprocess control"
    )
    def test_actual_runner_ownership_and_retention(self) -> None:
        directory = Path(os.environ["PSE_PLAN30_C_ACTUAL"]).resolve()
        directory.mkdir(mode=0o700, parents=True, exist_ok=False)
        for name in ("state", "observed", "barrier"):
            (directory / name).mkdir(mode=0o700)
        (directory / "pytest.ini").write_text(
            "[pytest]\nmarkers = unit: bounded control\njunit_family = xunit1\n"
        )
        # Execute the production collection/xdist hooks verbatim, without loading
        # the unrelated linked-extension probe or the project's scientific fixtures.
        source = ast.parse((ROOT / "conftest.py").read_text())
        names = {
            "pytest_collection_finish",
            "_write_selected_inventory",
            "pytest_xdist_node_collection_finished",
        }
        selected: list[ast.stmt] = [
            node
            for node in source.body
            if isinstance(node, ast.FunctionDef) and node.name in names
        ]
        hooks = ast.unparse(ast.Module(body=selected, type_ignores=[]))
        (directory / "conftest.py").write_text(
            "from __future__ import annotations\n"
            + PLUGIN
            + "\n_parallel_inventory_written_key = pytest.StashKey[bool]()\n"
            + hooks
        )
        (directory / "test_controls.py").write_text(TESTS)
        environment = dict(os.environ, PSE_C_CONTROL=str(directory))
        processes = []
        streams = []
        try:
            for name in ("first", "second"):
                stream = (directory / (name + ".log")).open("w")
                streams.append(stream)
                processes.append(
                    subprocess.Popen(
                        [
                            sys.executable,
                            "-m",
                            "scripts.tests.plan30_resources_actual_check",
                            "--runner",
                            str(directory),
                            "same_problem",
                        ],
                        cwd=ROOT,
                        env=environment,
                        stdout=stream,
                        stderr=subprocess.STDOUT,
                    )
                )
            for process in processes:
                assert process.wait(timeout=30) == 0
        finally:
            for process in processes:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=5)
            for stream in streams:
                stream.close()
        bindings(directory)
        observations = [
            json.loads(path.read_text())
            for path in (directory / "observed").glob("*.json")
        ]
        assert len(observations) == 4
        assert len({record["database"] for record in observations}) == 4
        assert len({record["invocation"] for record in observations}) == 2
        assert {record["worker"] for record in observations} == {"gw0", "gw1"}
        assert len({record["nodeid"] for record in observations}) == 2
        for record in observations:
            assert not Path(record["controls"]).exists()
            assert (
                test_resources.resource_status(record["resource"])["disposition"]
                == "pass"
            )
        for selection, expected in (("failure_after_drop", 1), ("interrupted", 23)):
            with (directory / (selection + ".log")).open("w") as stream:
                completed = subprocess.run(
                    [
                        sys.executable,
                        "-m",
                        "scripts.tests.plan30_resources_actual_check",
                        "--runner",
                        str(directory),
                        selection,
                    ],
                    cwd=ROOT,
                    env=environment,
                    stdout=stream,
                    stderr=subprocess.STDOUT,
                    timeout=20,
                    check=False,
                )
            assert completed.returncode == expected
        retained = [
            record
            for record in test_resources.resource_status().values()
            if record["cleanup"] == "retained"
        ]
        assert {record["disposition"] for record in retained} == {
            "failure",
            "incomplete",
        }
        assert all(
            record["pin"]
            and record["directory"] is not None
            and Path(record["directory"]).exists()
            for record in retained
        )
        failed = next(
            record for record in retained if record["disposition"] == "failure"
        )
        assert failed["drained"]
        interrupted = next(
            record for record in retained if record["disposition"] == "incomplete"
        )
        assert not interrupted["drained"]
        # A real reference borrower protects an original report through publication.
        report = directory / "build" / "origin"
        report.mkdir(parents=True)
        (report / "checks.json").write_text("{}")
        (report / "first.log").write_text("first payload")
        (report / "second.log").write_text("second payload")
        resource = test_resources.register_report(report, root=directory)
        # This report has no execution descendants. Its finite publisher settles
        # before cleanup; do not associate the outer control's still-live scope.
        publisher = dict(environment)
        publisher.pop(test_resources.host.MARKER, None)
        subprocess.run(
            [
                sys.executable,
                "-m",
                "scripts.tests.plan30_resources_actual_check",
                "--finish-report",
                str(directory),
                resource,
            ],
            cwd=ROOT,
            env=publisher,
            timeout=10,
            check=True,
        )
        with (directory / "reference.log").open("w") as stream:
            process = subprocess.Popen(
                [
                    sys.executable,
                    "-m",
                    "scripts.tests.plan30_resources_actual_check",
                    "--borrow",
                    str(directory),
                    resource,
                ],
                cwd=ROOT,
                env=environment,
                stdout=stream,
                stderr=subprocess.STDOUT,
            )
            try:
                wait_file(directory / "borrow-ready", time.monotonic() + 10)
                assert (
                    test_resources.reclaim(resource)
                    == "protected reference or active borrow"
                )
                (directory / "borrow-release").write_text("publish reference")
                assert process.wait(timeout=10) == 0
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=5)
        assert (
            test_resources.reclaim(resource) == "protected reference or active borrow"
        )
        test_resources.release_reference(resource, "actual-consumer", "c" * 64)
        completed = subprocess.run(
            [
                sys.executable,
                "-m",
                "scripts.tests.plan30_resources_actual_check",
                "--cleanup-crash",
                str(directory),
                resource,
            ],
            cwd=ROOT,
            env=environment,
            timeout=10,
            check=False,
        )
        assert completed.returncode == 27
        assert test_resources.resource_status(resource)["cleanup"] == "removing"
        assert len(list(report.glob("*.log"))) == 1
        assert test_resources.reclaim(resource) == "removed"
        assert (report / "checks.json").exists()
        assert list(report.glob("*.log")) == []
        native_operation.write_json(
            directory / "result.json",
            {
                "scope": "actual pytest/xdist + filesystem controls; service configuration stand-in; no canonical database/native science",
                "overlap_resources": observations,
                "expected_failure_and_interruption": retained,
                "reference_and_interrupted_cleanup": test_resources.resource_status(
                    resource
                ),
                "status": "passed",
            },
        )


if __name__ == "__main__":
    mode, *arguments = sys.argv[1:]
    if mode == "--runner":
        raise SystemExit(child(Path(arguments[0]), arguments[1]))
    directory = Path(arguments[0])
    resource = arguments[1]
    bindings(directory)
    if mode == "--borrow":
        with test_resources.borrow(resource):
            (directory / "borrow-ready").write_text("borrow held")
            wait_file(directory / "borrow-release", time.monotonic() + 15)
            test_resources.retain_reference(resource, "actual-consumer", "c" * 64)
    elif mode == "--finish-report":
        report = Path(test_resources.resource_status(resource)["state"])
        receipt = {
            "version": 5,
            "complete": True,
            "required_checks_covered": True,
            "input_coverage": True,
            "source_unchanged": True,
            "provenance_errors": [],
            "scope": [{"name": "filesystem-control"}],
            "checks": [{"gate": "filesystem-control", "status": "passed"}],
        }
        native_operation.write_json(report / "checks.json", receipt)
        native_operation.write_json(
            report / "fixture-source.json", {"scope": "filesystem-control"}
        )
        test_resources.finish_report(
            resource,
            receipt,
            roles={
                "checks.json": "receipt",
                "first.log": "scratch",
                "second.log": "scratch",
                "fixture-source.json": "provenance",
            },
            required_provenance=["fixture-source.json"],
        )
    elif mode == "--cleanup-crash":
        unlink = os.unlink

        def crash_after_unlink(
            path: str | bytes | os.PathLike[str] | os.PathLike[bytes],
            *,
            dir_fd: int | None = None,
        ) -> None:
            unlink(path, dir_fd=dir_fd)
            os._exit(27)

        os.unlink = crash_after_unlink
        test_resources.reclaim(resource)
    else:
        raise SystemExit("unknown bounded control mode")
