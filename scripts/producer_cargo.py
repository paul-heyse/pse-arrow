# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Single-use observation of the Cargo child selected by strict Maturin installation.

The Rust capture owner validates the supported actual arguments, executes Cargo and
seals its raw realization before this adapter permits installation to continue.
Ordinary development never selects this adapter.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

CAPTURE = "PSE_PRODUCER_CARGO_CAPTURE"
PYTHON = "PSE_PRODUCER_CARGO_PYTHON"
PREPARATION_GUARD_FD = "PSE_PRODUCER_PREPARATION_GUARD_FD"


def preparation_child_arguments(
    environment: Mapping[str, str],
) -> dict[str, tuple[int, ...]]:
    """Retain the target-local owner's lock through actual writer descendants."""
    value = environment.get(PREPARATION_GUARD_FD)
    if value is None:
        return {}
    descriptor = int(value)
    if descriptor < 3 or not stat.S_ISREG(os.fstat(descriptor).st_mode):
        raise ValueError("invalid preparation lifetime descriptor")
    return {"pass_fds": (descriptor,)}


# Effective compiler/SDK selections, never arbitrary environment or credentials.
EFFECTIVE_ENVIRONMENT = (
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
    "PYO3_ENVIRONMENT_SIGNATURE",
    "PYTHON_SYS_EXECUTABLE",
    "RUSTC_WRAPPER",
    "PSE_SCCACHE_BINARY",
    "RUSTC_WORKSPACE_WRAPPER",
    "PSE_NATIVE_COMPILER_CACHE",
    "CARGO_BUILD_JOBS",
    "CARGO_TERM_COLOR",
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
    "CC",
    "CXX",
    "CMAKE_TOOLCHAIN_FILE",
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
)


def write_new(path: Path, value: object) -> None:
    descriptor = os.open(
        path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600
    )
    with os.fdopen(descriptor, "w") as output:
        json.dump(value, output, indent=2)
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())


def cargo_executable(environment: Mapping[str, str]) -> str:
    selected = shutil.which("cargo", path=environment.get("PATH", os.defpath))
    if selected is None:
        raise ValueError("strict installer requires the selected Cargo executable")
    explicit = environment.get("CARGO")
    if explicit is not None:
        if not explicit:
            raise ValueError(
                "strict installer does not support an empty explicit CARGO"
            )
        chosen = shutil.which(explicit, path=environment.get("PATH", os.defpath))
        if chosen is None or os.path.abspath(chosen) != os.path.abspath(selected):
            raise ValueError(
                "strict installer does not support CARGO differing from the selected PATH Cargo"
            )
    return os.path.abspath(selected)


def configure(
    root: Path, directory: Path, options: list[str], environment: Mapping[str, str]
) -> dict[str, str]:
    """Bind one installer to a private immutable command selection before launch."""
    cargo = cargo_executable(environment)
    path = directory / "python-installer-cargo.json"
    write_new(
        path,
        {
            "version": 1,
            "workspace_root": str(root.resolve()),
            "cargo": cargo,
            "capture_options": options,
        },
    )
    return {
        **environment,
        CAPTURE: str(path),
        PYTHON: sys.executable,
        "CARGO": str(root / "scripts/producer-cargo"),
    }


def read_configuration(path: Path) -> dict[str, object]:
    observed = path.lstat()
    parent = path.parent.stat()
    if (
        not stat.S_ISREG(observed.st_mode)
        or observed.st_uid != os.getuid()
        or stat.S_IMODE(observed.st_mode) != 0o600
        or parent.st_uid != os.getuid()
        or stat.S_IMODE(parent.st_mode) != 0o700
    ):
        raise ValueError(
            "strict Cargo observer requires its owned private configuration"
        )
    with path.open() as stream:
        value = json.load(stream)
    if (
        not isinstance(value, dict)
        or type(value.get("version")) is not int
        or value.get("version") != 1
    ):
        raise ValueError("unsupported strict Cargo observer configuration")
    if not isinstance(value.get("workspace_root"), str) or not isinstance(
        value.get("cargo"), str
    ):
        raise ValueError("strict Cargo observer lacks selected executables/context")
    options = value.get("capture_options")
    if not isinstance(options, list) or any(
        not isinstance(item, str) for item in options
    ):
        raise ValueError("strict Cargo observer lacks capture options")
    return value


def effective_environment(environment: Mapping[str, str]) -> dict[str, str]:
    return {
        name: environment[name] for name in EFFECTIVE_ENVIRONMENT if name in environment
    }


