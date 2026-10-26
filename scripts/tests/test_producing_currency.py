# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Changed-input, membership and artifact-race controls for producing witnesses."""
# ruff: noqa: PT009, PT027 -- stdlib host-boundary controls

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import producing_currency, python_extension


class ProducingCurrencyTests(unittest.TestCase):
    def test_observer_alias_is_historical_but_helper_source_and_toolchain_are_current(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            alias, source, compiler = root / "target/debug/xtask", root / "xtask/src/producing_inputs.rs", root / "toolchain/rustc"
            interpreter, venv = root / "venv/bin/python", root / "venv/pyvenv.cfg"
            for path in (alias, source, compiler, interpreter, venv):
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("selected preparation input")
            witness = producing_currency.Witness()
            observed = {"inputs": {"version": 1, "scope": "ordinary-cargo-inputs", "files": [str(source)], "directories": [], "environment": []},
                        "invocation": {"observer": {"artifact": witness.file(alias)}}}
            context = {"tools": {"toolchain-rustc": witness.file(compiler)}, "configuration": {}, "generations": {},
                       "interpreter": {"binary": witness.file(interpreter), "venv": witness.file(venv), "configuration": {"files": []}}}
            with patch.object(python_extension, "ROOT", root), patch.object(python_extension, "PRODUCING_WITNESS", witness):
                selection = python_extension.producing_selection(observed, context)
                self.assertNotIn(str(alias), selection["files"])
                self.assertIn(str(source), selection["files"])
                self.assertIn(str(compiler), selection["files"])
                original = python_extension.producing_inputs(selection, {}, mode="dev", profile="dev", validated=True)
                alias.write_text("ordinary codegen helper overwrite")
                self.assertEqual(original, python_extension.producing_inputs(selection, {}, mode="dev", profile="dev", validated=True))
                source.write_text("changed helper implementation")
                self.assertNotEqual(original, python_extension.producing_inputs(selection, {}, mode="dev", profile="dev", validated=True))
                source.write_text("selected preparation input")
                self.assertEqual(original, python_extension.producing_inputs(selection, {}, mode="dev", profile="dev", validated=True))
                compiler.write_text("changed selected toolchain")
                self.assertNotEqual(original, python_extension.producing_inputs(selection, {}, mode="dev", profile="dev", validated=True))

    def test_consumed_change_rekeys_but_unrelated_workspace_file_does_not(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            consumed, unrelated = root / "consumed.rs", root / "unrelated.rs"
            consumed.write_text("original")
            unrelated.write_text("original")
            selection = {"version": 1, "scope": "ordinary-cargo-inputs",
                         "files": [str(consumed)], "directories": [], "environment": ["SELECTED"]}
            witness = producing_currency.Witness()
            with patch.object(hashlib, "file_digest", wraps=hashlib.file_digest) as digest:
                original = witness.inputs(selection, {"SELECTED": "one"})
                unrelated.write_text("changed")
                self.assertEqual(original, witness.inputs(selection, {"SELECTED": "one", "SESSION": "other"}))
                self.assertEqual(digest.call_count, 1)
                consumed.write_text("changed")
                self.assertNotEqual(original, witness.inputs(selection, {"SELECTED": "one"}))
                self.assertEqual(digest.call_count, 2)
                self.assertNotEqual(original, witness.inputs(selection, {"SELECTED": "two"}))

    def test_package_membership_deletion_nested_changes_and_missing_input(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package = root / "package"
            package.mkdir()
            nested = package / "nested"
            nested.mkdir()
            member = nested / "input"
            member.write_text("first")
            witness = producing_currency.Witness()
            original = witness.tree(package)
            self.assertEqual(original, json.loads(json.dumps(original)))
            member.write_text("other")
            self.assertNotEqual(original, witness.tree(package))
            original = witness.tree(package)
            (nested / "new").write_text("new member")
            self.assertNotEqual(original, witness.tree(package))
            original = witness.tree(package)
            member.unlink()
            self.assertNotEqual(original, witness.tree(package))
            self.assertIsNone(witness.file(member)["sha256"])

    def test_native_or_configuration_content_and_selection_link_are_inputs(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            native = root / "native-receipt"
            native.write_text("managed generation one")
            selected = root / "selected"
            selected.symlink_to(native.name)
            witness = producing_currency.Witness()
            original = witness.file(selected)
            native.write_text("managed generation two")
            self.assertNotEqual(original, witness.file(selected))
            selected.unlink()
            selected.symlink_to("missing")
            self.assertIsNone(witness.file(selected)["sha256"])

    def test_actual_artifact_replacement_during_digest_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            artifact = Path(temporary) / "image"
            artifact.write_bytes(b"prepared image")
            actual_digest = hashlib.file_digest
            def replace(stream: object, algorithm: str) -> object:
                result = actual_digest(stream, algorithm)
                replacement = artifact.with_suffix(".new")
                replacement.write_bytes(b"replacement image")
                replacement.replace(artifact)
                return result
            with patch.object(hashlib, "file_digest", side_effect=replace):
                with self.assertRaisesRegex(ValueError, "changed during"):
                    producing_currency.Witness().file(artifact)

    def test_directory_alias_and_unknown_selection_refuse(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "alias").symlink_to(root, target_is_directory=True)
            witness = producing_currency.Witness()
            with self.assertRaisesRegex(ValueError, "symlinks"):
                witness.tree(root)
            with self.assertRaisesRegex(ValueError, "unsupported"):
                witness.inputs({"version": 9}, {})

    def test_receipt_currency_uses_witness_without_workspace_hash_or_second_import(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package = root / "python/pse"
            package.mkdir(parents=True)
            binary = package / "_native.so"
            binary.write_bytes(b"installed image")
            source = root / "producing/src.rs"
            source.parent.mkdir()
            source.write_text("consumed")
            unrelated = root / "unrelated.rs"
            unrelated.write_text("unrelated")
            environment = {"VIRTUAL_ENV": str(root / ".venv"), "UV_PROJECT_ENVIRONMENT": str(root / ".venv")}
            selection = {"version": 1, "scope": "ordinary-cargo-inputs", "files": [str(source)],
                         "directories": [], "environment": ["SELECTOR"], "generations": {}}
            with (
                patch.object(python_extension, "ROOT", root),
                patch.object(python_extension, "PRODUCING_WITNESS", producing_currency.Witness()),
                patch.object(python_extension, "capture", side_effect=AssertionError("second interpreter")),
                patch.object(python_extension, "preparation_inputs", side_effect=AssertionError("broad source scan")),
            ):
                inputs = python_extension.producing_inputs(selection, environment, mode="dev", profile="dev", validated=True)
                artifact = {"binary": str(binary), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "features": ["force-validate"]}
                python_extension.publish_receipt(python_extension.stage_receipt("extension", environment), {
                    "inputs": inputs, "artifact": artifact,
                    "producing": {"selection": selection, "association": {"transformation": {"kind": "identical-bytes"}}},
                })
                current = python_extension.currency_check(environment, mode="dev")
                self.assertEqual(current.state, "current")
                unrelated.write_text("changed")
                self.assertEqual(python_extension.currency_check(environment, mode="dev"), current)
                self.assertEqual(python_extension.currency_check({**environment, "SELECTOR": "changed"}, mode="dev").state, "stale")
                self.assertEqual(python_extension.currency_check({**environment, "RUSTFLAGS": "new"}, mode="dev").state, "stale")
                binary.write_bytes(b"replaced installed image")
                self.assertEqual(python_extension.currency_check(environment, mode="dev").cause, "artifact-changed")
                source.write_text("changed")
                self.assertEqual(python_extension.currency_check(environment, mode="dev").cause, "inputs-changed")

    def test_actual_maturin_rpath_association_refuses_corruption_and_unrelated_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            original, installed = root / "raw.so", root / "installed.so"
            shutil.copyfile("/usr/bin/true", original)
            subprocess.run(["patchelf", "--set-rpath", "/original", str(original)], check=True)
            shutil.copyfile(original, installed)
            subprocess.run(["patchelf", "--remove-rpath", str(installed)], check=True)
            subprocess.run(["patchelf", "--force-rpath", "--set-rpath", "/original:/native", str(installed)], check=True)
            selection = {"root_artifacts": [{"filenames": [str(original)]}],
                         "files": [], "build_scripts": [{"reason": "build-script-executed", "linked_libs": ["dylib=fixture"], "linked_paths": ["native=/native"]}],
                         "invocation": {"artifacts": {str(original): producing_currency.Witness().file(original)}}}
            artifact = {"binary": str(installed), "sha256": hashlib.sha256(installed.read_bytes()).hexdigest(), "features": []}
            association = python_extension.installed_association(selection, artifact, os.environ)
            self.assertIsNotNone(association)
            self.assertEqual(association["transformation"]["kind"], "maturin-rpath-only")
            installed.write_bytes(installed.read_bytes() + b"unrelated mutation")
            artifact["sha256"] = hashlib.sha256(installed.read_bytes()).hexdigest()
            self.assertIsNone(python_extension.installed_association(selection, artifact, os.environ))
            shutil.copyfile("/usr/bin/false", original)
            self.assertIsNone(python_extension.installed_association(selection, artifact, os.environ))


if __name__ == "__main__":
    unittest.main()
