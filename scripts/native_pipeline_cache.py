# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Prepare pinned, scoped Uno/PETSc inputs using the existing native cache owner."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import tomllib
from pathlib import Path

if str(Path(__file__).resolve().parents[1]) not in sys.path:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from scripts import native_cache as cache
from scripts import native_operation as operation
from scripts import producer_deployment


def required_files(kind: str) -> tuple[str, ...]:
    if kind == "uno":
        return ("lib/libuno.a", "include/uno/Uno_C_API.h")
    if kind == "petsc":
        return ("lib/libpetsc.so", "include/petscsnes.h", "include/petscts.h")
    raise ValueError("unknown native pipeline capability")


def source_archive(base: Path, url: str, checksum: str) -> Path:
    directory = base / "sources"
    directory.mkdir(parents=True, exist_ok=True)
    archive = directory / f"{checksum}.tar.gz"
    if archive.is_file() and cache.digest(archive) == checksum:
        return archive
    temporary = archive.with_suffix(f".{os.getpid()}.download")
    try:
        cache.run(
            [
                "curl",
                "--fail",
                "--location",
                "--max-time",
                "120",
                url,
                "--output",
                str(temporary),
            ]
        )
        if cache.digest(temporary) != checksum:
            raise ValueError("foreign solver source checksum mismatch")
        temporary.replace(archive)
    finally:
        temporary.unlink(missing_ok=True)
    return archive


def highs_archive(env: dict[str, str]) -> Path:
    """Reuse only a common eligible provider capture, otherwise discover once."""
    request = {
        "root": str(cache.ROOT),
        **{
            name: env.get(name)
            for name in (
                *cache.INPUT_ENV,
                "RUSTFLAGS",
                "CARGO_ENCODED_RUSTFLAGS",
                "PSE_CARGO_TARGET_DIR",
                "PSE_NATIVE_PROVIDER_RECEIPT",
            )
        },
    }
    key = json.dumps(request, sort_keys=True)
    return Path(
        operation.observe(
            "highs-cargo:" + key, lambda: str(_highs_candidate(env, request))
        )
    )


def highs_files(directory: Path) -> dict[str, str]:
    required = (directory / "lib/libhighs.a", directory / "include/highs/Highs.h")
    if not all(path.is_file() for path in required):
        raise ValueError("Cargo's HiGHS archive/header installation is incomplete")
    paths = [required[0], *sorted((directory / "include").rglob("*"))]
    return {
        str(path.relative_to(directory)): cache.digest(path)
        for path in paths
        if path.is_file()
    }


def _highs_candidate(env: dict[str, str], request: dict) -> Path:
    # The proof locator admits an existing candidate; it does not select the
    # compiler output. Keep it in the operation observation key above so changing
    # the supplied receipt still causes a fresh admission attempt.
    candidate_inputs = {
        name: value
        for name, value in request.items()
        if name != "PSE_NATIVE_PROVIDER_RECEIPT"
    }
    owner = (
        cache.cache_root(env)
        / "highs-cargo"
        / hashlib.sha256(
            json.dumps(candidate_inputs, sort_keys=True).encode()
        ).hexdigest()
    )
    with cache.coordination(owner, ".build.lock"):
        candidate = owner / ".candidate.json"
        receipt = env.get("PSE_NATIVE_PROVIDER_RECEIPT")
        try:
            association = json.loads(candidate.read_text())
            directory = Path(association["directory"])
            if receipt and producer_deployment.verify_native_provider(
                Path(receipt), association["package"], directory, env
            ):
                return directory
        except (OSError, ValueError, KeyError):
            pass
        # A dev link is usable without persistent producer eligibility. Its actual
        # provider state is reused only by this real operation, never by existence.
        directory, package = _build_highs_archive(env)
        operation.write_json(
            candidate, {"directory": str(directory), "package": package}
        )
        return directory


def _build_highs_archive(env: dict[str, str]) -> tuple[Path, str]:
    """Cargo owns the exact archive and feature unit. Consume its actual build message."""
    from scripts import pse_env  # noqa: PLC0415 -- placement owner

    completed = subprocess.run(
        [
            *pse_env.placement(env, native=True),
            "cargo",
            "build",
            "--locked",
            "-p",
            "pse-uno-sys",
            "--features",
            "pse-uno-sys/highs-provider",
            "--message-format=json",
        ],
        cwd=cache.ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        check=False,
    )
    outputs = [
        json.loads(line)
        for line in completed.stdout.splitlines()
        if line.startswith("{")
    ]
    for message in outputs:
        if message.get("reason") == "compiler-message" and message["message"].get(
            "rendered"
        ):
            print(message["message"]["rendered"], file=sys.stderr, end="")
    completed.check_returncode()
    messages = [
        message
        for message in outputs
        if message.get("reason") == "build-script-executed"
        and message["package_id"].split("#")[-1].startswith("highs-sys@")
    ]
    if len(messages) != 1:
        raise ValueError("Cargo must supply one matching HiGHS build")
    message = messages[0]
    directory = Path(message["out_dir"])
    highs_files(directory)
    return directory, message["package_id"]


