# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Real FunctionTool selections compose native assessment and artifact owners."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import asyncio
import io
import json
import tempfile
import unittest
from dataclasses import asdict
from pathlib import Path
from unittest.mock import AsyncMock, patch

from fastmcp import Client, FastMCP
from fastmcp.exceptions import ToolError
from fastmcp.tools import FunctionTool
from pydantic import ValidationError

from scripts import assessment_store, test_resources, validation_result
from scripts.capabilities import process
from scripts.capabilities import qualification as subject
from scripts.capabilities import qualification_worker as worker
from scripts.capabilities.inventory import inventory
from scripts.capabilities.model import Contract
from scripts.tests import assessment_fixture
from scripts.tests.test_capabilities import content
from scripts.validation_scope import Gate, comprehensive, expand, functional_gates

ROOT = Path(__file__).resolve().parents[2]




class QualificationSelectionTests(unittest.TestCase):
    def test_actual_owner_expansions_and_input_distinctions(self) -> None:
        self.assertEqual(subject.AssessmentSelection().gates(), comprehensive("dev"))
        self.assertEqual(
            subject.AssessmentSelection(group="codegen-contracts-check").gates(),
            expand(("codegen-contracts-check",)),
        )
        self.assertEqual(
            subject.AssessmentSelection(functional_scopes=["managed-primary"]).gates(),
            functional_gates(["managed-primary"]),
        )
        self.assertEqual(
            subject.AssessmentSelection(functional_scopes=None).gates(),
            comprehensive("dev"),
        )
        for values in (
            {"functional_scopes": []},
            {"functional_scopes": ["unknown"]},
            {"group": "not-a-recipe-router"},
            {"group": "hygiene", "functional_scopes": ["process"]},
            {"functional_scopes": ["native", "admission"]},
        ):
            with self.subTest(values=values), self.assertRaises(ValidationError):
                subject.AssessmentSelection.model_validate(values)

    def test_scoped_compilation_cannot_inject_flags_or_reinterpret_target(self) -> None:
        selected = subject.CompilationRequest(
            output="build/control",
            package="pse-ids",
            target="test",
            test="public-api",
            features=["pse-ids/force-validate"],
            default_features=False,
        ).gates()[0]
        self.assertEqual(selected.recipe, "check-test")
        self.assertEqual(
            selected.args,
            (
                "pse-ids",
                "public-api",
                "--features",
                "pse-ids/force-validate",
                "--no-default-features",
            ),
        )
        for values in (
            {"package": "--workspace"},
            {"target": "library"},
            {"test": "unselected"},
            {"package": "pse-ids", "target": "test"},
            {"features": ["--jobs=1"]},
            {"default_features": "false"},
        ):
            with self.subTest(values=values), self.assertRaises(ValidationError):
                subject.CompilationRequest.model_validate(
                    {"output": "build/control", **values}
                )

    def test_reuse_is_admitted_against_actual_selected_gates(self) -> None:
        values = {
            "group": "codegen-contracts-check",
            "output": "build/new",
            "reuse_from": "build/origin",
        }
        self.assertEqual(
            subject.AssessmentRequest(**values, reuse=["codegen-python-check"]).reuse,
            ["codegen-python-check"],
        )
        for invalid in (
            {"reuse_from": None, "reuse": ["codegen-python-check"]},
            {"reuse": ["unselected"]},
            {"reuse": ["codegen-python-check"], "transfer": ["codegen-python-check"]},
        ):
            with self.subTest(invalid=invalid), self.assertRaises(ValidationError):
                subject.AssessmentRequest.model_validate({**values, **invalid})

    def test_fixed_worker_uses_native_output_and_retains_native_outcomes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = root / "new"
            native = {
                "complete": False,
                "required_checks_covered": False,
                "input_coverage": True,
                "source_unchanged": False,
                "contextual_changes": ["unrelated source changed"],
                "scope": [{"name": "compilation-check"}, {"name": "passed-control"}],
                "checks": [
                    {
                        "gate": "compilation-check",
                        "status": "failed",
                        "exit_code": 1,
                        "evidence_kind": "executed",
                        "report_errors": ["relevant inputs changed"],
                        "spawn_error": "native receiver refused startup",
                        "process": {"owned_drain": "complete"},
                    },
                    {
                        "gate": "passed-control",
                        "status": "passed",
                        "report_errors": [],
                        "results": [
                            {"nodeid": str(i), "status": "passed"} for i in range(10000)
                        ],
                    },
                ],
            }

            def fresh(selected_root: Path, requested: Path) -> Path:
                self.assertEqual(selected_root, root)
                self.assertEqual(requested, output)
                requested.mkdir()
                return requested

            def run(
                selected_root: Path,
                actual_output: Path,
                gates: list[Gate],
                **options: object,
            ) -> int:
                self.assertEqual(selected_root, root)
                self.assertEqual(gates[0].recipe, "check-library")
                self.assertTrue(options["capture"])
                assessment_fixture.publish(actual_output, native)
                # A printed string cannot become this adapter's result handle.
                print("validation evidence: /not/the/result")
                return 1

            with (
                patch.object(worker, "ROOT", root),
                patch.object(worker.validation, "fresh_output", side_effect=fresh),
                patch.object(worker.validation, "run_gates", side_effect=run),
                patch.object(
                    test_resources, "registry", return_value=root / "registry"
                ),
                patch.object(
                    assessment_store, "detail", wraps=assessment_store.detail
                ) as details,
                patch.object(
                    worker.validation,
                    "main",
                    side_effect=AssertionError("CLI reconstructed"),
                ),
            ):
                result = worker.execute(
                    "compilation",
                    subject.CompilationRequest(
                        output="new", package="pse-ids", target="library"
                    ).model_dump_json(),
                )
            self.assertEqual(result["output"], str(output))
            self.assertEqual(result["receipt"], str(output / "checks.json"))
            self.assertEqual(result["exit_code"], 1)
            self.assertFalse(result["complete"])
            self.assertFalse(result["source_unchanged"])
            checks = result["checks"]
            assert isinstance(checks, list)
            self.assertEqual(checks[0]["status"], "failed")
            self.assertEqual(
                checks[0]["report_errors"], native["checks"][0]["report_errors"]
            )
            self.assertEqual(
                checks[0]["spawn_error"], "native receiver refused startup"
            )
            self.assertEqual(checks[0]["detail_status"], "hydrated")
            self.assertEqual(checks[1]["detail_status"], "not_hydrated")
            self.assertEqual(checks[1]["population"]["reported"], 10000)
            self.assertNotIn("report_errors", checks[1])
            self.assertEqual(details.call_count, 1)
            self.assertEqual(details.call_args.args[1], checks[0]["detail"])
            changes = result["contextual_changes"]
            if not isinstance(changes, dict):
                self.fail("Expected an original contextual-change observation")
            self.assertEqual(changes["status"], "not_hydrated")
            source_cut = changes["source_cut_ref"]
            reader = changes["reader"]
            if not isinstance(source_cut, dict) or not isinstance(reader, dict):
                self.fail("Expected original source-cut and detail reader references")
            self.assertEqual(set(source_cut), {"sha256"})
            self.assertTrue(reader["source_detail"])
            with patch.object(
                test_resources, "registry", return_value=root / "registry"
            ):
                selected = validation_result.read_assessment(
                    output, gate="compilation-check"
                )
            self.assertEqual(
                selected["checks"][0]["record"]["report_errors"],
                native["checks"][0]["report_errors"],
            )

    def test_generation_and_documentation_keep_exact_recipe_and_provenance_scope(
        self,
    ) -> None:
        checked, _, capture, _ = worker.selected(
            "contracts-check", '{"output":"build/check"}'
        )
        self.assertEqual(checked, expand(("codegen-contracts-check",)))
        generated, _, capture, _ = worker.selected(
            "contracts-generate", '{"output":"build/generate"}'
        )
        self.assertEqual(
            generated, [Gate("codegen-contracts", input_scope="generation")]
        )
        self.assertFalse(capture)
        docs, _, capture, _ = worker.selected(
            "documentation", '{"output":"build/documentation"}'
        )
        self.assertEqual(docs, [Gate("docs", input_scope="documentation")])
        self.assertTrue(capture)

    def test_prerequisite_refusal_precedes_output_allocation(self) -> None:
        with (
            patch.object(worker.validation, "preflight_kinds", return_value=["store"]),
            patch.object(
                worker.preflight,
                "check",
                side_effect=worker.pse_env.BoundaryError(
                    "prerequisite store: unavailable"
                ),
            ),
            patch.object(
                worker.validation,
                "fresh_output",
                side_effect=AssertionError("output allocated"),
            ),
            self.assertRaisesRegex(worker.pse_env.BoundaryError, "prerequisite store"),
        ):
            worker.execute(
                "assessment",
                subject.AssessmentRequest(
                    output="build/new", functional_scopes=["managed-primary"]
                ).model_dump_json(),
            )

    def test_worker_error_keeps_native_prerequisite_status(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source, destination = (
                Path(temporary) / "request.json",
                Path(temporary) / "response.json",
            )
            source.write_text('{"output":"build/selected"}')
            with patch.object(
                worker,
                "execute",
                side_effect=worker.pse_env.BoundaryError(
                    "prerequisite store: unavailable"
                ),
            ):
                self.assertEqual(
                    worker.main(["assessment", str(source), str(destination)]), 0
                )
            failure = json.loads(destination.read_text())["error"]
            self.assertEqual(
                failure["code"], "pse.qualification.prerequisite_unavailable"
            )
            self.assertEqual(failure["exit_code"], 125)
            self.assertEqual(failure["cause"], "BoundaryError")
            self.assertEqual(failure["stage"], "preparation")
            self.assertNotIn("prerequisite store: unavailable", json.dumps(failure))
            self.assertIn("No automatic retry", failure["remedy"])

    def test_worker_input_refusal_redacts_unknown_keys_and_success_keeps_receipt(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source, destination = (
                Path(temporary) / "request.json",
                Path(temporary) / "response.json",
            )
            source.write_text(
                '{"output":"build/selected","PRIVATE-FIELD":"PRIVATE VALUE"}'
            )
            self.assertEqual(
                worker.main(["documentation", str(source), str(destination)]), 0
            )
            failure = json.loads(destination.read_text())["error"]
            self.assertEqual(failure["code"], "pse.qualification.input_refused")
            self.assertEqual(failure["stage"], "input")
            self.assertEqual(failure["fields"][0]["loc"], ["<redacted-key>"])
            self.assertNotIn("PRIVATE", json.dumps(failure))
            destination.unlink()
            native = {
                "operation": "assessment",
                "exit_code": 1,
                "receipt": "build/issued/checks.json",
                "receipt_sha256": "a" * 64,
                "complete": False,
                "reader": {
                    "capability": "assessment_read",
                    "directory": "build/issued",
                },
            }
            with patch.object(worker, "execute", return_value=native):
                self.assertEqual(
                    worker.main(["assessment", str(source), str(destination)]), 0
                )
            self.assertEqual(json.loads(destination.read_text()), {"result": native})


class QualificationToolTests(unittest.IsolatedAsyncioTestCase):
    async def test_metadata_and_schema_derive_from_real_callable_and_owner(
        self,
    ) -> None:
        specs, tools = subject.bindings(ROOT)
        domains, _workflows, _guides, families = inventory(ROOT)
        self.assertTrue(all(isinstance(tool, FunctionTool) for tool in tools.values()))
        for spec in specs:
            self.assertIn(spec.domain, {item.id for item in domains})
            self.assertIn(spec.family, {item.id for item in families})
            self.assertIn(spec.id, tools)
        selected = await tools["assessment_select"].run(
            {"request": {"group": "codegen-contracts-check"}}
        )
        self.assertEqual(
            content(selected)["result"]["checks"],
            json.loads(
                json.dumps(
                    [asdict(gate) for gate in expand(("codegen-contracts-check",))]
                )
            ),
        )

    async def test_actual_mcp_strict_inputs_refuse_before_launch(self) -> None:
        _, tools = subject.bindings(ROOT)
        server = FastMCP("qualification-controls", strict_input_validation=True)
        for tool in tools.values():
            server.add_tool(tool)
        with patch.object(process, "run_worker", new_callable=AsyncMock) as launch:
            async with Client(server) as client:
                for request in (
                    {"output": "build/new", "default_features": "false"},
                    {
                        "output": "build/new",
                        "package": "pse-ids",
                        "arbitrary_flags": ["--jobs=1"],
                    },
                ):
                    with self.subTest(request=request), self.assertRaises(ToolError):
                        await client.call_tool(
                            "compilation_check", {"request": request}
                        )
        launch.assert_not_awaited()

    async def test_transport_has_no_request_selected_module_or_shell(self) -> None:
        with patch.object(
            asyncio, "create_subprocess_exec", side_effect=AssertionError("spawned")
        ):
            for worker_name, operation in (
                ("shell", "assessment"),
                ("qualification", "bash"),
                ("host", "compilation"),
            ):
                with (
                    self.subTest(worker_name=worker_name, operation=operation),
                    self.assertRaisesRegex(
                        ValueError, "unsupported fixed worker operation"
                    ),
                ):
                    await process.run_worker(
                        ROOT,
                        worker_name,
                        operation,
                        subject.OutputRequest(output="build/new"),
                    )

    async def test_nonfinite_native_json_refuses_before_spawn_without_null_normalization(
        self,
    ) -> None:
        class NativeEnvelope(Contract):
            value: float

        with patch.object(
            asyncio,
            "create_subprocess_exec",
            side_effect=AssertionError("nonfinite request spawned"),
        ):
            for value in (float("nan"), float("inf"), float("-inf")):
                with (
                    self.subTest(value=value),
                    self.assertRaisesRegex(ValueError, "Out of range float"),
                ):
                    await process.run_worker(
                        ROOT,
                        "native",
                        "source_admit",
                        {
                            "schema_set_fingerprint": "native",
                            "arguments": {"value": value},
                        },
                    )
                with (
                    self.subTest(model_value=value),
                    self.assertRaisesRegex(ValueError, "Out of range float"),
                ):
                    await process.run_worker(
                        ROOT, "native", "source_admit", NativeEnvelope(value=value)
                    )

    async def test_native_unsettled_helper_withholds_source_bearing_diagnostics(
        self,
    ) -> None:
        class FailedProcess:
            returncode = 1

            async def wait(self) -> int:
                return 1

        async def spawn(*arguments: str, **options: object) -> FailedProcess:
            self.assertIn("--native=solver,klu,isolation,uno,petsc", arguments)
            log = options["stdout"]
            assert isinstance(log, io.BufferedIOBase)
            write = log.write
            write(b"PRIVATE SOURCE BODY /source/path-containing-values\n")
            return FailedProcess()

        with (
            patch.object(asyncio, "create_subprocess_exec", side_effect=spawn),
            self.assertRaises(ToolError) as failure,
        ):
            await process.run_worker(
                ROOT, "native", "source_admit", {"schema_set_fingerprint": "native"}
            )
        self.assertIn("source-bearing diagnostics withheld", str(failure.exception))
        self.assertNotIn("PRIVATE SOURCE BODY", str(failure.exception))
        self.assertNotIn("path-containing-values", str(failure.exception))

    async def test_host_and_qualification_helper_crashes_redact_logs_without_inventing_settlement(
        self,
    ) -> None:
        class FailedProcess:
            def __init__(self, code: int) -> None:
                self.returncode = code

            async def wait(self) -> int:
                return self.returncode

        cases = (
            ("host", "diagnostic", 7),
            ("qualification", "assessment", 0),
            ("rust-tests", "rust-test", 130),
        )
        for worker_name, operation, exit_code in cases:
            with self.subTest(worker=worker_name, exit_code=exit_code):

                async def spawn(
                    *_arguments: str, expected_exit: int = exit_code, **options: object
                ) -> FailedProcess:
                    stdout, stderr = options["stdout"], options["stderr"]
                    assert isinstance(stdout, io.BufferedIOBase)
                    assert isinstance(stderr, io.BufferedIOBase)
                    stdout.write(
                        b"CREDENTIAL_SENTINEL=private-value /private/source-containing-values\n"
                    )
                    stderr.write(
                        b"STDERR_SECRET=private-value /private/diagnostic-path\n"
                    )
                    return FailedProcess(expected_exit)

                with (
                    patch.object(asyncio, "create_subprocess_exec", side_effect=spawn),
                    self.assertRaises(ToolError) as failure,
                ):
                    await process.run_worker(
                        ROOT,
                        worker_name,
                        operation,
                        subject.OutputRequest(output="build/original-owner-output"),
                    )
                encoded = str(failure.exception)
                value = json.loads(encoded)
                self.assertEqual(value["code"], "pse.host.helper_unsettled")
                self.assertEqual(value["stage"], "helper_transport")
                self.assertEqual(value["helper_exit_code"], exit_code)
                self.assertEqual(value["output"], "build/original-owner-output")
                self.assertEqual(value["operation"], operation)
                self.assertIn(
                    "helper exit alone is not settlement", value["settlement"]
                )
                self.assertIn("Do not repeat work", value["remedy"])
                self.assertNotIn("CREDENTIAL_SENTINEL", encoded)
                self.assertNotIn("STDERR_SECRET", encoded)
                self.assertNotIn("private-value", encoded)
                self.assertNotIn("/private/", encoded)
                self.assertNotIn("receipt", value)
                self.assertNotIn("complete", value)

    async def test_scoped_rust_functiontool_uses_fixed_exclusive_owner(self) -> None:
        # Empty native preparation establishes the real operation lifetime
        # without building unrelated providers. The scoped runner needs that
        # owner before it can prepare one coherent catalogue/validation closure.
        self.assertIn(
            "--native=solver,klu,isolation,uno,petsc",
            process.WORKERS["rust-tests"].environment,
        )
        _, tools = subject.bindings(ROOT)
        server = FastMCP("scoped-rust-controls", strict_input_validation=True)
        for tool in tools.values():
            server.add_tool(tool)
        with patch.object(
            process,
            "run_worker",
            new_callable=AsyncMock,
            return_value={
                "exit_code": 1,
                "invocation": "/native/invocation.json",
                "native_terminal": {"status": "failed"},
            },
        ) as launch:
            async with Client(server) as client:
                result = await client.call_tool(
                    "rust_test_run",
                    {
                        "request": {
                            "package": "pse-ids",
                            "expression": "control",
                            "output": "build/selected",
                            "execution_effects": "native-local",
                        }
                    },
                )
                self.assertEqual(
                    content(result)["result"]["native_terminal"]["status"],
                    "failed",
                )
                self.assertEqual(
                    launch.call_args.args[1:3], ("rust-tests", "rust-test")
                )
                for invalid in (
                    {
                        "package": "--workspace",
                        "expression": "control",
                        "output": "build/new",
                    },
                    {
                        "package": "pse-ids",
                        "expression": "control",
                        "output": "build/new",
                        "args": ["--unknown"],
                    },
                    {
                        "package": "pse-ids",
                        "kind": "integration",
                        "output": "build/new",
                    },
                ):
                    with self.subTest(invalid=invalid), self.assertRaises(ToolError):
                        await client.call_tool("rust_test_run", {"request": invalid})
            self.assertEqual(launch.await_count, 1)


if __name__ == "__main__":
    unittest.main()
