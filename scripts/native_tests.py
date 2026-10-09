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

from scripts import producer_deployment, validation, validation_receipts

ROOT = Path(__file__).resolve().parents[1]
FEATURES = "pse-runtime/native-solvers,pse-runtime/canonical-tests,pse-tests-conformance/native-acceptance,xtask/canonical-tests,pse-relations/force-validate"
MANAGED_FEATURES = ",".join(
    feature
    for feature in FEATURES.split(",")
    if feature.startswith(("pse-runtime/", "pse-relations/"))
)


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
    # These receipts grant deployment reuse. Bind their actual bytes as well as
    # executable bytes; replacing a receipt at the same path changes the premise.
    for name in (
        "PSE_PRODUCER_RECEIPT",
        "PSE_PRODUCER_FIXTURE_RECEIPT",
        "PSE_WORKER_PRODUCER_RECEIPT",
        "PSE_PYTHON_PRODUCER_RECEIPT",
        "PSE_PYTHON_DEPLOYMENT_ATTESTATION",
        "PSE_DEPLOYMENT_ARTIFACT_OBSERVATIONS",
        "PSE_NATIVE_PROVIDER_RECEIPT",
    ):
        selected = native_environment.get(name)
        if selected:
            path = Path(selected).resolve()
            files[str(path)] = digest(path)
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


