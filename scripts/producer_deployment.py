# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Capture reviewed actual runtime, worker and Python deployment producers.

Host review contexts are explicit inputs, never generated or refreshed here.
An actual imported Python run header must independently supply the deployment
attestation consumed by qualification; receipt metadata is not that observation.
"""

from __future__ import annotations

import argparse
import fcntl
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING, BinaryIO, TypeAlias

from scripts import (
    artifact_publication,
    installed_artifact,
    producer_cargo,
    producer_receipts,
    python_extension,
)
from scripts.producer_cargo import EFFECTIVE_ENVIRONMENT as DECLARED_ENVIRONMENT

if TYPE_CHECKING:
    from collections.abc import Mapping


ProducerRole: TypeAlias = tuple[str, str, str, str, tuple[str, ...]]


ROLES: tuple[ProducerRole, ...] = (
    ("runtime", "pse-runtime", "pse_runtime", "lib", ("--lib",)),
    ("worker", "pse-runtime", "pse-worker", "bin", ("--bin", "pse-worker")),
    (
        "python",
        "pse-py",
        "_native",
        "cdylib",
        ("--cdylib-manifest", "crates/pse-py/Cargo.toml"),
    ),
)


def input_path(root: Path, name: str, environment: Mapping[str, str]) -> Path:
    value = environment.get(name)
    if not value:
        raise ValueError(f"Current reviewed deployment requires {name}")
    path = Path(value)
    return path if path.is_absolute() else root / path


def native_inputs(path: Path, root: Path) -> list[Path]:
    value = json.loads(path.read_text())
    if (
        not isinstance(value, list)
        or not value
        or any(not isinstance(item, str) or not item for item in value)
    ):
        raise ValueError(f"Native inputs must be a nonempty JSON path list: {path}")
    paths = [Path(item) if Path(item).is_absolute() else root / item for item in value]
    if any(not item.is_file() for item in paths):
        raise ValueError(f"Declared native input file is missing from {path}")
    return paths


def reviewed_inputs(
    root: Path, environment: Mapping[str, str]
) -> dict[str, tuple[Path, list[Path]]]:
    common = native_inputs(
        input_path(root, "PSE_NATIVE_PRODUCER_INPUTS", environment), root
    )
    python_path = environment.get("PSE_PYTHON_PRODUCER_INPUTS")
    python = (
        native_inputs(input_path(root, "PSE_PYTHON_PRODUCER_INPUTS", environment), root)
        if python_path
        else common
    )
    result = {}
    for role, *_ in ROLES:
        declaration = input_path(
            root, f"PSE_{role.upper()}_PRODUCER_DECLARATIONS", environment
        )
        context = producer_receipts.read_object(declaration)
        if (
            context.get("capture_actual_build") is not True
            or context.get("native_inputs_complete") is not True
            or not isinstance(context.get("native_reviewed_source"), str)
            or not context["native_reviewed_source"]
        ):
            raise ValueError(
                f"{role} requires a current source-bound actual-build review context"
            )
        result[role] = (declaration, python if role == "python" else common)
    return result


def verify_artifact(
    root: Path, receipt: Path, artifact: Path, environment: Mapping[str, str]
) -> dict[str, object]:
    result = subprocess.run(
        [
            str(root / "target/debug/xtask"),
            "verify-deployment-artifact",
            "--receipt",
            str(receipt),
            "--artifact",
            str(artifact),
        ],
        cwd=root,
        env=dict(environment),
        capture_output=True,
        check=False,
        text=True,
    )
    if result.returncode:
        raise ValueError(
            f"Actual artifact/consumed-input association refused: {result.stderr}"
        )
    value = json.loads(result.stdout)
    if not isinstance(value, dict) or value != producer_receipts.file_observation(
        artifact
    ):
        raise ValueError("Independent artifact observation differs from actual file")
    return value


def load_history(
    root: Path,
    output: Path,
    role: str,
    environment: Mapping[str, str],
    receipt: Mapping[str, object],
) -> dict[str, object]:
    """Read only the sealed core product; historical context grants no freshness."""
    if not producer_receipts.current_receipt(dict(receipt)):
        raise ValueError(f"{role} has unsupported deployment receipt interpretation")
    result = subprocess.run(
        [
            str(root / "target/debug/xtask"),
            "producer-history-verify",
            str(output / f"{role}-build-evidence.json.closure"),
        ],
        cwd=root,
        env=dict(environment),
        capture_output=True,
        check=False,
        text=True,
    )
    if result.returncode:
        raise ValueError(
            f"{role} protected historical evidence refused: {result.stderr}"
        )
    history = json.loads(result.stdout)
    if (
        not isinstance(history, dict)
        or type(history.get("version")) is not int
        or history["version"] != 1
        or history.get("historical_only") is not True
        or not producer_receipts.hash_value(receipt.get("identity"))
        or not isinstance(receipt.get("selected_root"), str)
        or not receipt["selected_root"]
        or history.get("producer_identity") != receipt["identity"]
        or history.get("selected_root") != receipt["selected_root"]
    ):
        raise ValueError(
            f"{role} protected historical evidence differs from selected producer"
        )
    evidence = history.get("actual_build_evidence")
    if not isinstance(evidence, dict) or evidence.get("success") is not True:
        raise ValueError(f"{role} lacks sealed successful actual build evidence")
    return evidence


def bind_python_artifact(
    root: Path, output: Path, environment: Mapping[str, str]
) -> None:
    """Bind the strict owner's single imported candidate and publish its generation."""
    aliases = sorted((root / "python/pse").glob("_native*.so"))
    if len(aliases) != 1:
        raise ValueError("strict installer must supply exactly one extension image")
    path = output / "python.json"
    receipt = producer_receipts.read_object(path)
    evidence = load_history(root, output, "python", environment, receipt)
    built = producer_receipts.receipt_artifact(receipt, "python")
    original = Path(str(built["path"]))
    verify_artifact(root, path, original, environment)
    messages = [
        json.loads(line)
        for line in str(evidence["stdout"]).splitlines()
        if line.startswith("{")
    ]

    def bind_candidate(candidate: Path) -> list[str]:
        selected = {
            **environment,
            "_PSE_NATIVE_PUBLISHED_IMAGE": str(candidate.absolute()),
        }
        program = (
            "import json,sys,pse; from pse._build import compiled_features; "
            "print(json.dumps({'binary':sys.modules['pse._native'].__file__,'features':compiled_features()}))"
        )
        observed = json.loads(
            subprocess.check_output(
                [str(root / ".venv/bin/python"), "-c", program],
                cwd=root,
                env=selected,
                text=True,
            )
        )
        if not isinstance(observed, dict) or observed.get("binary") != str(
            candidate.absolute()
        ):
            raise ValueError(
                "strict constructor imported a different extension candidate"
            )
        features = observed.get("features")
        if (
            not isinstance(features, list)
            or any(not isinstance(item, str) for item in features)
            or not {"force-validate", "native-solvers"} <= set(features)
        ):
            raise ValueError(
                "strict Python deployment lacks actual native correctness features"
            )
        installed = producer_receipts.file_observation(candidate)
        transformation = installed_artifact.associate(
            original,
            candidate,
            str(built["sha256"]),
            str(installed["sha256"]),
            messages,
            environment=environment,
        )
        association = receipt["deployment"]
        if not isinstance(association, dict):
            raise TypeError("Python deployment association absent")
        association["artifact"] = installed
        path.write_text(json.dumps(receipt, indent=2) + "\n")
        verify_artifact(root, path, candidate, environment)
        producer_cargo.write_new(
            output / "python-installation.json",
            {
                "version": 1,
                "raw_artifact": built,
                "installed_artifact": installed,
                "transformation": transformation,
            },
        )
        return features

    artifact_publication.publish(
        output / "python-publication.json",
        kind="extension",
        source=aliases[0],
        mode="native",
        profile="producer",
        validated=True,
        interpreter=str(root / ".venv/bin/python"),
        features=bind_candidate,
    )


