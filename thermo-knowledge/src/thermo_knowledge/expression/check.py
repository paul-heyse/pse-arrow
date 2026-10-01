# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Name resolution and dimensional checking of form expressions (expressions.md section 4).

`check_declaration` runs once the declaration has resolved. For each form it parses every local
and output, resolves names against the contract and the form, checks that dimensions close and
that each output has the dimension of its contract type, and checks the form's `status`. Every
refusal is a `Diagnostic` with the document, the construct path and the position in the
expression text.
"""

from __future__ import annotations

import keyword
from collections import Counter
from dataclasses import dataclass

from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.declaration import model as m
from thermo_knowledge.declaration.diagnostics import Code, Diagnostic
from thermo_knowledge.expression import tree as t
from thermo_knowledge.expression.parser import ExpressionError, parse, parse_residual
from thermo_knowledge.expression.scope import (
    FamilyRef,
    FormScope,
    OutputCall,
    RefError,
    SlotRef,
    SubformIter,
    classify_call,
    classify_slot,
    mentions,
)
from thermo_knowledge.expression.units import (
    DIMENSIONLESS,
    Dim,
    UnitLiteralError,
    describe,
    is_even,
    sqrt_dim,
    type_dimension,
    unit_literal,
)

_TRANSCENDENTAL = ("exp", "log", "log10", "sinh", "cosh", "tanh", "sin", "cos", "atan", "erf")


@dataclass(frozen=True)
class NumTy:
    """A number: its dimension; whether it is an integer; whether its value is known without
    argument values (index arithmetic) and, for a literal or a computation on literals, it."""

    dim: Dim
    integer: bool = False
    static: bool = False
    const: int | float | None = None


@dataclass(frozen=True)
class SubjectTy:
    """A subject: a role of the contract or an index variable over one of its sets."""

    kind: str
    origin: tuple[str, str]  # ("role", name) or ("set", name)


@dataclass(frozen=True)
class FlagTy:
    static: bool = False


@dataclass(frozen=True)
class ContribTy:
    """A contribution of a `many` (or `optional`) sub-form slot; `bound` are the roles of the
    accepted contract the iteration has fixed."""

    slot: m.SubformSlot
    bound: tuple[str, ...]


@dataclass(frozen=True)
class VecTy:
    """A quantity indexed over sets of the contract: an indexed local, an unknown over sets, or a
    comprehension. Its elements have dimension `dim`; `over` are the set names, in index order;
    `basis` is the composition basis it is declared or asserted to be on, if any."""

    dim: Dim
    over: tuple[str, ...]
    basis: str | None = None


type Ty = NumTy | SubjectTy | FlagTy | ContribTy | VecTy
type Env = dict[str, Ty]


def _describe(ty: Ty) -> str:
    if isinstance(ty, NumTy):
        return f"a number of dimension {describe(ty.dim)}"
    if isinstance(ty, SubjectTy):
        return f"a subject of kind `{ty.kind}`"
    if isinstance(ty, FlagTy):
        return "a condition"
    if isinstance(ty, VecTy):
        return (
            f"a quantity indexed over {', '.join(ty.over)} of dimension {describe(ty.dim)} "
            f"(subscript it, or pass it by name to a sub-form call)"
        )
    return "a contribution of a sub-form slot"


class Checker:
    """Checks the expressions of one form against its scope."""

    def __init__(self, scope: FormScope, document: str) -> None:
        self.scope = scope
        self.document = document
        self.diagnostics: list[Diagnostic] = []
        self.construct = ""
        self.index_names: frozenset[str] = frozenset()
        self.failed_locals: set[str] = set()
        self.in_block = False

    # -- diagnostics -----------------------------------------------------------------------

    def fail(self, node: t.Node, code: Code, message: str) -> None:
        line, column = node.pos
        self.diagnostics.append(
            Diagnostic(
                document=self.document,
                construct=self.construct,
                code=code,
                message=message,
                line=line,
                column=column,
            )
        )

    # -- entry -----------------------------------------------------------------------------

    def base_env(self) -> Env:
        return {
            name: SubjectTy(kind=role.type.element, origin=("role", name))
            for name, role in self.scope.roles.items()
        }

    def check_text(
        self, text: str, construct: str, env: Env, *, residual: bool = False
    ) -> tuple[Ty | None, t.Node]:
        """Parse and check one expression: its type (`None` after reporting when it is refused)
        and the root of its tree, for positions."""
        self.construct = construct
        try:
            tree = parse_residual(text) if residual else parse(text)
        except ExpressionError as error:
            self.diagnostics.append(
                Diagnostic(
                    document=self.document,
                    construct=construct,
                    code=error.code,
                    message=error.message,
                    line=error.line,
                    column=error.column,
                )
            )
            return None, t.Name(id="", pos=error.pos)
        self.index_names = frozenset(
            node.target if isinstance(node, t.Clause) else node.var
            for node in t.walk(tree)
            if isinstance(node, t.Clause | t.Integral)
        )
        return self.visit(tree, env), tree

    # -- values ----------------------------------------------------------------------------

    def dimension(self, node: t.Node, field: m.Field, what: str) -> Dim | None:
        dim = type_dimension(self.scope.decl, field.type, field.extra_order)
        if dim is None:
            self.fail(
                node,
                Code.BAD_TYPE,
                f"{what} `{field.name}` has type {field.type.text}, which is not a quantity",
            )
        return dim

    def number(self, node: t.Expr, env: Env) -> NumTy | None:
        ty = self.visit(node, env)
        if ty is None:
            return None
        if not isinstance(ty, NumTy):
            self.fail(
                node, Code.DIMENSION_MISMATCH, f"a number is needed here, found {_describe(ty)}"
            )
            return None
        return ty

    def dimensionless(self, node: t.Expr, env: Env, what: str) -> NumTy | None:
        ty = self.number(node, env)
        if ty is not None and ty.dim != DIMENSIONLESS:
            self.fail(
                node,
                Code.DIMENSIONLESS_REQUIRED,
                f"{what} needs a dimensionless argument, found dimension {describe(ty.dim)}",
            )
            return None
        return ty

    def integer(self, node: t.Expr, env: Env, what: str, *, static: bool = False) -> NumTy | None:
        ty = self.dimensionless(node, env, what)
        if ty is None:
            return None
        if not ty.integer:
            self.fail(
                node,
                Code.DIMENSION_MISMATCH,
                f"{what} is an integer, found a number that is not one",
            )
            return None
        if static and not ty.static:
            self.fail(
                node,
                Code.DIMENSION_MISMATCH,
                f"{what} is an integer known without argument values (index arithmetic), found "
                "one that depends on them",
            )
            return None
        return ty

    def visit(self, node: t.Expr, env: Env) -> Ty | None:
        match node:
            case t.Num():
                return NumTy(
                    DIMENSIONLESS,
                    integer=isinstance(node.value, int),
                    static=True,
                    const=node.value,
                )
            case t.UnitLiteral():
                try:
                    _, dim = unit_literal(node.text)
                except UnitLiteralError as error:
                    self.fail(node, Code.BAD_UNIT, str(error))
                    return None
                return NumTy(dim)
            case t.Name():
                return self.name(node, env)
            case t.Attribute() | t.Subscript():
                return self.reference(node, env)
            case t.Call():
                return self.call(node, env)
            case t.Func():
                return self.function(node, env)
            case t.BinOp():
                return self.binary(node, env)
            case t.UnaryOp():
                return self.unary(node, env)
            case t.Compare():
                return self.compare(node, env)
            case t.BoolOp():
                flags = [self.flag(v, env) for v in node.values]
                if any(f is None for f in flags):
                    return None
                return FlagTy(static=all(f.static for f in flags if f is not None))
            case t.IfExp():
                return self.conditional(node, env)
            case t.Reduce():
                return self.reduce(node, env)
            case t.Comprehension():
                return self.comprehension(node, env)
            case t.Derivative():
                return self.derivative(node, env)
            case t.At():
                return self.at(node, env)
            case t.Position():
                return self.position(node, env)
            case t.Integral():
                return self.integral(node, env)
            case t.Range():
                self.fail(
                    node,
                    Code.EXPRESSION_GRAMMAR,
                    "`range(a, b)` is an index set: write it after `in`",
                )
                return None
        raise TypeError(f"not an expression node: {node!r}")

    # -- names and references --------------------------------------------------------------

    def name(self, node: t.Name, env: Env) -> Ty | None:
        scope = self.scope
        ident = node.id
        if ident in env:
            return env[ident]
        if ident in scope.arguments:
            argument = scope.arguments[ident]
            if argument.over:
                self.fail(
                    node,
                    Code.SLOT_SUBJECTS,
                    f"argument `{ident}` ranges over {', '.join(argument.over)}: subscript it, "
                    f"`{ident}[{', '.join('i' for _ in argument.over)}]`",
                )
                return None
            dim = self.dimension(node, argument, "argument")
            return None if dim is None else NumTy(dim)
        if ident in scope.sets:
            self.fail(node, Code.NOT_INDEX_SET, f"`{ident}` is a set: use it after `in`")
        elif ident in scope.groups:
            self.fail(
                node,
                Code.UNKNOWN_NAME,
                f"`{ident}` is a slot group: name one of its slots, `{ident}.slot[...]`",
            )
        elif ident in scope.subforms:
            self.fail(
                node,
                Code.UNKNOWN_NAME,
                f"`{ident}` is a sub-form slot: call one of its outputs, `{ident}.output(...)`",
            )
        elif ident in self.failed_locals:
            pass  # its own refusal is already reported
        elif self.in_block and ident in scope.dependent:
            self.fail(
                node,
                Code.LOCAL_ORDER,
                f"local `{ident}` depends on an unknown of an implicit block, so the expressions "
                "of an implicit block cannot use it",
            )
        elif ident in scope.unknowns:
            self.fail(
                node,
                Code.UNKNOWN_NAME,
                f"`{ident}` is an unknown of implicit block `{scope.unknowns[ident][0].name}`: "
                "only that block's own expressions, the outputs and the locals after it can use it",
            )
        elif ident in scope.locals:
            self.fail(
                node,
                Code.LOCAL_ORDER,
                f"local `{ident}` is used before it is defined (a local may only use earlier ones)",
            )
        elif ident in self.index_names:
            self.fail(
                node,
                Code.INDEX_SCOPE,
                f"`{ident}` is an index variable used outside the `for` (or integral) that introduces it",
            )
        else:
            self.fail(
                node,
                Code.UNKNOWN_NAME,
                f"`{ident}` is not an argument, role, local or index variable",
            )
        return None

    def subject(self, node: t.Expr, expected: tuple[str, str] | None, what: str, env: Env) -> bool:
        """Whether `node` is a subject of the `expected` origin."""
        if not isinstance(node, t.Name):
            self.fail(
                node,
                Code.SLOT_SUBJECTS,
                f"{what} is a role or an index variable of a set, found an expression",
            )
            return False
        ty = self.visit(node, env)
        if ty is None:
            return False
        if not isinstance(ty, SubjectTy):
            self.fail(
                node, Code.SLOT_SUBJECTS, f"{what} is a subject, `{node.id}` is {_describe(ty)}"
            )
            return False
        if expected is None:
            self.fail(
                node, Code.SLOT_SUBJECTS, f"{what} is not bound to a role or set of the contract"
            )
            return False
        if ty.origin != expected:
            self.fail(
                node,
                Code.SLOT_SUBJECTS,
                f"{what} is bound to {expected[0]} `{expected[1]}`, but `{node.id}` is "
                f"{ty.origin[0]} `{ty.origin[1]}`",
            )
            return False
        return True

    def subjects_of(
        self, group: m.SlotGroup, raw: tuple[t.Expr, ...], node: t.Node, env: Env
    ) -> bool:
        if len(raw) != len(group.subjects):
            self.fail(
                node,
                Code.SLOT_SUBJECTS,
                f"slot group `{group.name}` has {len(group.subjects)} subject"
                f"{'' if len(group.subjects) == 1 else 's'} ({', '.join(f.name for f in group.subjects)}), "
                f"found {len(raw)}",
            )
            return False
        ok = True
        for expression, subject, binding in zip(raw, group.subjects, group.bindings):
            expected = None if binding.kind is None else (binding.kind, binding.target or "")
            what = f"subject `{subject.name}` of slot group `{group.name}`"
            ok = self.subject(expression, expected, what, env) and ok
        return ok

    def slot(self, ref: SlotRef, node: t.Node, env: Env) -> Dim | None:
        """Check the subscripts of a slot reference; return the dimension of its value."""
        scope = self.scope
        ok = True
        if len(ref.raw) != ref.expected:
            owner = f"{ref.group.name}.{ref.family.name}" if ref.family else ref.group.name
            self.fail(
                node,
                Code.SLOT_SUBJECTS,
                f"`{owner}.{ref.slot.name}` takes {ref.expected} subscript"
                f"{'' if ref.expected == 1 else 's'} ({len(ref.group.subjects)} subject"
                f"{'' if len(ref.group.subjects) == 1 else 's'}"
                f"{', then ' + ', '.join(i.name for i in ref.family.indices) if ref.family else ''}), "
                f"found {len(ref.raw)}",
            )
            return None
        ok = self.subjects_of(ref.group, ref.subjects, node, env)
        if ref.family is not None:
            for expression, index in zip(ref.indices, ref.family.indices):
                if index.type.text != "Integer":
                    self.fail(
                        node,
                        Code.BAD_FAMILY,
                        f"index `{index.name}` of family `{ref.family.name}` is {index.type.text}: "
                        "expressions index families by Integer only",
                    )
                    ok = False
                    continue
                ok = self.integer(expression, env, f"the index `{index.name}`") is not None and ok
        shape = ref.slot.shape
        assert shape is not None  # the resolver gives every slot its shape
        if shape in ("nested_set", "set_reference"):
            held = "a nested set" if shape == "nested_set" else "a reference to a set"
            self.fail(
                node,
                Code.SLOT_SHAPE,
                f"slot `{ref.slot.name}` holds {held}: call one of its outputs, "
                f"`...{ref.slot.name}[...].output(...)`",
            )
            return None
        if shape != "quantity":
            self.fail(
                node,
                Code.SLOT_SHAPE,
                f"slot `{ref.slot.name}` holds a {shape.replace('_', ' ')}, which an expression "
                "cannot use as a number",
            )
            return None
        dim = type_dimension(scope.decl, ref.slot.type, ref.slot.extra_order)
        return dim if ok else None

    def convention(self, node: t.Attribute) -> Ty | None:
        """`convention.<name>`: a convention fact the form declares it reads."""
        scope = self.scope
        declared = scope.conventions.get(node.attr)
        if declared is None:
            listed = ", ".join(scope.conventions) or "none"
            self.fail(
                node,
                Code.UNKNOWN_NAME,
                f"form `{scope.form.name}` does not declare the convention fact `{node.attr}` "
                f"(it declares {listed}): name it in `conventions`",
            )
            return None
        if declared.over is not None:
            self.fail(
                node,
                Code.BAD_CONVENTION,
                f"the convention fact `{node.attr}` is read per component of `{declared.over}`: "
                f"write `convention.{node.attr}[i]`",
            )
            return None
        dim = type_dimension(scope.decl, declared.type)
        if dim is None:
            self.fail(node, Code.BAD_TYPE, f"convention fact `{node.attr}` is not a quantity")
            return None
        return NumTy(dim)

    def component_convention(self, node: t.Subscript, env: Env) -> Ty | None:
        """`convention.<name>[i]`: the convention fact of the parameterization that supplied the
        set of the component `i`, for a fact the form declares in `component_conventions`."""
        scope = self.scope
        assert isinstance(node.base, t.Attribute)
        name = node.base.attr
        declared = scope.conventions.get(name)
        if declared is None:
            listed = ", ".join(scope.conventions) or "none"
            self.fail(
                node,
                Code.UNKNOWN_NAME,
                f"form `{scope.form.name}` does not declare the convention fact `{name}` "
                f"(it declares {listed}): name it in `component_conventions`",
            )
            return None
        if declared.over is None:
            self.fail(
                node,
                Code.BAD_CONVENTION,
                f"the convention fact `{name}` is a fact of the form's parameterization, not of "
                f"a component: write `convention.{name}`, or declare it in `component_conventions`",
            )
            return None
        if len(node.indices) != 1:
            self.fail(
                node,
                Code.SLOT_SUBJECTS,
                f"`convention.{name}` is read for one component of `{declared.over}`, found "
                f"{len(node.indices)} subscripts",
            )
            return None
        what = f"the subscript of `convention.{name}` over `{declared.over}`"
        if not self.subject(node.indices[0], ("set", declared.over), what, env):
            return None
        dim = type_dimension(scope.decl, declared.type)
        if dim is None:
            self.fail(node, Code.BAD_TYPE, f"convention fact `{name}` is not a quantity")
            return None
        return NumTy(dim)

    def reference(self, node: t.Attribute | t.Subscript, env: Env) -> Ty | None:
        scope = self.scope
        if (
            isinstance(node, t.Attribute)
            and isinstance(node.value, t.Name)
            and node.value.id == m.CONVENTION_NAME
            and m.CONVENTION_NAME not in env
        ):
            return self.convention(node)
        if (
            isinstance(node, t.Subscript)
            and isinstance(node.base, t.Attribute)
            and isinstance(node.base.value, t.Name)
            and node.base.value.id == m.CONVENTION_NAME
            and m.CONVENTION_NAME not in env
        ):
            return self.component_convention(node, env)
        if isinstance(node, t.Subscript) and isinstance(node.base, t.Name):
            ident = node.base.id
            argument = scope.arguments.get(ident)
            if argument is None:
                held = env.get(ident)
                if isinstance(held, VecTy):
                    return self.indexed_vector(node, ident, held, env)
                if ident in env or ident in scope.locals:
                    self.fail(node, Code.SLOT_SUBJECTS, f"`{ident}` cannot be subscripted")
                else:
                    self.fail(node, Code.UNKNOWN_NAME, f"`{ident}` is not an argument")
                return None
            return self.indexed_argument(node, argument, env)
        try:
            ref = classify_slot(scope, node)
        except RefError as error:
            self.fail(error.node, error.code, error.message)
            return None
        if ref is None:
            self.fail(
                node,
                Code.UNKNOWN_NAME,
                "this does not name a slot: write `group.slot[subjects]` or "
                "`group.family.slot[subjects, indices]`",
            )
            return None
        dim = self.slot(ref, node, env)
        return None if dim is None else NumTy(dim)

    def indexed_argument(self, node: t.Subscript, argument: m.Field, env: Env) -> Ty | None:
        if not argument.over:
            self.fail(node, Code.SLOT_SUBJECTS, f"argument `{argument.name}` is not indexed")
            return None
        dim = self.dimension(node, argument, "argument")
        if dim is None:
            return None
        return self.indexed_vector(
            node, argument.name, VecTy(dim, tuple(argument.over), argument.basis), env
        )

    def indexed_vector(self, node: t.Subscript, name: str, vector: VecTy, env: Env) -> Ty | None:
        """`name[i, j]` for an argument, a local or an unknown indexed over sets."""
        if len(node.indices) != len(vector.over):
            self.fail(
                node,
                Code.SLOT_SUBJECTS,
                f"`{name}` ranges over {', '.join(vector.over)} "
                f"({len(vector.over)} subscript{'' if len(vector.over) == 1 else 's'}), "
                f"found {len(node.indices)}",
            )
            return None
        ok = True
        for expression, set_name in zip(node.indices, vector.over):
            what = f"the subscript of `{name}` over `{set_name}`"
            ok = self.subject(expression, ("set", set_name), what, env) and ok
        return NumTy(vector.dim) if ok else None

    # -- calls -----------------------------------------------------------------------------

    def contributions(self, env: Env) -> dict[str, m.SubformSlot]:
        return {name: ty.slot for name, ty in env.items() if isinstance(ty, ContribTy)}

    def call(self, node: t.Call, env: Env) -> Ty | None:
        try:
            ref = classify_call(self.scope, node, self.contributions(env))
        except RefError as error:
            self.fail(error.node, error.code, error.message)
            return None
        if isinstance(ref, FamilyRef):
            self.fail(
                node,
                Code.NOT_INDEX_SET,
                "a family's index set is used after `in` or as the first argument of `at`",
            )
            return None
        if isinstance(ref, SubformIter):
            self.fail(node, Code.NOT_INDEX_SET, "a sub-form slot is used after `in`")
            return None
        return self.output_call(ref, node, env)

    def output_call(self, ref: OutputCall, node: t.Call, env: Env) -> Ty | None:
        accepted = ref.accepted
        ok = True
        bound: tuple[str, ...] = ()
        if ref.kind == "subform":
            assert ref.slot is not None
            if ref.slot.multiplicity != "one":
                self.fail(
                    node,
                    Code.BAD_CALL,
                    f"sub-form slot `{ref.slot.name}` is `{ref.slot.multiplicity}`: iterate it, "
                    f"`sum(c.output(...) for c in {ref.slot.name})`",
                )
                return None
        elif ref.kind == "contribution":
            assert ref.slot is not None and ref.var is not None
            contribution = env[ref.var]
            assert isinstance(contribution, ContribTy)
            bound = contribution.bound
        else:
            assert ref.nested is not None
            ok = self.slot_for_call(ref.nested, node, env)
            if ref.nested.slot.shape == "set_reference":
                # A referenced set is independently identified: its own subjects supply the roles
                # of the called contract that the call does not give.
                given_names = {keyword.name for keyword in ref.keywords}
                bound = tuple(r.name for r in accepted.roles if r.name not in given_names)
        output = next((o for o in accepted.outputs if o.name == ref.output), None)
        if output is None:
            self.fail(
                node,
                Code.UNKNOWN_OUTPUT,
                f"contract `{accepted.name}` has no output `{ref.output}` "
                f"(outputs: {', '.join(o.name for o in accepted.outputs)})",
            )
            return None
        ok = self.keywords(accepted, bound, ref.keywords, node, env) and ok
        dim = self.dimension(node, output, "output")
        return NumTy(dim) if ok and dim is not None else None

    def slot_for_call(self, ref: SlotRef, node: t.Node, env: Env) -> bool:
        """The subscripts of a nested-set or set-reference slot that is called, not read."""
        if len(ref.raw) != ref.expected:
            self.fail(
                node,
                Code.SLOT_SUBJECTS,
                f"`{ref.group.name}.{ref.slot.name}` takes {ref.expected} subscripts, found "
                f"{len(ref.raw)}",
            )
            return False
        if ref.family is not None and ref.slot.shape == "nested_set":
            self.fail(node, Code.SLOT_SHAPE, "a family slot cannot hold a nested set")
            return False
        ok = self.subjects_of(ref.group, ref.subjects, node, env)
        if ref.family is not None:
            for expression, index in zip(ref.indices, ref.family.indices):
                if index.type.text != "Integer":
                    self.fail(
                        node,
                        Code.BAD_FAMILY,
                        f"index `{index.name}` of family `{ref.family.name}` is "
                        f"{index.type.text}: expressions index families by Integer only",
                    )
                    ok = False
                    continue
                ok = self.integer(expression, env, f"the index `{index.name}`") is not None and ok
        return ok

    def keywords(
        self,
        accepted: m.Contract,
        bound: tuple[str, ...],
        keywords: tuple[t.Keyword, ...],
        node: t.Node,
        env: Env,
    ) -> bool:
        roles = {f.name: f for f in accepted.roles if f.name not in bound}
        arguments = {f.name: f for f in accepted.arguments}
        sets = {f.name: f for f in accepted.sets}
        ok = True
        given: dict[str, t.Keyword] = {}
        for keyword in keywords:
            if keyword.name in given:
                self.fail(keyword, Code.BAD_CALL, f"`{keyword.name}` is given twice")
                ok = False
            else:
                given[keyword.name] = keyword
        # The sets first: an indexed argument is checked against the sets the call passes.
        passed: dict[str, str] = {}
        for name, keyword in given.items():
            if name in sets:
                caller_set = self.set_argument(keyword, sets[name], accepted)
                if caller_set is None:
                    ok = False
                else:
                    passed[name] = caller_set
        for name, keyword in given.items():
            if name in roles:
                role = roles[name]
                ty = self.visit(keyword.value, env)
                if ty is None:
                    ok = False
                elif not isinstance(ty, SubjectTy) or not self.scope.related(
                    ty.kind, role.type.element
                ):
                    self.fail(
                        keyword.value,
                        Code.SLOT_SUBJECTS,
                        f"role `{role.name}` of contract `{accepted.name}` takes a subject of kind "
                        f"`{role.type.element}`, found {_describe(ty)}",
                    )
                    ok = False
            elif name in arguments:
                argument = arguments[name]
                if argument.over:
                    ok = self.vector_argument(keyword, argument, accepted, passed, env) and ok
                    continue
                expected = self.dimension(keyword, argument, "argument")
                ty = self.number(keyword.value, env)
                if expected is None or ty is None:
                    ok = False
                elif ty.dim != expected:
                    self.fail(
                        keyword.value,
                        Code.DIMENSION_MISMATCH,
                        f"argument `{argument.name}` of contract `{accepted.name}` has dimension "
                        f"{describe(expected)}, found {describe(ty.dim)}",
                    )
                    ok = False
            elif name not in sets:
                self.fail(
                    keyword,
                    Code.BAD_CALL,
                    f"`{name}` is neither a role, a set nor an argument of contract `{accepted.name}`",
                )
                ok = False
        missing = [n for n in (*roles, *sets, *arguments) if n not in given]
        if missing:
            self.fail(
                node,
                Code.BAD_CALL,
                f"the call does not give {', '.join(f'`{n}`' for n in missing)} "
                f"(contract `{accepted.name}`)",
            )
            ok = False
        return ok

    def set_argument(self, keyword: t.Keyword, callee: m.Field, accepted: m.Contract) -> str | None:
        """The set of this form's contract that `keyword` passes for the set `callee` of the
        accepted contract, or `None` after reporting."""
        value = keyword.value
        if not isinstance(value, t.Name) or value.id not in self.scope.sets:
            self.fail(
                value,
                Code.NOT_INDEX_SET,
                f"set `{callee.name}` of contract `{accepted.name}` is given a set of this form's "
                f"contract ({', '.join(self.scope.sets) or 'it has none'})",
            )
            return None
        ours = self.scope.sets[value.id]
        if not self.scope.related(ours.type.element, callee.type.element):
            self.fail(
                value,
                Code.SLOT_SUBJECTS,
                f"set `{callee.name}` of contract `{accepted.name}` holds `{callee.type.element}`, "
                f"but `{value.id}` holds `{ours.type.element}`: the kinds are unrelated",
            )
            return None
        return value.id

    def vector_argument(
        self,
        keyword: t.Keyword,
        argument: m.Field,
        accepted: m.Contract,
        passed: dict[str, str],
        env: Env,
    ) -> bool:
        """An argument indexed over sets is given an indexed argument or local by name, or a
        comprehension, over the sets the call passes and of the argument's dimension."""
        expected = self.dimension(keyword, argument, "argument")
        value = keyword.value
        ty: Ty | None
        if (
            isinstance(value, t.Name)
            and value.id in self.scope.arguments
            and self.scope.arguments[value.id].over
        ):
            held = self.scope.arguments[value.id]
            dim = self.dimension(value, held, "argument")
            ty = None if dim is None else VecTy(dim, tuple(held.over), held.basis)
        else:
            ty = self.visit(value, env)
        if ty is None or expected is None:
            return False
        if not isinstance(ty, VecTy):
            self.fail(
                value,
                Code.SLOT_SUBJECTS,
                f"argument `{argument.name}` of contract `{accepted.name}` ranges over "
                f"{', '.join(argument.over)}: give an indexed argument or local by name, or a "
                f"comprehension, found {_describe(ty)}",
            )
            return False
        ok = True
        wanted = tuple(passed.get(name) for name in argument.over)
        if None not in wanted and ty.over != wanted:
            self.fail(
                value,
                Code.SLOT_SUBJECTS,
                f"argument `{argument.name}` of contract `{accepted.name}` ranges over "
                f"{', '.join(str(w) for w in wanted)} as this call passes its sets, but the value "
                f"ranges over {', '.join(ty.over)}",
            )
            ok = False
        if ty.dim != expected:
            self.fail(
                value,
                Code.DIMENSION_MISMATCH,
                f"argument `{argument.name}` of contract `{accepted.name}` has dimension "
                f"{describe(expected)}, found {describe(ty.dim)}",
            )
            ok = False
        if argument.basis is not None and ty.basis != argument.basis:
            if isinstance(value, t.Comprehension):
                found = "a comprehension with no basis stated"
                fix = f"write it `basis('{argument.basis}', [...])`"
            elif ty.basis is None:
                found = "a value with no basis stated"
                fix = "give it the basis, by an argument that declares it or `basis('name', [...])`"
            else:
                found = f"a value on the basis `{ty.basis}`"
                fix = "convert it; the basis of a value is never changed by passing it"
            self.fail(
                value,
                Code.BASIS_MISMATCH,
                f"argument `{argument.name}` of contract `{accepted.name}` is on the basis "
                f"`{argument.basis}`, but the call passes {found}: {fix}",
            )
            ok = False
        return ok

    # -- functions and operators -----------------------------------------------------------

    def function(self, node: t.Func, env: Env) -> Ty | None:
        name = node.name
        if name in _TRANSCENDENTAL:
            arg = self.dimensionless(node.args[0], env, f"`{name}`")
            return None if arg is None else NumTy(DIMENSIONLESS)
        if name in ("chebyshev_t", "debye"):
            order = self.integer(node.args[0], env, f"the order of `{name}`", static=True)
            arg = self.dimensionless(node.args[1], env, f"`{name}`")
            return None if order is None or arg is None else NumTy(DIMENSIONLESS)
        args = [self.number(a, env) for a in node.args]
        if any(a is None for a in args):
            return None
        numbers = [a for a in args if a is not None]
        if name == "sqrt":
            (only,) = numbers
            if not is_even(only.dim):
                self.fail(
                    node,
                    Code.BAD_POWER,
                    f"the square root of dimension {describe(only.dim)} is not a dimension "
                    "(every exponent must be even)",
                )
                return None
            return NumTy(sqrt_dim(only.dim))
        if name == "abs":
            (only,) = numbers
            return NumTy(only.dim, integer=only.integer, static=only.static)
        # min and max
        first = numbers[0]
        for other in numbers[1:]:
            if other.dim != first.dim:
                self.fail(
                    node,
                    Code.DIMENSION_MISMATCH,
                    f"`{name}` compares {describe(first.dim)} with {describe(other.dim)}",
                )
                return None
        return NumTy(
            first.dim,
            integer=all(n.integer for n in numbers),
            static=all(n.static for n in numbers),
        )

    def binary(self, node: t.BinOp, env: Env) -> Ty | None:
        left = self.number(node.left, env)
        right = self.number(node.right, env)
        if left is None or right is None:
            return None
        static = left.static and right.static
        const = _fold(node.op, left.const, right.const)
        if node.op in ("+", "-"):
            if left.dim != right.dim:
                self.fail(
                    node,
                    Code.DIMENSION_MISMATCH,
                    f"cannot {'add' if node.op == '+' else 'subtract'} dimension "
                    f"{describe(left.dim)} and dimension {describe(right.dim)}",
                )
                return None
            return NumTy(
                left.dim, integer=left.integer and right.integer, static=static, const=const
            )
        if node.op == "*":
            return NumTy(
                left.dim * right.dim,
                integer=left.integer and right.integer,
                static=static,
                const=const,
            )
        if node.op == "/":
            return NumTy(left.dim / right.dim, static=static, const=const)
        # Power.
        if right.dim != DIMENSIONLESS:
            self.fail(
                node.right,
                Code.DIMENSIONLESS_REQUIRED,
                f"an exponent is dimensionless, found dimension {describe(right.dim)}",
            )
            return None
        if left.dim == DIMENSIONLESS:
            integer = (
                left.integer and right.integer and right.const is not None and right.const >= 0
            )
            return NumTy(DIMENSIONLESS, integer=integer, static=static, const=const)
        exponent = right.const
        if exponent is None or not float(exponent).is_integer():
            self.fail(
                node.right,
                Code.BAD_POWER,
                f"a non-integer power of a dimensioned base (dimension {describe(left.dim)}): "
                "the exponent is an integer literal",
            )
            return None
        return NumTy(left.dim ** int(exponent), static=static, const=const)

    def unary(self, node: t.UnaryOp, env: Env) -> Ty | None:
        if node.op == "not":
            flag = self.flag(node.operand, env)
            return None if flag is None else FlagTy(static=flag.static)
        operand = self.number(node.operand, env)
        if operand is None:
            return None
        const = None
        if operand.const is not None:
            const = -operand.const if node.op == "-" else operand.const
        return NumTy(operand.dim, integer=operand.integer, static=operand.static, const=const)

    def flag(self, node: t.Expr, env: Env) -> FlagTy | None:
        ty = self.visit(node, env)
        if ty is None:
            return None
        if not isinstance(ty, FlagTy):
            self.fail(
                node, Code.DIMENSION_MISMATCH, f"a condition is needed here, found {_describe(ty)}"
            )
            return None
        return ty

    def compare(self, node: t.Compare, env: Env) -> Ty | None:
        left = self.visit(node.left, env)
        right = self.visit(node.right, env)
        if left is None or right is None:
            return None
        if isinstance(left, NumTy) and isinstance(right, NumTy):
            if left.dim != right.dim:
                self.fail(
                    node,
                    Code.DIMENSION_MISMATCH,
                    f"cannot compare dimension {describe(left.dim)} with dimension {describe(right.dim)}",
                )
                return None
            return FlagTy(static=left.static and right.static)
        if isinstance(left, SubjectTy) and isinstance(right, SubjectTy):
            if node.op not in ("==", "!="):
                self.fail(
                    node, Code.DIMENSION_MISMATCH, "subjects are compared with `==` and `!=` only"
                )
                return None
            if not self.scope.related(left.kind, right.kind):
                self.fail(
                    node,
                    Code.DIMENSION_MISMATCH,
                    f"cannot compare a `{left.kind}` with a `{right.kind}`: the kinds are unrelated",
                )
                return None
            return FlagTy(static=True)
        self.fail(
            node,
            Code.DIMENSION_MISMATCH,
            f"cannot compare {_describe(left)} with {_describe(right)}",
        )
        return None

    def conditional(self, node: t.IfExp, env: Env) -> Ty | None:
        test = self.flag(node.test, env)
        body = self.number(node.body, env)
        orelse = self.number(node.orelse, env)
        if test is None or body is None or orelse is None:
            return None
        if body.dim != orelse.dim:
            self.fail(
                node,
                Code.DIMENSION_MISMATCH,
                f"the branches of a conditional have dimension {describe(body.dim)} and "
                f"dimension {describe(orelse.dim)}",
            )
            return None
        return NumTy(
            body.dim,
            integer=body.integer and orelse.integer,
            static=test.static and body.static and orelse.static,
        )

    # -- sums, derivatives, integrals ------------------------------------------------------

    def iterable(self, node: t.Expr, env: Env) -> Ty | None:
        """What an index variable bound by `in` is, or `None` after reporting."""
        scope = self.scope
        if isinstance(node, t.Name):
            if node.id in scope.sets:
                return SubjectTy(kind=scope.sets[node.id].type.element, origin=("set", node.id))
            slot = scope.subforms.get(node.id)
            if slot is not None:
                if slot.multiplicity == "one":
                    self.fail(
                        node,
                        Code.NOT_INDEX_SET,
                        f"sub-form slot `{slot.name}` is `one`: call it, `{slot.name}.output(...)`",
                    )
                    return None
                if slot.per == "subject":
                    self.fail(
                        node,
                        Code.NOT_INDEX_SET,
                        f"sub-form slot `{slot.name}` is chosen per subject: give the subject of "
                        f"each role, `{slot.name}(role=subject)`",
                    )
                    return None
                return ContribTy(slot=slot, bound=())
            self.fail(
                node,
                Code.NOT_INDEX_SET,
                f"`{node.id}` is not an index set (a set of the contract, `range(a, b)`, a "
                "family's index set or a `many` sub-form slot)",
            )
            return None
        if isinstance(node, t.Range):
            start = self.integer(node.start, env, "a range bound", static=True)
            stop = self.integer(node.stop, env, "a range bound", static=True)
            if start is None or stop is None:
                return None
            return NumTy(DIMENSIONLESS, integer=True, static=True)
        if isinstance(node, t.Call):
            try:
                ref = classify_call(scope, node, self.contributions(env))
            except RefError as error:
                self.fail(error.node, error.code, error.message)
                return None
            if isinstance(ref, FamilyRef):
                if not self.family_index_set(ref, node, env):
                    return None
                return NumTy(DIMENSIONLESS, integer=True, static=True)
            if isinstance(ref, SubformIter):
                return self.subform_iteration(ref, node, env)
            self.fail(node, Code.NOT_INDEX_SET, "an output call is a number, not an index set")
            return None
        self.fail(node, Code.NOT_INDEX_SET, "this is not an index set")
        return None

    def family_index_set(self, ref: FamilyRef, node: t.Node, env: Env) -> bool:
        ok = self.subjects_of(ref.group, ref.raw, node, env)
        if len(ref.family.indices) != 1 or ref.family.indices[0].type.text != "Integer":
            self.fail(
                node,
                Code.BAD_FAMILY,
                f"family `{ref.family.name}` has more than one index or an index that is not "
                "Integer: only a family with one Integer index has an index set to iterate",
            )
            return False
        return ok

    def subform_iteration(self, ref: SubformIter, node: t.Call, env: Env) -> Ty | None:
        slot = ref.slot
        if slot.multiplicity == "one":
            self.fail(node, Code.NOT_INDEX_SET, f"sub-form slot `{slot.name}` is `one`: call it")
            return None
        if slot.per != "subject":
            self.fail(
                node,
                Code.NOT_INDEX_SET,
                f"sub-form slot `{slot.name}` is chosen per model: iterate it as `{slot.name}`",
            )
            return None
        accepted = self.scope.decl.contracts[slot.accepts]
        expected = {f.name for f in accepted.roles}
        given = {k.name for k in ref.keywords}
        if given != expected:
            self.fail(
                node,
                Code.BAD_CALL,
                f"iterating `{slot.name}` gives the subject of each role of `{accepted.name}`: "
                f"{', '.join(sorted(expected)) or 'it has none'}",
            )
            return None
        ok = True
        for keyword in ref.keywords:
            role = next(f for f in accepted.roles if f.name == keyword.name)
            ty = self.visit(keyword.value, env)
            if (
                ty is None
                or not isinstance(ty, SubjectTy)
                or not self.scope.related(ty.kind, role.type.element)
            ):
                if ty is not None:
                    self.fail(
                        keyword.value,
                        Code.SLOT_SUBJECTS,
                        f"role `{role.name}` takes a subject of kind `{role.type.element}`, "
                        f"found {_describe(ty)}",
                    )
                ok = False
        return ContribTy(slot=slot, bound=tuple(sorted(expected))) if ok else None

    def bind(self, node: t.Node, target: str, env: Env) -> bool:
        scope = self.scope
        taken = (
            target in env
            or target in scope.arguments
            or target in scope.sets
            or target in scope.groups
            or target in scope.subforms
            or target in scope.locals
            or target in scope.unknowns
        )
        if taken:
            self.fail(
                node,
                Code.DUPLICATE_NAME,
                f"`{target}` is already a name in this form: an index variable needs a new one",
            )
        return not taken

    def reduce(self, node: t.Reduce, env: Env) -> Ty | None:
        inner = dict(env)
        ok = True
        for clause in node.clauses:
            ty = self.iterable(clause.iterable, inner)
            if ty is None:
                ok = False
                continue
            if not self.bind(clause, clause.target, inner):
                ok = False
                continue
            inner[clause.target] = ty
        if not ok:
            return None
        body = self.number(node.body, inner)
        if body is None:
            return None
        if node.kind == "prod" and body.dim != DIMENSIONLESS:
            self.fail(
                node,
                Code.DIMENSIONLESS_REQUIRED,
                f"a product over an index set has no fixed dimension: the body is dimension "
                f"{describe(body.dim)}, so it must be dimensionless",
            )
            return None
        return NumTy(body.dim, integer=body.integer, static=body.static)

    def comprehension(self, node: t.Comprehension, env: Env) -> Ty | None:
        inner = dict(env)
        over: list[str] = []
        ok = True
        for clause in node.clauses:
            ty = self.iterable(clause.iterable, inner)
            if ty is None:
                ok = False
                continue
            if not isinstance(ty, SubjectTy) or ty.origin[0] != "set":
                self.fail(
                    clause,
                    Code.NOT_INDEX_SET,
                    "a comprehension ranges over the sets of the contract",
                )
                ok = False
                continue
            if not self.bind(clause, clause.target, inner):
                ok = False
                continue
            inner[clause.target] = ty
            over.append(ty.origin[1])
        if not ok:
            return None
        body = self.number(node.body, inner)
        if body is None:
            return None
        if node.basis is not None and self.scope.decl.composition_basis_entity(node.basis) is None:
            role = self.scope.decl.framework.get(m.COMPOSITION_BASIS_ROLE)
            self.fail(
                node,
                Code.UNKNOWN_NAME,
                "`basis` needs the manifest's [framework] to bind `composition_basis`"
                if role is None
                else f"`{node.basis}` is not a declared entity of kind `{role}`",
            )
            return None
        return VecTy(body.dim, tuple(over), node.basis)

    def derivative(self, node: t.Derivative, env: Env) -> Ty | None:
        body = self.number(node.expr, env)
        wrt = self.derivative_target(node.wrt, env)
        if body is None or wrt is None:
            return None
        scope = self.scope
        if (
            isinstance(node.wrt, t.Name)
            and node.wrt.id in scope.locals
            and mentions(node.expr, {*scope.unknowns, *scope.dependent})
        ):
            self.fail(
                node,
                Code.BAD_DERIVATIVE,
                f"the expression depends on an unknown of an implicit block: differentiate "
                f"with respect to an argument, not the local `{node.wrt.id}`",
            )
            return None
        return NumTy(body.dim / wrt.dim)

    def derivative_target(self, node: t.Expr, env: Env) -> NumTy | None:
        scope = self.scope
        if isinstance(node, t.Name):
            if node.id in scope.locals and node.id in env:
                ty = env[node.id]
                return ty if isinstance(ty, NumTy) else None
            if node.id in scope.arguments and not scope.arguments[node.id].over:
                ty = self.visit(node, env)
                return ty if isinstance(ty, NumTy) else None
        elif isinstance(node, t.Subscript) and isinstance(node.base, t.Name):
            argument = scope.arguments.get(node.base.id)
            if argument is not None and argument.over:
                ty = self.indexed_argument(node, argument, env)
                return ty if isinstance(ty, NumTy) else None
        if isinstance(node, t.Name) and node.id in scope.locals and node.id not in env:
            if node.id not in self.failed_locals:
                self.fail(node, Code.LOCAL_ORDER, f"local `{node.id}` is used before it is defined")
            return None
        self.fail(
            node,
            Code.BAD_DERIVATIVE,
            "a derivative is taken with respect to an argument (subscripted when it is indexed) "
            "or an earlier local",
        )
        return None

    def at(self, node: t.At, env: Env) -> Ty | None:
        scope = self.scope
        family = node.family
        if not isinstance(family, t.Call):
            self.fail(
                family,
                Code.NOT_INDEX_SET,
                "the first argument of `at` is a family's index set, `group.family(subject)`",
            )
            return None
        try:
            ref = classify_call(scope, family, self.contributions(env))
        except RefError as error:
            self.fail(error.node, error.code, error.message)
            return None
        if not isinstance(ref, FamilyRef):
            self.fail(
                family,
                Code.NOT_INDEX_SET,
                "the first argument of `at` is a family's index set, `group.family(subject)`",
            )
            return None
        ok = self.family_index_set(ref, family, env)
        if ref.family.interval is None:
            self.fail(
                family,
                Code.BAD_FAMILY,
                f"family `{ref.family.name}` declares no `interval`, so `at` has nothing to select",
            )
            return None
        lower = next(s for s in ref.family.slots if s.name == ref.family.interval[0])
        expected = type_dimension(scope.decl, lower.type)
        value = self.number(node.value, env)
        if value is None or expected is None or not ok:
            return None
        if value.dim != expected:
            self.fail(
                node.value,
                Code.DIMENSION_MISMATCH,
                f"the interval of `{ref.family.name}` is dimension {describe(expected)}, "
                f"found dimension {describe(value.dim)}",
            )
            return None
        return NumTy(DIMENSIONLESS, integer=True)

    def position(self, node: t.Position, env: Env) -> Ty | None:
        """`position(t, s)`: a subject of kind `constituent_array` and a subject of kind `species`;
        an integer known without argument values, the position of `s` in `t`."""
        ok = True
        for operand, kind, what in (
            (
                node.array,
                pc.CONSTITUENT_ARRAY.declared,
                "the first argument of `position` is an array",
            ),
            (node.member, pc.SPECIES.declared, "the second argument of `position` is a species"),
        ):
            if not isinstance(operand, t.Name):
                self.fail(
                    operand,
                    Code.SLOT_SUBJECTS,
                    f"{what}: a role or an index variable of a set, found an expression",
                )
                ok = False
                continue
            ty = self.visit(operand, env)
            if ty is None:
                ok = False
            elif not isinstance(ty, SubjectTy):
                self.fail(operand, Code.SLOT_SUBJECTS, f"{what}, `{operand.id}` is {_describe(ty)}")
                ok = False
            elif not self.scope.decl.is_a(ty.kind, kind):
                self.fail(
                    operand,
                    Code.SLOT_SUBJECTS,
                    f"{what} of kind `{kind}`, `{operand.id}` is a subject of kind `{ty.kind}`",
                )
                ok = False
        return NumTy(DIMENSIONLESS, integer=True, static=True) if ok else None

    def integral(self, node: t.Integral, env: Env) -> Ty | None:
        lower = self.number(node.lower, env)
        upper = self.number(node.upper, env)
        if lower is None or upper is None:
            return None
        if lower.dim != upper.dim:
            self.fail(
                node,
                Code.DIMENSION_MISMATCH,
                f"the bounds of an integral have dimension {describe(lower.dim)} and "
                f"dimension {describe(upper.dim)}",
            )
            return None
        inner = dict(env)
        if not self.bind(node, node.var, inner):
            return None
        inner[node.var] = NumTy(lower.dim)
        body = self.number(node.body, inner)
        return None if body is None else NumTy(body.dim * lower.dim)


