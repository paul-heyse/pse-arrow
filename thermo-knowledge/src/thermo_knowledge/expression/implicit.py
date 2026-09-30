# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Implicit blocks at an evaluation point (expressions.md section 5).

An implicit block is expanded once, over the concrete index sets, into a `Block`: one SymPy
symbol for each scalar unknown, the residual vector, the bounds and starts, and the selection
rule. Stored values enter a block as parameter symbols, so a `BlockSolver` is compiled once for
every block of the same structure. Solving a block at one point of argument and parameter
values is numerical and lives here:

- one unknown with both bounds: every root in the interval is found (the real roots of a
  polynomial residual; otherwise a scan of the interval with a bracketing solver on each sign
  change) and the selection rule picks among them: `unique` asserts there is exactly one,
  `smallest` and `largest` take an end, `by` the root at which an expression is least;
- otherwise: a Newton-type solve from the start with the analytic Jacobian and the bounds
  (`scipy.optimize.root`, or `least_squares` when a bound is given), polished by Newton steps
  that stay inside the bounds.

Anything else is a `SolveFailure` naming what went wrong; a wrong answer is never returned.
"""

from __future__ import annotations

from collections.abc import Callable, Hashable, Iterator, Mapping, Sequence
from dataclasses import dataclass, field

import numpy as np
import scipy.optimize
import sympy

_EPS = float(np.finfo(float).eps)
_SCAN_POINTS = 4001
_POLE = 1.0e-3
_RESIDUAL_RTOL = 1.0e-8
_STEP_RTOL = 1.0e-10


class SolveFailure(Exception):
    """A block has no answer at a point: no root, no convergence, a root outside its bounds or
    not the only one. The text says which."""


type Compiler = Callable[[Sequence[sympy.Symbol], sympy.Basic], Callable[..., np.ndarray]]
"""Turns an expression into a NumPy function of the given symbols (`CompileCache.compile`)."""


@dataclass
class Block:
    """An implicit block of one bound form, expanded: every expression is in argument symbols,
    parameter symbols (whose values are passed when the block is solved) and the block's own
    unknown symbols only."""

    form: str
    name: str
    unknowns: tuple[sympy.Symbol, ...]
    labels: tuple[str, ...]
    residuals: tuple[sympy.Expr, ...]
    lower: tuple[sympy.Expr | None, ...]
    upper: tuple[sympy.Expr | None, ...]
    start: tuple[sympy.Expr | None, ...]
    select: str
    by: sympy.Expr | None
    guards: list[tuple[str, sympy.Basic]] = field(default_factory=list)

    @property
    def key(self) -> Hashable:
        """What determines the compiled solver: the expressions and the selection rule. The
        labels (they name subjects) and the guards are not part of it."""
        return (
            self.unknowns,
            self.residuals,
            self.lower,
            self.upper,
            self.start,
            self.select,
            self.by,
        )

    @property
    def params(self) -> tuple[sympy.Symbol, ...]:
        """The argument and parameter symbols the block depends on, in name order."""
        found: set[sympy.Symbol] = set()
        for expr in self.expressions():
            found |= expr.free_symbols
        found -= set(self.unknowns)
        return tuple(sorted(found, key=str))

    def expressions(self) -> Iterator[sympy.Basic]:
        yield from self.residuals
        for group in (self.lower, self.upper, self.start):
            yield from (e for e in group if e is not None)
        if self.by is not None:
            yield self.by

    def substituted(self, mapping: Mapping[sympy.Basic, sympy.Basic]) -> Block:
        """The block with `mapping` applied to every expression: the placeholders of a
        sub-form's arguments replaced by the values its caller passed."""

        def one(expr: sympy.Expr | None) -> sympy.Expr | None:
            return None if expr is None else expr.xreplace(mapping)

        return Block(
            form=self.form,
            name=self.name,
            unknowns=self.unknowns,
            labels=self.labels,
            residuals=tuple(e.xreplace(mapping) for e in self.residuals),
            lower=tuple(one(e) for e in self.lower),
            upper=tuple(one(e) for e in self.upper),
            start=tuple(one(e) for e in self.start),
            select=self.select,
            by=one(self.by),
            guards=[(text, cond.xreplace(mapping)) for text, cond in self.guards],
        )


