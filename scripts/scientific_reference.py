# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Evidence and frozen-value comparisons for independent scientific references.

Run the entrypoints as modules, in the locked CPython 3.12 test-group environment.
JSON output is a reproducibility artifact, never an input to product tests.
"""

from __future__ import annotations

import hashlib
import inspect
import json
import platform
import re
import sys
from decimal import Decimal
from importlib.metadata import distributions
from pathlib import Path

import numpy as np
import scipy
from scipy.integrate import solve_ivp
from scipy.optimize import least_squares

ROOT = Path(__file__).resolve().parents[1]
RESULTS = ROOT / "build/plan53a-reference-results"
NUMBER = r"[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?"


def frozen_number(path: str, pattern: str) -> str:
    """Read exactly one expected numeric literal from the consuming authored file."""
    matches = re.findall(pattern, (ROOT / path).read_text())
    if len(matches) != 1:
        raise ValueError(f"expected one frozen value in {path}: {pattern}: {matches}")
    return matches[0]


def comparison(observed: float, literal: str, numerical: float = 0.0) -> dict[str, object]:
    """Allow half a printed last digit plus a separately stated numerical premise."""
    expected = float(literal)
    exponent = Decimal(literal).as_tuple().exponent
    if not isinstance(exponent, int):
        raise ValueError("frozen literal must be finite")
    rounding = float(Decimal(10) ** exponent / 2)
    delta = abs(observed - expected)
    return {
        "observed": observed,
        "frozen_literal": literal,
        "absolute_difference": delta,
        "rounding_allowance": rounding,
        "numerical_allowance": numerical,
        "passed": bool(np.isfinite(observed) and np.isfinite(expected) and np.isfinite(rounding) and np.isfinite(numerical) and numerical >= 0.0 and delta <= rounding + numerical),
    }


def file_identity(path: Path) -> dict[str, str]:
    return {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def environment() -> dict[str, object]:
    if platform.python_implementation() != "CPython" or sys.version_info[:2] != (3, 12) or scipy.__version__ != "1.18.1" or np.__version__ != "2.5.3":
        raise RuntimeError("reference requires CPython 3.12, scipy==1.18.1, numpy==2.5.3")
    return {
        "python": sys.version,
        "implementation": platform.python_implementation(),
        "platform": platform.platform(),
        "numpy": np.__version__,
        "scipy": scipy.__version__,
        "installed_distributions": dict(sorted((item.metadata["Name"], item.version) for item in distributions())),
        "lockfile": file_identity(ROOT / "uv.lock"),
        "upstream_api_sources": [file_identity(Path(inspect.getfile(inspect.unwrap(api)))) for api in (solve_ivp, least_squares)],
    }


def publish(name: str, evidence: dict[str, object], checks: dict[str, bool], sources: list[str]) -> int:
    """Always retain diagnostics, including failed reproduction, and fail honestly."""
    evidence.update({
        "reference": name,
        "environment": environment(),
        "source_correspondence": [file_identity(ROOT / path) for path in sources],
        "generator": file_identity(ROOT / f"scripts/{name}.py"),
        "support": file_identity(Path(__file__)),
        "checks": checks,
        "failure_count": sum(not passed for passed in checks.values()),
        "limits": "Independent reduced equations and selected inputs only; solver tolerances and cross-method agreement are numerical evidence, not certified global error bounds or product qualification.",
    })
    RESULTS.mkdir(parents=True, exist_ok=True)
    destination = RESULTS / f"{name}.json"
    destination.write_text(json.dumps(evidence, indent=2, allow_nan=False) + "\n")
    failures = sum(not passed for passed in checks.values())
    print(f"{name}: {len(checks) - failures}/{len(checks)} checks passed; {failures} failures (baseline 0); {destination.relative_to(ROOT)}")
    return int(failures != 0)
