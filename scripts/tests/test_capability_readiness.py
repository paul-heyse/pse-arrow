# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Passive and selected live readiness retain independent prerequisite facts."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock, patch

from fastmcp import Client, FastMCP
from fastmcp.exceptions import ToolError

from scripts import python_extension
from scripts.capabilities import preparation, readiness, specialists
from scripts.capabilities.model import Capability, Prerequisite


def capability(*requirements: Prerequisite, identifier: str = "selected") -> Capability:
    return Capability(
        id=identifier,
        title="Selected operation",
        description="Selected operation controls",
        domain="environment",
        owner="scripts/python_extension.py",
        source="scripts/python_extension.py",
        exposure="tool",
        effects="Test selected operation only",
        requirements=requirements,
    )


def extension(
    mode: python_extension.Mode | None = "dev", *, validated: bool = True
) -> Prerequisite:
    return Prerequisite(
        kind="python-extension",
        owner="scripts/python_extension.py",
        preparation=(
            "python_extension_dev_stage"
            if mode == "dev"
            else "python_extension_native_stage",
        ),
        mode=mode,
        validated=validated,
    )


def observed(result: object) -> readiness.CapabilityReadiness:
    value = getattr(result, "structured_content", None)
    if value is None:
        raise AssertionError("expected typed readiness structured content")
    return readiness.CapabilityReadiness.model_validate_json(json.dumps(value))