class BlockSolver:
    """A block turned into NumPy functions of (unknowns..., params...), where `params` are the
    argument and parameter symbols of `Block.params`. It holds nothing of the block it was built
    from but its expressions, so every block with the same `Block.key` can use it."""

    def __init__(self, block: Block, compile_: Compiler) -> None:
        self.select = block.select
        params = block.params
        ys = block.unknowns
        both = [*ys, *params]
        residuals = list(block.residuals)
        self.n = len(ys)
        self.residual = compile_(both, sympy.Tuple(*residuals))
        self.jacobian = compile_(both, sympy.Matrix(residuals).jacobian(list(ys)))
        scales = [
            sympy.Add(*(sympy.Abs(term) for term in sympy.Add.make_args(expr)))
            for expr in residuals
        ]
        self.scale = compile_(both, sympy.Tuple(*scales))
        self.lower = [_bound(e, params, compile_, -np.inf) for e in block.lower]
        self.upper = [_bound(e, params, compile_, np.inf) for e in block.upper]
        self.start = [None if e is None else _bound(e, params, compile_, None) for e in block.start]
        self.by = None if block.by is None else compile_(both, block.by)
        self.coefficients: Callable[..., np.ndarray] | None = None
        if self.n == 1:
            try:
                poly = sympy.Poly(residuals[0], ys[0])
            except (sympy.PolynomialError, sympy.GeneratorsNeeded):
                poly = None
            if poly is not None and poly.degree() >= 1:
                self.coefficients = compile_(params, sympy.Tuple(*poly.all_coeffs()))

    # -- evaluation ------------------------------------------------------------------------

    def g(self, y: np.ndarray, p: tuple[float, ...]) -> np.ndarray:
        return np.asarray(self.residual(*y, *p), dtype=float).reshape(self.n)

    def jac(self, y: np.ndarray, p: tuple[float, ...]) -> np.ndarray:
        return np.asarray(self.jacobian(*y, *p), dtype=float).reshape(self.n, self.n)

    def scale_at(self, y: np.ndarray, p: tuple[float, ...]) -> np.ndarray:
        return np.asarray(self.scale(*y, *p), dtype=float).reshape(self.n)

    # -- solving ---------------------------------------------------------------------------

    def solve(self, p: Sequence[float], labels: Sequence[str]) -> np.ndarray:
        """The values of the unknowns at `p` (values of `Block.params`, in order); `labels` name
        the unknowns in a failure."""
        p = tuple(p)
        if not all(np.isfinite(p)):
            raise SolveFailure("an argument is not a finite number")
        lo = np.array([f(*p) for f in self.lower], dtype=float)
        hi = np.array([f(*p) for f in self.upper], dtype=float)
        for k in range(self.n):
            if not lo[k] <= hi[k]:
                raise SolveFailure(
                    f"the interval of `{labels[k]}` is empty: lower {lo[k]:g} is above "
                    f"upper {hi[k]:g}"
                )
        if self.n == 1 and np.isfinite(lo[0]) and np.isfinite(hi[0]):
            return np.array([self.interval(p, float(lo[0]), float(hi[0]), labels[0])])
        return self.system(p, lo, hi, labels)

    # one unknown, both bounds: every root of the interval

    def interval(self, p: tuple[float, ...], lo: float, hi: float, label: str) -> float:
        if self.coefficients is not None:
            roots = self.polynomial_roots(p, lo, hi)
        else:
            roots = self.scanned_roots(p, lo, hi)
        if not roots:
            raise SolveFailure(f"no root of the residual of `{label}` in [{lo:g}, {hi:g}]")
        select = self.select
        if select == "unique":
            if len(roots) > 1:
                raise SolveFailure(
                    f"`select = unique` but the residual of `{label}` has {len(roots)} roots in "
                    f"[{lo:g}, {hi:g}]: {', '.join(f'{r:.12g}' for r in roots)}"
                )
            return roots[0]
        if select == "smallest":
            return roots[0]
        if select == "largest":
            return roots[-1]
        assert self.by is not None
        measures = []
        for root in roots:
            measure = float(np.asarray(self.by(root, *p)))
            if not np.isfinite(measure):
                raise SolveFailure(
                    f"the `by` expression is not finite at the root {root:.12g} of `{label}`"
                )
            measures.append(measure)
        return roots[int(np.argmin(measures))]

    def accepted(self, y: float, p: tuple[float, ...]) -> bool:
        """Whether `y` makes the single residual vanish to working accuracy, relative to the
        size of its own terms."""
        point = np.array([y])
        with np.errstate(all="ignore"):
            g = self.g(point, p)
            scale = self.scale_at(point, p)
        return bool(np.all(np.isfinite(g)) and abs(g[0]) <= _RESIDUAL_RTOL * max(scale[0], 1e-300))

    def newton_polish(self, y: float, p: tuple[float, ...], lo: float, hi: float) -> float:
        for _ in range(4):
            point = np.array([y])
            with np.errstate(all="ignore"):
                g = self.g(point, p)[0]
                slope = self.jac(point, p)[0, 0]
            if not (np.isfinite(g) and np.isfinite(slope)) or slope == 0.0:
                break
            step = g / slope
            moved = y - step
            if not lo <= moved <= hi:
                break
            y = float(moved)
            if abs(step) <= 4 * _EPS * abs(y):
                break
        return y

    def polynomial_roots(self, p: tuple[float, ...], lo: float, hi: float) -> list[float]:
        assert self.coefficients is not None
        coefficients = np.asarray(self.coefficients(*p), dtype=float).reshape(-1)
        if not np.all(np.isfinite(coefficients)):
            raise SolveFailure("a coefficient of the residual polynomial is not finite")
        if not np.any(coefficients):
            raise SolveFailure("the residual vanishes identically")
        candidates = np.roots(coefficients / np.max(np.abs(coefficients)))
        slack = 1e-12 * max(abs(lo), abs(hi), 1e-300)
        found: list[float] = []
        for root in candidates:
            if abs(root.imag) > 1e-9 * max(abs(root.real), 1e-300):
                continue
            y = float(root.real)
            if not lo - slack <= y <= hi + slack:
                continue
            y = self.newton_polish(min(max(y, lo), hi), p, lo, hi)
            if self.accepted(y, p):
                found.append(y)
        return _distinct(found)

    def scanned_roots(self, p: tuple[float, ...], lo: float, hi: float) -> list[float]:
        if lo > 0.0 and hi / lo > 100.0:
            grid = np.geomspace(lo, hi, _SCAN_POINTS)
        else:
            grid = np.linspace(lo, hi, _SCAN_POINTS)
        values = self.on_grid(grid, p)
        found: list[float] = []
        for k in range(len(grid) - 1):
            left, right = values[k], values[k + 1]
            if not (np.isfinite(left) and np.isfinite(right)):
                continue
            if left == 0.0:
                found.append(float(grid[k]))
            elif left < 0.0 < right or right < 0.0 < left:
                root = self.bracket(float(grid[k]), float(grid[k + 1]), p)
                if root is None:
                    continue
                with np.errstate(all="ignore"):
                    at_root = abs(self.g(np.array([root]), p)[0])
                # a sign change across a pole is not a root: the residual there is far larger
                if at_root <= _POLE * max(abs(left), abs(right)):
                    found.append(root)
        if np.isfinite(values[-1]) and values[-1] == 0.0:
            found.append(float(grid[-1]))
        return _distinct(found)

    def on_grid(self, grid: np.ndarray, p: tuple[float, ...]) -> np.ndarray:
        with np.errstate(all="ignore"):
            try:
                values = np.asarray(self.residual(grid, *p)[0], dtype=float)
                return np.broadcast_to(values, grid.shape).astype(float)
            except (TypeError, ValueError):
                return np.array([self.g(np.array([y]), p)[0] for y in grid], dtype=float)

    def bracket(self, a: float, b: float, p: tuple[float, ...]) -> float | None:
        def scalar(y: float) -> float:
            with np.errstate(all="ignore"):
                return float(self.g(np.array([y]), p)[0])

        try:
            return float(
                scipy.optimize.brentq(scalar, a, b, xtol=1e-300, rtol=4 * _EPS, maxiter=500)
            )
        except (ValueError, RuntimeError):
            return None

    # several unknowns, or an unbounded one: a Newton-type solve from the start

    def system(
        self, p: tuple[float, ...], lo: np.ndarray, hi: np.ndarray, labels: Sequence[str]
    ) -> np.ndarray:
        x0 = np.empty(self.n)
        for k in range(self.n):
            starter = self.start[k]
            if starter is not None:
                x0[k] = float(starter(*p))
            else:  # the checker guarantees both bounds when there is no start
                x0[k] = 0.5 * (lo[k] + hi[k])
            if not np.isfinite(x0[k]):
                raise SolveFailure(f"the start of `{labels[k]}` is not finite")
            if not lo[k] <= x0[k] <= hi[k]:
                raise SolveFailure(
                    f"the start {x0[k]:g} of `{labels[k]}` is outside its bounds "
                    f"[{lo[k]:g}, {hi[k]:g}]"
                )
        with np.errstate(all="ignore"):
            g0 = self.g(x0, p)
        if not np.all(np.isfinite(g0)):
            raise SolveFailure("the residuals are not finite at the start")

        def fun(y: np.ndarray) -> np.ndarray:
            with np.errstate(all="ignore"):
                return np.nan_to_num(self.g(y, p), nan=1e300, posinf=1e300, neginf=-1e300)

        def jac(y: np.ndarray) -> np.ndarray:
            with np.errstate(all="ignore"):
                return np.nan_to_num(self.jac(y, p), nan=0.0, posinf=1e300, neginf=-1e300)

        bounded = bool(np.any(np.isfinite(lo)) or np.any(np.isfinite(hi)))
        try:
            if bounded:
                result = scipy.optimize.least_squares(
                    fun,
                    x0,
                    jac=jac,
                    bounds=(lo, hi),
                    method="trf",
                    xtol=1e-15,
                    ftol=1e-15,
                    gtol=1e-15,
                    max_nfev=2000,
                )
            else:
                result = scipy.optimize.root(fun, x0, jac=jac, method="hybr", tol=1e-14)
            x = np.asarray(result.x, dtype=float)
        except (ValueError, np.linalg.LinAlgError) as error:
            raise SolveFailure(f"the solver failed: {error}") from None
        # Why Newton steps follow the SciPy call: neither routine stops on the condition decided
        # below. `hybr` stops on a relative change of the iterate and `trf` on the change of its
        # cost, its step or its gradient; none of them says the residual vanishes, and the
        # status each returns is not read. Newton steps with the analytic Jacobian, kept inside
        # the bounds, take the iterate to working accuracy (a step of a few ulps), so a root is
        # not limited by where SciPy happened to stop (the cubic-root tests compare roots with
        # an independent calculation at a relative 1e-10). They also give the size of the last
        # step, which the acceptance test after them needs, and the same steps finish
        # `numpy.roots` results (`newton_polish`).
        x, step = self.polish(x, p, lo, hi)
        with np.errstate(all="ignore"):
            g = self.g(x, p)
            scale = np.maximum(self.scale_at(x, p), np.abs(g0))
        if not np.all(np.isfinite(x)) or not np.all(np.isfinite(g)):
            raise SolveFailure("the solver did not converge: the iterate is not finite")
        tolerance = _RESIDUAL_RTOL * np.maximum(scale, 1e-300)
        if step > _STEP_RTOL * (1.0 + float(np.max(np.abs(x)))) or np.any(np.abs(g) > tolerance):
            raise SolveFailure(
                f"the solver did not converge from the start: the largest residual is "
                f"{float(np.max(np.abs(g))):.3g} and the last Newton step {step:.3g}"
            )
        return x

    def polish(
        self, x: np.ndarray, p: tuple[float, ...], lo: np.ndarray, hi: np.ndarray
    ) -> tuple[np.ndarray, float]:
        """Newton steps from `x` that stay inside the bounds; the size of the last step."""
        step_size = np.inf
        for _ in range(12):
            with np.errstate(all="ignore"):
                g = self.g(x, p)
                matrix = self.jac(x, p)
            if not (np.all(np.isfinite(g)) and np.all(np.isfinite(matrix))):
                break
            try:
                step = np.linalg.solve(matrix, -g)
            except np.linalg.LinAlgError:
                break
            moved = x + step
            if np.any(moved < lo) or np.any(moved > hi) or not np.all(np.isfinite(moved)):
                break
            x = moved
            step_size = float(np.max(np.abs(step)))
            if step_size <= 4 * _EPS * (1.0 + float(np.max(np.abs(x)))):
                break
        if not np.isfinite(step_size):
            step_size = self.last_step(x, p)
        return x, step_size

    def last_step(self, x: np.ndarray, p: tuple[float, ...]) -> float:
        """The size of the Newton step from `x`, without taking it (infinite if singular)."""
        with np.errstate(all="ignore"):
            try:
                step = np.linalg.solve(self.jac(x, p), -self.g(x, p))
            except np.linalg.LinAlgError:
                return float("inf")
        return float(np.max(np.abs(step))) if np.all(np.isfinite(step)) else float("inf")


def _bound(
    expr: sympy.Expr | None,
    params: Sequence[sympy.Symbol],
    compile_: Compiler,
    default: float | None,
) -> Callable[..., float | None]:
    """A bound as a function of the parameters: the expression, else `default`."""
    if expr is None:
        return lambda *_: default
    function = compile_(params, expr)
    return lambda *p: float(np.asarray(function(*p)))


def _distinct(roots: list[float]) -> list[float]:
    """The roots in increasing order, those closer than a few parts in 10**12 counted once."""
    ordered = sorted(roots)
    kept: list[float] = []
    for root in ordered:
        if kept and abs(root - kept[-1]) <= 1e-12 * max(abs(root), abs(kept[-1]), 1e-300):
            continue
        kept.append(root)
    return kept