def _fold(op: str, left: int | float | None, right: int | float | None) -> int | float | None:
    """The value of an operation on literals, or `None` when either side is not one."""
    if left is None or right is None:
        return None
    try:
        if op == "+":
            return left + right
        if op == "-":
            return left - right
        if op == "*":
            return left * right
        if op == "/":
            return left / right
        result = left**right
    except ArithmeticError, ValueError:
        return None
    return result if isinstance(result, int | float) else None


# -- forms -----------------------------------------------------------------------------------


def _hygiene(checker: Checker) -> bool:
    """Names that can be written in an expression must be usable there. Reports each problem
    at the form; returns whether there were none."""
    scope = checker.scope
    form = scope.form
    problems: list[tuple[str, str]] = []
    top: dict[str, str] = {}
    sources: list[tuple[str, str, str]] = [
        *(("argument", f.name, f.construct) for f in scope.contract.arguments),
        *(("role", f.name, f.construct) for f in scope.contract.roles),
        *(("set", f.name, f.construct) for f in scope.contract.sets),
        *(("slot group", g.name, g.construct) for g in form.slot_groups),
        *(("sub-form slot", s.name, s.construct) for s in form.subforms),
        *(("local", local.name, local.construct) for local in form.locals),
        *(("unknown", u.name, u.construct) for b in form.implicit for u in b.unknowns),
    ]
    for category, name, construct in sources:
        if name == m.CONVENTION_NAME:
            problems.append(
                (
                    construct,
                    f"{category} `{name}` has the name the convention facts of a form are read "
                    f"through (`{m.CONVENTION_NAME}.<fact>`)",
                )
            )
        if name in top:
            article = "an" if top[name][0] in "aeiou" else "a"
            problems.append(
                (construct, f"{category} `{name}` has the name of {article} {top[name]}")
            )
        else:
            top[name] = category
        if name in t.FUNCTIONS or name in t.HEADS:
            problems.append(
                (construct, f"{category} `{name}` has the name of a function of the grammar")
            )
    attribute_names: list[tuple[str, str, str]] = []
    for group in form.slot_groups:
        attribute_names.append(("slot group", group.name, group.construct))
        attribute_names.extend(("slot", s.name, s.construct) for s in group.slots)
        for family in group.families:
            attribute_names.append(("family", family.name, family.construct))
            attribute_names.extend(("slot", s.name, s.construct) for s in family.slots)
    attribute_names.extend(("output", o.name, o.construct) for o in scope.contract.outputs)
    for category, name, construct in [*sources, *attribute_names]:
        if keyword.iskeyword(name) or name in ("True", "False", "None"):
            problems.append(
                (
                    construct,
                    f"{category} `{name}` is a Python keyword and cannot be written in an expression",
                )
            )
    seen: set[tuple[str, str]] = set()
    for construct, message in problems:
        if (construct, message) in seen:
            continue
        seen.add((construct, message))
        checker.diagnostics.append(
            Diagnostic(
                document=checker.document, construct=construct, code=Code.BAD_NAME, message=message
            )
        )
    return not problems


