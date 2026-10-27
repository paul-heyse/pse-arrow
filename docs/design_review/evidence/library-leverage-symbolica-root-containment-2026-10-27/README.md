# Symbolica cross-factor root-interval containment probe

This is evidence for the library-leverage opportunity review of
[CBO-025](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-025) (cubic
refinement ladder), dated 2026-10-27. Source inspected at `9cb8e4f4a`;
`crates/pse-math/src/implicit_cubic.rs` was unmodified in the working tree (last changed at
`4de4c68d6`). It extends the earlier
[root-refinement probe](../library-leverage-symbolica-root-refinement-2026-10-27/README.md),
which characterized `refine_root_interval` alone and did not exercise repeated roots or the
consumer's ladder.

## Question

1. Does Symbolica 3.0.1 `isolate_real_root_intervals` emit, for a cubic with a repeated rational
   root, an isolating interval that strictly contains another root? The static trace predicted
   `(0, 4)` and `(1, 2)` for 12x³ − 68x² + 123x − 72 = (3x − 8)(2x − 3)².
2. Does the consumer's ladder (`real_points` in `implicit_cubic.rs`: endpoint screen
   p(a) = 0 / p(b) = 0, `refine_root_interval` at 2⁻⁵⁴, binary64 agreement check, fallback to
   `isolate_real_roots` + `IsolatedRoot::refined`) lose or misreport a root for such inputs, in
   debug and in release?

## Inspected source

- Symbolica 3.0.1 installed source: `src/poly/univariate.rs` (`isolate_real_root_intervals` for
  `IntegerRing`, `isolate_real_roots_square_free`, `max_real_root_bound`) and
  `src/poly/univariate/roots.rs` (`isolate_real_root_intervals` and `refine_root_interval` for
  `RationalField`, `refine_root_interval_until_disjoint`). The overlap test in
  `refine_root_interval_until_disjoint` checks only whether an endpoint of the *first* interval
  lies in the second, so an earlier interval that strictly contains a later one is reported
  disjoint. `refine_root_interval` squares-free the *whole* polynomial and guards its
  sign-change precondition only with `debug_assert!` (roots.rs:2202).
- Consumer: `real_points` in `crates/pse-math/src/implicit_cubic.rs`, and the coefficient path in
  `CubicWorker::solve` (binary64 coefficients → `Rational::try_from`, trailing zeros popped,
  `UnivariatePolynomial::from_coefficients(&Q, …)`). `crate::initialize()` only sets the
  optional licence, disables tracing and registers symbols; it does not affect root arithmetic.

## Method

The probe does not link pse-math: `real_points` is private, and reaching it through
`CubicRoots` needs test-only problem constructors. Instead `probe/src/main.rs` carries a
**verbatim replica** of `real_points`. `diff` against the repository function shows only the
error type (`MathError` → `String`) and one added line recording that the fallback branch was
entered. Library calls, their order, the 2⁻⁵⁴ precision and the screens are unchanged. The
coefficients go through the consumer's binary64 → `Rational::try_from` path.

For each p = (αx − β)²(γx − δ) with distinct roots, deduplicated by the root pair
(β/α, δ/γ) because Symbolica strips content, the probe records:

- **straddle0:** a non-degenerate interval with a < 0 < b.
- **containment:** a non-degenerate interval holding both distinct roots strictly inside.
- **no sign change:** the square-free part u = (αx − β)(γx − δ), evaluated exactly, has equal
  nonzero sign at both ends of a non-degenerate interval.
- **endpoint root:** p(a) = 0 or p(b) = 0 on a non-degenerate interval.
- **ladder outcome:** panic (with message), points, and whether the fallback was taken.
- **recovered / lost / duplicated / spurious:** each true root is matched by points within
  1e-14·max(|r|, 1), the consumer's own unit-test criterion.
- **iso route:** whether `isolate_real_roots` + `refined(2⁻⁵⁴)` alone returns exactly the two
  roots.

Each case runs inside `catch_unwind`, with a panic hook capturing the message.

## Environment

- **Library:** `symbolica` =3.0.1, `default-features = false`, features `integer-gmp` and
  `float-mpfr`, the workspace selection. Versions resolved from a copy of the workspace
  `Cargo.lock`, built `--offline`.
- **Profiles:** the probe mirrors the workspace's relevant settings. Dev keeps Cargo's default
  `debug-assertions = true` for every package, dependencies at opt-level 3, which is what the
  workspace `[profile.dev]` / `[profile.dev.package."*"]` give Symbolica. Release has
  debug-assertions off. `panic = "unwind"` in both.
- **Toolchain:** the repository toolchain, `rustc 1.101.0-nightly (c1070d693 2026-09-28)`. The
  repository `.cargo/config.toml` applies because cargo runs from the repository root.
