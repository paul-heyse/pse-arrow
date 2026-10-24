# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Reference consumers retain captured bytes after mutable sources change."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import importlib.util
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]


class ScientificConsumerTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        spec = importlib.util.spec_from_file_location(
            "captured_science", ROOT / "python/pse/_scientific_inputs.py"
        )
        assert spec is not None
        assert spec.loader is not None
        self.consumer = importlib.util.module_from_spec(spec)
        modules = patch.dict(sys.modules, {spec.name: self.consumer})
        modules.start()
        self.addCleanup(modules.stop)
        spec.loader.exec_module(self.consumer)
        self.consumer.__dict__["_REFERENCE_ROOT"] = self.root / "packages/reference"
        self.package = self.root / "packages/reference/example"
        self.package.mkdir(parents=True)
        (self.package / "package.toml").write_text("live manifest")
        self.source = self.root / "inputs.json"
        self.value = {
            "schema": "scientific-inputs-v1",
            "files": {
                "packages/reference/example/package.toml": list(b"captured manifest"),
                "packages/reference/example/models/a.pse": list(b"package a {}\n"),
                "packages/reference/diagnostics/test.json": list(b'{"captured":true}'),
            },
            "families": {"authentic": {"packages": ["packages/reference/example"]}},
        }
        self.source.write_text(json.dumps(self.value))
        env = patch.dict(os.environ, {"PSE_SCIENTIFIC_INPUTS": str(self.source)})
        env.start()
        self.addCleanup(env.stop)

    def test_package_and_manifest_use_snapshot_after_live_edit_and_deletion(
        self,
    ) -> None:
        first = self.consumer.package_documents(self.package)
        (self.package / "package.toml").unlink()
        (self.package / "new.pse").write_text("package newly_present {}")
        self.assertEqual(self.consumer.package_documents(self.package), first)
        self.assertEqual(first["package.toml"], b"captured manifest")
        self.assertNotIn("new.pse", first)
        self.assertEqual(self.consumer.family_packages("authentic"), (first,))
        self.assertEqual(
            self.consumer.reference_bytes(self.package / "package.toml"),
            b"captured manifest",
        )
        self.assertEqual(
            self.consumer.reference_bytes(
                self.root / "packages/reference/diagnostics/test.json"
            ),
            b'{"captured":true}',
        )

    def test_missing_capture_and_missing_package_never_fall_back_to_live_sources(
        self,
    ) -> None:
        with (
            patch.dict(os.environ, {}, clear=True),
            self.assertRaisesRegex(ValueError, "explicit preparation"),
        ):
            self.consumer.package_documents(self.package)
        with self.assertRaisesRegex(ValueError, "absent from capture"):
            self.consumer.package_documents(self.package.parent / "unknown")
        ordinary = self.root / "ordinary.txt"
        ordinary.write_bytes(b"ordinary user input")
        self.assertIsNone(self.consumer.package_documents(ordinary.parent))
        self.assertEqual(
            self.consumer.reference_bytes(ordinary), b"ordinary user input"
        )

    def test_declared_package_without_captured_header_refuses_live_fallback(
        self,
    ) -> None:
        del self.value["files"]["packages/reference/example/package.toml"]
        self.source.write_text(json.dumps(self.value))
        with self.assertRaisesRegex(ValueError, "captured package header absent"):
            self.consumer.package_documents(self.package)

    def test_new_capture_changes_runtime_input_without_changing_consumer_source(
        self,
    ) -> None:
        consumer_before = (ROOT / "python/pse/_scientific_inputs.py").read_bytes()
        first = self.consumer.family_packages("authentic")
        self.value["files"]["packages/reference/example/models/a.pse"] = list(
            b"package changed {}\n"
        )
        newer = self.root / "new-inputs.json"
        newer.write_text(json.dumps(self.value))
        with patch.dict(os.environ, {"PSE_SCIENTIFIC_INPUTS": str(newer)}):
            second = self.consumer.family_packages("authentic")
        self.assertNotEqual(first, second)
        self.assertEqual(
            (ROOT / "python/pse/_scientific_inputs.py").read_bytes(), consumer_before
        )

    def test_unrelated_reference_tree_keeps_its_selected_bytes(self) -> None:
        other = self.root / "other/packages/reference/example"
        other.mkdir(parents=True)
        header = other / "package.toml"
        header.write_bytes(b"independent user manifest")
        self.assertIsNone(self.consumer.package_documents(other))
        self.assertEqual(
            self.consumer.reference_bytes(header), b"independent user manifest"
        )
        self.assertEqual(
            self.consumer.reference_bytes(self.package / "package.toml"),
            b"captured manifest",
        )
