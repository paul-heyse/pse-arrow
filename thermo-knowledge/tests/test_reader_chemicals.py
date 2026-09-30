# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The chemicals reader: synthetic payload files in the source's formats, then the real acquired
tree. Real-store counts are taken directly from the raw files, independently of the reader."""

from __future__ import annotations

import csv
import gzip
import io
import json
import math
import re
import zipfile
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq
import pytest
from tabular_reader_support import Staged, lines, stage

from thermo_knowledge import config
from thermo_knowledge.acquire import store
from thermo_knowledge.acquire.lock import read_lock
from thermo_knowledge.acquire.manifest import default_lock_path, default_sources_dir, load_sources
from thermo_knowledge.readers import chemicals
from thermo_knowledge.staging import load, stage as staging
from thermo_knowledge.staging import manifest as staged_manifest
from thermo_knowledge.staging import payload, schema
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.stage import StageContext
from thermo_knowledge.testing import TestDatabase

PIN = "7fca9acf7708"
TREE = store.pin_dir(config.raw_dir(), "chemicals", PIN) / store.TREE_DIR_NAME
needs_store = pytest.mark.skipif(
    not TREE.is_dir(),
    reason=f"the acquired chemicals tree {TREE} is absent (run `tk acquire chemicals`)",
)

PSRK = "chemicals/Critical Properties/Appendix to PSRK Revision 4.tsv"
WILSON = "chemicals/Critical Properties/wilson_jasperson_Tc_Pc_predictions.tsv"
GWP_2014 = "chemicals/Environment/Official Global Warming Potentials 2014.tsv"
PERRY = "chemicals/Heat Capacity/Perrys Table 2-151.tsv"


def run(tmp_path: Path, files: dict[str, str | bytes]) -> Staged:
    return stage(chemicals, tmp_path, files)


def meta(table: str, column: str) -> dict[str, str]:
    field = chemicals.TABLES[table].field(column)
    return {k.decode(): v.decode() for k, v in (field.metadata or {}).items()}


# -- declared schemas ---------------------------------------------------------------------------------


def test_every_table_is_declared_with_documented_typed_columns() -> None:
    assert len(chemicals.TABLES) == 151
    schema.check_declared(chemicals.TABLES)
    fields = chemicals.TABLES["critical_psrk_appendix"]
    assert [f.name for f in fields][2:] == ["CAS", "Chemical", "Tc", "Pc", "Vc", "omega"]
    assert fields.field("Pc").type == pa.float64()
    assert fields.field("CAS").type == pa.string()
    assert chemicals.TABLES["law_tsca"].field("UV").type == pa.bool_()
    assert chemicals.TABLES["law_einecs"].field("CASRN").type == pa.int64()
    assert chemicals.TABLES["identifiers_chemical"].field("synonyms").type == pa.list_(pa.string())
    assert chemicals.TABLES["chemsep_equations"].field("eqno").type == pa.int64()


def test_units_are_stated_only_where_the_file_states_them() -> None:
    assert meta("environment_gwp_2007", "Lifetime")["unit"] == "years"
    assert meta("environment_gwp_2007", "Radiative_efficiency")["unit"] == "W/m^2/ppb"
    assert meta("electrolytes_marcus_ion_conductivities", "Conductivity")["unit"] == (
        "cm^2 S^-1 mol^-1"
    )
    assert meta("heat_capacity_crc_solids", "Cp_300") == {
        "source_name": "300",
        "unit": "J/mol/K",
        "note": meta("heat_capacity_crc_solids", "Cp_300")["note"],
    }
    assert meta("critical_psrk_appendix", "Tc")["unit"] == "not stated"
    assert meta("safety_ontario_exposure_limits_json", "TWA_mg_m3")["unit"] == "mg/m^3"
    assert meta("chemsep_equations", "Tmin")["note"] == "unit in Tmin_units"


def test_repeated_and_empty_headings_get_distinct_documented_columns() -> None:
    laliberte = chemicals.TABLES["electrolytes_laliberte_2009"]
    names = laliberte.names
    assert {"Min_T", "Min_T_2", "Min_T_3"} <= set(names)
    assert meta("electrolytes_laliberte_2009", "Min_T_2")["source_name"] == "Min T"
    assert "occurrence 2" in meta("electrolytes_laliberte_2009", "Min_T_2")["note"]
    element = chemicals.TABLES["misc_element_data"]
    assert {"column_13", "column_14"} <= set(element.names)
    assert "empty heading" in meta("misc_element_data", "column_13")["source_name"]


# -- handbook tables ------------------------------------------------------------------------------------


def test_a_handbook_table_keeps_the_source_values_and_locators(tmp_path: Path) -> None:
    content = lines(
        "CAS|Chemical|Tc|Pc|Vc|omega",
        "1-01-1|Synthetic one |500.5|4600000||0.25",
        "2-02-2|Synthetic two|0|1e6|1.5e-4|-.5",
    )
    staged = run(tmp_path, {PSRK: content})
    first, second = staged.rows("critical_psrk_appendix")
    assert first["_artifact"] == PSRK
    assert first["_locator"] == f"{PSRK}#L2" and second["_locator"] == f"{PSRK}#L3"
    assert first["Chemical"] == "Synthetic one "  # trailing space kept
    assert first["Pc"] == 4600000.0 and isinstance(first["Pc"], float)
    assert first["Vc"] is None  # blank is missing, not zero
    assert second["Tc"] == 0.0 and second["Vc"] == 1.5e-4 and second["omega"] == -0.5
    assert staged.payload[PSRK].status == "read"


def test_a_cas_without_hyphens_is_an_integer_and_nan_stays_distinct(tmp_path: Path) -> None:
    content = lines("CAS|Tc|Pc", "50000|nan|", "50011|600.0|nan", "50028|650|1.2e7")
    rows = run(tmp_path, {WILSON: content}).rows("critical_wilson_jasperson_predictions")
    assert [r["CAS"] for r in rows] == [50000, 50011, 50028]
    assert math.isnan(rows[0]["Tc"]) and rows[0]["Pc"] is None
    assert rows[1]["Tc"] == 600.0 and math.isnan(rows[1]["Pc"])


def test_a_space_only_numeric_cell_is_missing_and_text_is_not_trimmed(tmp_path: Path) -> None:
    content = lines(
        "Ful CAS|Formula|main phase|subphase|Constant Term|+*T term|/T^2 term|*T^2 term|Tmin|Tmax|%error",
        "|Ag2Al |c||16.85|0.0045|||273|903|2",
        "|AlF3 |c||50.5||||288|326| ",
    )
    rows = run(tmp_path, {PERRY: lines(*content_rows(content))}).rows("heat_capacity_perry_2_151")
    assert rows[1]["Tmax"] == 326.0
    assert rows[1]["percent_error"] == " "  # text is kept exactly
    assert rows[1]["plus_T_term"] is None
    assert rows[0]["Formula"] == "Ag2Al "
    assert rows[0]["percent_error"] == "2"


def content_rows(content: str) -> list[str]:
    return [line.replace("\t", "|") for line in content.rstrip("\n").split("\n")]


def test_a_quoted_table_is_parsed_with_its_quotes(tmp_path: Path) -> None:
    head = "|".join(
        f'"{h}"'
        for h in (
            "CAS|Name|Formula|Lifetime, years|Radiative efficiency, W/m^2/ppb|20yr GWP|100yr GWP|"
            "20yr GTP|50yr GTP|100yr GTP|20yr AGWP|100yr AGWP|20yr AGTP|50yr AGTP|100yr AGTP"
        ).split("|")
    )
    row = '"1-1-1"|"Synthetic gas"|"CF4"|5|0.25|1|2|3|4|5|6|7|8|9|10'
    rows = run(tmp_path, {GWP_2014: lines(head, row)}).rows("environment_gwp_2014")
    assert rows[0]["CAS"] == "1-1-1" and rows[0]["Lifetime"] == 5.0
    assert rows[0]["x_100yr_AGTP"] == 10.0


def test_gzip_and_zip_files_are_read_and_locators_count_decompressed_lines(tmp_path: Path) -> None:
    dsl = gzip.compress(b"CASRN\tRegistry\n50000\t0\n50011\t1\n")
    archive = io.BytesIO()
    with zipfile.ZipFile(archive, "w") as handle:
        handle.writestr("member.csv", "100-00-5\tBand A\n100-01-6\tBand B\n")
    hpv = "2091294\n100005\n"
    files = {
        "chemicals/Law/Canada Feb 11 2015 - DSL.csv.gz": dsl,
        "chemicals/Law/ECHA Tonnage Bands.csv.zip": archive.getvalue(),
        "chemicals/Law/HPV 2015 March 3.csv": hpv,
    }
    staged = run(tmp_path, files)
    assert [r["CASRN"] for r in staged.rows("law_canada_dsl")] == [50000, 50011]
    assert staged.rows("law_canada_dsl")[1]["Registry"] == 1
    echa = staged.rows("law_echa_tonnage_bands")
    assert echa[1]["CASRN"] == "100-01-6" and echa[1]["Tonnage_band"] == "Band B"
    assert echa[0]["_locator"].endswith("ECHA Tonnage Bands.csv.zip#L1")
    hpv_rows = staged.rows("law_hpv")  # no heading: the first line is a data row
    assert [r["CASRN"] for r in hpv_rows] == [2091294, 100005]
    assert hpv_rows[0]["_locator"].endswith("#L1")


def test_boolean_columns_take_only_the_two_literals(tmp_path: Path) -> None:
    flags = "UV|E|F|N|P|S|R|T|XU|SP|TP|Y1|Y2"
    path = "chemicals/Law/TSCA Inventory 2016-01.csv.gz"
    good = gzip.compress(lines(f"CASRN|{flags}", "51456|" + "|".join(["True"] + ["False"] * 12)).encode())
    row = run(tmp_path, {path: good}).rows("law_tsca")[0]
    assert row["UV"] is True and row["Y2"] is False
    bad = gzip.compress(lines(f"CASRN|{flags}", "51456|" + "|".join(["yes"] + ["False"] * 12)).encode())
    with pytest.raises(StagingError, match=r"TSCA Inventory 2016-01\.csv\.gz#L2, column UV"):
        run(tmp_path / "bad", {path: bad})


# -- malformed input is refused with its location --------------------------------------------------------


def test_a_bad_number_is_refused_with_file_line_and_column(tmp_path: Path) -> None:
    content = lines("CAS|Chemical|Tc|Pc|Vc|omega", "1-01-1|A|500|4600000||0.25", "2-02-2|B|abc|1||1")
    with pytest.raises(StagingError, match=rf"{re.escape(PSRK)}#L3, column Tc: 'abc'"):
        run(tmp_path, {PSRK: content})


def test_changed_headings_are_refused(tmp_path: Path) -> None:
    content = lines("CAS|Chemical|Tc|Pc|Vc|acentric", "1-01-1|A|500|4600000||0.25")
    with pytest.raises(StagingError, match=r"#L1: the headings .* differ from the declared"):
        run(tmp_path, {PSRK: content})


def test_a_wrong_width_and_a_blank_line_are_refused(tmp_path: Path) -> None:
    head = "CAS|Chemical|Tc|Pc|Vc|omega"
    with pytest.raises(StagingError, match=r"#L2: 5 cells, the table declares 6"):
        run(tmp_path / "a", {PSRK: lines(head, "1-01-1|A|500|4600000|")})
    with pytest.raises(StagingError, match=r"#L3: blank line"):
        run(tmp_path / "b", {PSRK: lines(head, "1-01-1|A|500|4600000||0.25") + "\n"})


def test_a_payload_file_without_a_rule_is_refused(tmp_path: Path) -> None:
    with pytest.raises(StagingError, match=r"no rule for this payload file"):
        run(tmp_path, {"chemicals/Misc/new table.tsv": "CAS\n1-1-1\n"})


# -- identifiers ------------------------------------------------------------------------------------------


def test_identifier_rows_are_positional_with_a_list_of_synonyms(tmp_path: Path) -> None:
    path = "chemicals/Identifiers/Cation db.tsv"
    content = lines(
        "-1|100-00-1|Xy+2|55.5|[Xy+2]||KEY-ONE|synthetic ion|synthetic|alpha||beta",
        "42|100-00-2|Z|1.0|Z||KEY-TWO|zed|zed",
    )
    rows = run(tmp_path, {path: content}).rows("identifiers_chemical")
    assert rows[0]["pubchemid"] == -1  # the file's placeholder, not a missing marker
    assert rows[0]["InChI"] is None and rows[0]["smiles"] == "[Xy+2]"
    assert rows[0]["synonyms"] == ["alpha", None, "beta"]
    assert rows[1]["synonyms"] == [] and rows[1]["MW"] == 1.0
    assert rows[0]["_locator"] == f"{path}#L1"
    with pytest.raises(StagingError, match=r"#L1: 8 cells, an identifier row has at least 9"):
        run(tmp_path / "short", {path: lines("1|2|3|4|5|6|7|8")})


def test_preferences_fake_cas_and_dippr_lists(tmp_path: Path) -> None:
    files = {
        "chemicals/Identifiers/inorganic_preferences.json": json.dumps(
            {"preferred_cas": ["1-1-1"], "unpreferred_cas": ["2-2-2", "3-3-3"]}
        ),
        "chemicals/Identifiers/organic_preferences.json": json.dumps(
            {"preferred_cas": [], "unpreferred_cas": []}
        ),
        "chemicals/Identifiers/Fake CAS Registry.tsv": lines(
            "2099995000-00-0|smiles|[OH-]||", "2099979000-00-0|formula|P4|name|white", "2099000000-00-0||||"
        ),
        "chemicals/Identifiers/dippr_2014.csv": "74-82-8\n74-82-8\n",
    }
    staged = run(tmp_path, files)
    prefs = staged.rows("identifiers_preferences")
    assert [(r["list_name"], r["list_index"], r["CAS"]) for r in prefs] == [
        ("preferred_cas", 0, "1-1-1"),
        ("unpreferred_cas", 0, "2-2-2"),
        ("unpreferred_cas", 1, "3-3-3"),
    ]
    assert staged.payload["chemicals/Identifiers/organic_preferences.json"].status == "read"
    fake = staged.rows("identifiers_fake_cas")
    assert fake[0]["value_2"] is None and fake[1]["key_2"] == "name"
    assert fake[2]["key_1"] is None  # a reserved number with no assignment
    assert [r["CAS"] for r in staged.rows("identifiers_dippr_2014")] == ["74-82-8", "74-82-8"]


def test_mixture_compositions_split_the_per_component_blocks(tmp_path: Path) -> None:
    path = "chemicals/Identifiers/Mixtures Compositions.tsv"
    head = "Primary Name|Source|N components|CASRNs*N|Names*N|mass fracs*N|mole fracs*N|Synonyms||"
    row = "Mix |Synthetic|2|1-1-1|2-2-2|One|Two|0.6|0.4|0.5|0.5|blend||"
    rows = run(tmp_path, {path: lines(head, row)})
    mixture = rows.rows("identifiers_mixtures")[0]
    assert mixture["primary_name"] == "Mix " and mixture["n_components"] == 2
    assert mixture["synonyms"] == ["blend"]  # the blank padding is dropped
    components = rows.rows("identifiers_mixture_components")
    assert [(c["CASRN"], c["name"], c["mass_fraction"], c["mole_fraction"]) for c in components] == [
        ("1-1-1", "One", 0.6, 0.5),
        ("2-2-2", "Two", 0.4, 0.5),
    ]
    assert components[1]["_locator"] == f"{path}#L2[1]"
    with pytest.raises(StagingError, match=r"2 components need 11"):
        run(tmp_path / "short", {path: lines(head, "Mix |S|2|1-1-1|2-2-2|One|Two|0.6|0.4")})


# -- JSON files -------------------------------------------------------------------------------------------


JANAF = "chemicals/Heat Capacity/JANAF_1998_gas_Cp.json"
PERRY_JSON = "chemicals/Heat Capacity/Perrys Table 2-151.json"
SHOMATE = "chemicals/Heat Capacity/webbook_shomate_coefficients.json"
VDI = "chemicals/Misc/VDI Saturation Compounds Data.json"
ONTARIO = "chemicals/Safety/Ontario Exposure Limits.json"


def test_point_tables_become_one_row_per_point(tmp_path: Path) -> None:
    document = {"1-1-1": [[0.0, 100.0, 298.15], [0.0, 29.1, None]]}
    staged = run(tmp_path, {JANAF: json.dumps(document)})
    rows = staged.rows("heat_capacity_janaf_gas_cp")
    assert [(r["point_index"], r["temperature"], r["cp"]) for r in rows] == [
        (0, 0.0, 0.0),
        (1, 100.0, 29.1),
        (2, 298.15, None),
    ]
    assert rows[2]["_locator"] == f"{JANAF}#/1-1-1/2"
    with pytest.raises(StagingError, match=r"parallel arrays of different lengths"):
        run(tmp_path / "bad", {JANAF: json.dumps({"1-1-1": [[1.0, 2.0], [1.0]]})})


def test_perry_entries_keep_a_string_where_the_file_has_one(tmp_path: Path) -> None:
    entry = {
        "Formula": "Xy", "Phase": "c", "Subphase": None, "Const": 4, "Lin": 0, "Quadinv": 0,
        "Quad": 0, "Tmin": 273.0, "Tmax": 931.0, "Error": "2a",
    }  # fmt: skip
    other = {**entry, "Phase": "l", "Error": 5.0, "Tmax": ""}
    staged = run(tmp_path, {PERRY_JSON: json.dumps({"1-1-1": {"c": entry, "l": other}})})
    first, second = staged.rows("heat_capacity_perry_2_151_json")
    assert first["Error"] is None and first["Error_text"] == "2a"
    assert second["Error"] == 5.0 and second["Error_text"] is None
    assert second["Tmax"] is None and second["Tmax_text"] == ""
    assert first["Const"] == 4.0 and first["Subphase"] is None
    bad = {"1-1-1": {"c": {**entry, "Extra": 1}}}
    with pytest.raises(StagingError, match=r"keys no column declares: Extra"):
        run(tmp_path / "bad", {PERRY_JSON: json.dumps(bad)})


def test_shomate_slots_and_pieces(tmp_path: Path) -> None:
    piece = [298.0, 1249.0, 1.0, 2.0, 3.0, 4.0, 5.0]
    document = {"1-1-1": [[piece], None, [piece, [1249.0, 3000.0, 6.0, 7.0, 8.0, 9.0, 10.0]]]}
    rows = run(tmp_path, {SHOMATE: json.dumps(document)}).rows("heat_capacity_webbook_shomate")
    assert [(r["slot"], r["piece_index"]) for r in rows] == [(0, 0), (2, 0), (2, 1)]
    assert rows[2]["Tmin"] == 1249.0 and rows[2]["E"] == 10.0
    with pytest.raises(StagingError, match=r"expected 7 values"):
        run(tmp_path / "bad", {SHOMATE: json.dumps({"1-1-1": [[[1.0]], None, None]})})


def test_vdi_compounds_and_points_and_ontario_units(tmp_path: Path) -> None:
    vdi = {
        "1-1-1": {
            "Name": "Synthetic", "MW": 2.0, "Tc": 33.0, "T": [20.0, 21.0], "P": [1.0, 2.0],
            "Density (l)": [70.0, None], "Density (g)": [1.0, 2.0], "Hvap": [1.0, 2.0],
            "Cp (l)": [1.0, 2.0], "Cp (g)": [1.0, 2.0], "Mu (l)": [1.0, 2.0], "Mu (g)": [1.0, 2.0],
            "K (l)": [1.0, 2.0], "K (g)": [1.0, 2.0], "Pr (l)": [1.0, 2.0], "Pr (g)": [1.0, 2.0],
            "sigma": [1.0, 2.0], "Beta": [1.0, 2.0], "Volume (l)": [1.0, 2.0],
            "Volume (g)": [1.0, 2.0],
        }
    }  # fmt: skip
    ontario = {
        "2-2-2": {
            "Name": "Synthetic [2-2-2]", "TWA (ppm)": 100.0, "TWA (mg/m^3)": None,
            "STEL (ppm)": None, "STEL (mg/m^3)": None, "Ceiling (ppm)": None,
            "Ceiling (mg/m^3)": None, "Skin": True, "MW": None,
        }
    }  # fmt: skip
    staged = run(tmp_path, {VDI: json.dumps(vdi), ONTARIO: json.dumps(ontario)})
    assert staged.rows("misc_vdi_saturation_compounds")[0]["Tc"] == 33.0
    points = staged.rows("misc_vdi_saturation_points")
    assert [p["Density_l"] for p in points] == [70.0, None]
    assert points[1]["T"] == 21.0
    row = staged.rows("safety_ontario_exposure_limits_json")[0]
    assert row["TWA_ppm"] == 100.0 and row["TWA_mg_m3"] is None and row["Skin"] is True
    with pytest.raises(StagingError, match=r"keys no column declares: Extra"):
        run(tmp_path / "bad", {ONTARIO: json.dumps({"2-2-2": {**ontario["2-2-2"], "Extra": 1}})})


# -- ChemSep XML ------------------------------------------------------------------------------------------

CHEMSEP = "chemicals/Misc/ChemSep8.32.xml"
_XML = """<compounds library="synthetic library" file="a.xml" create-date="1-1-2000" create-time="0:00:00" create-user="nobody" >

