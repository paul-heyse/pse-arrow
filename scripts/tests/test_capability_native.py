# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Native schemas keep original resources and actual receivers keep admission."""
# ruff: noqa: PT009, PT027 -- stdlib host controls

from __future__ import annotations

import ast
import asyncio
import copy
import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock, patch

from fastmcp import Client
from fastmcp.exceptions import ToolError
from jsonschema import Draft202012Validator

from scripts.capabilities import native
from scripts.capabilities.native_schema import SchemaSet

ROOT = Path(__file__).resolve().parents[2]


def fingerprint() -> str:
    module = ast.parse((ROOT / "python/pse/contracts/documents/__init__.py").read_text())
    for statement in module.body:
        if isinstance(statement, ast.Assign) and any(isinstance(item, ast.Name) and item.id == "SCHEMA_SET_FINGERPRINT" for item in statement.targets):
            return ast.literal_eval(statement.value)
    raise AssertionError("generated fingerprint absent")


def schema_fixture(root: Path) -> None:
    """A miniature producer fixture; not a production fallback catalogue."""
    directory = root / "docs/generated/schema"
    directory.mkdir(parents=True)
    shutil.copyfile(ROOT / "docs/generated/schema/authoring.schema.json", directory / "authoring.schema.json")
    rows = []
    requested = {(slug, "input") for binding in native.BINDINGS for slug in binding.documents.values()}
    requested |= {(binding.output_document, "output") for binding in native.BINDINGS if binding.output_document}
    for slug, direction in sorted(requested):
        filename = f"{slug}.{direction}.schema.json"
        source = ROOT / "docs/generated/schema" / filename
        shutil.copyfile(source, directory / filename)
        schema = json.loads(source.read_text())
        rows.append({"slug": slug, "direction": direction, "name": schema["title"].removesuffix("Output"), "exported_name": schema["title"], "wire": "Native", "rust_type": "fixture::Owner", "registration_source": "fixture", "version": 1, "schema_path": f"docs/generated/schema/{filename}"})
    (directory / "catalogue.json").write_text(json.dumps({"version": 1, "schema_set_fingerprint": fingerprint(), "documents": rows}))


def source_request() -> dict[str, object]:
    physical = ROOT / "tests/fixtures/packages/physical-primitives"
    physical_documents = {str(path.relative_to(physical)): list(path.read_bytes()) for path in physical.rglob("*") if path.is_file()}
    manifest = (ROOT / "tests/fixtures/packages/minimal_explicit/package.toml").read_text().replace('id_policy = "explicit"', 'id_policy = "named"')
    manifest = manifest.replace('dependencies = []', 'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]')
    return {
        "physical": {"version": 1, "packages": [physical_documents]},
        "documents": {"version": 1, "packages": [{"package.toml": list(manifest.encode()), "models/fixed.pse": list(b"package local {param a:Scalar=1;}")}]},
        "resources": {"threads": 16, "memory_limit_bytes": 16 << 30, "max_spill_bytes": 1 << 30},
    }


def preparation_request(root: str) -> dict[str, object]:
    return {
        "document_version": 1, "root": root, "instance": root,
        "bindings": {"arguments": {}, "scope": {}, "facts": [], "demand": [], "formulation": {"omitted": [], "elastic": {}}},
        "limits": {"depth": 64, "items": 100000, "members": 100000, "body_occurrences": None, "body_slots": None},
        "durable_after": None,
    }


class NativeSchemaTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        self.addCleanup(self.directory.cleanup)
        schema_fixture(self.root)
        self.schemas = SchemaSet(self.root)
        self.specs, self.tools = native.bindings(self.root)

    def test_source_only_registration_uses_published_schema_and_direction(self) -> None:
        tool = self.tools["native_project_select"]
        facts = tool.meta["pse.native"]
        self.assertEqual(facts["receiver"], "pse.Runtime.project_select")
        self.assertEqual(facts["contracts"]["request"]["direction"], "input")
        self.assertEqual(facts["contracts"]["output_document"]["direction"], "output")
        self.assertEqual(tool.parameters["$defs"]["project-select-request"]["properties"]["reference"]["$ref"], "#/$defs/ProjectReference")
        self.assertEqual(self.schemas.document("project-select-request")["properties"]["reference"]["$ref"], "#/$defs/ProjectReference")

    def test_embedded_resources_resolve_tagged_refs_and_reject_unknown_variants(self) -> None:
        validator = Draft202012Validator(self.tools["native_project_select"].parameters)
        data = {"state": "/explicit/store", "database": "existing", "physical": {"version": 1, "packages": []}, "request": {"version": 1, "reference": {"kind": "bookmark", "key": "mark"}}}
        validator.validate(data)
        for bad in ({"kind": "latest", "key": "mark"}, {"kind": "bookmark", "key": "mark", "unknown": 1}):
            refused = copy.deepcopy(data)
            refused["request"]["reference"] = bad
            self.assertFalse(validator.is_valid(refused))
        data["request"]["reference"] = {"kind": "branch", "key": "main"}
        validator.validate(data)

    def test_generated_byte_numeric_constraints_survive_envelope(self) -> None:
        validator = Draft202012Validator(self.tools["native_source_admit"].parameters)
        data = {"physical": {"version": 1, "packages": []}, "documents": {"version": 1, "packages": [{"package.toml": [0, 255]}]}}
        validator.validate(data)
        for byte in (-1, 256, True, "1"):
            data["documents"]["packages"][0]["package.toml"] = [byte]
            self.assertFalse(validator.is_valid(data))

    def test_recursive_preparation_refs_and_nested_numeric_refusal(self) -> None:
        validator = Draft202012Validator(self.tools["native_preparation_discover"].parameters)
        request = preparation_request("ab" * 16)
        request["bindings"]["arguments"] = {"nested": {"value_kind": "tuple", "items": [{"value_kind": "set", "items": [{"value_kind": "number", "bits": 0, "quantity": "cd" * 16}]}]}}
        value = {"physical": {"version": 1, "packages": []}, "documents": {"version": 1, "packages": []}, "request": request}
        validator.validate(value)
        request["bindings"]["arguments"]["nested"]["items"][0]["items"][0]["bits"] = -1
        self.assertFalse(validator.is_valid(value))

    def test_schema_reader_serves_actual_native_refs_and_authoring_grammar(self) -> None:
        tool = self.tools["native_contract_schema"]
        native_result = asyncio.run(tool.run({"slug": "project-select-request", "direction": "input", "pointer": "/$defs/ProjectReference"})).structured_content["payload"]
        self.assertEqual(native_result["schema"]["oneOf"][0]["properties"]["kind"]["const"], "branch")
        self.assertEqual(native_result["root_ref"]["arguments"], {"slug": "project-select-request", "direction": "input"})
        authoring = self.schemas.read("authoring", None, "/anyOf/4/properties/package")
        self.assertEqual(authoring["identity"]["kind"], "authoring_guidance")
        self.assertNotIn("direction", authoring["identity"])
        self.assertEqual(authoring["schema"]["$ref"], "#/$defs/source:package_header:package")
        with patch.object(Path, "read_bytes", side_effect=AssertionError("prepared schema must be reused")):
            package = self.schemas.read("authoring", None, "/$defs/source:package_header:package")
        self.assertIn("package_id", package["schema"]["properties"])
        with self.assertRaises(ToolError):
            asyncio.run(tool.run({"slug": "authoring", "direction": "input"}))
        with self.assertRaises(ValueError):
            self.schemas.read("project-select-request", "input", "/trailing~")

    def test_actual_composed_mcp_schema_consumer_uses_native_catalogue(self) -> None:
        from scripts.capabilities.server import build

        async def exercise() -> None:
            runtime = build(ROOT)
            async with Client(runtime.server) as client:
                await client.call_tool("capabilities_select", {"capabilities": ["native_contract_schema"]})
                result = await client.call_tool("capabilities_invoke_read", {"capability": "native_contract_schema", "arguments": {"slug": "project-select-request", "direction": "input", "pointer": "/$defs/ProjectReference"}})
                payload = result.structured_content["payload"]
                self.assertEqual(payload["identity"]["wire"], "Native")
                self.assertEqual(payload["identity"]["name"], "ProjectSelectRequest")
                self.assertEqual(payload["schema"]["oneOf"][1]["properties"]["kind"]["const"], "bookmark")
                self.assertEqual(payload["root_ref"]["arguments"], {"slug": "project-select-request", "direction": "input"})

        asyncio.run(exercise())

    def test_host_refusal_is_redacted_and_does_not_invoke_child(self) -> None:
        tool = self.tools["native_project_metadata"]
        with patch.object(native, "run_child", side_effect=AssertionError("must not execute")):
            result = asyncio.run(tool.run({"state": "secret", "database": "db", "request": {}, "extra": "secret-value"}))
        self.assertEqual(result.structured_content["payload"]["outcome"], "malformed_invocation")
        self.assertNotIn("secret-value", json.dumps(result.structured_content))

    def test_absent_null_and_empty_native_document_are_not_normalized(self) -> None:
        tool = self.tools["native_project_metadata"]
        common = {"state": "/selected", "database": "existing"}
        seen = []

        def receiver(_root: Path, _operation: str, arguments: dict[str, object]) -> dict[str, str]:
            seen.append(arguments["request"])
            return {"outcome": "native_refusal"}

        with patch.object(native, "run_child", side_effect=receiver):
            absent = asyncio.run(tool.run(common))
            asyncio.run(tool.run({**common, "request": None}))
            asyncio.run(tool.run({**common, "request": {}}))
        self.assertEqual(absent.structured_content["payload"]["outcome"], "malformed_invocation")
        self.assertEqual(seen, [None, {}])

    def test_native_incomplete_result_preserves_explicit_omission(self) -> None:
        with patch.object(native, "run_child", return_value={"outcome": "incomplete_observation", "complete": False, "omissions": ["Original detail bound"]}):
            result = asyncio.run(self.tools["native_invocation_read"].run({"path": "selected"}))
        self.assertFalse(result.structured_content["complete"])
        self.assertIn("Original detail bound", result.structured_content["omissions"])

    def test_cancellation_detaches_and_retains_actual_child_observer(self) -> None:
        async def exercise() -> None:
            started = asyncio.Event()
            release = asyncio.Event()

            async def actual(*_args: object) -> dict[str, str]:
                started.set()
                await release.wait()
                return {"outcome": "observed"}

            with patch.object(native.asyncio, "to_thread", side_effect=actual):
                task = asyncio.create_task(self.tools["native_invocation_read"].run({"path": "selected"}))
                await started.wait()
                task.cancel()
                with self.assertRaises(asyncio.CancelledError):
                    await task
                self.assertEqual(len(native._PENDING), 1)
                child = next(iter(native._PENDING))
                self.assertFalse(child.cancelled())
                release.set()
                await child
                await asyncio.sleep(0)
                self.assertFalse(native._PENDING)

        asyncio.run(exercise())

    def test_read_only_child_never_requests_native_provider_preparation(self) -> None:
        with patch.object(native.subprocess, "run") as run:
            run.return_value.returncode = 125
            native.run_child(self.root, "invocation_read", {})
        command = run.call_args.args[0]
        self.assertFalse(any(item.startswith("--native") for item in command))
        self.assertTrue(self.tools["native_invocation_read"].annotations.read_only_hint)
        self.assertFalse(self.tools["native_preparation_discover"].annotations.read_only_hint)
        with patch.object(native, "run_worker", new_callable=AsyncMock) as run:
            run.return_value = {"outcome": "admitted"}
            result = asyncio.run(self.tools["native_source_admit"].run({"physical": {}, "documents": {}}))
        self.assertEqual(run.call_args.args[:3], (self.root, "native", "source_admit"))
        self.assertEqual(run.call_args.args[3]["schema_set_fingerprint"], self.schemas.fingerprint)
        self.assertEqual(result.structured_content["payload"]["outcome"], "admitted")

    def test_fixed_action_file_protocol_preserves_redacted_prerequisite(self) -> None:
        from scripts.capabilities import native_worker

        source = self.root / "request.json"
        target = self.root / "response.json"
        source.write_text(json.dumps({"schema_set_fingerprint": "loaded"}))
        with patch.object(native_worker, "execute", side_effect=ImportError("secret-loader-detail")):
            code = native_worker.main(["source_admit", str(source), str(target)])
        self.assertEqual(code, 0)
        result = json.loads(target.read_text())["result"]
        self.assertEqual(result["outcome"], "prerequisite_unavailable")
        self.assertNotIn("secret-loader-detail", target.read_text())

    def test_unknown_or_nonfinite_producer_values_are_never_stringified(self) -> None:
        from scripts.capabilities import native_worker

        for index, value in enumerate((object(), float("nan"))):
            source = self.root / f"request-{index}.json"
            target = self.root / f"response-{index}.json"
            source.write_text("{}")
            with patch.object(native_worker, "execute", return_value={"value": value}):
                native_worker.main(["source_admit", str(source), str(target)])
            result = json.loads(target.read_text())["result"]
            self.assertEqual(result["outcome"], "schema_failure")
            self.assertNotIn("object at", target.read_text())
            self.assertNotIn("NaN", target.read_text())

    def test_malformed_child_transport_does_not_echo_stderr(self) -> None:
        with patch.object(native.subprocess, "run") as run:
            run.return_value.returncode = 125
            result = native.run_child(self.root, "invocation_read", {})
        self.assertEqual(result["outcome"], "transport_failure")
        self.assertEqual(result["exit_status"], 125)
        command = run.call_args.args[0]
        self.assertEqual(command[-2:], ["-m", "scripts.capabilities.native_worker"])


if __name__ == "__main__":
    unittest.main()