def validate_receipt(
    root: Path, output: Path, role: ProducerRole, environment: Mapping[str, str]
) -> dict[str, object]:
    name, package, target, kind, _ = role
    receipt = producer_receipts.read_object(output / f"{name}.json")
    evidence = load_history(root, output, name, environment, receipt)
    producer_receipts.receipt_artifact(receipt, name)
    if receipt.get("frame") != "pse.producer.v1" or receipt.get("package") != package:
        raise ValueError(f"{name} receipt has the wrong frame or package")
    if (
        receipt.get("persistent_reuse_eligible") is not True
        or receipt.get("reasons") != []
    ):
        raise ValueError(f"{name} producer is ineligible or retains unresolved reasons")
    if not isinstance(receipt.get("native_abi"), str) or not receipt["native_abi"]:
        raise ValueError(f"{name} lacks reviewed native ABI contract")
    if not producer_receipts.hash_value(receipt.get("identity")):
        raise ValueError(f"{name} receipt lacks a valid producer identity")
    units = receipt.get("units")
    selected = receipt.get("selected_root")
    if not isinstance(units, list) or not isinstance(selected, str) or not selected:
        raise ValueError(f"{name} receipt lacks its actual selected root")
    roots = [
        unit for unit in units if isinstance(unit, dict) and unit.get("key") == selected
    ]
    if len(roots) != 1:
        raise ValueError(f"{name} selected root is absent or ambiguous")
    unit = roots[0]
    kinds = unit.get("target_kind")
    profile = unit.get("profile")
    if (
        unit.get("target_name") != target
        or not isinstance(kinds, list)
        or kind not in kinds
        or unit.get("mode") != "build"
        or not isinstance(profile, dict)
        or profile.get("name") != "producer"
    ):
        raise ValueError(
            f"{name} selected root does not match its actual deployment role"
        )
    graph = evidence.get("unit_graph")
    if not isinstance(graph, dict):
        raise TypeError(f"{name} lacks actual selected build graph")
    indexes, graph_units = graph.get("roots"), graph.get("units")
    if (
        not isinstance(indexes, list)
        or len(indexes) != 1
        or not isinstance(indexes[0], int)
        or isinstance(indexes[0], bool)
        or not isinstance(graph_units, list)
        or not 0 <= indexes[0] < len(graph_units)
    ):
        raise ValueError(f"{name} actual build graph has no unique root")
    actual = graph_units[indexes[0]]
    if not isinstance(actual, dict) or (
        actual.get("target") is None
        or not isinstance(actual["target"], dict)
        or actual["target"].get("name") != target
        or actual["target"].get("kind") != kinds
        or actual.get("mode") != unit["mode"]
        or actual.get("profile") != profile
        or actual.get("features") != unit.get("features")
    ):
        raise ValueError(f"{name} receipt root differs from actual selected build root")
    return producer_receipts.receipt_artifact(receipt, name)


