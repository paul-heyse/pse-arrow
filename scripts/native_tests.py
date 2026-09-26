# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Ordinary native test commands; selection belongs to nextest and pytest."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path

from scripts import validation, validation_receipts

ROOT = Path(__file__).resolve().parents[1]
FEATURES = "pse-runtime/native-solvers,pse-tests-conformance/native-acceptance,pse-relations/force-validate"


def native_provenance(profile: dict, binaries: list[str]) -> dict:

    def digest(path: Path) -> str:
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
                if str(library) not in files:
                    files[str(library)] = digest(library)
    if not files:
        raise ValueError("no actual native binaries to bind")
    return {
        "schema": "native-profile-v1",
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


def rust_command(action: str, extra: list[str]) -> list[str]:
    return [
        "cargo",
        "nextest",
        action,
        "--workspace",
        "--locked",
        "--features",
        FEATURES,
        *extra,
    ]


def main() -> int:
    kind, *extra = sys.argv[1:]
    path = os.environ.get("PSE_NATIVE_PROVENANCE")
    if kind == "python":
        report = next(
            (
                Path(arg.split("=", 1)[1])
                for arg in extra
                if arg.startswith("--junitxml=")
            ),
            None,
        )
        if report is None:
            raise ValueError("native Python execution requires a JUnit destination")
        if path:
            validation.write_json(
                Path(path),
                native_provenance(
                    {"features": ["force-validate", "native-solvers"]},
                    [str(p) for p in (ROOT / "python/pse").glob("_native*.so")],
                ),
            )
        started = time.time()
        code = subprocess.call(
            [
                sys.executable,
                "-m",
                "pytest",
                "python/pse/tests",
                "-m",
                "unit or component or integration",
                "--maxfail=0",
                "--continue-on-collection-errors",
                *extra,
            ],
            cwd=ROOT,
        )
        results, errors = validation.collect_report(
            report, report.parent, report.stem, started
        )
        return int(
            code != 0 or bool(errors) or any(r["status"] != "passed" for r in results)
        )
    if kind != "rust":
        raise ValueError("expected rust or python")
    listing = os.environ.get("PSE_NEXTEST_ACTION", "").startswith("list ")
    if listing or path:
        inventory = subprocess.run(
            rust_command("list", ["--message-format", "json", *extra]),
            cwd=ROOT,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
        )
        if inventory.returncode:
            print(inventory.stdout, end="")
            return inventory.returncode
        validation_receipts.native_selection(inventory.stdout)
        if path:
            data = json.loads(inventory.stdout)
            binaries = [
                s["binary-path"]
                for s in data["rust-suites"].values()
                if any(
                    t["filter-match"]["status"] == "matches"
                    for t in s["testcases"].values()
                )
            ]
            validation.write_json(
                Path(path),
                native_provenance({"features": FEATURES.split(",")}, binaries),
            )
        if listing:
            print(inventory.stdout, end="")
            return 0
    return subprocess.call(rust_command("run", ["--no-fail-fast", *extra]), cwd=ROOT)


if __name__ == "__main__":
    raise SystemExit(main())
