# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The IDAES reader: synthetic modules and JSON files (invented values, written here), then the
pinned checkout.

The reader never imports IDAES. The real-tree tests count with a different traversal than the
reader's (`ast.walk` over all statements; a recursive walk of `json.load` for the JSON files) and
check values by evaluating the literal structures with Python's own evaluator over a stand-in
namespace for the names, and by rebuilding each JSON file from its staged rows.
"""

from __future__ import annotations

import ast
import io
import json
import tokenize
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
from thermo_knowledge.readers import idaes
from thermo_knowledge.staging.errors import StagingError

TREE = config.REPO_ROOT / "external" / "idaes-pse"
PACKAGE = "idaes/models/properties"
EXAMPLES = f"{PACKAGE}/modular_properties/examples"
DEMO = f"{EXAMPLES}/demo.py"

CONFIGURATION = """
from pyomo.environ import units as pyunits

configuration = {
    "components": {
        "alpha": {
            "type": Component,
            "pressure_sat_comp": RPP4,
            "phase_equilibrium_form": {("Vap", "Liq"): log_fugacity},
            "parameter_data": {
                "mw": (10.5e-3, pyunits.kg / pyunits.mol),
                "omega": 0.25,
                "coeff": {
                    "A": (-1.5, pyunits.J / pyunits.mol / pyunits.K),
                    "B": (2 * 3.0, None),
                },
                "flag": True,
            },
        },
    },
    "state_bounds": {"pressure": (5e4, 1e5, 1e6, pyunits.Pa)},
    "parameter_data": {"kappa": {("alpha", "alpha"): 0.0, ("alpha", "beta"): -0.125}},
}
"""

OLD_STYLE = """
class DemoParameterData(Base):
    def build(self):
        self.pressure_reference = Param(mutable=True, default=101325, units=pyunits.Pa, doc="text")
        critical_data = {"alpha": 4.5e5, "beta": 3}
        self.pressure_critical = Param(
            self.component_list,
            within=NonNegativeReals,
            initialize=extract_data(critical_data),
            units=pyunits.Pa,
        )
        self.element_comp = {"alpha": {"C": 1, "H": 4}, "beta": {}}
        self.factor = Var(initialize=2.5)
        lookup = {"x": compute(1), "y": 2}
        ratio = 3 * 0.5
        self.stoich["R1", "Liq", "A"] = -1
"""

ALGORITHMIC = """
TOL = 1e-6

class Kind:
    first = 1
    second = 2

class Holder:
    table = {"a": [1.5, 2.5], "b": 7}

def compute(x):
    n_cons = 3
    local_table = {"k": 0.5}
    return x
