# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The parser and the canonical form: every construct of the grammar round-trips through the
canonical serialisation, everything else is refused with its position."""

from __future__ import annotations

import pytest

from thermo_knowledge.declaration import Code
from thermo_knowledge.expression import tree as t
from thermo_knowledge.expression.canonical import (
    content_hash,
    residual_hash,
    serialise,
    serialise_residual,
)
from thermo_knowledge.expression.parser import ExpressionError, parse, parse_residual

GRAMMAR = {
    "integer": "1",
    "float": "0.45724",
    "exponent float": "1e-3",
    "unit literal": "unit('Pa')",
    "dimensioned number": "8.314462618 * unit('J/(mol*K)')",
    "sum and difference": "a + b - c",
    "product and quotient": "a * b / c",
    "power": "a ** 2",
    "right-associative power": "a ** b ** c",
    "negative exponent": "a ** -1",
    "negation": "-a",
    "negated power": "-a ** 2",
    "power of a negation": "(-a) ** 2",
    "unary plus": "+a",
    "grouping changes the order": "(a + b) * c",
    "right operand of a difference": "a - (b - c)",
    "right operand of a quotient": "a / (b * c)",
    **{f"function {name}": f"{name}(x)" for name in t.ELEMENTARY if name not in ("min", "max")},
    "min": "min(a, b, c)",
    "max": "max(a, b)",
    "erf": "erf(x)",
    "chebyshev_t": "chebyshev_t(3, x)",
    "debye": "debye(3, x)",
    "sum": "sum(x[i] for i in components)",
    "prod": "prod(1 - x[i] for i in components)",
    "nested for clauses": "sum(x[i] * x[j] for i in components for j in components)",
    "range": "sum(n for n in range(1, 5))",
    "family index set": "sum(pure.piece.a1[i, n] for n in pure.piece(i))",
    "family of an empty subject": "sum(core.terms.n[k] for k in core.terms())",
    "conditional": "a if a < b else b",
    "nested conditional": "a if c else b if d else e",
    "conditional in a product": "(a if c else b) * d",
    "less": "a < b",
    "less or equal": "a <= b",
    "greater": "a > b",
    "greater or equal": "a >= b",
    "equal": "a == b",
    "not equal": "a != b",
    "and": "a < b and b < c and c < d",
    "or": "a < b or b < c",
    "not": "not a < b",
    "and inside or": "a < b and b < c or c < d",
    "or inside and": "(a < b or b < c) and c < d",
    "derivative wrt an argument": "d(alpha_r, delta)",
    "derivative wrt an indexed argument": "d(alpha_r, x[i])",
    "at": "at(pure.piece(i), T)",
    "position": "position(t, s)",
    "position in a conditional": "1 if position(t, i) == 1 else -1",
    "integral": "integral(a + b * z, z, 0, T)",
    "slot": "pure.A[i]",
    "pair slot": "pair.k[i, j]",
    "slot of an empty subject": "core.omega_a",
    "family slot": "pure.piece.a1[i, n]",
    "sub-form call": "alpha.value(i=i, T=T)",
    "remapped sub-form call": "alpha.value(i=i, Tr=T / pure.T_c[i] + 1)",
    "nested-set call": "pair.a[i, j].value(T=T)",
    "contribution call": "sum(c.alpha_r(delta=delta, tau=tau) for c in residual)",
    "sub-form iterated per subject": "sum(c.alpha_r(T=T) for c in terms(i=i))",
    "subject comparison": "0 if i == j else pair.k[i, j]",
    "comparison of sums": "sum(x[i] for i in s) < 1",
    "comprehension": "[x[i] * r[i] / total for i in components]",
    "comprehension over two sets": "[pair.k[i, j] for i in components for j in components]",
    "comprehension as an argument": "mixing.a_mix(components=components, x=[x[i] / total for i in components])",
    "set and vector arguments": "mixing.a_mix(components=components, T=T, x=x, b_pure=b_pure)",
}


@pytest.mark.parametrize("text", GRAMMAR.values(), ids=GRAMMAR)
def test_every_construct_round_trips_through_the_canonical_serialisation(text: str) -> None:
    tree = parse(text)
    canonical = serialise(tree)
    assert parse(canonical) == tree
    assert serialise(parse(canonical)) == canonical


@pytest.mark.parametrize(
    ("kind", "node"),
    [
        ("Num", t.Num),
        ("UnitLiteral", t.UnitLiteral),
        ("Attribute", t.Attribute),
        ("Subscript", t.Subscript),
        ("Call", t.Call),
        ("Func", t.Func),
        ("BinOp", t.BinOp),
        ("UnaryOp", t.UnaryOp),
        ("Compare", t.Compare),
        ("BoolOp", t.BoolOp),
        ("IfExp", t.IfExp),
        ("Range", t.Range),
        ("Reduce", t.Reduce),
        ("Position", t.Position),
        ("Comprehension", t.Comprehension),
        ("Derivative", t.Derivative),
        ("At", t.At),
        ("Integral", t.Integral),
    ],
)
def test_the_grammar_reaches_every_node_type(kind: str, node: type[t.Node]) -> None:
    seen = {type(n) for text in GRAMMAR.values() for n in t.walk(parse(text))}
    assert node in seen, kind


def test_the_grammar_covers_every_function_name() -> None:
    written = {
        n.name for text in GRAMMAR.values() for n in t.walk(parse(text)) if isinstance(n, t.Func)
    }
    assert written == set(t.FUNCTIONS)


def test_the_tree_is_immutable_and_positions_do_not_affect_equality() -> None:
    tree = parse("a + b")
    with pytest.raises(AttributeError):
        tree.op = "-"  # type: ignore[misc]
    assert parse("a + b") == parse("   a   +\n   b ")
    assert parse("a + b").pos == (1, 1)
    assert isinstance(tree, t.BinOp) and tree.right.pos == (1, 5)


VARIANTS = [
    ("a+b*c", "a + b * c"),
    ("a + b * c", " ( a ) + ( ( b ) * ( c ) ) "),
    ("a + b + c", "(a + b) + c"),
    ("x if c else y", "(x) if (c) else (y)"),
    ("sum(x[i]*x[j] for i in S for j in S)", "sum( x[ i ] * x[ j ]  for i in S\n   for j in S )"),
    ("a < b and b < c and c < d", "a < b and (b < c and c < d)"),
    ("1e-3", "0.001"),
    ("exp(x)", "exp( x )  # a comment"),
    ("unit('Pa')", 'unit( "Pa" )'),
    ("pair.a[i, j].value(T=T)", "pair . a [ i , j ] . value ( T = T )"),
]


@pytest.mark.parametrize(("left", "right"), VARIANTS)
def test_canonical_form_ignores_whitespace_comments_and_redundant_parentheses(
    left: str, right: str
) -> None:
    assert serialise(parse(left)) == serialise(parse(right))
    assert content_hash(left) == content_hash(right)


def test_parentheses_that_matter_are_kept_and_change_the_hash() -> None:
    assert serialise(parse("a - (b - c)")) == "a - (b - c)"
    assert serialise(parse("(a - b) - c")) == "a - b - c"
    assert serialise(parse("(a + b) * c")) == "(a + b) * c"
    assert serialise(parse("-a ** 2")) == "-a ** 2"
    assert serialise(parse("(-a) ** 2")) == "(-a) ** 2"
    assert content_hash("a - (b - c)") != content_hash("a - b - c")
    assert content_hash("1") != content_hash("1.0")


def test_the_content_hash_is_a_sha256_of_the_canonical_text() -> None:
    import hashlib

    assert content_hash("a+b") == hashlib.sha256(b"a + b").hexdigest()
    assert len(content_hash("a + b")) == 64


FORBIDDEN = [
    ("position(t)", Code.BAD_CALL, (1, 1), "a position of one argument"),
    ("position(t, s, 1)", Code.BAD_CALL, (1, 1), "a position of three arguments"),
    ("position(t, s=s)", Code.BAD_CALL, (1, 13), "a position with a keyword"),
    ("(a + b).real", Code.EXPRESSION_GRAMMAR, (1, 1), "attribute access on an expression"),
    ("x.__class__", Code.EXPRESSION_GRAMMAR, (1, 1), "a dunder attribute"),
    ("__import__('os')", Code.UNKNOWN_FUNCTION, (1, 1), "a dunder function"),
    ("f(x).y", Code.EXPRESSION_GRAMMAR, (1, 1), "attribute access on a call"),
    ("lambda: 1", Code.EXPRESSION_GRAMMAR, (1, 1), "lambda"),
    (
        "1 + [x for x in y]",
        Code.EXPRESSION_GRAMMAR,
        (1, 5),
        "bracketed comprehension inside an expression",
    ),
    ("{x for x in y}", Code.EXPRESSION_GRAMMAR, (1, 1), "set comprehension"),
    ("{x: 1 for x in y}", Code.EXPRESSION_GRAMMAR, (1, 1), "dict comprehension"),
    ("(x for x in y)", Code.EXPRESSION_GRAMMAR, (1, 1), "generator outside sum"),
    ("exp(x for x in y)", Code.EXPRESSION_GRAMMAR, (1, 4), "generator in another function"),
    ("'abc'", Code.EXPRESSION_GRAMMAR, (1, 1), "string"),
    ("exp('Pa')", Code.EXPRESSION_GRAMMAR, (1, 5), "string as an argument"),
    ("a + 'x'", Code.EXPRESSION_GRAMMAR, (1, 5), "string in arithmetic"),
    ("unit(1)", Code.BAD_CALL, (1, 1), "unit of a number"),
    ("unit('Pa', 'K')", Code.BAD_CALL, (1, 1), "unit of two strings"),
    ("unit(text='Pa')", Code.BAD_CALL, (1, 1), "unit by keyword"),
    ("foo(1)", Code.UNKNOWN_FUNCTION, (1, 1), "unknown function"),
    ("1 + foo(1)", Code.UNKNOWN_FUNCTION, (1, 5), "unknown function in arithmetic"),
    ("print(1)", Code.UNKNOWN_FUNCTION, (1, 1), "a builtin of Python"),
    ("eval('1')", Code.UNKNOWN_FUNCTION, (1, 1), "eval"),
    ("exp(x=1)", Code.BAD_CALL, (1, 5), "keyword on a function"),
    ("d(x, wrt=T)", Code.BAD_CALL, (1, 6), "keyword on d"),
    ("exp()", Code.BAD_CALL, (1, 1), "no argument"),
    ("exp(a, b)", Code.BAD_CALL, (1, 1), "too many arguments"),
    ("min(a)", Code.BAD_CALL, (1, 1), "min of one"),
    ("chebyshev_t(1)", Code.BAD_CALL, (1, 1), "too few arguments"),
    ("max(*a)", Code.BAD_CALL, (1, 5), "starred argument"),
    ("pair.a[i, j].value(**k)", Code.BAD_CALL, (1, 20), "double-starred argument"),
    ("alpha.value(i, T=T)", Code.BAD_CALL, (1, 1), "positional and keyword arguments"),
    ("sum(x)", Code.BAD_CALL, (1, 1), "sum of a non-generator"),
    (
        "sum(x for x in y, 1)",
        Code.EXPRESSION_SYNTAX,
        (1, 5),
        "an unparenthesised generator with a second argument",
    ),
    ("sum(x for x in y if x)", Code.EXPRESSION_GRAMMAR, (1, 21), "a filter"),
    ("sum(x for (a, b) in y)", Code.EXPRESSION_GRAMMAR, (1, 11), "tuple target"),
    ("sum(x for x in [1, 2])", Code.NOT_INDEX_SET, (1, 16), "a list as an index set"),
    ("sum(x for x in y + 1)", Code.NOT_INDEX_SET, (1, 16), "arithmetic as an index set"),
    ("sum(x for x in exp(y))", Code.EXPRESSION_GRAMMAR, (1, 16), "a function as an index set"),
    ("range(1, 3)", Code.EXPRESSION_GRAMMAR, (1, 1), "range outside a for"),
    ("sum(x for x in range(3))", Code.BAD_CALL, (1, 16), "range of one"),
    ("(y := 1)", Code.EXPRESSION_GRAMMAR, (1, 2), "walrus"),
    ("x[1:2]", Code.EXPRESSION_GRAMMAR, (1, 3), "slice"),
    ("x[::2]", Code.EXPRESSION_GRAMMAR, (1, 3), "extended slice"),
    ("x[i, 1:2]", Code.EXPRESSION_GRAMMAR, (1, 6), "slice among subscripts"),
    ("(a + b)[0]", Code.EXPRESSION_GRAMMAR, (1, 1), "subscript of an expression"),
    ("x[i][j]", Code.EXPRESSION_GRAMMAR, (1, 1), "subscript of a subscript"),
    ("x[*a]", Code.EXPRESSION_GRAMMAR, (1, 3), "starred subscript"),
    ("a // b", Code.EXPRESSION_GRAMMAR, (1, 1), "floor division"),
    ("a % b", Code.EXPRESSION_GRAMMAR, (1, 1), "modulo"),
    ("a @ b", Code.EXPRESSION_GRAMMAR, (1, 1), "matrix product"),
    ("a << b", Code.EXPRESSION_GRAMMAR, (1, 1), "shift"),
    ("a & b", Code.EXPRESSION_GRAMMAR, (1, 1), "bit and"),
    ("~a", Code.EXPRESSION_GRAMMAR, (1, 1), "invert"),
    ("a in b", Code.EXPRESSION_GRAMMAR, (1, 1), "membership"),
    ("a is b", Code.EXPRESSION_GRAMMAR, (1, 1), "identity"),
    ("a < b < c", Code.EXPRESSION_GRAMMAR, (1, 1), "chained comparison"),
    ("[1, 2]", Code.EXPRESSION_GRAMMAR, (1, 1), "list"),
    ("(1, 2)", Code.EXPRESSION_GRAMMAR, (1, 1), "tuple"),
    ("{1, 2}", Code.EXPRESSION_GRAMMAR, (1, 1), "set"),
    ("{1: 2}", Code.EXPRESSION_GRAMMAR, (1, 1), "dict"),
    ("f'{x}'", Code.EXPRESSION_GRAMMAR, (1, 1), "f-string"),
    ("True", Code.EXPRESSION_GRAMMAR, (1, 1), "boolean"),
    ("None", Code.EXPRESSION_GRAMMAR, (1, 1), "None"),
    ("1j", Code.EXPRESSION_GRAMMAR, (1, 1), "complex"),
    ("1e999", Code.EXPRESSION_GRAMMAR, (1, 1), "a literal out of range"),
    ("...", Code.EXPRESSION_GRAMMAR, (1, 1), "ellipsis"),
    ("a, b", Code.EXPRESSION_GRAMMAR, (1, 1), "a tuple without parentheses"),
    ("   ", Code.EXPRESSION_SYNTAX, (1, 1), "blank text"),
    ("b'x'", Code.EXPRESSION_GRAMMAR, (1, 1), "bytes"),
    ("x.é", Code.EXPRESSION_GRAMMAR, (1, 1), "a non-ascii attribute is not a name"),
    ("é", Code.EXPRESSION_GRAMMAR, (1, 1), "a non-ascii name"),
    ("integral(x, 1, 0, 1)", Code.BAD_CALL, (1, 13), "an integration variable that is not a name"),
    ("integral(x, z, 0)", Code.BAD_CALL, (1, 1), "an integral with three arguments"),
    ("d(x, 1)", Code.BAD_DERIVATIVE, (1, 6), "derivative with respect to a number"),
    ("d(x, a + b)", Code.BAD_DERIVATIVE, (1, 6), "derivative with respect to an expression"),
    ("at(pure.piece(i))", Code.BAD_CALL, (1, 1), "at with one argument"),
    ("a +", Code.EXPRESSION_SYNTAX, (1, 4), "unfinished expression"),
    ("a b", Code.EXPRESSION_SYNTAX, (1, 1), "two names"),
    ("", Code.EXPRESSION_SYNTAX, (1, 1), "empty text"),
    ("(a", Code.EXPRESSION_SYNTAX, (1, 1), "unclosed parenthesis"),
    ("a)", Code.EXPRESSION_SYNTAX, (1, 3), "unbalanced parenthesis"),
    ("a = 1", Code.EXPRESSION_SYNTAX, (1, 1), "assignment"),
    ("a;b", Code.EXPRESSION_SYNTAX, (1, 2), "two statements"),
    ("yield 1", Code.EXPRESSION_GRAMMAR, (1, 1), "yield"),
    ("a +\n  (b @ c)", Code.EXPRESSION_GRAMMAR, (2, 4), "a refusal on the second line"),
    ("a\nb", Code.EXPRESSION_SYNTAX, (1, 1), "a second expression"),
]


@pytest.mark.parametrize(
    ("text", "code", "position", "what"), FORBIDDEN, ids=[case[3] for case in FORBIDDEN]
)
def test_what_the_grammar_excludes_is_refused_with_its_position(
    text: str, code: Code, position: tuple[int, int], what: str
) -> None:
    with pytest.raises(ExpressionError) as refused:
        parse(text)
    error = refused.value
    assert error.code == code, (text, error.message)
    assert error.line >= 1 and error.column >= 1
    assert error.message
    assert (error.line, error.column) == position, (text, error.message)


def test_nothing_is_ever_executed() -> None:
    # A name that would be dangerous to evaluate is only a name; a call to it is refused.
    assert parse("os") == t.Name(id="os")
    with pytest.raises(ExpressionError):
        parse("__import__('os').system('true')")
    with pytest.raises(ExpressionError):
        parse("open('x')")


# -- comprehensions and residuals -------------------------------------------------------------


def test_a_bracketed_comprehension_is_a_local_or_a_call_argument_and_nothing_else() -> None:
    tree = parse("[x[i] for i in components]")
    assert isinstance(tree, t.Comprehension)
    assert [clause.target for clause in tree.clauses] == ["i"]
    call = parse("m.a(x=[x[i] for i in s], T=T)")
    assert isinstance(call, t.Call)
    assert isinstance(call.keywords[0].value, t.Comprehension)
    for text in (
        "1 + [x for x in y]",
        "sum([x for x in y])",
        "f.g(1, [x for x in y])",
        "exp([x for x in y])",
    ):
        with pytest.raises(ExpressionError):
            parse(text)


@pytest.mark.parametrize(
    "text",
    [
        "[x for x in y if x]",
        "[x for (a, b) in y]",
        "[x for x in [1, 2]]",
        "[x for x in y + 1]",
        "[x async for x in y]",
    ],
)
def test_a_comprehension_has_the_same_clauses_as_a_sum(text: str) -> None:
    with pytest.raises(ExpressionError):
        parse(text)


def test_a_residual_is_an_expression_or_an_expression_with_for_clauses() -> None:
    assert parse_residual("X - 1") == parse("X - 1")
    tree = parse_residual("X[a] * (1 + sum(x[b] for b in sites)) - 1 for a in sites")
    assert isinstance(tree, t.Comprehension)
    assert tree.clauses[0].target == "a"
    assert isinstance(parse_residual("r[a, b] for a in s for b in s"), t.Comprehension)


@pytest.mark.parametrize("text", ["[x for x in y]", "a, b", "x for", "x for x in y if x", ""])
def test_a_residual_refuses_what_is_not_one(text: str) -> None:
    with pytest.raises(ExpressionError):
        parse_residual(text)


def test_a_bare_generator_is_not_a_local() -> None:
    with pytest.raises(ExpressionError, match="comprehension"):
        parse("x[a] - 1 for a in sites")


def test_a_residual_serialises_without_brackets_and_keeps_its_hash_across_spellings() -> None:
    text = "X[a]*(1+rho*sum(x[b]*X[b]*d[a,b] for b in sites))-1   for a in sites"
    assert (
        serialise_residual(parse_residual(text))
        == "X[a] * (1 + rho * sum(x[b] * X[b] * d[a, b] for b in sites)) - 1 for a in sites"
    )
    assert parse_residual(serialise_residual(parse_residual(text))) == parse_residual(text)
    assert residual_hash(text) == residual_hash(
        " ( X[a] * (1 + rho * sum(x[b] * X[b] * d[a, b] for b in sites)) - 1 ) for a in sites"
    )
    # a scalar residual serialises like the same expression anywhere else
    assert serialise_residual(parse_residual("X +  1")) == "X + 1"
    assert residual_hash("X + 1") == content_hash("X + 1")
    # the bracketed and the bare comprehension are different texts
    assert residual_hash("x for x in s") != content_hash("[x for x in s]")


# -- basis('name', [...]): the basis a form's author asserts for a vector it builds --------------


def test_basis_wraps_a_comprehension_and_round_trips() -> None:
    text = "basis('volume_fraction', [x[i] * b[i] / c for i in components])"
    tree = parse(text)
    assert isinstance(tree, t.Comprehension) and tree.basis == "volume_fraction"
    assert serialise(tree) == text
    assert parse(serialise(tree)) == tree
    # the basis is part of what was written: the same comprehension without it is another tree
    plain = parse("[x[i] * b[i] / c for i in components]")
    assert isinstance(plain, t.Comprehension) and plain.basis is None and plain != tree
    assert serialise(plain) != serialise(tree)
    assert content_hash(text) != content_hash("[x[i] * b[i] / c for i in components]")


def test_basis_is_accepted_as_a_call_argument() -> None:
    tree = parse("k.value(s=s, x=basis('mole_fraction', [y[i] for i in s]))")
    assert isinstance(tree, t.Call)
    keyword = tree.keywords[1]
    assert isinstance(keyword.value, t.Comprehension) and keyword.value.basis == "mole_fraction"


@pytest.mark.parametrize(
    ("text", "code"),
    [
        ("basis('mole_fraction')", Code.BAD_CALL),
        ("basis('mole_fraction', x)", Code.BAD_CALL),
        ("basis(1, [x[i] for i in s])", Code.BAD_CALL),
        ("basis('a', [x[i] for i in s], 2)", Code.BAD_CALL),
        ("basis(name='a', body=[x[i] for i in s])", Code.BAD_CALL),
        ("2 * basis('mole_fraction', [x[i] for i in s])", Code.EXPRESSION_GRAMMAR),
        ("sum(basis('mole_fraction', [x[i] for i in s]))", Code.BAD_CALL),
    ],
)
def test_basis_is_only_the_wrapper_of_a_comprehension(text: str, code: Code) -> None:
    with pytest.raises(ExpressionError) as raised:
        parse(text)
    assert raised.value.code == code


def test_a_name_that_is_basis_is_reserved_by_the_grammar() -> None:
    assert "basis" in t.HEADS
