# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed preparation bindings preserve owner outcomes and one resolved environment."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock, patch

from fastmcp import Client, FastMCP

from scripts import python_extension
from scripts.capabilities import preparation, preparation_worker


def outcome(
    mode: python_extension.Mode, code: int = 0
) -> python_extension.PreparationResult:
    return python_extension.PreparationResult(
        operation="prepare",
        mode=mode,
        profile="producer",
        validated=False,
        stages=[
            python_extension.StageResult(
                stage="prepare", status="failed" if code else "prepared", exit_code=code
            )
        ],
        exit_code=code,
    )


class PreparationTests(unittest.IsolatedAsyncioTestCase):
    async def test_currency_check_uses_fixed_passive_worker_and_validates_selection(
        self,
    ) -> None:
        root = Path(__file__).resolve().parents[2]
        expected = python_extension.ExtensionCurrency(
            state="unsupported",
            checkout=str(root.resolve()),
            interpreter=str((root / ".venv/bin/python").absolute()),
            mode="native",
            profile="dev",
            validated=True,
            receipt_recorded=True,
            cause="incompatible-publication",
            remedy="Explicitly prepare, reattach and check again.",
        )
        with patch.object(
            preparation.process,
            "run_worker",
            new_callable=AsyncMock,
            return_value=expected.model_dump(mode="json"),
        ) as worker:
            self.assertEqual(
                await preparation.currency_check(root, "native", validated=True),
                expected,
            )
            self.assertEqual(worker.call_args.args[1], preparation.CURRENCY_CALL)
            worker.return_value = expected.model_copy(
                update={"validated": False}
            ).model_dump(mode="json")
            with self.assertRaisesRegex(ValueError, "differs"):
                await preparation.currency_check(root, "native", validated=True)

    async def test_currency_child_uses_existing_environment_without_preparing(
        self,
    ) -> None:
        root = Path(__file__).resolve().parents[2]
        expected = python_extension.ExtensionCurrency(
            state="unknown",
            checkout=str(root),
            interpreter="/selected/bin/python",
            mode="dev",
            profile="producer",
            validated=False,
            cause="missing-publication",
            remedy="Explicitly prepare.",
        )
        stages = []
        with (
            patch.object(
                python_extension, "currency_check", return_value=expected
            ) as check,
            patch.object(python_extension, "run_stages") as prepare,
        ):
            actual = python_extension.ExtensionCurrency.model_validate(
                preparation_worker.invoke(
                    "currency-check",
                    preparation.CurrencyCheckRequest(
                        mode="dev", profile="producer", validated=False
                    ).model_dump_json(),
                    on_stage=lambda stage, result: stages.append((stage, result)),
                )
            )
        self.assertEqual(actual, expected)
        self.assertEqual(
            check.call_args.kwargs,
            {"mode": "dev", "profile": "producer", "validated": False},
        )
        self.assertEqual(stages[0], ("currency", None))
        status = stages[1][1]
        assert status is not None
        self.assertEqual(status.status, "failed")
        prepare.assert_not_called()

    async def test_actual_stage_tools_have_owner_schema_and_fixed_worker_effects(
        self,
    ) -> None:
        root = Path(__file__).resolve().parents[2]
        specs, tools = preparation.bindings(root)
        server = FastMCP("preparation-controls", strict_input_validation=True)
        for tool in tools.values():
            server.add_tool(tool)
        with patch.object(
            preparation.process, "run_worker", new_callable=AsyncMock
        ) as worker:
            async with Client(server) as client:
                exposed = {tool.name: tool for tool in await client.list_tools()}
                for mode, name in (
                    ("dev", "python_extension_dev_stage"),
                    ("native", "python_extension_native_stage"),
                ):
                    expected = outcome(mode, 17)
                    worker.return_value = expected.model_dump(mode="json")
                    result = await client.call_tool(
                        name,
                        {
                            "request": {
                                "stage": "prepare",
                                "profile": "producer",
                                "validated": False,
                            }
                        },
                    )
                    self.assertEqual(
                        python_extension.PreparationResult.model_validate_json(
                            json.dumps(result.structured_content)
                        ),
                        expected,
                    )
                    self.assertEqual(
                        worker.call_args.args[1], preparation.STAGE_CALLS[mode]
                    )
                    invocation = worker.call_args.args[2]
                    self.assertEqual(invocation.mode, mode)
                    self.assertFalse(invocation.validated)
                    self.assertEqual(invocation.stage, "prepare")
                    schema = exposed[name].output_schema
                    if schema is None:
                        raise AssertionError("owner result must have an output schema")
                    self.assertIn("stages", schema["properties"])
                    self.assertIn("exit_code", schema["properties"])
                    annotations = exposed[name].annotations
                    if annotations is None:
                        raise AssertionError("preparation must declare its effects")
                    self.assertFalse(annotations.read_only_hint)
                    self.assertTrue(annotations.destructive_hint)
                    selected = next(spec for spec in specs if spec.id == name)
                    self.assertEqual(
                        selected.worker_calls, (preparation.STAGE_CALLS[mode],)
                    )
                worker.reset_mock()
                invalid = await client.call_tool(
                    "python_extension_dev_stage",
                    {"request": {"stage": "sync"}},
                    raise_on_error=False,
                )
                self.assertTrue(invalid.is_error)
                worker.assert_not_called()

    async def test_import_check_uses_fixed_selected_worker_and_typed_capture_result(
        self,
    ) -> None:
        expected = preparation.ImportCheckResult(
            mode="native", validated=True, compatible=False, cause="ValueError"
        )
        root = Path(__file__).resolve().parents[2]
        with patch.object(
            preparation.process,
            "run_worker",
            new_callable=AsyncMock,
            return_value=expected.model_dump(mode="json"),
        ) as worker:
            actual = await preparation.import_check(root, "native", validated=True)
        self.assertEqual(actual, expected)
        self.assertEqual(worker.call_args.args[1], preparation.IMPORT_CALLS["native"])

    async def test_import_check_refuses_mismatched_selection_or_missing_observed_identity(
        self,
    ) -> None:
        root = Path(__file__).resolve().parents[2]
        with (
            patch.object(
                preparation.process,
                "run_worker",
                new_callable=AsyncMock,
                return_value={"mode": "native", "validated": True, "compatible": False},
            ),
            self.assertRaisesRegex(ValueError, "selected requirement"),
        ):
            await preparation.import_check(root, "dev", validated=True)
        with (
            patch.object(
                preparation.process,
                "run_worker",
                new_callable=AsyncMock,
                return_value={"mode": "dev", "validated": True, "compatible": True},
            ),
            self.assertRaises(ValueError),
        ):
            await preparation.import_check(root, "dev", validated=True)


