# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# ruff: noqa: PT009, PT027 -- stdlib host controls
"""Native schemas keep original resources and actual receivers keep admission."""

from __future__ import annotations

import ast
import asyncio
import copy
import hashlib
import json
import os
import shutil
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import AsyncMock, patch

from fastmcp import Client
from fastmcp.exceptions import ToolError
from jsonschema import Draft202012Validator

from scripts.capabilities import native, native_worker
from scripts.capabilities.native_schema import JSON_OBJECT, SchemaSet
from scripts.capabilities.server import build

if TYPE_CHECKING:
    from pydantic import JsonValue

ROOT = Path(__file__).resolve().parents[2]


def object_value(value: JsonValue) -> dict[str, JsonValue]:
    if not isinstance(value, dict):
        raise TypeError("expected a JSON object")
    return value


def list_value(value: JsonValue) -> list[JsonValue]:
    if not isinstance(value, list):
        raise TypeError("expected a JSON array")
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
        self.assertEqual(run.call_args.args[:3], (self.root, "native", "source_admit"))
        self.assertEqual(
            at(run.call_args.args[3], "schema_set_fingerprint"),
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
