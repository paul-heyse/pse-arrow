# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Compose database controls inside the existing reference-run lifetime owner."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

from scripts import test_resources
from scripts.tests import surrealkit_fixture_check

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--tool-root", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--query-only", action="store_true")
    parser.add_argument("--initializer", type=Path, required=True)
    args = parser.parse_args()
    prior = os.environ.get(test_resources.MARKER)
    try:
        if not args.query_only:
            surrealkit_fixture_check.run(
                args.state, args.tool_root, args.evidence / "surrealkit"
            )
    finally:
        if prior is None:
            os.environ.pop(test_resources.MARKER, None)
        else:
            os.environ[test_resources.MARKER] = prior
    (args.evidence / "canonical").mkdir(parents=True, exist_ok=True, mode=0o700)
    environment = {
        **os.environ,
        "PSE_SURREAL_STATE": str(args.state),
    }
    code = subprocess.call(
        [
            str(ROOT / "scripts/pse-env"),
            "--native",
            "--store",
            "--",
            "just",
            "unit-package",
            "pse-operations",
            "test(canonical_candidates_inspection) | test(canonical_graph_explanation)",
            "--features",
            "pse-operations/canonical-tests",
        ],
        env=environment,
        cwd=ROOT,
    )
    if code:
        return code
    # The substantive API assertions and optional operator evidence have distinct
    # selected invocations. Neither can silently stand in for the other.
    return subprocess.call(
        [
            str(ROOT / "scripts/pse-env"),
            "--native",
            "--store",
            "--",
            sys.executable,
            "-m",
            "scripts.tests.canonical_boundary_run",
            "operator",
            str(args.evidence / "canonical" / "operator"),
            "--executor",
            str(args.initializer.resolve(strict=True)),
        ],
        env=environment,
        cwd=ROOT,
    )


if __name__ == "__main__":
    raise SystemExit(main())
