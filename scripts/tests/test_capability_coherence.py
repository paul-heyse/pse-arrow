# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Dispatch and automatic status use loaded contracts; provenance is explicit."""
# ruff: noqa: PT009 -- stdlib host controls

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from fastmcp import Client, FastMCP
from fastmcp.tools import Tool

from scripts.capabilities.catalogue import Catalogue
from scripts.capabilities.coherence import Sources, install
from scripts.capabilities.model import Capability, Domain


class CoherenceTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / "scripts/capabilities").mkdir(parents=True)
        self.first = self.root / "scripts/first.py"
        self.second = self.root / "scripts/second.py"
        self.dependency = self.root / "scripts/dependency.py"
        self.common = self.root / "scripts/capabilities/server.py"
        self.first.write_text("from scripts import dependency\n")
        self.second.write_text("# Independent second owner\n")
        self.dependency.write_text("VALUE = 1\n")
        self.common.write_text("# Shared serving interpretation\n")
        self.sources = Sources(
            self.root,
            ("scripts/first.py", "scripts/second.py", "scripts/capabilities/server.py"),
        )
        self.calls = 0

        def first(limit: int = 3) -> dict[str, int]:
            """Effect counter exposes accidentally dispatched old handlers."""
            self.calls += 1
            return {"count": limit}

        def second() -> dict[str, str]:
            """Independent operation remains usable."""
            return {"outcome": "second"}

        self.server = FastMCP("coherence-control", strict_input_validation=True)
        tools: dict[str, Tool] = {
            fn.__name__: Tool.from_function(fn) for fn in (first, second)
        }
        specs = tuple(
            Capability(
                id=name,
                title=name,
                description=name,
                domain="source",
                owner=f"scripts/{name}.py",
                source=f"scripts/{name}.py",
                exposure="tool",
                effects="Test fixture only",
            )
            for name in tools
        )
        self.catalogue = Catalogue(
            (Domain(id="source", title="Source", description="Source"),),
            (),
            (),
            specs,
            tools,
            loaded_source_revision=self.sources.revision,
        )
        for tool in tools.values():
            self.server.add_tool(tool)
        self.interface = install(self.server, self.sources, self.catalogue)
        self.client = Client(self.server)
        await self.client.__aenter__()
        self.addAsyncCleanup(self.client.__aexit__, None, None, None)

    async def test_source_edit_does_not_replace_loaded_handler_or_schema(
        self,
    ) -> None:
        self.first.write_text(
            "def first(new_required_parameter: str):\n    return new_required_parameter\n"
        )
        tools = await self.client.list_tools()
        loaded = next(tool for tool in tools if tool.name == "first")
        self.assertIn("limit", loaded.input_schema["properties"])
        self.assertNotIn("new_required_parameter", loaded.input_schema["properties"])
        result = await self.client.call_tool(
            "first", {"limit": 7}, raise_on_error=False
        )
        self.assertFalse(result.is_error)
        self.assertEqual(result.structured_content, {"count": 7})
        self.assertEqual(self.calls, 1)
        self.assertFalse(self.interface.status("first")["restart_required"])
        self.assertFalse(self.interface.status("second")["restart_required"])
        self.assertEqual(
            (await self.client.call_tool("second", {})).structured_content,
            {"outcome": "second"},
        )

    async def test_dispatch_and_require_do_not_scan_sources(self) -> None:
        self.dependency.unlink()
        self.common.write_text("# Controlled editing requires explicit drain/reload\n")
        with patch.object(
            self.sources, "check", side_effect=AssertionError("source scan")
        ):
            self.interface.require("first")
            self.assertFalse(self.interface.status("first")["restart_required"])
            self.assertFalse(self.interface.status()["restart_required"])
            result = await self.client.call_tool("first", {})
        self.assertEqual(result.structured_content, {"count": 3})
        changed = self.sources.check(self.interface.common | self.interface.operations["first"])
        self.assertIn("scripts/dependency.py", changed)
        self.assertIn("scripts/capabilities/server.py", changed)

    async def test_changed_loaded_wire_refuses_before_handler(self) -> None:
        self.catalogue.tools["first"].description = "Changed live interpretation"
        self.assertTrue(self.interface.status("first")["restart_required"])
        result = await self.client.call_tool("first", {}, raise_on_error=False)
        self.assertTrue(result.is_error)
        self.assertIn("pse.capabilities.restart_required", str(result.content))
        self.assertEqual(self.calls, 0)

    async def test_unrelated_source_and_guide_edits_do_not_invalidate_operation(
        self,
    ) -> None:
        (self.root / "model.pse").write_text("current model source")
        (self.root / "unrelated.md").write_text("current unrelated guide")
        observed = []
        original = Path.read_bytes

        def record(path: Path) -> bytes:
            if path.is_relative_to(self.root):
                observed.append(path.relative_to(self.root).as_posix())
            return original(path)

        with patch.object(Path, "read_bytes", record):
            result = await self.client.call_tool("second", {})
        self.assertEqual(result.structured_content, {"outcome": "second"})
        self.assertNotIn("scripts/first.py", observed)
        self.assertNotIn("scripts/dependency.py", observed)
        self.assertFalse(self.interface.status("second")["restart_required"])

    async def test_explicit_provenance_observation_does_not_revoke_loaded_contract(
        self,
    ) -> None:
        original = self.first.read_bytes()
        self.first.write_bytes(original + b"# revision two\n")
        self.assertIn("scripts/first.py", self.sources.check(self.sources.sources))
        self.assertFalse(self.interface.status("first")["restart_required"])
        self.first.write_bytes(original)
        self.assertIn("scripts/first.py", self.sources.check(self.sources.sources))
        self.assertFalse(self.interface.status("first")["restart_required"])
        fresh = Sources(self.root, self.sources.sources)
        self.assertFalse(fresh.check(fresh.sources))

    async def test_install_uses_constructed_provenance_without_second_scan(
        self,
    ) -> None:
        with patch.object(
            self.sources, "check", side_effect=AssertionError("source scan")
        ):
            interface = install(FastMCP("loaded"), self.sources, self.catalogue)
        self.assertEqual(interface.revision, self.interface.revision)

    async def test_revision_uses_bytes_not_checkout_location_or_mtime(self) -> None:
        original = self.sources.revision
        self.first.touch()
        copied = Sources(self.root, self.sources.sources)
        self.assertEqual(copied.revision, original)
        self.assertEqual(copied.check(copied.sources), ())


if __name__ == "__main__":
    unittest.main()
