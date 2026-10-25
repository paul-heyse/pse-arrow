# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Original capture schemas and capture-only worker cancellation ownership."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import copy
import signal
import subprocess
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace
from typing import TYPE_CHECKING, Self
from unittest.mock import patch

from jsonschema import Draft202012Validator
from pydantic import ValidationError

from scripts.capabilities import (
    native,
    native_worker,
    telemetry_profile_worker,
    telemetry_profiles,
)
from scripts.capabilities.native_schema import SchemaSet
from scripts.capabilities.workers import WORKERS

ROOT = Path(__file__).resolve().parents[2]

if TYPE_CHECKING:
    from pydantic import JsonValue


def suspect() -> dict:
    return {
        "request": {
            "signal": "metrics",
            "metric": "pse_build_records",
            "service_instance_id": "observed-service",
            "start_time_us": 1,
            "end_time_us": 2,
        },
        "status": "available",
        "source": {
            "configuration_id": "configuration",
            "backend_version": "1.1.2",
            "backend_process": {"pid": 11, "start": "backend-incarnation"},
            "stream": "pse_build_records",
            "signal": "metrics",
            "start_time_us": 1,
            "end_time_us": 2,
            "retrieved_at_ns": 2001,
        },
        "records": [{"value": 3}],
        "returned_rows": 1,
    }


def envelope() -> dict:
    return {
        "request": {
            "version": 1,
            "checkpoint": "/selected/native.json",
            "operation": {"process_instance": [1, 2], "sequence": 2},
            "kind": "Timeline",
            "output": "/fresh/capture",
            "duration": {"secs": 1, "nanos": 0},
        },
        "expected": {
            "document_version": 1,
            "process": "native-incarnation",
            "locator": {"pid": 123, "start_ticks": 456, "executable": None},
        },
        "suspect": suspect(),
    }


class ProfileBindingTests(unittest.TestCase):
    def test_factory_uses_original_native_documents_and_attached_worker(self) -> None:
        with patch.dict(sys.modules, {"pse": None}):
            specs, tools = telemetry_profiles.bindings(ROOT)
        tool = tools["telemetry_profile_capture"]
        schemas = SchemaSet(ROOT)
        self.assertIsInstance(tool, native.NativeTool)
        for field, slug in (
            ("request", "capture-request"),
            ("expected", "checkpoint-identity"),
        ):
            embedded = copy.deepcopy(tool.parameters["$defs"][slug])
            embedded.pop("$id")
            self.assertEqual(embedded, schemas.document(slug))
            self.assertIn(field, tool.parameters["required"])
        self.assertIsNotNone(tool.output_schema)
        Draft202012Validator.check_schema(tool.output_schema)
        output = tool.output_schema["properties"]["payload"]["oneOf"][0]["properties"]
        original = schemas.document("capture-document", "output")
        original.pop("$defs", {})
        self.assertEqual(output["document"], original)
        self.assertIn("telemetry_context", output)
        Draft202012Validator(tool.parameters).validate(envelope())
        annotations = tool.annotations
        assert annotations is not None
        self.assertFalse(annotations.read_only_hint)
        self.assertFalse(annotations.idempotent_hint)
        self.assertEqual(specs[0].worker_calls[0].lifetime, "attached")
        self.assertEqual(specs[0].requirements[0].kind, "python-extension")
        self.assertEqual(specs[0].requirements[0].mode, "native")
        self.assertIn("telemetry_profile_capture", WORKERS["native"].operations)

    def test_suspect_without_original_source_or_checkpoint_identity_refuses(
        self,
    ) -> None:
        for changes in ({"source": None}, {"status": "stale"}, {"records": []}):
            request = envelope()
            request["suspect"].update(changes)
            with self.subTest(changes=changes), self.assertRaises(ValidationError):
                telemetry_profiles.ProfileCaptureRequest.model_validate(request)
        with self.assertRaises(ValidationError):
            telemetry_profiles.ProfileCaptureRequest.model_validate(
                {**envelope(), "expected": None}
            )

    def test_oversized_capture_refuses_whole_document_with_capture_remedy(self) -> None:
        errors: list[JsonValue] = ["x" * (121 * 1024)]
        original: dict[str, JsonValue] = {
            "outcome": "observed",
            "document": {"errors": errors},
        }
        for name in ("telemetry_profile_capture", "native_telemetry_profile_capture"):
            result = native.intact_result(original, name)
            self.assertEqual(result.payload["outcome"], "detail_refused")
            self.assertNotIn("document", result.payload)
            remedy = result.payload["remedy"]
            assert isinstance(remedy, str)
            self.assertIn("CaptureDocument", remedy)
            self.assertNotIn("IPC", remedy)
        message = errors[0]
        assert isinstance(message, str)
        self.assertEqual(len(message), 121 * 1024)


