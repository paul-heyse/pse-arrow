# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The pipeline's shape contract (meta-model section 5): the loader refuses a declaration that
lacks what the stage code names, and the code takes its names from one module that reads the
contract file."""

from __future__ import annotations

import ast
import shutil
from pathlib import Path

import pytest

from mapping_support import FORMS, MODEL, real_declaration
from thermo_knowledge import config, pipeline_contract
from thermo_knowledge.declaration import contract_check, load_declaration
from thermo_knowledge.pipeline_contract import (
    PACKAGED,
    RESERVED,
    ContractError,
    Entry,
    EnumEntry,
)

SRC = config.TREE_DIR / "src" / "thermo_knowledge"
CONTRACT_MODULE = SRC / "pipeline_contract.py"


def copy_real(tmp_path: Path) -> Path:
    shutil.copytree(MODEL, tmp_path / "model")
    shutil.copytree(FORMS, tmp_path / "forms")
    return tmp_path


def edited(tmp_path: Path, module: str, old: str, new: str) -> list:  # type: ignore[type-arg]
    tree = copy_real(tmp_path)
    path = tree / "model" / f"{module}.toml"
    text = path.read_text()
    assert text.count(old) == 1, old
    path.write_text(text.replace(old, new))
    return list(load_declaration(tree / "model", tree / "forms").diagnostics)


# -- the loader ---------------------------------------------------------------------------------


def test_the_committed_declaration_satisfies_the_contract() -> None:
    assert contract_check.check(real_declaration(), PACKAGED) == []


def test_renaming_carrier_tree_hash_is_refused_at_load_naming_the_contract_entry(
    tmp_path: Path,
) -> None:
    found = edited(
        tmp_path,
        "provenance",
        'tree_hash = { type = "Hash"',
        'tree_digest = { type = "Hash"',
    )
    (diagnostic,) = found
    assert diagnostic.code == "pipeline-contract"
    assert diagnostic.document == "model/provenance.toml"
    assert diagnostic.construct == "kinds.carrier.attributes"
    assert "kind `carrier` has no attribute `tree_hash` (Hash)" in diagnostic.message
    assert "kinds.carrier.attributes.tree_hash" in diagnostic.message
    assert "required by the pipeline contract" in diagnostic.message


@pytest.mark.parametrize(
    ("module", "old", "new", "construct", "message"),
    [
        (
            "provenance",
            'retrieved = { type = "Timestamp"',
            'retrieved = { type = "Text"',
            "kinds.carrier.attributes.retrieved.type",
            "attribute `retrieved` is Text, not Timestamp",
        ),
        (
            "provenance",
            'tree_hash = { type = "Hash"',
            'tree_hash = { type = "Hash", optional = true',
            "kinds.carrier.attributes.tree_hash",
            "attribute `tree_hash` is optional, and the code needs it required",
        ),
        (
            "provenance",
            'reader = { type = "Text", doc = "The reader that produced the source-faithful row." }',
            'reader = { type = "Text", optional = true, doc = "The reader that produced the row." }',
            "kinds.import_record.attributes.reader",
            "attribute `reader` is optional, and the code needs it required",
        ),
        (
            "identity",
            'provisional = { type = "Boolean", default = false,',
            'provisional = { type = "Boolean",',
            "kinds.material_entity.attributes.provisional",
            "has no default, and the code relies on a declared default",
        ),
        (
            "provenance",
            'role = { type = "origin_role", doc = "How the carrier presents the content." }',
            'role = { type = "permission", doc = "How the carrier presents the content." }',
            "relations.record_origin.columns.role.type",
            "value column `role` is permission, not origin_role",
        ),
        (
            "provenance",
            'import_record = { type = "import_record", doc = "The import record it came from." }',
            'import_row = { type = "import_record", doc = "The import record it came from." }',
            "relations.record_origin.keys",
            "relation `record_origin` has no key `import_record` (import_record)",
        ),
        (
            "qualification",
            'members.held = { doc = "Not loaded because its subject is ambiguous or it failed '
            'validation; the reason is recorded." }\n',
            "",
            "enums.row_state.members",
            "enum `row_state` has no member `held`",
        ),
    ],
)
def test_each_kind_of_mismatch_is_a_located_diagnostic(
    tmp_path: Path, module: str, old: str, new: str, construct: str, message: str
) -> None:
    found = edited(tmp_path, module, old, new)
    matching = [d for d in found if message in d.message]
    assert matching, [str(d) for d in found]
    assert {d.code for d in found} == {"pipeline-contract"}
    # an attribute several listed kinds inherit is reported once for each of them
    for diagnostic in matching:
        assert diagnostic.document == f"model/{module}.toml"
    assert any(d.construct.startswith(construct.removesuffix(".type")) for d in matching)


def test_a_missing_kind_is_reported_at_the_contract_file(tmp_path: Path) -> None:
    tree = copy_real(tmp_path)
    path = tree / "model" / "qualification.toml"
    text = path.read_text()
    start = text.index("[kinds.held_row]")
    path.write_text(text[:start])
    found = load_declaration(tree / "model", tree / "forms").diagnostics
    (diagnostic,) = [d for d in found if "kind `held_row`" in d.message]
    assert diagnostic.code == "pipeline-contract"
    assert diagnostic.document == pipeline_contract.CONTRACT_DOCUMENT
    assert diagnostic.construct == "kinds.held_row"
    assert "the declaration has no kind `held_row`" in diagnostic.message


def test_a_declaration_that_is_not_this_pipelines_model_loads_without_the_contract() -> None:
    from declaration_support import FULL

    assert load_declaration(FULL / "model", FULL / "forms", contract=None).diagnostics == ()
    refused = load_declaration(FULL / "model", FULL / "forms")
    assert refused.declaration is None
    assert {d.code for d in refused.diagnostics} == {"pipeline-contract"}


# -- the accessors ------------------------------------------------------------------------------


def test_a_name_the_contract_does_not_list_is_an_error() -> None:
    with pytest.raises(ContractError, match="lists no `tree_hashh` for kind `carrier`"):
        _ = pipeline_contract.CARRIER.tree_hashh
    with pytest.raises(ContractError, match="lists no kind `nothing`"):
        pipeline_contract.kind("nothing")
    with pytest.raises(ContractError, match="lists no relation `nothing`"):
        pipeline_contract.relation("nothing")
    with pytest.raises(ContractError, match="lists no enum `nothing`"):
        pipeline_contract.enum("nothing")
    with pytest.raises(ContractError, match="lists no member `nothing` for enum `row_state`"):
        pipeline_contract.ROW_STATE.member("nothing")
    with pytest.raises(AttributeError):  # a ContractError is an AttributeError
        _ = pipeline_contract.CARRIER.nothing


def test_an_entry_answers_with_the_names_it_lists_and_those_of_the_kind_it_extends() -> None:
    carrier = pipeline_contract.CARRIER
    assert carrier.declared == "carrier" and carrier.table == "prov.carrier"
    assert carrier.tree_hash == "tree_hash"
    assert carrier.key == "key"  # an attribute of `source`, which `carrier` extends
    assert carrier.qualified("manifest_id") == "prov.carrier.manifest_id"
    assert pipeline_contract.RECORD_ORIGIN.role == "role"
    assert pipeline_contract.RECORD_ORIGIN.table == "prov.record_origin"


def test_the_constants_of_the_module_are_exactly_the_entries_of_the_file() -> None:
    constants = {
        name: value
        for name, value in vars(pipeline_contract).items()
        if isinstance(value, Entry | EnumEntry)
    }
    by_declared = {
        (type(v).__name__, getattr(v, "category", "enum"), v.declared) for v in constants.values()
    }
    listed = (
        {("Entry", "kind", name) for name in PACKAGED.kinds}
        | {("Entry", "relation", name) for name in PACKAGED.relations}
        | {("EnumEntry", "enum", name) for name in PACKAGED.enums}
    )
    # every entry of the file is exposed, and everything the module exposes is in the file
    assert by_declared == listed
    for name, value in constants.items():
        assert name == value.declared.upper()


def test_a_listed_column_does_not_take_a_name_an_entry_answers_itself() -> None:
    for entry in (*PACKAGED.kinds.values(), *PACKAGED.relations.values()):
        assert not set(entry.columns) & RESERVED, entry


def test_the_contract_file_is_well_formed() -> None:
    with pytest.raises(ContractError, match="unknown section"):
        pipeline_contract.parse("[tables.x]\n")
    with pytest.raises(ContractError, match="states its `schema`"):
        pipeline_contract.parse("[kinds.x]\n")
    with pytest.raises(ContractError, match="states its `type`"):
        pipeline_contract.parse('[kinds.x]\nschema = "tk"\nattributes.a = {}\n')
    with pytest.raises(ContractError, match="reserved by the contract module"):
        pipeline_contract.parse('[kinds.x]\nschema = "tk"\nattributes.table = { type = "Text" }\n')
    with pytest.raises(ContractError, match="extends `y`, which the contract does not list"):
        pipeline_contract.parse('[kinds.x]\nschema = "tk"\nextends = "y"\n')


# -- the code reads its names from the contract -----------------------------------------------

ENTRY_SELF = {"declared", "table", "schema", "identity", "extends", "category", "columns"}
ENTRY_METHODS = {"listed", "qualified", "member"}


def constants() -> dict[str, Entry | EnumEntry]:
    return {
        name: value
        for name, value in vars(pipeline_contract).items()
        if isinstance(value, Entry | EnumEntry)
    }


def _is_contract_constant(node: ast.expr) -> str | None:
    """`CONSTANT` when `node` is `pc.CONSTANT`."""
    if (
        isinstance(node, ast.Attribute)
        and isinstance(node.value, ast.Name)
        and node.value.id == "pc"
        and node.attr in constants()
    ):
        return node.attr
    return None


def _scope_nodes(scope: ast.AST) -> list[ast.AST]:
    """The nodes of a scope, not those of the functions nested in it."""
    found: list[ast.AST] = []
    pending = list(ast.iter_child_nodes(scope))
    while pending:
        node = pending.pop()
        found.append(node)
        if not isinstance(node, ast.FunctionDef | ast.AsyncFunctionDef | ast.Lambda):
            pending.extend(ast.iter_child_nodes(node))
    return found


def _alias_assignments(nodes: list[ast.AST]) -> tuple[dict[str, str], set[str]]:
    """The names a scope binds to a contract constant, and the names it binds to anything else."""
    aliases: dict[str, str] = {}
    rebound: set[str] = set()
    for node in nodes:
        if isinstance(node, ast.Assign):
            for target in node.targets:
                pairs = (
                    zip(target.elts, node.value.elts, strict=True)
                    if isinstance(target, ast.Tuple)
                    and isinstance(node.value, ast.Tuple)
                    and len(target.elts) == len(node.value.elts)
                    else [(target, node.value)]
                )
                for one, value in pairs:
                    if isinstance(one, ast.Name):
                        constant = _is_contract_constant(value)
                        if constant is not None:
                            aliases[one.id] = constant
                        else:
                            rebound.add(one.id)
        elif isinstance(node, ast.For | ast.comprehension):
            for name in ast.walk(node.target):
                if isinstance(name, ast.Name):
                    rebound.add(name.id)
        elif isinstance(node, ast.arg):
            rebound.add(node.arg)
    return aliases, rebound


def accessor_uses(path: Path) -> list[tuple[int, str, str]]:
    """Every `pc.CONSTANT.name` in the file, directly or through a name bound to the constant in
    the same scope or the module's (`ps = pc.PARAMETER_SET`), as (line, constant, name)."""
    tree = ast.parse(path.read_text())
    uses: list[tuple[int, str, str]] = []
    module_aliases, _ = _alias_assignments(_scope_nodes(tree))

    def visit(scope: ast.AST, inherited: dict[str, str]) -> None:
        nodes = _scope_nodes(scope)
        aliases, rebound = _alias_assignments(nodes)
        if isinstance(scope, ast.FunctionDef | ast.AsyncFunctionDef):
            arguments = {a.arg for a in ast.walk(scope.args) if isinstance(a, ast.arg)}
            rebound |= arguments
        known = {name: value for name, value in inherited.items() if name not in rebound}
        known.update(aliases)
        for node in nodes:
            if isinstance(node, ast.Attribute):
                direct = _is_contract_constant(node.value)
                if direct is not None:
                    uses.append((node.lineno, direct, node.attr))
                elif isinstance(node.value, ast.Name) and node.value.id in known:
                    uses.append((node.lineno, known[node.value.id], node.attr))
            elif isinstance(node, ast.FunctionDef | ast.AsyncFunctionDef):
                visit(node, known)
            elif isinstance(node, ast.ClassDef):
                visit(node, known)

    visit(tree, module_aliases)
    return uses


