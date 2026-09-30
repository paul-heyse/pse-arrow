# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The CoolProp reader against the real acquired tree.

Counts are computed here, directly from the JSON files, independently of the reader.
"""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq
import pytest

from thermo_knowledge import config
from thermo_knowledge.acquire import store
from thermo_knowledge.acquire.lock import read_lock
from thermo_knowledge.acquire.manifest import default_lock_path, default_sources_dir, load_sources
from thermo_knowledge.readers import coolprop
from thermo_knowledge.staging import load, stage
from thermo_knowledge.staging import manifest as staged_manifest
from thermo_knowledge.staging.stage import StageContext
from thermo_knowledge.testing import TestDatabase

PIN = "ae81610e7d23"
TREE = store.pin_dir(config.raw_dir(), "coolprop", PIN) / store.TREE_DIR_NAME

pytestmark = pytest.mark.skipif(
    not TREE.is_dir(),
    reason=f"the acquired CoolProp tree {TREE} is absent (run `tk acquire coolprop`)",
)


@pytest.fixture(scope="module")
def staged(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """The real `tk read coolprop`, into a temporary staged directory."""
    ctx = StageContext(staged_dir=tmp_path_factory.mktemp("staged"))
    manifests = load_sources(default_sources_dir())
    outcome = stage.read_source(ctx, manifests["coolprop"], read_lock(default_lock_path()))
    assert outcome.status == "read"
    return stage.staged_path(ctx, "coolprop", PIN)


@pytest.fixture(scope="module")
def manifest(staged: Path) -> staged_manifest.StagedManifest:
    return staged_manifest.read(staged)


def table(staged: Path, name: str) -> pa.Table:
    return pq.read_table(staged / f"{name}.parquet")


def fluid_documents() -> list[tuple[str, dict]]:
    return [
        (path.name, json.loads(path.read_text()))
        for path in sorted((TREE / "dev" / "fluids").glob("*.json"))
    ]


def json_array(relative: str) -> list:
    return json.loads((TREE / relative).read_text())


def coefficient_rows(term: dict) -> int:
    return max((len(v) for v in term.values() if isinstance(v, list)), default=0)


def type_rows(terms: list[dict]) -> Counter:
    counts: Counter = Counter()
    for term in terms:
        counts[term["type"]] += coefficient_rows(term)
    return counts


# -- the reader is what the manifest says ---------------------------------------------------------


def test_every_table_is_declared_and_staged(manifest: staged_manifest.StagedManifest) -> None:
    assert set(manifest.tables) == set(coolprop.TABLES)
    assert manifest.reader.name == "coolprop"
    assert manifest.reader.version == coolprop.READER_VERSION
    assert manifest.pin == PIN
    assert manifest.tables["fluids"].rows > 0


def test_every_payload_file_is_accounted_for(manifest: staged_manifest.StagedManifest) -> None:
    expected: set[str] = set()
    for pattern in ("dev/fluids/*.json", "dev/cubics/*.json", "dev/pcsaft/*.json"):
        expected |= {p.relative_to(TREE).as_posix() for p in TREE.glob(pattern)}
    for directory in ("dev/mixtures", "dev/incompressible_liquids"):
        expected |= {
            p.relative_to(TREE).as_posix() for p in (TREE / directory).rglob("*") if p.is_file()
        }
    recorded = {record.path: record for record in manifest.payload}
    assert set(recorded) == expected
    assert len(recorded) == 615
    for record in recorded.values():
        assert record.status in ("read", "partly_read", "skipped")
        if record.status != "read":
            assert record.reason


def test_skipped_files_are_code_or_binary_with_reasons(
    manifest: staged_manifest.StagedManifest,
) -> None:
    skipped = {r.path: r for r in manifest.payload if r.status == "skipped"}
    assert all(path.endswith((".py", ".pas", ".xlsx", ".ipynb", ".gitignore")) for path in skipped)
    for path, record in skipped.items():
        if (
            "/CPIncomp/" in path
            and path.endswith(".py")
            and not path.endswith(
                ("BaseObjects.py", "DataObjects.py", "WriterObjects.py", "__init__.py")
            )
        ):
            assert "out of proportion" in (record.reason or "")
    assert not any(r.status == "skipped" for r in manifest.payload if r.path.endswith(".json"))
    assert not any(
        r.status == "skipped" for r in manifest.payload if r.path.endswith((".txt", ".csv"))
    )


def test_every_column_is_documented(staged: Path) -> None:
    for name in coolprop.TABLES:
        for field in pq.read_schema(staged / f"{name}.parquet"):
            metadata = {k.decode(): v.decode() for k, v in (field.metadata or {}).items()}
            assert metadata.get("source_name"), (name, field.name)
            assert metadata.get("unit"), (name, field.name)
    water = pq.read_schema(staged / "states.parquet").field("hmolar").metadata
    assert water[b"unit"] == b"J/mol"


# -- row counts against independent counts ----------------------------------------------------------


def test_fluid_and_eos_counts(staged: Path) -> None:
    documents = fluid_documents()
    assert len(documents) == 136
    assert table(staged, "fluids").num_rows == 136
    eos = [entry for _, doc in documents for entry in doc["EOS"]]
    assert table(staged, "eos_entries").num_rows == len(eos) == 159
    fluids = table(staged, "fluids")
    assert sorted(fluids.column("NAME").to_pylist()) == sorted(
        d["INFO"]["NAME"] for _, d in documents
    )
    water = fluids.filter(pa.compute.equal(fluids["NAME"], "Water")).to_pylist()[0]
    info = dict(next(d for n, d in documents if n == "Water.json")["INFO"])
    assert water["CAS"] == info["CAS"] and water["ALIASES"] == info["ALIASES"]
    assert water["2DPNG_URL"] == info["2DPNG_URL"]


def test_term_counts_by_type(staged: Path) -> None:
    eos = [entry for _, doc in fluid_documents() for entry in doc["EOS"]]
    for key, terms_table, rows_table in (
        ("alphar", "alphar_terms", "alphar_term_rows"),
        ("alpha0", "alpha0_terms", "alpha0_term_rows"),
    ):
        terms = [term for entry in eos for term in entry[key]]
        staged_terms = table(staged, terms_table)
        staged_rows = table(staged, rows_table)
        assert staged_terms.num_rows == len(terms)
        assert Counter(staged_terms.column("type").to_pylist()) == Counter(t["type"] for t in terms)
        assert staged_rows.num_rows == sum(coefficient_rows(t) for t in terms)
        assert Counter(
            {t: c for t, c in Counter(staged_rows.column("type").to_pylist()).items()}
        ) == +type_rows(terms)


def test_states_ancillaries_and_splines(staged: Path) -> None:
    documents = fluid_documents()
    eos = [entry for _, doc in documents for entry in doc["EOS"]]
    states = sum(len(doc["STATES"]) for _, doc in documents) + sum(len(e["STATES"]) for e in eos)
    assert table(staged, "states").num_rows == states
    anc = [doc["ANCILLARIES"] for _, doc in documents]
    equations = [
        (name, a[name])
        for a in anc
        for name in ("pS", "pL", "pV", "rhoL", "rhoV", "hL", "hLV", "sL", "sLV")
        if name in a
    ]
    assert table(staged, "ancillary_equations").num_rows == len(equations)
    assert Counter(table(staged, "ancillary_equations").column("ancillary").to_pylist()) == Counter(
        name for name, _ in equations
    )
    assert table(staged, "ancillary_equation_rows").num_rows == sum(
        coefficient_rows(eq) for _, eq in equations
    )
    melting = [a["melting_line"] for a in anc if "melting_line" in a]
    assert table(staged, "melting_lines").num_rows == len(melting)
    parts = [p for m in melting for p in m["parts"]]
    assert table(staged, "melting_line_parts").num_rows == len(parts)
    assert table(staged, "melting_line_part_rows").num_rows == sum(
        coefficient_rows(p) for p in parts
    )
    surface = [a["surface_tension"] for a in anc if "surface_tension" in a]
    assert table(staged, "surface_tension").num_rows == len(surface)
    assert table(staged, "surface_tension_rows").num_rows == sum(
        coefficient_rows(s) for s in surface
    )
    splines = [e["critical_region_splines"] for e in eos if "critical_region_splines" in e]
    assert table(staged, "critical_region_splines").num_rows == len(splines)
    assert table(staged, "critical_region_spline_rows").num_rows == sum(
        coefficient_rows(s) for s in splines
    )


def test_superancillary_counts(staged: Path) -> None:
    supers = [
        e["SUPERANCILLARY"] for _, d in fluid_documents() for e in d["EOS"] if "SUPERANCILLARY" in e
    ]
    assert table(staged, "superancillaries").num_rows == len(supers)
    assert table(staged, "superancillary_check_points").num_rows == sum(
        len(s.get("check_points", [])) for s in supers
    )
    expansions = sum(len(s[k]) for s in supers for k in s if k.startswith("jexpansions_"))
    assert table(staged, "superancillary_expansions").num_rows == expansions
    first = table(staged, "superancillary_expansions").slice(0, 1).to_pylist()[0]
    assert isinstance(first["coef"], list) and len(first["coef"]) > 1


def count_objects(value: object) -> int:
    """Objects in a JSON value (the value itself when it is one), by an independent walk."""
    if isinstance(value, dict):
        return 1 + sum(count_objects(v) for v in value.values())
    if isinstance(value, list):
        return sum(count_objects(v) for v in value)
    return 0


def test_transport_counts_and_verbatim_json(staged: Path) -> None:
    blocks = [
        (doc["INFO"]["NAME"], doc["TRANSPORT"])
        for _, doc in fluid_documents()
        if "TRANSPORT" in doc
    ]
    assert len(blocks) == 66
    objects = table(staged, "transport_objects")
    assert objects.num_rows == sum(count_objects(block) for _, block in blocks)
    roots = objects.filter(pa.compute.equal(objects["object_path"], "")).to_pylist()
    assert sorted(r["fluid"] for r in roots) == sorted(name for name, _ in blocks)
    by_fluid = dict(blocks)
    for root in roots:
        assert json.loads(root["json"]) == by_fluid[root["fluid"]]
    # sub-objects are verbatim too
    nested = objects.filter(pa.compute.equal(objects["object_path"], "/viscosity")).to_pylist()
    assert len(nested) >= 50
    for row in nested:
        assert json.loads(row["json"]) == by_fluid[row["fluid"]]["viscosity"]

    def arrays(value: object) -> tuple[int, int]:
        """(array-valued fields, scalar fields that are not tags) in a JSON value."""
        lists = scalars = 0
        if isinstance(value, dict):
            for key, child in value.items():
                if isinstance(child, dict):
                    a, s = arrays(child)
                elif isinstance(child, list):
                    if child and all(isinstance(x, dict) for x in child):
                        a = s = 0
                        for x in child:
                            xa, xs = arrays(x)
                            a, s = a + xa, s + xs
                    else:
                        a, s = 1, 0
                elif key in ("type", "BibTeX", "reference_fluid", "hardcoded", "_note", "note"):
                    a, s = 0, 0
                else:
                    a, s = 0, 1
                lists, scalars = lists + a, scalars + s
        return lists, scalars

    assert table(staged, "transport_scalars").num_rows == sum(arrays(b)[1] for _, b in blocks)
    rows = table(staged, "transport_rows")

    def row_total(value: object) -> int:
        if isinstance(value, dict):
            own = max(
                (
                    len(v)
                    for v in value.values()
                    if isinstance(v, list) and not any(isinstance(x, dict) for x in v)
                ),
                default=0,
            )
            return own + sum(row_total(v) for v in value.values())
        if isinstance(value, list):
            return sum(row_total(v) for v in value if isinstance(v, dict))
        return 0

    assert rows.num_rows == sum(row_total(b) for _, b in blocks)


def test_mixture_counts(staged: Path) -> None:
    pairs = json_array("dev/mixtures/mixture_binary_pairs.json")
    old = json_array("dev/mixtures/old_BIP.json")
    staged_pairs = table(staged, "binary_pairs")
    assert staged_pairs.num_rows == len(pairs) + len(old) == 906
    main = staged_pairs.filter(
        pa.compute.equal(staged_pairs["_artifact"], "dev/mixtures/mixture_binary_pairs.json")
    ).to_pylist()
    # pair order is kept as written, not canonicalised
    assert [(r["CAS1"], r["CAS2"]) for r in main] == [(p["CAS1"], p["CAS2"]) for p in pairs]
    assert [r["betaT"] for r in main] == [
        float(p["betaT"]) if "betaT" in p else None for p in pairs
    ]
    assert [r["function"] for r in main] == [p.get("function") for p in pairs]
    assert sum(r["xi"] is not None for r in main) == 6

    departures = json_array("dev/mixtures/mixture_departure_functions.json")
    assert table(staged, "departure_functions").num_rows == len(departures) == 28
    assert table(staged, "departure_function_rows").num_rows == sum(
        coefficient_rows(d) for d in departures
    )
    assert Counter(table(staged, "departure_functions").column("type").to_pylist()) == Counter(
        d["type"] for d in departures
    )
    predefined = json_array("dev/mixtures/predefined_mixtures.json")
    assert table(staged, "predefined_mixtures").num_rows == len(predefined) == 154
    assert table(staged, "predefined_mixture_components").num_rows == sum(
        len(p["fluids"]) for p in predefined
    )


def test_cubic_pcsaft_and_incompressible_counts(staged: Path) -> None:
    cubic = json_array("dev/cubics/all_cubic_fluids.json")
    assert table(staged, "cubic_fluids").num_rows == len(cubic)
    terms = [t for c in cubic for t in c["alpha0"]]
    assert table(staged, "cubic_alpha0_terms").num_rows == len(terms)
    assert table(staged, "cubic_alpha0_term_rows").num_rows == sum(
        coefficient_rows(t) for t in terms
    )
    assert table(staged, "pcsaft_fluids").num_rows == len(
        json_array("dev/pcsaft/all_pcsaft_fluids.json")
    )
    assert table(staged, "pcsaft_binary_pairs").num_rows == len(
        json_array("dev/pcsaft/mixture_binary_pairs_pcsaft.json")
    )
    records = [
        json.loads(p.read_text())
        for p in sorted((TREE / "dev/incompressible_liquids/json").glob("*.json"))
    ]
    assert table(staged, "incompressible_fluids").num_rows == len(records) == 126
    functions = [
        (r["name"], key, value)
        for r in records
        for key, value in r.items()
        if isinstance(value, dict) and "coeffs" in value
    ]
    assert table(staged, "incompressible_functions").num_rows == len(functions)

    def cells(coeffs: object) -> int:
        if isinstance(coeffs, str):
            return 0
        return sum(len(x) if isinstance(x, list) else 1 for x in coeffs)

    assert table(staged, "incompressible_coefficients").num_rows == sum(
        cells(v["coeffs"]) for _, _, v in functions
    )
    kinds = Counter(table(staged, "incompressible_functions").column("coeffs_kind").to_pylist())
    assert kinds["string"] == sum(isinstance(v["coeffs"], str) for _, _, v in functions)


def test_text_lines_are_verbatim(staged: Path) -> None:
    lines = table(staged, "text_lines")
    files = table(staged, "text_files")
    tsv = "dev/mixtures/ASHRAE_predefined_2026.tsv"
    raw = (TREE / tsv).read_bytes().decode().split("\n")
    if raw[-1] == "":
        raw.pop()
    mine = lines.filter(pa.compute.equal(lines["_artifact"], tsv)).sort_by("line")
    assert mine.column("text").to_pylist() == raw
    info = files.filter(pa.compute.equal(files["_artifact"], tsv)).to_pylist()[0]
    assert info["line_count"] == len(raw) and info["encoding"] == "utf-8"
    assert not info["ends_with_newline"]  # the source's last line has no newline
    total = 0
    for path in TREE.joinpath("dev/incompressible_liquids/CPIncomp/data").rglob("*"):
        if path.is_file() and path.suffix in (".txt", ".csv"):
            data = path.read_bytes()
            count = data.count(b"\n") + (0 if data.endswith(b"\n") or not data else 1)
            total += count
    grids = sum(
        1
        for a in lines.column("_artifact").to_pylist()
        if a.startswith("dev/incompressible_liquids/CPIncomp/data/")
    )
    assert grids == total
    latin = files.filter(pa.compute.equal(files["encoding"], "latin-1")).num_rows
    assert latin == 23  # the SecCool CSV files that are not valid UTF-8


# -- round trip -----------------------------------------------------------------------------------


def rebuild(
    staged: Path, terms_table: str, rows_table: str, fluid: str, eos_index: int
) -> list[dict]:
    terms = table(staged, terms_table).to_pylist()
    rows = table(staged, rows_table).to_pylist()
    keys = ("_artifact", "_locator", "fluid", "eos_index", "term_index", "type", "array_lengths")
    rebuilt = []
    mine = sorted(
        (t for t in terms if t["fluid"] == fluid and t["eos_index"] == eos_index),
        key=lambda t: t["term_index"],
    )
    for term in mine:
        entry: dict = {"type": term["type"]}
        entry.update({k: v for k, v in term.items() if k not in keys and v is not None})
        mine_rows = sorted(
            (
                r
                for r in rows
                if r["fluid"] == fluid
                and r["eos_index"] == eos_index
                and r["term_index"] == term["term_index"]
            ),
            key=lambda r: r["row_index"],
        )
        for name, length in json.loads(term["array_lengths"]).items():
            entry[name] = [mine_rows[i][name] for i in range(length)]
        rebuilt.append(entry)
    return rebuilt


@pytest.mark.parametrize("name", ["Water", "CarbonDioxide", "R134a"])
def test_alphar_and_alpha0_round_trip(staged: Path, name: str) -> None:
    document = json.loads((TREE / "dev" / "fluids" / f"{name}.json").read_text())
    fluid = document["INFO"]["NAME"]
    for eos_index, entry in enumerate(document["EOS"]):
        assert (
            rebuild(staged, "alphar_terms", "alphar_term_rows", fluid, eos_index) == entry["alphar"]
        )
        assert (
            rebuild(staged, "alpha0_terms", "alpha0_term_rows", fluid, eos_index) == entry["alpha0"]
        )


def test_round_trip_keeps_array_lengths_and_empty_lists(staged: Path) -> None:
    functions = {f["Name"]: f for f in json_array("dev/mixtures/mixture_departure_functions.json")}
    assert table(staged, "departure_function_rows").num_rows == 150
    staged_functions = {r["Name"]: r for r in table(staged, "departure_functions").to_pylist()}
    for name, function in functions.items():
        lengths = {k: len(v) for k, v in function.items() if isinstance(v, list) and k != "aliases"}
        assert json.loads(staged_functions[name]["array_lengths"]) == lengths
        assert staged_functions[name]["aliases"] == function["aliases"]  # [] stays [], not null
    assert any(f["aliases"] == [] for f in functions.values())
    fluids = {r["NAME"]: r["ALIASES"] for r in table(staged, "fluids").to_pylist()}
    documents = {d["INFO"]["NAME"]: d["INFO"]["ALIASES"] for _, d in fluid_documents()}
    assert fluids == documents
    assert any(aliases == [] for aliases in fluids.values())


# -- loading ---------------------------------------------------------------------------------------


def test_the_staged_source_loads_into_a_database(staged: Path, test_database: TestDatabase) -> None:
    outcome = load.load_staged(test_database.url, staged, "src_coolprop")
    manifest = staged_manifest.read(staged)
    assert outcome.tables == {name: record.rows for name, record in manifest.tables.items()}
    from thermo_knowledge import db

    with db.connect(test_database.url) as conn:
        for name in ("fluids", "alphar_term_rows", "superancillary_expansions", "text_lines"):
            (count,) = conn.execute(f'SELECT count(*) FROM src_coolprop."{name}"').fetchone()
            assert count == manifest.tables[name].rows
        # source vocabulary survives as column names, array-valued columns load as arrays
        (first,) = conn.execute(
            'SELECT "coef" FROM src_coolprop.superancillary_expansions ORDER BY _locator LIMIT 1'
        ).fetchone()
        assert isinstance(first, list) and all(isinstance(x, float) for x in first)
        (aliases,) = conn.execute(
            'SELECT "ALIASES" FROM src_coolprop.fluids WHERE "NAME" = \'Water\''
        ).fetchone()
        assert "H2O" in aliases
        row = conn.execute(
            'SELECT "A", "a" FROM src_coolprop.alphar_term_rows WHERE "A" IS NOT NULL LIMIT 1'
        ).fetchone()
        assert row is not None and row[0] is not None