<compound>
<LibraryIndex name="Index"  value="7" />
<CompoundID name="Name"  value="Synthetic" />
<CriticalTemperature name="Critical temperature"  units="K"  value="132.45" />
<CAS name="CAS Number"  value="1-1-1" />
<LiquidDensity name="Liquid density"  units="kmol/m3" >
  <eqno value="105" />
  <A value="2.6731" />
  <B value="1e-3" />
  <Tmin units="K"  value="59.15" />
  <Tmax units="K"  value="132.5" />
  </LiquidDensity>
<Asog name="ASOG" >
  <group id="30"  value="2" />
  <group id="31"  value="1" />
  </Asog>
</compound>
</compounds>
"""


def test_chemsep_scalars_equations_and_groups(tmp_path: Path) -> None:
    staged = run(tmp_path, {CHEMSEP: _XML})
    library = staged.rows("chemsep_library")[0]
    assert library["library"] == "synthetic library" and library["create_user"] == "nobody"
    scalars = staged.rows("chemsep_scalars")
    assert [(s["element"], s["value"], s["value_number"], s["units"]) for s in scalars] == [
        ("LibraryIndex", "7", 7.0, None),
        ("CompoundID", "Synthetic", None, None),
        ("CriticalTemperature", "132.45", 132.45, "K"),
        ("CAS", "1-1-1", None, None),
    ]
    assert scalars[2]["_locator"].endswith("#/compounds/compound[0]/CriticalTemperature")
    (equation,) = staged.rows("chemsep_equations")
    assert equation["eqno"] == 105 and equation["A"] == 2.6731 and equation["B"] == 1e-3
    assert equation["C"] is None  # the element has no C child
    assert equation["Tmin"] == 59.15 and equation["Tmin_units"] == "K"
    groups = staged.rows("chemsep_groups")
    assert [(g["group_position"], g["group_id"], g["count"]) for g in groups] == [(0, 30, 2), (1, 31, 1)]


def test_chemsep_refuses_constructs_no_table_declares(tmp_path: Path) -> None:
    strange = _XML.replace('<Asog name="ASOG" >', '<Asog name="ASOG" extra="1" >')
    with pytest.raises(StagingError, match=r"attributes no column declares: extra"):
        run(tmp_path / "a", {CHEMSEP: strange})
    unknown_part = _XML.replace('<B value="1e-3" />', '<Z value="1" />')
    with pytest.raises(StagingError, match=r"an equation part no column declares"):
        run(tmp_path / "b", {CHEMSEP: unknown_part})
    with pytest.raises(StagingError, match=r"cannot be read as XML"):
        run(tmp_path / "c", {CHEMSEP: "<compounds><compound>"})


# -- the real tree ---------------------------------------------------------------------------------------


@pytest.fixture(scope="module")
def staged(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """The real `tk read chemicals`, into a temporary staged directory."""
    ctx = StageContext(staged_dir=tmp_path_factory.mktemp("staged"))
    manifests = load_sources(default_sources_dir())
    outcome = staging.read_source(ctx, manifests["chemicals"], read_lock(default_lock_path()))
    assert outcome.status == "read"
    return staging.staged_path(ctx, "chemicals", PIN)


@pytest.fixture(scope="module")
def manifest(staged: Path) -> staged_manifest.StagedManifest:
    return staged_manifest.read(staged)


def table(staged: Path, name: str) -> pa.Table:
    return pq.read_table(staged / f"{name}.parquet")


def raw_text(artifact: str) -> str:
    data = (TREE / artifact).read_bytes()
    if artifact.endswith(".gz"):
        data = gzip.decompress(data)
    elif artifact.endswith(".zip"):
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            data = archive.read(archive.namelist()[0])
    return data.decode("utf-8")


def physical_lines(artifact: str) -> int:
    """Counted from the bytes: newline characters, plus a last line without one."""
    data = raw_text(artifact).encode()
    return data.count(b"\n") + (0 if data.endswith(b"\n") else 1)


@needs_store
def test_every_table_is_staged_and_every_payload_file_accounted(
    manifest: staged_manifest.StagedManifest,
) -> None:
    assert set(manifest.tables) == set(chemicals.TABLES)
    assert manifest.reader.name == "chemicals" and manifest.reader.version == chemicals.READER_VERSION
    assert manifest.pin == PIN
    source = load_sources(default_sources_dir())["chemicals"]
    expected = payload.payload_files(TREE, source.payload.include, source.payload.exclude)
    assert len(expected) == 154
    assert sorted(r.path for r in manifest.payload) == expected
    assert all(record.status == "read" for record in manifest.payload)
    assert all(record.rows > 0 for record in manifest.tables.values())


@needs_store
def test_every_staged_column_is_documented(staged: Path) -> None:
    for name in chemicals.TABLES:
        for field in pq.read_schema(staged / f"{name}.parquet"):
            metadata = {k.decode(): v.decode() for k, v in (field.metadata or {}).items()}
            assert metadata.get("source_name"), (name, field.name)
            assert metadata.get("unit"), (name, field.name)


@needs_store
def test_delimited_row_counts_match_the_line_counts_of_the_raw_files(
    manifest: staged_manifest.StagedManifest,
) -> None:
    for spec in chemicals.DELIMITED:
        expected = sum(
            physical_lines(f"{chemicals.DATA_ROOT}/{rel}") - (1 if spec.headings else 0)
            for rel in spec.files
        )
        assert manifest.tables[spec.table].rows == expected, spec.table
    assert len(chemicals.DELIMITED) == 131


@needs_store
def test_identifier_and_list_counts_match_the_survey_and_the_files(
    manifest: staged_manifest.StagedManifest,
) -> None:
    lines_of = {
        path: physical_lines(path) for path in chemicals.identifiers.IDENTIFIER_FILES
    }
    assert manifest.tables["identifiers_chemical"].rows == sum(lines_of.values()) == 76500
    assert manifest.tables["identifiers_fake_cas"].rows == 999
    assert manifest.tables["identifiers_dippr_2014"].rows == 2278
    assert manifest.tables["identifiers_mixtures"].rows == physical_lines(chemicals.identifiers.MIXTURE_FILE) - 1
    preferences = sum(
        len(json.loads(raw_text(path))[key])
        for path in chemicals.identifiers.PREFERENCE_FILES
        for key in ("preferred_cas", "unpreferred_cas")
    )
    assert manifest.tables["identifiers_preferences"].rows == preferences == 32


@needs_store
def test_json_row_counts_match_the_parsed_files(manifest: staged_manifest.StagedManifest) -> None:
    for artifact, name in chemicals.jsonfiles.JANAF.items():
        document = json.loads(raw_text(artifact))
        assert manifest.tables[name].rows == sum(len(v[0]) for v in document.values())
    perry = json.loads(raw_text(chemicals.jsonfiles.PERRY_FILE))
    assert manifest.tables["heat_capacity_perry_2_151_json"].rows == sum(len(v) for v in perry.values()) == 370
    for artifact, name in chemicals.jsonfiles.PSI4.items():
        document = json.loads(raw_text(artifact))
        assert manifest.tables[name].rows == sum(len(v) for v in document.values())
    shomate = json.loads(raw_text(chemicals.jsonfiles.SHOMATE_FILE))
    assert manifest.tables["heat_capacity_webbook_shomate"].rows == sum(
        len(slot) for slots in shomate.values() for slot in slots if slot
    ) == 1800
    vdi = json.loads(raw_text(chemicals.jsonfiles.VDI_FILE))
    assert manifest.tables["misc_vdi_saturation_compounds"].rows == len(vdi) == 58
    assert manifest.tables["misc_vdi_saturation_points"].rows == sum(len(v["T"]) for v in vdi.values())
    ontario = json.loads(raw_text(chemicals.jsonfiles.ONTARIO_FILE))
    assert manifest.tables["safety_ontario_exposure_limits_json"].rows == len(ontario) == 765


@needs_store
def test_chemsep_counts_match_a_line_scan_of_the_file(manifest: staged_manifest.StagedManifest) -> None:
    text = raw_text(chemicals.chemsep_xml.ARTIFACT)
    assert text.count("<compound>") == 431
    scalar = re.findall(r"^<(\w+)\s+name=\"[^\"]*\"\s+(?:units=\"[^\"]*\"\s+)?value=\"", text, re.M)
    opened = re.findall(r"^<(\w+)\s+name=\"[^\"]*\"(?:\s+units=\"[^\"]*\")?\s*>\s*$", text, re.M)
    equations = text.count("<eqno ")
    groups = text.count("<group ")
    assert manifest.tables["chemsep_scalars"].rows == len(scalar) == 19071
    assert manifest.tables["chemsep_equations"].rows == equations == 6955
    assert manifest.tables["chemsep_groups"].rows == groups == 5809
    assert len(opened) == equations + len(re.findall(r"^<(?:GCmethod|Umr|UnifacVLE|UnifacLLE|ModifiedUnifac|Asog) ", text, re.M))


# -- round trips ---------------------------------------------------------------------------------------


def independent_cell(kind: str, cell: str) -> object:
    """What a cell should stay as, by Python's own parsers rather than the reader's."""
    if kind == "s":
        return cell or None
    if not cell.strip():
        return None
    if kind == "f":
        return float(cell)
    if kind in ("i", "c"):
        return int(cell)
    return cell == "True"


def same(left: object, right: object) -> bool:
    if isinstance(left, float) and isinstance(right, float) and math.isnan(left):
        return math.isnan(right)
    return left == right


@needs_store
@pytest.mark.parametrize(
    "name",
    [
        "critical_psrk_appendix",
        "critical_wilson_jasperson_predictions",
        "heat_capacity_perry_2_151",
        "environment_gwp_2014",
        "electrolytes_laliberte_2009",
        "misc_element_data",
        "law_tsca",
        "law_echa_tonnage_bands",
        "vapor_pressure_landolt_antoine_v20",
        "reactions_yaws_hf_s0_gas",
    ],
)
def test_delimited_rows_reproduce_the_source_cells(staged: Path, name: str) -> None:
    spec = next(s for s in chemicals.DELIMITED if s.table == name)
    staged_rows = table(staged, name).to_pylist()
    expected: list[list[str]] = []
    for rel in spec.files:
        reader = csv.reader(
            io.StringIO(raw_text(f"{chemicals.DATA_ROOT}/{rel}"), newline=""),
            delimiter="\t",
            quotechar='"' if spec.quoted else None,
            quoting=csv.QUOTE_MINIMAL if spec.quoted else csv.QUOTE_NONE,
        )
        rows = list(reader)
        expected.extend(rows[1:] if spec.headings else rows)
    assert len(expected) == len(staged_rows)
    for cells, row in zip(expected, staged_rows, strict=True):
        assert len(cells) == len(spec.columns)
        for column, cell in zip(spec.columns, cells, strict=True):
            assert same(row[column.name], independent_cell(column.kind, cell)), (
                row["_locator"],
                column.name,
            )


@needs_store
def test_identifier_rows_reproduce_the_source_line(staged: Path) -> None:
    by_locator = {r["_locator"]: r for r in table(staged, "identifiers_chemical").to_pylist()}
    for artifact in chemicals.identifiers.IDENTIFIER_FILES:
        source_lines = raw_text(artifact).split("\n")
        for number in (1, 2, len(source_lines) // 2, len(source_lines) - 1):
            row = by_locator[f"{artifact}#L{number}"]
            cells = source_lines[number - 1].split("\t")
            rebuilt = [
                "" if row["pubchemid"] is None else str(row["pubchemid"]),
                row["CAS"] or "",
                row["formula"] or "",
                "" if row["MW"] is None else cells[3],
                row["smiles"] or "",
                row["InChI"] or "",
                row["InChI_key"] or "",
                row["iupac_name"] or "",
                row["common_name"] or "",
                *[s or "" for s in row["synonyms"]],
            ]
            assert rebuilt == cells
            assert float(cells[3]) == row["MW"]


@needs_store
def test_json_tables_rebuild_the_source_documents(staged: Path) -> None:
    gas = json.loads(raw_text("chemicals/Heat Capacity/JANAF_1998_gas_Cp.json"))
    rebuilt: dict[str, list[list[float]]] = {}
    for row in table(staged, "heat_capacity_janaf_gas_cp").to_pylist():
        pair = rebuilt.setdefault(row["CAS"], [[], []])
        pair[0].append(row["temperature"])
        pair[1].append(row["cp"])
    assert rebuilt == gas

    perry = json.loads(raw_text(chemicals.jsonfiles.PERRY_FILE))
    rebuilt_perry: dict[str, dict[str, dict]] = {}
    for row in table(staged, "heat_capacity_perry_2_151_json").to_pylist():
        entry = {
            key: row[key] for key in ("Formula", "Phase", "Subphase", "Const", "Lin", "Quadinv", "Quad")
        }
        for key in ("Tmin", "Tmax", "Error"):
            entry[key] = row[f"{key}_text"] if row[f"{key}_text"] is not None else row[key]
        rebuilt_perry.setdefault(row["CAS"], {})[row["phase_key"]] = entry
    assert rebuilt_perry == perry  # equal numerically (4 == 4.0) and in the strings

    shomate = json.loads(raw_text(chemicals.jsonfiles.SHOMATE_FILE))
    slots: dict[str, list] = {cas: [None, None, None] for cas in shomate}
    for row in table(staged, "heat_capacity_webbook_shomate").to_pylist():
        slot = slots[row["CAS"]][row["slot"]] or []
        slot.append([row[k] for k in ("Tmin", "Tmax", "A", "B", "C", "D", "E")])
        slots[row["CAS"]][row["slot"]] = slot
    assert slots == shomate

    vdi = json.loads(raw_text(chemicals.jsonfiles.VDI_FILE))
    points = table(staged, "misc_vdi_saturation_points").to_pylist()
    hydrogen = sorted((p for p in points if p["CAS"] == "1333-74-0"), key=lambda p: p["point_index"])
    assert [p["T"] for p in hydrogen] == vdi["1333-74-0"]["T"]
    assert [p["Density_g"] for p in hydrogen] == vdi["1333-74-0"]["Density (g)"]
    nulls = sum(v is None for d in vdi.values() for k, v in ((k, x) for k, xs in d.items() if isinstance(xs, list) for x in xs))
    staged_nulls = sum(
        value is None
        for p in points
        for key, value in p.items()
        if not key.startswith("_") and key not in ("CAS", "point_index")
    )
    assert staged_nulls == nulls  # the 430 JSON nulls stay null, none became zero


@needs_store
def test_chemsep_properties_reproduce_the_xml_lines(staged: Path) -> None:
    text = raw_text(chemicals.chemsep_xml.ARTIFACT)
    scalar = re.compile(
        r'^<(\w+)\s+name="([^"]*)"\s+(?:units="([^"]*)"\s+)?value="([^"]*)"\s*/>$', re.M
    )
    expected = []
    for index, block in enumerate(text.split("<compound>")[1:]):
        for element, name, units, value in scalar.findall(block):
            expected.append((index, element, name, units or None, value))
    staged_scalars = [
        (r["compound_index"], r["element"], r["name"], r["units"], r["value"])
        for r in table(staged, "chemsep_scalars").to_pylist()
    ]
    assert staged_scalars == expected

    equation = re.compile(r'^<(\w+)\s+name="[^"]*"\s+units="[^"]*"\s*>$', re.M)
    parameter = re.compile(r'^\s+<(eqno|[A-E]) value="([^"]*)"\s*/>$', re.M)
    blocks = []
    for index, block in enumerate(text.split("<compound>")[1:]):
        for match in equation.finditer(block):
            end = block.index(f"</{match.group(1)}>", match.end())
            values = dict(parameter.findall(block[match.end() : end]))
            blocks.append((index, match.group(1), values))
    staged_equations = table(staged, "chemsep_equations").to_pylist()
    assert len(staged_equations) == len(blocks)
    for row, (index, tag, values) in zip(staged_equations, blocks, strict=True):
        assert (row["compound_index"], row["element"]) == (index, tag)
        assert row["eqno"] == int(values["eqno"])
        for letter in "ABCDE":
            assert same(row[letter], float(values[letter]) if letter in values else None)


# -- loading ---------------------------------------------------------------------------------------------


@needs_store
def test_the_staged_source_loads_into_a_database(staged: Path, test_database: TestDatabase) -> None:
    outcome = load.load_staged(test_database.url, staged, "src_chemicals")
    manifest = staged_manifest.read(staged)
    assert outcome.tables == {name: record.rows for name, record in manifest.tables.items()}
    from thermo_knowledge import db

    with db.connect(test_database.url) as conn:
        (count,) = conn.execute(
            'SELECT count(*) FROM src_chemicals."critical_psrk_appendix" WHERE "Vc" IS NULL'
        ).fetchone()
        assert count == 23  # the blank cells stay null
        (nan_count,) = conn.execute(
            'SELECT count(*) FROM src_chemicals."critical_wilson_jasperson_predictions" '
            "WHERE \"Tc\" = 'NaN'"
        ).fetchone()
        assert nan_count == 11
