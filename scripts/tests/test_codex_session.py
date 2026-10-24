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


def object_value(value: object) -> dict[str, object]:
    if not isinstance(value, dict):
        raise TypeError("expected a session description object")
    return value


def text_value(value: object) -> str:
    if not isinstance(value, str):
        raise TypeError("expected session description text")
    return value


class CodexSessionTests(unittest.TestCase):
    def describe(
        self,
        *,
        state: Path | None = Path("/selected state"),
        database: bool = True,
        rust: bool = True,
        require_database: bool = False,
        require_rust: bool = False,
        arguments: list[str] | None = None,
        python: bool = True,
        require_python: bool = False,
    ) -> tuple[list[str], dict[str, object]]:
        return session.session_description(
            state=state,
            database=database,
            rust=rust,
            require_database=require_database,
            require_rust=require_rust,
            arguments=[] if arguments is None else arguments,
            python=python,
            require_python=require_python,
        )

    def test_missing_checkout_interpreter_refuses_without_borrowing(self) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(session, "ROOT", Path(directory)),
            patch.object(session.shutil, "which", return_value="/machine/tool"),
            self.assertRaisesRegex(ValueError, ".venv/bin/python is unavailable"),
        ):
            self.describe()

    def test_one_entrypoint_literal_overrides_and_nonsecret_lazy_configuration(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            state = Path(directory)
            literal = [
                "-c",
                "mcp_servers.pse-capabilities.enabled=false",
                "spaces $HOME `pwd`",
            ]
            with (
                patch.object(session.shutil, "which", return_value="/machine/tool"),
                patch.object(
                    session,
                    "inherited_specialists",
                    return_value=("rust-analyzer", "pse-python", "pse-surreal"),
                ),
            ):
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
            capabilities = object_value(description["capabilities"])
            database = object_value(capabilities["database"])
            self.assertEqual(database["attached"], "not observed")
            self.assertIn("lazily", text_value(database["identity"]))

    def test_absent_specialists_do_not_create_transportless_disabled_entries(
        self,
    ) -> None:
        with (
            patch.object(session.shutil, "which", return_value="/machine/tool"),
            patch.object(session, "inherited_specialists", return_value=()),
        ):
            command, _ = self.describe()
        for name in ("rust-analyzer", "pse-python", "pse-surreal"):
            self.assertFalse(
                any(value.startswith("mcp_servers." + name + ".") for value in command)
            )

    def test_standard_config_layers_only_disable_real_legacy_transports(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "checkout"
            root.mkdir()
            config = Path(directory) / "codex"
            config.mkdir()
            (config / "config.toml").write_text(
                '[mcp_servers.rust-analyzer]\ncommand="rust-tool"\n[mcp_servers.unrelated]\nurl="https://example.invalid/mcp"\n'
            )
            (root / ".codex").mkdir()
            (root / ".codex/config.toml").write_text(
                '[mcp_servers.pse-python]\ncommand="python-tool"\n[mcp_servers.pse-surreal]\nenabled=false\n'
            )
            with patch.dict(session.os.environ, {"CODEX_HOME": str(config)}):
                self.assertEqual(
                    session.inherited_specialists(root), ("pse-python", "rust-analyzer")
                )

    def test_required_database_checks_preparation_without_observing_service(
        self,
    ) -> None:
        with patch.object(session.shutil, "which", return_value="/machine/tool"):
            with self.assertRaisesRegex(ValueError, "Required selected database state"):
                self.describe(
                    state=Path("/not-a-selected-state"), require_database=True
                )
            with tempfile.TemporaryDirectory() as directory:
                _, description = self.describe(
                    state=Path(directory), require_database=True
                )
                capabilities = object_value(description["capabilities"])
                database = object_value(capabilities["database"])
                self.assertTrue(database["prepared"])
                self.assertIn(
                    "actual selected calls", text_value(description["readiness"])
                )

    def test_explicit_opt_out_is_carried_by_canonical_session(self) -> None:
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

    def test_missing_required_rust_fails_before_selection(self) -> None:
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
