# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pure candidate annotations neither load scientific libraries nor replace codecs."""
# ruff: noqa: PT009 -- stdlib bootstrap controls

import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class AnnotationBoundaryTests(unittest.TestCase):
    """Exercise the actual candidate checker with valid and refused contracts."""

    def check_candidate(self, body: str) -> subprocess.CompletedProcess[str]:
        """Check a fresh candidate against the repository's real scalar type owner."""
        with tempfile.TemporaryDirectory() as directory:
            candidate = Path(directory)
            contracts = candidate / "python/pse/contracts"
            contracts.mkdir(parents=True)
            shutil.copyfile(
                ROOT / "python/pse/contracts/values.py", contracts / "values.py"
            )
            shutil.copyfile(
                ROOT / "python/pse/contracts/enums.py", contracts / "enums.py"
            )
            (contracts / "__init__.py").write_text(body)
            environment = dict(os.environ)
            environment.pop("LD_LIBRARY_PATH", None)
            return subprocess.run(
                [
                    sys.executable,
                    str(ROOT / "scripts/check_python_contracts.py"),
                    "--root",
                    str(candidate),
                ],
                env=environment,
                text=True,
                capture_output=True,
                check=False,
            )

    def test_identity_annotations_do_not_load_native_extension(self) -> None:
        """Real identity types remain usable for annotations with no native import."""
        result = self.check_candidate(
            "import sys\nimport attrs\nfrom pse.contracts.values import SemanticId\n"
            "@attrs.define\nclass Row:\n    key: SemanticId\n"
            "assert 'pse._native' not in sys.modules\n"
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_unconstrained_annotations_still_fail(self) -> None:
        """Removing native initialization does not weaken the annotation obligation."""
        result = self.check_candidate(
            "import attrs\n@attrs.define\nclass Row:\n    values: list\n"
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("a bare list", result.stderr)

    def test_scalar_codec_execution_fails_instead_of_being_substituted(self) -> None:
        """The pure checker cannot become a second scientific identity codec."""
        result = self.check_candidate(
            "from pse.contracts.values import SemanticId\n"
            "value = SemanticId.from_hex('00000000000000000000000000000000')\n"
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("cannot execute native scalar codecs", result.stderr)


if __name__ == "__main__":
    unittest.main()
