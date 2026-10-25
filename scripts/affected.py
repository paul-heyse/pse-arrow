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
import tomllib
from collections.abc import Sequence
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


def changed_files(base: str, *, root: Path = ROOT) -> list[tuple[str, str]]:
    diff = subprocess.run(
        ["git", "diff", "--name-status", "--no-renames", base],
        cwd=root,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    entries = [tuple(line.split("\t", 1)) for line in diff.splitlines() if "\t" in line]
    untracked = subprocess.run(
        ["git", "ls-files", "--others", "--exclude-standard"],
        cwd=root,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    entries += [("?", path) for path in untracked.splitlines()]
    return [(status, path) for status, path in entries]


def workspace_packages(*, root: Path = ROOT) -> tuple[dict[str, Path], dict[str, set[str]]]:
    metadata = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"],
            cwd=root,
            capture_output=True,
            text=True,
            check=True,
            timeout=10,
        ).stdout
    )
    roots = {
        package["name"]: Path(package["manifest_path"]).parent.relative_to(root)
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


def selected_package_roots(changes: Sequence[tuple[str, str]], *, root: Path) -> dict[str, Path]:
    """Nearest authored package roots only; never a replacement dependency resolver."""
    roots = {}
    for _status, name in changes:
        parent = (root / name).parent
        while parent != root and parent.is_relative_to(root):
            manifest = parent / "Cargo.toml"
            if manifest.is_file() and not manifest.is_symlink():
                if manifest.stat().st_size > 64 << 10:
                    break
                try:
                    package = tomllib.loads(manifest.read_text()).get("package", {}).get("name")
                    if isinstance(package, str):
                        roots[package] = parent.relative_to(root)
                except (OSError, ValueError, UnicodeDecodeError):
                    pass
                break
            parent = parent.parent
    return roots


def preview(
    changes: Sequence[tuple[str, str]], *, root: Path = ROOT, base: str = "HEAD",
    features: str = "", extra: Sequence[str] = (), coverage: Path | None = None,
    prepare: bool = True,
) -> dict:
    """Original affected-selection meaning for an explicit change population.

    prepare=False performs no compile, scientific capture or test discovery. Cargo
    metadata is a read-only dependency observation; the proposed Arrow route is
    admitted by the original receiver only when the caller explicitly executes it.
    """
    for _status, path in changes:
        if Path(path).is_absolute() or ".." in Path(path).parts:
            raise ValueError("Change paths must remain inside the selected checkout")
    if not changes and not prepare:
        return {
            "schema": "affected-preview-v1", "base": base, "changes": [],
            "changed_packages": [], "selected_packages": [], "workspace_widening": [],
            "scientific_inputs": {"consumers": [], "families": [], "unknown": []},
            "command": None, "graph_status": "not-needed", "graph_error": None,
            "validation": "no selected changes", "exclusions": [],
            "qualification": "empty selected change population; no test execution",
        }
    graph_status = "available"
    graph_error = None
    try:
        roots, dependents = workspace_packages(root=root)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        if prepare:
            raise
        graph_status = "unavailable"
        graph_error = str(error)
        roots = selected_package_roots(changes, root=root)
        dependents = {name: set() for name in roots}
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
    # selected verification owner; context discovery remains source-only
    from scripts import scientific_inputs  # noqa: PLC0415

    try:
        scientific = scientific_inputs.affected(changes, root=root, prepare=prepare)
    except (OSError, ValueError, TypeError, KeyError) as error:
        scientific = {
            "consumers": sorted(roots),
            "families": [],
            "unknown": [path for _, path in changes],
            "reason": str(error),
        }
    seeds.update(name for name in scientific["consumers"] if name in roots)
    selected = sorted(closure(seeds, dependents)) if not wide else sorted(roots)
    command = ["cargo", "nextest", "run", "--locked"]
    if features:
        command += ["--features", features]
    if wide:
        command.append("--workspace")
    else:
        for name in selected:
            command += ["-p", name]
        if seeds:
            filterset = " | ".join(f"rdeps({name})" for name in sorted(seeds))
            command += ["-E", filterset]
    command += list(extra)
    if (seeds or wide) and graph_status == "available":
        if prepare:
            command = compose(command)
        else:
            command = ["scripts/pse-env", "--", ".venv/bin/python", "-m", "scripts.arrow_validation", *command[1:]]
    native = sorted(NATIVE_PACKAGES & set(selected))
    exclusions = [
        {
            "kind": "feature-gated-native",
            "packages": native,
            "route": "--features " + NATIVE_FEATURES,
        },
        {
            "kind": "python",
            "files": python,
            "route": "just py-unit; just py-test for components",
        },
        {
            "kind": "generator",
            "files": generators,
            "route": "just codegen; just codegen-check at scope end",
        },
        {"kind": "unowned", "files": unowned},
        {
            "kind": "dynamic",
            "meaning": "unknown dynamic dependencies broaden scientific-family reach; other external-process reach is not inferred",
        },
    ]
    if graph_status != "available":
        exclusions.append({"kind": "workspace-graph", "meaning": "Package roots are source leads only; reverse dependency closure is unavailable.", "reason": graph_error})
    result = {
        "schema": "affected-preview-v1",
        "base": base,
        "changes": changes,
        "changed_packages": sorted(seeds),
        "scientific_inputs": scientific,
        "selected_packages": selected,
        "workspace_widening": wide,
        "command": command if (seeds or wide) and graph_status == "available" else None,
        "graph_status": graph_status,
        "graph_error": graph_error,
        "validation": "composed Arrow closure" if prepare else "proposed route; Arrow closure admitted by its original receiver at execution",
        "exclusions": exclusions,
        "qualification": "dependency reach only; not exhaustive behavior coverage",
    }
    if coverage is not None:
        # selected verification owner; context discovery remains source-only
        from scripts.coverage_correspondence import inspect  # noqa: PLC0415

        try:
            result["coverage"] = inspect(coverage, base)
        except (
            OSError,
            ValueError,
            KeyError,
            TypeError,
            IndexError,
            subprocess.SubprocessError,
        ) as error:
            result["coverage"] = {
                "status": "unavailable",
                "error": str(error),
                "selection": "conservative dependency reach remains authoritative; coverage never omits tests",
            }
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--base", default="HEAD", help="compare the working tree to REF"
    )
    parser.add_argument("--run", action="store_true", help="run the selection")
    parser.add_argument("--passive", action="store_true", help="do not prepare scientific capture or compose a live Arrow unit graph")
    parser.add_argument("--changes-json", help="explicit selected list of [status, checkout-relative path]; default remains current whole-worktree changes")
    parser.add_argument(
        "--json",
        action="store_true",
        help="structured preview with the same selection and exclusions",
    )
    parser.add_argument(
        "--coverage",
        type=Path,
        help="optional source-fresh coverage capture; never narrows dependency selection",
    )
    parser.add_argument("--features", default="", help="extra cargo features")
    parser.add_argument("nextest", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    extra = args.nextest[1:] if args.nextest[:1] == ["--"] else args.nextest
    changes = changed_files(args.base) if args.changes_json is None else json.loads(args.changes_json)
    if not isinstance(changes, list) or any(not isinstance(row, (list, tuple)) or len(row) != 2 or not all(isinstance(value, str) for value in row) for row in changes):
        parser.error("--changes-json requires a list of [status, path] string pairs")
    result = preview(changes, base=args.base, features=args.features, extra=extra, coverage=args.coverage, prepare=not args.passive)
    seeds = result["changed_packages"]
    wide = result["workspace_widening"]
    selected = result["selected_packages"]
    command = result["command"]
    exclusions = result["exclusions"]
    if args.json:
        print(json.dumps(result, sort_keys=True))
    else:
        print(f"affected: {len(changes)} changed file(s) against {args.base}")
        print("affected: changed packages: " + (", ".join(sorted(seeds)) or "none"))
        print(f"affected: selected packages: {len(selected)}")
        if wide:
            print(
                "affected: root configuration widens to workspace: "
                + ", ".join(wide[:5])
            )
        print("not covered by this selection:")
        for exclusion in exclusions:
            print("  - " + json.dumps(exclusion))
        print("command: " + (shlex.join(command) if command else "none; selected graph is unavailable" if seeds or wide else "none"))
    if args.coverage is not None and not args.json:
        print(json.dumps(result["coverage"], indent=2))
    if not args.run or (not seeds and not wide):
        return 0
    runner = [str(ROOT / "scripts/pse-env")]
    if args.features:
        runner.append("--native")
    if args.passive:
        command = preview(changes, base=args.base, features=args.features, extra=extra)["command"]
    os.execv(runner[0], [*runner, "--", *command])  # noqa: S606 -- the preview becomes the run
    return 127


if __name__ == "__main__":
    sys.exit(main())
