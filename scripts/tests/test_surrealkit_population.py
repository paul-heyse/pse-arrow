# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Named population admission, independent settlement and protected interruption."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import contextlib
import copy
import hashlib
import json
import re
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import surreal_server as server
from scripts import surrealkit as tool
from scripts import surrealkit_population as population
from scripts import test_resources as resources


class PopulationTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.state = self.root / "state"
        self.state.mkdir()
        self.config: dict[str, object] = {
            "instance_id": "fixture-generation",
            "namespace": "pse",
            "admission": "open",
            "accepting_writes": True,
        }
        self.readbacks = {query: [rows] for query, rows in population.checks()}
        self.queries: list[str] = []
        for replacement in (
            patch.object(resources, "registry", return_value=self.root / "registry"),
            patch.object(server, "config_for", return_value=self.config),
            patch.object(resources.host, "inherit", return_value=None),
            patch.dict("os.environ", {resources.MARKER: ""}),
            patch.object(
                server, "context_admission", return_value=contextlib.nullcontext()
            ),
            patch.object(resources, "namespace_sql", side_effect=self.sql),
            patch.object(tool, "inspection_sql", side_effect=self.inspection),
        ):
            replacement.start()
            self.addCleanup(replacement.stop)

    def sql(self, _config: object, query: str, **_kwargs: object) -> list[object]:
        self.queries.append(query)
        if query.startswith("DEFINE USER"):
            return [None, None]
        if query == "INFO FOR NS;":
            return [{"databases": {}}]
        if query in self.readbacks:
            return [copy.deepcopy(row) for row in self.readbacks[query]]
        if query.startswith("SELECT VALUE ordinal"):
            return [population.READINESS_ANSWER]
        if query == "SELECT * FROM canonical_projects:fixture_viewer_forbidden;":
            return [[]]
        return [None]

    @staticmethod
    def inspection(
        _config: object, _record: object, _credentials: object, query: str
    ) -> list[object]:
        return (
            [["fixture_alpha", "fixture_beta", "fixture_gamma"]]
            if query.startswith("SELECT")
            else [[]]
        )

    def test_two_fresh_fixture_identities_have_independently_settled_equal_content(
        self,
    ) -> None:
        first, second = tool.provision(self.state), tool.provision(self.state)
        left, right = tool.populate(first), tool.populate(second)
        self.assertEqual(left["content_sha256"], right["content_sha256"])
        self.assertEqual(left["rows"], right["rows"])
        self.assertNotEqual(left["namespace"], right["namespace"])
        for key in ("password", "inspection_password"):
            self.assertNotEqual(
                tool.fixture(first)[2][key], tool.fixture(second)[2][key]
            )
        for resource in (first, second):
            record = resources.resource_status(resource)
            self.assertTrue(record["drained"])
            self.assertEqual(record["tool_population"]["phase"], "settled")
        for query in self.readbacks:
            self.assertEqual(self.queries.count(query), 2)
        for hint in ("", "WITH INDEX study_ready_candidates", "WITH NOINDEX"):
            self.assertEqual(
                self.queries.count(population.READINESS.format(hint=hint) + ";"), 2
            )

    def test_exact_activity_borrow_refuses_forged_stale_and_concurrent_tokens(
        self,
    ) -> None:
        resource = tool.provision(self.state)
        with resources.borrow(resource) as token:
            original = copy.deepcopy(resources.resource_status(resource)["borrows"])
            observed, _, _ = tool.fixture(resource, _activity_borrow=token)
            self.assertEqual(observed["borrows"], original)
            for change in (
                "wrong-token",
                "missing-token",
                "pid",
                "start",
                "boot",
                "concurrent",
            ):
                with self.subTest(change=change):
                    try:
                        with resources.resource_update(resource) as record:
                            record["borrows"] = copy.deepcopy(original)
                            if change == "missing-token":
                                record["borrows"].clear()
                            elif change == "concurrent":
                                # Matching process identity cannot admit another token.
                                record["borrows"]["another-token"] = copy.deepcopy(
                                    original[token]
                                )
                            elif change == "pid":
                                record["borrows"][token]["pid"] = -1
                            elif change in {"start", "boot"}:
                                record["borrows"][token][change] = "stale-owner"
                        with (
                            patch.object(resources, "namespace_sql") as sql,
                            self.assertRaisesRegex(tool.ToolError, "borrow differs"),
                        ):
                            tool.fixture(
                                resource,
                                _activity_borrow="forged-token"
                                if change == "wrong-token"
                                else token,
                            )
                        sql.assert_not_called()
                    finally:
                        with resources.resource_update(resource) as record:
                            record["borrows"] = copy.deepcopy(original)
        self.assertFalse(resources.resource_status(resource)["borrows"])

    def test_unknown_population_refuses_before_any_activity(self) -> None:
        with (
            patch.object(tool, "activity") as activity,
            self.assertRaisesRegex(tool.ToolError, "Unknown named population"),
        ):
            tool.populate("unselected", "arbitrary.sql")
        activity.assert_not_called()

    def test_atomic_mutation_is_submitted_before_send_and_does_not_itself_settle(
        self,
    ) -> None:
        resource = tool.provision(self.state)
        original = self.sql

        def observed(config: object, query: str, **kwargs: object) -> list[object]:
            if query.startswith("BEGIN TRANSACTION"):
                record = resources.resource_status(resource)
                self.assertEqual(
                    record["tool_population"]["phase"], "population_submitted"
                )
                self.assertFalse(record["drained"])
                self.assertTrue(record["pin"])
                self.assertTrue(record["borrows"])
            return original(config, query, **kwargs)

        with patch.object(resources, "namespace_sql", side_effect=observed):
            tool.populate(resource)
        schema = (tool.ROOT / tool.GENERATED_SCHEMA).read_text()
        self.assertIn(schema, self.queries)
        self.assertLess(
            self.queries.index("DEFINE DATABASE `" + population.DATABASE + "`;"),
            self.queries.index(schema),
        )

    def test_interrupted_after_submit_stays_pinned_and_recovery_only_reads_original(
        self,
    ) -> None:
        resource = tool.provision(self.state)
        with (
            patch.object(tool, "settle_population", side_effect=KeyboardInterrupt),
            self.assertRaises(KeyboardInterrupt),
        ):
            tool.populate(resource)
        record = resources.resource_status(resource)
        self.assertFalse(record["drained"])
        self.assertTrue(record["pin"])
        self.assertFalse(record["borrows"])
        self.assertIn("creation_borrower", record)
        with self.assertRaisesRegex(tool.ToolError, "settlement"):
            tool.fixture(resource)
        self.queries.clear()
        self.assertEqual(tool.settle_population(resource)["phase"], "settled")
        self.assertTrue(all(query.startswith("SELECT") for query in self.queries))
        self.assertTrue(resources.resource_status(resource)["drained"])

    def test_wrong_rows_relation_endpoints_and_activation_never_settle(self) -> None:
        for table in (
            "canonical_projects",
            "canonical_revisions",
            "canonical_study_points",
            "canonical_analysis_edges",
            "canonical_interpretations",
        ):
            with self.subTest(table=table):
                resource = tool.provision(self.state)
                query = next(
                    query for query in self.readbacks if "FROM " + table in query
                )
                prior = self.readbacks[query]
                self.readbacks[query] = [[{"wrong": "row or endpoint"}]]
                with self.assertRaisesRegex(tool.ToolError, "settlement differs"):
                    tool.populate(resource)
                self.assertFalse(resources.resource_status(resource)["drained"])
                self.assertTrue(resources.resource_status(resource)["pin"])
                self.assertEqual(
                    resources.resource_status(resource)["tool_population"]["phase"],
                    "population_submitted",
                )
                self.readbacks[query] = prior

    def test_empty_mutation_acknowledgment_does_not_mark_drained(self) -> None:
        resource = tool.provision(self.state)
        original = self.sql
        with (
            patch.object(
                resources,
                "namespace_sql",
                side_effect=lambda config, query, **kwargs: (
                    []
                    if query.startswith("BEGIN TRANSACTION")
                    else original(config, query, **kwargs)
                ),
            ),
            self.assertRaisesRegex(tool.ToolError, "acknowledgment unavailable"),
        ):
            tool.populate(resource)
        self.assertFalse(resources.resource_status(resource)["drained"])

    def test_failed_statement_preserves_private_complete_response_and_sql_identity(
        self,
    ) -> None:
        resource = tool.provision(self.state)
        original = self.sql
        response = [
            "The query was not executed",
            "Field revision: expected string",
            "The query was not executed",
        ]
        with (
            patch.object(
                resources,
                "namespace_sql",
                side_effect=lambda config, query, **kwargs: (
                    response
                    if query.startswith("BEGIN TRANSACTION")
                    else original(config, query, **kwargs)
                ),
            ),
            self.assertRaisesRegex(tool.ToolError, "Field revision: expected string"),
        ):
            tool.populate(resource)
        record = resources.resource_status(resource)
        artifacts = list(
            Path(str(record["directory"])).glob("population-response-*.json")
        )
        self.assertEqual(len(artifacts), 1)
        self.assertEqual(artifacts[0].stat().st_mode & 0o777, 0o600)
        evidence = server.read_json(artifacts[0])
        self.assertEqual(evidence["response"], response)
        errors = evidence["errors"]
        if not isinstance(errors, list):
            self.fail("Expected original statement failure rows")
        self.assertEqual(errors[1], {"statement": 1, "error": response[1]})
        self.assertEqual(
            evidence["sql_sha256"],
            hashlib.sha256(population.sql().encode()).hexdigest(),
        )
        self.assertEqual(
            evidence["response_sha256"],
            hashlib.sha256(
                json.dumps(response, sort_keys=True, separators=(",", ":")).encode()
            ).hexdigest(),
        )
        self.assertFalse(record["drained"])
        self.assertEqual(record["tool_population"]["phase"], "population_submitted")

    def test_permission_denial_requires_privileged_absence_not_empty_mutation_output(
        self,
    ) -> None:
        resource = tool.provision(self.state)
        original = self.sql
        with (
            patch.object(
                resources,
                "namespace_sql",
                side_effect=lambda config, query, **kwargs: (
                    [[{"key": "fixture_viewer_forbidden"}]]
                    if query.startswith(
                        "SELECT * FROM canonical_projects:fixture_viewer_forbidden"
                    )
                    else original(config, query, **kwargs)
                ),
            ),
            self.assertRaisesRegex(tool.ToolError, "write denial unproved"),
        ):
            tool.populate(resource)
        self.assertFalse(resources.resource_status(resource)["drained"])

    def test_empty_viewer_read_does_not_prove_read_permission(self) -> None:
        resource = tool.provision(self.state)
        with (
            patch.object(tool, "inspection_sql", return_value=[[]]),
            self.assertRaisesRegex(tool.ToolError, "read permission unproved"),
        ):
            tool.populate(resource)

    def test_wrong_ordered_index_answer_cannot_settle(self) -> None:
        resource = tool.provision(self.state)
        original = self.sql
        with (
            patch.object(
                resources,
                "namespace_sql",
                side_effect=lambda config, query, **kwargs: (
                    [["12", "10", "6", "4"]]
                    if "WITH INDEX study_ready_candidates" in query
                    else original(config, query, **kwargs)
                ),
            ),
            self.assertRaisesRegex(tool.ToolError, "ordered answer differs"),
        ):
            tool.populate(resource)
        self.assertFalse(resources.resource_status(resource)["drained"])

    def test_recovery_refuses_actual_service_or_secret_replacement_before_sql(
        self,
    ) -> None:
        for replacement in ("service", "credentials"):
            with self.subTest(replacement=replacement):
                resource = tool.provision(self.state)
                with (
                    patch.object(
                        tool, "settle_population", side_effect=KeyboardInterrupt
                    ),
                    self.assertRaises(KeyboardInterrupt),
                ):
                    tool.populate(resource)
                if replacement == "service":
                    self.config["instance_id"] = "replacement-generation"
                else:
                    secret = (
                        Path(str(resources.resource_status(resource)["directory"]))
                        / "credentials.json"
                    )
                    secret.write_text("{}")
                before = len(self.queries)
                with self.assertRaisesRegex(tool.ToolError, "changed"):
                    tool.settle_population(resource)
                self.assertEqual(len(self.queries), before)
                self.config["instance_id"] = "fixture-generation"

    def test_recovery_refuses_changed_schema_service_generation_credentials_and_live_owner(
        self,
    ) -> None:
        for changed in ("schema_sha256", "service_generation", "credentials_sha256"):
            with self.subTest(changed=changed):
                resource = tool.provision(self.state)
                with (
                    patch.object(
                        tool, "settle_population", side_effect=KeyboardInterrupt
                    ),
                    self.assertRaises(KeyboardInterrupt),
                ):
                    tool.populate(resource)
                with resources.resource_update(resource) as record:
                    record["tool_population"][changed] = "changed"
                before = len(self.queries)
                with self.assertRaisesRegex(tool.ToolError, "identity differs"):
                    tool.settle_population(resource)
                self.assertEqual(len(self.queries), before)
        resource = tool.provision(self.state)
        with resources.resource_update(resource) as record:
            record["tool_population"] = {"phase": "population_submitted"}
            record["creation_borrower"] = {
                "pid": 999999,
                "start": "owner",
                "boot": "host",
            }
        with (
            patch.object(resources, "borrower_alive", return_value=True),
            self.assertRaisesRegex(tool.ToolError, "live borrower"),
        ):
            tool.settle_population(resource)

    def test_settled_population_cannot_drain_unrelated_unknown_tool_activity(
        self,
    ) -> None:
        resource = tool.provision(self.state)
        tool.populate(resource)
        with resources.resource_update(resource) as record:
            record["drained"] = False
        before = len(self.queries)
        with self.assertRaisesRegex(tool.ToolError, "Prior tooling activity"):
            tool.settle_population(resource)
        self.assertEqual(len(self.queries), before)
        self.assertFalse(resources.resource_status(resource)["drained"])

    def test_generated_fields_cover_every_required_field_without_production_activation(
        self,
    ) -> None:
        schema = (tool.ROOT / tool.GENERATED_SCHEMA).read_text()
        for table, records in population.rows().items():
            fields = dict(
                re.findall(
                    r"DEFINE FIELD (\w+) ON " + table + r" TYPE ([^;]+);", schema
                )
            )
            for row in records:
                self.assertLessEqual(row.keys(), fields.keys())
                required = {
                    field
                    for field, kind in fields.items()
                    if not kind.startswith("option<")
                }
                self.assertLessEqual(required, row.keys())
        self.assertNotIn("canonical_interpretations", population.rows())
        self.assertEqual(population.identity(), population.identity())
        self.assertIn("BEGIN TRANSACTION", population.sql())
        self.assertTrue(population.sql().endswith("COMMIT TRANSACTION;"))
        # Deterministic expected answers are independently derived from rows.
        ready = []
        for row in population.rows()["canonical_study_readiness"]:
            ordinal = row["ordinal"]
            if not isinstance(ordinal, int):
                self.fail("Expected original readiness ordinal")
            if row["study"] == "fixture_alpha" and row["eligible"] and ordinal >= 4:
                ready.append(ordinal)
        ready.sort()
        self.assertEqual(
            [str(value) for value in ready[:4]], population.READINESS_ANSWER
        )

    def test_declared_population_satisfies_every_selected_generated_unique_index(
        self,
    ) -> None:
        schema = (tool.ROOT / tool.GENERATED_SCHEMA).read_text()
        rows = population.rows()
        selected = []
        for index, table, columns in re.findall(
            r"DEFINE INDEX (\w+) ON (\w+) FIELDS ([\w, ]+) UNIQUE;", schema
        ):
            if table not in rows:
                continue
            selected.append(index)
            fields = [field.strip() for field in columns.split(",")]
            keys = [tuple(row[field] for field in fields) for row in rows[table]]
            self.assertEqual(len(keys), len(set(keys)), index)
        self.assertIn("revision_operation", selected)
        self.assertIn("study_point_run", selected)
        self.assertIn("study_point_occurrence", selected)


if __name__ == "__main__":
    unittest.main()