- **Host:** Linux 7.0.0-38-generic x86_64, 32 logical CPUs.
- **Probe crate:** built in agent scratch with its own `--target-dir`. `probe/` keeps
  `Cargo.toml`, `src/main.rs` and raw outputs. The lockfile and target directory are not
  retained.

## Commands

Run from the repository root on 2026-10-27:

```bash
P=<scratch>/probes/symbolica-containment
scripts/pse-env -- cargo build --offline --manifest-path $P/Cargo.toml --target-dir $P/target
scripts/pse-env -- cargo build --release --offline --manifest-path $P/Cargo.toml --target-dir $P/target
for prof in debug release; do B=$P/target/$prof/symbolica-root-containment-probe
  for c in "2 3 3 8" "4 3 5 8" "4 -3 5 -8" "1 -12 3 -11" "2 1 1 3"; do
    timeout 60 scripts/pse-env -- $B case $c > $P/outputs/$prof-case-${c// /_}.txt 2>&1; done
  for c in "-10 37 -22 3" "-10 47 -54 9" "-6 11 -6 1"; do
    timeout 60 scripts/pse-env -- $B poly $c > $P/outputs/$prof-poly-${c// /_}.txt 2>&1; done
  for r in "0 12" "-12 12"; do set -- $r
    scripts/pse-env -- /usr/bin/time -f "exit=%x maxrss_kb=%M wall=%e" \
      timeout 600 $B sweep $1 $2 > $P/outputs/$prof-sweep_${1}_${2}.csv 2>$P/outputs/$prof-sweep_${1}_${2}.time
  done; done
```

`sweep LO HI` takes α, γ ∈ 1..=6 and β, δ ∈ LO..=HI. `0 12` is the assigned sweep, with all roots
≥ 0. `-12 12` extends it to negative and mixed-sign roots. The sweep outputs are deterministic
(exact arithmetic), so `probe/outputs/` retains only the summary and the flagged rows
(`*-flagged.csv`: containment, straddle, any non-`ok` ladder, lost or spurious points, iso-route
failure). The full CSVs regenerate from the commands. Every run exited 0. The sweeps took
≤ 13 s wall and ≤ 56 MB RSS.

## Outcome

### Worked example and mirrors

*Tested*, one run per case:

| Case | `isolate_real_root_intervals` | Debug ladder | Release ladder |
|---|---|---|---|
| (2x−3)²(3x−8), roots 3/2 (double) and 8/3 | `(0,4)` m1 and `(1,2)` m2. Both roots lie strictly inside `(0,4)`; u(0) = 24, u(4) = 20 | **panic**: `assertion failed: u.evaluate(&interval.1).is_negative() != left_bound_neg` (roots.rs:2202) | **`[1.5, 1.5]`**, no fallback. 8/3 is lost and 3/2 duplicated |
| (4x−3)²(5x−8), roots 3/4 and 8/5 | `(0,2)` m1 and `(1/2,1)` m2, containment | **panic**, same assertion | **`[0.75, 0.75]`**. 8/5 is lost |
| (4x+3)²(5x+8), the mirror: roots −3/4 and −8/5 | `(−2,0)` m1 and `(−1,−1/2)` m2, containment | **panic**, same assertion | `[−1.6, −0.75]`, correct *by chance*: bisection of `(−2,0)` reached the outer root |
| (x+12)²(3x+11), control | `(−16,−8)` and `(−4,0)`, no containment | correct via fallback | same |
| (2x−1)²(x−3), endpoint control | `(0,1/2)` with 1/2 a root, and `(2,4)` | correct via fallback | same |

So prediction 1 is **reproduced exactly**: `(0, 4)` and `(1, 2)` for the worked example.

### Sweep summary

*Tested*. Counts are distinct root pairs:

| Flag | 0..=12 debug | 0..=12 release | −12..=12 debug | −12..=12 release |
|---|---|---|---|---|
| Distinct (double, simple) pairs | 2162 | 2162 | 8556 | 8556 |
| Interval straddling 0 | **0** | **0** | **0** | **0** |
| Containment (both roots strictly inside one interval) | 34 | 34 | 90 | 90 |
| No sign change of u across an interval | 34 (= containment) | 34 | 90 (= containment) | 90 |
| Endpoint is a root | 661 | 661 | 1322 | 1322 |
| Ladder panicked | **34** (all containment) | 0 | **90** (all containment) | 0 |
| Ladder returned, but lost a root (and duplicated the other) | n/a | **20** | n/a | **20** |
| Ladder returned a spurious (non-root) point | 0 | 0 | 0 | 0 |
| Containment rescued, all roots recovered | 0 | 14 (all via fallback) | 0 | 70 (69 via fallback, 1 by chance) |
| Ladder took the `IsolatedRoot` fallback | 1586 | 1600 | 5582 | 5651 |
| `isolate_real_roots` route alone incorrect | 0 | 0 | 0 | 0 |

