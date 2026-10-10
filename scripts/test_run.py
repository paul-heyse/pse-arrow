# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Join existing nextest/pytest selection and terminal composition to fixture owners."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import native_operation, validation, validation_receipts
from scripts import test_resources as resources

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

ROOT = Path(__file__).resolve().parents[1]
RUST_OBSERVER = "PSE_RUST_OBSERVER_CHILD"
EXECUTION_EFFECTS = "PSE_TEST_EXECUTION_EFFECTS"


def rust_selected(inventory: Mapping[str, object]) -> list[dict[str, str]]:
    suites = inventory["rust-suites"]
    if not isinstance(suites, dict):
        raise TypeError("Invalid nextest suite inventory")
    selected = []
    for suite in suites.values():
        if not isinstance(suite, dict) or not isinstance(suite.get("testcases"), dict):
            raise TypeError("Invalid nextest test inventory")
        for name, test in suite["testcases"].items():
            if not isinstance(test, dict) or not isinstance(
                test.get("filter-match"), dict
            ):
                raise TypeError("Invalid nextest selected test")
            if test["filter-match"]["status"] == "matches":
                identity, executable = suite["binary-id"], suite["binary-path"]
                if not all(
                    isinstance(value, str) for value in (name, identity, executable)
                ):
                    raise TypeError("Invalid nextest executable or test identity")
                selected.append(
                    {"binary_id": identity, "executable": executable, "name": name}
                )
    return selected


def finish(path: Path, check: dict[str, object], terminal_owner: str) -> bool:
    results = check.get("results", [])
    report_errors = check.get("report_errors", [])
    if (
        not isinstance(results, list)
        or not isinstance(report_errors, list)
        or any(not isinstance(item, dict) for item in results)
    ):
        raise ValueError("Invalid terminal outcome or errors")
    outcomes = {
        str(
            item.get("nodeid") or (str(item["class"]) + "::" + str(item["name"]))
        ): "pass" if item["status"] == "passed" else "failure"
        for item in results
    }
    eligible = resources.finalize(
        path,
        outcomes,
        complete=not check.get("report_errors")
        and check.get("exit_code") in {0, 1, 100},
        terminal_owner=terminal_owner,
    )
    failed = False
    for resource in eligible:
        try:
            reason = resources.reclaim(resource)
            if reason != "removed":
                # A referenced/retained resource is valid; an uncertain drain remains pinned.
                print(f"test-resources: {resource}: retained: {reason}")
        except Exception as error:
            failed = True
            report_errors.append(f"fixture cleanup {resource}: {error}")
    if failed:
        check["status"] = "failed"
    return not failed


def run_python(command: Sequence[str], *, env: Mapping[str, str] | None = None) -> int:
    environment = dict(os.environ if env is None else env)
    if resources.MARKER in environment:
        return subprocess.call(command, cwd=ROOT, env=environment)
    path = resources.invocation("python", [])
    environment[resources.MARKER] = str(path)
    report = path.with_suffix(".xml")
    selection = path.with_suffix(".selected.txt")
    environment["PSE_TEST_ENUMERATION"] = str(selection)
    started = time.time()
    code = subprocess.call(
        [*command, "--junitxml=" + str(report)], cwd=ROOT, env=environment
    )
    check: dict[str, object] = {
        "exit_code": code,
        "status": "passed" if code == 0 else "failed",
        "started": started,
        "results": [],
        "report_errors": [],
        "command": list(command),
        "artifacts": {},
    }
    try:
        check["selected"] = validation_receipts.python_selection(selection)
    except (OSError, ValueError) as error:
        errors = check["report_errors"]
        if not isinstance(errors, list):
            raise TypeError("Invalid report errors") from error
        errors.append(str(error))
    validation.compose_terminal(check, report, report.parent, report.stem)
    finish(path, check, "runner")
    operation_report = path.with_suffix(".terminal.json")
    validation.write_json(operation_report, check)
    return code or int(check["status"] != "passed")


