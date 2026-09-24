# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Behavioral qualification of setup contracts using disposable fixtures."""
# These tests must run before pytest or any project environment is installed.
# ruff: noqa: PT009, PT027

from __future__ import annotations

import importlib.util
import json
import re
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
doctor = load("doctor")
agent_checks = load("check_agent_config")


def tracked_skills() -> list[str]:
    """Skill directories that `.gitignore` does not exclude (the repository-process skills)."""
    names = sorted(p.name for p in (ROOT / ".codex/skills").iterdir() if p.is_dir())
    ignored = subprocess.run(
        ["git", "check-ignore", "--no-index", "--stdin"],
        input="".join(f".codex/skills/{name}/SKILL.md\n" for name in names),
        capture_output=True,
        text=True,
        cwd=ROOT,
        check=False,
    ).stdout.split()
    return [name for name in names if f".codex/skills/{name}/SKILL.md" not in ignored]


class EditPolicyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def settings_deny_paths(self) -> list[str]:
        settings = json.loads((ROOT / ".claude/settings.json").read_text())
        paths = []
        for rule in settings["permissions"]["deny"]:
            if rule.startswith(("Edit(", "Read(")):
                paths.append(rule[rule.index("(") + 1 : -1])
        return paths

    @staticmethod
    def anchored_match(pattern: str, relative: str) -> bool:
        """Match a `/`-anchored permission pattern against a repository-relative path."""
        body = re.escape(pattern.removeprefix("/"))
        body = body.replace(r"\*\*", ".*").replace(r"\*", "[^/]*")
        return re.fullmatch(body, relative) is not None

    def test_settings_deny_rules_are_anchored_edit_rules(self) -> None:
        problems: list[str] = []
        agent_checks.check_deny_rules(ROOT / ".claude/settings.json", problems)
        self.assertEqual(problems, [])

    def test_settings_deny_rules_agree_with_the_hook(self) -> None:
        # Every path the permission layer refuses, the shared hook refuses too, so the two
        # layers AGENTS.md describes never disagree about what is protected.
        for pattern in self.settings_deny_paths():
            sample = (
                pattern.removeprefix("/")
                .replace("**", "probe.txt")
                .replace("*", "pse-x")
            )
            with self.subTest(pattern=pattern):
                self.assertIsNotNone(hooks.protected(self.root, sample))

    def test_settings_deny_rules_do_not_reach_nested_directories(self) -> None:
        # A single-segment deny such as `./build/**` resolves against the current directory
        # and matches a `build` directory at any depth; anchored, it protects only the root's.
        nested = [
            ".codex/skills/example/build/acquire.py",
            ".codex/skills/example/target/debug/x",
            "crates/pse-x/build/out.rs",
            "docs/plans/build/notes.md",
        ]
        for pattern in self.settings_deny_paths():
            for path in nested:
                with self.subTest(pattern=pattern, path=path):
                    self.assertFalse(self.anchored_match(pattern, path))
        self.assertTrue(self.anchored_match("/build/**", "build/x.json"))

    def test_patch_includes_all_files_and_both_rename_paths(self) -> None:
        command = "*** Begin Patch\n*** Update File: a.py\n*** Move to: b.py\n@@\n-x\n+y\n*** Delete File: c.py\n*** Add File: docs/generated/new.md\n+x\n*** End Patch"
        paths = hooks.edit_paths({"tool_input": {"command": command}})
        self.assertEqual(paths, ["a.py", "b.py", "c.py", "docs/generated/new.md"])
        self.assertIsNotNone(hooks.protected(self.root, paths[-1]))

    def test_claude_payload_and_invalid_payload(self) -> None:
        self.assertEqual(
            hooks.edit_paths({"tool_input": {"file_path": "a.py"}}), ["a.py"]
        )
        self.assertEqual(
            hooks.edit_paths({"tool_input": {"notebook_path": "a.ipynb"}}), ["a.ipynb"]
        )
        with self.assertRaises(ValueError):
            hooks.edit_paths({"tool_input": {"command": "echo unchecked"}})
        for payload in (
            [],
            {"tool_input": None},
            {"tool_input": {"file_path": 3}},
            {"tool_input": {"notebook_path": ""}},
        ):
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
        ):
            with self.subTest(name=name):
                self.assertIsNotNone(hooks.protected(self.root, name, design_edit=True))
        self.assertIsNone(hooks.protected(self.root, "crates/a/src/lib.rs"))
        self.assertIsNone(
            hooks.protected(self.root, str((self.root / "a.py").resolve()))
        )

    def test_runtime_areas_are_writable_and_other_places_are_not(self) -> None:
        other = tempfile.TemporaryDirectory()
        self.addCleanup(other.cleanup)
        area = Path(other.name) / "agent-home"
        with patch.object(hooks, "agent_areas", return_value=[area]):
            self.assertIsNone(hooks.protected(self.root, str(area / "memory/note.md")))
            self.assertIsNotNone(
                hooks.protected(self.root, str(Path(other.name) / "elsewhere/a.py"))
            )

    def test_runtime_areas_follow_the_runtime_configuration(self) -> None:
        override = {
            "CLAUDE_CONFIG_DIR": str(self.root / "cfg"),
            hooks.WRITABLE_ENV: str(self.root / "extra"),
        }
        with patch.dict(hooks.os.environ, override):
            areas = hooks.agent_areas()
        for expected in (
            self.root / "cfg",
            self.root / "extra",
            Path(tempfile.gettempdir()),
            Path.home() / ".codex",
        ):
            self.assertIn(expected.resolve(), areas)

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

    def test_formatter_ignores_files_outside_the_working_copy(self) -> None:
        other = tempfile.TemporaryDirectory()
        self.addCleanup(other.cleanup)
        stray = Path(other.name) / "scratch.py"
        stray.write_text("x=1\n")
        with patch.object(hooks.subprocess, "run") as run:
            hooks.format_paths(self.root, [str(stray)])
        run.assert_not_called()

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
        self.assertEqual(run.call_args.args[0][-1], str((self.root / "a.py").resolve()))
        self.assertEqual((self.root / "neighbor.py").read_text(), "x=1\n")

    def test_formatter_excludes_canonical_and_materialized_skills(self) -> None:
        skills = [
            f"{runtime}/skills/example/{name}"
            for runtime in (".codex", ".claude", ".agents")
            for name in ("scripts/cli.py", "build/Cargo.toml", "examples/demo.rs")
        ]
        product = ["scripts/tool.py", ".codex/agents/reviewer.toml", "crates/demo.rs"]
        for name in skills + product:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("unformatted fixture\n")
        with patch.object(hooks.subprocess, "run") as run:
            hooks.format_paths(self.root, skills + product)
        self.assertEqual(
            [call.args[0][-1] for call in run.call_args_list],
            [str(self.root / name) for name in product],
        )
        for name in skills:
            self.assertEqual((self.root / name).read_text(), "unformatted fixture\n")

    def test_formatter_excludes_skill_symlink_aliases(self) -> None:
        canonical = self.root / ".codex/skills/example/scripts/cli.py"
        canonical.parent.mkdir(parents=True)
        canonical.write_text("x=1\n")
        for runtime in (".claude", ".agents"):
            alias = self.root / runtime / "skills"
            alias.parent.mkdir()
            try:
                alias.symlink_to(self.root / ".codex/skills", target_is_directory=True)
            except OSError:
                # The materialized-copy test covers systems without symlink support.
                return
        paths = [
            f"{runtime}/skills/example/scripts/cli.py"
            for runtime in (".codex", ".claude", ".agents")
        ]
        with patch.object(hooks.subprocess, "run") as run:
            hooks.format_paths(self.root, paths)
        run.assert_not_called()
        self.assertEqual(canonical.read_text(), "x=1\n")


