# Library contract refresh for the integrated solve pipeline

Supporting evidence for
[the principal review](../../reviews/design_review_integrated-solve-pipeline_2026-10-03.md).
This file records bounded interface/source observations, not a separate architecture verdict
or finding-status owner. No solver was installed, built, benchmarked or executed for this
refresh.

## Baseline and method

The consumer baseline is HEAD `f0b902589723a86bc6755ce1c76b45a2ef90224b` plus the
uncommitted Plan 25k tree inspected on 2026-10-03. Workspace pins remain POUNCE family
0.12.0, FERAL 0.18.0, faer 0.24.4 and sundials-sys 0.6.2 with bundled SUNDIALS 7.1.1.
These are read from `Cargo.toml` and the resolved local release sources, not inferred
from current online documentation.

The native-solver-libraries and symbolica-faer-oximo skills, together with the earlier
[pinned capability study](../solver-acceleration-2026-10-03/pinned-library-capabilities.md),
provided discovery routes. Context7 resolution and queries refreshed SUNDIALS setup reuse
and PETSc domain-error handling. Two Uno resolution queries produced unrelated projects;
official upstream source supplied the fallback evidence. Current documentation does not
override pinned-release contracts.

## POUNCE path following: a useful capability, not a drop-in factor replacement

**Interface-checked.** In pounce-sens-core 0.12.0,
`boundcheck::step_along_path` delegates solves through `SensBacksolver`, walks bound
breakpoints, and returns the accumulated step and `PathSegment`s. It does not relinearize
an NLP between those breakpoints. Its documentation explicitly reports that reaching
`max_iter` causes the remaining perturbation to be taken under the last active set.
Segment-limit exhaustion therefore must not be represented as a completely tracked path.

The release methods, `supports_release`, `bound_rows` and natural-unit conversions are
part of the consumed contract. The default release methods return false. The current
consumer's `kkt::KktFactor` implements the base backsolve and bound-row mapping, but not
release. It contains an unregularized active-set matrix with explicit active-constraint
rows; the library's bound classifier also describes interior-point sigma-based rows.
An inequality row in the consumer's layout is not automatically a slack in the library's
primal prefix.

Preserve M1b. Implement an explicit coordinate/activity/release binding, including row
inequalities and immutable factor provenance. Use pounce-qp's homotopy for its admitted QP
class. NLP output remains a start proposal requiring nonlinear correction; neither a
base-point KKT certificate nor a linearized active-set screen certifies the target NLP.
There is no new library or theory rejection here. The earlier U1 is narrowed to this
integration obligation.

Decisive sources: `pounce-sens-core-0.12.0/src/boundcheck.rs::step_along_path`,
`src/backsolver.rs::SensBacksolver`; consumer
`crates/pse-backend-native/src/kkt.rs::{KktFactor,NormalizedFactor}` and
`src/kkt/advance.rs::predict`. These local release paths are supplied by the native
skill's corpus and the Cargo registry.

## Retention must reach the actual factorization object

**Interface-checked.** FERAL 0.18.0 `numeric::solver::Solver` retains `last_symbolic`,
the pattern fingerprint and the actual last pattern. POUNCE 0.12.0 application factories
are invoked per `optimize_tnlp` call. The consumer reuses its application when compatible,
clears its option table and installs a new `default_backend_factory_with_sink` on every
solve (`pounce.rs::Session::solve`). Retaining an application or a factory description alone
does not establish cross-solve retention of the actual FERAL `Solver`.

M5's adapter session must own the reusable numerical object, with separate predicates for
symbolic-pattern compatibility and numerical-factor validity. Pattern, ordering, scaling,
solver configuration, structure transformation and ownership must agree as required by
the reused product. Each solve still needs its own outcomes and statistics. Do not promise
reuse from the `warm_start_same_structure` spelling alone.

