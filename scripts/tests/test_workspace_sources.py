# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent source questions, freshness cuts and research provenance."""

# ruff: noqa: PT009 -- stdlib setup controls
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import docs
from scripts import workspace_sources as sources


class SourceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)
        self.write(
            "docs/site.toml",
            """[publication]
exclude = []
current_work = ["plans/active.md"]
entrypoints = ["README.md"]
[[collections]]
root = "."
recursive = false
title = "Entry"
scope = "Current"
[[collections]]
root = "contracts"
title = "Contract"
scope = "Current"
[[collections]]
root = "plans"
title = "Plans"
scope = "History"
[[collections]]
root = "reviews"
title = "Reviews"
scope = "History"
""",
        )
        self.write("docs/README.md", "# Selected checkout\n")
        self.write(
            "docs/contracts/freshness.md",
            "---\ndoc_role: contract\ndoc_owner: docs/contracts/freshness.md\n---\n# Freshness\n\n## 14.3 Acceptance\n\nReject stale results at acceptance.\n",
        )
        self.write(
            "docs/plans/active.md",
            "---\nstatus: in-progress\ndoc_role: plan\n---\n# Current work\n\nUse result_guard for acceptance.\n",
        )
        self.write(
            "docs/reviews/obsolete.md",
            "---\nstatus: complete\ndoc_role: review\n---\n# Obsolete advice\n\nStale results acceptance result_guard stale results acceptance.\n",
        )
        self.write(
            "src/guard.rs",
            "pub fn result_guard() -> bool { true }\n// Reject stale results during acceptance.\n",
        )
        subprocess.run(["git", "add", "."], cwd=self.root, check=True)

    def write(self, relative: str, text: str) -> Path:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        return path

    def test_concept_question_returns_independently_named_current_contract_and_code(
        self,
    ) -> None:
        result = sources.search("stale results acceptance", root=self.root)
        names = [row["path"] for row in result["items"]]
        self.assertIn("docs/contracts/freshness.md", names)
        self.assertIn("src/guard.rs", names)
        self.assertGreater(
            names.index("docs/reviews/obsolete.md"),
            names.index("docs/contracts/freshness.md"),
        )
        contract = next(
            row
            for row in result["items"]
            if row["path"] == "docs/contracts/freshness.md"
        )
        self.assertEqual(contract["role"], "contract")
        self.assertEqual(contract["owner"], "docs/contracts/freshness.md")
        self.assertEqual(contract["excerpts"][0]["section_id"], "14.3")
        self.assertTrue(result["complete"])
        self.assertIn("no database required", result["meaning"])

    def test_scope_is_identical_to_native_docs_discovery_including_current_override(
        self,
    ) -> None:
        native = {
            str(page.path): page.scope
            for page in docs.discover(self.root, docs.configuration(self.root))
        }
        for path, scope in native.items():
            self.assertEqual(
                sources.source("docs/" + path, root=self.root)["scope"], scope
            )
        self.assertEqual(
            sources.source("docs/plans/active.md", root=self.root)["scope"], "Current"
        )
        self.write(
            "docs/site.toml",
            (self.root / "docs/site.toml").read_text()
            + """\n[[overrides]]\npatterns = ["contracts/freshness.md"]\nscope = "History"\n""",
        )
        self.assertEqual(
            sources.source("docs/contracts/freshness.md", root=self.root)["scope"],
            "History",
        )

    def test_literal_query_roles_and_dirty_overlay_have_exact_excerpts_and_identity(
        self,
    ) -> None:
        selected = sources.search(
            "result_guard()", root=self.root, literal=True, roots=("src",)
        )
        self.assertEqual([row["path"] for row in selected["items"]], ["src/guard.rs"])
        before = selected["items"][0]
        self.write(
            "src/guard.rs",
            "pub fn result_guard() -> bool { false }\n// dirty replacement\n",
        )
        after = sources.search("dirty replacement", root=self.root, literal=True)
        self.assertEqual(after["items"][0]["path"], "src/guard.rs")
        self.assertNotEqual(before["digest"], after["items"][0]["digest"])
        self.assertIn("dirty replacement", after["items"][0]["excerpts"][0]["excerpt"])
        with self.assertRaisesRegex(ValueError, "changed since selection"):
            sources.source(
                before["path"], root=self.root, expected_digest=before["digest"]
            )
        filtered = sources.search("stale", root=self.root, roles=("contract",))
        self.assertEqual(
            [row["path"] for row in filtered["items"]], ["docs/contracts/freshness.md"]
        )

    def test_deleted_and_renamed_sources_invalidate_selected_detail_and_membership(
        self,
    ) -> None:
        selected = sources.source("src/guard.rs", root=self.root)
        (self.root / "src/guard.rs").rename(self.root / "src/renamed.rs")
        with self.assertRaisesRegex(ValueError, "absent"):
            sources.source(
                "src/guard.rs", root=self.root, expected_digest=selected["digest"]
            )
        result = sources.search("result_guard", root=self.root, literal=True)
        self.assertNotIn("src/guard.rs", [row["path"] for row in result["items"]])
        self.assertIn("src/renamed.rs", [row["path"] for row in result["items"]])

    def test_discovery_read_and_command_output_bounds_report_incomplete(self) -> None:
        result = sources.search("stale", root=self.root, max_files=1)
        self.assertFalse(result["complete"])
        limited = sources.search("stale", root=self.root, max_bytes=8)
        self.assertFalse(limited["complete"])
        self.assertTrue(limited["gaps"])
        output = sources._command(
            self.root,
            [sys.executable, "-c", "print('x'*2000000)"],
            max_bytes=100,
            seconds=2,
        )
        self.assertFalse(output["complete"])
        self.assertEqual(len(output["stdout"]), 100)
        timed = sources._command(
            self.root,
            [sys.executable, "-c", "import time; time.sleep(2)"],
            seconds=0.01,
        )
        self.assertFalse(timed["complete"])

    def test_secret_environment_and_link_paths_are_excluded_before_reads(self) -> None:
        self.write("secrets/token.json", '{"secret":"do-not-project"}')
        self.write(".env.json", '{"secret":"do-not-project"}')
        (self.root / "src/link.rs").symlink_to(self.root / "secrets/token.json")
        result = sources.search("do-not-project", root=self.root)
        self.assertEqual(result["items"], [])
        with self.assertRaises(ValueError):
            sources.source("../outside.rs", root=self.root)

    def cargo(self) -> None:
        self.write(
            "Cargo.toml",
            """[workspace]\nmembers = []\n[workspace.dependencies]\nsample = { version = "=1.2.3", features = ["fast"] }\n[patch.crates-io]\nsample = { version = "=1.2.3", path = "vendor/sample" }\n""",
        )
        self.write(
            "Cargo.lock",
            """version = 4\n[[package]]\nname = "sample"\nversion = "1.2.3"\n""",
        )
        self.write(
            "vendor/sample/Cargo.toml",
            """[package]\nname = "sample"\nversion = "1.2.3"\n""",
        )
        self.write(
            "vendor/sample/src/lib.rs",
            'pub fn patched_read() -> &\'static str { "patched" }\n',
        )
        self.write(
            "patches/sample.patch",
            "--- a/src/lib.rs\n+++ b/src/lib.rs\n-pub fn registry_read() {}\n+pub fn patched_read() {}\n",
        )

    def test_interpreted_metadata_inheritance_and_policy_change_are_bound_separately(
        self,
    ) -> None:
        self.write(
            "docs/lifecycle.toml",
            """version = 1
topics = ["architecture"]
[defaults]
doc_owner = "docs/README.md"
[collections.contracts]
doc_role = "contract"
doc_topics = ["architecture"]
""",
        )
        self.write(
            "docs/contracts/inherited.md",
            "---\ntitle: Native contract\nstatus: proposed\n---\n# Native section\n",
        )
        before = sources.source("docs/contracts/inherited.md", root=self.root)
        self.assertEqual(before["role"], "contract")
        self.assertEqual(before["topics"], ["architecture"])
        self.assertEqual(before["owner"], "docs/README.md")
        self.assertEqual(
            before["metadata"]["metadata_origins"]["doc_role"], "collection:contracts"
        )
        self.assertEqual(before["scope"], "Current")
        self.assertEqual(before["native_status"], "proposed")
        policy = self.root / "docs/lifecycle.toml"
        policy.write_text(
            policy.read_text().replace(
                'doc_role = "contract"', 'doc_role = "reference"'
            )
        )
        after = sources.source("docs/contracts/inherited.md", root=self.root)
        self.assertEqual(after["role"], "reference")
        self.assertEqual(before["digest"], after["digest"])
        self.assertNotEqual(
            before["metadata_inputs"][0]["digest"],
            after["metadata_inputs"][0]["digest"],
        )

    def test_exact_production_filter_never_invents_shared_generator_correspondence(
        self,
    ) -> None:
        self.write(
            "docs/generated/source-relationships/test/relationships.json",
            json.dumps(
                {
                    "schema_version": 1,
                    "producer": "exact-test",
                    "relationships": [
                        {
                            "declaration_kind": "relation",
                            "declaration_key": "A@1",
                            "source_path": None,
                            "generator_path": "src/shared.rs",
                            "generator_symbol": "generate",
                            "output_path": "a.rs",
                        },
                        {
                            "declaration_kind": "relation",
                            "declaration_key": "B@1",
                            "source_path": None,
                            "generator_path": "src/shared.rs",
                            "generator_symbol": "generate",
                            "output_path": "b.rs",
                        },
                    ],
                }
            ),
        )
        selected = sources.relationships(
            root=self.root,
            predicates={"producer": "exact-test", "declaration_key": "A@1"},
        )
        self.assertEqual(selected["selection_basis"], "exact declared fields and paths")
        self.assertEqual([row["output_path"] for row in selected["items"]], ["a.rs"])
        self.assertIsNone(selected["items"][0]["source_path"])
        absent = sources.relationships(
            root=self.root, predicates={"declaration_key": "A@1", "output_path": "b.rs"}
        )
        self.assertEqual(absent["items"], [])
        path_selected = sources.relationships(root=self.root, exact_paths=("a.rs",))
        self.assertEqual(
            [item["declaration_key"] for item in path_selected["items"]], ["A@1"]
        )
        self.assertEqual(
            sources.relationships(root=self.root, exact_paths=("a.r",))["items"], []
        )
        self.assertIn("unverified", selected["producer_freshness"])

    def test_site_policy_change_during_scope_interpretation_refuses_mixed_cut(
        self,
    ) -> None:
        self.write("docs/selected.md", "---\nstatus: proposed\n---\n# Selected\n")
        original = sources.docs.document_scope

        def changing_scope(path, status, site):
            target = self.root / "docs/site.toml"
            target.write_text(target.read_text() + "\n# changed during selection\n")
            return original(path, status, site)

        with patch.object(sources.docs, "document_scope", side_effect=changing_scope):
            with self.assertRaisesRegex(ValueError, "metadata interpretation changed"):
                sources.source("docs/selected.md", root=self.root)

    def test_research_packet_reconstructs_exact_pin_patch_api_and_unknown_lifetime(
        self,
    ) -> None:
        self.cargo()
        packet = sources.library_packet(
            "sample",
            root=self.root,
            references=("vendor/sample/src/lib.rs", "patches/sample.patch"),
            features=("fast",),
        )
        self.assertEqual(packet["exact_version"], "1.2.3")
        self.assertEqual(packet["resolved"][0]["version"], "1.2.3")
        self.assertEqual(packet["patches"]["crates-io"]["path"], "vendor/sample")
        self.assertEqual(packet["local_sources"][0]["package_version"], "1.2.3")
        self.assertIn(
            "patched_read", packet["references"][0]["signature_leads"][0]["excerpt"]
        )
        self.assertEqual(packet["references"][0]["evidence_kind"], "primary_source")
        self.assertEqual(packet["references"][1]["evidence_kind"], "primary_patch")
        self.assertIn(
            "-pub fn registry_read()",
            packet["references"][1]["patch_change_leads"][0]["excerpt"],
        )
        self.assertIn(
            "+pub fn patched_read()",
            packet["references"][1]["patch_change_leads"][1]["excerpt"],
        )
        self.assertEqual(packet["observed_closure"]["status"], "unknown")
        self.assertIn("intent", packet["feature_basis"])
        self.assertIn("lifetimes", packet["unresolved"][0])
        self.assertEqual(
            {row["path"] for row in packet["source_cut"]}, {"Cargo.toml", "Cargo.lock"}
        )

    def test_python_project_optional_build_and_group_pins_read_uv_lock(self) -> None:
        self.write(
            "pyproject.toml",
            """[project]\nname = "fixture"\nversion = "0.1.0"\ndependencies = ["Sample_Name[fast]==2.0.1; python_version >= '3.11'"]\n[project.optional-dependencies]\nextra = ["optional-one==1.0.0"]\n[build-system]\nrequires = ["builder==3.0.0"]\n[dependency-groups]\ndev = ["tooling==4.0.0"]\n""",
        )
        self.write(
            "uv.lock",
            """version = 1\n[[package]]\nname = "sample-name"\nversion = "2.0.1"\n[[package]]\nname = "optional-one"\nversion = "1.0.0"\n[[package]]\nname = "builder"\nversion = "3.0.0"\n[[package]]\nname = "tooling"\nversion = "4.0.0"\n""",
        )
        packet = sources.library_packet("sample_name", root=self.root)
        self.assertEqual(packet["exact_version"], "2.0.1")
        self.assertEqual(packet["ecosystem"], "python")
        for name, location in (
            ("optional-one", "project.optional-dependencies.extra"),
            ("builder", "build-system.requires"),
            ("tooling", "dependency-groups.dev"),
        ):
            self.assertEqual(
                sources.library_packet(name, root=self.root)["declaration_location"],
                location,
            )

    def test_wrong_rustdoc_version_feature_target_and_unknown_format_are_unsuitable(
        self,
    ) -> None:
        self.cargo()
        context = {
            "target": "other-target",
            "features": ["other"],
            "manifest_digest": "old",
            "lock_digest": "old",
        }
        self.write(
            "docs/wrong.json",
            json.dumps(
                {
                    "rustdoc": {
                        "format_version": 999,
                        "crate_version": "9.9.9",
                        "index": {},
                        "paths": {},
                    },
                    "context": context,
                }
            ),
        )
        packet = sources.library_packet(
            "sample",
            root=self.root,
            features=("fast",),
            target="chosen-target",
            rustdoc="docs/wrong.json",
        )
        self.assertEqual(packet["rustdoc"]["status"], "unsuitable_or_unknown")
        self.assertTrue(
            {
                "target_mismatch",
                "feature_mismatch",
                "unknown_rustdoc_format",
                "crate_version_mismatch",
                "source_cut_unknown_or_mismatch",
            }.issubset(packet["rustdoc"]["issues"])
        )
        self.assertEqual(
            sources.library_packet(
                "sample", root=self.root, rustdoc="docs/absent.json"
            )["rustdoc"]["status"],
            "unknown",
        )

    def test_supplied_metadata_retains_observed_features_and_current_source_cut(
        self,
    ) -> None:
        self.cargo()
        digest = lambda path: hashlib.sha256(
            (self.root / path).read_bytes()
        ).hexdigest()
        context = {
            "target": "chosen-target",
            "features": ["fast"],
            "manifest_digest": digest("Cargo.toml"),
            "lock_digest": digest("Cargo.lock"),
        }
        context["source_digests"] = {
            "vendor/sample/src/lib.rs": digest("vendor/sample/src/lib.rs")
        }
        metadata = {
            "version": 1,
            "packages": [{"id": "sample-id", "name": "sample", "version": "1.2.3"}],
            "resolve": {
                "nodes": [{"id": "sample-id", "features": ["fast", "unified"]}]
            },
        }
        self.write(
            "docs/metadata.json",
            json.dumps({"cargo_metadata": metadata, "context": context}),
        )
        self.write(
            "docs/rustdoc.json",
            json.dumps(
                {
                    "rustdoc": {
                        "format_version": 61,
                        "crate_version": "1.2.3",
                        "root": 0,
                        "index": {"0": {"name": "sample"}},
                        "paths": {},
                    },
                    "context": context,
                }
            ),
        )
        packet = sources.library_packet(
            "sample",
            root=self.root,
            references=("vendor/sample/src/lib.rs",),
            features=("fast",),
            target="chosen-target",
            cargo_metadata="docs/metadata.json",
            rustdoc="docs/rustdoc.json",
        )
        self.assertEqual(packet["observed_closure"]["status"], "suitable")
        self.assertEqual(
            packet["observed_closure"]["observed_features"], ["fast", "unified"]
        )
        self.assertEqual(packet["rustdoc"]["status"], "suitable")
        self.assertEqual(packet["selected_features"], ["fast"])

    def test_batch_relationships_keep_declared_identity_distinct_from_current_bytes(
        self,
    ) -> None:
        generator = self.write("scripts/generator.py", "def generate(): pass\n")
        document = {
            "schema_version": 1,
            "producer": "fixture",
            "relationships": [
                {
                    "declaration_key": "Guard",
                    "source_path": None,
                    "source_identity": "registry:old",
                    "generator_path": "scripts/generator.py",
                    "output_path": "src/guard.rs",
                    "basis": "declared",
                },
                {
                    "declaration_key": "Other",
                    "source_path": "src/guard.rs",
                    "generator_path": "scripts/generator.py",
                    "output_path": "python/guard.py",
                },
            ],
        }
        self.write(
            "docs/generated/source-relationships/fixture/relationships.json",
            json.dumps(document),
        )
        with patch.object(sources, "source", wraps=sources.source) as reader:
            result = sources.relationships(("Guard", "python/guard.py"), root=self.root)
        self.assertEqual(len(result["producer_documents"]), 1)
        calls = [
            call
            for call in reader.call_args_list
            if call.args[0].endswith("relationships.json")
        ]
        self.assertEqual(len(calls), 1)
        self.assertEqual(result["items"][0]["source_identity"], "registry:old")
        self.assertEqual(result["items"][0]["source_path"], None)
        self.assertIn(
            "scripts/generator.py",
            [fact["path"] for fact in result["current_source_facts"]],
        )
        self.assertEqual(
            next(
                fact["digest"]
                for fact in result["current_source_facts"]
                if fact["path"] == "scripts/generator.py"
            ),
            hashlib.sha256(generator.read_bytes()).hexdigest(),
        )


if __name__ == "__main__":
    unittest.main()
