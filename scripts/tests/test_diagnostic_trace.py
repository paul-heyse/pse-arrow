# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pinned optional bundle layout and archive containment."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import hashlib
import io
import tarfile
import tempfile
import unittest
import zipfile
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


class DiagnosticBinaryTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.environment = {"HOME": str(self.root)}
        self.binary = b"pinned executable bytes"
        self.spec = {
            "version": "0.19.0",
            "url": "https://example.test/sccache.tar.gz",
            "sha256": "",
            "binary_sha256": hashlib.sha256(self.binary).hexdigest(),
            "member": "sccache",
            "archive": "tar.gz",
        }
        self.patcher = patch.object(trace, "tool_spec", return_value=self.spec)
        self.patcher.start()
        self.addCleanup(self.patcher.stop)

    def archive(self, names: list[str], *, symlink: bool = False) -> bytes:
        output = io.BytesIO()
        with tarfile.open(fileobj=output, mode="w:gz") as bundle:
            for name in names:
                item = tarfile.TarInfo(name)
                if symlink:
                    item.type = tarfile.SYMTYPE
                    item.linkname = "elsewhere"
                else:
                    item.size = len(self.binary)
                bundle.addfile(item, io.BytesIO(self.binary))
        data = output.getvalue()
        self.spec["sha256"] = hashlib.sha256(data).hexdigest()
        return data

    def install(self, data: bytes) -> Path:
        with patch.object(
            trace.urllib.request, "urlopen", return_value=io.BytesIO(data)
        ):
            return trace.install("sccache", self.environment)

    def test_selected_archive_and_binary_allowances_precede_publication(self) -> None:
        data = self.archive(["release/sccache"])
        self.spec["max_archive_bytes"] = "1"
        with self.assertRaisesRegex(ValueError, "archive exceeds"):
            self.install(data)
        self.assertFalse(trace.binary_path("sccache", self.environment).exists())
        self.spec["max_archive_bytes"] = str(len(data))
        self.spec["max_binary_bytes"] = "1"
        with self.assertRaisesRegex(ValueError, "oversized"):
            self.install(data)
        self.assertFalse(trace.binary_path("sccache", self.environment).exists())
        self.spec["max_binary_bytes"] = str(len(self.binary))
        self.assertEqual(self.install(data).read_bytes(), self.binary)

    def test_environment_controls_versioned_path_without_ambient_xdg(self) -> None:
        with patch.dict("os.environ", {"XDG_DATA_HOME": "/ambient/not-selected"}):
            selected = trace.binary_path("sccache", self.environment)
        self.assertEqual(
            selected, self.root / ".local/share/pse/tools/sccache/0.19.0/sccache"
        )
        self.assertEqual(
            trace.binary_path("sccache", {"XDG_DATA_HOME": str(self.root)}),
            self.root / "pse/tools/sccache/0.19.0/sccache",
        )

    def test_verified_install_reused_and_changed_bytes_refused(self) -> None:
        target = self.install(self.archive(["release/sccache"]))
        self.assertEqual(target.read_bytes(), self.binary)
        self.assertEqual(trace.verify_binary("sccache", self.environment), target)
        with patch.object(trace.urllib.request, "urlopen") as download:
            self.assertEqual(trace.install("sccache", self.environment), target)
            download.assert_not_called()
            target.write_bytes(b"wrong version or changed executable")
            with self.assertRaisesRegex(ValueError, "checksum differs"):
                trace.install("sccache", self.environment)
            download.assert_not_called()
        self.assertEqual(target.read_bytes(), b"wrong version or changed executable")

    def test_bad_archive_binary_and_member_refused_before_publication(self) -> None:
        for names, symlink, wrong_archive, wrong_binary in [
            (["release/sccache"], False, True, False),
            (["release/sccache"], False, False, True),
            (["first/sccache", "second/sccache"], False, False, False),
            (["release/sccache"], True, False, False),
        ]:
            with self.subTest(names=names, symlink=symlink):
                data = self.archive(names, symlink=symlink)
                self.spec["binary_sha256"] = (
                    "wrong" if wrong_binary else hashlib.sha256(self.binary).hexdigest()
                )
                if wrong_archive:
                    self.spec["sha256"] = "wrong"
                with self.assertRaises(ValueError):
                    self.install(data)
                self.assertFalse(
                    trace.binary_path("sccache", self.environment).exists()
                )

    def test_nonexecutable_symlink_and_missing_installations_refused(self) -> None:
        target = trace.binary_path("sccache", self.environment)
        with self.assertRaisesRegex(ValueError, "missing or nonregular"):
            trace.verify_binary("sccache", self.environment)
        target.parent.mkdir(parents=True)
        target.write_bytes(self.binary)
        target.chmod(0o644)
        with self.assertRaisesRegex(ValueError, "not executable"):
            trace.verify_binary("sccache", self.environment)
        target.unlink()
        original = self.root / "other"
        original.write_bytes(self.binary)
        original.chmod(0o755)
        target.symlink_to(original)
        with self.assertRaisesRegex(ValueError, "missing or nonregular"):
            trace.install("sccache", self.environment)

    def test_interrupted_publication_cannot_leave_an_accepted_partial_binary(
        self,
    ) -> None:
        data = self.archive(["release/sccache"])
        with (
            patch.object(
                trace.os, "link", side_effect=OSError("publication interrupted")
            ),
            self.assertRaisesRegex(OSError, "publication interrupted"),
        ):
            self.install(data)
        self.assertFalse(trace.binary_path("sccache", self.environment).exists())
        self.assertEqual(self.install(data).read_bytes(), self.binary)

    def test_racing_incompatible_install_is_preserved_and_refused(self) -> None:
        data = self.archive(["release/sccache"])
        target = trace.binary_path("sccache", self.environment)

        def race(_source: Path, destination: Path) -> None:
            destination.write_bytes(b"other installation")
            destination.chmod(0o755)
            raise FileExistsError

        with (
            patch.object(trace.os, "link", side_effect=race),
            self.assertRaisesRegex(ValueError, "checksum differs"),
        ):
            self.install(data)
        self.assertEqual(target.read_bytes(), b"other installation")

    def test_zip_symlink_cannot_be_published_as_executable(self) -> None:
        output = io.BytesIO()
        with zipfile.ZipFile(output, "w") as bundle:
            member = zipfile.ZipInfo("release/sccache")
            member.external_attr = 0o120777 << 16
            bundle.writestr(member, b"elsewhere")
        data = output.getvalue()
        self.spec["archive"] = "zip"
        self.spec["sha256"] = hashlib.sha256(data).hexdigest()
        with self.assertRaisesRegex(ValueError, "missing, ambiguous or oversized"):
            self.install(data)
        self.assertFalse(trace.binary_path("sccache", self.environment).exists())


if __name__ == "__main__":
    unittest.main()
