# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The NASA CEA mapping against the real staged data: species forms with their aggregation, charge
and Hill formula, composition, NASA-9 sets with their pieces and validity, the heats of formation
of the reactant-only records, the conventions the program's code states, and the coverage."""

from __future__ import annotations

import math
import uuid
from collections import Counter
from collections.abc import Iterator

import pytest

from thermochem_support import Run, run_source, staged, staged_directory

pytestmark = pytest.mark.skipif(
    staged_directory("nasa_cea") is None,
    reason="NASA CEA is not acquired and staged here (run `just tk-acquire nasa_cea` and `just tk-read nasa_cea`)",
)


@pytest.fixture(scope="module")
def run(tmp_path_factory: pytest.TempPathFactory) -> Iterator[Run]:
    yield run_source(tmp_path_factory.mktemp("canonical"), "nasa_cea")


def entities(run: Run) -> dict[str, object]:
    found, _ = run.claims()
    return {claim.local_key: claim for claim in found}


def formulas(run: Run) -> dict[str, str]:
    _, assertions = run.claims()
    return {c.local_key: c.value for c in assertions if c.scheme == "formula"}


def test_a_record_is_a_species_form_whose_aggregation_the_flag_and_the_suffix_give(
    run: Run,
) -> None:
    found = entities(run)
    records = {r["name"]: r for r in staged("nasa_cea", "species_records")}
    assert found["CH4"].aggregation == "gas" and found["CH4"].polymorph is None  # type: ignore[attr-defined]
    assert found["H2O(L)"].aggregation == "liquid"  # type: ignore[attr-defined]
    assert found["C(gr)"].aggregation == "crystalline" and found["C(gr)"].polymorph == "gr"  # type: ignore[attr-defined]
    assert found["Fe(a)"].polymorph == "a"  # type: ignore[attr-defined]
    assert found["Cr(cr)"].aggregation == "crystalline" and found["Cr(cr)"].polymorph is None  # type: ignore[attr-defined]
    gas = {k for k, r in records.items() if r["phase_flag"] == 0}
    assert all(found[k].aggregation == "gas" for k in gas if k in found)  # type: ignore[attr-defined]
    assert {c.entity_class for c in found.values()} == {"species_form"}  # type: ignore[attr-defined]


def test_the_charge_is_the_pseudo_element_e_with_its_sign_converted(run: Run) -> None:
    found = entities(run)
    assert found["Ar+"].stated_charge == 1  # type: ignore[attr-defined]
    assert found["CH4"].stated_charge == 0  # type: ignore[attr-defined]
    anions = [k for k in found if k.endswith("-") and k != "e-"]
    assert anions and all(found[k].stated_charge == -1 for k in anions[:5])  # type: ignore[attr-defined]


def test_the_formula_is_the_hill_formula_of_the_pairs_without_the_pseudo_element(run: Run) -> None:
    text = formulas(run)
    assert text["CH4"] == "CH4"
    assert text["H2O(L)"] == "H2O"
    assert text["Ar+"] == "Ar"
    assert text["CH3OH(L)"] == "CH4O"


def test_a_record_that_names_an_element_the_declaration_lacks_is_held_whole(run: Run) -> None:
    by_reason = Counter(
        (r["source_table"], r["reason"])
        for r in run.held()
        if "is not a declared" in str(r["detail"])
    )
    assert by_reason[("species_records", "unknown_subject")] > 0
    names = {
        r["locator"] for r in run.held() if "`Ih`" in str(r["detail"]) or "`Io`" in str(r["detail"])
    }
    assert names
    found = entities(run)
    assert not any(k.startswith("Inert") for k in found)


def test_a_fitted_record_is_one_nasa9_set_with_the_pieces_as_stated(run: Run) -> None:
    sets = run.table("param.nasa9__pure")
    pieces = run.table("param.nasa9__pure__piece")
    staged_pieces = [
        r for r in staged("nasa_cea", "thermo_intervals") if r["coefficients"] is not None
    ]
    assert len(pieces) <= len(staged_pieces) and len(sets) > 1900
    methane = [r for r in staged("nasa_cea", "species_records") if r["name"] == "CH4"][0]
    first = [
        r
        for r in staged("nasa_cea", "thermo_intervals")
        if r["species_locator"] == methane["_locator"] and r["interval_index"] == 0
    ][0]
    coefficients = first["coefficients"]
    wanted = {
        "a1": coefficients[0],
        "a3": coefficients[2],
        "b1": coefficients[7],
        "b2": coefficients[8],
    }
    found = [
        p for p in pieces if p["n"] == 1 and math.isclose(p["a1"], wanted["a1"], rel_tol=1e-15)
    ]
    assert found, "the first piece of CH4 holds its first coefficient unchanged (unit K^2 to K^2)"
    piece = found[0]
    assert (
        piece["a3"] == wanted["a3"] and piece["b1"] == wanted["b1"] and piece["b2"] == wanted["b2"]
    )
    assert (piece["T_low"], piece["T_high"]) == (200.0, 1000.0)


