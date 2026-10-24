# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Inspect the actual imported Python extension before correctness execution."""

from __future__ import annotations

import hashlib
import json
import os
import shlex
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

from scripts import arrow_validation

ROOT = Path(__file__).resolve().parents[1]


def maturin_cargo_command(command: Sequence[str], root: Path) -> list[str]:
    """Project supported PyO3 Maturin selections onto their Cargo library graph.

    Maturin 1.15's configure_bin_lib_flags selects --lib for PyO3. A build
    unit graph has the same dependency/feature closure as its cargo rustc call.
    Packaging and interpreter options stay on the original installer command.
    """
    if command[1:2] not in (["develop"], ["build"]):
        raise arrow_validation.ValidationError("expected Maturin develop or build")
    config = tomllib.loads((root / "pyproject.toml").read_text())["tool"]["maturin"]
    if config.get("bindings") != "pyo3":
        raise arrow_validation.ValidationError(
            "only the PyO3 library graph is supported"
        )
    manifest = config.get("manifest-path", "Cargo.toml")
    if not isinstance(manifest, str):
        raise arrow_validation.ValidationError("invalid Maturin manifest path")
    cli_keys = {argument.split("=", 1)[0] for argument in command[2:]}
    cargo = ["cargo", "build", "--lib"]
    if not cli_keys & {"--manifest-path", "-m"}:
        cargo.extend(["--manifest-path", manifest])
    for key in ("features", "config", "unstable-flags"):
        values = config.get(key, [])
        if not isinstance(values, list) or any(not isinstance(v, str) for v in values):
            raise arrow_validation.ValidationError(
                f"unsupported Maturin {key} selection"
            )
        for value in values:
            cargo.extend([{"unstable-flags": "-Z"}.get(key, f"--{key}"), value])
    for key in ("all-features", "no-default-features", "locked", "frozen"):
        if config.get(key, False) and f"--{key}" not in cli_keys:
            cargo.append(f"--{key}")
    profile = config.get("profile", "release")
    if command[1] == "develop":
        profile = config.get("editable-profile", profile)
    selected_profile = False
    cargo_values = {
        "--features",
        "-F",
        "--profile",
        "--target",
        "--target-dir",
        "--manifest-path",
        "-m",
        "--config",
        "-Z",
        "--jobs",
        "-j",
    }
    cargo_flags = {
        "--locked",
        "--frozen",
        "--offline",
        "--release",
        "-r",
        "--all-features",
        "--no-default-features",
        "--quiet",
        "-q",
        "--verbose",
        "-v",
        "-vv",
        "--future-incompat-report",
    }
    packaging_values = {
        "--interpreter",
        "-i",
        "--out",
        "-o",
        "--compression-method",
        "--compression-level",
    }
    packaging_flags = {
        "--uv",
        "--skip-install",
        "--strip",
        "--compression-enable-large-file-support",
    }
    aliases = {"-m": "--manifest-path", "-r": "--release"}
    index = 2
    while index < len(command):
        argument = command[index]
        key, separator, value = argument.partition("=")
        if key in cargo_values | packaging_values:
            if not separator:
                index += 1
                if index == len(command):
                    raise arrow_validation.ValidationError(f"missing value for {key}")
                value = command[index]
            if key in cargo_values:
                if (
                    key in {"--manifest-path", "-m"}
                    and (root / value).resolve() != (root / manifest).resolve()
                ):
                    raise arrow_validation.ValidationError(
                        "unsupported foreign Maturin project selection"
                    )
                cargo.extend([aliases.get(key, key), value])
                selected_profile |= key == "--profile"
        elif argument in cargo_flags:
            cargo.append(aliases.get(argument, argument))
            selected_profile |= argument in {"--release", "-r"}
        elif argument not in packaging_flags:
            raise arrow_validation.ValidationError(
                f"unsupported Maturin scope option {argument}"
            )
        index += 1
    if not selected_profile:
        if not isinstance(profile, str):
            raise arrow_validation.ValidationError("invalid Maturin profile")
        cargo.extend(["--profile", profile])
    return cargo


def validated_maturin_command(
    command: Sequence[str],
    environment: Mapping[str, str],
    *,
    wrapper: Sequence[str] = (),
    root: Path | None = None,
) -> list[str]:
    """Prove actual Arrow leaves before invoking the unchanged installer selection."""
    root = ROOT if root is None else root
    cargo = maturin_cargo_command(command, root)

    def capture_graph(selected: Sequence[str]) -> bytes:
        return capture_build_graph(selected, environment, wrapper=wrapper, root=root)

    resolution = arrow_validation.resolve(cargo, run=capture_graph)
    # resolve only appends opt-in features; keep Maturin's packaging options and
    # selected interpreter, profile, manifest and features on the actual build.
    return [*command, *resolution.command[len(cargo) :]]


