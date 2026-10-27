# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Supplied-only executable currency and held-inode execution race controls."""
# ruff: noqa: PT009, PT027 -- stdlib host-boundary controls

from __future__ import annotations

import contextlib
import hashlib
import os
import subprocess
import tempfile
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import (
    evidence_store,
    producing_currency,
    pse_env,
    python_extension,
    surreal_server,
)
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
            patch.object(
                pse_env,
                "local_keys",
                side_effect=AssertionError("local shell evaluation"),
            ),
            patch.object(hashlib, "file_digest", wraps=hashlib.file_digest) as digest,
        ):
            self.assertEqual(self.run_tool(), "prepared-image\n")
            count = digest.call_count
            self.assertEqual(self.run_tool(), "prepared-image\n")
            self.assertEqual(digest.call_count, count)

    def test_same_pure_receipt_admits_source_and_native_callers(self) -> None:
        source_path = evidence_store.compiler_environment(os.environ)["PATH"]
        with patch.dict(os.environ, {"PATH": source_path}):
            executable(self.root, self.tool, files=(self.source,))
            self.assertEqual(self.run_tool(), "prepared-image\n")
            with patch.dict(
                os.environ,
                {
                    "PATH": str(python_extension.interpreter(os.environ).parent)
                    + os.pathsep
                    + source_path,
                    "LD_LIBRARY_PATH": "/runtime/native/lib",
                    "IPOPT_DIR": "/runtime/ipopt",
                    "PETSC_DIR": "/runtime/petsc",
                    "PSE_KLU_PROVIDER_RECEIPT": "/runtime/klu/.complete.json",
                    "BINDGEN_EXTRA_CLANG_ARGS": "-resource-dir=/runtime/clang",
                    "CMAKE_TOOLCHAIN_FILE": "/runtime/native-cache.cmake",
                    "PSE_NATIVE_COMPILER_CACHE": "/runtime/sccache",
                },
            ):
                self.assertEqual(self.run_tool(), "prepared-image\n")
            self.assertEqual(self.run_tool(), "prepared-image\n")

    def test_build_entry_removes_native_loader_path_before_preparation(self) -> None:
        def prepare(environment: Mapping[str, str]) -> tuple[int, None]:
            self.assertNotIn("LD_LIBRARY_PATH", environment)
            self.assertEqual(environment["RUSTFLAGS"], "-C debuginfo=1")
            return 41, None

        with (
            patch.dict(
                os.environ,
                {
                    "LD_LIBRARY_PATH": "/runtime/native/lib",
                    "RUSTFLAGS": "-C debuginfo=1",
                },
            ),
            patch.object(python_extension, "prepare_extractor", side_effect=prepare),
        ):
            self.assertEqual(evidence_store.build(), 41)
            self.assertEqual(os.environ["LD_LIBRARY_PATH"], "/runtime/native/lib")

    def test_actual_children_have_no_native_loader_path_and_keep_original_context(
        self,
    ) -> None:
        self.tool.write_text(
            "#!/bin/sh\n"
            'printf "%s\\n" "${LD_LIBRARY_PATH-unset}" '
            '"${PSE_NATIVE_OPERATION-unset}" "${PSE_EVIDENCE_STATE-unset}" '
            '"${PSE_EVIDENCE_ROLE-unset}" "${PSE_TOOLING_ACTIVITY_FD-unset}" '
            '"${PSE_TOOLING_SERVICE_INSTANCE_ID-unset}"\n'
        )
        executable(self.root, self.tool, files=(self.source,))
        with patch.dict(
            os.environ,
            {
                "LD_LIBRARY_PATH": "/runtime/native/lib",
                "PSE_NATIVE_OPERATION": "original-native-owner",
                "PSE_EVIDENCE_STATE": "original-evidence-state",
                "PSE_EVIDENCE_ROLE": "reader",
                "PSE_TOOLING_ACTIVITY_FD": "unselected-borrower",
                "PSE_TOOLING_SERVICE_INSTANCE_ID": "unselected-service",
            },
        ):
            self.assertEqual(
                self.run_tool().splitlines(),
                [
                    "unset",
                    "original-native-owner",
                    "original-evidence-state",
                    "reader",
                    "unset",
                    "unset",
                ],
            )
            with (
                tempfile.TemporaryFile() as activity,
                tempfile.TemporaryFile(mode="w+") as output,
                patch.object(
                    surreal_server,
                    "tooling_activity",
                    return_value=contextlib.nullcontext(
                        ({"instance_id": "selected-service"}, activity.fileno())
                    ),
                ),
                patch.object(evidence_store, "selected_configuration"),
                patch.object(surreal_server, "owns_listener", return_value=True),
            ):
                self.assertEqual(
                    evidence_store.run_native(
                        self.root,
                        self.tool,
                        ["status"],
                        deadline=time.monotonic() + 10,
                        stdout=output,
                    ),
                    0,
                )
                output.seek(0)
                self.assertEqual(
                    output.read().splitlines(),
                    [
                        "unset",
                        "original-native-owner",
                        "original-evidence-state",
                        "reader",
                        str(activity.fileno()),
                        "selected-service",
                    ],
                )
            self.assertEqual(os.environ["LD_LIBRARY_PATH"], "/runtime/native/lib")
            self.assertEqual(
                os.environ["PSE_TOOLING_ACTIVITY_FD"], "unselected-borrower"
            )

    def test_changed_compiler_flags_and_selection_refuse_without_launch(self) -> None:
        for name in ("RUSTFLAGS", "RUSTC", "LDFLAGS", "CXX"):
            with self.subTest(name=name):
                executable(self.root, self.tool, files=(self.source,))
                with (
                    patch.dict(os.environ, {name: "changed-compiler-selection"}),
                    patch.object(evidence_store.subprocess, "run") as launch,
                    self.assertRaisesRegex(
                        evidence_store.ExecutableCurrencyError, "inputs changed"
                    ),
                ):
                    self.run_tool()
                launch.assert_not_called()

    def test_changed_compiler_search_path_refuses_without_launch(self) -> None:
        executable(self.root, self.tool, files=(self.source,))
        with (
            patch.dict(
                os.environ,
                {
                    "PATH": str(self.root / "compiler-override")
                    + os.pathsep
                    + os.environ["PATH"]
                },
            ),
            patch.object(evidence_store.subprocess, "run") as launch,
            self.assertRaisesRegex(
                evidence_store.ExecutableCurrencyError, "inputs changed"
            ),
        ):
            self.run_tool()
        launch.assert_not_called()

    def test_changed_witnessed_local_configuration_refuses_without_evaluation(
        self,
    ) -> None:
        local = self.root / ".envrc.local"
        local.write_text("export RUSTFLAGS=original\n")
        executable(self.root, self.tool, files=(self.source, local))
        local.write_text("export RUSTFLAGS=changed\n")
        with (
            patch.object(
                pse_env,
                "local_keys",
                side_effect=AssertionError("local shell evaluation"),
            ),
            patch.object(evidence_store.subprocess, "run") as launch,
            self.assertRaisesRegex(
                evidence_store.ExecutableCurrencyError, "inputs changed"
            ),
        ):
            self.run_tool()
        launch.assert_not_called()

    def test_actual_consumed_native_inputs_still_refuse_changed_values(self) -> None:
        for name in ("IPOPT_DIR", "BINDGEN_EXTRA_CLANG_ARGS", "CMAKE_TOOLCHAIN_FILE"):
            with self.subTest(name=name):
                executable(
                    self.root,
                    self.tool,
                    files=(self.source,),
                    consumed_environment=(name,),
                )
                with (
                    patch.dict(os.environ, {name: "changed-consumed-input"}),
                    patch.object(evidence_store.subprocess, "run") as launch,
                    self.assertRaisesRegex(
                        evidence_store.ExecutableCurrencyError, "inputs changed"
                    ),
                ):
                    self.run_tool()
                launch.assert_not_called()

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
