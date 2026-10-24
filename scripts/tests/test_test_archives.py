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


from scripts import assessment_store
from scripts.tests import assessment_fixture

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
        fixtures = archives.stage_inputs(
            self.source, self.archive / "workspace", ["fixtures"]
        )
        (self.archive / "tests.tar.zst").write_bytes(b"selected executable archive")
        self.library = self.root / "runtime.so"
        self.library.write_bytes(b"exact library")
        self.identity = {
            "name": "cargo-nextest",
            "version": "0.9.146",
            "sha256": "tool",
            "path": "/selected/nextest",
        }
        self.manifest = {
            "schema": archives.SCHEMA,
            "native_replay": "unsupported",
            "platform": [platform.system(), platform.machine()],
            "nextest": self.identity,
            "fixtures": fixtures,
            "archive_sha256": archives.digest(self.archive / "tests.tar.zst"),
            "libraries": {str(self.library): archives.digest(self.library)},
            "runtime_environment": archives.runtime_environment({}),
            "source_commit": "historical",
            "selected": [
                {
                    "binary_id": "pkg::test",
                    "name": "semantic",
                    "executable": "/old/test",
                }
            ],
            "producer_invocation": "/original/invocation.json",
            "claims": "Historical executable only",
        }
        tool = patch.object(
            archives.cargo_tools, "selected_identity", return_value=self.identity
        )
        tool.start()
        self.addCleanup(tool.stop)

    def test_fixture_relocation_uses_only_declared_historical_bytes(self) -> None:
        (self.source / "fixtures/value.txt").write_text("today's changed input")
        archives.preflight(self.manifest, self.archive, {})
        self.assertEqual(
            (self.archive / "workspace/fixtures/value.txt").read_text(),
            "historical input",
        )

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
        with self.assertRaisesRegex(
            archives.ArchiveError, "archive missing or changed"
        ):
            archives.preflight(self.manifest, self.archive, {})

    def test_runtime_absence_secret_and_tool_identity_remain_exact(self) -> None:
        environment = {"SYMBOLICA_LICENSE": "private license value"}
        opaque = archives.runtime_environment(environment)
        self.assertNotIn("private license value", json.dumps(opaque))
        self.assertNotEqual(opaque, archives.runtime_environment({}))
        with self.assertRaisesRegex(archives.ArchiveError, "runtime environment"):
            archives.preflight(self.manifest, self.archive, environment)
        with (
            patch.object(
                archives.cargo_tools,
                "selected_identity",
                return_value={**self.identity, "sha256": "other"},
            ),
            self.assertRaisesRegex(archives.ArchiveError, "nextest tool"),
        ):
            archives.preflight(self.manifest, self.archive, {})

    def test_fixture_traversal_and_symlink_refuse_before_copy(self) -> None:
        for value in ("../other", "/absolute"):
            with self.assertRaises(archives.ArchiveError):
                archives.stage_inputs(self.source, self.root / "copy", [value])
        (self.source / "fixtures/link").symlink_to(self.library)
        with self.assertRaisesRegex(archives.ArchiveError, "Symlinked"):
            archives.stage_inputs(self.source, self.root / "copy", ["fixtures"])

    def test_native_unit_refuses_before_executable_launch(self) -> None:
        metadata = {
            "packages": [
                {"id": "native", "name": "pse-backend-native"},
                {"id": "pure", "name": "pse-ids"},
            ]
        }
        archives.require_pure_closure(metadata, {"units": [{"pkg_id": "pure"}]})
        with self.assertRaisesRegex(
            archives.ArchiveError, "Native archival is not adopted"
        ):
            archives.require_pure_closure(metadata, {"units": [{"pkg_id": "native"}]})

    def test_archive_reference_protects_detail_and_release_is_exact(self) -> None:
        with (
            patch.dict(
                os.environ, {"PSE_TEST_RESOURCE_STATE": str(self.root / "registry")}
            ),
            patch.object(resources.host, "inherit", return_value=None),
        ):
            resource = resources.register_report(self.archive, root=self.root)
            validation.write_json(self.archive / "archive.json", self.manifest)
            receipt = {
                "version": 6,
                "complete": True,
                "required_checks_covered": True,
                "input_coverage": True,
                "source_unchanged": True,
                "provenance_errors": [],
                "scope": [{"name": "producer"}],
                "checks": [
                    {"gate": "producer", "status": "passed", "log": "archive.json"}
                ],
            }
            assessment_fixture.publish(self.archive, receipt)
            archives.seal(
                self.archive, resource, receipt, ["archive.json"], ["tests.tar.zst"]
            )
            resources.pin(resource, False)
            consumer = self.root / "consumer.json"
            with resources.reference_report(self.archive, consumer):
                self.assertEqual(
                    resources.eligible(
                        resources.resource_status(resource), check_units=False
                    ),
                    "protected reference or active borrow",
                )
            original = archives.digest(self.archive / "checks.json")
            self.assertEqual(
                resources.eligible(
                    resources.resource_status(resource), check_units=False
                ),
                "protected reference or active borrow",
            )
            with self.assertRaises(resources.ResourceError):
                resources.release_reference(resource, str(consumer), "a" * 64)
            resources.release_reference(resource, str(consumer), original)
            self.assertIsNone(
                resources.eligible(
                    resources.resource_status(resource), check_units=False
                )
            )

    def corrupt_unselected_sealed_detail(self) -> None:
        self.enterContext(patch.object(resources, "registry", return_value=self.root / "registry"))
        self.enterContext(patch.object(resources.host, "inherit", return_value=None))
        resource = resources.register_report(self.archive, root=self.root)
        validation.write_json(self.archive / "archive.json", self.manifest)
        receipt = assessment_fixture.publish(self.archive, {
            "checks": [{"gate": "producer", "status": "passed", "log": "archive.json"}],
            "source_files": {"historical": "a" * 64},
        })
        archives.seal(self.archive, resource, receipt, ["archive.json"], ["tests.tar.zst"])
        image = assessment_store.root(self.archive)
        selected = self.archive / "details" / (image["source_cut"]["sha256"] + ".json")
        selected.write_text("{}")

    def test_inspection_checks_the_entire_sealed_archive(self) -> None:
        self.corrupt_unselected_sealed_detail()
        with self.assertRaisesRegex(resources.ResourceError, "ownership changed"):
            archives.inspect(self.archive)

    def test_replay_checks_entire_sealed_archive_before_executable_preparation(self) -> None:
        self.corrupt_unselected_sealed_detail()
        with (
            patch.object(archives.arrow_validation, "rust_test_environment", return_value={}),
            patch.object(archives, "new_output", return_value="unpublished-control"),
            patch.object(archives, "preflight", side_effect=AssertionError("unverified archive prepared")),
            self.assertRaisesRegex(resources.ResourceError, "ownership changed"),
        ):
            archives.replay(self.archive, self.root / "build" / "replay")


class ScientificArchiveBridgeTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.workspace = self.root / "historical" / "workspace"
        self.source = self.root / "producer-input.json"
        self.source.write_bytes(
            b'{"schema":"scientific-inputs-v1","files":{"law.pse":[111,108,100]},"families":{}}'
        )
        self.original = self.source.read_bytes()
        self.role = archives.scientific_inputs.ROLE
        self.native = {
            "rust_invocation": {"fixtures": {self.role: str(self.source)}},
            "files": {str(self.source): archives.digest(self.source)},
        }

    def stage(self) -> dict:
        fixtures, roles = archives.stage_scientific_input(self.native, self.workspace)
        return {"fixtures": fixtures, "input_roles": roles}

    def test_stage_and_relocate_replays_exact_bytes_without_producer_or_live_source(
        self,
    ) -> None:
        manifest = self.stage()
        logical = ".pse-inputs/scientific.json"
        self.assertEqual(set(manifest["fixtures"]), {logical})
        self.assertEqual(
            manifest["input_roles"],
            {
                self.role: {
                    "path": logical,
                    "sha256": self.native["files"][str(self.source)],
                }
            },
        )
        self.source.unlink()
        relocated = self.root / "relocated-workspace"
        self.workspace.rename(relocated)
        environment = {self.role: "/deleted/producer/capsule.json", "OTHER": "retained"}
        archives.bind_input_roles(manifest, relocated, environment)
        self.assertEqual(environment["OTHER"], "retained")
        self.assertEqual(environment[self.role], str(relocated / logical))
        self.assertEqual(Path(environment[self.role]).read_bytes(), self.original)
        self.assertFalse(self.source.exists())

    def test_stage_refuses_missing_tampered_or_unrecorded_producer_input(self) -> None:
        self.source.write_bytes(self.original + b"changed")
        with self.assertRaises(archives.ArchiveError):
            self.stage()
        self.source.unlink()
        with self.assertRaises(archives.ArchiveError):
            self.stage()
        self.source.write_bytes(self.original)
        self.native["files"] = {}
        with self.assertRaises(archives.ArchiveError):
            self.stage()
        self.assertFalse(self.workspace.exists())

    def test_legacy_archive_clears_inherited_role_without_requiring_new_input(
        self,
    ) -> None:
        environment = {self.role: str(self.source), "OTHER": "retained"}
        archives.bind_input_roles({"fixtures": {}}, self.workspace, environment)
        self.assertEqual(environment, {"OTHER": "retained"})
        self.assertEqual(archives.stage_scientific_input({}, self.workspace), ({}, {}))
        self.assertFalse(self.workspace.exists())

    def test_role_refuses_missing_tampered_or_unstaged_historical_bytes(self) -> None:
        manifest = self.stage()
        target = self.workspace / ".pse-inputs/scientific.json"
        for mutate in (lambda: target.write_bytes(b"tampered"), target.unlink):
            target.write_bytes(self.original)
            mutate()
            environment = {self.role: "/old/producer"}
            with self.assertRaises(archives.ArchiveError):
                archives.bind_input_roles(manifest, self.workspace, environment)
            self.assertNotIn(self.role, environment)
        target.write_bytes(self.original)
        manifest["fixtures"] = {}
        with self.assertRaises(archives.ArchiveError):
            archives.bind_input_roles(manifest, self.workspace, {})

    def test_role_refuses_unknown_roles_and_relative_path_escape(self) -> None:
        manifest = self.stage()
        binding = manifest["input_roles"][self.role]
        with self.assertRaises(archives.ArchiveError):
            archives.bind_input_roles(
                {
                    "fixtures": manifest["fixtures"],
                    "input_roles": {"OTHER_ROLE": binding},
                },
                self.workspace,
                {},
            )
        for path in ("../producer-input.json", str(self.source)):
            with self.subTest(path=path), self.assertRaises(archives.ArchiveError):
                archives.bind_input_roles(
                    {
                        "fixtures": {path: binding["sha256"]},
                        "input_roles": {self.role: {**binding, "path": path}},
                    },
                    self.workspace,
                    {},
                )

    def test_role_refuses_a_symlinked_input_or_parent_even_with_matching_bytes(
        self,
    ) -> None:
        manifest = self.stage()
        logical = self.workspace / ".pse-inputs/scientific.json"
        logical.unlink()
        logical.symlink_to(self.source)
        with self.assertRaises(archives.ArchiveError):
            archives.bind_input_roles(manifest, self.workspace, {})
        logical.unlink()
        logical.parent.rmdir()
        external = self.root / "external"
        external.mkdir()
        (external / "scientific.json").write_bytes(self.original)
        logical.parent.symlink_to(external, target_is_directory=True)
        with self.assertRaises(archives.ArchiveError):
            archives.bind_input_roles(manifest, self.workspace, {})


if __name__ == "__main__":
    unittest.main()
