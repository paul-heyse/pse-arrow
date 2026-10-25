# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed indexed native routing preserves original documents without source fallback."""

# ruff: noqa: PT009, PT027 -- stdlib focused controls
from __future__ import annotations

import unittest
from pathlib import Path
from unittest.mock import patch

from jsonschema import Draft202012Validator

from scripts import knowledge_store
from scripts.capabilities import knowledge_queries


class PublishedSchemaFixture:
    """A wire fixture, not a duplicate implementation of the native query grammar."""

    def __init__(self, _root: Path) -> None:
        pass

    def document(self, _slug: str, _direction: str) -> dict:
        return {"type": "object"}

    def identity(self, slug: str, direction: str = "input") -> dict:
        return {"slug": slug, "direction": direction}

    def envelope(self, host: dict, fields: dict, _required: tuple) -> dict:
        if fields != {"request": "knowledge-query"}:
            raise AssertionError("native request grammar must come from its published owner")
        return host


class IndexedCapabilityTests(unittest.IsolatedAsyncioTestCase):
    def setUp(self) -> None:
        with patch.object(knowledge_queries, "SchemaSet", PublishedSchemaFixture):
            self.specs, self.tools = knowledge_queries.bindings(Path.cwd())

    async def test_query_preserves_original_document_and_exact_private_routing(self) -> None:
        request = {"version": 1, "mode": "productions", "artifact_id": "exact", "limit": 1}
        original = {"version": 1, "issuing_store": "original", "generation": "a" * 64,
                    "subjects": [{"label": "x" * 17000}], "next": {"after": "next"}}
        answer = {"outcome": "observed", "document": original, "source_observations": []}
        with patch.object(knowledge_store, "indexed", return_value=answer) as read:
            result = await self.tools["knowledge_query"].run({"request": request, "state": "/selected"})
        self.assertEqual(result.structured_content["payload"]["document"], original)
        self.assertEqual(read.call_args.args[0], request)
        self.assertFalse(read.call_args.kwargs["explain"])
        self.assertEqual(read.call_args.kwargs["state"], Path("/selected"))
        self.assertTrue(all(spec.invocation_lifetime == "single" for spec in self.specs))

    async def test_native_cursor_refusal_and_unavailability_are_distinct(self) -> None:
        with patch.object(knowledge_store, "indexed", return_value={"outcome": "refused", "code": "pse.knowledge.cursor_context"}):
            refused = await self.tools["knowledge_explain"].run({"request": {"version": 1}, "state": "/selected"})
        self.assertEqual(refused.structured_content["payload"]["code"], "pse.knowledge.cursor_context")
        with patch.object(knowledge_store, "indexed", side_effect=knowledge_store.NativeKnowledgeUnavailableError("unavailable")):
            unavailable = await self.tools["knowledge_query"].run({"request": {"version": 1}, "state": "/selected"})
        self.assertEqual(unavailable.structured_content["payload"]["outcome"], "prerequisite_unavailable")

    async def test_actual_published_native_contracts_validate_bridge_without_store_activity(self) -> None:
        root = Path(__file__).resolve().parents[2]
        _specs, tools = knowledge_queries.bindings(root)
        tool = tools["knowledge_query"]
        request = {"request": {"version": 1, "mode": "productions", "artifact_id": "generated_artifact:python/pse/_native.pyi", "limit": 1}, "state": "/selected"}
        Draft202012Validator(tool.parameters).validate(request)
        original = {"version": 1, "issuing_store": "original", "generation": "a" * 64,
                    "freshness": "imported-cut-only", "coverage": {"producer_freshness": "unverified", "declaration_locations": "not-established", "undeclared_relationships": "not-inferred", "excerpt_attribution": "explicit-subject-only"},
                    "sources": [], "subjects": [], "excerpts": [], "edges": [], "productions": [], "excerpts_truncated": False, "next": None}
        with patch.object(knowledge_store, "indexed", return_value={"outcome": "observed", "document": original, "source_observations": []}):
            result = await tool.run(request)
        self.assertIsNotNone(tool.output_schema)
        Draft202012Validator(tool.output_schema).validate(result.structured_content)
        self.assertEqual(result.structured_content["payload"]["document"], original)
        missing_version = {**request, "request": {key: value for key, value in request["request"].items() if key != "version"}}
        self.assertFalse(Draft202012Validator(tool.parameters).is_valid(missing_version))


if __name__ == "__main__":
    unittest.main()
