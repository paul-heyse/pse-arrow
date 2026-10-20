# End-to-end execution performance review (2026-10-20)

- **Principal review:** `docs/design_review/reviews/design_review_end-to-end-execution-performance-target_2026-10-20.md`
- **Measurements:** `docs/design_review/evidence/end-to-end-execution-performance-2026-10-20/measurements.md`
- **Baseline:** pse-arrow `1ee81798a`, which is the campaign baseline plus documentation only.
- **Method:** the campaign cases were re-run on the profiling binaries with diagnostics off, using gdb/lldb/perf/callgrind attribution. This was combined with source review.
- **Decision:** Revise (G9 fails; G2/G7 fail narrowly).

## Headline measurements

**Small problems (2–16 unknowns, all adapters).** Each takes about 1.7–1.8 s per process (re-run, diagnostics off):

- Schema registry assembly: 0.94 s, about 53% of CPU.
- Authored-document admission: about 0.5–0.7 s per package, about 40% of CPU.
- Workflow invocation including the native solve: 11 ms, 3% of CPU or less.
- libc allocation/copy: 54% of cycles (one case).

**Banded-root sweep, N = 16–256.** Total time goes from about 2–3 s to 53.7 s, growing ≈N^2.5–3.0 above baseline (the source structure is cubic). At N = 256:

- 94% of CPU is modeling admission (`CheckedPackage::named_types_with` eager environments).
- Native work is 67 ms.

**Reference property packages.** These show 36–41 s `modeling.prepare` spans; a heater spends 58 s in source admission (campaign timelines).

**Long fitting, shooting and QP cases (35–59 s).** Only 1.2–6.4 s of each is inside the workflow invocation; the rest is preparation.

**Durable studies.** About 1.2 s of coordination per trivial point. Study width gives no throughput, and accounted pools read 13–22 GB against 0.29 GB RSS.

## Ranking

Numerical methods are the third lever, not the first. The order is:

1. Modeling admission: make it linear and reusable.
2. Fixed per-process/per-package preparation: static registry, trusted-schema fast path.
3. Orchestration.
4. Numerics at scale.

Several numerical defects should still be fixed early, because they are correctness or receipt issues:

- KINSOL step tolerance acting as a relative test (banded-64 refusal).
- Clarabel relative `tol_feas`.
- Primal-only warm starts treated as cold.
- Recycle default without Anderson acceleration.

## Target

Compile once, persist, bind values, solve natively, publish off-path. The full target and portfolio are in review §4. Wave 1 changes are in §11, and rule impacts RC01–RC08 are in §13.
