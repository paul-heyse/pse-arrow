# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected toolkit transport and source-input provenance controls."""
# ruff: noqa: PT009, PT027 -- stdlib source provenance controls

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import surrealkit as tool
from scripts import surrealkit_source as source


class ToolkitSourceTests(unittest.TestCase):
    def test_manifest_selects_released_remote_sdk_without_embedded_engine(self) -> None:
        upstream = (
            "surrealdb = { version = '3.3.0', features = ['protocol-http', 'jwks'] }\n"
            "surrealdb-types = { version = '3.3.0' }\n"
            "panic = 'abort'"
        )
        result = source.manifest(upstream)
        self.assertIn('features = ["protocol-grpc", "rustls"]', result)
        self.assertIn("default-features = false", result)
        self.assertNotIn("path =", result)
        self.assertIn('version = "=3.3.4"', result)
        self.assertNotIn("protocol-http", result)
        self.assertNotIn("jwks", result)
        self.assertIn("panic = 'unwind'", result)

    def test_changed_upstream_integration_refuses(self) -> None:
        with self.assertRaises(source.BuildError):
            source.manifest("surrealdb = 'unknown'")

    def test_unconsumed_vendor_source_does_not_change_toolkit_inputs(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            sdk = root / "vendor/surrealdb/src"
            sdk.mkdir(parents=True)
            code = sdk / "lib.rs"
            code.write_text("old implementation")
            declared = {"source_revision": "a" * 40}
            with patch.object(source, "ROOT", root):
                before = source.inputs(declared)
                code.write_text("changed implementation")
                self.assertEqual(before, source.inputs(declared))
                (root / "Cargo.toml").write_text("changed released SDK selection")
                self.assertNotEqual(before, source.inputs(declared))

    def test_new_acquire_preserves_old_publication_and_current_verify_refuses_stale(
        self,
    ) -> None:
        declared = {
            "version": "1.0.0-beta.6",
            "repository": "surrealdb/surrealkit",
            "source_revision": "a" * 40,
        }
        identity = ["a" * 64]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            built = root / "built"
            built.write_bytes(b"first binary")

            def build(
                directory: Path, selection: dict[str, str], *, expected_inputs: str
            ) -> Path:
                return source.publish(
                    directory,
                    selection,
                    expected_inputs,
                    built,
                    {"source_build": True, "release_build_locked": True},
                )

            with (
                patch.object(tool, "selection", return_value=declared),
                patch.object(
                    source, "inputs", side_effect=lambda _declared: identity[0]
                ),
                patch.object(source, "build", side_effect=build) as builder,
                patch.object(tool.platform, "system", return_value="Linux"),
                patch.object(tool.platform, "machine", return_value="x86_64"),
                patch.object(
                    tool.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess(
                        [], 0, "surrealkit 1.0.0-beta.6\n", ""
                    ),
                ),
            ):
                old = tool.acquire(root / "tools")
                original = old.read_bytes()
                original_receipt = (old.parent / "release.json").read_bytes()
                saved = json.loads(original_receipt)
                identity[0] = "b" * 64
                built.write_bytes(b"second binary")
                current = tool.acquire(root / "tools")
                self.assertNotEqual(old, current)
                self.assertEqual(old.read_bytes(), original)
                self.assertEqual(
                    (old.parent / "release.json").read_bytes(), original_receipt
                )
                self.assertEqual(tool.acquire(root / "tools"), current)
                self.assertEqual(builder.call_count, 2)
                with self.assertRaisesRegex(tool.ToolError, "declared release"):
                    tool.verify(old)
                self.assertEqual(tool.verify_saved(old, saved), saved)
                with self.assertRaisesRegex(tool.ToolError, "submitted rollout"):
                    tool.verify_saved(current, saved)
                with self.assertRaises(source.BuildError):
                    source.publish(
                        old.parent.parent.parent, declared, "a" * 64, built, {}
                    )
                self.assertEqual(old.read_bytes(), original)

    def test_admin_listener_is_same_port_but_tool_calls_use_grpc(self) -> None:
        self.assertEqual(
            tool.grpc_endpoint("http://127.0.0.1:18240"), "grpc://127.0.0.1:18240"
        )
        self.assertEqual(
            tool.grpc_endpoint("grpc://127.0.0.1:18240"), "grpc://127.0.0.1:18240"
        )
        for value in (
            "ws://127.0.0.1:18240",
            "http://remote:18240",
            "http://user@127.0.0.1:18240",
            "http://127.0.0.1:18240/sql",
        ):
            with self.subTest(value=value), self.assertRaises(tool.ToolError):
                tool.grpc_endpoint(value)
