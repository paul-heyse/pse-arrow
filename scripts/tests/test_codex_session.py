# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Single selected-session attachment, literal argv and lazy readiness controls."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import json
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import codex_session as session


class CodexSessionTests(unittest.TestCase):
    def describe(self, **overrides):
        values = {
            "state": Path("/selected state"),
            "database": True,
            "rust": True,
            "require_database": False,
            "require_rust": False,
            "arguments": [],
        }
        values.update(overrides)
        return session.session_description(**values)

    def test_missing_checkout_interpreter_refuses_without_borrowing(self):
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(session, "ROOT", Path(directory)),
            patch.object(session.shutil, "which", return_value="/machine/tool"),
            self.assertRaisesRegex(ValueError, ".venv/bin/python is unavailable"),
        ):
            self.describe()

    def test_one_entrypoint_literal_overrides_and_nonsecret_lazy_configuration(self):
        with tempfile.TemporaryDirectory() as directory:
            state = Path(directory)
            literal = [
                "-c",
                "mcp_servers.pse-capabilities.enabled=false",
                "spaces $HOME `pwd`",
            ]
            with patch.object(session.shutil, "which", return_value="/machine/tool"):
                command, description = self.describe(state=state, arguments=literal)
            self.assertEqual(command[-len(literal) :], literal)
            self.assertIn("mcp_servers.pse-capabilities.enabled=true", command)
            self.assertIn("mcp_servers.pse-capabilities.required=true", command)
            for specialist in ("rust-analyzer", "pse-python", "pse-surreal"):
                self.assertIn("mcp_servers." + specialist + ".enabled=false", command)
                self.assertFalse(
                    any(
                        value.startswith("mcp_servers." + specialist + ".command=")
                        for value in command
                    )
                )
            setting = next(
                value
                for value in command
                if value.startswith("mcp_servers.pse-capabilities.env=")
            )
            parsed = tomllib.loads(setting)["mcp_servers"]["pse-capabilities"]["env"]
            self.assertEqual(parsed["PSE_CAPABILITIES_DATABASE_STATE"], str(state))
            self.assertEqual(parsed["PSE_CAPABILITIES_DATABASE"], "1")
            self.assertNotIn("Authorization", json.dumps(description))
            self.assertEqual(description["entrypoint"], "pse-capabilities")
            self.assertEqual(
                description["capabilities"]["database"]["attached"], "not observed"
            )
            self.assertIn("lazily", description["capabilities"]["database"]["identity"])

    def test_required_database_checks_preparation_without_observing_service(self):
        with patch.object(session.shutil, "which", return_value="/machine/tool"):
            with self.assertRaisesRegex(ValueError, "Required selected database state"):
                self.describe(
                    state=Path("/not-a-selected-state"), require_database=True
                )
            with tempfile.TemporaryDirectory() as directory:
                _, description = self.describe(
                    state=Path(directory), require_database=True
                )
                self.assertTrue(description["capabilities"]["database"]["prepared"])
                self.assertIn("actual selected calls", description["readiness"])

    def test_explicit_opt_out_is_carried_by_canonical_session(self):
        with patch.object(session.shutil, "which", return_value="/machine/tool"):
            command, _ = self.describe(database=False, rust=False, python=False)
        setting = next(
            value
            for value in command
            if value.startswith("mcp_servers.pse-capabilities.env=")
        )
        parsed = tomllib.loads(setting)["mcp_servers"]["pse-capabilities"]["env"]
        self.assertEqual(parsed["PSE_CAPABILITIES_DATABASE"], "0")
        self.assertEqual(parsed["PSE_CAPABILITIES_LANGUAGES"], "")

    def test_missing_required_rust_fails_before_selection(self):
        with (
            patch.object(
                session.shutil,
                "which",
                side_effect=lambda name: "/codex" if name == "codex" else None,
            ),
            self.assertRaisesRegex(ValueError, "Required rust-analyzer"),
        ):
            self.describe(require_rust=True)


if __name__ == "__main__":
    unittest.main()
