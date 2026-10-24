# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# ruff: noqa: PT009, PT027 -- stdlib host controls
"""Native schemas keep original resources and actual receivers keep admission."""

from __future__ import annotations

import ast
import asyncio
import base64
import contextlib
import copy
import hashlib
import io
import json
import os
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import AsyncMock, Mock, patch

from fastmcp import Client
from fastmcp.exceptions import ToolError
from jsonschema import Draft202012Validator
from pydantic import JsonValue

from scripts.capabilities import native, native_contracts, native_worker
from scripts.capabilities.native_schema import JSON_OBJECT, SchemaSet
from scripts.capabilities.server import build

ROOT = Path(__file__).resolve().parents[2]


def object_value(value: JsonValue) -> dict[str, JsonValue]:
    if not isinstance(value, dict):
        raise TypeError("expected a JSON object")
    return value


def list_value(value: JsonValue) -> list[JsonValue]:
    if not isinstance(value, list):
        raise TypeError("expected a JSON array")
    return value


def text_value(value: JsonValue) -> str:
    if not isinstance(value, str):
        raise TypeError("expected an exact text value")
    return value


def at(value: JsonValue, *path: str | int) -> JsonValue:
    for key in path:
        value = (
            object_value(value)[key] if isinstance(key, str) else list_value(value)[key]
        )
    return value


def fingerprint() -> str:
    module = ast.parse(
        (ROOT / "python/pse/contracts/documents/__init__.py").read_text()
    )
    for statement in module.body:
        if isinstance(statement, ast.Assign) and any(
            isinstance(item, ast.Name) and item.id == "SCHEMA_SET_FINGERPRINT"
            for item in statement.targets
        ):
            return ast.literal_eval(statement.value)
    raise AssertionError("generated fingerprint absent")


def schema_fixture(root: Path) -> None:
    """A miniature producer fixture; not a production fallback catalogue."""
    directory = root / "docs/generated/schema"
    directory.mkdir(parents=True)
    shutil.copyfile(
        ROOT / "docs/generated/schema/authoring.schema.json",
        directory / "authoring.schema.json",
    )
    rows = []
    requested = {
        (slug, "input")
        for binding in native.BINDINGS
        for slug in binding.documents.values()
    }
    requested |= {
        (binding.output_document, "output")
        for binding in native.BINDINGS
        if binding.output_document
    }
    for slug, direction in sorted(requested):
        filename = f"{slug}.{direction}.schema.json"
        source = ROOT / "docs/generated/schema" / filename
        shutil.copyfile(source, directory / filename)
        schema = json.loads(source.read_text())
        rows.append(
            {
                "slug": slug,
                "direction": direction,
                "name": schema["title"].removesuffix("Output"),
                "exported_name": schema["title"],
                "wire": "Native",
                "rust_type": "fixture::Owner",
                "registration_source": "fixture",
                "version": 1,
                "schema_path": f"docs/generated/schema/{filename}",
                "schema_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
            }
        )
    (directory / "catalogue.json").write_text(
        json.dumps(
            {
                "version": 1,
                "schema_set_fingerprint": fingerprint(),
                "documents": rows,
                "authoring": {
                    "schema_path": "docs/generated/schema/authoring.schema.json",
                    "sha256": hashlib.sha256(
                        (directory / "authoring.schema.json").read_bytes()
                    ).hexdigest(),
                },
            }
        )
    )


