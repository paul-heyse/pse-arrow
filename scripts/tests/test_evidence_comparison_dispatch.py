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


class PrivateEvidenceWorkerTests(unittest.TestCase):
    def test_private_comparison_uses_existing_cli_without_scientific_extension(self) -> None:
        import sys
        from types import SimpleNamespace
        from scripts.capabilities import evidence_worker

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            tool = root / "target/debug/pse-evidence"
            tool.parent.mkdir(parents=True)
            tool.write_text(
                f"#!{sys.executable}\n"
                "import json,sys\n"
                "from pathlib import Path\n"
                "args=sys.argv[1:]\n"
                "assert args[:2]==['evidence','compare-offline']\n"
                "request=json.loads(Path(args[args.index('--request')+1]).read_text())\n"
                "sources=[json.loads(Path(args[i+1]).read_text()) for i,v in enumerate(args) if v=='--input']\n"
                "print(json.dumps({'cut':'private-cut','request':request,'sources':sources}))\n"
            )
            tool.chmod(0o700)
            with (
                patch.object(evidence_worker, "SchemaSet", return_value=SimpleNamespace(fingerprint="private-interface", document=lambda *_args: {})),
                patch.object(evidence_store, "prepare", side_effect=AssertionError("service setup forbidden")),
                patch.object(evidence_store, "run_native", side_effect=AssertionError("indexed read forbidden")),
            ):
                value = evidence_worker.execute(
                    {"schema_set_fingerprint":"private-interface", "request":{"version":1},
                     "snapshots":[{"counter":(1 << 63) + 3}], "observations":[]}, root=root,
                )
            self.assertEqual(value["outcome"], "observed")
            document = value["document"]
            assert isinstance(document, dict)
            sources = document["sources"]
            assert isinstance(sources, list)
            first = sources[0]
            assert isinstance(first, dict)
            self.assertEqual(first["counter"], (1 << 63) + 3)
            self.assertNotIn("pse", sys.modules)

    def test_private_interface_mismatch_refuses_before_cli_or_source_effects(self) -> None:
        from types import SimpleNamespace
        from scripts.capabilities import evidence_worker

        with (
            patch.object(evidence_worker, "SchemaSet", return_value=SimpleNamespace(fingerprint="new-interface")),
            patch.object(evidence_store, "run_offline") as execute,
        ):
            value = evidence_worker.execute({"schema_set_fingerprint":"old-interface"})
        self.assertEqual(value["code"], "pse.evidence.schema_set_mismatch")
        execute.assert_not_called()

    def test_private_binding_requires_only_its_independent_executable(self) -> None:
        from scripts.capabilities.native_contracts import BINDINGS

        binding = next(item for item in BINDINGS if item.operation == "evidence_compare")
        self.assertEqual(binding.worker.worker, "evidence")
        self.assertEqual([item.kind for item in binding.requirements], ["executable"])
        self.assertEqual(binding.worker.declaration.environment, ())


if __name__ == "__main__":
    unittest.main()
