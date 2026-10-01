# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The CoolProp oracle and the first real case: the saturation-pressure form evaluated from the
stored CoolProp ancillary coefficients against CoolProp's own evaluation of its ancillary.

These tests read `pse_thermo` (they write nothing to it or to the store) and need CoolProp in
this environment and the acquired source tree; each is skipped, with the reason, when one is
missing."""

from __future__ import annotations

import json
import math
import re
from collections.abc import Iterator
from pathlib import Path

import numpy as np
import pytest

from mapping_support import real_declaration
from thermo_knowledge import config, db
from thermo_knowledge.qualify import harness, persist
from thermo_knowledge.qualify.case import load_case
from thermo_knowledge.qualify.run import CaseOutcome, Context, run_case

CASE = config.TREE_DIR / "qualification" / "vapor_pressure_exp_series_tau__coolprop.toml"
EPS = float(np.finfo(float).eps)


def _skip_reason() -> str | None:
    try:
        import CoolProp  # noqa: F401
    except ImportError:
        return "CoolProp is not installed in this environment"
    try:
        with db.connect() as conn:
            row = conn.execute(
                "SELECT resolved_pin FROM prov.carrier WHERE manifest_id = 'coolprop'"
            ).fetchone()
            sets = conn.execute(
                "SELECT count(*) FROM tk.parameterization WHERE key = 'coolprop_saturation_ancillaries'"
            ).fetchone()
    except Exception as error:  # the server, the database or the schemas are not there
        return f"pse_thermo is not readable ({type(error).__name__})"
    if row is None or sets is None or sets[0] == 0:
        return "pse_thermo does not hold the CoolProp slice: run `tk build` after `tk map coolprop`"
    if not _source_tree(row[0]).is_dir():
        return "the acquired CoolProp source is not in the raw store"
    return None


def _source_tree(pin: str) -> Path:
    return config.raw_dir() / "coolprop" / pin / "tree"


REASON = _skip_reason()
pytestmark = pytest.mark.skipif(REASON is not None, reason=REASON or "")


def source_fluids() -> dict[str, dict[str, object]]:
    """The `pS` ancillary of every acquired fluid that has one, by `INFO/NAME`."""
    with db.connect() as conn:
        (pin,) = conn.execute(
            "SELECT resolved_pin FROM prov.carrier WHERE manifest_id = 'coolprop'"
        ).fetchone()  # type: ignore[misc]
    found: dict[str, dict[str, object]] = {}
    for path in sorted((_source_tree(pin) / "dev" / "fluids").glob("*.json")):
        document = json.loads(path.read_text(encoding="utf-8"))
        ancillary = document["ANCILLARIES"].get("pS")
        if ancillary is not None:
            found[document["INFO"]["NAME"]] = ancillary
    return found


def evaluate_ancillary(a: dict[str, object], T: float) -> float:
    """CoolProp's documented pS equation, written here from the acquired JSON."""
    theta = 1 - T / a["T_r"]  # type: ignore[operator]
    total = sum(n * theta**t for n, t in zip(a["n"], a["t"], strict=True))  # type: ignore[arg-type]
    return a["reducing_value"] * math.exp(a["T_r"] / T * total)  # type: ignore[operator]


def test_the_installed_wheel_carries_the_ancillaries_of_the_acquired_source() -> None:
    from CoolProp import CoolProp as CP

    assert CP.get_global_param_string("version") == "8.0.0"
    fluids = source_fluids()
    assert len(fluids) >= 123
    for name, ancillary in fluids.items():
        bundled = json.loads(CP.get_fluid_param_string(name, "JSON"))[0]["ANCILLARIES"]["pS"]
        assert bundled == ancillary, name


def test_the_harness_evaluates_the_ancillary_equation_and_not_the_equation_of_state() -> None:
    from CoolProp import CoolProp as CP

    ancillary = source_fluids()["Water"]
    T = [300.0, 400.0, 500.0, 640.0]
    request = harness.Request(
        "coolprop",
        "saturation_pressure_ancillary",
        {"T": "K"},
        "Pa",
        (harness.SubjectRequest(("Water",), {"T": T}, len(T)),),
    )
    result = harness.run(request, "core", oracles=config.TREE_DIR / "oracles")
    assert (result.library, result.version) == ("CoolProp", "8.0.0")
    for point, temperature in zip(result.subjects[0], T, strict=True):
        assert point.status == "ok"
        assert point.value == pytest.approx(evaluate_ancillary(ancillary, temperature), rel=1e-12)
        # the equation of state's saturation solve answers differently (by far more than rounding)
        solved = CP.PropsSI("P", "T", temperature, "Q", 0, "Water")
        assert abs(point.value - solved) / solved > 1e-7  # type: ignore[operator]


