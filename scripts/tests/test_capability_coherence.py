# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Restart fences follow loaded serving contracts, not the inspected project."""
# ruff: noqa: PT009, PT027 -- stdlib host controls

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
from scripts.capabilities.workers import WORKERS, WorkerCall


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

    async def test_changed_owner_never_dispatches_old_handler_with_new_schema(
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
        self.assertTrue(result.is_error)
        self.assertIn("pse.capabilities.restart_required", str(result.content))
        self.assertEqual(self.calls, 0)
        self.assertTrue(self.interface.status("first")["restart_required"])
        self.assertFalse(self.interface.status("second")["restart_required"])
        self.assertEqual(
            (await self.client.call_tool("second", {})).structured_content,
            {"outcome": "second"},
        )

    async def test_transitive_local_owner_change_and_deletion_refuse(self) -> None:
        self.dependency.unlink()
        result = await self.client.call_tool("first", {}, raise_on_error=False)
        self.assertTrue(result.is_error)
        changed = self.interface.status("first")["changed_serving_sources"]
        self.assertIsInstance(changed, list)
        if not isinstance(changed, list):
            self.fail("Restart status must identify the changed serving paths")
        self.assertIn("scripts/dependency.py", changed)
        self.assertEqual(self.calls, 0)
        self.assertFalse(self.interface.status("second")["restart_required"])

    async def test_common_change_blocks_all_affected_execution(self) -> None:
        self.common.write_text("# New dispatch interpretation\n")
        for name in ("first", "second"):
            result = await self.client.call_tool(name, {}, raise_on_error=False)
            self.assertTrue(result.is_error)
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

    async def test_reversion_does_not_clear_observed_restart_and_new_process_cut_does(
        self,
    ) -> None:
        original = self.first.read_bytes()
        self.first.write_bytes(original + b"# revision two\n")
        self.assertTrue(self.interface.status("first")["restart_required"])
        self.first.write_bytes(original)
        self.assertTrue(self.interface.status("first")["restart_required"])
        fresh = Sources(self.root, self.sources.sources)
        self.assertFalse(fresh.check(fresh.sources))
        fresh.verify_composition()

    async def test_edit_during_composition_refuses_before_publication(self) -> None:
        self.first.write_text("# Changed between capture and composition\n")
        with self.assertRaisesRegex(ValueError, "changed while loading"):
            install(FastMCP("not-published"), self.sources, self.catalogue)

    async def test_revision_uses_bytes_not_checkout_location_or_mtime(self) -> None:
        original = self.sources.revision
        self.first.touch()
        copied = Sources(self.root, self.sources.sources)
        self.assertEqual(copied.revision, original)
        self.assertEqual(copied.check(copied.sources), ())

    async def test_fixed_worker_edit_refuses_before_child_or_handler_effect(
        self,
    ) -> None:
        for worker_name, capability, module in (
            ("native", "native_source_admit", "scripts.capabilities.native"),
            ("host", "experiment_archive", "scripts.capabilities.host_actions"),
            ("qualification", "assessment_run", "scripts.capabilities.qualification"),
        ):
            with self.subTest(worker=worker_name):
                relative = WORKERS[worker_name].module.replace(".", "/") + ".py"
                worker = self.root / relative
                worker.write_text("# Original worker generation\n")
                sources = Sources(self.root, (relative, "scripts/first.py"))

                def action() -> dict[str, str]:
                    self.calls += 1
                    return {"outcome": "must not execute"}

                action.__module__ = module
                tool = Tool.from_function(action, name=capability)
                spec = Capability(
                    id=capability,
                    title=capability,
                    description=capability,
                    domain="source",
                    owner="scripts/first.py",
                    source="scripts/first.py",
                    exposure="tool",
                    effects="Test fixture counter",
                    worker_calls=(
                        WorkerCall(
                            worker_name, next(iter(WORKERS[worker_name].operations))
                        ),
                    ),
                )
                catalogue = Catalogue(
                    (Domain(id="source", title="Source", description="Source"),),
                    (),
                    (),
                    (spec,),
                    {capability: tool},
                    loaded_source_revision=sources.revision,
                )
                endpoint = FastMCP("worker-fence")
                endpoint.add_tool(tool)
                interface = install(endpoint, sources, catalogue)
                self.assertIn(relative, interface.operations[capability])
                worker.write_text(
                    "# Newly edited child must not execute under old schema\n"
                )
                async with Client(endpoint) as client:
                    result = await client.call_tool(
                        capability, {}, raise_on_error=False
                    )
                self.assertTrue(result.is_error)
                self.assertIn("pse.capabilities.restart_required", str(result.content))
                self.assertEqual(self.calls, 0)


if __name__ == "__main__":
    unittest.main()
