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
            checkout=str(self.root),
            operation="selected",
            wire_revision=basis.digest(
                {"name": "selected", "annotations": {"readOnlyHint": False}}
            ),
            sources={
                "handler.py": __import__("hashlib").sha256(b"original").hexdigest()
            },
        )

    def publish(self) -> dict[str, str]:
        with basis.selected(self.definition):
            return basis.publish(self.root, self.root, "host", "archive", self.document)

    def test_unchanged_definition_request_and_route_admit(self) -> None:
        with patch.dict(os.environ, self.publish()):
            basis.verify(self.root, "host", "archive", self.document)

    def test_changed_request_or_route_refuses_before_effect(self) -> None:
        with patch.dict(os.environ, self.publish()):
            for worker, operation, data in [
                ("host", "replay", self.document),
                ("qualification", "archive", self.document),
                ("host", "archive", b"changed"),
            ]:
                with self.assertRaises(basis.AdmissionRefusedError):
                    basis.verify(self.root, worker, operation, data)

    def test_changed_implementation_refuses(self) -> None:
        supplied = self.publish()
        self.source.write_text("changed effect")
        with (
            patch.dict(os.environ, supplied),
            self.assertRaisesRegex(ValueError, "source changed"),
        ):
            basis.verify(self.root, "host", "archive", self.document)

    def test_shared_imports_do_not_expand_the_admitted_operation_basis(self) -> None:
        shared = self.root / "scripts/shared.py"
        shared.parent.mkdir()
        shared.write_text("from scripts import unrelated\n")
        (shared.parent / "unrelated.py").write_text("VALUE = 1\n")
        self.definition = self.definition.model_copy(
            update={
                "sources": {
                    "scripts/shared.py": __import__("hashlib")
                    .sha256(shared.read_bytes())
                    .hexdigest()
                }
            }
        )
        with patch.dict(os.environ, self.publish()):
            basis.verify(self.root, "host", "archive", self.document)
            (shared.parent / "unrelated.py").write_text("invalid unrelated syntax (\n")
            basis.verify(self.root, "host", "archive", self.document)
            shared.write_text("from scripts import new_interpreter\n")
            with self.assertRaises(basis.AdmissionRefusedError):
                basis.verify(self.root, "host", "archive", self.document)

    def test_new_facade_glue_and_generated_defaults_remain_in_the_admitted_import_closure(self) -> None:
        from scripts.capabilities.source_basis import Sources  # noqa: PLC0415 -- selected source-only owner

        files = {
            "scripts/bridge.py": "from pse.generated.project_operations import ProjectRuntimeOperations\n",
            "python/pse/generated/project_operations.py": "from .._project_operations_glue import ProjectRuntimeGlue\n",
            "python/pse/_project_operations_glue.py": "from ._project_bundle import ProjectBundle\nfrom .generated.project_operation_defaults import DEFAULT_BRANCH\n",
            "python/pse/_project_bundle.py": "class ProjectBundle: pass\n",
            "python/pse/generated/project_operation_defaults.py": "DEFAULT_BRANCH = 'main'\n",
        }
        for relative, body in files.items():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(body)
        captured = Sources(self.root, ("scripts/bridge.py",))
        closure = captured.closure(("scripts/bridge.py",))
        self.assertTrue(set(files).issubset(closure))
        self.definition = self.definition.model_copy(update={"sources": {path: captured.sources[path].digest for path in closure}})
        with patch.dict(os.environ, self.publish()):
            basis.verify(self.root, "host", "archive", self.document)
            (self.root / "python/pse/_project_bundle.py").write_text("class ProjectBundle: changed = True\n")
            with self.assertRaisesRegex(basis.AdmissionRefusedError, "source changed"):
                basis.verify(self.root, "host", "archive", self.document)

    def test_bare_package_import_captures_its_relative_private_owner(self) -> None:
        from scripts.capabilities.source_basis import Sources  # noqa: PLC0415 -- source-only capture

        files = {
            "scripts/bridge.py": "import pse\n",
            "python/pse/__init__.py": "from ._private import value\n",
            "python/pse/_private.py": "value = 1\n",
        }
        for relative, body in files.items():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(body)
        captured = Sources(self.root, ("scripts/bridge.py",))
        closure = captured.closure(("scripts/bridge.py",))
        self.assertTrue(set(files).issubset(closure))
        self.definition = self.definition.model_copy(update={"sources": {
            path: captured.sources[path].digest for path in closure
        }})
        with patch.dict(os.environ, self.publish()):
            basis.verify(self.root, "host", "archive", self.document)
            (self.root / "python/pse/_private.py").write_text("value = 2\n")
            with self.assertRaises(basis.AdmissionRefusedError):
                basis.verify(self.root, "host", "archive", self.document)

    def test_every_declared_native_binding_uses_the_same_closed_worker_set(
        self,
    ) -> None:
        from scripts.capabilities.native_contracts import (  # noqa: PLC0415 -- selected owner loads only at this boundary
            worker_bindings,
        )
        from scripts.capabilities.workers import (  # noqa: PLC0415 -- selected owner loads only at this boundary
            WORKERS,
        )

        self.assertEqual(
            WORKERS["native"].operations,
            frozenset(binding.operation for binding in worker_bindings() if binding.worker.worker == "native"),
        )
        self.assertEqual(
            WORKERS["native"].operations,
            frozenset(
                {
                    "source_admit",
                    "model_declarations",
                    "model_inspect",
                    "model_compare",
                    "preparation_discover",
                    "project_select",
                    "project_metadata",
                    "revision_source",
                    "revision_preparation_discover",
                    "project_bundle_inspect",
                    "research_read",
                    "result_read",
                    "result_inspect",
                    "physical_inspect",
                    "diagnostic_inspect",
                    "diagnostic_processes",
                    "diagnostic_checkpoint",
                    "evidence_inspect",
                    "invocation_read",
                    "configured_capabilities",
                    "telemetry_profile_capture",
                }
            ),
        )
        for binding in worker_bindings():
            with (
                self.subTest(operation=binding.operation),
                tempfile.TemporaryDirectory() as directory,
            ):
                with basis.selected(self.definition):
                    supplied = basis.publish(
                        Path(directory),
                        self.root,
                        binding.worker.worker,
                        binding.operation,
                        self.document,
                    )
                with patch.dict(os.environ, supplied):
                    basis.verify(self.root, binding.worker.worker, binding.operation, self.document)

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
            selected = basis.Definition(
                checkout="/selected",
                operation=name,
                wire_revision=basis.digest({}),
                sources={},
            )
            with basis.selected(selected):
                arrivals += 1
                if arrivals == 2:
                    ready.set()
                await asyncio.wait_for(ready.wait(), timeout=2)
                current = basis._CURRENT.get()  # noqa: SLF001 -- request-local admission owner
                assert current is not None
                return current.operation

        self.assertEqual(
            await asyncio.gather(request("one"), request("two")), ["one", "two"]
        )


