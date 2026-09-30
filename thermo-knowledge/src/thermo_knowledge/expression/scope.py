# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""What the names of a form's expressions can refer to (expressions.md section 3, Names).

`FormScope` gathers, for one form and the contract it implements, the contract's arguments,
roles and sets, the form's slot groups, sub-form slots and locals. The checker and the
evaluator both read expressions through the classification here, so they cannot disagree about
what `pure.piece.a1[i, n]` or `alpha.value(i=i)` means.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass

from thermo_knowledge.declaration import model as m
from thermo_knowledge.declaration.diagnostics import Code
from thermo_knowledge.expression import tree as t
from thermo_knowledge.expression.parser import ExpressionError, parse_cached


class RefError(Exception):
    """A reference names nothing or names it wrongly; `node` is where."""

    def __init__(self, code: Code, message: str, node: t.Node) -> None:
        super().__init__(message)
        self.code = code
        self.message = message
        self.node = node


@dataclass(frozen=True)
class FormScope:
    decl: m.Declaration
    form: m.Form
    contract: m.Contract
    arguments: Mapping[str, m.Field]
    roles: Mapping[str, m.Field]
    sets: Mapping[str, m.Field]
    groups: Mapping[str, m.SlotGroup]
    subforms: Mapping[str, m.SubformSlot]
    locals: tuple[str, ...]
    unknowns: Mapping[str, tuple[m.ImplicitBlock, m.Unknown]]
    dependent: frozenset[str]
    """The locals that depend on an unknown of an implicit block, directly or through another
    local. A block's own expressions cannot use them; outputs and these locals can use the
    unknowns."""

    @staticmethod
    def of(decl: m.Declaration, form: m.Form) -> FormScope:
        contract = decl.contracts[form.implements]
        unknowns = {
            unknown.name: (block, unknown) for block in form.implicit for unknown in block.unknowns
        }
        return FormScope(
            decl=decl,
            form=form,
            contract=contract,
            arguments={f.name: f for f in contract.arguments},
            roles={f.name: f for f in contract.roles},
            sets={f.name: f for f in contract.sets},
            groups={g.name: g for g in form.slot_groups},
            subforms={s.name: s for s in form.subforms},
            locals=tuple(local.name for local in form.locals),
            unknowns=unknowns,
            dependent=_dependent_locals(form, frozenset(unknowns)),
        )

    def related(self, a: str, b: str) -> bool:
        """Whether kinds `a` and `b` are the same or one refines the other."""
        return self.decl.is_a(a, b) or self.decl.is_a(b, a)


def mentions(node: t.Node, names: frozenset[str] | set[str]) -> bool:
    """Whether a name of `names` is written anywhere in the tree `node`."""
    return any(isinstance(child, t.Name) and child.id in names for child in t.walk(node))


def _dependent_locals(form: m.Form, unknown_names: frozenset[str]) -> frozenset[str]:
    if not unknown_names:
        return frozenset()
    tainted: set[str] = set(unknown_names)
    dependent: set[str] = set()
    for local in form.locals:
        try:
            tree = parse_cached(local.text)
        except ExpressionError:
            continue  # the checker reports the text; it depends on nothing here
        if mentions(tree, tainted):
            tainted.add(local.name)
            dependent.add(local.name)
    return frozenset(dependent)


@dataclass(frozen=True)
class SlotRef:
    """`group.slot[...]` or `group.family.slot[...]`: `raw` are the subscripts as written, the
    subjects of the group first, then the family's indices."""

    group: m.SlotGroup
    family: m.Family | None
    slot: m.Field
    raw: tuple[t.Expr, ...]

    @property
    def subjects(self) -> tuple[t.Expr, ...]:
        return self.raw[: len(self.group.subjects)]

    @property
    def indices(self) -> tuple[t.Expr, ...]:
        return self.raw[len(self.group.subjects) :]

    @property
    def expected(self) -> int:
        return len(self.group.subjects) + (len(self.family.indices) if self.family else 0)


@dataclass(frozen=True)
class FamilyRef:
    """`group.family(subjects...)`: the index set of a family for one subject tuple."""

    group: m.SlotGroup
    family: m.Family
    raw: tuple[t.Expr, ...]


@dataclass(frozen=True)
class OutputCall:
    """`x.output(keyword=...)`: an output of a sub-form (`kind` `subform`), of a nested set
    (`nested`) or of a contribution of a sub-form iterated by `for` (`contribution`)."""

    kind: str
    output: str
    keywords: tuple[t.Keyword, ...]
    accepted: m.Contract
    slot: m.SubformSlot | None = None
    nested: SlotRef | None = None
    var: str | None = None


