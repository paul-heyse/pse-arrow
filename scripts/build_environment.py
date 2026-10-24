# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Shared local compiler environment, also usable by noninteractive commands."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import diagnostic_trace

if TYPE_CHECKING:
    from collections.abc import Mapping

ROOT = Path(__file__).resolve().parents[1]
FOREGROUND_CACHE = "PSE_SCCACHE_FOREGROUND"


def selected_cache_binary(root: Path, environment: Mapping[str, str]) -> str | None:
    """Resolve the declared default or a deliberate real-executable override."""
    managed = (root / "scripts/sccache").resolve()
    wrapper = environment.get("RUSTC_WRAPPER")
    explicit = environment.get("PSE_SCCACHE_BINARY")
    if "PSE_SCCACHE_BINARY" in environment and not explicit:
        raise ValueError("PSE_SCCACHE_BINARY must name the real executable")
    if (
        wrapper
        and Path(wrapper).name == "sccache"
        and Path(wrapper).resolve() != managed
    ):
        chosen = shutil.which(wrapper, path=environment.get("PATH")) or wrapper
        if explicit and Path(explicit).resolve() != Path(chosen).resolve():
            raise ValueError(
                "conflicting RUSTC_WRAPPER and PSE_SCCACHE_BINARY selections"
            )
        return chosen
    if explicit:
        return explicit
    default = diagnostic_trace.binary_path("sccache", environment)
    if not default.exists():
        return None
    if not default.is_file() or not os.access(default, os.X_OK):
        raise ValueError(
            "managed sccache is not executable; run just diagnostic-tools sccache"
        )
    return str(default)


def validate_cache_binary(root: Path, selected: str) -> Path:
    """Validate the real Linux executable shared by admission and diagnostics."""
    binary = Path(selected).resolve(strict=True)
    if (
        binary == (root / "scripts/sccache").resolve()
        or not binary.is_file()
        or not os.access(binary, os.X_OK)
    ):
        raise ValueError("PSE_SCCACHE_BINARY must select the real executable")
    with binary.open("rb") as source:
        if source.read(4) != b"\x7fELF":
            raise ValueError("PSE_SCCACHE_BINARY must select the installed ELF binary")
    return binary


def cache_mode_problem(root: Path, environment: Mapping[str, str]) -> str | None:
    """Explain why the selected cache cannot keep compilation in the caller."""
    if "SCCACHE_LOG" in environment:
        return "SCCACHE_LOG disables supervised client compilation"
    if "SCCACHE_ERROR_LOG" in environment:
        return "SCCACHE_ERROR_LOG is unsupported for supervised client compilation"
    config_path = Path(
        environment.get("SCCACHE_CONF", str(root / ".config/sccache.toml"))
    )
    config_text = config_path.read_text()
    settings = (
        json.loads(config_text)
        if config_path.suffix == ".json"
        else tomllib.loads(config_text)
    )
    if not isinstance(settings, dict):
        return "sccache configuration must be a table"
    dist = settings.get("dist", {})
    if not isinstance(dist, dict):
        return "sccache distribution configuration must be a table"
    if dist.get("scheduler_url") is not None:
        return "sccache distribution disables supervised client compilation"
    return None