def verify_petsc_strings(
    headers: Path, libraries: Path, env: dict[str, str], work: Path
) -> None:
    """Check the actual inline helper's boundary behavior with guarded storage."""
    work.mkdir(parents=True, exist_ok=True)
    source = work / "petsc-string-control.c"
    source.write_text(r"""#include <petscsys.h>
#include <string.h>
#define CHECK(condition) do { if (!(condition)) return __LINE__; } while (0)
int main(void) {
  char exact[5] = "a", truncated[5] = "a", invalid[5] = "abcd", zero[1] = "";
  struct { char data[1]; char guard; } one = {{0}, 'G'};
  struct { char data[4]; char guard; } boundary = {{'a', 0, 0, 0}, 'G'};
  if (PetscInitializeNoArguments()) return 1;
  if (PetscPushErrorHandler(PetscReturnErrorHandler, NULL)) return 2;
  CHECK(PetscStrlcat(exact, "bcd", sizeof(exact)) == PETSC_SUCCESS);
  CHECK(strcmp(exact, "abcd") == 0);
  CHECK(PetscStrlcat(truncated, "bcdefgh", sizeof(truncated)) == PETSC_SUCCESS);
  CHECK(strcmp(truncated, "abcd") == 0);
  CHECK(PetscStrlcat(boundary.data, "bcdefgh", sizeof(boundary.data)) == PETSC_SUCCESS);
  CHECK(strcmp(boundary.data, "abc") == 0 && boundary.guard == 'G');
  CHECK(PetscStrlcat(one.data, "x", sizeof(one.data)) == PETSC_SUCCESS);
  CHECK(one.data[0] == 0 && one.guard == 'G');
  CHECK(PetscStrlcat(zero, "x", 0) == PETSC_ERR_ARG_SIZ && zero[0] == 0);
  CHECK(PetscStrlcat(invalid, "x", 4) == PETSC_ERR_ARG_SIZ && strcmp(invalid, "abcd") == 0);
  CHECK(PetscStrlcat(invalid, "x", 1) == PETSC_ERR_ARG_SIZ && strcmp(invalid, "abcd") == 0);
  CHECK(PetscStrlcat(one.data, NULL, 0) == PETSC_SUCCESS && one.data[0] == 0);
  if (PetscPopErrorHandler()) return 3;
  return PetscFinalize();
}
""")
    binary = work / "petsc-string-control"
    from scripts import pse_env  # noqa: PLC0415 -- placement owner

    cap = pse_env.placement(env, native=True)
    cache.run(
        [
            *cap,
            env.get("CC", "cc"),
            "-std=c11",
            "-O3",
            "-Wall",
            "-Werror=stringop-overflow",
            "-I",
            str(headers),
            str(source),
            "-L",
            str(libraries),
            f"-Wl,-rpath,{libraries}",
            "-lpetsc",
            "-o",
            str(binary),
        ],
        env=env,
    )
    control_env = dict(env)
    control_env["LD_LIBRARY_PATH"] = cache.prepend_library_path(str(libraries), env)
    cache.run([*cap, str(binary)], env=control_env)


def prepare(kind: str, env: dict[str, str]) -> Path:

    key = json.dumps(
        {
            "kind": kind,
            "base": str(cache.cache_root(env)),
            "root": str(cache.ROOT),
            "solver": env.get("IPOPT_DIR"),
            "inputs": {name: env.get(name) for name in cache.INPUT_ENV},
        },
        sort_keys=True,
    )
    return Path(operation.observe("pipeline:" + key, lambda: str(_prepare(kind, env))))


