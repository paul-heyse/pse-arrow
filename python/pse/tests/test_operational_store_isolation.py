# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Real database ownership controls for the Python store fixture."""

import gc

import pytest

import pse
from pse._build import _TestOperationalStore


@pytest.mark.component
def test_isolated_store_teardown_preserves_other_owned_store(
    inspection_settings: pse.EngineSettings,
    operational_store: pse.OperationalStore,
) -> None:
    first = pse.Runtime(inspection_settings, store=operational_store)
    owner = _TestOperationalStore()
    try:
        other_store = owner.store()
        assert other_store.url != operational_store.url
        second = pse.Runtime(inspection_settings, store=other_store)
        assert first.jobs() == ()
        assert second.jobs() == ()

        owner.remove()
        with pytest.raises(pse.InspectionError):
            pse.Runtime(inspection_settings, store=other_store)
        with pytest.raises(pse.InspectionError, match="removed"):
            owner.store()
        assert first.jobs() == ()
    finally:
        owner.remove()


@pytest.mark.component
def test_abandoned_isolated_store_owner_drops_its_database(
    inspection_settings: pse.EngineSettings,
) -> None:
    owner = _TestOperationalStore()
    store = owner.store()
    assert pse.Runtime(inspection_settings, store=store).jobs() == ()
    del owner
    gc.collect()
    with pytest.raises(pse.InspectionError):
        pse.Runtime(inspection_settings, store=store)