def capture_build_graph(
    command: Sequence[str],
    environment: Mapping[str, str],
    *,
    wrapper: Sequence[str] = (),
    root: Path | None = None,
) -> bytes:
    """Use the installer's checkout, environment and capability boundary for Cargo."""
    selected = [*wrapper, *command]
    response = subprocess.run(
        selected,
        cwd=ROOT if root is None else root,
        env=environment,
        capture_output=True,
        check=False,
    )
    if response.returncode:
        raise arrow_validation.ValidationError(
            f"{shlex.join(selected)} failed:\n{response.stderr.decode()}"
        )
    return response.stdout


def maturin(command: Sequence[str]) -> int:
    """Run an explicitly selected correctness installer through graph assurance."""
    selected = validated_maturin_command(command, os.environ)
    print("arrow-validation: " + shlex.join(selected), file=sys.stderr, flush=True)
    return subprocess.call(selected, cwd=ROOT)


def capture(
    environment: Mapping[str, str], *, validated: bool = True, native: bool = False
) -> dict[str, object]:
    """Bind Rust cfg flags to the imported checkout path and observed binary bytes."""
    program = (
        "import json, sys, pse\n"
        "from pse._build import compiled_features\n"
        "print(json.dumps({'package': pse.__file__, "
        "'native': sys.modules['pse._native'].__file__, "
        "'features': compiled_features()}))\n"
    )
    try:
        imported = json.loads(
            subprocess.check_output(
                [sys.executable, "-c", program], cwd=ROOT, env=environment, text=True
            )
        )
    except (subprocess.CalledProcessError, json.JSONDecodeError) as error:
        raise ValueError(
            "cannot inspect compiled extension features; rebuild with just py-sync-correctness"
        ) from error
    if not isinstance(imported, dict):
        raise TypeError("invalid compiled extension report")
    package, binary, features = (
        imported.get("package"),
        imported.get("native"),
        imported.get("features"),
    )
    if not isinstance(package, str) or not isinstance(binary, str):
        raise TypeError("invalid compiled extension import paths")
    package_path, binary_path = Path(package).resolve(), Path(binary).resolve()
    expected = (ROOT / "python/pse").resolve()
    if package_path != expected / "__init__.py" or binary_path.parent != expected:
        raise ValueError("native Python import belongs to a different checkout")
    if (
        not isinstance(features, list)
        or any(not isinstance(feature, str) for feature in features)
        or len(features) != len(set(features))
    ):
        raise ValueError("invalid compiled extension feature flags")
    if validated and "force-validate" not in features:
        raise ValueError(
            "Python correctness requires a force-validated extension; run just py-sync-correctness"
        )
    if native and "native-solvers" not in features:
        raise ValueError(
            "native Python correctness requires the linked extension; run just py-sync-correctness native"
        )
    with binary_path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {"binary": str(binary_path), "sha256": digest, "features": features}


def execution_environment(
    environment: Mapping[str, str], artifact: Mapping[str, object]
) -> dict[str, str]:
    """Carry observations to pytest, which independently verifies its own import."""
    return {
        **environment,
        "PSE_NATIVE_EXPECTED_BINARY": str(artifact["binary"]),
        "PSE_NATIVE_EXPECTED_SHA256": str(artifact["sha256"]),
        "PSE_NATIVE_EXPECTED_FEATURES": json.dumps(artifact["features"]),
    }


def develop_command(root: Path, mode: str, profile: str) -> list[str]:
    """The actual Maturin correctness installer selection shared by strict capture."""
    if mode not in {"dev", "native"}:
        raise ValueError("correctness extension mode must be dev or native")
    return [
        str(root / ".venv/bin/maturin"),
        "develop",
        "--uv",
        "--profile",
        profile,
        "--locked",
        "--features",
        "force-validate,native-solvers" if mode == "native" else "force-validate",
    ]


def sync(mode: str, profile: str = "dev") -> int:
    """Build the explicitly validated correctness extension before test collection."""
    if mode == "auto":
        # This selects a build, never certifies its features. The resulting imported
        # binary is checked independently before correctness execution.
        from scripts import doctor  # noqa: PLC0415

        mode = doctor.extension_kind()
        if mode == "absent":
            mode = "dev"
    if mode not in {"dev", "native"}:
        raise ValueError("correctness extension mode must be dev, native or auto")
    wrapper = [str(ROOT / "scripts/pse-env"), "--recipe-default-class", "compile"]
    if mode == "native":
        wrapper.append("--native=solver,klu,isolation,uno,petsc")
    wrapper.append("--")
    environment = {**os.environ, "VIRTUAL_ENV": str(ROOT / ".venv")}
    commands = [
        ["uv", "sync", "--locked", "--no-install-project"],
        develop_command(ROOT, mode, profile),
        [
            "cargo",
            "run",
            "-p",
            "xtask",
            "--no-default-features",
            "--locked",
            "--",
            "python-stubs",
        ],
    ]
    for command in commands:
        if command[1:2] == ["develop"]:
            command = validated_maturin_command(command, environment, wrapper=wrapper)
        elif command[:2] == ["cargo", "run"]:
            command[2:2] = ["--bin", "xtask"]
            command = arrow_validation.resolve(
                command,
                run=lambda selected: capture_build_graph(
                    selected, environment, wrapper=wrapper
                ),
            ).command
        result = subprocess.call([*wrapper, *command], cwd=ROOT, env=environment)
        if result:
            return result
    return 0


