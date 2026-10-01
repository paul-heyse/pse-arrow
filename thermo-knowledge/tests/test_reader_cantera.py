# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The Cantera reader: synthetic payloads in Cantera's YAML (schemas, locators, missing markers,
refusals) and the real acquired tree (row counts against independent line-pattern counts, round
trips). Counts on the real tree are computed here, from the raw text, independently of the
reader."""

from __future__ import annotations

import collections
import re
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq
import pytest

from thermo_knowledge import config
from thermo_knowledge.acquire import store
from thermo_knowledge.acquire.lock import read_lock
from thermo_knowledge.acquire.manifest import default_lock_path, default_sources_dir, load_sources
from thermo_knowledge.readers import cantera
from thermo_knowledge.readers.cantera import yaml12
from thermo_knowledge.readers.cantera.yamlfile import parse_equation
from thermo_knowledge.staging import load, stage
from thermo_knowledge.staging import manifest as staged_manifest
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.schema import check_declared
from thermo_knowledge.staging.stage import StageContext
from thermo_knowledge.staging.writer import Writer
from thermo_knowledge.testing import TestDatabase

PIN = "4a8358eb80cf"
TREE = store.pin_dir(config.raw_dir(), "cantera", PIN) / store.TREE_DIR_NAME


# -- synthetic payloads ------------------------------------------------------------------------


def run_reader(tmp_path: Path, files: dict[str, str]) -> tuple[dict[str, pa.Table], Writer]:
    """Run the reader over `files` (relative path to text) and return the tables it wrote."""
    tree = tmp_path / "tree"
    for name, content in files.items():
        path = tree / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
    out = tmp_path / "out"
    out.mkdir()
    writer = Writer(out, cantera.TABLES, sorted(files))
    cantera.read(tree, writer)
    result = writer.finish()
    tables = {name: pq.read_table(out / record.file) for name, record in result.tables.items()}
    return tables, writer


def rows(tables: dict[str, pa.Table], name: str) -> list[dict]:
    return tables[name].to_pylist()


MECHANISM = """\
description: A synthetic mechanism.
generator: ck2yaml
cantera-version: 2.6.0
date: Mon, 01 Jan 2024 00:00:00 +0000
input-files: [a.inp, a.dat]

units: {length: cm, quantity: mol, activation-energy: cal/mol}

phases:
- name: gas
  thermo: ideal-gas
  elements: [H, O, N]
  species: [{species: [H2, O2, NO, N]}, {other.yaml/species: all}]
  kinetics: gas
  reactions: all
  transport: mixture-averaged
  state: {T: 300.0, P: 1 atm}
- name: surf
  thermo: ideal-surface
  species: surface-species
  site-density: 2.7e-09 mol/cm^2
  adjacent-phases: [gas]

species:
- name: H2
  composition: {H: 2}
  thermo:
    model: NASA7
    temperature-ranges: [200.0, 1000.0, 3500.0]
    data:
    - [2.34433112, 0.00798052075, -1.9478151e-05, 2.01572094e-08, -7.37611761e-12,
      -917.935173, 0.683010238]
    - [3.3372792, -4.94024731e-05, 4.99456778e-07, -1.79566394e-10, 2.00255376e-14,
      -950.158922, -3.20502331]
    note: TPIS78
  transport:
    model: gas
    geometry: linear
    well-depth: 38.0
    diameter: 2.92
    polarizability: 0.79
    rotational-relaxation: 280.0
- name: NO
  composition: {N: 1, O: 1}
  thermo:
    model: constant-cp
    T0: 298.15 K
    h0: -393.5 kJ/mol
    s0: 213.8
  sites: 0
  note:
- name: N
  composition: {}
  equation-of-state: {model: constant-volume, molar-volume: 1.3 cm^3/mol, units: {length: m}}
  critical-parameters: {critical-temperature: 33, critical-pressure: 1.3 MPa}

reactions:
- units: {length: m}
- equation: 2 H + M <=> H2 + M
  type: three-body
  rate-constant: {A: 1.0e+18, b: -1.0, Ea: 0.0}
  efficiencies: {H2: 2.5}
- equation: H + O2 (+ M) <=> HO2 (+M)
  type: falloff
  duplicate: true
  low-P-rate-constant: {A: 1e+10, b: 1, Ea: "1.2 kcal/mol"}
  Troe: {A: 0.5, T3: 1e-30}
  units: {quantity: molec}
