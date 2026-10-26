# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Supplied-only executable currency and held-inode execution race controls."""
# ruff: noqa: PT009, PT027 -- stdlib host-boundary controls

from __future__ import annotations

import hashlib
import os
import subprocess
import tempfile
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import evidence_store, producing_currency, python_extension, surreal_server
from scripts.tests.producing_receipt_fixture import executable

if TYPE_CHECKING:
    from collections.abc import Mapping
    from typing import TextIO


class EvidenceCurrencyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.enterContext(patch.object(evidence_store, "ROOT", self.root))
        self.enterContext(
            patch.object(
                python_extension, "PRODUCING_WITNESS", producing_currency.Witness()
            )
        )
        self.tool = self.root / "pse-evidence"
        self.tool.write_text("#!/bin/sh\nprintf 'prepared-image\\n'\n")
        self.tool.chmod(0o700)
        self.source = self.root / "source.rs"
        self.source.write_text("prepared source")

    def run_tool(self) -> str:
        with tempfile.TemporaryFile(mode="w+") as output:
            code = evidence_store.run_offline(
                self.tool, [], deadline=time.monotonic() + 10, stdout=output
            )
            self.assertEqual(code, 0)
            output.seek(0)
            return output.read()

    def test_supplied_only_execution_never_imports_prepares_or_starts_service(
        self,
    ) -> None:
        executable(self.root, self.tool, files=(self.source,))
        with (
            patch.object(
                evidence_store, "build", side_effect=AssertionError("implicit build")
            ),
            patch.object(
                python_extension,
                "capture",
                side_effect=AssertionError("extension import"),
            ),
            patch.object(
                surreal_server,
                "tooling_activity",
                side_effect=AssertionError("store startup"),
            ),
            patch.object(hashlib, "file_digest", wraps=hashlib.file_digest) as digest,
        ):
            self.assertEqual(self.run_tool(), "prepared-image\n")
            count = digest.call_count
            self.assertEqual(self.run_tool(), "prepared-image\n")
            self.assertEqual(digest.call_count, count)

    def test_unprepared_source_change_and_image_change_refuse_without_launch(
        self,
    ) -> None:
        with patch.object(evidence_store.subprocess, "run") as launch:
            with self.assertRaisesRegex(
                evidence_store.ExecutableCurrencyError, "unprepared"
            ):
                self.run_tool()
            executable(self.root, self.tool, files=(self.source,))
            self.source.write_text("changed source")
            with self.assertRaisesRegex(
                evidence_store.ExecutableCurrencyError, "inputs changed"
            ):
                self.run_tool()
            executable(self.root, self.tool, files=(self.source,))
            self.tool.write_text("#!/bin/sh\nexit 9\n")
            with self.assertRaisesRegex(
                evidence_store.ExecutableCurrencyError, "bytes changed"
            ):
                self.run_tool()
            launch.assert_not_called()

    def test_replacement_between_admission_and_open_refuses(self) -> None:
        executable(self.root, self.tool)
        actual_open = os.open

        def raced(
            path: str | bytes | os.PathLike[str] | os.PathLike[bytes],
            flags: int,
            mode: int = 0o777,
            *,
            dir_fd: int | None = None,
        ) -> int:
            if path == self.tool:
                replacement = self.tool.with_suffix(".new")
                replacement.write_text("#!/bin/sh\nexit 9\n")
                replacement.chmod(0o700)
                replacement.replace(self.tool)
            return actual_open(path, flags, mode, dir_fd=dir_fd)

        with (
            patch.object(os, "open", side_effect=raced),
            self.assertRaisesRegex(
                evidence_store.ExecutableCurrencyError, "during admission"
            ),
        ):
            self.run_tool()

    def test_replacement_after_open_executes_the_exact_held_image(self) -> None:
        executable(self.root, self.tool)
        actual_run = subprocess.run

        def raced(
            command: list[str],
            *,
            stdout: TextIO | None,
            check: bool,
            timeout: float,
            env: Mapping[str, str],
            pass_fds: tuple[int, ...],
        ) -> subprocess.CompletedProcess[bytes]:
            replacement = self.tool.with_suffix(".new")
            replacement.write_text("#!/bin/sh\nprintf 'wrong-image\\n'\n")
            replacement.chmod(0o700)
            replacement.replace(self.tool)
            return actual_run(
                command,
                stdout=stdout,
                check=check,
                timeout=timeout,
                env=env,
                pass_fds=pass_fds,
            )

        with patch.object(evidence_store.subprocess, "run", side_effect=raced):
            self.assertEqual(self.run_tool(), "prepared-image\n")


if __name__ == "__main__":
    unittest.main()
