# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Admission carries the original definition across request-local worker boundaries."""
# ruff: noqa: PT009, PT027 -- stdlib host controls
from __future__ import annotations

import asyncio
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from fastmcp.tools import FunctionTool

from scripts.capabilities import execution_basis as basis


class AdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.source = self.root / "handler.py"
        self.source.write_text("original")
        self.document = b'{"output":"selected"}'
        self.definition = basis.Definition(
            checkout=str(self.root), operation="selected",
            wire_revision=basis.digest({"name": "selected", "annotations": {"readOnlyHint": False}}),
            sources={"handler.py": __import__("hashlib").sha256(b"original").hexdigest()},
        )

    def publish(self) -> dict[str, str]:
        with basis.selected(self.definition):
            return basis.publish(self.root, self.root, "host", "archive", self.document)

    def test_unchanged_definition_request_and_route_admit(self) -> None:
        with patch.dict(os.environ, self.publish()):
            basis.verify(self.root, "host", "archive", self.document)

    def test_changed_request_or_route_refuses_before_effect(self) -> None:
        with patch.dict(os.environ, self.publish()):
            for worker, operation, data in [("host", "replay", self.document), ("qualification", "archive", self.document), ("host", "archive", b"changed")]:
                with self.assertRaises(basis.AdmissionRefused):
                    basis.verify(self.root, worker, operation, data)

    def test_changed_implementation_refuses(self) -> None:
        supplied = self.publish()
        self.source.write_text("changed effect")
        with patch.dict(os.environ, supplied), self.assertRaisesRegex(ValueError, "source changed"):
            basis.verify(self.root, "host", "archive", self.document)

    def test_shared_imports_do_not_expand_the_admitted_operation_basis(self) -> None:
        shared = self.root / "scripts/shared.py"
        shared.parent.mkdir()
        shared.write_text("from scripts import unrelated\n")
        (shared.parent / "unrelated.py").write_text("VALUE = 1\n")
        self.definition = self.definition.model_copy(update={
            "sources": {"scripts/shared.py": __import__("hashlib").sha256(shared.read_bytes()).hexdigest()}
        })
        with patch.dict(os.environ, self.publish()):
            basis.verify(self.root, "host", "archive", self.document)
            shared.write_text("from scripts import new_interpreter\n")
            with self.assertRaises(basis.AdmissionRefused):
                basis.verify(self.root, "host", "archive", self.document)

    def test_annotations_metadata_and_description_affect_wire(self) -> None:
        def read() -> str:
            return "observed"
        tool = FunctionTool.from_function(read)
        original = basis.digest(basis.wire(tool))
        tool.meta = {"new": "metadata"}
        self.assertNotEqual(original, basis.digest(basis.wire(tool)))
        tool.meta = {}
        tool.description = "different operation meaning"
        self.assertNotEqual(original, basis.digest(basis.wire(tool)))


class ConcurrentAdmissionTests(unittest.IsolatedAsyncioTestCase):
    async def test_overlapping_calls_do_not_share_selection(self) -> None:
        ready = asyncio.Event()
        arrivals = 0
        async def request(name: str) -> str:
            nonlocal arrivals
            selected = basis.Definition(checkout="/selected", operation=name, wire_revision=basis.digest({}), sources={})
            with basis.selected(selected):
                arrivals += 1
                if arrivals == 2:
                    ready.set()
                await asyncio.wait_for(ready.wait(), timeout=2)
                return basis._CURRENT.get().operation
        self.assertEqual(await asyncio.gather(request("one"), request("two")), ["one", "two"])


class NativeCurrencyAdmissionTests(unittest.TestCase):
    def test_matching_schema_cannot_admit_unknown_artifact_before_runtime(self) -> None:
        import sys
        import types
        from scripts import python_extension
        from scripts.capabilities import native_worker
        fake = types.ModuleType("pse")
        fake.SCHEMA_SET_FINGERPRINT = "same-schema"
        fake.codec = object()
        fake.diagnostics = object()
        contracts = types.ModuleType("pse.contracts")
        contracts.documents = object()
        observation = python_extension.ExtensionCurrency(
            state="unknown", checkout="/selected", interpreter="/selected/.venv/bin/python",
            mode="native", profile=None, validated=True,
            cause="missing-preparation-evidence", remedy="Explicitly prepare and reattach.",
        )
        with patch.dict(sys.modules, {"pse": fake, "pse.contracts": contracts}), patch.dict(os.environ, {basis.ENVIRONMENT: "/private/admission.json"}), patch.object(python_extension, "currency_check_installed", return_value=observation) as checked:
            result = native_worker.execute("diagnostic_checkpoint", {"schema_set_fingerprint": "same-schema"}, scratch=Path("/unused"))
        self.assertEqual(result["code"], "pse.native.artifact_not_current")
        checked.assert_called_once()
