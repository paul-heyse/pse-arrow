# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent source/module and actual binding dependency expectations."""
# ruff: noqa: PT009 -- stdlib source-ownership controls

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock, patch

from fastmcp import Client, FastMCP

from scripts.capabilities import entrypoints, process, server
from scripts.capabilities.coherence import Interface, Sources, install
from scripts.capabilities.source_modules import module_sources

ROOT = Path(__file__).resolve().parents[2]


class OwnershipTests(unittest.TestCase):
    def test_ordinary_nested_package_and_namespace_launch_owners(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name in (
                "scripts/__init__.py",
                "scripts/plain.py",
                "scripts/deep/__init__.py",
                "scripts/deep/leaf.py",
                "scripts/capabilities/__init__.py",
                "scripts/capabilities/__main__.py",
                "scripts/namespace/__main__.py",
            ):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text('raise AssertionError("discovery executed source")\n')
            self.assertEqual(
                module_sources(root, "scripts.plain", launch=True),
                ("scripts/__init__.py", "scripts/plain.py"),
            )
            self.assertEqual(
                module_sources(root, "scripts.deep.leaf", launch=True),
                (
                    "scripts/__init__.py",
                    "scripts/deep/__init__.py",
                    "scripts/deep/leaf.py",
                ),
            )
            self.assertEqual(
                module_sources(root, "scripts.capabilities", launch=True),
                (
                    "scripts/__init__.py",
                    "scripts/capabilities/__init__.py",
                    "scripts/capabilities/__main__.py",
                ),
            )
            self.assertEqual(
                module_sources(root, "scripts.namespace", launch=True),
                ("scripts/__init__.py", "scripts/namespace/__main__.py"),
            )
            self.assertEqual(module_sources(root, "scripts.absent", launch=True), ())
            self.assertEqual(module_sources(root, "scripts...outside", launch=True), ())
            leads = entrypoints._leaf_sources(  # noqa: SLF001 -- exercise the non-executing recipe inference kernel.
                root, {"body": [["python -m scripts.deep.leaf"]]}, None
            )
            self.assertIn("scripts/deep/leaf.py", leads)
            self.assertNotIn("scripts/deep.py", leads)

    def test_new_module_or_package_membership_invalidates_absence_cut(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "scripts").mkdir()
            owner = root / "scripts/owner.py"
            owner.write_text("from scripts import candidate\n")
            cut = Sources(root, ("scripts/owner.py",))
            self.assertFalse(cut.check(cut.sources))
            (root / "scripts/candidate.py").write_text("VALUE = 1\n")
            self.assertIn(
                "scripts/candidate.py", cut.check(cut.closure(("scripts/owner.py",)))
            )

    def test_actual_pure_schema_is_independent_of_workers_and_coverage(self) -> None:
        runtime = server.build(ROOT)
        interface = Interface(
            Sources(ROOT, runtime.interface.sources.sources), runtime.catalogue
        )
        schema = interface.operations["native_contract_schema"]
        self.assertNotIn("scripts/capabilities/native_worker.py", schema)
        self.assertNotIn("scripts/coverage_correspondence.py", schema)
        self.assertNotIn("scripts/capabilities/native.py", schema)
        self.assertIn("scripts/capabilities/schema_bindings.py", schema)
        self.assertIn(
            "scripts/capabilities/native_worker.py",
            interface.operations["native_diagnostic_inspect"],
        )
        self.assertNotIn(
            "scripts/capabilities/host_worker.py",
            interface.operations["experiment_inspect"],
        )
        self.assertIn(
            "scripts/capabilities/host_worker.py",
            interface.operations["experiment_archive"],
        )
        original = interface.sources._read_optional  # noqa: SLF001 -- inject changed source bytes at the captured-cut boundary.

        def changed(path: str) -> bytes | None:
            raw = original(path)
            return (
                (raw or b"") + b"\n# owner changed\n"
                if path
                in {
                    "scripts/coverage_correspondence.py",
                    "scripts/capabilities/native_worker.py",
                }
                else raw
            )

        with patch.object(interface.sources, "_read_optional", side_effect=changed):
            self.assertFalse(
                interface.status("native_contract_schema")["restart_required"]
            )
            self.assertFalse(
                interface.status("environment_explain")["restart_required"]
            )
            self.assertFalse(
                interface.status("workspace_inventory")["restart_required"]
            )
            self.assertTrue(
                interface.status("native_diagnostic_inspect")["restart_required"]
            )
            self.assertTrue(
                interface.status("coverage_correspondence_read")["restart_required"]
            )


class RecipeFenceTests(unittest.IsolatedAsyncioTestCase):
    async def test_actual_recipe_edit_refuses_before_fixed_worker_launch(self) -> None:
        runtime = server.build(ROOT)
        cut = Sources(ROOT, runtime.interface.sources.sources)
        endpoint = FastMCP("actual-recipe-fence")
        endpoint.add_tool(runtime.catalogue.tools["compilation_check"])
        interface = install(endpoint, cut, runtime.catalogue)
        self.assertIn("justfile", interface.operations["compilation_check"])
        self.assertNotIn("justfile", interface.operations["native_contract_schema"])
        original = cut._read_optional  # noqa: SLF001 -- inject recipe drift before worker admission.

        def changed(path: str) -> bytes | None:
            raw = original(path)
            return (
                (raw or b"") + b"\n# changed recipe interpretation\n"
                if path == "justfile"
                else raw
            )

        with (
            patch.object(cut, "_read_optional", side_effect=changed),
            patch.object(process, "run_worker", new_callable=AsyncMock) as launch,
        ):
            async with Client(endpoint) as client:
                result = await client.call_tool(
                    "compilation_check",
                    {"request": {"output": "build/never-issued"}},
                    raise_on_error=False,
                )
            self.assertTrue(result.is_error)
            self.assertIn("restart_required", str(result.content))
            launch.assert_not_awaited()