def run_rust(
    command: Sequence[str],
    *,
    env: Mapping[str, str] | None = None,
    inventory: Mapping[str, object] | None = None,
    effects: str = "canonical",
) -> int:
    # Preparation and terminal execution call each other; defer symbol binding.
    from scripts.native_tests import rust_test_environment  # noqa: PLC0415

    environment = rust_test_environment(os.environ if env is None else env)
    if effects not in {"canonical", "native-local"}:
        raise ValueError("Unknown Rust execution effects")
    environment[EXECUTION_EFFECTS] = effects
    if effects == "native-local":
        # Local execution cannot accidentally inherit canonical fixture authority.
        for name in (
            "PSE_SURREAL_STATE",
            "PSE_WORKER_BINARY",
            "PSE_TEST_EXECUTION_PROFILE",
        ):
            environment.pop(name, None)
    if (
        environment.get("PSE_NATIVE_OPERATION")
        and environment.get(RUST_OBSERVER) != "1"
        and "--binaries-metadata" not in command
    ):
        return observer_rust(list(command), environment, effects=effects)
    inherited = environment.get(resources.MARKER)
    run = list(command)
    position = run.index("run")
    from scripts.native_tests import managed_list_arguments  # noqa: PLC0415

    listing = [
        *run[:position],
        "list",
        *managed_list_arguments(run[position + 1 :]),
        "--message-format",
        "json",
    ]
    if inventory is None:
        response = subprocess.run(
            listing,
            cwd=ROOT,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
        )
        if response.returncode:
            print(response.stdout)
            print(response.stderr)
            return response.returncode
        inventory = validation_receipts.native_inventory(response.stdout)
    selected = rust_selected(inventory)
    if inherited:
        path = Path(inherited)
        declaration = resources.read_invocation(path)
        if declaration["kind"] != "rust" or (
            declaration["selected"] and declaration["selected"] != selected
        ):
            raise resources.ResourceError(
                "Runner inventory differs from selected invocation"
            )

        declaration["selected"] = selected
        native_operation.write_json(path, declaration)
        if environment.get("PSE_RUST_TERMINAL_OWNER") != "runner":
            return subprocess.call(command, cwd=ROOT, env=environment)
    else:
        path = resources.invocation("rust", selected)
    environment[resources.MARKER] = str(path)
    profile = "default"
    for index, item in enumerate(run):
        if item == "--profile":
            profile = run[index + 1]
        elif item.startswith("--profile="):
            profile = item.split("=", 1)[1]
    configuration = ROOT / ".config/nextest.toml"
    forwarded = []
    index = 0
    while index < len(run):
        if run[index] == "--config-file":
            configuration = Path(run[index + 1])
            index += 2
        elif run[index].startswith("--config-file="):
            configuration = Path(run[index].split("=", 1)[1])
            index += 1
        else:
            forwarded.append(run[index])
            index += 1
    run = forwarded
    config = configuration.read_text()

    expression = rf"(?m)(^\[profile\.{re.escape(profile)}\.junit\]\n)path\s*=\s*[^\n]+"
    report = path.with_suffix(".xml")
    config, count = re.subn(
        expression, lambda match: match[1] + "path = " + json.dumps(str(report)), config
    )
    if not count:
        config += f"\n[profile.{profile}.junit]\npath = {json.dumps(str(report))}\nstore-failure-output = true\n"
    config_path = path.with_suffix(".toml")
    config_path.write_text(config)
    started = time.time()
    code = subprocess.call(
        [*run, "--config-file", str(config_path)], cwd=ROOT, env=environment
    )
    check: dict[str, object] = {
        "exit_code": code,
        "status": "passed" if code == 0 else "failed",
        "started": started,
        "results": [],
        "report_errors": [],
        "selected": [
            {"class": item["binary_id"], "name": item["name"]} for item in selected
        ],
        "command": run,
        "artifacts": {},
    }
    validation.compose_terminal(check, report, report.parent, report.stem)
    finish(path, check, "runner")
    validation.write_json(path.with_suffix(".terminal.json"), check)
    return code or int(check["status"] != "passed")