- equation: A => B
"""


def test_every_table_is_declared_documented_and_located(tmp_path: Path) -> None:
    check_declared(cantera.TABLES)
    tables, _ = run_reader(tmp_path, {"data/m.yaml": MECHANISM})
    assert set(tables) == set(cantera.TABLES)
    for name, schema in cantera.TABLES.items():
        assert schema.names[:2] == ["_artifact", "_locator"], name
        for field in schema:
            metadata = {k.decode(): v.decode() for k, v in (field.metadata or {}).items()}
            assert metadata.get("source_name") and metadata.get("unit"), (name, field.name)
    for name, table in tables.items():
        for row in table.to_pylist():
            assert row["_artifact"] == "data/m.yaml"
            assert row["_locator"].startswith("data/m.yaml#"), (name, row["_locator"])


def test_file_level_metadata_sections_and_units(tmp_path: Path) -> None:
    tables, _ = run_reader(tmp_path, {"data/m.yaml": MECHANISM})
    (meta,) = rows(tables, "yaml_files")
    assert meta["_locator"] == "data/m.yaml#"
    assert meta["generator"] == "ck2yaml" and meta["cantera_version"] == "2.6.0"
    assert meta["input_files"] == ["a.inp", "a.dat"] and meta["git_commit"] is None
    assert meta["top_level_keys"] == [
        "description",
        "generator",
        "cantera-version",
        "date",
        "input-files",
        "units",
        "phases",
        "species",
        "reactions",
    ]
    sections = {r["section"]: r for r in rows(tables, "file_sections")}
    assert sections["reactions"]["entry_kinds"] == ["units", "reaction"]
    assert sections["reactions"]["entry_count"] == 4
    assert sections["units"]["shape"] == "mapping" and sections["generator"]["shape"] == "scalar"
    units = {
        (r["scope"], r["scope_path"], r["dimension"]): r["value_text"]
        for r in rows(tables, "units_entries")
    }
    assert units[("file", "", "length")] == "cm"
    assert units[("file", "", "activation-energy")] == "cal/mol"
    assert units[("section", "/reactions", "length")] == "m"
    assert units[("entry", "/reactions/2", "quantity")] == "molec"
    assert units[("entry", "/species/2/equation-of-state", "length")] == "m"
    locators = {r["_locator"] for r in rows(tables, "units_entries")}
    assert "data/m.yaml#/reactions/0/units/length" in locators


def test_species_thermo_pieces_and_quantities(tmp_path: Path) -> None:
    tables, _ = run_reader(tmp_path, {"data/m.yaml": MECHANISM})
    species = {r["name"]: r for r in rows(tables, "species")}
    assert species["H2"]["_locator"] == "data/m.yaml#/species/0"
    thermo = {r["species_locator"]: r for r in rows(tables, "species_thermo")}
    h2 = thermo["data/m.yaml#/species/0"]
    assert h2["model"] == "NASA7" and h2["piece_count"] == 2 and h2["note"] == "TPIS78"
    assert h2["temperature_ranges"] == [200.0, 1000.0, 3500.0]
    assert h2["_locator"] == "data/m.yaml#/species/0/thermo"
    pieces = sorted(rows(tables, "thermo_pieces"), key=lambda r: r["piece_index"])
    assert [(p["temperature_low"], p["temperature_high"]) for p in pieces] == [
        (200.0, 1000.0),
        (1000.0, 3500.0),
    ]
    assert pieces[0]["coefficients"][:2] == [2.34433112, 0.00798052075]
    assert pieces[1]["coefficients"][5] == -950.158922
    assert [p["coefficient_count"] for p in pieces] == [7, 7]
    assert pieces[1]["_locator"] == "data/m.yaml#/species/0/thermo/data/1"
    no = thermo["data/m.yaml#/species/1"]
    # a value with its own unit stays a string; a bare number stays a number
    assert no["h0"] is None and no["h0_text"] == "-393.5 kJ/mol"
    assert no["T0_text"] == "298.15 K" and no["s0"] == 213.8 and no["s0_text"] is None
    assert no["piece_count"] is None and no["temperature_ranges"] is None
    transport = rows(tables, "species_transport")
    assert len(transport) == 1 and transport[0]["well_depth"] == 38.0
    assert transport[0]["dipole"] is None
    critical = rows(tables, "species_critical_parameters")
    assert critical[0]["critical_temperature"] == 33.0
    assert critical[0]["critical_pressure"] is None
    assert critical[0]["critical_pressure_text"] == "1.3 MPa"


def test_missing_markers_stay_distinct_from_zero_and_null(tmp_path: Path) -> None:
    tables, _ = run_reader(tmp_path, {"data/m.yaml": MECHANISM})
    species = {r["name"]: r for r in rows(tables, "species")}
    assert species["NO"]["sites"] == 0.0  # written zero
    assert species["H2"]["sites"] is None  # not written
    params = [
        r
        for r in rows(tables, "entry_parameters")
        if r["entry_locator"] == "data/m.yaml#/species/1" and r["block"] == "note"
    ]
    # `note:` with no value is a key present with null: its own marker, not an absent note
    assert len(params) == 1 and params[0]["value_kind"] == "null"
    assert species["NO"]["note"] is None
    composition = [
        r for r in rows(tables, "entry_parameters") if r["entry_locator"].endswith("/species/2")
    ]
    assert [(r["block"], r["value_kind"]) for r in composition] == [
        ("composition", "empty_mapping")
    ]


def test_phases_and_references(tmp_path: Path) -> None:
    tables, _ = run_reader(tmp_path, {"data/m.yaml": MECHANISM})
    gas, surf = rows(tables, "phases")
    assert gas["name"] == "gas" and gas["thermo"] == "ideal-gas" and gas["kinetics"] == "gas"
    assert surf["site_density"] is None and surf["site_density_text"] == "2.7e-09 mol/cm^2"
    assert surf["adjacent_phases"] == ["gas"]
    refs = [
        (
            r["reference"],
            r["form"],
            r["position"],
            r["name"],
            r["section"],
            r["selection"],
            r["names"],
        )
        for r in rows(tables, "phase_references")
    ]
    assert ("elements", "name", 1, "O", None, None, None) in refs
    assert ("species", "section", 0, None, "species", None, ["H2", "O2", "NO", "N"]) in refs
    assert ("species", "section", 1, None, "other.yaml/species", "all", None) in refs
    assert ("reactions", "string", None, None, None, "all", None) in refs
    assert ("species", "string", None, None, None, "surface-species", None) in refs
    state = [
        r
        for r in rows(tables, "entry_parameters")
        if r["entry_table"] == "phases" and r["block"] == "state"
    ]
    assert {r["path"]: r.get("value_number") or r["value_text"] for r in state} == {
        "/state/T": 300.0,
        "/state/P": "1 atm",
    }


def test_reactions_terms_and_parameters(tmp_path: Path) -> None:
    tables, _ = run_reader(tmp_path, {"data/m.yaml": MECHANISM})
    reactions = rows(tables, "reactions")
    assert [r["index"] for r in reactions] == [1, 2, 3]  # index 0 is the units entry
    first, second, third = reactions
    assert first["equation"] == "2 H + M <=> H2 + M" and first["arrow"] == "<=>"
    assert first["equation_parsed"] and first["type"] == "three-body"
    assert second["duplicate"] is True and first["duplicate"] is None
    assert third["arrow"] == "=>" and third["type"] is None
    terms = [
        (r["side"], r["position"], r["role"], r["coefficient"], r["species"])
        for r in rows(tables, "reaction_terms")
        if r["reaction_locator"] == first["_locator"]
    ]
    assert terms == [
        ("reactant", 0, "participant", 2.0, "H"),
        ("reactant", 1, "participant", None, "M"),
        ("product", 0, "participant", None, "H2"),
        ("product", 1, "participant", None, "M"),
    ]
    collider = [
        r
        for r in rows(tables, "reaction_terms")
        if r["reaction_locator"] == second["_locator"] and r["role"] == "falloff_collider"
    ]
    assert [(r["side"], r["species"]) for r in collider] == [("reactant", "M"), ("product", "M")]
    params = {
        r["path"]: r
        for r in rows(tables, "entry_parameters")
        if r["entry_locator"] == first["_locator"]
    }
    assert params["/rate-constant/A"]["value_number"] == 1.0e18
    assert params["/rate-constant/b"]["value_number"] == -1.0
    assert params["/rate-constant/Ea"]["value_number"] == 0.0
    assert params["/efficiencies/H2"]["value_number"] == 2.5
    assert params["/rate-constant/A"]["_locator"] == "data/m.yaml#/reactions/1/rate-constant/A"
    fall = {
        r["path"]: r
        for r in rows(tables, "entry_parameters")
        if r["entry_locator"] == second["_locator"]
    }
    # a rate coefficient written with its unit keeps the string; integer spellings stay integers
    assert fall["/low-P-rate-constant/Ea"]["value_text"] == "1.2 kcal/mol"
    assert fall["/low-P-rate-constant/b"]["value_kind"] == "integer"
    assert fall["/low-P-rate-constant/b"]["value_integer"] == 1
    assert fall["/Troe/T3"]["value_number"] == 1e-30


@pytest.mark.parametrize(
    ("equation", "arrow", "terms"),
    [
        ("H + O2 <=> OH + O", "<=>", [("reactant", "H"), ("reactant", "O2"), ("product", "OH")]),
        ("CH3 + CH3 (+M) = C2H6 (+M)", "=", None),
        ("NH4+ + e- => NH3 + H", "=>", None),
        ("0.5 O2 + H2 => H2O", "=>", None),
    ],
)
def test_equation_grammar(equation: str, arrow: str, terms: list | None) -> None:
    parsed = parse_equation(equation)
    assert parsed is not None and parsed[0] == arrow
    if terms is not None:
        assert [(side, species) for side, _, _, _, species in parsed[1]][:3] == terms
    if equation.startswith("NH4+"):
        assert [t[4] for t in parsed[1]] == ["NH4+", "e-", "NH3", "H"]
    if equation.startswith("0.5"):
        assert parsed[1][0][3] == 0.5


@pytest.mark.parametrize(
    "equation",
    ["H + O2", "H + + O2 => OH", "H + O2 <=> OH => O", "H + O2 => X Y", "H + O2 => 3", "=> H"],
)
def test_an_equation_outside_the_grammar_is_not_parsed(equation: str, tmp_path: Path) -> None:
    assert parse_equation(equation) is None
    tables, _ = run_reader(tmp_path, {"data/x.yaml": f"reactions:\n- equation: {equation}\n"})
    (row,) = rows(tables, "reactions")
    assert row["equation"] == equation and row["equation_parsed"] is False
    assert row["arrow"] is None and tables["reaction_terms"].num_rows == 0


def test_yaml_is_read_with_the_1_2_core_schema(tmp_path: Path) -> None:
    text = "species:\n- name: NO\n- name: N\n- name: on\n  sites: 1e-3\n  charge: 0123\n"
    tables, _ = run_reader(tmp_path, {"data/x.yaml": text})
    species = rows(tables, "species")
    assert [r["name"] for r in species] == ["NO", "N", "on"]
    assert species[2]["sites"] == 0.001 and species[2]["charge"] == 123.0
    document, duplicates = yaml12.load("a: {b: 2020-01-01, c: yes, d: [~, true]}\n")
    assert document == {"a": {"b": "2020-01-01", "c": "yes", "d": [None, True]}}
    assert duplicates == []


def test_a_repeated_key_keeps_the_last_value_and_records_the_first(tmp_path: Path) -> None:
    text = "phases:\n- name: p\n  species:\n  species: [a, b]\n  thermo: ideal-gas\n"
    tables, _ = run_reader(tmp_path, {"data/x.yaml": text})
    (duplicate,) = rows(tables, "yaml_duplicate_keys")
    assert duplicate["key"] == "species" and duplicate["line"] == 3
    assert duplicate["replaced_at_line"] == 4 and duplicate["discarded_value_json"] == "null"
    assert duplicate["_locator"] == "data/x.yaml#L3"
    refs = rows(tables, "phase_references")
    assert [(r["reference"], r["form"], r["name"]) for r in refs] == [
        ("species", "name", "a"),
        ("species", "name", "b"),
    ]


def test_unclassified_sections_and_other_entries_are_kept(tmp_path: Path) -> None:
    text = (
        "foo: [spam, eggs]\nbar: {spam: [1, 2]}\nextensions:\n- type: python\n  name: m\n"
        "electron-collisions:\n- {target: N2, kind: effective, energy-levels: [0.0, 1.5]}\n"
        "elements:\n- {symbol: C13, atomic-weight: 13.003}\n"
    )
    tables, _ = run_reader(tmp_path, {"data/x.yaml": text})
    others = rows(tables, "other_entries")
    assert [(r["section"], r["index"], r["shape"]) for r in others] == [
        ("foo", 0, "scalar"),
        ("foo", 1, "scalar"),
        ("extensions", 0, "mapping"),
    ]
    (collision,) = rows(tables, "collisions")
    assert collision["target"] == "N2" and collision["kind"] == "effective"
    (element,) = rows(tables, "elements")
    assert element["symbol"] == "C13" and element["atomic_weight"] == 13.003
    by_locator = {r["_locator"]: r for r in rows(tables, "entry_parameters")}
    assert by_locator["data/x.yaml#/bar/spam/1"]["value_integer"] == 2
    assert by_locator["data/x.yaml#/bar/spam/1"]["entry_table"] == "file_sections"
    assert by_locator["data/x.yaml#/foo/0"]["value_text"] == "spam"
    energy = by_locator["data/x.yaml#/electron-collisions/0/energy-levels/1"]
    assert energy["value_number"] == 1.5 and energy["block"] == "energy-levels"
    assert energy["path"] == "/energy-levels/1"


def test_text_files_are_kept_as_lines_and_hdf5_is_skipped(tmp_path: Path) -> None:
    tree = tmp_path / "tree"
    (tree / "test/data").mkdir(parents=True)
    (tree / "test/data/m.inp").write_text("ELEMENTS\nH O\nEND\n")
    (tree / "test/data/r.csv").write_bytes(b"1,2\n3,4")
    (tree / "test/data/s.h5").write_bytes(b"\x89HDF\r\n\x1a\n")
    out = tmp_path / "out"
    out.mkdir()
    names = ["test/data/m.inp", "test/data/r.csv", "test/data/s.h5"]
    writer = Writer(out, cantera.TABLES, names)
    cantera.read(tree, writer)
    result = writer.finish()
    accounts = {record.path: record for record in result.payload}
    assert accounts["test/data/m.inp"].status == "partly_read"
    assert accounts["test/data/s.h5"].status == "skipped" and accounts["test/data/s.h5"].reason
    lines = pq.read_table(out / "text_lines.parquet").to_pylist()
    assert [(r["_artifact"], r["line"], r["text"]) for r in lines] == [
        ("test/data/m.inp", 1, "ELEMENTS"),
        ("test/data/m.inp", 2, "H O"),
        ("test/data/m.inp", 3, "END"),
        ("test/data/r.csv", 1, "1,2"),
        ("test/data/r.csv", 2, "3,4"),
    ]
    assert lines[0]["_locator"] == "test/data/m.inp#L1"


@pytest.mark.parametrize(
    ("content", "message"),
    [
        ("species:\n- name: [unclosed\n", "x.yaml: cannot be read as YAML: line "),
        ("- a\n- b\n", "x.yaml: the top level must be a mapping of sections, found list"),
        ("", "x.yaml: the top level must be a mapping of sections, found NoneType"),
        ("a: 1\n\tb: 2\n", "x.yaml: cannot be read as YAML: line 2"),
    ],
)
def test_malformed_yaml_is_refused_with_its_location(
    tmp_path: Path, content: str, message: str
) -> None:
    with pytest.raises(StagingError) as error:
        run_reader(tmp_path, {"data/x.yaml": content})
    assert message.replace("x.yaml", "data/x.yaml") in str(error.value)


def test_a_payload_file_without_a_rule_is_refused(tmp_path: Path) -> None:
    with pytest.raises(StagingError, match="data/x.bin: the Cantera reader has no rule"):
        run_reader(tmp_path, {"data/x.bin": "?"})


def test_an_integer_beyond_64_bits_is_refused_with_its_locator(tmp_path: Path) -> None:
    with pytest.raises(StagingError, match=r"data/x\.yaml#/foo/0: integer .* does not fit"):
        run_reader(tmp_path, {"data/x.yaml": "foo: [123456789012345678901234567890]\n"})


# -- the real tree -----------------------------------------------------------------------------

real = pytest.mark.skipif(
    not TREE.is_dir(),
    reason=f"the acquired Cantera tree {TREE} is absent (run `tk acquire cantera`)",
)


@pytest.fixture(scope="module")
def staged(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """The real `tk read cantera`, into a temporary staged directory."""
    ctx = StageContext(staged_dir=tmp_path_factory.mktemp("staged"))
    manifests = load_sources(default_sources_dir())
    outcome = stage.read_source(ctx, manifests["cantera"], read_lock(default_lock_path()))
    assert outcome.status == "read"
    return stage.staged_path(ctx, "cantera", PIN)


@pytest.fixture(scope="module")
def manifest(staged: Path) -> staged_manifest.StagedManifest:
    return staged_manifest.read(staged)


def table(staged: Path, name: str) -> pa.Table:
    return pq.read_table(staged / f"{name}.parquet")


def yaml_files() -> list[Path]:
    return sorted(
        [*TREE.glob("data/*.yaml"), *TREE.glob("data/example_data/*.yaml")]
        + list(TREE.glob("test/data/**/*.yaml"))
    )


def relative(path: Path) -> str:
    return path.relative_to(TREE).as_posix()


def raw(path: Path) -> str:
    return path.read_text(encoding="utf-8")


@real
def test_every_table_is_staged_and_every_payload_file_accounted(
    manifest: staged_manifest.StagedManifest,
) -> None:
    assert set(manifest.tables) == set(cantera.TABLES)
    assert manifest.reader.name == "cantera" and manifest.reader.version == cantera.READER_VERSION
    status = collections.Counter(record.status for record in manifest.payload)
    assert status == {"read": 113, "partly_read": 136, "skipped": 8}
    assert len(manifest.payload) == 257
    for record in manifest.payload:
        if record.status != "read":
            assert record.reason
    assert {r.path for r in manifest.payload if r.status == "skipped"} == {
        relative(p) for p in TREE.glob("test/data/*.h5")
    }


@real
def test_reaction_and_phase_counts_against_line_patterns(staged: Path) -> None:
    expected: dict[str, int] = {}
    phases = 0
    for path in yaml_files():
        content = raw(path)
        expected[relative(path)] = len(
            re.findall(r"(?m)(?:^\s*(?:-\s+)?|[{,]\s*)equation:", content)
        )
        in_phases = False
        for line in content.split("\n"):
            if re.match(r"^phases:\s*$", line):
                in_phases = True
            elif in_phases and re.match(r"^[A-Za-z_]", line):
                in_phases = False
            elif in_phases and line.startswith("- name:"):
                phases += 1
    reactions = table(staged, "reactions")
    assert reactions.num_rows == sum(expected.values()) == 7393
    by_file = collections.Counter(reactions.column("_artifact").to_pylist())
    assert {k: by_file.get(k, 0) for k in expected} == expected
    assert table(staged, "phases").num_rows == phases == 262
    assert all(table(staged, "reactions").column("equation_parsed").to_pylist())
    # The survey counts 1378 (data/) and 5728 (example_data/) in sections named for reactions;
    # example_data/ also has 37 entries with an `equation` in a section named `collisions`
    # (type electron-collision-plasma), and test/data/ has 10 more in sections named `surface`
    # and `*-rxns`.
    groups: collections.Counter = collections.Counter()
    for artifact, count in by_file.items():
        if artifact.startswith("data/example_data/"):
            groups["example_data"] += count
        elif artifact.startswith("test/"):
            groups["test"] += count
        else:
            groups["data"] += count
    assert groups == {"data": 1378, "example_data": 5728 + 37, "test": 250}
    sections = collections.Counter(
        r["section"]
        for r in reactions.to_pylist()
        if r["_artifact"].startswith(("data/example_data/", "test/"))
        and "reaction" not in r["section"]
    )
    assert sections == {
        "collisions": 37,
        "surface": 8,
        "chebyshev-deprecated-rxns": 1,
        "plog-invalid-rxns": 1,
    }


@real
def test_species_thermo_counts_against_line_patterns(staged: Path) -> None:
    models: collections.Counter = collections.Counter()
    pieces = symbols = compositions = transports = 0
    for path in yaml_files():
        content = raw(path)
        models.update(
            re.findall(r"\bmodel:\s*(NASA7|NASA9|Shomate|constant-cp|piecewise-Gibbs)\b", content)
        )
        for ranges in re.findall(r"temperature-ranges:\s*\[([^\]]*)\]", content):
            pieces += len([x for x in ranges.split(",") if x.strip()]) - 1
        symbols += len(re.findall(r"(?m)^\s*-?\s*\{?symbol:", content))
        compositions += len(re.findall(r"(?m)^\s*composition:", content))
        transports += len(re.findall(r"(?m)^\s*transport:\s*$", content))
    thermo = table(staged, "species_thermo")
    assert collections.Counter(thermo.column("model").to_pylist()) == models
    assert models["NASA7"] == 2990 and models["NASA9"] == 26 and models["Shomate"] == 24
    assert table(staged, "thermo_pieces").num_rows == pieces == 5795
    assert table(staged, "elements").num_rows == symbols == 90
    assert table(staged, "species_transport").num_rows == transports == 241
    # 3142 composition lines: 3140 species and 2 phases; ten species write an empty mapping
    with_atoms = len(
        set(table(staged, "species_composition").column("species_locator").to_pylist())
    )
    assert compositions == 3142 and with_atoms == 3130


@real
def test_files_sections_and_units_against_the_text(staged: Path) -> None:
    files = table(staged, "yaml_files")
    assert files.num_rows == len(yaml_files()) == 113
    with_units = sum(1 for p in yaml_files() if re.search(r"(?m)^units:", raw(p)))
    units = table(staged, "units_entries").to_pylist()
    assert len({r["_artifact"] for r in units if r["scope"] == "file"}) == with_units == 57
    generators = collections.Counter(
        re.search(r"(?m)^generator:\s*(.+?)\s*$", raw(p)).group(1)  # type: ignore[union-attr]
        for p in yaml_files()
        if re.search(r"(?m)^generator:", raw(p))
    )
    assert collections.Counter(g for g in files.column("generator").to_pylist() if g) == generators
    duplicate = table(staged, "yaml_duplicate_keys").to_pylist()
    assert [(r["_artifact"], r["key"]) for r in duplicate] == [
        ("test/data/debye-huckel-all.yaml", "species")
    ]
    sections = table(staged, "file_sections").to_pylist()
    top_level = sum(len(re.findall(r"(?m)^[^\s#-][^:\n]*:", raw(p))) for p in yaml_files())
    assert len(sections) == top_level


@real
def test_species_are_kept_per_occurrence_with_their_file_and_position(staged: Path) -> None:
    species = table(staged, "species").to_pylist()
    names = collections.defaultdict(set)
    for row in species:
        names[row["name"]].add(row["_artifact"])
    assert {"NO", "N"} <= set(names)  # unquoted NO and N are names, not booleans
    assert len(names["NO"]) > 5  # one row per file occurrence, not deduplicated
    assert len({r["_locator"] for r in species}) == len(species)
    gri = [r for r in species if r["_artifact"] == "data/gri30.yaml"]
    assert [r["index"] for r in gri] == list(range(len(gri)))


def written_terms(equation: str) -> list[tuple[str, str, float | None, str]]:
    """The terms of an equation by an independent method: regular expressions over the text."""
    left, arrow, right = re.split(r"\s(<=>|=>|=)\s", equation)
    out: list[tuple[str, str, float | None, str]] = []
    for side, text in (("reactant", left), ("product", right)):
        collider = re.search(r"\s\(\+\s*(\S+?)\)$", text)
        if collider:
            text = text[: collider.start()]
        for term in text.split(" + "):
            parts = term.split()
            if len(parts) == 2:
                out.append((side, "participant", float(parts[0]), parts[1]))
            else:
                out.append((side, "participant", None, parts[0]))
        if collider:
            out.append((side, "falloff_collider", None, collider.group(1)))
    return out


def pointer_get(document: object, pointer: str) -> object:
    for token in pointer.split("/")[1:]:
        token = token.replace("~1", "/").replace("~0", "~")
        document = document[int(token)] if isinstance(document, list) else document[token]  # type: ignore[index]
    return document


@real
@pytest.mark.parametrize(
    "artifact",
    [
        "data/gri30.yaml",
        "data/airNASA9.yaml",
        "data/nasa_condensed.yaml",
        "data/example_data/co2-thermo.yaml",
        "test/data/thermo-models.yaml",
        "test/data/nasa9-test.yaml",
    ],
)
def test_thermo_round_trips_to_the_parsed_yaml(staged: Path, artifact: str) -> None:
    document, _ = yaml12.load(raw(TREE / artifact))
    thermo = {
        r["_locator"]: r
        for r in table(staged, "species_thermo").to_pylist()
        if r["_artifact"] == artifact
    }
    pieces = collections.defaultdict(list)
    for piece in table(staged, "thermo_pieces").to_pylist():
        if piece["_artifact"] == artifact:
            pieces[piece["thermo_locator"]].append(piece)
    checked = 0
    for locator, row in thermo.items():
        source = pointer_get(document, locator.split("#")[1])
        assert isinstance(source, dict)
        assert row["model"] == source["model"]
        if "temperature-ranges" in source:
            assert row["temperature_ranges"] == [float(x) for x in source["temperature-ranges"]]
        if isinstance(source.get("data"), list):
            mine = sorted(pieces[locator], key=lambda p: p["piece_index"])
            assert [p["coefficients"] for p in mine] == [
                [float(x) for x in rowdata] for rowdata in source["data"]
            ]
            ranges = source["temperature-ranges"]
            assert [(p["temperature_low"], p["temperature_high"]) for p in mine] == [
                (float(ranges[i]), float(ranges[i + 1])) for i in range(len(mine))
            ]
        if "note" in source and isinstance(source["note"], str):
            assert row["note"] == source["note"]
        checked += 1
    assert checked > 0


@real
def test_coefficients_equal_the_numbers_written_in_the_file(staged: Path) -> None:
    content = raw(TREE / "data/gri30.yaml")
    block = re.search(r"- name: CH4\n(?:.*\n)*?  thermo:\n((?:    .*\n)+)", content)
    assert block is not None
    written = [float(x) for x in re.findall(r"[-+]?\d+\.\d+(?:[eE][-+]?\d+)?", block.group(1))]
    rows = [
        r
        for r in table(staged, "species").to_pylist()
        if r["_artifact"] == "data/gri30.yaml" and r["name"] == "CH4"
    ]
    (species,) = rows
    thermo_locator = species["_locator"] + "/thermo"
    pieces = sorted(
        (
            p
            for p in table(staged, "thermo_pieces").to_pylist()
            if p["thermo_locator"] == thermo_locator
        ),
        key=lambda p: p["piece_index"],
    )
    staged_numbers = [x for p in pieces for x in p["coefficients"]]
    assert staged_numbers == written[-len(staged_numbers) :] and len(staged_numbers) == 14


def rebuild(parameters: list[dict]) -> dict:
    """The keys the reader kept as leaf rows, rebuilt into the nested value they came from."""
    kinds = {
        "string": "value_text",
        "integer": "value_integer",
        "float": "value_number",
        "boolean": "value_bool",
    }
    empty = {"null": None, "empty_list": [], "empty_mapping": {}}
    tree: dict = {}
    for param in parameters:
        kind = param["value_kind"]
        value = param[kinds[kind]] if kind in kinds else empty[kind]
        tokens = [t.replace("~1", "/").replace("~0", "~") for t in param["path"].split("/")[1:]]
        node = tree
        for token in tokens[:-1]:
            node = node.setdefault(token, {})
        node[tokens[-1]] = value

    def listify(value: object) -> object:
        if isinstance(value, dict) and value:
            if all(key.isdigit() for key in value):
                ordered = sorted(value, key=int)
                if ordered == [str(i) for i in range(len(value))]:
                    return [listify(value[key]) for key in ordered]
            return {key: listify(child) for key, child in value.items()}
        return value

    return listify(tree)  # type: ignore[return-value]


@real
@pytest.mark.parametrize(
    "artifact",
    [
        "data/gri30.yaml",
        "data/h2o2.yaml",
        "data/ptcombust.yaml",
        "data/example_data/methane-plasma-pavan-2023.yaml",
        "test/data/pdep-test.yaml",
        "test/data/linearBurke-test.yaml",
    ],
)
def test_reactions_round_trip_to_the_parsed_yaml(staged: Path, artifact: str) -> None:
    document, _ = yaml12.load(raw(TREE / artifact))
    reactions = [r for r in table(staged, "reactions").to_pylist() if r["_artifact"] == artifact]
    terms = collections.defaultdict(list)
    for term in table(staged, "reaction_terms").to_pylist():
        if term["_artifact"] == artifact:
            terms[term["reaction_locator"]].append(term)
    params = collections.defaultdict(list)
    for param in table(staged, "entry_parameters").to_pylist():
        if param["_artifact"] == artifact and param["entry_table"] == "reactions":
            params[param["entry_locator"]].append(param)
    assert reactions
    for row in reactions:
        source = pointer_get(document, row["_locator"].split("#")[1])
        assert isinstance(source, dict)
        assert row["equation"] == source["equation"]
        assert row["type"] == source.get("type")
        assert row["duplicate"] == source.get("duplicate")
        mine = sorted(
            terms[row["_locator"]], key=lambda t: (t["side"] != "reactant", t["position"])
        )
        staged_terms = [(t["side"], t["role"], t["coefficient"], t["species"]) for t in mine]
        assert staged_terms == written_terms(source["equation"])
        rebuilt = rebuild(params[row["_locator"]])
        expected = {
            k: v
            for k, v in source.items()
            if k not in {"equation", "type", "duplicate", "note", "id", "units"}
        }
        assert rebuilt == expected


@real
def test_the_staged_source_loads_into_a_database(staged: Path, test_database: TestDatabase) -> None:
    outcome = load.load_staged(test_database.url, staged, "src_cantera")
    manifest = staged_manifest.read(staged)
    assert outcome.tables == {name: record.rows for name, record in manifest.tables.items()}
    from thermo_knowledge import db

    with db.connect(test_database.url) as conn:
        (count,) = conn.execute('SELECT count(*) FROM src_cantera."reactions"').fetchone()
        assert count == 7393
        row = conn.execute(
            'SELECT "coefficients" FROM src_cantera.thermo_pieces '
            "WHERE _locator = 'data/gri30.yaml#/species/0/thermo/data/0'"
        ).fetchone()
        assert row is not None and len(row[0]) == 7
        (names,) = conn.execute(
            "SELECT count(*) FROM src_cantera.species WHERE name IN ('NO', 'N') "
            "AND _artifact = 'data/gri30.yaml'"
        ).fetchone()
        assert names == 2
