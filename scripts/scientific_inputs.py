# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Invocation preparation and conservative reach for authored scientific families.

The Rust document reader, parser and P0 resolver own dependency interpretation.
This adapter supplies paths and forwards their exact captured facts.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import tempfile
import time
import tomllib
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping, MutableMapping, Sequence

ROOT = Path(__file__).resolve().parents[1]
CATALOG = Path("tests/fixtures/scientific-inputs.toml")
ROLE = "PSE_SCIENTIFIC_INPUTS"
# Reviewed test owners that actually read the immutable capsule. Package-level
# consumers below still own family/change reach, not demand from every unit test.
# Other packages retain conservative package demand until their owners are reviewed.
RUST_TEST_OWNERS = {
    "pse-runtime": (
        "workflow::modeling::forms_tests::",
        "workflow::modeling::conformance::",
        "workflow::strategies::conditional::reference_fixture::",
    ),
}
SHARED_INPUTS = {
    str(CATALOG),
    "tests/fixtures/scientific-inputs.rs",
    "tests/fixtures/reference-source-fixture.rs",
    "scripts/scientific_inputs.py",
}


def catalog(root: Path = ROOT) -> dict:
    return tomllib.loads((root / CATALOG).read_text())


def capture(
    destination: Path, environment: Mapping[str, str], *, root: Path = ROOT
) -> Path:
    """Capture once at the invocation boundary; children only read the capsule."""
    destination = destination.absolute()
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.exists():
        raise ValueError("scientific capture destination already exists")
    subprocess.run(
        [
            "cargo",
            "run",
            "--locked",
            "-p",
            "xtask",
            "--features",
            "physical-admission,force-validate",
            "--",
            "scientific-inputs",
            str(root / CATALOG),
            str(destination),
        ],
        cwd=root,
        env=dict(environment),
        check=True,
    )
    read(destination)
    return destination


def read(path: Path) -> dict:
    value = json.loads(path.read_bytes())
    if value.get("schema") != "scientific-inputs-v1":
        raise ValueError("unknown scientific input capsule")
    if not isinstance(value.get("files"), dict) or not isinstance(
        value.get("families"), dict
    ):
        raise TypeError("scientific capsule requires files and families")
    return value


def selected_consumers(inventory: dict, *, root: Path = ROOT) -> set[str]:
    consumers = {
        consumer for f in catalog(root)["families"] for consumer in f["consumers"]
    }
    return {
        suite["binary-id"].split("::", 1)[0]
        for suite in inventory["rust-suites"].values()
        if suite["binary-id"].split("::", 1)[0] in consumers
        and any(
            test.get("filter-match", {}).get("status") == "matches"
            and (
                (owners := RUST_TEST_OWNERS.get(suite["binary-id"].split("::", 1)[0]))
                is None
                or name.startswith(owners)
            )
            for name, test in suite["testcases"].items()
        )
    }


def prepare(
    inventory: dict, destination: Path, environment: dict[str, str]
) -> dict[str, str]:
    if not selected_consumers(inventory):
        return {}
    return prepare_controller(destination, environment)


def prepare_controller(
    destination: Path | None, environment: MutableMapping[str, str]
) -> dict[str, str]:
    """Bind one validated capsule before a controller starts any consumers."""
    # A caller may explicitly supply a retained immutable capsule. Never infer one
    # from a cache directory or read mutable sources inside an individual test.
    supplied = environment.get(ROLE)
    path = (
        Path(supplied).resolve(strict=True)
        if supplied
        else capture(
            destination
            if destination is not None
            else ROOT / "build/scientific-inputs" / (str(time.time_ns()) + ".json"),
            environment,
        )
    )
    read(path)
    environment[ROLE] = str(path)
    return {ROLE: str(path)}


def prepare_python(
    arguments: Sequence[str], environment: MutableMapping[str, str]
) -> None:
    """Prepare in the pytest controller before workers start, including bare pytest."""
    # selected input preparation owner; import must not prepare capabilities
    from scripts import test_invocation  # noqa: PLC0415

    request = test_invocation.python(arguments)
    paths = [Path(arg.split("::", 1)[0]).resolve() for arg in request.targets]
    consumers = [
        ROOT / "python/pse/parity",
        *(
            ROOT / "python/pse/tests" / name
            for name in (
                "test_studies.py",
                "test_plan14_acceptance.py",
                "test_modeling_kernel.py",
            )
        ),
    ]
    relevant = (
        not paths
        or "--parity" in arguments
        or any(
            path == consumer or path in consumer.parents or consumers[0] in path.parents
            for path in paths
            for consumer in consumers
        )
    )
    if not relevant:
        return
    prepare_controller(None, environment)


def affected(changes: Sequence[tuple[str, str]], *, root: Path = ROOT) -> dict:
    declaration = catalog(root)
    inventory = str(declaration["package_inventory"]).rstrip("/") + "/"
    relevant = [
        path
        for _, path in changes
        if path.startswith((inventory, "tests/fixtures/scientific/"))
        or path in SHARED_INPUTS
    ]
    if not relevant:
        return {"consumers": [], "families": [], "unknown": []}
    all_consumers = sorted({c for f in declaration["families"] for c in f["consumers"]})
    try:
        with tempfile.TemporaryDirectory(prefix="pse-scientific-reach-") as temporary:
            value = read(
                capture(Path(temporary) / "inputs.json", os.environ, root=root)
            )
        selected = []
        consumers = set()
        known = set(value["files"])
        for family in declaration["families"]:
            facts = value["families"][family["name"]]
            prefixes = tuple(package.rstrip("/") + "/" for package in facts["packages"])
            if any(
                path in SHARED_INPUTS
                or path in facts.get("substitutions", [])
                or path in facts.get("documents", [])
                or path.startswith(prefixes)
                for path in relevant
            ):
                selected.append(family["name"])
                consumers.update(family["consumers"])
        # New/deleted members, package manifests and unresolved imports may change
        # closure membership, including a formerly absent lookup. Widen, never prune.
        unknown = [
            path
            for path in relevant
            if path not in known or path.endswith("package.toml")
        ]
        if unknown:
            consumers.update(all_consumers)
        return {
            "consumers": sorted(consumers),
            "families": selected,
            "unknown": unknown,
        }
    except (
        OSError,
        ValueError,
        TypeError,
        KeyError,
        subprocess.SubprocessError,
    ) as error:
        return {
            "consumers": all_consumers,
            "families": [],
            "unknown": relevant,
            "reason": str(error),
        }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    capture_parser = commands.add_parser("capture")
    capture_parser.add_argument("output", type=Path)
    run_parser = commands.add_parser("run")
    run_parser.add_argument("arguments", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.command == "capture":
        print(capture(args.output, os.environ))
        return 0
    arguments = args.arguments
    if arguments[:1] == ["--"]:
        arguments = arguments[1:]
    if not arguments:
        parser.error("run requires a consumer command")
    environment = dict(os.environ)
    prepare_controller(None, environment)
    return subprocess.run(arguments, env=environment, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
