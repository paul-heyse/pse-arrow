# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Guidance preserves source ownership, bounds and loaded-revision coherence."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import json
import os
import tempfile
import unittest
from contextlib import contextmanager
from hashlib import sha256
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from fastmcp import Client, FastMCP
from mcp_types import TextResourceContents

from scripts.capabilities.guidance import (
    MAX_GUIDE_BYTES,
    Guidance,
    GuidanceError,
    register_guides,
)
from scripts.capabilities.model import Guide

if TYPE_CHECKING:
    from collections.abc import Generator, Iterator
    from typing import IO, Literal

    from scripts.capabilities.guidance import _Identity, _Snapshot


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
            first["digest"],
            sha256((self.root / self.first.path).read_bytes()).hexdigest(),
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
        self.assertFalse(uncaptured["restart_required"])
        self.assertTrue(uncaptured["complete"])
        self.assertEqual(uncaptured["content"], "# Second\n")
        self.assertIsNotNone(uncaptured["digest"])
        self.assertEqual(uncaptured["omissions"], [])
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
        for guide_id, code in [
            ("unknown", "unknown_guide"),
            ("missing", "missing_source"),
        ]:
            with (
                self.subTest(guide=guide_id),
                self.assertRaises(GuidanceError) as caught,
            ):
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
        open_binary = os.fdopen
        reads = 0

        @contextmanager
        def open_then_change(
            descriptor: int, mode: Literal["rb"]
        ) -> Generator[IO[bytes], None, None]:
            nonlocal reads
            with open_binary(descriptor, mode) as source:
                reads += 1
                if reads == 2:
                    (self.root / self.first.path).write_text("changed during bundle")
                yield source

        with patch(
            "scripts.capabilities.guidance.os.fdopen", side_effect=open_then_change
        ):
            bundle = guidance.instruction_bundle(["first", "second"])
        self.assertTrue(all(item["restart_required"] for item in bundle))
        self.assertTrue(all(not item["complete"] for item in bundle))

    def test_planning_skill_is_exact_owned_content_without_execution(self) -> None:
        root = Path(__file__).resolve().parents[2]
        skill = Guide(
            id="planning",
            title="Plan execution",
            kind="skill",
            path=".codex/skills/plan-execution/SKILL.md",
        )
        original = (root / skill.path).read_text()
        with (
            patch(
                "subprocess.run", side_effect=AssertionError("must not run commands")
            ),
            patch(
                "subprocess.Popen",
                side_effect=AssertionError("must not start services"),
            ),
            patch("os.system", side_effect=AssertionError("must not execute skills")),
        ):
            result = Guidance(root, [skill]).read("planning")
        self.assertEqual(result["content"], original)
        self.assertIn("plan-execution", str(result["content"]))
        self.assertIn("execute-plan", str(result["content"]))
        self.assertTrue(result["complete"])


class SkillPackageTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.directory = self.root / ".codex/skills/owned"
        self.directory.mkdir(parents=True)
        (self.directory / "SKILL.md").write_text(
            "# Owned\nRead [support](examples/support.md).\n"
        )
        (self.directory / "examples").mkdir()
        self.support = self.directory / "examples/support.md"
        self.support.write_text("# Complete supporting instructions\n")
        self.guide = Guide(
            id="owned", title="Owned", path=".codex/skills/owned/SKILL.md", kind="skill"
        )

    def test_lazy_manifest_and_deliberate_support_share_one_package_digest(
        self,
    ) -> None:
        with patch(
            "scripts.capabilities.guidance.os.fdopen",
            side_effect=AssertionError("startup must stay lazy"),
        ):
            guidance = Guidance(self.root, [self.guide])
        reference = guidance.references(["owned"])[0]
        self.assertNotIn("content", reference)
        manifest = guidance.manifest("owned")
        self.assertEqual(reference["package_digest"], manifest["package_digest"])
        self.assertEqual(reference["digest"], manifest["guide_digest"])
        self.assertEqual(
            {row["path"] for row in manifest["files"]},
            {"SKILL.md", "examples/support.md"},
        )
        supporting = next(
            row for row in manifest["files"] if row["path"] == "examples/support.md"
        )
        result = guidance.read_support("owned", supporting["id"])
        self.assertEqual(result.contents[0].content, self.support.read_text())
        self.assertEqual(result.contents[0].mime_type, "text/markdown")
        metadata = result.meta
        if metadata is None:
            self.fail("Supporting resources must retain captured source metadata.")
        self.assertEqual(
            metadata["digest"], sha256(self.support.read_bytes()).hexdigest()
        )
        self.assertEqual(metadata["package_digest"], manifest["package_digest"])
        with self.assertRaises(GuidanceError):
            guidance.read_support("owned", "../SKILL.md")

    def test_binary_support_is_bounded_and_served_with_its_exact_digest(self) -> None:
        data = b"\x89PNG\r\n\x1a\n\x00\xff"
        (self.directory / "diagram.png").write_bytes(data)
        guidance = Guidance(self.root, [self.guide])
        manifest = guidance.manifest("owned")
        support = next(row for row in manifest["files"] if row["path"] == "diagram.png")
        result = guidance.read_support("owned", support["id"])
        self.assertEqual(result.contents[0].content, data)
        self.assertEqual(result.contents[0].mime_type, "image/png")
        metadata = result.meta
        if metadata is None:
            self.fail(
                "Binary supporting resources must retain captured source metadata."
            )
        self.assertEqual(metadata["digest"], sha256(data).hexdigest())

    def test_support_changes_before_capture_refuse_new_main_and_manifest(self) -> None:
        for mutation in ("change", "missing", "added"):
            with self.subTest(mutation=mutation):
                self.support.write_text("# Original\n")
                extra = self.directory / "added.md"
                extra.unlink(missing_ok=True)
                guidance = Guidance(self.root, [self.guide])
                if mutation == "change":
                    self.support.write_text("# Changed\n")
                elif mutation == "missing":
                    self.support.unlink()
                else:
                    extra.write_text("new package member")
                main = guidance.read("owned")
                self.assertTrue(main["restart_required"])
                self.assertFalse(main["complete"])
                with self.assertRaises(GuidanceError) as caught:
                    guidance.manifest("owned")
                self.assertEqual(caught.exception.details["code"], "restart_required")

    def test_captured_main_is_retained_but_changed_support_reads_refuse(self) -> None:
        guidance = Guidance(self.root, [self.guide])
        main = guidance.read("owned")
        manifest = guidance.manifest("owned")
        supporting = next(
            row for row in manifest["files"] if row["path"] == "examples/support.md"
        )
        self.support.unlink()
        captured = guidance.read("owned")
        self.assertEqual(captured["content"], main["content"])
        self.assertEqual(captured["package_digest"], main["package_digest"])
        self.assertTrue(captured["restart_required"])
        with self.assertRaises(GuidanceError) as caught:
            guidance.read_support("owned", supporting["id"])
        self.assertEqual(caught.exception.details["code"], "restart_required")

    def test_streaming_entry_bound_refuses_before_sorting_or_descending(self) -> None:
        for index in range(24):
            (self.directory / ("empty-" + str(index))).mkdir()
        real_scandir = os.scandir
        scanned = 0
        yielded = 0

        @contextmanager
        def counted(
            descriptor: int,
        ) -> Generator[Iterator[os.DirEntry[str]], None, None]:
            nonlocal scanned
            scanned += 1
            with real_scandir(descriptor) as scan:

                def entries() -> Iterator[os.DirEntry[str]]:
                    nonlocal yielded
                    for entry in scan:
                        yielded += 1
                        self.assertLessEqual(
                            yielded, 5, "must stop at entry limit plus one"
                        )
                        yield entry

                yield entries()

        with (
            patch("scripts.capabilities.guidance.MAX_SKILL_ENTRIES", 4),
            patch("scripts.capabilities.guidance.os.scandir", side_effect=counted),
            patch(
                "scripts.capabilities.guidance.os.listdir",
                side_effect=AssertionError("must stream entries"),
            ),
            patch(
                "scripts.capabilities.guidance.sorted",
                side_effect=AssertionError("must refuse before sorting excess"),
                create=True,
            ),
            patch(
                "scripts.capabilities.guidance.os.fdopen",
                side_effect=AssertionError("must not read bodies"),
            ),
        ):
            guidance = Guidance(self.root, [self.guide])
            with self.assertRaises(GuidanceError) as caught:
                guidance.read("owned")
        self.assertEqual(caught.exception.details["code"], "oversized_package")
        self.assertEqual((scanned, yielded), (1, 5))

    def test_directory_bound_refuses_many_empty_directories_before_sorting(
        self,
    ) -> None:
        for index in range(24):
            (self.directory / ("empty-" + str(index))).mkdir()
        real_scandir = os.scandir
        scanned = 0
        yielded = 0

        @contextmanager
        def counted(
            descriptor: int,
        ) -> Generator[Iterator[os.DirEntry[str]], None, None]:
            nonlocal scanned
            scanned += 1
            with real_scandir(descriptor) as scan:

                def entries() -> Iterator[os.DirEntry[str]]:
                    nonlocal yielded
                    for entry in scan:
                        yielded += 1
                        self.assertLessEqual(
                            yielded,
                            4,
                            "two allowed directories plus one excess and main file",
                        )
                        yield entry

                yield entries()

        with (
            patch("scripts.capabilities.guidance.MAX_SKILL_DIRECTORIES", 2),
            patch("scripts.capabilities.guidance.os.scandir", side_effect=counted),
            patch(
                "scripts.capabilities.guidance.sorted",
                side_effect=AssertionError(
                    "must refuse directory excess before sorting"
                ),
                create=True,
            ),
            patch(
                "scripts.capabilities.guidance.os.fdopen",
                side_effect=AssertionError("must not read bodies"),
            ),
        ):
            guidance = Guidance(self.root, [self.guide])
            with self.assertRaises(GuidanceError) as caught:
                guidance.manifest("owned")
        self.assertEqual(caught.exception.details["code"], "oversized_package")
        self.assertEqual(scanned, 1)
        self.assertLessEqual(yielded, 4)

    def test_depth_bound_refuses_before_opening_excess_descendant(self) -> None:
        self.support.unlink()
        self.support.parent.rmdir()
        parent = self.directory
        for index in range(24):
            parent /= "level-" + str(index)
            parent.mkdir()
        real_scandir = os.scandir
        real_open = os.open
        scanned = 0
        opened: list[str | bytes | os.PathLike[str] | os.PathLike[bytes]] = []

        @contextmanager
        def counted_scan(
            descriptor: int,
        ) -> Generator[Iterator[os.DirEntry[str]], None, None]:
            nonlocal scanned
            scanned += 1
            self.assertLessEqual(scanned, 3, "root plus two allowed levels")
            with real_scandir(descriptor) as scan:
                yield scan

        def counted_open(
            path: str | bytes | os.PathLike[str] | os.PathLike[bytes],
            flags: int,
            *,
            dir_fd: int | None = None,
        ) -> int:
            opened.append(path)
            return real_open(path, flags, dir_fd=dir_fd)

        with (
            patch("scripts.capabilities.guidance.MAX_SKILL_DEPTH", 2),
            patch("scripts.capabilities.guidance.os.scandir", side_effect=counted_scan),
            patch("scripts.capabilities.guidance.os.open", side_effect=counted_open),
            patch(
                "scripts.capabilities.guidance.os.fdopen",
                side_effect=AssertionError("must not read bodies"),
            ),
        ):
            guidance = Guidance(self.root, [self.guide])
            with self.assertRaises(GuidanceError) as caught:
                guidance.read("owned")
        self.assertEqual(caught.exception.details["code"], "oversized_package")
        self.assertEqual(scanned, 3)
        self.assertNotIn("level-2", opened)

    def test_streaming_scan_refuses_directory_symlinks_before_following_them(
        self,
    ) -> None:
        real_scandir = os.scandir
        with tempfile.TemporaryDirectory() as outside:
            for target in (self.support.parent, Path(outside)):
                with self.subTest(target=target):
                    alias = self.directory / "alias-directory"
                    alias.symlink_to(target, target_is_directory=True)
                    with patch(
                        "scripts.capabilities.guidance.os.scandir", wraps=real_scandir
                    ) as scan:
                        guidance = Guidance(self.root, [self.guide])
                        with self.assertRaises(GuidanceError) as caught:
                            guidance.read("owned")
                    self.assertEqual(
                        caught.exception.details["code"], "package_symlink"
                    )
                    self.assertEqual(scan.call_count, 1)
                    alias.unlink()

    def test_bounded_membership_is_deterministic_and_exposes_traversal_limits(
        self,
    ) -> None:
        (self.directory / "z-last.md").write_text("last")
        (self.directory / "a-first.md").write_text("first")
        manifest = Guidance(self.root, [self.guide]).manifest("owned")
        paths = [item["path"] for item in manifest["files"]]
        self.assertEqual(paths, sorted(paths))
        self.assertEqual(manifest["limits"]["entries"], 128)
        self.assertEqual(manifest["limits"]["directories"], 64)
        self.assertEqual(manifest["limits"]["depth"], 8)

    def test_package_size_count_and_symlink_refusals(self) -> None:
        cases = (
            (1, MAX_GUIDE_BYTES + 1, "oversized_source"),
            (64, 1, "oversized_package"),
            (8, MAX_GUIDE_BYTES, "oversized_package"),
        )
        for count, size, expected in cases:
            with self.subTest(count=count, size=size):
                for path in self.directory.glob("bound-*"):
                    path.unlink()
                for index in range(count):
                    (self.directory / ("bound-" + str(index))).write_text("x" * size)
                guidance = Guidance(self.root, [self.guide])
                with self.assertRaises(GuidanceError) as caught:
                    guidance.read("owned")
                self.assertEqual(caught.exception.details["code"], expected)
        for path in self.directory.glob("bound-*"):
            path.unlink()
        (self.directory / "alias.md").symlink_to(self.support)
        with self.assertRaises(GuidanceError) as caught:
            Guidance(self.root, [self.guide]).read("owned")
        self.assertEqual(caught.exception.details["code"], "package_symlink")

    def test_change_during_package_capture_publishes_no_partial_snapshot(self) -> None:
        guidance = Guidance(self.root, [self.guide])
        capture = guidance._capture  # noqa: SLF001 -- inject a race at the snapshot capture boundary.

        def mutate(guide: Guide, baseline: _Identity) -> _Snapshot | None:
            result = capture(guide, baseline)
            if guide.path.endswith("SKILL.md"):
                self.support.write_text("changed while capturing main")
            return result

        with patch.object(guidance, "_capture", side_effect=mutate):
            result = guidance.read("owned")
        self.assertFalse(result["complete"])
        self.assertTrue(result["restart_required"])
        self.assertEqual(guidance._package_snapshots, {})  # noqa: SLF001 -- prove the injected race published no partial package.


