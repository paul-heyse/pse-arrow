# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Compose database controls inside the existing reference-run lifetime owner."""

from __future__ import annotations

import argparse
import os
import subprocess
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
    parser.add_argument("--initializer",type=Path,required=True)
    args = parser.parse_args()
    prior = os.environ.get(test_resources.MARKER)
    try:
        if not args.query_only:
            surrealkit_fixture_check.run(args.state, args.tool_root, args.evidence / "surrealkit")
    finally:
        if prior is None:
            os.environ.pop(test_resources.MARKER, None)
        else:
            os.environ[test_resources.MARKER] = prior
    (args.evidence/"canonical").mkdir(parents=True,exist_ok=True,mode=0o700)
    environment = {**os.environ, "PSE_SURREAL_STATE": str(args.state), "PSE_DIAGNOSTIC_EVIDENCE":str(args.evidence/"canonical"),"PSE_CANONICAL_TOOLS":str(args.initializer.resolve(strict=True))}
    return subprocess.call(
        [str(ROOT / "scripts/pse-env"), "--native", "--store", "--", "just", "unit-package", "pse-operations",
         "test(canonical_candidates_inspection) | test(canonical_graph_explanation)", "--features", "pse-operations/canonical-tests"],
        env=environment, cwd=ROOT,
    )


if __name__ == "__main__":
    raise SystemExit(main())
