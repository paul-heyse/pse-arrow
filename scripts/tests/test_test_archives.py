# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Historical closure controls, independent of current checkout preparation."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import json
import os
import platform
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import test_archives as archives
from scripts import test_resources as resources
from scripts import validation


class ArchiveClosureTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.archive = self.root / "build" / "archive"
        self.archive.mkdir(parents=True)
        self.source = self.root / "source"
        (self.source / "fixtures").mkdir(parents=True)
        (self.source / "fixtures/value.txt").write_text("historical input")
        fixtures = archives.stage_inputs(self.source, self.archive / "workspace", ["fixtures"])
        (self.archive / "tests.tar.zst").write_bytes(b"selected executable archive")
        self.library = self.root / "runtime.so"
        self.library.write_bytes(b"exact library")
        self.identity = {"name": "cargo-nextest", "version": "0.9.146", "sha256": "tool", "path": "/selected/nextest"}
        self.manifest = {
            "schema": archives.SCHEMA, "native_replay": "unsupported",
            "platform": [platform.system(), platform.machine()],
            "nextest": self.identity, "fixtures": fixtures,
            "archive_sha256": archives.digest(self.archive / "tests.tar.zst"),
            "libraries": {str(self.library): archives.digest(self.library)},
            "runtime_environment": archives.runtime_environment({}),
            "source_commit": "historical", "selected": [{"binary_id": "pkg::test", "name": "semantic", "executable": "/old/test"}],
            "producer_invocation": "/original/invocation.json",
            "claims": "Historical executable only",
        }
        tool = patch.object(archives.cargo_tools, "selected_identity", return_value=self.identity)
        tool.start()
        self.addCleanup(tool.stop)

    def test_fixture_relocation_uses_only_declared_historical_bytes(self) -> None:
        (self.source / "fixtures/value.txt").write_text("today's changed input")
        archives.preflight(self.manifest, self.archive, {})
        self.assertEqual((self.archive / "workspace/fixtures/value.txt").read_text(), "historical input")

    def test_missing_and_changed_runtime_library_refuse(self) -> None:
        self.library.write_bytes(b"replacement library")
        with self.assertRaisesRegex(archives.ArchiveError, "runtime dependency"):
            archives.preflight(self.manifest, self.archive, {})
        self.library.unlink()
        with self.assertRaisesRegex(archives.ArchiveError, "runtime dependency"):
            archives.preflight(self.manifest, self.archive, {})

    def test_changed_archive_or_fixture_refuse(self) -> None:
        path = self.archive / "workspace/fixtures/value.txt"
        path.write_text("different fixture")
        with self.assertRaisesRegex(archives.ArchiveError, "fixture/config"):
            archives.preflight(self.manifest, self.archive, {})
        path.write_text("historical input")
        (self.archive / "tests.tar.zst").write_bytes(b"different binary")
        with self.assertRaisesRegex(archives.ArchiveError, "archive missing or changed"):
            archives.preflight(self.manifest, self.archive, {})

    def test_runtime_absence_secret_and_tool_identity_remain_exact(self) -> None:
        environment = {"SYMBOLICA_LICENSE": "private license value"}
        opaque = archives.runtime_environment(environment)
        self.assertNotIn("private license value", json.dumps(opaque))
        self.assertNotEqual(opaque, archives.runtime_environment({}))
        with self.assertRaisesRegex(archives.ArchiveError, "runtime environment"):
            archives.preflight(self.manifest, self.archive, environment)
        with patch.object(archives.cargo_tools, "selected_identity", return_value={**self.identity, "sha256": "other"}), self.assertRaisesRegex(archives.ArchiveError, "nextest tool"):
            archives.preflight(self.manifest, self.archive, {})

    def test_fixture_traversal_and_symlink_refuse_before_copy(self) -> None:
        for value in ("../other", "/absolute"):
            with self.assertRaises(archives.ArchiveError):
                archives.stage_inputs(self.source, self.root / "copy", [value])
        (self.source / "fixtures/link").symlink_to(self.library)
        with self.assertRaisesRegex(archives.ArchiveError, "Symlinked"):
            archives.stage_inputs(self.source, self.root / "copy", ["fixtures"])

    def test_native_unit_refuses_before_executable_launch(self) -> None:
        metadata = {"packages": [{"id": "native", "name": "pse-backend-native"}, {"id": "pure", "name": "pse-ids"}]}
        archives.require_pure_closure(metadata, {"units": [{"pkg_id": "pure"}]})
        with self.assertRaisesRegex(archives.ArchiveError, "Native archival is not adopted"):
            archives.require_pure_closure(metadata, {"units": [{"pkg_id": "native"}]})

    def test_archive_reference_protects_detail_and_release_is_exact(self) -> None:
        with patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(self.root / "registry")}), patch.object(resources.host, "inherit", return_value=None):
            resource = resources.register_report(self.archive, root=self.root)
            validation.write_json(self.archive / "archive.json", self.manifest)
            receipt = {"version": 5, "complete": True, "required_checks_covered": True,
                "input_coverage": True, "source_unchanged": True, "provenance_errors": [],
                "scope": [{"name": "producer"}],
                "checks": [{"gate": "producer", "status": "passed", "log": "archive.json"}]}
            archives.seal(self.archive, resource, receipt, ["archive.json"], ["tests.tar.zst"])
            resources.pin(resource, False)
            consumer = self.root / "consumer.json"
            with resources.reference_report(self.archive, consumer):
                self.assertEqual(resources.eligible(resources.resource_status(resource), check_units=False), "protected reference or active borrow")
            original = archives.digest(self.archive / "checks.json")
            self.assertEqual(resources.eligible(resources.resource_status(resource), check_units=False), "protected reference or active borrow")
            with self.assertRaises(resources.ResourceError):
                resources.release_reference(resource, str(consumer), "a" * 64)
            resources.release_reference(resource, str(consumer), original)
            self.assertIsNone(resources.eligible(resources.resource_status(resource), check_units=False))


if __name__ == "__main__":
    unittest.main()