def require_sealed_realization(options: list[str]) -> None:
    receipt_path = Path(options[options.index("--output") + 1])
    evidence_path = Path(options[options.index("--build-evidence") + 1])
    with receipt_path.open() as stream:
        receipt = json.load(stream)
    if (
        not isinstance(receipt, dict)
        or receipt.get("persistent_reuse_eligible") is not True
        or receipt.get("reasons") != []
        or receipt.get("receipt_version") != 2
    ):
        raise ValueError(
            "actual installer realization lacks qualified producer capture"
        )
    closure = Path(str(evidence_path) + ".closure")
    with (closure / "index.json").open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    if (closure / "COMPLETE").read_text() != digest:
        raise ValueError(
            "actual installer realization lacks completed protected evidence"
        )


def observe(arguments: list[str], environment: Mapping[str, str]) -> int:
    selected = environment.get(CAPTURE)
    if not selected:
        raise ValueError("strict Cargo observer has no selected capture")
    path = Path(selected)
    configuration = read_configuration(path)
    root = Path(str(configuration["workspace_root"]))
    if Path.cwd().resolve() != root.resolve():
        raise ValueError(
            "actual installer Cargo child belongs to a different working directory"
        )
    cargo = str(configuration["cargo"])
    child_environment = {
        name: value
        for name, value in environment.items()
        if name not in {"CARGO", CAPTURE, PYTHON}
    }
    actual_cargo = shutil.which("cargo", path=child_environment.get("PATH", os.defpath))
    if actual_cargo is None or os.path.abspath(actual_cargo) != os.path.abspath(cargo):
        raise ValueError("actual installer PATH selects a different Cargo executable")
    if arguments and arguments[0] in {"--version", "-V", "version", "metadata"}:
        return subprocess.call(
            [cargo, *arguments],
            cwd=root,
            env=child_environment,
            **preparation_child_arguments(environment),
        )
    if not arguments or arguments[0] != "rustc":
        raise ValueError("unsupported actual installer Cargo command")
    # A second build cannot replace the selected realization or its evidence.
    write_new(
        path.with_suffix(".claim.json"), {"pid": os.getpid(), "arguments": arguments}
    )
    arguments_path = path.with_suffix(".arguments.json")
    write_new(arguments_path, arguments)
    effective = effective_environment(environment)
    observation: dict[str, object] = {
        "version": 1,
        "cargo": cargo,
        "cwd": str(Path.cwd()),
        "arguments": arguments,
        "effective_environment": effective,
    }
    if effective.get("PYO3_CONFIG_FILE"):
        source = Path(effective["PYO3_CONFIG_FILE"])
        if not source.is_absolute():
            source = root / source
        raw = source.read_bytes()
        protected = path.with_suffix(".pyo3-config")
        descriptor = os.open(
            protected, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600
        )
        with os.fdopen(descriptor, "wb") as output:
            output.write(raw)
        observation["pyo3_configuration"] = {
            "original": str(source),
            "canonical": str(source.resolve()),
            "protected": str(protected),
            "sha256": hashlib.sha256(raw).hexdigest(),
            "mode": source.stat().st_mode,
        }
    write_new(path.with_suffix(".observation.json"), observation)
    options = configuration["capture_options"]
    if not isinstance(options, list) or any(
        not isinstance(item, str) for item in options
    ):
        raise TypeError("invalid capture option list")
    command = [
        str(root / "target/debug/xtask"),
        "producer-identity",
        *options,
        "--observed-cargo-arguments",
        str(arguments_path),
    ]
    for name, value in effective.items():
        command.extend(("--environment", f"{name}={value}"))
    result = subprocess.call(
        command,
        cwd=root,
        env=child_environment,
        **preparation_child_arguments(environment),
    )
    if result == 0:
        try:
            require_sealed_realization(options)
            config = observation.get("pyo3_configuration")
            if isinstance(config, dict):
                source = Path(str(config["original"]))
                if (
                    str(source.resolve()) != config["canonical"]
                    or source.stat().st_mode != config["mode"]
                    or hashlib.sha256(source.read_bytes()).hexdigest()
                    != config["sha256"]
                ):
                    raise ValueError(
                        "actual PyO3 configuration changed during installer capture"
                    )
        except (OSError, ValueError, TypeError, KeyError) as error:
            write_new(
                path.with_suffix(".status.json"),
                {
                    "returncode": 1,
                    "capture_returncode": result,
                    "sealed": False,
                    "refusal": str(error),
                },
            )
            raise
    write_new(
        path.with_suffix(".status.json"), {"returncode": result, "sealed": result == 0}
    )
    return result


def main() -> int:
    try:
        return observe(sys.argv[1:], os.environ)
    except (OSError, ValueError, TypeError, KeyError) as error:
        print(
            f"strict Cargo observer refused: {type(error).__name__}: {error}",
            file=sys.stderr,
        )
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
