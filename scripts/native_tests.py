# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Ordinary native test commands; selection belongs to nextest and pytest."""

from __future__ import annotations

import contextlib
import fnmatch
import hashlib
import json
import os
import shlex
import subprocess
import sys
import time
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import (
    arrow_validation,
    pse_env,
    python_extension,
    scientific_inputs,
    surreal_server,
)

if TYPE_CHECKING:
    from collections.abc import Generator, Mapping

from scripts import producer_deployment, validation, validation_receipts

ROOT = Path(__file__).resolve().parents[1]
FEATURES = "pse-runtime/native-solvers,pse-runtime/canonical-tests,pse-tests-conformance/native-acceptance,xtask/canonical-tests,pse-relations/force-validate"
MANAGED_FEATURES = ",".join(
    feature
    for feature in FEATURES.split(",")
    if feature.startswith(("pse-runtime/", "pse-relations/"))
)


@contextlib.contextmanager
def worker_binding(worker: Path) -> Generator[None, None, None]:
    """Bind the selected artifact before reference-state admission and launch."""
    prior = os.environ.get("PSE_WORKER_BINARY")
    os.environ["PSE_WORKER_BINARY"] = str(worker)
    try:
        yield
    finally:
        if prior is None:
            os.environ.pop("PSE_WORKER_BINARY", None)
        else:
            os.environ["PSE_WORKER_BINARY"] = prior


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
        scientific_inputs.ROLE,
        "PSE_CANONICAL_TOOLS",
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


def native_root_features(selection: list[str], *, managed: bool) -> str:
    """Select declared native opt-ins only on the actual requested workspace roots."""
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import test_invocation  # noqa: PLC0415

    request = test_invocation.rust(selection)
    requested = list(request.values("--package"))
    excluded = list(request.values("--exclude"))
    workspace = any(
        item.name in {"--workspace", "--all"} for item in request.options
    ) or (not requested and not managed)
    if not requested and not workspace:
        requested = ["pse-runtime"]
    declaration = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]
    features = []
    for pattern in declaration["members"]:
        for member in sorted(ROOT.glob(pattern)):
            manifest = member / "Cargo.toml"
            if not manifest.is_file():
                continue
            package = tomllib.loads(manifest.read_text())
            name = package["package"]["name"]
            if (
                not workspace
                and not any(
                    fnmatch.fnmatchcase(name, item.split(":", 1)[0])
                    for item in requested
                )
            ) or any(fnmatch.fnmatchcase(name, item) for item in excluded):
                continue
            declared = package.get("features", {})
            features.extend(
                f"{name}/{feature}"
                for feature in (
                    "native-solvers",
                    "canonical-tests",
                    "native-acceptance",
                )
                if feature in declared
            )
    return ",".join(sorted(set(features)))


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
        *([] if scoped else ["-p", "pse-runtime"] if managed else ["--workspace"]),
        "--locked",
        *(
            ["--features", features]
            if (features := native_root_features(extra, managed=managed))
            else []
        ),
        *extra,
    ]


def correctness_command(
    command: list[str], environment: Mapping[str, str]
) -> list[str]:
    """Compose the actual requested build under its own environment."""
    return correctness_resolution(command, environment).command


def correctness_resolution(
    command: list[str], environment: Mapping[str, str]
) -> arrow_validation.Resolution:
    return arrow_validation.resolve(
        command,
        run=lambda selected: subprocess.check_output(
            selected,
            cwd=ROOT,
            env=environment,
        ),
    )


def rust_source_identity() -> dict:
    """Use the existing complete Rust input owner, including missing tracked files."""
    return validation.sources(ROOT, scope="rust-product")


def rust_configuration_paths(
    arguments: list[str], environment: Mapping[str, str]
) -> list[str]:
    paths = {
        str(parent / ".cargo" / name)
        for parent in (ROOT, *ROOT.parents)
        for name in ("config", "config.toml")
    }
    cargo_home = Path(
        environment.get(
            "CARGO_HOME",
            str(Path(environment.get("HOME", str(Path.home()))) / ".cargo"),
        )
    )
    paths.update(str(cargo_home / name) for name in ("config", "config.toml"))
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import test_invocation  # noqa: PLC0415

    request = test_invocation.rust(arguments, environment)
    for name in ("--config", "--manifest-path", "--config-file"):
        for value in request.values(name):
            if name == "--manifest-path" or "=" not in value:
                paths.add(str((ROOT / value).absolute()))
    for value in request.values("--tool-config-file"):
        paths.add(str((ROOT / value.split(":", 1)[1]).absolute()))
    return sorted(paths)


def rust_configuration_identity(paths: list[str]) -> dict[str, str]:
    return {
        name: hashlib.sha256(
            b"file:" + Path(name).read_bytes() if Path(name).is_file() else b"absent"
        ).hexdigest()
        for name in paths
    }


