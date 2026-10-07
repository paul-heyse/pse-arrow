# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Check a long command's known prerequisites before it starts, cheaply.

``scripts/preflight.py KIND... [--manifest PATH]`` checks, in the order given:

``native``
    The solver prefix native setup would use: an explicit ``IPOPT_DIR`` with the consumed
    files, an already extracted cache generation, or the pinned solver image present
    locally so setup can extract it. Nothing is pulled, extracted or built. KLU, the root
    isolation library, Uno and PETSc are built from repository sources by native setup and
    are not checked here.
``store``
    A serving canonical server at ``$PSE_SURREAL_STATE`` (``pse_env.require_store``).
``extension`` / ``native-extension``
    An installed ``pse`` extension, observed without importing it
    (``doctor.extension_kind``); ``native-extension`` requires the linked build.
``conformance``
    The ``--manifest`` file parses as TOML, declares runs, and every package root and
    diagnostics profile it names exists. ``pse.conformance.load_manifest`` remains the
    authoritative decode; this checks only the files a run would need.

The first missing prerequisite prints ``pse-env: prerequisite <kind>: ...`` with its fix and
exits 125, the environment boundary's status. Passing says only that these prerequisites
were present when observed: it does not prevent late code, fixture or scientific failures,
and a prerequisite can still disappear while the command runs.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
from scripts import (  # noqa: E402 -- direct-script path routing
    doctor,
    native_cache,
    pse_env,
)

BoundaryError = pse_env.BoundaryError


def native(env: Mapping[str, str], _manifest: Path | None) -> None:
    explicit = env.get("IPOPT_DIR")
    if explicit:
        missing = [
            name for name in native_cache.SOLVER_FILES if not (Path(explicit) / name).is_file()
        ]
        if missing:
            raise BoundaryError(
                f"prerequisite native: IPOPT_DIR={explicit} lacks {missing[0]}"
                f"{f' and {len(missing) - 1} more' if len(missing) > 1 else ''}; "
                "point it at a complete solver prefix or unset it to use the pinned image"
            )
        return
    if native_cache.prepared_solver(native_cache.cache_root(dict(env))) is not None:
        return
    try:
        image = native_cache.solver_image()
    except ValueError as error:
        raise BoundaryError(
            f"prerequisite native: cannot resolve the solver image ({error}); "
            "check PSE_SOLVER_IMAGE or run just bootstrap-solvers"
        ) from error
    docker = shutil.which("docker", path=env.get("PATH"))
    if docker is None:
        raise BoundaryError(
            "prerequisite native: the solver prefix is not extracted and docker is not "
            "available to extract it; install docker and run just bootstrap-solvers"
        )
    inspected = subprocess.run(
        [docker, "image", "inspect", image],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
        timeout=60,
    )
    if inspected.returncode != 0:
        raise BoundaryError(
            f"prerequisite native: solver image {image} is not present locally; "
            "run just bootstrap-solvers (or set PSE_SOLVER_IMAGE)"
        )


def store(env: Mapping[str, str], _manifest: Path | None) -> None:
    try:
        pse_env.require_store(env)
    except BoundaryError as error:
        raise BoundaryError(f"prerequisite store: {error}") from error


def extension(env: Mapping[str, str], _manifest: Path | None) -> None:
    del env
    if doctor.extension_kind() == "absent":
        raise BoundaryError(
            "prerequisite extension: no installed pse extension; "
            "run just py-sync (or just py-sync-native for the native solvers)"
        )


def native_extension(env: Mapping[str, str], _manifest: Path | None) -> None:
    del env
    kind = doctor.extension_kind()
    if kind != "native":
        found = "no installed pse extension" if kind == "absent" else "the dev build"
        raise BoundaryError(
            f"prerequisite native-extension: the command needs the linked native build, "
            f"found {found}; run just py-sync-native"
        )


def conformance(env: Mapping[str, str], manifest: Path | None) -> None:
    del env
    if manifest is None:
        raise BoundaryError("prerequisite conformance: no --manifest given")
    path = manifest if manifest.is_absolute() else ROOT / manifest
    try:
        document = tomllib.loads(path.read_text())
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise BoundaryError(f"prerequisite conformance: cannot read {path}: {error}") from error
    runs = document.get("runs")
    if not isinstance(runs, list) or not runs:
        raise BoundaryError(f"prerequisite conformance: {path} declares no [[runs]]")
    for run in runs:
        if not isinstance(run, dict):
            raise BoundaryError(f"prerequisite conformance: {path} has a malformed run")
        roots = [run.get("package"), run.get("physical"), *run.get("dependencies", [])]
        for root in roots:
            if not isinstance(root, str) or not (path.parent / root / "package.toml").is_file():
                raise BoundaryError(
                    f"prerequisite conformance: run {run.get('name')!r} in {path} names "
                    f"{root!r}, which has no package.toml"
                )
    diagnostics = document.get("settings", {}).get("diagnostics")
    if isinstance(diagnostics, str) and not (path.parent / diagnostics).is_file():
        raise BoundaryError(
            f"prerequisite conformance: {path} names diagnostics {diagnostics}, "
            "which is not a file"
        )


KINDS: dict[str, Callable[[Mapping[str, str], Path | None], None]] = {
    "native": native,
    "store": store,
    "extension": extension,
    "native-extension": native_extension,
    "conformance": conformance,
}


def check(kinds: Sequence[str], env: Mapping[str, str], manifest: Path | None = None) -> None:
    """Raise ``BoundaryError`` for the first missing prerequisite, in the order given."""
    for kind in kinds:
        KINDS[kind](env, manifest)


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="preflight",
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("kinds", nargs="+", choices=sorted(KINDS), metavar="KIND")
    parser.add_argument("--manifest", type=Path, help="the conformance manifest to check")
    args = parser.parse_args(argv)
    if os.environ.get("PSE_PREFLIGHT") == "off":
        # The caller chose to proceed without the early check (e.g. partial evidence).
        print(f"preflight: skipped {' '.join(args.kinds)} (PSE_PREFLIGHT=off)", file=sys.stderr)
        return 0
    try:
        check(args.kinds, pse_env.compose(ROOT, os.environ), args.manifest)
    except BoundaryError as error:
        print(f"pse-env: {error}", file=sys.stderr)
        return pse_env.FAILURE
    except Exception as error:  # an unobservable prerequisite is not admitted
        print(f"pse-env: preflight could not observe {args.kinds}: {type(error).__name__}: {error}", file=sys.stderr)
        return pse_env.FAILURE
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
