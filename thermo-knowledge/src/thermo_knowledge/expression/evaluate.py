# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The reference evaluator (expressions.md section 5): it exists to qualify forms, not to be fast.

1. Bind: a form, its subjects (the roles and the ordered sets of its contract), a parameter
   source and, through the source, the forms chosen for its sub-form slots.
2. Expand: sums, products and iterations run over the concrete subjects, and the result is one
   SymPy expression per output in the argument symbols and parameter symbols. A slot value, a
   family-row value and a `unit(...)` conversion factor are each a parameter symbol; the stored
   number is kept beside it (in storage units) and is never put into the expression. A decision
   that depends on a stored value (a conditional on static values, the order of the pieces of
   `at`) is made here, from the values, and only its consequence is in the expression. A
   transposition is not: a parameter source returns the values of the order it is asked for. Locals stay symbols until the output is finished, so that
   `d(expr, local)` is a partial derivative; a call to a sub-form is expanded in its own frame,
   given the sets and the indexed values its caller passed. An implicit block contributes one
   symbol for each scalar unknown and a `Block` (residuals, bounds, selection) that is solved
   numerically at each point of argument values.
3. Differentiate symbolically where `d(...)` occurs; through an implicit block by the
   implicit-function theorem.
4. Compile with `lambdify` to NumPy (SciPy for the special functions), once for each structure
   (`CompileCache`), solve the blocks the output needs at each point and evaluate at arrays,
   passing the parameter values beside the argument arrays.

