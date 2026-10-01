# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Parse expression text into the typed tree (expressions.md sections 1 and 3).

The text is parsed by the standard `ast` module in `eval` mode and the resulting syntax tree is
converted node by node: only the grammar is accepted and everything else is refused with its
position. The text is never compiled or executed.

`ast` needs one expression on one line. The text is wrapped in parentheses, with a newline
before the closing one, so a declaration may spread an expression over several lines and end
it with a comment; positions are reported in the text as written.
"""

from __future__ import annotations

import ast
import functools
import math
import re
from dataclasses import dataclass

from thermo_knowledge.declaration.diagnostics import Code
from thermo_knowledge.expression import tree as t

_IDENTIFIER = re.compile(r"^[A-Za-z][A-Za-z0-9_]*$")
_ARITY: dict[str, tuple[int, int | None]] = {
    **{name: (1, 1) for name in t.ELEMENTARY if name not in ("min", "max")},
    "min": (2, None),
    "max": (2, None),
    "erf": (1, 1),
    "chebyshev_t": (2, 2),
    "debye": (2, 2),
}
_BINARY: dict[type[ast.operator], str] = {ast.Add: "+", ast.Sub: "-", ast.Mult: "*", ast.Div: "/", ast.Pow: "**"}
_COMPARE: dict[type[ast.cmpop], str] = {
    ast.Lt: "<",
    ast.LtE: "<=",
    ast.Gt: ">",
    ast.GtE: ">=",
    ast.Eq: "==",
    ast.NotEq: "!=",
}
_REFUSED = {
    ast.Lambda: "a lambda",
    ast.NamedExpr: "an assignment expression (`:=`)",
    ast.JoinedStr: "an f-string",
    ast.List: "a list",
    ast.Set: "a set",
    ast.Dict: "a dictionary",
    ast.Tuple: "a tuple",
    ast.Starred: "a starred expression",
    ast.Await: "`await`",
    ast.Yield: "`yield`",
    ast.YieldFrom: "`yield from`",
    ast.Slice: "a slice",
    ast.SetComp: "a set comprehension",
    ast.DictComp: "a dictionary comprehension",
}


class ExpressionError(Exception):
    """Expression text is refused: a stable diagnostic `code`, the `message` and the position
    (`line` and `column`, from 1) in the text."""

    def __init__(self, code: Code, message: str, line: int, column: int) -> None:
        super().__init__(message)
        self.code = code
        self.message = message
        self.line = line
        self.column = column

    @property
    def pos(self) -> t.Pos:
        return (self.line, self.column)


@dataclass
class _Source:
    """Turns `ast` positions (line, UTF-8 byte offset) into positions in the original text."""

    lines: list[str]

    def pos(self, node: ast.AST) -> t.Pos:
        line = getattr(node, "lineno", 1)
        offset = getattr(node, "col_offset", 0)
        return self.at(line, offset)

    def at(self, line: int, byte_offset: int) -> t.Pos:
        # Line 1 of the parsed text begins with the wrapping parenthesis.
        text = self.lines[line - 1] if 0 < line <= len(self.lines) else ""
        column = len(text.encode()[:byte_offset].decode(errors="ignore")) + 1
        return (line, max(column - 1 if line == 1 else column, 1))


@functools.cache
def parse_cached(text: str) -> t.Expr:
    """`parse(text)`, remembered: trees are immutable, so one is shared by every reader of a text."""
    return parse(text)


def parse(text: str) -> t.Expr:
    """The typed tree of `text`, a local or an output; raises `ExpressionError` when it is not in
    the grammar. A local may be a bracketed comprehension, `[body for i in S]`."""
    return _parse(text, residual=False)


def parse_residual(text: str) -> t.Expr:
    """The typed tree of a residual of an implicit block: an expression, or an expression
    followed by `for` clauses (`body for a in S`), which states one residual for each element."""
    return _parse(text, residual=True)


def _parse(text: str, *, residual: bool) -> t.Expr:
    if not text.strip():
        raise ExpressionError(Code.EXPRESSION_SYNTAX, "the expression is empty", 1, 1)
    wrapped = f"({text}\n)"
    source = _Source(wrapped.split("\n"))
    try:
        module = ast.parse(wrapped, mode="eval")
    except SyntaxError as error:
        lines = text.split("\n")
        line = error.lineno or 1
        if line > len(lines):
            # The error is at the closing parenthesis this module added: the end of the text.
            line, column = len(lines), len(lines[-1]) + 1
        else:
            # `SyntaxError.offset` counts characters from 1; line 1 begins with the added `(`.
            column = max((error.offset or 1) - (1 if line == 1 else 0), 1)
        raise ExpressionError(
            Code.EXPRESSION_SYNTAX, f"not an expression: {error.msg}", line, column
        ) from None
    except (ValueError, RecursionError, MemoryError) as error:
        raise ExpressionError(Code.EXPRESSION_SYNTAX, f"not an expression: {error}", 1, 1) from None
    converter = _Converter(source)
    body = module.body
    if isinstance(body, ast.ListComp) and not residual:
        return converter.comprehension(body, body.elt, body.generators)
    if _is_basis_call(body) and not residual:
        return converter.basis(body)
    if isinstance(body, ast.GeneratorExp) and residual:
        return converter.comprehension(body, body.elt, body.generators)
    return converter.expr(body)


def _is_basis_call(node: ast.expr) -> bool:
    return (
        isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == "basis"
    )


class _Converter:
    def __init__(self, source: _Source) -> None:
        self.source = source

    def fail(self, node: ast.AST, code: Code, message: str) -> ExpressionError:
        line, column = self.source.pos(node)
        return ExpressionError(code, message, line, column)

    def grammar(self, node: ast.AST, what: str) -> ExpressionError:
        return self.fail(
            node, Code.EXPRESSION_GRAMMAR, f"{what} is not part of the expression grammar"
        )

    def identifier(self, node: ast.AST, text: str) -> str:
        if not _IDENTIFIER.match(text):
            raise self.fail(
                node,
                Code.EXPRESSION_GRAMMAR,
                f"`{text}` is not a name: names start with a letter and hold letters, digits "
                "and underscores",
            )
        return text

    # -- expressions -----------------------------------------------------------------------

    def expr(self, node: ast.expr) -> t.Expr:
        pos = self.source.pos(node)
        match node:
            case ast.Constant():
                return self.constant(node)
            case ast.Name():
                return t.Name(id=self.identifier(node, node.id), pos=pos)
            case ast.Attribute() | ast.Subscript():
                return self.reference(node)
            case ast.Call():
                return self.call(node)
            case ast.BinOp():
                op = _BINARY.get(type(node.op))
                if op is None:
                    raise self.grammar(node, f"the operator `{type(node.op).__name__}`")
                return t.BinOp(
                    op=op, left=self.expr(node.left), right=self.expr(node.right), pos=pos
                )
            case ast.UnaryOp():
                if isinstance(node.op, ast.USub):
                    op = "-"
                elif isinstance(node.op, ast.UAdd):
                    op = "+"
                elif isinstance(node.op, ast.Not):
                    op = "not"
                else:
                    raise self.grammar(node, "the operator `~`")
                return t.UnaryOp(op=op, operand=self.expr(node.operand), pos=pos)
            case ast.Compare():
                if len(node.ops) != 1:
                    raise self.grammar(
                        node, "a chained comparison (join two comparisons with `and`); it"
                    )
                op = _COMPARE.get(type(node.ops[0]))
                if op is None:
                    raise self.grammar(node, f"the comparison `{type(node.ops[0]).__name__}`")
                return t.Compare(
                    op=op, left=self.expr(node.left), right=self.expr(node.comparators[0]), pos=pos
                )
            case ast.BoolOp():
                name = "and" if isinstance(node.op, ast.And) else "or"
                values: list[t.Expr] = []
                for value in node.values:
                    converted = self.expr(value)
                    if isinstance(converted, t.BoolOp) and converted.op == name:
                        values.extend(converted.values)
                    else:
                        values.append(converted)
                return t.BoolOp(op=name, values=tuple(values), pos=pos)
            case ast.IfExp():
                return t.IfExp(
                    test=self.expr(node.test),
                    body=self.expr(node.body),
                    orelse=self.expr(node.orelse),
                    pos=pos,
                )
            case ast.ListComp():
                raise self.fail(
                    node,
                    Code.EXPRESSION_GRAMMAR,
                    "a bracketed comprehension is only allowed as the value of a local or as an "
                    "argument of a sub-form call",
                )
            case ast.GeneratorExp():
                raise self.fail(
                    node,
                    Code.EXPRESSION_GRAMMAR,
                    "a comprehension is only allowed as the argument of `sum` or `prod`",
                )
        for kind, what in _REFUSED.items():
            if isinstance(node, kind):
                raise self.grammar(node, what)
        raise self.grammar(node, f"`{type(node).__name__}`")

    def constant(self, node: ast.Constant) -> t.Expr:
        value = node.value
        pos = self.source.pos(node)
        if type(value) is int:
            return t.Num(value=value, pos=pos)
        if type(value) is float:
            if not math.isfinite(value):
                raise self.fail(node, Code.EXPRESSION_GRAMMAR, "a number literal out of range")
            return t.Num(value=value, pos=pos)
        if isinstance(value, str):
            raise self.fail(
                node,
                Code.EXPRESSION_GRAMMAR,
                "a string is only allowed as the argument of `unit(...)`",
            )
        raise self.grammar(node, f"the constant {value!r}")

    def reference(self, node: ast.expr) -> t.Expr:
        """A name, `a.b`, `a[i]` and their combinations: the ways to name an argument, a slot
        or an output. Attribute access is never applied to anything else."""
        pos = self.source.pos(node)
        if isinstance(node, ast.Name):
            return t.Name(id=self.identifier(node, node.id), pos=pos)
        if isinstance(node, ast.Attribute):
            if not isinstance(node.value, ast.Name | ast.Attribute | ast.Subscript):
                raise self.fail(
                    node,
                    Code.EXPRESSION_GRAMMAR,
                    "attribute access applies to names, slots and sub-form outputs only",
                )
            return t.Attribute(
                value=self.reference(node.value), attr=self.identifier(node, node.attr), pos=pos
            )
        if isinstance(node, ast.Subscript):
            if not isinstance(node.value, ast.Name | ast.Attribute):
                raise self.fail(
                    node, Code.EXPRESSION_GRAMMAR, "only an argument or a slot can be subscripted"
                )
            elements = node.slice.elts if isinstance(node.slice, ast.Tuple) else [node.slice]
            indices: list[t.Expr] = []
            for element in elements:
                if isinstance(element, ast.Slice):
                    raise self.grammar(element, "a slice")
                if isinstance(element, ast.Starred):
                    raise self.grammar(element, "a starred expression")
                indices.append(self.expr(element))
            return t.Subscript(base=self.reference(node.value), indices=tuple(indices), pos=pos)
        raise self.grammar(node, f"`{type(node).__name__}`")

    # -- calls -----------------------------------------------------------------------------

    def positional(self, node: ast.Call, name: str) -> list[t.Expr]:
        if node.keywords:
            raise self.fail(
                node.keywords[0],
                Code.BAD_CALL,
                f"`{name}` takes positional arguments only",
            )
        return [self.argument(arg) for arg in node.args]

    def argument(self, arg: ast.expr) -> t.Expr:
        if isinstance(arg, ast.Starred):
            raise self.fail(arg, Code.BAD_CALL, "a starred argument is not allowed")
        return self.expr(arg)

    def count(self, node: ast.Call, name: str, expected: int) -> list[t.Expr]:
        args = self.positional(node, name)
        if len(args) != expected:
            raise self.fail(
                node,
                Code.BAD_CALL,
                f"`{name}` takes {expected} argument{'s' if expected != 1 else ''}, "
                f"found {len(args)}",
            )
        return args

    def call(self, node: ast.Call) -> t.Expr:
        pos = self.source.pos(node)
        func = node.func
        if isinstance(func, ast.Name):
            name = func.id
            if name in t.FUNCTIONS:
                args = self.positional(node, name)
                low, high = _ARITY[name]
                if len(args) < low or (high is not None and len(args) > high):
                    bound = f"{low}" if low == high else f"at least {low}"
                    raise self.fail(
                        node, Code.BAD_CALL, f"`{name}` takes {bound} arguments, found {len(args)}"
                    )
                return t.Func(name=name, args=tuple(args), pos=pos)
            if name in ("sum", "prod"):
                return self.reduce(node, name)
            if name == "d":
                expression, wrt = self.count(node, "d", 2)
                if not isinstance(wrt, t.Name | t.Subscript):
                    raise self.fail(
                        node.args[1],
                        Code.BAD_DERIVATIVE,
                        "`d(expr, name)` differentiates with respect to an argument (subscripted "
                        "when it is indexed) or a local",
                    )
                return t.Derivative(expr=expression, wrt=wrt, pos=pos)
            if name == "at":
                family, value = self.count(node, "at", 2)
                return t.At(family=family, value=value, pos=pos)
            if name == "position":
                array, member = self.count(node, "position", 2)
                return t.Position(array=array, member=member, pos=pos)
            if name == "integral":
                body, variable, lower, upper = self.count(node, "integral", 4)
                if not isinstance(variable, t.Name):
                    raise self.fail(
                        node.args[1],
                        Code.BAD_CALL,
                        "the second argument of `integral` is the integration variable, a name",
                    )
                return t.Integral(body=body, var=variable.id, lower=lower, upper=upper, pos=pos)
            if name == "unit":
                if (
                    len(node.args) != 1
                    or node.keywords
                    or not isinstance(node.args[0], ast.Constant)
                    or not isinstance(node.args[0].value, str)
                ):
                    raise self.fail(
                        node, Code.BAD_CALL, "`unit` takes one string, a `pint` unit: unit('Pa')"
                    )
                return t.UnitLiteral(text=node.args[0].value, pos=pos)
            if name == "range":
                raise self.fail(
                    node,
                    Code.EXPRESSION_GRAMMAR,
                    "`range(a, b)` is an index set: write it after `in`",
                )
            if name == "basis":
                raise self.fail(
                    node,
                    Code.EXPRESSION_GRAMMAR,
                    "`basis('name', [...])` wraps a comprehension, and is only allowed as the "
                    "value of a local or as an argument of a sub-form call",
                )
            raise self.fail(
                node, Code.UNKNOWN_FUNCTION, f"`{name}` is not a function of the grammar"
            )
        if isinstance(func, ast.Attribute):
            return self.generic_call(node)
        raise self.fail(
            node,
            Code.EXPRESSION_GRAMMAR,
            "only functions, sub-form outputs and index sets are called",
        )

    def generic_call(self, node: ast.Call) -> t.Call:
        func = self.reference(node.func)
        args = [self.argument(arg) for arg in node.args]
        keywords: list[t.Keyword] = []
        for keyword in node.keywords:
            if keyword.arg is None:
                raise self.fail(keyword, Code.BAD_CALL, "`**` arguments are not allowed")
            value = keyword.value
            if isinstance(value, ast.ListComp):
                converted: t.Expr = self.comprehension(value, value.elt, value.generators)
            elif _is_basis_call(value):
                converted = self.basis(value)
            else:
                converted = self.expr(value)
            keywords.append(
                t.Keyword(
                    name=self.identifier(keyword, keyword.arg),
                    value=converted,
                    pos=self.source.pos(keyword),
                )
            )
        if args and keywords:
            raise self.fail(
                node, Code.BAD_CALL, "give a call's arguments either by position or by keyword"
            )
        return t.Call(
            func=func, args=tuple(args), keywords=tuple(keywords), pos=self.source.pos(node)
        )

    # -- sum and prod ----------------------------------------------------------------------

    def reduce(self, node: ast.Call, name: str) -> t.Reduce:
        if node.keywords or len(node.args) != 1 or not isinstance(node.args[0], ast.GeneratorExp):
            raise self.fail(
                node, Code.BAD_CALL, f"`{name}` takes one generator: {name}(expr for i in S)"
            )
        generator = node.args[0]
        return t.Reduce(
            kind=name,
            body=self.expr(generator.elt),
            clauses=self.clauses(generator.generators),
            pos=self.source.pos(node),
        )

    def basis(self, node: ast.expr) -> t.Comprehension:
        """`basis('name', [body for i in S])`: the comprehension, with the basis asserted."""
        assert isinstance(node, ast.Call)
        arguments = node.args
        if (
            node.keywords
            or len(arguments) != 2
            or not isinstance(arguments[0], ast.Constant)
            or not isinstance(arguments[0].value, str)
            or not isinstance(arguments[1], ast.ListComp)
        ):
            raise self.fail(
                node,
                Code.BAD_CALL,
                "`basis` takes a string naming a composition basis and a bracketed "
                "comprehension: basis('mole_fraction', [x[i] for i in components])",
            )
        comprehension = arguments[1]
        built = self.comprehension(comprehension, comprehension.elt, comprehension.generators)
        return t.Comprehension(
            body=built.body,
            clauses=built.clauses,
            basis=arguments[0].value,
            pos=self.source.pos(node),
        )

    def comprehension(
        self, node: ast.expr, element: ast.expr, generators: list[ast.comprehension]
    ) -> t.Comprehension:
        return t.Comprehension(
            body=self.expr(element), clauses=self.clauses(generators), pos=self.source.pos(node)
        )

    def clauses(self, generators: list[ast.comprehension]) -> tuple[t.Clause, ...]:
        clauses: list[t.Clause] = []
        for comprehension in generators:
            target = comprehension.target
            if not isinstance(target, ast.Name):
                raise self.fail(target, Code.EXPRESSION_GRAMMAR, "an index variable is one name")
            if comprehension.ifs:
                raise self.fail(
                    comprehension.ifs[0],
                    Code.EXPRESSION_GRAMMAR,
                    "a filter is not part of the grammar: use a conditional in the body",
                )
            if comprehension.is_async:
                raise self.grammar(target, "`async for`")
            clauses.append(
                t.Clause(
                    target=self.identifier(target, target.id),
                    iterable=self.iterable(comprehension.iter),
                    pos=self.source.pos(target),
                )
            )
        return tuple(clauses)

    def iterable(self, node: ast.expr) -> t.Expr:
        """What follows `in`: a set, `range(a, b)`, a family's index set or a sub-form."""
        if isinstance(node, ast.Name):
            return t.Name(id=self.identifier(node, node.id), pos=self.source.pos(node))
        if isinstance(node, ast.Call):
            func = node.func
            if isinstance(func, ast.Name) and func.id == "range":
                args = self.count(node, "range", 2)
                return t.Range(start=args[0], stop=args[1], pos=self.source.pos(node))
            if isinstance(func, ast.Name):
                if func.id in t.FUNCTIONS or func.id in t.HEADS:
                    raise self.fail(
                        node,
                        Code.EXPRESSION_GRAMMAR,
                        f"`{func.id}(...)` is not an index set",
                    )
                # A sub-form slot iterated per subject: `terms(i=i)`.
                if node.args:
                    raise self.fail(
                        node, Code.BAD_CALL, f"`{func.id}` takes keyword arguments only"
                    )
                return self.generic_call_named(node)
            if isinstance(func, ast.Attribute):
                return self.generic_call(node)
        raise self.fail(
            node,
            Code.NOT_INDEX_SET,
            "an index set is a set, `range(a, b)`, a family's index set or a sub-form slot",
        )

    def generic_call_named(self, node: ast.Call) -> t.Call:
        assert isinstance(node.func, ast.Name)
        keywords: list[t.Keyword] = []
        for keyword in node.keywords:
            if keyword.arg is None:
                raise self.fail(keyword, Code.BAD_CALL, "`**` arguments are not allowed")
            keywords.append(
                t.Keyword(
                    name=self.identifier(keyword, keyword.arg),
                    value=self.expr(keyword.value),
                    pos=self.source.pos(keyword),
                )
            )
        return t.Call(
            func=t.Name(id=node.func.id, pos=self.source.pos(node.func)),
            keywords=tuple(keywords),
            pos=self.source.pos(node),
        )