# These values are introduced by the existing observer/terminal launch. They do
# not select a Cargo unit or test. All other environment changes, including
# previously unknown build-script inputs, invalidate preparation. Values are
# hashed so credentials cannot leak into the retained association.
RUST_EXECUTION_ENVIRONMENT = frozenset(
    {
        "PSE_NATIVE_PROVENANCE",
        "PSE_NATIVE_SELECTION",
        "PSE_TEST_INVOCATION",
        "PSE_TEST_EXECUTION_PROFILE",
        "PSE_TEST_EXECUTION_EFFECTS",
        "PSE_RUST_OBSERVER_CHILD",
        "PSE_RUST_TERMINAL_OWNER",
        "PSE_NATIVE_OPERATION",
        "PSE_NATIVE_HANDOFF",
        # Request-private observation/handoff values never select compiler behavior.
        "PSE_CAPABILITY_ADMISSION",
        "PSE_TELEMETRY_CARRIER",
        "PSE_NATIVE_TELEMETRY_SETTINGS",
        "FASTMCP_TELEMETRY_MODE",
        "PSE_CAPABILITIES_LANGUAGES",
        "PSE_CAPABILITIES_DATABASE",
        "PSE_CAPABILITIES_DATABASE_STATE",
        "PSE_NATIVE_WORKER_SLOT",
        "PSE_NATIVE_WORKER_MEMORY_BYTES",
        "PSE_MEMORY_MAX",
        "PSE_SURREAL_STATE",
        "PSE_HOST_ALLOCATION",
        "PSE_ADMISSION_DEADLINE",
        "PSE_RESOURCE_CLASS",
        "PSE_NATIVE_SETUP_PYTHON",
        "INVOCATION_ID",
        "JOURNAL_STREAM",
        "SYSTEMD_EXEC_PID",
    }
)


def rust_environment_identity(environment: Mapping[str, str]) -> dict[str, str]:
    # The observer receives a new authenticated allocation. Automatically chosen
    # executor widths follow that placement; explicit caller widths remain inputs.
    automatic_widths = {
        value
        for value, marker in (
            ("CARGO_BUILD_JOBS", "PSE_CARGO_DEFAULT_JOBS"),
            ("NEXTEST_TEST_THREADS", "PSE_NEXTEST_DEFAULT_THREADS"),
        )
        if environment.get(marker) is not None
        and environment.get(value) == environment[marker]
    }
    return {
        name: hashlib.sha256(value.encode()).hexdigest()
        for name, value in environment.items()
        if name not in RUST_EXECUTION_ENVIRONMENT
        and name not in automatic_widths
        and name not in {"PSE_CARGO_DEFAULT_JOBS", "PSE_NEXTEST_DEFAULT_THREADS"}
    }


def rust_selection_identity(
    arguments: list[str], environment: Mapping[str, str] | None = None
) -> dict:
    """Bind the shared effective selection independently of report placement."""
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import test_invocation  # noqa: PLC0415

    request = test_invocation.rust(arguments, environment)
    selection = []
    configs = []
    for item in request.options:
        if item.name in test_invocation.RUN_VALUE | test_invocation.RUN_FLAG | {
            "--profile"
        }:
            continue
        if item.name in {"--config-file", "--tool-config-file"}:
            value = str(item.value)
            tool, path = (
                value.split(":", 1)
                if item.name == "--tool-config-file"
                else ("", value)
            )
            config = tomllib.loads(Path(path).read_text())
            for name, profile in list(config.get("profile", {}).items()):
                profile.pop("junit", None)
                if not profile:
                    del config["profile"][name]
            configs.append([item.name, tool, config])
        else:
            selection.extend(
                [item.name] if item.value is None else [item.name, item.value]
            )
    selection.extend(request.opaque)
    if not any(item[0] == "--config-file" for item in configs):
        config = tomllib.loads((ROOT / ".config/nextest.toml").read_text())
        for name, profile in list(config.get("profile", {}).items()):
            profile.pop("junit", None)
            if not profile:
                del config["profile"][name]
        configs.insert(0, ["--config-file", "", config])
    configs.sort(key=lambda item: item[0] != "--config-file")
    return {
        "arguments": selection,
        "profile": request.nextest_profile,
        "configuration": configs,
    }


