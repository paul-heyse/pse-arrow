# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The loaded declaration against the pipeline's shape contract (meta-model section 5).

`pipeline_contract.toml` lists the kinds, relations and enums the stage code names and the
attributes, keys and columns it uses. A declaration that lacks one, or spells it another way,
loads and generates cleanly and would fail inside a stage; this check refuses it at load. Each
mismatch is a diagnostic with code `pipeline-contract`, placed at the declaration of the kind or
relation it concerns (or at the contract file when the construct is missing altogether), and
naming the contract entry that requires it.
"""

from __future__ import annotations

from thermo_knowledge.declaration import model as m
from thermo_knowledge.declaration.diagnostics import Code, Diagnostic
from thermo_knowledge.pipeline_contract import CONTRACT_DOCUMENT, Column, Entry, PipelineContract


class _Report:
    """The diagnostics of one check, each placed in a module's document (or the contract's)."""

    def __init__(self, decl: m.Declaration) -> None:
        self.decl = decl
        self.found: list[Diagnostic] = []

    def add(self, module: str | None, construct: str, required_by: str, message: str) -> None:
        owner = self.decl.modules.get(module) if module is not None else None
        self.found.append(
            Diagnostic(
                document=owner.document if owner is not None else CONTRACT_DOCUMENT,
                construct=construct,
                code=Code.PIPELINE_CONTRACT,
                message=f"{message} (required by the pipeline contract, {required_by})",
            )
        )


def check(decl: m.Declaration, contract: PipelineContract) -> list[Diagnostic]:
    """Every way `decl` differs from what `contract` requires."""
    report = _Report(decl)
    for name, entry in contract.kinds.items():
        kind = decl.kinds.get(name)
        if kind is None or kind.origin != "declared":
            report.add(None, f"kinds.{name}", f"kinds.{name}", f"the declaration has no kind `{name}`")
        else:
            _kind(decl, kind, entry, report)
    for name, entry in contract.relations.items():
        relation = decl.relations.get(name)
        if relation is None:
            report.add(
                None,
                f"relations.{name}",
                f"relations.{name}",
                f"the declaration has no relation `{name}`",
            )
        else:
            _relation(relation, entry, report)
    for name, enum_entry in contract.enums.items():
        enum = decl.enums.get(name)
        if enum is None:
            report.add(None, f"enums.{name}", f"enums.{name}", f"the declaration has no enum `{name}`")
            continue
        declared = {member.name for member in enum.members}
        for member in enum_entry.members:
            if member not in declared:
                report.add(
                    enum.module,
                    f"{enum.construct}.members",
                    f"enums.{name}.members",
                    f"enum `{name}` has no member `{member}`",
                )
    return report.found


def _kind(decl: m.Declaration, kind: m.Kind, entry: Entry, report: _Report) -> None:
    required_by = f"kinds.{kind.name}"
    if kind.schema != entry.schema:
        report.add(
            kind.module,
            kind.construct,
            required_by,
            f"kind `{kind.name}` is in schema `{kind.schema}`, not `{entry.schema}`",
        )
    if entry.extends is not None and kind.extends != entry.extends:
        report.add(
            kind.module,
            f"{kind.construct}.extends",
            required_by,
            f"kind `{kind.name}` extends `{kind.extends}`, not `{entry.extends}`",
        )
    if entry.identity is not None:
        root = decl.kinds[kind.root]
        if root.identity != entry.identity:
            report.add(
                kind.module,
                f"{decl.kinds[kind.root].construct}.identity",
                required_by,
                f"kind `{kind.name}` is identified by {list(root.identity)}, "
                f"not {list(entry.identity)}",
            )
    attributes = {a.name: a for a in decl.attributes_of(kind.name)}
    for column in entry.columns.values():
        _column(
            report,
            kind.module,
            f"kind `{kind.name}`",
            f"{kind.construct}.attributes",
            f"{required_by}.attributes.{column.name}",
            attributes.get(column.name),
            column,
        )


def _relation(relation: m.Relation, entry: Entry, report: _Report) -> None:
    required_by = f"relations.{relation.name}"
    if relation.schema != entry.schema:
        report.add(
            relation.module,
            relation.construct,
            required_by,
            f"relation `{relation.name}` is in schema `{relation.schema}`, not `{entry.schema}`",
        )
    keys = {f.name: f for f in relation.keys}
    values = {f.name: f for f in relation.values}
    for column in entry.columns.values():
        if column.role == "key":
            found, part = keys.get(column.name), "keys"
        else:
            found, part = values.get(column.name), "columns"
        _column(
            report,
            relation.module,
            f"relation `{relation.name}`",
            f"{relation.construct}.{part}",
            f"{required_by}.{part}.{column.name}",
            found,
            column,
        )


def _column(
    report: _Report,
    module: str,
    owner: str,
    where: str,
    required_by: str,
    found: m.Field | None,
    wanted: Column,
) -> None:
    noun = {"attribute": "attribute", "key": "key", "column": "value column"}[wanted.role]
    if found is None:
        report.add(
            module, where, required_by, f"{owner} has no {noun} `{wanted.name}` ({wanted.type})"
        )
        return
    at = found.construct
    if found.type.text != wanted.type:
        report.add(
            module,
            f"{at}.type",
            required_by,
            f"{owner}: {noun} `{wanted.name}` is {found.type.text}, not {wanted.type}",
        )
    if found.optional != wanted.optional:
        state = "optional" if found.optional else "required"
        expected = "optional" if wanted.optional else "required"
        report.add(
            module,
            at,
            required_by,
            f"{owner}: {noun} `{wanted.name}` is {state}, and the code needs it {expected}",
        )
    if (found.default is not None) != wanted.defaulted:
        state = "has a default" if found.default is not None else "has no default"
        needed = "relies on a declared default" if wanted.defaulted else "relies on there being none"
        report.add(
            module,
            at,
            required_by,
            f"{owner}: {noun} `{wanted.name}` {state}, and the code {needed}",
        )
