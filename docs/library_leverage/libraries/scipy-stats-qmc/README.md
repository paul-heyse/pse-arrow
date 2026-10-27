# scipy.stats.qmc (Python route)

Realizations considered:
- **Installed.** `scipy` 1.18.1 for Python ≥ 3.12 and 1.17.1 below, pinned in the `test`
  dependency group as test-time oracles. `numpy` 2.5.3 / 2.4.6 is in the `test` group and
  in the optional `array` extra. Neither is a runtime dependency of the `pse` package.
- **Promotion route.** Adding `scipy` to runtime dependencies.

Research date 2026-10-27.

Sources:
- The installed module `.venv/lib/python*/site-packages/scipy/stats/_qmc.py`, scipy
  1.18.1: class and method signatures.
- `pyproject.toml` dependency groups.

## Types

### scipy-qmc/design-engines
- **Need it serves:** A complete, documented Python design toolkit.
  - Scrambled Sobol and Halton.
  - LHS with strength 1 or 2 (orthogonal-array based) and optional `random-cd` or `lloyd`
    optimization.
  - Poisson disk.
  - Discrepancy metrics.
  - Bound scaling.
  - Integer draws.
  - Correlated normal QMC.
- **Recognizable code shapes:** Python helpers that build sweep grids or random designs
  by hand.
- **Mechanisms and APIs:**
  - `qmc.Sobol(d, scramble=True, bits=None, rng=..., optimization=None)` with
    `random_base2(m)` and `fast_forward(n)`.
  - `qmc.Halton(d, scramble=True, rng=...)`.
  - `qmc.LatinHypercube(d, scramble=True, strength=1|2, optimization=..., rng=...)`.
  - `QMCEngine.integers(l_bounds, u_bounds=, n=, endpoint=)`.
  - `qmc.scale(sample, l_bounds, u_bounds, reverse=False)`, which validates `l < u`.
  - `qmc.discrepancy` and `geometric_discrepancy`.
  - `qmc.MultivariateNormalQMC(mean, cov | cov_root, inv_transform=True, engine=, rng=)`.
- **Obligations discharged:**
  - Every design construction listed above.
  - Correlated normal QMC: covariance root plus inverse transform.
  - Integer level draws.
  - Quality metrics.
- **Residual caller obligations:**
  - Pass a seeded `numpy.random.Generator`.
  - Record the realized points.
  - Convert to the Rust boundary.
- **Semantic conditions:**
  - **Streams.** Seeding goes through numpy `Generator` (PCG64 by default). numpy keeps
    bit-generator streams stable but documents that `Generator` distribution methods may
    change between versions. That statement comes from general knowledge of numpy's
    policy and was not re-read here.
  - **Halton.** The docstring notes that `scramble=False` "does not ensure deterministic
    output".
  - **Float math.** Transforms use numpy and libm floats.
- **Version / build conditions:** scipy 1.18.1 / 1.17.1 split by interpreter, as pinned.
- **Limits and negative results:** No Rust-side use without a Python boundary crossing.
- **Evidence:** Interface-checked: class and method signatures in the installed `_qmc.py`.

## Current applicability

### [CBO-033](../../../library_leverage/codebase/numerical-workflows.md#cbo-033) — No producer of sampling or experimental designs
- **Types:** `scipy-qmc/design-engines`.
- **Realization:** Promote `scipy` from the `test` group to a runtime or optional extra
  dependency. The `pyproject.toml` comment records that import-linter currently keeps
  test oracles out of `pse`.
- **Conclusion:** fits with conditions. It answers the CBO's "Python helper" option with
  the richest design catalogue inspected. The cost is a runtime scientific-Python
  dependency, and generation would sit outside the Rust replay path unless the realized
  points are recorded.
- **What it would discharge / what remains local:**
  - **Discharged.** Design construction and metrics.
  - **Local.** Bound resolution from declarations, seed policy and recording points as
    inputs.
- **Prerequisites, configuration or extension:** A dependency-boundary change for
  `scipy` in `pse`.
- **Semantic gaps against the CBO's preservation constraints:** Regeneration depends on
  numpy and scipy versions and on the platform libm. Replay must rest on recorded points.
- **Unsettled premises and what would settle them:** Where designs are authored, which is
  the CBO's own open question. Rust-side generation keeps designs inside native
  evidence; Python-side generation reuses scipy.
- **Evidence:** Interface-checked. The application is Proposed.
