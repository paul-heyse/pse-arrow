# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Preview, and on request run, the Rust tests a change can reach.

``just affected [--base REF] [--run] [--features F] [-- nextest args]`` compares the
working tree (staged, unstaged, deleted and untracked files) against REF (default HEAD),
maps changed files to workspace packages, and selects their reverse dependencies with a
nextest ``rdeps()`` filterset under Arrow force-validation. It always prints what it can
NOT establish: a dependency selector is not proof that every affected behaviour is tested.
Root build configuration widens the selection to the workspace; Python, generator and
registry inputs are reported with the command that covers them; native packages suggest
the native feature set, which is only used when given with ``--features``.
"""

from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if not __package__:
    sys.path.insert(0, str(ROOT))
from scripts.arrow_validation import compose  # noqa: E402 -- script bootstrap

WORKSPACE_WIDE = (
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "clippy.toml",
    ".cargo/",
    ".config/nextest.toml",
)
GENERATOR_INPUTS = ("xtask/src/codegen", "crates/pse-codegen/", "crates/pse-schema/")
NATIVE_PACKAGES = {
    "pse-backend-native",
    "pse-runtime",
    "pse-ipopt-sys",
    "pse-uno-sys",
    "pse-petsc-sys",
    "pse-py",
}
NATIVE_FEATURES = "pse-runtime/native-solvers"


def changed_files(base: str) -> list[tuple[str, str]]:
    diff = subprocess.run(
        ["git", "diff", "--name-status", "--no-renames", base],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    entries = [tuple(line.split("\t", 1)) for line in diff.splitlines() if "\t" in line]
    untracked = subprocess.run(
        ["git", "ls-files", "--others", "--exclude-standard"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    entries += [("?", path) for path in untracked.splitlines()]
    return [(status, path) for status, path in entries]


def workspace_packages() -> tuple[dict[str, Path], dict[str, set[str]]]:
    metadata = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    )
    roots = {
        package["name"]: Path(package["manifest_path"]).parent.relative_to(ROOT)
        for package in metadata["packages"]
    }
    dependents: dict[str, set[str]] = {name: set() for name in roots}
    for package in metadata["packages"]:
        for dependency in package["dependencies"]:
            if dependency["name"] in roots:
                dependents[dependency["name"]].add(package["name"])
    return roots, dependents


def owner(path: str, roots: dict[str, Path]) -> str | None:
    candidates = [
        (len(root.parts), name)
        for name, root in roots.items()
        if root.parts and Path(path).parts[: len(root.parts)] == root.parts
    ]
    return max(candidates)[1] if candidates else None


def closure(seeds: set[str], dependents: dict[str, set[str]]) -> set[str]:
    found = set(seeds)
    frontier = list(seeds)
    while frontier:
        for dependent in dependents.get(frontier.pop(), ()):
            if dependent not in found:
                found.add(dependent)
                frontier.append(dependent)
    return found


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--base", default="HEAD", help="compare the working tree to REF"
    )
    parser.add_argument("--run", action="store_true", help="run the selection")
    parser.add_argument("--json", action="store_true", help="structured preview with the same selection and exclusions")
    parser.add_argument("--features", default="", help="extra cargo features")
    parser.add_argument("nextest", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    extra = args.nextest[1:] if args.nextest[:1] == ["--"] else args.nextest
    changes = changed_files(args.base)
    roots, dependents = workspace_packages()
    seeds: set[str] = set()
    wide: list[str] = []
    python: list[str] = []
    generators: list[str] = []
    unowned: list[str] = []
    for status, path in changes:
        if path.startswith(WORKSPACE_WIDE) or path in WORKSPACE_WIDE:
            wide.append(path)
        if path.startswith(GENERATOR_INPUTS):
            generators.append(path)
        if path.startswith(("python/", "conftest.py", "pyproject.toml", "uv.lock")):
            python.append(path)
        name = owner(path, roots)
        if name is not None:
            seeds.add(name)
        elif not path.startswith(("python/", "docs/")) and path not in WORKSPACE_WIDE:
            unowned.append(f"{status} {path}")
    selected = sorted(closure(seeds, dependents)) if not wide else sorted(roots)
    command = ["cargo", "nextest", "run", "--locked"]
    if args.features:
        command += ["--features", args.features]
    if wide:
        command.append("--workspace")
    else:
        for name in selected:
            command += ["-p", name]
        if seeds:
            filterset = " | ".join(f"rdeps({name})" for name in sorted(seeds))
            command += ["-E", filterset]
    command += extra
    if seeds or wide:
        command = compose(command)
    native = sorted(NATIVE_PACKAGES & set(selected))
    exclusions = [
        {"kind": "feature-gated-native", "packages": native, "route": "--features " + NATIVE_FEATURES},
        {"kind": "python", "files": python, "route": "just py-unit; just py-test for components"},
        {"kind": "generator", "files": generators, "route": "just codegen; just codegen-check at scope end"},
        {"kind": "unowned", "files": unowned},
        {"kind": "dynamic", "meaning": "data, fixtures and external-process reach is not inferred"},
    ]
    result = {"schema": "affected-preview-v1", "base": args.base, "changes": changes,
              "changed_packages": sorted(seeds), "selected_packages": selected,
              "workspace_widening": wide, "command": command if seeds or wide else None,
              "exclusions": exclusions, "qualification": "dependency reach only; not exhaustive behavior coverage"}
    if args.json:
        print(json.dumps(result, sort_keys=True))
    else:
        print(f"affected: {len(changes)} changed file(s) against {args.base}")
        print("affected: changed packages: " + (", ".join(sorted(seeds)) or "none"))
        print(f"affected: selected packages: {len(selected)}")
        if wide:
            print("affected: root configuration widens to workspace: " + ", ".join(wide[:5]))
        print("not covered by this selection:")
        for exclusion in exclusions:
            print("  - " + json.dumps(exclusion))
        print("command: " + (shlex.join(command) if seeds or wide else "none"))
    if not args.run or (not seeds and not wide):
        return 0
    runner = [str(ROOT / "scripts/pse-env")]
    if args.features:
        runner.append("--native")
    os.execv(runner[0], [*runner, "--", *command])  # noqa: S606 -- the preview becomes the run
    return 127


if __name__ == "__main__":
    sys.exit(main())
