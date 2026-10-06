# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Ordinary native test commands; selection belongs to nextest and pytest."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping

from scripts import validation, validation_receipts

ROOT = Path(__file__).resolve().parents[1]
FEATURES = "pse-runtime/native-solvers,pse-runtime/canonical-tests,pse-tests-conformance/native-acceptance,pse-relations/force-validate"


def native_provenance(
    profile: dict, binaries: list[str], *, environment: Mapping[str, str] | None = None
) -> dict:
    native_environment = os.environ if environment is None else environment

    def digest(path: Path) -> str:
        with Path(path).open("rb") as stream:
            return hashlib.file_digest(stream, "sha256").hexdigest()

    files = {}
    links = {}
    for name in binaries:
        path = Path(name).resolve()
        files[str(path)] = digest(path)
        linked = subprocess.check_output(
            ["ldd", str(path)], text=True, env=native_environment
        )
        if "not found" in linked:
            raise ValueError("native dependency is not linked")
        links[str(path)] = linked
        for line in linked.splitlines():
            parts = line.split()
            targets = [part for part in parts if part.startswith("/")]
            if targets:
                library = Path(targets[0]).resolve()
                if str(library) not in files:
                    files[str(library)] = digest(library)
    if not files:
        raise ValueError("no actual native binaries to bind")
    return {
        "schema": "native-profile-v1",
        "profile": profile,
        "captured": time.time(),
        "files": files,
        "links": links,
        "toolchain": subprocess.check_output(
            ["rustc", "-Vv"], text=True, env=native_environment
        ),
        "environment": validation.relevant_environment(native_environment),
        "threads": {
            k: native_environment.get(k)
            for k in ["OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"]
        },
    }


def rust_command(action: str, extra: list[str]) -> list[str]:
    return [
        "cargo",
        "nextest",
        action,
        "--workspace",
        "--locked",
        "--features",
        FEATURES,
        *extra,
    ]


def python_native_binary(environment: Mapping[str, str]) -> Path:
    """Resolve the imported extension using pytest's interpreter and import environment."""
    program = (
        "import json, sys, pse\n"
        "print(json.dumps({'package': pse.__file__, "
        "'native': sys.modules['pse._native'].__file__}))\n"
    )
    imported = json.loads(
        subprocess.check_output(
            [sys.executable, "-c", program], cwd=ROOT, env=environment, text=True
        )
    )
    package = Path(imported["package"]).resolve()
    binary = Path(imported["native"]).resolve()
    expected = (ROOT / "python/pse").resolve()
    if package != expected / "__init__.py" or binary.parent != expected:
        raise ValueError("native Python import belongs to a different checkout")
    return binary


def worker_binary(extra: list[str], environment: Mapping[str, str]) -> Path:
    """Supply the actual child-process executable in the selected Cargo profile."""
    supplied = environment.get("PSE_WORKER_BINARY")
    if supplied:
        binary = Path(supplied).resolve()
        if not binary.is_file() or not os.access(binary, os.X_OK):
            raise ValueError("PSE_WORKER_BINARY must name an executable file")
        return binary
    profile = (
        "release"
        if "--release" in extra
        else extra[extra.index("--cargo-profile") + 1]
        if "--cargo-profile" in extra
        else next(
            (
                arg.split("=", 1)[1]
                for arg in extra
                if arg.startswith("--cargo-profile=")
            ),
            "dev",
        )
    )
    built = subprocess.run(
        [
            "cargo",
            "build",
            "-p",
            "xtask",
            "--bin",
            "pse-worker",
            "--locked",
            "--features",
            "native-solvers,pse-relations/force-validate",
            "--profile",
            profile,
            "--message-format",
            "json-render-diagnostics",
        ],
        cwd=ROOT,
        env=environment,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
    )
    if built.returncode:
        print(built.stdout, end="")
        raise ValueError(f"native worker build failed with exit {built.returncode}")
    artifacts = set()
    for line in built.stdout.splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        if (
            message.get("reason") == "compiler-artifact"
            and message.get("target", {}).get("name") == "pse-worker"
            and message.get("executable")
        ):
            artifacts.add(Path(message["executable"]).resolve())
    if len(artifacts) != 1:
        raise ValueError("native worker build did not identify one executable")
    binary = artifacts.pop()
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise ValueError("native worker artifact is not executable")
    return binary


