# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Persistent, identity-keyed preparation of the existing pinned native inputs."""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import IO, TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

ROOT = Path(__file__).resolve().parents[1]
KLU_OPTIONS = (
    "CMAKE_INSTALL_LIBDIR=lib",
    "CMAKE_BUILD_TYPE=Release",
    "CMAKE_POSITION_INDEPENDENT_CODE=ON",
    "BUILD_SHARED_LIBS=OFF",
    "BUILD_STATIC_LIBS=ON",
    "SUITESPARSE_ENABLE_PROJECTS=suitesparse_config;amd;btf;colamd;klu",
    "KLU_USE_CHOLMOD=OFF",
    "SUITESPARSE_USE_CUDA=OFF",
    "SUITESPARSE_CONFIG_USE_OPENMP=OFF",
    "SUITESPARSE_DEMOS=OFF",
)
KLU_FILES = (
    "include/suitesparse/klu.h",
    "lib/libklu.a",
    "lib/libamd.a",
    "lib/libbtf.a",
    "lib/libcolamd.a",
    "lib/libsuitesparseconfig.a",
)
SOLVER_FILES = (
    "include/coin-or/IpIpoptApplication.hpp",
    "lib/libipopt.so",
    "lib/libcoinmumps.so.3",
)


def cache_root(env: dict[str, str]) -> Path:
    base = Path(env.get("XDG_CACHE_HOME", str(Path.home() / ".cache")))
    return (
        Path(env.get("PSE_NATIVE_CACHE", str(base / "pse-arrow/native")))
        .expanduser()
        .resolve()
    )


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def valid(prefix: Path, identity: dict, required: tuple[str, ...]) -> bool:
    try:
        receipt = json.loads((prefix / ".complete.json").read_text())
        return (
            receipt["identity"] == identity
            and all((prefix / name).is_file() for name in required)
            and bool(receipt["files"])
            and all(
                digest(prefix / name) == value
                for name, value in receipt["files"].items()
            )
        )
    except (OSError, ValueError, KeyError):
        return False