@dataclass(frozen=True)
class SubformIter:
    """`terms(i=i)`: a sub-form slot iterated for one subject."""

    slot: m.SubformSlot
    keywords: tuple[t.Keyword, ...]


def classify_slot(scope: FormScope, node: t.Attribute | t.Subscript) -> SlotRef | None:
    """The slot `node` names, or `None` when it does not begin with a slot group."""
    raw: tuple[t.Expr, ...] = ()
    target: t.Expr = node
    if isinstance(target, t.Subscript):
        raw = target.indices
        target = target.base
    if not isinstance(target, t.Attribute):
        return None
    inner = target.value
    if isinstance(inner, t.Name) and inner.id in scope.groups:
        group = scope.groups[inner.id]
        slot = next((s for s in group.slots if s.name == target.attr), None)
        if slot is not None:
            return SlotRef(group, None, slot, raw)
        if any(f.name == target.attr for f in group.families):
            raise RefError(
                Code.UNKNOWN_NAME,
                f"`{group.name}.{target.attr}` is a family: name one of its slots, "
                f"`{group.name}.{target.attr}.slot[...]`",
                target,
            )
        raise RefError(
            Code.UNKNOWN_NAME,
            f"slot group `{group.name}` has no slot or family `{target.attr}`",
            target,
        )
    if (
        isinstance(inner, t.Attribute)
        and isinstance(inner.value, t.Name)
        and inner.value.id in scope.groups
    ):
        group = scope.groups[inner.value.id]
        family = next((f for f in group.families if f.name == inner.attr), None)
        if family is None:
            raise RefError(
                Code.UNKNOWN_NAME,
                f"slot group `{group.name}` has no family `{inner.attr}`",
                inner,
            )
        slot = next((s for s in family.slots if s.name == target.attr), None)
        if slot is None:
            raise RefError(
                Code.UNKNOWN_NAME,
                f"family `{group.name}.{family.name}` has no slot `{target.attr}`",
                target,
            )
        return SlotRef(group, family, slot, raw)
    return None


def classify_call(
    scope: FormScope, node: t.Call, contributions: Mapping[str, m.SubformSlot]
) -> FamilyRef | OutputCall | SubformIter:
    """What a call that is not a function of the grammar is. `contributions` maps each index
    variable bound to a contribution of a sub-form slot to that slot."""
    func = node.func
    if isinstance(func, t.Name):
        slot = scope.subforms.get(func.id)
        if slot is None:
            raise RefError(
                Code.UNKNOWN_FUNCTION, f"`{func.id}` is not a sub-form slot of this form", func
            )
        return SubformIter(slot, node.keywords)
    if not isinstance(func, t.Attribute):
        raise RefError(Code.BAD_CALL, "this is not something that can be called", node)
    owner = func.value
    if isinstance(owner, t.Name):
        if owner.id in scope.groups:
            group = scope.groups[owner.id]
            family = next((f for f in group.families if f.name == func.attr), None)
            if family is None:
                raise RefError(
                    Code.UNKNOWN_NAME,
                    f"slot group `{group.name}` has no family `{func.attr}`",
                    func,
                )
            if node.keywords:
                raise RefError(
                    Code.BAD_CALL, "a family's index set takes the subjects by position", node
                )
            return FamilyRef(group, family, node.args)
        if owner.id in contributions:
            slot = contributions[owner.id]
            return OutputCall(
                "contribution",
                func.attr,
                node.keywords,
                scope.decl.contracts[slot.accepts],
                slot=slot,
                var=owner.id,
            )
        if owner.id in scope.subforms:
            slot = scope.subforms[owner.id]
            return OutputCall(
                "subform", func.attr, node.keywords, scope.decl.contracts[slot.accepts], slot=slot
            )
        raise RefError(
            Code.UNKNOWN_NAME,
            f"`{owner.id}` is neither a sub-form slot, a slot group nor a contribution",
            owner,
        )
    if isinstance(owner, t.Attribute | t.Subscript):
        ref = classify_slot(scope, owner)
        if ref is None or ref.slot.shape != "nested_set" or ref.slot.accepts is None:
            raise RefError(
                Code.BAD_CALL, "only a sub-form slot or a nested-set slot has outputs to call", node
            )
        return OutputCall(
            "nested",
            func.attr,
            node.keywords,
            scope.decl.contracts[ref.slot.accepts],
            nested=ref,
        )
    raise RefError(Code.BAD_CALL, "this is not something that can be called", node)