def observer_rust(
    command: list[str], environment: dict[str, str], *, effects: str = "canonical"
) -> int:
    """Prepare exact artifacts before the scientific observer's process cap."""
    from scripts import (  # noqa: PLC0415 -- runner cycle
        native_operation,
        native_tests,
        surreal_server,
    )

    path = environment.get("PSE_NATIVE_PROVENANCE")
    if path is None:
        output = validation.fresh_output(
            ROOT, ROOT / "build/native-tests" / str(time.time_ns())
        )
        path = str(output / "native.json")
    provenance = Path(path).resolve()
    selection = Path(
        environment.get(
            "PSE_NATIVE_SELECTION", str(provenance.with_name("native-selected.json"))
        )
    ).resolve()
    execution, worker = native_tests.ordinary_rust_capture(
        command, provenance, selection, environment, effects=effects
    )
    inventory = validation_receipts.native_inventory(selection.read_text())
    if effects == "native-local":
        validation_receipts.verify_native(json.loads(provenance.read_text()))
        return run_rust(
            ["cargo", "nextest", "run", *execution],
            env={
                **environment,
                "PSE_NATIVE_PROVENANCE": str(provenance),
                "PSE_NATIVE_SELECTION": str(selection),
            },
            inventory=inventory,
            effects=effects,
        )
    if worker is None:
        raise resources.ResourceError("Canonical execution requires its worker")
    selected = rust_selected(inventory)
    inherited = environment.get(resources.MARKER)
    if inherited:
        invocation = Path(inherited)
        declaration = resources.read_invocation(invocation)
        if declaration["kind"] != "rust" or (
            declaration["selected"] and declaration["selected"] != selected
        ):
            raise resources.ResourceError(
                "Runner inventory differs from selected invocation"
            )
        declaration["selected"] = selected
        native_operation.write_json(invocation, declaration)
    else:
        invocation = resources.invocation("rust", selected)
    forwarded = {
        resources.MARKER: str(invocation),
        "PSE_NATIVE_PROVENANCE": str(provenance),
        "PSE_NATIVE_SELECTION": str(selection),
        "PSE_WORKER_BINARY": str(worker),
        "PSE_TEST_EXECUTION_PROFILE": "exclusive-observer",
        RUST_OBSERVER: "1",
        "PSE_RUST_TERMINAL_OWNER": "runner" if not inherited else "inherited",
        "RUST_MIN_STACK": environment["RUST_MIN_STACK"],
    }
    # observer uses the existing process environment; restore every carried
    # value after launch rather than leaving invocation authority ambient.
    previous = {key: os.environ.get(key) for key in forwarded}
    os.environ.update(forwarded)
    try:
        return surreal_server.observer(
            Path(environment["PSE_SURREAL_STATE"]),
            [sys.executable, "-m", "scripts.test_run", "--observer-child", *execution],
            profile="exclusive-observer",
        )
    finally:
        for key, value in previous.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def observer_child(extra: list[str]) -> int:
    """Verify retained artifacts, then run; this role never builds."""
    from scripts import native_tests  # noqa: PLC0415 -- owner cycle

    if (
        os.environ.get(RUST_OBSERVER) != "1"
        or os.environ.get("PSE_TEST_EXECUTION_PROFILE") != "exclusive-observer"
    ):
        raise resources.ResourceError(
            "Rust observer child requires its declared profile"
        )
    provenance = Path(os.environ["PSE_NATIVE_PROVENANCE"])
    # Reuse the managed verifier, not its feature or test-selection lowering.
    # run_rust sees retained metadata and the authenticated observer sentinel.
    return native_tests.managed_rust_run(
        [f"--managed-primary-worker={os.environ['PSE_WORKER_BINARY']}", *extra],
        provenance,
    )


if __name__ == "__main__":
    if sys.argv[1:2] != ["--observer-child"]:
        raise SystemExit("test_run expects an observer child invocation")
    raise SystemExit(observer_child(sys.argv[2:]))
