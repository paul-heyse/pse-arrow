# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Issued executable generations, pure child context and explicit construction controls."""
# ruff: noqa: PT009, PT027 -- standard-library owner controls

from __future__ import annotations

import contextlib
import json
import os
import subprocess
import tempfile
import time
import unittest
from pathlib import Path
from types import SimpleNamespace
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import (
    arrow_validation,
    evidence_store,
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
            patch.dict(os.environ, {"UV_PROJECT_ENVIRONMENT": str(self.root / ".venv")})
        )
        self.tool = self.root / "pse-evidence"
        self.tool.write_text("#!/bin/sh\nprintf 'first-published-image\\n'\n")
        self.tool.chmod(0o700)

    def run_tool(self) -> str:
        with tempfile.TemporaryFile(mode="w+") as output:
            code = evidence_store.run_offline(
                [], deadline=time.monotonic() + 10, stdout=output
            )
            self.assertEqual(code, 0)
            output.seek(0)
            return output.read()

    def test_published_image_works_without_alias_source_or_compiler_revalidation(
        self,
    ) -> None:
        published = executable(self.root, self.tool)
        self.tool.unlink()
        (self.root / "new.rs").write_text("new source for the next constructor")
        actual_open = Path.open

        def metadata_only(path: Path, mode: str = "r", buffering: int = -1) -> object:
            if path == Path(published.artifact.binary):
                raise AssertionError("consumer content read")
            return actual_open(path, mode, buffering=buffering)

        with (
            patch.dict(
                os.environ,
                {
                    "RUSTFLAGS": "later compiler selection",
                    "LD_LIBRARY_PATH": "/runtime/native/lib",
                },
            ),
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
            patch.object(Path, "open", metadata_only),
            patch.object(Path, "stat", side_effect=AssertionError("consumer stat")),
        ):
            self.assertEqual(self.run_tool(), "first-published-image\n")
        self.assertEqual(
            python_extension.read_publication(
                "evidence-executable", os.environ, root=self.root
            ),
            published,
        )

    def test_missing_publication_never_launches_or_prepares(self) -> None:
        with (
            patch.object(evidence_store.subprocess, "run") as launch,
            patch.object(evidence_store, "build") as prepare,
            self.assertRaisesRegex(
                evidence_store.ExecutableCurrencyError, "unpublished"
            ),
        ):
            self.run_tool()
        launch.assert_not_called()
        prepare.assert_not_called()

    def test_generation_switch_does_not_change_already_issued_execution(self) -> None:
        first = executable(self.root, self.tool)
        actual_run = subprocess.run
        replacement = []

        def switched(
            command: list[str],
            *,
            stdout: TextIO | None,
            check: bool,
            timeout: float,
            env: Mapping[str, str],
            pass_fds: tuple[int, ...],
        ) -> subprocess.CompletedProcess[bytes]:
            self.tool.write_text("#!/bin/sh\nprintf 'second-published-image\\n'\n")
            replacement.append(executable(self.root, self.tool))
            return actual_run(
                command,
                stdout=stdout,
                check=check,
                timeout=timeout,
                env=env,
                pass_fds=pass_fds,
            )

        with patch.object(evidence_store.subprocess, "run", side_effect=switched):
            self.assertEqual(self.run_tool(), "first-published-image\n")
        self.assertNotEqual(first.generation, replacement[0].generation)
        self.assertEqual(self.run_tool(), "second-published-image\n")
        self.assertTrue(Path(first.artifact.binary).exists())

    def test_actual_children_remove_loader_path_and_keep_original_context(self) -> None:
        self.tool.write_text(
            "#!/bin/sh\n"
            'printf "%s\\n" "${LD_LIBRARY_PATH-unset}" "${PSE_NATIVE_OPERATION-unset}" '
            '"${PSE_EVIDENCE_STATE-unset}" "${PSE_EVIDENCE_ROLE-unset}" '
            '"${PSE_TOOLING_ACTIVITY_FD-unset}" "${PSE_TOOLING_SERVICE_INSTANCE_ID-unset}"\n'
        )
        executable(self.root, self.tool)
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

    def test_explicit_builder_publishes_actual_cargo_artifact_with_pure_context(
        self,
    ) -> None:
        cargo = {
            "reason": "compiler-artifact",
            "target": {"name": "pse-evidence"},
            "executable": str(self.tool),
            "features": ["evidence-tools", "force-validate"],
        }
        with (
            patch.dict(os.environ, {"LD_LIBRARY_PATH": "/runtime/native/lib"}),
            patch.object(
                arrow_validation,
                "resolve",
                side_effect=lambda command, **_kwargs: SimpleNamespace(
                    command=list(command)
                ),
            ),
            patch.object(
                evidence_store.subprocess,
                "run",
                return_value=subprocess.CompletedProcess(
                    [], 0, stdout=json.dumps(cargo)
                ),
            ) as build,
        ):
            self.assertEqual(evidence_store.build(), 0)
            self.assertNotIn("LD_LIBRARY_PATH", build.call_args.kwargs["env"])
            self.assertEqual(os.environ["LD_LIBRARY_PATH"], "/runtime/native/lib")
        published = python_extension.read_publication(
            "evidence-executable", os.environ, root=self.root
        )
        self.assertIsNotNone(published)
        assert published is not None
        self.assertEqual(
            published.artifact.features, ["evidence-tools", "force-validate"]
        )
        self.assertNotEqual(published.artifact.binary, str(self.tool))

    def test_failed_build_preserves_preceding_publication(self) -> None:
        first = executable(self.root, self.tool)
        with (
            patch.object(
                arrow_validation,
                "resolve",
                side_effect=lambda command, **_kwargs: SimpleNamespace(
                    command=list(command)
                ),
            ),
            patch.object(
                evidence_store.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 17, stdout=""),
            ),
        ):
            self.assertEqual(evidence_store.build(), 17)
        self.assertEqual(
            python_extension.read_publication(
                "evidence-executable", os.environ, root=self.root
            ),
            first,
        )


if __name__ == "__main__":
    unittest.main()
