# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual CLI dispatch preserves original native requests and source-only operation."""

# ruff: noqa: PT009 -- stdlib protocol controls remain runnable with unittest
from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import evidence_store


class EvidenceComparisonDispatchTests(unittest.TestCase):
    def test_offline_comparison_does_not_open_state_or_build_and_forwards_original_bytes(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            request = directory / "request.json"
            request.write_text('{"original":"native request admitted only by Rust"}')
            source = directory / "source.json"
            source.write_text('{"original":"owner snapshot admitted only by Rust"}')
            tool = directory / "tool"
            tool.write_text("#!/bin/sh\nexit 0\n")
            tool.chmod(0o700)
            original_request = request.read_bytes()
            original_source = source.read_bytes()
            with (
                patch.dict(
                    os.environ, {"PSE_SURREAL_STATE": str(directory / "unavailable")}
                ),
                patch.object(
                    evidence_store,
                    "run_native",
                    side_effect=AssertionError("service open"),
                ),
                patch.object(evidence_store.subprocess, "run") as run,
            ):
                run.return_value.returncode = 0
                result = evidence_store.main(
                    [
                        "--tool",
                        str(tool),
                        "compare-offline",
                        "--request",
                        str(request),
                        "--input",
                        str(source),
                    ]
                )
            self.assertEqual(result, 0)
            self.assertEqual(
                run.call_args.args[0],
                [
                    str(tool),
                    "evidence",
                    "compare-offline",
                    "--request",
                    str(request),
                    "--input",
                    str(source),
                ],
            )
            self.assertNotIn("PSE_TOOLING_ACTIVITY_FD", run.call_args.kwargs["env"])
            self.assertEqual(request.read_bytes(), original_request)
            self.assertEqual(source.read_bytes(), original_source)

    def test_indexed_comparison_uses_existing_lifetime_owner_and_exact_request(
        self,
    ) -> None:
        with patch.object(evidence_store, "run_native", return_value=0) as run:
            result = evidence_store.main(
                [
                    "--state",
                    "/selected/state",
                    "compare-indexed",
                    "--request",
                    "/selected/request.json",
                    "--observations",
                    "/selected/observations.json",
                ]
            )
        self.assertEqual(result, 0)
        self.assertEqual(run.call_args.args[0], Path("/selected/state"))
        self.assertEqual(
            run.call_args.args[2],
            [
                "compare-indexed",
                "--request",
                "/selected/request.json",
                "--observations",
                "/selected/observations.json",
            ],
        )


if __name__ == "__main__":
    unittest.main()