def validate_receipts(
    root: Path, output: Path, environment: Mapping[str, str]
) -> dict[str, dict[str, object]]:
    """Validate each actual role; role artifact digests have no equality rule."""
    associations = {
        role[0]: validate_receipt(root, output, role, environment) for role in ROLES
    }
    shared: dict[str, dict[str, set[str]]] = {}
    for role in ROLES:
        receipt = producer_receipts.read_object(output / f"{role[0]}.json")
        units = receipt.get("units")
        if not isinstance(units, list):
            raise TypeError("Malformed production unit list")
        for unit in units:
            if not isinstance(unit, dict):
                raise TypeError("Malformed production unit")
            key = json.dumps(
                [
                    unit.get("package_id"),
                    unit.get("target_name"),
                    unit.get("target_kind", []),
                    unit.get("mode"),
                    unit.get("platform"),
                ],
                sort_keys=True,
            )
            contract = json.dumps(
                [
                    unit.get("profile"),
                    unit.get("features"),
                    unit.get("dependencies"),
                ],
                sort_keys=True,
            )
            shared.setdefault(key, {}).setdefault(role[0], set()).add(contract)
    # Cargo can select several contexts for one logical unit inside a role.
    # A participating root may add helper contexts, but every common context
    # must remain an exact contract in one role's subset of the other's set.
    for grouped_roles in shared.values():
        role_contracts = list(grouped_roles.items())
        for index, (name, contracts) in enumerate(role_contracts):
            for other, other_contracts in role_contracts[:index]:
                if not (contracts <= other_contracts or other_contracts <= contracts):
                    raise ValueError(
                        f"Participating roles {other}/{name} have incompatible "
                        "shared unit contracts"
                    )
    return associations


