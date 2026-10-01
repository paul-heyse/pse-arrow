# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The NASA CEA reader: synthetic records in the fixed-column formats (schemas, locators,
missing markers, refusals) and the real acquired tree (row counts against independent
line-pattern counts, round trips to the raw text)."""

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
from thermo_knowledge.readers import nasa_cea
from thermo_knowledge.readers.nasa_cea.fortran import FieldError, read_real
from thermo_knowledge.staging import load, stage
from thermo_knowledge.staging import manifest as staged_manifest
from thermo_knowledge.staging.errors import StagingError
from thermo_knowledge.staging.schema import check_declared
from thermo_knowledge.staging.stage import StageContext
from thermo_knowledge.staging.writer import Writer
from thermo_knowledge.testing import TestDatabase

PIN = "4c5c612efa20"
TREE = store.pin_dir(config.raw_dir(), "nasa_cea", PIN) / store.TREE_DIR_NAME

# -- fixed-column line builders (the layouts of the format, written independently) ---------------

EXPONENTS = (-2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 0.0)


def species_1(name: str, notes: str) -> str:
    return f"{name:<15}{notes}".rstrip()


def species_2(
    count: int, date: str, pairs: list[tuple[str, str]], flag: int, weight: str, hf: str
) -> str:
    """`i2, 1x, a6, 1x, 5(a2, f6.2), i2, f13.5, f15.3`; `pairs` hold the two field texts."""
    slots = [f"{symbol:<2}{value:>6}" for symbol, value in pairs]
    slots += ["  " + "  0.00"] * (5 - len(slots))
    return f"{count:2d} {date:<6} " + "".join(slots) + f"{flag:2d}{weight:>13}{hf:>15}"


def interval(low: float, high: float, increment: float, count: int = 7) -> str:
    exponents = "".join(f"{e:5.1f}" for e in EXPONENTS)
    return f"{low:11.3f}{high:11.3f}{count:1d}{exponents}  {increment:15.3f}"


def number(value: float) -> str:
    return f"{value:16.9E}".replace("E", "D")


def coefficients(values: list[float]) -> tuple[str, str]:
    first = "".join(number(v) for v in values[:5])
    second = (
        number(values[5]) + number(values[6]) + " " * 16 + number(values[7]) + number(values[8])
    )
    return first, second


C_H2 = [1.0, -2.0, 3.5, 0.0, 5.0e-05, -6.0e-09, 7.0e-13, -8.0e03, 9.75]
C_OH = [-1.5e04, 2.0e02, 4.0, 1.0e-03, -1.0e-07, 2.0e-11, -3.0e-15, 5.0e03, -2.5]

GRID = "    200.00   1000.00   6000.00  20000.   9/8/2021"


def thermo_text(records: list[list[str]], *, extra_end: bool = True) -> str:
    """A thermo.inp with CRLF line endings; `records` are lists of lines."""
    lines = ["!", "!  A synthetic header", "thermo", GRID]
    for record in records:
        lines.extend(record)
    if extra_end:
        lines.append("END REACTANTS")
    return "\r\n".join(lines) + "\r\n"


def fitted(
    name: str, notes: str, pairs: list[tuple[str, str]], values: list[list[float]]
) -> list[str]:
    out = [
        species_1(name, notes),
        species_2(len(values), "g 6/97", pairs, 0, "2.01588", "0.000"),
    ]
    low = 300.0
    for index, row in enumerate(values):
        high = (1000.0, 6000.0, 20000.0)[index]
        out.append(interval(low, high, 8467.0))
        out.extend(coefficients(row))
        low = high
    return out


PRODUCTS = [
    fitted("H2", "Hydrogen. Gurvich,1978 pt1 p103.", [("H", "2.00")], [C_H2, C_OH]),
    fitted("OH", "Hydroxyl.", [("O", "1.00"), ("H", "1.00"), ("E", "0.00")], [C_OH]),
    ["END PRODUCTS"],
]
REACTANTS = [
    [
        species_1("Fuel", "Assigned enthalpy only."),
        species_2(0, "g 5/23", [("C", "4.00"), ("H", " 10.00")], 1, "", "-251140.000"),
        interval(298.15, 0.0, 0.0, count=0).replace("7", "0", 0),
    ],
]


def run_reader(tmp_path: Path, files: dict[str, str]) -> tuple[dict[str, pa.Table], Writer]:
    tree = tmp_path / "tree"
    for name, content in files.items():
        path = tree / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content.encode("utf-8"))
    out = tmp_path / "out"
    out.mkdir()
    writer = Writer(out, nasa_cea.TABLES, sorted(files))
    nasa_cea.read(tree, writer)
    result = writer.finish()
    return {
        name: pq.read_table(out / record.file) for name, record in result.tables.items()
    }, writer


def rows(tables: dict[str, pa.Table], name: str) -> list[dict]:
    return tables[name].to_pylist()


def sample_thermo() -> str:
    return thermo_text([*PRODUCTS, *REACTANTS])


def test_tables_are_declared_documented_and_located(tmp_path: Path) -> None:
    check_declared(nasa_cea.TABLES)
    tables, _ = run_reader(
        tmp_path, {"data/thermo.inp": sample_thermo(), "data/trans.inp": TRANS_SAMPLE}
    )
    assert set(tables) == set(nasa_cea.TABLES)
    for name, schema in nasa_cea.TABLES.items():
        assert schema.names[:2] == ["_artifact", "_locator"], name
        for field in schema:
            metadata = {k.decode(): v.decode() for k, v in (field.metadata or {}).items()}
            assert metadata.get("source_name") and metadata.get("unit"), (name, field.name)
    for name, table in tables.items():
        for row in table.to_pylist():
            assert re.fullmatch(r"data/(thermo|trans)\.inp#L\d+(/\d)?", row["_locator"]), (
                name,
                row["_locator"],
            )


def test_thermo_frame_records_and_locators(tmp_path: Path) -> None:
    tables, _ = run_reader(tmp_path, {"data/thermo.inp": sample_thermo()})
    frames = rows(tables, "frame_lines")
    sentinels = [
        i + 1 for i, line in enumerate(sample_thermo().split("\r\n")) if line.startswith("END")
    ]
    assert [(r["kind"], r["_locator"]) for r in frames] == [
        ("comment", "data/thermo.inp#L1"),
        ("comment", "data/thermo.inp#L2"),
        ("keyword", "data/thermo.inp#L3"),
        ("grid", "data/thermo.inp#L4"),
        ("sentinel", f"data/thermo.inp#L{sentinels[0]}"),
        ("sentinel", f"data/thermo.inp#L{sentinels[1]}"),
    ]
    grid = frames[3]
    assert grid["temperatures"] == [200.0, 1000.0, 6000.0, 20000.0] and grid["date"] == "9/8/2021"
    assert grid["text"] == GRID  # the CRLF is not part of the text
    records = rows(tables, "species_records")
    assert [(r["name"], r["section"], r["record_index"]) for r in records] == [
        ("H2", "products", 0),
        ("OH", "products", 1),
        ("Fuel", "reactants", 2),
    ]
    h2 = records[0]
    assert h2["_locator"] == "data/thermo.inp#L5" and h2["line"] == 5
    assert h2["notes"] == "Hydrogen. Gurvich,1978 pt1 p103."
    assert h2["interval_count"] == 2 and h2["reference_date_code"] == "g 6/97"
    assert h2["phase_flag"] == 0 and h2["molecular_weight"] == 2.01588
    assert h2["heat_of_formation"] == 0.0
    assert h2["raw_line_1"] == species_1("H2", "Hydrogen. Gurvich,1978 pt1 p103.")


def test_formula_pairs_keep_zero_and_skip_padding(tmp_path: Path) -> None:
    tables, _ = run_reader(tmp_path, {"data/thermo.inp": sample_thermo()})
    pairs = [
        (r["species_locator"], r["slot"], r["symbol"], r["count"])
        for r in rows(tables, "species_formula_pairs")
    ]
    oh = [p for p in pairs if p[0] == "data/thermo.inp#L" + str(_line_of(tables, "OH"))]
    # a written 0.00 with a symbol is a value; the padding slots 4 and 5 are not emitted
    assert [(p[1], p[2], p[3]) for p in oh] == [(1, "O", 1.0), (2, "H", 1.0), (3, "E", 0.0)]
    assert {r["_locator"] for r in rows(tables, "species_formula_pairs")} >= {
        f"data/thermo.inp#L{_line_of(tables, 'OH') + 1}/{slot}" for slot in (1, 2, 3)
    }


def _line_of(tables: dict[str, pa.Table], name: str) -> int:
    (row,) = [r for r in rows(tables, "species_records") if r["name"] == name]
    return row["line"]


def test_intervals_exponents_and_d_exponents(tmp_path: Path) -> None:
    tables, _ = run_reader(tmp_path, {"data/thermo.inp": sample_thermo()})
    species = rows(tables, "species_records")[0]
    intervals = [
        r for r in rows(tables, "thermo_intervals") if r["species_locator"] == species["_locator"]
    ]
    assert [(r["interval_index"], r["t_low"], r["t_high"]) for r in intervals] == [
        (0, 300.0, 1000.0),
        (1, 1000.0, 6000.0),
    ]
    first = intervals[0]
    assert first["exponent_count"] == 7 and first["enthalpy_increment"] == 8467.0
    assert first["exponents"] == list(EXPONENTS)
    assert first["coefficients"] == C_H2 and intervals[1]["coefficients"] == C_OH
    assert first["raw_line_coefficients_1"] == coefficients(C_H2)[0]
    assert first["raw_line_coefficients_2"] == coefficients(C_H2)[1]
    assert first["_locator"] == f"data/thermo.inp#L{species['line'] + 2}"


def test_a_record_without_coefficients_and_blank_fields(tmp_path: Path) -> None:
    tables, _ = run_reader(tmp_path, {"data/thermo.inp": sample_thermo()})
    fuel = next(r for r in rows(tables, "species_records") if r["name"] == "Fuel")
    assert fuel["interval_count"] == 0 and fuel["section"] == "reactants"
    # a blank molecular-weight field is no value, not zero; the written heat of formation is
    assert fuel["molecular_weight"] is None and fuel["heat_of_formation"] == -251140.0
    (reference,) = [
        r for r in rows(tables, "thermo_intervals") if r["species_locator"] == fuel["_locator"]
    ]
    assert reference["coefficients"] is None and reference["t_low"] == 298.15
    assert reference["raw_line_coefficients_1"] is None
    hydrogen = rows(tables, "species_records")[0]
    assert hydrogen["heat_of_formation"] == 0.0  # written zero stays zero
    assert fuel["notes"] == "Assigned enthalpy only."


TRANS_SAMPLE = "\r\n".join(
    [
        "transport property coefficients         ",
        f"{'Ar':<16}{'':<16}  V2C1  BICH ET AL (1990)",
        " V  200.0   1000.0   0.61205763E 00-0.67714354E 02 0.19040660E 03 0.21588272E 01",
        " V 1000.0   5000.0   0.69357334E+00 0.70953943E+02-0.28386007E+05 0.14856447E+01",
        " C  200.0   1000.0   0.60968928E 00-0.70892249E 02 0.58420624E 03 0.19337152E 01",
        f"{'C':<16}{'O':<16}  V1C0  CAPITELLI & FICOCELLI (1973)",
        " V 4000.0  15000.0   0.12635466E+01 0.46866528E+04-0.59789292E+07-0.43066246E+01",
        "end ",
        "",
    ]
)


def test_transport_entries_and_intervals(tmp_path: Path) -> None:
    tables, _ = run_reader(tmp_path, {"data/trans.inp": TRANS_SAMPLE})
    frames = rows(tables, "frame_lines")
    assert [(r["kind"], r["_locator"]) for r in frames] == [
        ("title", "data/trans.inp#L1"),
        ("end", "data/trans.inp#L8"),
    ]
    entries = rows(tables, "trans_entries")
    assert [(e["name_1"], e["name_2"], e["reference"]) for e in entries] == [
        ("Ar", None, "BICH ET AL (1990)"),
        ("C", "O", "CAPITELLI & FICOCELLI (1973)"),
    ]
    assert [(e["viscosity_interval_count"], e["conductivity_interval_count"]) for e in entries] == [
        (2, 1),
        (1, 0),
    ]
    assert entries[0]["_locator"] == "data/trans.inp#L2"
    intervals = rows(tables, "trans_intervals")
    assert [(r["property"], r["interval_index"], r["t_low"], r["t_high"]) for r in intervals] == [
        ("V", 0, 200.0, 1000.0),
        ("V", 1, 1000.0, 5000.0),
        ("C", 0, 200.0, 1000.0),
        ("V", 0, 4000.0, 15000.0),
    ]
    # a space in place of the exponent sign reads as a plus
    assert intervals[0]["coefficients"] == [0.61205763, -67.714354, 190.4066, 2.1588272]
    assert intervals[1]["coefficients"] == [0.69357334, 70.953943, -28386.007, 1.4856447]
    assert intervals[0]["entry_locator"] == "data/trans.inp#L2"
    assert intervals[3]["entry_locator"] == "data/trans.inp#L6"
    assert intervals[0]["raw_line"] == TRANS_SAMPLE.split("\r\n")[2]


@pytest.mark.parametrize(
    ("field", "decimals", "expected"),
    [
        ("0.61205763E 00", 8, 0.61205763),
        ("-0.67714354E 02", 8, -67.714354),
        ("-7.453750000D+02", 9, -745.375),
        ("2.5d-1", 9, 0.25),
        ("1.5-03", 2, 0.0015),
        ("  20000.  ", 3, 20000.0),
        ("  .00000", 5, 0.0),
        ("100", 2, 1.0),
        ("-150", 1, -15.0),
        ("   ", 3, None),
    ],
)
def test_fortran_reals(field: str, decimals: int, expected: float | None) -> None:
    assert read_real(field, decimals) == expected


@pytest.mark.parametrize("field", ["abc", "1.2.3", "E5", "--1.0", "1.0E", "."])
def test_unreadable_reals_are_refused(field: str) -> None:
    with pytest.raises(ValueError):
        read_real(field, 2)


def test_a_bad_numeric_field_is_refused_with_line_and_columns(tmp_path: Path) -> None:
    text = sample_thermo().replace(" 2.01588", " 2.0x588", 1)
    with pytest.raises(FieldError, match=r"data/thermo\.inp, line 6, columns 53-65: cannot read"):
        run_reader(tmp_path, {"data/thermo.inp": text})
    bad = thermo_text(PRODUCTS).replace("D+", "Q+", 1)
    with pytest.raises(FieldError, match=r"data/thermo\.inp, line 8, columns 1-16: cannot read"):
        run_reader(tmp_path / "again", {"data/thermo.inp": bad})


@pytest.mark.parametrize(
    ("mutate", "message"),
    [
        (
            lambda t: t.replace("thermo\r\n", "therm\r\n", 1),
            r"data/thermo\.inp, line 3: expected a comment line or the word thermo",
        ),
        (lambda t: t.replace("END REACTANTS\r\n", ""), "the file ends without an END REACTANTS"),
        (
            lambda t: t.rsplit("\r\n", 4)[0] + "\r\n",
            "inside",
        ),
        (
            lambda t: t.replace("Hydrogen.", "x" * 40 + "Hydrogen.", 1),
            r"line 5: 87 characters; a record line has at most 80",
        ),
    ],
)
def test_malformed_thermo_is_refused_with_its_location(
    tmp_path: Path, mutate: object, message: str
) -> None:
    text = mutate(thermo_text([*PRODUCTS, *REACTANTS]))  # type: ignore[operator]
    with pytest.raises(StagingError, match=message):
        run_reader(tmp_path, {"data/thermo.inp": text})


def test_a_thermo_file_without_the_keyword_is_refused(tmp_path: Path) -> None:
    with pytest.raises(StagingError, match="no line starting with the word thermo"):
        run_reader(tmp_path, {"data/thermo.inp": "! only comments\r\n"})


@pytest.mark.parametrize(
    ("mutate", "message"),
    [
        (
            lambda t: t.replace("V2C1", "X2C1"),
            r"trans\.inp, line 2, columns 35-37: an entry header",
        ),
        (
            lambda t: t.replace(" C  200.0", " V  200.0"),
            r"line 5, columns 2-2: expected the property letter C",
        ),
        (lambda t: t.replace("end \r\n", ""), "the file ends without the closing end line"),
        (
            lambda t: t.replace("0.61205763E 00", "0.6120x763E 00"),
            r"line 3, columns 21-35: cannot read",
        ),
    ],
)
def test_malformed_transport_is_refused_with_its_location(
    tmp_path: Path, mutate: object, message: str
) -> None:
    with pytest.raises(StagingError, match=message):
        run_reader(tmp_path, {"data/trans.inp": mutate(TRANS_SAMPLE)})  # type: ignore[operator]


def test_a_payload_file_without_a_rule_is_refused(tmp_path: Path) -> None:
    with pytest.raises(StagingError, match="data/other.inp: the NASA CEA reader has no rule"):
        run_reader(tmp_path, {"data/other.inp": "x\r\n"})


# -- the real tree -----------------------------------------------------------------------------

real = pytest.mark.skipif(
    not TREE.is_dir(),
    reason=f"the acquired NASA CEA tree {TREE} is absent (run `tk acquire nasa_cea`)",
)


@pytest.fixture(scope="module")
def staged(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """The real `tk read nasa_cea`, into a temporary staged directory."""
    ctx = StageContext(staged_dir=tmp_path_factory.mktemp("staged"))
    manifests = load_sources(default_sources_dir())
    outcome = stage.read_source(ctx, manifests["nasa_cea"], read_lock(default_lock_path()))
    assert outcome.status == "read"
    return stage.staged_path(ctx, "nasa_cea", PIN)


def table(staged: Path, name: str) -> pa.Table:
    return pq.read_table(staged / f"{name}.parquet")


def raw_lines(name: str) -> list[str]:
    text = (TREE / "data" / name).read_bytes().decode("utf-8")
    return text.split("\r\n")[:-1]


NUMBER = r"[ -]\d\.\d{9}D[+-]\d\d"
INTERVAL_LINE = re.compile(
    r"^ +[\d.]+ +[\d.]+ ?7 -2\.0 -1\.0  0\.0  1\.0  2\.0  3\.0  4\.0  0\.0 +[\d.]+$"
)
REFERENCE_LINE = re.compile(r"^ +[\d.]+ +0\.0000(?:  0\.0){8} +0\.000$")
COEFFICIENTS_1 = re.compile(rf"^(?:{NUMBER}){{5}}$")
# the 16 columns that the format skips (16X) are blank in most records and hold a written zero
# in 58 intervals
COEFFICIENTS_2 = re.compile(rf"^(?:{NUMBER}){{2}}(?: {{16}}|{NUMBER})(?:{NUMBER}){{2}}$")


def species_lines(lines: list[str]) -> list[str]:
    """The species lines (two per record) of thermo.inp: what remains after the comment block,
    the keyword, the grid, the sentinels and every interval and coefficient line."""
    body = lines[lines.index("thermo") + 2 :]
    return [
        line
        for line in body
        if not (
            line.startswith("END")
            or INTERVAL_LINE.match(line)
            or REFERENCE_LINE.match(line)
            or COEFFICIENTS_1.match(line)
            or COEFFICIENTS_2.match(line)
        )
    ]


@real
def test_every_table_is_staged_and_both_files_are_read(staged: Path) -> None:
    manifest = staged_manifest.read(staged)
    assert set(manifest.tables) == set(nasa_cea.TABLES)
    assert manifest.reader.name == "nasa_cea" and manifest.reader.version == nasa_cea.READER_VERSION
    assert {r.path: r.status for r in manifest.payload} == {
        "data/thermo.inp": "read",
        "data/trans.inp": "read",
    }


@real
def test_thermo_counts_against_line_patterns(staged: Path) -> None:
    lines = raw_lines("thermo.inp")
    comments = sum(1 for line in lines if line.startswith("!"))
    intervals = sum(1 for line in lines if INTERVAL_LINE.match(line))
    references = sum(1 for line in lines if REFERENCE_LINE.match(line))
    # a second coefficient line that writes the zero in its skipped columns also has five fields
    assert (
        sum(1 for l in lines if COEFFICIENTS_1.match(l) or COEFFICIENTS_2.match(l)) == 2 * intervals
    )
    assert (len(lines), comments, intervals, references) == (15802, 62, 3820, 54)
    skipped = [lines[i + 2][32:48] for i, line in enumerate(lines) if INTERVAL_LINE.match(line)]
    assert collections.Counter(skipped) == {" " * 16: 3762, " 0.000000000D+00": 58}
    both = species_lines(lines)
    # the 62 comments, the keyword, the grid and the two sentinels frame the records; every
    # record has two species lines
    framing = comments + 4
    assert len(lines) == framing + len(both) + 3 * intervals + references
    assert len(both) % 2 == 0 and len(both) // 2 == 2111
    second = both[1::2]
    assert all(line[2] == " " and line[:2].strip().isdigit() and line[9] == " " for line in second)
    records = table(staged, "species_records")
    assert records.num_rows == 2111
    assert table(staged, "thermo_intervals").num_rows == intervals + references
    frames = table(staged, "frame_lines")
    assert (
        frames.filter(pa.compute.equal(frames["_artifact"], "data/thermo.inp")).num_rows == framing
    )
    assert [line[:15].strip() for line in both[0::2]] == records.column("name").to_pylist()
    assert len(set(records.column("name").to_pylist())) == 2099
    by_section = collections.Counter(records.column("section").to_pylist())
    assert by_section == {"products": 2030, "reactants": 81}
    end_products = lines.index("END PRODUCTS")
    first_reactant = next(r for r in records.to_pylist() if r["section"] == "reactants")["line"]
    assert first_reactant == end_products + 2  # line numbers are 1-based
    counts = collections.Counter(records.column("interval_count").to_pylist())
    assert counts[0] == references == 54
    assert sum(k * v for k, v in counts.items()) == intervals == 3820


@real
def test_formula_pairs_against_the_formula_columns(staged: Path) -> None:
    expected = 0
    anomalies = 0
    for line in species_lines(raw_lines("thermo.inp"))[1::2]:
        region = line.ljust(80)[10:50]
        for slot in range(5):
            chunk = region[8 * slot : 8 * slot + 8]
            if chunk[:2].strip():
                expected += 1
            elif chunk[2:].strip() and float(chunk[2:]) != 0.0:
                anomalies += 1
    pairs = table(staged, "species_formula_pairs")
    assert pairs.num_rows == expected == 4718
    assert anomalies == 0
    assert collections.Counter(pairs.column("slot").to_pylist())[5] == 1  # one 5-element record
    symbols = set(pairs.column("symbol").to_pylist())
    assert {"E", "IH", "IC", "IO", "AR"} <= symbols
    assert any(c is not None and c < 0 for c in pairs.column("count").to_pylist())  # E -1.00 ions


@real
def test_every_interval_reproduces_its_raw_lines(staged: Path) -> None:
    intervals = table(staged, "thermo_intervals").to_pylist()
    lines = {i + 1: line for i, line in enumerate(raw_lines("thermo.inp"))}
    fitted = 0
    for row in intervals:
        number = int(row["_locator"].split("#L")[1])
        line = lines[number]
        assert row["raw_line_interval"] == line
        # independent slicing of the interval line (the files space the bounds irregularly)
        assert row["t_low"] == float(line[0:11]) and row["t_high"] == float(line[11:22])
        assert row["exponent_count"] == int(line[22])
        assert row["exponents"] == [float(line[23 + 5 * i : 28 + 5 * i]) for i in range(8)]
        assert row["enthalpy_increment"] == float(line[65:80])
        if row["coefficients"] is None:
            assert row["exponent_count"] == 0 and row["raw_line_coefficients_1"] is None
            continue
        fitted += 1
        one, two = lines[number + 1], lines[number + 2]
        assert (row["raw_line_coefficients_1"], row["raw_line_coefficients_2"]) == (one, two)
        fields = [one[16 * i : 16 * i + 16] for i in range(5)]
        fields += [two[0:16], two[16:32], two[48:64], two[64:80]]
        # the Fortran D exponent is read as a float: re-serialised it is the text of the field
        assert [f"{v:.9E}".replace("E", "D") for v in row["coefficients"]] == [
            field.strip() for field in fields
        ]
        assert row["coefficients"] == [float(field.strip().replace("D", "E")) for field in fields]
    assert fitted == 3820


@real
def test_species_fields_reproduce_their_raw_lines(staged: Path) -> None:
    lines = {i + 1: line for i, line in enumerate(raw_lines("thermo.inp"))}
    for row in table(staged, "species_records").to_pylist():
        first, second = lines[row["line"]], lines[row["line"] + 1]
        assert row["raw_line_1"] == first and row["raw_line_2"] == second
        assert row["name"] == first[:15].strip()
        assert (row["notes"] or "") == first[15:80].strip()
        assert row["interval_count"] == int(second[0:2])
        assert row["phase_flag"] == int(second[50:52])
        assert row["molecular_weight"] == float(second[52:65])
        assert row["heat_of_formation"] == float(second[65:80])
        assert row["reference_date_code"] == (second[3:9].strip() or None)


@real
def test_air_and_the_electron_are_read_as_written(staged: Path) -> None:
    records = {r["name"]: r for r in table(staged, "species_records").to_pylist()}
    electron, air = records["e-"], records["Air"]
    assert electron["molecular_weight"] == 0.000548579903 and electron["interval_count"] == 3
    assert air["molecular_weight"] == 28.9651159 and air["section"] == "reactants"
    pairs = [
        (p["symbol"], p["count"])
        for p in table(staged, "species_formula_pairs").to_pylist()
        if p["species_locator"] == air["_locator"]
    ]
    assert pairs == [("N", 1.5617), ("O", 0.41959), ("AR", 0.00937), ("C", 0.00032)]
    butanol = [r for r in table(staged, "species_records").to_pylist() if r["name"] == "n-Butanol"]
    assert [(r["phase_flag"], r["heat_of_formation"]) for r in butanol] == [
        (0, -251140.0),
        (1, -278510.0),
    ]


@real
def test_transport_counts_and_values_against_the_text(staged: Path) -> None:
    lines = raw_lines("trans.inp")
    headers = [line for line in lines if re.search(r"^.{34}V\dC\d", line)]
    rows_ = [line for line in lines if re.match(r"^ [VC] ", line)]
    rows_ = [line for line in lines if re.match(r"^ [VC]", line)]
    assert (len(lines), len(headers), len(rows_)) == (488, 107, 379)
    entries = table(staged, "trans_entries")
    intervals = table(staged, "trans_intervals")
    assert entries.num_rows == 107 and intervals.num_rows == 379
    assert (
        sum(
            a + b
            for a, b in zip(
                entries.column("viscosity_interval_count").to_pylist(),
                entries.column("conductivity_interval_count").to_pylist(),
                strict=True,
            )
        )
        == 379
    )
    assert sum(1 for n in entries.column("name_2").to_pylist() if n is not None) == 41
    assert entries.column("raw_line").to_pylist() == headers
    assert intervals.column("raw_line").to_pylist() == rows_
    frames = table(staged, "frame_lines").to_pylist()
    assert [(r["kind"], r["_locator"]) for r in frames if r["_artifact"] == "data/trans.inp"] == [
        ("title", "data/trans.inp#L1"),
        ("end", "data/trans.inp#L488"),
    ]
    for row in intervals.to_pylist():
        line = row["raw_line"]
        numbers = re.findall(r"[-+]?\d\.\d{8}E[-+ ]\d\d", line)
        assert len(numbers) == 4
        assert row["coefficients"] == [float(n.replace("E ", "E+")) for n in numbers]
        assert row["property"] == line[1]
        assert (row["t_low"], row["t_high"]) == (float(line[2:11]), float(line[11:20]))
    uf6 = next(r for r in entries.to_pylist() if r["name_1"] == "UF6")
    assert (uf6["viscosity_interval_count"], uf6["conductivity_interval_count"]) == (2, 0)
    assert sum(1 for x in intervals.column("coefficients").to_pylist() if None in x) == 0
    spaced = sum(1 for row in rows_ if re.search(r"E \d\d", row))
    assert spaced > 0 and row_sign_fields(rows_) == 990


def row_sign_fields(rows_: list[str]) -> int:
    """Coefficient fields that write a space instead of an exponent sign."""
    return sum(len(re.findall(r"\dE \d\d", row)) for row in rows_)


@real
def test_the_staged_source_loads_into_a_database(staged: Path, test_database: TestDatabase) -> None:
    outcome = load.load_staged(test_database.url, staged, "src_nasa_cea")
    manifest = staged_manifest.read(staged)
    assert outcome.tables == {name: record.rows for name, record in manifest.tables.items()}
    from thermo_knowledge import db

    with db.connect(test_database.url) as conn:
        (count,) = conn.execute('SELECT count(*) FROM src_nasa_cea."species_records"').fetchone()
        assert count == 2111
        row = conn.execute(
            'SELECT "coefficients", "exponents" FROM src_nasa_cea.thermo_intervals '
            "WHERE _locator = 'data/thermo.inp#L67'"
        ).fetchone()
        assert row is not None and len(row[0]) == 9 and row[1][0] == -2.0