def diagnostic_identity(selected: Path, environment: Mapping[str, str]) -> dict:
    """Require the installed diagnostic wheel's own binary and actual Rust flags."""
    program = (
        "import json,sys,pse; from pse._build import compiled_features; "
        "print(json.dumps({'package':pse.__file__,"
        "'binary':sys.modules['pse._native'].__file__,"
        "'prefix':sys.prefix,'features':compiled_features()}))"
    )
    report = json.loads(
        subprocess.check_output(
            [str(selected / "bin/python"), "-c", program],
            cwd=ROOT,
            env=dict(environment),
            text=True,
        )
    )
    if not isinstance(report, dict):
        raise TypeError("invalid diagnostic extension identity")
    binary, package = report.get("binary"), report.get("package")
    features = report.get("features")
    if (
        not isinstance(binary, str)
        or not isinstance(package, str)
        or not isinstance(features, list)
    ):
        raise TypeError("diagnostic extension identity is incomplete")
    if (
        report.get("prefix") != str(selected)
        or not Path(binary).is_relative_to(selected)
        or Path(binary).parent != Path(package).parent
    ):
        raise ValueError("diagnostic wheel imported a foreign package or extension")
    if not {"force-validate", "diagnostic-taskdump"} <= set(features):
        raise ValueError("installed wheel lacks actual diagnostic-taskdump flags")
    with Path(binary).open("rb") as source:
        report["sha256"] = hashlib.file_digest(source, "sha256").hexdigest()
    return report


def diagnostic_sync(mode: str = "dev") -> int:
    """Build/install the explicit cooperative extension into its separate environment.

    Never uses editable develop: that would replace the ordinary checkout binary.
    The wheel owns a snapshot of Python code and contracts alongside its actual binary.
    """
    from scripts import build_environment  # noqa: PLC0415

    if mode not in {"dev", "native"}:
        raise ValueError("diagnostic extension mode must be dev or native")
    selected = ROOT / ".venv-diagnostic-taskdump"
    environment = build_environment.configure(ROOT, dict(os.environ), taskdump=True)
    environment.update(UV_PROJECT_ENVIRONMENT=str(selected), VIRTUAL_ENV=str(selected))
    features = "force-validate,diagnostic-taskdump"
    if mode == "native":
        features += ",native-solvers"
    output = ROOT / "build/diagnostic-taskdump/wheels"
    output.mkdir(parents=True, exist_ok=True)
    commands = [
        ["uv", "sync", "--locked", "--no-install-project"],
        [
            str(ROOT / ".venv/bin/maturin"),
            "build",
            "--locked",
            "--profile",
            "diagnostic-taskdump",
            "--interpreter",
            str(selected / "bin/python"),
            "--features",
            features,
            "--compression-enable-large-file-support",
            "--compression-method",
            "stored",
            "--out",
            str(output),
        ],
    ]
    for command in commands:
        if command[1:2] == ["build"]:
            command = validated_maturin_command(command, environment)
        result = subprocess.call(command, cwd=ROOT, env=environment)
        if result:
            return result
    wheels = list(output.glob("pse_arrow-*.whl"))
    if len(wheels) != 1:
        raise ValueError("diagnostic wheel selection is missing or ambiguous")
    result = subprocess.call(
        [
            "uv",
            "pip",
            "install",
            "--python",
            str(selected / "bin/python"),
            "--force-reinstall",
            "--no-deps",
            str(wheels[0]),
        ],
        cwd=ROOT,
        env=environment,
    )
    if result:
        return result
    print(json.dumps(diagnostic_identity(selected, environment), sort_keys=True))
    return 0


if __name__ == "__main__":
    if sys.argv[1:2] == ["maturin"]:
        raise SystemExit(maturin(sys.argv[2:]))
    if len(sys.argv) == 3 and sys.argv[1] == "diagnostic-sync":
        raise SystemExit(diagnostic_sync(sys.argv[2]))
    if len(sys.argv) not in {3, 4} or sys.argv[1] != "sync":
        raise SystemExit(
            "usage: python -m scripts.python_extension sync dev|native|auto [profile]"
        )
    raise SystemExit(sync(sys.argv[2], sys.argv[3] if len(sys.argv) == 4 else "dev"))
