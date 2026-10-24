# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Frozen enumeration, per-attempt origin authentication and conservative retry."""
# ruff: noqa: PT009, PT027 -- stdlib unittest controls

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import validation_feature_campaign as campaign
from scripts import validation_receipts as receipts


class FeatureCampaignTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.commands = [
            [
                "check",
                "--manifest-path",
                'odd path/with "quotes"/Cargo.toml',
                "--features",
                feature,
            ]
            for feature in ("a", "b", "a,b")
        ]
        self.source = {"crates/member/src/lib.rs": "before"}
        self.executed = []
        self.statuses = ["passed", "interrupted"]
        for selected in (
            patch.object(
                campaign,
                "compiler_context",
                return_value={"complete": "compiler-native-fixture"},
            ),
            patch.object(
                campaign.validation,
                "sources",
                side_effect=lambda _root, **_kwargs: dict(self.source),
            ),
            patch.object(
                campaign,
                "enumerate_commands",
                return_value=(
                    self.commands,
                    {
                        "tool": {"version": "0.6.45"},
                        "command": ["cargo-hack", "hack"],
                        "capture_sha256": "0" * 64,
                    },
                ),
            ),
            patch.object(campaign.validation, "execute", side_effect=self.execute),
        ):
            selected.start()
            self.addCleanup(selected.stop)

    def execute(
        self,
        _root: Path,
        output: Path,
        name: str,
        command: list[str],
        env: dict[str, str],
    ) -> dict:
        self.executed.append(command)
        status = self.statuses.pop(0) if self.statuses else "passed"
        log = output / f"{name}.log"
        log.write_text(status)
        literal = command[2:]
        execution = {
            "schema": "cargo-feature-execution-v1",
            "literal_argv": literal,
            "qualified_argv": literal,
            "actual_argv": ["cargo", *literal],
            "correctness": "arrow-unit-graph-force-validation-v1",
            "effective_environment": {"fixture": "1" * 64},
            "validation_graphs": [
                {"argv": ["cargo", "check", "--unit-graph"], "sha256": "1" * 64}
            ],
        }
        Path(env[campaign.ATTEMPT]).write_text(json.dumps(execution))
        return {
            "status": status,
            "exit_code": 0 if status == "passed" else -2,
            "log": log.name,
            "report_errors": [],
        }

    def test_interrupt_resume_reuses_only_completed_passes_then_source_repair_invalidates_all(
        self,
    ) -> None:
        first = self.root / "first"
        second = self.root / "second"
        third = self.root / "third"
        self.assertEqual(
            campaign.run(self.root, first, "combinations", environment={}), 130
        )
        prior = receipts.feature_campaign(first)
        self.assertEqual(
            [row["status"] for row in prior["combinations"]],
            ["passed", "interrupted", "not_run"],
        )
        self.assertEqual(
            campaign.run(
                self.root, second, "combinations", resume_from=first, environment={}
            ),
            0,
        )
        resumed = receipts.feature_campaign(second)
        self.assertEqual(len(self.executed), 4)
        self.assertEqual(
            resumed["combinations"][0]["evidence_kind"], "unchanged-input-reuse"
        )
        self.assertEqual(resumed["combinations"][1]["history"]["status"], "interrupted")
        self.source["crates/member/src/lib.rs"] = "repaired"
        self.assertEqual(
            campaign.run(
                self.root, third, "combinations", resume_from=second, environment={}
            ),
            0,
        )
        self.assertEqual(len(self.executed), 7)
        self.assertTrue(
            all(
                row["evidence_kind"] == "executed"
                for row in receipts.feature_campaign(third)["combinations"]
            )
        )

    def test_original_artifact_change_refuses_reuse(self) -> None:
        first = self.root / "first"
        campaign.run(self.root, first, "combinations", environment={})
        (first / "feature-00000.log").write_text("tampered")
        with self.assertRaisesRegex(ValueError, "changed original feature artifact"):
            campaign.run(
                self.root,
                self.root / "second",
                "combinations",
                resume_from=first,
                environment={},
            )

    def test_missing_failed_or_unproved_combination_prevents_qualification(
        self,
    ) -> None:
        self.statuses = ["passed", "failed", "passed"]
        directory = self.root / "first"
        self.assertEqual(
            campaign.run(self.root, directory, "combinations", environment={}), 1
        )
        report = receipts.feature_campaign(directory)
        self.assertTrue(report["complete"])
        self.assertFalse(report["required_checks_covered"])
        report["combinations"].pop()
        (directory / "feature-campaign.json").write_text(json.dumps(report))
        with self.assertRaisesRegex(ValueError, "missing feature combination"):
            receipts.feature_campaign(directory)

    def test_changed_compiler_context_disables_reuse(self) -> None:
        self.statuses = []
        first = self.root / "first"
        campaign.run(self.root, first, "combinations", environment={})
        with patch.object(
            campaign, "compiler_context", return_value={"complete": "changed compiler"}
        ):
            self.assertEqual(
                campaign.run(
                    self.root,
                    self.root / "second",
                    "combinations",
                    resume_from=first,
                    environment={},
                ),
                0,
            )
        self.assertEqual(len(self.executed), 6)

    def test_direct_recipe_uses_existing_assessment_owner(self) -> None:
        selected = self.root / "assessment"
        with (
            patch.dict(campaign.os.environ, {}, clear=True),
            patch.object(campaign.validation, "fresh_output", return_value=selected),
            patch.object(campaign.validation, "run_gates", return_value=0) as owner,
            patch.object(campaign, "run") as execute,
        ):
            self.assertEqual(campaign.main(["combinations"]), 0)
        execute.assert_not_called()
        call = owner.call_args
        self.assertEqual(call.args[:2], (campaign.ROOT, selected))
        self.assertEqual(call.args[2][0].name, "features-combinations")
        self.assertEqual(call.args[2][0].input_scope, "rust-product")


if __name__ == "__main__":
    unittest.main()