def prepared_rust_inventory(
    arguments: list[str],
    environment: Mapping[str, str],
    *,
    inventory: Mapping | None = None,
) -> dict:
    """Verify one prepared invocation before reusing its exact executable catalog."""
    provenance = environment.get("PSE_NATIVE_PROVENANCE")
    if not provenance:
        raise ValueError("Retained Rust inventory requires its preparation provenance")
    native = json.loads(Path(provenance).read_text())
    prepared = native.get("rust_invocation")
    if not isinstance(prepared, dict) or prepared.get("schema") != "prepared-rust-v1":
        raise ValueError("Unknown Rust preparation identity")
    required = {
        "source",
        "nextest",
        "environment",
        "selection",
        "configuration",
        "unit_graph",
        "inventory",
        "binaries_metadata",
        "cargo_metadata",
    }
    if (
        not required.issubset(prepared)
        or not isinstance(prepared["unit_graph"], dict)
        or prepared["unit_graph"].get("version") != 1
    ):
        raise ValueError("Unknown Rust preparation inputs or unit graph")
    if prepared["source"] != rust_source_identity():
        raise ValueError("Rust source or configuration changed after preparation")
    if prepared["configuration"] != rust_configuration_identity(
        list(prepared["configuration"])
    ):
        raise ValueError("Rust Cargo configuration changed after preparation")
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import cargo_tools  # noqa: PLC0415

    if prepared["nextest"] != cargo_tools.selected_identity("cargo-nextest"):
        raise ValueError("Prepared nextest tool identity changed before execution")
    actual_environment = rust_environment_identity(environment)
    if prepared["environment"] != actual_environment:
        names = sorted(
            name
            for name in prepared["environment"].keys() | actual_environment.keys()
            if prepared["environment"].get(name) != actual_environment.get(name)
        )
        raise ValueError(
            "Rust environment changed after preparation: " + ", ".join(names)
        )
    if prepared["selection"] != rust_selection_identity(arguments, environment):
        raise ValueError("Rust selection differs from prepared invocation")
    files = native.get("files")
    if not isinstance(files, dict) or not files:
        raise ValueError("Rust preparation lacks artifact identity")
    if any(
        prepared[key] not in files
        for key in ("inventory", "binaries_metadata", "cargo_metadata")
    ):
        raise ValueError("Rust preparation metadata is outside artifact provenance")
    for name, expected in files.items():
        if validation_receipts.digest(Path(name)) != expected:
            raise ValueError("Rust parent artifacts changed before execution")
    for option, key in (
        ("--binaries-metadata", "binaries_metadata"),
        ("--cargo-metadata", "cargo_metadata"),
    ):
        if arguments.count(option) != 1:
            raise ValueError("Rust execution requires retained nextest metadata")
        position = arguments.index(option) + 1
        if position >= len(arguments) or arguments[position] != prepared[key]:
            raise ValueError("Rust metadata differs from prepared invocation")
    retained = validation_receipts.native_inventory(
        Path(prepared["inventory"]).read_text()
    )
    if inventory is not None and inventory != retained:
        raise ValueError("Rust inventory differs from prepared invocation")
    fixtures = rust_fixture_bindings(Path(provenance), environment)
    if set(fixtures) - {"PSE_DIAGNOSTIC_EVIDENCE"} != selected_rust_fixtures(retained):
        raise ValueError("Rust selected fixtures differ from preparation")
    binaries = json.loads(Path(prepared["binaries_metadata"]).read_text()).get(
        "rust-binaries"
    )
    packages = json.loads(Path(prepared["cargo_metadata"]).read_text()).get("packages")
    graph = prepared["unit_graph"]
    if (
        not isinstance(binaries, dict)
        or not isinstance(packages, list)
        or not isinstance(graph.get("units"), list)
        or not isinstance(graph.get("roots"), list)
    ):
        raise TypeError("Invalid Rust package, binary or unit metadata shape")
    package_ids = {package["id"] for package in packages}
    roots = [graph["units"][index] for index in graph["roots"]]
    for suite in retained["rust-suites"].values():
        if any(
            test["filter-match"]["status"] == "matches"
            for test in suite["testcases"].values()
        ):
            binary = binaries.get(suite["binary-id"])
            keys = (
                "binary-id",
                "binary-name",
                "package-id",
                "kind",
                "binary-path",
                "build-platform",
            )
            if not isinstance(binary, dict) or any(
                key not in suite or suite[key] != binary.get(key) for key in keys
            ):
                raise ValueError("Rust suite differs from prepared binary metadata")
            if binary["package-id"] not in package_ids or not any(
                unit["pkg_id"] == binary["package-id"]
                and unit["target"]["name"] == binary["binary-name"]
                and binary["kind"] in unit["target"]["kind"]
                and unit["mode"] == "test"
                for unit in roots
            ):
                raise ValueError("Rust binary is outside validated Cargo roots")
            if str(Path(suite["binary-path"]).resolve()) not in files:
                raise ValueError("Rust executable is outside preparation provenance")
            if not os.access(suite["binary-path"], os.X_OK):
                raise ValueError("Rust prepared artifact is no longer executable")
    return retained


def rust_completion_arguments(extra: list[str]) -> list[str]:
    return [*([] if "--no-fail-fast" in extra else ["--no-fail-fast"]), *extra]


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
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import test_invocation  # noqa: PLC0415

    request = test_invocation.python(extra)
    selected, forwarded = request.marker, list(request.forwarded)
    partition = "managed_primary" if managed else "not managed_primary"
    return [
        sys.executable,
        "-m",
        "pytest",
        *PYTHON_DEFAULT_SELECTION,
        "--maxfail=0",
        "--continue-on-collection-errors",
        *(
            []
            if managed
            else [
                "-n",
                str(pse_env.default_test_width(os.environ)),
                "--dist",
                "loadgroup",
            ]
        ),
        *forwarded,
        "-m",
        f"({selected}) and {partition}",
        *(["-n", "0"] if managed else []),
        *request.opaque,
    ]


def _fixture_binary(
    extra: list[str],
    environment: Mapping[str, str],
    *,
    target: str,
    variable: str,
    features: str,
    role: str,
) -> Path:
    """Supply the actual child-process executable in the selected Cargo profile."""
    supplied = environment.get(variable)
    if supplied:
        binary = Path(supplied).resolve()
        if not binary.is_file() or not os.access(binary, os.X_OK):
            raise ValueError(f"{variable} must name an executable file")
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
        correctness_command(
            [
                "cargo",
                "build",
                "-p",
                "xtask",
                "--bin",
                target,
                "--locked",
                "--features",
                features,
                "--profile",
                profile,
                "--message-format",
                "json-render-diagnostics",
            ],
            environment,
        ),
        cwd=ROOT,
        env=environment,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
    )
    if built.returncode:
        print(built.stdout, end="")
        raise ValueError(f"{role} build failed with exit {built.returncode}")
    artifacts = set()
    for line in built.stdout.splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        if (
            message.get("reason") == "compiler-artifact"
            and message.get("target", {}).get("name") == target
            and message.get("executable")
        ):
            artifacts.add(Path(message["executable"]).resolve())
    if len(artifacts) != 1:
        raise ValueError(f"{role} build did not identify one executable")
    binary = artifacts.pop()
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise ValueError(f"{role} artifact is not executable")
    return binary


