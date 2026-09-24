# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Disposable controls for the implementation-only phase boundary."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import copy
import json
import tempfile
import unittest
from dataclasses import asdict
from pathlib import Path
from unittest.mock import patch

from scripts import implementation_phase as phase
from scripts.validation_scope import Gate


class PhaseTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / "unit.log").write_text("one isolated test passed\n")
        (self.root / "source.rs").write_text("fn example() {}\n")
        self.case = {
            "id": "m00.fixture",
            "package": "M00",
            "owner": "M00",
            "binary": "fixture",
            "tests": ["integrated_unit::example"],
            "class": "unit",
            "mode": "force-validate",
            "positive": "source-qualified evidence",
            "negative": "stale evidence refused",
            "status": "implemented",
        }
        self.declaration = {"plan": 14, "cases": [self.case]}

    def development_fixture(self):
        directory = self.root / "development"
        directory.mkdir()
        log = directory / "unit.log"
        log.write_text("actual runner output")
        report = directory / "unit.xml"
        report.write_text(
            '<testsuite tests="1" failures="0" errors="0" skipped="0"><testcase classname="fixture" name="example"/></testsuite>'
        )
        from scripts import validation

        results, errors = validation.collect_report(report, directory, "unit", 0)
        self.assertFalse(errors)
        case = {
            **self.case,
            "binary": "fixture",
            "tests": ["example"],
            "runner": "unittest",
            "profile": "python-tools",
            "source": "source.rs",
        }
        gate = Gate(
            "unit", phase="development", mode=case["mode"], profile=case["profile"]
        )
        receipt = {
            "version": 3,
            "plan": 14,
            "mode": "development",
            "baseline_failures": 0,
            "complete": True,
            "source_unchanged": True,
            "required_checks_covered": True,
            "provenance_errors": [],
            "source_files": {"source": "current"},
            "scope": [asdict(gate)],
            "parent": None,
            "checks": [
                {
                    "gate": "unit",
                    "status": "passed",
                    "exit_code": 0,
                    "started": 0,
                    "mode": case["mode"],
                    "profile": case["profile"],
                    "results": results,
                    "report_errors": [],
                    "artifacts": {
                        "unit.log": phase.digest(log),
                        "unit.xml": phase.digest(report),
                    },
                }
            ],
        }
        path = directory / "checks.json"
        path.write_text(json.dumps(receipt))
        return directory, receipt, {"plan": 14, "cases": [case]}, gate

    def test_actual_runner_receipt_and_rejection_controls(self):
        directory, receipt, declaration, gate = self.development_fixture()
        with (
            patch.object(phase, "development", return_value=[gate]),
            patch.object(
                phase.validation_tools, "sources", return_value={"source": "current"}
            ),
        ):
            phase.validate_development(self.root, declaration, directory)
            for field, value in [
                ("version", 2),
                ("plan", 13),
                ("mode", "functional"),
                ("complete", False),
                ("source_files", {}),
                ("scope", []),
                ("checks", []),
            ]:
                with self.subTest(field=field), self.assertRaises(ValueError):
                    (directory / "checks.json").write_text(
                        json.dumps({**receipt, field: value})
                    )
                    phase.validate_development(self.root, declaration, directory)
            tampered = copy.deepcopy(receipt)
            tampered["checks"][0]["results"][0]["name"] = "invented"
            (directory / "checks.json").write_text(json.dumps(tampered))
            with self.assertRaises(ValueError):
                phase.validate_development(self.root, declaration, directory)
            (directory / "checks.json").write_text(json.dumps(receipt))
            (directory / "unit.xml").write_text("tampered")
            with self.assertRaises(ValueError):
                phase.validate_development(self.root, declaration, directory)

    def test_native_identity_revalidated_after_success(self):
        from scripts.validation_receipts import verify_native

        library = self.root / "library.so"
        library.write_bytes(b"original linked library")
        native = {
            "schema": "plan14-native-profile-v1",
            "files": {str(library): phase.digest(library)},
            "threads": {
                name: "1"
                for name in (
                    "OMP_NUM_THREADS",
                    "OPENBLAS_NUM_THREADS",
                    "MKL_NUM_THREADS",
                )
            },
        }
        verify_native(native)
        for threads in (
            {"OMP_NUM_THREADS": "1"},
            {**native["threads"], "MKL_NUM_THREADS": "2"},
        ):
            with self.assertRaises(ValueError):
                verify_native({**native, "threads": threads})
        library.write_bytes(b"changed linked library")
        with self.assertRaises(ValueError):
            verify_native(native)

    def test_performance_requires_authenticated_current_complete_functional_receipt(
        self,
    ) -> None:
        (self.root / "Cargo.toml").write_text(
            "[workspace.metadata.pse.execution]\nactive-plan = 14\n"
        )
        directory = self.root / "evidence"
        directory.mkdir()
        log = directory / "test.log"
        log.write_text("one test passed")
        path = directory / "checks.json"
        gate = Gate("test")
        receipt = {
            "version": 3,
            "plan": 14,
            "mode": "functional",
            "baseline_failures": 0,
            "complete": True,
            "source_unchanged": True,
            "required_checks_covered": True,
            "provenance_errors": [],
            "case_coverage": {"complete": True},
            "source_files": {"source": "exact"},
            "scope": [asdict(gate)],
            "parent": None,
            "checks": [
                {
                    "gate": "test",
                    "status": "passed",
                    "artifacts": {"test.log": phase.digest(log)},
                }
            ],
        }

        def save(value: dict) -> None:
            path.write_text(json.dumps(value))

        with (
            patch.object(phase, "comprehensive", return_value=[gate]),
            patch.object(phase, "manifest", return_value={"plan": 14}),
            patch("scripts.validation_cases.coverage", return_value={"complete": True}),
            patch.object(
                phase.validation_tools, "sources", return_value={"source": "exact"}
            ),
        ):
            save(receipt)
            self.assertEqual(phase.require_functional(self.root, directory)["plan"], 14)
            for key, value in [
                ("plan", 11),
                ("mode", "performance"),
                ("complete", False),
                ("source_unchanged", False),
                ("baseline_failures", 1),
                ("case_coverage", {"complete": False}),
                ("checks", []),
                ("scope", []),
                ("source_files", {"source": "old"}),
            ]:
                with self.subTest(key=key), self.assertRaises(ValueError):
                    save({**receipt, key: value})
                    phase.require_functional(self.root, directory)
            save(receipt)
            log.write_text("tampered")
            with self.assertRaises(ValueError):
                phase.require_functional(self.root, directory)

    def test_direct_performance_recipes_cannot_bypass_functional_guard(self) -> None:
        with (
            patch.object(
                phase,
                "manifest",
                return_value={"plan": 14, "blocked_before_barrier": ["bench-smoke"]},
            ),
            patch.object(phase, "preflight"),
            patch.object(
                phase,
                "require_functional",
                side_effect=ValueError("missing functional evidence"),
            ),
        ):
            for name in ("bench-smoke", "bench-cache", "bench-consolidation-native"):
                with (
                    self.subTest(recipe=name),
                    self.assertRaisesRegex(ValueError, "functional evidence"),
                ):
                    phase.guard(self.root, [name])

    def test_every_package_and_deletion_must_close(self) -> None:
        declaration = {
            "plan": 14,
            "inventory": "inventory.md",
            "implementation": ["M00", "M21"],
            "deletions": ["X01"],
        }
        inventory = self.root / "inventory.md"
        inventory.write_text(
            "| M00 | fixture | complete |\n| M21 | fixture | open |\n| X01 | fixture | complete |\n"
        )
        with self.assertRaises(ValueError):
            phase.require_closed(self.root, declaration)
        inventory.write_text(inventory.read_text().replace("open", "complete"))
        phase.require_closed(self.root, declaration)
        inventory.write_text(inventory.read_text() + "| M00 | duplicate | complete |\n")
        with self.assertRaises(ValueError):
            phase.require_closed(self.root, declaration)

    def test_transitive_campaign_guard_and_unit_allowance(self) -> None:
        declaration = {"blocked_before_barrier": ["test", "codegen-relations-check"]}
        with (
            patch.object(phase, "manifest", return_value=declaration),
            patch.object(
                phase, "preflight", side_effect=ValueError("open")
            ) as preflight,
        ):
            phase.guard(self.root, ["unit-package", "codegen", "check-library"])
            preflight.assert_not_called()
            for group in ["ci-fast", "governance", "codegen-check"]:
                with self.subTest(group=group), self.assertRaises(ValueError):
                    phase.guard(self.root, [group])

    def test_manifest_refuses_duplicate_case_identity(self) -> None:
        root = Path(__file__).resolve().parents[2]
        text = (root / "docs/plans/14-acceptance-cases.toml").read_text()
        (self.root / "Cargo.toml").write_text(
            "[workspace.metadata.pse.execution]\nactive-plan = 14\n"
        )
        path = self.root / "docs/plans"
        path.mkdir(parents=True)
        declaration = path / "14-acceptance-cases.toml"
        declaration.write_text(text)
        value = phase.manifest(self.root)
        declaration.write_text(
            text + '\n[[cases]]\nid = "' + value["cases"][0]["id"] + '"\n'
        )
        with self.assertRaises(ValueError):
            phase.manifest(self.root)

    def test_active_plan_and_historical_bypass(self) -> None:
        (self.root / "Cargo.toml").write_text(
            "[workspace.metadata.pse.execution]\nactive-plan = 14\n"
        )
        self.assertEqual(phase.execution_plan(self.root, None), 14)
        self.assertEqual(phase.execution_plan(self.root, 14), 14)
        with self.assertRaises(ValueError):
            phase.execution_plan(self.root, 11)

    def test_target_scope_refuses_inherited_and_unclassified_cases(self) -> None:
        root = Path(__file__).resolve().parents[2]
        value = phase.manifest(root)
        for change in (
            {"plan": 13},
            {"carried": [{"id": "old"}]},
            {"cases": [{"id": "plan13:Q01", "owner": "M00"}]},
            {"cases": [{"id": "new", "current_source": "old.rs", "owner": "M00"}]},
            {"cases": [{"id": "new"}]},
        ):
            with self.subTest(change=change), self.assertRaises(ValueError):
                phase.validate_scope({**value, **change})
        phase.validate_scope(value)

    def test_old_source_seal_cannot_authorize_current_plan(self) -> None:
        (self.root / "Cargo.toml").write_text(
            "[workspace.metadata.pse.execution]\nactive-plan = 14\n"
        )
        (self.root / "barrier.json").write_text(json.dumps({"version": 1, "plan": 13}))
        with (
            patch.object(
                phase, "manifest", return_value={"plan": 14, "barrier": "barrier.json"}
            ),
            patch.object(phase, "require_closed"),
            self.assertRaisesRegex(ValueError, "wrong plan"),
        ):
            phase.preflight(self.root)


if __name__ == "__main__":
    unittest.main()
