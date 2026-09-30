# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

from __future__ import annotations

from collections.abc import Iterator

from pathlib import Path

import pytest
from test_acquire_support import unseal

from thermo_knowledge.testing import TestDatabase

_FAILED: pytest.StashKey[bool] = pytest.StashKey()


@pytest.hookimpl(wrapper=True)
def pytest_runtest_makereport(item: pytest.Item, call: pytest.CallInfo[None]) -> Iterator[None]:
    report = yield
    # Expose each phase's outcome so fixtures know whether the test failed.
    item.stash.setdefault(_FAILED, False)
    if report.failed:
        item.stash[_FAILED] = True
    return report


@pytest.fixture
def test_database(request: pytest.FixtureRequest) -> Iterator[TestDatabase]:
    """A fresh empty database on the configured server, dropped after the test
    (kept when the test failed and PSE_THERMO_KEEP_FAILED_DATABASES=1)."""
    database = TestDatabase.create()
    try:
        yield database
    finally:
        database.remove(failed=request.node.stash.get(_FAILED, False))


@pytest.fixture(autouse=True)
def _unseal_acquired_stores(tmp_path: Path) -> Iterator[None]:
    """A completed acquisition is read-only; make what a test acquired writable again so its
    temporary store can be removed."""
    yield
    unseal(tmp_path)
