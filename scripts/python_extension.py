# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Inspect the actual imported Python extension before correctness execution."""

from __future__ import annotations

import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING, Annotated, Literal

from pydantic import BaseModel, ConfigDict, Field, TypeAdapter, model_validator

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence

from scripts import arrow_validation, producing_currency

ROOT = Path(__file__).resolve().parents[1]
PRODUCING_WITNESS = producing_currency.Witness()

# Positive compiler/installer selectors supplement actual dep-info and build
# directives, including newly introduced values in these supported namespaces.
PRODUCING_ENVIRONMENT_PREFIXES = (
    "PYO3_",
    "BINDGEN_",
    "PKG_CONFIG_",
    "CMAKE_",
)
PRODUCING_ENVIRONMENT = (
    "PATH",
    "HOME",
    "TMPDIR",
    "TEMP",
    "TMP",
    "LD_LIBRARY_PATH",
    "LIBRARY_PATH",
    "CPATH",
    "C_INCLUDE_PATH",
    "CPLUS_INCLUDE_PATH",
    "PYTHONPATH",
    "PYTHONHOME",
    "PYTHONNOUSERSITE",
    "PYTHONUSERBASE",
    "PYTHONSAFEPATH",
    "PYTHONPLATLIBDIR",
    "PYTHON_SYS_EXECUTABLE",
    "_PYTHON_SYSCONFIGDATA_NAME",
    "_PYTHON_SYSCONFIGDATA_PATH",
    "_PYTHON_HOST_PLATFORM",
    "_PYTHON_PROJECT_BASE",
    "CARGO",
    "CARGO_HOME",
    "CARGO_TARGET_DIR",
    "CARGO_INCREMENTAL",
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTDOC",
    "RUSTDOCFLAGS",
    "CARGO_ENCODED_RUSTDOCFLAGS",
    "RUSTC_BOOTSTRAP",
    "RUSTUP_TOOLCHAIN",
    "RUSTUP_HOME",
    "RUSTUP_OVERRIDE_HOST_TRIPLE",
    "CARGO_BUILD_WARNINGS",
    "CARGO_BUILD_JOBS",
    "CARGO_BUILD_RUSTC",
    "CARGO_BUILD_RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
    "CARGO_BUILD_RUSTDOC",
    "CARGO_BUILD_TARGET",
    "CARGO_BUILD_TARGET_DIR",
    "CARGO_BUILD_BUILD_DIR",
    "CARGO_BUILD_RUSTFLAGS",
    "CARGO_BUILD_RUSTDOCFLAGS",
    "CARGO_BUILD_INCREMENTAL",
    "CARGO_BUILD_DEP_INFO_BASEDIR",
    "CARGO_RESOLVER_LOCKFILE_PATH",
    "CC",
    "CXX",
    "FC",
    "AR",
    "RANLIB",
    "CFLAGS",
    "CXXFLAGS",
    "CPPFLAGS",
    "LDFLAGS",
    "SOURCE_DATE_EPOCH",
    "TZ",
    "LIBCLANG_PATH",
    "CLANG_PATH",
    "LLVM_CONFIG_PATH",
    "COMPILER_PATH",
    "GCC_EXEC_PREFIX",
    "IPOPT_DIR",
    "SCIPOPTDIR",
    "UNO_DIR",
    "PETSC_DIR",
    "SUITESPARSE_INCLUDE_DIR",
    "SUITESPARSE_LIBRARY_DIR",
    "PSE_KLU_PROVIDER_RECEIPT",
    "PSE_ROOT_ISOLATION_DIR",
    "PSE_SCCACHE_BINARY",
    "PSE_NATIVE_COMPILER_CACHE",
    "PSE_LLVM_PREFIX",
    "SCCACHE_CONF",
)
# The fixed pure evidence target has no solver/provider or optional bindgen/CMake
# composition. These are supplemental inputs only for extension/native preparation;
# actual Cargo-declared reads below remain authoritative for every target.
# LD_LIBRARY_PATH remains a tracked compiler selector; the evidence owner instead
# prevents its consumption by removing it from both compiler and executable envs.
EVIDENCE_RUNTIME_ENVIRONMENT = frozenset(
    {
        "IPOPT_DIR",
        "SCIPOPTDIR",
        "UNO_DIR",
        "PETSC_DIR",
        "SUITESPARSE_INCLUDE_DIR",
        "SUITESPARSE_LIBRARY_DIR",
        "PSE_KLU_PROVIDER_RECEIPT",
        "PSE_ROOT_ISOLATION_DIR",
        "PSE_NATIVE_COMPILER_CACHE",
    }
)
PRODUCING_CARGO_SELECTOR = re.compile(
    r"CARGO_TARGET_[A-Z0-9_]+_(?:LINKER|RUSTFLAGS|RUSTDOCFLAGS)"
    r"|CARGO_PROFILE_[A-Z0-9_]+_(?:BUILD_OVERRIDE_)?(?:CODEGEN_UNITS|DEBUG|DEBUG_ASSERTIONS|INCREMENTAL|LTO|OVERFLOW_CHECKS|OPT_LEVEL|PANIC|RPATH|SPLIT_DEBUGINFO|STRIP)"
)


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
Profile = Annotated[str, Field(pattern=r"^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$")]
PROFILE = TypeAdapter(Profile)