class ReadinessTests(unittest.IsolatedAsyncioTestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.environment = patch.dict(
            os.environ, {"UV_PROJECT_ENVIRONMENT": str(self.root / ".venv")}
        )
        self.environment.start()
        self.addCleanup(self.environment.stop)
        self.owner = specialists.Specialists(self.root)
        self.addAsyncCleanup(self.owner.close)

    def tools(self, selected: Capability) -> dict:
        _, tools = readiness.bindings(
            self.root,
            capabilities=lambda: {selected.id: selected},
            specialists=self.owner,
        )
        return tools

    def receipt(
        self, *, mode: str = "dev", validated: bool = True, profile: str = "dev"
    ) -> None:
        path = python_extension.stage_receipt("extension", os.environ, root=self.root)
        path.parent.mkdir(parents=True)
        path.write_text(
            json.dumps(
                {
                    "inputs": {
                        "mode": mode,
                        "validated": validated,
                        "profile": profile,
                    },
                    "artifact": {
                        "binary": str(self.root / "python/pse/_native.so"),
                        "sha256": "recorded",
                        "features": (["force-validate"] if validated else [])
                        + (["native-solvers"] if mode == "native" else []),
                    },
                }
            )
        )

    async def test_selected_currency_keeps_historical_import_and_operation_facts_separate(
        self,
    ) -> None:
        self.receipt()
        tools = self.tools(capability(extension()))
        currency = python_extension.ExtensionCurrency(
            state="stale",
            checkout=str(self.root),
            interpreter=str(self.root / ".venv/bin/python"),
            mode="dev",
            profile="producer",
            validated=True,
            source_basis="fresh-source-basis",
            receipt_recorded=True,
            cause="inputs-changed",
            remedy="Explicitly prepare and reattach.",
        )
        with (
            patch.object(
                preparation,
                "currency_check",
                new_callable=AsyncMock,
                return_value=currency,
            ) as check,
            patch.object(
                preparation, "import_check", new_callable=AsyncMock
            ) as imported,
        ):
            result = observed(
                await tools["capability_readiness_check"].run(
                    {
                        "request": {
                            "capability": "selected",
                            "checks": ["native-currency"],
                            "profile": "producer",
                        }
                    }
                )
            )
        self.assertEqual(result.native_currency, currency)
        self.assertTrue(result.artifact_prepared.value)
        self.assertIsNone(result.import_compatible.value)
        self.assertIsNone(result.operation_exercised.value)
        self.assertEqual(result.receipts[0].currency, "unknown")
        check.assert_awaited_once_with(
            self.root, "dev", profile="producer", validated=True
        )
        imported.assert_not_called()

    async def test_currency_transport_failure_is_unknown_and_redacted(self) -> None:
        tools = self.tools(capability(extension("native")))
        with (
            patch.object(
                preparation,
                "currency_check",
                new_callable=AsyncMock,
                side_effect=ToolError("private credential"),
            ),
            patch.object(python_extension, "run_stages") as stages,
        ):
            result = observed(
                await tools["capability_readiness_check"].run(
                    {
                        "request": {
                            "capability": "selected",
                            "checks": ["native-currency"],
                            "profile": "selected-producer",
                        }
                    }
                )
            )
        assert result.native_currency is not None
        self.assertEqual(result.native_currency.state, "unknown")
        self.assertEqual(result.native_currency.cause, "check-unavailable")
        self.assertNotIn("private credential", result.model_dump_json())
        stages.assert_not_called()

    async def test_installed_selection_uses_receipt_and_preserves_explicit_fields(
        self,
    ) -> None:
        self.receipt(mode="native", profile="actual-producer")
        tools = self.tools(capability(extension(None)))
        for supplied, mode, profile in (
            ({}, "native", "actual-producer"),
            ({"mode": "dev"}, "dev", "actual-producer"),
            ({"profile": "explicit"}, "native", "explicit"),
            ({"mode": "dev", "profile": "explicit"}, "dev", "explicit"),
        ):
            currency = python_extension.ExtensionCurrency(
                state="unknown",
                checkout=str(self.root),
                interpreter=str(self.root / ".venv/bin/python"),
                mode=mode,
                profile=profile,
                validated=True,
                cause="check-unavailable",
                remedy="Inspect owner.",
            )
            with (
                patch.object(
                    preparation,
                    "currency_check",
                    new_callable=AsyncMock,
                    return_value=currency,
                ) as check,
                patch.object(python_extension, "run_stages") as stages,
                patch.object(
                    preparation, "import_check", new_callable=AsyncMock
                ) as imported,
            ):
                await tools["capability_readiness_check"].run(
                    {
                        "request": {
                            "capability": "selected",
                            "checks": ["native-currency"],
                            **supplied,
                        }
                    }
                )
            check.assert_awaited_once_with(
                self.root, mode, profile=profile, validated=True
            )
            stages.assert_not_called()
            imported.assert_not_called()

    async def test_installed_missing_receipt_refuses_without_guessed_mode_or_live_owner(
        self,
    ) -> None:
        tools = self.tools(capability(extension(None)))
        with (
            patch.object(
                preparation, "currency_check", new_callable=AsyncMock
            ) as check,
            patch.object(
                preparation, "import_check", new_callable=AsyncMock
            ) as imported,
            patch.object(python_extension, "run_stages") as stages,
        ):
            for checks in (["native-currency"], ["import-compatible"]):
                with self.assertRaises(ToolError):
                    await tools["capability_readiness_check"].run(
                        {"request": {"capability": "selected", "checks": checks}}
                    )
        check.assert_not_called()
        imported.assert_not_called()
        stages.assert_not_called()

    async def test_currency_selection_is_validated_before_any_live_check(self) -> None:
        tools = self.tools(capability(extension()))
        with (
            patch.object(
                preparation, "currency_check", new_callable=AsyncMock
            ) as check,
            self.assertRaises(ToolError),
        ):
            await tools["capability_readiness_check"].run(
                {
                    "request": {
                        "capability": "selected",
                        "checks": ["native-currency", "language-provider"],
                    }
                }
            )
        check.assert_not_called()

    async def test_passive_lookup_with_missing_providers_never_imports_prepares_or_connects(
        self,
    ) -> None:
        selected = capability(
            extension(),
            Prerequisite(
                kind="selected-service",
                owner="scripts/surreal_server.py",
                provider="surreal-selected",
            ),
        )
        tools = self.tools(selected)
        with (
            patch.object(
                python_extension,
                "capture",
                side_effect=AssertionError("must not import"),
            ) as capture,
            patch.object(
                python_extension,
                "preparation_inputs",
                side_effect=AssertionError("must not probe compiler"),
            ) as inputs,
            patch.object(
                python_extension,
                "run_stages",
                side_effect=AssertionError("must not prepare"),
            ) as stages,
            patch.object(
                preparation.process, "run_worker", new_callable=AsyncMock
            ) as worker,
            patch.object(
                self.owner, "select_database", new_callable=AsyncMock
            ) as service,
        ):
            result = observed(
                await tools["capability_readiness"].run({"capability": selected.id})
            )
        self.assertTrue(result.declared_requirements.value)
        self.assertIsNone(result.artifact_prepared.value)
        self.assertIsNone(result.import_compatible.value)
        self.assertIsNone(result.selected_service_available.value)
        self.assertIsNone(result.operation_exercised.value)
        capture.assert_not_called()
        self.assertIsNone(result.native_currency)
        inputs.assert_not_called()
        stages.assert_not_called()
        worker.assert_not_called()
        service.assert_not_called()
        self.assertFalse((self.root / "build").exists())

    async def test_receipt_prepared_is_independent_from_incompatible_live_import(
        self,
    ) -> None:
        self.receipt()
        tools = self.tools(capability(extension()))
        before = observed(
            await tools["capability_readiness"].run({"capability": "selected"})
        )
        self.assertTrue(before.artifact_prepared.value)
        self.assertEqual(before.receipts[0].currency, "unknown")
        self.assertIsNone(before.import_compatible.value)
        incompatible = preparation.ImportCheckResult(
            mode="dev", validated=True, compatible=False, cause="ValueError"
        )
        with patch.object(
            preparation,
            "import_check",
            new_callable=AsyncMock,
            return_value=incompatible,
        ) as check:
            after = observed(
                await tools["capability_readiness_check"].run(
                    {
                        "request": {
                            "capability": "selected",
                            "checks": ["import-compatible"],
                        }
                    }
                )
            )
        self.assertTrue(after.artifact_prepared.value)
        self.assertFalse(after.import_compatible.value)
        self.assertIsNone(after.operation_exercised.value)
        check.assert_awaited_once_with(self.root, "dev", validated=True)

    async def test_explicit_stronger_import_check_preserves_required_validation(
        self,
    ) -> None:
        tools = self.tools(capability(extension(validated=False)))
        checked = preparation.ImportCheckResult(
            mode="dev",
            validated=True,
            compatible=True,
            artifact=python_extension.ExtensionArtifact(
                binary="/actual/native.so", sha256="actual", features=["force-validate"]
            ),
        )
        with patch.object(
            preparation, "import_check", new_callable=AsyncMock, return_value=checked
        ) as check:
            result = observed(
                await tools["capability_readiness_check"].run(
                    {
                        "request": {
                            "capability": "selected",
                            "checks": ["import-compatible"],
                            "validated": True,
                        }
                    }
                )
            )
        self.assertTrue(result.import_compatible.value)
        check.assert_awaited_once_with(self.root, "dev", validated=True)

    async def test_transport_refusal_is_unknown_compatibility_without_automatic_preparation(
        self,
    ) -> None:
        tools = self.tools(capability(extension("native")))
        with (
            patch.object(
                preparation,
                "import_check",
                new_callable=AsyncMock,
                side_effect=ToolError("private"),
            ),
            patch.object(python_extension, "run_stages") as stages,
        ):
            result = observed(
                await tools["capability_readiness_check"].run(
                    {
                        "request": {
                            "capability": "selected",
                            "checks": ["import-compatible"],
                        }
                    }
                )
            )
        self.assertIsNone(result.import_compatible.value)
        self.assertEqual(result.import_compatible.observation, "live")
        self.assertNotIn("private", result.model_dump_json())
        stages.assert_not_called()

    async def test_unavailable_selected_service_and_native_request_store_stay_distinct(
        self,
    ) -> None:
        tools = self.tools(
            capability(
                Prerequisite(
                    kind="selected-service",
                    owner="scripts/surreal_server.py",
                    provider="surreal-selected",
                )
            )
        )
        with patch.object(
            self.owner,
            "select_database",
            new_callable=AsyncMock,
            side_effect=RuntimeError("unavailable"),
        ) as service:
            result = observed(
                await tools["capability_readiness_check"].run(
                    {
                        "request": {
                            "capability": "selected",
                            "checks": ["selected-service"],
                        }
                    }
                )
            )
        self.assertFalse(result.selected_service_available.value)
        service.assert_awaited_once()
        native = self.tools(
            capability(
                Prerequisite(
                    kind="selected-service",
                    owner="native request",
                    provider="canonical-request",
                )
            )
        )
        with patch.object(
            self.owner, "select_database", new_callable=AsyncMock
        ) as service:
            result = observed(
                await native["capability_readiness_check"].run(
                    {
                        "request": {
                            "capability": "selected",
                            "checks": ["selected-service"],
                        }
                    }
                )
            )
        self.assertIsNone(result.selected_service_available.value)
        service.assert_not_called()

    async def test_language_check_requires_explicit_selection_and_uses_original_owner(
        self,
    ) -> None:
        tools = self.tools(
            capability(
                Prerequisite(
                    kind="language-provider", owner="scripts/semantic_navigation.py"
                )
            )
        )
        with patch.object(
            self.owner,
            "language",
            new_callable=AsyncMock,
            return_value={"native": {"isError": False}},
        ) as language:
            with self.assertRaises(ToolError):
                await tools["capability_readiness_check"].run(
                    {
                        "request": {
                            "capability": "selected",
                            "checks": ["language-provider"],
                        }
                    }
                )
            language.assert_not_called()
            result = observed(
                await tools["capability_readiness_check"].run(
                    {
                        "request": {
                            "capability": "selected",
                            "checks": ["language-provider"],
                            "language": "python",
                            "file_path": "sample.py",
                        }
                    }
                )
            )
        self.assertTrue(result.language_provider_available.value)
        self.assertIsNone(result.selected_service_available.value)
        self.assertIsNone(result.operation_exercised.value)
        language.assert_awaited_once_with(
            "python", "get_tool_support", {"file_path": "sample.py"}
        )

    async def test_missing_declarations_and_nonmatching_receipts_do_not_claim_ready(
        self,
    ) -> None:
        unknown = observed(
            await self.tools(capability())["capability_readiness"].run(
                {"capability": "selected"}
            )
        )
        self.assertIsNone(unknown.declared_requirements.value)
        self.assertIsNone(unknown.artifact_prepared.value)
        self.receipt(mode="native", validated=False)
        mismatched = observed(
            await self.tools(capability(extension()))["capability_readiness"].run(
                {"capability": "selected"}
            )
        )
        self.assertIsNone(mismatched.artifact_prepared.value)
        self.assertIsNone(mismatched.import_compatible.value)

    async def test_malformed_owner_receipt_is_unknown_without_probing_or_repair(
        self,
    ) -> None:
        self.receipt()
        path = python_extension.stage_receipt("extension", os.environ, root=self.root)
        document = json.loads(path.read_text())
        document["inputs"]["mode"] = {"invalid": "selection"}
        path.write_text(json.dumps(document))
        original = path.read_bytes()
        result = observed(
            await self.tools(capability(extension()))["capability_readiness"].run(
                {"capability": "selected"}
            )
        )
        self.assertIsNone(result.artifact_prepared.value)
        self.assertFalse(result.receipts[0].recorded)
        self.assertEqual(path.read_bytes(), original)

    async def test_all_live_selections_validate_before_any_owner_acquisition(
        self,
    ) -> None:
        tools = self.tools(capability(extension()))
        with patch.object(preparation, "import_check", new_callable=AsyncMock) as check:
            for request in (
                {
                    "capability": "selected",
                    "checks": ["import-compatible", "selected-service"],
                },
                {
                    "capability": "selected",
                    "checks": ["import-compatible"],
                    "mode": "native",
                },
                {
                    "capability": "selected",
                    "checks": ["import-compatible"],
                    "validated": False,
                },
            ):
                with self.subTest(request=request), self.assertRaises(ToolError):
                    await tools["capability_readiness_check"].run({"request": request})
            check.assert_not_called()

    async def test_actual_mcp_schema_and_successful_check_preserve_unknown_operation(
        self,
    ) -> None:
        selected = capability(extension())
        specs, tools = readiness.bindings(
            self.root,
            capabilities=lambda: {selected.id: selected},
            specialists=self.owner,
        )
        server = FastMCP("readiness-controls", strict_input_validation=True)
        for tool in tools.values():
            server.add_tool(tool)
        artifact = python_extension.ExtensionArtifact(
            binary="/actual/native.so", sha256="actual", features=["force-validate"]
        )
        imported = preparation.ImportCheckResult(
            mode="dev", validated=True, compatible=True, artifact=artifact
        )
        with patch.object(
            preparation, "import_check", new_callable=AsyncMock, return_value=imported
        ):
            async with Client(server) as client:
                exposed = {tool.name: tool for tool in await client.list_tools()}
                result = observed(
                    await client.call_tool(
                        "capability_readiness_check",
                        {
                            "request": {
                                "capability": "selected",
                                "checks": ["import-compatible"],
                            }
                        },
                    )
                )
                self.assertTrue(result.import_compatible.value)
                self.assertEqual(result.imported_artifact, artifact)
                self.assertIsNone(result.operation_exercised.value)
                for name in tools:
                    schema = exposed[name].output_schema
                    if schema is None:
                        raise AssertionError("readiness needs typed output schema")
                    if name == "capability_explain":
                        schema = schema["properties"]["readiness"]
                    self.assertIn("import_compatible", schema["properties"])
                    self.assertIn("operation_exercised", schema["properties"])
                    self.assertIn("native_currency", schema["properties"])
                passive_annotations = exposed["capability_readiness"].annotations
                live_annotations = exposed["capability_readiness_check"].annotations
                if passive_annotations is None or live_annotations is None:
                    raise AssertionError("readiness effects must be described")
                self.assertTrue(passive_annotations.read_only_hint)
                self.assertFalse(live_annotations.read_only_hint)
        declared = next(
            spec for spec in specs if spec.id == "capability_readiness_check"
        )
        self.assertEqual(
            declared.worker_calls,
            (*preparation.IMPORT_CALLS.values(), preparation.CURRENCY_CALL),
        )


if __name__ == "__main__":
    unittest.main()