def _count(over: tuple[str, ...]) -> tuple[str, ...]:
    return tuple(sorted(over))


def _count_text(counts: Counter[tuple[str, ...]]) -> str:
    if not counts:
        return "0"
    terms = []
    for monomial, times in sorted(counts.items()):
        size = " * ".join(f"n({name})" for name in monomial) or "1"
        terms.append(size if times == 1 else f"{times} * {size}")
    return " + ".join(terms)


def _check_block(checker: Checker, block: m.ImplicitBlock, independent: Env) -> None:
    """The unknowns, bounds, residuals and selection of one implicit block (expressions.md
    section 3). `independent` is what the block's expressions can use: the contract's roles and
    the locals that do not depend on an unknown."""
    decl = checker.scope.decl
    checker.in_block = True
    try:
        own: Env = {}
        for unknown in block.unknowns:
            dim = type_dimension(decl, unknown.type)
            if dim is None:
                continue
            own[unknown.name] = VecTy(dim, unknown.over) if unknown.over else NumTy(dim)
        for unknown in block.unknowns:
            dim = type_dimension(decl, unknown.type)
            for label in ("lower", "upper", "start"):
                bound = getattr(unknown, label)
                if bound is None or bound.text is None:
                    continue
                construct = f"{unknown.construct}.{label}"
                ty, root = checker.check_text(bound.text, construct, independent)
                if ty is None:
                    continue
                if not isinstance(ty, NumTy):
                    checker.fail(
                        root,
                        Code.DIMENSION_MISMATCH,
                        f"a {label} is a number, found {_describe(ty)}",
                    )
                elif dim is not None and ty.dim != dim:
                    checker.construct = construct
                    checker.fail(
                        root,
                        Code.DIMENSION_MISMATCH,
                        f"the {label} of unknown `{unknown.name}` ({unknown.type.text}) has "
                        f"dimension {describe(dim)}, but the expression has dimension "
                        f"{describe(ty.dim)}",
                    )
            if unknown.start is None and (unknown.lower is None or unknown.upper is None):
                checker.diagnostics.append(
                    Diagnostic(
                        document=checker.document,
                        construct=unknown.construct,
                        code=Code.BAD_IMPLICIT,
                        message=f"unknown `{unknown.name}` has no `start` and lacks a bound: "
                        "give a `start`, or both `lower` and `upper` (the start is then their midpoint)",
                    )
                )
        inside: Env = {**independent, **own}
        unknown_count: Counter[tuple[str, ...]] = Counter(_count(u.over) for u in block.unknowns)
        residual_count: Counter[tuple[str, ...]] = Counter()
        counted = True
        for residual in block.residuals:
            ty, root = checker.check_text(residual.text, residual.construct, inside, residual=True)
            if ty is None:
                counted = False
            elif isinstance(ty, NumTy):
                residual_count[()] += 1
            elif isinstance(ty, VecTy):
                residual_count[_count(ty.over)] += 1
            else:
                checker.fail(
                    root, Code.DIMENSION_MISMATCH, f"a residual is a number, found {_describe(ty)}"
                )
                counted = False
        if counted and unknown_count != residual_count:
            checker.diagnostics.append(
                Diagnostic(
                    document=checker.document,
                    construct=f"{block.construct}.residuals",
                    code=Code.RESIDUAL_COUNT,
                    message=f"implicit block `{block.name}` has as many residuals as unknowns "
                    f"after expansion: the unknowns expand to {_count_text(unknown_count)}, "
                    f"the residuals to {_count_text(residual_count)}",
                )
            )
        if block.select != "unique":
            (only,) = block.unknowns if len(block.unknowns) == 1 else (None,)
            if only is None or only.over or only.lower is None or only.upper is None:
                checker.diagnostics.append(
                    Diagnostic(
                        document=checker.document,
                        construct=f"{block.construct}.select",
                        code=Code.BAD_IMPLICIT,
                        message=f"`select = {block.select}` picks among the roots found in an "
                        "interval: the block has exactly one unknown, not over a set, with both "
                        "`lower` and `upper`",
                    )
                )
        if block.select_by is not None:
            ty, root = checker.check_text(block.select_by, f"{block.construct}.select", inside)
            if ty is not None and not isinstance(ty, NumTy):
                checker.fail(
                    root, Code.DIMENSION_MISMATCH, f"`by` is a number, found {_describe(ty)}"
                )
    finally:
        checker.in_block = False


