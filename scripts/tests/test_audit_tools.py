# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Audit findings and invocation failures remain different evidence."""

# ruff: noqa: PT009
import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from scripts import audit_tools


class AuditTests(unittest.TestCase):
    def test_geiger_cannot_clean_the_shared_cargo_target(self) -> None:
        with (
            patch.object(
                audit_tools, "command_env", return_value={"CARGO_TARGET_DIR": "/shared"}
            ),
            patch.object(
                audit_tools.subprocess,
                "run",
                return_value=SimpleNamespace(returncode=0, stdout="{}", stderr=""),
            ) as run,
        ):
            receipt = audit_tools.invoke(["cargo", "geiger"], Path("/repo"))
        self.assertEqual(
            run.call_args.kwargs["env"]["CARGO_TARGET_DIR"],
            "/repo/build/audits/geiger-target",
        )
        self.assertEqual(
            receipt["target_directory"], "/repo/build/audits/geiger-target"
        )

    def test_deny_error_mask_requires_complete_summary(self) -> None:
        summary = {
            "type": "summary",
            "fields": {
                name: {"errors": count}
                for name, count in (
                    ("advisories", 3),
                    ("bans", 1),
                    ("licenses", 0),
                    ("sources", 0),
                )
            },
        }
        self.assertEqual(
            audit_tools.finding_status(
                "audit-dependencies", 3, "", json.dumps(summary)
            ),
            "findings",
        )
        self.assertEqual(
            audit_tools.finding_status(
                "audit-dependencies", 1, "", json.dumps(summary)
            ),
            "failed",
        )
        self.assertEqual(
            audit_tools.finding_status("audit-dependencies", 3, "", "network error"),
            "failed",
        )

    def test_shear_requires_actual_findings_and_reconciled_summary(self) -> None:
        report = {
            "summary": {"errors": 1, "warnings": 0},
            "findings": [{"code": "shear/unused_dependency", "severity": "error"}],
        }
        self.assertEqual(
            audit_tools.finding_status("audit-shear", 1, json.dumps(report), ""),
            "findings",
        )
        self.assertEqual(
            audit_tools.finding_status(
                "audit-shear", 1, '{"error":"failed to parse manifest"}', ""
            ),
            "failed",
        )
        self.assertEqual(
            audit_tools.finding_status("audit-shear", 2, json.dumps(report), ""),
            "failed",
        )

    def test_shear_warning_only_findings_are_strict_and_reports_are_required(
        self,
    ) -> None:
        warning = {
            "summary": {"errors": 0, "warnings": 1},
            "findings": [{"code": "shear/shared_path", "severity": "warning"}],
        }
        clean = {"summary": {"errors": 0, "warnings": 0}, "findings": []}
        self.assertEqual(
            audit_tools.finding_status("audit-shear", 1, json.dumps(warning), ""),
            "findings",
        )
        self.assertEqual(
            audit_tools.finding_status("audit-shear", 0, json.dumps(clean), ""),
            "passed",
        )
        for code, report in (
            (0, ""),
            (0, "{}"),
            (0, json.dumps(warning)),
            (1, json.dumps(clean)),
            (2, json.dumps(warning)),
            (0, '{"findings":[],"summary":{"errors":false,"warnings":0}}'),
            (0, '{"findings":[null],"summary":{"errors":0,"warnings":0}}'),
        ):
            with self.subTest(code=code, report=report):
                self.assertEqual(
                    audit_tools.finding_status("audit-shear", code, report, ""),
                    "failed",
                )

    def test_shear_has_one_exact_read_only_command_and_preserves_wrapper_exits(
        self,
    ) -> None:
        command = [
            "cargo",
            "shear",
            "--locked",
            "--check-test-targets",
            "--deny-warnings",
            "--format",
            "json",
        ]
        self.assertEqual(audit_tools.COMMANDS["audit-shear"], command)
        self.assertNotIn("audit-machete", audit_tools.COMMANDS)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = root / "Cargo.toml"
            lock = root / "Cargo.lock"
            vendor = root / "vendor/Cargo.lock"
            vendor.parent.mkdir()
            for path in (manifest, lock, vendor):
                path.write_text("original bytes\n")
            for status, expected_exit in (
                ("passed", 0),
                ("findings", 1),
                ("failed", 2),
            ):
                record = {"status": status, "exit_code": expected_exit}
                with (
                    patch.object(audit_tools, "invoke", return_value=record) as invoke,
                    patch.object(audit_tools, "finding_status", return_value=status),
                ):
                    record.update(stdout="{}", stderr="")
                    self.assertEqual(
                        audit_tools.run(root, "audit-shear", root), expected_exit
                    )
                invoke.assert_called_once_with(command, root)
                receipt = json.loads((root / "audit-shear-tool.json").read_text())
                self.assertEqual(receipt["status"], status)
                self.assertEqual(receipt["invocations"], [record])
            self.assertTrue(
                all(
                    path.read_text() == "original bytes\n"
                    for path in (manifest, lock, vendor)
                )
            )

    def test_shear_missing_executable_fails_with_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with (
                patch.object(audit_tools, "command_env", return_value={}),
                patch.object(
                    audit_tools.subprocess,
                    "run",
                    side_effect=FileNotFoundError("cargo"),
                ),
            ):
                self.assertEqual(audit_tools.run(root, "audit-shear", root), 2)
            receipt = json.loads((root / "audit-shear-tool.json").read_text())
            self.assertEqual(receipt["status"], "failed")
            self.assertIsNone(receipt["invocations"][0]["exit_code"])

    def test_virtual_workspace_enumerates_only_real_members(self) -> None:
        metadata = {
            "workspace_members": ["a", "b"],
            "packages": [
                {"id": key, "name": key, "manifest_path": f"/repo/{key}/Cargo.toml"}
                for key in ("a", "external", "b")
            ],
        }
        with patch.object(
            audit_tools.subprocess,
            "check_output",
            return_value=json.dumps(metadata).encode(),
        ):
            commands = audit_tools.unsafe_members(Path("/repo"))
        self.assertEqual(len(commands), 2)
        self.assertTrue(all("--workspace" not in command for command in commands))
        self.assertEqual(
            [command[command.index("--package") + 1] for command in commands],
            ["a", "b"],
        )
