"""Time full SHA-256 passes over large runtime binaries (light probe; read-only).

Usage: scripts/pse-env --resource-class light -- python3 -I docs/design_review/evidence/efficiency-alignment-codebase-2026-10-11/sha256-pass-time.py <file>...
"""
import hashlib
import sys
import time

for path in sys.argv[1:]:
    for attempt in range(2):
        start = time.perf_counter()
        with open(path, "rb") as handle:
            hashlib.file_digest(handle, "sha256")
        print(path.rsplit("/", 1)[-1], "pass", attempt + 1, round(time.perf_counter() - start, 3), "s")
