# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Loaded contracts serve direct, projected and fixed reads without source scans."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import unittest
from pathlib import Path
from unittest.mock import patch

from fastmcp import Client

from scripts import telemetry_selection
from scripts.capabilities import server
from scripts.capabilities.coherence import METRIC_DECLARATIONS, Sources
from scripts.telemetry_selection import SelectionPage, SelectionReference

ROOT = Path(__file__).resolve().parents[2]


class ActualTelemetryContractTests(unittest.IsolatedAsyncioTestCase):
    async def check_loaded_dispatch(self, changed_source: str) -> None:
        runtime = server.build(ROOT)
        # Preserve the loaded provenance while forbidding implicit source scans.
        cut = Sources(ROOT, runtime.interface.sources.sources)
        runtime.interface.sources = cut
        self.assertIn(
            changed_source, runtime.interface.operations["telemetry_selection_read"]
        )
        reference = SelectionReference(directory="/explicit/copy", sha256="a" * 64)
        page = SelectionPage(
            reference=reference,
            selection=0,
            question="Source coherence fixture",
            original_references=[],
            next_readers=[],
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
                with patch.object(cut, "check", side_effect=AssertionError("source scan")):
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
                    await client.read_resource(uri)
                self.assertFalse(direct.is_error)
                self.assertFalse(fixed.is_error)
                self.assertFalse(projected.is_error)
                self.assertEqual(owner.call_count, 4)

    async def test_selection_dispatch_uses_loaded_contract_on_every_path(
        self,
    ) -> None:
        await self.check_loaded_dispatch("scripts/telemetry_selection.py")

    async def test_loaded_metric_declaration_provenance_includes_resources(
        self,
    ) -> None:
        self.assertTrue((ROOT / METRIC_DECLARATIONS).is_file())
        await self.check_loaded_dispatch(METRIC_DECLARATIONS)


if __name__ == "__main__":
    unittest.main()
