# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Retrospective scope, inherited deduplication and nonsecret projection controls."""
# ruff: noqa: PT009 -- stdlib setup controls
from __future__ import annotations

import datetime as dt
import json
import tempfile
import unittest
from pathlib import Path

from scripts import session_retrospective


class RetrospectiveTests(unittest.TestCase):
    def test_exact_checkout_time_and_inherited_record_deduplication(self) -> None:
        now = dt.datetime(2026, 10, 22, 20, tzinfo=dt.UTC)
        stamp = now.isoformat()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            partition = root / '2026/10/22'
            partition.mkdir(parents=True)
            meta = {'type': 'session_meta', 'timestamp': stamp, 'payload': {'cwd': '/checkout'}}
            call = {'type': 'response_item', 'timestamp': stamp, 'payload': {
                'type': 'function_call', 'name': 'exec_command', 'arguments': 'SECRET_TOKEN=do-not-project'}}
            failure = {'type': 'response_item', 'timestamp': stamp, 'payload': {
                'type': 'function_call_output', 'output': '{"exit_code": 101, "output": "could not compile SECRET"}'}}
            old = {**failure, 'timestamp': '2026-10-20T20:00:00+00:00'}
            for name in ['a', 'b']:
                (partition / (name + '.jsonl')).write_text(''.join(json.dumps(item) + '\n' for item in [meta, call, failure, old]))
            (partition / 'foreign.jsonl').write_text(json.dumps({**meta, 'payload': {'cwd': '/foreign'}}) + '\n' + json.dumps(failure) + '\n')
            observed = session_retrospective.inspect(root, Path('/checkout'), now=now)
            self.assertEqual(observed['tool_calls'], {'exec_command': 1})
            self.assertEqual(observed['observed_record_classes'], {'observed_exit_101': 1, 'compile_failure': 1})
            self.assertEqual(observed['inherited_duplicates_removed'], 3)
            self.assertEqual(len(observed['sources']), 2)
            self.assertNotIn('SECRET', json.dumps(observed))


if __name__ == '__main__':
    unittest.main()
