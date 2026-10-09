# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Capture reviewed actual runtime, worker and Python deployment producers.

Host review contexts are explicit inputs, never generated or refreshed here.
An actual imported Python run header must independently supply the deployment
attestation consumed by qualification; receipt metadata is not that observation.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path
from typing import TYPE_CHECKING, TypeAlias

if TYPE_CHECKING:
    from collections.abc import Mapping


ProducerRole: TypeAlias = tuple[str, str, str, str, tuple[str, ...]]


ROLES: tuple[ProducerRole, ...] = (
    ("runtime", "pse-runtime", "pse_runtime", "lib", ("--lib",)),
    ("worker", "xtask", "pse-worker", "bin", ("--bin", "pse-worker")),
    (
        "python",
        "pse-py",
        "_native",
        "cdylib",
        ("--cdylib-manifest", "crates/pse-py/Cargo.toml"),
    ),
)
# Explicit compiler/tool inputs, never arbitrary environment or credentials.
DECLARED_ENVIRONMENT = (
    "CFLAGS_x86_64_unknown_linux_gnu",
    "CXXFLAGS_x86_64_unknown_linux_gnu",
    "CC_ENABLE_DEBUG_OUTPUT",
    "SOURCE_DATE_EPOCH",
    "TZ",
    "RUSTFMT",
    "LIBCLANG_PATH",
    "CLANG_NO_DEFAULT_CONFIG",
    "PYO3_CONFIG_FILE",
    "PYO3_PYTHON",
    "RUSTC_WRAPPER",
    "PSE_SCCACHE_BINARY",
    "RUSTC_WORKSPACE_WRAPPER",
    "PSE_NATIVE_COMPILER_CACHE",
    "PYO3_BUILD_EXTENSION_MODULE",
    "PSE_LLVM_PREFIX",
    "CLANG_PATH",
    "LLVM_CONFIG_PATH",
    "BINDGEN_EXTRA_CLANG_ARGS",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CFLAGS",
    "CXXFLAGS",
    "CPPFLAGS",
    "LDFLAGS",
)


