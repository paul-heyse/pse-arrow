# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The generator: deterministic bytes, `--check`, quoting and the shape of the DDL."""

from __future__ import annotations

import re
import shutil
import socket
from pathlib import Path

import pytest
from typer.testing import CliRunner

from declaration_support import copy_full, empty_declaration, full_declaration, overlay
from mapping_support import FORMS, MODEL
from thermo_knowledge import config
from thermo_knowledge.cli import app
from thermo_knowledge.declaration import DeclarationError, load_declaration
from thermo_knowledge.generate import (
    SCHEMA_PATH,
    Difference,
    compare_tree,
    declaration_fingerprint,
    generate,
    schema_fingerprint,
    write_tree,
)
from thermo_knowledge.generate.meta_rows import meta_rows
from thermo_knowledge.generate.meta_tables import META_TABLES
from thermo_knowledge.generate.plan import build_plan
from thermo_knowledge.generate.reified import insert_batches, serialise
from thermo_knowledge.generate.sqltext import MAX_IDENTIFIER_BYTES, ident, literal, qname

runner = CliRunner()


def schema_text() -> str:
    return generate(full_declaration())[SCHEMA_PATH].decode()


def make_tree(root: Path) -> Path:
    """A tree with the valid fixture's model and forms, as `tk generate --tree` reads it."""
    return copy_full(root)


# -- determinism and purity ---------------------------------------------------------------


def test_generation_is_byte_identical_across_runs_and_loads() -> None:
    first = generate(full_declaration())
    second = generate(full_declaration())
    assert first == second
    fresh = load_declaration(
        full_declaration_dir() / "model", full_declaration_dir() / "forms", contract=None
    ).require()
    assert generate(fresh) == first
    assert list(first) == [SCHEMA_PATH]
    assert all(isinstance(content, bytes) for content in first.values())


def full_declaration_dir() -> Path:
    from declaration_support import FULL

    return FULL


