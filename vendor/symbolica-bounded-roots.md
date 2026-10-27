# Bounded exact cubic root isolation

The workspace retains Symbolica **3.0.1** from its published crate archive, selected by
an exact-version path override. [The archive and complete delta](deltas/readme.md)
retain every published source, normalized manifest, lockfile, license and VCS record.
The local changes are in `src/poly/univariate.rs` and `src/poly/univariate/roots.rs`.
No upstream contact or unreleased API is involved.

The cubic consumer needs cancellation on the original operation clock and admission
before exact arithmetic, stack growth and retained output. The released API provides
neither a bounded VAS-CF overload nor an allocation callback. An external check after
that call cannot undo its allocation. The local overload therefore shares the existing
quotient/remainder and VAS-CF implementations, retaining the original unbounded APIs
for other consumers. It adds:

- `square_free_part_bounded`: deterministic Euclidean reduction at degree at most three,
  with admission before each quotient step. No preferred modular polynomial GCD is used.
- `isolate_real_root_intervals_square_free_bounded`: consumes that square-free polynomial
  directly, bypasses Yun factorization and the global root cache, and admits every VAS
  stage. A refusal returns no partial list.
- Exact endpoint normalization: VAS can return the designated root at an endpoint of a
  nondegenerate interval. Endpoint roots become exact singleton intervals; an interval
  remains only when exact one-sided signs identify an open-interior root. Repeated
  occurrences of the same exact rational endpoint refer to one exact root. Distinct
  roots never merge through floating-point conversion.

`RealRootIsolationStep` reports incremental scalar exact-arithmetic operations and peak
mathematical storage. A scalar power, gcd or rational arithmetic call is one operation;
integer bit lengths separately bound its numeric payload. This is not an OS RSS bound
or a claim of cancellation within a scalar library call. The consumer debits the existing
`EvaluationLimits`, checks its original cancellation/deadline before each stage, and grows
only the positive high-water excess of the retained construction admission. Each
bounded supplier overload reports a monotonic peak, retaining coverage for integer
capacities after canonical reduction shrinks their significant bits.

For each degree-three quotient step, current rational limb height `h` gives quotient
limbs at most `2h`, multiplied divisor limbs at most `3h`, and unreduced remainder
subtraction limbs at most `4h+1`. The admitted `8(h+4)` envelope includes canonical
reduction temporaries and vector overlap. Initial derivatives multiply by integers at
most three. Euclidean degree descent bounds the sequence of stages, while every stage
is independently charged before its arithmetic.

For VAS-CF, integer coefficient height `h` gives local-max root-bound and reciprocal
limbs at most `h+5`. Scaling degree at most three and the following shift give coefficient
limbs at most `4h+32`. A current projective matrix limb height `m` gives matrix products
and additions at most `m+h+8`. Each callback includes all pending states and already
retained endpoint limbs. The common envelope covers 512 integer slots, each with its
Rust owner and twice rounded limb payload, plus vector owners: at most three pending
intervals, six pre-normalization root intervals, temporary polynomials, arithmetic
scratch and old/new vector capacity. Rational-to-primitive-integer setup and exact
endpoint normalization have their own preceding callbacks and explicit limb envelopes.
No guessed global transform count or coefficient-height admission cap is introduced.

The consumer's local refiner separately derives its finite bisection, rational limb,
MPFR precision and storage envelopes. It certifies rational-to-binary64 conversion with
directed MPFR enclosures and bit-identical nearest-even binary64 endpoint images.
The selected contract remains original-residual/branch-verified candidate selection;
it does not promise correct rounding of an algebraic root. If an exact root has no
finite selected binary64 representative, the whole enumeration refuses; an omitted
unrepresentable root cannot make another association appear unique.

The maintenance obligation is to rebase the complete delta, recheck these source-derived
bounds and run the targeted cubic/conversion controls whenever this supplier changes.
An upstream bounded overload can replace the local hooks only when it meets these same
consumed contracts. Current execution and acceptance status belongs to the owning plan.
