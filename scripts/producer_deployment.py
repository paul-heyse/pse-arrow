# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Capture reviewed actual runtime, worker and Python deployment producers.

Host review contexts are explicit inputs, never generated or refreshed here.
An actual imported Python run header must independently supply the deployment
attestation consumed by qualification; receipt metadata is not that observation.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
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


def receipt_outer(receipt: dict[str, object], role: str) -> dict[str, str] | None:
    outer = receipt.get("outer_attestation")
    if outer is None and role == "runtime":
        return None  # The runtime library archive has no executable buildinfo owner.
    if (
        not isinstance(outer, dict)
        or set(outer) != {"source", "build"}
        or not all(hash_value(outer[key]) for key in ("source", "build"))
    ):
        raise ValueError(f"{role} lacks an actual outer source/build attestation")
    return {key: str(outer[key]) for key in ("source", "build")}


def validate_receipt(output: Path, role: ProducerRole) -> dict[str, str] | None:
    name, package, target, kind, _ = role
    receipt = read_object(output / f"{name}.json")
    if receipt.get("frame") != "pse.producer.v1" or receipt.get("package") != package:
        raise ValueError(f"{name} receipt has the wrong frame or package")
    if (
        receipt.get("persistent_reuse_eligible") is not True
        or receipt.get("reasons") != []
    ):
        raise ValueError(f"{name} producer is ineligible or retains unresolved reasons")
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
    return receipt_outer(receipt, name)


def validate_receipts(
    root: Path, output: Path, *, expected_outer: dict[str, str] | None = None
) -> dict[str, str]:
    """Check captured roles and actual outer agreement; no identities are minted."""
    outers = {role[0]: validate_receipt(output, role) for role in ROLES}
    common = outers["worker"]
    if (
        common is None
        or outers["python"] != common
        or outers["runtime"] not in (None, common)
    ):
        raise ValueError(
            "Actual outer source/build attestation differs across deployment roles"
        )
    if expected_outer is not None and common != expected_outer:
        raise ValueError(
            "Captured outer attestation differs from independently observed deployment"
        )
    if not (root / "target/producer/pse-worker").is_file():
        raise ValueError("Actual producer worker binary is missing")
    return common


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
        validate_receipts(root, output)
    except (OSError, ValueError, KeyError, TypeError) as error:
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
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"Producer deployment capture failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps(environment, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
