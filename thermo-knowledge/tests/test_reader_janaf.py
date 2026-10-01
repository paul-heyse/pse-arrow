# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The JANAF reader: synthetic table files in the source's format, then the real acquired tables.

Real-tree counts come from `wc` and `grep` over the files, independently of the reader.
"""

from __future__ import annotations

import subprocess
from pathlib import Path

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
from thermo_knowledge.acquire.lock import read_lock
from thermo_knowledge.acquire.manifest import default_lock_path
from thermo_knowledge.readers import janaf
from thermo_knowledge.staging.errors import StagingError

HEADER = "T(K)\tCp\tS\t-[G-H(Tr)]/T\tH-H(Tr)\tdelta-f H\tdelta-f G\tlog Kf"


def table(*lines: str, title: str = "Demo Substance (Dm)\tDm1(cr,l)") -> str:
    return "\n".join([title, HEADER, *lines]) + "\n"


SYNTHETIC = table(
    "0\t0.\t0.\tINFINITE\t-4.539\t0.\t0.\tINFINITE",
    "100\t12.997\t6.987\t47.543\t-4.056\t0.\t0.\t0.",
    "298.15\t24.209\t28.275\t28.275\t0.\t-5.5\t-2.25\t.75",
    "933.450\t32.959\t59.738\t40.474\t17.982\tCRYSTAL <--> LIQUID",
    "+\t",
    "933.450\t31.751\t71.213\t40.474\t28.693\tTRANSITION",
    "1000\t34.358\t62.055\t41.834\t20.221\t10.585 0.760  0.040",
    "1100\t20.786\t127.243\t168.431\t-4.119\t\t\t\t\t\t",
    "1200\t20.786\t1.5E+2",
    "H\t\t\t\t\t\t\t\t\t\t",
    "\t\t\t\t\t\t\t\t\t",
    "",
    "288.5001548.088\t111.062\t118.520\t-2.152\tII <--> III",
)


def by_line(run, number: int) -> dict:  # type: ignore[no-untyped-def]
    return next(r for r in run.rows("rows") if r["line_number"] == number)


def test_every_declared_column_documents_name_and_unit() -> None:
    for name, schema in janaf.TABLES.items():
        for field in schema:
            metadata = field_metadata(schema, field.name)
            assert metadata.get("source_name"), (name, field.name)
            assert metadata.get("unit"), (name, field.name)
    assert field_metadata(janaf.TABLES["rows"], "t")["unit"] == "K"
    assert field_metadata(janaf.TABLES["rows"], "cp")["unit"] == "not stated"
    assert field_metadata(janaf.TABLES["rows"], "log_kf")["unit"] == "not stated"


def test_title_header_and_counts(tmp_path: Path) -> None:
    run = run_reader(janaf, tmp_path, {"Al-001.txt": SYNTHETIC})
    (head,) = run.rows("tables")
    assert head["_locator"] == "Al-001.txt#L1" and head["code"] == "Al-001"
    assert head["title_field_1"] == "Demo Substance (Dm)" and head["title_field_2"] == "Dm1(cr,l)"
    assert (head["substance_name"], head["name_formula"]) == ("Demo Substance", "Dm")
    assert (head["janaf_formula"], head["phase_designator"]) == ("Dm1", "cr,l")
    assert head["header_line"] == HEADER and head["ends_with_newline"] is True
    assert (head["line_count"], head["data_row_count"]) == (15, 10)
    assert (head["plus_count"], head["empty_count"], head["blank_count"]) == (1, 1, 1)
    kinds = [r["row_kind"] for r in run.rows("rows")]
    assert kinds.count("data") == 10 and kinds.count("plus") == 1
    assert [r["_locator"] for r in run.rows("rows")][:2] == ["Al-001.txt#L3", "Al-001.txt#L4"]
    assert [r["row_index"] for r in run.rows("rows")][:2] == [0, 1]


def test_nested_parentheses_in_a_title(tmp_path: Path) -> None:
    content = table(
        "0\t0.\t0.\tINFINITE\t0.\t0.\t0.\t0.", title="Aluminum Bromide ((AlBr3)2)\tAl2Br6(g)"
    )
    (head,) = run_reader(janaf, tmp_path, {"Al-084.txt": content}).rows("tables")
    assert (head["substance_name"], head["name_formula"]) == ("Aluminum Bromide", "(AlBr3)2")
    assert (head["janaf_formula"], head["phase_designator"]) == ("Al2Br6", "g")
    content = table("0\t0.", title="Odd name\tno group")
    (head,) = run_reader(janaf, tmp_path / "second", {"Al-002.txt": content}).rows("tables")
    assert head["substance_name"] is None and head["phase_designator"] is None
    assert head["title_field_1"] == "Odd name"  # the raw fields always stay


def test_special_values_stay_distinct_from_zero(tmp_path: Path) -> None:
    run = run_reader(janaf, tmp_path, {"Al-001.txt": SYNTHETIC})
    first = by_line(run, 3)
    assert first["t"] == 0.0 and first["cp"] == 0.0 and first["s"] == 0.0  # written 0 and 0.
    assert first["g_function"] is None and first["g_function_text"] == "INFINITE"
    assert first["log_kf"] is None and first["log_kf_text"] == "INFINITE"
    assert first["marker"] is None and first["cell_count"] == 8
    plain = by_line(run, 5)
    assert plain["delta_f_h"] == -5.5 and plain["log_kf"] == 0.75 and plain["log_kf_text"] == ".75"
    assert by_line(run, 4)["delta_f_h"] == 0.0 and by_line(run, 4)["delta_f_h_text"] == "0."
    padded = by_line(run, 10)
    assert padded["h_increment"] == -4.119
    assert padded["delta_f_h"] is None and padded["delta_f_h_text"] == ""  # present and empty
    assert padded["cell_count"] == 11
    short = by_line(run, 11)
    assert short["g_function"] is None and short["g_function_text"] is None  # the line ends first
    assert short["s"] == 1.5e2 and short["s_text"] == "1.5E+2" and short["cell_count"] == 3
    stray = by_line(run, 12)
    assert stray["t"] is None and stray["t_text"] == "H" and stray["row_kind"] == "data"
    assert by_line(run, 13)["row_kind"] == "empty"
    assert by_line(run, 14)["row_kind"] == "blank"
    assert by_line(run, 14)["cell_count"] is None


def test_marker_rows_and_fused_formation_cells(tmp_path: Path) -> None:
    run = run_reader(janaf, tmp_path, {"Al-001.txt": SYNTHETIC})
    lower, upper = by_line(run, 6), by_line(run, 8)
    assert lower["marker"] == "CRYSTAL <--> LIQUID" and lower["delta_f_h"] is None
    assert lower["delta_f_h_text"] == "CRYSTAL <--> LIQUID" and lower["delta_f_g_text"] is None
    assert upper["marker"] == "TRANSITION" and upper["h_increment"] == 28.693
    assert by_line(run, 7)["row_kind"] == "plus"
    fused = next(r for r in run.rows("rows") if r["fused_cells"])
    assert fused["line_number"] == 9
    assert (fused["delta_f_h"], fused["delta_f_g"], fused["log_kf"]) == (10.585, 0.76, 0.04)
    assert fused["fused_cells"] == "delta_f_h,delta_f_g,log_kf"
    assert fused["delta_f_h_text"] == "10.585 0.760  0.040"
    assert fused["delta_f_g_text"] is None and fused["marker"] is None
    assert fused["raw_line"].endswith("10.585 0.760  0.040")


def test_digits_run_together_get_no_positional_values(tmp_path: Path) -> None:
    run = run_reader(janaf, tmp_path, {"Al-001.txt": SYNTHETIC})
    joined = by_line(run, 15)
    assert joined["row_kind"] == "data" and joined["cell_count"] == 5
    assert "run together" in joined["parse_note"]
    assert joined["t"] is None and joined["t_text"] is None and joined["cp"] is None
    assert joined["marker"] is None and joined["fused_cells"] is None
    assert joined["raw_line"] == "288.5001548.088\t111.062\t118.520\t-2.152\tII <--> III"
    assert by_line(run, 3)["parse_note"] is None


def test_files_without_final_newline_and_the_payload_accounting(tmp_path: Path) -> None:
    content = table("0\t0.\t0.\tINFINITE\t0.\t0.\t0.\t0.").rstrip("\n")
    run = run_reader(janaf, tmp_path, {"Al-001.txt": content, "B-002.txt": SYNTHETIC})
    heads = {h["code"]: h for h in run.rows("tables")}
    assert heads["Al-001"]["ends_with_newline"] is False and heads["Al-001"]["line_count"] == 3
    assert {r.path: r.status for r in run.result.payload} == {
        "Al-001.txt": "read",
        "B-002.txt": "read",
    }


@pytest.mark.parametrize(
    ("files", "message"),
    [
        ({"Al-001.txt": "only a title\n"}, r"Al-001\.txt#L1: a table file needs a title line"),
        ({"Al-001.txt": "a\tb\tc\n" + HEADER + "\n"}, r"Al-001\.txt#L1: the title line has 3"),
        (
            {"Al-001.txt": "a\tb\nT(K)\tCp\n"},
            r"Al-001\.txt#L2: the header line is not the standard",
        ),
        ({"notes.txt": table("0\t0.")}, r"notes\.txt: not a JANAF table file name"),
        (
            {"Al-001.txt": table("0\t0.\t0.\t0.\t0.\t0.\t0.\t0.\tstray")},
            r"Al-001\.txt#L3: a non-empty cell beyond the eighth column",
        ),
        ({"Al-001.txt": table("0\t0.\r")}, r"Al-001\.txt#L3: a carriage return"),
    ],
)
def test_malformed_files_are_refused_with_a_located_error(
    tmp_path: Path, files: dict[str, str], message: str
) -> None:
    with pytest.raises(StagingError, match=message):
        run_reader(janaf, tmp_path, files)


# -- the real acquired tables -----------------------------------------------------------------------


def acquired_tree() -> Path | None:
    try:
        entry = read_lock(default_lock_path()).get("janaf")
    except Exception:  # noqa: BLE001 - an unreadable lock means there is nothing to read
        return None
    if entry is None or entry.pin is None:
        return None
    tree = store.pin_dir(config.raw_dir(), "janaf", entry.pin) / store.TREE_DIR_NAME
    return tree if tree.is_dir() else None


TREE = acquired_tree()
real = pytest.mark.skipif(
    TREE is None, reason="the JANAF tables are not acquired (run `tk acquire janaf`)"
)


@pytest.fixture(scope="module")
def staged(tmp_path_factory: pytest.TempPathFactory) -> Path:
    return stage_real("janaf", tmp_path_factory.mktemp("staged"))


def shell(command: str) -> str:
    assert TREE is not None
    return subprocess.run(  # noqa: S602 - fixed commands over the acquired tree
        command, shell=True, cwd=TREE, capture_output=True, text=True, check=True
    ).stdout.strip()


@real
def test_counts_match_wc_and_grep(staged: Path) -> None:
    manifest = staged_manifest_of(staged)
    files = int(shell("ls *.txt | wc -l"))
    assert manifest.tables["tables"].rows == files == len(manifest.payload)
    assert all(r.status == "read" for r in manifest.payload)
    assert {r.path for r in manifest.payload} == {p.name for p in TREE.glob("*.txt")}  # type: ignore[union-attr]
    total_lines = int(shell("cat *.txt | wc -l"))
    assert manifest.tables["rows"].rows == total_lines - 2 * files
    plus = int(shell("cat *.txt | grep -c -E '^\\+[[:space:]]*$' || true"))
    blank = int(shell("cat *.txt | grep -c -E '^$' || true"))
    empty = int(shell("cat *.txt | grep -c -P '^\\t+$' || true"))
    kinds = staged_table(staged, "rows").column("row_kind").to_pylist()
    assert kinds.count("plus") == plus
    assert kinds.count("blank") == blank
    assert kinds.count("empty") == empty
    assert kinds.count("data") == total_lines - 2 * files - plus - blank - empty
    titles = staged_table(staged, "tables").to_pylist()
    assert sum(t["data_row_count"] for t in titles) == kinds.count("data")
    assert all(t["header_line"] == janaf.HEADER for t in titles)


@real
def test_markers_fusions_and_infinite_against_grep(staged: Path) -> None:
    rows = staged_table(staged, "rows").to_pylist()
    marked = int(
        shell("cat *.txt | grep -c -P '^[^\\t]*(\\t[^\\t]*){4}\\t[A-Za-z][^\\t]*(\\t|$)' || true")
    )
    markers = [r for r in rows if r["marker"]]
    fused_digits = [r for r in rows if r["parse_note"]]
    assert len(markers) == marked - len(
        staged_table(staged, "tables").to_pylist()
    )  # less the headers
    assert {r["_locator"] for r in fused_digits} >= {"C-083.txt#L8", "C-083.txt#L9"}
    assert {r["marker"] for r in markers} >= {"TRANSITION", "CRYSTAL <--> LIQUID"}
    infinite_cells = int(shell("cat *.txt | grep -o 'INFINITE' | wc -l"))
    staged_infinite = sum(
        (r["g_function_text"] == "INFINITE") + (r["log_kf_text"] == "INFINITE") for r in rows
    )
    assert staged_infinite == infinite_cells
    assert all(r["g_function"] is None for r in rows if r["g_function_text"] == "INFINITE")
    fused = [r for r in rows if r["fused_cells"]]
    assert len(fused) == int(
        shell("cat *.txt | grep -c -P '\\t[-0-9.]+ +[-0-9.]+ +[-0-9.]+( |$)' || true")
    )
    assert all(r["delta_f_g"] is not None and r["log_kf"] is not None for r in fused)


@real
def test_round_trip_reproduces_every_file_exactly(staged: Path) -> None:
    tables = {t["_artifact"]: t for t in staged_table(staged, "tables").to_pylist()}
    lines: dict[str, list[tuple[int, str]]] = {}
    for row in staged_table(staged, "rows").to_pylist():
        lines.setdefault(row["_artifact"], []).append((row["line_number"], row["raw_line"]))
    assert set(lines) <= set(tables)
    for artifact, head in tables.items():
        ordered = [text for _, text in sorted(lines.get(artifact, []))]
        rebuilt = "\n".join(
            [head["title_field_1"] + "\t" + head["title_field_2"], head["header_line"], *ordered]
        )
        if head["ends_with_newline"]:
            rebuilt += "\n"
        with (TREE / artifact).open(encoding="utf-8", newline="") as handle:  # type: ignore[operator]
            assert rebuilt == handle.read(), artifact


@real
def test_staged_numbers_equal_the_cells_of_the_files(staged: Path) -> None:
    """Every staged float is `float()` of the cell text it came from, and every cell the reader
    read as text or a fused field is not silently numeric."""
    names = [name for name, _, _ in janaf.COLUMNS]
    checked = 0
    for row in staged_table(staged, "rows").to_pylist():
        if row["row_kind"] != "data" or row["parse_note"]:
            continue
        cells = row["raw_line"].split("\t")
        for position, name in enumerate(names):
            value = row[name]
            if row["fused_cells"] and name in janaf.FORMATION:
                continue
            if position >= len(cells):
                assert value is None and row[f"{name}_text"] is None
            else:
                assert row[f"{name}_text"] == cells[position]
                if value is not None:
                    assert value == float(cells[position])
                    checked += 1
                else:
                    assert not janaf.NUMBER.match(cells[position])
    assert checked > 400_000
    lock_pin = read_lock(default_lock_path())["janaf"].pin
    assert staged_manifest_of(staged).pin == lock_pin