class ExtensionInputChanges(ExtensionContract):
    """Bounded input KEY names only; no recorded values or value digests."""

    sections: Annotated[tuple[str, ...], Field(max_length=16)]
    source_keys: Annotated[tuple[str, ...], Field(max_length=32)] = ()
    configuration_keys: Annotated[tuple[str, ...], Field(max_length=32)] = ()
    environment_keys: Annotated[tuple[str, ...], Field(max_length=32)] = ()
    complete: bool


def changed_inputs(
    previous: dict[str, object], current: dict[str, object]
) -> ExtensionInputChanges:
    """Explain an unequal owner input snapshot without exposing opaque values."""
    sections = sorted(
        name
        for name in previous.keys() | current.keys()
        if name not in previous
        or name not in current
        or previous[name] != current[name]
    )
    complete = len(sections) <= 16
    known_sections = {
        "schema",
        "stage",
        "mode",
        "profile",
        "validated",
        "files",
        "environment",
        "configuration",
        "interpreter",
        "tools",
        "generations",
        "compiler",
    }
    section_names = tuple(name for name in sections if name in known_sections)[:16]
    complete = complete and len(section_names) == len(sections)

    def keys(section: str, *, paths: bool = False) -> tuple[str, ...]:
        nonlocal complete
        before, after = previous.get(section, {}), current.get(section, {})
        if not isinstance(before, dict) or not isinstance(after, dict):
            complete = False
            return ()
        names = []
        for key in sorted(before.keys() | after.keys()):
            if key in before and key in after and before[key] == after[key]:
                continue
            if not isinstance(key, str):
                complete = False
                continue
            name = key
            if paths:
                path = Path(name)
                if path.is_absolute():
                    try:
                        path = path.relative_to(ROOT)
                    except ValueError:
                        complete = False
                        continue
                name = path.as_posix()
                if (
                    ".." in path.parts
                    or name.startswith("resolved:")
                    or not re.fullmatch(r"[A-Za-z0-9_./:-]{1,256}", name)
                ):
                    complete = False
                    continue
            elif not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]{0,127}", name):
                complete = False
                continue
            names.append(name)
        if len(names) > 32:
            complete = False
        return tuple(names[:32])

    source_keys = keys("files", paths=True)
    configuration_keys = keys("configuration", paths=True)
    environment_keys = keys("environment")
    return ExtensionInputChanges(
        sections=section_names,
        source_keys=source_keys,
        configuration_keys=configuration_keys,
        environment_keys=environment_keys,
        complete=complete,
    )


