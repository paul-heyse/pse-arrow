# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual explicitly selected diagnostic Python extension and executor smoke."""
from __future__ import annotations

import hashlib
import json
import sys
import tempfile
from pathlib import Path

import pse
from pse._build import compiled_features
from pse.contracts.documents import ClosedDuration, TaskDumpRequest


def main() -> int:
    features = compiled_features()
    if not {'force-validate', 'diagnostic-taskdump'} <= set(features):
        raise ValueError('select the separately built diagnostic-taskdump correctness extension')
    binary = Path(sys.modules['pse._native'].__file__)
    if '.venv-diagnostic-taskdump' not in binary.parts:
        raise ValueError('diagnostic extension must be installed separately from the editable checkout')
    with tempfile.TemporaryDirectory(prefix='pse-taskdump-smoke-') as directory:
        runtime = pse.Runtime(pse.EngineSettings(
            memory_limit_bytes=3 << 30, threads=2, target_partitions=1, spill_dir=directory,
            max_spill_bytes=1 << 30, batch_size=7, math_workspace_bytes=1 << 30,
            math_worker_bytes=256 << 20, math_artifact_bytes=128 << 20,
            cache=pse.CacheSettings(working_bytes=128 << 20, concurrent_loads=2,
                                    inflight_bytes=32 << 20, inspection_bytes=4 << 20),
        ), ephemeral=True)
        try:
            if runtime.task_dump_status() is not None:
                raise ValueError('new process unexpectedly has an issued task dump')
            result = runtime.task_dump(TaskDumpRequest(wait=ClosedDuration(secs=5, nanos=0), maximum_tasks=64, maximum_bytes=65536))
            status = runtime.task_dump_status()
            if status is None or status.request != result.request or result.target_state != 'completed':
                raise ValueError('actual executor cooperative request did not complete under independent observation')
            print(json.dumps({'passed': 1, 'failed': 0, 'binary': str(binary),
                              'sha256': hashlib.file_digest(binary.open('rb'), 'sha256').hexdigest(),
                              'features': features, 'executor': result.executor,
                              'request': result.request, 'task_count': len(result.tasks),
                              'scope': 'actual Python-owned executor and retained status; pending/blocked/observer lifetime controlled in Rust tests'}))
        finally:
            runtime.close()
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