def deployment_environment(root: Path, output: Path) -> dict[str, str]:
    """Pure environment selection; runtime is archived, worker/Python consume theirs."""
    root = root.resolve()
    output = (output if output.is_absolute() else root / output).resolve()
    return {
        "PSE_RUNTIME_PRODUCER_RECEIPT": str(output / "runtime.json"),
        "PSE_WORKER_PRODUCER_RECEIPT": str(output / "worker.json"),
        "PSE_PYTHON_PRODUCER_RECEIPT": str(output / "python.json"),
        "PSE_PRODUCER_RECEIPT": str(output / "worker.json"),
        "PSE_WORKER_BINARY": str(root / "target/producer/pse-worker"),
        "PSE_DEPLOYMENT_ARTIFACT_OBSERVATIONS": str(output / "observed-artifacts.json"),
        "PSE_PYTHON_DEPLOYMENT_ATTESTATION": str(output / "python-attestation.json"),
        "_PSE_NATIVE_PUBLICATION": str(output / "python-publication.json"),
    }


def require_capture_success(name: str, returncode: int) -> None:
    if returncode:
        raise ValueError(
            f"{name} actual producer capture failed ({returncode}); see {name}.log"
        )


def require_installer_capture(output: Path) -> None:
    path = output / "python-installer-cargo.status.json"
    if not path.is_file():
        raise ValueError("Maturin did not consume the selected Cargo observer")
    status = producer_receipts.read_object(path)
    if status.get("returncode") != 0 or status.get("sealed") is not True:
        raise ValueError("Maturin actual Cargo child lacks completed capture")


def own_preparation(root: Path, output: Path) -> BinaryIO:
    """Exclude other strict operations before dependency setup, without host locking."""
    directory = output / "preparation"
    directory.mkdir(mode=0o700)
    (root / "target").mkdir(exist_ok=True)
    descriptor = os.open(
        root / "target/.producer-preparation.lock",
        os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW,
        0o600,
    )
    guard = os.fdopen(descriptor, "r+b")
    try:
        fcntl.flock(guard, fcntl.LOCK_EX | fcntl.LOCK_NB)
        owner_pid = os.getpid()
        owner_start = (
            Path(f"/proc/{owner_pid}/stat").read_text().rsplit(") ", 1)[1].split()[19]
        )
        guard.seek(0)
        guard.truncate()
        guard.write(
            json.dumps(
                {
                    "directory": str(directory),
                    "owner_pid": owner_pid,
                    "owner_start": owner_start,
                }
            ).encode()
        )
        guard.flush()
        os.fsync(guard.fileno())
    except BaseException:
        guard.close()
        raise
    else:
        return guard