def _prepare(kind: str, env: dict[str, str]) -> Path:
    base = cache.cache_root(env)
    with (cache.ROOT / "Cargo.toml").open("rb") as stream:
        pin = tomllib.load(stream)["workspace"]["metadata"]["pse"][kind]
    solver = Path(env["IPOPT_DIR"]) if env.get("IPOPT_DIR") else cache.solver(base)
    cache.admit_external(solver, cache.SOLVER_FILES)
    blas = [
        solver / "lib" / name
        for name in (
            "libmkl_intel_lp64.so.3",
            "libmkl_gnu_thread.so.3",
            "libmkl_core.so.3",
        )
    ]
    if not all(path.is_file() for path in blas):
        raise ValueError(
            "scoped foreign solvers require the existing real-double LP64 BLAS provider"
        )
    high = highs_archive(env) if kind == "uno" else None
    patch = cache.ROOT / pin["patch"]
    archive = source_archive(base, pin["archive"], pin["sha256"])
    cc = cache.tool_identity(env, "CC", "cc")
    cxx = cache.tool_identity(env, "CXX", "c++")
    identity = {
        "pin": pin,
        "blas": {str(path): cache.digest(path) for path in blas},
        "highs": {
            "archive": cache.digest(high / "lib/libhighs.a"),
            "headers": {
                str(path.relative_to(high)): cache.digest(path)
                for path in sorted((high / "include").rglob("*"))
                if path.is_file()
            },
        }
        if high
        else None,
        "patch": cache.digest(patch),
        "compiler": {"CC": cc, "CXX": cxx},
        "flags": {
            name: env.get(name)
            for name in ("CFLAGS", "CXXFLAGS", "LDFLAGS", "CMAKE_TOOLCHAIN_FILE")
        },
        "target": env.get("CARGO_BUILD_TARGET") or cc["target"],
    }
    if kind == "petsc":
        identity["source_archive"] = cache.digest(archive)
    if kind == "uno":
        identity["provider_contract"] = "single-lp64-blas-and-lapack"

    def build(stage: Path, work: Path) -> None:
        destination = cache.generation_destination(stage)
        work.mkdir(parents=True)
        with tarfile.open(archive) as packed:
            packed.extractall(work / "source", filter="data")
        roots = list((work / "source").iterdir())
        if len(roots) != 1 or not roots[0].is_dir():
            raise ValueError("expected one foreign source root")
        source = roots[0]
        from scripts import pse_env  # noqa: PLC0415 -- placement owner

        cap = pse_env.placement(env, native=True)
        subprocess.run(["git", "apply", "--check", str(patch)], cwd=source, check=True)
        subprocess.run(["git", "apply", str(patch)], cwd=source, check=True)
        if kind == "uno":
            if high is None:
                raise ValueError("Uno requires the admitted HiGHS unit")
            cache.run(
                [
                    *cap,
                    "cmake",
                    "-S",
                    str(source),
                    "-B",
                    str(work / "compiled"),
                    f"-DCMAKE_INSTALL_PREFIX={stage}",
                    f"-DHIGHS={high / 'lib/libhighs.a'}",
                    f"-DHIGHS_INCLUDE_DIR={high / 'include/highs'}",
                    f"-DBLAS_LIBRARIES={';'.join(str(path) for path in blas)}",
                    f"-DLAPACK_LIBRARIES={';'.join(str(path) for path in blas)}",
                    *[f"-D{option}" for option in pin["options"]],
                ],
                env=env,
            )
            cache.run(
                [
                    *cap,
                    "cmake",
                    "--build",
                    str(work / "compiled"),
                    "--target",
                    "uno_static",
                    "--parallel",
                    "8",
                ],
                env=env,
            )
            cache.run(["cmake", "--install", str(work / "compiled")], env=env)
        else:
            settings = [
                *pin["options"],
                f"--prefix={destination}",
                f"--with-cc={env.get('CC', 'cc')}",
                f"--with-cxx={env.get('CXX', 'c++')}",
                "COPTFLAGS=-O3 -ffp-contract=off -fno-fast-math",
                "CXXOPTFLAGS=-O3 -ffp-contract=off -fno-fast-math",
                f"--with-blaslapack-lib=[{','.join(str(path) for path in blas)},-lgomp,-lpthread,-lm,-ldl]",
            ]
            subprocess.run(
                [*cap, sys.executable, str(source / "configure"), *settings],
                cwd=source,
                env=env,
                check=True,
                stdout=sys.stderr,
            )
            subprocess.run(
                [*cap, "make", "-j8", "all"],
                cwd=source,
                env=env,
                check=True,
                stdout=sys.stderr,
            )
            destdir = work / "destination"
            subprocess.run(
                ["make", "install", f"DESTDIR={destdir}"],
                cwd=source,
                env=env,
                check=True,
                stdout=sys.stderr,
            )
            installed = destdir / destination.relative_to(destination.anchor)
            for entry in installed.iterdir():
                shutil.move(str(entry), stage / entry.name)
            verify_petsc_strings(
                stage / "include", stage / "lib", env, work / "string-control"
            )
        for pattern in ("*.pc", "*.cmake"):
            for metadata in stage.rglob(pattern):
                metadata.write_text(
                    metadata.read_text().replace(str(stage), str(destination))
                )

    return cache.prepare(base, kind, identity, required_files(kind), build)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kind", choices=("uno", "petsc"))
    arguments = parser.parse_args()
    print(prepare(arguments.kind, cache.compiler_env(dict(os.environ))))


if __name__ == "__main__":
    main()
