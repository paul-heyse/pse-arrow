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
                '[unstable]\nchecksum-freshness=true\n'
            )
            selected.write_text(original)
            env = {"HOME": temporary, "CARGO_HOME": str(Path(temporary) / "cargo-home")}
            before, raw, _ = context.configuration(root, env)
            selected.write_text(
                '# same declarations\n[unstable]\nchecksum-freshness = true\n'
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
                    self.assertNotEqual(context.identity(before), context.identity(after))
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