def canonical_tools_binary(extra: list[str], environment: Mapping[str, str]) -> Path:
    return _fixture_binary(
        extra,
        environment,
        target="xtask",
        variable="PSE_CANONICAL_TOOLS",
        features="canonical-tools",
        role="canonical initializer",
    )


def worker_binary(extra: list[str], environment: Mapping[str, str]) -> Path:
    return _fixture_binary(
        extra,
        environment,
        target="pse-worker",
        variable="PSE_WORKER_BINARY",
        features="native-solvers,canonical-tests",
        role="native worker",
    )


def native_lane_binary(extra: list[str], environment: Mapping[str, str]) -> Path:
    return _fixture_binary(
        extra,
        environment,
        target="pse-native-lane",
        variable="PSE_NATIVE_LANE_EXECUTABLE",
        features="native-solvers",
        role="native lane",
    )


def selected_rust_fixtures(inventory: dict) -> set[str]:
    """Preparation consumes the same requirements as actual effect admission."""
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import test_requirements  # noqa: PLC0415

    catalog = test_requirements.rust_catalog(inventory, validate_declarations=False)
    selected = {
        fixture
        for requirement in catalog.values()
        for fixture in requirement["fixtures"]
    }
    if scientific_inputs.selected_consumers(inventory):
        selected.add(scientific_inputs.ROLE)
    return selected


def prepare_rust_fixtures(
    inventory: dict,
    extra: list[str],
    destination: Path,
    environment: dict[str, str],
) -> dict[str, str]:
    """Prepare selected artifacts within the original source/configuration capture."""
    selected = selected_rust_fixtures(inventory)
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import test_requirements  # noqa: PLC0415

    requirements = test_requirements.rust_catalog(
        inventory, validate_declarations=False
    )
    if "PSE_NATIVE_LANE_EXECUTABLE" in selected:
        test_requirements.fixture_guard(
            requirements, "PSE_NATIVE_LANE_EXECUTABLE", "native"
        )
    bindings = {}
    if "PSE_CANONICAL_TOOLS" in selected:
        tool = environment.get("PSE_CANONICAL_TOOLS")
        evidence = environment.get("PSE_DIAGNOSTIC_EVIDENCE")
        if (
            not tool
            or not Path(tool).is_absolute()
            or not os.access(tool, os.X_OK)
            or not evidence
        ):
            raise ValueError(
                "Selected operator evidence requires explicit executable and evidence destination"
            )
        bindings["PSE_CANONICAL_TOOLS"] = tool
        bindings["PSE_DIAGNOSTIC_EVIDENCE"] = evidence
    if "PSE_NATIVE_LANE_EXECUTABLE" in selected:
        bindings["PSE_NATIVE_LANE_EXECUTABLE"] = str(
            native_lane_binary(extra, environment)
        )
    if "PSE_PRODUCER_FIXTURE_RECEIPT" in selected:
        supplied = environment.get("PSE_PRODUCER_FIXTURE_RECEIPT")
        receipt = Path(supplied).resolve() if supplied else destination.resolve()
        if not supplied:
            result = subprocess.run(
                ["just", "producer-fixture", str(receipt)],
                cwd=ROOT,
                env=environment,
                check=False,
            )
            if result.returncode:
                raise ValueError(
                    f"native producer fixture capture failed with exit {result.returncode}"
                )
        if not receipt.is_file():
            raise ValueError("producer fixture capture did not identify one receipt")
        bindings["PSE_PRODUCER_FIXTURE_RECEIPT"] = str(receipt)
    bindings.update(
        scientific_inputs.prepare(
            inventory,
            destination.with_name(destination.stem + "-scientific.json"),
            environment,
        )
    )
    environment.update(bindings)
    return bindings


def rust_fixture_bindings(
    provenance: Path, environment: Mapping[str, str]
) -> dict[str, str]:
    """Forward only exact source-bound fixture roles in the prepared artifact set."""
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import test_requirements  # noqa: PLC0415

    native = json.loads(provenance.read_text())
    prepared = native.get("rust_invocation", {})
    if not isinstance(prepared, dict):
        raise TypeError("Unknown Rust preparation identity")
    bindings = prepared.get("fixtures", {})
    if not isinstance(bindings, dict) or set(bindings) - (
        test_requirements.RUST_FIXTURES.keys()
        | {scientific_inputs.ROLE, "PSE_CANONICAL_TOOLS", "PSE_DIAGNOSTIC_EVIDENCE"}
    ):
        raise ValueError("Unknown prepared Rust fixture roles")
    files = native.get("files", {})
    recorded = prepared.get("environment", {})
    for name, value in bindings.items():
        if not isinstance(value, str) or str(Path(value).resolve()) != value:
            raise ValueError("Rust fixture path is not an exact prepared artifact")
        if name in environment and str(Path(environment[name]).resolve()) != value:
            raise ValueError(f"Rust fixture binding differs from preparation: {name}")
        if recorded.get(name) != hashlib.sha256(value.encode()).hexdigest():
            raise ValueError("Rust fixture role differs from prepared environment")
        if name == "PSE_DIAGNOSTIC_EVIDENCE":
            continue  # Authenticated output selection, not an immutable input file.
        if (
            value not in files
            or validation_receipts.digest(Path(value)) != files[value]
        ):
            raise ValueError("Rust fixture artifact changed after preparation")
        if name == "PSE_NATIVE_LANE_EXECUTABLE" and not os.access(value, os.X_OK):
            raise ValueError("Rust native lane artifact is not executable")
    return bindings


