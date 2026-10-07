# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Persistent, identity-keyed preparation of the resolved native inputs."""

from __future__ import annotations

import argparse
import contextlib
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
import uuid
from pathlib import Path
from typing import IO, TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable, Generator, Mapping

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
RECEIPT_VERSION = 2
ISOLATION_FILES = (
    "include/ibex.h",
    "lib/libibex.a",
    "lib/ibex/3rd/libprim.a",
    "lib/ibex/3rd/libsoplex.a",
    "share/pkgconfig/ibex.pc",
)
INPUT_ENV = (
    "PATH",
    "CC",
    "CXX",
    "FC",
    "CFLAGS",
    "CXXFLAGS",
    "FFLAGS",
    "CPPFLAGS",
    "LDFLAGS",
    "CMAKE_TOOLCHAIN_FILE",
    "SDKROOT",
    "MACOSX_DEPLOYMENT_TARGET",
    "CARGO_BUILD_TARGET",
    "RUSTC_WRAPPER",
    "LIBCLANG_PATH",
    "CLANG_PATH",
    "BINDGEN_EXTRA_CLANG_ARGS",
    "PSE_SOLVER_IMAGE",
)


def operation_preparation(
    kind: str,
) -> Callable[
    [Callable[[Path, dict[str, str]], Path]], Callable[[Path, dict[str, str]], Path]
]:
    def decorate(
        provider: Callable[[Path, dict[str, str]], Path],
    ) -> Callable[[Path, dict[str, str]], Path]:
        def prepared(base: Path, env: dict[str, str]) -> Path:
            from scripts import (  # noqa: PLC0415 -- owner cycle
                native_operation as operation,
            )

            inputs = {name: env.get(name) for name in INPUT_ENV}
            key = json.dumps(
                {"kind": kind, "base": str(base), "root": str(ROOT), "inputs": inputs},
                sort_keys=True,
            )
            return Path(operation.observe(key, lambda: str(provider(base, env))))

        return prepared

    return decorate


def tool_identity(env: dict[str, str], name: str, default: str) -> dict[str, str]:
    from scripts import native_operation as operation  # noqa: PLC0415 -- owner cycle

    command = env.get(name, default)
    selected = shutil.which(command, path=env.get("PATH"))
    if selected is None:
        raise ValueError(f"selected native tool {name} is unavailable")

    def observe() -> dict[str, str]:
        value = {
            "path": selected,
            "bytes": digest(Path(selected)),
            "version": subprocess.check_output(
                [selected, "--version"], env=env, text=True
            ),
        }
        if name == "CC":
            value["target"] = subprocess.check_output(
                [selected, "-dumpmachine"], env=env, text=True
            ).strip()
        return value

    return operation.observe("tool:" + name + ":" + selected, observe)


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
    "include/coin-or/IpTypes.h",
    "include/coin-or/IpoptConfig.h",
    "include/coin-or/IpReturnCodes.h",
    "include/coin-or/IpReturnCodes_inc.h",
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


def installation_files(prefix: Path) -> dict[str, str]:
    files = {}
    for path in sorted(prefix.rglob("*")):
        if path == prefix / ".complete.json" or not path.is_file():
            continue
        if not path.resolve().is_relative_to(prefix.resolve()):
            raise ValueError("native generation contains an unmanaged external file")
        files[str(path.relative_to(prefix))] = digest(path)
    return files


def valid(
    prefix: Path, identity: Mapping[str, object], required: tuple[str, ...]
) -> bool:
    try:
        receipt = json.loads((prefix / ".complete.json").read_text())
        return (
            receipt.get("version") == RECEIPT_VERSION
            and receipt["identity"] == identity
            and set(required).issubset(receipt["files"])
            and bool(receipt["files"])
            and installation_files(prefix) == receipt["files"]
        )
    except (OSError, ValueError, KeyError):
        return False


def identity_location(base: Path, kind: str, identity: Mapping[str, object]) -> Path:
    key = hashlib.sha256(json.dumps(identity, sort_keys=True).encode()).hexdigest()
    return base / kind / key


def location(base: Path, kind: str, identity: Mapping[str, object]) -> Path:
    """Read-only candidate lookup; this does not grant installed validity."""
    owner = identity_location(base, kind, identity)
    try:
        pointer = json.loads((owner / ".current.json").read_text())
        generation = pointer["generation"]
        if (
            pointer.get("version") != RECEIPT_VERSION
            or len(generation) != 32
            or any(c not in "0123456789abcdef" for c in generation)
        ):
            return owner
        return owner / "generations" / generation
    except (OSError, ValueError, KeyError):
        return owner


@contextlib.contextmanager
def coordination(owner: Path, name: str) -> Generator[None, None, None]:
    # Separate construction from short publication/use coordination.
    owner.mkdir(parents=True, exist_ok=True)
    with (owner / name).open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        yield