class ExtensionCurrency(ExtensionContract):
    """Redacted admission-time correspondence, separate from import compatibility."""

    state: Literal["current", "stale", "unknown", "unsupported"]
    checkout: str
    interpreter: str
    mode: Mode
    profile: str | None
    validated: bool
    source_basis: str | None = None
    input_changes: ExtensionInputChanges | None = None
    artifact: ExtensionArtifact | None = None
    artifact_stamp: tuple[int, int, int, int, int] | None = None
    receipt_recorded: bool = False
    cause: Literal[
        "matching-inputs-and-artifact",
        "missing-preparation-evidence",
        "unsupported-preparation-evidence",
        "unsupported-inputs-or-providers",
        "inputs-unavailable",
        "inputs-changed",
        "import-unavailable",
        "artifact-changed",
        "not-selected",
        "check-unavailable",
    ]
    remedy: str

    @model_validator(mode="after")
    def established_current(self) -> ExtensionCurrency:
        if self.state == "current" and (
            self.source_basis is None
            or self.artifact is None
            or self.profile is None
            or not self.receipt_recorded
            or self.cause != "matching-inputs-and-artifact"
        ):
            raise ValueError("current currency requires source and artifact evidence")
        return self


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
        "SHLVL",
    }
)


def preparation_environment_identity(environment: Mapping[str, str]) -> dict[str, str]:
    """Bind build inputs without tool-call and shell restoration bookkeeping.

    Direnv's evaluated exports and local configuration remain inputs. Its diff and
    watch state describe how to restore the invoking shell, while the Codex
    identifiers route the invoking session, and SHLVL records shell nesting.
    Unknown variables remain bound.
    The invocation-selected scientific capsule is a runtime input, separately
    bound by scientific execution provenance rather than the extension build.
    Rust fixture forwarding retains its independent full environment identity.
    """
    from scripts import native_tests, scientific_inputs  # noqa: PLC0415

    return native_tests.rust_environment_identity(
        {
            name: value
            for name, value in environment.items()
            if name not in PREPARATION_SESSION_ENVIRONMENT
            and name != scientific_inputs.ROLE
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
    files = validation.sources(ROOT, scope=stage)
    if any(
        (ROOT / name).is_symlink() and (ROOT / name).is_dir()
        for name in files
        if not name.startswith("resolved:")
    ):
        return None
    selected_python = interpreter(environment)
    # Observation must never let a rustup proxy install a missing toolchain.
    # Keep this inspection control out of the actual build-input identity.
    probe_environment = {**environment, "RUSTUP_AUTO_INSTALL": "0"}
    configuration = json.loads(
        subprocess.check_output(
            [
                str(selected_python),
                "-c",
                (
                    "import importlib,json,pathlib,sys,sysconfig; "
                    "data=importlib.import_module(sysconfig._get_sysconfigdata_name()); "
                    "paths=[sysconfig.__file__,data.__file__,sysconfig.get_config_h_filename(),sysconfig.get_makefile_filename(),getattr(sysconfig,'__cached__',None),getattr(data,'__cached__',None)]; "
                    "libdir=sysconfig.get_config_var('LIBDIR'); library=sysconfig.get_config_var('LDLIBRARY'); "
                    "paths += [str(pathlib.Path(libdir)/library)] if libdir and library else []; "
                    "print(json.dumps({'executable':sys.executable,'prefix':sys.prefix,'version':sys.version,'config':sysconfig.get_config_vars(),'files':[p for p in paths if p]}))"
                ),
            ],
            cwd=ROOT,
            env=probe_environment,
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
            ["rustc", "--print", "sysroot"], cwd=ROOT, env=probe_environment, text=True
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
            ["rustc", "-Vv"], cwd=ROOT, env=probe_environment, text=True
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


def artifact_stamp(artifact: ExtensionArtifact) -> tuple[int, int, int, int, int]:
    """Observe the selected ordinary binary's import-window mutation witness."""
    path = Path(artifact.binary)
    if path.is_symlink() or not path.is_file():
        raise ValueError("prepared native artifact is not an ordinary file")
    value = path.stat()
    return (
        value.st_dev,
        value.st_ino,
        value.st_size,
        value.st_mtime_ns,
        value.st_ctime_ns,
    )


def currency_check(
    environment: Mapping[str, str],
    *,
    mode: Mode,
    profile: str = "dev",
    validated: bool = True,
) -> ExtensionCurrency:
    """Inspect existing preparation and actual import; never prepare or repair.

    Native providers must already have been admitted by their original operation
    owner. A source-only caller cannot establish that prerequisite by importing.
    No result is cached: callers may carry it only within their admitted operation.
    """
    remedy = (
        "Explicitly select python_extension_" + mode + "_stage with stage=prepare, "
        f"profile={profile} and validated={str(validated).lower()} in the selected "
        "checkout/environment; then restart or reattach and check again. "
        "Preparation does not update a running endpoint or replay effects."
    )
    base: dict[str, object] = {
        "checkout": str(ROOT.resolve()),
        "interpreter": str(interpreter(environment).absolute()),
        "mode": mode,
        "profile": profile,
        "validated": validated,
        "remedy": remedy,
    }

    def observed(state: str, cause: str, **facts: object) -> ExtensionCurrency:
        return ExtensionCurrency.model_validate(
            {**base, "state": state, "cause": cause, **facts}
        )

    previous = read_receipt(stage_receipt("extension", environment))
    if not previous:
        return observed("unknown", "missing-preparation-evidence")
    inputs = previous.get("inputs")
    try:
        recorded = ExtensionArtifact.model_validate(previous.get("artifact"))
    except ValueError:
        return observed("unknown", "missing-preparation-evidence")
    if not isinstance(inputs, dict) or inputs.get("schema") not in {
        "ordinary-editable-inputs-v1",
        "producing-cargo-inputs-v1",
    }:
        return observed(
            "unsupported", "unsupported-preparation-evidence", receipt_recorded=True
        )
    try:
        if inputs.get("schema") == "producing-cargo-inputs-v1":
            producing = previous.get("producing")
            if (
                not isinstance(producing, dict)
                or not isinstance(producing.get("selection"), dict)
                or not isinstance(producing.get("association"), dict)
            ):
                return observed(
                    "unsupported",
                    "unsupported-preparation-evidence",
                    receipt_recorded=True,
                )
            current = producing_inputs(
                producing["selection"],
                environment,
                mode=mode,
                profile=profile,
                validated=validated,
            )
        else:
            current = preparation_inputs(
                "python-extension", mode, profile, environment, validated=validated
            )
    except (OSError, ValueError, TypeError, subprocess.SubprocessError):
        return observed("unknown", "inputs-unavailable", receipt_recorded=True)
    if current is None:
        return observed(
            "unsupported", "unsupported-inputs-or-providers", receipt_recorded=True
        )
    basis = hashlib.sha256(
        json.dumps(current, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()
    facts: dict[str, object] = {"source_basis": basis, "receipt_recorded": True}
    if inputs != current:
        return observed(
            "stale",
            "inputs-changed",
            input_changes=changed_inputs(inputs, current),
            **facts,
        )
    try:
        before = artifact_stamp(recorded)
        if inputs.get("schema") == "producing-cargo-inputs-v1":
            identity = PRODUCING_WITNESS.file(Path(recorded.binary))
            if (
                Path(recorded.binary).parent != (ROOT / "python/pse").resolve()
                or identity["sha256"] != recorded.sha256
                or (validated and "force-validate" not in recorded.features)
                or (mode == "native" and "native-solvers" not in recorded.features)
            ):
                return observed("stale", "artifact-changed", **facts)
            artifact = recorded
        else:
            artifact = ExtensionArtifact.model_validate(
                capture(environment, validated=validated, native=mode == "native")
            )
        after = artifact_stamp(artifact)
    except (OSError, ValueError, TypeError, subprocess.SubprocessError):
        return observed("unknown", "import-unavailable", **facts)
    facts["artifact"] = artifact
    facts["artifact_stamp"] = after
    if artifact != recorded or before != after:
        return observed("stale", "artifact-changed", **facts)
    return observed("current", "matching-inputs-and-artifact", **facts)


def currency_check_installed(
    environment: Mapping[str, str], *, mode: Mode, validated: bool = True
) -> ExtensionCurrency:
    """Use only the installed preparation owner's explicit supported profile.

    Execution requirements select mode and validation, not an invented build
    profile. A missing or malformed recorded selection cannot establish currency.
    """
    previous = read_receipt(stage_receipt("extension", environment))
    inputs = previous.get("inputs")
    try:
        profile = PROFILE.validate_python(
            inputs.get("profile") if isinstance(inputs, dict) else None,
            strict=True,
        )
    except ValueError:
        return ExtensionCurrency(
            state="unknown",
            checkout=str(ROOT.resolve()),
            interpreter=str(interpreter(environment).absolute()),
            mode=mode,
            profile=None,
            validated=validated,
            cause="missing-preparation-evidence",
            remedy="Inspect the original Python preparation owner and explicitly select its prepare stage with a supported profile, mode and validation; then restart or reattach and check again. No profile was guessed or preparation attempted.",
        )
    return currency_check(environment, mode=mode, profile=profile, validated=validated)


def publish_receipt(path: Path, value: dict[str, object]) -> None:
    from scripts import validation  # noqa: PLC0415

    path.parent.mkdir(parents=True, exist_ok=True)
    validation.write_json(path, value)


def prepare_extractor(
    environment: Mapping[str, str],
) -> tuple[int, dict[str, object] | None]:
    """Explicit preparation of the finite observer, never a read-time action."""
    from scripts import producer_cargo  # noqa: PLC0415

    witness = producing_currency.Witness()
    sources = {
        str(ROOT / name): witness.file(ROOT / name)
        for name in producer_cargo.OBSERVER_SOURCE_INPUTS
    }
    command = [
        str(interpreter(environment)),
        "-m",
        "scripts.arrow_validation",
        "build",
        "-p",
        "xtask",
        "--bin",
        "xtask",
        "--no-default-features",
        "--locked",
    ]
    code = subprocess.call(command, cwd=ROOT, env=environment)
    if code:
        return code, None
    if any(value["sha256"] is None for value in sources.values()) or sources != {
        name: witness.file(Path(name)) for name in sources
    }:
        raise ValueError(
            "Producing observer source changed during explicit preparation"
        )
    extractor = ROOT / "target/debug/xtask"
    if not extractor.is_file():
        raise ValueError(
            "Explicit producing observer build did not supply its fixed executable"
        )
    return 0, {
        "source_basis": producing_currency.basis(sources),
        "artifact_stamp": list(producing_currency.stamp(extractor)),
        "command": command,
    }


def observed_installation(
    command: Sequence[str], environment: Mapping[str, str]
) -> tuple[int, dict[str, object] | None]:
    """Capture actual ordinary Cargo input selection; unsupported stays conservative."""
    from scripts import producer_cargo  # noqa: PLC0415

    extractor = ROOT / "target/debug/xtask"
    config = (
        tomllib.loads((ROOT / "pyproject.toml").read_text())
        .get("tool", {})
        .get("maturin", {})
    )
    if (
        config.get("strip")
        or config.get("pgo-command")
        or any(name.startswith(("PYO3_CROSS", "MATURIN_")) for name in environment)
    ):
        return subprocess.call(command, cwd=ROOT, env=environment), None
    code, observer = prepare_extractor(environment)
    if code:
        return code, None
    if observer is None or not extractor.is_file():
        return subprocess.call(command, cwd=ROOT, env=environment), None
    with tempfile.TemporaryDirectory(prefix="pse-ordinary-installation-") as temporary:
        selected = producer_cargo.configure_currency(
            ROOT, Path(temporary), environment, observer=observer
        )
        code = subprocess.call(command, cwd=ROOT, env=selected)
        if code:
            return code, None
        path = Path(selected[producer_cargo.CURRENCY_CAPTURE])
        status = read_receipt(path.with_suffix(".status.json"))
        if status.get("supported") is not True:
            print(
                "producing-currency: retained conservative preparation checks ("
                + str(status.get("cause", "observation-unavailable"))
                + ")",
                file=sys.stderr,
            )
            return 0, None
        inputs = read_receipt(path.with_suffix(".inputs.json"))
        invocation = read_receipt(path.with_suffix(".selection.json"))
        if not inputs or not invocation:
            return 0, None
        return 0, {"inputs": inputs, "invocation": invocation}


def producing_selection(
    observed: dict[str, object], context: dict[str, object]
) -> dict[str, object]:
    """Bind preparation tools/configuration to the actual producing selection."""
    from scripts import validation_scope  # noqa: PLC0415

    raw = observed.get("inputs")
    if not isinstance(raw, dict):
        raise TypeError("missing producing input observation")
    selection = dict(raw)
    files = set(producing_currency.string_list(selection.get("files")))
    files.update(
        str(ROOT / name) for name in validation_scope.PYTHON_EXTENSION_SUPPORT_INPUTS
    )
    for section in ("tools", "configuration"):
        values = context.get(section)
        if not isinstance(values, dict):
            raise TypeError("missing producing tool/configuration identity")
        for value in values.values():
            if isinstance(value, dict) and isinstance(value.get("path"), str):
                files.add(value["path"])
    selected_interpreter = context.get("interpreter")
    if not isinstance(selected_interpreter, dict):
        raise TypeError("missing producing interpreter identity")
    for key in ("binary", "venv"):
        value = selected_interpreter.get(key)
        if not isinstance(value, dict) or not isinstance(value.get("path"), str):
            raise TypeError("missing producing interpreter file")
        files.add(value["path"])
    configuration = selected_interpreter.get("configuration")
    if not isinstance(configuration, dict):
        raise TypeError("missing producing interpreter configuration")
    files.update(producing_currency.string_list(configuration.get("files")))
    selection["files"] = sorted(files)
    selection["environment"] = sorted(
        set(producing_currency.string_list(selection.get("environment")))
        | set(PRODUCING_ENVIRONMENT)
    )
    selection["invocation"] = observed.get("invocation")
    selection["generations"] = context.get("generations")
    if isinstance(selection["generations"], dict):
        for generation in selection["generations"].values():
            if isinstance(generation, dict) and isinstance(
                generation.get("receipt"), dict
            ):
                path = generation["receipt"].get("path")
                if isinstance(path, str):
                    files.add(path)
    selection["files"] = sorted(files)
    return producing_currency.retained_selection(selection)


def producing_inputs(
    selection: dict[str, object],
    environment: Mapping[str, str],
    *,
    mode: str,
    profile: str,
    validated: bool,
    stage: str = "python-extension",
) -> dict[str, object] | None:
    """Read only changed relevant files; tools and interpreter are witnessed inputs."""
    from scripts import native_operation  # noqa: PLC0415

    if any(name.startswith(("PYO3_CROSS", "MATURIN_")) for name in environment):
        return None
    generations = selection.get("generations")
    if not isinstance(generations, dict):
        return None
    if mode == "native":
        for name, expected in generations.items():
            if not isinstance(expected, dict):
                return None
            prefix = expected.get("prefix")
            selected = environment.get(name)
            if not isinstance(prefix, str) or not selected:
                return None
            path = Path(selected).resolve()
            if name == "SUITESPARSE_LIBRARY_DIR":
                path = path.parent
            if str(path) != prefix:
                return None
            admitted = native_operation.remembered("prefix:" + prefix)
            if admitted != expected.get("admission"):
                return None
    supplemental = set(PRODUCING_ENVIRONMENT)
    prefixes = PRODUCING_ENVIRONMENT_PREFIXES
    if stage == "evidence-executable":
        supplemental.difference_update(EVIDENCE_RUNTIME_ENVIRONMENT)
        prefixes = tuple(
            prefix for prefix in prefixes if prefix not in {"BINDGEN_", "CMAKE_"}
        )
    names = (
        set(producing_currency.string_list(selection.get("environment"))) | supplemental
    )
    names.update(
        name
        for name in environment
        if name.startswith(prefixes)
        or PRODUCING_CARGO_SELECTOR.fullmatch(name) is not None
    )
    observed = PRODUCING_WITNESS.inputs(
        {**selection, "environment": sorted(names)}, environment
    )
    return {
        "schema": "producing-cargo-inputs-v1",
        "stage": stage,
        "mode": mode,
        "profile": profile,
        "validated": validated,
        **observed,
        "configuration": {"producing-selection": producing_currency.basis(selection)},
    }


def installed_association(
    selection: dict[str, object],
    installed: dict[str, object],
    environment: Mapping[str, str],
) -> dict[str, object] | None:
    """Associate the imported installation with its actual Cargo root output."""
    records = selection.get("root_artifacts")
    if not isinstance(records, list):
        return None
    names = set()
    for record in records:
        if not isinstance(record, dict) or not isinstance(
            record.get("filenames"), list
        ):
            return None
        for name in record["filenames"]:
            if isinstance(name, str) and Path(name).suffix in {".so", ".pyd", ".dylib"}:
                names.add(name)
    if len(names) != 1:
        return None
    built = PRODUCING_WITNESS.file(Path(names.pop()))
    from scripts import installed_artifact  # noqa: PLC0415

    invocation = selection.get("invocation")
    recorded = invocation.get("artifacts") if isinstance(invocation, dict) else None
    if not isinstance(recorded, dict) or recorded.get(str(built["path"])) != built:
        return None
    messages = selection.get("build_scripts")
    if not isinstance(messages, list) or any(
        not isinstance(message, dict) for message in messages
    ):
        return None
    try:
        before = producing_currency.stamp(Path(str(installed["binary"])))
        transformation = installed_artifact.associate(
            Path(str(built["path"])),
            Path(str(installed["binary"])),
            str(built["sha256"]),
            str(installed["sha256"]),
            messages,
            environment=environment,
        )
        if before != producing_currency.stamp(Path(str(installed["binary"]))):
            return None
        tool = transformation.get("tool")
        if isinstance(tool, str):
            selection["files"] = sorted(
                set(producing_currency.string_list(selection.get("files"))) | {tool}
            )
    except (OSError, ValueError, TypeError, subprocess.SubprocessError):
        return None
    return {"built": built, "installed": installed, "transformation": transformation}


def extension_prepare(
    mode: str, profile: str, environment: Mapping[str, str], *, validated: bool = True
) -> StageResult:
    """Install only when a completed supported preparation no longer applies."""
    command = develop_command(
        ROOT, mode, profile, environment=environment, validated=validated
    )
    # Refuse unsupported correctness selections before any currency shortcut.
    maturin_cargo_command(command, ROOT)
    path = stage_receipt("extension", environment)
    previous = read_receipt(path)
    previous_inputs = previous.get("inputs")
    if (
        isinstance(previous_inputs, dict)
        and previous_inputs.get("schema") == "producing-cargo-inputs-v1"
    ):
        current = currency_check(
            environment,
            mode="native" if mode == "native" else "dev",
            profile=profile,
            validated=validated,
        )
        if current.state == "current":
            return StageResult(
                stage="prepare",
                status="reused",
                receipt=str(path),
                artifact=current.artifact,
            )
    inputs = preparation_inputs(
        "python-extension", mode, profile, environment, validated=validated
    )
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
    result, producing = (
        observed_installation(command, environment)
        if inputs is not None
        else (subprocess.call(command, cwd=ROOT, env=environment), None)
    )
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
        receipt: dict[str, object] = {"inputs": inputs, "artifact": installed}
        if producing is not None:
            selection = producing_selection(producing, inputs)
            association = installed_association(selection, installed, environment)
            if association is not None:
                selected_inputs = producing_inputs(
                    selection,
                    environment,
                    mode=mode,
                    profile=profile,
                    validated=validated,
                )
                if selected_inputs is not None:
                    receipt.update(
                        inputs=selected_inputs,
                        producing={"selection": selection, "association": association},
                    )
            else:
                print(
                    "producing-currency: retained conservative preparation checks (installed-artifact-association-unavailable)",
                    file=sys.stderr,
                )
        publish_receipt(path, receipt)
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