The **20 release losses** are all positive-root pairs. Every lost root is the *simple* root, and
the returned pair is two copies of the double root:

- double root 3/2, simple 8/3, 10/3, 11/3;
- double root 3/4, simple 4/3, 5/3, 8/3, 10/3, 11/3, 8/5, 9/5, 11/6;
- double root 5/4, simple 8/3, 10/3, 11/3;
- double root 7/4, simple 8/3, 10/3, 11/3;
- double root 7/5, simple 8/3, 10/3, 11/3.

In each, `refine_root_interval` on the containing interval converged to the inner (double)
root, and its two ends agreed in binary64, so the screen accepted it. No later interval tripped
the fallback.

### Further observations

- **The fallback is the common path, not an exception.** 4282 of the 7166 −12..=12 pairs that
  have no endpoint root and no containment still took the fallback. The cause is the binary64
  agreement check. `refine_root_interval`'s relative stop leaves an interval up to about 1 ulp
  wide, which often straddles a binary64 rounding boundary for a non-dyadic root. For
  (x+12)²(3x+11), the root −11/3 refined to `[-3.666666666666667, -3.6666666666666665]`.
- **The isolator often emits root endpoints even for square-free cubics.** For (x−1)(x−2)(x−3)
  it emits `(0,2)` beside `(2,2)`. So most fallbacks have one of these two causes, and the
  process-global `RootCache` is populated on the majority of solves in this family.
- **Containment never involved two dyadic roots** within the sweep: 0 of 2352 both-dyadic pairs.
  Integer coefficients such as 12, −68, 123, −72 are exact binary64, so the consumer's
  coefficient path does admit the losing inputs.

## Interpretation for the consumer (`CubicWorker::solve`)

Every returned point is checked against the original residual, and acceptance requires exactly
one verified point in the declared branch interval. Modelled (not executed through pse-math),
the outcomes are:

- **Release, lost cases:** both returned points are the double root.
  - A branch holding only the double root sees two verified copies, and refuses with "exactly
    one … root".
  - A branch holding only the simple root sees none, and refuses.
  - Both are **false refusals of a valid unique root**.
  - No case returned a non-root, so no wrong acceptance was observed.
- **Debug, all containment cases:** the `debug_assert` panic propagates out of
  `CubicWorker::solve`. pse-math has no non-test `catch_unwind` on this path; handling further
  up the runtime was not examined.
- **Release, rescued cases:** these are correct only incidentally. A later interval failed the
  screen, or bisection happened to reach the outer root.

## Implications for the G5 recommendation (CBO-025)

- The draft's hazard 2 (strict containment across square-free factors) is **reproduced**, with
  the predicted intervals.
  - Its consequence is profile-dependent: a debug panic, and in release a duplicate of the inner
    root with the outer root lost. That matches the draft's static trace.
  - The draft's tentative "release returns a duplicate of the inner root rather than a non-root"
    holds across 8556 pairs. It remains an observation, not a proof.
- Hazard 1 (zero-straddling intervals) is **unreached** across all 8556 pairs, consistent with
  the draft's static argument.
- Hazard 3 (unbounded `RootCache` retention) is **stronger than the draft states**.
  - The fallback runs on roughly two thirds of this family, mostly because of the binary64
    agreement rule and isolator root endpoints. Containment is a minor cause.
  - Option 2 (complete the precondition screen but keep the fallback) would therefore keep
    `IsolatedRoot` caching on the main path, unless its stop rule is also changed.
  - This supports option 4. Its proposed stop rule (ends round to the same *or adjacent*
    binary64) addresses the dominant fallback trigger.
- `isolate_real_roots` + `refined` was correct on every input. The case against option 3 remains
  retention and absolute-radius semantics, not correctness.

## Limits

- The probe replicates `real_points`; it does not call pse-math. Coefficient evaluation through
  the compiled evaluator, `problem.verify` and branch selection are modelled, not executed.
- One polynomial family, (αx − β)²(γx − δ) with small integers, plus three square-free controls.
  Triple roots, irrational near-double roots and larger coefficients were not swept.
- The "fallback is common" observation is measured on this family and three square-free
  controls only; its frequency on production cubics (for example Peng–Robinson compressibility
  cubics) is not measured.
- The sweep's iso-route column itself populates `RootCache`, so the sweep RSS figures do not
  isolate the ladder's retention. Retention growth was not measured.
- Released Symbolica 3.0.1 only.
