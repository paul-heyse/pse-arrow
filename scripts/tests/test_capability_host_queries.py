# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual read-owner consumers preserve selection, authority and bounded detail."""

# ruff: noqa: PT009, PT027 -- stdlib endpoint controls
from __future__ import annotations

import json
import unittest
from pathlib import Path
from unittest.mock import patch

from fastmcp import Client
from fastmcp.exceptions import ToolError

from scripts.capabilities.server import build

ROOT = Path(__file__).resolve().parents[2]


class QueryTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        self.runtime = build(ROOT)
        self.client = Client(self.runtime.server)
        await self.client.__aenter__()
        self.addAsyncCleanup(self.client.__aexit__, None, None, None)

    async def call(self, name: str, arguments: dict) -> dict:
        await self.client.call_tool("capabilities_select", {"capabilities": [name]})
        result = (
            await self.client.call_tool(
                "capabilities_invoke_read", {"capability": name, "arguments": arguments}
            )
        ).structured_content
        assert result is not None
        return result

    async def test_environment_observation_contains_no_credentials_or_admission(
        self,
    ) -> None:
        with patch.dict("os.environ", {"AWS_SECRET_ACCESS_KEY": "do-not-disclose"}):
            result = await self.call("environment_explain", {})
        self.assertEqual(
            result["payload"]["placement"]["admission"], "not observed or acquired"
        )
        self.assertEqual(result["payload"]["checkout"], str(ROOT))
        self.assertNotIn("do-not-disclose", json.dumps(result))

    async def test_scope_pages_actual_plan_owner_without_authorizing_execution(
        self,
    ) -> None:
        result = await self.call(
            "document_scope_query",
            {
                "path": "docs/plans/52-agent-effectiveness-and-engineering-workflows.md",
                "limit": 1,
            },
        )
        payload = result["payload"]
        self.assertGreater(payload["total"], 1)
        self.assertEqual(len(payload["items"]), 1)
        self.assertEqual(payload["next_offset"], 1)
        self.assertIn("no scheduling or authorization", payload["authority"])
        with self.assertRaises(ToolError):
            await self.client.call_tool("document_scope_query", {"limit": "3"})

    async def test_exact_source_knowledge_is_attributed_and_not_a_store_query(
        self,
    ) -> None:
        result = await self.call(
            "knowledge_source_corpus", {"paths": ["docs/dev/capability-interface.md"]}
        )
        self.assertEqual(result["operation"], "knowledge_source_corpus")
        self.assertIn("docs/dev/capability-interface.md", json.dumps(result))
        with self.assertRaises(ToolError):
            await self.client.call_tool("knowledge_source_corpus", {"paths": []})

    async def test_storage_identity_is_not_reclamation_or_size_measurement(
        self,
    ) -> None:
        result = await self.call("storage_observe", {})
        self.assertFalse(result["payload"]["reclamation_authorized"])
        for item in result["payload"]["items"]:
            self.assertIsNone(item["reclaimable_bytes"])
            self.assertIn(item["status"], {"metadata-only", "unavailable", "alias"})

    async def test_library_packet_uses_actual_exact_pin_without_building(self) -> None:
        with patch(
            "subprocess.run",
            side_effect=AssertionError("library packet must not build or resolve"),
        ):
            result = await self.call("library_source_packet", {"package": "fastmcp"})
        self.assertIn("4.1.3", json.dumps(result))
        self.assertIn("pyproject.toml", json.dumps(result))

    async def test_coverage_capture_requires_actual_structured_owner_evidence(
        self,
    ) -> None:
        with self.assertRaises(ToolError):
            await self.call(
                "coverage_correspondence_read",
                {"directory": "docs/dev", "base": "HEAD"},
            )


if __name__ == "__main__":
    unittest.main()
