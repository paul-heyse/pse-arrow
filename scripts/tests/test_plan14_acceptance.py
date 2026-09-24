# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated controls for current-source, exact-witness qualification contracts."""

from __future__ import annotations
import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from scripts import implementation_phase as phase, validation, validation_cases as cases
from scripts.plan14_acceptance import expected, verify_selection
from scripts.validation_scope import comprehensive

ROOT = Path(__file__).resolve().parents[2]


class AcceptanceTests(unittest.TestCase):
    def test_removed_mechanisms_and_current_dependency_roles(self):
        self.assertFalse((ROOT / "crates/pse-plans").exists())
        self.assertFalse((ROOT / "crates/pse-kernels-ext").exists())
        source = (ROOT / "crates/pse-schema/src/catalog/s6_3_domains.rs").read_text()
        for obsolete in [
            '"meshes"',
            '"mesh_nodes"',
            '"stencils"',
            '"quadrature_rules"',
        ]:
            self.assertNotIn(obsolete, source)
        semantic = (
            ROOT / "crates/pse-codegen/src/codegen/rust/semantic.rs"
        ).read_text()
        self.assertNotIn("fn share_payloads", semantic)
        import tomllib

        py = tomllib.loads((ROOT / "pyproject.toml").read_text())
        self.assertFalse(
            any(p.split("=")[0] == "pyomo" for p in py["project"]["dependencies"])
        )
        self.assertTrue(
            any(
                p.startswith("numpy")
                for p in [
                    p
                    for dependencies in py["project"]["optional-dependencies"].values()
                    for p in dependencies
                ]
            )
        )
        names = validation.sources(ROOT)
        self.assertNotIn(".envrc.local", names)
        self.assertFalse(any("/skills/deltalake/" in p for p in names))
        self.assertIn("Cargo.lock", names)

    def test_exact_witnesses_and_phase_boundaries(self):
        declaration = phase.manifest(ROOT)
        phase.validate_sources(ROOT, declaration)
        self.assertEqual(len(declaration["acceptance"]), 18)
        self.assertEqual(
            {r["id"] for r in declaration["acceptance"] if r["phase"] == "performance"},
            {"Q18"},
        )
        for gate in ("plan14-native", "plan14-python", "codegen-contracts-check", "codegen"):
            phase.guard(ROOT, [gate])
        for gate in ("plan14-measure", "plan14-reviews", "bench-smoke"):
            with (
                self.subTest(gate=gate),
                patch.object(phase, "require_functional", side_effect=ValueError("functional evidence")),
                self.assertRaisesRegex(ValueError, "functional evidence"),
            ):
                phase.guard(ROOT, [gate])
        self.assertEqual(
            [g.name for g in comprehensive("performance")],
            ["plan14-measure", "plan14-reviews"],
        )

    def test_manifest_and_discovery_reject_partial_wrong_profile_and_stale_sources(
        self,
    ):
        declaration = phase.manifest(ROOT)
        selected = [
            dict(zip(("class", "name"), r, strict=True))
            for r in expected(declaration, "rust-native")
        ]
        verify_selection(declaration, "rust-native", selected)
        for wrong in [
            selected[:-1],
            selected + [selected[0]],
            [{"class": "wrong", "name": r["name"]} for r in selected],
        ]:
            with self.assertRaises(ValueError):
                verify_selection(declaration, "rust-native", wrong)
        c = next(c for c in declaration["cases"] if c["id"] == "m20.physical-nlp")
        check = {
            "gate": "plan14-native",
            "status": "passed",
            "mode": c["mode"],
            "profile": c["profile"],
            "results": [
                {"class": c["binary"], "name": t, "status": "passed"}
                for t in c["tests"]
            ],
        }
        self.assertTrue(cases.case_witnesses(c, [check]))
        for change in [
            {"profile": "rust-boundary"},
            {"results": []},
            {"status": "interrupted"},
            {"mode": "default"},
        ]:
            self.assertFalse(cases.case_witnesses(c, [{**check, **change}]))
        bad = copy.deepcopy(declaration)
        bad["cases"][0]["profile"] = "absent"
        with self.assertRaises(ValueError):
            cases.validate_manifest(bad)
        for mutation in ["duplicate", "runner"]:
            bad = copy.deepcopy(declaration)
            if mutation == "duplicate":
                bad["cases"].append(bad["cases"][0])
            else:
                bad["cases"][0]["runner"] = "pytest"
            with self.assertRaises(ValueError):
                cases.validate_manifest(bad)
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with self.assertRaises(ValueError):
                phase.validate_sources(root, declaration)
        for c in (c for c in declaration["cases"] if c.get("artifact")):
            with self.assertRaises(ValueError):
                cases.validate_artifact(
                    {
                        "schema": c["artifact"],
                        "profile": c["profile"],
                        "source_digest": "0" * 64,
                        "cases": [{"id": t} for t in c["tests"]],
                    },
                    c,
                )

    def test_measurement_and_review_content_contracts(self):
        declaration = phase.manifest(ROOT)
        cost = next(
            c for c in declaration["cases"] if c.get("artifact") == "process-cost-v1"
        )
        digest = "1" * 64
        threads = {
            name: "1"
            for name in ["OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"]
        }
        report = {
            "schema": cost["artifact"],
            "profile": cost["profile"],
            "source_digest": digest,
            "binary_digest": digest,
            "binary_path": "/fixture/benchmark",
            "toolchain": "fixture toolchain",
            "thread_environment": threads,
            "native": {
                "schema": "plan14-native-profile-v1",
                "profile": {"id": cost["profile"]},
                "threads": threads,
                "toolchain": "fixture toolchain",
                "files": {"/fixture/benchmark": digest},
                "links": {"/fixture/benchmark": "fixture"},
            },
            "cases": [],
        }
        for name in cost["tests"]:
            _, size, count = name.split("-")
            blocks = {"small": 1, "medium": 8, "large": 32}[size]
            report["cases"].append(
                {
                    "id": name,
                    "pool_peak_bytes": 100,
                    "process_peak_rss_bytes": 200,
                    "mean_nanoseconds": 1000,
                    "iterations": 20,
                    "sample_count": 10,
                    "native_threads": 1,
                    "threads": int(count),
                    "blocks": blocks,
                    "variables": 3 * blocks,
                    "artifacts": {name + ".json": digest},
                    "confidence_interval": {
                        "confidence_level": 0.95,
                        "lower_bound": 900,
                        "upper_bound": 1100,
                    },
                }
            )
        cases.validate_artifact(report, cost)
        for edit in [
            "metric",
            "shape",
            "threads",
            "samples",
            "native",
            "duplicate",
            "digest",
        ]:
            bad = copy.deepcopy(report)
            if edit == "metric":
                bad["cases"][0]["pool_peak_bytes"] = float("nan")
            if edit == "shape":
                bad["cases"][0]["variables"] += 1
            if edit == "threads":
                bad["thread_environment"] = {"OMP_NUM_THREADS": "1"}
            if edit == "samples":
                bad["cases"][0]["sample_count"] = 0
            if edit == "native":
                bad.pop("native")
            if edit == "duplicate":
                bad["cases"].append(bad["cases"][0])
            if edit == "digest":
                bad["cases"][0]["artifacts"] = {"file": "invalid"}
            with self.subTest(edit=edit), self.assertRaises(ValueError):
                cases.validate_artifact(bad, cost)
        review = next(
            c
            for c in declaration["cases"]
            if c.get("artifact") == "architecture-review-v1"
        )
        report = {
            "schema": review["artifact"],
            "profile": review["profile"],
            "source_digest": digest,
            "cases": [
                {
                    "id": gate,
                    "gate": gate,
                    "source_digest": digest,
                    "verdict": "Accept-scoped",
                    "reviewer": "independent",
                    "implementation_authors": ["author"],
                    "scope": "fixture scope",
                    "open_must_findings": 0,
                    "evidence": [{"path": "fixture.md", "digest": digest}],
                    "review_path": "review.json",
                    "review_digest": digest,
                    "artifacts": {"fixture.md": digest, "review.json": digest},
                }
                for gate in review["tests"]
            ],
        }
        cases.validate_artifact(report, review)
        for edit in ["identity", "source", "finding", "evidence"]:
            bad = copy.deepcopy(report)
            if edit == "identity":
                bad["cases"][0]["reviewer"] = "author"
            if edit == "source":
                bad["cases"][0]["source_digest"] = "2" * 64
            if edit == "finding":
                bad["cases"][0]["open_must_findings"] = 1
            if edit == "evidence":
                bad["cases"][0]["artifacts"].pop("fixture.md")
            with self.subTest(edit=edit), self.assertRaises(ValueError):
                cases.validate_artifact(bad, review)