class CaptureBridgeTests(unittest.TestCase):
    def invoke(
        self, *, mismatch: bool = False, interrupt: bool = False
    ) -> tuple[dict, list[str], subprocess.Popen]:
        capture_child = subprocess.Popen(
            [sys.executable, "-c", "import time; time.sleep(60)"]
        )

        def cleanup_capture() -> None:
            if capture_child.poll() is None:
                capture_child.terminate()
            capture_child.wait(timeout=5)

        self.addCleanup(cleanup_capture)
        science = subprocess.Popen(
            [sys.executable, "-c", "import time; time.sleep(60)"]
        )

        def cleanup_science() -> None:
            if science.poll() is None:
                science.terminate()
            science.wait(timeout=5)

        self.addCleanup(cleanup_science)
        calls = []
        locator = SimpleNamespace(pid=123, start_ticks=456)
        expected = SimpleNamespace(process="native-incarnation", locator=locator)
        request = SimpleNamespace(
            checkpoint="/selected/native.json", operation="original-operation"
        )
        target = SimpleNamespace(
            process="changed-incarnation" if mismatch else "native-incarnation",
            locator=locator,
        )
        document = SimpleNamespace(targets=[target])

        class Session:
            def __enter__(self) -> Self:
                return self

            def snapshot(self) -> SimpleNamespace:
                calls.append("snapshot")
                return document

            def wait(self) -> SimpleNamespace:
                calls.append("wait")
                if interrupt:
                    signal.raise_signal(signal.SIGTERM)
                return document

            def stop(self) -> SimpleNamespace:
                calls.append("stop")
                if capture_child.poll() is None:
                    capture_child.terminate()
                capture_child.wait(timeout=5)
                return document

            def __exit__(self, *_exception: object) -> None:
                self.stop()

        def preflight(path: Path, *, expected: SimpleNamespace) -> None:
            self.assertEqual(path, Path("/selected/native.json"))
            self.assertEqual(expected.process, "native-incarnation")
            calls.append("preflight")

        def start(received: SimpleNamespace) -> Session:
            self.assertIs(received, request)
            calls.append("capture")
            return Session()

        fake = types.ModuleType("pse")
        fake.__dict__.update(
            codec=SimpleNamespace(
                document_builtins=lambda value: {"original_capture": value is document}
            ),
            diagnostics=SimpleNamespace(read_checkpoint=preflight, capture=start),
        )
        contracts = types.ModuleType("pse.contracts")
        capture_type, identity_type = (
            type("CaptureRequest", (), {}),
            type("CheckpointIdentity", (), {}),
        )
        contracts.__dict__["documents"] = SimpleNamespace(
            CaptureRequest=capture_type, CheckpointIdentity=identity_type
        )
        previous = signal.getsignal(signal.SIGTERM)
        with (
            patch.dict(sys.modules, {"pse": fake, "pse.contracts": contracts}),
            patch.object(native_worker, "decoded", side_effect=[request, expected]),
        ):
            if mismatch:
                with self.assertRaises(ValueError):
                    telemetry_profile_worker.capture(envelope())
                result = {}
            else:
                result = telemetry_profile_worker.capture(envelope())
        self.assertIs(signal.getsignal(signal.SIGTERM), previous)
        self.assertIsNotNone(capture_child.poll())
        self.assertIsNone(science.poll())
        return result, calls, science

    def test_exact_target_capture_preserves_original_document_and_source(self) -> None:
        result, calls, _science = self.invoke()
        self.assertEqual(calls, ["preflight", "capture", "snapshot", "wait", "stop"])
        self.assertEqual(result["document"], {"original_capture": True})
        for name, value in suspect()["source"].items():
            self.assertEqual(result["telemetry_context"]["source"][name], value)
        self.assertFalse(result["telemetry_context"]["atomic_identity_admission"])

    def test_wrong_capture_target_drains_only_capture(self) -> None:
        _result, calls, _science = self.invoke(mismatch=True)
        self.assertEqual(calls, ["preflight", "capture", "snapshot", "stop"])

    def test_helper_sigterm_stops_and_reaps_capture_child_leaves_science_alive(
        self,
    ) -> None:
        result, calls, _science = self.invoke(interrupt=True)
        self.assertEqual(
            calls, ["preflight", "capture", "snapshot", "wait", "stop", "stop"]
        )
        self.assertEqual(result["outcome"], "observed")


if __name__ == "__main__":
    unittest.main()