def rust_command(action: str, extra: list[str], *, managed: bool = False) -> list[str]:
    # An explicit package is a build selection, not merely a nextest filter.
    # Cargo's --workspace would widen it back to every workspace test binary.
    selection = extra[: extra.index("--")] if "--" in extra else extra
    scoped = any(
        argument in {"-p", "--package", "--workspace"}
        or argument.startswith(("--package=", "-p"))
        for argument in selection
    )
    return [
        "cargo",
        "nextest",
        action,
        *(
            []
            if managed and scoped
            else ["-p", "pse-runtime"]
            if managed
            else ["--workspace"]
        ),
        "--locked",
        "--features",
        MANAGED_FEATURES if managed else FEATURES,
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


def observe_deployed_artifacts(
    environment: Mapping[str, str], python_binary: Path | None = None
) -> None:
    """Refresh control observations from actual role paths, never receipt claims."""
    destination = environment.get("PSE_DEPLOYMENT_ARTIFACT_OBSERVATIONS")
    if not destination:
        return
    worker = environment.get("PSE_WORKER_BINARY")
    if not worker:
        raise ValueError("deployment controls require the actual worker executable")
    python = (
        python_binary
        if python_binary is not None
        else python_native_binary(environment)
    )
    observations = {
        "worker": producer_deployment.file_observation(Path(worker)),
        "python": producer_deployment.file_observation(python),
    }
    validation.write_json(Path(destination), observations)


PYTHON_DEFAULT_SELECTION = ("-o", "testpaths=python/pse/tests")


def python_command(extra: list[str], *, managed: bool = False) -> list[str]:
    """Keep user selection within one process's declared runtime budget."""
    selected = "unit or component or integration"
    forwarded: list[str] = []
    index = 0
    while index < len(extra):
        argument = extra[index]
        if argument in {"-m", "--markexpr"}:
            index += 1
            if index == len(extra):
                raise ValueError("pytest marker selection requires an expression")
            selected = extra[index]
        elif argument.startswith("--markexpr="):
            selected = argument.split("=", 1)[1]
        elif argument.startswith("-m") and len(argument) > 2:
            selected = argument[2:]
        else:
            forwarded.append(argument)
        index += 1
    partition = "managed_primary" if managed else "not managed_primary"
    return [
        sys.executable,
        "-m",
        "pytest",
        *PYTHON_DEFAULT_SELECTION,
        "--maxfail=0",
        "--continue-on-collection-errors",
        *([] if managed else ["-n", "16"]),
        *forwarded,
        "-m",
        f"({selected}) and {partition}",
        *(["-n", "0"] if managed else []),
    ]


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
            "native-solvers,canonical-tests,pse-relations/force-validate",
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


def managed_rust_arguments(extra: list[str]) -> tuple[list[str], list[str]]:
    """Consume Cargo choices in metadata; keep nextest execution choices intact."""
    values = {
        "-p",
        "--package",
        "--exclude",
        "--bin",
        "--example",
        "--test",
        "--bench",
        "-F",
        "--features",
        "--build-jobs",
        "--cargo-profile",
        "--target",
        "--target-dir",
        "--manifest-path",
        "--cargo-message-format",
        "--config",
        "-Z",
    }
    flags = {
        "--workspace",
        "--all",
        "--lib",
        "--bins",
        "--examples",
        "--tests",
        "--benches",
        "--all-targets",
        "--all-features",
        "--no-default-features",
        "-r",
        "--release",
        "--unit-graph",
        "--timings",
        "--frozen",
        "--locked",
        "--offline",
        "--cargo-quiet",
        "--cargo-verbose",
        "--ignore-rust-version",
        "--future-incompat-report",
    }
    metadata_values = {"--manifest-path", "--config", "-Z"}
    metadata_flags = {
        "--all-features",
        "--no-default-features",
        "--frozen",
        "--offline",
    }
    execution: list[str] = []
    metadata: list[str] = []
    features: list[str] = [MANAGED_FEATURES]
    index = 0
    while index < len(extra):
        argument = extra[index]
        if argument == "--":
            execution.extend(extra[index:])
            break
        name, separator, value = argument.partition("=")
        if name in {"--binaries-metadata", "--cargo-metadata", "--archive-file"}:
            raise ValueError("managed Rust execution owns its retained build metadata")
        if name in {"--rerun", "-R"}:
            raise ValueError(
                "managed Rust inventory requires an explicit nextest selection, not rerun history"
            )
        if not separator and any(
            argument.startswith(short) and len(argument) > len(short)
            for short in ("-p", "-F", "-Z")
        ):
            name, value, separator = argument[:2], argument[2:], "="
        if name in values:
            if not separator:
                index += 1
                if index == len(extra):
                    raise ValueError(f"{name} requires a value")
                value = extra[index]
            if name in {"--features", "-F"}:
                features.append(value)
            elif name in metadata_values:
                metadata.extend([name, value])
            elif name == "--target":
                metadata.extend(["--filter-platform", value])
        elif name in flags:
            if name in metadata_flags:
                metadata.append(name)
        else:
            execution.append(argument)
        index += 1
    return execution, ["--features", ",".join(features), *metadata]


def managed_list_arguments(extra: list[str]) -> list[str]:
    """Inventory the same filters without run-only output/scheduling controls."""
    values = {
        "--success-output",
        "--failure-output",
        "--status-level",
        "--final-status-level",
        "--test-threads",
        "-j",
        "--jobs",
        "--retries",
        "--flaky-result",
        "--max-fail",
        "--stress-count",
        "--stress-duration",
        "--debugger",
        "--tracer",
        "--no-tests",
        "--message-format",
        "--message-format-version",
        "--max-progress-running",
    }
    flags = {
        "--no-run",
        "--fail-fast",
        "--ff",
        "--no-fail-fast",
        "--nff",
        "--no-capture",
        "--no-output-indent",
        "--hide-progress-bar",
        "--no-input-handler",
    }
    result: list[str] = []
    index = 0
    while index < len(extra):
        argument = extra[index]
        if argument == "--":
            result.extend(extra[index:])
            break
        name, separator, _ = argument.partition("=")
        if name in values:
            if not separator:
                index += 1
                if index == len(extra):
                    raise ValueError(f"{name} requires a value")
        elif name not in flags:
            result.append(argument)
        index += 1
    return result


def managed_cargo_profile(extra: list[str]) -> str:
    if "--release" in extra or "-r" in extra:
        return "release"
    for index, argument in enumerate(extra):
        if argument.startswith("--cargo-profile="):
            return argument.split("=", 1)[1]
        if argument == "--cargo-profile":
            if index + 1 == len(extra):
                raise ValueError("--cargo-profile requires a value")
            return extra[index + 1]
    return "dev"


def managed_rust_selection(extra: list[str]) -> list[str]:
    """Intersect the caller's filterset union with the observer partition.

    Nextest combines repeated -E expressions with OR, so appending another
    expression would admit ordinary tests into the observer allocation.
    """
    expressions: list[str] = []
    forwarded: list[str] = []
    tail: list[str] = []
    index = 0
    while index < len(extra):
        argument = extra[index]
        if argument == "--":
            tail = extra[index:]
            break
        name, separator, value = argument.partition("=")
        if name in {"-E", "--filterset", "--filter-expr"}:
            if not separator:
                index += 1
                if index == len(extra):
                    raise ValueError("nextest filterset requires an expression")
                value = extra[index]
            expressions.append(value)
        elif argument.startswith("-E"):
            expressions.append(argument[2:])
        elif argument != "--ignore-default-filter":
            forwarded.append(argument)
        index += 1
    expression = "test(managed_primary_)"
    if expressions:
        expression += (
            " and (" + " or ".join(f"({value})" for value in expressions) + ")"
        )
    return [*forwarded, "--ignore-default-filter", "-E", expression, *tail]


def managed_rust_capture(
    extra: list[str], provenance: Path, selected: Path
) -> tuple[list[str], Path]:
    """Build and enumerate before entering the observer's finite process cap."""
    scoped = managed_rust_selection(extra)
    execution, metadata_options = managed_rust_arguments(scoped)
    binary_metadata = selected.with_name(
        selected.stem + "-binaries-metadata.json"
    ).resolve()
    cargo_metadata = selected.with_name(
        selected.stem + "-cargo-metadata.json"
    ).resolve()
    for destination in (provenance, selected, binary_metadata, cargo_metadata):
        destination.parent.mkdir(parents=True, exist_ok=True)
    environment = dict(os.environ)
    # Managed controls live in the runtime unit binary, so their binary-name is
    # not "worker". They still require the actual qualified receiver artifact.
    profile = managed_cargo_profile(extra)
    worker = worker_binary(["--cargo-profile", profile], environment)
    environment["PSE_WORKER_BINARY"] = str(worker)
    build = subprocess.run(
        rust_command(
            "list",
            [
                "--list-type",
                "binaries-only",
                "--message-format",
                "json",
                *managed_list_arguments(scoped),
            ],
            managed=True,
        ),
        cwd=ROOT,
        env=environment,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
    )
    if build.returncode:
        print(build.stdout, end="")
        raise ValueError(f"managed Rust build failed with exit {build.returncode}")
    json.loads(build.stdout)
    binary_metadata.write_text(build.stdout)
    graph = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1", *metadata_options],
        cwd=ROOT,
        env=environment,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
    )
    if graph.returncode:
        print(graph.stdout, end="")
        raise ValueError(f"managed Rust metadata failed with exit {graph.returncode}")
    json.loads(graph.stdout)
    cargo_metadata.write_text(graph.stdout)
    reuse = [
        "--binaries-metadata",
        str(binary_metadata),
        "--cargo-metadata",
        str(cargo_metadata),
    ]
    inventory = subprocess.run(
        [
            "cargo",
            "nextest",
            "list",
            *reuse,
            "--message-format",
            "json",
            *managed_list_arguments(execution),
        ],
        cwd=ROOT,
        env=environment,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
    )
    selected.write_text(inventory.stdout)
    if inventory.returncode:
        print(inventory.stdout, end="")
        raise ValueError(
            f"managed Rust inventory failed with exit {inventory.returncode}"
        )
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
    observe_deployed_artifacts(environment)
    native = native_provenance(
        {
            "features": MANAGED_FEATURES.split(","),
            "cargo_profile": profile,
            "managed_build_arguments": extra,
        },
        [*binaries, str(worker)],
        environment=environment,
    )
    for destination in (selected, binary_metadata, cargo_metadata):
        with destination.open("rb") as stream:
            native["files"][str(destination)] = hashlib.file_digest(
                stream, "sha256"
            ).hexdigest()
    validation.write_json(provenance, native)
    return [*reuse, *execution], worker


