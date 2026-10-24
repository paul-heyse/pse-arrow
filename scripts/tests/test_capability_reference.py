# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Complete source-derived reference and nonmutating drift comparison."""
# ruff: noqa: PT009, PT027 -- stdlib host controls

from __future__ import annotations

import asyncio
import json
import re
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

from fastmcp import Client, FastMCP
from fastmcp.server.transforms import ResourcesAsTools

from scripts.capabilities import reference
from scripts.capabilities.catalogue import Catalogue
from scripts.capabilities.model import Capability
from scripts.capabilities.server import build

ROOT = Path(__file__).resolve().parents[2]
EMPTY: list[str] = []


class ReferenceTests(unittest.TestCase):
    def test_complete_composition_includes_hidden_and_noncallable_definitions(self) -> None:
        runtime = build()
        content = reference.render(runtime)
        for identity, capability in runtime.catalogue.capabilities.items():
            self.assertIn(f"### `{identity}` — {capability.title}", content)
            self.assertIn(f'"exposure": "{capability.exposure}"', content)
        self.assertTrue(any(item.exposure == "native" for item in runtime.catalogue.capabilities.values()))
        for tool in runtime.catalogue.tools.values():
            self.assertIn(f"Exact callable component key: `{tool.key}`", content)
            section = content.split(f"### `{tool.name}` —", 1)[1].split("\n### ", 1)[0]
            blocks = re.findall(r"```json\n(.*?)\n```", section, flags=re.DOTALL)
            actual = json.loads(blocks[1])
            self.assertEqual(actual["inputSchema"], tool.parameters)
            self.assertEqual(actual.get("outputSchema"), tool.output_schema)
        self.assertIn("### `capabilities_domains`", content)
        self.assertIn('"default": 20', content)
        self.assertIn('"default": false', content)
        self.assertIn('"maximum": 100', content)
        self.assertIn("Exposure: **native**", content)
        self.assertIn("docs/generated/schema/project-select-request.input.schema.json", content)
        self.assertIn("AGENTS.md](../../../AGENTS.md)", content)
        self.assertNotIn(str(ROOT), content)

    def test_schema_retains_required_nullable_defaults_and_empty_values(self) -> None:
        runtime = build()

        @runtime.server.tool
        def reference_fixture(
            required_nullable: str | None,
            nullable_default: str | None = None,
            empty: list[str] = EMPTY,
        ) -> dict[str, str | None]:
            """Signature fidelity fixture; its body must never run."""
            raise AssertionError(f"reference executed the tool: {required_nullable}, {nullable_default}, {empty}")

        content = reference.render(runtime)
        section = content.split("### `reference_fixture`\n", 1)[1].split("\n### ", 1)[0]
        self.assertIn('"required": [\n      "required_nullable"\n    ]', section)
        self.assertIn('"default": null', section)
        self.assertIn('"default": []', section)
        self.assertIn('"type": "null"', section)

    def test_deferred_declaration_retains_reason_and_owner_without_callable_schema(self) -> None:
        runtime = build()
        spec = Capability(
            id="reference-deferred-fixture",
            title="Deferred fixture",
            description="Not a callable operation.",
            domain=next(iter(runtime.catalogue.domains)),
            owner="native owner",
            source="scripts/capabilities/reference.py",
            exposure="deferred",
            effects="No callable binding.",
            reason="Awaiting the concrete receiver contract.",
            work_owner="fixture owning plan",
        )
        previous = runtime.catalogue
        catalogue = Catalogue(
            tuple(previous.domains.values()),
            tuple(previous.workflows.values()),
            tuple(previous.guides.values()),
            (*previous.capabilities.values(), spec),
            previous.tools,
        )
        content = reference.render(replace(runtime, catalogue=catalogue))
        section = content.split("### `reference-deferred-fixture`", 1)[1].split("\n### ", 1)[0]
        self.assertIn("Exposure: **deferred**", section)
        self.assertIn(spec.reason, section)
        self.assertIn(spec.work_owner, section)
        self.assertNotIn("Exact callable component key:", section)

    def test_selection_and_reset_do_not_change_reference(self) -> None:
        runtime = build()

        async def exercise() -> None:
            before = await reference.render_async(runtime)
            async with Client(runtime.server) as client:
                hidden = next(iter(runtime.catalogue.tools))
                self.assertNotIn(hidden, {tool.name for tool in await client.list_tools()})
                await client.call_tool("capabilities_select", {"capabilities": [hidden]})
                self.assertEqual(before, await reference.render_async(runtime))
                await client.call_tool("capabilities_reset", {})
                self.assertEqual(before, await reference.render_async(runtime))

        asyncio.run(exercise())

    def test_registered_resources_templates_and_prompts_are_described_without_execution(self) -> None:
        runtime = build()

        @runtime.server.resource("pse://reference-fixture")
        def resource_fixture() -> str:
            raise AssertionError("reference read a resource")

        @runtime.server.resource("pse://reference-fixture/{identity}")
        def template_fixture(identity: str) -> str:
            raise AssertionError("reference expanded a template: " + identity)

        @runtime.server.prompt
        def prompt_fixture(identity: str) -> str:
            raise AssertionError("reference rendered a prompt: " + identity)

        runtime.server.disable(names={"prompt_fixture"}, components={"prompt"})
        content = reference.render(runtime)
        self.assertIn("pse://reference-fixture", content)
        self.assertIn("pse://reference-fixture/{identity}", content)
        self.assertIn('"name": "prompt_fixture"', content)
        self.assertIn('"required": true', content)

    def test_composed_provider_and_framework_projection_controls_are_included(self) -> None:
        runtime = build()
        child = FastMCP("reference-child", mask_error_details=True)

        @child.tool
        def composed_fixture() -> str:
            raise AssertionError("reference invoked a composed provider")

        runtime.server.mount(child, namespace="reference")
        runtime.server.add_transform(ResourcesAsTools(runtime.server))
        content = reference.render(runtime)
        self.assertIn("### `reference_composed_fixture`", content)
        self.assertIn("### `read_resource`", content)
        self.assertIn("### `list_resources`", content)

    def test_source_only_construction_and_render_never_launch_or_import_native(self) -> None:
        before = {name for name in sys.modules if name == "pse" or name.startswith("pse.")}

        with (
            patch.dict(sys.modules, {"pse": None, "pse._native": None}),
            patch("subprocess.Popen", side_effect=AssertionError("child process launched")),
            patch("subprocess.run", side_effect=AssertionError("child process run")),
        ):
            content = reference.render(build())
        self.assertIn(reference.MARKER, content)
        self.assertEqual(before, {name for name in sys.modules if name == "pse" or name.startswith("pse.")})

    def test_check_reports_missing_extra_and_stale_without_mutation(self) -> None:
        content = reference.MARKER + "\n# Fixture\n"
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / "output"
            self.assertEqual(reference.check(content, directory), ("missing generated reference: reference.md",))
            self.assertFalse(directory.exists())
            reference.publish(content, directory)
            self.assertEqual(reference.check(content, directory), ())
            target = directory / reference.FILENAME
            extra = directory / "obsolete.md"
            extra.write_text(content)
            target.write_text("stale\n")
            before = {path.name: (path.read_bytes(), path.stat().st_mtime_ns) for path in directory.iterdir()}
            self.assertEqual(reference.check(content, directory), (
                "unexpected generated reference: obsolete.md",
                "stale generated reference: reference.md",
            ))
            self.assertEqual(before, {path.name: (path.read_bytes(), path.stat().st_mtime_ns) for path in directory.iterdir()})
            unrelated = directory / "other-owner.md"
            unrelated.write_text("Other generator owns this.\n")
            reference.publish(content, directory)
            self.assertFalse(extra.exists())
            self.assertEqual(unrelated.read_text(), "Other generator owns this.\n")
            self.assertEqual(reference.check(content, directory), ("unexpected generated reference: other-owner.md",))

    def test_check_inventories_unmarked_and_nested_outputs_without_mutation(self) -> None:
        content = reference.MARKER + "\n# Fixture\n"
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / "output"
            reference.publish(content, directory)
            (directory / "extra.md").write_text("unmarked Markdown\n")
            (directory / "extra.json").write_bytes(b"\xff\x00unmarked binary\n")
            nested = directory / "nested"
            nested.mkdir()
            (nested / "unknown.json").write_text("{}\n")
            before = {path.relative_to(directory): (path.read_bytes(), path.stat().st_mtime_ns) for path in directory.rglob("*") if path.is_file()}
            self.assertEqual(reference.check(content, directory), (
                "unexpected generated reference: extra.json",
                "unexpected generated reference: extra.md",
                "unexpected generated reference: nested/unknown.json",
                "unexpected generated reference directory: nested",
            ))
            self.assertEqual(before, {path.relative_to(directory): (path.read_bytes(), path.stat().st_mtime_ns) for path in directory.rglob("*") if path.is_file()})
            reference.publish(content, directory)
            self.assertEqual(before, {path.relative_to(directory): (path.read_bytes(), path.stat().st_mtime_ns) for path in directory.rglob("*") if path.is_file()})

    def test_check_rejects_file_directory_and_target_symlinks_without_external_reads(self) -> None:
        content = reference.MARKER + "\n# Fixture\n"
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            directory = root / "output"
            reference.publish(content, directory)
            external = root / "external"
            external.mkdir()
            secret = external / "secret.md"
            secret.write_text("external content must not be read\n")
            (directory / "file.md").symlink_to(secret)
            (directory / "linked-directory").symlink_to(external, target_is_directory=True)
            target = directory / "reference.md"
            target.unlink()
            target.symlink_to(secret)
            original_read = Path.read_bytes

            def guarded_read(path: Path) -> bytes:
                if path.is_symlink() or path.is_relative_to(external):
                    raise AssertionError("check read outside its generated namespace")
                return original_read(path)

            before = {path.name: path.lstat() for path in directory.iterdir()}
            with patch.object(Path, "read_bytes", guarded_read):
                self.assertEqual(reference.check(content, directory), (
                    "symlink generated reference: file.md",
                    "symlink generated reference: linked-directory",
                    "symlink generated reference: reference.md",
                    "missing generated reference: reference.md",
                ))
                linked_root = root / "linked-output"
                linked_root.symlink_to(external, target_is_directory=True)
                self.assertEqual(reference.check(content, linked_root), (
                    f"symlink generated reference path: {linked_root}",
                    "missing generated reference: reference.md",
                ))
                with self.assertRaisesRegex(ValueError, "symlink generated reference"):
                    reference.publish(content, directory)
            self.assertEqual(before, {path.name: path.lstat() for path in directory.iterdir()})
            self.assertEqual(secret.read_text(), "external content must not be read\n")

    def test_default_owner_migrates_only_exact_marker_owned_legacy_output(self) -> None:
        content = reference.MARKER + "\n# Fixture\n"
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            directory = root / "docs/generated/capabilities"
            legacy = root / "docs/generated/capabilities.md"
            legacy.parent.mkdir(parents=True)
            legacy.write_text(content)
            before = (legacy.read_bytes(), legacy.stat().st_mtime_ns)
            with patch.object(reference, "ROOT", root):
                self.assertEqual(reference.check(content, directory), (
                    "missing generated reference: reference.md",
                    "legacy generated reference: docs/generated/capabilities.md",
                ))
                self.assertEqual(before, (legacy.read_bytes(), legacy.stat().st_mtime_ns))
                self.assertFalse(directory.exists())
                # Alternate output destinations do not retire canonical artifacts.
                reference.publish(content, root / "alternate")
                self.assertTrue(legacy.exists())
                reference.publish(content, directory)
                self.assertFalse(legacy.exists())
                self.assertEqual((directory / "reference.md").read_text(), content)
                self.assertEqual(reference.check(content, directory), ())
                legacy.write_text("A different owner has this path.\n")
                reference.publish(content, directory)
                self.assertEqual(legacy.read_text(), "A different owner has this path.\n")
                self.assertEqual(reference.check(content, directory), ())

    def test_main_default_publishes_dedicated_nested_owner(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with patch.object(reference, "ROOT", root):
                self.assertEqual(reference.main([]), 0)
                target = root / "docs/generated/capabilities/reference.md"
                self.assertTrue(target.is_file())
                self.assertFalse((root / "docs/generated/capabilities.md").exists())
                self.assertEqual(reference.main(["--check"]), 0)

    def test_main_check_compares_without_publication(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / "output"
            with patch("builtins.print"):
                self.assertEqual(reference.main(["--check", "--output-dir", str(directory)]), 1)
            self.assertFalse(directory.exists())
            self.assertEqual(reference.main(["--output-dir", str(directory)]), 0)
            target = directory / reference.FILENAME
            before = (target.read_bytes(), target.stat().st_mtime_ns)
            self.assertEqual(reference.main(["--check", "--output-dir", str(directory)]), 0)
            self.assertEqual(before, (target.read_bytes(), target.stat().st_mtime_ns))


if __name__ == "__main__":
    unittest.main()