"""

PARAMETER_FILE = """{
    "comp": "fluidx",
    "basic": {"R": 0.25, "MW": 40, "P_max": 6.5e7, "rho_v": 0.0000125, "T_min": -10},
    "eos": {
        "reference": ["First invented line", "Second invented line"],
        "n": {"1": 0.5, "2": -1.25, "10": 3},
        "a/b": {"t~c": [1, 2.0, [3]]},
        "[0]": true,
        "absent": null,
        "none_yet": {},
        "no_items": []
    },
    "have_visc": false
}
"""

ROOT_SCALAR = "12.50"


def test_every_declared_column_documents_name_and_unit() -> None:
    for name, schema in idaes.TABLES.items():
        for field in schema:
            metadata = field_metadata(schema, field.name)
            assert metadata.get("source_name"), (name, field.name)
            assert metadata.get("unit"), (name, field.name)


def rows_of(run, **filters: object) -> list[dict]:  # type: ignore[no-untyped-def]
    return [
        row for row in run.rows("literal_entries") if all(row[k] == v for k, v in filters.items())
    ]


def test_configuration_parameter_data_keyed_by_scope_parameter_and_index(tmp_path: Path) -> None:
    run = run_reader(idaes, tmp_path, {DEMO: CONFIGURATION})
    rows = rows_of(run, parameter_scope="components/alpha")
    by_key = {(r["parameter"], r["parameter_index"]): r for r in rows}
    mw, unit = by_key[("mw", "[0]")], by_key[("mw", "[1]")]
    assert (mw["kind"], mw["number"], mw["text"]) == ("number", 10.5e-3, "10.5e-3")
    assert (unit["kind"], unit["number"], unit["text"]) == ("expression", None, "pyunits.kg / pyunits.mol")
    assert by_key[("omega", None)]["number"] == 0.25
    assert by_key[("coeff", "A/[0]")]["number"] == -1.5
    folded = by_key[("coeff", "B/[0]")]
    assert folded["number"] == 6.0 and folded["text"] == "2 * 3.0"  # arithmetic on literals folds
    none_unit = by_key[("coeff", "B/[1]")]
    assert none_unit["kind"] == "none" and none_unit["number"] is None  # a declared absent unit
    assert by_key[("flag", None)]["kind"] == "boolean" and by_key[("flag", None)]["number"] is None
    assert mw["target"] == "configuration"
    assert mw["path"] == ["components", "alpha", "parameter_data", "mw", "[0]"]
    assert mw["scope_kind"] == "module" and mw["scope"] == ""
    assert mw["_locator"] == f"{DEMO}#L{mw['line']}:{mw['column']}"

    package = {
        (r["parameter"], r["parameter_index"]): r for r in rows_of(run, parameter_scope="")
    }
    assert package[("kappa", "('alpha', 'alpha')")]["number"] == 0.0  # zero stays a number
    assert package[("kappa", "('alpha', 'beta')")]["number"] == -0.125

    structure = {
        r["path_text"]: r
        for r in rows_of(run)
        if r["parameter"] is None and r["path"][0] == "components"
    }
    assert structure["components/alpha/type"]["kind"] == "name"
    assert structure["components/alpha/pressure_sat_comp"]["text"] == "RPP4"
    key = "components/alpha/phase_equilibrium_form/('Vap', 'Liq')"
    assert structure[key]["text"] == "log_fugacity"
    bounds = rows_of(run, path_text="state_bounds/pressure/[3]")
    assert bounds[0]["text"] == "pyunits.Pa" and bounds[0]["parameter"] is None
    (record,) = run.result.payload
    assert record.status == "read"


def test_old_style_declarations_and_local_tables(tmp_path: Path) -> None:
    module = f"{PACKAGE}/activity_coeff_models/demo.py"
    run = run_reader(idaes, tmp_path, {module: OLD_STYLE})
    entries = run.rows("literal_entries")
    critical = {r["path_text"]: r for r in entries if r["target"] == "critical_data"}
    assert critical["alpha"]["number"] == 4.5e5 and critical["alpha"]["text"] == "4.5e5"
    assert critical["beta"]["number"] == 3.0 and critical["beta"]["text"] == "3"
    assert {r["scope"] for r in entries} == {"DemoParameterData.build"}
    elements = {r["path_text"]: r for r in entries if r["target"] == "self.element_comp"}
    assert elements["alpha/C"]["number"] == 1.0 and elements["alpha/H"]["number"] == 4.0
    assert elements["beta"]["kind"] == "empty" and elements["beta"]["text"] == "{}"
    assert any(r["target"] == "ratio" and r["number"] == 1.5 for r in entries)  # scalar in a data module
    assert any(r["target"] == "self.stoich['R1', 'Liq', 'A']" and r["number"] == -1.0 for r in entries)
    lookup = {r["path_text"]: r for r in entries if r["target"] == "lookup"}
    assert lookup["x"]["kind"] == "expression" and lookup["y"]["number"] == 2.0
    assert not [r for r in entries if r["target"] == "ratio" and r["kind"] != "number"]

    declarations = {d["name"]: d for d in run.rows("parameter_declarations")}
    reference = declarations["pressure_reference"]
    assert reference["default_number"] == 101325.0 and reference["units_text"] == "pyunits.Pa"
    assert reference["mutable_text"] == "True" and reference["index_sets"] == []
    critical_declaration = declarations["pressure_critical"]
    assert critical_declaration["initialize_name"] == "critical_data"  # joins to the literal rows
    assert critical_declaration["index_sets"] == ["self.component_list"]
    assert critical_declaration["within_text"] == "NonNegativeReals"
    assert declarations["factor"]["constructor"] == "Var"
    assert declarations["factor"]["initialize_text"] == "2.5"
    assert "doc" not in idaes.TABLES["parameter_declarations"].names  # text keywords are not read
    assert all("doc" not in str(v) for d in declarations.values() for v in d.values())


def test_algorithmic_modules_yield_containers_only(tmp_path: Path) -> None:
    module = f"{PACKAGE}/modular_properties/base/demo.py"
    run = run_reader(idaes, tmp_path, {module: ALGORITHMIC})
    entries = run.rows("literal_entries")
    assert {(r["target"], r["path_text"]) for r in entries} == {
        ("table", "a/[0]"),
        ("table", "a/[1]"),
        ("table", "b"),
        ("local_table", "k"),
    }  # no TOL, no enumeration codes, no n_cons
    assert {r["scope_kind"] for r in entries} == {"class", "function"}
    assert run.rows("parameter_declarations") == []


def test_test_modules_are_skipped_and_modules_without_data_are_read(tmp_path: Path) -> None:
    run = run_reader(
        idaes,
        tmp_path,
        {
            f"{PACKAGE}/modular_properties/base/tests/test_demo.py": "configuration = {'a': 1}\n",
            f"{PACKAGE}/modular_properties/base/plain.py": "x = compute(1)\n",
        },
    )
    accounts = {r.path: r for r in run.result.payload}
    skipped = accounts[f"{PACKAGE}/modular_properties/base/tests/test_demo.py"]
    assert skipped.status == "skipped" and "fixtures" in (skipped.reason or "")
    assert accounts[f"{PACKAGE}/modular_properties/base/plain.py"].status == "read"
    assert run.rows("literal_entries") == []


def test_a_syntax_error_is_refused_with_its_line(tmp_path: Path) -> None:
    with pytest.raises(StagingError, match=r"demo\.py#L2: cannot be parsed as Python"):
        run_reader(idaes, tmp_path, {DEMO: "a = 1\nb = = 2\n"})


JSON_FILE = f"{PACKAGE}/general_helmholtz/components/parameters/fluidx.json"


def json_rows_of(run) -> dict[str, dict]:  # type: ignore[no-untyped-def]
    return {row["pointer"]: row for row in run.rows("json_entries")}


def test_json_file_is_read_leaf_by_leaf_in_its_own_keys(tmp_path: Path) -> None:
    run = run_reader(idaes, tmp_path, {JSON_FILE: PARAMETER_FILE})
    rows = run.rows("json_entries")
    by_pointer = json_rows_of(run)
    assert len(by_pointer) == len(rows) == 19  # pointers are unique; leaves counted by hand

    r_value = by_pointer["/basic/R"]
    assert (r_value["kind"], r_value["number"], r_value["integer"], r_value["text"]) == (
        "number",
        0.25,
        False,
        "0.25",
    )
    assert r_value["path"] == ["basic", "R"] and r_value["path_steps"] == ["key", "key"]
    assert r_value["key"] == "R" and r_value["position"] is None
    assert r_value["_artifact"] == JSON_FILE
    assert r_value["_locator"] == f"{JSON_FILE}#/basic/R"
    # numbers keep their own spelling and kind
    assert by_pointer["/basic/MW"]["integer"] is True and by_pointer["/basic/MW"]["number"] == 40.0
    big = by_pointer["/basic/P_max"]
    assert (big["text"], big["number"], big["integer"]) == ("6.5e7", 6.5e7, False)
    small = by_pointer["/basic/rho_v"]
    assert (small["text"], small["number"]) == ("0.0000125", 1.25e-5)
    assert by_pointer["/basic/T_min"]["number"] == -10.0
    assert by_pointer["/eos/n/2"]["number"] == -1.25 and by_pointer["/eos/n/10"]["text"] == "3"
    # strings are the strings themselves; an array element has a position and no key
    line = by_pointer["/eos/reference/1"]
    assert (line["kind"], line["text"], line["position"], line["key"]) == (
        "string",
        "Second invented line",
        1,
        None,
    )
    assert line["path"] == ["eos", "reference", "1"] and line["path_steps"] == ["key", "key", "index"]
    # a key that looks like a number is a key; slash and tilde are escaped in the pointer only
    assert by_pointer["/eos/n/1"]["path_steps"] == ["key", "key", "key"]
    nested = by_pointer["/eos/a~1b/t~0c/2/0"]
    assert nested["path"] == ["eos", "a/b", "t~c", "2", "0"]
    assert nested["path_steps"] == ["key", "key", "key", "index", "index"]
    assert (nested["number"], nested["integer"]) == (3.0, True)
    assert by_pointer["/eos/a~1b/t~0c/1"]["integer"] is False  # 2.0 is a real literal
    # a key that looks like one of the literal reader's positions stays a key
    bracket = by_pointer["/eos/[0]"]
    assert (bracket["kind"], bracket["flag"], bracket["text"], bracket["key"]) == (
        "boolean",
        True,
        "true",
        "[0]",
    )
    assert by_pointer["/have_visc"]["flag"] is False and by_pointer["/have_visc"]["number"] is None
    assert by_pointer["/eos/absent"]["kind"] == "null" and by_pointer["/eos/absent"]["text"] == "null"
    assert by_pointer["/eos/none_yet"]["kind"] == "empty_object"
    assert by_pointer["/eos/no_items"]["kind"] == "empty_array"
    # file order is kept
    assert [row["pointer"] for row in rows][:3] == ["/comp", "/basic/R", "/basic/MW"]
    assert run.rows("literal_entries") == [] and run.rows("parameter_declarations") == []
    (record,) = run.result.payload
    assert record.path == JSON_FILE and record.status == "read"


def test_json_root_scalar_and_test_directories(tmp_path: Path) -> None:
    run = run_reader(
        idaes,
        tmp_path,
        {
            f"{PACKAGE}/scalar.json": ROOT_SCALAR,
            f"{PACKAGE}/tests/fixture.json": '{"a": 1}',
        },
    )
    (row,) = run.rows("json_entries")
    assert (row["pointer"], row["path"], row["key"], row["position"]) == ("", [], None, None)
    assert row["_locator"] == f"{PACKAGE}/scalar.json#" and row["number"] == 12.5
    assert row["text"] == "12.50"
    accounts = {r.path: r for r in run.result.payload}
    assert accounts[f"{PACKAGE}/tests/fixture.json"].status == "skipped"


@pytest.mark.parametrize(
    ("content", "message"),
    [
        ('{"a": 1,\n "b": }', r"fluidx\.json#L2: cannot be parsed as JSON"),
        ('{"a": 1, "a": 2}', r"fluidx\.json: the object has the key 'a' more than once"),
        ('{"a": NaN}', r"fluidx\.json: the constant NaN is not a JSON number"),
        ('{"a": 1e999}', r"fluidx\.json#/a: the number 1e999 is not finite"),
    ],
)
def test_malformed_json_is_refused(tmp_path: Path, content: str, message: str) -> None:
    with pytest.raises(StagingError, match=message):
        run_reader(idaes, tmp_path, {JSON_FILE: content})


# -- the pinned checkout ---------------------------------------------------------------------------

real = pytest.mark.skipif(
    not (TREE / PACKAGE).is_dir(),
    reason=f"the IDAES checkout {TREE} is absent (scripts/fetch-external.sh)",
)


@pytest.fixture(scope="module")
def staged(tmp_path_factory: pytest.TempPathFactory) -> Path:
    return stage_real("idaes", tmp_path_factory.mktemp("staged"))


def payload_modules() -> list[str]:
    return sorted(p.relative_to(TREE).as_posix() for p in (TREE / PACKAGE).rglob("*.py"))


def payload_json_files() -> list[str]:
    return sorted(p.relative_to(TREE).as_posix() for p in (TREE / PACKAGE).rglob("*.json"))


def non_test_modules() -> list[str]:
    return [m for m in payload_modules() if not idaes.is_test(m)]


def leaf_count(node: ast.expr) -> int:
    """Leaves of a literal container, by recursion on the node kinds (an empty container is one)."""
    if isinstance(node, ast.Dict):
        return sum(leaf_count(v) for v in node.values) or 1
    if isinstance(node, ast.List | ast.Tuple):
        return sum(leaf_count(e) for e in node.elts) or 1
    return 1


def foldable(node: ast.expr) -> bool:
    if isinstance(node, ast.Constant):
        return isinstance(node.value, int | float) and not isinstance(node.value, bool)
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub | ast.UAdd):
        return foldable(node.operand)
    if isinstance(node, ast.BinOp) and isinstance(
        node.op, ast.Add | ast.Sub | ast.Mult | ast.Div | ast.Pow | ast.FloorDiv | ast.Mod
    ):
        return foldable(node.left) and foldable(node.right)
    return False


def has_number_leaf(node: ast.expr) -> bool:
    if isinstance(node, ast.Dict):
        return any(has_number_leaf(v) for v in node.values)
    if isinstance(node, ast.List | ast.Tuple):
        return any(has_number_leaf(e) for e in node.elts)
    return foldable(node)


def independent_leaf_counts() -> Counter:
    """Leaves per module over every assignment `ast.walk` finds, with the reader's two
    eligibility rules restated."""
    counts: Counter = Counter()
    for module in non_test_modules():
        tree = ast.parse((TREE / module).read_text(encoding="utf-8"))
        data = module.startswith(idaes.literals.DATA_MODULES)
        for node in ast.walk(tree):
            if isinstance(node, ast.Assign | ast.AnnAssign) and node.value is not None:
                value = node.value
                if isinstance(value, ast.Dict | ast.List | ast.Tuple):
                    if has_number_leaf(value):
                        counts[module] += leaf_count(value)
                elif data and foldable(value):
                    counts[module] += 1
    return counts


@real
def test_payload_is_accounted_for(staged: Path) -> None:
    manifest = staged_manifest_of(staged)
    assert set(manifest.tables) == set(idaes.TABLES)
    accounts = {r.path: r for r in manifest.payload}
    assert set(accounts) == set(payload_modules()) | set(payload_json_files())
    assert len(payload_json_files()) == 22
    assert len(accounts) == 211 + 22
    skipped = [p for p, r in accounts.items() if r.status == "skipped"]
    assert skipped and all(idaes.is_test(p) for p in skipped)
    assert len(skipped) == sum(idaes.is_test(p) for p in accounts) == 102
    assert all(r.status == "read" for p, r in accounts.items() if not idaes.is_test(p))


@real
def test_leaf_counts_match_an_independent_walk(staged: Path) -> None:
    expected = independent_leaf_counts()
    staged_counts = Counter(staged_table(staged, "literal_entries").column("_artifact").to_pylist())
    assert staged_counts == expected
    assert sum(expected.values()) == staged_table(staged, "literal_entries").num_rows
    declared = 0
    for module in non_test_modules():
        if not module.startswith(idaes.literals.DATA_MODULES):
            continue
        for node in ast.walk(ast.parse((TREE / module).read_text(encoding="utf-8"))):
            if (
                isinstance(node, ast.Assign)
                and isinstance(node.value, ast.Call)
                and isinstance(node.value.func, ast.Name)
                and node.value.func.id in ("Param", "Var")
            ):
                declared += 1
    assert staged_table(staged, "parameter_declarations").num_rows == declared


def json_leaf_count(node: object) -> int:
    """Leaves of a `json.load` document by recursion on the value types (an empty container
    is one)."""
    if isinstance(node, dict):
        return sum(json_leaf_count(v) for v in node.values()) or 1
    if isinstance(node, list):
        return sum(json_leaf_count(v) for v in node) or 1
    return 1


def json_number_count(node: object) -> int:
    if isinstance(node, dict):
        return sum(json_number_count(v) for v in node.values())
    if isinstance(node, list):
        return sum(json_number_count(v) for v in node)
    return int(isinstance(node, int | float) and not isinstance(node, bool))


@real
def test_json_leaf_counts_match_an_independent_walk(staged: Path) -> None:
    documents = {
        name: json.loads((TREE / name).read_text(encoding="utf-8")) for name in payload_json_files()
    }
    expected = Counter({name: json_leaf_count(document) for name, document in documents.items()})
    table = staged_table(staged, "json_entries")
    assert Counter(table.column("_artifact").to_pylist()) == expected
    assert table.num_rows == sum(expected.values())
    assert len(set(table.column("_locator").to_pylist())) == table.num_rows
    rows = table.to_pylist()
    assert sum(r["kind"] == "number" for r in rows) == sum(
        json_number_count(d) for d in documents.values()
    )
    assert {r["kind"] for r in rows} <= set(idaes.jsonfiles.KINDS)
    assert len({r["_artifact"].rsplit("/", 1)[-1] for r in rows}) == 22


def rebuild(rows: list[dict]) -> object:
    """A file's document from its staged rows alone: containers from `path_steps`, leaves from
    `kind`, `number`, `integer`, `flag` and `text`."""
    root: object = None
    for row in rows:
        if row["kind"] == "number":
            leaf: object = int(row["number"]) if row["integer"] else row["number"]
        elif row["kind"] == "boolean":
            leaf = row["flag"]
        elif row["kind"] == "null":
            leaf = None
        elif row["kind"] == "string":
            leaf = row["text"]
        else:
            leaf = {} if row["kind"] == "empty_object" else []
        steps = list(zip(row["path"], row["path_steps"], strict=True))
        if not steps:
            return leaf
        if root is None:
            root = {} if steps[0][1] == "key" else []
        node = root
        for (step, kind), (_, next_kind) in zip(steps, [*steps[1:], ("", "leaf")], strict=True):
            last = next_kind == "leaf"
            child = leaf if last else ({} if next_kind == "key" else [])
            if kind == "key":
                assert isinstance(node, dict)
                if last:
                    assert step not in node
                    node[step] = child
                else:
                    node = node.setdefault(step, child)
            else:
                assert isinstance(node, list)
                assert int(step) <= len(node)
                if last:
                    assert int(step) == len(node)
                    node.append(child)
                elif int(step) == len(node):
                    node.append(child)
                    node = child
                else:
                    node = node[int(step)]
    return root


@real
def test_staged_rows_rebuild_each_files_parameter_values(staged: Path) -> None:
    by_file: dict[str, list[dict]] = {}
    for row in staged_table(staged, "json_entries").to_pylist():
        by_file.setdefault(row["_artifact"], []).append(row)
    assert set(by_file) == set(payload_json_files())
    for name, rows in by_file.items():
        loaded = json.loads((TREE / name).read_text(encoding="utf-8"))
        rebuilt = rebuild(rows)
        # dumps compares values, key order, and integer against real literals
        assert json.dumps(rebuilt) == json.dumps(loaded), name
        for row in rows:
            assert row["_locator"] == f"{name}#{row['pointer']}"
            if row["kind"] == "number":
                assert float(row["text"]) == row["number"]
    example = by_file[f"{PACKAGE}/general_helmholtz/components/parameters/h2o.json"]
    n_values = [r["number"] for r in example if r["path"][:2] == ["eos", "n"]]
    assert len(n_values) == 56  # the survey states 56 n values in the water file


@real
def test_example_configurations_are_found_by_name_and_count(staged: Path) -> None:
    rows = staged_table(staged, "literal_entries").to_pylist()
    configurations = {
        (r["_artifact"].rsplit("/", 1)[-1], r["target"])
        for r in rows
        if r["_artifact"].startswith(EXAMPLES) and "/" not in r["target"] and r["parameter"]
    }
    files = {name for name, _ in configurations}
    assert files == {
        "ASU_PR.py",
        "BT_PR.py",
        "BT_ideal.py",
        "CO2_H2O_Ideal_VLE.py",
        "CO2_bmimPF6_PR.py",
        "HC_PR.py",
        "HC_PR_vap.py",
        "enrtl_H2O_NaCl_KCl.py",
        "enrtl_NaBr_mixed_solvent.py",
        "reaction_example.py",
    }
    assert sum(1 for name, _ in configurations if name != "reaction_example.py") == 10
    texts = {
        (r["_artifact"].rsplit("/", 1)[-1], r["parameter_scope"], r["parameter"], r["parameter_index"]): r
        for r in rows
        if r["parameter"]
    }
    # the benzene-toluene cubic example states benzene's critical pressure with its unit
    value = texts[("BT_PR.py", "components/benzene", "pressure_crit", "[0]")]
    unit = texts[("BT_PR.py", "components/benzene", "pressure_crit", "[1]")]
    assert value["number"] == 48.9e5 and value["text"] == "48.9e5"
    assert unit["text"] == "pyunits.Pa"


class Stand:
    """A name the evaluator does not know: it renders as the dotted expression that built it."""

    def __init__(self, text: str) -> None:
        self.text = text

    def __getattr__(self, name: str) -> Stand:
        return Stand(f"{self.text}.{name}")

    def __call__(self, *args: object) -> Stand:
        return Stand(f"{self.text}(...)")

    def __repr__(self) -> str:
        return self.text

    def __hash__(self) -> int:
        return hash(self.text)

    def __eq__(self, other: object) -> bool:
        return isinstance(other, Stand) and other.text == self.text

    def _op(self, other: object, symbol: str) -> Stand:
        return Stand(f"{self.text} {symbol} {other!r}")

    def __truediv__(self, other: object) -> Stand:
        return self._op(other, "/")

    def __mul__(self, other: object) -> Stand:
        return self._op(other, "*")

    def __pow__(self, other: object) -> Stand:
        return self._op(other, "**")

    def __rtruediv__(self, other: object) -> Stand:
        return Stand(f"{other!r} / {self.text}")

    def __rmul__(self, other: object) -> Stand:
        return Stand(f"{other!r} * {self.text}")


class Names(dict):
    def __missing__(self, key: str) -> Stand:
        return Stand(key)


def flatten(value: object, path: tuple[str, ...], out: dict[tuple[str, ...], object]) -> None:
    if isinstance(value, dict | list | tuple) and not value:
        out[path] = value
    elif isinstance(value, dict):
        for key, inner in value.items():
            flatten(inner, (*path, key if isinstance(key, str) else repr(key)), out)
    elif isinstance(value, list | tuple):
        for index, inner in enumerate(value):
            flatten(inner, (*path, f"[{index}]"), out)
    else:
        out[path] = value


@real
def test_configuration_numbers_equal_python_evaluation(staged: Path) -> None:
    """Evaluate each example configuration dictionary with Python's own evaluator (names stand
    in for classes and units) and compare every number with the staged row at the same path."""
    rows = staged_table(staged, "literal_entries").to_pylist()
    by_assignment: dict[tuple[str, int], dict[str, dict]] = {}
    for row in rows:
        by_assignment.setdefault((row["_artifact"], row["assignment_line"]), {})[row["path_text"]] = row
    checked = 0
    for module in (m for m in non_test_modules() if m.startswith(EXAMPLES)):
        source = (TREE / module).read_text(encoding="utf-8")
        for node in ast.parse(source).body:
            if not (
                isinstance(node, ast.Assign)
                and isinstance(node.value, ast.Dict)
                and (module, node.lineno) in by_assignment
            ):
                continue
            expression = ast.Expression(node.value)
            ast.fix_missing_locations(expression)
            evaluated = eval(  # noqa: S307 - a literal structure of constants and names only
                compile(expression, module, "eval"), {"__builtins__": {}}, Names()
            )
            flat: dict[tuple[str, ...], object] = {}
            flatten(evaluated, (), flat)
            staged_rows = by_assignment[(module, node.lineno)]
            assert len(flat) == len(staged_rows), (module, node.lineno)
            for path, value in flat.items():
                row = staged_rows["/".join(path)]
                if isinstance(value, dict | list | tuple):
                    assert row["kind"] == "empty"
                elif isinstance(value, bool):
                    assert row["kind"] == "boolean"
                elif isinstance(value, int | float):
                    assert row["kind"] == "number" and row["number"] == float(value)
                    checked += 1
                elif isinstance(value, str):
                    assert row["kind"] == "string" and row["text"] == value
                elif value is None:
                    assert row["kind"] == "none"
                else:
                    assert row["kind"] in ("name", "expression")
    staged_numbers = sum(
        row["kind"] == "number"
        for (module, _), assignment in by_assignment.items()
        if module.startswith(EXAMPLES)
        for row in assignment.values()
    )
    assert checked == staged_numbers > 1000


@real
def test_no_docstring_or_comment_text_is_carried(staged: Path) -> None:
    """Clean room: no staged text of the Python modules equals or contains a docstring or a
    comment of the source. The JSON files hold no docstring or comment; their strings are the
    files' own values (the literature references also appear in the prose of the fluid modules,
    so they are checked by the round trip to the parsed JSON, not here)."""
    protected: set[str] = set()
    for module in non_test_modules():
        source = (TREE / module).read_text(encoding="utf-8")
        for node in ast.walk(ast.parse(source)):
            if isinstance(node, ast.Module | ast.ClassDef | ast.FunctionDef | ast.AsyncFunctionDef):
                doc = ast.get_docstring(node)
                if doc and len(doc) >= 12:
                    protected.add(doc)
        for token in tokenize.generate_tokens(io.StringIO(source).readline):
            if token.type == tokenize.COMMENT:
                comment = token.string.lstrip("# ").strip()
                if len(comment) >= 12 and " " in comment:  # prose, not a restated identifier
                    protected.add(comment)
    assert len(protected) > 500
    staged_strings: set[str] = set()
    for name in idaes.TABLES:
        if name in idaes.jsonfiles.SCHEMAS:
            continue
        table = staged_table(staged, name)
        for field in table.schema:
            if pa.types.is_string(field.type) and field.name not in ("_artifact", "_locator"):
                staged_strings.update(v for v in table.column(field.name).to_pylist() if v)
            elif pa.types.is_list(field.type):
                for values in table.column(field.name).to_pylist():
                    staged_strings.update(v for v in (values or []) if v)
    leaked = [s for s in staged_strings for p in protected if p in s]
    assert not leaked, leaked[:5]
