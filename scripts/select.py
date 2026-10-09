# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Build and run one cargo/nextest selection the way the unit recipes mean it.

``select.py unit <package> <filter> [args...]`` runs ``cargo nextest`` for the package with
Arrow force-validation. Force-validation needs ``pse-relations`` in Cargo's package
selection, so the test filter is intersected with ``package(...)`` over the final package
set (the named package plus any ``-p``/``--package`` in ``args``): pse-relations' own
tests are never selected by accident, and an explicit ``-p pse-relations`` still selects
them. A filter that is a bare test-path word (``numerics``, ``a::b``) becomes
``test(word)``; every other expression passes through unchanged. ``workspace`` selects
across the workspace's libraries with the same filter rule and no package intersection.

The effective command is printed before it runs. An empty selection still fails, as
nextest decides. ``PSE_NEXTEST_ACTION`` (default ``run --no-fail-fast``) chooses the
action, e.g. ``list`` to see the selection without running it.
"""

from __future__ import annotations

import os
import re
import shlex
import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if not __package__:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from scripts.test_run import run_rust

if TYPE_CHECKING:
    from collections.abc import Sequence

VALIDATE = ("--features", "pse-relations/force-validate")
BARE = re.compile(r"[A-Za-z0-9_]+(?:::[A-Za-z0-9_]+)*(?:::)?")


def test_filter(expression: str) -> str:
    return f"test({expression})" if BARE.fullmatch(expression) else expression


def packages(args: Sequence[str]) -> list[str]:
    found = []
    items = iter(args)
    for item in items:
        if item in ("-p", "--package"):
            value = next(items, None)
            if value is not None:
                found.append(value)
        elif item.startswith("--package="):
            found.append(item.split("=", 1)[1])
        elif item.startswith("-p") and len(item) > 2:
            found.append(item[2:])
    return found


def nextest_action() -> list[str]:
    return shlex.split(os.environ.get("PSE_NEXTEST_ACTION", "run --no-fail-fast"))


def unit(
    package: str, expression: str, args: Sequence[str], *, features: Sequence[str] = ()
) -> list[str]:
    selected = [package, *packages(args)]
    scope = " | ".join(f"package({name})" for name in dict.fromkeys(selected))
    joined = ",".join(["pse-relations/force-validate", *features])
    return [
        "cargo",
        "nextest",
        *nextest_action(),
        "-p",
        package,
        "-p",
        "pse-relations",
        "--lib",
        "--locked",
        "--features",
        joined,
        "-E",
        f"({test_filter(expression)}) & ({scope})",
        *args,
    ]


def workspace(
    expression: str, args: Sequence[str], *, features: Sequence[str] = ()
) -> list[str]:
    joined = ",".join(["pse-relations/force-validate", *features])
    return [
        "cargo",
        "nextest",
        *nextest_action(),
        "--workspace",
        "--lib",
        "--locked",
        "--features",
        joined,
        "-E",
        test_filter(expression),
        *args,
    ]


def main(argv: Sequence[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    features: list[str] = []
    while argv[:1] == ["--features"]:
        features.extend(item for item in argv[1].split(",") if item)
        argv = argv[2:]
    if argv[:1] == ["unit"] and len(argv) >= 3:
        command = unit(argv[1], argv[2], argv[3:], features=features)
    elif argv[:1] == ["workspace"] and len(argv) >= 2:
        command = workspace(argv[1], argv[2:], features=features)
    else:
        print(
            "usage: select.py [--features F] unit <package> <filter> [args...] | "
            "workspace <filter> [args...]",
            file=sys.stderr,
        )
        return 2
    print("select: " + shlex.join(command), file=sys.stderr, flush=True)
    if "run" not in command:
        return subprocess.call(command)

    return run_rust(command)


if __name__ == "__main__":
    raise SystemExit(main())