def managed_rust_arguments(
    extra: list[str], *, include_managed_features: bool = True
) -> tuple[list[str], list[str]]:
    """Project the shared request into retained build and execution inputs."""
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import test_invocation  # noqa: PLC0415

    request = test_invocation.rust(extra)
    if any(
        request.values(name)
        for name in ("--binaries-metadata", "--cargo-metadata", "--archive-file")
    ):
        raise ValueError("managed Rust execution owns its retained build metadata")
    return request.preparation(MANAGED_FEATURES if include_managed_features else "")


def managed_list_arguments(extra: list[str]) -> list[str]:
    """Inventory exactly the shared request without run-only controls."""
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import test_invocation  # noqa: PLC0415

    return test_invocation.rust(extra).listing()


def managed_cargo_profile(extra: list[str]) -> str:
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import test_invocation  # noqa: PLC0415

    return test_invocation.rust(extra).cargo_profile


def managed_rust_selection(extra: list[str]) -> list[str]:
    """Intersect the shared caller filter union with the observer partition."""
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import test_invocation  # noqa: PLC0415

    request = test_invocation.rust(extra)
    expressions = request.values("--filter-expr")
    forwarded = request.arguments(
        omit=frozenset({"--filter-expr", "--ignore-default-filter"})
    )
    position = forwarded.index("--") if "--" in forwarded else len(forwarded)
    expression = "test(managed_primary_)"
    if expressions:
        expression += (
            " and (" + " or ".join(f"({value})" for value in expressions) + ")"
        )
    forwarded[position:position] = ["--ignore-default-filter", "-E", expression]
    return forwarded


def managed_rust_capture(
    extra: list[str], provenance: Path, selected: Path
) -> tuple[list[str], Path]:
    """Build and enumerate before entering the observer's finite process cap."""
    scoped = managed_rust_selection(extra)
    execution, worker = _rust_capture(scoped, provenance, selected, managed=True)
    if worker is None:
        raise ValueError("Managed execution requires its worker")
    return execution, worker


def ordinary_rust_capture(
    command: list[str],
    provenance: Path,
    selected: Path,
    environment: Mapping[str, str],
    *,
    effects: str = "canonical",
    package_scope: bool = False,
    canonical: bool | None = None,
) -> tuple[list[str], Path | None]:
    """Retain exactly the ordinary caller's build and test selection."""
    return _rust_capture(
        command[command.index("run") + 1 :],
        provenance,
        selected,
        managed=False,
        environment=environment,
        canonical=effects == "canonical" if canonical is None else canonical,
        package_scope=package_scope,
    )


def scheduled_rust_arguments(
    arguments: list[str],
    inventory: dict,
    requirements: dict,
    destination: Path,
    environment: Mapping[str, str],
) -> list[str]:
    """Bind the exact collected scheduling policy before retained execution."""
    import tomlkit  # noqa: PLC0415

    from scripts import test_invocation, test_requirements  # noqa: PLC0415

    request = test_invocation.rust(arguments, environment)
    configurations = request.values("--config-file")
    source = (
        Path(configurations[0]) if configurations else ROOT / ".config/nextest.toml"
    )
    document = tomlkit.parse(source.read_text())
    # A caller-selected config may specialize policy; its group capacities must
    # be explicit instead of silently acquiring another scheduler's defaults.
    groups = document.get("test-groups", {})
    for requirement in requirements.values():
        group, _threads = test_requirements.scheduling(requirement)
        if group is not None and group not in groups:
            raise ValueError(
                f"Selected nextest configuration lacks required group {group}"
            )
    test_requirements.nextest_configuration(
        document, request.nextest_profile, inventory, requirements
    )
    destination.write_text(tomlkit.dumps(document))
    selected = request.arguments(omit=frozenset({"--config-file"}))
    position = selected.index("--") if "--" in selected else len(selected)
    selected[position:position] = ["--config-file", str(destination)]
    return selected