def read_object(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise TypeError(f"Expected JSON object: {path}")
    return value


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
        context = read_object(declaration)
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


def hash_value(value: object) -> bool:
    return (
        isinstance(value, str)
        and re.fullmatch(r"(?:blake3:)?[0-9a-f]{64}", value) is not None
    )


def file_observation(path: Path) -> dict[str, object]:
    path = path.absolute()
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {
        "path": str(path),
        "sha256": digest,
        "canonical": str(path.resolve(strict=True)),
        "mode": path.stat().st_mode,
    }


def current_receipt(receipt: dict[str, object]) -> bool:
    version = receipt.get("receipt_version")
    return isinstance(version, int) and not isinstance(version, bool) and version == 2


def receipt_artifact(receipt: dict[str, object], role: str) -> dict[str, object]:
    if not current_receipt(receipt):
        raise ValueError(f"{role} has unsupported deployment receipt interpretation")
    association = receipt.get("deployment")
    if not isinstance(association, dict) or not isinstance(
        association.get("artifact"), dict
    ):
        raise TypeError(f"{role} lacks actual artifact association")
    artifact = association["artifact"]
    if not isinstance(artifact.get("path"), str) or artifact != file_observation(
        Path(artifact["path"])
    ):
        raise ValueError(f"{role} actual artifact changed or is incomplete")
    return artifact


def verify_native_provider(
    receipt_path: Path, package_id: str, out_dir: Path, environment: Mapping[str, str]
) -> bool:
    """Pure verification through the single Rust capture/framing owner; no Cargo."""
    try:
        value = read_object(receipt_path)
        if (
            not current_receipt(value)
            or value.get("persistent_reuse_eligible") is not True
            or value.get("reasons") != []
        ):
            return False
        association = value.get("deployment")
        if not isinstance(association, dict) or not isinstance(
            association.get("workspace_root"), str
        ):
            return False
        tool = Path(association["workspace_root"]) / "target/debug/xtask"
        result = subprocess.run(
            [
                str(tool),
                "verify-native-provider",
                "--receipt",
                str(receipt_path),
                "--package-id",
                package_id,
                "--out-dir",
                str(out_dir),
            ],
            env=dict(environment),
            capture_output=True,
            check=False,
        )
    except (OSError, ValueError, TypeError):
        return False
    else:
        return result.returncode == 0


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
    if not isinstance(value, dict) or value != file_observation(artifact):
        raise ValueError("Independent artifact observation differs from actual file")
    return value


def bind_python_artifact(
    root: Path, output: Path, environment: Mapping[str, str]
) -> None:
    """Associate independently imported bytes after exact admitted RPATH replay."""
    program = (
        "import json,sys,pse; print(json.dumps(sys.modules['pse._native'].__file__))"
    )
    imported = Path(
        json.loads(
            subprocess.check_output(
                [str(root / ".venv/bin/python"), "-c", program],
                cwd=root,
                env=dict(environment),
                text=True,
            )
        )
    ).resolve(strict=True)
    if imported.parent != (root / "python/pse").resolve():
        raise ValueError("Actual imported extension belongs to another checkout")
    path = output / "python.json"
    receipt = read_object(path)
    built = receipt_artifact(receipt, "python")
    original = Path(str(built["path"]))
    verify_artifact(root, path, original, environment)
    installed = file_observation(imported)
    if installed["sha256"] != built["sha256"]:
        evidence = read_object(output / "python-build-evidence.json")
        messages = [
            json.loads(line)
            for line in str(evidence["stdout"]).splitlines()
            if line.startswith("{")
        ]
        linked = []
        for message in messages:
            if message.get("reason") != "build-script-executed":
                continue
            libraries = message.get("linked_libs", [])
            if not any(
                "=" not in value or value.split("=", 1)[0].startswith("dylib")
                for value in libraries
            ):
                continue
            for value in message.get("linked_paths", []):
                if "=" not in value:
                    linked.append(value)
                elif value.split("=", 1)[0] in {"native", "all"}:
                    linked.append(value.split("=", 1)[1])
        old = (
            subprocess.check_output(
                ["patchelf", "--print-rpath", str(original)], text=True
            )
            .strip()
            .split(":")
        )
        old = [value for value in old if value]
        actual = (
            subprocess.check_output(
                ["patchelf", "--print-rpath", str(imported)], text=True
            )
            .strip()
            .split(":")
        )
        # Maturin 1.15 editable install appends dynamic linked paths absent from
        # the original RPATH. Cargo's independent message scheduling can reorder
        # the appended paths; their exact multiplicity and original prefix remain.
        appended = [value for value in linked if value not in old]
        if actual[: len(old)] != old or Counter(actual[len(old) :]) != Counter(
            appended
        ):
            raise ValueError(
                "Imported extension is not the admitted Maturin RPATH transformation"
            )
        with tempfile.TemporaryDirectory() as scratch:
            replay = Path(scratch) / original.name
            shutil.copyfile(original, replay)
            subprocess.run(["patchelf", "--remove-rpath", str(replay)], check=True)
            subprocess.run(
                [
                    "patchelf",
                    "--force-rpath",
                    "--set-rpath",
                    ":".join(actual),
                    str(replay),
                ],
                check=True,
            )
            if file_observation(replay)["sha256"] != installed["sha256"]:
                raise ValueError(
                    "Imported extension differs beyond the admitted RPATH-only transformation"
                )
    association = receipt["deployment"]
    if not isinstance(association, dict):
        raise TypeError("Python deployment association absent")
    association["artifact"] = installed
    path.write_text(json.dumps(receipt, indent=2) + "\n")
    verify_artifact(root, path, imported, environment)


def validate_receipt(output: Path, role: ProducerRole) -> dict[str, object]:
    name, package, target, kind, _ = role
    receipt = read_object(output / f"{name}.json")
    receipt_artifact(receipt, name)
    if receipt.get("frame") != "pse.producer.v1" or receipt.get("package") != package:
        raise ValueError(f"{name} receipt has the wrong frame or package")
    if (
        receipt.get("persistent_reuse_eligible") is not True
        or receipt.get("reasons") != []
    ):
        raise ValueError(f"{name} producer is ineligible or retains unresolved reasons")
    if not isinstance(receipt.get("native_abi"), str) or not receipt["native_abi"]:
        raise ValueError(f"{name} lacks reviewed native ABI contract")
    if not hash_value(receipt.get("identity")):
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
    evidence = read_object(output / f"{name}-build-evidence.json")
    if evidence.get("success") is not True:
        raise ValueError(f"{name} lacks successful actual build evidence")
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
    return receipt_artifact(receipt, name)


def validate_receipts(output: Path) -> dict[str, dict[str, object]]:
    """Validate each actual role; role artifact digests have no equality rule."""
    associations = {role[0]: validate_receipt(output, role) for role in ROLES}
    shared: dict[str, dict[str, set[str]]] = {}
    for role in ROLES:
        receipt = read_object(output / f"{role[0]}.json")
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
    }


def require_capture_success(name: str, returncode: int) -> None:
    if returncode:
        raise ValueError(
            f"{name} actual producer capture failed ({returncode}); see {name}.log"
        )


def capture(
    root: Path, output: Path, environment: Mapping[str, str] | None = None
) -> dict[str, str]:
    root = root.resolve()
    output = (output if output.is_absolute() else root / output).resolve()
    env = dict(os.environ if environment is None else environment)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.mkdir()  # A previous successful or failed capture cannot be overwritten.
    try:
        inputs = reviewed_inputs(root, env)
        for role in ROLES:
            name, package, _, _, target_args = role
            role_env = dict(env)
            # Replay Maturin's actual PyO3 extension linking selection; a Rust
            # library/worker build does not inherit that Python-only switch.
            if name == "python":
                role_env["PYO3_BUILD_EXTENSION_MODULE"] = "1"
            else:
                role_env.pop("PYO3_BUILD_EXTENSION_MODULE", None)
                # The installed extension's explicit PyO3 configuration belongs
                # to its capture, not to the independent Rust library/worker.
                role_env.pop("PYO3_CONFIG_FILE", None)
            declarations, files = inputs[name]
            features = (
                "native-solvers,force-validate"
                if name == "python"
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
                *target_args,
            ]
            for path in files:
                command.extend(("--native-input", str(path)))
            for key in DECLARED_ENVIRONMENT:
                if key in role_env:
                    command.extend(("--environment", f"{key}={role_env[key]}"))
            with (output / f"{name}.log").open("xb") as log:
                result = subprocess.run(
                    command,
                    cwd=root,
                    env=role_env,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    check=False,
                )
            require_capture_success(name, result.returncode)
            validate_receipt(output, role)
        bind_python_artifact(root, output, env)
        associations = validate_receipts(output)
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
        raise
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
