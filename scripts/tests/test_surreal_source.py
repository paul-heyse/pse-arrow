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


class SourceAdmissionTests(unittest.TestCase):
    def test_changed_binary_is_refused_before_rebuild_or_execution(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch)
            binary = directory / "source/target/release/surreal"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"changed executable")
            (directory / "release.json").write_text(
                json.dumps(
                    {
                        "inputs": source.inputs(),
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
            directory = Path(scratch)
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

    def test_changed_patch_toolchain_or_flags_select_new_installation(self) -> None:
        prior = source.installation_name()
        actual_digest = source.digest
        for selected in (
            *(source.INPUTS / name for name in source.PATCHES),
            source.ROOT / "rust-toolchain.toml",
            source.ROOT / ".cargo/config.toml",
        ):

            def changed(path: Path, selected: Path = selected) -> str:
                return "0" * 64 if path == selected else actual_digest(path)

            with patch.object(source, "digest", side_effect=changed):
                self.assertNotEqual(source.installation_name(), prior)


if __name__ == "__main__":
    unittest.main()
