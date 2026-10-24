# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-build admission controls; no network, compiler or selected state."""
# ruff: noqa: PT009, PT027 -- stdlib source admission controls

from __future__ import annotations

import copy
import importlib.util
import io
import json
import tempfile
import unittest
from pathlib import Path
from types import ModuleType
from unittest.mock import patch

from scripts import surreal_server as server
from scripts import surreal_source as source
from scripts.tests.server_receipt_fixture import fixture_context, fixture_receipt


class SourceAdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        selected = patch.object(
            source,
            "build_context",
            return_value=(fixture_context(), {"fixture": True}, {}),
        )
        selected.start()
        self.addCleanup(selected.stop)

    def test_changed_binary_is_refused_before_rebuild_or_execution(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch) / source.installation_name()
            directory.mkdir()
            binary = directory / "source/target/release/surreal"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"changed executable")
            (binary.parents[2] / "Cargo.lock").write_text("fixture lock")
            (directory / "release.json").write_text(
                json.dumps(
                    {
                        **fixture_receipt(Path(scratch)),
                        "lockfile_sha256": source.digest(
                            binary.parents[2] / "Cargo.lock"
                        ),
                        "binary": str(binary),
                        "binary_sha256": "0" * 64,
                    }
                )
            )
            with (
                patch.object(source, "verify_source"),
                patch.object(source, "command") as command,
                self.assertRaisesRegex(source.BuildError, "binary differs"),
            ):
                source.install(directory)
            command.assert_not_called()

    def test_changed_patch_inputs_refuse_cached_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch) / source.installation_name()
            directory.mkdir()
            (directory / "release.json").write_text(json.dumps({"inputs": {}}))
            with (
                patch.object(source, "verify_source") as verify,
                patch.object(source, "command") as command,
                self.assertRaisesRegex(source.BuildError, "inputs changed"),
            ):
                source.install(directory)
            verify.assert_not_called()
            command.assert_not_called()

    def test_unowned_source_modification_refuses_before_compile(self) -> None:
        with (
            patch.object(
                source, "command", side_effect=[source.REVISION + "\n", "changed.rs\n"]
            ),
            self.assertRaisesRegex(source.BuildError, "unowned modifications"),
        ):
            source.verify_source(Path("/unopened-control-source"))

    def test_dependency_checksum_refuses_before_extraction(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch)
            with (
                patch.object(
                    source.urllib.request,
                    "urlopen",
                    return_value=io.BytesIO(b"wrong source archive"),
                ),
                patch.object(source.tarfile, "open") as extract,
                self.assertRaisesRegex(source.BuildError, "SHA256"),
            ):
                source.acquire_diskann(directory / "source", directory)
            extract.assert_not_called()
            self.assertFalse((directory / "source").exists())

    def test_surrealism_feature_and_locked_cli_patch_are_build_inputs(self) -> None:
        self.assertIn("surrealism", source.FEATURES.split(","))
        self.assertEqual(source.inputs()["surrealism_sdk_version"], "0.6.4")
        prior = source.installation_name()
        with patch.object(source, "FEATURES", "storage-mem,cli,mcp"):
            self.assertNotEqual(source.installation_name(), prior)

    def test_changed_patches_select_new_package_but_compiler_context_does_not(self) -> None:
        prior = source.installation_name()
        actual_digest = source.digest
        for selected in (source.INPUTS / name for name in source.PATCHES):

            def changed(path: Path, selected: Path = selected) -> str:
                return "0" * 64 if path == selected else actual_digest(path)

            with patch.object(source, "digest", side_effect=changed):
                self.assertNotEqual(source.installation_name(), prior)
        with patch.object(
            source,
            "build_context",
            return_value=(
                {"schema": "changed-effective-context"},
                {"fixture": True},
                {},
            ),
        ):
            self.assertEqual(source.installation_name(), prior)

    def test_complete_receipt_uses_effective_key_and_preserves_original_raw_provenance(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / source.installation_name()
            directory.mkdir()
            binary = directory / "source/target/release/surreal"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"admitted")
            lock = directory / "source/Cargo.lock"
            lock.write_text("lock")
            receipt = {
                **fixture_receipt(Path(temporary)),
                "binary": str(binary),
                "binary_sha256": source.digest(binary),
                "lockfile_sha256": source.digest(lock),
            }
            (directory / "release.json").write_text(json.dumps(receipt))
            with (
                patch.object(source, "verify_source"),
                patch.object(source, "command") as execute,
            ):
                self.assertEqual(source.install(directory), receipt)
            execute.assert_not_called()
            incomplete = {
                key: value for key, value in receipt.items() if key != "schema"
            }
            self.assertFalse(source.admitted_receipt(incomplete))

    def test_passive_receipt_checks_do_not_probe_compilers_or_native_trees(
        self,
    ) -> None:
        receipt = fixture_receipt()
        with patch.object(
            source, "build_context", side_effect=AssertionError("passive compiler scan")
        ):
            self.assertTrue(source.admitted_receipt(receipt))
        with patch.object(source, "build_context", side_effect=AssertionError("launch compiler scan")):
            self.assertTrue(source.admitted_receipt(receipt))

    def test_package_selection_does_not_inspect_caller_context(self) -> None:
        prior = source.installation_name()
        with patch.object(source, "build_context", side_effect=AssertionError("caller compiler scan")):
            self.assertEqual(source.installation_name(), prior)

    def completed_package(self, directory: Path) -> tuple[dict, Path]:
        binary = directory / "source/target/release/surreal"
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"verified installed executable")
        lock = directory / "source/Cargo.lock"
        lock.write_text("locked dependencies")
        receipt = {
            **fixture_receipt(directory.parent),
            "binary": str(binary),
            "binary_sha256": source.digest(binary),
            "lockfile_sha256": source.digest(lock),
        }
        path = directory / "release.json"
        path.write_text(json.dumps(receipt))
        return receipt, path

    def test_existing_completed_artifact_is_adopted_without_rebuild_or_relocation(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            receipt, path = self.completed_package(root / "v3.3.4-mx1.0.0-effective-v1-original")
            original = path.read_bytes()
            directory = root / source.installation_name()
            directory.mkdir()
            with (
                patch.dict(source.os.environ, {"PATH": "/different/caller/bin"}),
                patch.object(source, "verify_source", side_effect=AssertionError("completed source probe")),
                patch.object(source, "build_context", side_effect=AssertionError("compiler scan")),
                patch.object(source, "command", side_effect=AssertionError("compiler or executable")),
            ):
                self.assertEqual(source.require_installed(directory), receipt)
                self.assertEqual(source.install(directory), receipt)
            self.assertEqual(path.read_bytes(), original)
            self.assertFalse((directory / "source").exists())
            self.assertTrue((directory / "selection.json").exists())

    def test_selected_artifact_survives_caller_worktree_and_cache_environment_changes(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            directory = root / source.installation_name()
            receipt, path = self.completed_package(directory)
            original = path.read_bytes()
            with patch.object(source, "verify_source"):
                source.require_installed(directory)
            for variable in ("PATH", "LD_LIBRARY_PATH", "RUSTC_WRAPPER", "SCCACHE_CONF", "CARGO_HOME"):
                with (
                    self.subTest(variable=variable),
                    patch.dict(source.os.environ, {variable: "/different/worktree/value"}),
                    patch.object(source, "ROOT", root / "another-worktree"),
                    patch.object(source, "build_context", side_effect=AssertionError("compiler scan")),
                    patch.object(source.installation_context, "configuration", side_effect=AssertionError("configuration scan")),
                    patch.object(source, "command", side_effect=AssertionError("compiler or executable")),
                ):
                    self.assertEqual(source.require_installed(directory), receipt)
                    self.assertTrue(source.admitted_receipt(receipt))
            self.assertEqual(path.read_bytes(), original)

    def test_ordinary_missing_package_never_compiles(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            with (
                patch.object(source, "install", side_effect=AssertionError("implicit compilation")),
                patch.object(source, "build_context", side_effect=AssertionError("compiler scan")),
                self.assertRaisesRegex(server.SupervisorError, "explicitly run build-server"),
            ):
                server.install(None, Path(scratch))

    def test_published_selection_refuses_corruption_without_fallback_or_compilation(self) -> None:
        for change in ("binary", "lock", "receipt", "selection", "path"):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as scratch:
                root = Path(scratch)
                directory = root / source.installation_name()
                receipt, path = self.completed_package(directory)
                with patch.object(source, "verify_source"):
                    source.require_installed(directory)
                # A different valid artifact must not mask corruption of the selection.
                self.completed_package(root / "v3.3.4-mx1.0.0-effective-v1-other")
                selection = directory / "selection.json"
                if change == "binary":
                    Path(receipt["binary"]).write_bytes(b"changed")
                elif change == "lock":
                    (directory / "source/Cargo.lock").write_text("changed")
                elif change == "receipt":
                    path.write_text("{}")
                elif change == "selection":
                    selection.write_text("[]")
                else:
                    value = json.loads(selection.read_text())
                    value["release"] = "../release.json"
                    selection.write_text(json.dumps(value))
                with (
                    patch.object(source, "command", side_effect=AssertionError("compiler or executable")),
                    patch.object(source, "build_context", side_effect=AssertionError("compiler scan")),
                    self.assertRaises(source.BuildError),
                ):
                    source.install(directory)

    def test_package_patch_change_refuses_old_artifact_without_implicit_build(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            self.completed_package(root / "v3.3.4-mx1.0.0-effective-v1-old")
            with (
                patch.object(source, "FEATURES", "different-package-feature"),
                patch.object(source, "command", side_effect=AssertionError("compiler or executable")),
                patch.object(source, "build_context", side_effect=AssertionError("compiler scan")),
                self.assertRaisesRegex(server.SupervisorError, "explicitly run build-server"),
            ):
                server.install(None, root)

    def immutable_source(self, state: Path) -> tuple[ModuleType, Path]:
        state.mkdir(mode=0o700)
        bound = server.publish_generation(state)
        generation = Path(bound["supervisor_script"]).parent.parent
        spec = importlib.util.spec_from_file_location(
            "immutable_surreal_source_fixture", generation / "scripts/surreal_source.py"
        )
        assert spec is not None and spec.loader is not None
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module, generation

    def test_verified_immutable_runtime_preserves_original_compilation_identity(
        self,
    ) -> None:
        receipt = fixture_receipt()
        with tempfile.TemporaryDirectory() as scratch:
            immutable, generation = self.immutable_source(Path(scratch) / "state")
            self.assertEqual(
                (generation / ".cargo/config.toml").read_bytes(),
                (source.ROOT / ".cargo/config.toml").read_bytes(),
            )
            # The runtime path is not a new compilation root or package identity.
            self.assertTrue(immutable.admitted_receipt(receipt))
            with patch.object(
                immutable,
                "build_context",
                side_effect=AssertionError("runtime compiler scan"),
            ):
                self.assertTrue(immutable.admitted_runtime_receipt(receipt, generation))
            self.assertFalse(source.admitted_runtime_receipt(receipt, generation))

    def test_runtime_receipt_still_requires_complete_key_source_and_toolchain_proofs(
        self,
    ) -> None:
        receipt = fixture_receipt()
        with tempfile.TemporaryDirectory() as scratch:
            immutable, generation = self.immutable_source(Path(scratch) / "state")
            for change in (
                "effective_key",
                "configuration",
                "tools",
                "patch",
            ):
                candidate = copy.deepcopy(receipt)
                if change == "effective_key":
                    candidate["effective_key"] = "0" * 64
                elif change in {"configuration", "tools"}:
                    candidate["inputs"]["build_context"][change] = {}
                elif change == "patch":
                    candidate["inputs"]["patches"][source.PATCHES[0]] = "0" * 64
                else:
                    candidate["inputs"]["build_context"]["toolchain_declaration"] = {
                        "changed": True
                    }
                if change != "effective_key":
                    candidate["effective_key"] = immutable.effective_key(
                        candidate["inputs"]
                    )
                with self.subTest(change=change):
                    self.assertFalse(
                        immutable.admitted_runtime_receipt(candidate, generation)
                    )

    def test_runtime_receipt_refuses_changed_or_fake_immutable_generation(self) -> None:
        receipt = fixture_receipt()
        with tempfile.TemporaryDirectory() as scratch:
            immutable, generation = self.immutable_source(Path(scratch) / "state")
            captured = generation / ".cargo/config.toml"
            original = captured.read_bytes()
            captured.write_bytes(original + b"\n# changed after publication\n")
            self.assertFalse(immutable.admitted_runtime_receipt(receipt, generation))
            captured.write_bytes(original)
            manifest = generation / "generation.json"
            declared = json.loads(manifest.read_text())
            manifest.write_text(json.dumps({**declared, "identity": "0" * 64}))
            self.assertFalse(immutable.admitted_runtime_receipt(receipt, generation))

    def test_caller_configuration_and_compiler_changes_preserve_installed_package(self) -> None:
        receipt = fixture_receipt()
        with (
            patch.object(source.installation_context, "configuration", side_effect=AssertionError("caller configuration scan")),
            patch.object(source, "build_context", side_effect=AssertionError("caller compiler scan")),
        ):
            self.assertTrue(source.admitted_receipt(receipt))

    def test_unknown_existing_target_is_preserved_and_refused_before_compile(
        self,
    ) -> None:
        for recorded in (None, b"not json", b'{"inputs": {}, "effective_key": "old"}'):
            with (
                self.subTest(recorded=recorded),
                tempfile.TemporaryDirectory() as scratch,
            ):
                directory = Path(scratch) / source.installation_name()
                target = directory / "source/target"
                target.mkdir(parents=True)
                artifact = target / "unknown-artifact"
                artifact.write_bytes(b"preserve")
                request = directory / "build-request.json"
                if recorded is not None:
                    request.write_bytes(recorded)
                with (
                    patch.object(source, "verify_source"),
                    patch.object(source, "command") as execute,
                    self.assertRaisesRegex(source.BuildError, "target.*build request"),
                ):
                    source.install(directory)
                execute.assert_not_called()
                self.assertEqual(artifact.read_bytes(), b"preserve")
                self.assertEqual(
                    request.read_bytes() if request.exists() else None, recorded
                )
                self.assertFalse((directory / "release.json").exists())

    def test_exact_interrupted_build_request_can_resume_existing_target(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch) / source.installation_name()
            target = directory / "source/target"
            target.mkdir(parents=True)
            selected = source.inputs(Path(scratch))
            (directory / "build-request.json").write_text(
                json.dumps(source.build_request(selected))
            )

            def execute(arguments: list[str], **kwargs: object) -> str:
                if arguments[0] == str(source.ROOT / "scripts/pse-env"):
                    binary = target / source.PROFILE / "surreal"
                    binary.parent.mkdir()
                    binary.write_bytes(b"resumed executable")
                    (directory / "source/Cargo.lock").write_text("resumed lock")
                    return ""
                if arguments[-1] == "version":
                    return f"{source.VERSION} {selected['build_metadata']}"
                return "module help"

            with (
                patch.object(source, "verify_source"),
                patch.object(source, "command", side_effect=execute) as compile_command,
            ):
                receipt = source.install(directory)
            self.assertEqual(compile_command.call_count, 3)
            self.assertTrue(source.admitted_receipt(receipt))
            self.assertTrue((directory / "release.json").exists())

    def test_existing_target_cannot_resume_request_from_another_namespace(self) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch) / source.installation_name()
            (directory / "source/target").mkdir(parents=True)
            previous = source.build_request(
                source.inputs(Path(scratch) / "old-namespace")
            )
            request = directory / "build-request.json"
            request.write_text(json.dumps(previous))
            original = request.read_bytes()
            with (
                patch.object(source, "verify_source"),
                patch.object(source, "command") as execute,
                self.assertRaisesRegex(source.BuildError, "target differs"),
            ):
                source.install(directory)
            execute.assert_not_called()
            self.assertEqual(request.read_bytes(), original)

    def test_target_symlink_is_preserved_and_refused_even_with_matching_request(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as scratch:
            directory = Path(scratch) / source.installation_name()
            (directory / "source").mkdir(parents=True)
            target = directory / "source/target"
            target.symlink_to(
                Path(scratch) / "unowned-target", target_is_directory=True
            )
            request = directory / "build-request.json"
            request.write_text(
                json.dumps(source.build_request(source.inputs(Path(scratch))))
            )
            original = request.read_bytes()
            with (
                patch.object(source, "verify_source"),
                patch.object(source, "command") as execute,
                self.assertRaisesRegex(source.BuildError, "target is a symlink"),
            ):
                source.install(directory)
            execute.assert_not_called()
            self.assertTrue(target.is_symlink())
            self.assertEqual(request.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
