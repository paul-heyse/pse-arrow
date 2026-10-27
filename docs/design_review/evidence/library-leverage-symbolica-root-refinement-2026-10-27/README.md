# Symbolica root-interval refinement probe

This is evidence for the library-leverage applicability assessment of
[CBO-025](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-025), dated 2026-10-27.
The source baseline is `dbc4c9436`, inspected at `c2f8b485a`.

The probe characterizes Symbolica 3.0.1 in isolation, using synthetic polynomials. It does not
call any pse-arrow code. In particular it does not exercise the consumer's cubic fallback ladder
(`real_points` in `crates/pse-math/src/implicit_cubic.rs`), which wraps this API.

## Question

The consumer's ladder says "the public refiner requires non-root endpoints". A static reading of
`UnivariatePolynomial<RationalField>::refine_root_interval` in Symbolica 3.0.1
(`src/poly/univariate/roots.rs`, around line 2176) found three things:

- A root at the *left* endpoint is classified through the sign of the derivative.
- A root at the *right* endpoint is guarded only by
  `debug_assert!(u.evaluate(&interval.1).is_negative() != left_bound_neg)`.
- The loop stops on a *relative* width, `(b − a) / |a + b| > tol`.

Static reading could not settle how the refiner actually behaves in debug and release builds for
four kinds of input:

- (a) a root at the right endpoint;
- (b) an interval symmetric about 0, or an interval straddling a root at 0;
- (c) a root at the left endpoint;
- (d) the isolator's own intervals, whose endpoint is another root.

## Environment

- **Library:** `symbolica` =3.0.1 with `default-features = false` and features `integer-gmp` and
  `float-mpfr`, the same selection as the workspace. `numerica` 3.0.1 is resolved from a copy of
  the workspace `Cargo.lock`, and cargo pruned it to this crate's dependency closure.
- **Toolchain:** the repository toolchain `rustc 1.101.0-nightly (c1070d693 2026-09-28)`
  (`nightly-2026-09-29`). Linking uses the repository `.cargo/config.toml` (clang plus mold), and
  `panic = "unwind"` is set in both profiles.
- **Host:** Linux 7.0.0-38-generic x86_64, 32 logical CPUs.
- **Probe crate:** originally built in agent scratch outside the repository, with a separate
  `--target-dir` kept away from the workspace target directory. Its
  [Cargo.toml](probe/Cargo.toml), [src/main.rs](probe/src/main.rs) and raw `probe/outputs/`
  are retained. The lockfile and build outputs are not retained.
