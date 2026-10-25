# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Private worker handoffs retain their exact admitted route before effects."""

# ruff: noqa: PT009 -- stdlib unittest controls.
from __future__ import annotations

import hashlib
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.capabilities import execution_basis as basis
from scripts.capabilities import qualification_worker as worker


class QualificationWorkerAdmissionTests(unittest.TestCase):
    def run_handoff(self, route: str, operation: str) -> tuple[dict[str, object], int]:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            request = directory / "request.json"
            response = directory / "response.json"
            document = b'{"output":"selected"}'
            request.write_bytes(document)
            relative = "scripts/capabilities/qualification_worker.py"
            definition = basis.Definition(
                checkout=str(worker.ROOT.resolve()),
                operation="original-tool",
                wire_revision=basis.digest({"name": "original-tool"}),
                sources={
                    relative: hashlib.sha256(
                        (worker.ROOT / relative).read_bytes()
                    ).hexdigest()
                },
            )
            with basis.selected(definition):
                supplied = basis.publish(
                    directory, worker.ROOT, route, operation, document
                )
            # Verification is real. Only the downstream effect is replaced, so the
            # route regression cannot pass by avoiding the admission boundary.
            with (
                patch.dict(os.environ, supplied),
                patch.object(
                    worker, "execute", return_value={"owner": "selected"}
                ) as effect,
            ):
                self.assertEqual(
                    worker.main([operation, str(request), str(response)]), 0
                )
            return json.loads(response.read_text()), effect.call_count

    def test_valid_rust_tests_private_handoff_reaches_its_original_owner(self) -> None:
        result, effects = self.run_handoff("rust-tests", "rust-test")
        self.assertEqual(result, {"result": {"owner": "selected"}})
        self.assertEqual(effects, 1)

    def test_wrong_rust_route_is_refused_before_any_owner_effect(self) -> None:
        result, effects = self.run_handoff("qualification", "rust-test")
        error = result["error"]
        assert isinstance(error, dict)
        self.assertEqual(error["code"], "pse.capabilities.stale_operation")
        self.assertEqual(error["stage"], "admission")
        self.assertEqual(effects, 0)

    def test_original_qualification_route_remains_distinct_and_valid(self) -> None:
        result, effects = self.run_handoff("qualification", "contracts-generate")
        self.assertEqual(result, {"result": {"owner": "selected"}})
        self.assertEqual(effects, 1)


if __name__ == "__main__":
    unittest.main()