def _rust_capture(
    scoped: list[str],
    provenance: Path,
    selected: Path,
    *,
    managed: bool,
    environment: Mapping[str, str] | None = None,
    canonical: bool = True,
    package_scope: bool = False,
) -> tuple[list[str], Path | None]:
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import cargo_tools  # noqa: PLC0415

    # Refuse unsupported syntax and tool identity before worker/build preparation.
    managed_rust_arguments(scoped, include_managed_features=managed)
    nextest_identity = cargo_tools.selected_identity("cargo-nextest")
    provenance = provenance.resolve()
    selected = selected.resolve()
    execution, metadata_options = managed_rust_arguments(
        scoped,
        include_managed_features=managed,
    )
    binary_metadata = selected.with_name(
        selected.stem + "-binaries-metadata.json"
    ).resolve()
    cargo_metadata = selected.with_name(
        selected.stem + "-cargo-metadata.json"
    ).resolve()
    for destination in (provenance, selected, binary_metadata, cargo_metadata):
        destination.parent.mkdir(parents=True, exist_ok=True)
    environment = arrow_validation.rust_test_environment(
        os.environ if environment is None else environment
    )
    source = rust_source_identity()
    configuration_paths = rust_configuration_paths(scoped, environment)
    configuration = rust_configuration_identity(configuration_paths)
    # Managed controls live in the runtime unit binary, so their binary-name is
    # not "worker". They still require the actual qualified receiver artifact.
    profile = managed_cargo_profile(scoped)
    worker = None
    build_arguments = [
        "--list-type",
        "binaries-only",
        "--message-format",
        "json",
        *managed_list_arguments(scoped),
    ]
    offered_build = (
        rust_command("list", build_arguments, managed=True)
        if managed
        else ["cargo", "nextest", "list", *build_arguments]
    )
    if package_scope:
        resolution = arrow_validation.resolve(
            offered_build,
            run=lambda selected: subprocess.check_output(
                selected, cwd=ROOT, env=environment
            ),
        )
        build_command = resolution.command
        scope = " | ".join(f"package({name})" for name in resolution.packages)
        if not scope or "-E" not in execution:
            raise ValueError(
                "package-scoped invocation lacks selected Cargo roots or filter"
            )
        position = execution.index("-E") + 1
        execution[position] = f"({execution[position]}) & ({scope})"
        print(
            "select: " + shlex.join(["cargo", "nextest", "run", *execution]),
            file=sys.stderr,
            flush=True,
        )
    else:
        resolution = correctness_resolution(offered_build, environment)
        build_command = resolution.command
    _, metadata_options = managed_rust_arguments(
        build_command[3:],
        include_managed_features=False,
    )
    build = subprocess.run(
        build_command,
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
    # Correctness resolution already observed package metadata and the actual
    # validated target/host unit graph. Reuse them within this invocation.
    json.loads(resolution.metadata)
    cargo_metadata.write_bytes(resolution.metadata)
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
    data = validation_receipts.native_inventory(inventory.stdout)
    # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
    from scripts import test_requirements  # noqa: PLC0415

    packages = {
        item["id"]: item["name"] for item in json.loads(resolution.metadata)["packages"]
    }
    active_features = {
        packages[resolution.graph.units[index].pkg_id]: resolution.graph.units[
            index
        ].features
        for index in resolution.graph.roots
    }
    requirements = test_requirements.rust_catalog(
        data,
        required_roles=test_requirements.required_roles(environment),
        active_features=active_features,
    )
    if not requirements:
        # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
        from scripts import test_run  # noqa: PLC0415

        result = test_run.empty_selection(data, selected)
        raise ValueError(f"Known empty test selection; no execution: {result}")
    policy = selected.with_name(selected.stem + "-scheduling.toml")
    execution = scheduled_rust_arguments(
        execution, data, requirements, policy, environment
    )
    try:
        worker = (
            worker_binary(["--cargo-profile", profile], environment)
            if canonical
            and (
                managed
                or any("native" in item["effects"] for item in requirements.values())
            )
            else None
        )
        if worker is not None:
            environment["PSE_WORKER_BINARY"] = str(worker)
        else:
            environment.pop("PSE_WORKER_BINARY", None)
        fixtures = prepare_rust_fixtures(
            data,
            ["--cargo-profile", profile],
            selected.with_name(selected.stem + "-producer-fixture.json"),
            environment,
        )
    except Exception as error:
        # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
        from scripts import test_run  # noqa: PLC0415

        failed = test_run.preparation_failure(data, "fixture-preparation")
        error.add_note(f"Selected invocation: {failed}")
        raise
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
            "features": metadata_options[
                metadata_options.index("--features") + 1
            ].split(",")
            if "--features" in metadata_options
            else [],
            "cargo_profile": profile,
            "managed_build_arguments" if managed else "build_arguments": scoped,
        },
        [
            *binaries,
            *([str(worker)] if worker is not None else []),
            *(
                [fixtures["PSE_NATIVE_LANE_EXECUTABLE"]]
                if "PSE_NATIVE_LANE_EXECUTABLE" in fixtures
                else []
            ),
        ],
        environment=environment,
    )
    for destination in (selected, binary_metadata, cargo_metadata, policy):
        with destination.open("rb") as stream:
            native["files"][str(destination)] = hashlib.file_digest(
                stream, "sha256"
            ).hexdigest()
    if rust_source_identity() != source:
        raise ValueError("Rust source or configuration changed during preparation")
    if rust_configuration_identity(configuration_paths) != configuration:
        raise ValueError("Rust Cargo configuration changed during preparation")
    native["rust_invocation"] = {
        "schema": "prepared-rust-v1",
        "nextest": nextest_identity,
        "fixtures": fixtures,
        "requirements": requirements,
        "source": source,
        "configuration": configuration,
        "environment": rust_environment_identity(environment),
        "selection": rust_selection_identity([*reuse, *execution], environment),
        "unit_graph": json.loads(resolution.unit_graph),
        "inventory": str(selected.resolve()),
        "binaries_metadata": str(binary_metadata),
        "cargo_metadata": str(cargo_metadata),
    }
    validation.write_json(provenance, native)
    return [*reuse, *execution], worker


