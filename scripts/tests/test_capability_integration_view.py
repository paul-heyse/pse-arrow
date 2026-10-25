# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only integration uses explicit current-byte ownership and original owners."""
# ruff: noqa: PT009, PT027 -- standard-library isolated Git fixtures

from __future__ import annotations

import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from fastmcp import Client, FastMCP

from scripts import validation_result
from scripts.capabilities import integration_view as view


class IntegrationViewTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.git("init", "-q")
        self.write(
            "crates/selected/Cargo.toml",
            '[package]\nname="selected"\nversion="0.0.1"\n',
        )
        self.write(
            "crates/selected/src/lib.rs",
            "".join(f"original line {index}\n" for index in range(1, 13)),
        )
        self.write("scripts/tests/test_selected.py", "# original test\n")
        self.write("scripts/unrelated.py", "# other work\n")
        self.write("src/generator.rs", "// generator declaration\n")
        self.write("python/pse/contracts/selected.py", "# generated original\n")
        self.write(
            "docs/generated/source-relationships/fixture/relationships.json",
            json.dumps(
                {
                    "schema_version": 1,
                    "producer": "fixture-producer",
                    "relationships": [
                        {
                            "source_path": "crates/selected/src/lib.rs",
                            "generator_path": "src/generator.rs",
                            "output_path": "python/pse/contracts/selected.py",
                            "basis": "declared",
                        }
                    ],
                }
            ),
        )
        self.git("add", ".")
        self.git(
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "-qm",
            "fixture",
            "--no-gpg-sign",
        )

    def git(self, *arguments: str) -> str:
        return subprocess.check_output(["git", *arguments], cwd=self.root, text=True)

    def write(self, path: str, body: str) -> None:
        selected = self.root / path
        selected.parent.mkdir(parents=True, exist_ok=True)
        selected.write_text(body)

    def claim(
        self, path: str, owner: str = "task", *, hunks: list[view.Lines] | None = None
    ) -> view.OwnershipClaim:
        selected = self.root / path
        return view.OwnershipClaim(
            path=path,
            owner=owner,
            source_sha256=hashlib.sha256(selected.read_bytes()).hexdigest()
            if selected.exists()
            else None,
            hunks=hunks or [],
            explanation="Explicit assignment for " + owner,
        )

    def result(
        self, *claims: view.OwnershipClaim, **selection: object
    ) -> view.IntegrationPayload:
        request = view.IntegrationRequest.model_validate(
            {"task": "task", "claims": list(claims), **selection}
        )
        result = view.inspect(self.root, request)
        assert isinstance(result, view.IntegrationResult)
        return result.payload

    def edit_lines(self) -> str:
        path = "crates/selected/src/lib.rs"
        lines = (self.root / path).read_text().splitlines(keepends=True)
        lines[1] = "changed task line\n"
        lines[7] = "changed concurrent line\n"
        self.write(path, "".join(lines))
        return path

    def test_dirty_git_without_explicit_claim_is_unknown_and_excluded(self) -> None:
        path = self.edit_lines()
        result = self.result()
        self.assertEqual(result.changes[0].path, path)
        self.assertEqual(result.changes[0].classification, "unknown")
        self.assertEqual(result.commit_selection[0].selection, "excluded")
        self.assertEqual(result.test_scope.changed_owned_paths, [])

    def test_original_assessment_context_never_becomes_automatic_acceptance(
        self,
    ) -> None:
        path = self.edit_lines()
        source = self.root / path
        # Independent original-source encoding oracle, including executable mode.
        stamp = hashlib.sha256(
            str(source.stat().st_mode & 0o777).encode() + b":" + source.read_bytes()
        ).hexdigest()
        record = {
            "gate": "selected",
            "profile": "unit",
            "mode": "force-validate",
            "status": "passed",
            "evidence_kind": "executed",
            "inputs": {
                "files": {path: stamp},
                "environment": {"RUSTFLAGS": "source-owner-value"},
            },
            "invocation": {"args": ["--features", "force-validate"]},
            "selected": ["original-case"],
        }
        reference = {"kind": "assessment", "directory": "retained", "gate": "selected"}
        with patch.object(
            validation_result,
            "read_assessment",
            return_value={
                "checks": [{"record": record}],
                "execution_explanation": None,
            },
        ) as reader:
            result = self.result(
                self.claim(path),
                validation_evidence=[reference],
                features="force-validate",
                validation_profile="unit",
                validation_mode="force-validate",
            )
        evidence = result.test_scope.validation_evidence[0].model_dump()
        self.assertEqual(evidence["applicability"], "unknown")
        self.assertEqual(evidence["source_mismatches"], [])
        self.assertEqual(evidence["original_contexts"][0]["status"], "passed")
        self.assertIn("no automatic reuse", evidence["qualification"])
        self.assertNotIn("source-owner-value", json.dumps(evidence))
        reader.assert_called_once_with(
            self.root / "retained",
            gate="selected",
            source_detail=True,
            full_export=False,
        )

        for key, changed in (
            ("features", "native-solvers"),
            ("validation_profile", "component"),
            ("validation_mode", "ordinary"),
        ):
            with (
                self.subTest(key=key),
                patch.object(
                    validation_result,
                    "read_assessment",
                    return_value={"checks": [{"record": record}]},
                ),
            ):
                request = {
                    "validation_evidence": [reference],
                    "features": "force-validate",
                    "validation_profile": "unit",
                    "validation_mode": "force-validate",
                    key: changed,
                }
                stale = (
                    self.result(self.claim(path), **request)
                    .test_scope.validation_evidence[0]
                    .model_dump()
                )
                self.assertEqual(stale["applicability"], "inapplicable")
        self.write(path, "source changed after original assessment\n")
        with patch.object(
            validation_result,
            "read_assessment",
            return_value={"checks": [{"record": record}]},
        ):
            stale = (
                self.result(self.claim(path), validation_evidence=[reference])
                .test_scope.validation_evidence[0]
                .model_dump()
            )
        self.assertEqual(stale["source_mismatches"], [path])
        self.assertEqual(stale["applicability"], "inapplicable")

    def test_assessment_summary_does_not_hydrate_unselected_sources(self) -> None:
        path = self.edit_lines()
        with patch.object(
            validation_result,
            "read_assessment",
            return_value={"checks": [], "execution_explanation": None},
        ) as reader:
            result = self.result(
                self.claim(path),
                validation_evidence=[{"kind": "assessment", "directory": "retained"}],
            )
        reader.assert_called_once_with(
            self.root / "retained", gate=None, source_detail=False, full_export=False
        )
        evidence = result.test_scope.validation_evidence[0].model_dump()
        self.assertEqual(evidence["applicability"], "unknown")
        self.assertIn("Select an exact original assessment gate", evidence["reason"])

    def test_disjoint_current_line_ownership_proposes_only_owned_hunks(self) -> None:
        path = self.edit_lines()
        result = self.result(
            self.claim(path, hunks=[view.Lines(start_line=2, end_line=2)]),
            self.claim(
                path, "other-task", hunks=[view.Lines(start_line=8, end_line=8)]
            ),
        )
        change = result.changes[0]
        self.assertEqual(change.classification, "mixed")
        self.assertEqual(
            [item.classification for item in change.ranges], ["owned", "unrelated"]
        )
        proposal = result.commit_selection[0]
        self.assertEqual(proposal.selection, "selected-hunks")
        self.assertEqual(proposal.hunks, [view.Lines(start_line=2, end_line=2)])
        self.assertEqual(result.test_scope.package_seeds, ["selected"])
        self.assertIn(["just", "unit-package", "selected"], result.test_scope.routes)
        proposal = next(
            route
            for route in result.test_scope.routes
            if route[:3] == ["just", "affected", "--json"]
        )
        self.assertIn("--passive", proposal)
        self.assertIn("--changes-json", proposal)
        self.assertEqual([row[1] for row in json.loads(proposal[-1])], [path])

    def test_overlapping_explicit_owners_and_stale_witness_remain_excluded(
        self,
    ) -> None:
        path = self.edit_lines()
        own = self.claim(path)
        other = self.claim(
            path, "other-task", hunks=[view.Lines(start_line=2, end_line=2)]
        )
        result = self.result(own, other)
        self.assertEqual(result.changes[0].classification, "unknown")
        self.assertIn("overlapping owners", " ".join(result.changes[0].conflicts))
        self.assertEqual(result.changes[0].claims, [own, other])
        self.assertEqual(result.commit_selection[0].selection, "excluded")
        self.write(path, "changed again\n")
        stale = self.result(own)
        self.assertEqual(stale.changes[0].classification, "unknown")
        self.assertIn("witness differs", stale.changes[0].conflicts[0])

    def test_generated_consumer_and_unrelated_dirty_file_preserve_owner_routes(
        self,
    ) -> None:
        source = self.edit_lines()
        generated = "python/pse/contracts/selected.py"
        unrelated = "scripts/unrelated.py"
        self.write(generated, "# generated changed\n")
        self.write(unrelated, "# concurrent changed\n")
        before = self.git("status", "--porcelain=v1")
        index = (self.root / ".git/index").read_bytes()
        with (
            patch.object(
                view.workspace_tasks,
                "task_packet",
                side_effect=AssertionError("no native selection"),
            ),
            patch.object(
                view.document_lifecycle,
                "snapshot",
                side_effect=AssertionError("no retirement selection"),
            ),
        ):
            result = self.result(
                self.claim(source),
                self.claim(generated),
                self.claim(unrelated, "other-task"),
            )
        changed = {item.path: item for item in result.changes}
        selections = {item.path: item for item in result.commit_selection}
        self.assertTrue(changed[generated].generated)
        self.assertEqual(changed[unrelated].classification, "unrelated")
        self.assertEqual(selections[source].selection, "whole-path")
        self.assertEqual(selections[generated].selection, "excluded")
        self.assertEqual(selections[unrelated].selection, "excluded")
        self.assertEqual(result.changed_consumers, [generated])
        self.assertEqual(
            result.regeneration_routes, [["just", "codegen"], ["just", "codegen-check"]]
        )
        assert result.relationships is not None
        self.assertEqual(
            result.relationships.items[0].model_dump()["generator_path"],
            "src/generator.rs",
        )
        self.assertIn("unverified", result.relationships.producer_freshness)
        self.assertEqual(self.git("status", "--porcelain=v1"), before)
        self.assertEqual((self.root / ".git/index").read_bytes(), index)

    def test_deletion_requires_absence_and_matching_diff_witness(self) -> None:
        path = "scripts/unrelated.py"
        (self.root / path).unlink()
        unbound = self.result(self.claim(path))
        change = unbound.changes[0]
        self.assertEqual(change.classification, "unknown")
        bound = self.claim(path).model_copy(update={"diff_sha256": change.diff_sha256})
        result = self.result(bound)
        self.assertEqual(result.changes[0].classification, "owned")
        self.assertEqual(result.commit_selection[0].selection, "whole-path")

    def test_unresolved_git_index_conflict_cannot_enter_proposed_commit(self) -> None:
        path = self.edit_lines()
        mode, digest, _stage = (
            self.git("ls-files", "--stage", "--", path).split("\t")[0].split()
        )
        rows = f"0 {'0' * 40}\t{path}\n" + "".join(
            f"{mode} {digest} {stage}\t{path}\n" for stage in (1, 2, 3)
        )
        subprocess.run(
            ["git", "update-index", "--index-info"],
            cwd=self.root,
            input=rows,
            text=True,
            check=True,
        )
        result = self.result(self.claim(path))
        self.assertEqual(result.changes[0].classification, "unknown")
        self.assertIn(
            "unresolved index conflict", " ".join(result.changes[0].conflicts)
        )
        self.assertEqual(result.commit_selection[0].selection, "excluded")

    def test_page_bounds_and_actual_source_change_during_composition_are_explicit(
        self,
    ) -> None:
        path = self.edit_lines()
        self.write("scripts/unrelated.py", "# changed\n")
        result = view.inspect(self.root, view.IntegrationRequest(task="task", limit=1))
        assert isinstance(result, view.IntegrationResult)
        self.assertFalse(result.complete)
        self.assertEqual(result.payload.total_changes, 2)
        self.assertEqual(result.payload.next_offset, 1)
        original = view.workspace_sources.relationships

        def mutate(
            symbol: str | tuple[str, ...] = (),
            *,
            root: Path = view.workspace_sources.ROOT,
            limit: int = 40,
            predicates: dict[str, str] | None = None,
            exact_paths: tuple[str, ...] = (),
        ) -> dict:
            self.write(path, "concurrent replacement with same Git status\n")
            return original(
                symbol,
                root=root,
                limit=limit,
                predicates=predicates,
                exact_paths=exact_paths,
            )

        claim = self.claim(path)
        with (
            patch.object(view.workspace_sources, "relationships", side_effect=mutate),
            self.assertRaisesRegex(ValueError, "source changed"),
        ):
            self.result(claim)

    def test_explicit_retirement_delegates_preflight_and_preserves_release_blocker(
        self,
    ) -> None:
        self.write("docs/plans/selected.md", "# Selected plan\n")
        snapshot = {
            "candidates": [
                {
                    "path": "docs/plans/selected.md",
                    "references_to_repair": ["docs/plans/other.md"],
                    "blockers": ["owner release not established"],
                    "native_status": "done",
                }
            ],
            "errors": [],
        }
        with (
            patch.object(view.document_metadata, "load_policy", return_value={}),
            patch.object(
                view.document_lifecycle, "snapshot", return_value=snapshot
            ) as preflight,
        ):
            result = self.result(retiring_paths=["docs/plans/selected.md"])
        preflight.assert_called_once_with(self.root, "retire-plan", {})
        row = result.retirement_preflight[0].model_dump()
        self.assertEqual(row["references_to_repair"], ["docs/plans/other.md"])
        self.assertEqual(row["blockers"], ["owner release not established"])

    def test_selected_native_task_and_outcome_references_keep_original_omissions(
        self,
    ) -> None:
        packet = view.workspace_tasks.task_packet(None, (), root=self.root)
        packet.update(
            task="docs/plans/selected.md#G04",
            population_complete=False,
            omissions=["native owner reports unavailable scope"],
        )
        with patch.object(
            view.workspace_tasks, "task_packet", return_value=packet
        ) as native:
            result = view.inspect(
                self.root,
                view.IntegrationRequest(
                    task="task",
                    native_task="docs/plans/selected.md#G04",
                    outcomes=["selected-evidence-reference"],
                ),
            )
        assert isinstance(result, view.IntegrationResult)
        self.assertFalse(result.complete)
        assert result.payload.native_task is not None
        self.assertEqual(
            result.payload.native_task.omissions,
            ["native owner reports unavailable scope"],
        )
        self.assertEqual(
            result.payload.outcome_references, ["selected-evidence-reference"]
        )
        native.assert_called_once_with(
            "docs/plans/selected.md#G04",
            (),
            root=self.root,
            evidence=("selected-evidence-reference",),
        )

    def test_oversize_answer_refuses_without_mutating_its_typed_fields(self) -> None:
        path = self.edit_lines()
        with patch.object(view.read_contracts, "MAX_ANSWER_BYTES", 1000):
            result = view.inspect(
                self.root,
                view.IntegrationRequest(task="task", claims=[self.claim(path)]),
            )
        self.assertIsInstance(result, view.read_contracts.DetailRefused)
        assert isinstance(result, view.read_contracts.DetailRefused)
        self.assertEqual(result.operation, "workspace_integration_view")
        self.assertGreater(result.encoded_bytes, 1000)
        self.assertNotIn("payload", result.model_dump())


