# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Focused source-only inventory and real declaration inspection controls."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.capabilities import inventory as subject

ROOT = Path(__file__).resolve().parents[2]


class InventoryTests(unittest.TestCase):
    def test_metadata_is_source_only_and_all_references_have_owners(self) -> None:
        with patch.object(subject, "_just_dump", side_effect=AssertionError("startup ran Just")), patch.object(
            subject.subprocess, "Popen", side_effect=AssertionError("startup launched a process")
        ):
            domains, workflows, guides, capabilities = subject.inventory(ROOT)
        domain_ids = {row.id for row in domains}
        workflow_ids = {row.id for row in workflows}
        guide_ids = {row.id for row in guides}
        self.assertEqual(len(domain_ids), len(domains))
        self.assertEqual(len(workflow_ids), len(workflows))
        self.assertEqual(len(guide_ids), len(guides))
        self.assertEqual(len({row.id for row in capabilities}), len(capabilities))
        self.assertEqual(domain_ids, {"workspace", "editing", "knowledge", "environment", "validation", "execution", "diagnostics", "storage", "projects", "evidence", "experiments", "generation", "governance", "integration"})
        self.assertEqual(workflow_ids, {"inspect-plan", "change-generated-contract", "plan-functional-review", "diagnose-run", "qualify-change", "investigate-evidence", "manage-project", "run-experiment"})
        for row in guides:
            self.assertTrue((ROOT / row.path).is_file(), row.path)
        for row in (*domains, *workflows, *capabilities):
            self.assertLessEqual(set(row.guides), guide_ids, row.id)
        for row in capabilities:
            self.assertIn(row.domain, domain_ids)
            self.assertLessEqual(set(row.workflows), workflow_ids)
            self.assertTrue((ROOT / row.owner).is_file(), row.owner)
            self.assertTrue((ROOT / row.source).is_file(), row.source)
            self.assertNotEqual(row.exposure, "tool", "Only the composition root owns real callable bindings")
            self.assertTrue(row.reason)
            if row.exposure == "deferred":
                self.assertTrue(row.work_owner)
                self.assertTrue((ROOT / row.work_owner.split("#")[0]).is_file())

    def test_representative_families_retain_real_routes_and_honest_maturity(self) -> None:
        capabilities = {row.id: row for row in subject.inventory(ROOT)[3]}
        expected = {
            "language-navigation": ("editing", "scripts/semantic_navigation.py"),
            "knowledge-recall": ("knowledge", "scripts/knowledge_store.py"),
            "build-cache": ("environment", "scripts/build_environment.py"),
            "validation-selection": ("validation", "scripts/select.py"),
            "workload-observation": ("execution", "scripts/activity.py"),
            "scientific-diagnostics": ("diagnostics", "python/pse/diagnostics.py"),
            "database-tooling": ("storage", "scripts/surreal_server.py"),
            "projects-revisions": ("projects", "xtask/src/project_tools.rs"),
            "scientific-inputs": ("projects", "scripts/scientific_inputs.py"),
            "evidence-inspection": ("evidence", "xtask/src/evidence_tools.rs"),
            "experiments-recovery": ("experiments", "scripts/experiments.py"),
            "generated-contracts": ("generation", "xtask/src/codegen.rs"),
            "decisions-register": ("governance", "scripts/adr.py"),
            "session-specialists": ("integration", "scripts/codex_session.py"),
        }
        for identity, (domain, owner) in expected.items():
            row = capabilities[identity]
            self.assertEqual((row.domain, row.owner), (domain, owner))
            self.assertTrue(row.route)
            self.assertEqual(row.exposure, "deferred")
        self.assertEqual(capabilities["host-implementation"].exposure, "internal")
        self.assertEqual(capabilities["git-integration"].exposure, "native")
        self.assertEqual(capabilities["scientific-python-api"].exposure, "native")
        self.assertIn("Rust reader/parser/P0", capabilities["scientific-inputs"].description)
        self.assertIn("ADP04", capabilities["scientific-inputs"].work_owner)
        self.assertIn("Instruction retrieval only", capabilities["skill-plan-functional-extension-review"].effects)

    def test_every_current_public_recipe_is_classified_without_running_it(self) -> None:
        declaration = subject._just_dump(ROOT)
        expected = {name for name, row in declaration["recipes"].items() if not row.get("private")}
        pages = []
        offset = 0
        while True:
            page = subject.inspect_entrypoints(ROOT, offset=offset, limit=100, kind="recipe")
            self.assertLessEqual(len(page["items"]), 100)
            self.assertEqual(page["unclassified_counts"]["recipe"], 0)
            pages.extend(page["items"])
            if page["next_offset"] is None:
                break
            offset = page["next_offset"]
        self.assertEqual({row["identity"] for row in pages}, expected)
        self.assertEqual(len(pages), len(expected))
        by_name = {row["identity"]: row for row in pages}
        self.assertEqual(by_name["evidence"]["capability"], "evidence-inspection")
        self.assertEqual(by_name["coverage"]["capability"], "verification-techniques")
        self.assertEqual(by_name["canonical-recovery-test"]["capability"], "experiments-recovery")
        self.assertEqual(by_name["context"]["outer_parameters"], declaration["recipes"]["context"]["parameters"])
        self.assertIn("leaf arguments/defaults remain owned", by_name["context"]["contract"])

    def test_exact_recipe_description_reaches_leaf_source_without_fabricating_schema(self) -> None:
        result = subject.describe_recipe(ROOT, "experiment")
        self.assertEqual(result["name"], "experiment")
        self.assertFalse(result["leaf_schema_complete"])
        sources = {row["path"]: row for row in result["leaf_contracts"]}
        self.assertIn("scripts/experiments.py", sources)
        self.assertTrue(sources["scripts/experiments.py"]["excerpt"])
        self.assertIn("add_argument", str(sources["scripts/experiments.py"]["excerpt"]))
        self.assertEqual(len(result["source"]["captured_sha256"]), 64)
        self.assertEqual(result["definition_route"], ["just", "--show", "experiment"])
        for name in ("does-not-exist", "_preflight"):
            with self.assertRaisesRegex(ValueError, "Unknown public recipe"):
                subject.describe_recipe(ROOT, name)

    def test_unknown_candidates_are_counted_and_not_guessed_from_prose(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for path in ("scripts", "xtask/src/bin", "python/pse", ".codex/skills"):
                (root / path).mkdir(parents=True)
            (root / "scripts/future_inspector.py").write_text("# Inspect evidence and projects.\n")
            declaration = {"recipes": {
                "future-tool": {"doc": "Inspect evidence and projects", "parameters": []},
                "evidence": {"doc": "Real declared recipe", "parameters": []},
                "_hidden": {"private": True},
            }}
            with patch.object(subject, "_just_dump", return_value=declaration):
                page = subject.inspect_entrypoints(root, limit=100)
            self.assertEqual(page["unclassified_counts"]["recipe"], 1)
            self.assertEqual(page["unclassified_counts"]["script"], 1)
            self.assertFalse(page["complete"])
            by_id = {row["identity"]: row for row in page["items"]}
            self.assertNotIn("_hidden", by_id)
            self.assertEqual(by_id["future-tool"]["classification"], "unclassified")
            self.assertIsNone(by_id["future-tool"]["capability"])
            self.assertEqual(by_id["future-tool"]["work_owner"], subject.CAP05)

    def test_descriptor_reads_current_source_and_retains_read_bounds(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            (root / "justfile").write_text("new-command *args:\n  python scripts/new.py\n")
            source = root / "scripts/new.py"
            source.write_text("parser.add_argument('--original')\n")
            declaration = {"recipes": {"new-command": {"body": [["python scripts/new.py"]]}}}
            with patch.object(subject, "_just_dump", return_value=declaration):
                before = subject.describe_recipe(root, "new-command")
                source.write_text("parser.add_argument('--changed')\n")
                after = subject.describe_recipe(root, "new-command")
                source.write_text("#" * (300 << 10))
                bounded = subject.describe_recipe(root, "new-command")
            self.assertNotEqual(before["leaf_contracts"][0]["captured_sha256"], after["leaf_contracts"][0]["captured_sha256"])
            self.assertIn("--changed", str(after["leaf_contracts"][0]["excerpt"]))
            self.assertTrue(bounded["leaf_contracts"][0]["source_truncated"])
            self.assertEqual(bounded["leaf_contracts"][0]["captured_bytes"], 256 << 10)
            self.assertLessEqual(len(str(bounded)), 12000)

    def test_inspection_rejects_invalid_selection_before_starting_just(self) -> None:
        with patch.object(subject, "_just_dump", side_effect=AssertionError("invalid request launched Just")):
            for kwargs in ({"limit": 0}, {"limit": 101}, {"offset": -1}, {"kind": "similarity"}):
                with self.assertRaises(ValueError):
                    subject.inspect_entrypoints(ROOT, **kwargs)

    def test_just_output_is_bounded_and_failed_child_is_drained(self) -> None:
        original = subprocess.Popen
        processes = []

        def launch(*args, **kwargs):
            process = original([sys.executable, "-c", "import sys,time;sys.stdout.write('x'*(5<<20));sys.stdout.flush();time.sleep(30)"], **kwargs)
            processes.append(process)
            return process

        with patch.object(subject.subprocess, "Popen", side_effect=launch):
            with self.assertRaisesRegex(ValueError, "exceeded 4 MiB"):
                subject._just_dump(ROOT)
        self.assertEqual(len(processes), 1)
        self.assertIsNotNone(processes[0].poll())


if __name__ == "__main__":
    unittest.main()