def begin_preparation(
    root: Path,
    output: Path,
    environment: Mapping[str, str],
    inputs: dict[str, tuple[Path, list[Path]]],
    guard: BinaryIO,
) -> None:
    """Establish the selected family baseline only after dependency setup."""
    directory = output / "preparation"
    guard.seek(0)
    owner = json.load(guard)
    owner_pid, owner_start = owner["owner_pid"], owner["owner_start"]
    selections = []
    for role, package, _, _, target_args in ROLES:
        declaration, files = inputs[role]
        target: str | dict[str, str] = "Library"
        if role == "worker":
            target = {"Binary": target_args[1]}
        elif role == "python":
            target = {"Cdylib": target_args[1]}
        selections.append(
            {
                "workspace_root": str(root),
                "package": package,
                "profile": "producer",
                "target": None,
                "production_target": target,
                "features": [
                    "native-solvers",
                    *(["product-operators"] if role == "worker" else []),
                    "force-validate"
                    if role == "python"
                    else "pse-relations/force-validate",
                ],
                "no_default_features": False,
                "dep_info": [],
                "declared_inputs": [],
                "native_inputs": [str(path) for path in files],
                "declared_environment": {},
                "declarations": producer_receipts.read_object(declaration),
            }
        )
    selection = directory / "selection.json"
    producer_cargo.write_new(selection, selections)
    with (directory / "preparation.log").open("xb") as log:
        # Preserve the existing capture-tool bootstrap boundary: cargo run would
        # inject launcher-local loader selections into the observed operation.
        result = subprocess.run(
            [
                "cargo",
                "build",
                "-p",
                "xtask",
                "--bin",
                "xtask",
                "--no-default-features",
                "--locked",
            ],
            cwd=root,
            env=dict(environment),
            stdout=log,
            stderr=subprocess.STDOUT,
            check=False,
            pass_fds=producer_cargo.preparation_child_fds(environment),
        )
        require_capture_success("preparation tool", result.returncode)
        result = subprocess.run(
            [
                str(root / "target/debug/xtask"),
                "producer-preparation-begin",
                "--selection",
                str(selection),
                "--directory",
                str(directory),
                "--owner-pid",
                str(owner_pid),
                "--owner-start",
                owner_start,
            ],
            cwd=root,
            env=dict(environment),
            stdout=log,
            stderr=subprocess.STDOUT,
            check=False,
            pass_fds=producer_cargo.preparation_child_fds(environment),
        )
    require_capture_success("preparation", result.returncode)