def check_form(decl: m.Declaration, form: m.Form) -> list[Diagnostic]:
    """Every refusal of the expressions of `form` (expressions.md section 4)."""
    scope = FormScope.of(decl, form)
    document = decl.modules[form.module].document
    checker = Checker(scope, document)
    contract = scope.contract
    declared = {output.name for output in form.outputs}
    wanted = {field.name: field for field in contract.outputs}
    for output in form.outputs:
        if output.name not in wanted:
            checker.diagnostics.append(
                Diagnostic(
                    document=document,
                    construct=output.construct,
                    code=Code.UNKNOWN_OUTPUT,
                    message=f"contract `{contract.name}` has no output `{output.name}` "
                    f"(outputs: {', '.join(wanted)})",
                )
            )
    if form.status != "catalogued":
        for name in wanted:
            if name not in declared:
                checker.diagnostics.append(
                    Diagnostic(
                        document=document,
                        construct=f"{form.construct}.outputs",
                        code=Code.MISSING_OUTPUT,
                        message=f"form `{form.name}` is `{form.status}` but has no expression "
                        f"for output `{name}` of contract `{contract.name}`",
                    )
                )
    if not (form.locals or form.outputs or form.implicit):
        return sorted(set(checker.diagnostics))
    _hygiene(checker)
    env = checker.base_env()
    for unknown_name, (_, unknown) in scope.unknowns.items():
        dim = type_dimension(decl, unknown.type)
        if dim is not None:
            env[unknown_name] = VecTy(dim, unknown.over) if unknown.over else NumTy(dim)
    for local in form.locals:
        ty, root = checker.check_text(local.text, local.construct, env)
        if ty is None:
            checker.failed_locals.add(local.name)
        elif not isinstance(ty, NumTy | VecTy):
            checker.fail(
                root,
                Code.DIMENSION_MISMATCH,
                f"a local is a number or an indexed quantity, found {_describe(ty)}",
            )
            checker.failed_locals.add(local.name)
        else:
            env[local.name] = ty
    independent: Env = {
        name: ty
        for name, ty in env.items()
        if name in scope.roles or (name in scope.locals and name not in scope.dependent)
    }
    for block in form.implicit:
        _check_block(checker, block, independent)
    for output in form.outputs:
        field = wanted.get(output.name)
        ty, root = checker.check_text(output.text, output.construct, env)
        if ty is None or field is None:
            continue
        if not isinstance(ty, NumTy):
            checker.fail(
                root, Code.OUTPUT_DIMENSION, f"an output is a number, found {_describe(ty)}"
            )
            continue
        expected = type_dimension(decl, field.type, field.extra_order)
        if expected is not None and ty.dim != expected:
            checker.fail(
                root,
                Code.OUTPUT_DIMENSION,
                f"output `{output.name}` of contract `{contract.name}` is {field.type.text} "
                f"(dimension {describe(expected)}), but the expression has dimension "
                f"{describe(ty.dim)}",
            )
    return sorted(set(checker.diagnostics))


def check_declaration(decl: m.Declaration) -> list[Diagnostic]:
    """Every refusal of the expressions of every form of `decl`, in form order."""
    found: list[Diagnostic] = []
    for form in decl.forms.values():
        found.extend(check_form(decl, form))
    return found
