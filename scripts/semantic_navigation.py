# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit maintained MCP bridge to this checkout's pinned Pyrefly LSP."""

from __future__ import annotations

import argparse
import json
import subprocess
import shutil
import tomllib
import tempfile
from pathlib import Path

from scripts import diagnostic_trace

ROOT = Path(__file__).resolve().parents[1]


def configuration(root: Path, language: str = "python") -> str:
    """Resolve execution paths once; neither import nor configuration starts a server."""
    if language not in {"python", "rust"}:
        raise ValueError("navigation language must be python or rust")
    if language == "rust":
        settings = tomllib.loads((root / "rust-analyzer.toml").read_text())
        analyzer = shutil.which("rust-analyzer")
        if analyzer is None:
            raise ValueError("pinned rust-analyzer is unavailable; install the pinned rustup component")
        native = ('[[lsp_servers]]\nlanguage_id = "rust"\ncommand = ' + json.dumps(analyzer)
                  + '\nargs = []\nfile_patterns = ["**/*.rs"]\ntimeout_seconds = 45\nrequest_timeout_seconds = 45\n'
                  + '[lsp_servers.initialization_options]\ncheck.enable = false\n')
        for group, values in settings.items():
            for key, value in values.items():
                native += f"{group}.{key} = " + json.dumps(value) + '\n'
    else:
        native = ('[[lsp_servers]]\nlanguage_id = "python"\n'
                  'command = ' + json.dumps(str(root / '.venv/bin/pyrefly')) + '\n'
                  'args = ["lsp"]\nfile_patterns = ["**/*.py", "**/*.pyi"]\n'
                  'timeout_seconds = 30\nrequest_timeout_seconds = 45\n')
    return (
        '[workspace]\nroots = ' + json.dumps([str(root)]) + '\n'
        'position_encodings = ["utf-8", "utf-16"]\n'
        'max_documents = 100\nmax_file_size = 10485760\n'
        'indexing_ready_timeout_seconds = 45\n'
        + native
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=('serve', 'describe'))
    parser.add_argument('--language', choices=('python', 'rust'), default='python')
    args = parser.parse_args()
    binary = diagnostic_trace.binary_path('mcpls')
    if not binary.is_file() or (args.language == 'python' and not (ROOT / '.venv/bin/pyrefly').is_file()):
        parser.exit(125, 'navigation unavailable: explicitly install mcpls with python -m scripts.diagnostic_trace install mcpls and sync the pinned checkout Python tools\n')
    if args.command == 'describe':
        print(json.dumps({'adapter': str(binary), 'server': str(ROOT / '.venv/bin/pyrefly') if args.language == 'python' else shutil.which('rust-analyzer'),
                          'checkout': str(ROOT), 'available': True, 'attached': 'not observed',
                          'coverage': 'Pyrefly Python/stub semantics; cross-language leads require source/catalogue owners; readiness is per real request'}))
        return 0
    # Explicit configuration avoids cwd configuration discovery. Its lifetime is the
    # client-owned bridge lifetime; no index or second type universe is introduced.
    with tempfile.TemporaryDirectory(prefix='pse-navigation-') as directory:
        config = Path(directory) / 'mcpls.toml'
        config.write_text(configuration(ROOT, args.language))
        return subprocess.call([str(binary), '--config', str(config)], cwd=ROOT)


if __name__ == '__main__':
    raise SystemExit(main())