def managed_rust_run(extra: list[str], provenance: Path) -> int:
    """Reuse verified parent artifacts; no Cargo build or metadata query occurs here."""
    worker_arguments = [
        argument.split("=", 1)[1]
        for argument in extra
        if argument.startswith("--managed-primary-worker=")
    ]
    if len(worker_arguments) != 1:
        raise ValueError("managed Rust child requires one prebuilt worker")
    extra = [
        argument
        for argument in extra
        if not argument.startswith("--managed-primary-worker=")
    ]
    for option in ("--binaries-metadata", "--cargo-metadata"):
        if extra.count(option) != 1:
            raise ValueError("managed Rust child requires retained nextest metadata")
    native = json.loads(provenance.read_text())
    files = native.get("files")
    if not isinstance(files, dict) or not files:
        raise ValueError("managed Rust child requires parent artifact provenance")
    for name, expected in files.items():
        with Path(name).open("rb") as stream:
            if hashlib.file_digest(stream, "sha256").hexdigest() != expected:
                raise ValueError(
                    "managed Rust parent artifacts changed before execution"
                )
    for option in ("--binaries-metadata", "--cargo-metadata"):
        position = extra.index(option) + 1
        if position >= len(extra) or extra[position] not in files:
            raise ValueError("managed Rust metadata is outside parent provenance")
    environment = dict(os.environ)
    worker = Path(worker_arguments[0]).resolve()
    if (
        str(worker) not in files
        or not worker.is_file()
        or not os.access(worker, os.X_OK)
    ):
        raise ValueError("managed Rust worker is outside parent provenance")
    environment["PSE_WORKER_BINARY"] = str(worker)
    return subprocess.call(
        ["cargo", "nextest", "run", "--no-fail-fast", *extra], cwd=ROOT, env=environment
    )


