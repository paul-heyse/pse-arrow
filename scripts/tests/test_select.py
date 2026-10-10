# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Direct selector entry preserves native selection without PYTHONPATH repair."""

# ruff: noqa: PT009

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class SelectorTests(unittest.TestCase):
    def test_direct_and_module_selection_match(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            scratch = Path(directory)
            witness = scratch / "cargo"
            witness.write_text(
                f"#!{sys.executable}\n"
                "import json, os, pathlib, sys\n"
                "args = sys.argv[1:]\n"
                "if args[0] == 'metadata':\n"
                "    print(json.dumps({'packages': [\n"
                "        {'id': 'pse-structural', 'name': 'pse-structural', 'features': {}},\n"
                "        {'id': 'pse-relations', 'name': 'pse-relations', 'features': {'force-validate': []}},\n"
                "        {'id': 'arrow-data', 'name': 'arrow-data', 'features': {'force_validate': []}}]}))\n"
                "elif '--unit-graph' in args:\n"
                "    def unit(name, deps, features):\n"
                "        return {'pkg_id': name, 'target': {'name': name, 'kind': ['lib'], 'src_path': '/' + name}, 'dependencies': [{'index': i} for i in deps], 'features': features, 'platform': None, 'mode': 'test'}\n"
                "    enabled = any('pse-relations/force-validate' in item for item in args)\n"
                "    print(json.dumps({'version': 1, 'roots': [0, 1], 'units': [unit('pse-structural', [], []), unit('pse-relations', [2], []), unit('arrow-data', [], ['force_validate'] if enabled else [])]}))\n"
                "else:\n"
                "    pathlib.Path(os.environ['SELECT_WITNESS']).write_text(json.dumps(args))\n"
            )
            witness.chmod(0o700)
            output = scratch / "argv.json"
            env = dict(os.environ)
            env.pop("PYTHONPATH", None)
            env.update(
                PATH=f"{scratch}:{env['PATH']}",
                PSE_NEXTEST_ACTION="list --message-format json",
                SELECT_WITNESS=str(output),
            )
            arguments = ["unit", "pse-structural", "a::b", "-p", "pse-relations"]
            seen = []
            for entry in [[str(ROOT / "scripts/select.py")], ["-m", "scripts.select"]]:
                subprocess.run(
                    [sys.executable, *entry, *arguments],
                    cwd=ROOT,
                    env=env,
                    check=True,
                    capture_output=True,
                )
                seen.append(json.loads(output.read_text()))
            self.assertEqual(seen[0], seen[1])
            self.assertIn("pse-relations/force-validate", seen[0])
            self.assertIn(
                "(test(a::b)) & (package(pse-relations) | package(pse-structural))",
                seen[0],
            )


if __name__ == "__main__":
    unittest.main()