def test_generation_is_independent_of_file_layout(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    # move a module into a subdirectory and rename it: names decide order, not files
    (tree / "model" / "nested").mkdir()
    shutil.move(tree / "model" / "vocab.toml", tree / "model" / "nested" / "zzz_words.toml")
    moved = load_declaration(tree / "model", tree / "forms", contract=None).require()
    assert generate(moved) == generate(full_declaration())


def test_generator_reads_no_network_database_or_environment(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def refuse(*args: object, **kwargs: object) -> None:
        raise AssertionError("the generator must not open a socket")

    monkeypatch.setattr(socket.socket, "connect", refuse)
    monkeypatch.setenv(config.DATABASE_URL_ENV, "postgres:///does_not_exist?host=/nowhere")
    assert generate(full_declaration()) == generate(full_declaration())


def test_committed_schema_is_the_generation_of_the_committed_declaration() -> None:
    decl = load_declaration().require()
    assert compare_tree(generate(decl), config.TREE_DIR) == []


# -- shape of the DDL ---------------------------------------------------------------------


def test_statements_are_ordered_to_apply_to_an_empty_database() -> None:
    text = schema_text()
    assert text.index("CREATE SCHEMA") < text.index("CREATE EXTENSION") < text.index("CREATE TYPE")
    last_table = text.rindex("CREATE TABLE")
    assert last_table < text.index("ALTER TABLE")
    assert all(
        line.startswith("ALTER TABLE")
        for line in text[text.index("ALTER TABLE") :].splitlines()
        if line
    )
    assert text.index("CREATE DOMAIN") < text.index("CREATE TABLE")
    assert 'CREATE EXTENSION IF NOT EXISTS btree_gist WITH SCHEMA "meta"' in text


def test_extension_only_where_an_exclusion_constraint_needs_it(tmp_path: Path) -> None:
    real = generate(empty_declaration(tmp_path))[SCHEMA_PATH].decode()
    assert "btree_gist" not in real
    assert "EXCLUDE USING gist" not in real
    assert "btree_gist" in schema_text()


def test_empty_declaration_has_the_base_schemas_meta_tables_and_record(tmp_path: Path) -> None:
    real = generate(empty_declaration(tmp_path))[SCHEMA_PATH].decode()
    for schema in ("meta", "prov", "tk", "ev", "qual", "param"):
        assert f'CREATE SCHEMA "{schema}";' in real
    assert 'CREATE TABLE "prov"."record"' in real
    assert 'CREATE TYPE "meta"."float8range" AS RANGE' in real
    assert 'CREATE TABLE "meta"."kind"' in real
    assert real.count("CREATE TABLE") == len(META_TABLES) + 1


def test_every_table_and_column_carries_its_doc_and_every_constraint_is_named() -> None:
    plan = build_plan(full_declaration())
    for table in plan.tables:
        assert table.comment.strip(), table.name
        for column in table.columns:
            assert column.comment.strip(), f"{table.name}.{column.name}"
        assert {c.name for c in table.constraints}, table.name
    text = schema_text()
    tables = text.count("CREATE TABLE")
    columns = sum(len(t.columns) for t in plan.tables)
    assert text.count("COMMENT ON TABLE") == tables
    assert text.count("COMMENT ON COLUMN") == columns
    assert not re.search(r"\n    (?:PRIMARY KEY|UNIQUE|CHECK|FOREIGN KEY)", text)


def test_every_generated_identifier_fits_postgresql() -> None:
    for table in build_plan(full_declaration()).tables:
        names = [table.name, *(c.name for c in table.columns)]
        names += [c.name for c in table.constraints] + [f.name for f in table.foreign_keys]
        assert all(len(name.encode()) <= MAX_IDENTIFIER_BYTES for name in names), table.name


def test_foreign_keys_are_deferrable() -> None:
    text = schema_text()
    alters = [line for line in text.splitlines() if line.startswith("ALTER TABLE")]
    assert alters and all(line.endswith("DEFERRABLE INITIALLY DEFERRED;") for line in alters)


def test_symmetric_and_reciprocal_groups_get_a_canonical_orientation_check() -> None:
    text = schema_text()
    assert (
        'CONSTRAINT "kij__pair__ck__canonical" CHECK ("i" <= "j")' in text
        and 'CONSTRAINT "kij__pair__ck__diagonal" CHECK ("i" <> "j")' in text
    )
    assert 'CONSTRAINT "ratio__pair__ck__canonical" CHECK ("i" <= "j")' in text
    assert 'CONSTRAINT "margules__pair__ck__canonical" CHECK ("i" <= "j")' in text
    assert 'CONSTRAINT "margules__pair__ck__diagonal" CHECK ("i" <> "j")' in text
    assert 'CONSTRAINT "signed_interaction__ck__canonical"' in text
    assert 'ROW("a", "b", "c") <= ROW("b", "a", "c")' in text


def test_no_index_is_declared_outside_constraints() -> None:
    assert "CREATE INDEX" not in schema_text()
    assert "CREATE UNIQUE INDEX" not in schema_text()


def test_identifier_too_long_is_refused_by_the_generator(tmp_path: Path) -> None:
    tree = overlay("identifier_too_long", tmp_path)
    result = load_declaration(tree / "model", tree / "forms", contract=None)
    assert any(d.code == "identifier-too-long" for d in result.diagnostics)
    with pytest.raises(DeclarationError):
        generate(_declaration_with_long_table())


def _declaration_with_long_table():  # noqa: ANN202
    import dataclasses

    decl = full_declaration()
    kind = decl.kinds["element"]
    long = dataclasses.replace(kind, name="e" * (MAX_IDENTIFIER_BYTES + 1))
    return dataclasses.replace(decl, kinds={**decl.kinds, long.name: long})


# -- quoting ------------------------------------------------------------------------------


def test_identifiers_are_always_quoted_and_quotes_doubled() -> None:
    assert ident("species") == '"species"'
    assert ident("T_low") == '"T_low"'
    assert ident('a"b') == '"a""b"'
    assert qname("tk", "species") == '"tk"."species"'


def test_literals_double_quotes_and_escape_backslashes() -> None:
    assert literal("it's") == "'it''s'"
    assert literal("'; DROP TABLE x; --") == "'''; DROP TABLE x; --'"
    assert literal("a\\b") == "E'a\\\\b'"
    assert literal(True) == "true"
    assert literal(2.5) == "2.5"
    assert literal("two\nlines") == "'two\nlines'"


def test_a_doc_with_quotes_survives_into_the_ddl(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    path = tree / "model" / "vocab.toml"
    path.write_text(
        path.read_text().replace(
            'doc = "A state of aggregation."', 'doc = "It\'s a \\"state\\"; \\\\ done."'
        )
    )
    text = generate(load_declaration(tree / "model", tree / "forms", contract=None).require())[
        SCHEMA_PATH
    ].decode()
    assert "IS E'It''s a \"state\"; \\\\ done.';" in text


# -- meta tables and rows -----------------------------------------------------------------


def test_every_meta_column_has_a_comment() -> None:
    for name, table in META_TABLES.items():
        assert table.comment, name
        assert all(column.comment for column in table.columns), name


def test_meta_rows_follow_the_column_order_of_their_tables() -> None:
    rows = meta_rows(full_declaration())
    assert set(rows) == set(META_TABLES)
    for name, table_rows in rows.items():
        width = len(META_TABLES[name].columns)
        assert all(len(row) == width for row in table_rows), name
    decl = full_declaration()
    assert len(rows["kind"]) == len(decl.kinds)
    assert len(rows["slot_group"]) == len(decl.slot_groups)
    assert len(rows["entity"]) == len(decl.entities)
    assert len(rows["family"]) == 1
    # the fixture exercises every construct, so every meta table has rows
    assert [name for name, table_rows in rows.items() if not table_rows] == []


def test_meta_rows_for_the_empty_declaration_are_empty(tmp_path: Path) -> None:
    rows = meta_rows(empty_declaration(tmp_path))
    assert all(not table_rows for table_rows in rows.values())


# -- fingerprint --------------------------------------------------------------------------


def test_fingerprint_covers_the_ddl_physical_sql_and_reified_rows() -> None:
    ddl = generate(full_declaration())[SCHEMA_PATH]
    base = schema_fingerprint(ddl, b"", b"rows")
    assert len(base) == 64
    assert schema_fingerprint(ddl, b"", b"rows") == base
    assert schema_fingerprint(ddl, b"CREATE INDEX x ON t (a);", b"rows") != base
    assert schema_fingerprint(ddl + b" ", b"", b"rows") != base
    assert schema_fingerprint(ddl, b"", b"other rows") != base
    # the split between the parts matters
    assert schema_fingerprint(b"ab", b"c", b"") != schema_fingerprint(b"a", b"bc", b"")
    assert schema_fingerprint(b"", b"ab", b"c") != schema_fingerprint(b"", b"a", b"bc")


def test_declaration_fingerprint_is_the_composition_of_its_parts() -> None:
    decl = full_declaration()
    expected = schema_fingerprint(
        generate(decl)[SCHEMA_PATH], b"phys", serialise(insert_batches(decl))
    )
    assert declaration_fingerprint(decl, b"phys") == expected
    assert declaration_fingerprint(decl, b"") != expected


CHANGES = [
    ("model/physical.toml", 'doc = "A gas or vapour."', 'doc = "A gas."'),  # enum member doc
    ("model/physical.toml", 'doc = "Coarse classification of a phase."', 'doc = "Phase class."'),
    (
        "model/identity.toml",
        'doc = "A chemical element: a declared vocabulary."',
        'doc = "An element."',
    ),
    ("model/vocab.toml", "molar_mass = 0.001008", "molar_mass = 0.0010081"),  # entity value
    ("model/vocab.toml", 'doc = "Hydrogen."', 'doc = "The lightest element."'),  # entity doc
    ("model/vocab.toml", '[entities.aggregation.liquid]\ndoc = "Liquid."\n', ""),  # entity removed
    ("model/vocab.toml", 'traces = ["IC-20"]', 'traces = ["IC-21"]'),
    ("forms/forms.toml", 'citations = ["antoine_1888"]', 'citations = ["antoine_1889"]'),
]


@pytest.mark.parametrize(("path", "old", "new"), CHANGES)
def test_any_change_to_the_declaration_changes_the_fingerprint(
    path: str, old: str, new: str, tmp_path: Path
) -> None:
    base = declaration_fingerprint(full_declaration(), b"")
    tree = copy_full(tmp_path)
    text = (tree / path).read_text()
    assert old in text, path
    (tree / path).write_text(text.replace(old, new))
    changed = load_declaration(tree / "model", tree / "forms", contract=None).require()
    assert declaration_fingerprint(changed, b"") != base


def test_reified_serialisation_is_deterministic_and_covers_every_inserted_batch(
    tmp_path: Path,
) -> None:
    decl = full_declaration()
    batches = insert_batches(decl)
    assert serialise(batches) == serialise(insert_batches(decl))
    tree = copy_full(tmp_path)
    fresh = load_declaration(tree / "model", tree / "forms", contract=None).require()
    assert serialise(insert_batches(fresh)) == serialise(batches)
    tables = [(b.schema, b.table) for b in batches]
    assert len(tables) == len(set(tables)) and all(b.rows for b in batches)
    assert ("prov", "record") in tables and ("tk", "release") in tables
    assert [t for t in tables if t[0] == "meta"] == [
        ("meta", name) for name in META_TABLES if ("meta", name) in tables
    ]


# -- --check and the CLI ------------------------------------------------------------------


def test_compare_reports_changed_missing_and_extra(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    files = generate(load_declaration(tree / "model", tree / "forms", contract=None).require())
    assert compare_tree(files, tree) == [Difference(SCHEMA_PATH, "missing")]
    assert write_tree(files, tree) == [Difference(SCHEMA_PATH, "missing")]
    assert compare_tree(files, tree) == []
    target = tree / SCHEMA_PATH
    target.write_bytes(target.read_bytes() + b"-- edited\n")
    assert compare_tree(files, tree) == [Difference(SCHEMA_PATH, "changed")]
    target.unlink()
    (tree / "sql" / "generated" / "stray.sql").write_text("select 1;\n")
    expected = [
        Difference(SCHEMA_PATH, "missing"),
        Difference("sql/generated/stray.sql", "extra"),
    ]
    assert compare_tree(files, tree) == expected
    assert write_tree(files, tree) == expected
    assert not (tree / "sql" / "generated" / "stray.sql").exists()
    assert compare_tree(files, tree) == []


def test_files_outside_the_generated_roots_are_not_extra(tmp_path: Path) -> None:
    tree = make_tree(tmp_path)
    files = generate(load_declaration(tree / "model", tree / "forms", contract=None).require())
    write_tree(files, tree)
    (tree / "sql" / "physical.sql").write_text("-- hand written\n")
    assert compare_tree(files, tree) == []


def test_cli_generate_then_check(tmp_path: Path) -> None:
    # the command generates this pipeline's model, which the loader checks against the contract
    tree = tmp_path
    shutil.copytree(MODEL, tree / "model")
    shutil.copytree(FORMS, tree / "forms")
    missing = runner.invoke(app, ["generate", "--check", "--tree", str(tree)])
    assert missing.exit_code == 1
    assert f"missing: {SCHEMA_PATH}" in missing.output

    written = runner.invoke(app, ["generate", "--tree", str(tree)])
    assert written.exit_code == 0, written.output
    assert (tree / SCHEMA_PATH).is_file()

    current = runner.invoke(app, ["generate", "--check", "--tree", str(tree)])
    assert current.exit_code == 0, current.output
    assert "current" in current.output

    (tree / SCHEMA_PATH).write_text("-- tampered\n")
    changed = runner.invoke(app, ["generate", "--check", "--tree", str(tree)])
    assert changed.exit_code == 1
    assert f"changed: {SCHEMA_PATH}" in changed.output

    (tree / "sql" / "generated" / "extra.sql").write_text("select 1;\n")
    both = runner.invoke(app, ["generate", "--check", "--tree", str(tree)])
    assert both.exit_code == 1
    assert "extra: sql/generated/extra.sql" in both.output
    assert f"changed: {SCHEMA_PATH}" in both.output

    repaired = runner.invoke(app, ["generate", "--tree", str(tree)])
    assert repaired.exit_code == 0
    assert runner.invoke(app, ["generate", "--check", "--tree", str(tree)]).exit_code == 0


def test_cli_generate_reports_a_refused_declaration(tmp_path: Path) -> None:
    tree = overlay("unknown_key", tmp_path)
    result = runner.invoke(app, ["generate", "--tree", str(tree)])
    assert result.exit_code == 1
    assert "[unknown-key]" in result.output
    assert not (tree / "sql").exists()


def test_the_committed_tree_is_current() -> None:
    result = runner.invoke(app, ["generate", "--check"])
    assert result.exit_code == 0, result.output


def test_justfile_declares_the_generate_recipes() -> None:
    text = (config.TREE_DIR / "justfile").read_text()
    assert "tk-generate:" in text and "tk-generate-check:" in text
    assert "tk generate --check" in text