Stored values and arguments are in coherent SI storage units and `unit(...)` literals are
converted to them, so evaluation has no unit handling of its own.
"""

from __future__ import annotations

import functools
import hashlib
import itertools
from collections.abc import Callable, Hashable, Iterator, Mapping, Sequence
from dataclasses import dataclass, field

import numpy as np
import scipy.integrate
import sympy
from sympy.core.function import ArgumentIndexError

from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge import transposition
from thermo_knowledge.declaration import model as m
from thermo_knowledge.expression import tree as t
from thermo_knowledge.expression.compiled import CompileCache
from thermo_knowledge.expression.parameters import (
    ConventionFact,
    FamilyRows,
    FormChoice,
    ParameterSource,
    RecordValidity,
    SetRead,
    SlotValues,
    Subject,
)
from thermo_knowledge.expression.implicit import Block, BlockSolver, SolveFailure
from thermo_knowledge.expression.parser import parse_cached, parse_residual
from thermo_knowledge.expression.scope import (
    FamilyRef,
    FormScope,
    OutputCall,
    SlotRef,
    SubformIter,
    classify_call,
    classify_slot,
)
from thermo_knowledge.expression.units import unit_literal
from thermo_knowledge.expression.validity import membership

_PRECISION = 17
_SIMPLE = {
    "exp": sympy.exp,
    "log": sympy.log,
    "sqrt": sympy.sqrt,
    "abs": sympy.Abs,
    "sinh": sympy.sinh,
    "cosh": sympy.cosh,
    "tanh": sympy.tanh,
    "sin": sympy.sin,
    "cos": sympy.cos,
    "atan": sympy.atan,
    "erf": sympy.erf,
}


class EvaluationRefusal(Exception):
    """Evaluation was refused: a parameter set is missing, a value is outside the domain of the
    form, or something the form needs is not bound. Never answered with a zero."""


_parsed = parse_cached


@functools.cache
def _parsed_residual(text: str) -> t.Expr:
    return parse_residual(text)


# -- numbers -------------------------------------------------------------------------------------

type Value = int | sympy.Expr | str | bool | Contribution
"""What an expression is while it is expanded: a Python int for index arithmetic, a SymPy
expression for any other number, a subject (text), a condition or a contribution."""


@dataclass(frozen=True)
class Contribution:
    """One chosen form of a `many` (or `optional`) sub-form slot, with the subjects of the roles
    its iteration fixed."""

    slot: m.SubformSlot
    choice: FormChoice
    roles: Mapping[str, Subject]


def _normal(value: Value) -> Value:
    if isinstance(value, sympy.Basic) and value.is_Integer:
        return int(value)
    return value


def _expr(value: Value) -> sympy.Expr:
    if isinstance(value, bool) or isinstance(value, str | Contribution):
        raise EvaluationRefusal(f"{value!r} is not a number")
    if isinstance(value, int):
        return sympy.Integer(value)
    return value


def _float(value: float) -> sympy.Float:
    return sympy.Float(value, _PRECISION)


def _truth(value: sympy.Basic | bool) -> bool | sympy.Basic:
    if value is sympy.true or value is True:
        return True
    if value is sympy.false or value is False:
        return False
    return value


class _Debye(sympy.Function):
    """The Debye function `D_n(x) = n / x**n * integral(t**n / (exp(t) - 1), t, 0, x)`,
    evaluated by quadrature."""

    nargs = 2

    def fdiff(self, argindex: int = 2) -> sympy.Expr:
        if argindex != 2:
            raise ArgumentIndexError(self, argindex)
        n, x = self.args
        return -n / x * self + n / (sympy.exp(x) - 1)


def _debye_value(n: float, x: float) -> float:
    if x == 0:
        return 1.0
    integral, _ = scipy.integrate.quad(
        lambda s: s**n / np.expm1(s), 0.0, x, epsabs=0.0, epsrel=1e-13, limit=500
    )
    return float(n * integral / x**n)


_debye_numeric = np.vectorize(_debye_value, otypes=[float])


# -- frames --------------------------------------------------------------------------------------


@dataclass
class _Frame:
    """The form being expanded and what it is bound to."""

    scope: FormScope
    roles: Mapping[str, Subject]
    sets: Mapping[str, tuple[Subject, ...]]
    args: dict[str, sympy.Symbol | dict[tuple[Subject, ...], sympy.Symbol]]
    source: ParameterSource
    local_symbols: dict[str, sympy.Symbol] = field(default_factory=dict)
    local_values: dict[str, Value] = field(default_factory=dict)
    local_guards: dict[str, list[tuple[str, sympy.Basic]]] = field(default_factory=dict)
    guards: list[tuple[str, sympy.Basic]] = field(default_factory=list)
    path: sympy.Basic | bool = True
    locals_done: bool = False
    vectors: dict[str, dict[tuple[Subject, ...], str]] = field(default_factory=dict)
    """Indexed locals: for each, the name under which each element is held in `local_values`."""
    unknowns: dict[str, sympy.Symbol | dict[tuple[Subject, ...], sympy.Symbol]] = field(
        default_factory=dict
    )
    blocks: list[Block] = field(default_factory=list)
    """The implicit blocks of this form and, substituted, of the sub-forms it called."""


class _Machine:
    """Expands forms to SymPy; one per bound form, shared by the frames of its sub-forms."""

    def __init__(self, decl: m.Declaration, cache: CompileCache) -> None:
        self.decl = decl
        self.cache = cache
        self.counter = itertools.count(1)
        self.functions: dict[str, Callable[..., np.ndarray]] = {"_Debye": _debye_numeric}
        self.parameters: dict[Hashable, sympy.Symbol] = {}
        """The parameter symbol of each stored value the expansion has read, by what it was read
        from (never by its value)."""
        self.sources: dict[int, ParameterSource] = {}
        self.reads: dict[int, list[SetRead]] = {}
        """The sets each source supplied to this expansion: the slot group and the subjects as
        asked for, in reading order."""
        self.conventions: dict[str, dict[int, ParameterSource]] = {}
        """For each convention fact the expansion reads, the sources of the frames that read it."""
        self.values: dict[sympy.Symbol, float] = {}
        """The number each parameter symbol stands for, in storage units, as bound."""
        self.numbers: dict[sympy.Basic, sympy.Basic] = {}
        """The same as SymPy numbers, to decide a condition on static values."""

    # -- frames and outputs ----------------------------------------------------------------

    def fresh(self, name: str, **assumptions: bool) -> sympy.Symbol:
        return sympy.Symbol(f"{name}#{next(self.counter)}", real=True, **assumptions)

    def parameter(self, frame: _Frame, origin: Hashable, label: str, value: float) -> sympy.Symbol:
        """The symbol for the stored `value` read from `origin` (a slot or a row of a family for
        given subjects, through the frame's source). Reading the same thing twice gives the
        same symbol; the name orders the symbols by first reading, so that two bindings of one
        structure name them alike."""
        self.sources[id(frame.source)] = frame.source  # kept alive: an identity is never reused
        key = (id(frame.source), origin)
        symbol = self.parameters.get(key)
        if symbol is None:
            symbol = sympy.Symbol(f"param:{label}#{len(self.parameters) + 1}", real=True)
            self.parameters[key] = symbol
            self.values[symbol] = value
            self.numbers[symbol] = _float(value)
        return symbol

    def convention(self, frame: _Frame, name: str) -> sympy.Symbol:
        """The symbol for the convention fact `name`: one symbol however many frames read it. Its
        value is set when the expansion is finished (`resolve_conventions`), once every set the
        frames read is known."""
        self.sources[id(frame.source)] = frame.source
        self.conventions.setdefault(name, {})[id(frame.source)] = frame.source
        key = ("convention", name)
        symbol = self.parameters.get(key)
        if symbol is None:
            symbol = sympy.Symbol(f"param:convention.{name}#{len(self.parameters) + 1}", real=True)
            self.parameters[key] = symbol
        return symbol

    def resolve_conventions(self) -> None:
        """Give each convention fact the expansion reads its value, from the parameterizations
        that supplied the sets it read. Every source states the fact; two parameterizations that
        state different values are refused, naming the fact, both and both values."""
        for name, sources in self.conventions.items():
            facts: list[ConventionFact] = []
            for source in sources.values():
                facts.extend(source.convention_facts(name, tuple(self.reads.get(id(source), ()))))
            for fact in facts:
                if fact.value is None:
                    raise EvaluationRefusal(
                        f"parameterization `{fact.parameterization}` {fact.absent}: a form reads "
                        f"the convention fact `{name}`"
                    )
            first = next(iter(facts))
            for other in facts:
                if other.value != first.value:
                    raise EvaluationRefusal(
                        f"the convention fact `{name}` differs between the parameterizations an "
                        f"evaluation draws sets from: `{first.parameterization}` has "
                        f"{first.value!r}, `{other.parameterization}` has {other.value!r}"
                    )
            symbol = self.parameters[("convention", name)]
            assert first.value is not None
            self.values[symbol] = first.value
            self.numbers[symbol] = _float(first.value)

    def decide(self, frame: _Frame, condition: Value) -> bool | sympy.Basic:
        """`condition` as a truth value when it is one: already, or because it depends on
        stored values alone, which are known here."""
        truth = _truth(condition)  # type: ignore[arg-type]
        if isinstance(truth, bool):
            return truth
        inlined = self.inline(frame, truth)
        free = inlined.free_symbols
        if free and free <= set(self.values):
            decided = _truth(inlined.xreplace(self.numbers))
            if isinstance(decided, bool):
                return decided
        return truth

    def describe(self, expr: sympy.Basic) -> str:
        """`expr` for a message: the stored values in place of their symbols."""
        return str(expr.xreplace(self.numbers))

    def output(self, frame: _Frame, name: str) -> tuple[sympy.Expr, list[tuple[str, sympy.Basic]]]:
        """The expression of output `name` of the frame's form in its argument symbols, and the
        domain conditions that must hold wherever it is evaluated."""
        form = frame.scope.form
        definition = next((o for o in form.outputs if o.name == name), None)
        if definition is None:
            raise EvaluationRefusal(
                f"form `{form.name}` has no expression for output `{name}`"
                + (" (it is catalogued)" if form.status == "catalogued" else "")
            )
        self.locals(frame)
        frame.guards = []
        frame.path = True
        raw = _expr(self.ev(frame, _parsed(definition.text), {}))
        guards = list(frame.guards)
        used = self.used_locals(frame, raw)
        for local in used:
            guards.extend(g for g in frame.local_guards.get(local, []) if g not in guards)
        inlined = self.inline(frame, raw)
        for block in frame.blocks:
            if set(block.unknowns) & inlined.free_symbols:
                guards.extend(g for g in block.guards if g not in guards)
        return inlined, [(text, self.inline(frame, cond)) for text, cond in guards]

    def locals(self, frame: _Frame) -> None:
        """Expand the locals and implicit blocks of the frame's form, once. The locals that do not
        depend on an unknown come first, then the blocks, then the locals that do."""
        if frame.locals_done:
            return
        frame.locals_done = True
        form = frame.scope.form
        dependent = frame.scope.dependent
        for local in form.locals:
            if local.name not in dependent:
                self.define_local(frame, local)
        for block in form.implicit:
            self.build_block(frame, block)
        for local in form.locals:
            if local.name in dependent:
                self.define_local(frame, local)

    def define_local(self, frame: _Frame, local: m.ExpressionDef) -> None:
        tree = _parsed(local.text)
        frame.guards = []
        frame.path = True
        if isinstance(tree, t.Comprehension):
            elements = self.vector(frame, tree, {})
            frame.vectors[local.name] = {}
            for key, element in elements.items():
                flat = f"{local.name}[{','.join(key)}]"
                frame.vectors[local.name][key] = flat
                self.hold(frame, flat, _normal(element), list(frame.guards), local.name)
            return
        self.hold(
            frame, local.name, _normal(self.ev(frame, tree, {})), list(frame.guards), local.name
        )

    def hold(
        self,
        frame: _Frame,
        name: str,
        value: Value,
        guards: list[tuple[str, sympy.Basic]],
        label: str,
    ) -> None:
        """Hold a local under `name`. Its symbol is named from `label`, which does not name a
        subject: the structure of an expansion must not depend on which subjects it is for."""
        frame.local_guards[name] = guards
        frame.local_values[name] = value
        if not isinstance(value, int):
            frame.local_symbols[name] = self.fresh(f"local:{label}")

    def held_value(self, frame: _Frame, flat: str) -> Value:
        value = frame.local_values[flat]
        return value if isinstance(value, int) else frame.local_symbols[flat]

    def build_block(self, frame: _Frame, block: m.ImplicitBlock) -> None:
        """Expand an implicit block over the concrete sets into a `Block`: a symbol for each
        scalar unknown, the residuals, the bounds and starts and the selection."""
        unknowns: list[sympy.Symbol] = []
        labels: list[str] = []
        lower: list[sympy.Expr | None] = []
        upper: list[sympy.Expr | None] = []
        start: list[sympy.Expr | None] = []
        frame.guards = []
        frame.path = True

        def bound(value: m.Bound | None) -> sympy.Expr | None:
            if value is None:
                return None
            if value.number is not None:
                return _float(value.number)
            assert value.text is not None
            return _expr(self.ev(frame, _parsed(value.text), {}))

        # Every unknown has its symbol before any expression is expanded: residuals name them.
        for unknown in block.unknowns:
            if unknown.over:
                table: dict[tuple[Subject, ...], sympy.Symbol] = {}
                for key in itertools.product(*(frame.sets[name] for name in unknown.over)):
                    symbol = self.fresh(unknown.name)
                    table[key] = symbol
                    unknowns.append(symbol)
                    labels.append(f"{unknown.name}[{', '.join(key)}]")
                frame.unknowns[unknown.name] = table
            else:
                symbol = self.fresh(unknown.name)
                frame.unknowns[unknown.name] = symbol
                unknowns.append(symbol)
                labels.append(unknown.name)
        for unknown in block.unknowns:
            count = len(frame.unknowns[unknown.name]) if unknown.over else 1  # type: ignore[arg-type]
            lower.extend([bound(unknown.lower)] * count)
            upper.extend([bound(unknown.upper)] * count)
            start.extend([bound(unknown.start)] * count)
        residuals: list[sympy.Expr] = []
        for residual in block.residuals:
            tree = _parsed_residual(residual.text)
            if isinstance(tree, t.Comprehension):
                residuals.extend(_expr(v) for v in self.vector(frame, tree, {}).values())
            else:
                residuals.append(_expr(self.ev(frame, tree, {})))
        if len(residuals) != len(unknowns):
            raise EvaluationRefusal(
                f"implicit block `{block.name}` of form `{block.form}` expands to "
                f"{len(residuals)} residuals for {len(unknowns)} unknowns"
            )
        by = None
        if block.select_by is not None:
            by = _expr(self.ev(frame, _parsed(block.select_by), {}))
        # Midpoint starts for the unknowns that have none (the checker needs both bounds then).
        start = [
            s
            if s is not None
            else (low + high) / 2
            if low is not None and high is not None
            else None
            for s, low, high in zip(start, lower, upper)
        ]
        frame.blocks.append(
            Block(
                form=block.form,
                name=block.name,
                unknowns=tuple(unknowns),
                labels=tuple(labels),
                residuals=tuple(self.inline(frame, r) for r in residuals),  # type: ignore[misc]
                lower=tuple(None if e is None else self.inline(frame, e) for e in lower),  # type: ignore[misc]
                upper=tuple(None if e is None else self.inline(frame, e) for e in upper),  # type: ignore[misc]
                start=tuple(None if e is None else self.inline(frame, e) for e in start),  # type: ignore[misc]
                select=block.select,
                by=None if by is None else self.inline(frame, by),  # type: ignore[arg-type]
                guards=[(text, self.inline(frame, cond)) for text, cond in frame.guards],
            )
        )

    def vector(
        self, frame: _Frame, node: t.Comprehension, env: dict[str, Value]
    ) -> dict[tuple[Subject, ...], Value]:
        """The elements of a comprehension, keyed by the subjects of its index variables."""
        result: dict[tuple[Subject, ...], Value] = {}
        names = [clause.target for clause in node.clauses]

        def loop(position: int, scope: dict[str, Value]) -> None:
            if position == len(node.clauses):
                key = tuple(str(scope[name]) for name in names)
                result[key] = self.ev(frame, node.body, scope)
                return
            clause = node.clauses[position]
            for item in self.iterate(frame, clause.iterable, scope):
                loop(position + 1, {**scope, clause.target: item})

        loop(0, env)
        return result

    def vector_value(
        self, frame: _Frame, node: t.Expr, env: dict[str, Value]
    ) -> dict[tuple[Subject, ...], sympy.Expr]:
        """What an indexed argument of a call is given: a comprehension, or an indexed argument,
        local or unknown by name."""
        if isinstance(node, t.Comprehension):
            return {key: _expr(value) for key, value in self.vector(frame, node, env).items()}
        assert isinstance(node, t.Name)
        if node.id in frame.vectors:
            return {
                key: _expr(self.held_value(frame, flat))
                for key, flat in frame.vectors[node.id].items()
            }
        table = frame.args[node.id] if node.id in frame.args else frame.unknowns.get(node.id)
        if not isinstance(table, dict):
            raise EvaluationRefusal(f"`{node.id}` is not an indexed value")
        return dict(table)

    def local_definitions(self, frame: _Frame) -> Iterator[tuple[str, sympy.Symbol, sympy.Expr]]:
        for name, symbol in frame.local_symbols.items():
            yield name, symbol, _expr(frame.local_values[name])

    def inline(self, frame: _Frame, expr: sympy.Basic) -> sympy.Basic:
        """`expr` with every local replaced by its definition."""
        for _, symbol, definition in reversed(list(self.local_definitions(frame))):
            expr = expr.xreplace({symbol: definition})
        return expr

    def used_locals(self, frame: _Frame, expr: sympy.Basic) -> list[str]:
        """The locals `expr` depends on, directly or through other locals."""
        by_symbol = {symbol: name for name, symbol, _ in self.local_definitions(frame)}
        definitions = {name: definition for name, _, definition in self.local_definitions(frame)}
        found: list[str] = []
        pending = [by_symbol[s] for s in expr.free_symbols if s in by_symbol]
        while pending:
            name = pending.pop()
            if name in found:
                continue
            found.append(name)
            pending.extend(by_symbol[s] for s in definitions[name].free_symbols if s in by_symbol)
        return found

    def inline_dependents(self, frame: _Frame, expr: sympy.Basic, local: str) -> sympy.Basic:
        """`expr` with the locals that depend on `local` replaced by their definitions, so a
        derivative with respect to `local` follows them and holds the other locals fixed."""
        dependent = {frame.local_symbols[local]}
        chain: list[tuple[sympy.Symbol, sympy.Expr]] = []
        for name, symbol, definition in self.local_definitions(frame):
            if name != local and definition.free_symbols & dependent:
                dependent.add(symbol)
                chain.append((symbol, definition))
        for symbol, definition in reversed(chain):
            expr = expr.xreplace({symbol: definition})
        return expr

    # -- expansion -------------------------------------------------------------------------

    def ev(self, frame: _Frame, node: t.Expr, env: dict[str, Value]) -> Value:
        match node:
            case t.Num():
                return node.value if isinstance(node.value, int) else _float(node.value)
            case t.UnitLiteral():
                factor, _ = unit_literal(node.text)
                return self.parameter(frame, ("unit", node.text), f"unit({node.text})", factor)
            case t.Name():
                return self.name(frame, node.id, env)
            case t.Attribute() | t.Subscript():
                return self.reference(frame, node, env)
            case t.Call():
                return self.call(frame, node, env)
            case t.Func():
                return self.function(frame, node, env)
            case t.BinOp():
                return self.binary(
                    node.op, self.ev(frame, node.left, env), self.ev(frame, node.right, env)
                )
            case t.UnaryOp():
                return self.unary(frame, node, env)
            case t.Compare():
                return self.compare(
                    frame, node, self.ev(frame, node.left, env), self.ev(frame, node.right, env)
                )
            case t.BoolOp():
                return self.boolean(frame, node, env)
            case t.IfExp():
                return self.conditional(frame, node, env)
            case t.Reduce():
                return self.reduce(frame, node, env)
            case t.Derivative():
                return self.derivative(frame, node, env)
            case t.At():
                return self.at(frame, node, env)
            case t.Integral():
                return self.integral(frame, node, env)
        raise EvaluationRefusal(f"cannot evaluate {type(node).__name__}")

    def name(self, frame: _Frame, ident: str, env: dict[str, Value]) -> Value:
        if ident in env:
            return env[ident]
        if ident in frame.local_values:
            value = frame.local_values[ident]
            return value if isinstance(value, int) else frame.local_symbols[ident]
        if ident in frame.roles:
            return frame.roles[ident]
        argument = frame.args.get(ident)
        if isinstance(argument, sympy.Symbol):
            return argument
        unknown = frame.unknowns.get(ident)
        if isinstance(unknown, sympy.Symbol):
            return unknown
        raise EvaluationRefusal(f"`{ident}` has no value here")

    def binary(self, op: str, left: Value, right: Value) -> Value:
        if isinstance(left, int) and isinstance(right, int) and not isinstance(left, bool):
            if op == "+":
                return left + right
            if op == "-":
                return left - right
            if op == "*":
                return left * right
            if op == "/":
                if right == 0:
                    raise EvaluationRefusal("division by zero in index arithmetic")
                return _normal(sympy.Rational(left, right))
            if right >= 0:
                return left**right
        x, y = _expr(left), _expr(right)
        if op == "+":
            return _normal(x + y)
        if op == "-":
            return _normal(x - y)
        if op == "*":
            return _normal(x * y)
        if op == "/":
            return _normal(x / y)
        return _normal(x**y)

    def unary(self, frame: _Frame, node: t.UnaryOp, env: dict[str, Value]) -> Value:
        operand = self.ev(frame, node.operand, env)
        if node.op == "not":
            truth = self.decide(frame, operand)
            return (not truth) if isinstance(truth, bool) else sympy.Not(truth)
        if node.op == "+":
            return operand
        if isinstance(operand, int):
            return -operand
        return _normal(-_expr(operand))

    def compare(self, frame: _Frame, node: t.Compare, left: Value, right: Value) -> Value:
        if isinstance(left, str) or isinstance(right, str):
            return (left == right) if node.op == "==" else (left != right)
        if isinstance(left, int) and isinstance(right, int):
            return {
                "<": left < right,
                "<=": left <= right,
                ">": left > right,
                ">=": left >= right,
                "==": left == right,
                "!=": left != right,
            }[node.op]
        x, y = _expr(left), _expr(right)
        relation = {
            "<": sympy.Lt,
            "<=": sympy.Le,
            ">": sympy.Gt,
            ">=": sympy.Ge,
            "==": sympy.Eq,
            "!=": sympy.Ne,
        }[node.op](x, y)
        return self.decide(frame, relation)

    def boolean(self, frame: _Frame, node: t.BoolOp, env: dict[str, Value]) -> Value:
        is_and = node.op == "and"
        symbolic: list[sympy.Basic] = []
        for element in node.values:
            truth = self.decide(frame, self.ev(frame, element, env))
            if truth is True:
                if not is_and:
                    return True
            elif truth is False:
                if is_and:
                    return False
            else:
                symbolic.append(truth)  # type: ignore[arg-type]
        if not symbolic:
            return is_and
        return sympy.And(*symbolic) if is_and else sympy.Or(*symbolic)

    def conditional(self, frame: _Frame, node: t.IfExp, env: dict[str, Value]) -> Value:
        test = self.decide(frame, self.ev(frame, node.test, env))
        if test is True:
            return self.ev(frame, node.body, env)
        if test is False:
            return self.ev(frame, node.orelse, env)
        saved = frame.path
        frame.path = sympy.And(saved, test)
        body = _expr(self.ev(frame, node.body, env))
        frame.path = sympy.And(saved, sympy.Not(test))
        orelse = _expr(self.ev(frame, node.orelse, env))
        frame.path = saved
        return _normal(sympy.Piecewise((body, test), (orelse, True)))

    def function(self, frame: _Frame, node: t.Func, env: dict[str, Value]) -> Value:
        values = [self.ev(frame, a, env) for a in node.args]
        name = node.name
        if name in _SIMPLE:
            return _normal(_SIMPLE[name](_expr(values[0])))
        if name == "log10":
            return _normal(sympy.log(_expr(values[0]), 10))
        if name in ("min", "max"):
            function = sympy.Min if name == "min" else sympy.Max
            return _normal(function(*(_expr(v) for v in values)))
        order = values[0]
        if not isinstance(order, int):
            raise EvaluationRefusal(f"the order of `{name}` is not an integer known at expansion")
        if name == "chebyshev_t":
            return _normal(sympy.chebyshevt(order, _expr(values[1])))
        return _normal(_Debye(sympy.Integer(order), _expr(values[1])))

    # -- references ------------------------------------------------------------------------

    def subjects(
        self, frame: _Frame, raw: Sequence[t.Expr], env: dict[str, Value]
    ) -> tuple[Subject, ...]:
        result: list[Subject] = []
        for node in raw:
            value = self.ev(frame, node, env)
            if not isinstance(value, str):
                raise EvaluationRefusal("a subject is expected here")
            result.append(value)
        return tuple(result)

    def reference(
        self, frame: _Frame, node: t.Attribute | t.Subscript, env: dict[str, Value]
    ) -> Value:
        scope = frame.scope
        if (
            isinstance(node, t.Attribute)
            and isinstance(node.value, t.Name)
            and node.value.id == m.CONVENTION_NAME
            and m.CONVENTION_NAME not in env
        ):
            if node.attr not in scope.conventions:
                raise EvaluationRefusal(
                    f"form `{scope.form.name}` does not declare the convention fact `{node.attr}`"
                )
            return self.convention(frame, node.attr)
        if isinstance(node, t.Subscript) and isinstance(node.base, t.Name):
            base = node.base.id
            key = self.subjects(frame, node.indices, env)
            if base in frame.vectors:
                if key not in frame.vectors[base]:
                    raise EvaluationRefusal(
                        f"local `{base}` has no element for subject ({', '.join(key)})"
                    )
                return self.held_value(frame, frame.vectors[base][key])
            table = frame.args[base] if base in frame.args else frame.unknowns[base]
            assert isinstance(table, dict)
            if key not in table:
                raise EvaluationRefusal(
                    f"argument `{base}` has no value for subject ({', '.join(key)})"
                )
            return table[key]
        ref = classify_slot(scope, node)
        assert ref is not None
        subjects = self.subjects(frame, ref.subjects, env)
        indices = [self.ev(frame, i, env) for i in ref.indices]
        return self.slot_value(frame, ref, subjects, indices)

    def check_diagonal(self, group: m.SlotGroup, subjects: tuple[Subject, ...]) -> None:
        if transposition.is_diagonal(group, subjects):
            raise EvaluationRefusal(
                f"slot group `{group.qualified}` forbids the diagonal: its subjects "
                f"({', '.join(subjects)}) name one instance twice"
            )

    def held[T](
        self,
        frame: _Frame,
        group: m.SlotGroup,
        subjects: tuple[Subject, ...],
        read: Callable[[tuple[Subject, ...]], T | None],
        what: str,
        default: Callable[[], T | None] | None = None,
    ) -> T:
        """What `read` finds for `subjects`, in the order asked for: the source applies the
        transposition of the group when it holds the set for another order (`transposition.py`),
        so nothing here does. Else the declared default, else a refusal naming the slot group and
        the subject."""
        self.check_diagonal(group, subjects)
        try:
            found = read(subjects)
        except transposition.TranspositionError as error:
            raise EvaluationRefusal(
                f"the parameter set of slot group `{group.qualified}` for subject "
                f"({', '.join(subjects)}) {error}"
            ) from None
        if found is not None:
            self.sources[id(frame.source)] = frame.source
            self.reads.setdefault(id(frame.source), []).append((group.qualified, subjects))
            return found
        fallback = default() if default is not None else None
        if fallback is not None:
            return fallback
        raise EvaluationRefusal(
            f"no {what} of slot group `{group.qualified}` for subject ({', '.join(subjects)})"
        )

    def slot_value(
        self, frame: _Frame, ref: SlotRef, subjects: tuple[Subject, ...], indices: list[Value]
    ) -> Value:
        group, slot = ref.group, ref.slot
        source = frame.source
        if ref.family is None:
            values = self.held(
                frame,
                group,
                subjects,
                lambda candidate: source.slot_values(group.qualified, candidate),
                "parameter set",
                lambda: source.default_slot_values(group.qualified, subjects),
            )
            if slot.name not in values:
                raise EvaluationRefusal(
                    f"the parameter set of slot group `{group.qualified}` for subject "
                    f"({', '.join(subjects)}) holds no value for slot `{slot.name}`"
                )
            return self.parameter(
                frame,
                ("slot", group.qualified, subjects, slot.name),
                f"{group.qualified}.{slot.name}",
                float(values[slot.name]),
            )
        family = ref.family
        rows = self.held(
            frame,
            group,
            subjects,
            lambda candidate: source.family_rows(group.qualified, family.name, candidate),
            f"rows of family `{family.name}`",
        )
        terms: list[tuple[sympy.Basic | bool, sympy.Symbol]] = []
        for key, row in rows.items():
            if slot.name not in row:
                raise EvaluationRefusal(
                    f"row {key} of family `{group.qualified}.{family.name}` for subject "
                    f"({', '.join(subjects)}) holds no value for slot `{slot.name}`"
                )
            conditions: list[sympy.Basic | bool] = []
            for position, index in enumerate(indices):
                if isinstance(index, int):
                    if index != key[position]:
                        break
                else:
                    conditions.append(sympy.Eq(_expr(index), key[position]))
            else:
                condition = sympy.And(*conditions) if conditions else True
                symbol = self.parameter(
                    frame,
                    ("row", group.qualified, family.name, subjects, key, slot.name),
                    f"{group.qualified}.{family.name}.{slot.name}",
                    float(row[slot.name]),
                )
                terms.append((condition, symbol))
        if all(isinstance(i, int) for i in indices):
            if not terms:
                raise EvaluationRefusal(
                    f"family `{group.qualified}.{family.name}` for subject ({', '.join(subjects)}) "
                    f"has no row {tuple(indices)}"
                )
            return terms[0][1]
        pieces = [(value, condition) for condition, value in terms]
        return _normal(sympy.Piecewise(*pieces, (sympy.nan, True)))

    # -- calls -----------------------------------------------------------------------------

    def contributions(self, env: dict[str, Value]) -> dict[str, m.SubformSlot]:
        return {name: v.slot for name, v in env.items() if isinstance(v, Contribution)}

    def call(self, frame: _Frame, node: t.Call, env: dict[str, Value]) -> Value:
        ref = classify_call(frame.scope, node, self.contributions(env))
        if not isinstance(ref, OutputCall):
            raise EvaluationRefusal("an index set is not a value")
        accepted = ref.accepted
        set_names = {f.name for f in accepted.sets}
        indexed = {a.name for a in accepted.arguments if a.over}
        sets: dict[str, tuple[Subject, ...]] = {}
        vectors: dict[str, dict[tuple[Subject, ...], sympy.Expr]] = {}
        given: dict[str, Value] = {}
        for keyword in ref.keywords:
            if keyword.name in set_names:
                assert isinstance(keyword.value, t.Name)
                sets[keyword.name] = frame.sets[keyword.value.id]
            elif keyword.name in indexed:
                vectors[keyword.name] = self.vector_value(frame, keyword.value, env)
            else:
                given[keyword.name] = self.ev(frame, keyword.value, env)
        roles: dict[str, Subject] = {}
        if ref.kind == "contribution":
            assert ref.var is not None
            contribution = env[ref.var]
            assert isinstance(contribution, Contribution)
            choice = contribution.choice
            roles.update(contribution.roles)
        elif ref.kind == "subform":
            assert ref.slot is not None
            role_values = self.role_values(accepted, given)
            key = (
                tuple(role_values[r.name] for r in accepted.roles)
                if ref.slot.per == "subject"
                else ()
            )
            choices = frame.source.subform_choices(ref.slot.qualified, key)
            if len(choices) != 1:
                raise EvaluationRefusal(
                    f"sub-form slot `{ref.slot.qualified}` needs exactly one form chosen for "
                    f"subject ({', '.join(key)}), found {len(choices)}"
                )
            choice = choices[0]
        else:
            assert ref.nested is not None
            nested = ref.nested
            nested_subjects = self.subjects(frame, nested.subjects, env)
            positions = tuple(self.ev(frame, node, env) for node in nested.indices)
            if not all(isinstance(position, int) for position in positions):
                raise EvaluationRefusal(
                    f"the family index of `{nested.slot.name}` is an integer known at expansion"
                )
            where = (
                nested.slot.name
                if nested.family is None
                else f"{nested.family.name}.{nested.slot.name}"
            )
            choice = self.held(
                frame,
                nested.group,
                nested_subjects,
                lambda candidate: frame.source.nested_set(
                    nested.group.qualified,
                    where,
                    candidate,
                    positions,  # type: ignore[arg-type]
                ),
                f"set held by `{where}`",
            )
        if ref.kind == "nested":
            roles.update(self.held_roles(choice, given))
        roles.update(self.role_values(accepted, given, skip=tuple(roles)))
        arguments = {
            name: _expr(value)
            for name, value in given.items()
            if name in {a.name for a in accepted.arguments}
        }
        return self.call_form(frame, choice, accepted, roles, arguments, vectors, sets, ref.output)

    def held_roles(self, choice: FormChoice, given: Mapping[str, Value]) -> dict[str, Subject]:
        """The roles of the called contract that the set `choice` holds supplies: the subjects of
        its slot group, bound as the group binds them, for each role the call does not give."""
        if choice.group is None:
            return {}
        group = next((g for g in self.decl.slot_groups if g.qualified == choice.group), None)
        if group is None:
            raise EvaluationRefusal(f"`{choice.group}` is not a slot group (`form.group`)")
        return {
            binding.target: subject
            for binding, subject in zip(group.bindings, choice.subjects, strict=False)
            if binding.kind == "role" and binding.target is not None and binding.target not in given
        }

    def role_values(
        self, accepted: m.Contract, given: Mapping[str, Value], skip: tuple[str, ...] = ()
    ) -> dict[str, Subject]:
        result: dict[str, Subject] = {}
        for role in accepted.roles:
            if role.name in skip:
                continue
            value = given.get(role.name)
            if not isinstance(value, str):
                raise EvaluationRefusal(f"role `{role.name}` of `{accepted.name}` has no subject")
            result[role.name] = value
        return result

    def call_form(
        self,
        frame: _Frame,
        choice: FormChoice,
        accepted: m.Contract,
        roles: Mapping[str, Subject],
        arguments: Mapping[str, sympy.Expr],
        vectors: Mapping[str, Mapping[tuple[Subject, ...], sympy.Expr]],
        sets: Mapping[str, tuple[Subject, ...]],
        output: str,
    ) -> Value:
        form = self.decl.forms.get(choice.form)
        if form is None:
            raise EvaluationRefusal(f"`{choice.form}` is not a declared form")
        if form.implements != accepted.name:
            raise EvaluationRefusal(
                f"form `{form.name}` implements `{form.implements}`, not `{accepted.name}`"
            )
        scope = FormScope.of(self.decl, form)
        placeholders: dict[str, sympy.Symbol | dict[tuple[Subject, ...], sympy.Symbol]] = {
            name: self.fresh(f"{form.name}.{name}") for name in arguments
        }
        substitution: dict[sympy.Basic, sympy.Basic] = {
            placeholders[name]: value
            for name, value in arguments.items()  # type: ignore[misc]
        }
        for name, elements in vectors.items():
            table = {key: self.fresh(f"{form.name}.{name}") for key in elements}
            placeholders[name] = table
            substitution.update({table[key]: value for key, value in elements.items()})
        callee = _Frame(
            scope=scope,
            roles=dict(roles),
            sets=dict(sets),
            args=placeholders,
            source=choice.source,
        )
        expr, guards = self.output(callee, output)
        for text, condition in guards:
            frame.guards.append(
                (text, sympy.Or(sympy.Not(frame.path), condition.xreplace(substitution)))
            )
        used = set(expr.free_symbols)
        for _, condition in guards:
            used |= condition.free_symbols
        for block in callee.blocks:
            if set(block.unknowns) & used:
                frame.blocks.append(block.substituted(substitution))
        return _normal(expr.xreplace(substitution))

    # -- sums and iteration ----------------------------------------------------------------

    def iterate(self, frame: _Frame, node: t.Expr, env: dict[str, Value]) -> Iterator[Value]:
        scope = frame.scope
        if isinstance(node, t.Name):
            if node.id in scope.sets:
                yield from frame.sets[node.id]
                return
            slot = scope.subforms[node.id]
            for choice in frame.source.subform_choices(slot.qualified, ()):
                yield Contribution(slot, choice, {})
            return
        if isinstance(node, t.Range):
            start, stop = self.ev(frame, node.start, env), self.ev(frame, node.stop, env)
            if not (isinstance(start, int) and isinstance(stop, int)):
                raise EvaluationRefusal("the bounds of a range are integers known at expansion")
            yield from range(start, stop)
            return
        assert isinstance(node, t.Call)
        ref = classify_call(scope, node, self.contributions(env))
        if isinstance(ref, FamilyRef):
            subjects = self.subjects(frame, ref.raw, env)
            rows = self.held(
                frame,
                ref.group,
                subjects,
                lambda candidate: frame.source.family_rows(
                    ref.group.qualified, ref.family.name, candidate
                ),
                f"rows of family `{ref.family.name}`",
            )
            yield from sorted(key[0] for key in rows)
            return
        assert isinstance(ref, SubformIter)
        accepted = scope.decl.contracts[ref.slot.accepts]
        given = {k.name: self.ev(frame, k.value, env) for k in ref.keywords}
        roles = self.role_values(accepted, given)
        key = tuple(roles[r.name] for r in accepted.roles)
        for choice in frame.source.subform_choices(ref.slot.qualified, key):
            yield Contribution(ref.slot, choice, roles)

    def reduce(self, frame: _Frame, node: t.Reduce, env: dict[str, Value]) -> Value:
        terms: list[sympy.Expr] = []

        def loop(position: int, scope: dict[str, Value]) -> None:
            if position == len(node.clauses):
                terms.append(_expr(self.ev(frame, node.body, scope)))
                return
            clause = node.clauses[position]
            for item in self.iterate(frame, clause.iterable, scope):
                loop(position + 1, {**scope, clause.target: item})

        loop(0, env)
        combine = sympy.Add if node.kind == "sum" else sympy.Mul
        return _normal(combine(*terms))

    # -- derivative, selection, integral ---------------------------------------------------

    def derivative(self, frame: _Frame, node: t.Derivative, env: dict[str, Value]) -> Value:
        expr = _expr(self.ev(frame, node.expr, env))
        wrt = node.wrt
        unknowns = {y for block in frame.blocks for y in block.unknowns}
        if isinstance(wrt, t.Name) and wrt.id in frame.scope.locals:
            symbol = frame.local_symbols.get(wrt.id)
            if symbol is None:  # a local that is a constant integer: nothing depends on it
                return 0
            if self.inline(frame, expr).free_symbols & unknowns:
                raise EvaluationRefusal(
                    f"a derivative with respect to the local `{wrt.id}` of an expression that "
                    "depends on an implicit block's unknowns: differentiate with respect to an "
                    "argument"
                )
            expr = self.inline_dependents(frame, expr, wrt.id)  # type: ignore[assignment]
            return _normal(sympy.diff(expr, symbol))
        if isinstance(wrt, t.Name):
            symbol = frame.args[wrt.id]
        else:
            assert isinstance(wrt, t.Subscript) and isinstance(wrt.base, t.Name)
            table = frame.args[wrt.base.id]
            assert isinstance(table, dict)
            symbol = table[self.subjects(frame, wrt.indices, env)]
        assert isinstance(symbol, sympy.Symbol)
        inlined = self.inline(frame, expr)
        return _normal(sympy.diff(inlined, symbol) + self.implicit_terms(frame, inlined, symbol))

    def implicit_terms(self, frame: _Frame, expr: sympy.Basic, symbol: sympy.Symbol) -> sympy.Expr:
        """The part of the total derivative of `expr` with respect to the argument `symbol` that
        runs through the unknowns of implicit blocks. The residuals g(y, u) vanish, so
        `dy/du = -g_y^{-1} g_u` (implicit-function theorem), and each unknown the expression
        contains adds `d expr / dy * dy/du`."""
        total: sympy.Expr = sympy.Integer(0)
        free = expr.free_symbols
        for block in frame.blocks:
            used = [k for k, y in enumerate(block.unknowns) if y in free]
            if not used:
                continue
            residuals = sympy.Matrix(block.residuals)
            g_u = residuals.diff(symbol)
            if g_u.is_zero_matrix:
                continue
            g_y = residuals.jacobian(list(block.unknowns))
            if len(block.unknowns) == 1:
                moved = [-g_u[0] / g_y[0, 0]]
            else:
                moved = list(g_y.LUsolve(-g_u))
            for k in used:
                total += sympy.diff(expr, block.unknowns[k]) * moved[k]
        return total

    def at(self, frame: _Frame, node: t.At, env: dict[str, Value]) -> Value:
        assert isinstance(node.family, t.Call)
        ref = classify_call(frame.scope, node.family, self.contributions(env))
        assert isinstance(ref, FamilyRef) and ref.family.interval is not None
        subjects = self.subjects(frame, ref.raw, env)
        rows = self.held(
            frame,
            ref.group,
            subjects,
            lambda candidate: frame.source.family_rows(
                ref.group.qualified, ref.family.name, candidate
            ),
            f"rows of family `{ref.family.name}`",
        )
        lower, upper = ref.family.interval
        for key, row in rows.items():
            if lower not in row or upper not in row:
                raise EvaluationRefusal(
                    f"row {key} of family `{ref.group.qualified}.{ref.family.name}` for subject "
                    f"({', '.join(subjects)}) holds no value for its interval `{lower}`, `{upper}`"
                )
        pieces = sorted(
            ((float(row[lower]), float(row[upper]), key[0]) for key, row in rows.items())
        )
        for (_, high, _), (low, _, _) in itertools.pairwise(pieces):
            if low < high:
                raise EvaluationRefusal(
                    f"the pieces of family `{ref.group.qualified}.{ref.family.name}` for subject "
                    f"({', '.join(subjects)}) overlap"
                )
        z = _expr(self.ev(frame, node.value, env))
        branches: list[tuple[sympy.Expr, sympy.Basic]] = []
        for position, (low, high, index) in enumerate(pieces):
            closed = position == len(pieces) - 1
            where = ("row", ref.group.qualified, ref.family.name, subjects, (index,))
            label = f"{ref.group.qualified}.{ref.family.name}"
            low_symbol = self.parameter(frame, (*where, lower, False), f"{label}.{lower}", low)
            high_symbol = self.parameter(frame, (*where, upper, False), f"{label}.{upper}", high)
            inside = sympy.And(z >= low_symbol, z <= high_symbol if closed else z < high_symbol)
            branches.append((sympy.Integer(index), inside))
        anywhere = sympy.Or(*(condition for _, condition in branches)) if branches else sympy.false
        frame.guards.append(
            (
                f"{self.describe(z)} is outside every piece of family `{ref.group.qualified}.{ref.family.name}` "
                f"for subject ({', '.join(subjects)})",
                sympy.Or(sympy.Not(frame.path), anywhere),
            )
        )
        return _normal(sympy.Piecewise(*branches, (sympy.nan, True)))

    def integral(self, frame: _Frame, node: t.Integral, env: dict[str, Value]) -> Value:
        variable = self.fresh(node.var)
        body = _expr(self.ev(frame, node.body, {**env, node.var: variable}))
        lower = _expr(self.ev(frame, node.lower, env))
        upper = _expr(self.ev(frame, node.upper, env))
        closed = sympy.integrate(body, (variable, lower, upper))
        if not closed.has(sympy.Integral):
            return _normal(closed)
        return _normal(self.quadrature(body, variable, lower, upper))

    def quadrature(
        self, body: sympy.Expr, variable: sympy.Symbol, lower: sympy.Expr, upper: sympy.Expr
    ) -> sympy.Expr:
        """A definite integral with no closed form, as a function of its bounds and of the other
        symbols of its integrand (arguments and parameters), evaluated by
        `scipy.integrate.quad`. The function is named by the structure of the integrand, so two
        integrals of one structure are one function and two of different structure never are."""
        free = sorted(body.free_symbols - {variable}, key=str)
        digest = hashlib.sha256(sympy.srepr((variable, body, tuple(free))).encode()).hexdigest()
        name = f"_Quad{digest[:24]}"
        integrand = self.cache.entry(
            ("integrand", name), lambda: self.cache.compile([variable, *free], body, self.functions)
        )

        def integrate(low: np.ndarray, high: np.ndarray, *values: np.ndarray) -> np.ndarray:
            arrays = np.broadcast_arrays(
                *(np.asarray(v, dtype=float) for v in (low, high, *values))
            )
            result = np.empty(arrays[0].shape)
            for position in np.ndindex(result.shape):
                a, b, *rest = (array[position] for array in arrays)
                result[position] = scipy.integrate.quad(
                    lambda s: float(integrand(s, *rest)),  # noqa: B023
                    a,
                    b,
                    epsabs=0.0,
                    epsrel=1e-12,
                    limit=500,
                )[0]
            return result

        self.functions[name] = integrate
        return sympy.Function(name)(lower, upper, *free)


# -- the bound form ------------------------------------------------------------------------------


@dataclass(frozen=True)
class _Prepared:
    """One output, expanded: the expression, the conditions that must hold, the blocks it needs
    and every symbol it reads."""

    expr: sympy.Expr
    guards: tuple[tuple[str, sympy.Basic], ...]
    blocks: tuple[Block, ...]
    symbols: tuple[sympy.Symbol, ...]
    """Every symbol of the expression and the conditions, in name order."""


@dataclass(frozen=True)
class _Guard:
    symbols: tuple[sympy.Symbol, ...]
    function: Callable[..., np.ndarray]


@dataclass(frozen=True)
class _Output:
    symbols: tuple[sympy.Symbol, ...]
    function: Callable[..., np.ndarray]


class BoundForm:
    """A form bound to subjects and a parameter source, ready to evaluate its outputs.

    Binding expands an output with its stored values held as parameter symbols. What is compiled
    is the structure of the expansion, found in `cache` when an earlier binding had the same:
    bindings that share a `CompileCache` and differ only in values compile once."""

    def __init__(
        self,
        decl: m.Declaration,
        form: m.Form,
        source: ParameterSource,
        roles: Mapping[str, Subject],
        sets: Mapping[str, Sequence[Subject]],
        cache: CompileCache,
    ) -> None:
        self.decl = decl
        self.form = form
        self.cache = cache
        self.machine = _Machine(decl, cache)
        scope = FormScope.of(decl, form)
        self.scope = scope
        if set(roles) != set(scope.roles):
            raise EvaluationRefusal(
                f"form `{form.name}` needs a subject for each role of contract "
                f"`{scope.contract.name}` ({', '.join(scope.roles) or 'none'}), given "
                f"{', '.join(roles) or 'none'}"
            )
        if set(sets) != set(scope.sets):
            raise EvaluationRefusal(
                f"form `{form.name}` needs the members of each set of contract "
                f"`{scope.contract.name}` ({', '.join(scope.sets) or 'none'}), given "
                f"{', '.join(sets) or 'none'}"
            )
        members = {name: tuple(values) for name, values in sets.items()}
        self.args: dict[str, sympy.Symbol | dict[tuple[Subject, ...], sympy.Symbol]] = {}
        self.labels: dict[sympy.Symbol, str] = {}
        """How a message names an argument symbol: by its subjects. The symbols themselves are
        named by position, so that two bindings of one structure have the same symbols."""
        for argument in scope.contract.arguments:
            if not argument.over:
                self.args[argument.name] = sympy.Symbol(argument.name, real=True)
                continue
            table: dict[tuple[Subject, ...], sympy.Symbol] = {}
            sizes = [range(len(members[s])) for s in argument.over]
            for positions in itertools.product(*sizes):
                key = tuple(members[s][p] for s, p in zip(argument.over, positions))
                symbol = sympy.Symbol(
                    f"{argument.name}[{','.join(map(str, positions))}]", real=True
                )
                table[key] = symbol
                self.labels[symbol] = f"{argument.name}[{','.join(key)}]"
            self.args[argument.name] = table
        self.frame = _Frame(
            scope=scope, roles=dict(roles), sets=members, args=dict(self.args), source=source
        )
        self._outputs: dict[str, _Prepared] = {}

    @property
    def parameters(self) -> Mapping[sympy.Symbol, float]:
        """The number each parameter symbol of the outputs expanded so far stands for, in
        storage units."""
        return dict(self.machine.values)

    def _built(self, output: str) -> _Prepared:
        if output not in self._outputs:
            expr, guards = self.machine.output(self.frame, output)
            self.machine.resolve_conventions()
            needed = set(expr.free_symbols)
            for _, condition in guards:
                needed |= condition.free_symbols
            blocks = tuple(b for b in self.frame.blocks if set(b.unknowns) & needed)
            self._outputs[output] = _Prepared(
                expr, tuple(guards), blocks, tuple(sorted(needed, key=str))
            )
        return self._outputs[output]

    def expression(self, output: str) -> sympy.Expr:
        """The SymPy expression of `output` in the argument symbols and the parameter symbols
        (`parameters` gives their values)."""
        return self._built(output).expr

    def _compile(
        self, symbols: Sequence[sympy.Symbol], expr: sympy.Basic
    ) -> Callable[..., np.ndarray]:
        return self.cache.compile(symbols, expr, self.machine.functions)

    def evaluate(
        self,
        output: str,
        **arguments: float
        | np.ndarray
        | Mapping[Subject | tuple[Subject, ...], float | np.ndarray],
    ) -> np.ndarray:
        """`output` at the given argument values, in storage units. A scalar argument is a number
        or an array; an argument indexed over sets is a mapping from the subject (or the tuple of
        subjects, for several sets) to its value. Refuses where the form has no answer."""
        unknown = set(arguments) - set(self.args)
        if unknown:
            raise EvaluationRefusal(
                f"`{', '.join(sorted(unknown))}` is not an argument of contract "
                f"`{self.scope.contract.name}`"
            )
        prepared = self._built(output)
        values: dict[sympy.Symbol, np.ndarray] = {}
        for name, symbol in self.args.items():
            if isinstance(symbol, sympy.Symbol):
                if name in arguments:
                    values[symbol] = np.asarray(arguments[name], dtype=float)
                continue
            given = arguments.get(name)
            if not isinstance(given, Mapping):
                continue
            keyed = {(k,) if isinstance(k, str) else k: v for k, v in given.items()}
            for key, element in symbol.items():
                if key in keyed:
                    values[element] = np.asarray(keyed[key], dtype=float)
        needed = set(prepared.symbols)
        solved = {y for block in prepared.blocks for y in block.unknowns}
        inputs = needed - solved
        for block in prepared.blocks:
            inputs |= set(block.params)
        parameters = self.machine.values
        for symbol in inputs:
            if symbol in parameters:
                values[symbol] = np.asarray(parameters[symbol], dtype=float)
        missing = sorted(self.labels.get(s, str(s)) for s in inputs - set(values))
        if missing:
            raise EvaluationRefusal(f"output `{output}` needs a value for {', '.join(missing)}")
        if prepared.blocks:
            self._solve(list(prepared.blocks), values, inputs)
        for text, condition in prepared.guards:
            guard = self.cache.entry(("guard", condition), lambda c=condition: self._guard(c))
            holds = guard.function(*(values[s] for s in guard.symbols))
            if not np.all(holds):
                raise EvaluationRefusal(text)
        if prepared.expr.has(sympy.Derivative, sympy.Subs):
            raise EvaluationRefusal(
                f"output `{output}` has a derivative of a numerical integral, which has no closed form"
            )
        compiled = self.cache.entry(("output", prepared.expr), lambda: self._output(prepared.expr))
        result = np.asarray(compiled.function(*(values[s] for s in compiled.symbols)), dtype=float)
        shape = (
            np.broadcast(*(values[s] for s in prepared.symbols)).shape if prepared.symbols else ()
        )
        return np.broadcast_to(result, shape).copy() if shape else result.reshape(())

    def validity(
        self,
        output: str,
        kind: str,
        **arguments: float | np.ndarray,
    ) -> np.ndarray:
        """Where each point lies with respect to the validity regions of `kind` (an
        `envelope_kind` member) of the records `output` reads, as an array of `Membership` codes
        broadcast over the argument values: inside, outside, undetermined, or no region of that
        kind stated (`expression/validity.py`).

        A clause is decided by the argument of the contract that names its observable (an
        argument declares `observable`); a clause about a component or an aggregation, or on an
        observable that no argument names, names more than one argument or has no value here,
        leaves its region undetermined. It never refuses because of a region: evaluating is
        `evaluate`'s business, and what to do outside is the caller's."""
        kinds = [member.name for member in self.decl.enums[pc.ENVELOPE_KIND.declared].members]
        if kind not in kinds:
            raise EvaluationRefusal(
                f"`{kind}` is not a validity region kind ({', '.join(kinds)})"
            )
        unknown = set(arguments) - set(self.args)
        if unknown:
            raise EvaluationRefusal(
                f"`{', '.join(sorted(unknown))}` is not an argument of contract "
                f"`{self.scope.contract.name}`"
            )
        self._built(output)
        named: dict[str, list[str]] = {}
        for argument in self.scope.contract.arguments:
            if argument.observable is not None and not argument.over:
                named.setdefault(argument.observable, []).append(argument.name)
        observed = {
            observable: np.asarray(arguments[names[0]], dtype=float)
            for observable, names in named.items()
            if len(names) == 1 and names[0] in arguments
        }
        shapes = [
            np.shape(np.asarray(value))
            for name, value in arguments.items()
            if isinstance(self.args[name], sympy.Symbol)
        ]
        shape = np.broadcast_shapes(*shapes) if shapes else ()
        records: list[RecordValidity] = []
        for key, source in self.machine.sources.items():
            reads = tuple(self.machine.reads.get(key, ()))
            if reads:
                records.extend(source.validity(kind, reads))
        return membership(records, observed, shape)

    def _guard(self, condition: sympy.Basic) -> _Guard:
        symbols = tuple(sorted(condition.free_symbols, key=str))
        return _Guard(symbols, self._compile(symbols, condition))

    def _output(self, expr: sympy.Expr) -> _Output:
        symbols = tuple(sorted(expr.free_symbols, key=str))
        return _Output(symbols, self._compile(symbols, expr))

    def _solve(
        self, blocks: list[Block], values: dict[sympy.Symbol, np.ndarray], inputs: set[sympy.Symbol]
    ) -> None:
        """Solve each block at every point of the argument values and add its unknowns to
        `values`, as arrays of the broadcast shape."""
        shape = np.broadcast_shapes(*(np.shape(values[s]) for s in inputs)) if inputs else ()
        for block in blocks:
            solver = self.cache.entry(
                ("block", block.key), lambda b=block: BlockSolver(b, self._compile)
            )
            params = block.params
            arrays = [np.broadcast_to(values[p], shape) for p in params]
            solution = np.empty((len(block.unknowns), *shape))
            for index in np.ndindex(shape):
                point = tuple(float(array[index]) for array in arrays)
                try:
                    solution[(slice(None), *index)] = solver.solve(point, block.labels)
                except SolveFailure as failure:
                    where = (
                        ", ".join(
                            f"{p}={v:.12g}"
                            for p, v in zip(params, point)
                            if p not in self.machine.values
                        )
                        or "no arguments"
                    )
                    raise EvaluationRefusal(
                        f"form `{block.form}`, implicit block `{block.name}`, at {where}: {failure}"
                    ) from None
            for k, y in enumerate(block.unknowns):
                values[y] = solution[k]


def bind(
    decl: m.Declaration,
    form: str,
    *,
    source: ParameterSource,
    roles: Mapping[str, Subject] | None = None,
    sets: Mapping[str, Sequence[Subject]] | None = None,
    cache: CompileCache | None = None,
) -> BoundForm:
    """Bind form `form` of `decl` to the subject of each role and the ordered members of each set
    of its contract, and to a parameter source. Bindings given one `cache` share what they
    compile; without one, the binding has a cache of its own."""
    chosen = decl.forms.get(form)
    if chosen is None:
        raise EvaluationRefusal(f"`{form}` is not a declared form")
    return BoundForm(decl, chosen, source, roles or {}, sets or {}, cache or CompileCache())
