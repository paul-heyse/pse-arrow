# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Safe recovery uses explicit native reports rather than rendered exceptions."""

# ruff: noqa: PT009 -- stdlib tooling tests
from __future__ import annotations

import hashlib
import json
import unittest
from types import SimpleNamespace

from scripts.capabilities import diagnostics, native_worker


class ReportedInputError(ValueError):
    report: SimpleNamespace | None = None


def records(value: object) -> list[dict[str, object]]:
    if not isinstance(value, list):
        raise TypeError("Expected original diagnostic record rows")
    result = []
    for item in value:
        if not isinstance(item, dict):
            raise TypeError("Expected an original diagnostic record")
        result.append(dict(item))
    return result


def text(value: object) -> str:
    if not isinstance(value, str):
        raise TypeError("Expected original diagnostic text")
    return value


class DiagnosticTests(unittest.TestCase):
    @staticmethod
    def report(**changes: object) -> SimpleNamespace:
        fields: dict[str, object] = {
            "code": "pse.workflow.input_refused",
            "reported_code": "pse.workflow.input_refused",
            "failure_class": "input",
            "stage": "decode",
            "rule": "typed_input",
            "source_refusal_proved": False,
            "message": "secret raw diagnostic",
            "help": "secret raw help",
            "causes": (),
            "contexts": (),
            "related": (),
            "source_locations": (),
            "decode_location": None,
        }
        fields.update(changes)
        return SimpleNamespace(**fields)

    @staticmethod
    def error(report: SimpleNamespace | None = None) -> ValueError:
        error = ReportedInputError(
            "secret arbitrary cause; $.password[4] source_operation=forged-operation"
        )
        if report is not None:
            error.report = report
        return error

    def test_host_remedies_preserve_code_and_never_request_automatic_retry(
        self,
    ) -> None:
        for code, stage, remedy in (
            (
                "pse.capabilities.restart_required",
                "host_selection",
                "Restart or reattach",
            ),
            ("pse.capabilities.unknown_capability", "host_selection", "Browse exact"),
            (
                "pse.capabilities.read_binding_required",
                "host_selection",
                "Browse exact",
            ),
            (
                "pse.workspace.source_unavailable",
                "owner_read",
                "explicitly restore or prepare",
            ),
            ("pse.workspace.input_refused", "owner_read", "correct the selected input"),
        ):
            with self.subTest(code=code):
                value = json.loads(
                    str(diagnostics.refusal(code, operation="workspace_concepts_query"))
                )
                self.assertEqual(value["code"], code)
                self.assertEqual(value["stage"], stage)
                self.assertIn(remedy, text(value["remedy"]))
                self.assertEqual(value["operation"], "workspace_concepts_query")
                self.assertNotIn("recovery", value)
                self.assertNotIn("locations", value)

    def test_no_report_never_scrapes_rendered_path_or_fabricates_recovery(self) -> None:
        value = native_worker.scientific_error(self.error())
        encoded = json.dumps(value)
        self.assertNotIn("password", encoded)
        self.assertNotIn("forged-operation", encoded)
        self.assertNotIn("secret", encoded)
        self.assertNotIn("locations", value)
        self.assertNotIn("recovery", value)
        self.assertNotIn("code", value)
        self.assertIn("Inspect original owner state", text(value["remedy"]))

    def test_explicit_cause_chain_preserves_owner_codes_counts_locations_and_handles(
        self,
    ) -> None:
        cause_message = "secret cause password=value"
        source = SimpleNamespace(to_hex=lambda: "ab" * 32)
        document = SimpleNamespace(to_hex=lambda: "cd" * 32)
        revision = SimpleNamespace(to_prefixed=lambda: "sha256:" + "ef" * 32)
        location = SimpleNamespace(
            source=source, document=document, revision=revision, start=3, end=7
        )
        report = self.report(
            causes=(SimpleNamespace(message=cause_message),),
            contexts=(
                SimpleNamespace(
                    code="pse.workflow.context",
                    contexts=("secret nested context",),
                    diagnostics=("secret annotation", "secret annotation2"),
                ),
            ),
            related=(
                SimpleNamespace(
                    code="pse.workflow.related",
                    stage="execute",
                    message="secret related",
                ),
            ),
            source_locations=(location,),
            analysis_key="analysis:issued-1",
            source_problem="problem:issued-1",
            source_operation="operation:issued-1",
        )
        original = self.error(report)
        wrapped = RuntimeError("secret wrapper")
        wrapped.__cause__ = original
        value = native_worker.scientific_error(wrapped)
        self.assertEqual(value["code"], report.code)
        self.assertEqual(value["stage"], "decode")
        self.assertFalse(value["source_refusal_proved"])
        self.assertEqual(value["cause_count"], 1)
        self.assertEqual(value["context_count"], 1)
        self.assertEqual(value["related_count"], 1)
        self.assertEqual(value["location_count"], 1)
        self.assertEqual(
            value["causes"],
            [
                {
                    "index": 0,
                    "message_sha256": hashlib.sha256(
                        cause_message.encode()
                    ).hexdigest(),
                    "message_redacted": True,
                }
            ],
        )
        self.assertEqual(
            value["locations"],
            [
                {
                    "source": "ab" * 32,
                    "document": "cd" * 32,
                    "revision": "sha256:" + "ef" * 32,
                    "start": 3,
                    "end": 7,
                }
            ],
        )
        self.assertEqual(records(value["contexts"])[0]["context_count"], 1)
        self.assertEqual(records(value["contexts"])[0]["annotation_count"], 2)
        self.assertEqual(
            value["recovery"],
            {
                "analysis_key": "analysis:issued-1",
                "source_problem": "problem:issued-1",
                "source_operation": "operation:issued-1",
            },
        )
        self.assertIn("No automatic retry", text(value["remedy"]))
        self.assertNotIn("secret", json.dumps(value))
        self.assertNotIn("password", json.dumps(value))
        self.assertTrue(value["complete"])

    def test_bounded_causes_related_and_contexts_retain_actual_denominators(
        self,
    ) -> None:
        report = self.report(
            causes=tuple(SimpleNamespace(message="secret") for _ in range(18)),
            contexts=tuple(
                SimpleNamespace(code="context", contexts=(), diagnostics=())
                for _ in range(19)
            ),
            related=tuple(
                SimpleNamespace(code="related", stage="execute") for _ in range(10)
            ),
        )
        value = native_worker.scientific_error(self.error(report))
        self.assertEqual(
            (len(records(value["causes"])), value["cause_count"]), (16, 18)
        )
        self.assertEqual(
            (len(records(value["contexts"])), value["context_count"]), (16, 19)
        )
        self.assertEqual(
            (len(records(value["related"])), value["related_count"]), (8, 10)
        )
        self.assertFalse(value["complete"])
        self.assertNotIn("secret", json.dumps(value))

    def test_structural_error_uses_only_preprojected_locations(self) -> None:
        location: dict[str, object] = {
            "path": ["schema_declared", 2, "<redacted-key>"],
            "rule": "type",
        }
        value = native_worker.scientific_error(
            native_worker.StructuralInputError([location])
        )
        self.assertEqual(value["code"], "pse.native.input_schema_refused")
        self.assertEqual(value["stage"], "decode")
        self.assertEqual(value["locations"], [location])
        self.assertNotIn("recovery", value)

    def test_invalid_recovery_strings_and_implicit_context_are_not_issued_references(
        self,
    ) -> None:
        report = self.report(
            analysis_key="",
            artifact_key="/secret/path",
            artifact_owner="credential=value",
            artifact_owner_kind="☃",
            artifact_read_key="x" * 161,
            study_key="bad value",
        )
        value = native_worker.scientific_error(self.error(report))
        self.assertNotIn("recovery", value)
        self.assertNotIn("secret", json.dumps(value))
        contextual = RuntimeError("secret wrapper")
        contextual.__context__ = self.error(
            self.report(analysis_key="analysis:context-only")
        )
        self.assertNotIn("recovery", native_worker.scientific_error(contextual))
        cycle = self.error()
        cycle.__cause__ = cycle
        self.assertNotIn("recovery", native_worker.scientific_error(cycle))