class IntegrationBindingTests(unittest.IsolatedAsyncioTestCase):
    async def test_actual_mcp_has_typed_view_readonly_annotations_and_safe_refusal(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            specs, tools = view.bindings(root)
            server = FastMCP("integration-fixture", strict_input_validation=True)
            server.add_tool(tools["workspace_integration_view"])
            with patch.object(
                view, "inspect", side_effect=ValueError("private source input")
            ):
                async with Client(server) as client:
                    declared = (await client.list_tools())[0]
                    assert declared.annotations is not None
                    self.assertTrue(declared.annotations.read_only_hint)
                    assert declared.output_schema is not None
                    self.assertIn("source_sha256", json.dumps(declared.output_schema))
                    self.assertIn(
                        "retirement_preflight", json.dumps(declared.output_schema)
                    )
                    response = await client.call_tool(
                        "workspace_integration_view",
                        {"request": {"task": "task"}},
                        raise_on_error=False,
                    )
                    self.assertTrue(response.is_error)
                    self.assertNotIn("private source input", str(response))
            self.assertEqual(specs[0].worker_calls, ())

    async def test_successful_mcp_view_returns_the_advertised_typed_result(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _, tools = view.bindings(root)
            server = FastMCP("integration-success", strict_input_validation=True)
            server.add_tool(tools["workspace_integration_view"])

            def git(_root: Path, *arguments: str, **_options: object) -> bytes:
                return b"a" * 40 if arguments[0] == "rev-parse" else b""

            with patch.object(view, "_git", side_effect=git):
                async with Client(server) as client:
                    response = await client.call_tool(
                        "workspace_integration_view", {"request": {"task": "task"}}
                    )
                    document = response.structured_content
                    assert isinstance(document, dict)
                    result = view.IntegrationResult.model_validate(document["result"])
                    self.assertEqual(result.payload.changes, [])
                    self.assertTrue(result.complete)
                    self.assertIn("no staging", result.payload.effects)


if __name__ == "__main__":
    unittest.main()
