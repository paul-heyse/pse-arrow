# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Check N06 ownership from resolved Cargo metadata without executing product code."""

import json
import sys
from pathlib import Path
from typing import TypedDict, cast


class Dependency(TypedDict):
    """Cargo manifest dependency fields used by this check."""

    name: str
    kind: str | None


class Package(TypedDict):
    """Cargo package identity and declared dependency edges."""

    id: str
    name: str
    dependencies: list[Dependency]


class Kind(TypedDict):
    """One resolved dependency edge kind."""

    kind: str | None


class Edge(TypedDict):
    """Resolved destination and its actual edge kinds."""

    pkg: str
    dep_kinds: list[Kind]


class Node(TypedDict):
    """Cargo's resolved dependency node."""

    id: str
    deps: list[Edge]


class Resolve(TypedDict):
    """Resolved graph supplied by cargo metadata."""

    nodes: list[Node]


class Metadata(TypedDict):
    """Selected fields of Cargo's version-one metadata contract."""

    packages: list[Package]
    resolve: Resolve
    workspace_root: str
    workspace_members: list[str]


def check(metadata: Metadata) -> list[str]:
    """Return independent dependency and deletion violations."""
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    by_name = {package["name"]: key for key, package in packages.items()}
    root = Path(metadata["workspace_root"])
    failures = []

    def runtime_dependencies(start: str) -> set[str]:
        visited = set()
        pending = [by_name[start]]
        while pending:
            key = pending.pop()
            if key in visited:
                continue
            visited.add(key)
            for edge in nodes[key]["deps"]:
                if any(kind["kind"] != "dev" for kind in edge["dep_kinds"]):
                    pending.append(edge["pkg"])
        return {packages[key]["name"] for key in visited}

    for crate in ("pse-engine", "pse-testkit"):
        reachable = runtime_dependencies(crate)
        forbidden = {
            name
            for name in reachable
            if name in {"pse-catalog", "pse-runtime", "pse-compiler"}
            or name.startswith(("deltalake", "pse-backend", "pse-ipopt"))
        }
        if forbidden:
            failures.append(
                f"{crate} reaches forbidden product/storage dependencies: {sorted(forbidden)}"
            )
    for key in metadata["workspace_members"]:
        for dependency in packages[key]["dependencies"]:
            if dependency["name"] == "pse-testkit" and dependency["kind"] != "dev":
                failures.append(
                    f"{packages[key]['name']} has a production/build testkit edge"
                )
    for name in (
        "pse-engine",
        "pse-ids",
        "pse-diagnostics",
        "pse-schema",
        "pse-relations",
    ):
        if any(
            dep["name"] == "pse-testkit"
            for dep in packages[by_name[name]]["dependencies"]
        ):
            failures.append(f"{name} creates a testkit dependency cycle")
    for relative in (
        "crates/pse-catalog/src/session",
        "crates/pse-catalog/src/provider",
        "crates/pse-catalog/src/error.rs",
        "crates/pse-runtime/src/reserve.rs",
        "tests/support/session_factory.rs",
    ):
        if (root / relative).exists():
            failures.append(f"superseded owner remains: {relative}")
    return failures


def main() -> bool:
    """Read the recipe's native metadata and report a zero-baseline static result."""
    failures = check(cast("Metadata", json.load(sys.stdin)))
    for failure in failures:
        print(f"FAIL: {failure}")
    print(f"N06 dependency/deletion checks: {len(failures)} failures; baseline 0")
    return bool(failures)


if __name__ == "__main__":
    sys.exit(main())
