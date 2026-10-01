# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Dispositions: one recorded decision per survey construct whose mapping loses or assumes.

A disposition file, `survey/dispositions/<source id>.toml`, holds one `[[disposition]]` per
construct that needs one (`docs/survey.md`, section 4). The loader checks each against the
source's survey, the loaded declaration and the two documents a `ref` can cite.
"""

from __future__ import annotations

import re
import tomllib
from dataclasses import dataclass
from pathlib import Path

import msgspec

from thermo_knowledge.declaration import Declaration
from thermo_knowledge.survey_index import structs as st
from thermo_knowledge.survey_index.diagnostics import Code, SurveyDiagnostic
from thermo_knowledge.survey_index.loader import SURVEY_DIR, relative_label
from thermo_knowledge.survey_index.validate import Context, KeyCheck, validate_struct

DISPOSITION_DIR = f"{SURVEY_DIR}/dispositions"
"""Directory of the disposition files, relative to the tree."""

MODEL_CHANGE = "model_change"
MAPPING_LOSS = "mapping_loss"
OUT_OF_SCOPE = "out_of_scope"
HELD_BY_MODEL = "held_by_model"
KINDS = (MODEL_CHANGE, MAPPING_LOSS, OUT_OF_SCOPE, HELD_BY_MODEL)

UNSCHEDULED = "unscheduled"
"""The `ref` of a `model_change` that no alignment item or finding schedules yet."""

ALIGNMENT_NOTES_RELATIVE = "docs/alignment-notes.md"
"""Relative to the tree."""
REVIEW_RELATIVE = "docs/design_review/reviews/design_review_thermo-knowledge-core_2026-09-30.md"
"""Relative to the repository root (the parent of the tree)."""

DECLARATION_PREFIXES = ("kind", "relation", "form", "enum")

_ALIGNMENT_REF = re.compile(r"alignment-notes #(\d+)")
_REVIEW_REF = re.compile(r"review (F\d{2})")
_DECLARED_REF = re.compile(r"(kind|relation|form|enum):(\S+)")
_MECHANISM_REF = re.compile(r"mechanism:(\S.*)", re.DOTALL)
_ALIGNMENT_ROW = re.compile(r"^\|\s*(\d+)\s*\|", re.MULTILINE)
_REVIEW_HEADING = re.compile(r"^#{2,4}\s+(F\d{2})\b", re.MULTILINE)


class Disposition(st.Struct, frozen=True, kw_only=True):
    construct: str
    disposition: str
    ref: str = ""
    reason: str


def needs_disposition(construct: st.Construct) -> bool:
    """Whether `construct` needs a disposition: its precision is not `exact`, or its loss is
    non-empty after trimming and is not a bare "none"."""
    if construct.precision != "exact":
        return True
    loss = construct.loss.strip().rstrip(".").strip().lower()
    return bool(loss) and loss != "none"


@dataclass(frozen=True)
class DeclaredNames:
    """The names of the model constructs a `ref` can cite."""

    kinds: frozenset[str]
    relations: frozenset[str]
    forms: frozenset[str]
    enums: frozenset[str]

    @classmethod
    def of(cls, declaration: Declaration) -> DeclaredNames:
        return cls(
            kinds=frozenset(declaration.kinds),
            relations=frozenset(declaration.relations),
            forms=frozenset(declaration.forms),
            enums=frozenset(declaration.enums),
        )

    def names(self, prefix: str) -> frozenset[str]:
        return {
            "kind": self.kinds,
            "relation": self.relations,
            "form": self.forms,
            "enum": self.enums,
        }[prefix]


@dataclass(frozen=True)
class RefIndex:
    """What a `ref` may name: the declaration (`None` when it could not be loaded; such refs
    are then not checked and the caller reports the declaration), the alignment items and the
    review findings."""

    declared: DeclaredNames | None
    alignment_items: frozenset[int]
    review_findings: frozenset[str]


def alignment_paths(tree: Path) -> tuple[Path, Path]:
    """The alignment notes and the design review a `ref` cites, resolved from the tree (the
    review lives under the repository root, the tree's parent)."""
    return tree / ALIGNMENT_NOTES_RELATIVE, tree.parent / REVIEW_RELATIVE


def parse_alignment_items(text: str) -> frozenset[int]:
    """The item numbers of the alignment notes: the first column of its numbered rows."""
    return frozenset(int(number) for number in _ALIGNMENT_ROW.findall(text))


def parse_review_findings(text: str) -> frozenset[str]:
    """The finding identifiers (`F14`) of the design review: its `### F14 - ...` headings."""
    return frozenset(_REVIEW_HEADING.findall(text))


def _read(tree: Path, path: Path, what: str, diagnostics: list[SurveyDiagnostic]) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as error:
        shown = next(
            (
                path.relative_to(base).as_posix()
                for base in (tree, tree.parent)
                if base in path.parents
            ),
            path.name,
        )
        diagnostics.append(
            SurveyDiagnostic(
                file=shown,
                code=Code.INPUT_UNAVAILABLE,
                message=f"cannot read the {what} a ref may cite: {error.strerror or error}",
            )
        )
        return ""


def build_ref_index(
    tree: Path, declaration: Declaration | None
) -> tuple[RefIndex, list[SurveyDiagnostic]]:
    """The index of what refs may name, and a diagnostic for each document that is missing."""
    diagnostics: list[SurveyDiagnostic] = []
    notes_path, review_path = alignment_paths(tree)
    notes = _read(tree, notes_path, "alignment items", diagnostics)
    review = _read(tree, review_path, "review findings", diagnostics)
    declared = DeclaredNames.of(declaration) if declaration is not None else None
    index = RefIndex(declared, parse_alignment_items(notes), parse_review_findings(review))
    return index, diagnostics


def is_scheduled(ref: str) -> bool:
    """Whether `ref` names an alignment item or a review finding: a change that is scheduled."""
    return bool(_ALIGNMENT_REF.fullmatch(ref) or _REVIEW_REF.fullmatch(ref))


def _check_ref(ctx: Context, disposition: Disposition, index: RefIndex) -> bool:
    ref = disposition.ref.strip()
    kind = disposition.disposition
    if not ref:
        if kind == OUT_OF_SCOPE:
            return True
        ctx.report(
            Code.MISSING_REF,
            "ref",
            f"a {kind} disposition needs a ref: what holds the construct or schedules the change",
        )
        return False
    if ref == UNSCHEDULED:
        if kind == MODEL_CHANGE:
            return True
        ctx.report(Code.BAD_REF, "ref", f"{UNSCHEDULED!r} is only for {MODEL_CHANGE}")
        return False
    if _MECHANISM_REF.fullmatch(ref):
        return True
    declared = _DECLARED_REF.fullmatch(ref)
    if declared is not None:
        prefix, name = declared.groups()
        if index.declared is None or name in index.declared.names(prefix):
            return True
        ctx.report(Code.UNKNOWN_REF_TARGET, "ref", f"the declaration has no {prefix} {name!r}")
        return False
    alignment = _ALIGNMENT_REF.fullmatch(ref)
    if alignment is not None:
        number = int(alignment.group(1))
        if number in index.alignment_items:
            return True
        ctx.report(
            Code.UNKNOWN_REF_TARGET, "ref", f"docs/alignment-notes.md has no item #{number}"
        )
        return False
    review = _REVIEW_REF.fullmatch(ref)
    if review is not None:
        if review.group(1) in index.review_findings:
            return True
        ctx.report(
            Code.UNKNOWN_REF_TARGET,
            "ref",
            f"the design review has no finding {review.group(1)}",
        )
        return False
    ctx.report(
        Code.BAD_REF,
        "ref",
        f"{ref!r} is not a ref: use kind:<name>, relation:<name>, form:<name>, enum:<name>, "
        f"mechanism:<name>, 'alignment-notes #<n>', 'review F<nn>' or {UNSCHEDULED!r}",
    )
    return False


def _kind_check() -> KeyCheck:
    def check(value: object) -> str | None:
        if value in KINDS:
            return None
        return f"{value!r} is not one of {', '.join(repr(kind) for kind in KINDS)}"

    return check


def _non_empty(value: object) -> str | None:
    if isinstance(value, str) and value.strip():
        return None
    return "must not be empty"


@dataclass(frozen=True)
class DispositionSet:
    """The valid dispositions by source id and construct name, and every deviation found."""

    by_source: dict[str, dict[str, Disposition]]
    diagnostics: tuple[SurveyDiagnostic, ...]
    files: tuple[str, ...] = ()
    """Labels of the disposition files read."""


def disposition_files(tree: Path) -> list[Path]:
    directory = tree / DISPOSITION_DIR
    return sorted(directory.glob("*.toml")) if directory.is_dir() else []


def load_dispositions(
    tree: Path, surveys: dict[str, st.Survey], index: RefIndex
) -> DispositionSet:
    """Load and validate every `survey/dispositions/*.toml` of `tree` against `surveys`."""
    diagnostics: list[SurveyDiagnostic] = []
    by_source: dict[str, dict[str, Disposition]] = {}
    labels: list[str] = []
    checks = {"disposition": _kind_check(), "construct": _non_empty, "reason": _non_empty}
    for path in disposition_files(tree):
        label = relative_label(tree, path)
        labels.append(label)
        try:
            with path.open("rb") as handle:
                data = tomllib.load(handle)
        except (tomllib.TOMLDecodeError, UnicodeDecodeError) as error:
            diagnostics.append(
                SurveyDiagnostic(file=label, code=Code.TOML_SYNTAX, message=str(error))
            )
            continue
        for key in data:
            if key != "disposition":
                diagnostics.append(
                    SurveyDiagnostic(
                        file=label,
                        key=key,
                        code=Code.UNKNOWN_KEY,
                        message="a disposition file holds only [[disposition]] tables",
                    )
                )
        survey = surveys.get(path.stem)
        if survey is None:
            diagnostics.append(
                SurveyDiagnostic(
                    file=label,
                    code=Code.UNKNOWN_SOURCE,
                    message=f"no loaded survey for source {path.stem!r}",
                )
            )
            continue
        constructs = {item.name: item for item in survey.construct}
        records = data.get("disposition", [])
        if not isinstance(records, list) or not all(isinstance(item, dict) for item in records):
            diagnostics.append(
                SurveyDiagnostic(
                    file=label,
                    table="disposition",
                    code=Code.NOT_A_TABLE,
                    message="'disposition' must be an array of tables ([[disposition]])",
                )
            )
            continue
        accepted = by_source.setdefault(path.stem, {})
        for position, record in enumerate(records):
            name = record.get("construct")
            ctx = Context(
                file=label,
                table="disposition",
                record=name if isinstance(name, str) and name else f"[{position}]",
            )
            ok = validate_struct(Disposition, record, ctx, checks=checks)
            if ok:
                disposition = msgspec.convert(record, Disposition, strict=True)
                ok = _check(ctx, disposition, constructs, accepted, index)
                if ok:
                    accepted[disposition.construct] = disposition
            diagnostics.extend(ctx.diagnostics)
    return DispositionSet(
        by_source={source: items for source, items in by_source.items() if items},
        diagnostics=tuple(diagnostics),
        files=tuple(labels),
    )


def _check(
    ctx: Context,
    disposition: Disposition,
    constructs: dict[str, st.Construct],
    accepted: dict[str, Disposition],
    index: RefIndex,
) -> bool:
    construct = constructs.get(disposition.construct)
    if construct is None:
        ctx.report(
            Code.UNKNOWN_CONSTRUCT,
            "construct",
            "the source's survey has no construct of this name (the name is matched exactly)",
        )
        return False
    ok = True
    if disposition.construct in accepted:
        ctx.report(
            Code.DUPLICATE_DISPOSITION,
            "construct",
            "the construct already has a disposition in this file",
        )
        ok = False
    if not needs_disposition(construct):
        ctx.report(
            Code.NOT_NEEDED,
            "construct",
            "the construct is exact and states no loss; it needs no disposition",
        )
        ok = False
    if not _check_ref(ctx, disposition, index):
        ok = False
    return ok


__all__ = [
    "ALIGNMENT_NOTES_RELATIVE",
    "DISPOSITION_DIR",
    "HELD_BY_MODEL",
    "KINDS",
    "MAPPING_LOSS",
    "MODEL_CHANGE",
    "OUT_OF_SCOPE",
    "REVIEW_RELATIVE",
    "UNSCHEDULED",
    "DeclaredNames",
    "Disposition",
    "DispositionSet",
    "RefIndex",
    "alignment_paths",
    "build_ref_index",
    "disposition_files",
    "is_scheduled",
    "load_dispositions",
    "needs_disposition",
    "parse_alignment_items",
    "parse_review_findings",
]
