# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Manual entered-pull/SIGINT control for an exclusively owned canonical fixture.

Requires an already activated/readmitted authentic historical fixture and a
force-validated installed extension. It creates no source, solve, server or
configuration and never stops an unselected instance.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import msgspec

import pse
from pse.tests.test_canonical_results import _assert_owned_delayed_canonical_sigint


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", required=True, type=Path)
    parser.add_argument("--database", required=True)
    parser.add_argument("--run", required=True)
    parser.add_argument("--attempt", required=True)
    parser.add_argument("--exclusive-owned-instance", required=True)
    parser.add_argument("--spill-directory", required=True, type=Path)
    options = parser.parse_args()
    options.spill_directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    settings = pse.EngineSettings(
        memory_limit_bytes=512 << 20,
        threads=2,
        top_consumers=4096,
        target_partitions=1,
        spill_dir=str(options.spill_directory.resolve()),
        max_spill_bytes=64 << 20,
        batch_size=2048,
        cache=pse.CacheSettings(
            working_bytes=64 << 20,
            metadata_bytes=8 << 20,
            concurrent_loads=2,
            inflight_bytes=16 << 20,
            inspection_bytes=4 << 20,
        ),
    )
    outcome = _assert_owned_delayed_canonical_sigint(
        settings,
        state=options.state,
        database=options.database,
        run=options.run,
        attempt=options.attempt,
        owned_instance=options.exclusive_owned_instance,
    )
    sys.stdout.write(msgspec.json.encode(outcome).decode() + "\n")


if __name__ == "__main__":
    main()