def generation_destination(stage: Path) -> Path:
    """The builder's immutable final prefix, chosen before configuration."""
    return Path((stage.parent / ".destination").read_text())


def seal_generation(prefix: Path) -> None:
    """Remove write bits from an owned generation without changing its bytes.

    Existing version-two receipts predate read-only publication. Managers seal
    those generations under publication coordination before admitting their use;
    readers remain valid and privileged owner mutation is outside this contract.
    """
    for path in (*prefix.rglob("*"), prefix):
        if not path.is_symlink():
            mode = path.stat().st_mode
            if mode & 0o222:
                path.chmod(mode & ~0o222)


def collect(base: Path, kind: str, identity: Mapping[str, object]) -> list[Path]:
    from scripts import native_operation as operation  # noqa: PLC0415 -- owner cycle

    owner = identity_location(base, kind, identity)
    removed = []
    with coordination(owner, ".publication.lock"):
        selected = location(base, kind, identity)
        for generation in (owner / "generations").glob("*"):
            if generation == selected or operation.generation_in_use(base, generation):
                continue
            for directory in (
                generation,
                *(
                    path
                    for path in generation.rglob("*")
                    if path.is_dir() and not path.is_symlink()
                ),
            ):
                directory.chmod(directory.stat().st_mode | 0o200)
            shutil.rmtree(generation)
            removed.append(generation)
    return removed


def admit_external(prefix: Path, required: tuple[str, ...]) -> None:
    """External mutable installations get full byte/interface admission each use."""
    if not all((prefix / name).is_file() for name in required):
        raise ValueError("explicit native prefix lacks required consumed interfaces")
    # A manager-issued generation may reuse its already admitted immutable owner.
    from scripts import native_operation as operation  # noqa: PLC0415 -- owner cycle

    key = "prefix:" + str(prefix.resolve())
    prior = operation.remembered(key)
    if (
        prior is not None
        and prior.get("managed")
        and set(required).issubset(prior["required"])
    ):
        return
    receipt_path = prefix / ".complete.json"
    try:
        receipt = json.loads(receipt_path.read_text())
    except (OSError, ValueError):
        receipt = None
    base = cache_root(dict(os.environ))
    prefix = prefix.resolve()
    owner = prefix.parent.parent
    managed = (
        receipt is not None
        and prefix.parent.name == "generations"
        and owner.parent.parent == base
        and owner
        == identity_location(base, owner.parent.name, receipt.get("identity", {}))
    )
    if managed:
        with coordination(owner, ".publication.lock"):
            operation.pin(base, prefix)
            seal_generation(prefix)
        if not valid(prefix, receipt["identity"], required):
            raise ValueError("explicit managed native generation is corrupt")
        operation.admit(
            key,
            {
                "managed": True,
                "required": list(required),
                "identity": receipt["identity"],
            },
        )
    else:
        observed = installation_files(prefix)
        operation.admit(
            key, {"managed": False, "required": list(required), "files": observed}
        )


def prepare(
    base: Path,
    kind: str,
    identity: Mapping[str, object],
    required: tuple[str, ...],
    builder: Callable[[Path, Path], None],
) -> Path:
    """Verify once per real operation and publish immutable installation generations."""
    from scripts import native_operation as operation  # noqa: PLC0415 -- owner cycle

    owner = identity_location(base, kind, identity)
    key = "installation:" + str(owner)
    remembered = operation.remembered(key)
    if remembered is not None:
        if (
            remembered["identity"] != identity
            or tuple(remembered["required"]) != required
        ):
            raise ValueError("native operation request changed")
        return Path(remembered["prefix"])

    def candidate() -> Path | None:
        with coordination(owner, ".publication.lock"):
            prefix = location(base, kind, identity)
            if prefix == owner or not prefix.is_dir():
                return None
            # Registration and selection are coherent with collection. Hashing is
            # outside publication coordination, and readers never take build lock.
            operation.pin(base, prefix)
            seal_generation(prefix)
        if valid(prefix, identity, required):
            operation.admit(
                key,
                {
                    "prefix": str(prefix),
                    "identity": identity,
                    "required": list(required),
                },
            )
            operation.admit(
                "prefix:" + str(prefix.resolve()),
                {"managed": True, "required": list(required), "identity": identity},
            )
            collect(base, kind, identity)
            return prefix
        return None

    found = candidate()
    if found is not None:
        return found
    with coordination(owner, ".build.lock"):
        found = candidate()
        if found is not None:
            return found
        generation = uuid.uuid4().hex
        destination = owner / "generations" / generation
        with tempfile.TemporaryDirectory(prefix=".stage-", dir=owner) as scratch:
            stage = Path(scratch) / "install"
            stage.mkdir()
            (Path(scratch) / ".destination").write_text(str(destination))
            builder(stage, Path(scratch) / "build")
            missing = [name for name in required if not (stage / name).is_file()]
            if missing:
                raise ValueError(
                    f"incomplete {kind} installation: missing {', '.join(missing)}"
                )
            files = installation_files(stage)
            (stage / ".complete.json").write_text(
                json.dumps(
                    {"version": RECEIPT_VERSION, "identity": identity, "files": files},
                    sort_keys=True,
                )
            )
            if not valid(stage, identity, required):
                raise ValueError("native generation failed complete byte admission")
            # Only manager-controlled publication/retirement may mutate generations.
            # Arbitrary owner-privileged chmod/writes require new admission.
            for path in stage.rglob("*"):
                if not path.is_symlink():
                    path.chmod(path.stat().st_mode & ~0o222)
            with coordination(owner, ".publication.lock"):
                destination.parent.mkdir(parents=True, exist_ok=True)
                stage.replace(destination)
                destination.chmod(destination.stat().st_mode & ~0o222)
                operation.pin(base, destination)
                operation.write_json(
                    owner / ".current.json",
                    {"version": RECEIPT_VERSION, "generation": generation},
                )
        operation.admit(
            key,
            {
                "prefix": str(destination),
                "identity": identity,
                "required": list(required),
            },
        )
        operation.admit(
            "prefix:" + str(destination.resolve()),
            {"managed": True, "required": list(required), "identity": identity},
        )
        collect(base, kind, identity)
        return destination


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
    from scripts import native_operation as operation  # noqa: PLC0415 -- owner cycle

    resource = operation.observe(
        "clang-resource:" + clang,
        lambda: subprocess.check_output(
            [clang, "-print-resource-dir"], text=True
        ).strip(),
    )
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


