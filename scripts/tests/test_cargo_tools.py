# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Protocol pins are shared selections, not an implicit tool upgrade."""
import unittest
from unittest.mock import patch
from scripts import cargo_tools


class CargoToolsTests(unittest.TestCase):
    def test_pins_reject_override_but_leave_other_explicit_versions(self):
        with patch.object(cargo_tools, "versions", return_value={"cargo-hack": "0.6.45"}):
            self.assertEqual(cargo_tools.specifications(["cargo-hack", "cargo-nextest@0.9.0"]),
                             ["cargo-hack@0.6.45", "cargo-nextest@0.9.0"])
            with self.assertRaisesRegex(ValueError, "declared version"):
                cargo_tools.specifications(["cargo-hack@0.6.44"])
            with self.assertRaisesRegex(ValueError, "invalid Cargo"):
                cargo_tools.specifications(["cargo-hack;false"])
