# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-build admission controls; no network, compiler or selected state."""
# ruff: noqa: PT009, PT027 -- stdlib source admission controls

from __future__ import annotations

import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import surreal_source as source
from scripts.tests.server_receipt_fixture import fixture_context, fixture_receipt


class SourceAdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        selected = patch.object(
            source,
            "build_context",
            return_value=(fixture_context(), {"fixture": True}, {}),
        )
        selected.start()
        self.addCleanup(selected.stop)

    def test_changed_binary_is_refused_before_rebuild_or_execution(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch) / source.installation_name(Path(scratch))
            directory.mkdir()
            binary = directory / "source/target/release/surreal"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"changed executable")
            (binary.parents[2] / "Cargo.lock").write_text("fixture lock")
            (directory / "release.json").write_text(
                json.dumps(
                    {
                        **fixture_receipt(Path(scratch)),
                        "lockfile_sha256": source.digest(
                            binary.parents[2] / "Cargo.lock"
                        ),
                        "binary_sha256": "0" * 64,
                    }
                )
            )
            with (
                patch.object(source, "verify_source"),
                patch.object(source, "command") as command,
                self.assertRaisesRegex(source.BuildError, "binary differs"),
            ):
                source.install(directory)
            command.assert_not_called()

    def test_changed_patch_inputs_refuse_cached_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch) / source.installation_name(Path(scratch))
            directory.mkdir()
            (directory / "release.json").write_text(json.dumps({"inputs": {}}))
            with (
                patch.object(source, "verify_source") as verify,
                patch.object(source, "command") as command,
                self.assertRaisesRegex(source.BuildError, "inputs changed"),
            ):
                source.install(directory)
            verify.assert_not_called()
            command.assert_not_called()

    def test_unowned_source_modification_refuses_before_compile(self) -> None:
        with (
            patch.object(
                source, "command", side_effect=[source.REVISION + "\n", "changed.rs\n"]
            ),
            self.assertRaisesRegex(source.BuildError, "unowned modifications"),
        ):
            source.verify_source(Path("/unopened-control-source"))

    def test_dependency_checksum_refuses_before_extraction(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch)
            with (
                patch.object(
                    source.urllib.request,
                    "urlopen",
                    return_value=io.BytesIO(b"wrong source archive"),
                ),
                patch.object(source.tarfile, "open") as extract,
                self.assertRaisesRegex(source.BuildError, "SHA256"),
            ):
                source.acquire_diskann(directory / "source", directory)
            extract.assert_not_called()
            self.assertFalse((directory / "source").exists())

    def test_surrealism_feature_and_locked_cli_patch_are_build_inputs(self) -> None:
        self.assertIn("surrealism", source.FEATURES.split(","))
        self.assertEqual(source.inputs()["surrealism_sdk_version"], "0.6.4")
        prior = source.installation_name()
        with patch.object(source, "FEATURES", "storage-mem,cli,mcp"):
            self.assertNotEqual(source.installation_name(), prior)

    def test_changed_patches_or_effective_context_select_new_installation(self) -> None:
        prior = source.installation_name()
        actual_digest = source.digest
        for selected in (source.INPUTS / name for name in source.PATCHES):

            def changed(path: Path, selected: Path = selected) -> str:
                return "0" * 64 if path == selected else actual_digest(path)

            with patch.object(source, "digest", side_effect=changed):
                self.assertNotEqual(source.installation_name(), prior)
        with patch.object(
            source,
            "build_context",
            return_value=(
                {"schema": "changed-effective-context"},
                {"fixture": True},
                {},
            ),
        ):
            self.assertNotEqual(source.installation_name(), prior)

    def test_complete_receipt_uses_effective_key_and_preserves_original_raw_provenance(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / source.installation_name(Path(temporary))
            directory.mkdir()
            binary = directory / "source/target/release/surreal"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"admitted")
            lock = directory / "source/Cargo.lock"
            lock.write_text("lock")
            receipt = {
                **fixture_receipt(Path(temporary)),
                "binary_sha256": source.digest(binary),
                "lockfile_sha256": source.digest(lock),
            }
            (directory / "release.json").write_text(json.dumps(receipt))
            with (
                patch.object(source, "verify_source"),
                patch.object(source, "command") as execute,
            ):
                self.assertEqual(source.install(directory), receipt)
            execute.assert_not_called()
            incomplete = {
                key: value for key, value in receipt.items() if key != "schema"
            }
            self.assertFalse(source.admitted_receipt(incomplete))

    def test_passive_receipt_checks_do_not_probe_compilers_or_native_trees(
        self,
    ) -> None:
        receipt = fixture_receipt()
        with patch.object(
            source, "build_context", side_effect=AssertionError("passive compiler scan")
        ):
            self.assertTrue(source.admitted_receipt(receipt))
        with patch.object(
            source, "build_context", side_effect=AssertionError("full launch scan")
        ):
            with self.assertRaisesRegex(AssertionError, "full launch scan"):
                source.admitted_receipt(receipt, revalidate=True)

    def test_installation_namespace_is_an_executable_input(self) -> None:
        self.assertNotEqual(
            source.installation_name(Path("/one")),
            source.installation_name(Path("/two")),
        )

    def test_unknown_existing_target_is_preserved_and_refused_before_compile(
        self,
    ) -> None:
        for recorded in (None, b"not json", b'{"inputs": {}, "effective_key": "old"}'):
            with (
                self.subTest(recorded=recorded),
                tempfile.TemporaryDirectory() as scratch,
            ):
                directory = Path(scratch) / source.installation_name(Path(scratch))
                target = directory / "source/target"
                target.mkdir(parents=True)
                artifact = target / "unknown-artifact"
                artifact.write_bytes(b"preserve")
                request = directory / "build-request.json"
                if recorded is not None:
                    request.write_bytes(recorded)
                with (
                    patch.object(source, "verify_source"),
                    patch.object(source, "command") as execute,
                    self.assertRaisesRegex(source.BuildError, "target.*build request"),
                ):
                    source.install(directory)
                execute.assert_not_called()
                self.assertEqual(artifact.read_bytes(), b"preserve")
                self.assertEqual(
                    request.read_bytes() if request.exists() else None, recorded
                )
                self.assertFalse((directory / "release.json").exists())

    def test_exact_interrupted_build_request_can_resume_existing_target(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch) / source.installation_name(Path(scratch))
            target = directory / "source/target"
            target.mkdir(parents=True)
            selected = source.inputs(Path(scratch))
            (directory / "build-request.json").write_text(
                json.dumps(source.build_request(selected))
            )

            def execute(arguments: list[str], **kwargs: object) -> str:
                if arguments[0] == str(source.ROOT / "scripts/pse-env"):
                    binary = target / source.PROFILE / "surreal"
                    binary.parent.mkdir()
                    binary.write_bytes(b"resumed executable")
                    (directory / "source/Cargo.lock").write_text("resumed lock")
                    return ""
                if arguments[-1] == "version":
                    return f"{source.VERSION} {selected['build_metadata']}"
                return "module help"

            with (
                patch.object(source, "verify_source"),
                patch.object(source, "command", side_effect=execute) as compile_command,
            ):
                receipt = source.install(directory)
            self.assertEqual(compile_command.call_count, 3)
            self.assertTrue(source.admitted_receipt(receipt, selected))
            self.assertTrue((directory / "release.json").exists())

    def test_existing_target_cannot_resume_request_from_another_namespace(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch) / source.installation_name(Path(scratch))
            (directory / "source/target").mkdir(parents=True)
            previous = source.build_request(
                source.inputs(Path(scratch) / "old-namespace")
            )
            request = directory / "build-request.json"
            request.write_text(json.dumps(previous))
            original = request.read_bytes()
            with (
                patch.object(source, "verify_source"),
                patch.object(source, "command") as execute,
                self.assertRaisesRegex(source.BuildError, "target differs"),
            ):
                source.install(directory)
            execute.assert_not_called()
            self.assertEqual(request.read_bytes(), original)

    def test_target_symlink_is_preserved_and_refused_even_with_matching_request(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch) / source.installation_name(Path(scratch))
            (directory / "source").mkdir(parents=True)
            target = directory / "source/target"
            target.symlink_to(
                Path(scratch) / "unowned-target", target_is_directory=True
            )
            request = directory / "build-request.json"
            request.write_text(
                json.dumps(source.build_request(source.inputs(Path(scratch))))
            )
            original = request.read_bytes()
            with (
                patch.object(source, "verify_source"),
                patch.object(source, "command") as execute,
                self.assertRaisesRegex(source.BuildError, "target is a symlink"),
            ):
                source.install(directory)
            execute.assert_not_called()
            self.assertTrue(target.is_symlink())
            self.assertEqual(request.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