def test_above_the_reducing_temperature_the_library_answers_nan_and_an_unknown_fluid_errors() -> (
    None
):
    request = harness.Request(
        "coolprop",
        "saturation_pressure_ancillary",
        {"T": "K"},
        "Pa",
        (
            harness.SubjectRequest(("Water",), {"T": [700.0]}, 1),
            harness.SubjectRequest(("NoSuchFluid",), {"T": [300.0, 310.0]}, 2),
        ),
    )
    water, unknown = harness.run(request, "core", oracles=config.TREE_DIR / "oracles").subjects
    assert water[0].status == "nan" and water[0].value is None
    assert [p.status for p in unknown] == ["error", "error"] and "NoSuchFluid" in (
        unknown[0].message or ""
    )


@pytest.fixture(scope="module")
def runs(tmp_path_factory: pytest.TempPathFactory) -> Iterator[dict[str, CaseOutcome]]:
    """The committed case, and the same case with the ends of each range left out, evaluated
    against `pse_thermo` without recording anything."""
    scratch = tmp_path_factory.mktemp("coolprop-case")
    variant = scratch / "interior.toml"
    variant.write_text(
        re.sub(r"inset = .*", "inset = 0.01", CASE.read_text(encoding="utf-8")), encoding="utf-8"
    )
    context = Context(real_declaration(), config.database_url(), scratch / "canonical", force=True)
    original = persist.load_live
    persist.load_live = lambda *args, **kwargs: None  # type: ignore[assignment]
    try:
        with db.connect() as conn:
            yield {
                "committed": run_case(context, conn, load_case(CASE)),
                "interior": run_case(context, conn, load_case(variant)),
            }
    finally:
        persist.load_live = original  # type: ignore[assignment]


def test_the_committed_case_compares_every_fluid_at_25_points_and_the_library_answers_them_all(
    runs: dict[str, CaseOutcome],
) -> None:
    report = runs["committed"].report
    assert report["basis"] == "source_library" and report["library"] == "CoolProp"
    assert (report["subjects"], report["points"], report["invalid"]) == (123, 123 * 25, 0)
    assert not [s for s in report["per_subject"] if s["refused"]]  # type: ignore[union-attr]
    # the tolerance and its rationale are in the case file's header; the ends of each range, where
    # theta is a few ulp, are among the points and agree within it
    assert report["relative_tolerance"] == 1e-10
    assert runs["committed"].outcome == "passed" and report["failed"] == 0
    # every point of the grid lies inside the fitted range the grid was spread within
    assert report["validity"] == {
        "kind": "fitted_range",
        "outside_policy": "compare",
        "inside": 123 * 25,
        "outside": 0,
        "undetermined": 0,
        "not_stated": 0,
        "excluded": 0,
    }


def test_every_deviation_inside_each_range_is_within_a_rounding_bound_of_its_coefficients(
    runs: dict[str, CaseOutcome],
) -> None:
    """Where the two evaluations differ, it is the conditioning of the equation, not the
    mapping: a bound from the coefficients themselves (the sum of the absolute terms of the
    exponent, times a few units in the last place) holds for every fluid away from the reducing
    temperature. At the reducing temperature, where theta is a few ulp, the answer is rounding
    noise in both evaluations."""
    report = runs["interior"].report
    ancillaries = source_fluids()
    assert report["points"] == 123 * 25 and report["invalid"] == 0
    for subject in report["per_subject"]:  # type: ignore[union-attr]
        a = ancillaries[subject["key"][0]]
        bound = 1e-12
        T_r = a["T_r"]  # type: ignore[assignment]
        for T in np.linspace(
            a["Tmin"] + 0.01 * (a["Tmax"] - a["Tmin"]),
            a["Tmax"] - 0.01 * (a["Tmax"] - a["Tmin"]),
            25,
        ):  # type: ignore[operator]
            theta = 1 - T / T_r  # type: ignore[operator]
            terms = [abs(T_r / T * n * theta**t) for n, t in zip(a["n"], a["t"], strict=True)]  # type: ignore[operator,arg-type]
            bound = max(bound, 2 * EPS * sum(terms))
        deviation = subject["worst_relative_deviation"]
        assert deviation is not None and deviation <= bound, (subject["key"], deviation, bound)