def capture(
    root: Path, output: Path, environment: Mapping[str, str] | None = None
) -> dict[str, str]:
    root = root.resolve()
    output = (output if output.is_absolute() else root / output).resolve()
    env = dict(os.environ if environment is None else environment)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.mkdir(mode=0o700)  # Never overwrite successful or failed capture evidence.
    guard: BinaryIO | None = None
    try:
        # Validate required operator selections before any effect; no source or
        # native digest baseline is taken until dependency setup has completed.
        reviewed_inputs(root, env)
        producer_cargo.cargo_executable(
            env
        )  # Refuse unsupported installer selection before role builds.
        guard = own_preparation(root, output)
        env[producer_cargo.PREPARATION_GUARD_FD] = str(guard.fileno())
        # Dependency installation is an effect before the shared source/native
        # baseline; it must not invalidate an already captured Rust role.
        with (output / "dependency-setup.log").open("xb") as log:
            result = subprocess.run(
                ["uv", "sync", "--locked", "--no-install-project"],
                cwd=root,
                env=env,
                stdout=log,
                stderr=subprocess.STDOUT,
                check=False,
                pass_fds=producer_cargo.preparation_child_fds(env),
            )
        require_capture_success("dependency setup", result.returncode)
        inputs = reviewed_inputs(root, env)
        begin_preparation(root, output, env, inputs, guard)
        for role in ROLES:
            name, package, _, _, target_args = role
            role_env = dict(env)
            if name != "python":
                role_env.pop("PYO3_BUILD_EXTENSION_MODULE", None)
                # The installed extension's explicit PyO3 configuration belongs
                # to its capture, not to the independent Rust library/worker.
                role_env.pop("PYO3_CONFIG_FILE", None)
            declarations, files = inputs[name]
            features = (
                "native-solvers,force-validate"
                if name == "python"
                else "native-solvers,product-operators,pse-relations/force-validate"
                if name == "worker"
                else "native-solvers,pse-relations/force-validate"
            )
            command = [
                "just",
                "producer-identity",
                "--package",
                package,
                "--profile",
                "producer",
                "--features",
                features,
                "--declarations",
                str(declarations),
                "--build-evidence",
                str(output / f"{name}-build-evidence.json"),
                "--output",
                str(output / f"{name}.json"),
                "--preparation",
                str(output / "preparation"),
                *target_args,
            ]
            for path in files:
                command.extend(("--native-input", str(path)))
            for key in DECLARED_ENVIRONMENT:
                if name != "python" and key in role_env:
                    command.extend(("--environment", f"{key}={role_env[key]}"))
            with (output / f"{name}.log").open("xb") as log:
                if name == "python":
                    # Maturin selects the only Python realization. Its actual Cargo
                    # child is observed and sealed before packaging can continue.
                    installer_env = {**role_env, "VIRTUAL_ENV": str(root / ".venv")}
                    installer_command = python_extension.validated_maturin_command(
                        python_extension.develop_command(root, "native", "producer"),
                        installer_env,
                        root=root,
                    )
                    # Graph discovery must precede installing the single-use Cargo
                    # observer: it neither consumes nor substitutes its actual child.
                    installer_env = producer_cargo.configure(
                        root, output, command[2:], installer_env
                    )
                    result = subprocess.run(
                        installer_command,
                        cwd=root,
                        env=installer_env,
                        stdout=log,
                        stderr=subprocess.STDOUT,
                        check=False,
                        pass_fds=producer_cargo.preparation_child_fds(installer_env),
                    )
                    require_capture_success(name, result.returncode)
                    require_installer_capture(output)
                else:
                    result = subprocess.run(
                        command,
                        cwd=root,
                        env=role_env,
                        stdout=log,
                        stderr=subprocess.STDOUT,
                        check=False,
                        pass_fds=producer_cargo.preparation_child_fds(role_env),
                    )
            require_capture_success(name, result.returncode)
            validate_receipt(root, output, role, role_env)
        bind_python_artifact(root, output, env)
        associations = validate_receipts(root, output, env)
        for name, artifact in associations.items():
            verify_artifact(
                root, output / f"{name}.json", Path(str(artifact["path"])), env
            )
        (output / "observed-artifacts.json").write_text(
            json.dumps(associations, indent=2) + "\n"
        )
        subprocess.run(
            [
                str(root / "target/debug/xtask"),
                "observe-deployment",
                "--output",
                str(output / "outer-observation.json"),
            ],
            cwd=root,
            env=env,
            check=True,
        )
        producer_cargo.write_new(
            output / "preparation/COMPLETE", {"terminal": "complete"}
        )
    except (
        OSError,
        ValueError,
        KeyError,
        TypeError,
        subprocess.CalledProcessError,
    ) as error:
        (output / "deployment-error.log").write_text(
            f"{type(error).__name__}: {error}\n"
        )
        if guard is not None and not (output / "preparation/FAILED").exists():
            producer_cargo.write_new(
                output / "preparation/FAILED", {"terminal": "failed"}
            )
        raise
    finally:
        if guard is not None:
            # Interrupted ownership is terminal as well; it cannot be resumed by
            # presenting a leftover completed-role marker.
            if (
                not (output / "preparation/COMPLETE").exists()
                and not (output / "preparation/FAILED").exists()
            ):
                producer_cargo.write_new(
                    output / "preparation/FAILED", {"terminal": "interrupted"}
                )
            guard.close()
    return deployment_environment(root, output)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    try:
        environment = capture(Path(__file__).resolve().parents[1], args.output)
    except (
        OSError,
        ValueError,
        KeyError,
        TypeError,
        subprocess.CalledProcessError,
    ) as error:
        print(f"Producer deployment capture failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps(environment, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
