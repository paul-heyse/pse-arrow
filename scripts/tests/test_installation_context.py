# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed syntax equivalence and closed-context refusal controls."""
# ruff: noqa: PT009, PT027 -- stdlib unittest controls

from __future__ import annotations

import json
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import installation_context as context


class InstallationContextTests(unittest.TestCase):
    def launcher_capture(self, root: Path, env: dict[str, str]) -> dict:
        """Exercise capture with real input hashes and bounded tool/system fixtures."""
        executable = root / "compiler"
        executable.write_bytes(b"controlled compiler executable")

        def output(command: list[str], **_kwargs: object) -> str:
            if command[:2] == ["rustup", "which"]:
                return str(executable) + "\n"
            if command[1:] == ["--print", "sysroot"]:
                return str(root / "sysroot") + "\n"
            if command[1:] == ["-print-resource-dir"]:
                self.assertEqual(
                    command[0], str(Path(env.get("CLANG_PATH", executable)).resolve())
                )
                return str(root / "clang-resource") + "\n"
            return "controlled tool version\n"

        actual_tree = context.tree
        launcher = root / ".config/native-cache.cmake"
        with (
            patch.object(context, "configuration", return_value=([], [], [])),
            patch.object(context.subprocess, "check_output", side_effect=output),
            patch.object(
                context.subprocess,
                "run",
                return_value=context.subprocess.CompletedProcess([], 0, "", ""),
            ),
            patch.object(context.shutil, "which", return_value=str(executable)),
            patch.object(
                context,
                "tree",
                side_effect=lambda path: (
                    actual_tree(path)
                    if path in {launcher, root / "clang-resource"}
                    else {"path": str(path), "controlled_system_input": True}
                ),
            ),
        ):
            captured, _provenance = context.capture(root, env)
        return captured

    def test_selected_clang_resource_headers_are_captured(self) -> None:
        repository = Path(context.__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "rust-toolchain.toml").write_bytes(
                (repository / "rust-toolchain.toml").read_bytes()
            )
            clang = root / "selected-clang"
            clang.write_bytes(b"controlled selected Clang")
            resource = root / "clang-resource"
            include = resource / "include"
            include.mkdir(parents=True)
            header = include / "stddef.h"
            header.write_bytes(b"controlled resource header")
            env = {
                "CLANG_PATH": str(clang),
                "BINDGEN_EXTRA_CLANG_ARGS": f"-resource-dir={resource}",
            }
            before = self.launcher_capture(root, env)
            self.assertIn(context.tree(resource), before["native"])
            repeated = self.launcher_capture(root, env)
            self.assertEqual(context.identity(before), context.identity(repeated))
            header.write_bytes(b"changed resource header")
            changed = self.launcher_capture(root, env)
            self.assertNotEqual(context.identity(before), context.identity(changed))
            (include / "float.h").write_bytes(b"another resource header")
            added = self.launcher_capture(root, env)
            self.assertNotEqual(context.identity(changed), context.identity(added))

    def test_bindgen_refuses_alternate_flags_paths_and_missing_selected_headers(
        self,
    ) -> None:
        repository = Path(context.__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "rust-toolchain.toml").write_bytes(
                (repository / "rust-toolchain.toml").read_bytes()
            )
            resource = root / "clang-resource"
            include = resource / "include"
            include.mkdir(parents=True)
            header = include / "stddef.h"
            header.write_bytes(b"controlled resource header")
            alternate = root / "other-resource"
            (alternate / "include").mkdir(parents=True)
            (alternate / "include/stddef.h").write_bytes(b"other header")
            alias = root / "resource-alias"
            alias.symlink_to(resource, target_is_directory=True)
            exact = f"-resource-dir={resource}"
            for flags in (
                "-I/external/include",
                "-resource-dir=",
                "-resource-dir=relative",
                f" {exact}",
                f"{exact} -DOTHER=1",
                f"-resource-dir {resource}",
                f'"{exact}"',
                f'-resource-dir="{resource}"',
                f"-resource-dir={resource}/.",
                f"-resource-dir={alternate}",
                f"-resource-dir={alias}",
                "-resource-dir='unclosed",
            ):
                with (
                    self.subTest(flags=flags),
                    self.assertRaisesRegex(
                        context.ContextError, "BINDGEN_EXTRA_CLANG_ARGS"
                    ),
                ):
                    self.launcher_capture(root, {"BINDGEN_EXTRA_CLANG_ARGS": flags})
            header.unlink()
            with self.assertRaisesRegex(
                context.ContextError, "BINDGEN_EXTRA_CLANG_ARGS"
            ):
                self.launcher_capture(root, {"BINDGEN_EXTRA_CLANG_ARGS": exact})
            for selected in ("relative", str(root / "absent-resource"), ""):
                with (
                    self.subTest(clang_output=selected),
                    patch.object(
                        context.subprocess, "check_output", return_value=selected
                    ),
                    self.assertRaisesRegex(
                        context.ContextError, "BINDGEN_EXTRA_CLANG_ARGS"
                    ),
                ):
                    context.bindgen_resource(
                        root,
                        {"BINDGEN_EXTRA_CLANG_ARGS": f"-resource-dir={selected}"},
                        "clang",
                    )

    def test_repository_cache_toolchain_and_launcher_bytes_are_captured(self) -> None:
        repository = Path(context.__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / ".config").mkdir()
            launcher = root / ".config/native-cache.cmake"
            launcher.write_bytes(
                (repository / ".config/native-cache.cmake").read_bytes()
            )
            (root / "rust-toolchain.toml").write_bytes(
                (repository / "rust-toolchain.toml").read_bytes()
            )
            wrapper = root / "cache-wrapper"
            wrapper.write_bytes(b"controlled cache wrapper")
            binary = root / "sccache"
            binary.write_bytes(b"controlled cache executable")
            env = {
                "HOME": temporary,
                "CMAKE_TOOLCHAIN_FILE": str(launcher),
                "PSE_NATIVE_COMPILER_CACHE": str(wrapper),
                "PSE_SCCACHE_BINARY": str(binary),
            }
            before = self.launcher_capture(root, env)
            self.assertIn(context.tree(launcher), before["native"])
            self.assertEqual(
                before["tools"]["PSE_NATIVE_COMPILER_CACHE"]["sha256"],
                context.sha(wrapper),
            )
            self.assertEqual(
                before["tools"]["PSE_SCCACHE_BINARY"]["sha256"], context.sha(binary)
            )
            repeated = self.launcher_capture(root, env)
            self.assertEqual(context.identity(before), context.identity(repeated))
            launcher.write_bytes(launcher.read_bytes() + b"\n# changed source\n")
            changed_source = self.launcher_capture(root, env)
            self.assertNotEqual(
                context.identity(before), context.identity(changed_source)
            )
            wrapper.write_bytes(b"changed cache wrapper")
            changed_wrapper = self.launcher_capture(root, env)
            self.assertNotEqual(
                context.identity(changed_source), context.identity(changed_wrapper)
            )
            binary.write_bytes(b"changed cache executable")
            changed_binary = self.launcher_capture(root, env)
            self.assertNotEqual(
                context.identity(changed_wrapper), context.identity(changed_binary)
            )
            relative = self.launcher_capture(
                root, {**env, "CMAKE_TOOLCHAIN_FILE": ".config/native-cache.cmake"}
            )
            self.assertIn(context.tree(launcher), relative["native"])

    def test_unknown_external_and_symlink_toolchain_programs_refuse(self) -> None:
        repository = Path(context.__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "workspace"
            config = root / ".config"
            config.mkdir(parents=True)
            expected = config / "native-cache.cmake"
            contents = (repository / ".config/native-cache.cmake").read_bytes()
            expected.write_bytes(contents)
            unknown = config / "unknown.cmake"
            unknown.write_bytes(contents)
            external = Path(temporary) / "external"
            external.mkdir()
            external_program = external / "native-cache.cmake"
            external_program.write_bytes(contents)
            alias = config / "alias.cmake"
            alias.symlink_to(expected)

            def refuses(selected: Path) -> None:
                with (
                    patch.object(context.subprocess, "check_output") as tool_probe,
                    self.assertRaisesRegex(context.ContextError, "CMake toolchain"),
                ):
                    context.capture(root, {"CMAKE_TOOLCHAIN_FILE": str(selected)})
                tool_probe.assert_not_called()

            for selected in (unknown, external_program, alias):
                with self.subTest(selected=selected):
                    refuses(selected)
            expected.unlink()
            expected.symlink_to(external_program)
            refuses(expected)
            expected.unlink()
            refuses(expected)
            expected.mkdir()
            refuses(expected)
            expected.rmdir()
            alias.unlink()
            unknown.unlink()
            config.rmdir()
            config.symlink_to(external, target_is_directory=True)
            refuses(expected)

    def test_comments_formatting_and_key_order_preserve_typed_toml(self) -> None:
        before = 'a=1\nb=true\nflags=["-C", "one", "two"]\n[nested]\nx="v"\ny=2.0\n'
        after = '# comment\nb = true\nflags = [ "-C", "one", "two", ]\na = 1\n[nested]\ny=2.0\nx="v"\n'
        self.assertEqual(
            context.canonical(tomllib.loads(before)),
            context.canonical(tomllib.loads(after)),
        )
        for changed in (
            before.replace("a=1", "a=true"),
            before.replace("a=1", "a=1.0"),
            before.replace('"one", "two"', '"two", "one"'),
        ):
            self.assertNotEqual(
                context.canonical(tomllib.loads(before)),
                context.canonical(tomllib.loads(changed)),
            )

    def test_hierarchy_roots_legacy_precedence_and_raw_provenance(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "workspace"
            root.mkdir()
            cargo = root / ".cargo"
            cargo.mkdir()
            home = Path(temporary) / "home"
            home.mkdir()
            (home / "config.toml").write_text('[build]\nrustc-wrapper="sccache"\n')
            selected = cargo / "config.toml"
            selected.write_text(
                '[build]\njobs=16\n[target.x86_64-unknown-linux-gnu]\nlinker="../bin/clang"\n'
            )
            env = {"HOME": str(home.parent), "CARGO_HOME": str(home)}
            before, raw, _ = context.configuration(root, env)
            selected.write_text(
                '# comment\n[build]\njobs = 16\n[target.x86_64-unknown-linux-gnu]\nlinker = "../bin/clang"\n'
            )
            after, raw_after, _ = context.configuration(root, env)
            self.assertEqual(before, after)
            self.assertNotEqual(raw, raw_after)
            (cargo / "config").write_text("[build]\njobs=8\n")
            changed, _, _ = context.configuration(root, env)
            self.assertNotEqual(before, changed)
            self.assertEqual(changed[-1]["definition_root"], str(root))
            self.assertIn("rustc-wrapper", json.dumps(before))
            (cargo / "config").write_text('[env]\nUNTRACKED_FILE="secret-path"\n')
            with self.assertRaisesRegex(
                context.ContextError, "Unsupported Cargo configuration"
            ):
                context.configuration(root, env)

    def test_native_file_same_path_replacement_and_absence_change_identity(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            selected = directory / "native"
            absent = context.tree(selected)
            selected.mkdir()
            library = selected / "lib.so"
            library.write_bytes(b"old")
            before = context.tree(selected)
            library.write_bytes(b"new")
            self.assertNotEqual(before, context.tree(selected))
            self.assertNotEqual(absent, before)

    def test_freshness_configuration_preserves_syntax_equivalence_and_policy_identity(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "workspace"
            (root / ".cargo").mkdir(parents=True)
            selected = root / ".cargo/config.toml"
            original = (
                '[build]\njobs=16\nfingerprint="content"\n'
                "[unstable]\nchecksum-freshness=true\n"
            )
            selected.write_text(original)
            env = {"HOME": temporary, "CARGO_HOME": str(Path(temporary) / "cargo-home")}
            before, raw, _ = context.configuration(root, env)
            selected.write_text(
                "# same declarations\n[unstable]\nchecksum-freshness = true\n"
                '[build]\nfingerprint = "content"\njobs = 16\n'
            )
            equivalent, reformatted, _ = context.configuration(root, env)
            self.assertEqual(context.identity(before), context.identity(equivalent))
            self.assertNotEqual(raw, reformatted)
            for changed in (
                original.replace('fingerprint="content"', 'fingerprint="mtime"'),
                original.replace("checksum-freshness=true", "checksum-freshness=false"),
            ):
                with self.subTest(configuration=changed):
                    selected.write_text(changed)
                    after, _, _ = context.configuration(root, env)
                    self.assertNotEqual(
                        context.identity(before), context.identity(after)
                    )
            for unknown in (
                original.replace("fingerprint=", "future-fingerprint="),
                original.replace("checksum-freshness=", "future-checksum-freshness="),
            ):
                with self.subTest(configuration=unknown):
                    selected.write_text(unknown)
                    with self.assertRaisesRegex(
                        context.ContextError, "Unsupported Cargo configuration"
                    ):
                        context.configuration(root, env)

    def test_controlled_environment_drops_unknown_variables_and_refuses_unknown_cargo_controls(
        self,
    ) -> None:
        root = Path("/fixture")
        with patch(
            "scripts.pse_env.compose",
            return_value={
                "HOME": "/home/test",
                "PATH": "/usr/bin",
                "PRIVATE_TOKEN": "never-published",
                "CC": "clang",
                "SCCACHE_MAX_FRAME_LENGTH": "134217728",
            },
        ):
            selected = context.controlled_environment(root, {})
        self.assertEqual(
            selected,
            {
                "HOME": "/home/test",
                "PATH": "/usr/bin",
                "CC": "clang",
                "SCCACHE_MAX_FRAME_LENGTH": "134217728",
            },
        )
        for variable in ("CARGO_UNDERSTOOD_IN_FUTURE", "CARGO_BUILD_FINGERPRINT"):
            with (
                self.subTest(variable=variable),
                patch("scripts.pse_env.compose", return_value={variable: "value"}),
                self.assertRaisesRegex(
                    context.ContextError, "Unsupported Cargo environment"
                ),
            ):
                context.controlled_environment(root, {})

    def test_literal_tool_path_with_spaces_and_changed_executable_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            executable = Path(temporary) / "compiler path"
            executable.write_bytes(b"first")
            before = context.tool(str(executable), Path(temporary), {})
            executable.write_bytes(b"other")
            self.assertNotEqual(
                before, context.tool(str(executable), Path(temporary), {})
            )

    def test_relative_linker_and_added_parent_configuration_change_identity(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "workspace"
            (root / ".cargo").mkdir(parents=True)
            config = root / ".cargo/config.toml"
            config.write_text('[target.x86_64-unknown-linux-gnu]\nlinker="bin/clang"\n')
            env = {"HOME": temporary, "CARGO_HOME": str(Path(temporary) / "home-cargo")}
            before, _, _ = context.configuration(root, env)
            config.write_text(
                '[target.x86_64-unknown-linux-gnu]\nlinker="other/clang"\n'
            )
            after, _, _ = context.configuration(root, env)
            self.assertNotEqual(before, after)
            (Path(temporary) / ".cargo").mkdir()
            (Path(temporary) / ".cargo/config.toml").write_text("[build]\njobs=8\n")
            parent, _, _ = context.configuration(root, env)
            self.assertNotEqual(after, parent)


if __name__ == "__main__":
    unittest.main()
