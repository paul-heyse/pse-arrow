# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Checkout-scoped native navigation configuration and diagnostic flag controls."""
# ruff: noqa: PT009 -- stdlib setup controls
from __future__ import annotations

import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

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

    def test_rust_preserves_prepared_compiler_native_selection_without_secrets(self) -> None:
        root = Path(__file__).resolve().parents[2]
        environment = {"RUSTC_WRAPPER": str(root / "scripts/sccache"),
            "PSE_SCCACHE_BINARY": "/selected/sccache", "SCCACHE_SERVER_UDS": "/selected/cache.sock",
            "SCCACHE_CLIENT_SIDE": "1", "CARGO_ENCODED_RUSTFLAGS": "--cfg\x1ftokio_unstable",
            "IPOPT_DIR": "/selected/native", "PSE_NATIVE_OFF": "petsc",
            "PSE_HOST_ALLOCATION": "/selected/allocation", "PSE_NATIVE_OPERATION": "/selected/operation",
            "SYMBOLICA_LICENSE": "must-not-forward", "AWS_SECRET_ACCESS_KEY": "must-not-forward",
            "MCPLS_TEST_SECRET": "must-not-forward"}
        with patch.object(semantic_navigation.shutil, "which", return_value="/selected/rust-analyzer"):
            config = tomllib.loads(semantic_navigation.configuration(root, "rust", environment))
        server = config["lsp_servers"][0]
        selected = server["env"]
        self.assertEqual(selected["RUSTC_WRAPPER"], environment["RUSTC_WRAPPER"])
        self.assertEqual(selected["SCCACHE_SERVER_UDS"], environment["SCCACHE_SERVER_UDS"])
        self.assertEqual(selected["CARGO_ENCODED_RUSTFLAGS"], environment["CARGO_ENCODED_RUSTFLAGS"])
        self.assertEqual(selected["IPOPT_DIR"], environment["IPOPT_DIR"])
        self.assertEqual(selected["PSE_HOST_ALLOCATION"], environment["PSE_HOST_ALLOCATION"])
        self.assertNotIn("SYMBOLICA_LICENSE", selected)
        self.assertNotIn("AWS_SECRET_ACCESS_KEY", selected)
        self.assertNotIn("MCPLS_TEST_SECRET", selected)
        self.assertEqual(server["initialization_options"]["cargo"]["features"], ["pse-relations/force-validate"])
        self.assertTrue(server["initialization_options"]["cargo"]["targetDir"])
        self.assertEqual(build_environment.language_server_environment({}), {})

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
