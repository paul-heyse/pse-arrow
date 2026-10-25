# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual-request explanation never invokes the target or prepares its runtime."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from fastmcp.tools import Tool

from scripts.capabilities import readiness, specialists
from scripts.capabilities.model import Capability
from scripts.capabilities.qualification_contracts import CompilationRequest


class ExplanationTests(unittest.IsolatedAsyncioTestCase):
    async def test_native_compile_selection_is_explained_without_invocation(
        self,
    ) -> None:
        calls = []

        def target(request: CompilationRequest) -> None:
            calls.append(request)

        tool = Tool.from_function(target, name="compilation_check")
        spec = Capability(
            id="compilation_check",
            title="Check",
            description="Check",
            domain="validation",
            owner="fixture",
            source="fixture",
            exposure="tool",
            effects="compile",
            invocation_lifetime="single",
        )
        with tempfile.TemporaryDirectory() as directory:
            owner = specialists.Specialists(Path(directory))
            try:
                _, tools = readiness.bindings(
                    Path(directory),
                    capabilities=lambda: {spec.id: spec},
                    operation_tools=lambda: {spec.id: tool},
                    specialists=owner,
                )
                request = CompilationRequest(
                    output="build/check", package="pse-runtime", target="library"
                ).model_dump(mode="json", exclude_defaults=True)
                result = await tools["capability_explain"].run(
                    {"capability": spec.id, "arguments": {"request": request}}
                )
                document = result.structured_content
                assert isinstance(document, dict)
                assert document["input_schema_valid"]
                assert (document["admission"]) == ("not_attempted")
                assert (document["selection"]["gates"][0]["recipe"]) == (
                    "check-library"
                )
                assert (document["selection"]["gates"][0]["args"]) == (["pse-runtime"])
                assert (calls) == ([])
                request["target"] = "test"
                result = await tools["capability_explain"].run(
                    {"capability": spec.id, "arguments": {"request": request}}
                )
                assert isinstance(result.structured_content, dict)
                assert not (result.structured_content["input_schema_valid"])
                assert (calls) == ([])
            finally:
                await owner.close()