- **Inputs:** each case calls `refine_root_interval((a, b), &tol)` once, with `tol = 2^-54` (the
  consumer's precision). The call runs inside `catch_unwind`, one case per process. The
  polynomials are built with `UnivariatePolynomial::from_coefficients(&Q, …)`:
  - `p123 = x³ − 6x² + 11x − 6`, with roots 1, 2 and 3;
  - `pm0p = x³ − x`, with roots −1, 0 and 1;
  - `pq = 4x³ − x² − 4x + 1`, with roots −1, 1/4 and 1.

## Commands

Run from the repository root on 2026-10-27:

```bash
P=<scratch>/probe/refine-root
scripts/pse-env -- cargo build --offline --manifest-path $P/Cargo.toml --target-dir $P/target
scripts/pse-env -- cargo build --release --offline --manifest-path $P/Cargo.toml --target-dir $P/target
for prof in debug release; do for c in iso n0 a1 a2 b1 b2 b3 c1 c2 d1 d2; do
  scripts/pse-env -- /usr/bin/time -f "exit=%x maxrss_kb=%M wall=%e" \
    timeout 20 $P/target/$prof/symbolica-refine-root-probe $c > $P/outputs/$prof-$c.txt 2>&1
done; done
```

Each case ran once. The raw outputs are kept in scratch at `$P/outputs/{debug,release}-<case>.txt`.

## Outcome

**Tested** (Symbolica 3.0.1, the conditions above, one run per case):

| Case | Input | Debug build | Release build |
|---|---|---|---|
| n0 (control) | p123, (3/2, 5/2), interior root 2 | `[2, 2+2⁻⁵²]`, correct | same, correct |
| a1 | p123, (3/2, 2], root at right end, p > 0 inside | **panic**: `assertion failed: u.evaluate(&interval.1).is_negative() != left_bound_neg` (roots.rs:2202) | `[2−2⁻⁵⁴, 2]`, correct |
| a2 | p123, (5/2, 3], root at right end, p < 0 inside | `[3−2⁻⁵², 3]`, correct | same, correct |
| b1 | pm0p, (−1/2, 1/2), root 0 | **panic**: `Division by 0` (numerica rational.rs:773) | **panic**, same message |
| b2 | pq, (−1/2, 1/2), root 1/4 | **panic**: `Division by 0` | **panic**, same message |
| b3 | pm0p, (−1/2, 1/4), root 0 | **no return**: killed by `timeout 20` (exit 124); RSS stayed at about 22 MB | **no return**: exit 124; RSS stayed at about 8.7 MB |
| c1 | p123, [1, 3/2), root at left end, p > 0 inside | **panic**: same `debug_assert` | **wrong result, silently**: `[3/2−2⁻⁵³, 3/2]` contains no root (p(hi) = 3/8) |
| c2 | p123, [2, 5/2), root at left end, p < 0 inside | **panic**: same `debug_assert` | **wrong result, silently**: `[5/2−2⁻⁵², 5/2]` contains no root (p(hi) = −3/8) |
| d1 | p123, isolator interval (0, 2] for root 1; 2 is another root | `[1−2⁻⁵⁴, 1]`, correct | same, correct |
| d2 | pq, isolator interval (0, 1] for root 1/4; 1 is another root | **panic**: same `debug_assert` | `[1/4, 1/4+2⁻⁵⁵]`, correct |

`isolate_real_root_intervals` on the same polynomials returned:

- `p123`: `(0, 2)`, `(2, 2)` and `(8/3, 16/5)`;
- `pm0p`: `(−1, −1/2)`, `(0, 0)` and `(1/2, 1)`;
- `pq`: `(−2, 0)`, `(0, 1)` and `(1, 1)`.

So the isolator itself emits intervals whose endpoint is a different root (case d). It returns
exact roots as degenerate intervals.

### Interpretation

- **Endpoint roots.** An endpoint root is not handled uniformly in 3.0.1.
  - A root at either end can trip the `debug_assert`, so debug builds panic.
  - In release builds, a *left*-endpoint root makes bisection move away from the root and return
    a narrow interval that contains no root, with no error (c1, c2).
  - A *right*-endpoint root is refined correctly in release builds (a1, d2).
  - The consumer's guard is therefore needed for correctness, not only precision, in both build
    profiles. That guard is: "if p(a) = 0 or p(b) = 0, use the `IsolatedRoot` fallback".
- **Relative stopping rule.**
  - It panics on any interval with a + b = 0 (b1, b2).
  - It does not terminate when the interval keeps straddling a root at 0 (b3): the ratio
    (b − a)/|a + b| stays at or above 1 while both ends approach 0.
  - On these three polynomials the isolator returned 0 as an exact degenerate interval and kept
    the other intervals on one side of 0. So the (b) cases were *not observed* on the
    isolator-to-refiner path. That is an observation on three polynomials, not a guarantee.
- **The consumer's ladder** sends endpoint-root intervals to the fallback before calling the
  refiner. That guard covers cases a, c and d. The guard does not cover a zero-symmetric or
  zero-straddling interval; covering them depends on the isolator never producing one.

## Limits

- One run per case, with three synthetic polynomials of degree 3 and the consumer's tolerance
  only.
- Not run: other tolerances, higher degrees, roots very close to (but not at) 0, and repeated
  roots.
- The b3 non-termination was observed for 20 s at constant RSS. Longer runs, and the eventual
  growth of rational sizes, were not measured.
- The probe does not call the consumer's `real_points`. Its correctness on these inputs is
  inferred from its documented guard and not tested here.
- Released Symbolica only. Unreleased `main` at `bfb481cdc` has no change to
  `refine_root_interval` (diff inspected, not run).
