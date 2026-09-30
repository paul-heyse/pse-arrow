# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Compiled functions, prepared once and executed many times (expressions.md section 5).

The expansion of a form holds stored values as parameter symbols, so what is compiled is the
structure of an expression and nothing of the values it is evaluated with. `CompileCache` keeps
a compiled function under the structure it was compiled from (the SymPy expression itself, which
compares by structure), counts the compilations it performs and the lookups it answered, and is
shared by every bound form that is given it.
"""

from __future__ import annotations

from collections.abc import Callable, Hashable, Mapping, Sequence
from typing import cast

import numpy as np
import sympy


class CompileCache:
    """Compiled functions by the structure they were compiled from."""

    def __init__(self) -> None:
        self.compilations = 0
        """The `lambdify` calls made through this cache."""
        self.reused = 0
        """The lookups that found what an earlier one had compiled."""
        self._entries: dict[Hashable, object] = {}

    def compile(
        self,
        symbols: Sequence[sympy.Symbol],
        expr: sympy.Basic,
        functions: Mapping[str, Callable[..., np.ndarray]],
    ) -> Callable[..., np.ndarray]:
        """`expr` as a NumPy function of `symbols`, with `functions` for the evaluator's own
        special functions. The expression is printed as it stands: `cse` is off, so no common
        subexpression is pulled out, and SciPy and NumPy supply the rest."""
        self.compilations += 1
        return sympy.lambdify(
            list(symbols), expr, modules=[dict(functions), "scipy", "numpy"], cse=False
        )

    def entry[T](self, key: Hashable, build: Callable[[], T]) -> T:
        """What was built under `key`, built now when there is none."""
        if key in self._entries:
            self.reused += 1
            return cast(T, self._entries[key])
        built = build()
        self._entries[key] = built
        return built
