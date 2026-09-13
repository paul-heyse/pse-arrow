# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The host capability probe is typed and stdlib-only (blueprint §21.4).

The adapter's real pre-flight -- refuse *before* a ``ConcreteModel`` is built --
lands with the adapter in phase 1. What exists now is the probe it will consult,
and the property that matters most about it: it reports, in a typed shape, what
is actually on this machine, without importing anything optional.
"""

import platform

import msgspec
import pytest

from pse import HostCapabilities, probe_host


@pytest.mark.component
def test_probe_reports_typed_fields() -> None:
    host = probe_host()
    assert isinstance(host, HostCapabilities)
    assert isinstance(host.python_version, str)
    assert isinstance(host.platform, str)
    assert isinstance(host.ipopt_on_path, bool)
    assert host.ipopt_version is None or isinstance(host.ipopt_version, str)
    assert host.idaes_version is None or isinstance(host.idaes_version, str)
    assert host.pyomo_version is None or isinstance(host.pyomo_version, str)


@pytest.mark.component
def test_probe_reports_this_interpreter() -> None:
    assert probe_host().python_version == platform.python_version()


@pytest.mark.component
def test_capabilities_are_frozen() -> None:
    host = probe_host()
    with pytest.raises(AttributeError):
        # pyrefly: ignore[read-only]  -- refusing this is what is under test
        host.ipopt_on_path = True


@pytest.mark.component
def test_ipopt_version_accompanies_ipopt_on_path() -> None:
    host = probe_host()
    if not host.ipopt_on_path:
        assert host.ipopt_version is None


@pytest.mark.component
def test_capabilities_round_trip_as_json() -> None:
    host = probe_host()
    encoded = msgspec.json.encode(host)
    assert msgspec.json.decode(encoded, type=HostCapabilities) == host
