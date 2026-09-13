# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Behavioral qualification of setup contracts using disposable fixtures."""
# These tests must run before pytest or any project environment is installed.
# ruff: noqa: PT009, PT027

from __future__ import annotations

import importlib.util
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

if TYPE_CHECKING:
    from types import ModuleType

ROOT = Path(__file__).resolve().parents[2]


def load(name: str) -> ModuleType:
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load setup module {name}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


hooks = load("agent-hooks")
agents = load("agent-config")
images = load("solver-images")
adr = load("adr")


class EditPolicyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def test_patch_includes_all_files_and_both_rename_paths(self) -> None:
        command = "*** Begin Patch\n*** Update File: a.py\n*** Move to: b.py\n@@\n-x\n+y\n*** Delete File: c.py\n*** Add File: docs/generated/new.md\n+x\n*** End Patch"
        paths = hooks.edit_paths({"tool_input": {"command": command}})
        self.assertEqual(paths, ["a.py", "b.py", "c.py", "docs/generated/new.md"])
        self.assertIsNotNone(hooks.protected(self.root, paths[-1]))

    def test_claude_payload_and_invalid_payload(self) -> None:
        self.assertEqual(
            hooks.edit_paths({"tool_input": {"file_path": "a.py"}}), ["a.py"]
        )
        with self.assertRaises(ValueError):
            hooks.edit_paths({"tool_input": {"command": "echo unchecked"}})
        for payload in ([], {"tool_input": None}, {"tool_input": {"file_path": 3}}):
            with (
                self.subTest(payload=payload),
                self.assertRaises((ValueError, TypeError)),
            ):
                hooks.edit_paths(payload)

    def test_paths_are_normalized_before_protection(self) -> None:
        for name in (
            "docs/../docs/generated/a",
            "./external/a",
            ".git/config",
            "crates/a/src/generated/b.rs",
            "python/pse/contracts/a.py",
            "crates/pse-ipopt-sys/src/bindings.rs",
            "../outside",
        ):
            with self.subTest(name=name):
                self.assertIsNotNone(hooks.protected(self.root, name, design_edit=True))
        self.assertIsNone(hooks.protected(self.root, "crates/a/src/lib.rs"))
        self.assertIsNone(hooks.protected(self.root, str(self.root / "a.py")))

    def test_symlink_cannot_hide_a_protected_destination(self) -> None:
        (self.root / "external").mkdir()
        try:
            (self.root / "alias").symlink_to(
                self.root / "external", target_is_directory=True
            )
        except OSError:
            # Windows fallback is covered separately without requiring symlink privileges.
            return
        self.assertIsNotNone(hooks.protected(self.root, "alias/a.py"))

    def test_decision_guard_and_narrow_escape(self) -> None:
        path = self.root / "docs/adr/0001-example.md"
        path.parent.mkdir(parents=True)
        path.write_text("---\nstatus: accepted\n---\n# Decision\n")
        self.assertIsNotNone(hooks.protected(self.root, str(path)))
        self.assertIsNone(hooks.protected(self.root, str(path), design_edit=True))
        path.write_text("---\nstatus: proposed\n---\n# Decision\n")
        self.assertIsNone(hooks.protected(self.root, str(path)))
        self.assertIsNotNone(
            hooks.protected(self.root, "docs/authoritative_design/blueprint.md")
        )

    def test_formatter_only_receives_permitted_edited_files(self) -> None:
        for name in ("a.py", "neighbor.py", "docs/generated/skip.py"):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("x=1\n")
        with patch.object(hooks.subprocess, "run") as run:
            hooks.format_paths(
                self.root, ["a.py", "a.py", "deleted.py", "docs/generated/skip.py"]
            )
        self.assertEqual(run.call_count, 1)
        self.assertEqual(run.call_args.args[0][-1], str(self.root / "a.py"))
        self.assertEqual((self.root / "neighbor.py").read_text(), "x=1\n")


class ConfigurationTests(unittest.TestCase):
    def test_materialized_skills_and_native_roles_detect_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(ROOT / ".claude/agents", root / ".claude/agents")
            shutil.copytree(ROOT / ".codex/skills", root / ".codex/skills")
            (root / ".codex/agents").write_text("../.claude/agents")
            (root / ".claude/skills").write_text("../.codex/skills")
            with patch.object(
                Path, "symlink_to", side_effect=OSError("no Windows privilege")
            ):
                self.assertEqual(agents.synchronize(root, check=False), [])
            self.assertTrue((root / ".agents/skills/adr/SKILL.md").is_file())
            self.assertEqual(agents.synchronize(root, check=True), [])
            native = root / ".codex/agents/plan-scout.toml"
            native.write_text(
                native.read_text().replace("read-only", "workspace-write")
            )
            self.assertTrue(agents.synchronize(root, check=True))

    def test_solver_pins_require_matching_trees_and_digests(self) -> None:
        pins = json.loads((ROOT / ".github/setup/solver-images.json").read_text())
        images.validate(pins)
        self.assertEqual(images.projections(ROOT, pins), {})
        with self.assertRaises(ValueError):
            images.validate({**pins, "dev": pins["dev"].split("@")[0]})
        with self.assertRaises(ValueError):
            images.validate(
                {
                    **pins,
                    "dev": pins["dev"].replace("dev-e84fdce84e2f", "dev-000000000000"),
                }
            )

    def test_accepted_adr_edits_rejected_against_base(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/adr/0001-example.md"
            path.parent.mkdir(parents=True)
            path.write_text(
                "---\nstatus: accepted\ntitle: Original\n---\n## Context\nOriginal\n"
            )
            commands = (
                ["init", "-b", "main"],
                ["add", "."],
                [
                    "-c",
                    "commit.gpgsign=false",
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "commit",
                    "-m",
                    "fixture",
                ],
                ["update-ref", "refs/remotes/origin/main", "HEAD"],
            )
            for args in commands:
                subprocess.run(
                    ["git", *args], cwd=root, check=True, capture_output=True
                )
            path.write_text(path.read_text().replace("Original", "Altered"))
            with patch.object(adr, "ROOT", root):
                self.assertTrue(adr.lint_immutability([path]))


if __name__ == "__main__":
    unittest.main()
