# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Original invocation fallback readers do not predict admission or settlement."""
from __future__ import annotations
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from scripts import test_resources
from scripts.capabilities.rust_references import read
from scripts.capabilities.rust_test_contracts import InvocationReference

class ReferenceTests(unittest.TestCase):
    def test_absence_is_unconfirmed_and_real_registry_identity_is_required(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            output = root / 'build/selected'
            self.assertEqual(read(root, str(output)).outcome, 'not_observed')
            output.mkdir(parents=True)
            with patch.dict(os.environ, {'PSE_TEST_RESOURCE_STATE': str(root / 'registry')}), patch.object(test_resources.host, 'inherit', return_value=None):
                invocation = test_resources.invocation('rust', [])
                reference = InvocationReference(invocation=str(invocation), registry=str(root / 'registry'), ownership='owned', terminal_owner='native runner')
                (output / 'invocation-reference.json').write_text(reference.model_dump_json())
                observed = read(root, str(output))
                self.assertEqual(observed.outcome, 'issued')
                self.assertEqual(observed.reference, reference)
                self.assertEqual(observed.reader, 'product_run_read')
                self.assertIsNotNone(observed.source_sha256)
                self.assertIn('do not establish settlement', observed.settlement)
                value = json.loads(reference.model_dump_json())
                value['invocation'] = str(root / 'foreign.json')
                (output / 'invocation-reference.json').write_text(json.dumps(value))
                with self.assertRaisesRegex(ValueError, 'does not belong'):
                    read(root, str(output))

    def test_path_and_record_refusals_do_not_read_unbounded_or_foreign_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            output = root / 'build/selected'
            output.mkdir(parents=True)
            source = output / 'invocation-reference.json'
            source.write_text('x' * 17000)
            with self.assertRaisesRegex(ValueError, 'bounded'):
                read(root, str(output))
            source.unlink()
            source.symlink_to(root / 'missing')
            with self.assertRaises(OSError):
                read(root, str(output))
            with self.assertRaises(ValueError):
                read(root, '../foreign')
