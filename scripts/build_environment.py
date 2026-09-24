# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Shared local compiler environment, also usable by noninteractive commands."""

from __future__ import annotations

import argparse
import hashlib
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def configure(
    root: Path,
    original: dict[str, str],
    *,
    mode: str | None = None,
    cache: str = "auto",
    frontend: int | None = None,
    jobs: int | None = None,
) -> dict[str, str]:
    """Preserve explicit overrides; never disable workspace incremental compilation."""
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
            LIBCLANG_PATH=str(prefix / "lib"),
            LLVM_CONFIG_PATH=str(prefix / "bin/llvm-config"),
        )
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
    if cache == "off":
        env.update(RUSTC_WRAPPER="", RUSTC_WORKSPACE_WRAPPER="")
        if "PSE_NATIVE_COMPILER_CACHE" in env:
            env["PSE_NATIVE_COMPILER_CACHE"] = ""
    elif "RUSTC_WRAPPER" not in env and (cache == "on" or not env.get("CI")):
        wrapper = shutil.which("sccache", path=env.get("PATH"))
        if not wrapper and cache == "on":
            raise ValueError("cache requested but sccache is not installed")
        if wrapper:
            env["RUSTC_WRAPPER"] = wrapper
    if cache == "on" and not env.get("RUSTC_WRAPPER"):
        raise ValueError("cache requested but RUSTC_WRAPPER explicitly disables it")
    wrapper = env.get("RUSTC_WRAPPER", "")
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
    if mode:
        stable = tomllib.loads((root / "rust-toolchain.toml").read_text())["toolchain"][
            "channel"
        ]
        env["RUSTUP_TOOLCHAIN"] = settings["nightly"] if mode == "nightly" else stable
        if mode == "nightly":
            flags = effective_flags(root, env)
            if any(flag.startswith(("-Zthreads", "--jobs-frontend")) for flag in flags):
                raise ValueError(
                    "frontend flags already supplied; use --frontend instead"
                )
            flags.append(f"-Zthreads={frontend or settings['frontend_threads']}")
            env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
            env["CARGO_TARGET_DIR"] = str(root / "target" / settings["nightly"])
            env["CARGO_BUILD_JOBS"] = str(jobs or settings["jobs"])
        elif any(flag.startswith("-Z") for flag in effective_flags(root, env)):
            raise ValueError("stable route refuses caller-supplied unstable flags")
        else:
            env["CARGO_TARGET_DIR"] = str(root / "target")
            if jobs:
                env["CARGO_BUILD_JOBS"] = str(jobs)
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
    parser.add_argument("--mode", choices=("stable", "nightly"))
    parser.add_argument("--cache", choices=("auto", "on", "off"), default="auto")
    parser.add_argument("--frontend", type=int, choices=(1, 2, 4, 8))
    parser.add_argument("--jobs", type=int, choices=range(1, 257))
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    env = configure(
        ROOT,
        dict(os.environ),
        mode=args.mode,
        cache=args.cache,
        frontend=args.frontend,
        jobs=args.jobs,
    )
    if args.shell:
        for key, value in env.items():
            if os.environ.get(key) != value:
                print(f"export {key}={shlex.quote(value)}")
        return 0
    command = args.command
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        parser.error("provide a command after --, or --shell")
    print(
        f"build: toolchain={env.get('RUSTUP_TOOLCHAIN', 'repository pin')} wrapper={env.get('RUSTC_WRAPPER') or 'disabled'} target={env.get('CARGO_TARGET_DIR', 'target')}",
        file=sys.stderr,
    )
    return subprocess.call(command, env=env)


if __name__ == "__main__":
    raise SystemExit(main())
