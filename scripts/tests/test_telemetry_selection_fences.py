# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual canonical source fences protect direct, projected and fixed reads."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import unittest
from pathlib import Path
from unittest.mock import patch

from fastmcp import Client
from mcp import MCPError

from scripts import telemetry_selection
from scripts.capabilities import server
from scripts.capabilities.coherence import METRIC_DECLARATIONS, Sources
from scripts.telemetry_selection import SelectionPage, SelectionReference

ROOT = Path(__file__).resolve().parents[2]


class ActualTelemetryFenceTests(unittest.IsolatedAsyncioTestCase):
    async def check_fence(self, changed_source: str) -> None:
        runtime = server.build(ROOT)
        # Isolate the sticky changed-generation account from other tests and the
        # module's loaded snapshot. The actual handlers retain this same Interface.
        cut = Sources(ROOT, runtime.interface.sources.sources)
        runtime.interface.sources = cut
        self.assertIn(
            changed_source, runtime.interface.operations["telemetry_selection_read"]
        )
        reference = SelectionReference(directory="/explicit/copy", sha256="a" * 64)
        page = SelectionPage(
            reference=reference,
            selection=0,
            request={},
            source=None,
            records=[{"fixture_row": 0}],
            copied_rows=1,
            returned_rows=1,
            copied_selection_complete=True,
            next_cursor=None,
            missing_context=[],
            interpretation="source-fence fixture; no native custody exercised",
        )
        arguments = {"reference": reference.model_dump(mode="json"), "limit": 2}
        uri = f"pse://telemetry/selection/{telemetry_selection.resource_reference(reference)}/0/2"
        original = cut._read_optional  # noqa: SLF001 -- inject serving generation drift without editing actual sources

        def changed(path: str) -> bytes | None:
            raw = original(path)
            return (raw or b"") + b"\n " if path == changed_source else raw

        with patch.object(telemetry_selection, "page", return_value=page) as owner:
            async with Client(runtime.server) as client:
                await client.call_tool(
                    "capabilities_select",
                    {"capabilities": ["telemetry_selection_read"]},
                )
                self.assertFalse(
                    (
                        await client.call_tool("telemetry_selection_read", arguments)
                    ).is_error
                )
                await client.read_resource(uri)
                self.assertFalse(
                    (await client.call_tool("read_resource", {"uri": uri})).is_error
                )
                owner.reset_mock()
                with patch.object(cut, "_read_optional", side_effect=changed):
                    direct = await client.call_tool(
                        "telemetry_selection_read", arguments, raise_on_error=False
                    )
                    fixed = await client.call_tool(
                        "capabilities_invoke_read",
                        {
                            "capability": "telemetry_selection_read",
                            "arguments": arguments,
                        },
                        raise_on_error=False,
                    )
                    projected = await client.call_tool(
                        "read_resource", {"uri": uri}, raise_on_error=False
                    )
                    with self.assertRaises(MCPError):
                        await client.read_resource(uri)
                self.assertTrue(direct.is_error)
                self.assertTrue(fixed.is_error)
                self.assertTrue(projected.is_error)
                self.assertIn("restart_required", str(direct.content))
                self.assertIn("restart_required", str(fixed.content))
                owner.assert_not_called()

    async def test_changed_selection_owner_refuses_before_every_dispatch_path(
        self,
    ) -> None:
        await self.check_fence("scripts/telemetry_selection.py")

    async def test_changed_metric_declaration_generation_refuses_resources_too(
        self,
    ) -> None:
        self.assertTrue((ROOT / METRIC_DECLARATIONS).is_file())
        await self.check_fence(METRIC_DECLARATIONS)


if __name__ == "__main__":
    unittest.main()