def observer_execution(environment: Mapping[str, str]) -> dict:
    """Observe the admitted child separately from its parent's preparation context."""
    profile = environment.get("PSE_TEST_EXECUTION_PROFILE", "")
    if profile not in {"reference", "exclusive-observer"}:
        raise ValueError("native execution requires its declared observer profile")
    state = Path(environment["PSE_SURREAL_STATE"]).resolve()
    config = surreal_server.config_for(state)
    resources = surreal_server.execution_resources(profile)
    if profile == "reference" and config["resources"] != resources:
        raise ValueError("managed native execution requires the reference allocation")
    execution = resources["execution"]
    if not isinstance(execution, dict):
        raise ValueError(  # noqa: TRY004 -- invalid evidence is a boundary refusal
            "native observer requires an explicit execution allocation"
        )
    cgroup = next(
        (
            line.removeprefix("0::")
            for line in Path("/proc/self/cgroup").read_text().splitlines()
            if line.startswith("0::")
        ),
        "",
    )
    group = Path("/sys/fs/cgroup") / cgroup.lstrip("/")
    memory, cpu = surreal_server.effective_limits(group)
    placement = surreal_server.placement_observation(state)
    physical_cpus = (
        placement.get("physical_cpus") if isinstance(placement, dict) else None
    )
    if not isinstance(physical_cpus, list) or any(
        not isinstance(physical_cpu, int) or isinstance(physical_cpu, bool)
        for physical_cpu in physical_cpus
    ):
        raise ValueError("native observer differs from its admitted allocation")
    cpus = [
        physical_cpu for physical_cpu in physical_cpus if isinstance(physical_cpu, int)
    ]
    owner = pse_env.host.inherit(environment)
    unbounded = (
        profile == "exclusive-observer"
        and owner is not None
        and owner.profile.unbounded
    )
    if unbounded and owner is not None:
        pse_env.require_unbounded_slice(pse_env.host.allocation_slice(owner))
    if (
        not group.is_relative_to(
            surreal_server.group_for_slice(surreal_server.execution_slice(state))
        )
        or memory != (None if unbounded else execution["observer_memory_bytes"])
        or cpu is None
        or cpu < execution["cpu_threads"]
        or not surreal_server.role_affinity_ready(os.getpid(), execution, cpus)
    ):
        raise ValueError("native observer differs from its admitted allocation")
    return {
        "captured": time.time(),
        "environment": validation.relevant_environment(environment),
        "profile": profile,
        "resources": resources,
        "state_resources": config["resources"],
        "control_group": cgroup,
        "memory_max": memory,
        "cpu_threads": cpu,
    }


def managed_rust_run(
    extra: list[str], provenance: Path, *, require_worker: bool = True
) -> int:
    """Reuse verified parent artifacts; no Cargo build or metadata query occurs here."""
    offered_workers = [
        argument
        for argument in extra
        if argument.startswith("--managed-primary-worker")
    ]
    if (
        any(
            not argument.startswith("--managed-primary-worker=")
            or not argument.split("=", 1)[1]
            for argument in offered_workers
        )
        or len(offered_workers) > 1
    ):
        raise ValueError(
            "managed Rust child requires one exact prebuilt worker argument"
        )
    worker_arguments = [argument.split("=", 1)[1] for argument in offered_workers]
    if require_worker and len(worker_arguments) != 1:
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
    environment = arrow_validation.rust_test_environment(os.environ)
    if worker_arguments:
        worker = Path(worker_arguments[0]).resolve()
        if (
            str(worker) not in files
            or not worker.is_file()
            or not os.access(worker, os.X_OK)
        ):
            raise ValueError("managed Rust worker is outside parent provenance")
        environment["PSE_WORKER_BINARY"] = str(worker)
    else:
        environment.pop("PSE_WORKER_BINARY", None)
    environment["PSE_NATIVE_PROVENANCE"] = str(provenance)
    environment.update(rust_fixture_bindings(provenance, environment))
    prepared = native.get("rust_invocation")
    if not isinstance(prepared, dict) or prepared.get("schema") != "prepared-rust-v1":
        raise ValueError("Unknown Rust preparation identity")
    inventory = validation_receipts.native_inventory(
        Path(prepared["inventory"]).read_text()
    )
    if not worker_arguments:
        # This preliminary classification permits only the read-only observer.
        # run_rust authenticates this exact inventory and all retained preparation
        # premises once at its existing execution boundary, before any effects.
        from scripts import test_requirements  # noqa: PLC0415 -- selected demand owner

        requirements = test_requirements.rust_catalog(
            inventory,
            required_roles=test_requirements.required_roles(environment),
            validate_declarations=False,
        )
        if (
            not requirements
            or not any("canonical" in item["effects"] for item in requirements.values())
            or any(
                set(item["effects"]) - {"canonical"} for item in requirements.values()
            )
        ):
            raise ValueError(
                "Workerless observer requires canonical-only selected demand"
            )
    native["execution"] = observer_execution(environment)
    validation.write_json(provenance, native)
    from scripts.test_run import run_rust  # noqa: PLC0415 -- owner cycle

    completion = [] if "--no-fail-fast" in extra else ["--no-fail-fast"]
    return run_rust(
        ["cargo", "nextest", "run", *completion, *extra],
        env=environment,
        inventory=inventory,
    )