def prepend_library_path(libraries: str, env: dict[str, str]) -> str:
    """Move selected libraries first without multiplying paths on recipe reentry."""
    previous = env.get("LD_LIBRARY_PATH")
    if not previous:
        return libraries
    selected = libraries.split(os.pathsep)
    return os.pathsep.join(
        [
            *selected,
            *(path for path in previous.split(os.pathsep) if path not in selected),
        ]
    )


@operation_preparation("klu")
def klu(base: Path, env: dict[str, str]) -> Path:
    env = env.copy()
    env.setdefault("CC", shutil.which("cc") or "cc")
    env.setdefault("CXX", shutil.which("c++") or "c++")
    compiler = tool_identity(env, "CC", "cc")
    cmake = tool_identity(env, "CMAKE", "cmake")
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
        "target": env.get("CARGO_BUILD_TARGET") or compiler["target"],
        "compiler": compiler["version"],
        "compiler_bytes": compiler["bytes"],
        "cmake": cmake["version"],
        "cmake_bytes": cmake["bytes"],
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


@operation_preparation("isolation")
def isolation(base: Path, env: dict[str, str]) -> Path:
    """Prepare the source-pinned validated interval and supported LP binding."""
    with (ROOT / "Cargo.toml").open("rb") as stream:
        pin = tomllib.load(stream)["workspace"]["metadata"]["pse"]["root-isolation"]
    env = env.copy()
    env.setdefault("CC", shutil.which("cc") or "cc")
    env.setdefault("CXX", shutil.which("c++") or "c++")
    cc = tool_identity(env, "CC", "cc")
    cxx = tool_identity(env, "CXX", "c++")
    cmake = tool_identity(env, "CMAKE", "cmake")
    identity = {
        "pin": pin,
        "target": env.get("CARGO_BUILD_TARGET") or cc["target"],
        "toolchain_file": digest(Path(env["CMAKE_TOOLCHAIN_FILE"]))
        if env.get("CMAKE_TOOLCHAIN_FILE")
        else None,
        "cc": cc["version"],
        "cxx": cxx["version"],
        "cc_bytes": cc["bytes"],
        "cxx_bytes": cxx["bytes"],
        "cmake": cmake["version"],
        "cmake_bytes": cmake["bytes"],
        "flags": {
            key: env.get(key)
            for key in ("CFLAGS", "CXXFLAGS", "LDFLAGS", "CMAKE_TOOLCHAIN_FILE")
        },
    }
    required = ISOLATION_FILES

    def build(stage: Path, work: Path) -> None:
        destination = generation_destination(stage)
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
        from scripts import pse_env  # noqa: PLC0415 -- placement owner

        # A fresh scope per command: a transient unit name cannot be reused.
        def cap() -> list[str]:
            return pse_env.placement(env, native=True)
        run(
            [
                *cap(),
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
                *cap(),
                "cmake",
                "--build",
                str(work / "compiled"),
                "--parallel",
                env.get("CARGO_BUILD_JOBS", "4"),
            ],
            env=env,
        )
        run([*cap(), "cmake", "--install", str(work / "compiled")], env=env)
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
    from scripts import native_operation as operation  # noqa: PLC0415 -- owner cycle

    image = operation.observe(
        "solver-image:" + str(ROOT) + ":" + os.environ.get("PSE_SOLVER_IMAGE", ""),
        solver_image,
    )

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