class GuidanceResourceTests(unittest.IsolatedAsyncioTestCase):
    async def test_resources_are_lazy_exact_and_readable_through_mcp(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "guide.md").write_text("# Owned\n[More](other.md)\n")
            guide = Guide(
                id="owned", title="Owned guide", path="guide.md", kind="skill"
            )
            server = FastMCP("guidance-test")
            with patch(
                "scripts.capabilities.guidance.os.fdopen",
                side_effect=AssertionError(
                    "registration and listing must not read bodies"
                ),
            ):
                guidance = Guidance(root, [guide])
                keys = register_guides(server, guidance)
                async with Client(server) as client:
                    resources = await client.list_resources()
            self.assertEqual(keys, {"owned": "resource:pse://guidance/owned@"})
            self.assertEqual(resources[0].uri, guide.uri)
            self.assertEqual(resources[0].mime_type, "application/json")
            async with Client(server) as client:
                result = await client.read_resource(guide.uri)
            content = result[0]
            if not isinstance(content, TextResourceContents):
                self.fail("Guidance must be returned as text resource content.")
            payload = json.loads(content.text)
            self.assertEqual(payload["content"], "# Owned\n[More](other.md)\n")
            self.assertTrue(payload["complete"])
            self.assertFalse(payload["restart_required"])


if __name__ == "__main__":
    unittest.main()