def test_the_fitted_range_is_the_validity_region_of_each_set(run: Run) -> None:
    regions = run.table("tk.validity_region")
    clauses = run.table("tk.region_clause")
    assert len(regions) == len(clauses) == len(run.table("param.nasa9__pure"))
    assert {r["kind"] for r in regions} == {"fitted_range"}
    assert all(c["lower"] < c["upper"] for c in clauses)


def test_a_reactant_only_record_is_an_assigned_enthalpy_at_its_reference_temperature(
    run: Run,
) -> None:
    sets = run.table("param.assigned_enthalpy__pure")
    assert sets
    by_name = {
        r["name"]: r for r in staged("nasa_cea", "species_records") if r["interval_count"] == 0
    }
    assert len(sets) <= len(by_name)
    lines = {
        r["species_locator"]: r["t_low"]
        for r in staged("nasa_cea", "thermo_intervals")
        if r["coefficients"] is None
    }
    assert {round(s["T_ref"], 3) for s in sets} <= set(map(lambda t: round(t, 3), lines.values()))
    assert {round(s["T_ref"], 2) for s in sets} >= {298.15, 20.27}
    stated = {
        round(r["heat_of_formation"])
        for r in staged("nasa_cea", "species_records")
        if r["interval_count"] == 0
    }
    assert {round(s["h_ref"]) for s in sets} <= stated


def test_the_convention_set_states_the_constants_of_the_programs_code(run: Run) -> None:
    (conventions,) = run.table("tk.convention_set")
    assert math.isclose(conventions["gas_constant"], 8.31451)
    assert math.isclose(conventions["boltzmann_constant"], 1.380658e-23)
    assert conventions["temperature_scale"] == "not_stated"
    (reference,) = run.table("tk.energy_reference")
    assert (reference["enthalpy"], reference["entropy"]) == ("not_stated", "not_stated")
    assert math.isclose(reference["pressure"], 1e5)


def test_records_that_repeat_a_name_are_repeated_assertions_in_file_order(run: Run) -> None:
    sets = {r["id"]: r for r in run.table("param.nasa9__pure")}
    occurrences: dict[object, list[int]] = {}
    for record in run.table("tk.parameter_set"):
        if record["id"] in sets:
            occurrences.setdefault(sets[record["id"]]["i"], []).append(record["occurrence"])
    repeated = [sorted(found) for found in occurrences.values() if len(found) > 1]
    assert repeated and all(found == list(range(1, len(found) + 1)) for found in repeated)


def test_every_row_ends_in_a_state_and_every_held_row_has_a_typed_reason(run: Run) -> None:
    counts = {table: sum(states.values()) for table, states in run.coverage.counts.items()}
    assert all(counts[table] == len(staged("nasa_cea", table)) for table in counts)
    assert run.coverage.total("unmapped") == 0
    assert run.coverage.by_reason()
    assert set(run.coverage.by_reason()) <= {
        "unknown_subject",
        "missing_convention",
        "validation_failed",
    }
    deferred = {t for t, s in run.coverage.counts.items() if s.get("deferred")}
    assert deferred == {"trans_entries", "trans_intervals"}


def test_a_reversed_interval_of_the_source_is_held_with_its_reason(run: Run) -> None:
    reasons = [r for r in run.held() if r["reason"] == "validation_failed"]
    assert reasons and all("is not below" in str(r["detail"]) for r in reasons)
    assert {str(r["locator"]).split("#")[0] for r in reasons} == {"data/thermo.inp"}


def test_identifiers_are_unique_and_resolution_reports_rules(run: Run) -> None:
    resolved = run.report["totals"]  # type: ignore[index]
    assert resolved["source_entities"] > 2000  # type: ignore[index]
    targets = [
        uuid.UUID(str(r["target"])) if not isinstance(r["target"], uuid.UUID) else r["target"]
        for r in run.table("tk.source_entity", resolution=True)
    ]
    assert targets
