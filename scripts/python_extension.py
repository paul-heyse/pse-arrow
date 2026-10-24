# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Inspect the actual imported Python extension before correctness execution."""

from __future__ import annotations

import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING, Literal

from pydantic import BaseModel, ConfigDict

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence

from scripts import arrow_validation

ROOT = Path(__file__).resolve().parents[1]


class ExtensionContract(BaseModel):
    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )


class ExtensionArtifact(ExtensionContract):
    """Host-observed imported artifact identity, not a scientific package schema."""

    binary: str
    sha256: str
    features: list[str]


Stage = Literal["environment", "prepare", "stubs", "stubs-check", "admit"]
Mode = Literal["dev", "native"]


class StageResult(ExtensionContract):
    stage: Stage
    status: Literal["reused", "prepared", "checked", "admitted", "failed"]
    exit_code: int = 0
    artifact: ExtensionArtifact | None = None
    receipt: str | None = None
    published: bool = False


class PreparationResult(ExtensionContract):
    operation: Literal[
        "sync", "environment", "prepare", "stubs", "stubs-check", "admit"
    ]
    mode: Mode
    profile: str
    validated: bool
    stages: list[StageResult]
    exit_code: int


def maturin_cargo_command(command: Sequence[str], root: Path) -> list[str]:
    """Project supported PyO3 Maturin selections onto their Cargo library graph.

    Maturin 1.15's configure_bin_lib_flags selects --lib for PyO3. A build
    unit graph has the same dependency/feature closure as its cargo rustc call.
    Packaging and interpreter options stay on the original installer command.
    """
    if command[1:2] not in (["develop"], ["build"]):
        raise arrow_validation.ValidationError("expected Maturin develop or build")
    config = tomllib.loads((root / "pyproject.toml").read_text())["tool"]["maturin"]
    supported = {
        "bindings",
        "manifest-path",
        "module-name",
        "python-source",
        "python-packages",
        "locked",
        "frozen",
        "sdist-generator",
        "profile",
        "editable-profile",
        "strip",
        "auditwheel",
        "include",
        "exclude",
        "features",
        "config",
        "unstable-flags",
        "all-features",
        "no-default-features",
        # Maturin 1.15 ToolMaturin packaging/metadata settings do not select
        # another Cargo root or conditional feature closure for this direct route.
        "compatibility",
        "manylinux",
        "skip-auditwheel",
        "data",
        "sbom",
        "include-import-lib",
        "generate-ci",
        "pgo-command",
        "use-base-python",
    }
    if unknown := set(config) - supported:
        raise arrow_validation.ValidationError(
            "unsupported Maturin configuration: " + ", ".join(sorted(unknown))
        )
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
    if environment.get("MATURIN_PGO"):
        raise arrow_validation.ValidationError(
            "unsupported multiphase Maturin PGO selection"
        )
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
        "'native': sys.modules['pse._native'].__file__, 'prefix': sys.prefix, "
        "'features': compiled_features()}))\n"
    )
    try:
        imported = json.loads(
            subprocess.check_output(
                [str(interpreter(environment)), "-c", program],
                cwd=ROOT,
                env=environment,
                text=True,
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
    if imported.get("prefix") != str(interpreter(environment).parent.parent):
        raise ValueError("native Python import belongs to a different environment")
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


def develop_command(
    root: Path,
    mode: str,
    profile: str,
    *,
    environment: Mapping[str, str] | None = None,
    validated: bool = True,
) -> list[str]:
    """The actual Maturin correctness installer selection shared by strict capture."""
    if mode not in {"dev", "native"}:
        raise ValueError("correctness extension mode must be dev or native")
    return [
        str(
            (
                Path(environment["VIRTUAL_ENV"])
                if environment is not None
                else root / ".venv"
            )
            / "bin/maturin"
        ),
        "develop",
        "--uv",
        "--profile",
        profile,
        "--locked",
        *(
            [
                "--features",
                ",".join(
                    [
                        *(["force-validate"] if validated else []),
                        *(["native-solvers"] if mode == "native" else []),
                    ]
                ),
            ]
            if validated or mode == "native"
            else []
        ),
    ]


def selected_environment(
    environment: Mapping[str, str] | None = None,
) -> dict[str, str]:
    """Resolve the environment once through the existing checkout owner."""
    from scripts import pse_env  # noqa: PLC0415

    return bind_interpreter(
        pse_env.compose(ROOT, os.environ if environment is None else environment)
    )


def bind_interpreter(environment: Mapping[str, str]) -> dict[str, str]:
    """Bind installer aliases in an already resolved checkout environment."""
    from scripts import pse_env  # noqa: PLC0415

    result = dict(environment)
    selected = pse_env.venv(ROOT, result).absolute()
    result.update(UV_PROJECT_ENVIRONMENT=str(selected), VIRTUAL_ENV=str(selected))
    return result


def interpreter(environment: Mapping[str, str]) -> Path:
    from scripts import pse_env  # noqa: PLC0415

    return pse_env.venv(ROOT, environment) / "bin/python"


def file_identity(path: Path) -> dict[str, object]:
    """Include absence, link spelling, target bytes and executable mode."""
    return {
        "path": str(path.absolute()),
        "resolved": str(path.resolve()),
        "link": str(path.readlink()) if path.is_symlink() else None,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest()
        if path.is_file()
        else None,
        "mode": path.stat().st_mode & 0o777 if path.is_file() else None,
    }


def tool_identity(name: str, environment: Mapping[str, str]) -> dict[str, object]:
    selected = shutil.which(name, path=environment.get("PATH"))
    if selected is None:
        raise ValueError(f"Python preparation tool unavailable: {name}")
    return file_identity(Path(selected))


def environment_ensure(environment: Mapping[str, str]) -> StageResult:
    """Ask uv about the selected locked installation before mutating it."""
    command = ["uv", "sync", "--locked", "--no-install-project", "--inexact"]
    if subprocess.call([*command, "--check"], cwd=ROOT, env=environment) == 0:
        return StageResult(stage="environment", status="reused")
    code = subprocess.call(command, cwd=ROOT, env=environment)
    return StageResult(
        stage="environment", status="failed" if code else "prepared", exit_code=code
    )


PREPARATION_SESSION_ENVIRONMENT = frozenset(
    {
        "CODEX_SESSION_ID",
        "CODEX_THREAD_ID",
        "CODEX_TOOL_CALL_ID",
        "DIRENV_DIFF",
        "DIRENV_WATCHES",
        "DIRENV_DIR",
        "DIRENV_FILE",
    }
)


def preparation_environment_identity(environment: Mapping[str, str]) -> dict[str, str]:
    """Bind build inputs without tool-call and shell restoration bookkeeping.

    Direnv's evaluated exports and local configuration remain inputs. Its diff and
    watch state describe how to restore the invoking shell, while the Codex
    identifiers route the invoking session. Unknown variables remain bound.
    Scientific execution provenance retains its independent full environment.
    """
    from scripts import native_tests  # noqa: PLC0415

    return native_tests.rust_environment_identity(
        {
            name: value
            for name, value in environment.items()
            if name not in PREPARATION_SESSION_ENVIRONMENT
        }
    )


def preparation_inputs(
    stage: str,
    mode: str,
    profile: str,
    environment: Mapping[str, str],
    *,
    validated: bool,
) -> dict[str, object] | None:
    """Known ordinary editable inputs; unknown native providers always prepare.

    Managed providers have already been admitted and sealed by pse-env. Consume
    that owner here rather than rescanning native installation trees.
    """
    from scripts import native_operation, native_tests, validation  # noqa: PLC0415

    # Cross compilation and additional Maturin environment selection do not
    # belong to the supported ordinary editable domain. They still prepare.
    if any(name.startswith(("PYO3_CROSS", "MATURIN_")) for name in environment):
        return None
    files = validation.sources(ROOT, scope=stage)
    if any(
        (ROOT / name).is_symlink() and (ROOT / name).is_dir()
        for name in files
        if not name.startswith("resolved:")
    ):
        return None
    generations = {}
    if mode == "native":
        for name in (
            "IPOPT_DIR",
            "UNO_DIR",
            "PETSC_DIR",
            "PSE_ROOT_ISOLATION_DIR",
            "SUITESPARSE_LIBRARY_DIR",
        ):
            value = environment.get(name)
            if not value:
                return None
            prefix = Path(value).resolve()
            if name == "SUITESPARSE_LIBRARY_DIR":
                prefix = prefix.parent
            admitted = native_operation.remembered("prefix:" + str(prefix))
            if not isinstance(admitted, dict) or not admitted.get("managed"):
                return None
            generations[name] = {
                "prefix": str(prefix),
                "admission": admitted,
                "receipt": file_identity(prefix / ".complete.json"),
            }
    selected_python = interpreter(environment)
    configuration = json.loads(
        subprocess.check_output(
            [
                str(selected_python),
                "-c",
                "import json,sys,sysconfig; print(json.dumps({'executable':sys.executable,'prefix':sys.prefix,'version':sys.version,'config':sysconfig.get_config_vars()}))",
            ],
            cwd=ROOT,
            env=environment,
            text=True,
        )
    )
    tools = {
        name: tool_identity(name, environment)
        for name in ("cargo", "rustc", "uv", "cc", "c++", "pkg-config", "clang", "mold")
    }
    tools["maturin"] = file_identity(interpreter(environment).parent / "maturin")
    sysroot = Path(
        subprocess.check_output(
            ["rustc", "--print", "sysroot"], cwd=ROOT, env=environment, text=True
        ).strip()
    )
    for name in ("rustc", "cargo"):
        tools["toolchain-" + name] = file_identity(sysroot / "bin" / name)
    if stage == "python-stubs":
        tools["ruff"] = file_identity(interpreter(environment).parent / "ruff")
    configuration_paths = native_tests.rust_configuration_paths([], environment)
    for name in ("PYO3_CONFIG_FILE", "CMAKE_TOOLCHAIN_FILE", "SCCACHE_CONF"):
        if environment.get(name):
            configuration_paths.append(environment[name])
    for name in (
        "PYO3_PYTHON",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "CC",
        "CXX",
        "FC",
        "CLANG_PATH",
        "LLVM_CONFIG_PATH",
    ):
        if environment.get(name):
            executable = shutil.which(environment[name], path=environment.get("PATH"))
            if executable is None:
                return None
            tools[name] = file_identity(Path(executable))
    return {
        "schema": "ordinary-editable-inputs-v1",
        "stage": stage,
        "mode": mode,
        "profile": profile,
        "validated": validated,
        "files": files,
        "environment": preparation_environment_identity(environment),
        "configuration": {
            name: file_identity(Path(name)) for name in sorted(set(configuration_paths))
        },
        "interpreter": {
            "binary": file_identity(selected_python),
            "configuration": configuration,
            "venv": file_identity(selected_python.parent.parent / "pyvenv.cfg"),
        },
        "compiler": subprocess.check_output(
            ["rustc", "-Vv"], cwd=ROOT, env=environment, text=True
        ),
        "tools": tools,
        "generations": generations,
    }


def stage_receipt(
    stage: str, environment: Mapping[str, str], *, root: Path | None = None
) -> Path:
    from scripts import pse_env  # noqa: PLC0415

    selected_root = ROOT if root is None else root
    selected_python = pse_env.venv(selected_root, environment) / "bin/python"
    key = hashlib.sha256(str(selected_python.absolute()).encode()).hexdigest()[:20]
    return selected_root / "build/python-extension" / key / f"{stage}.json"


def read_receipt(path: Path) -> dict[str, object]:
    try:
        result = json.loads(path.read_text())
        return result if isinstance(result, dict) else {}
    except (OSError, ValueError):
        return {}


def publish_receipt(path: Path, value: dict[str, object]) -> None:
    from scripts import validation  # noqa: PLC0415

    path.parent.mkdir(parents=True, exist_ok=True)
    validation.write_json(path, value)


def extension_prepare(
    mode: str, profile: str, environment: Mapping[str, str], *, validated: bool = True
) -> StageResult:
    """Install only when a completed supported preparation no longer applies."""
    command = develop_command(
        ROOT, mode, profile, environment=environment, validated=validated
    )
    # Refuse unsupported correctness selections before any currency shortcut.
    maturin_cargo_command(command, ROOT)
    inputs = preparation_inputs(
        "python-extension", mode, profile, environment, validated=validated
    )
    path = stage_receipt("extension", environment)
    previous = read_receipt(path)
    if inputs is not None and previous.get("inputs") == inputs:
        # A stale or unsupported preparation must build regardless of the old
        # installed bytes. Import/hash them only when they can establish reuse.
        try:
            installed = capture(
                environment, validated=validated, native=mode == "native"
            )
        except (ValueError, OSError, subprocess.SubprocessError):
            installed = None
        if installed is not None and previous == {
            "inputs": inputs,
            "artifact": installed,
        }:
            return StageResult(
                stage="prepare",
                status="reused",
                receipt=str(path),
                artifact=ExtensionArtifact.model_validate(installed),
            )
    if validated:
        command = validated_maturin_command(command, environment)
    result = subprocess.call(command, cwd=ROOT, env=environment)
    if result:
        return StageResult(stage="prepare", status="failed", exit_code=result)
    installed = capture(environment, validated=validated, native=mode == "native")
    after = preparation_inputs(
        "python-extension", mode, profile, environment, validated=validated
    )
    if after != inputs:
        raise ValueError("Python extension inputs changed during preparation")
    # Bind installed/imported bytes. Maturin may legitimately transform editable
    # RPATH bytes, so raw Cargo-artifact equality is not a currency premise.
    if inputs is not None:
        publish_receipt(path, {"inputs": inputs, "artifact": installed})
    return StageResult(
        stage="prepare",
        status="prepared",
        artifact=ExtensionArtifact.model_validate(installed),
        receipt=str(path) if inputs is not None else None,
        published=inputs is not None,
    )


def stub_outputs() -> dict[str, object]:
    root = ROOT / "docs/generated/source-relationships/native-stubs"
    names = [ROOT / "python/pse/_native.pyi", *sorted(root.rglob("*"))]
    return {
        str(path.relative_to(ROOT)): file_identity(path)
        for path in names
        if not path.is_dir()
    }


def stubs_prepare(
    mode: str,
    profile: str,
    environment: Mapping[str, str],
    *,
    validated: bool = True,
    check: bool = False,
) -> StageResult:
    """Independent declaration/tool/binary/output currency for actual API stubs."""
    artifact = capture(environment, validated=validated, native=mode == "native")
    inputs = preparation_inputs(
        "python-stubs", mode, profile, environment, validated=validated
    )
    expected = {"inputs": inputs, "artifact": artifact, "outputs": stub_outputs()}
    path = stage_receipt("stubs", environment)
    if not check and inputs is not None and read_receipt(path) == expected:
        return StageResult(
            stage="stubs",
            status="reused",
            receipt=str(path),
            artifact=ExtensionArtifact.model_validate(artifact),
        )
    command = [
        "cargo",
        "run",
        "--bin",
        "xtask",
        "-p",
        "xtask",
        "--no-default-features",
        "--locked",
        "--",
        "python-stubs",
        *(["--check"] if check else []),
    ]
    command = arrow_validation.resolve(
        command, run=lambda selected: capture_build_graph(selected, environment)
    ).command
    result = subprocess.call(command, cwd=ROOT, env=environment)
    if result:
        return StageResult(
            stage="stubs-check" if check else "stubs", status="failed", exit_code=result
        )
    if artifact != capture(
        environment, validated=validated, native=mode == "native"
    ) or inputs != preparation_inputs(
        "python-stubs", mode, profile, environment, validated=validated
    ):
        raise ValueError("Python stub inputs changed during generation")
    if inputs is not None and not check:
        publish_receipt(
            path, {"inputs": inputs, "artifact": artifact, "outputs": stub_outputs()}
        )
    return StageResult(
        stage="stubs-check" if check else "stubs",
        status="checked" if check else "prepared",
        artifact=ExtensionArtifact.model_validate(artifact),
        receipt=str(path) if inputs is not None and not check else None,
        published=inputs is not None and not check,
    )


def run_stages(
    operation: str,
    mode: str,
    profile: str,
    environment: Mapping[str, str],
    *,
    validated: bool = True,
    on_stage: Callable[[str, StageResult | None], None] | None = None,
) -> PreparationResult:
    """Run selected owner stages in an already resolved environment, without relaunch."""
    if operation not in {
        "sync",
        "environment",
        "prepare",
        "stubs",
        "stubs-check",
        "admit",
    }:
        raise ValueError("unsupported Python preparation stage")
    if mode not in {"dev", "native"}:
        raise ValueError("resolved extension mode must be dev or native")
    stages: list[StageResult] = []
    selected = (
        ("environment", "prepare", "stubs", "admit")
        if operation == "sync"
        else (operation,)
    )
    for stage in selected:
        progress_stage = (
            "extension"
            if stage == "prepare"
            else "stubs"
            if stage == "stubs-check"
            else stage
        )
        if on_stage is not None:
            on_stage(progress_stage, None)
        if stage == "environment":
            result = environment_ensure(environment)
        elif stage == "prepare":
            result = extension_prepare(mode, profile, environment, validated=validated)
        elif stage in {"stubs", "stubs-check"}:
            result = stubs_prepare(
                mode,
                profile,
                environment,
                validated=validated,
                check=stage == "stubs-check",
            )
        else:
            result = StageResult(
                stage="admit",
                status="admitted",
                artifact=ExtensionArtifact.model_validate(
                    capture(environment, validated=validated, native=mode == "native")
                ),
            )
        stages.append(result)
        if on_stage is not None:
            on_stage(progress_stage, result)
        if result.exit_code:
            break
    return PreparationResult.model_validate(
        {
            "operation": operation,
            "mode": mode,
            "profile": profile,
            "validated": validated,
            "stages": stages,
            "exit_code": stages[-1].exit_code,
        }
    )


def launch(
    operation: str, mode: str, profile: str = "dev", *, validated: bool = True
) -> int:
    """One pse-env/native entry establishes every stage's effective environment."""
    environment = selected_environment()
    if mode == "auto":
        mode = (
            subprocess.check_output(
                [sys.executable, "-m", "scripts.doctor", "--extension-kind"],
                cwd=ROOT,
                env=environment,
                text=True,
            ).strip()
            if interpreter(environment).is_file()
            else "absent"
        )
        if mode == "absent":
            mode = "dev"
    if mode not in {"dev", "native"}:
        raise ValueError("extension mode must be dev, native or auto")
    wrapper = [str(ROOT / "scripts/pse-env"), "--recipe-default-class", "compile"]
    if mode == "native":
        wrapper.append("--native=solver,klu,isolation,uno,petsc")
    command = [
        *wrapper,
        "--",
        sys.executable,
        "-m",
        "scripts.python_extension",
        operation,
        mode,
        profile,
        "--resolved",
        *([] if validated else ["--interactive"]),
    ]
    return subprocess.call(command, cwd=ROOT, env=environment)


def sync(mode: str, profile: str = "dev") -> int:
    return launch("sync", mode, profile)


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
    for original_command in commands:
        command = original_command
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


def main(argv: Sequence[str] | None = None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    if arguments[:1] == ["maturin"]:
        return maturin(arguments[1:])
    if len(arguments) == 2 and arguments[0] == "diagnostic-sync":
        return diagnostic_sync(arguments[1])
    resolved = "--resolved" in arguments
    validated = "--interactive" not in arguments
    arguments = [
        item for item in arguments if item not in {"--resolved", "--interactive"}
    ]
    if len(arguments) not in {2, 3} or arguments[0] not in {
        "sync",
        "environment",
        "prepare",
        "stubs",
        "stubs-check",
        "admit",
    }:
        raise SystemExit(
            "usage: python -m scripts.python_extension sync|environment|prepare|stubs|stubs-check|admit dev|native|auto [profile] [--interactive]"
        )
    operation, mode = arguments[:2]
    profile = arguments[2] if len(arguments) == 3 else "dev"
    if resolved:
        result = run_stages(
            operation, mode, profile, dict(os.environ), validated=validated
        )
        print(result.model_dump_json())
        return result.exit_code
    return launch(operation, mode, profile, validated=validated)


if __name__ == "__main__":
    raise SystemExit(main())
