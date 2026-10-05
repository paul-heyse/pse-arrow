# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Persistent, identity-keyed preparation of the resolved native inputs."""

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
import tarfile
import tempfile
import tomllib
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
#: The solver image contract (ADR-0108, ADR-0105): Ipopt with MUMPS+METIS,
#: SPRAL and oneMKL Pardiso, oneMKL as the one BLAS/LAPACK, and SCIP 10.
SOLVER_FILES = (
    "include/coin-or/IpIpoptApplication.hpp",
    "include/coin-or/IpStdCInterface.h",
    "include/coin-or/IpLinearSolvers.h",
    "lib/libipopt.so",
    "lib/libcoinmumps.so.3",
    "lib/libmetis.so",
    "include/spral_ssids.h",
    "lib/libspral.so",
    "include/mkl.h",
    "lib/libmkl_intel_lp64.so.3",
    "lib/libmkl_gnu_thread.so.3",
    "lib/libmkl_core.so.3",
    "lib/pkgconfig/mkl-dynamic-lp64-gomp.pc",
    "include/scip/scip.h",
    "include/scip/scipdefplugins.h",
    "include/scip/config.h",
    "lib/libscip.so",
    "share/pse-solvers/build-info.txt",
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


def location(base: Path, kind: str, identity: dict) -> Path:
    """Where the installation of ``identity`` lives, whether or not it is prepared."""
    key = hashlib.sha256(json.dumps(identity, sort_keys=True).encode()).hexdigest()
    return base / kind / key


def prepare(
    base: Path,
    kind: str,
    identity: dict,
    required: tuple[str, ...],
    builder: Callable[[Path, Path], None],
) -> Path:
    """Serialize per identity; expose a new installation only after full verification."""
    prefix = location(base, kind, identity)
    parent = prefix.parent
    parent.mkdir(parents=True, exist_ok=True)
    with (parent / f"{prefix.name}.lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if valid(prefix, identity, required):
            return prefix
        with tempfile.TemporaryDirectory(
            prefix=f".{prefix.name}-", dir=parent
        ) as scratch:
            stage = Path(scratch) / "install"
            stage.mkdir()
            builder(stage, Path(scratch) / "build")
            missing = [name for name in required if not (stage / name).is_file()]
            if missing:
                raise ValueError(
                    f"incomplete {kind} installation: missing {', '.join(missing)}"
                )
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
    packages = [p for p in graph["packages"] if p["name"] == "suitesparse_sys"]
    if len(packages) != 1:
        raise ValueError(
            "expected one resolved suitesparse_sys source for the native build"
        )
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


def isolation(base: Path, env: dict[str, str]) -> Path:
    """Prepare the source-pinned validated interval and supported LP binding."""
    with (ROOT / "Cargo.toml").open("rb") as stream:
        pin = tomllib.load(stream)["workspace"]["metadata"]["pse"]["root-isolation"]
    env = env.copy()
    env.setdefault("CC", shutil.which("cc") or "cc")
    env.setdefault("CXX", shutil.which("c++") or "c++")
    identity = {
        "pin": pin,
        "target": env.get("CARGO_BUILD_TARGET")
        or subprocess.check_output([env["CC"], "-dumpmachine"], text=True).strip(),
        "toolchain_file": digest(Path(env["CMAKE_TOOLCHAIN_FILE"]))
        if env.get("CMAKE_TOOLCHAIN_FILE")
        else None,
        "cc": subprocess.check_output([env["CC"], "--version"], text=True),
        "cxx": subprocess.check_output([env["CXX"], "--version"], text=True),
        "cc_bytes": digest(Path(shutil.which(env["CC"]) or env["CC"])),
        "cxx_bytes": digest(Path(shutil.which(env["CXX"]) or env["CXX"])),
        "cmake": subprocess.check_output(["cmake", "--version"], text=True),
        "flags": {
            key: env.get(key)
            for key in ("CFLAGS", "CXXFLAGS", "LDFLAGS", "CMAKE_TOOLCHAIN_FILE")
        },
    }
    required = (
        "include/ibex.h",
        "lib/libibex.a",
        "lib/ibex/3rd/libprim.a",
        "lib/ibex/3rd/libsoplex.a",
        "share/pkgconfig/ibex.pc",
    )
    destination = location(base, "isolation", identity)

    def build(stage: Path, work: Path) -> None:
        work.mkdir(parents=True)
        archive = work / "source.tar.gz"
        run(
            [
                "curl",
                "--fail",
                "--location",
                "--proto",
                "-all,https",
                "--proto-redir",
                "-all,https",
                "--max-time",
                "120",
                "--output",
                str(archive),
                pin["archive"],
            ],
            env=env,
        )
        if digest(archive) != pin["sha256"]:
            raise ValueError("validated-root source archive checksum mismatch")
        source = work / "source"
        source.mkdir()
        with tarfile.open(archive, "r:gz") as package:
            package.extractall(source, filter="data")
        directories = [path for path in source.iterdir() if path.is_dir()]
        if len(directories) != 1:
            raise ValueError("validated-root archive layout")
        cap = ["bash", str(ROOT / "scripts/memory-cap.sh")]
        run(
            [
                *cap,
                "cmake",
                "-S",
                str(directories[0]),
                "-B",
                str(work / "compiled"),
                "-G",
                "Unix Makefiles",
                f"-DCMAKE_INSTALL_PREFIX={stage}",
                *[f"-D{option}" for option in pin["options"]],
            ],
            env=env,
        )
        run(
            [
                *cap,
                "cmake",
                "--build",
                str(work / "compiled"),
                "--parallel",
                env.get("CARGO_BUILD_JOBS", "4"),
            ],
            env=env,
        )
        run([*cap, "cmake", "--install", str(work / "compiled")], env=env)
        configuration = (stage / "share/ibex/cmake/ibex-config.cmake").read_text()
        for key, expected in (
            ("IBEX_INTERVAL_LIB_VERSION", pin["interval-version"]),
            ("IBEX_LP_LIB_VERSION", pin["lp-version"]),
        ):
            if f"set ({key} {expected})" not in configuration:
                raise ValueError(f"validated-root native dependency mismatch: {key}")
        # prepare() atomically relocates the stage. Native metadata must describe
        # the final prefix, not the temporary build/install directory.
        for pattern in ("*.pc", "*.cmake"):
            for metadata in stage.rglob(pattern):
                metadata.write_text(
                    metadata.read_text().replace(str(stage), str(destination))
                )

    return prepare(base, "isolation", identity, required, build)


def solver_image() -> str:
    """The solver image native work uses.

    It is the pinned dev image, or the one ``PSE_SOLVER_IMAGE`` names (a local image
    ID or a digest reference).
    """
    resolved = subprocess.run(
        [sys.executable, str(ROOT / "scripts/solver-images.py"), "runtime", "dev"],
        text=True,
        capture_output=True,
        check=False,
    )
    if resolved.returncode:
        raise ValueError(resolved.stderr.strip() or "cannot resolve the solver image")
    image = resolved.stdout.strip()
    if not (image.startswith("sha256:") or "@sha256:" in image):
        raise ValueError("solver extraction requires an immutable image identity")
    return image


def prepared_solver(base: Path) -> Path | None:
    """The solver prefix if it is already extracted for the current image, else None.

    Never runs docker or extracts anything, so it is cheap enough for the doctor. It
    reads the receipt's identity only; ``solver`` still verifies every file digest
    before a build uses the prefix.
    """
    try:
        identity = {"image": solver_image()}
    except ValueError:
        return None
    prefix = location(base, "solver", identity)
    try:
        receipt = json.loads((prefix / ".complete.json").read_text())
    except (OSError, ValueError):
        return None
    return prefix if receipt.get("identity") == identity else None


def solver(base: Path) -> Path:
    """Extract ``/opt/pse-solvers`` from the solver image into the cache."""
    image = solver_image()

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

    try:
        return prepare(base, "solver", {"image": image}, SOLVER_FILES, extract)
    except ValueError as error:
        raise ValueError(
            f"{error} (image {image}); an image built before this contract "
            "cannot serve it: pin a rebuilt image or set PSE_SOLVER_IMAGE"
        ) from error


def runtime_env(dockerfile: Path) -> dict[str, str]:
    """The solver image's runtime process environment, without its paths into the image.

    The ``solvers`` stage's ``ENV`` of ``docker/solvers/Dockerfile`` is the one
    declaration (``solver-pin-check`` ties the pinned image to that tree): the MKL and
    OpenMP settings the libraries need, such as ``OMP_CANCELLATION`` for SPRAL. Native
    binaries that run on the host against the extracted prefix need them too; variables
    that point into ``/opt/pse-solvers``, and ``PATH``, are the prefix's and are left out.
    """
    lines: list[str] = []
    pending = ""
    for raw in dockerfile.read_text().splitlines():
        line = raw.rstrip()
        if not pending and line.lstrip().startswith("#"):
            continue
        if line.endswith("\\"):
            pending += line[:-1] + " "
            continue
        lines.append(pending + line)
        pending = ""
    stage = None
    env: dict[str, str] = {}
    for line in lines:
        words = line.split()
        if not words:
            continue
        if words[0].upper() == "FROM":
            stage = words[3] if len(words) >= 4 and words[2].upper() == "AS" else None
        elif words[0].upper() == "ENV" and stage == "solvers":
            for pair in shlex.split(line.strip()[3:]):
                key, _, value = pair.partition("=")
                env[key] = value
    if not env:
        raise ValueError(f"{dockerfile} declares no solvers-stage ENV")
    return {
        key: value
        for key, value in env.items()
        if key != "PATH" and "/opt/pse-solvers" not in value
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "action",
        choices=("klu", "isolation", "solver", "compiler-shell", "runtime-env"),
    )
    args = parser.parse_args()
    if args.action == "runtime-env":
        for key, value in runtime_env(ROOT / "docker/solvers/Dockerfile").items():
            print(f"export {key}={shlex.quote(value)}")
    elif args.action == "compiler-shell":
        for key, value in compiler_env(dict(os.environ)).items():
            if os.environ.get(key) != value:
                print(f"export {key}={shlex.quote(value)}")
    else:
        base = cache_root(dict(os.environ))
        print(
            klu(base, compiler_env(dict(os.environ)))
            if args.action == "klu"
            else isolation(base, compiler_env(dict(os.environ)))
            if args.action == "isolation"
            else solver(base)
        )


if __name__ == "__main__":
    main()