class ConfigurationTests(unittest.TestCase):
    def test_instruction_scan_excludes_all_skill_contents(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected = [
                "AGENTS.md",
                "CLAUDE.md",
                ".claude/rules/python.md",
                ".claude/agents/reviewer.md",
            ]
            skill_files = [
                f"{runtime}/skills/example/{name}"
                for runtime in (".codex", ".claude", ".agents")
                for name in ("SKILL.md", "REFERENCE.md", "evidence/SKILL.md")
            ]
            for name in expected + skill_files:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("`scripts/missing.py` and `just missing-recipe`\n")
            with patch.object(agent_checks, "ROOT", root):
                scanned = agent_checks.scanned_files()
            self.assertEqual(scanned, [root / name for name in expected])
            # The excluded documents must not disable validation of root instructions.
            for path in scanned:
                self.assertEqual(
                    agent_checks.path_refs(path.read_text()), {"scripts/missing.py"}
                )
                self.assertEqual(
                    agent_checks.just_recipes(path.read_text()), {"missing-recipe"}
                )

    def test_doctor_reads_distribution_versions_without_launching_wrappers(
        self,
    ) -> None:
        with (
            patch.object(doctor, "quality_tool_names", return_value=["pyrefly"]),
            patch.object(Path, "exists", return_value=True),
            patch.object(
                doctor, "run", return_value=(0, '{"pyrefly": "1.3.0"}')
            ) as run,
        ):
            check = doctor.check_quality_tools()
        self.assertTrue(check.ok)
        self.assertIn("pyrefly 1.3.0", check.detail)
        run.assert_called_once()
        self.assertEqual(run.call_args.args[0], str(doctor.venv_bin("python")))

    def test_materialized_skills_and_native_roles_detect_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(ROOT / ".claude/agents", root / ".claude/agents")
            # Only the skills the repository tracks by rule: library skills are gitignored and
            # local-only, and copying them made this test copy many gigabytes.
            for skill in tracked_skills():
                shutil.copytree(
                    ROOT / ".codex/skills" / skill, root / ".codex/skills" / skill
                )
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
