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
                "pathlib.Path(os.environ['SELECT_WITNESS']).write_text(json.dumps(sys.argv[1:]))\n"
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
                "(test(a::b)) & (package(pse-structural) | package(pse-relations))",
                seen[0],
            )


if __name__ == "__main__":
    unittest.main()