def main() -> int:
    kind, *extra = sys.argv[1:]
    path = os.environ.get("PSE_NATIVE_PROVENANCE")
    if kind == "python":
        managed = "--managed-primary-route" in extra
        child = "--managed-primary-child" in extra
        if child and not managed:
            raise ValueError("managed primary child requires its declared route")
        if managed and not child:
            state = os.environ.get("PSE_SURREAL_STATE")
            if not state:
                raise ValueError("managed primary route requires PSE_SURREAL_STATE")
            from scripts import (  # noqa: PLC0415 -- selected managed placement owner
                surreal_server,
            )

            return surreal_server.observer(
                Path(state),
                [
                    sys.executable,
                    "-m",
                    "scripts.native_tests",
                    "python",
                    *extra,
                    "--managed-primary-child",
                ],
            )
        extra = [
            arg
            for arg in extra
            if arg not in {"--managed-primary-route", "--managed-primary-child"}
        ]
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
        observe_deployed_artifacts(env, binary)
        native = native_provenance(
            {"features": ["force-validate", "native-solvers"]},
            [str(binary)],
            environment=env,
        )
        validation.write_json(provenance, native)
        if observed := env.pop("PSE_PYTHON_DEPLOYMENT_OBSERVATION_OUTPUT", None):
            if Path(observed).exists():
                raise ValueError("deployment observation output already exists")
            env["PSE_PYTHON_DEPLOYMENT_ATTESTATION"] = observed
        # The running pytest session must consume the same imported extension.
        env["PSE_NATIVE_EXPECTED_BINARY"] = str(binary)
        env["PSE_NATIVE_EXPECTED_SHA256"] = native["files"][str(binary)]
        selection = report.with_name(report.stem + "-selected.txt")
        if owner == "standalone":
            env["PSE_TEST_ENUMERATION"] = str(selection)
        command = python_command(extra, managed=managed)
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
    managed = "--managed-primary-route" in extra
    child = "--managed-primary-child" in extra
    extra = [
        arg
        for arg in extra
        if arg not in {"--managed-primary-route", "--managed-primary-child"}
    ]
    if managed and child:
        raise ValueError("managed primary route cannot recurse")
    if managed or child:
        if managed and not os.environ.get("PSE_SURREAL_STATE"):
            raise ValueError("managed native execution requires PSE_SURREAL_STATE")
        if child and not path:
            raise ValueError("managed Rust child requires parent provenance")
        if not path:
            output = validation.fresh_output(
                ROOT, ROOT / "build/native-tests" / str(time.time_ns())
            )
            path = str(output / "native.json")
        provenance = Path(path).resolve()
        selected_path = Path(
            os.environ.get(
                "PSE_NATIVE_SELECTION",
                str(provenance.with_name("native-selected.json")),
            )
        ).resolve()
        if child:
            return managed_rust_run(extra, provenance)
        from scripts import surreal_server  # noqa: PLC0415 -- selected deployment owner

        execution, worker = managed_rust_capture(extra, provenance, selected_path)
        # Carry exact retained artifact paths through the existing observer launcher.
        prior = os.environ.get("PSE_NATIVE_PROVENANCE")
        os.environ["PSE_NATIVE_PROVENANCE"] = str(provenance)
        try:
            return surreal_server.observer(
                Path(os.environ["PSE_SURREAL_STATE"]),
                [
                    sys.executable,
                    "-m",
                    "scripts.native_tests",
                    "rust",
                    "--managed-primary-child",
                    f"--managed-primary-worker={worker}",
                    *execution,
                ],
            )
        finally:
            if prior is None:
                os.environ.pop("PSE_NATIVE_PROVENANCE", None)
            else:
                os.environ["PSE_NATIVE_PROVENANCE"] = prior

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
    observe_deployed_artifacts(environment)
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