def main() -> int:
    kind, *extra = sys.argv[1:]
    path = os.environ.get("PSE_NATIVE_PROVENANCE")
    if kind == "python":
        owners = [
            arg.split("=", 1)[1] for arg in extra if arg.startswith("--terminal-owner=")
        ]
        if len(owners) > 1 or (
            owners and owners[0] not in {"assessment", "standalone"}
        ):
            raise ValueError("invalid terminal owner")
        owner = owners[0] if owners else "standalone"
        extra = [arg for arg in extra if not arg.startswith("--terminal-owner=")]
        report = next(
            (
                Path(arg.split("=", 1)[1])
                for arg in extra
                if arg.startswith("--junitxml=")
            ),
            None,
        )
        if report is None:
            raise ValueError("native Python execution requires a JUnit destination")
        report.parent.mkdir(parents=True, exist_ok=True)
        started = time.time()
        provenance = Path(path) if path else report.with_suffix(".native.json")
        env = dict(os.environ)
        binary = python_native_binary(env)
        native = native_provenance(
            {"features": ["force-validate", "native-solvers"]},
            [str(binary)],
            environment=env,
        )
        validation.write_json(provenance, native)
        # The running pytest session must consume the same imported extension.
        env["PSE_NATIVE_EXPECTED_BINARY"] = str(binary)
        selection = report.with_name(report.stem + "-selected.txt")
        if owner == "standalone":
            env["PSE_TEST_ENUMERATION"] = str(selection)
        command = [
            sys.executable,
            "-m",
            "pytest",
            "python/pse/tests",
            "-m",
            "unit or component or integration",
            "--maxfail=0",
            "--continue-on-collection-errors",
            *extra,
        ]
        code = subprocess.call(command, cwd=ROOT, env=env)
        if owner == "assessment":
            return code
        check = {
            "exit_code": code,
            "status": "passed" if code == 0 else "failed",
            "started": started,
            "results": [],
            "report_errors": [],
            "artifacts": {},
            "command": command,
            "mode": "python-native",
            "native": native,
            "native_provenance": str(provenance),
        }
        try:
            validation_receipts.require_fresh(selection, started, "Python")
            check["selected"] = validation_receipts.python_selection(selection)
        except (OSError, ValueError) as error:
            check["report_errors"].append(f"missing current collection: {error}")
        validation.compose_terminal(check, report, report.parent, report.stem)
        validation.write_json(report.with_suffix(".terminal.json"), check)
        return code or int(check["status"] != "passed")
    if kind != "rust":
        raise ValueError("expected rust or python")
    if not path:
        output = validation.fresh_output(
            ROOT, ROOT / "build/native-tests" / str(time.time_ns())
        )
        path = str(output / "native.json")
    selected_path = Path(
        os.environ.get(
            "PSE_NATIVE_SELECTION", str(Path(path).with_name("native-selected.json"))
        )
    )
    # Run-only output options cannot be sent to nextest list.
    list_extra = [
        arg
        for i, arg in enumerate(extra)
        if arg
        not in {
            "--success-output",
            "--failure-output",
            "--status-level",
            "--final-status-level",
        }
        and (
            i == 0
            or extra[i - 1]
            not in {
                "--success-output",
                "--failure-output",
                "--status-level",
                "--final-status-level",
            }
        )
    ]
    inventory = subprocess.run(
        rust_command("list", ["--message-format", "json", *list_extra]),
        cwd=ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
    )
    selected_path.write_text(inventory.stdout)
    if inventory.returncode:
        print(inventory.stdout, end="")
        return inventory.returncode
    validation_receipts.native_selection(inventory.stdout)
    data = json.loads(inventory.stdout)
    binaries = [
        suite["binary-path"]
        for suite in data["rust-suites"].values()
        if any(
            test["filter-match"]["status"] == "matches"
            for test in suite["testcases"].values()
        )
    ]
    environment = dict(os.environ)
    if any(
        suite.get("binary-name") == "worker"
        or suite.get("binary-id", "").endswith("::worker")
        for suite in data["rust-suites"].values()
        if any(
            test["filter-match"]["status"] == "matches"
            for test in suite["testcases"].values()
        )
    ):
        worker = worker_binary(extra, environment)
        environment["PSE_WORKER_BINARY"] = str(worker)
        binaries.append(str(worker))
    validation.write_json(
        Path(path),
        native_provenance(
            {
                "features": FEATURES.split(","),
                "cargo_profile": "release"
                if "--release" in extra
                else extra[extra.index("--cargo-profile") + 1]
                if "--cargo-profile" in extra
                else "dev",
            },
            binaries,
            environment=environment,
        ),
    )
    return subprocess.call(
        rust_command("run", ["--no-fail-fast", *extra]), cwd=ROOT, env=environment
    )


if __name__ == "__main__":
    raise SystemExit(main())