def test_every_name_the_code_takes_from_the_contract_is_listed_in_it() -> None:
    found = 0
    problems: list[str] = []
    for path in sorted(SRC.rglob("*.py")):
        if path == CONTRACT_MODULE:
            continue
        for line, name, attribute in accessor_uses(path):
            found += 1
            entry = constants()[name]
            if isinstance(entry, EnumEntry):
                if attribute not in {"declared", "members", "member"}:
                    problems.append(f"{path.name}:{line}: {name}.{attribute}")
            elif attribute not in entry.listed() and attribute not in ENTRY_SELF | ENTRY_METHODS:
                problems.append(f"{path.relative_to(SRC)}:{line}: {name}.{attribute}")
    assert found > 100, "the scan found too few uses: it no longer sees the code's accessors"
    assert problems == []


def test_every_member_the_code_names_is_listed_for_its_enum() -> None:
    known = constants()
    for path in sorted(SRC.rglob("*.py")):
        if path == CONTRACT_MODULE:
            continue
        for node in ast.walk(ast.parse(path.read_text())):
            if (
                isinstance(node, ast.Call)
                and isinstance(node.func, ast.Attribute)
                and node.func.attr == "member"
                and isinstance(node.func.value, ast.Attribute)
                and isinstance(node.func.value.value, ast.Name)
                and node.func.value.value.id == "pc"
            ):
                enum = known[node.func.value.attr]
                assert isinstance(enum, EnumEntry)
                (argument,) = node.args
                assert isinstance(argument, ast.Constant)
                assert argument.value in enum.members, f"{path.name}:{node.lineno}"


def test_the_stage_code_holds_no_literal_table_name_of_a_contract_entry() -> None:
    """A table the stages write or read is named through the contract, never as `tk.species`."""
    tables = {entry.table for entry in (*PACKAGED.kinds.values(), *PACKAGED.relations.values())}
    offenders: list[str] = []
    for package in ("canonical", "mapping", "resolve", "build", "qualify"):
        for path in sorted((SRC / package).rglob("*.py")):
            tree = ast.parse(path.read_text())
            docstrings = {
                id(node.body[0].value)
                for node in ast.walk(tree)
                if isinstance(node, ast.Module | ast.FunctionDef | ast.ClassDef)
                and node.body
                and isinstance(node.body[0], ast.Expr)
            }
            for node in ast.walk(tree):
                if (
                    isinstance(node, ast.Constant)
                    and isinstance(node.value, str)
                    and id(node) not in docstrings
                    and node.value in tables
                ):
                    offenders.append(f"{path.relative_to(SRC)}:{node.lineno}: {node.value}")
    assert offenders == []
