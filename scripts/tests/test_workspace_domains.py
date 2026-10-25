# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact semantic owners, independent identities and declared production triples."""

# ruff: noqa: PT009, PT027 -- stdlib unittest controls
from __future__ import annotations

import contextlib
import io
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import active_diagnostics, workspace_context
from scripts import workspace_domains as domains
from scripts.workspace_domain_contracts import DecisionQuery, parse_query


class WorkspaceDomainTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory(prefix="pse-domain-test-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)
        self.write("docs/README.md", "# Owner\n")
        self.write("docs/authoritative_design/README.md", "# Architecture owner\n")
        self.write(
            "docs/authoritative_design/blueprint.md",
            "---\ndoc_role: contract\n---\n# Architecture index\n",
        )
        self.write(
            "docs/authoritative_design/sections/different-name.md",
            "---\ntitle: Contract body\nstatus: current\n---\n# Contract\n\n## 20.5 Accepted results\n\nThe current contract.\n\n### 20.5.1 Narrow condition\n\nA separate exact section.\n",
        )
        self.write(
            "docs/adr/9999-misleading-name.md",
            "---\nid: ADR-0101\ntitle: Proposed decision\nstatus: proposed\nsupersedes: [ADR-0004]\n---\n# ADR with a misleading filename\n",
        )
        self.write(
            "docs/adr/0101-misleading-name.md",
            "---\nid: ADR-0004\ntitle: Old decision\nstatus: superseded\nsupersedes: []\nsuperseded-by: ADR-0101\n---\n# Different original identity\n",
        )
        self.write(
            "docs/plans/coordinator.md",
            "---\nstatus: in-progress\n---\n# Coordinator\n\n## Package state\n\n| Package | Status | Owner |\n| --- | --- | --- |\n| A01 | Open | Maintainer |\n| A02 | Done | Maintainer |\n",
        )
        self.write(
            "docs/plans/support.md",
            "---\nstatus: in-progress\ndoc_owner: docs/plans/coordinator.md\n---\n# Supporting plan\n\n## Work\n\n| Package | Status | Owner |\n| --- | --- | --- |\n| A01 | Done | Supporting worker |\n",
        )
        self.write("docs/plans/unrelated.md", "# Unrelated malformed table owner\n")
        self.write(
            "docs/site.toml",
            """[publication]
exclude = []
current_work = ["plans/coordinator.md", "plans/support.md"]
entrypoints = ["README.md"]
[[collections]]
root = "."
recursive = false
title = "Entry"
scope = "Current"
[[collections]]
root = "adr"
title = "Decisions"
scope = "Current"
[[collections]]
root = "authoritative_design"
title = "Contracts"
scope = "Current"
[[collections]]
root = "plans"
title = "Plans"
scope = "History"
[[overrides]]
patterns = ["adr/9999-misleading-name.md"]
scope = "Current"
""",
        )
        self.write(
            "docs/lifecycle.toml",
            """version = 1
topics = ["design", "runtime", "plans"]
[defaults]
doc_role = "reference"
doc_owner = "docs/README.md"
[collections.adr]
doc_role = "decision"
doc_topics = ["design"]
[collections.authoritative_design]
doc_role = "contract"
doc_owner = "docs/authoritative_design/README.md"
doc_topics = ["design", "runtime"]
[collections.plans]
doc_role = "plan"
doc_topics = ["plans"]
[status_profiles.packet]
order = ["open", "complete"]
open = ["^Open$"]
complete = ["^Done$"]
[[scope_tables]]
path = "docs/plans/coordinator.md"
name = "packages"
heading = "Package state"
headers = ["Package", "Status", "Owner"]
kind = "packet"
id_column = 0
status_column = 1
owner_column = 2
status_profile = "packet"
[[scope_tables]]
path = "docs/plans/support.md"
name = "work"
heading = "Work"
headers = ["Package", "Status", "Owner"]
kind = "packet"
id_column = 0
status_column = 1
owner_column = 2
status_profile = "packet"
[[scope_tables]]
path = "docs/plans/unrelated.md"
name = "broken"
heading = "Absent"
headers = ["Package", "Status", "Owner"]
kind = "packet"
id_column = 0
status_column = 1
status_profile = "packet"
[[relationships]]
from = "docs/plans/support.md::work::A01"
to = "docs/plans/coordinator.md::packages::A01"
kind = "status-owned-by"
[[relationships]]
from = "docs/plans/coordinator.md::packages::A02"
to = "docs/plans/coordinator.md::packages::A01"
kind = "depends-on"
""",
        )
        self.write("src/generator.rs", "fn emit() {}\n")
        self.write("src/generated/alpha.rs", "pub struct Alpha;\n")
        self.write("src/generated/beta.rs", "pub struct Beta;\n")
        rows = []
        for key, output in (
            ("alpha@1", "src/generated/alpha.rs"),
            ("beta@1", "src/generated/beta.rs"),
        ):
            rows.append(
                {
                    "declaration_kind": "relation",
                    "declaration_key": key,
                    "source_path": None,
                    "source_symbol": None,
                    "source_role": "semantic_declaration_without_source_location",
                    "source_identity": "blake3:" + "1" * 64,
                    "generator_path": "src/generator.rs",
                    "generator_symbol": "emit",
                    "output_path": output,
                    "direction": None,
                    "basis": "declared",
                }
            )
        self.write(
            "docs/generated/source-relationships/fixture/relationships.json",
            json.dumps(
                {"schema_version": 1, "producer": "fixture", "relationships": rows}
            ),
        )
        subprocess.run(["git", "add", "."], cwd=self.root, check=True)

    def write(self, name: str, text: str) -> None:
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def test_authored_decision_identity_and_proposed_status_are_not_filename_or_scope(
        self,
    ) -> None:
        result = domains.query(
            parse_query({"concept": "decision", "identity": "ADR-0101"}), root=self.root
        )
        self.assertTrue(result["complete"], result["omissions"])
        self.assertEqual(len(result["items"]), 1)
        item = result["items"][0]
        self.assertEqual(item["native_state"], "proposed")
        self.assertEqual(item["publication_scope"], "Current")
        self.assertEqual(item["document_role"], "decision")
        self.assertIn(
            {"source_id": "docs/adr/9999-misleading-name.md", "role": "metadata"},
            item["sources"],
        )
        self.assertFalse(
            any(
                subject["kind"] == "document"
                and subject["native_id"].startswith("docs/adr/")
                for subject in result["subjects"]
            )
        )
        self.assertTrue(
            any(
                edge["from"] == "decision:ADR-0004"
                and edge["to"] == "decision:ADR-0101"
                and edge["relation"] == "superseded_by"
                for edge in result["edges"]
            )
        )
        self.assertEqual(
            domains.query(
                parse_query({"concept": "decision", "identity": "ADR-9999"}),
                root=self.root,
            )["items"],
            [],
        )

    def test_exact_metadata_facets_are_conjunctive_and_case_sensitive(self) -> None:
        selected = domains.query(
            parse_query(
                {
                    "concept": "decision",
                    "identity": "ADR-0101",
                    "topics": ["design"],
                    "role": "decision",
                    "owner": "docs/README.md",
                    "publication_scope": "Current",
                    "native_status": "proposed",
                }
            ),
            root=self.root,
        )
        self.assertEqual(len(selected["items"]), 1)
        for options in (
            {"topics": ["design", "runtime"]},
            {"role": "Decision"},
            {"owner": "docs/readme.md"},
            {"native_status": "accepted"},
        ):
            with self.subTest(options=options):
                self.assertEqual(
                    domains.query(
                        parse_query(
                            {"concept": "decision", "identity": "ADR-0101", **options}
                        ),
                        root=self.root,
                    )["items"],
                    [],
                )

    def test_stable_contract_owner_range_and_interpretation_inputs_are_exact(
        self,
    ) -> None:
        result = domains.query(
            parse_query({"concept": "contract-section", "identity": "20.5"}),
            root=self.root,
        )
        self.assertEqual(result["items"][0]["native_id"], "20.5")
        selected = next(
            row for row in result["section_ranges"] if row.get("section_id") == "20.5"
        )
        self.assertEqual(
            selected["source_id"],
            "docs/authoritative_design/sections/different-name.md",
        )
        text = (
            (self.root / selected["source_id"])
            .read_text()
            .splitlines()[selected["start_line"] - 1 : selected["end_line"]]
        )
        self.assertIn("The current contract.", text)
        self.assertFalse(any("20.5.1" in line for line in text))
        contributors = {source["source_id"] for source in result["items"][0]["sources"]}
        self.assertTrue(
            {
                "docs/authoritative_design/blueprint.md",
                "docs/lifecycle.toml",
                "docs/site.toml",
            }
            <= contributors
        )
        self.assertTrue(
            any(
                edge["relation"] == "contains" and edge["to"] == "contract:20.5"
                for edge in result["edges"]
            )
        )

    def test_native_package_status_owner_and_dependency_use_selected_component(
        self,
    ) -> None:
        identity = "docs/plans/support.md::work::A01"
        result = domains.query(
            parse_query({"concept": "plan-package", "identity": identity}),
            root=self.root,
        )
        self.assertTrue(result["complete"], result["omissions"])
        item = result["items"][0]
        self.assertEqual(item["native_state"], "open")
        self.assertIn("canonical owner", item["state_basis"])
        self.assertIn(
            {"source_id": "docs/plans/coordinator.md", "role": "native_scope"},
            item["sources"],
        )
        self.assertTrue(
            any(edge["relation"] == "status_owned_by" for edge in result["edges"])
        )
        direct = domains.query(
            parse_query(
                {"concept": "plan-package", "identity": "docs/plans/coordinator.md#A02"}
            ),
            root=self.root,
        )
        self.assertTrue(
            any(
                edge["relation"] == "depends_on" and edge["to"].endswith("::A01")
                for edge in direct["edges"]
            )
        )
        self.assertFalse(
            any(
                fact["path"] == "docs/plans/unrelated.md"
                for fact in direct["source_facts"]
            )
        )

    def test_shared_generator_preserves_specific_declaration_output_triples(
        self,
    ) -> None:
        result = domains.query(
            parse_query({"concept": "generator", "identity": "src/generator.rs#emit"}),
            root=self.root,
        )
        self.assertTrue(result["complete"], result["omissions"])
        self.assertEqual(len(result["items"]), 1)
        pairs = {
            (row["declaration_id"], row["artifact_id"]) for row in result["productions"]
        }
        self.assertEqual(
            pairs,
            {
                (
                    "declaration:relation:alpha@1",
                    "generated_artifact:src/generated/alpha.rs",
                ),
                (
                    "declaration:relation:beta@1",
                    "generated_artifact:src/generated/beta.rs",
                ),
            },
        )
        self.assertEqual(result["edges"], [])
        alpha = domains.query(
            parse_query(
                {"concept": "generated-output", "identity": "src/generated/alpha.rs"}
            ),
            root=self.root,
        )
        self.assertEqual(len(alpha["productions"]), 1)

        self.assertEqual(
            domains.query(
                parse_query(
                    {
                        "concept": "generated-output",
                        "identity": "src/generated/Alpha.rs",
                    }
                ),
                root=self.root,
            )["items"],
            [],
        )
        self.assertTrue(
            any("source location unavailable" in value for value in alpha["unknowns"])
        )

    def test_explicit_producer_contract_is_projected_without_shared_generator_cross_product(self) -> None:
        path = self.root / "docs/generated/source-relationships/fixture/relationships.json"
        document = json.loads(path.read_text())
        document["relationships"][0]["contract_sections"] = ["20.5"]
        path.write_text(json.dumps(document))
        result = domains.query(parse_query({"concept": "generated-output", "identity": "src/generated/alpha.rs"}), root=self.root)
        self.assertIn({"from": "contract:20.5", "to": "generated_artifact:src/generated/alpha.rs", "relation": "governs"}, [
            {key: edge[key] for key in ("from", "to", "relation")} for edge in result["edges"]
        ])
        self.assertFalse(any(edge["to"] == "generated_artifact:src/generated/beta.rs" for edge in result["edges"]))
        self.assertEqual(len(result["productions"]), 1)

    def test_original_executed_contract_evidence_links_exact_producer_and_keeps_failure(self) -> None:
        path = self.root / "docs/generated/source-relationships/fixture/relationships.json"
        producer = json.loads(path.read_text())
        producer["relationships"][0]["contract_sections"] = ["20.5"]
        path.write_text(json.dumps(producer))
        snapshot = {
            "version": 2, "authority": "assessment:/selected", "codec": "assessment-v6",
            "source_id": "/selected", "snapshot_digest": "a" * 64, "coverage": "complete",
            "facts": [{"member": "contract-execution:test:stub", "kind": "declared_link",
                       "status": "failure", "test_id": "test_stub.py::test_surface",
                       "link_relation": "executed_contract", "link_target": "20.5",
                       "detail_ref": "src/generated/alpha.rs"}],
        }
        with patch.object(domains.evidence_sources, "assessment_snapshot", return_value=snapshot):
            result = domains.query(parse_query({"concept": "evidence-reference", "selected_reference": {"adapter": "assessment", "path": "/selected"}}), root=self.root)
        self.assertTrue(any(edge["relation"] == "supports" and edge["to"] == "contract:20.5" for edge in result["edges"]))
        self.assertTrue(result["complete"], result["omissions"])
        self.assertEqual([(row["declaration_id"], row["artifact_id"]) for row in result["productions"]], [("declaration:relation:alpha@1", "generated_artifact:src/generated/alpha.rs")])
        self.assertFalse(any(row["id"] == "generated_artifact:src/generated/beta.rs" for row in result["subjects"]))
        fact = next(row["original_fact"] for row in result["native_facts"] if "original_fact" in row)
        self.assertEqual(fact["status"], "failure")
        self.assertIsNone(result["items"][0]["native_state"])

    def test_cross_kind_status_chain_captures_intermediate_documents_without_inventing_packages(
        self,
    ) -> None:
        for name, status in (("intermediate", "Open"), ("final", "Done")):
            self.write(
                "docs/plans/" + name + ".md",
                "# Native finding owner\n\n## Findings\n\n| Package | Status | Owner |\n| --- | --- | --- |\n| F01 | "
                + status
                + " | Maintainer |\n",
            )
        policy_path = self.root / "docs/lifecycle.toml"
        policy = policy_path.read_text()
        policy = policy.replace(
            'to = "docs/plans/coordinator.md::packages::A01"\nkind = "status-owned-by"',
            'to = "docs/plans/intermediate.md::findings::F01"\nkind = "status-owned-by"',
        )
        for name in ("intermediate", "final"):
            policy += (
                '\n[[scope_tables]]\npath = "docs/plans/'
                + name
                + '.md"\nname = "findings"\nheading = "Findings"\nheaders = ["Package", "Status", "Owner"]\nkind = "finding"\nid_column = 0\nstatus_column = 1\nstatus_profile = "packet"\n'
            )
        policy += '\n[[relationships]]\nfrom = "docs/plans/intermediate.md::findings::F01"\nto = "docs/plans/final.md::findings::F01"\nkind = "status-owned-by"\n'
        policy_path.write_text(policy)
        result = domains.query(
            parse_query(
                {
                    "concept": "plan-package",
                    "identity": "docs/plans/support.md::work::A01",
                }
            ),
            root=self.root,
        )
        item = result["items"][0]
        self.assertEqual(item["native_state"], "complete")
        self.assertTrue(
            {"docs/plans/intermediate.md", "docs/plans/final.md"}
            <= {source["source_id"] for source in item["sources"]}
        )
        self.assertFalse(
            any(
                subject["kind"] == "plan_package"
                and "::findings::" in subject["native_id"]
                for subject in result["subjects"]
            )
        )
        self.assertFalse(result["complete"])
        self.assertTrue(
            any(
                "endpoint kind has no typed adapter" in value
                for value in result["omissions"]
            )
        )
        original = next(
            fact for fact in result["native_facts"] if fact["subject_id"] == item["id"]
        )
        self.assertEqual(
            [row["document"] for row in original["status_owner_chain"]],
            [
                "docs/plans/support.md",
                "docs/plans/intermediate.md",
                "docs/plans/final.md",
            ],
        )

    def test_native_scope_parser_uses_captured_document_bytes(self) -> None:
        projection = domains._Projection(self.root, 256)  # noqa: SLF001 -- exercise the original owner failure/consistency boundary
        path = "docs/plans/coordinator.md"
        captured = projection.fact(path)
        self.assertIsNotNone(captured)
        self.write(
            path, (self.root / path).read_text().replace("A01 | Open", "A01 | Done")
        )
        domains._packages(projection, {path})  # noqa: SLF001 -- exercise the original owner failure/consistency boundary
        package = projection.subjects["plan_package:" + path + "::packages::A01"]
        self.assertEqual(package["native_state"], "open")
        result = projection.result()
        self.assertFalse(result["complete"])
        self.assertTrue(any("source changed" in value for value in result["omissions"]))

    def test_producer_rows_cannot_be_attached_to_changed_sidecar_bytes(self) -> None:
        owner = domains.workspace_sources.relationships

        def changed_after_owner_read(
            symbol: str | tuple[str, ...] = (),
            *,
            root: Path = domains.ROOT,
            limit: int = 40,
            predicates: dict[str, str] | None = None,
            exact_paths: tuple[str, ...] = (),
        ) -> dict:
            result = owner(
                symbol,
                root=root,
                limit=limit,
                predicates=predicates,
                exact_paths=exact_paths,
            )
            path = (
                self.root
                / "docs/generated/source-relationships/fixture/relationships.json"
            )
            path.write_text(path.read_text() + "\n")
            return result

        with patch.object(
            domains.workspace_sources,
            "relationships",
            side_effect=changed_after_owner_read,
        ):
            result = domains.query(
                parse_query(
                    {"concept": "generator", "identity": "src/generator.rs#emit"}
                ),
                root=self.root,
            )
        self.assertFalse(result["complete"])
        self.assertEqual(result["items"], [])
        self.assertTrue(
            any(
                "producer declaration changed" in value for value in result["omissions"]
            )
        )

    def test_source_only_projection_retains_full_versions_and_unknown_links(
        self,
    ) -> None:
        result = domains.project(("docs/adr/9999-misleading-name.md",), root=self.root)
        self.assertTrue(result["complete"], result["omissions"])
        self.assertEqual(
            sum(subject["kind"] == "decision" for subject in result["subjects"]), 1
        )
        self.assertFalse(
            any(
                subject["kind"] == "document"
                and subject["native_id"] == "docs/adr/9999-misleading-name.md"
                for subject in result["subjects"]
            )
        )
        decision = next(
            subject for subject in result["subjects"] if subject["kind"] == "decision"
        )
        self.assertIn(
            {"source_id": "docs/README.md", "role": "metadata"}, decision["sources"]
        )
        self.assertIn("text", result["source_facts"][0])
        missing = domains.query(
            parse_query({"concept": "contract-section", "identity": "55.99"}),
            root=self.root,
        )
        self.assertEqual(missing["items"], [])
        self.assertTrue(
            any("qualification-evidence" in value for value in missing["unknowns"])
        )

    def test_malformed_evidence_selection_is_refused_before_payload_access(
        self,
    ) -> None:
        with patch.object(
            domains.evidence_sources,
            "diagnostic_snapshot",
            side_effect=AssertionError("unexpected original payload read"),
        ):
            for selection in (
                {
                    "codec": "unrecognized-native-codec",
                    "path": "/untrusted/history.json",
                },
                {"adapter": "unrecognized", "path": "/untrusted/history.json"},
                {"adapter": "diagnostic", "path": None},
                {"adapter": "diagnostic", "path": ""},
            ):
                with self.subTest(selection=selection), self.assertRaises(ValueError):
                    domains.query(
                        parse_query(
                            {
                                "concept": "evidence-reference",
                                "selected_reference": selection,
                            }
                        ),
                        root=self.root,
                    )

    def test_original_diagnostic_action_keeps_codec_authority_and_native_member_without_lifecycle(
        self,
    ) -> None:
        action = self.root / "diagnostic-action"
        action.mkdir(mode=0o700)
        active_diagnostics.publish(
            action,
            {
                "schema": active_diagnostics.SCHEMA,
                "action": "control",
                "status": "tool_failure",
                "target_outcome": "unknown",
                "invocation": None,
            },
        )
        selection = {"adapter": "diagnostic", "path": str(action)}
        snapshot = domains.evidence_sources.diagnostic_snapshot(action)
        result = domains.query(
            parse_query({"concept": "diagnostic", "selected_reference": selection}),
            root=self.root,
        )
        self.assertEqual(len(result["items"]), 1)
        self.assertTrue(result["complete"], result["omissions"])
        diagnostic = result["items"][0]
        self.assertEqual(diagnostic["owner"], snapshot["authority"])
        self.assertEqual(diagnostic["native_version"], snapshot["snapshot_digest"])
        self.assertIsNone(diagnostic["native_state"])
        self.assertIsNone(diagnostic["state_basis"])
        reference = domains.query(
            parse_query(
                {
                    "concept": "evidence-reference",
                    "selected_reference": {**selection, "member": "action"},
                }
            ),
            root=self.root,
        )["items"][0]
        self.assertEqual(
            reference["evidence"],
            {
                "authority": snapshot["authority"],
                "codec": snapshot["codec"],
                "source_id": snapshot["source_id"],
                "version": snapshot["snapshot_digest"],
                "member": "action",
            },
        )
        self.assertEqual(
            domains.query(
                parse_query(
                    {
                        "concept": "diagnostic",
                        "selected_reference": {**selection, "member": "target"},
                    }
                ),
                root=self.root,
            )["items"],
            [],
        )
        self.assertEqual(
            domains.query(
                parse_query(
                    {
                        "concept": "evidence-reference",
                        "selected_reference": {**selection, "codec": "other"},
                    }
                ),
                root=self.root,
            )["items"],
            [],
        )

    def test_question_identity_origin_and_source_version_are_explicit(self) -> None:
        with self.assertRaises(ValueError):
            domains.query(
                parse_query(
                    {
                        "concept": "research-question",
                        "identity": "RQ-7",
                        "question": "Which layout?",
                    }
                ),
                root=self.root,
            )
        result = domains.query(
            parse_query(
                {
                    "concept": "research-question",
                    "identity": "RQ-7",
                    "question": "Which layout?",
                    "origin": "user-question",
                }
            ),
            root=self.root,
        )
        item = result["items"][0]
        self.assertEqual(item["basis"], "user-question")
        self.assertIsNone(item["native_state"])
        self.assertIsNone(item["publication_scope"])
        self.assertEqual(item["sources"][0]["role"], "question_input")
        self.assertTrue(result["complete"], result["omissions"])

    def test_independently_authored_requests_execute_every_concept_branch(self) -> None:
        action = self.root / "selected-diagnostic"
        action.mkdir(mode=0o700)
        active_diagnostics.publish(
            action,
            {
                "schema": active_diagnostics.SCHEMA,
                "action": "control",
                "status": "tool_failure",
                "target_outcome": "unknown",
                "invocation": None,
            },
        )
        reference = {"adapter": "diagnostic", "path": str(action)}
        cases = (
            (
                {"concept": "document", "identity": "docs/README.md"},
                "document",
                "available",
            ),
            ({"concept": "decision", "identity": "ADR-0101"}, "decision", "available"),
            (
                {"concept": "contract-section", "identity": "20.5"},
                "contract",
                "available",
            ),
            (
                {
                    "concept": "plan-package",
                    "identity": "docs/plans/coordinator.md#A01",
                },
                "plan_package",
                "available",
            ),
            (
                {"concept": "declaration", "identity": "relation:alpha@1"},
                "declaration",
                "available",
            ),
            (
                {"concept": "generator", "identity": "src/generator.rs#emit"},
                "generator",
                "available",
            ),
            (
                {"concept": "generated-output", "identity": "src/generated/alpha.rs"},
                "generated_artifact",
                "available",
            ),
            (
                {"concept": "diagnostic", "selected_reference": reference},
                "diagnostic",
                "available",
            ),
            (
                {
                    "concept": "evidence-reference",
                    "selected_reference": {**reference, "member": "action"},
                },
                "evidence_reference",
                "available",
            ),
            (
                {"concept": "experiment", "selected_reference": reference},
                "experiment",
                "not_implemented",
            ),
            (
                {
                    "concept": "research-question",
                    "identity": "RQ-2",
                    "origin": "agent-interpretation",
                    "question": "Which package is open?",
                },
                "research_question",
                "available",
            ),
        )
        for supplied, kind, status in cases:
            with self.subTest(concept=supplied["concept"]):
                result = domains.query(parse_query(supplied), root=self.root)
                self.assertEqual(result["adapter_status"], status)
                self.assertEqual(result["query"]["concept"], supplied["concept"])
                if status == "available":
                    self.assertEqual(len(result["items"]), 1)
                    self.assertEqual(result["items"][0]["kind"], kind)
                    self.assertTrue(result["items"][0]["sources"])
                    self.assertTrue(result["source_facts"])
                else:
                    self.assertEqual(result["items"], [])
                    self.assertIn(
                        "experiment concept adapter not implemented",
                        " ".join(result["unknowns"]),
                    )

    def test_required_and_optional_request_fields_have_distinct_meanings(self) -> None:
        for concept in (
            "contract-section",
            "plan-package",
            "declaration",
            "generator",
            "generated-output",
        ):
            for fields in (
                {},
                {"identity": None},
                {"identity": ""},
                {"identity": "  "},
            ):
                with (
                    self.subTest(concept=concept, fields=fields),
                    self.assertRaises(ValueError),
                ):
                    parse_query({"concept": concept, **fields})
        for concept in ("diagnostic", "evidence-reference", "experiment"):
            for fields in (
                {},
                {"selected_reference": None},
                {"selected_reference": {}},
            ):
                with (
                    self.subTest(concept=concept, fields=fields),
                    self.assertRaises(ValueError),
                ):
                    parse_query({"concept": concept, **fields})
        for fields in (
            {"identity": None},
            {"origin": None},
            {"origin": "inferred"},
            {"question": None},
            {"question": ""},
            {"question": "  "},
        ):
            with self.subTest(fields=fields), self.assertRaises(ValueError):
                parse_query(
                    {
                        "concept": "research-question",
                        "identity": "Q1",
                        "origin": "user-question",
                        "question": "Which owner?",
                        **fields,
                    }
                )
        absent = domains.query(parse_query({"concept": "decision"}), root=self.root)
        explicit_null = domains.query(
            parse_query({"concept": "decision", "identity": None}), root=self.root
        )
        self.assertEqual(absent["items"], explicit_null["items"])
        self.assertEqual(len(absent["items"]), 2)
        with self.assertRaises(ValueError):
            parse_query({"concept": "decision", "identity": ""})
        for fields in (
            {"topics": None},
            {"topics": [""]},
            {"topics": ["x"] * 17},
            {"role": ""},
            {"owner": ""},
            {"document": ""},
            {"publication_scope": "current"},
            {"native_status": ""},
            {"limit": 0},
            {"limit": True},
            {"offset": -1},
            {"origin": "user-question"},
            {"semantic_query": "owner"},
        ):
            with self.subTest(fields=fields), self.assertRaises(ValueError):
                parse_query({"concept": "decision", **fields})
        empty_topics = parse_query({"concept": "decision", "topics": []})
        if not isinstance(empty_topics, DecisionQuery):
            self.fail("Expected the decision request alternative")
        self.assertEqual(empty_topics.topics, [])
        with self.assertRaises(ValueError):
            parse_query(
                {
                    "concept": "diagnostic",
                    "selected_reference": {"adapter": "assessment", "path": "owned"},
                }
            )
        with self.assertRaises(ValueError):
            parse_query(
                {
                    "concept": "research-question",
                    "identity": "Q1",
                    "origin": "user-question",
                    "question": "Why?",
                    "topics": [],
                }
            )
        with self.assertRaises(ValueError):
            parse_query({"concept": "not-declared"})

    def test_reference_and_question_cli_construct_owner_requests(self) -> None:
        reference = self.root / "reference.json"
        reference.write_text(
            json.dumps({"adapter": "diagnostic", "path": "not-read-for-experiment"})
        )
        for args, expected in (
            (
                [
                    "--concept",
                    "research-question",
                    "--identity",
                    "RQ-cli",
                    "--origin",
                    "user-question",
                    "Which contract?",
                ],
                "available",
            ),
            (
                ["--concept", "experiment", "--native-reference", str(reference)],
                "not_implemented",
            ),
        ):
            with (
                self.subTest(args=args),
                patch.object(workspace_context, "ROOT", self.root),
                contextlib.redirect_stdout(io.StringIO()) as output,
            ):
                self.assertEqual(workspace_context.main([*args, "--json"]), 0)
                self.assertEqual(
                    json.loads(output.getvalue())["adapter_status"], expected
                )

    def test_query_pagination_retains_global_population_and_only_selected_context(
        self,
    ) -> None:
        result = domains.query(
            parse_query({"concept": "decision", "limit": 1}), root=self.root
        )
        self.assertEqual(
            result["population"],
            {
                "matched_count": 2,
                "returned_count": 1,
                "offset": 0,
                "limit": 1,
                "complete": True,
            },
        )
        self.assertEqual(result["next_offset"], 1)
        # Both decisions are genuinely related by authored supersession.
        self.assertEqual(
            {
                row["native_id"]
                for row in result["subjects"]
                if row["kind"] == "decision"
            },
            {"ADR-0004", "ADR-0101"},
        )
        last = domains.query(
            parse_query({"concept": "decision", "offset": 2, "limit": 1}),
            root=self.root,
        )
        self.assertEqual(last["population"]["matched_count"], 2)
        self.assertEqual(last["population"]["returned_count"], 0)
        for field in (
            "items",
            "subjects",
            "edges",
            "productions",
            "section_ranges",
            "native_facts",
            "source_facts",
        ):
            self.assertEqual(last[field], [], field)
        self.assertIsNone(last["next_offset"])
        declaration = domains.query(
            parse_query({"concept": "declaration", "identity": "relation:alpha@1"}),
            root=self.root,
        )
        self.assertEqual(len(declaration["productions"]), 1)
        self.assertEqual(
            {row["native_id"] for row in declaration["subjects"]},
            {
                "relation:alpha@1",
                "src/generator.rs#emit",
                "src/generated/alpha.rs",
            },
        )
        self.assertFalse(any("text" in fact for fact in declaration["source_facts"]))
        self.assertTrue(
            all(
                row.get("producer_row", {}).get("declaration_key") != "beta@1"
                for row in declaration["native_facts"]
            )
        )

    def test_subject_bound_is_explicit_incomplete(self) -> None:
        result = domains.project(
            ("docs/authoritative_design/sections/different-name.md",),
            root=self.root,
            limit=1,
        )
        self.assertFalse(result["complete"])
        self.assertTrue(any("bound exceeded" in value for value in result["omissions"]))

    def test_exact_cli_does_not_run_recipe_lexical_database_or_lsp_discovery(
        self,
    ) -> None:
        with (
            patch.object(workspace_context, "ROOT", self.root),
            patch.object(
                workspace_context.subprocess,
                "check_output",
                side_effect=AssertionError("recipe catalogue"),
            ),
            patch.object(
                workspace_context.workspace_sources,
                "search",
                side_effect=AssertionError("lexical fallback"),
            ),
            contextlib.redirect_stdout(io.StringIO()) as output,
        ):
            self.assertEqual(
                workspace_context.main(
                    [
                        "--concept",
                        "decision",
                        "--identity",
                        "ADR-0101",
                        "--topic",
                        "design",
                        "--json",
                    ]
                ),
                0,
            )
        result = json.loads(output.getvalue())
        self.assertEqual(result["items"][0]["native_state"], "proposed")
        self.assertFalse(any("text" in source for source in result["source_facts"]))


if __name__ == "__main__":
    unittest.main()
