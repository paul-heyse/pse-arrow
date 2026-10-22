# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pinned optional bundle layout and archive containment."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import io
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import diagnostic_trace as trace


class DiagnosticBundleTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.spec = {
            "version": "test",
            "sha256": "selected-digest",
            "url": "https://example.test/tool",
            "prefix": "release",
            "member": "bin/rr",
        }

    def archive(self, entries: dict[str, bytes], symlink: str | None = None) -> Path:
        path = self.root / "tool.tar.gz"
        with tarfile.open(path, "w:gz") as archive:
            for name, data in entries.items():
                item = tarfile.TarInfo(name)
                item.size = len(data)
                item.mode = 0o755
                archive.addfile(item, io.BytesIO(data))
            if symlink is not None:
                item = tarfile.TarInfo("release/alias")
                item.type = tarfile.SYMTYPE
                item.linkname = symlink
                archive.addfile(item)
        return path

    def test_complete_runtime_retained_and_replacement_refused(self) -> None:
        names = (
            "bin/rr",
            "bin/rr_exec_stub",
            "lib/rr/librrpreload.so",
            "lib/rr/librrpage.so",
            "share/rr/64bit-core.xml",
        )
        archive = self.archive({"release/" + name: name.encode() for name in names})
        selected = trace.stage_bundle(archive, self.root, self.spec)
        trace.verify_bundle(selected, self.spec)
        self.assertTrue((selected / "lib/rr/librrpreload.so").is_file())
        (selected / "lib/rr/librrpreload.so").write_bytes(b"changed runtime")
        with self.assertRaisesRegex(ValueError, "member changed"):
            trace.verify_bundle(selected, self.spec)

    def test_escape_symlink_and_incomplete_runtime_refused(self) -> None:
        for entries, link in [
            ({"release/../outside": b"bad"}, None),
            ({"release/bin/rr": b"rr"}, "/etc/passwd"),
            ({"release/bin/rr": b"rr"}, None),
        ]:
            with (
                self.subTest(entries=entries, link=link),
                tempfile.TemporaryDirectory(dir=self.root) as directory,
                self.assertRaises(ValueError),
            ):
                trace.stage_bundle(
                    self.archive(entries, link), Path(directory), self.spec
                )
        self.assertFalse((self.root / "outside").exists())

    def test_binary_paths_keep_existing_tools_and_rr_bundle(self) -> None:
        with patch.dict("os.environ", {"XDG_DATA_HOME": str(self.root)}):
            self.assertEqual(
                trace.binary_path("rr"), self.root / "pse/tools/rr/5.10.0/bin/rr"
            )
            self.assertEqual(trace.binary_path("mcpls").name, "mcpls")
            self.assertEqual(trace.binary_path().name, "trace_processor_shell")


if __name__ == "__main__":
    unittest.main()