def language_server_environment(environment: Mapping[str, str]) -> dict[str, str]:
    """Project already selected compiler/native state through a filtering LSP bridge.

    No defaults are composed here. In particular credentials, license values and
    unrelated caller state never become LSP server configuration.
    """
    from scripts import (  # noqa: PLC0415 -- reciprocal owners; bootstrap configure remains stdlib-only
        host_admission,
        native_operation,
    )

    keys = {
        "RUSTC",
        "RUSTDOC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTDOCFLAGS",
        "CARGO_ENCODED_RUSTDOCFLAGS",
        "CARGO_BUILD_JOBS",
        "CARGO_BUILD_TARGET",
        "CARGO_INCREMENTAL",
        "PSE_CARGO_TARGET_DIR",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
        "PSE_NATIVE_COMPILER_CACHE",
        "PSE_SCCACHE_BINARY",
        "SCCACHE_DIR",
        "SCCACHE_CACHE_SIZE",
        "SCCACHE_CONF",
        "SCCACHE_SERVER_UDS",
        "SCCACHE_CLIENT_SIDE",
        "SCCACHE_DIRECT",
        "SCCACHE_NO_DAEMON",
        FOREGROUND_CACHE,
        "PSE_LLVM_PREFIX",
        "LLVM_CONFIG_PATH",
        "LIBCLANG_PATH",
        "LIBRARY_PATH",
        "LD_LIBRARY_PATH",
        "PKG_CONFIG_PATH",
        "PYO3_CONFIG_FILE",
        "PYO3_PYTHON",
        "PYO3_BUILD_EXTENSION_MODULE",
        "PYO3_ENVIRONMENT_SIGNATURE",
        "PYTHON_SYS_EXECUTABLE",
        "XDG_STATE_HOME",
        "XDG_CACHE_HOME",
        "XDG_DATA_HOME",
        "PSE_NATIVE_CACHE",
        "PSE_NATIVE_SETUP_PYTHON",
        "PSE_NATIVE_HANDOFF",
        "PSE_ROOT_ISOLATION_DIR",
        "PSE_ADMISSION_DEADLINE",
        "PSE_RESOURCE_CLASS",
        "PSE_MEMORY_MAX",
        host_admission.MARKER,
        native_operation.MARKER,
        native_operation.OFF_MARKER,
        *native_operation.OVERRIDABLE,
        *native_operation.AUTHORITATIVE,
        *(key for group in native_operation.CAPABILITY_PATHS.values() for key in group),
    }
    return {key: environment[key] for key in sorted(keys) if key in environment}