def main() -> int:
    kind, *extra = sys.argv[1:]
    path = os.environ.get("PSE_NATIVE_PROVENANCE")
    if kind == "python":
        # selected test lifecycle owner; preserve source-only discovery and reciprocal imports
        from scripts import test_invocation  # noqa: PLC0415

        request = test_invocation.python(extra)
        controls = {item.name for item in request.options}
        from scripts import test_run  # noqa: PLC0415 -- observer composer

        managed = "--managed-primary-route" in controls
        child = "--managed-primary-child" in controls
        if child and not managed:
            raise ValueError("managed primary child requires its declared route")
        if managed and not child:
            state = os.environ.get("PSE_SURREAL_STATE")
            if not state:
                raise ValueError("managed primary route requires PSE_SURREAL_STATE")

            worker = worker_binary([], os.environ)
            with worker_binding(worker):
                command = [
                    sys.executable,
                    "-m",
                    "scripts.native_tests",
                    "python",
                    *test_invocation.append_options(extra, ["--managed-primary-child"]),
                ]
                return test_run.python_observer(
                    command,
                    extra,
                    lambda observation: surreal_server.observer(
                        surreal_server.reference_state(Path(state)),
                        command,
                        profile="reference",
                        observation=observation,
                    ),
                )
        if not managed and "--functional-observer-child" not in controls:
            command = [
                sys.executable,
                "-m",
                "scripts.native_tests",
                "python",
                *test_invocation.append_options(extra, ["--functional-observer-child"]),
            ]
            return test_run.python_observer(
                command,
                extra,
                lambda observation: surreal_server.observer(
                    Path(os.environ["PSE_SURREAL_STATE"]),
                    command,
                    profile="exclusive-observer",
                    observation=observation,
                ),
            )
        owners = request.values("--terminal-owner")
        if len(owners) > 1 or (
            owners and owners[0] not in {"assessment", "standalone"}
        ):
            raise ValueError("invalid terminal owner")
        owner = owners[0] if owners else "standalone"
        extra = request.arguments(
            omit=frozenset(
                {
                    "--terminal-owner",
                    "--managed-primary-route",
                    "--managed-primary-child",
                    "--functional-observer-child",
                }
            )
        )
        reports = request.values("--junitxml")
        if len(reports) > 1:
            raise ValueError("native Python execution requires one JUnit destination")
        report = Path(reports[0]) if reports else None
        if report is None:
            raise ValueError("native Python execution requires a JUnit destination")
        report.parent.mkdir(parents=True, exist_ok=True)
        provenance = Path(path) if path else report.with_suffix(".native.json")
        env = dict(os.environ)
        scientific_inputs.prepare_python(extra, env)
        artifact = python_extension.capture(env, native=True)
        binary = Path(str(artifact["binary"]))
        observe_deployed_artifacts(env, binary)
        native = native_provenance(
            {"features": artifact["features"]},
            [str(binary)],
            environment=env,
        )
        validation.write_json(provenance, native)
        if observed := env.pop("PSE_PYTHON_DEPLOYMENT_OBSERVATION_OUTPUT", None):
            if Path(observed).exists():
                raise ValueError("deployment observation output already exists")
            env["PSE_PYTHON_DEPLOYMENT_ATTESTATION"] = observed
        # The running pytest session must consume the same imported extension.
        if native["files"][str(binary)] != artifact["sha256"]:
            raise ValueError("Python extension changed during provenance capture")
        env = python_extension.execution_environment(env, artifact)
        selection = report.with_name(report.stem + "-selected.txt")
        if owner == "standalone":
            env["PSE_TEST_ENUMERATION"] = str(selection)
        from scripts import (  # noqa: PLC0415 -- runner cycle
            test_resources,
            test_run,
        )

        invocation = None
        if owner == "standalone" and test_resources.MARKER not in env:
            invocation = test_resources.invocation(
                "python", [], terminal_owner="standalone"
            )
            env[test_resources.MARKER] = str(invocation)
        command = python_command(extra, managed=managed)
        if invocation is None:
            return subprocess.call(command, cwd=ROOT, env=env)
        return test_run.execute_selected(
            invocation,
            command,
            env,
            report,
            selection=selection,
            terminal_owner="standalone",
            evidence_path=report.with_suffix(".terminal.json"),
            details={
                "mode": "python-native",
                "native": native,
                "native_provenance": str(provenance),
            },
        )
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

        execution, worker = managed_rust_capture(extra, provenance, selected_path)
        # Carry exact retained artifact paths through the existing observer launcher.
        prior = os.environ.get("PSE_NATIVE_PROVENANCE")
        os.environ["PSE_NATIVE_PROVENANCE"] = str(provenance)
        try:
            with worker_binding(worker):
                return surreal_server.observer(
                    surreal_server.reference_state(
                        Path(os.environ["PSE_SURREAL_STATE"])
                    ),
                    [
                        sys.executable,
                        "-m",
                        "scripts.native_tests",
                        "rust",
                        "--managed-primary-child",
                        f"--managed-primary-worker={worker}",
                        *execution,
                    ],
                    profile="reference",
                )
        finally:
            if prior is None:
                os.environ.pop("PSE_NATIVE_PROVENANCE", None)
            else:
                os.environ["PSE_NATIVE_PROVENANCE"] = prior

    if os.environ.get("PSE_NATIVE_OPERATION"):
        from scripts.test_run import run_rust  # noqa: PLC0415 -- owner cycle

        return run_rust(rust_command("run", rust_completion_arguments(extra)))
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
    environment = arrow_validation.rust_test_environment(os.environ)
    execution, worker = ordinary_rust_capture(
        rust_command("run", rust_completion_arguments(extra)),
        Path(path),
        selected_path,
        environment,
        canonical=True,
    )
    # Preparation, validation and enumeration share the exact resolved artifacts.
    # Execution consumes those artifacts rather than rebuilding the requested units.
    inventory = validation_receipts.native_inventory(selected_path.read_text())
    environment["PSE_NATIVE_PROVENANCE"] = str(Path(path).resolve())
    environment["PSE_NATIVE_SELECTION"] = str(selected_path.resolve())
    environment.update(rust_fixture_bindings(Path(path), environment))
    if worker is not None:
        environment["PSE_WORKER_BINARY"] = str(worker)
    from scripts.test_run import run_rust  # noqa: PLC0415 -- owner cycle

    return run_rust(
        ["cargo", "nextest", "run", *execution],
        env=environment,
        inventory=inventory,
    )


if __name__ == "__main__":
    raise SystemExit(main())
