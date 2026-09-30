# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The JANAF mapping against the real staged data: a dataset for each phase segment of each
table, with the species form it is about, typed columns with the units the rules assume, a point
for each row and a datum for each number, and the conventions the tables are inferred to assume."""

from __future__ import annotations

import math
from collections import Counter
from collections.abc import Iterator

import pytest
from thermochem_support import Run, run_source, staged, staged_directory

pytestmark = pytest.mark.skipif(
    staged_directory("janaf") is None,
    reason="JANAF is not acquired and staged here (run `just tk-acquire janaf` and `just tk-read janaf`)",
)


@pytest.fixture(scope="module")
def run(tmp_path_factory: pytest.TempPathFactory) -> Iterator[Run]:
    yield run_source(tmp_path_factory.mktemp("canonical"), "janaf")


def entities(run: Run) -> dict[str, object]:
    found, _ = run.claims()
    return {claim.local_key: claim for claim in found}


def test_a_table_of_one_phase_is_one_segment_whatever_its_markers_say(run: Run) -> None:
    found = entities(run)
    assert found["Sr-004#1"].aggregation == "liquid"  # type: ignore[attr-defined]
    assert "Sr-004#2" not in found, "a liquid table lists the liquid below its melting point too"
    assert found["Al-005#1"].aggregation == "gas"  # type: ignore[attr-defined]


def test_a_table_of_several_phases_changes_aggregation_at_its_transition_marker(run: Run) -> None:
    found = entities(run)
    assert found["Al-004#1"].aggregation == "crystalline"  # type: ignore[attr-defined]
    assert found["Al-004#2"].aggregation == "liquid"  # type: ignore[attr-defined]
    datasets = {r["local_key"]: r for r in run.table("ev.dataset")}
    assert {"Al-004#1", "Al-004#2"} <= set(datasets)
    rows = [r for r in staged("janaf", "rows") if r["_artifact"] == "Al-004.txt" and r["row_kind"] == "data"]
    marker = next(i for i, r in enumerate(rows) if r["marker"] and "<-->" in r["marker"])
    points = Counter(p["dataset"] for p in run.table("ev.data_point"))
    assert points[datasets["Al-004#1"]["id"]] == marker + 1
    assert points[datasets["Al-004#2"]["id"]] == len(rows) - marker - 1


def test_a_dataset_is_evaluated_and_has_one_component_and_one_phase(run: Run) -> None:
    datasets = run.table("ev.dataset")
    assert {d["kind"] for d in datasets} == {"evaluated"}
    assert len(datasets) == len(entities(run))
    assert Counter(c["dataset"] for c in run.table("ev.dataset_component")) == Counter(
        {d["id"]: 1 for d in datasets}
    )
    assert {p["ordinal"] for p in run.table("ev.dataset_phase")} == {1}


def test_the_eight_columns_are_typed_and_the_enthalpy_increment_is_relative_to_298_15_k(run: Run) -> None:
    columns = [c for c in run.table("ev.dataset_column")]
    first = min({c["dataset"] for c in columns}, key=str)
    mine = sorted((c for c in columns if c["dataset"] == first), key=lambda c: c["ordinal"])
    assert [c["ordinal"] for c in mine] == list(range(1, 9))
    assert [c["role"] for c in mine] == ["variable"] + ["property"] * 7
    relative = [c for c in columns if c["presentation"] == "difference_from_reference"]
    assert len(relative) == len(run.table("ev.dataset"))
    assert {c["reference_state_kind"] for c in relative} == {"reference_phase_fixed_tp"}
    assert {round(c["reference_temperature"], 2) for c in relative} == {298.15}
    assert all(c["reference_phase"] is not None for c in relative)


def test_a_number_is_a_datum_in_the_storage_unit_and_a_divergent_cell_has_none(run: Run) -> None:
    table = next(t for t in staged("janaf", "tables") if t["code"] == "Al-005")
    rows = [r for r in staged("janaf", "rows") if r["_artifact"] == "Al-005.txt" and r["row_kind"] == "data"]
    dataset = next(d for d in run.table("ev.dataset") if d["local_key"] == "Al-005#1")
    columns = {c["ordinal"]: c for c in run.table("ev.dataset_column") if c["dataset"] == dataset["id"]}
    points = {p["index"]: p["id"] for p in run.table("ev.data_point") if p["dataset"] == dataset["id"]}
    datums = {
        (d["point"], d["column"]): d for d in run.table("ev.datum") if d["point"] in set(points.values())
    }
    assert table["phase_designator"] == "g" and len(points) == len(rows)
    names = ["t", "cp", "s", "g_function", "h_increment", "delta_f_h", "delta_f_g", "log_kf"]
    expected = sum(1 for r in rows for n in names if r[n] is not None)
    assert len(datums) == expected
    zero = rows[0]
    assert zero["g_function_text"] == "INFINITE"
    assert (points[1], columns[4]["id"]) not in datums
    hot = rows[5]
    point = points[6]
    assert math.isclose(datums[(point, columns[2]["id"])]["value"], hot["cp"], rel_tol=1e-15)
    assert math.isclose(datums[(point, columns[5]["id"])]["value"], hot["h_increment"] * 1000.0, rel_tol=1e-12)
    assert datums[(point, columns[2]["id"])]["state"] == "known"
    assert datums[(point, columns[2]["id"])]["digits"] >= 4


def test_the_conventions_are_the_inferred_energy_reference_and_no_gas_constant(run: Run) -> None:
    (conventions,) = run.table("tk.convention_set")
    assert conventions["gas_constant"] is None and conventions["temperature_scale"] == "not_stated"
    (reference,) = run.table("tk.energy_reference")
    assert (reference["enthalpy"], reference["entropy"]) == ("formation_from_elements", "third_law")
    assert math.isclose(reference["temperature"], 298.15) and math.isclose(reference["pressure"], 1e5)
    states = {s["kind"]: s for s in run.table("tk.standard_state")}
    assert set(states) == {"pure_ideal_gas", "pure_real"}
    assert all(math.isclose(s["pressure"], 1e5) and s["pressure_rule"] == "fixed" for s in states.values())


def test_the_composition_and_charge_of_a_table_come_from_its_formula(run: Run) -> None:
    table = next(t for t in staged("janaf", "tables") if t["janaf_formula"] == "Ar1+")
    resolved = {
        (r["scope"], r["local_key"]): r["target"] for r in run.table("tk.source_entity", resolution=True)
    }
    form = resolved[("tables", f"{table['code']}#1")]
    mine = [c for c in run.table("tk.composition") if c["entity"] == form]
    assert sorted(c["value"] for c in mine) == [1.0, 1.0]  # one argon atom and one unit of charge
    assert entities(run)[f"{table['code']}#1"].stated_charge == 1  # type: ignore[attr-defined]


def test_a_table_that_names_no_aggregation_or_no_atom_is_held_with_its_reason(run: Run) -> None:
    held = [r for r in run.held() if r["source_table"] == "rows"]
    reasons = Counter(str(r["detail"]).split(": ", 1)[-1][:40] for r in held)
    assert any("designator names no single aggregation" in str(r["detail"]) for r in held)
    assert any("is not a JANAF formula" in str(r["detail"]) for r in held)
    assert any("run together" in str(r["detail"]) or "does not map yet" in str(r["detail"]) for r in held)
    assert reasons


def test_every_row_ends_in_a_state_and_the_separators_are_out_of_scope(run: Run) -> None:
    for table, states in run.coverage.counts.items():
        assert sum(states.values()) == len(staged("janaf", table))
    assert run.coverage.total("unmapped") == 0
    assert run.coverage.counts["rows"]["out_of_scope"] == 86
    assert set(run.coverage.by_reason()) <= {"missing_convention", "pattern_mismatch"}