class NativeCurrencyAdmissionTests(unittest.TestCase):
    def test_matching_schema_cannot_admit_unknown_artifact_before_runtime(self) -> None:
        import sys  # noqa: PLC0415 -- selected owner loads only at this boundary
        import types  # noqa: PLC0415 -- selected owner loads only at this boundary

        from scripts import (  # noqa: PLC0415 -- selected owner loads only at this boundary
            python_extension,
        )
        from scripts.capabilities import (  # noqa: PLC0415 -- selected owner loads only at this boundary
            native_worker,
        )

        fake = types.ModuleType("pse")
        fake.__dict__["SCHEMA_SET_FINGERPRINT"] = "same-schema"
        fake.__dict__["codec"] = object()
        fake.__dict__["diagnostics"] = object()
        contracts = types.ModuleType("pse.contracts")
        contracts.__dict__["documents"] = object()
        observation = python_extension.ExtensionCurrency(
            state="unknown",
            checkout="/selected",
            interpreter="/selected/.venv/bin/python",
            mode="native",
            profile=None,
            validated=True,
            cause="missing-preparation-evidence",
            remedy="Explicitly prepare and reattach.",
        )
        from scripts.tests.test_telemetry_profiles import envelope  # noqa: PLC0415 -- shared valid host-only fixture

        for operation in ("diagnostic_checkpoint", "telemetry_profile_capture"):
            with (
                self.subTest(operation=operation),
                patch.dict(sys.modules, {"pse": fake, "pse.contracts": contracts}),
                patch.dict(os.environ, {basis.ENVIRONMENT: "/private/admission.json"}),
                patch.object(
                    python_extension,
                    "currency_check_installed",
                    return_value=observation,
                ) as checked,
            ):
                result = native_worker.execute(
                    operation,
                    {
                        **(envelope() if operation == "telemetry_profile_capture" else {"checkpoint": "/selected/native.json"}),
                        "schema_set_fingerprint": "same-schema",
                    },
                    scratch=Path("/unused"),
                )
            self.assertEqual(result["code"], "pse.native.artifact_not_current")
            checked.assert_called_once()