def configure(
    root: Path,
    original: dict[str, str],
    *,
    cache: str = "auto",
    frontend: int | None = None,
    jobs: int | None = None,
    taskdump: bool = False,
) -> dict[str, str]:
    """Preserve explicit overrides; never disable workspace incremental compilation.

    The toolchain is always rust-toolchain.toml's dated nightly, selected by rustup; this
    environment never names one. ``frontend`` is the explicit parallel-frontend experiment.
    """
    env = original.copy()
    # A selected system installation also works in long-running agent processes
    # whose inherited PATH/libclang variables predate the system migration.
    llvm_prefix = env.get("PSE_LLVM_PREFIX")
    system_llvm = Path("/opt/llvm-current")
    if llvm_prefix or (system_llvm / "bin/llvm-config").is_file():
        prefix = Path(llvm_prefix) if llvm_prefix else system_llvm
        if (
            not (prefix / "bin/clang").is_file()
            or not (prefix / "lib/libclang.so").is_file()
        ):
            raise ValueError("selected LLVM prefix lacks clang or libclang")
        env.update(
            PSE_LLVM_PREFIX=str(prefix),
            CLANG_PATH=str(prefix / "bin/clang"),
            LLVM_CONFIG_PATH=str(prefix / "bin/llvm-config"),
        )
        # An exact selected library file closes discovery more narrowly than the
        # installation directory. Nested setup must preserve that explicit input.
        env.setdefault("LIBCLANG_PATH", str(prefix / "lib"))
        binary = str(prefix / "bin")
        env["PATH"] = os.pathsep.join(
            [
                binary,
                *[
                    part
                    for part in env.get("PATH", os.defpath).split(os.pathsep)
                    if part != binary
                ],
            ]
        )
    settings = tomllib.loads((root / ".config/build.toml").read_text())
    cargo_settings = tomllib.loads((root / ".cargo/config.toml").read_text())
    if jobs is not None:
        env["CARGO_BUILD_JOBS"] = str(jobs)
    else:
        env.setdefault("CARGO_BUILD_JOBS", str(cargo_settings["build"]["jobs"]))
    if cache == "off":
        env.update(RUSTC_WRAPPER="", RUSTC_WORKSPACE_WRAPPER="")
        if "PSE_NATIVE_COMPILER_CACHE" in env:
            env["PSE_NATIVE_COMPILER_CACHE"] = ""
    elif "RUSTC_WRAPPER" not in env and (cache == "on" or not env.get("CI")):
        binary = selected_cache_binary(root, env)
        if not binary and cache == "on":
            raise ValueError("cache requested but sccache is not installed")
        if binary:
            env["RUSTC_WRAPPER"] = str(root / "scripts/sccache")
            env["PSE_SCCACHE_BINARY"] = binary
    if cache == "on" and not env.get("RUSTC_WRAPPER"):
        raise ValueError("cache requested but RUSTC_WRAPPER explicitly disables it")
    wrapper = env.get("RUSTC_WRAPPER", "")
    if wrapper and Path(wrapper).name == "sccache":
        # Keep actual compiler children in their caller's supervised lifetime.
        # Preprocessing must observe newly present headers: sccache's direct
        # include shortcut cannot establish the recorded absence premises.
        env.update(SCCACHE_CLIENT_SIDE="1", SCCACHE_DIRECT="false")
        env.setdefault("SCCACHE_CONF", str(root / ".config/sccache.toml"))
        if problem := cache_mode_problem(root, env):
            # Normal builds fall back; explicit experiments refuse a different
            # execution mode. Keep the caller's diagnostic settings intact.
            if cache == "on":
                raise ValueError(problem)
            env.update(
                RUSTC_WRAPPER="",
                RUSTC_WORKSPACE_WRAPPER="",
                PSE_NATIVE_COMPILER_CACHE="",
            )
            wrapper = ""
    if wrapper and Path(wrapper).name == "sccache" and "SCCACHE_DIR" not in env:
        base = Path(env.get("XDG_CACHE_HOME", str(Path.home() / ".cache")))
        directory = base / "pse-arrow/sccache"
        env["SCCACHE_DIR"] = str(directory)
        env.setdefault("SCCACHE_CACHE_SIZE", settings["cache_size"])
        # A dedicated directory and deterministic endpoint have one server owner.
        # Never reconfigure or stop the user's default/global server.
        if os.name == "posix":
            key = hashlib.sha256(str(directory.resolve()).encode()).hexdigest()[:16]
            # User UID and cache digest identify the server; socket access is user-owned.
            env.setdefault(
                "SCCACHE_SERVER_UDS",
                str(
                    Path(tempfile.gettempdir())
                    / f"pse-sccache-{os.getuid()}-{key}.sock"
                ),
            )
    if wrapper and Path(wrapper).name == "sccache":
        # The cache daemon has a distinct supervised light lifetime. Compiler
        # processes remain in the caller's allocation; the wrapper verifies the
        # admitted daemon generation and contains sccache's lazy-start fallback.
        managed = root / "scripts/sccache"
        real = selected_cache_binary(root, env)
        if not real:
            raise ValueError(
                "managed compiler cache requires its exact installed binary"
            )
        env["PSE_SCCACHE_BINARY"] = real
        foreground = env.get(FOREGROUND_CACHE)
        if foreground is not None and (
            not foreground
            or foreground != env.get("SCCACHE_SERVER_UDS")
            or env.get("SCCACHE_NO_DAEMON") != "1"
        ):
            raise ValueError(
                "foreground compiler cache requires its selected endpoint and NO_DAEMON=1"
            )
        # Explicit experiments own their foreground service in the operation's
        # allocation. Nested recipe setup must retain that same endpoint.
        selected = real if foreground is not None else str(managed)
        env["RUSTC_WRAPPER"] = selected
        if env.get("PSE_NATIVE_COMPILER_CACHE"):
            env["PSE_NATIVE_COMPILER_CACHE"] = selected
    # Each checkout, git worktrees included, builds into its own target directory. A
    # directory inherited from another checkout is not an override for this one: a
    # long-running agent or editor process started under that checkout's direnv carries
    # its absolute export into every worktree, which would then share one build and one
    # Cargo lock. The checkout's default <root>/target is Cargo's own default, so it is
    # not exported at all: sccache keys every rustc call on its CARGO_* environment, and
    # a per-checkout absolute path would make every Rust compilation miss the cache
    # shared by the other checkouts. Choose another directory with PSE_CARGO_TARGET_DIR.
    if env.get("PSE_CARGO_TARGET_DIR"):
        env["CARGO_TARGET_DIR"] = env["PSE_CARGO_TARGET_DIR"]
    elif not env.get("CARGO_TARGET_DIR"):
        env.pop("CARGO_TARGET_DIR", None)
    else:
        chosen = Path(env["CARGO_TARGET_DIR"])
        chosen = chosen if chosen.is_absolute() else root / chosen
        inside = chosen.resolve().is_relative_to(root.resolve())
        if not inside or chosen.resolve() == (root / "target").resolve():
            del env["CARGO_TARGET_DIR"]
    if frontend is not None:
        flags = effective_flags(root, env)
        if any(flag.startswith(("-Zthreads", "--jobs-frontend")) for flag in flags):
            raise ValueError("frontend flags already supplied; use --frontend instead")
        flags.append(f"-Zthreads={frontend}")
        env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
        # The experiment keeps its own target directory, so its flags never replace
        # this checkout's ordinary artifacts.
        env["CARGO_TARGET_DIR"] = str(root / "target" / f"frontend-{frontend}")
    if taskdump:
        flags = effective_flags(root, env)
        if "tokio_unstable" not in flags:
            flags += ["--cfg", "tokio_unstable"]
        env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    return env


