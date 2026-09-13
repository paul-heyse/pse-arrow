# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The parity pre-flight (plan §5, blueprint §24, §25).

Runs first (hence the name) and fails, never skips, when the environment cannot
produce a meaningful comparison: the wrong IDAES, no Ipopt, or a Pyomo that
cannot see the solver. Every later parity assertion is only as trustworthy as
this one.

The observed capabilities are written to ``artifacts/probe_host.json`` so a CI
run's evidence says which solver produced its numbers.
"""

import json
from pathlib import Path

import msgspec
import pytest
from pyomo.environ import SolverFactory

from pse import HostCapabilities, probe_host

#: The parity reference; moving it is an ADR.
EXPECTED_IDAES_VERSION = "2.12.0"

#: Ipopt 3.11 (the common system build) produces different iteration counts.
EXPECTED_IPOPT_SERIES = "3.14."

#: python/pse/parity/tests/test_00_preflight.py -> pse-arrow/
REPO_ROOT = Path(__file__).resolve().parents[4]

ARTIFACTS = REPO_ROOT / "artifacts"


@pytest.fixture(scope="module")
def host() -> HostCapabilities:
    """Probe the machine once for the whole pre-flight.

    Returns:
        The observed host capabilities.
    """
    return probe_host()


@pytest.mark.unit
@pytest.mark.parity
def test_ipopt_is_on_path(host: HostCapabilities) -> None:
    assert host.ipopt_on_path, (
        "ipopt is not on PATH; parity never skips. Use the solver container "
        "(`just bootstrap-solvers`) or build docker/solvers/build.sh natively."
    )


@pytest.mark.unit
@pytest.mark.parity
def test_ipopt_is_the_pinned_series(host: HostCapabilities) -> None:
    assert host.ipopt_version is not None, "`ipopt -v` did not report a version"
    assert host.ipopt_version.startswith(EXPECTED_IPOPT_SERIES), (
        f"ipopt {host.ipopt_version} is not {EXPECTED_IPOPT_SERIES}x; "
        "iteration counts are only comparable against the pinned build"
    )


@pytest.mark.unit
@pytest.mark.parity
def test_idaes_is_the_parity_pin(host: HostCapabilities) -> None:
    assert host.idaes_version == EXPECTED_IDAES_VERSION, (
        f"idaes-pse is {host.idaes_version}, expected {EXPECTED_IDAES_VERSION}"
    )


@pytest.mark.unit
@pytest.mark.parity
def test_pyomo_can_see_ipopt() -> None:
    assert SolverFactory("ipopt").available(False), (
        "Pyomo cannot see ipopt even though it is on PATH"
    )


@pytest.mark.unit
@pytest.mark.parity
def test_probe_is_recorded(host: HostCapabilities) -> None:
    ARTIFACTS.mkdir(parents=True, exist_ok=True)
    target = ARTIFACTS / "probe_host.json"
    target.write_bytes(msgspec.json.format(msgspec.json.encode(host)))
    recorded = json.loads(target.read_text(encoding="utf-8"))
    assert recorded["ipopt_version"] == host.ipopt_version
