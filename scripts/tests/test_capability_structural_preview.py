# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite structural preview uses original exact edit custody and protection."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from fastmcp.exceptions import ToolError

from scripts.capabilities.specialists import Specialists, bindings


class StructuralPreviewTests(unittest.IsolatedAsyncioTestCase):
    async def test_preview_apply_reconcile_and_stale_basis(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "sample.py"
            source.write_text("import json\nvalue=json.loads(path.read_text())\n", encoding="utf-8")
            owner = Specialists(root)
            try:
                specs, tools = bindings(root, owner=owner)
                spec = next(item for item in specs if item.id == "structural_json_source_bytes_preview")
                self.assertEqual(spec.invocation_lifetime, "endpoint")
                self.assertEqual(spec.requirements, ())
                result = await tools[spec.id].run({"paths": ["sample.py"]})
                document = result.structured_content
                handle = document["payload"]["preview_sha256"]
                self.assertIn("path.read_bytes()", owner.retained_preview(handle)["files"][0]["after"])
                owner.apply_preview(handle, "receipt.json")
                self.assertIn("path.read_bytes()", source.read_text())
                self.assertTrue(owner.reconcile_preview(handle)["complete"])
                source.write_text("import json\nvalue=json.loads(path.read_text())\n")
                result = await tools[spec.id].run({"paths": ["sample.py"]})
                handle = result.structured_content["payload"]["preview_sha256"]
                source.write_text("changed=1\n")
                with self.assertRaises(ValueError):
                    owner.apply_preview(handle, "stale-receipt.json")
                (root / "target").mkdir()
                (root / "target/protected.py").write_text("import json\n")
                with self.assertRaises(ToolError):
                    await tools[spec.id].run({"paths": ["target/protected.py"]})
            finally:
                await owner.close()
