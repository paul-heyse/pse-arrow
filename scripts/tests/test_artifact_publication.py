# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Artifact-constructor publication, failure settlement and issued-generation controls."""
# ruff: noqa: PT009, PT027 -- standard-library constructor controls

from __future__ import annotations

import io
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import artifact_publication


class ArtifactPublicationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.source = self.root / "_native.abi3.so"
        self.source.write_bytes(b"first completed image")
        self.receipt = self.root / "owner/extension.json"

    def publish(self) -> artifact_publication.Publication:
        return artifact_publication.publish(
            self.receipt, kind="extension", source=self.source, mode="dev", profile="dev",
            validated=True, interpreter="/selected/python", features=lambda _candidate: ["force-validate"],
        )

    def test_copy_precedes_single_candidate_probe_and_atomic_publication(self) -> None:
        probes = []

        def describe(candidate: Path) -> list[str]:
            self.assertNotEqual(candidate, self.source)
            self.assertEqual(candidate.read_bytes(), b"first completed image")
            self.assertFalse(self.receipt.exists())
            probes.append(candidate)
            return ["force-validate"]

        result = artifact_publication.publish(
            self.receipt, kind="extension", source=self.source, mode="dev", profile="release",
            validated=True, interpreter="/selected/python", features=describe,
        )
        self.assertEqual(probes, [Path(result.artifact.binary)])
        self.assertEqual(artifact_publication.read(self.receipt), result)
        self.assertEqual(Path(result.artifact.binary).stat().st_mode & 0o777, 0o500)

    def test_issued_generation_survives_source_and_lint_alias_changes(self) -> None:
        first = self.publish()
        self.source.write_bytes(b"later build alias")
        (self.root / "lint-stdout").write_text("independent Cargo control output")
        with (
            patch.object(artifact_publication.hashlib, "sha256", side_effect=AssertionError("consumer digest")),
            patch.object(Path, "stat", side_effect=AssertionError("consumer stat")),
        ):
            self.assertEqual(artifact_publication.read(self.receipt), first)
        self.assertEqual(Path(first.artifact.binary).read_bytes(), b"first completed image")
        second = self.publish()
        self.assertNotEqual(first.generation, second.generation)
        self.assertEqual(Path(second.artifact.binary).read_bytes(), b"later build alias")
        self.assertEqual(Path(first.artifact.binary).read_bytes(), b"first completed image")

    def test_partial_copy_failure_settles_only_new_candidate(self) -> None:
        first = self.publish()
        previous = self.receipt.read_bytes()
        raw = b"x" * ((1 << 20) + 1)
        self.source.write_bytes(raw)
        actual_open = Path.open

        class BrokenReader(io.BytesIO):
            def read(self, size: int = -1) -> bytes:
                if self.tell():
                    raise OSError("copy interrupted")
                return super().read(size)

        def open_file(path: Path, *args: object, **kwargs: object) -> object:
            return BrokenReader(raw) if path == self.source else actual_open(path, *args, **kwargs)

        with patch.object(Path, "open", open_file), self.assertRaisesRegex(OSError, "copy interrupted"):
            self.publish()
        self.assertEqual(self.receipt.read_bytes(), previous)
        self.assertEqual(list((self.receipt.parent / "extension.generations").iterdir()), [Path(first.artifact.binary).parent])

    def test_probe_and_descriptor_failures_preserve_previous_publication(self) -> None:
        first = self.publish()
        previous = self.receipt.read_bytes()
        for failure_stage in ("probe", "serialize", "replace"):
            with self.subTest(stage=failure_stage):
                if failure_stage == "probe":
                    def describe(_candidate: Path) -> list[str]:
                        raise ValueError("probe failed")
                    scope = patch.object(artifact_publication.os, "replace", wraps=artifact_publication.os.replace)
                elif failure_stage == "serialize":
                    describe = lambda _candidate: ["force-validate"]
                    scope = patch.object(artifact_publication.Publication, "model_dump_json", side_effect=ValueError("serialize failed"))
                else:
                    describe = lambda _candidate: ["force-validate"]
                    scope = patch.object(artifact_publication.os, "replace", side_effect=ValueError("replace failed"))
                with scope, self.assertRaises(ValueError):
                    artifact_publication.publish(
                        self.receipt, kind="extension", source=self.source, mode="dev", profile="dev",
                        validated=True, interpreter="/selected/python", features=describe,
                    )
                self.assertEqual(self.receipt.read_bytes(), previous)
                generations = list((self.receipt.parent / "extension.generations").iterdir())
                self.assertIn(Path(first.artifact.binary).parent, generations)
                self.assertEqual(len(generations), 2 if failure_stage == "replace" else 1)

    def test_replace_then_interrupt_retains_both_possible_issued_generations(self) -> None:
        first = self.publish()
        actual_replace = artifact_publication.os.replace

        def interrupted(source: Path, target: Path) -> None:
            actual_replace(source, target)
            raise KeyboardInterrupt("publication replaced before interruption")

        with patch.object(artifact_publication.os, "replace", side_effect=interrupted), self.assertRaises(KeyboardInterrupt):
            self.publish()
        selected = artifact_publication.read(self.receipt)
        assert selected is not None
        self.assertNotEqual(selected.generation, first.generation)
        self.assertEqual(Path(selected.artifact.binary).read_bytes(), b"first completed image")
        self.assertEqual(Path(first.artifact.binary).read_bytes(), b"first completed image")

    def test_cleanup_failure_retains_the_original_construction_failure(self) -> None:
        first = self.publish()
        original = ValueError("original probe failure")
        cleanup = OSError("candidate cleanup failed")

        def describe(_candidate: Path) -> list[str]:
            raise original

        with patch.object(artifact_publication.shutil, "rmtree", side_effect=cleanup), self.assertRaises(ExceptionGroup) as raised:
            artifact_publication.publish(
                self.receipt, kind="extension", source=self.source, mode="dev", profile="dev",
                validated=True, interpreter="/selected/python", features=describe,
            )
        self.assertEqual(raised.exception.exceptions, (original, cleanup))
        self.assertEqual(artifact_publication.read(self.receipt), first)
