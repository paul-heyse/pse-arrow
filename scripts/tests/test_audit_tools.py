# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Audit findings and invocation failures remain different evidence."""

# ruff: noqa: PT009
import json
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
