# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The Cantera mapping against the real staged data: species forms whose aggregation the phases
that include an entry give, composition and charge, the five thermo models as parameter sets under
one parameterization for each file (with the unit conversions of the Shomate coefficients, the
defaults of the constant-cp model and the points of a piecewise-Gibbs block), the conventions of the
Cantera code, and the coverage."""

from __future__ import annotations

import math
from collections import Counter
from collections.abc import Iterator

import pytest
from thermochem_support import Run, run_source, staged, staged_directory

pytestmark = pytest.mark.skipif(
    staged_directory("cantera") is None,
    reason="Cantera is not acquired and staged here (run `just tk-acquire cantera` and `just tk-read cantera`)",
)

GRI = "data/gri30.yaml"


@pytest.fixture(scope="module")
def run(tmp_path_factory: pytest.TempPathFactory) -> Iterator[Run]:
    yield run_source(tmp_path_factory.mktemp("canonical"), "cantera")


def entity(run: Run, locator: str):  # noqa: ANN201
    found, _ = run.claims()
    return next(c for c in found if c.local_key == locator)


def species_locator(artifact: str, name: str) -> str:
    return next(
        str(r["_locator"])
        for r in staged("cantera", "species")
        if r["_artifact"] == artifact and r["name"] == name
    )


def test_an_entry_is_a_species_form_keyed_by_its_locator_with_the_aggregation_of_its_phases(
    run: Run,
) -> None:
    claim = entity(run, species_locator(GRI, "CH4"))
    assert claim.aggregation == "gas" and claim.entity_class == "species_form"
    found, _ = run.claims()
    assert Counter(c.aggregation for c in found)["gas"] > 2000
    assert {c.aggregation for c in found} == {"gas", "surface", "liquid", "fluid"}
    assert claim.local_key == f"{GRI}#/species/{int(claim.local_key.rsplit('/', 1)[1])}"


def test_the_charge_is_the_pseudo_element_e_with_its_sign_converted(run: Run) -> None:
    ions = [
        r for r in staged("cantera", "species")
        if r["_artifact"] == "data/gri30_ion.yaml" and r["name"] in ("H3O+", "HCO+")
    ]
    assert ions
    for row in ions:
        assert entity(run, str(row["_locator"])).stated_charge == 1
    assert entity(run, species_locator(GRI, "CH4")).stated_charge == 0


def test_an_entry_no_phase_includes_and_one_of_an_undecodable_model_are_held_with_the_reason(
    run: Run,
) -> None:
    held = [r for r in run.held() if r["source_table"] == "species"]
    reasons = Counter(
        "no phase" if "no phase of the tree includes" in str(r["detail"])
        else "undecoded" if "has no rule for thermo" in str(r["detail"])
        else "different" if "phases of different" in str(r["detail"])
        else "other"
        for r in held
    )
    assert reasons["no phase"] > 300 and reasons["undecoded"] > 50
    assert {r["reason"] for r in held} == {"missing_convention"}
    found, _ = run.claims()
    assert not any(c.local_key.startswith("data/nasa_condensed.yaml") for c in found)


def test_a_nasa7_block_is_a_set_with_its_pieces_under_the_parameterization_of_its_file(
    run: Run,
) -> None:
    locator = species_locator(GRI, "CH4")
    block = next(r for r in staged("cantera", "species_thermo") if r["species_locator"] == locator)
    pieces = sorted(
        (r for r in staged("cantera", "thermo_pieces") if r["thermo_locator"] == block["_locator"]),
        key=lambda r: r["piece_index"],
    )
    stored = [p for p in run.table("param.nasa7__pure__piece")
              if math.isclose(p["a1"], pieces[0]["coefficients"][0], rel_tol=1e-15)
              and math.isclose(p["T_low"], pieces[0]["temperature_low"])]
    assert stored
    piece = stored[0]
    assert [piece[f"a{k}"] for k in range(1, 8)] == pieces[0]["coefficients"]
    keys = {r["key"] for r in run.table("tk.parameterization")}
    assert f"cantera_{GRI}" in keys and len(keys) > 40
    assert all(key.startswith("cantera_") for key in keys)


def test_a_shomate_block_is_converted_from_t_over_a_kilokelvin_to_kelvin_slots(run: Run) -> None:
    piece = next(
        r for r in staged("cantera", "thermo_pieces")
        if r["coefficient_count"] == 7
        and any(
            t["_locator"] == r["thermo_locator"] and t["model"] == "Shomate"
            for t in staged("cantera", "species_thermo")
        )
    )
    a, b, c, d, e, f, g = piece["coefficients"]
    stored = [
        p for p in run.table("param.shomate__pure__piece")
        if math.isclose(p["A"], a, rel_tol=1e-14) and math.isclose(p["T_low"], piece["temperature_low"])
    ]
    assert stored
    found = stored[0]
    assert math.isclose(found["B"], b / 1e3, rel_tol=1e-14)
    assert math.isclose(found["C"], c / 1e6, rel_tol=1e-14)
    assert math.isclose(found["D"], d / 1e9, rel_tol=1e-14)
    assert math.isclose(found["E"], e * 1e6, rel_tol=1e-14)
    assert math.isclose(found["F"], f * 1e3, rel_tol=1e-14)  # kJ/mol to J/mol
    assert math.isclose(found["G"], g, rel_tol=1e-14)


def test_a_constant_cp_block_takes_the_documented_defaults_and_states_its_own_units(run: Run) -> None:
    sets = run.table("param.constant_cp__pure")
    assert sets
    assert {s["T0"] for s in sets} <= {298.15, 200.0, 1000.0}
    stored = {(round(s["h0"], 1), round(s["s0"], 3)) for s in sets}
    assert (0.0, 0.0) in stored  # a block that states zeros, or none
    assert any(math.isclose(h, -2040.0, abs_tol=1e-6) for h, _ in stored)  # -2.04 kJ/mol
    assert any(math.isclose(h, 26.9 * 4184.0, abs_tol=1e-6) for h, _ in stored)  # 26.9 kcal/mol
    assert any(math.isclose(s0, 12.679, abs_tol=1e-9) for _, s0 in stored)  # 12.679 J/mol/K


def test_a_piecewise_gibbs_block_keeps_its_points_in_ascending_temperature(run: Run) -> None:
    sets = run.table("param.piecewise_gibbs__pure")
    points = run.table("param.piecewise_gibbs__pure__point")
    assert sets and points
    by_set: dict[object, list[dict[str, object]]] = {}
    for point in points:
        by_set.setdefault(point["set_id"], []).append(point)
    for found in by_set.values():
        ordered = sorted(found, key=lambda p: p["n"])
        assert [p["n"] for p in ordered] == list(range(1, len(ordered) + 1))
        assert [p["T"] for p in ordered] == sorted(p["T"] for p in ordered)
    assert all(math.isclose(s["p_ref"], 101325.0) or math.isclose(s["p_ref"], 1e5) for s in sets)


def test_the_convention_set_states_the_constants_of_the_cantera_code(run: Run) -> None:
    (conventions,) = run.table("tk.convention_set")
    assert math.isclose(conventions["gas_constant"], 8.31446261815324, rel_tol=1e-15)
    assert math.isclose(conventions["avogadro_constant"], 6.02214076e23, rel_tol=1e-15)
    (reference,) = run.table("tk.energy_reference")
    assert math.isclose(reference["pressure"], 101325.0)


def test_a_block_that_states_a_number_without_a_unit_or_a_limit_is_held(run: Run) -> None:
    held = [r for r in run.held() if "which the mapping does not map yet" in str(r["detail"])]
    assert held
    assert any("`h0` states" in str(r["detail"]) for r in held)
    assert any("`T_min` states" in str(r["detail"]) for r in held)


def test_every_row_ends_in_a_state_and_nothing_is_unexplained(run: Run) -> None:
    for table, states in run.coverage.counts.items():
        assert sum(states.values()) == len(staged("cantera", table))
    assert run.coverage.total("unmapped") == 0
    assert set(run.coverage.by_reason()) <= {"missing_convention", "unknown_subject"}
    counts = run.coverage.counts
    assert counts["species_thermo"].get("mapped_with_loss", 0) > 2500
    assert counts["reactions"] == {"deferred": len(staged("cantera", "reactions"))}
    assert counts["file_sections"] == {"out_of_scope": len(staged("cantera", "file_sections"))}
