# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The FeOS reader: synthetic payloads in the source's format, then the real acquired tree.

The real-tree counts are computed here from the JSON files by walks independent of the reader.
"""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path

import pyarrow as pa
import pytest
from r3_reader_support import (
    field_metadata,
    run_reader,
    stage_real,
    staged_manifest_of,
    staged_table,
)

from thermo_knowledge import config
from thermo_knowledge.acquire import store
from thermo_knowledge.readers import feos
from thermo_knowledge.staging.errors import StagingError

PIN = "c658aeab484f"
TREE = store.pin_dir(config.raw_dir(), "feos", PIN) / store.TREE_DIR_NAME

IDENT = {
    "cas": "74-82-8",
    "name": "methane",
    "iupac_name": "methane",
    "smiles": "C",
    "inchi": "InChI=1S/CH4/h1H4",
    "formula": "CH4",
}


def dump(value: object) -> str:
    return json.dumps(value)


# -- synthetic payloads ---------------------------------------------------------------------


def test_every_declared_column_documents_name_and_unit() -> None:
    for name, schema in feos.TABLES.items():
        for field in schema:
            metadata = field_metadata(schema, field.name)
            assert metadata.get("source_name"), (name, field.name)
            assert metadata.get("unit"), (name, field.name)
    assert field_metadata(feos.TABLES["pure_records"], "sigma")["unit"] == "Angstrom"
    assert field_metadata(feos.TABLES["pure_records"], "epsilon_k")["unit"] == "K"
    assert field_metadata(feos.TABLES["pure_records"], "m")["unit"] == "not stated"


def test_pure_records_keep_identifier_blocks_sites_and_missing_fields(tmp_path: Path) -> None:
    water = {
        "identifier": {"cas": "7732-18-5", "name": "water", "formula": "H2O"},
        "molarweight": 18,
        "m": 1,
        "sigma": 2.7927,
        "epsilon_k": 353.95,
        "mu": 0.0,
        "viscosity": [-0.5, 1.0, 2, 3.25],
        "association_sites": [
            {"na": 1, "nb": 1.0, "kappa_ab": 0.045, "epsilon_k_ab": 2425.7},
            {"nb": 2.0, "rc_ab": 0.5},
        ],
    }
    methane = {"identifier": IDENT, "molarweight": 16.043, "m": 1.0, "sigma": 3.7, "epsilon_k": 150.0}
    run = run_reader(
        feos,
        tmp_path,
        {"parameters/pcsaft/demo.json": dump([water, methane])},
    )
    rows = run.rows("pure_records")
    assert [r["_locator"] for r in rows] == [
        "parameters/pcsaft/demo.json#/0",
        "parameters/pcsaft/demo.json#/1",
    ]
    first, second = rows
    assert first["model_directory"] == "pcsaft"
    assert first["identifier_cas"] == "7732-18-5"
    assert first["identifier_iupac_name"] is None  # absent key, not empty
    assert first["molarweight"] == 18.0 and first["m"] == 1.0
    assert first["mu"] == 0.0  # an explicit zero stays a zero
    assert second["mu"] is None  # an absent key stays null
    assert first["viscosity"] == [-0.5, 1.0, 2.0, 3.25] and second["viscosity"] is None
    assert first["association_site_count"] == 2 and second["association_site_count"] is None
    assert second["identifier_inchi"] == IDENT["inchi"]
    sites = run.rows("association_sites")
    assert [(s["owner_table"], s["record_index"], s["site_index"]) for s in sites] == [
        ("pure_records", 0, 0),
        ("pure_records", 0, 1),
    ]
    assert sites[0]["_locator"] == "parameters/pcsaft/demo.json#/0/association_sites/0"
    assert sites[0]["na"] == 1.0 and sites[0]["nb"] == 1.0 and sites[0]["rc_ab"] is None
    assert sites[1]["na"] is None and sites[1]["rc_ab"] == 0.5 and sites[1]["kappa_ab"] is None
    (record,) = run.result.payload
    assert record.status == "read"


def test_segment_and_binary_records_including_empty_ones(tmp_path: Path) -> None:
    segment = {"identifier": "CH3", "molarweight": 15.0345, "m": 0.6, "sigma": 3.7, "epsilon_k": 229.9}
    other = {"id1": dict(IDENT), "id2": {**IDENT, "name": "ethane"}}
    binary = [
        {**other, "k_ij": -0.02, "l_ij": 0.0},
        {"id1": IDENT, "id2": {"cas": "1-1-1", "name": "x"}},
        {
            "id1": IDENT,
            "id2": {"cas": "2-2-2", "name": "y"},
            "association_sites": [{"kappa_ab": 0.04, "epsilon_k_ab": 2319.8}],
        },
    ]
    pair = [{"id1": ">CH", "id2": "C≡CH", "k_ij": -0.18}]
    run = run_reader(
        feos,
        tmp_path,
        {
            "parameters/pcsaft/seg.json": dump([segment]),
            "parameters/pcsaft/bin.json": dump(binary),
            "parameters/pcsaft/segbin.json": dump(pair),
        },
    )
    (seg,) = run.rows("segment_records")
    assert seg["segment"] == "CH3" and seg["association_site_count"] is None
    rows = run.rows("binary_records")
    assert [r["record_index"] for r in rows] == [0, 1, 2]
    assert rows[0]["k_ij"] == -0.02 and rows[0]["l_ij"] == 0.0
    empty = rows[1]
    assert empty["k_ij"] is None and empty["l_ij"] is None
    assert empty["association_site_count"] is None
    assert empty["id2_name"] == "x" and empty["id2_smiles"] is None
    assert rows[2]["association_site_count"] == 1 and rows[2]["k_ij"] is None
    (binary_segment,) = run.rows("binary_segment_records")
    assert (binary_segment["segment1"], binary_segment["segment2"]) == (">CH", "C≡CH")
    (site,) = run.rows("association_sites")
    assert site["owner_table"] == "binary_records" and site["record_index"] == 2


def test_permittivity_records(tmp_path: Path) -> None:
    ion = {
        "identifier": {"cas": "1", "name": "ion"},
        "molarweight": 23.0,
        "m": 1,
        "sigma": 2.8,
        "epsilon_k": 230.0,
        "z": 1,
        "permittivity_record": {"ExperimentalData": {"data": [[280.15, 84.89], [298.15, 78.39]]}},
    }
    theory = {
        "identifier": {"cas": "2", "name": "solvent"},
        "molarweight": 1.0,
        "m": 1.0,
        "sigma": 1.0,
        "epsilon_k": 1.0,
        "permittivity_record": {
            "PerturbationTheory": {
                "dipole_scaling": 0.5,
                "polarizability_scaling": 0.25,
                "correlation_integral_parameter": 2.0,
            }
        },
    }
    run = run_reader(feos, tmp_path, {"parameters/epcsaft/p.json": dump([ion, theory])})
    records = run.rows("permittivity_records")
    assert [(r["record_index"], r["variant"], r["point_count"]) for r in records] == [
        (0, "ExperimentalData", 2),
        (1, "PerturbationTheory", None),
    ]
    assert records[1]["dipole_scaling"] == 0.5 and records[0]["dipole_scaling"] is None
    points = run.rows("permittivity_data_points")
    assert [(p["temperature"], p["permittivity"]) for p in points] == [
        (280.15, 84.89),
        (298.15, 78.39),
    ]
    assert points[1]["_locator"].endswith("#/0/permittivity_record/ExperimentalData/data/1")
    pure = run.rows("pure_records")
    assert [p["permittivity_variant"] for p in pure] == ["ExperimentalData", "PerturbationTheory"]
    assert pure[0]["z"] == 1.0 and pure[1]["z"] is None


def test_ideal_gas_group_and_smarts_files(tmp_path: Path) -> None:
    run = run_reader(
        feos,
        tmp_path,
        {
            "parameters/ideal_gas/poling2000.json": dump(
                [{"identifier": {"cas": "56-23-5", "name": "x"}, "DIPPR100": [1.0, 2, -3.5]}]
            ),
            "parameters/ideal_gas/burkhardt2025.json": dump(
                [{"identifier": IDENT, "DIPPR107": [1, 2, 3, 4, 5]}]
            ),
            "parameters/ideal_gas/joback1987.json": dump(
                [{"identifier": "CH3", "molarweight": 15.0, "a": 19.5, "b": -0.008, "c": 1e-4, "d": -1e-7, "e": 0.0}]
            ),
            "parameters/pcsaft/gc_substances.json": dump(
                [
                    {"identifier": IDENT, "segments": ["CH3", "CH2", "CH3"]},
                    {"identifier": {"cas": "3", "name": "ring"}, "segments": ["a", "b"], "bonds": [[0, 1], [1, 0]]},
                    {"identifier": {"cas": "4", "name": "chain"}, "segments": ["a"], "bonds": []},
                ]
            ),
            "parameters/pcsaft/sauer2014_smarts.json": dump(
                [{"group": "CH3", "smarts": "[CH3]"}, {"group": "CH2", "smarts": "[CX4H2]", "max": 3}]
            ),
        },
    )
    dippr = run.rows("dippr_records")
    assert [(d["equation"], d["coefficients"]) for d in dippr] == [
        ("DIPPR107", [1.0, 2.0, 3.0, 4.0, 5.0]),
        ("DIPPR100", [1.0, 2.0, -3.5]),
    ]  # files in sorted order: burkhardt2025 before poling2000
    (joback,) = run.rows("joback_groups")
    assert joback["segment"] == "CH3" and joback["e"] == 0.0 and joback["b"] == -0.008
    chemicals = run.rows("chemical_records")
    assert [c["bond_count"] for c in chemicals] == [None, 2, 0]
    assert chemicals[0]["segments"] == ["CH3", "CH2", "CH3"]
    bonds = run.rows("chemical_record_bonds")
    assert [(b["record_index"], b["segment_index_1"], b["segment_index_2"]) for b in bonds] == [
        (1, 0, 1),
        (1, 1, 0),
    ]
    smarts = run.rows("smarts_records")
    assert [s["max"] for s in smarts] == [None, 3]


def test_multiparameter_terms_keep_scalars_arrays_and_their_lengths(tmp_path: Path) -> None:
    fluid = {
        "identifier": {"name": "Demo"},
        "molarweight": 10.5,
        "tc": 300,
        "rhoc": 1000,
        "ideal_gas": [
            {"a1": 1, "a2": 2.5, "type": "IdealGasHelmholtzLead"},
            {"a": 3, "type": "IdealGasHelmholtzLogTau"},
            {"n": [1.0, 2.0], "t": [0.5, 1], "type": "IdealGasHelmholtzPlanckEinstein"},
            {"a1": 1.0, "a2": 2.0, "reference": "NBP", "_note": "free text", "type": "IdealGasHelmholtzEnthalpyEntropyOffset"},
        ],
        "residual": [
            {"d": [1, 2, 3], "l": [0, 0, 1], "n": [0.1, 0.2, 0.3], "t": [1, 2, 3.5], "type": "ResidualHelmholtzPower"},
            {"A": [1.0], "B": [2.0], "C": [3], "D": [4], "a": [5.0], "b": [6.0], "beta": [7.0], "n": [8.0], "type": "ResidualHelmholtzNonAnalytic"},
            {"n": [], "t": [], "type": "ResidualHelmholtzPower"},
        ],
    }
    run = run_reader(
        feos, tmp_path, {"parameters/multiparameter/coolprop.json": dump([fluid])}
    )
    (head,) = run.rows("multiparameter_fluids")
    assert head["identifier_name"] == "Demo" and head["identifier_cas"] is None
    assert (head["ideal_gas_term_count"], head["residual_term_count"]) == (4, 3)
    terms = run.rows("multiparameter_terms")
    assert [(t["section"], t["term_index"], t["type"]) for t in terms][:2] == [
        ("ideal_gas", 0, "IdealGasHelmholtzLead"),
        ("ideal_gas", 1, "IdealGasHelmholtzLogTau"),
    ]
    assert terms[0]["a1"] == 1.0 and terms[0]["a"] is None  # scalar `a` only on LogTau
    assert terms[1]["a"] == 3.0
    assert json.loads(terms[2]["array_lengths"]) == {"n": 2, "t": 2}
    assert list(json.loads(terms[2]["array_lengths"])) == ["n", "t"]
    assert (terms[3]["reference"], terms[3]["note_field"]) == ("NBP", "free text")
    nonanalytic = terms[5]
    assert json.loads(nonanalytic["array_lengths"])["a"] == 1
    assert json.loads(terms[6]["array_lengths"]) == {"n": 0, "t": 0}  # present and empty
    rows = run.rows("multiparameter_term_rows")
    power = [r for r in rows if r["section"] == "residual" and r["term_index"] == 0]
    assert [(r["row_index"], r["d"], r["t"]) for r in power] == [(0, 1.0, 1.0), (1, 2.0, 2.0), (2, 3.0, 3.5)]
    arrays_a = [r for r in rows if r["term_index"] == 1 and r["section"] == "residual"]
    assert arrays_a[0]["A"] == 1.0 and arrays_a[0]["a"] == 5.0  # A and a stay distinct columns
    assert not [r for r in rows if r["section"] == "residual" and r["term_index"] == 2]
    assert rows[0]["_locator"].endswith("#/0/ideal_gas/2[0]")


@pytest.mark.parametrize(
    ("content", "message"),
    [
        (dump([{"identifier": IDENT, "molarweight": 1.0, "model_record": {"m": 1}}]), r"demo\.json#/0: fields no column declares: model_record"),
        (dump([{"identifier": IDENT, "m": None}]), r"demo\.json#/0/m: explicit null"),
        (dump([{"identifier": IDENT, "m": "one"}]), r"demo\.json#/0/m: expected a number"),
        (dump([{"identifier": {**IDENT, "cpi": "x"}}]), r"demo\.json#/0/identifier: fields no column declares: cpi"),
        (dump([{"molarweight": 1.0}]), r"demo\.json#/0: a record with neither identifier"),
        (dump([{"id1": IDENT, "id2": "CH3"}]), r"demo\.json#/0: id1 and id2 must both be"),
        (dump({"identifier": IDENT}), r"expected a JSON array at the top level"),
        ("[{", r"demo\.json: cannot be read as JSON"),
        (dump([{"identifier": IDENT, "association_sites": [{"na": 1, "id": "A"}]}]), r"demo\.json#/0/association_sites/0: fields no column declares: id"),
        (dump([{"identifier": IDENT, "permittivity_record": {"Unknown": {}}}]), r"unknown permittivity variant 'Unknown'"),
    ],
)
def test_malformed_records_are_refused_with_a_located_error(
    tmp_path: Path, content: str, message: str
) -> None:
    with pytest.raises(StagingError, match=message):
        run_reader(feos, tmp_path, {"parameters/pcsaft/demo.json": content})


def test_malformed_special_files_are_refused(tmp_path: Path) -> None:
    with pytest.raises(StagingError, match=r"expected one DIPPR<number> key"):
        run_reader(
            feos,
            tmp_path,
            {"parameters/ideal_gas/poling2000.json": dump([{"identifier": IDENT, "other": [1.0]}])},
        )
    with pytest.raises(StagingError, match=r"multiparameter/coolprop\.json#/0/residual/0: parallel arrays of different lengths"):
        run_reader(
            feos,
            tmp_path / "second",
            {
                "parameters/multiparameter/coolprop.json": dump(
                    [
                        {
                            "identifier": {"name": "X"},
                            "molarweight": 1.0,
                            "tc": 1.0,
                            "rhoc": 1.0,
                            "ideal_gas": [],
                            "residual": [{"n": [1.0, 2.0], "t": [1.0], "type": "ResidualHelmholtzPower"}],
                        }
                    ]
                )
            },
        )
    with pytest.raises(StagingError, match=r"an array no column declares"):
        run_reader(
            feos,
            tmp_path / "third",
            {
                "parameters/multiparameter/coolprop.json": dump(
                    [
                        {
                            "identifier": {"name": "X"},
                            "molarweight": 1.0,
                            "tc": 1.0,
                            "rhoc": 1.0,
                            "ideal_gas": [{"zz": [1.0], "type": "T"}],
                            "residual": [],
                        }
                    ]
                )
            },
        )
    with pytest.raises(StagingError, match=r"expected a pair of segment indexes"):
        run_reader(
            feos,
            tmp_path / "fourth",
            {
                "parameters/pcsaft/gc_substances.json": dump(
                    [{"identifier": IDENT, "segments": ["a"], "bonds": [[0, 1.5]]}]
                )
            },
        )


# -- the real acquired tree ---------------------------------------------------------------------

real = pytest.mark.skipif(
    not TREE.is_dir(),
    reason=f"the acquired FeOS tree {TREE} is absent (run `tk acquire feos`)",
)


@pytest.fixture(scope="module")
def staged(tmp_path_factory: pytest.TempPathFactory) -> Path:
    return stage_real("feos", tmp_path_factory.mktemp("staged"))


def json_files() -> list[Path]:
    return sorted((TREE / "parameters").rglob("*.json"))


def load(path: Path) -> list:
    return json.loads(path.read_text())


def count_dicts_under(records: list, key: str) -> int:
    """Objects inside the arrays found under `key` anywhere in `records`."""
    total = 0

    def walk(value: object) -> None:
        nonlocal total
        if isinstance(value, dict):
            for name, inner in value.items():
                if name == key and isinstance(inner, list):
                    total += len(inner)
                walk(inner)
        elif isinstance(value, list):
            for inner in value:
                walk(inner)

    walk(records)
    return total


@real
def test_payload_is_the_32_parameter_json_files(staged: Path) -> None:
    manifest = staged_manifest_of(staged)
    assert {r.path for r in manifest.payload} == {p.relative_to(TREE).as_posix() for p in json_files()}
    assert len(manifest.payload) == 32
    assert all(r.status == "read" for r in manifest.payload)
    assert set(manifest.tables) == set(feos.TABLES)


@real
def test_record_counts_match_the_files(staged: Path) -> None:
    pure = segment = binary = binary_segment = 0
    for path in json_files():
        relative = path.relative_to(TREE).as_posix()
        if relative in (
            "parameters/ideal_gas/poling2000.json",
            "parameters/ideal_gas/burkhardt2025.json",
            "parameters/ideal_gas/joback1987.json",
            "parameters/pcsaft/gc_substances.json",
            "parameters/pcsaft/sauer2014_smarts.json",
            "parameters/multiparameter/coolprop.json",
        ):
            continue
        for record in load(path):
            if "id1" in record:
                if isinstance(record["id1"], dict):
                    binary += 1
                else:
                    binary_segment += 1
            elif isinstance(record["identifier"], dict):
                pure += 1
            else:
                segment += 1
    assert (pure, segment, binary, binary_segment) == (2208, 110, 7932, 268)
    assert staged_table(staged, "pure_records").num_rows == pure
    assert staged_table(staged, "segment_records").num_rows == segment
    assert staged_table(staged, "binary_records").num_rows == binary
    assert staged_table(staged, "binary_segment_records").num_rows == binary_segment
    by_directory = Counter(staged_table(staged, "pure_records").column("model_directory").to_pylist())
    assert by_directory == {"pcsaft": 2146, "epcsaft": 17, "saftvrmie": 27, "saftvrqmie": 18}


@real
def test_other_counts_match_the_survey_and_the_files(staged: Path) -> None:
    dippr = sum(len(load(TREE / f"parameters/ideal_gas/{n}.json")) for n in ("poling2000", "burkhardt2025"))
    assert dippr == 1470 == staged_table(staged, "dippr_records").num_rows
    assert staged_table(staged, "joback_groups").num_rows == 22
    chemicals = load(TREE / "parameters/pcsaft/gc_substances.json")
    assert staged_table(staged, "chemical_records").num_rows == len(chemicals) == 88
    assert staged_table(staged, "chemical_record_bonds").num_rows == sum(len(c.get("bonds", [])) for c in chemicals) == 191
    smarts = load(TREE / "parameters/pcsaft/sauer2014_smarts.json")
    assert staged_table(staged, "smarts_records").num_rows == len(smarts) == 22
    assert sum("max" in s for s in smarts) == 13
    fluids = load(TREE / "parameters/multiparameter/coolprop.json")
    assert staged_table(staged, "multiparameter_fluids").num_rows == len(fluids) == 124
    terms = [(s, t) for f in fluids for s in ("ideal_gas", "residual") for t in f[s]]
    assert staged_table(staged, "multiparameter_terms").num_rows == len(terms) == 654
    residual = [t for s, t in terms if s == "residual"]
    assert sum(max(len(v) for v in t.values() if isinstance(v, list)) for t in residual) == 2334
    rows = staged_table(staged, "multiparameter_term_rows")
    assert rows.num_rows == sum(
        max((len(v) for v in t.values() if isinstance(v, list)), default=0) for _, t in terms
    )
    residual_types = Counter(t["type"] for t in residual)
    assert residual_types["ResidualHelmholtzPower"] == 122
    assert residual_types["ResidualHelmholtzGaussian"] == 66
    staged_types = Counter(
        staged_table(staged, "multiparameter_terms").filter(
            pa.compute.equal(staged_table(staged, "multiparameter_terms")["section"], "residual")
        ).column("type").to_pylist()
    )
    assert staged_types == residual_types


@real
def test_site_and_permittivity_counts(staged: Path) -> None:
    sites = sum(count_dicts_under(load(path), "association_sites") for path in json_files())
    assert staged_table(staged, "association_sites").num_rows == sites
    owners = Counter(staged_table(staged, "association_sites").column("owner_table").to_pylist())
    assert set(owners) == {"pure_records", "segment_records", "binary_records"}
    with_sites = sum(
        1
        for path in json_files()
        for record in load(path)
        if isinstance(record, dict) and "association_sites" in record
    )
    assert with_sites == sum(
        (t.column("association_site_count").drop_null().length())
        for t in (
            staged_table(staged, name)
            for name in ("pure_records", "segment_records", "binary_records", "binary_segment_records")
        )
    )
    held = load(TREE / "parameters/epcsaft/held2014_w_permittivity_added.json")
    assert staged_table(staged, "permittivity_records").num_rows == len(held) == 17
    assert staged_table(staged, "permittivity_data_points").num_rows == sum(
        len(r["permittivity_record"]["ExperimentalData"]["data"]) for r in held
    )


def cells(row: dict, names: tuple[str, ...]) -> dict:
    return {name: row[name] for name in names if row[name] is not None}


@real
def test_round_trip_reproduces_every_record(staged: Path) -> None:
    """Rebuild each record of each file from its staged rows and compare with the file's JSON
    (numbers compare by value: the files' integers come back as floats)."""
    pure = staged_table(staged, "pure_records").to_pylist()
    segment = staged_table(staged, "segment_records").to_pylist()
    binary = staged_table(staged, "binary_records").to_pylist()
    binary_segment = staged_table(staged, "binary_segment_records").to_pylist()
    sites: dict[tuple[str, str, int], list[dict]] = {}
    for site in staged_table(staged, "association_sites").to_pylist():
        key = (site["_artifact"], site["owner_table"], site["record_index"])
        sites.setdefault(key, []).append(cells(site, ("na", "nb", "kappa_ab", "epsilon_k_ab", "rc_ab")))
    permit: dict[tuple[str, int], dict] = {}
    for row in staged_table(staged, "permittivity_records").to_pylist():
        permit[(row["_artifact"], row["record_index"])] = {"ExperimentalData": {"data": []}}
    for point in staged_table(staged, "permittivity_data_points").to_pylist():
        permit[(point["_artifact"], point["record_index"])]["ExperimentalData"]["data"].append(
            [point["temperature"], point["permittivity"]]
        )
    ident = ("cas", "name", "iupac_name", "smiles", "inchi", "formula")
    numbers = ("molarweight", "m", "sigma", "epsilon_k", "mu", "q", "z", "viscosity", "lr", "la", "fh")

    def block(row: dict, prefix: str) -> dict:
        return {k: row[f"{prefix}_{k}"] for k in ident if row[f"{prefix}_{k}"] is not None}

    def rebuild(row: dict, table: str) -> dict:
        record: dict = {}
        if table == "pure_records":
            record["identifier"] = block(row, "identifier")
        elif table == "segment_records":
            record["identifier"] = row["segment"]
        elif table == "binary_records":
            record["id1"], record["id2"] = block(row, "id1"), block(row, "id2")
        else:
            record["id1"], record["id2"] = row["segment1"], row["segment2"]
        for name in (*numbers, "k_ij", "l_ij"):
            if name in row and row[name] is not None:
                record[name] = row[name]
        if row["association_site_count"] is not None:
            record["association_sites"] = sites.get(
                (row["_artifact"], table, row["record_index"]), []
            )
            assert len(record["association_sites"]) == row["association_site_count"]
        if table in ("pure_records", "segment_records") and row["permittivity_variant"]:
            record["permittivity_record"] = permit[(row["_artifact"], row["record_index"])]
        return record

    rebuilt: dict[str, dict[int, dict]] = {}
    for table, rows in (
        ("pure_records", pure),
        ("segment_records", segment),
        ("binary_records", binary),
        ("binary_segment_records", binary_segment),
    ):
        for row in rows:
            rebuilt.setdefault(row["_artifact"], {})[row["record_index"]] = rebuild(row, table)
    special = {"ideal_gas", "multiparameter"}
    checked = 0
    for path in json_files():
        relative = path.relative_to(TREE).as_posix()
        if path.parent.name in special or relative.endswith(("gc_substances.json", "smarts.json")):
            continue
        source = load(path)
        assert len(rebuilt[relative]) == len(source), relative
        for index, record in enumerate(source):
            assert rebuilt[relative][index] == record, (relative, index)
            checked += 1
    assert checked == 2208 + 110 + 7932 + 268


@real
def test_round_trip_of_multiparameter_dippr_and_group_files(staged: Path) -> None:
    fluids = load(TREE / "parameters/multiparameter/coolprop.json")
    terms = staged_table(staged, "multiparameter_terms").to_pylist()
    term_rows = staged_table(staged, "multiparameter_term_rows").to_pylist()
    array_rows: dict[tuple[int, str, int], list[dict]] = {}
    for row in term_rows:
        array_rows.setdefault((row["record_index"], row["section"], row["term_index"]), []).append(row)
    scalar_names = ("a", "a1", "a2", "T0", "Tc", "Tcrit", "R", "cp_over_R")
    for term in terms:
        source = fluids[term["record_index"]][term["section"]][term["term_index"]]
        rebuilt: dict = {"type": term["type"]}
        rebuilt.update(cells(term, scalar_names))
        for name in ("Tcrit_units", "reference"):
            if term[name] is not None:
                rebuilt[name] = term[name]
        if term["note_field"] is not None:
            rebuilt["_note"] = term["note_field"]
        lengths = json.loads(term["array_lengths"])
        rows = sorted(
            array_rows.get((term["record_index"], term["section"], term["term_index"]), []),
            key=lambda r: r["row_index"],
        )
        for name, length in lengths.items():
            rebuilt[name] = [r[name] for r in rows][:length]
            assert len(rebuilt[name]) == length
        assert rebuilt == source, (term["_locator"])
        assert list(lengths) == [k for k, v in source.items() if isinstance(v, list)]
    heads = staged_table(staged, "multiparameter_fluids").to_pylist()
    for head, source in zip(heads, fluids, strict=True):
        assert head["identifier_name"] == source["identifier"]["name"]
        assert (head["molarweight"], head["tc"], head["rhoc"]) == (
            source["molarweight"],
            source["tc"],
            source["rhoc"],
        )
    dippr = staged_table(staged, "dippr_records").to_pylist()
    for row in dippr:
        source = load(TREE / row["_artifact"])[row["record_index"]]
        assert source[row["equation"]] == row["coefficients"]
        assert source["identifier"]["cas"] == row["identifier_cas"]
    chemicals = load(TREE / "parameters/pcsaft/gc_substances.json")
    bonds: dict[int, list[list[int]]] = {}
    for bond in staged_table(staged, "chemical_record_bonds").to_pylist():
        bonds.setdefault(bond["record_index"], []).append([bond["segment_index_1"], bond["segment_index_2"]])
    for row in staged_table(staged, "chemical_records").to_pylist():
        source = chemicals[row["record_index"]]
        assert row["segments"] == source["segments"]
        assert bonds.get(row["record_index"], None) == source.get("bonds")
        assert (row["bond_count"] is None) == ("bonds" not in source)
    joback = load(TREE / "parameters/ideal_gas/joback1987.json")
    for row in staged_table(staged, "joback_groups").to_pylist():
        source = joback[row["record_index"]]
        assert {k: row[k] for k in ("molarweight", "a", "b", "c", "d", "e")} == {
            k: source[k] for k in ("molarweight", "a", "b", "c", "d", "e")
        }
        assert row["segment"] == source["identifier"]
    smarts = load(TREE / "parameters/pcsaft/sauer2014_smarts.json")
    for row in staged_table(staged, "smarts_records").to_pylist():
        source = smarts[row["record_index"]]
        assert (row["group"], row["smarts"], row["max"]) == (source["group"], source["smarts"], source.get("max"))
