# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected-session argv, optional capability and nonsecret observation controls."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from scripts import codex_session as session


class CodexSessionTests(unittest.TestCase):
    def test_missing_checkout_interpreter_disables_helpers_without_borrowing(self) -> None:
        with tempfile.TemporaryDirectory() as directory, \
             patch.object(session, 'ROOT', Path(directory)), \
             patch.object(session.shutil, 'which', return_value='/machine/tool'), \
             patch.object(session.surreal_server, 'selected_service') as observe:
            command, description = self.describe()
            self.assertFalse(session.surreal_server.object_mapping(session.surreal_server.object_mapping(description['capabilities'])['database'])['available'])
            self.assertFalse(session.surreal_server.object_mapping(session.surreal_server.object_mapping(description['capabilities'])['rust'])['available'])
            self.assertFalse(session.surreal_server.object_mapping(session.surreal_server.object_mapping(description['capabilities'])['python'])['available'])
            self.assertIn('.venv/bin/python is unavailable', str(session.surreal_server.object_mapping(session.surreal_server.object_mapping(description['capabilities'])['database'])['reason']))
            self.assertIn('mcp_servers.pse-surreal.enabled=false', command)
            observe.assert_not_called()
            with self.assertRaisesRegex(ValueError, 'Required selected database'):
                self.describe(require_database=True)

    def describe(
        self, *, state: Path = Path('/selected state'), database: bool = True,
        rust: bool = True, require_database: bool = False,
        require_rust: bool = False, arguments: list[str] | None = None,
    ) -> tuple[list[str], dict[str, object]]:
        return session.session_description(
            state=state, database=database, rust=rust,
            require_database=require_database, require_rust=require_rust,
            arguments=arguments or [],
        )

    def test_selected_endpoint_and_literal_caller_overrides(self) -> None:
        selected = Mock(port=12345, state=Path('/selected state'))
        selected.identity = {'state': '/selected state', 'invocation': 'authentic'}
        selected.selection_fence = 'a' * 64
        literal = ['-c', 'mcp_servers.pse-surreal.enabled=false', 'spaces $HOME `pwd`']
        with patch.object(session.shutil, 'which', side_effect=lambda name: '/tools/' + name), \
             patch.object(session.surreal_server, 'selected_service', return_value=selected):
            command, description = self.describe(arguments=literal)
        self.assertEqual(command[-len(literal):], literal)
        self.assertIn('mcp_servers.pse-surreal.url="http://127.0.0.1:12345/mcp"', command)
        helper = next(value for value in command if '.http_headers_helper=' in value)
        self.assertIn("'/selected state'", helper)
        self.assertIn("--expected-selection " + "a" * 64, helper)
        self.assertNotIn('Authorization', json.dumps(description))
        self.assertEqual(str(description['attachment']).split(';')[0], 'requires a real client interaction')
        selected.recheck.assert_called_once()

    def test_optional_failure_disables_database_without_fallback(self) -> None:
        with patch.object(session.shutil, 'which', return_value='/codex'), \
             patch.object(session.surreal_server, 'selected_service', side_effect=ValueError('not ready')) as observe:
            command, description = self.describe()
        self.assertIn('mcp_servers.pse-surreal.enabled=false', command)
        self.assertFalse(session.surreal_server.object_mapping(session.surreal_server.object_mapping(description['capabilities'])['database'])['available'])
        self.assertEqual(observe.call_args.args[0], Path('/selected state'))

    def test_required_failure_and_explicit_opt_out(self) -> None:
        with patch.object(session.shutil, 'which', return_value='/codex'), \
             patch.object(session.surreal_server, 'selected_service', side_effect=ValueError('not ready')) as observe:
            with self.assertRaisesRegex(ValueError, 'Required selected database'):
                self.describe(require_database=True)
            observe.reset_mock()
            command, _ = self.describe(database=False, rust=False)
            observe.assert_not_called()
        self.assertIn('mcp_servers.rust-analyzer.enabled=false', command)

    def test_missing_required_rust_fails_before_database_observation(self) -> None:
        with (
            patch.object(session.shutil, 'which', side_effect=lambda name: '/codex' if name == 'codex' else None),
            patch.object(session.surreal_server, 'selected_service') as observe,
            self.assertRaisesRegex(ValueError, 'Required rust-analyzer'),
        ):
            self.describe(require_rust=True)
        observe.assert_not_called()


if __name__ == '__main__':
    unittest.main()
