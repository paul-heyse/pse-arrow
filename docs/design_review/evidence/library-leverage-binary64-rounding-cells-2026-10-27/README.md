# Binary64 rounding-cell counterexamples

**Tested — 2026-10-27, source baseline `4f52fd239`, zero failure baseline.** This probe
challenges the proposed CBO-025 remedy in the comparative review at `b29fcf802`. It does
not execute the current cubic ladder or an implemented replacement.

## Question and method

Does `d ± ulp(d)/2` describe the exact round-to-nearest-even cell at a power of two?
The stdlib-only `probe.py` uses exact `Fraction` arithmetic and the adjacent finite
binary64 values from `math.nextafter`. For each of `1`, `-1`, `2` and `-2`, it constructs
an exact rational root inside the proposed symmetric cell but outside the true cell.
It checks the rounded witness and both midpoint ties independently with CPython's
`Fraction` conversion. No native library, canonical service or provider is started.

Command, from the repository root, using the pinned checkout interpreter:

```bash
scripts/pse-env -- .venv/bin/python docs/design_review/evidence/library-leverage-binary64-rounding-cells-2026-10-27/probe.py > docs/design_review/evidence/library-leverage-binary64-rounding-cells-2026-10-27/output.json
```

## Outcome

Exit 0; all four counterexamples and their midpoint-tie assertions passed, with zero
failed assertions. The exact inputs and observations are retained in `output.json`.
For `d = 1`, the real lower midpoint is `1 − 2⁻⁵⁴`, whereas the proposed lower endpoint
is `1 − 2⁻⁵³`. The rational root `1 − 3·2⁻⁵⁵` is inside the proposed cell but rounds to
the predecessor of `1`. The same error occurs on the opposite side at negative powers
of two. This disproves that symmetric formula as a general certificate.

Use the exact adjacent-value midpoints `(prev(d) + d)/2` and `(d + next(d))/2`, with
ties to the even significand. A polynomial sign certificate must additionally identify
the selected root within its one-root bracket; this arithmetic probe does not establish
that part of a root-refinement implementation.

## Limits

Only the four stated normal finite candidates are exercised. Signed zero, subnormal
transitions, overflow and multi-root bracket endpoints still need explicit treatment
and implementation tests before a replacement can be called Tested. This is evidence
against a proposed algorithmic detail, not product qualification or a speed measurement.
