# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The thermo reader: synthetic payload files in the source's formats, then the real acquired
tree. Real-store counts are taken directly from the raw files, independently of the reader."""

from __future__ import annotations

import csv
import gzip
import io
import json
import re
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq
import pytest
from tabular_reader_support import Staged, lines, stage

from thermo_knowledge import config
from thermo_knowledge.acquire import store
from thermo_knowledge.acquire.lock import read_lock
from thermo_knowledge.acquire.manifest import default_lock_path, default_sources_dir, load_sources
from thermo_knowledge.readers import thermo
from thermo_knowledge.staging import load, payload, schema
from thermo_knowledge.staging import manifest as staged_manifest
from thermo_knowledge.staging import stage as staging
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.stage import StageContext
from thermo_knowledge.testing import TestDatabase

PIN = "4f51c4343773"
TREE = store.pin_dir(config.raw_dir(), "thermo", PIN) / store.TREE_DIR_NAME
needs_store = pytest.mark.skipif(
    not TREE.is_dir(),
    reason=f"the acquired thermo tree {TREE} is absent (run `tk acquire thermo`)",
)

ICS = "thermo/Interaction Parameters"
HENRY = f"{ICS}/ChemSep/henry.json"
NRTL = f"{ICS}/ChemSep/nrtl.json"
PR = f"{ICS}/ChemSep/pr.json"
SCALAR = "thermo/Scalar Parameters/PRVolumeTranslation_PinaMartinez.json"
PHASE = "thermo/Phase Change"


def run(tmp_path: Path, files: dict[str, str | bytes]) -> Staged:
    return stage(thermo, tmp_path, files)


def meta(table: str, column: str) -> dict[str, str]:
    field = thermo.TABLES[table].field(column)
    return {k.decode(): v.decode() for k, v in (field.metadata or {}).items()}


# -- declared schemas --------------------------------------------------------------------------------


def test_every_table_is_declared_with_documented_typed_columns() -> None:
    assert len(thermo.TABLES) == 34
    schema.check_declared(thermo.TABLES)
    assert thermo.TABLES["henry_pairs"].field("A").type == pa.float64()
    assert thermo.TABLES["pr_kij_pairs"].field("page").type == pa.list_(pa.string())
    assert thermo.TABLES["parameter_files"].field("necessary_keys").type == pa.list_(pa.string())
    assert thermo.TABLES["correlation_leaves"].field("coeffs").type == pa.list_(pa.float64())
    assert thermo.TABLES["correlation_leaves"].field("N_T").type == pa.int64()
    assert thermo.TABLES["ddbst_unifac_assignments"].field("PSRK_flag").type == pa.int64()
    assert thermo.TABLES["unifac_subgroups"].field("hydrogen_from_smarts").type == pa.bool_()
    assert meta("correlation_leaves", "A")["source_name"] == "A"
    assert meta("correlation_leaves", "a")["source_name"] == "a"  # the two spellings are two columns
    assert "law_echa_tonnage_bands" not in thermo.TABLES  # the manifest includes no .zip


# -- interaction and scalar parameter files ------------------------------------------------------------


def henry_document(**rows: object) -> str:
    metadata = {
        "symmetric": False, "source": "synthetic", "T dependent": True, "P dependent": False,
        "type": "henry", "necessary keys": list("ABCDEF"), "components": 2,
        "missing": {k: 0.0 for k in "ABCDEF"},
    }  # fmt: skip
    return json.dumps({"metadata": metadata, "data": rows})


def test_pair_files_keep_the_key_the_metadata_and_the_parameters(tmp_path: Path) -> None:
    row = {"name": "Gas/Liquid", **{k: float(i) for i, k in enumerate("ABCDEF")}}
    staged = run(tmp_path, {HENRY: henry_document(**{"1-1-1 2-2-2": row})})
    (pair,) = staged.rows("henry_pairs")
    assert (pair["pair"], pair["CAS1"], pair["CAS2"]) == ("1-1-1 2-2-2", "1-1-1", "2-2-2")
    assert pair["name"] == "Gas/Liquid" and pair["F"] == 5.0
    assert pair["_locator"] == f"{HENRY}#/data/1-1-1 2-2-2"
    (files,) = staged.rows("parameter_files")
    assert files["type"] == "henry" and files["T_dependent"] is True and files["P_dependent"] is False
    assert files["necessary_keys"] == list("ABCDEF") and files["symmetric"] is False
    assert json.loads(files["missing"]) == {k: 0.0 for k in "ABCDEF"}


def test_null_parameters_stay_null_and_lists_stay_lists(tmp_path: Path) -> None:
    body = {"Tmin": None, "P": None, "Pmin": 1.0, "kij": -0.5, "T": None, "page": ["p242"], "Pmax": 2}
    metadata = {"symmetric": True, "source": "s", "components": 2, "necessary keys": ["kij"]}
    document = json.dumps({"metadata": metadata, "data": {"1-1-1 2-2-2": body}})
    (row,) = run(tmp_path, {PR: document}).rows("pr_kij_pairs")
    assert row["Tmin"] is None and row["Pmin"] == 1.0 and row["Pmax"] == 2.0
    assert row["kij"] == -0.5 and row["page"] == ["p242"]


def test_scalar_files_are_keyed_by_cas(tmp_path: Path) -> None:
    document = json.dumps(
        {"metadata": {"source": "s", "necessary keys": ["PRc"], "missing": {"PRc": 0.0}},
         "data": {"1-1-1": {"PRc": 1.8e-05, "name": "1-1-1"}}}
    )  # fmt: skip
    (row,) = run(tmp_path, {SCALAR: document}).rows("scalar_pr_volume_translation_pina_martinez")
    assert (row["CAS"], row["PRc"], row["name"]) == ("1-1-1", 1.8e-05, "1-1-1")


def test_a_malformed_pair_key_or_an_unknown_parameter_is_refused(tmp_path: Path) -> None:
    one = {"name": "x", **{k: 1.0 for k in "ABCDEF"}}
    with pytest.raises(StagingError, match=r"the key '1-1-1' is not 'CAS1 CAS2'"):
        run(tmp_path / "a", {HENRY: henry_document(**{"1-1-1": one})})
    with pytest.raises(StagingError, match=r"keys no column declares: G"):
        run(tmp_path / "b", {HENRY: henry_document(**{"1-1-1 2-2-2": {**one, "G": 1.0}})})
    with pytest.raises(StagingError, match=r"expected a number"):
        run(tmp_path / "c", {HENRY: henry_document(**{"1-1-1 2-2-2": {**one, "A": "x"}})})
    with pytest.raises(StagingError, match=r"cannot be read as JSON"):
        run(tmp_path / "d", {HENRY: "{"})


# -- ChemSep IPD text ------------------------------------------------------------------------------------

NRTL_IPD = f"{ICS}/ChemSep/nrtl.ipd"
_IPD = (
    "\r\n ----\r\n  synthetic banner\r\n ----\r\n\r\n[IPD]\r\nComment=synthetic data\r\nUnits=cal/mol\r\n#\r\n"
    "# ID/CASN       ID/CASN    A12         A21   alpha12      Name/Name Comments\r\n"
    "1-1-1       2-2-2  378.8  -14.5  .187e-1  One/Two p18 1/2c\r\n"
    "2-2-2       1-1-1  1.5  2.5  .3\r\n"
    "\r\n"
)


def test_ipd_rows_carry_the_legend_named_values_and_the_remainder(tmp_path: Path) -> None:
    staged = run(tmp_path, {NRTL_IPD: _IPD})
    (files,) = staged.rows("ipd_files")
    assert files["Comment"] == "synthetic data" and files["Units"] == "cal/mol"
    assert files["value_names"] == ["A12", "A21", "alpha12"]
    first, second = staged.rows("ipd_rows")
    assert (first["ID1"], first["ID2"]) == ("1-1-1", "2-2-2")
    assert first["values"] == [378.8, -14.5, 0.0187]  # .187e-1 read as a decimal number
    assert first["remainder"] == "One/Two p18 1/2c"
    assert second["remainder"] is None  # a row may end after its numbers
    assert first["_locator"] == f"{NRTL_IPD}#L11"
    other = staged.rows("ipd_lines")
    assert other[0]["text"] == "" and other[5]["text"] == "[IPD]"  # carriage returns dropped
    assert other[-1]["text"] == ""  # blank lines after the data are kept


def test_ipd_refuses_a_row_short_of_its_legend_values(tmp_path: Path) -> None:
    short = _IPD.replace("2-2-2       1-1-1  1.5  2.5  .3", "2-2-2       1-1-1  1.5")
    with pytest.raises(StagingError, match=r"#L12: 3 fields, the legend needs 5"):
        run(tmp_path, {NRTL_IPD: short})
    with pytest.raises(StagingError, match=r"no legend line"):
        run(tmp_path / "b", {NRTL_IPD: "[IPD]\r\n1-1-1 2-2-2 1.0\r\n"})
    with pytest.raises(StagingError, match=r"column alpha12: 'x'"):
        run(tmp_path / "c", {NRTL_IPD: _IPD.replace(".187e-1", "x")})


# -- correlations ------------------------------------------------------------------------------------------

ORGANIC = "thermo/Misc/organic_correlations.json"
HO = "thermo/Misc/Ho_1972_thermal_conductivity_solid.json"
MIXTURE = "thermo/Misc/mixture_correlations.json"


def test_correlation_leaves_and_series(tmp_path: Path) -> None:
    document = {
        "1-1-1": {
            "SurfaceTension": {
                "REFPROP_sigma_parameters": {
                    "Fit 2023": {
                        "Tc": 500.0, "Tmax": 400.0, "Tmin": 200.0, "n0": 1.0, "n1": 2.0, "n2": 3.0,
                        "sigma0": 0.1, "sigma1": 0.2, "sigma2": 0.3,
                    }
                }
            },
            "VaporPressure": {
                "Wagner_parameters": {
                    "Fit 2023": {"Pc": 1e6, "Tc": 500.0, "Tmax": 400.0, "Tmin": 200.0,
                                 "a": -7.0, "b": 1.0, "c": -2.0, "d": -3.0}
                },
                "DIPPR100_parameters": {
                    "Fit 2023": {"A": 1.0, "B": 2.0, "C": 3.0, "D": 4.0, "E": 5.0, "F": 6.0,
                                 "G": 7.0, "Tmax": 400.0, "Tmin": 200.0}
                },
            },
        }
    }  # fmt: skip
    staged = run(tmp_path, {ORGANIC: json.dumps(document)})
    leaves = {(r["property"], r["model_key"]): r for r in staged.rows("correlation_leaves")}
    sigma = leaves[("SurfaceTension", "REFPROP_sigma_parameters")]
    assert sigma["key"] == "1-1-1" and sigma["label"] == "Fit 2023"
    assert sigma["sigma2"] == 0.3 and sigma["A"] is None
    wagner = leaves[("VaporPressure", "Wagner_parameters")]
    dippr = leaves[("VaporPressure", "DIPPR100_parameters")]
    assert (wagner["a"], wagner["A"]) == (-7.0, None)  # a and A are different columns
    assert (dippr["A"], dippr["a"], dippr["G"]) == (1.0, None, 7.0)
    assert dippr["_locator"].endswith("#/1-1-1/VaporPressure/DIPPR100_parameters/Fit 2023")


def test_flat_arrays_nested_arrays_and_tabular_leaves(tmp_path: Path) -> None:
    mixture = {
        "1-1-1 2-2-2": {"SurfaceTensionMixture": {"redlick_kister_parameters": {"fit 2023": {
            "coeffs": [[1.0, 2.0], [3.0, 4.0]], "N_T": 2, "N_terms": 2}}}}
    }  # fmt: skip
    ho = {"7440-37-1": {"ThermalConductivitySolid": {"tabular_data": {"Ho (1972)": [[8.0, 9.0], [6.0, 4.6]]}}}}
    flat = {"1-1-1": {"HeatCapacityGas": {"stable_polynomial_parameters": {"HEOS_FIT": {
        "Tmax": 6000.0, "Tmin": 0.0, "coeffs": [1.0, 2.0, 3.0], "int_T_coeffs": [4.0], "int_T_log_coeff": 5.0}}}}}  # fmt: skip
    staged = run(
        tmp_path,
        {MIXTURE: json.dumps(mixture), HO: json.dumps(ho), "thermo/Misc/janaf_correlations.json": json.dumps(flat)},
    )
    series = staged.rows("correlation_series")
    assert [(r["key"], r["field"], r["series_index"], r["values"]) for r in series] == [
        ("7440-37-1", None, 0, [8.0, 9.0]),
        ("7440-37-1", None, 1, [6.0, 4.6]),
        ("1-1-1 2-2-2", "coeffs", 0, [1.0, 2.0]),
        ("1-1-1 2-2-2", "coeffs", 1, [3.0, 4.0]),
    ]
    by_key = {r["key"]: r for r in staged.rows("correlation_leaves")}
    assert by_key["1-1-1 2-2-2"]["N_T"] == 2 and by_key["1-1-1 2-2-2"]["coeffs"] is None
    assert by_key["7440-37-1"]["Tmin"] is None  # a tabular leaf keeps its identity, no coefficients
    assert by_key["1-1-1"]["coeffs"] == [1.0, 2.0, 3.0] and by_key["1-1-1"]["int_T_coeffs"] == [4.0]
    assert by_key["1-1-1"]["int_T_log_coeff"] == 5.0


def test_a_leaf_key_no_column_declares_is_refused(tmp_path: Path) -> None:
    document = {"1-1-1": {"P": {"m": {"l": {"Tmin": 1.0, "Zzz": 2.0}}}}}
    with pytest.raises(StagingError, match=r"keys no column declares: Zzz"):
        run(tmp_path, {ORGANIC: json.dumps(document)})


# -- UNIFAC tables -----------------------------------------------------------------------------------------


def test_interaction_tables_fill_the_columns_their_width_has(tmp_path: Path) -> None:
    files = {
        f"{PHASE}/UNIFAC original interaction parameters.tsv": lines("1|2|86.02", "2|1|-35.36"),
        f"{PHASE}/PSRK interaction parameters.tsv": lines("1|2|86.02|0|0", "1|3|61.13|-0.5|1e-3"),
        # the last line of several files has no trailing newline
        f"{PHASE}/UNIFAC modified NIST 2015 interaction parameters.tsv": "1\t2\t257.05\t-0.1\t0\t277.59\t576.96",
    }
    rows = run(tmp_path, files).rows("unifac_interaction_parameters")
    by_file = {}
    for row in rows:
        by_file.setdefault(row["_artifact"].rsplit("/", 1)[1], []).append(row)
    original = by_file["UNIFAC original interaction parameters.tsv"]
    assert (original[0]["m"], original[0]["n"], original[0]["a"]) == (1, 2, 86.02)
    assert original[0]["b"] is None and original[0]["Tmax"] is None
    psrk = by_file["PSRK interaction parameters.tsv"]
    assert psrk[0]["b"] == 0.0 and psrk[1]["b"] == -0.5 and psrk[1]["c"] == 1e-3
    assert psrk[0]["Tmin"] is None  # a column the file lacks is null, a stored 0 is 0
    nist = by_file["UNIFAC modified NIST 2015 interaction parameters.tsv"]
    assert (nist[0]["Tmin"], nist[0]["Tmax"]) == (277.59, 576.96)
    assert original[1]["_locator"].endswith("#L2")


def test_an_interaction_file_has_one_width_throughout(tmp_path: Path) -> None:
    path = f"{PHASE}/PSRK interaction parameters.tsv"
    with pytest.raises(StagingError, match=r"#L2: 3 cells, the file's first row has 5"):
        run(tmp_path / "a", {path: lines("1|2|1|0|0", "1|3|2")})
    with pytest.raises(StagingError, match=r"4 cells, expected one of"):
        run(tmp_path / "b", {path: lines("1|2|1|0")})
    with pytest.raises(StagingError, match=r"column a: 'x'"):
        run(tmp_path / "c", {path: lines("1|2|x|0|0")})


def test_ddbst_assignments_keep_cells_and_split_pairs(tmp_path: Path) -> None:
    path = f"{PHASE}/DDBST UNIFAC assignments.tsv"
    content = (
        "AAAAAAAAAAAAAA-UHFFFAOYSA-N\t0 1 1\t2 3 3 1 \t9 8 10 2 \t-1 1 \n"
        "BBBBBBBBBBBBBB-UHFFFAOYSA-N\t1 1 1\t1 2 \t1 2 \t1 2 \n"
    )
    staged = run(tmp_path, {path: content})
    first, second = staged.rows("ddbst_unifac_assignments")
    assert (first["UNIFAC_flag"], first["modified_flag"], first["PSRK_flag"]) == (0, 1, 1)
    assert first["UNIFAC_cell"] == "2 3 3 1 " and first["PSRK_cell"] == "-1 1 "  # verbatim, trailing space
    pairs = [
        (p["InChIKey"][:1], p["scheme"], p["pair_index"], p["subgroup_id"], p["count"])
        for p in staged.rows("ddbst_unifac_assignment_pairs")
    ]
    assert pairs[:5] == [
        ("A", "UNIFAC", 0, 2, 3),
        ("A", "UNIFAC", 1, 3, 1),
        ("A", "modified", 0, 9, 8),
        ("A", "modified", 1, 10, 2),
        ("A", "PSRK", 0, -1, 1),
    ]
    assert second["_locator"].endswith("#L2") and len(pairs) == 8
    with pytest.raises(StagingError, match=r"a cell of 3 numbers is not made of pairs"):
        run(tmp_path / "odd", {path: "K\t1 1 1\t1 2 3\t1 2 \t1 2 \n"})
    with pytest.raises(StagingError, match=r"the flags '1 2 1' are not three of 0 or 1"):
        run(tmp_path / "flags", {path: "K\t1 2 1\t1 2 \t1 2 \t1 2 \n"})


_UNIFAC_PY = '''"""Synthetic module in the shape of thermo/unifac.py."""
UFMG = {}
UFMG[1] = ("CH2", [1, 2])
UFSG = {}
UFSG[1] = UNIFAC_subgroup(1, "CH3", 1, "CH2", 0.9, 0.85, smarts="[CX4;H3]", atoms={"C": 1, "H": 3})
UFSG[2] = UNIFAC_subgroup(2, "CH2", 1, "CH2", 0.67, 0,
                          atoms=UFSG[1].atoms, bonds={DOUBLE_BOND: 1}, smarts=["[a]", "[b]"],
                          priority=1000, hydrogen_from_smarts=True)
VTPRMG = {1: ("CH2", [1, 2]),
2: ("ACH", [9, 10], "Aromatic carbon")}

def code():
    return UFMG
'''


def test_unifac_source_constructs_are_read_without_being_evaluated(tmp_path: Path) -> None:
    staged = run(tmp_path, {"thermo/unifac.py": _UNIFAC_PY})
    first, second = staged.rows("unifac_subgroups")
    assert (first["set_name"], first["group_id"], first["group"], first["main_group"]) == (
        "UFSG", 1, "CH3", "CH2",
    )  # fmt: skip
    assert (first["R"], first["Q"]) == (0.9, 0.85)
    assert first["smarts"] == ["[CX4;H3]"] and first["smarts_expr"] == '"[CX4;H3]"'
    assert json.loads(first["atoms_json"]) == {"C": 1, "H": 3} and first["bonds_expr"] is None
    assert second["Q"] == 0.0 and second["priority"] == 1000 and second["hydrogen_from_smarts"] is True
    assert second["atoms_expr"] == "UFSG[1].atoms" and second["atoms_json"] is None  # a reference
    assert second["bonds_expr"] == "{DOUBLE_BOND: 1}" and second["smarts"] == ["[a]", "[b]"]
    assert second["_locator"] == "thermo/unifac.py#L6"
    groups = staged.rows("unifac_main_groups")
    assert [(g["set_name"], g["main_group_id"], g["name"], g["subgroup_ids"]) for g in groups] == [
        ("UFMG", 1, "CH2", [1, 2]),
        ("VTPRMG", 1, "CH2", [1, 2]),
        ("VTPRMG", 2, "ACH", [9, 10]),
    ]
    assert groups[2]["description"] == "Aromatic carbon" and groups[0]["description"] is None
    record = staged.payload["thermo/unifac.py"]
    assert record.status == "partly_read" and "code" in (record.reason or "")


def test_unifac_source_refuses_what_it_cannot_declare(tmp_path: Path) -> None:
    path = "thermo/unifac.py"
    bad_key = _UNIFAC_PY.replace("UFSG[2] = UNIFAC", "UFSG[3] = UNIFAC")
    with pytest.raises(StagingError, match=r"the key of UFSG\[\.\.\] differs from group_id"):
        run(tmp_path / "a", {path: bad_key})
    unknown = _UNIFAC_PY + 'NEWSG = {}\nNEWSG[1] = 1\n'
    with pytest.raises(StagingError, match=r"NEWSG\[\.\.\] looks like a group table"):
        run(tmp_path / "b", {path: unknown})
    extra = _UNIFAC_PY.replace("priority=1000", "priority=1000, colour=3")
    with pytest.raises(StagingError, match=r"keyword arguments no column declares"):
        run(tmp_path / "c", {path: extra})
    with pytest.raises(StagingError, match=r"cannot be parsed as Python"):
        run(tmp_path / "d", {path: "UFSG[1] = ("})


# -- tables shared with the chemicals payload --------------------------------------------------------------


def test_inventory_and_bell_tables_are_read_under_the_thermo_root(tmp_path: Path) -> None:
    dsl = gzip.compress(b"CASRN\tRegistry\n50000\t0\n")
    bell = lines("Tc|Pc|c0|c1|c2|name|inchikey", "548.4|4203546.06|0.25|0.83|2.0|synthetic|KEY-ONE")
    files = {
        "thermo/Law/Canada Feb 11 2015 - DSL.csv.gz": dsl,
        f"{PHASE}/Bell 2018 je7b00967_si_001.tsv": bell,
    }
    staged = run(tmp_path, files)
    assert staged.rows("law_canada_dsl")[0]["CASRN"] == 50000
    (row,) = staged.rows("phase_change_bell_2018")
    assert row["Pc"] == 4203546.06 and row["inchikey"] == "KEY-ONE"
    assert row["_artifact"] == f"{PHASE}/Bell 2018 je7b00967_si_001.tsv"


def test_a_payload_file_without_a_rule_is_refused(tmp_path: Path) -> None:
    with pytest.raises(StagingError, match=r"no rule for this payload file"):
        run(tmp_path, {"thermo/Misc/new.json": "{}"})


# -- the real tree -----------------------------------------------------------------------------------------


@pytest.fixture(scope="module")
def staged(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """The real `tk read thermo`, into a temporary staged directory."""
    ctx = StageContext(staged_dir=tmp_path_factory.mktemp("staged"))
    manifests = load_sources(default_sources_dir())
    outcome = staging.read_source(ctx, manifests["thermo"], read_lock(default_lock_path()))
    assert outcome.status == "read"
    return staging.staged_path(ctx, "thermo", PIN)


@pytest.fixture(scope="module")
def manifest(staged: Path) -> staged_manifest.StagedManifest:
    return staged_manifest.read(staged)


def table(staged: Path, name: str) -> pa.Table:
    return pq.read_table(staged / f"{name}.parquet")


def raw_text(artifact: str) -> str:
    data = (TREE / artifact).read_bytes()
    if artifact.endswith(".gz"):
        data = gzip.decompress(data)
    return data.decode("utf-8")


def physical_lines(artifact: str) -> int:
    data = raw_text(artifact).encode()
    return data.count(b"\n") + (0 if data.endswith(b"\n") else 1)


def raw_json(artifact: str) -> dict:
    return json.loads(raw_text(artifact))


@needs_store
def test_every_table_is_staged_and_every_payload_file_accounted(
    manifest: staged_manifest.StagedManifest,
) -> None:
    assert set(manifest.tables) == set(thermo.TABLES)
    assert manifest.reader.name == "thermo" and manifest.reader.version == thermo.READER_VERSION
    assert manifest.pin == PIN
    source = load_sources(default_sources_dir())["thermo"]
    expected = payload.payload_files(TREE, source.payload.include, source.payload.exclude)
    assert len(expected) == 52
    assert sorted(r.path for r in manifest.payload) == expected
    statuses = {r.path: r.status for r in manifest.payload}
    assert statuses.pop("thermo/unifac.py") == "partly_read"
    assert set(statuses.values()) == {"read"}
    assert all(record.rows > 0 for record in manifest.tables.values())


@needs_store
def test_every_staged_column_is_documented(staged: Path) -> None:
    for name in thermo.TABLES:
        for field in pq.read_schema(staged / f"{name}.parquet"):
            metadata = {k.decode(): v.decode() for k, v in (field.metadata or {}).items()}
            assert metadata.get("source_name"), (name, field.name)
            assert metadata.get("unit"), (name, field.name)


@needs_store
def test_row_counts_match_independent_counts(manifest: staged_manifest.StagedManifest) -> None:
    tables = manifest.tables
    # pair and scalar files: the number of data keys, counted with json.load
    for family in thermo.interaction.FAMILIES:
        expected = sum(len(raw_json(path)["data"]) for path in family.files)
        assert tables[family.table].rows == expected, family.table
    assert tables["parameter_files"].rows == sum(
        "metadata" in raw_json(path) for family in thermo.interaction.FAMILIES for path in family.files
    ) == 16
    # the survey's counts
    assert tables["henry_pairs"].rows == 41 + 4407 + 953 + 20138
    assert tables["eppr78_kij_pairs"].rows == 87990
    # IPD rows are the lines that begin with a CAS-shaped token
    cas = re.compile(r"^\d+-\d\d-\d\s")
    for path in thermo.ipd.FILES:
        assert sum(bool(cas.match(line)) for line in raw_text(path).split("\r\n")) > 0
    assert tables["ipd_rows"].rows == sum(
        bool(cas.match(line)) for path in thermo.ipd.FILES for line in raw_text(path).split("\r\n")
    ) == 530
    # the UNIFAC tables
    assert tables["unifac_interaction_parameters"].rows == sum(
        physical_lines(path) for path in thermo.unifac.INTERACTION_FILES
    ) == 17689
    assert tables["ddbst_unifac_assignments"].rows == physical_lines(thermo.unifac.DDBST_FILE) == 31778
    # law and Bell
    for spec in thermo.DELIMITED:
        expected = sum(
            physical_lines(f"{thermo.DATA_ROOT}/{rel}") - (1 if spec.headings else 0)
            for rel in spec.files
        )
        assert tables[spec.table].rows == expected, spec.table


@needs_store
def test_correlation_counts_match_a_nested_traversal(manifest: staged_manifest.StagedManifest) -> None:
    leaves = 0
    series = 0
    for name in thermo.correlations.FILES:
        for properties in raw_json(f"thermo/Misc/{name}").values():
            for models in properties.values():
                for labels in models.values():
                    for leaf in labels.values():
                        leaves += 1
                        if isinstance(leaf, list):
                            series += len(leaf)
                        elif isinstance(leaf.get("coeffs"), list) and leaf["coeffs"] and isinstance(
                            leaf["coeffs"][0], list
                        ):
                            series += len(leaf["coeffs"])
    assert manifest.tables["correlation_leaves"].rows == leaves == 4764  # 4018 + 745 + 1
    assert manifest.tables["correlation_series"].rows == series == 88


@needs_store
def test_unifac_source_counts_match_a_regex_scan(manifest: staged_manifest.StagedManifest) -> None:
    source = raw_text("thermo/unifac.py")
    subgroups = re.findall(r"^[A-Z0-9]*SG\[\d+\] = UNIFAC_subgroup\(", source, re.M)
    subscripts = re.findall(r"^[A-Z0-9]*MG\[\d+\] = \(", source, re.M)
    dict_entries = 0
    for name in ("VTPRMG", "PSRKMG", "LLEMG", "LUFMG", "NISTKTUFMG"):
        block = re.search(rf"^{name} = \{{(.*?)\}}\s*$", source, re.M | re.S)
        assert block is not None
        dict_entries += len(re.findall(r"\d+:\s*\(", block.group(1)))
    assert manifest.tables["unifac_subgroups"].rows == len(subgroups) == 936
    assert manifest.tables["unifac_main_groups"].rows == len(subscripts) + dict_entries == 411


# -- round trips ------------------------------------------------------------------------------------------


@needs_store
@pytest.mark.parametrize(
    ("family", "file"),
    [
        ("henry_pairs", "thermo/Interaction Parameters/ChemSep/henry.json"),
        ("henry_pairs", "thermo/Interaction Parameters/Sander_henry_T_dep.json"),
        ("henry_pairs", "thermo/Interaction Parameters/PRTranslated_best_henry_T_dep.json"),
        ("nrtl_pairs", "thermo/Interaction Parameters/ChemSep/nrtl.json"),
        ("uniquac_pairs", "thermo/Interaction Parameters/ChemSep/uniquac.json"),
        ("wilson_pairs", "thermo/Interaction Parameters/ChemSep/wilson.json"),
        ("pr_kij_pairs", "thermo/Interaction Parameters/ChemSep/pr.json"),
        ("eppr78_kij_pairs", "thermo/Interaction Parameters/eppr78_common.json"),
        ("scalar_pr_twu_ibell_2018", "thermo/Scalar Parameters/PRTwu_ibell_2018.json"),
        ("scalar_chemsep_regular_solution", "thermo/Scalar Parameters/chemsep_regular_solution.json"),
    ],
)
def test_parameter_tables_rebuild_the_data_objects(staged: Path, family: str, file: str) -> None:
    spec = next(f for f in thermo.interaction.FAMILIES if f.table == family)
    rebuilt: dict[str, dict] = {}
    for row in table(staged, family).to_pylist():
        if row["_artifact"] != file:
            continue
        key = row["pair"] if spec.pairs else row["CAS"]
        body = {}
        for field in spec.fields:
            value = row[field.column]
            if value is not None:
                body[field.key] = value
        rebuilt[key] = body
    source = raw_json(file)["data"]
    # a JSON null stays a missing key here; compare only the non-null source entries
    expected = {k: {n: v for n, v in body.items() if v is not None} for k, body in source.items()}
    assert rebuilt == expected


@needs_store
def test_parameter_files_repeat_the_metadata(staged: Path) -> None:
    rows = {r["_artifact"]: r for r in table(staged, "parameter_files").to_pylist()}
    assert len(rows) == 16
    for artifact, row in rows.items():
        metadata = raw_json(artifact)["metadata"]
        assert row["necessary_keys"] == metadata["necessary keys"]
        assert row["source"] == metadata["source"]
        assert json.loads(row["missing"]) == metadata["missing"]
        assert row["T_dependent"] == metadata.get("T dependent")
    pr = rows[PR]
    assert pr["symmetric"] is True and pr["components"] == 2 and pr["type"] == "PR kij"


@needs_store
def test_correlations_rebuild_every_source_document(staged: Path) -> None:
    leaves = table(staged, "correlation_leaves").to_pylist()
    series = table(staged, "correlation_series").to_pylist()
    rebuilt: dict[str, dict] = {}
    for row in leaves:
        leaf: dict = {}
        for name in (*thermo.correlations._SCALARS, *thermo.correlations._INTEGERS, *thermo.correlations._ARRAYS):
            if row[name] is not None:
                leaf[name] = row[name]
        rebuilt.setdefault(row["_artifact"], {}).setdefault(row["key"], {}).setdefault(
            row["property"], {}
        ).setdefault(row["model_key"], {})[row["label"]] = leaf
    for row in sorted(series, key=lambda r: (r["_artifact"], r["series_index"])):
        slot = rebuilt[row["_artifact"]][row["key"]][row["property"]][row["model_key"]]
        if row["field"] is None:
            slot[row["label"]] = slot[row["label"]] or []
            slot[row["label"]].append(row["values"])
        else:
            slot[row["label"]].setdefault(row["field"], []).append(row["values"])
    for name in thermo.correlations.FILES:
        artifact = f"thermo/Misc/{name}"
        assert rebuilt[artifact] == raw_json(artifact), artifact


@needs_store
def test_ipd_rows_reproduce_the_file_lines(staged: Path) -> None:
    rows = table(staged, "ipd_rows").to_pylist()
    for path in thermo.ipd.FILES:
        source = raw_text(path).split("\r\n")
        mine = [r for r in rows if r["_artifact"] == path]
        assert mine
        for row in mine:
            number = int(row["_locator"].rsplit("#L", 1)[1])
            tokens = source[number - 1].split()
            assert (row["ID1"], row["ID2"]) == (tokens[0], tokens[1])
            assert row["values"] == [float(t) for t in tokens[2 : 2 + len(row["values"])]]
            tail = source[number - 1].split(None, 2 + len(row["values"]))
            assert row["remainder"] == (tail[2 + len(row["values"])] if len(tail) > 2 + len(row["values"]) else None)
    files = {r["_artifact"]: r for r in table(staged, "ipd_files").to_pylist()}
    nrtl = files["thermo/Interaction Parameters/ChemSep/nrtl.ipd"]
    assert nrtl["Units"] == "cal/mol" and nrtl["value_names"] == ["A12", "A21", "alpha12"]
    assert files["thermo/Interaction Parameters/ChemSep/pr.ipd"]["Units"] is None
    assert files["thermo/Interaction Parameters/ChemSep/pr.ipd"]["value_names"] == ["k12"]


@needs_store
def test_interaction_tables_reproduce_the_source_cells(staged: Path) -> None:
    rows = table(staged, "unifac_interaction_parameters").to_pylist()
    for artifact in thermo.unifac.INTERACTION_FILES:
        reader = csv.reader(io.StringIO(raw_text(artifact), newline=""), delimiter="\t", quoting=csv.QUOTE_NONE)
        expected = list(reader)
        mine = [r for r in rows if r["_artifact"] == artifact]
        assert len(mine) == len(expected)
        for cells, row in zip(expected, mine, strict=True):
            assert (row["m"], row["n"]) == (int(cells[0]), int(cells[1]))
            for name, cell in zip(("a", "b", "c", "Tmin", "Tmax"), cells[2:], strict=False):
                assert row[name] == float(cell), (artifact, row["_locator"], name)
            for name in ("a", "b", "c", "Tmin", "Tmax")[len(cells) - 2 :]:
                assert row[name] is None


@needs_store
def test_ddbst_rows_reproduce_the_source_lines(staged: Path) -> None:
    rows = table(staged, "ddbst_unifac_assignments").to_pylist()
    pairs = table(staged, "ddbst_unifac_assignment_pairs").to_pylist()
    by_key: dict[str, dict[str, list]] = {}
    for pair in pairs:
        by_key.setdefault(pair["InChIKey"], {}).setdefault(pair["scheme"], []).append(
            (pair["pair_index"], pair["subgroup_id"], pair["count"])
        )
    source = raw_text(thermo.unifac.DDBST_FILE).split("\n")[:-1]
    assert len(rows) == len(source)
    for line, row in zip(source, rows, strict=True):
        key, flags, *cells = line.split("\t")
        assert key == row["InChIKey"]
        assert flags == f"{row['UNIFAC_flag']} {row['modified_flag']} {row['PSRK_flag']}"
        assert cells == [row["UNIFAC_cell"], row["modified_cell"], row["PSRK_cell"]]
        for scheme, cell in zip(("UNIFAC", "modified", "PSRK"), cells, strict=True):
            rebuilt = "".join(f"{s} {c} " for _, s, c in sorted(by_key[key][scheme]))
            assert rebuilt == cell


@needs_store
def test_unifac_source_rows_match_the_constructor_lines(staged: Path) -> None:
    source = raw_text("thermo/unifac.py").split("\n")
    by_id = {
        (r["set_name"], r["group_id"]): r for r in table(staged, "unifac_subgroups").to_pylist()
    }
    first = by_id[("UFSG", 1)]
    line = source[int(first["_locator"].rsplit("#L", 1)[1]) - 1]
    numbers = re.match(r'UFSG\[1\] = UNIFAC_subgroup\(1, "CH3", 1, "CH2", ([\d.]+), ([\d.]+)', line)
    assert numbers is not None
    assert (first["R"], first["Q"]) == (float(numbers.group(1)), float(numbers.group(2)))
    assert first["smarts"] == ["[CX4;H3]"] and json.loads(first["atoms_json"]) == {"C": 1, "H": 3}
    referring = by_id[("DOUFSG", 1)]
    assert referring["atoms_expr"] == "UFSG[1].atoms" and referring["atoms_json"] is None
    groups = {
        (r["set_name"], r["main_group_id"]): r for r in table(staged, "unifac_main_groups").to_pylist()
    }
    assert groups[("UFMG", 1)]["name"] == "CH2" and groups[("UFMG", 1)]["subgroup_ids"] == [1, 2, 3, 4]
    assert groups[("NISTUFMG", 1)]["description"] == "Alkyl chains"


# -- loading -----------------------------------------------------------------------------------------------


@needs_store
def test_the_staged_source_loads_into_a_database(staged: Path, test_database: TestDatabase) -> None:
    outcome = load.load_staged(test_database.url, staged, "src_thermo")
    manifest = staged_manifest.read(staged)
    assert outcome.tables == {name: record.rows for name, record in manifest.tables.items()}
    from thermo_knowledge import db

    with db.connect(test_database.url) as conn:
        row = conn.execute(
            'SELECT "A", "a" FROM src_thermo.correlation_leaves WHERE "a" IS NOT NULL LIMIT 1'
        ).fetchone()
        assert row is not None and row[0] is None and row[1] is not None
        (subgroups,) = conn.execute(
            "SELECT count(*) FROM src_thermo.unifac_subgroups WHERE smarts IS NOT NULL"
        ).fetchone()
        assert subgroups > 0