def effective_flags(root: Path, env: dict[str, str]) -> list[str]:
    """Honor Cargo precedence and preserve the repository's target linker flags."""
    if "CARGO_ENCODED_RUSTFLAGS" in env:
        return (
            env["CARGO_ENCODED_RUSTFLAGS"].split("\x1f")
            if env["CARGO_ENCODED_RUSTFLAGS"]
            else []
        )
    if "RUSTFLAGS" in env:
        return env["RUSTFLAGS"].split()
    target = env.get("CARGO_BUILD_TARGET")
    if not target:
        identity = subprocess.check_output(["rustc", "-vV"], env=env, text=True)
        target = next(
            line.removeprefix("host: ")
            for line in identity.splitlines()
            if line.startswith("host: ")
        )
    config = tomllib.loads((root / ".cargo/config.toml").read_text())
    flags = list(config.get("target", {}).get(target, {}).get("rustflags", []))
    flags.extend(
        env.get(
            f"CARGO_TARGET_{target.upper().replace('-', '_')}_RUSTFLAGS", ""
        ).split()
    )
    return flags


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shell", action="store_true")
    parser.add_argument("--cache", choices=("auto", "on", "off"), default="auto")
    parser.add_argument("--frontend", type=int, choices=(1, 2, 4, 8))
    parser.add_argument("--jobs", type=int, choices=range(1, 257))
    parser.add_argument(
        "--taskdump",
        action="store_true",
        help="explicit diagnostic-taskdump profile flags",
    )
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    env = configure(
        ROOT,
        dict(os.environ),
        cache=args.cache,
        frontend=args.frontend,
        jobs=args.jobs,
        taskdump=args.taskdump,
    )
    if args.shell:
        for key, value in env.items():
            if os.environ.get(key) != value:
                print(f"export {key}={shlex.quote(value)}")
        for key in os.environ.keys() - env.keys():
            print(f"unset {key}")
        return 0
    command = args.command
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        parser.error("provide a command after --, or --shell")
    if args.taskdump and not any(
        command[index : index + 2]
        in (
            ["--profile", "diagnostic-taskdump"],
            ["--cargo-profile", "diagnostic-taskdump"],
        )
        for index in range(len(command))
    ):
        parser.error(
            "--taskdump requires --profile diagnostic-taskdump to isolate diagnostic artifacts"
        )
    print(
        f"build: toolchain={env.get('RUSTUP_TOOLCHAIN', 'rust-toolchain.toml')} wrapper={env.get('RUSTC_WRAPPER') or 'disabled'} target={env.get('CARGO_TARGET_DIR', 'target')}",
        file=sys.stderr,
    )
    return subprocess.call(command, env=env)


if __name__ == "__main__":
    raise SystemExit(main())
