"""Exact counterexamples to symmetric binary64 rounding cells at binade edges."""

import json
import math
from fractions import Fraction


def check_cell(candidate: float) -> dict[str, str | bool]:
    value = Fraction(candidate)
    predecessor = Fraction(math.nextafter(candidate, -math.inf))
    successor = Fraction(math.nextafter(candidate, math.inf))
    lower = (predecessor + value) / 2
    upper = (value + successor) / 2
    symmetric_half_width = Fraction(math.ulp(candidate)) / 2
    symmetric_lower = value - symmetric_half_width
    symmetric_upper = value + symmetric_half_width
    if symmetric_lower < lower:
        witness = (symmetric_lower + lower) / 2
    else:
        witness = (upper + symmetric_upper) / 2
    symmetric_accepts = symmetric_lower < witness < symmetric_upper
    true_cell_accepts = lower < witness < upper
    rounded = float(witness)
    assert symmetric_accepts
    assert not true_cell_accepts
    assert rounded != candidate
    assert float(lower) == candidate
    assert float(upper) == candidate
    return {
        "candidate": candidate.hex(),
        "predecessor": float(predecessor).hex(),
        "successor": float(successor).hex(),
        "lower_midpoint": str(lower),
        "upper_midpoint": str(upper),
        "witness_root": str(witness),
        "symmetric_cell_accepts": symmetric_accepts,
        "true_cell_accepts": true_cell_accepts,
        "correctly_rounded_witness": rounded.hex(),
        "both_midpoint_ties_round_to_candidate": True,
    }


print(json.dumps([check_cell(x) for x in (1.0, -1.0, 2.0, -2.0)], indent=2))
