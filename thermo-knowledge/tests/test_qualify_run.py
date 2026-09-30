# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Running a qualification case against a database built from fixture forms and a fake harness:
comparison and tolerances, invalid points, blocked runs, persistence and reuse, and the CLI."""

from __future__ import annotations

import json
import shutil
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import psycopg
import pytest
from build_support import count
from mapping_support import real_declaration
from qualify_support import (
    FIXTURES,
    FLUIDS,
    PIN,
    SATURATION,
    World,
    build_world,
    case_text,
    fixture_declaration,
)
from typer.testing import CliRunner

from thermo_knowledge import config, db
from thermo_knowledge.build import build_database, discover
from thermo_knowledge.canonical import store
from thermo_knowledge.build.inputs import QUALIFICATION_DIR, discover_qualification
from thermo_knowledge.cli import app
from thermo_knowledge.generate.fingerprint import declaration_fingerprint
from thermo_knowledge.qualify import persist
from thermo_knowledge.qualify.case import CaseError, load_case, validate
from thermo_knowledge.schema_build import read_physical
from thermo_knowledge.qualify.run import Context, grids, run_case, select_subjects, table_lines
from thermo_knowledge.testing import TestDatabase

runner = CliRunner()


@dataclass
class Env:
    tree: Path
    """A tree for the fixture declaration: model/, forms/, oracles/ and qualification/."""
    world: World
    database: TestDatabase
    log: Path

    @property
    def canonical(self) -> Path:
        return self.tree / "canonical"

    def context(self, *, force: bool = False) -> Context:
        return Context(
            self.world.decl, self.database.url, self.canonical, tree=self.tree, force=force
        )

    def case(self, text: str, name: str = "case"):  # noqa: ANN201
        path = self.tree / "qualification" / f"{name}.toml"
        path.write_text(text, encoding="utf-8")
        return load_case(path)

    def run(self, text: str, name: str = "case", *, force: bool = False):  # noqa: ANN201
        with db.connect(self.database.url) as conn:
            return run_case(self.context(force=force), conn, self.case(text, name))

    def sql(self, query: str, *params: object) -> list[tuple[object, ...]]:
        with db.connect(self.database.url) as conn:
            return conn.execute(query, params).fetchall()  # type: ignore[arg-type]

    def harness_calls(self) -> list[int]:
        return [int(line) for line in self.log.read_text().split()] if self.log.exists() else []


@pytest.fixture
def env(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Iterator[Env]:
    tree = tmp_path / "tree"
    tree.mkdir()
    decl = fixture_declaration(tree)
    shutil.copytree(FIXTURES / "oracles", tree / "oracles")
    (tree / "sql").mkdir()
    shutil.copy(config.TREE_DIR / "sql" / "physical.sql", tree / "sql" / "physical.sql")
    (tree / "qualification").mkdir()
    log = tmp_path / "harness.log"
    monkeypatch.setenv("FAKE_HARNESS_LOG", str(log))
    with TestDatabase() as database:
        world = build_world(tree / "canonical", decl, database.url)
        yield Env(tree, world, database, log)


def run_row(env: Env) -> dict[str, object]:
    with (
        db.connect(env.database.url) as conn,
        conn.cursor(row_factory=psycopg.rows.dict_row) as cursor,
    ):
        cursor.execute("SELECT * FROM qual.qualification_run")
        (row,) = cursor.fetchall()
    return row


# -- occurrences --------------------------------------------------------------------------------


def test_a_subject_with_several_occurrences_blocks_the_run_until_the_case_chooses(env: Env) -> None:
    done = env.run(case_text(key="sat_twice", subjects='select = "list"\nkeys = ["sp-a", "sp-b"]'))
    assert done.outcome == "blocked"
    note = str(done.report["note"])
    assert (
        "occurrences 1, 2" in note
        and str(env.world.species["sp-a"]) in note
        and "occurrence = <n>" in note
    )
    assert env.sql("SELECT outcome FROM qual.qualification_run") == [("blocked",)]


def test_the_occurrence_a_case_states_selects_the_set_that_is_compared(env: Env) -> None:
    both = 'select = "list"\nkeys = ["sp-a", "sp-b"]'
    first = env.run(case_text(key="sat_twice", occurrence=1, subjects=both, tolerance="1e-12"))
    assert first.outcome == "passed" and first.report["subjects"] == 2
    assert set(first.report["sets"]) == {  # type: ignore[arg-type]
        str(env.world.repeated["sp-a#1"]),
        str(env.world.repeated["sp-b#1"]),
    }
    # occurrence 2 holds another reducing pressure for sp-a, which the library does not have
    only_a = 'select = "list"\nkeys = ["sp-a"]'
    second = env.run(
        case_text(key="sat_twice", occurrence=2, subjects=only_a, tolerance="1e-9"), "second"
    )
    assert second.outcome == "failed" and second.report["sets"] == [
        str(env.world.repeated["sp-a#2"])
    ]
    # a subject the stated occurrence does not exist for is no subject of the case
    third = env.run(case_text(key="sat_twice", occurrence=2, subjects='select = "all"'), "third")
    assert third.report["subjects"] == 1


def test_a_case_refuses_an_occurrence_below_one(env: Env) -> None:
    case = env.case(case_text(key="sat_twice", occurrence=0))
    assert "parameterization: `occurrence` counts from 1" in validate(case, env.world.decl)


# -- qualification is derived ----------------------------------------------------------------------

VIEW = (
    "SELECT form, output, run, basis, reference, points, relative_tolerance, "
    "worst_relative_deviation, expression_hash FROM qual.form_qualification ORDER BY run"
)


def test_a_passing_run_against_the_current_expression_is_listed_by_form_and_output(
    env: Env,
) -> None:
    from thermo_knowledge.expression.canonical import evaluation_hash

    done = env.run(case_text(tolerance="1e-12"))
    assert done.outcome == "passed"
    row = run_row(env)
    expected = evaluation_hash(env.world.decl.forms["vapor_pressure_exp_series_tau"], "p_sat")
    assert bytes(row["expression_hash"]).hex() == expected  # type: ignore[arg-type]
    (listed,) = env.sql(VIEW)
    assert listed[:6] == (
        "vapor_pressure_exp_series_tau",
        "p_sat",
        row["key"],
        "source_library",
        "FakeLib@1.2.3",
        15,
    )
    assert (
        listed[6] == 1e-12
        and listed[7] == row["worst_relative_deviation"]
        and listed[8] == expected
    )
    assert env.sql(
        "SELECT o.evaluation_hash FROM meta.form_output o WHERE o.form = 'vapor_pressure_exp_series_tau'"
    ) == [(expected,)]


def test_a_run_made_against_an_earlier_text_of_the_form_is_not_listed(env: Env) -> None:
    env.run(case_text(tolerance="1e-12"))
    assert len(env.sql(VIEW)) == 1
    with db.connect(env.database.url) as conn:
        # what a rebuild from an edited form holds: another evaluation hash for the output
        conn.execute(
            "UPDATE meta.form_output SET evaluation_hash = %s WHERE name = 'p_sat'", ("0" * 64,)
        )
        conn.commit()
    assert env.sql(VIEW) == []


def test_a_run_that_did_not_pass_is_not_listed(env: Env) -> None:
    sp_b = env.world.sets["sp-b"]
    with db.connect(env.database.url) as conn:
        conn.execute(
            "UPDATE param.vapor_pressure_exp_series_tau__pure__term SET n = n * (1 + 1e-6) "
            "WHERE set_id = %s AND k = 1",
            (sp_b,),
        )
        conn.commit()
    assert env.run(case_text(tolerance="1e-9")).outcome == "failed"
    blocked = env.run(case_text(call="crash"), "blocked")
    assert blocked.outcome == "blocked"
    assert {outcome for (outcome,) in env.sql("SELECT outcome FROM qual.qualification_run")} == {
        "blocked",
        "failed",
    }
    assert env.sql(VIEW) == []


def test_every_run_records_the_hash_it_was_made_against_even_when_blocked(env: Env) -> None:
    done = env.run(case_text(key="sat_twice", subjects='select = "list"\nkeys = ["sp-a"]'))
    assert done.outcome == "blocked"
    (row,) = env.sql("SELECT length(expression_hash), outcome FROM qual.qualification_run")
    assert row == (32, "blocked")
    assert env.sql(VIEW) == []


# -- comparison --------------------------------------------------------------------------------


def test_a_form_evaluated_from_the_database_agrees_with_the_library_and_is_recorded(
    env: Env,
) -> None:
    done = env.run(case_text(tolerance="1e-12"))
    report = done.report
    assert done.status == "ran" and done.outcome == "passed"
    assert (
        report["subjects"],
        report["points"],
        report["passed"],
        report["failed"],
        report["invalid"],
    ) == (3, 15, 15, 0, 0)
    assert report["library"] == "FakeLib" and report["library_version"] == "1.2.3"
    worst = report["worst"]
    assert isinstance(worst, dict) and worst["relative_deviation"] < 1e-12  # type: ignore[operator]
    row = run_row(env)
    assert row["outcome"] == "passed" and row["points"] == 15 and row["basis"] == "source_library"
    assert row["relative_tolerance"] == 1e-12 and row["absolute_tolerance"] is None
    assert str(row["key"]).startswith(f"case/vapor_pressure_exp_series_tau/sat@{PIN}/")
    observable = env.sql("SELECT key FROM tk.observable WHERE id = %s", row["observable"])
    assert observable == [("vapor_pressure",)]
    assert env.sql("SELECT count(*) FROM qual.run_parameter_set") == [(3,)]
    assert {str(s) for (s,) in env.sql("SELECT parameter_set FROM qual.run_parameter_set")} == {
        str(i) for i in env.world.sets.values()
    }
    assert env.sql(
        "SELECT s.key, r.version FROM prov.source s JOIN prov.software_release r ON r.id = s.id "
        "WHERE s.id = %s",
        row["reference"],
    ) == [("FakeLib@1.2.3", "1.2.3")]


def test_a_perturbed_coefficient_fails_and_the_report_names_the_subject_and_the_worst_point(
    env: Env,
) -> None:
    sp_b = env.world.sets["sp-b"]
    with db.connect(env.database.url) as conn:
        conn.execute(
            "UPDATE param.vapor_pressure_exp_series_tau__pure__term SET n = n * (1 + 1e-6) "
            "WHERE set_id = %s AND k = 1",
            (sp_b,),
        )
        conn.commit()
    done = env.run(case_text(tolerance="1e-9"))
    report = done.report
    assert done.outcome == "failed" and report["failed"] > 0
    assert report["note"] and "outside the tolerance" in report["note"]  # type: ignore[operator]
    worst = report["worst"]
    assert isinstance(worst, dict) and worst["subject"] == ["sp-b"]
    # the worst point, computed here independently: where one part in 10^6 in n_1 moves p most
    fluid = FLUIDS["sp-b"]
    grid = np.linspace(*fluid["range"], 5)  # type: ignore[misc]

    def pressure(scale: float, T: np.ndarray) -> np.ndarray:
        theta = 1 - T / fluid["T_r"]
        n = [fluid["n"][0] * scale, *fluid["n"][1:]]  # type: ignore[index]
        total = sum(c * theta**e for c, e in zip(n, fluid["t"], strict=True))  # type: ignore[arg-type]
        return fluid["p_r"] * np.exp(fluid["T_r"] / T * total)  # type: ignore[operator]

    deviation = np.abs(pressure(1 + 1e-6, grid) - pressure(1.0, grid)) / pressure(1.0, grid)
    assert worst["at"]["T"] == pytest.approx(grid[np.argmax(deviation)])  # type: ignore[index]
    assert worst["relative_deviation"] == pytest.approx(deviation.max(), rel=1e-6)  # type: ignore[index]
    by_key = {tuple(s["key"]): s for s in report["per_subject"]}  # type: ignore[union-attr]
    assert by_key[("sp-b",)]["failed"] > 0
    assert by_key[("sp-a",)]["failed"] == 0 and by_key[("sp-c",)]["failed"] == 0
    assert run_row(env)["outcome"] == "failed"
    assert run_row(env)["worst_relative_deviation"] == pytest.approx(worst["relative_deviation"])  # type: ignore[index]


@pytest.mark.parametrize(
    ("tolerance", "extra", "outcome"),
    [
        ("1e-9", "", "failed"),
        ("1e-2", "", "passed"),
        ("1e-12", 'absolute_tolerance = { value = 1.0, unit = "bar" }', "passed"),
        ("1e-12", 'absolute_tolerance = { value = 1.0, unit = "Pa" }', "failed"),
    ],
)
def test_tolerances_apply_as_declared(env: Env, tolerance: str, extra: str, outcome: str) -> None:
    with db.connect(env.database.url) as conn:
        conn.execute(
            "UPDATE param.vapor_pressure_exp_series_tau__pure__term SET n = n * (1 + 1e-6) "
            "WHERE set_id = %s AND k = 1",
            (env.world.sets["sp-b"],),
        )
        conn.commit()
    done = env.run(case_text(tolerance=tolerance, extra_comparison=extra))
    assert done.outcome == outcome
    row = run_row(env)
    if extra:
        assert row["absolute_tolerance"] == (1.0e5 if "bar" in extra else 1.0)
        assert env.sql("SELECT key FROM tk.observable WHERE id = %s", row["observable"]) == [
            ("vapor_pressure",)
        ]


def test_nan_and_error_points_are_listed_and_follow_the_declared_policy(env: Env) -> None:
    failing = env.run(case_text(call="mixed", tolerance="1e-12"), "failing")
    report = failing.report
    assert failing.outcome == "failed"
    assert report["invalid"] == 6 and report["passed"] == 9 and report["failed"] == 0
    assert report["points"] == 15  # the invalid points count: they fail the run
    listed = report["invalid_points"]
    assert len(listed) == 6  # type: ignore[arg-type]
    assert {(p["status"]) for p in listed} == {"nan", "error"}  # type: ignore[union-attr]
    assert {tuple(p["subject"]) for p in listed} == {("sp-a",), ("sp-b",), ("sp-c",)}  # type: ignore[union-attr]
    assert "which fail the run" in str(report["note"])

    excluded = env.run(
        case_text(
            call="mixed",
            tolerance="1e-12",
            extra_comparison='invalid_points = "exclude"\ninvalid_reason = "the fixture answers NaN there"',
        ),
        "excluding",
    )
    assert excluded.outcome == "passed"
    assert excluded.report["invalid"] == 6 and excluded.report["points"] == 9  # type: ignore[index]
    assert "excluded: the fixture answers NaN there" in str(excluded.report["note"])  # type: ignore[index]
    rows = {
        key: points for key, points in env.sql("SELECT key, points FROM qual.qualification_run")
    }
    assert {key.split("/")[0]: points for key, points in rows.items()} == {
        "failing": 15,
        "excluding": 9,
    }  # type: ignore[attr-defined]


def test_a_point_the_library_answers_nan_above_the_reducing_temperature_is_invalid(
    env: Env,
) -> None:
    argument = 'values = [300.0, 350.0]\nunit = "K"'  # sp-c's reducing temperature is 310 K
    done = env.run(case_text(argument=argument))
    assert done.outcome == "failed"
    invalid = done.report["invalid_points"]
    assert [(tuple(p["subject"]), p["at"]["T"], p["status"]) for p in invalid] == [
        (("sp-c",), 350.0, "nan")
    ]  # type: ignore[union-attr,index]


# -- subjects and grids ------------------------------------------------------------------------


def test_subjects_are_all_a_declared_list_or_a_seeded_sample(env: Env) -> None:
    with db.connect(env.database.url) as conn:

        def keys(text: str) -> list[str]:
            case = env.case(text)
            form = env.world.decl.forms[case.spec.form]
            group = next(g for g in form.slot_groups if g.qualified == SATURATION)
            parameterization = conn.execute(
                "SELECT id FROM tk.parameterization WHERE key = 'sat'"
            ).fetchone()[0]  # type: ignore[index]
            return [s.keys[0] for s in select_subjects(conn, case, group, parameterization)]

        assert keys(case_text()) == ["sp-a", "sp-b", "sp-c"]
        assert keys(case_text(subjects='select = "list"\nkeys = ["sp-c", "sp-a"]')) == [
            "sp-a",
            "sp-c",
        ]
        sample = case_text(subjects='select = "sample"\nsize = 2\nseed = 7')
        assert keys(sample) == keys(sample) and len(keys(sample)) == 2
        seen = {
            tuple(keys(case_text(subjects=f'select = "sample"\nsize = 2\nseed = {seed}')))
            for seed in range(12)
        }
        assert len(seen) > 1  # the seed decides


def test_a_grid_is_spread_within_each_envelope_with_the_inset_a_fraction_of_its_range(
    env: Env,
) -> None:
    with db.connect(env.database.url) as conn:
        case = env.case(
            case_text(
                argument='points = 5\nwithin = "envelope"\nobservable = "temperature"\ninset = 0.1'
            )
        )
        form = env.world.decl.forms[case.spec.form]
        group = next(g for g in form.slot_groups if g.qualified == SATURATION)
        parameterization = conn.execute(
            "SELECT id FROM tk.parameterization WHERE key = 'sat'"
        ).fetchone()[0]  # type: ignore[index]
        subjects = select_subjects(conn, case, group, parameterization)
        contract = env.world.decl.contracts[form.implements]
        made = grids(conn, case, env.world.decl, contract, subjects)
        for subject in subjects:
            low, high = {"sp-a": (250.0, 399.0), "sp-b": (300.0, 519.0), "sp-c": (200.0, 309.0)}[
                subject.keys[0]
            ]
            span = high - low
            np.testing.assert_allclose(
                made[subject.set_id]["T"], np.linspace(low + 0.1 * span, high - 0.1 * span, 5)
            )
        explicit = env.case(case_text(argument='values = [25.0, 30.0]\nunit = "degC"'))
        made = grids(conn, explicit, env.world.decl, contract, subjects)
        assert made[subjects[0].set_id]["T"] == pytest.approx([298.15, 303.15])


# -- cases the declaration refuses, runs that cannot be carried out ------------------------------


def test_a_case_the_declaration_refuses_is_reported_and_records_nothing(env: Env) -> None:
    text = case_text().replace('form = "vapor_pressure_exp_series_tau"', 'form = "nothing"')
    with pytest.raises(CaseError, match="`nothing` is not a declared form"):
        env.run(text)
    assert env.sql("SELECT count(*) FROM qual.qualification_run") == [(0,)]
    problems = validate(
        env.case(
            case_text(
                tolerance="0",
                argument="values = [1.0]",
                extra_comparison='invalid_points = "exclude"',
            )
            .replace('basis = "source_library"', 'basis = "magic"')
            .replace('output = "p_sat"', 'output = "q"')
            .replace("{pin}", "{pin}{x}")
        ),
        env.world.decl,
    )
    joined = "\n".join(problems)
    for expected in (
        "`q` is not an output",
        "`magic` is not a comparison basis",
        "`values` need their `unit`",
        "`relative_tolerance` is positive",
        "needs the `invalid_reason`",
        "a revision may use only the placeholder",
    ):
        assert expected in joined


@pytest.mark.parametrize(
    ("changes", "reason"),
    [
        ({"library": "absent"}, "harness unavailable: the harness"),
        ({"call": "crash"}, "harness unavailable: the harness exited 3"),
        ({"call": "wrong_unit"}, "answered in 'bar'"),
        ({"call": "short"}, "no answer for each of 5 points"),
        (
            {"subjects": 'select = "list"\nkeys = ["nobody"]'},
            "no set in the parameterization for subject(s) nobody",
        ),
        ({"revision": "zzz"}, "parameterization sat@zzz is not in the database"),
    ],
)
def test_a_run_that_cannot_be_carried_out_is_blocked_with_the_reason(
    env: Env, changes: dict[str, str], reason: str
) -> None:
    done = env.run(case_text(**changes))
    assert done.outcome == "blocked" and reason in str(done.report["note"])
    row = run_row(env)
    assert row["outcome"] == "blocked" and row["points"] == 0 and reason in str(row["note"])
    assert env.sql("SELECT count(*) FROM qual.run_parameter_set") == [(0,)]


def test_a_run_blocked_before_the_library_version_is_known_has_no_reference(env: Env) -> None:
    done = env.run(case_text(library="absent"))
    assert done.outcome == "blocked" and done.report["library_version"] is None
    assert run_row(env)["reference"] is None
    assert env.sql("SELECT count(*) FROM prov.software_release") == [(0,)]
    directory = env.canonical / QUALIFICATION_DIR / "case"
    assert sorted(path.name for path in directory.iterdir()) == [
        "manifest.json",
        "qual.qualification_run.parquet",
        "report.json",
    ]
    assert "unknown" not in " ".join(table_lines(done.report))
    inputs = discover(env.canonical)
    outputs, skipped = discover_qualification(
        env.canonical, inputs, declaration_fingerprint(env.world.decl, read_physical(env.tree))
    )
    assert [o.source_id for o in outputs] == [f"{QUALIFICATION_DIR}/case"] and skipped == []
    with TestDatabase() as fresh:
        result = build_database(fresh.url, env.world.decl, [*inputs, *outputs])
        assert result.tables["qual.qualification_run"] == 1
        assert count(fresh.url, "prov.software_release") == 0


def test_a_run_blocked_after_the_version_is_known_names_the_library_release(env: Env) -> None:
    env.run(case_text(call="short"), "second")
    assert env.sql(
        "SELECT s.key FROM qual.qualification_run r JOIN prov.source s ON s.id = r.reference "
        "WHERE starts_with(r.key, 'second/')"
    ) == [("FakeLib@1.2.3",)]


# -- persistence and reuse ---------------------------------------------------------------------


def test_an_unchanged_case_is_a_no_op_by_reuse_key_until_something_it_depends_on_changes(
    env: Env,
) -> None:
    text = case_text()
    assert env.run(text).status == "ran"
    assert env.harness_calls() == [0, 3]  # the version probe, then the subjects
    before = persist.read_output(env.canonical, "case")
    assert before is not None
    again = env.run(text)
    assert again.status == "current" and again.report == before.report
    assert env.harness_calls() == [0, 3, 0]  # only the version probe
    assert persist.read_output(env.canonical, "case").manifest == before.manifest  # type: ignore[union-attr]
    assert env.run(text, force=True).status == "ran"
    assert env.run(case_text(tolerance="1e-11")).status == "ran"  # the case file's content changed
    # a stored value changed under the same identifiers: not current
    with db.connect(env.database.url) as conn:
        conn.execute(
            "UPDATE param.vapor_pressure_exp_series_tau__pure SET p_r = p_r * 1.5 WHERE id = %s",
            (env.world.sets["sp-a"],),
        )
        conn.commit()
    assert env.run(case_text(tolerance="1e-11")).status == "ran"


def test_a_blocked_run_is_never_reused(env: Env) -> None:
    text = case_text(call="short")
    assert env.run(text).status == "ran" and env.run(text).status == "ran"


def test_the_live_database_replaces_the_rows_of_the_case_only(env: Env) -> None:
    env.run(case_text(tolerance="1e-12"), "first")
    second = env.run(case_text(tolerance="1e-9"), "second")
    keys = lambda: sorted(
        str(k).split("/")[0] for (k,) in env.sql("SELECT key FROM qual.qualification_run")
    )  # noqa: E731
    assert keys() == ["first", "second"]
    ids = env.sql("SELECT id, relative_tolerance FROM qual.qualification_run ORDER BY key")
    env.run(case_text(tolerance="1e-12"), "first", force=True)
    assert keys() == ["first", "second"]
    assert env.sql("SELECT id, relative_tolerance FROM qual.qualification_run ORDER BY key") == ids
    assert env.sql("SELECT count(*) FROM qual.run_parameter_set") == [(6,)]
    assert env.sql("SELECT count(*) FROM prov.software_release") == [(1,)]
    assert second.outcome == "passed"


def test_the_parquet_of_a_run_loads_in_a_build_and_survives_it(env: Env, tmp_path: Path) -> None:
    env.run(case_text())
    directory = env.canonical / QUALIFICATION_DIR / "case"
    assert sorted(path.name for path in directory.iterdir()) == [
        "manifest.json",
        "prov.software_release.parquet",
        "prov.source.parquet",
        "qual.qualification_run.parquet",
        "qual.run_parameter_set.parquet",
        "report.json",
    ]
    manifest = store.read_manifest(directory)
    fingerprint = declaration_fingerprint(env.world.decl, read_physical(env.tree))
    assert manifest.phase == "qualification" and manifest.inputs["declaration"] == fingerprint
    assert set(manifest.inputs) == {
        "declaration",
        "form",
        "case",
        "library",
        "subjects",
        "stored",
        "format",
    }
    inputs = discover(env.canonical)
    outputs, skipped = discover_qualification(env.canonical, inputs, fingerprint)
    assert [o.source_id for o in outputs] == [f"{QUALIFICATION_DIR}/case"] and skipped == []
    with TestDatabase() as fresh:
        result = build_database(fresh.url, env.world.decl, [*inputs, *outputs])
        assert (
            result.tables["qual.qualification_run"] == 1
            and result.tables["qual.run_parameter_set"] == 3
        )
        assert count(fresh.url, "qual.qualification_run") == 1
        assert count(fresh.url, "prov.software_release") == 1


def test_an_output_is_skipped_with_its_reason_when_its_key_or_its_sets_do_not_fit_the_build(
    env: Env,
) -> None:
    env.run(case_text())
    fingerprint = declaration_fingerprint(env.world.decl, read_physical(env.tree))
    inputs = discover(env.canonical)
    _, skipped = discover_qualification(env.canonical, inputs, "0" * 64)
    assert len(skipped) == 1 and skipped[0].case == "case"
    assert (
        "made against another declaration" in skipped[0].reason
        and "tk qualify case" in skipped[0].reason
    )
    outputs, skipped = discover_qualification(env.canonical, [], fingerprint)
    assert outputs == [] and [s.reason for s in skipped] == [
        "3 parameter set(s) it evaluated are not in this build"
    ]
    assert discover_qualification(env.canonical / "nowhere", inputs, fingerprint) == ([], [])


def test_an_output_that_does_not_match_its_manifest_is_an_error(env: Env) -> None:
    env.run(case_text())
    (env.canonical / QUALIFICATION_DIR / "case" / "qual.qualification_run.parquet").write_bytes(
        b"damaged"
    )
    with pytest.raises(store.CanonicalError, match="does not match its manifest"):
        discover_qualification(env.canonical, [], "0" * 64)


# -- the commands ------------------------------------------------------------------------------


def test_tk_qualify_and_tk_build_carry_a_run_through_a_rebuild(
    env: Env, monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    (env.tree / "qualification" / "case.toml").write_text(
        case_text(tolerance="1e-12"), encoding="utf-8"
    )
    monkeypatch.setenv(config.STORE_ENV, str(env.tree))
    monkeypatch.setenv(config.DATABASE_URL_ENV, env.database.url)
    report = tmp_path / "report.json"
    tree = ["--tree", str(env.tree)]
    shown = runner.invoke(app, ["qualify", "--report", str(report), *tree])
    assert shown.exit_code == 0, shown.output
    assert "ran      case" in shown.output and "case: passed (FakeLib 1.2.3" in shown.output
    assert "subjects   points   passed   failed  invalid" in shown.output
    assert json.loads(report.read_text())["cases"][0]["per_subject"][0]["key"] == ["sp-a"]
    again = runner.invoke(app, ["qualify", "case", "--report", str(report), *tree])
    assert again.exit_code == 0 and "current  case" in again.output
    assert runner.invoke(app, ["qualify", "nothing", *tree]).exit_code == 2

    rebuilt = runner.invoke(app, ["build", *tree])
    assert rebuilt.exit_code == 0, rebuilt.output
    assert "qualification outputs: 1 loaded, 0 skipped" in rebuilt.output
    assert env.sql("SELECT outcome, points FROM qual.qualification_run") == [("passed", 15)]
    assert env.sql("SELECT count(*) FROM qual.run_parameter_set") == [(3,)]

    only_resolution = runner.invoke(app, ["build", "--dry-run", "--sources", "src", *tree])
    assert only_resolution.exit_code == 0
    # a failing case makes the command fail, and a refused one too
    (env.tree / "qualification" / "bad.toml").write_text(
        case_text().replace("vapor_pressure_exp_series_tau", "nothing", 1), encoding="utf-8"
    )
    refused = runner.invoke(app, ["qualify", "bad", *tree])
    assert refused.exit_code == 1 and "`nothing` is not a declared form" in refused.output
    (env.tree / "qualification" / "blocked.toml").write_text(
        case_text(library="absent"), encoding="utf-8"
    )
    blocked = runner.invoke(app, ["qualify", "blocked", *tree])
    assert blocked.exit_code == 1 and "blocked" in blocked.output


def test_tk_qualify_refuses_a_database_built_from_another_declaration(
    env: Env, monkeypatch: pytest.MonkeyPatch
) -> None:
    (env.tree / "qualification" / "case.toml").write_text(case_text(), encoding="utf-8")
    with TestDatabase() as empty:
        build_database(empty.url, real_declaration())
        monkeypatch.setenv(config.STORE_ENV, str(env.tree))
        monkeypatch.setenv(config.DATABASE_URL_ENV, empty.url)
        result = runner.invoke(app, ["qualify", "--tree", str(env.tree)])
    assert result.exit_code == 1 and "rebuild it with `tk build`" in result.output