SUNDIALS' setup APIs provide a separate, fitting reuse mechanism: `KINSetNoInitSetup`,
`KINSetMaxSetupCalls`, `KINSetMaxSubSetupCalls` and residual monitoring. The current
consumer explicitly sets `KINSetNoInitSetup(...,0)` in `kinsol.rs`. A stale setup can be
useful for iteration without being a fresh root-sensitivity factor at the accepted point.
The [official KINSOL usage source](https://github.com/llnl/sundials/blob/v7.1.1/doc/kinsol/guide/source/Usage/index.rst)
owns the exact 7.1.1 semantics.

## POUNCE second opinions must become visible attempts

**Interface-checked.** `pounce-algorithm-0.12.0/src/second_opinion.rs` exposes
`second_opinion_rungs`. Its availability inputs distinguish the failure trigger, options
already in effect and actual factor-quality escalations. A rung that would repeat the
same configuration is omitted. Iteration-limit handling is conditional on quality
escalation, rather than an unconditional request for another solve.

Use these library-owned rung descriptions through the shared executor. Snapshot and
restore the complete effective configuration, including whether a key was set; prevent
user, typed-profile and rung settings from competing. Record each attempted route, start,
configuration, budget and outcome. Do not substitute an opaque internal retry ladder for
this record. The enclosing deadline and terminal-error latch apply to every rung.

## Uno: retain the recommendation and specify the foreign boundary

**Interface-checked.** Official source at commit
[`805a27ecf802eec33d1aea85ae73efa8a24f71be`](https://github.com/cvanaret/Uno/tree/805a27ecf802eec33d1aea85ae73efa8a24f71be)
identifies the C API as 2.9.0. The
[C interface](https://github.com/cvanaret/Uno/blob/805a27ecf802eec33d1aea85ae73efa8a24f71be/interfaces/C/Uno_C_API.h)
offers sparse derivatives and termination callbacks. Positive evaluation returns become
evaluation exceptions in its implementation. The
[trust-region strategy](https://github.com/cvanaret/Uno/blob/805a27ecf802eec33d1aea85ae73efa8a24f71be/uno/ingredients/globalization_mechanisms/TrustRegionStrategy.cpp)
catches evaluation errors and reduces its radius.

Retain Uno for bounded trust-region SQP/SLP. Specify sparse ordering, multiplier sign,
integer width, callback lifetime and a terminal-error latch distinct from recoverable
trials. Inspecting `uno_optimize` and getters does not establish a universally nonthrowing
C ABI; guard foreign exceptions in the integration owner. Pin the chosen source/build
before implementation. Acquisition and backend qualification are integration work, not
a benchmark prerequisite for the method.

## PETSc: correct the blanket trust-region exclusion

**Interface-checked.** The earlier review used a development branch. Tagged
[PETSc v3.24.0 function evaluation](https://github.com/petsc/petsc/blob/v3.24.0/src/snes/interface/snes.c)
flags invalid-domain output so its norm is invalid. Its
[trust-region implementation](https://github.com/petsc/petsc/blob/v3.24.0/src/snes/impls/tr/tr.c)
rejects an invalid trial objective and shrinks the region. An invalid initial point still
terminates; this method also explicitly rejects variable bounds.

Thus the prior categorical claim that PETSc trust-region methods abort on domain errors
cannot justify excluding this tagged implementation. It is not a claim about every PETSc
method, release or failure phase. Preserve Uno for bounded NLP and keep capability-scoped
PETSc integration eligible, particularly where library-owned pseudo-transient or nonlinear
composition would remove proposed project controllers. Neither adoption of a whole second
stack nor rejection of the whole library follows from one method's behavior.

The [TSPSEUDO implementation](https://petsc.org/release/src/ts/impls/pseudo/posindep.c.html)
and [NASM interface](https://petsc.org/release/manualpages/SNES/SNESNASMSetSubdomains/)
are discovery evidence for those additional roles. Their chosen tagged methods need their
own bounds, failure, termination, global-state and ownership binding. No new runtime fit
or speed claim is made.