class PreparationWorkerTests(unittest.TestCase):
    def test_import_checks_preserve_endpoint_provider_environment(self) -> None:
        for call in preparation.IMPORT_CALLS.values():
            self.assertEqual(call.declaration.environment, ())
            self.assertEqual(call.operation, "import-check")
        self.assertIn(
            "--native=solver,klu,isolation,uno,petsc",
            preparation.STAGE_CALLS["native"].declaration.environment,
        )

    def test_import_check_uses_its_fixed_interpreter_despite_foreign_installer_selection(
        self,
    ) -> None:
        import sys  # noqa: PLC0415

        with (
            patch.dict(
                os.environ,
                {"UV_PROJECT_ENVIRONMENT": "/foreign", "VIRTUAL_ENV": "/foreign"},
            ),
            patch.object(
                python_extension, "admit_import", side_effect=ValueError("refused")
            ) as capture,
        ):
            preparation_worker.invoke(
                "import-check",
                preparation.ImportCheckRequest(mode="native").model_dump_json(),
            )
        self.assertEqual(
            capture.call_args.args[0]["UV_PROJECT_ENVIRONMENT"], sys.prefix
        )
        self.assertEqual(capture.call_args.args[0]["VIRTUAL_ENV"], sys.prefix)

    def test_stage_child_calls_owner_once_in_resolved_environment_without_launch(
        self,
    ) -> None:
        result = outcome("dev")

        def callback(_stage: str, _result: python_extension.StageResult | None) -> None:
            pass

        with (
            patch.object(python_extension, "run_stages", return_value=result) as stages,
            patch.object(python_extension, "launch") as launch,
            patch.object(python_extension, "selected_environment") as select,
            patch.dict(
                os.environ,
                {
                    "VIRTUAL_ENV": "/foreign",
                    "UV_PROJECT_ENVIRONMENT": "/already/resolved",
                },
            ),
        ):
            actual = preparation_worker.invoke(
                "stage",
                preparation.StageInvocation(
                    stage="prepare", mode="dev", profile="producer", validated=False
                ).model_dump_json(),
                on_stage=callback,
            )
        self.assertEqual(actual, result.model_dump(mode="json"))
        self.assertEqual(stages.call_count, 1)
        self.assertEqual(stages.call_args.args[:3], ("prepare", "dev", "producer"))
        self.assertEqual(stages.call_args.args[3]["VIRTUAL_ENV"], "/already/resolved")
        self.assertIs(stages.call_args.kwargs["on_stage"], callback)
        launch.assert_not_called()
        select.assert_not_called()

    def test_stage_child_binds_missing_installer_alias_without_resolving_again(
        self,
    ) -> None:
        with (
            patch.dict(
                os.environ, {"UV_PROJECT_ENVIRONMENT": "/already/resolved"}, clear=True
            ),
            patch.object(
                python_extension, "run_stages", return_value=outcome("dev")
            ) as stages,
            patch.object(python_extension, "selected_environment") as select,
        ):
            preparation_worker.invoke(
                "stage",
                preparation.StageInvocation(
                    stage="prepare", mode="dev"
                ).model_dump_json(),
            )
        self.assertEqual(stages.call_args.args[3]["VIRTUAL_ENV"], "/already/resolved")
        self.assertEqual(
            stages.call_args.args[3]["UV_PROJECT_ENVIRONMENT"], "/already/resolved"
        )
        select.assert_not_called()

    def test_worker_main_publishes_actual_stage_outcome_without_inventing_handle(
        self,
    ) -> None:
        from scripts.capabilities.worker_progress import read  # noqa: PLC0415

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, destination = root / "request.json", root / "response.json"
            source.write_text(
                preparation.StageInvocation(
                    stage="environment", mode="dev"
                ).model_dump_json()
            )
            failed = python_extension.StageResult(
                stage="environment", status="failed", exit_code=17
            )
            with patch.object(
                python_extension, "environment_ensure", return_value=failed
            ):
                self.assertEqual(
                    preparation_worker.main(["stage", str(source), str(destination)]), 0
                )
            progress = read(root / "progress.json", "stage")
            if progress is None:
                raise AssertionError("expected real stage progress snapshot")
            self.assertEqual(progress.stage, "environment")
            self.assertEqual(progress.state, "failed")
            self.assertEqual(progress.sequence, 2)
            self.assertFalse(progress.references)
            result = python_extension.PreparationResult.model_validate_json(
                json.dumps(json.loads(destination.read_text())["result"])
            )
            self.assertEqual(result.exit_code, 17)

    def test_import_child_refusal_is_observation_without_implicit_preparation(
        self,
    ) -> None:
        stages = []
        with (
            patch.object(
                python_extension,
                "admit_import",
                side_effect=ValueError("private failure"),
            ) as capture,
            patch.object(python_extension, "run_stages") as prepare,
        ):
            actual = preparation.ImportCheckResult.model_validate(
                preparation_worker.invoke(
                    "import-check",
                    preparation.ImportCheckRequest(
                        mode="native", validated=True
                    ).model_dump_json(),
                    on_stage=lambda stage, result: stages.append((stage, result)),
                )
            )
        self.assertFalse(actual.compatible)
        self.assertEqual(actual.cause, "ValueError")
        self.assertNotIn("private failure", actual.model_dump_json())
        self.assertTrue(capture.call_args.kwargs["native"])
        self.assertTrue(capture.call_args.kwargs["validated"])
        self.assertEqual(stages[0], ("import", None))
        self.assertEqual(stages[1][0], "import")
        result = stages[1][1]
        assert result is not None
        self.assertEqual(result.status, "failed")
        prepare.assert_not_called()

    def test_import_child_reports_actual_artifact_identity_and_preserves_interactive_mode(
        self,
    ) -> None:
        artifact = {"binary": "/selected/native.so", "sha256": "actual", "features": []}
        with patch.object(
            python_extension, "admit_import", return_value=artifact
        ) as capture:
            actual = preparation.ImportCheckResult.model_validate(
                preparation_worker.invoke(
                    "import-check",
                    preparation.ImportCheckRequest(
                        mode="dev", validated=False
                    ).model_dump_json(),
                )
            )
        self.assertTrue(actual.compatible)
        self.assertEqual(
            actual.artifact, python_extension.ExtensionArtifact.model_validate(artifact)
        )
        self.assertFalse(capture.call_args.kwargs["validated"])
        self.assertFalse(capture.call_args.kwargs["native"])


if __name__ == "__main__":
    unittest.main()
