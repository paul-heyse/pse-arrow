# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Execute/discover exact manifest selections through existing native test runners."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
import sys
import unittest
from pathlib import Path

from scripts import implementation_phase as phase
from scripts import validation, validation_receipts

ROOT = Path(__file__).resolve().parents[1]


def expected(declaration: dict, profile: str) -> list[dict]:
    return sorted(
        {
            (c["source"] if c["runner"] == "pytest" else c["binary"], t)
            for c in declaration["cases"]
            if c["profile"] == profile and not c.get("artifact")
            for t in c["tests"]
        }
    )


def verify_selection(declaration: dict, profile: str, selected: list[dict]) -> None:
    observed = [
        (r.get("nodeid", "").partition("::")[0], r["nodeid"])
        if r.get("nodeid")
        else (r["class"], r["name"])
        for r in selected
    ]
    if len(observed) != len(set(observed)) or set(observed) != set(
        expected(declaration, profile)
    ):
        raise ValueError(
            f"{profile}: exact discovery mismatch; missing={sorted(set(expected(declaration, profile)) - set(observed))}, extra={sorted(set(observed) - set(expected(declaration, profile)))}"
        )


def command(
    declaration: dict, profile: dict, action: str, extra: list[str]
) -> list[str]:
    if profile["runner"] == "nextest":
        clauses = [
            f"(package(={c['package']}) & test(={t}))"
            for c in declaration["cases"]
            if c["profile"] == profile["id"]
            for t in c["tests"]
        ]
        result = [
            "cargo",
            "nextest",
            *shlex.split(
                "list --message-format json"
                if action == "discover"
                else os.environ.get("PSE_NEXTEST_ACTION", "run --no-fail-fast")
            ),
            "--locked",
            *profile["target_args"],
            "--features",
            ",".join(profile["features"]),
        ]
        for package in profile["packages"]:
            result.extend(["-p", package])
        return [*result, "-E", " | ".join(clauses), *extra]
    if profile["runner"] == "pytest":
        return [
            sys.executable,
            "-m",
            "pytest",
            "--collect-only" if action == "discover" else "--strict-markers",
            "-q",
            *sorted(
                {
                    t
                    for c in declaration["cases"]
                    if c["profile"] == profile["id"]
                    for t in c["tests"]
                }
            ),
            *extra,
        ]
    raise ValueError("not a test runner profile")


def native_provenance(profile: dict, binaries: list[str]) -> dict:
    import hashlib
    import time

    def digest(path):
        with Path(path).open("rb") as stream:
            return hashlib.file_digest(stream, "sha256").hexdigest()

    files = {}
    links = {}
    for name in binaries:
        path = Path(name).resolve()
        files[str(path)] = digest(path)
        linked = subprocess.check_output(["ldd", str(path)], text=True)
        if "not found" in linked:
            raise ValueError("native dependency is not linked")
        links[str(path)] = linked
        for line in linked.splitlines():
            parts = line.split()
            targets = [part for part in parts if part.startswith("/")]
            if targets:
                library = Path(targets[0]).resolve()
                files[str(library)] = digest(library)
    if not files:
        raise ValueError("no actual native binaries to bind")
    return {
        "schema": "plan14-native-profile-v1",
        "profile": profile,
        "captured": time.time(),
        "files": files,
        "links": links,
        "toolchain": subprocess.check_output(["rustc", "-Vv"], text=True),
        "threads": {
            k: os.environ.get(k)
            for k in ["OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"]
        },
    }


def tools(output: Path, declaration: dict | None = None) -> int:
    import xmlrunner

    output.mkdir(parents=True, exist_ok=True)
    declaration = declaration or phase.manifest(ROOT)
    names = sorted(
        {
            f"{c['binary']}.{t}"
            for c in declaration["cases"]
            if c["profile"] == "python-tools"
            for t in c["tests"]
        }
    )
    suite = unittest.defaultTestLoader.loadTestsFromNames(names)
    with (output / "plan14-tools.xml").open("wb") as report:
        result = xmlrunner.XMLTestRunner(output=report).run(suite)
    return 0 if result.wasSuccessful() else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["run", "discover", "tools", "development"])
    parser.add_argument("profile")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--unit-only", action="store_true")
    args, extra = parser.parse_known_args()
    declaration = phase.manifest(ROOT)
    phase.validate_sources(ROOT, declaration)
    if args.action == "development":
        from scripts.validation_scope import development

        output = validation.fresh_output(ROOT, Path(args.profile))
        code = validation.run_gates(ROOT, output, development(), phase="development")
        if code == 0:
            phase.validate_development(ROOT, declaration, output)
            pointer = phase.relative(ROOT, declaration["development"])
            pointer.parent.mkdir(parents=True, exist_ok=True)
            validation.write_json(
                pointer,
                {
                    "receipt": str((output / "checks.json").relative_to(ROOT)),
                    "digest": validation_receipts.digest(output / "checks.json"),
                },
            )
        return code
    if args.unit_only:
        declaration = {
            **declaration,
            "cases": [c for c in declaration["cases"] if c["class"] == "unit"],
        }
    if args.action == "tools":
        return tools(Path(args.profile), declaration)
    profile = next(p for p in declaration["profiles"] if p["id"] == args.profile)
    listing = args.action == "discover" or os.environ.get(
        "PSE_NEXTEST_ACTION", ""
    ).startswith("list ")
    if not listing and not args.unit_only:
        phase.guard(ROOT, [profile["gate"]])
    cmd = command(declaration, profile, "discover" if listing else "run", extra)
    provenance = os.environ.get("PSE_NATIVE_PROVENANCE")
    if not listing:
        if provenance:
            if profile["runner"] == "pytest":
                binaries = [str(p) for p in (ROOT / "python/pse").glob("_native*.so")]
            else:
                previous = json.loads(Path(provenance).read_text())
                binaries = list(previous["links"])
            validation.write_json(
                Path(provenance), native_provenance(profile, binaries)
            )
        return subprocess.call(cmd, cwd=ROOT)
    result = subprocess.run(cmd, cwd=ROOT, text=True, stdout=subprocess.PIPE)
    print(result.stdout, end="")
    if result.returncode:
        return result.returncode
    if profile["runner"] == "nextest":
        selected = validation_receipts.native_selection(result.stdout)
        if provenance:
            inventory = next(
                json.loads(line)
                for line in result.stdout.splitlines()
                if line.startswith("{") and "rust-suites" in line
            )
            binaries = [
                s["binary-path"]
                for s in inventory["rust-suites"].values()
                if any(
                    t["filter-match"]["status"] == "matches"
                    for t in s["testcases"].values()
                )
            ]
            validation.write_json(
                Path(provenance), native_provenance(profile, binaries)
            )
    else:
        selected = [
            {"nodeid": line}
            for line in result.stdout.splitlines()
            if line.startswith("python/") and "::" in line
        ]
    verify_selection(declaration, profile["id"], selected)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        validation.write_json(
            args.output,
            {
                "schema": "plan14-discovery-v1",
                "profile": profile,
                "command": cmd,
                "selected": selected,
                "source_files": validation.sources(ROOT),
            },
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
