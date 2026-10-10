# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Build and run one cargo/nextest selection the way the unit recipes mean it.

``select.py unit <package> <filter> [args...]`` runs ``cargo nextest`` for the package with
scoped Arrow force-validation without adding unrelated roots. The test filter is
intersected with ``package(...)`` over the requested package set. A filter that is a
bare test-path word (``numerics``, ``a::b``) becomes
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

from scripts.arrow_validation import resolve
from scripts.test_run import run_rust

if TYPE_CHECKING:
    from collections.abc import Sequence

BARE = re.compile(r"[A-Za-z0-9_]+(?:::[A-Za-z0-9_]+)*(?:::)?")


def test_filter(expression: str) -> str:
    return f"test({expression})" if BARE.fullmatch(expression) else expression


def nextest_action() -> list[str]:
    return shlex.split(os.environ.get("PSE_NEXTEST_ACTION", "run --no-fail-fast"))


def unit(
    package: str, expression: str, args: Sequence[str], *, features: Sequence[str] = ()
) -> list[str]:
    joined = ",".join(features)
    return [
        "cargo",
        "nextest",
        *nextest_action(),
        "-p",
        package,
        "--lib",
        "--locked",
        *(["--features", joined] if joined else []),
        "-E",
        test_filter(expression),
        *args,
    ]


def workspace(
    expression: str, args: Sequence[str], *, features: Sequence[str] = ()
) -> list[str]:
    joined = ",".join(features)
    return [
        "cargo",
        "nextest",
        *nextest_action(),
        "--workspace",
        "--lib",
        "--locked",
        *(["--features", joined] if joined else []),
        "-E",
        test_filter(expression),
        *args,
    ]


def main(argv: Sequence[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    effects = "canonical"
    # This is an explicit invocation requirement, independent of test names,
    # native capabilities and the word `unit`. Extra recipe args may override it.
    index = 0
    while index < len(argv) and argv[index] != "--":
        if argv[index] == "--execution-effects":
            if index + 1 == len(argv) or argv[index + 1] not in {
                "canonical",
                "native-local",
            }:
                raise ValueError(
                    "--execution-effects requires canonical or native-local"
                )
            effects = argv[index + 1]
            del argv[index : index + 2]
        else:
            index += 1
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
    resolved = resolve(command)
    command = resolved.command
    if argv[:1] == ["unit"]:
        position = command.index("-E") + 1
        scope = " | ".join(f"package({name})" for name in resolved.packages)
        command[position] = f"({command[position]}) & ({scope})"
    print("select: " + shlex.join(command), file=sys.stderr, flush=True)
    if "run" not in command:
        return subprocess.call(command)

    return run_rust(command, effects=effects)


if __name__ == "__main__":
    raise SystemExit(main())
