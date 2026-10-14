# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Manual entered-pull/SIGINT control for an exclusively owned canonical fixture.

Requires the explicit two-file receipt from pse-canonical-recovery
--seed-signal-results and a force-validated installed extension. The selected
private fixture's results are collected after cancellation and transport drain;
this control never stops an unselected server instance.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import msgspec

import pse
from pse.tests.test_canonical_results import _assert_owned_delayed_canonical_sigint


class SignalResultFixture(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    version: int
    event: str
    state: str
    database: str
    run: str
    attempt: str
    relation: str
    terminal_class: str
    first_page_ordinals: tuple[int, int]
    file_instances: tuple[str, str]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", required=True, type=Path)
    parser.add_argument("--fixture-receipt", required=True, type=Path)
    parser.add_argument("--exclusive-owned-instance", required=True)
    parser.add_argument("--spill-directory", required=True, type=Path)
    options = parser.parse_args()
    fixture = msgspec.json.decode(options.fixture_receipt.read_bytes(), type=SignalResultFixture)
    assert fixture.version == 1 and fixture.event == "signal-results-seeded"
    assert Path(fixture.state).resolve(strict=True) == options.state.resolve(strict=True)
    assert fixture.terminal_class == "partial"
    assert fixture.relation == "runtime.solve_metrics"
    assert fixture.first_page_ordinals == (0, 1)
    assert fixture.file_instances[0] != fixture.file_instances[1]
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
        database=fixture.database,
        run=fixture.run,
        attempt=fixture.attempt,
        relation=fixture.relation,
        file_instances=fixture.file_instances,
        owned_instance=options.exclusive_owned_instance,
    )
    sys.stdout.write(msgspec.json.encode(outcome).decode() + "\n")


if __name__ == "__main__":
    main()