def prepare(
    base: Path,
    kind: str,
    identity: dict,
    required: tuple[str, ...],
    builder: Callable[[Path, Path], None],
) -> Path:
    """Serialize per identity; expose a new installation only after full verification."""
    key = hashlib.sha256(json.dumps(identity, sort_keys=True).encode()).hexdigest()
    parent = base / kind
    parent.mkdir(parents=True, exist_ok=True)
    prefix = parent / key
    with (parent / f"{key}.lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if valid(prefix, identity, required):
            return prefix
        with tempfile.TemporaryDirectory(prefix=f".{key}-", dir=parent) as scratch:
            stage = Path(scratch) / "install"
            stage.mkdir()
            builder(stage, Path(scratch) / "build")
            if not all((stage / name).is_file() for name in required):
                raise ValueError(f"incomplete {kind} installation")
            files = {
                str(path.relative_to(stage)): digest(path)
                for directory in ("include", "lib")
                for path in (stage / directory).rglob("*")
                if path.is_file()
            }
            (stage / ".complete.json").write_text(
                json.dumps({"identity": identity, "files": files}, sort_keys=True)
            )
            if prefix.exists():
                shutil.rmtree(prefix)
            stage.replace(prefix)
    return prefix


def run(
    command: list[str],
    *,
    env: dict[str, str] | None = None,
    stdin: IO[bytes] | None = None,
) -> None:
    subprocess.run(command, check=True, stdout=sys.stderr, env=env, stdin=stdin)


def compiler_env(original: dict[str, str]) -> dict[str, str]:
    """Pair bindgen's Clang with its own development resource headers."""
    env = original.copy()
    clang = env.get("CLANG_PATH")
    if not clang and env.get("LIBCLANG_PATH"):
        candidate = Path(env["LIBCLANG_PATH"]).parent / "bin/clang"
        if candidate.is_file():
            clang = str(candidate)
    clang = clang or shutil.which("clang", path=env.get("PATH"))
    if not clang:
        raise ValueError("native preparation requires clang")
    resource = subprocess.check_output(
        [clang, "-print-resource-dir"], text=True
    ).strip()
    if not (Path(resource) / "include/stddef.h").is_file():
        raise ValueError("selected Clang lacks development resource headers")
    env.setdefault("CLANG_PATH", clang)
    env.setdefault("BINDGEN_EXTRA_CLANG_ARGS", f"-resource-dir={resource}")
    if (
        Path(env.get("RUSTC_WRAPPER", "")).name == "sccache"
        and "CMAKE_TOOLCHAIN_FILE" not in env
    ):
        env["CMAKE_TOOLCHAIN_FILE"] = str(ROOT / ".config/native-cache.cmake")
        env["PSE_NATIVE_COMPILER_CACHE"] = env["RUSTC_WRAPPER"]
    return env


def klu(base: Path, env: dict[str, str]) -> Path:
    env = env.copy()
    env.setdefault("CC", shutil.which("cc") or "cc")
    env.setdefault("CXX", shutil.which("c++") or "c++")
    graph = json.loads(
        subprocess.check_output(
            [
                "cargo",
                "metadata",
                "--offline",
                "--locked",
                "--features",
                "pse-backend-native/native-solvers",
                "--format-version",
                "1",
            ],
            cwd=ROOT,
            env=env,
        )
    )
    packages = [
        p
        for p in graph["packages"]
        if p["name"] == "suitesparse_sys" and p["version"] == "0.1.4"
    ]
    if len(packages) != 1:
        raise ValueError("expected pinned suitesparse_sys 0.1.4")
    source = Path(packages[0]["manifest_path"]).parent / "vendor"
    source_hash = hashlib.sha256()
    for path in sorted(source.rglob("*")):
        if path.is_file():
            source_hash.update(str(path.relative_to(source)).encode())
            source_hash.update(bytes.fromhex(digest(path)))
    identity = {
        "source": source_hash.hexdigest(),
        "options": list(KLU_OPTIONS),
        "target": env.get("CARGO_BUILD_TARGET")
        or subprocess.check_output([env["CC"], "-dumpmachine"], text=True).strip(),
        "compiler": subprocess.check_output([env["CC"], "--version"], text=True),
        "compiler_bytes": digest(Path(shutil.which(env["CC"]) or env["CC"])),
        "cmake": subprocess.check_output(["cmake", "--version"], text=True),
        "toolchain_file": digest(Path(env["CMAKE_TOOLCHAIN_FILE"]))
        if env.get("CMAKE_TOOLCHAIN_FILE")
        else None,
        "flags": {
            key: env.get(key)
            for key in (
                "CC",
                "CXX",
                "CFLAGS",
                "CXXFLAGS",
                "FC",
                "FFLAGS",
                "CPPFLAGS",
                "LDFLAGS",
                "CMAKE_TOOLCHAIN_FILE",
                "SDKROOT",
                "MACOSX_DEPLOYMENT_TARGET",
            )
        },
    }

    def build(stage: Path, work: Path) -> None:
        launcher = env.get("RUSTC_WRAPPER", "")
        options = [f"-D{item}" for item in KLU_OPTIONS]
        if Path(launcher).name == "sccache":
            options += [
                f"-DCMAKE_C_COMPILER_LAUNCHER={launcher}",
                f"-DCMAKE_CXX_COMPILER_LAUNCHER={launcher}",
            ]
        run(
            [
                "cmake",
                "-S",
                str(source),
                "-B",
                str(work),
                f"-DCMAKE_INSTALL_PREFIX={stage}",
                *options,
            ],
            env=env,
        )
        run(
            [
                "cmake",
                "--build",
                str(work),
                "--parallel",
                env.get("CARGO_BUILD_JOBS", "4"),
            ],
            env=env,
        )
        run(["cmake", "--install", str(work)], env=env)

    return prepare(base, "klu", identity, KLU_FILES, build)


def solver(base: Path) -> Path:
    image = subprocess.check_output(
        [sys.executable, str(ROOT / "scripts/solver-images.py"), "ref", "dev"],
        text=True,
    ).strip()
    if "@sha256:" not in image:
        raise ValueError("solver extraction requires an immutable image digest")

    def extract(stage: Path, _work: Path) -> None:
        producer = subprocess.Popen(
            [
                "docker",
                "run",
                "--rm",
                "--entrypoint",
                "tar",
                image,
                "-C",
                "/opt/pse-solvers",
                "-cf",
                "-",
                ".",
            ],
            stdout=subprocess.PIPE,
        )
        try:
            run(["tar", "-xf", "-", "-C", str(stage)], stdin=producer.stdout)
        finally:
            if producer.stdout:
                producer.stdout.close()
            code = producer.wait()
        if code:
            raise RuntimeError(f"solver extraction failed: {code}")

    return prepare(base, "solver", {"image": image}, SOLVER_FILES, extract)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("klu", "solver", "compiler-shell"))
    args = parser.parse_args()
    if args.action == "compiler-shell":
        for key, value in compiler_env(dict(os.environ)).items():
            if os.environ.get(key) != value:
                print(f"export {key}={shlex.quote(value)}")
    else:
        base = cache_root(dict(os.environ))
        print(
            klu(base, compiler_env(dict(os.environ)))
            if args.action == "klu"
            else solver(base)
        )


if __name__ == "__main__":
    main()