def source_request() -> dict[str, JsonValue]:
    physical = ROOT / "tests/fixtures/packages/physical-primitives"
    physical_documents: dict[str, JsonValue] = {
        str(path.relative_to(physical)): list(path.read_bytes())
        for path in physical.rglob("*")
        if path.is_file()
    }
    manifest = (
        (ROOT / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    manifest = manifest.replace(
        "dependencies = []",
        'dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]',
    )
    return {
        "physical": {"version": 1, "packages": [physical_documents]},
        "documents": {
            "version": 1,
            "packages": [
                {
                    "package.toml": list(manifest.encode()),
                    "models/fixed.pse": list(b"package local {param a:Scalar=1;}"),
                }
            ],
        },
        "resources": {
            "threads": 16,
            "memory_limit_bytes": 16 << 30,
            "max_spill_bytes": 1 << 30,
        },
    }


def preparation_request(root: str) -> dict[str, JsonValue]:
    return {
        "document_version": 1,
        "root": root,
        "instance": root,
        "bindings": {
            "arguments": {},
            "scope": {},
            "facts": [],
            "demand": [],
            "formulation": {"omitted": [], "elastic": {}},
        },
        "limits": {
            "depth": 64,
            "items": 100000,
            "members": 100000,
            "body_occurrences": None,
            "body_slots": None,
        },
        "durable_after": None,
    }


class IntactNativeResultTests(unittest.TestCase):
    """Transport controls; these byte fixtures do not claim scientific eligibility."""

    def test_large_encoded_result_survives_worker_and_host_without_null_replacement(
        self,
    ) -> None:
        original = bytes(range(256)) * 128
        encoded = base64.b64encode(original).decode("ascii")
        self.assertGreater(len(encoded), 16 << 10)
        value = {
            "outcome": "observed",
            "run": "selected-run",
            "attempt": "selected-attempt",
            "relation": "selected-relation",
            "start": 0,
            "end": 1,
            "rows": 1,
            "encoding": "arrow-ipc-stream-base64",
            "bytes": len(original),
            "chunks": [encoded],
        }
        with (
            patch.object(
                native_worker, "worker_request", return_value=("result_read", {})
            ),
            patch.object(native_worker, "execute", return_value=value),
            contextlib.redirect_stdout(io.StringIO()) as output,
        ):
            self.assertEqual(native_worker.main([]), 0)
        worker_answer = json.loads(output.getvalue())
        self.assertEqual(worker_answer, value)
        response = native.intact_result(worker_answer, "native_result_read")
        self.assertTrue(response.complete)
        self.assertEqual(response.omissions, [])
        self.assertEqual(response.payload, value)
        self.assertEqual(
            base64.b64decode(
                "".join(
                    text_value(chunk)
                    for chunk in list_value(response.payload["chunks"])
                ),
                validate=True,
            ),
            original,
        )

    def test_oversized_selected_document_is_refused_as_a_whole(self) -> None:
        document: dict[str, JsonValue] = {
            "outcome": "observed",
            "document": {"required_identity": "selected", "detail": "x" * (121 << 10)},
        }
        response = native.intact_result(document, "native_result_inspect")
        self.assertFalse(response.complete)
        self.assertEqual(response.payload["outcome"], "detail_refused")
        self.assertEqual(response.payload["code"], "pse.native.result_detail_bound")
        self.assertTrue(response.payload["remedy"])
        self.assertNotIn("document", response.payload)
        self.assertNotIn(None, response.payload.values())
        self.assertLess(len(response.model_dump_json().encode()), 128 << 10)

    def test_original_native_incomplete_observation_keeps_its_omission(self) -> None:
        value: dict[str, JsonValue] = {
            "outcome": "incomplete_observation",
            "complete": False,
            "omissions": ["Native selected row window exceeds its bound."],
            "code": "pse.native.result_detail_bound",
            "rows_copied": 0,
        }
        response = native.intact_result(value, "native_result_read")
        self.assertEqual(response.payload, value)
        self.assertFalse(response.complete)
        self.assertEqual(response.omissions, value["omissions"])


class DirectDiagnosticReaderTests(unittest.TestCase):
    def setUp(self) -> None:
        self.pse = Mock(name="pse-reader-fixture")
        self.codec = Mock(name="pse.codec")
        self.diagnostics = Mock(name="pse.diagnostics")
        self.contracts = Mock(name="pse.contracts")
        self.documents = Mock(name="pse.contracts.documents")
        self.pse.SCHEMA_SET_FINGERPRINT = "reader-fixture"
        self.pse.Runtime = Mock(
            side_effect=AssertionError("reader constructed Runtime")
        )
        self.pse.EngineSettings = Mock(
            side_effect=AssertionError("reader constructed settings")
        )
        self.pse.codec = self.codec
        self.pse.diagnostics = self.diagnostics
        self.contracts.documents = self.documents
        self.documents.DiagnosticListRequest = object()
        self.documents.CheckpointIdentity = object()
        self.codec.document_builtins = Mock(side_effect=lambda value: value)
        self.page = {"version": 1, "processes": [], "next": None}
        self.image = {
            "process": "actual-target",
            "locator": {"pid": 123, "start_ticks": 456},
        }
        self.diagnostics.list_processes = Mock(return_value=self.page)
        self.diagnostics.read_checkpoint = Mock(return_value=self.image)
        modules = patch.dict(
            sys.modules,
            {
                "pse": self.pse,
                "pse.codec": self.codec,
                "pse.diagnostics": self.diagnostics,
                "pse.contracts": self.contracts,
                "pse.contracts.documents": self.documents,
            },
        )
        modules.start()
        self.addCleanup(modules.stop)

    def test_discovery_uses_original_reader_and_returns_before_runtime_or_scratch_effects(
        self,
    ) -> None:
        listing = object()
        with patch.object(native_worker, "decoded", return_value=listing) as decoded:
            result = native_worker.execute(
                "diagnostic_processes",
                {
                    "schema_set_fingerprint": "reader-fixture",
                    "request": {"document_version": 1, "after": None, "limit": 16},
                    "directory": "/exact/diagnostics",
                },
                scratch=Path("/not-created-by-read"),
            )
        decoded.assert_called_once_with(
            {"document_version": 1, "after": None, "limit": 16},
            self.documents.DiagnosticListRequest,
        )
        self.diagnostics.list_processes.assert_called_once_with(
            listing, directory="/exact/diagnostics"
        )
        self.assertEqual(result, {"outcome": "observed", "document": self.page})
        self.pse.Runtime.assert_not_called()

    def test_checkpoint_keeps_optional_expected_identity_at_original_owner(
        self,
    ) -> None:
        identity = {
            "document_version": 1,
            "process": "actual-target",
            "locator": {"pid": 123, "start_ticks": 456},
        }
        expected = object()
        with patch.object(native_worker, "decoded", return_value=expected) as decoded:
            result = native_worker.execute(
                "diagnostic_checkpoint",
                {
                    "schema_set_fingerprint": "reader-fixture",
                    "checkpoint": "/exact/checkpoint.json",
                    "expected": identity,
                },
                scratch=Path("/not-created-by-read"),
            )
        decoded.assert_called_once_with(identity, self.documents.CheckpointIdentity)
        self.diagnostics.read_checkpoint.assert_called_once_with(
            Path("/exact/checkpoint.json"), expected=expected
        )
        self.assertEqual(result, {"outcome": "observed", "document": self.image})
        self.diagnostics.read_checkpoint.reset_mock()
        with patch.object(
            native_worker,
            "decoded",
            side_effect=AssertionError("decoded absent identity"),
        ):
            native_worker.execute(
                "diagnostic_checkpoint",
                {
                    "schema_set_fingerprint": "reader-fixture",
                    "checkpoint": "/exact/checkpoint.json",
                },
                scratch=Path("/not-created-by-read"),
            )
        self.diagnostics.read_checkpoint.assert_called_once_with(
            Path("/exact/checkpoint.json"), expected=None
        )
        self.pse.Runtime.assert_not_called()

    def test_checkpoint_owner_refusal_is_not_replaced_with_success_or_fallback(
        self,
    ) -> None:
        self.diagnostics.read_checkpoint.side_effect = ValueError(
            "original identity mismatch"
        )
        with self.assertRaisesRegex(ValueError, "original identity mismatch"):
            native_worker.execute(
                "diagnostic_checkpoint",
                {
                    "schema_set_fingerprint": "reader-fixture",
                    "checkpoint": "/exact/checkpoint.json",
                },
                scratch=Path("/not-created-by-read"),
            )
        self.pse.Runtime.assert_not_called()

    def test_checkpoint_worker_and_host_preserve_all_target_records_or_refuse_whole_detail(
        self,
    ) -> None:
        original = {
            "outcome": "observed",
            "document": {
                "process": "actual-target",
                "operations": [{"sequence": index} for index in range(101)],
            },
        }
        with (
            patch.object(
                native_worker,
                "worker_request",
                return_value=("diagnostic_checkpoint", {}),
            ),
            patch.object(native_worker, "execute", return_value=original),
            contextlib.redirect_stdout(io.StringIO()) as output,
        ):
            self.assertEqual(native_worker.main([]), 0)
        answer = json.loads(output.getvalue())
        self.assertEqual(answer, original)
        self.assertEqual(
            native.intact_result(answer, "native_diagnostic_checkpoint").payload,
            original,
        )
        refused = native.intact_result(
            {"outcome": "observed", "document": {"target": "x" * (121 << 10)}},
            "native_diagnostic_checkpoint",
        )
        self.assertEqual(refused.payload["outcome"], "detail_refused")
        self.assertEqual(refused.payload["code"], "pse.native.observation_detail_bound")
        self.assertNotIn("document", refused.payload)


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
        facts = object_value(at(tool.meta, "pse.native"))
        self.assertEqual(facts["receiver"], "pse.Runtime.project_select")
        self.assertEqual(at(facts, "contracts", "request", "direction"), "input")
        self.assertEqual(
            at(facts, "contracts", "output_document", "direction"), "output"
        )
        self.assertEqual(
            at(
                tool.parameters,
                "$defs",
                "project-select-request",
                "properties",
                "reference",
                "$ref",
            ),
            "#/$defs/ProjectReference",
        )
        self.assertEqual(
            at(
                self.schemas.document("project-select-request"),
                "properties",
                "reference",
                "$ref",
            ),
            "#/$defs/ProjectReference",
        )

    def test_direct_diagnostic_readers_retain_original_schemas_and_nullable_identity(
        self,
    ) -> None:
        for name, slug, receiver in (
            (
                "native_diagnostic_processes",
                "diagnostic-process-page",
                "pse.diagnostics.list_processes",
            ),
            (
                "native_diagnostic_checkpoint",
                "process-checkpoint",
                "pse.diagnostics.read_checkpoint",
            ),
        ):
            with self.subTest(operation=name):
                tool = self.tools[name]
                self.assertEqual(at(tool.meta, "pse.native", "receiver"), receiver)
                self.assertEqual(
                    at(tool.meta, "pse.native", "contracts", "output_document", "slug"),
                    slug,
                )
                annotations = tool.annotations
                if annotations is None:
                    self.fail("Expected explicit native reader annotations")
                self.assertTrue(annotations.read_only_hint)
                original = self.schemas.document(slug, "output")
                definitions = original.pop("$defs", {})
                schema = tool.output_schema
                assert schema is not None
                Draft202012Validator.check_schema(schema)
                self.assertEqual(
                    at(
                        schema,
                        "properties",
                        "payload",
                        "oneOf",
                        0,
                        "properties",
                        "document",
                    ),
                    original,
                )
                self.assertEqual(at(schema, "$defs"), definitions)
        processes = Draft202012Validator(
            self.tools["native_diagnostic_processes"].parameters
        )
        processes.validate(
            {"request": {"document_version": 1, "after": None, "limit": 16}}
        )
        self.assertFalse(processes.is_valid({"request": {"document_version": 999}}))
        checkpoint = Draft202012Validator(
            self.tools["native_diagnostic_checkpoint"].parameters
        )
        checkpoint.validate({"checkpoint": "exact.json"})
        checkpoint.validate({"checkpoint": "exact.json", "expected": None})
        checkpoint.validate(
            {
                "checkpoint": "exact.json",
                "expected": {
                    "document_version": 1,
                    "process": "original-process",
                    "locator": {"pid": 123, "start_ticks": 456, "executable": None},
                },
            }
        )
        self.assertFalse(
            checkpoint.is_valid({"checkpoint": "exact.json", "expected": {}})
        )
        self.assertFalse(
            checkpoint.is_valid(
                {
                    "checkpoint": "exact.json",
                    "expected": {
                        "document_version": 1,
                        "process": "original-process",
                        "locator": {"pid": -1},
                    },
                }
            )
        )

    def test_result_inspection_registration_uses_original_directional_rust_contracts(
        self,
    ) -> None:
        tool = self.tools["native_result_inspect"]
        facts = object_value(at(tool.meta, "pse.native"))
        self.assertEqual(facts["receiver"], "pse.Runtime.inspect_result")
        self.assertEqual(at(facts, "contracts", "request", "direction"), "input")
        self.assertEqual(
            at(facts, "contracts", "request", "slug"), "result-inspection-request"
        )
        self.assertEqual(
            at(facts, "contracts", "output_document", "direction"), "output"
        )
        embedded = copy.deepcopy(
            at(tool.parameters, "$defs", "result-inspection-request")
        )
        embedded = object_value(embedded)
        embedded.pop("$id")
        self.assertEqual(
            embedded, self.schemas.document("result-inspection-request", "input")
        )
        self.assertEqual(
            native_contracts.ResultInspection.model_fields["request"].annotation,
            JsonValue,
        )
        output = tool.output_schema
        self.assertIsNotNone(output)
        if output is None:
            self.fail("inspection output schema absent")
        Draft202012Validator.check_schema(output)
        original = self.schemas.document("result-inspection-document", "output")
        definitions = original.pop("$defs", {})
        self.assertEqual(
            at(output, "properties", "payload", "oneOf", 0, "properties", "document"),
            original,
        )
        self.assertEqual(at(output, "$defs"), definitions)
        refusal = native.intact_result(
            {"outcome": "native_refusal", "code": "owner-issued-code"},
            "native_result_inspect",
        )
        Draft202012Validator(output).validate(refusal.model_dump(mode="json"))
        self.assertNotIn("result", output["properties"])

    def test_inspection_generated_request_and_actual_composed_output_contract(
        self,
    ) -> None:
        request: dict[str, JsonValue] = {
            "document_version": 1,
            "run": "selected-run",
            "attempt": "selected-attempt",
            "relation": "selected-relation",
            "start": 0,
            "end": 1,
            "expected_manifest": None,
            "view": {"kind": "schema", "offset": 0, "limit": 2},
        }
        arguments: dict[str, JsonValue] = {
            "state": "/selected/store",
            "database": "existing",
            "request": request,
        }
        validator = Draft202012Validator(self.tools["native_result_inspect"].parameters)
        validator.validate(arguments)
        for view in (
            {"kind": "invented"},
            {"kind": "schema", "offset": -1, "limit": 2},
            {"kind": "schema", "offset": 0, "limit": 2, "unknown": True},
        ):
            invalid = copy.deepcopy(arguments)
            object_value(at(invalid, "request"))["view"] = JSON_OBJECT.validate_python(
                view
            )
            self.assertFalse(validator.is_valid(invalid))
        document = {
            "document_version": 1,
            "kind": "refused",
            "reason": "selection",
            "guidance": "Use the existing exact Arrow IPC route.",
        }
        observed = {"outcome": "observed", "document": document}

        async def exercise() -> None:
            runtime = build(ROOT)
            tool = runtime.catalogue.tools["native_result_inspect"]
            schema = tool.output_schema
            if schema is None:
                self.fail("Expected the original inspection output schema")
            async with Client(runtime.server) as client:
                await client.call_tool(
                    "capabilities_select", {"capabilities": ["native_result_inspect"]}
                )
                with patch.object(native, "run_child", return_value=observed) as child:
                    result = await client.call_tool("native_result_inspect", arguments)
                self.assertEqual(child.call_args.args[:2], (ROOT, "result_inspect"))
                self.assertEqual(child.call_args.args[2]["request"], request)
                Draft202012Validator(schema).validate(result.structured_content)
                self.assertEqual(at(result.structured_content, "payload"), observed)
                self.assertEqual(
                    at(result.structured_content, "operation"), "native_result_inspect"
                )
                invalid_output = copy.deepcopy(result.structured_content)
                object_value(at(invalid_output, "payload", "document"))["reason"] = (
                    "invented"
                )
                self.assertFalse(Draft202012Validator(schema).is_valid(invalid_output))

        asyncio.run(exercise())

    def test_native_result_tool_preserves_exact_selected_ipc(self) -> None:
        encoded = base64.b64encode(bytes(range(256)) * 128).decode("ascii")
        selected = {
            "outcome": "observed",
            "encoding": "arrow-ipc-stream-base64",
            "chunks": [encoded],
        }
        with patch.object(native, "run_child", return_value=selected):
            result = asyncio.run(
                self.tools["native_result_read"].run(
                    {
                        "state": "/selected/store",
                        "database": "existing",
                        "run": "selected-run",
                        "attempt": "selected-attempt",
                        "relation": "selected-relation",
                        "start": 0,
                        "end": 1,
                    }
                )
            )
        self.assertEqual(at(result.structured_content, "payload"), selected)
        self.assertTrue(at(result.structured_content, "complete"))

    def test_embedded_resources_resolve_tagged_refs_and_reject_unknown_variants(
        self,
    ) -> None:
        validator = Draft202012Validator(self.tools["native_project_select"].parameters)
        data: dict[str, JsonValue] = {
            "state": "/explicit/store",
            "database": "existing",
            "physical": {"version": 1, "packages": []},
            "request": {"version": 1, "reference": {"kind": "bookmark", "key": "mark"}},
        }
        validator.validate(data)
        for bad in (
            {"kind": "latest", "key": "mark"},
            {"kind": "bookmark", "key": "mark", "unknown": 1},
        ):
            refused = copy.deepcopy(data)
            object_value(at(refused, "request"))["reference"] = (
                JSON_OBJECT.validate_python(bad, strict=True)
            )
            self.assertFalse(validator.is_valid(refused))
        object_value(at(data, "request"))["reference"] = {
            "kind": "branch",
            "key": "main",
        }
        validator.validate(data)

    def test_generated_byte_numeric_constraints_survive_envelope(self) -> None:
        validator = Draft202012Validator(self.tools["native_source_admit"].parameters)
        data: dict[str, JsonValue] = {
            "physical": {"version": 1, "packages": []},
            "documents": {"version": 1, "packages": [{"package.toml": [0, 255]}]},
        }
        validator.validate(data)
        for byte in (-1, 256, True, "1"):
            object_value(at(data, "documents", "packages", 0))["package.toml"] = [byte]
            self.assertFalse(validator.is_valid(data))

    def test_recursive_preparation_refs_and_nested_numeric_refusal(self) -> None:
        validator = Draft202012Validator(
            self.tools["native_preparation_discover"].parameters
        )
        request = preparation_request("ab" * 16)
        object_value(at(request, "bindings"))["arguments"] = {
            "nested": {
                "value_kind": "tuple",
                "items": [
                    {
                        "value_kind": "set",
                        "items": [
                            {"value_kind": "number", "bits": 0, "quantity": "cd" * 16}
                        ],
                    }
                ],
            }
        }
        value = {
            "physical": {"version": 1, "packages": []},
            "documents": {"version": 1, "packages": []},
            "request": request,
        }
        validator.validate(value)
        object_value(
            at(request, "bindings", "arguments", "nested", "items", 0, "items", 0)
        )["bits"] = -1
        self.assertFalse(validator.is_valid(value))

    def test_schema_reader_serves_actual_native_refs_and_authoring_grammar(
        self,
    ) -> None:
        tool = self.tools["native_contract_schema"]
        native_result = at(
            asyncio.run(
                tool.run(
                    {
                        "slug": "project-select-request",
                        "direction": "input",
                        "pointer": "/$defs/ProjectReference",
                    }
                )
            ).structured_content,
            "payload",
        )
        self.assertEqual(
            at(native_result, "schema", "oneOf", 0, "properties", "kind", "const"),
            "branch",
        )
        self.assertEqual(
            at(native_result, "root_ref", "arguments"),
            {"slug": "project-select-request", "direction": "input"},
        )
        authoring = self.schemas.read("authoring", None, "/anyOf/4/properties/package")
        self.assertEqual(at(authoring, "identity", "kind"), "authoring_guidance")
        self.assertNotIn("direction", object_value(authoring["identity"]))
        self.assertEqual(
            at(authoring, "schema", "$ref"), "#/$defs/source:package_header:package"
        )
        with patch.object(
            Path,
            "read_bytes",
            side_effect=AssertionError("prepared schema must be reused"),
        ):
            package = self.schemas.read(
                "authoring", None, "/$defs/source:package_header:package"
            )
        self.assertIn("package_id", object_value(at(package, "schema", "properties")))
        with self.assertRaises(ToolError):
            asyncio.run(tool.run({"slug": "authoring", "direction": "input"}))
        with self.assertRaises(ValueError):
            self.schemas.read("project-select-request", "input", "/trailing~")

    def test_actual_composed_mcp_schema_consumer_uses_native_catalogue(self) -> None:
        async def exercise() -> None:
            runtime = build(ROOT)
            async with Client(runtime.server) as client:
                await client.call_tool(
                    "capabilities_select", {"capabilities": ["native_contract_schema"]}
                )
                result = await client.call_tool(
                    "capabilities_invoke_read",
                    {
                        "capability": "native_contract_schema",
                        "arguments": {
                            "slug": "project-select-request",
                            "direction": "input",
                            "pointer": "/$defs/ProjectReference",
                        },
                    },
                )
                payload = at(result.structured_content, "payload")
                self.assertEqual(at(payload, "identity", "wire"), "Native")
                self.assertEqual(
                    at(payload, "identity", "name"), "ProjectSelectRequest"
                )
                self.assertEqual(
                    at(payload, "schema", "oneOf", 1, "properties", "kind", "const"),
                    "bookmark",
                )
                self.assertEqual(
                    at(payload, "root_ref", "arguments"),
                    {"slug": "project-select-request", "direction": "input"},
                )

        asyncio.run(exercise())

    def test_uncached_output_replacement_requires_interface_restart(self) -> None:
        path = self.root / "docs/generated/schema/project-selection.output.schema.json"
        original = path.read_bytes()
        changed = original.replace(
            b'"title": "ProjectSelection"', b'"title": "ProjectSelectioX"'
        )
        self.assertEqual(len(changed), len(original))
        self.assertNotEqual(changed, original)
        before = path.stat()
        path.write_bytes(changed)
        os.utime(path, ns=(before.st_atime_ns, before.st_mtime_ns))
        with self.assertRaisesRegex(ValueError, "restart required"):
            self.schemas.document("project-selection", "output")

    def test_new_catalogue_refuses_old_schema_captured_during_generation(self) -> None:
        catalogue = self.root / "docs/generated/schema/catalogue.json"
        manifest = JSON_OBJECT.validate_json(catalogue.read_bytes(), strict=True)
        row = next(
            object_value(item)
            for item in list_value(manifest["documents"])
            if at(item, "slug") == "project-selection"
            and at(item, "direction") == "output"
        )
        path = self.root / str(row["schema_path"])
        row["schema_sha256"] = hashlib.sha256(path.read_bytes() + b" ").hexdigest()
        catalogue.write_text(json.dumps(manifest))
        schemas = SchemaSet(self.root)
        with self.assertRaisesRegex(ValueError, "catalogue digest; restart required"):
            schemas.document("project-selection", "output")

    def test_new_catalogue_refuses_old_authoring_captured_during_generation(
        self,
    ) -> None:
        catalogue = self.root / "docs/generated/schema/catalogue.json"
        manifest = JSON_OBJECT.validate_json(catalogue.read_bytes(), strict=True)
        row = object_value(manifest["authoring"])
        path = self.root / str(row["schema_path"])
        row["sha256"] = hashlib.sha256(path.read_bytes() + b" ").hexdigest()
        catalogue.write_text(json.dumps(manifest))
        schemas = SchemaSet(self.root)
        with self.assertRaisesRegex(ValueError, "catalogue digest; restart required"):
            schemas.read("authoring", None)

    def test_cached_schema_retains_original_snapshot_after_replacement(self) -> None:
        original = self.schemas.document("project-select-request")
        path = (
            self.root / "docs/generated/schema/project-select-request.input.schema.json"
        )
        replacement = path.with_suffix(".replacement")
        replacement.write_text("{}")
        replacement.replace(path)
        self.assertEqual(self.schemas.document("project-select-request"), original)

    def test_authoring_changed_before_lazy_read_requires_restart(self) -> None:
        path = self.root / "docs/generated/schema/authoring.schema.json"
        path.write_bytes(path.read_bytes() + b" ")
        with self.assertRaisesRegex(ValueError, "restart required"):
            self.schemas.read("authoring", None)

    def test_cached_authoring_keeps_original_digest_and_grammar(self) -> None:
        original = self.schemas.read("authoring", None, "/anyOf/4/properties/package")
        path = self.root / "docs/generated/schema/authoring.schema.json"
        path.write_text("{}")
        self.assertEqual(
            self.schemas.read("authoring", None, "/anyOf/4/properties/package"),
            original,
        )

    def test_host_refusal_is_redacted_and_does_not_invoke_child(self) -> None:
        tool = self.tools["native_project_metadata"]
        with patch.object(
            native, "run_child", side_effect=AssertionError("must not execute")
        ):
            result = asyncio.run(
                tool.run(
                    {
                        "state": "secret",
                        "database": "db",
                        "request": {},
                        "extra": "secret-value",
                    }
                )
            )
        self.assertEqual(
            at(result.structured_content, "payload", "outcome"), "malformed_invocation"
        )
        self.assertNotIn("secret-value", json.dumps(result.structured_content))

    def test_absent_null_and_empty_native_document_are_not_normalized(self) -> None:
        tool = self.tools["native_project_metadata"]
        common = {"state": "/selected", "database": "existing"}
        seen = []

        def receiver(
            _root: Path, _operation: str, arguments: dict[str, object]
        ) -> dict[str, str]:
            seen.append(arguments["request"])
            return {"outcome": "native_refusal"}

        with patch.object(native, "run_child", side_effect=receiver):
            absent = asyncio.run(tool.run(common))
            asyncio.run(tool.run({**common, "request": None}))
            asyncio.run(tool.run({**common, "request": {}}))
        self.assertEqual(
            at(absent.structured_content, "payload", "outcome"), "malformed_invocation"
        )
        self.assertEqual(seen, [None, {}])

    def test_native_incomplete_result_preserves_explicit_omission(self) -> None:
        with patch.object(
            native,
            "run_child",
            return_value={
                "outcome": "incomplete_observation",
                "complete": False,
                "omissions": ["Original detail bound"],
            },
        ):
            result = asyncio.run(
                self.tools["native_invocation_read"].run({"path": "selected"})
            )
        self.assertFalse(at(result.structured_content, "complete"))
        self.assertIn(
            "Original detail bound",
            list_value(at(result.structured_content, "omissions")),
        )

    def test_cancellation_detaches_and_retains_actual_child_observer(self) -> None:

        async def exercise() -> None:
            started = asyncio.Event()
            release = asyncio.Event()

            async def actual(*_args: object) -> dict[str, str]:
                started.set()
                await release.wait()
                return {"outcome": "observed"}

            pending: set[asyncio.Task[dict[str, JsonValue]]] = set()
            with (
                patch.object(native.asyncio, "to_thread", side_effect=actual),
                patch.object(native, "_PENDING", pending),
            ):
                task = asyncio.create_task(
                    self.tools["native_invocation_read"].run({"path": "selected"})
                )
                await started.wait()
                task.cancel()
                with self.assertRaises(asyncio.CancelledError):
                    await task
                self.assertEqual(len(pending), 1)
                child = next(iter(pending))
                self.assertFalse(child.cancelled())
                release.set()
                await child
                await asyncio.sleep(0)
                self.assertFalse(pending)

        asyncio.run(exercise())

    def test_read_only_child_never_requests_native_provider_preparation(self) -> None:
        with patch.object(native.subprocess, "run") as run:
            run.return_value.returncode = 125
            native.run_child(self.root, "invocation_read", {})
        command = run.call_args.args[0]
        self.assertFalse(any(item.startswith("--native") for item in command))
        read_annotations = self.tools["native_invocation_read"].annotations
        assert read_annotations is not None
        self.assertTrue(read_annotations.read_only_hint)
        action_annotations = self.tools["native_preparation_discover"].annotations
        assert action_annotations is not None
        self.assertFalse(action_annotations.read_only_hint)
        with patch.object(native, "run_worker", new_callable=AsyncMock) as run:
            run.return_value = {"outcome": "admitted"}
            result = asyncio.run(
                self.tools["native_source_admit"].run({"physical": {}, "documents": {}})
            )
        self.assertEqual(run.call_args.args[1], native.BINDINGS[0].worker)
        self.assertEqual(
            at(run.call_args.args[2], "schema_set_fingerprint"),
            self.schemas.fingerprint,
        )
        self.assertEqual(
            at(result.structured_content, "payload", "outcome"), "admitted"
        )

    def test_fixed_action_file_protocol_preserves_redacted_prerequisite(self) -> None:
        source = self.root / "request.json"
        target = self.root / "response.json"
        source.write_text(json.dumps({"schema_set_fingerprint": "loaded"}))
        with patch.object(
            native_worker, "execute", side_effect=ImportError("secret-loader-detail")
        ):
            code = native_worker.main(["source_admit", str(source), str(target)])
        self.assertEqual(code, 0)
        result = json.loads(target.read_text())["result"]
        self.assertEqual(result["outcome"], "prerequisite_unavailable")
        self.assertNotIn("secret-loader-detail", target.read_text())

    def test_unknown_or_nonfinite_producer_values_are_never_stringified(self) -> None:
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
