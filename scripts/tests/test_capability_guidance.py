# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Guidance preserves source ownership, bounds and loaded-revision coherence."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import json
import tempfile
import unittest
from hashlib import sha256
from pathlib import Path
from unittest.mock import patch

from fastmcp import Client, FastMCP

from scripts.capabilities.guidance import (
    MAX_GUIDE_BYTES,
    Guidance,
    GuidanceError,
    _Identity,
    _Snapshot,
    register_guides,
)
from scripts.capabilities.model import Guide


class GuidanceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.first = Guide(id="first", title="First guide", path="first.md")
        self.second = Guide(id="second", title="Second guide", path="second.md")
        (self.root / self.first.path).write_text("# First\n[Owner](owner.md)\n")
        (self.root / self.second.path).write_text("# Second\n")

    def test_complete_markdown_digest_and_ordered_deduplicated_bundle(self) -> None:
        guidance = Guidance(self.root, [self.first, self.second])
        bundle = guidance.instruction_bundle(["second", "first", "second"])
        self.assertEqual([item["id"] for item in bundle], ["second", "first"])
        first = bundle[1]
        self.assertEqual(first["content"], "# First\n[Owner](owner.md)\n")
        self.assertEqual(
            first["digest"], sha256((self.root / self.first.path).read_bytes()).hexdigest()
        )
        self.assertEqual(first["path"], "first.md")
        self.assertEqual(first["uri"], "pse://guidance/first")
        self.assertTrue(first["complete"])
        self.assertFalse(first["restart_required"])
        self.assertEqual(first["omissions"], [])
        json.dumps(bundle)

    def test_changed_captured_guide_keeps_original_and_latches_restart(self) -> None:
        guidance = Guidance(self.root, [self.first, self.second])
        first = guidance.read("first")
        (self.root / self.first.path).write_text("# Revised\n")
        changed = guidance.read("first")
        self.assertEqual(changed["content"], first["content"])
        self.assertEqual(changed["digest"], first["digest"])
        self.assertTrue(changed["complete"])
        self.assertTrue(changed["restart_required"])
        uncaptured = guidance.read("second")
        self.assertTrue(uncaptured["restart_required"])
        self.assertFalse(uncaptured["complete"])
        self.assertEqual(uncaptured["content"], "")
        self.assertIsNone(uncaptured["digest"])
        self.assertTrue(uncaptured["omissions"])
        (self.root / self.first.path).write_text(str(first["content"]))
        self.assertTrue(guidance.read("first")["restart_required"])

    def test_change_before_first_read_refuses_new_instructions(self) -> None:
        guidance = Guidance(self.root, [self.first])
        (self.root / self.first.path).write_text("# Revised before selection\n")
        result = guidance.read("first")
        self.assertTrue(result["restart_required"])
        self.assertFalse(result["complete"])
        self.assertEqual(result["content"], "")

    def test_deleted_captured_guide_returns_snapshot(self) -> None:
        guidance = Guidance(self.root, [self.first])
        first = guidance.read("first")
        (self.root / self.first.path).unlink()
        result = guidance.read("first")
        self.assertEqual(result["content"], first["content"])
        self.assertTrue(result["restart_required"])

    def test_unknown_and_missing_sources_are_structured_value_errors(self) -> None:
        missing = Guide(id="missing", title="Missing", path="missing.md")
        guidance = Guidance(self.root, [missing])
        for guide_id, code in [("unknown", "unknown_guide"), ("missing", "missing_source")]:
            with self.subTest(guide=guide_id), self.assertRaises(GuidanceError) as caught:
                guidance.read(guide_id)
            self.assertIsInstance(caught.exception, ValueError)
            self.assertEqual(caught.exception.details["code"], code)
            self.assertEqual(caught.exception.args[0]["guide_id"], guide_id)
        (self.root / missing.path).write_text("# Appeared later\n")
        self.assertTrue(guidance.read("missing")["restart_required"])
        self.assertFalse(guidance.read("missing")["complete"])

    def test_arbitrary_paths_and_symlink_escape_are_refused(self) -> None:
        for path in ["../outside.md", str(self.root / "first.md"), "first.py"]:
            with self.subTest(path=path), self.assertRaises(GuidanceError) as caught:
                Guidance(self.root, [Guide(id="bad", title="Bad", path=path)])
            self.assertEqual(caught.exception.details["code"], "invalid_path")
        with tempfile.TemporaryDirectory() as outside:
            source = Path(outside) / "secret.md"
            source.write_text("outside content")
            (self.root / "escape.md").symlink_to(source)
            guide = Guide(id="escape", title="Escape", path="escape.md")
            guidance = Guidance(self.root, [guide])
            with self.assertRaises(GuidanceError) as caught:
                guidance.read("escape")
            self.assertEqual(caught.exception.details["code"], "path_escape")

    def test_symlink_swap_after_construction_cannot_read_outside(self) -> None:
        guidance = Guidance(self.root, [self.first])
        with tempfile.TemporaryDirectory() as outside:
            source = Path(outside) / "secret.md"
            source.write_text("outside content")
            (self.root / self.first.path).unlink()
            (self.root / self.first.path).symlink_to(source)
            result = guidance.read("first")
        self.assertTrue(result["restart_required"])
        self.assertEqual(result["content"], "")

    def test_large_instructions_are_complete_and_oversized_ones_refuse(self) -> None:
        (self.root / self.first.path).write_text("x" * (40 * 1024))
        guidance = Guidance(self.root, [self.first])
        self.assertEqual(len(str(guidance.read("first")["content"])), 40 * 1024)
        (self.root / self.first.path).write_text("x" * (MAX_GUIDE_BYTES + 1))
        oversized = Guidance(self.root, [self.first])
        with self.assertRaises(GuidanceError) as caught:
            oversized.read("first")
        self.assertEqual(caught.exception.details["code"], "oversized_source")

    def test_source_changed_during_capture_never_publishes_partial_bundle(self) -> None:
        guidance = Guidance(self.root, [self.first, self.second])
        capture = guidance._capture

        def capture_then_change(guide: Guide, baseline: _Identity) -> _Snapshot | None:
            result = capture(guide, baseline)
            if guide.id == "second":
                (self.root / self.first.path).write_text("changed during bundle")
            return result

        with patch.object(guidance, "_capture", side_effect=capture_then_change):
            bundle = guidance.instruction_bundle(["first", "second"])
        self.assertTrue(all(item["restart_required"] for item in bundle))
        self.assertTrue(all(not item["complete"] for item in bundle))

    def test_planning_skill_is_exact_owned_content_without_execution(self) -> None:
        root = Path(__file__).resolve().parents[2]
        skill = Guide(
            id="planning", title="Plan execution", kind="skill",
            path=".codex/skills/plan-execution/SKILL.md",
        )
        original = (root / skill.path).read_text()
        with (
            patch("subprocess.run", side_effect=AssertionError("must not run commands")),
            patch("subprocess.Popen", side_effect=AssertionError("must not start services")),
            patch("os.system", side_effect=AssertionError("must not execute skills")),
        ):
            result = Guidance(root, [skill]).read("planning")
        self.assertEqual(result["content"], original)
        self.assertIn("plan-execution", str(result["content"]))
        self.assertIn("execute-plan", str(result["content"]))
        self.assertTrue(result["complete"])


class GuidanceResourceTests(unittest.IsolatedAsyncioTestCase):
    async def test_resources_are_lazy_exact_and_readable_through_mcp(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "guide.md").write_text("# Owned\n[More](other.md)\n")
            guide = Guide(id="owned", title="Owned guide", path="guide.md", kind="skill")
            server = FastMCP("guidance-test")
            with patch(
                "scripts.capabilities.guidance.os.fdopen",
                side_effect=AssertionError("registration and listing must not read bodies"),
            ):
                guidance = Guidance(root, [guide])
                keys = register_guides(server, guidance)
                async with Client(server) as client:
                    resources = await client.list_resources()
            self.assertEqual(keys, {"owned": "resource:pse://guidance/owned@"})
            self.assertEqual(str(resources[0].uri), guide.uri)
            self.assertEqual(resources[0].mime_type, "application/json")
            async with Client(server) as client:
                result = await client.read_resource(guide.uri)
            payload = json.loads(result[0].text)
            self.assertEqual(payload["content"], "# Owned\n[More](other.md)\n")
            self.assertTrue(payload["complete"])
            self.assertFalse(payload["restart_required"])


if __name__ == "__main__":
    unittest.main()
