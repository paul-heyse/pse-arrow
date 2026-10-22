# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Checkout-scoped native navigation configuration and diagnostic flag controls."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls
from __future__ import annotations

import tomllib
import unittest
from pathlib import Path

from scripts import build_environment, semantic_navigation


class SemanticNavigationTests(unittest.TestCase):
    def test_literal_checkout_and_single_pinned_python_server(self) -> None:
        root = Path('/checkout space/$literal')
        selected = tomllib.loads(semantic_navigation.configuration(root))
        self.assertEqual(selected['workspace']['roots'], [str(root)])
        self.assertEqual(len(selected['lsp_servers']), 1)
        server = selected['lsp_servers'][0]
        self.assertEqual(server['command'], str(root / '.venv/bin/pyrefly'))
        self.assertEqual(server['args'], ['lsp'])
        self.assertIn('**/*.pyi', server['file_patterns'])

    def test_diagnostic_flags_preserve_explicit_linker_and_nested_selection(self) -> None:
        chosen = {'CARGO_ENCODED_RUSTFLAGS': '-C\x1flinker=clang\x1f-C\x1flink-arg=-fuse-ld=lld',
                  'CARGO_BUILD_JOBS': '16', 'RUSTC_WRAPPER': ''}
        selected = build_environment.configure(build_environment.ROOT, chosen, taskdump=True)
        self.assertTrue(selected['CARGO_ENCODED_RUSTFLAGS'].startswith(chosen['CARGO_ENCODED_RUSTFLAGS']))
        self.assertEqual(selected['CARGO_ENCODED_RUSTFLAGS'].split('\x1f')[-2:], ['--cfg', 'tokio_unstable'])
        self.assertEqual(build_environment.configure(build_environment.ROOT, selected, taskdump=True), selected)
        self.assertEqual(selected['CARGO_BUILD_JOBS'], '16')
        self.assertNotIn('CARGO_TARGET_DIR', selected)
        self.assertNotIn('CARGO_ENCODED_RUSTFLAGS', build_environment.configure(build_environment.ROOT, {'RUSTC_WRAPPER': ''}))


if __name__ == '__main__':
    unittest.main()