class NativeFailureRemedyTests(unittest.TestCase):
    def test_remedies_follow_typed_boundary_class_instead_of_exception_prose(
        self,
    ) -> None:
        for boundary_class, expected in [
            ("resource_limit", "resource, effort and deadline"),
            ("cancelled", "work has drained"),
            ("internal", "effect and integrity state"),
            ("infrastructure", "effect and integrity state"),
            ("unsupported", "published capabilities"),
            ("trial_rejected", "scientific assessments"),
            ("invalid_model", "attributed scientific requirements"),
        ]:
            report = DiagnosticTests.report(boundary_class=boundary_class)
            value = native_worker.scientific_error(DiagnosticTests.error(report))
            self.assertEqual(value["boundary_class"], boundary_class)
            self.assertIn(expected, text(value["remedy"]))
            self.assertNotIn("secret", json.dumps(value))
            self.assertNotIn("forged-operation", json.dumps(value))

    def test_typed_effects_and_integrity_are_allowlisted_without_private_observation_text(
        self,
    ) -> None:
        report = DiagnosticTests.report(
            boundary_class="infrastructure",
            source_problem="original-problem",
            source_operation="original-operation",
            observations={
                "admission_effect": SimpleNamespace(value="unresolved_publication"),
                "integrity_failure": SimpleNamespace(value=True),
                "detail": SimpleNamespace(value="secret private source text"),
            },
        )
        value = native_worker.scientific_error(DiagnosticTests.error(report))
        self.assertEqual(value["admission_effect"], "unresolved_publication")
        self.assertTrue(value["integrity_failure"])
        self.assertIn(
            "Read back the original publication occurrence", text(value["remedy"])
        )
        self.assertIn("original-operation", json.dumps(value["recovery"]))
        self.assertNotIn("secret", json.dumps(value))
        report.observations["admission_effect"] = SimpleNamespace(
            value="secret forged effect"
        )
        self.assertNotIn(
            "admission_effect",
            native_worker.scientific_error(DiagnosticTests.error(report)),
        )


if __name__ == "__main__":
    unittest.main()
