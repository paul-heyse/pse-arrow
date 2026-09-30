# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Read every survey file under `survey/`, validate it and report every deviation.

The loader accepts exactly the specification of `docs/survey.md`. A record that deviates is
reported at each key that deviates and left out of the loaded survey; nothing stops at the
first deviation.
"""

from __future__ import annotations

import re
import tomllib
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path

import msgspec

from thermo_knowledge.survey_index import structs as st
from thermo_knowledge.survey_index.diagnostics import Code, SurveyDiagnostic
from thermo_knowledge.survey_index.validate import Context, KeyCheck, validate_struct

SURVEY_DIR = "survey"
"""Directory of the survey files, relative to the tree."""

_DATE = re.compile(r"\d{4}-\d{2}-\d{2}")
_CALCULATION_KEY = re.compile(r"[a-z][a-z0-9_]*")


@dataclass(frozen=True)
class SurveySet:
    """The surveys that loaded, by source id, and every deviation found."""

    surveys: dict[str, st.Survey]
    diagnostics: tuple[SurveyDiagnostic, ...]


def survey_dir(tree: Path) -> Path:
    return tree / SURVEY_DIR


def survey_files(tree: Path) -> list[Path]:
    """The survey files of `tree`, by name."""
    directory = survey_dir(tree)
    return sorted(directory.glob("*.toml")) if directory.is_dir() else []


def relative_label(tree: Path, path: Path) -> str:
    return path.relative_to(tree).as_posix()


# -- value checks -----------------------------------------------------------------------------


def _one_of(vocabulary: tuple[str, ...]) -> KeyCheck:
    def check(value: object) -> str | None:
        if value in vocabulary:
            return None
        return f"{value!r} is not one of {', '.join(repr(item) for item in vocabulary)}"

    return check


def _keyword_prefix(keywords: tuple[str, ...]) -> KeyCheck:
    """The value starts with one of `keywords` as a whole word; any detail may follow."""
    pattern = re.compile("(?:" + "|".join(re.escape(word) for word in keywords) + r")(?!\w)")

    def check(value: object) -> str | None:
        if isinstance(value, str) and pattern.match(value):
            return None
        shown = value if not isinstance(value, str) or len(value) <= 60 else value[:57] + "..."
        return (
            f"{shown!r} does not start with one of "
            f"{', '.join(repr(word) for word in keywords)}"
        )

    return check


def _non_empty(value: object) -> str | None:
    if isinstance(value, str) and value.strip():
        return None
    return "must not be empty"


_OTHER_CLASS = re.compile(r"other( \(.+\))?", re.DOTALL)


def _class_check(value: object) -> str | None:
    if isinstance(value, str) and (value in st.CLASSES or _OTHER_CLASS.fullmatch(value)):
        return None
    return (
        f"{value!r} is not a representation class (section 3); `other` may carry "
        "`(what it is)`"
    )


def _date_check(value: object) -> str | None:
    if isinstance(value, str) and _DATE.fullmatch(value):
        return None
    return f"{value!r} is not an ISO date (YYYY-MM-DD)"


def _calculation_check(vocabulary: frozenset[str] | None) -> KeyCheck:
    def check(value: object) -> str | None:
        if not isinstance(value, str):
            return None
        proposed = value.endswith(st.PROPOSED_SUFFIX)
        key = value[: -len(st.PROPOSED_SUFFIX)] if proposed else value
        if not _CALCULATION_KEY.fullmatch(key):
            return (
                f"{value!r} is not a calculation key (lower-case words joined by `_`), "
                f"optionally followed by {st.PROPOSED_SUFFIX!r}"
            )
        if proposed or vocabulary is None or key in vocabulary:
            return None
        return (
            f"{key!r} is not in the calculation vocabulary; a key that is not in it is "
            f"proposed and ends with {st.PROPOSED_SUFFIX!r}"
        )

    return check


def table_checks(calculations: frozenset[str] | None) -> dict[str, dict[str, KeyCheck]]:
    """The value checks of each record table, by key path."""
    return {
        "payload": {"read_by_source": _one_of(st.READ_BY_SOURCE)},
        "construct": {
            "name": _non_empty,
            "candidate": _non_empty,
            "precision": _one_of(st.PRECISIONS),
            "origin": _keyword_prefix(st.ORIGIN_KEYWORDS),
            "fields.shape": _one_of(st.SHAPES),
            "fields.role": _one_of(st.FIELD_ROLES),
        },
        "model_family": {
            "name": _non_empty,
            "class": _class_check,
            "closed_form": _keyword_prefix(st.CLOSED_FORM_KEYWORDS),
        },
        "convention": {"name": _non_empty},
        "selection": {"name": _non_empty},
        "capability": {
            "calculation": _calculation_check(calculations),
            "offered": _one_of(st.OFFERED),
            "evidence": _one_of(st.EVIDENCE),
        },
    }


# -- loading ----------------------------------------------------------------------------------


def _record_label(table: str, record: object, index: int) -> str:
    """How a diagnostic names a record: its `name` (or `calculation`, `about`, first path,
    `locator`, whichever its table is identified by), else its position."""
    if not isinstance(record, dict):
        return f"[{index}]"
    if table == "payload":
        paths = record.get("paths")
        if isinstance(paths, list) and paths and isinstance(paths[0], str) and paths[0]:
            return paths[0]
        return f"[{index}]"
    for key in ("name", "calculation", "about", "locator"):
        if key in record:
            value = record[key]
            if isinstance(value, str) and value:
                return value
            return f"[{index}]"
    return f"[{index}]"


def _load_file(
    path: Path,
    label: str,
    checks: Mapping[str, Mapping[str, KeyCheck]],
    diagnostics: list[SurveyDiagnostic],
) -> st.Survey | None:
    try:
        with path.open("rb") as handle:
            data = tomllib.load(handle)
    except (tomllib.TOMLDecodeError, UnicodeDecodeError) as error:
        diagnostics.append(
            SurveyDiagnostic(file=label, code=Code.TOML_SYNTAX, message=str(error))
        )
        return None
    clean = True
    header_ctx = Context(file=label, table="", record="")
    header = {key: value for key, value in data.items() if key not in st.TABLES}
    if not validate_struct(
        st.Survey,
        header,
        header_ctx,
        checks={"surveyed": _date_check, "source": _non_empty, "pin": _non_empty},
    ):
        clean = False
    if isinstance(header.get("source"), str) and header["source"] != path.stem:
        header_ctx.report(
            Code.SOURCE_MISMATCH,
            "source",
            f"source {header['source']!r} does not match the file name {path.stem!r}",
        )
    diagnostics.extend(header_ctx.diagnostics)
    struct_of = {
        "payload": st.Payload,
        "construct": st.Construct,
        "model_family": st.ModelFamily,
        "convention": st.Convention,
        "selection": st.Selection,
        "discrepancy": st.Discrepancy,
        "capability": st.Capability,
        "question": st.Question,
    }
    tables: dict[str, list[dict[str, object]]] = {}
    for table in st.TABLES:
        if table not in data:
            continue
        records = data[table]
        if not isinstance(records, list) or not all(isinstance(item, dict) for item in records):
            diagnostics.append(
                SurveyDiagnostic(
                    file=label,
                    table=table,
                    code=Code.NOT_A_TABLE,
                    message=f"{table!r} must be an array of tables ([[{table}]])",
                )
            )
            clean = False
            continue
        valid: list[dict[str, object]] = []
        seen: set[str] = set()
        for index, record in enumerate(records):
            ctx = Context(file=label, table=table, record=_record_label(table, record, index))
            ok = validate_struct(struct_of[table], record, ctx, checks=checks.get(table))
            name = record.get("name")
            if table == "construct" and isinstance(name, str) and name:
                if name in seen:
                    ctx.report(
                        Code.DUPLICATE_NAME,
                        "name",
                        "another construct of this source has the same name; dispositions are "
                        "keyed by it",
                    )
                    ok = False
                seen.add(name)
            diagnostics.extend(ctx.diagnostics)
            if ok:
                valid.append(record)
        tables[table] = valid
    if not clean:
        return None
    return msgspec.convert({**header, **tables}, st.Survey, strict=True)


def load_surveys(tree: Path, *, calculations: frozenset[str] | None = None) -> SurveySet:
    """Load every `survey/*.toml` of `tree`.

    `calculations` is the calculation vocabulary of the declaration: a capability key outside
    it must carry the proposed mark. With `None` only the key's form is checked.
    """
    checks = table_checks(calculations)
    diagnostics: list[SurveyDiagnostic] = []
    surveys: dict[str, st.Survey] = {}
    for path in survey_files(tree):
        label = relative_label(tree, path)
        survey = _load_file(path, label, checks, diagnostics)
        if survey is not None:
            surveys[survey.source] = survey
    return SurveySet(surveys=surveys, diagnostics=tuple(diagnostics))
