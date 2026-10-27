# Native solver integration

**Group G6 of the [comparative review](README.md), 2026-10-27.**
This page compares complete options for the native-solver opportunities
[CBO-026](../codebase/native-solver-integration.md#cbo-026) to
[original CBO-032](https://github.com/paul-heyse/pse-arrow/blob/b29fcf802f29251961871b2e12fb2dc57111aefb/docs/library_leverage/codebase/native-solver-integration.md#cbo-032), and supplies the native input to
the joint judgment of [CBO-001](../codebase/cross-cutting-mechanisms.md#cbo-001), which G4
owns. It authorizes no implementation. Targets are *Proposed*.

| Item | Value |
| --- | --- |
| Reassessment | 2026-10-27 at source `4f52fd239`; consumes the complete original review and updated inputs at `b29fcf802`, base patterns 0.2 and additional patterns revision 3 (refined to 4 here). All named dispositions were reconsidered with representative current source, consumers and inherited evidence; unchanged evidence retains its original conditions. |
| Original source inspected | `main` at `3e9a2525b`. `git log dbc4c9436..HEAD` shows no change under `crates/pse-backend-native`, `crates/pse-uno-sys`, `crates/pse-ipopt-sys`, the vendored SUNDIALS/Diffsol/SuiteSparse trees, `docker/solvers` or `Cargo.toml`. The audit baseline therefore still describes this subsystem. |
| Accounts consumed | Codebase account (native page plus CBO-001, including the dated library-assessment corrections). Library accounts: `faer/sparse-lu-scratch-and-reuse`, `feral/*`, `klu/symbolic-estimates-and-measured-usage`, `diffsol/*`, `sundials/*`, `uno/*`, `pounce/*`, `clarabel/*`, `quick-cache/weighted-native-handle-retention`. Patterns: base edition 0.2 and additional patterns revision 2. |
| Library realizations | Installed pins: faer 0.24.4, FERAL 0.18.0 and POUNCE 0.12.0 at fork revision `7f5e2e9d`, `suitesparse_sys` 0.1.4, `sundials-sys` 0.6.2 (bundled SUNDIALS 7.1.1, locally patched), Diffsol 0.16.2 / `diffsol-la` 0.1.1, Clarabel 0.11.1, Uno `805a27ec` plus `uno-pse.patch`, `quick_cache` 0.6.24. Upgrade routes come from the library accounts. The SUNDIALS 7.2–7.9 changelog was re-read for this review. |
| Design standard | Core 3.4, process-simulator profile 1.5 and the pse-arrow binding, applied qualitatively. The authoritative contracts are numerical-execution §15.4, §16.5, §18.2, §18.3, §18.8 and §18.10. |
| Evidence | Mechanisms are *Interface-checked* by static reading of the files named in each section. Nothing was built, run or measured. Every target is *Proposed*. |

## Group summary

**The joint decision.** The native layer is not fragmented the way the audit framed it. Three of
the seven opportunities rest on a premise that the code does not bear out:

- **CBO-029.** There is one TNLP boundary plus one C-ABI dispatch, which are layers of a single
  chain. There are not two parallel translators.
- **CBO-031.** The pse cone form is already the hub. Each solver gets one spoke table.
- **CBO-032.** The library sensitivity surface is already used.

The opportunities with real value are narrower than first described. Three supplier-level
questions carry the most consequence:

1. **The vendored SUNDIALS tree should move to 7.9.0.** Its recorded trigger has fired, and the
   7.3–7.8 releases fix defects that matter here: Anderson-acceleration errors were not
   captured, and memory leaked after a failed `malloc`.
2. **The move needs a prerequisite that is a defect in its own right.** Native build identity
   does not cover changes to the vendored SUNDIALS patch at an unchanged package version.
3. **Conservative factor-storage bounds should live with the code that allocates.** The
   maintainer-owned POUNCE/FERAL fork already does this for LDLᵀ (`conservative_storage_bytes`
   in fork commit `a695be61`). The Markowitz LU bound can move there the same way.

The other recommendations are small, local consolidations:

- One faer-LU helper for three owners, with uniform typed failure classification.
- One quiescent process-gate helper for the IBEX, SCIP and PETSc gates.

Elsewhere, *retain current* is the strongest case:

- Diffsol linear solvers and adjoint (CBO-027).
- The Uno patch (the Uno part of CBO-028).
- The KINSOL session cache (part of CBO-030).
- Conic encodings and refinement (CBO-031).

**Interactions with other groups.**
- **G4 (CBO-001).** Owns the joint judgment on prepaid accounting. This page's
  [native input](#cbo-001-native) separates two things that the audit grouped together:
  - true mirrors of library internals;
  - deliberately structure-blind complete bounds (dense n²).

  These have different remedies.
- **G2 (CBO-015, CBO-017).** The process-gate helper recommended here is built on the
  `pse_diagnostics` `WakeSignal`, `Cancellation` and `WakeWatch` primitives. A G2 consolidation of
  waits or cancellation should keep those primitives, or migrate this helper with them.
- **G2/G3 (CBO-011, LOP-X005).** The decision not to use `quick_cache` here is about drop timing
  and strict LRU in a retention cache. It is not the drop-to-cancel concern, so it neither
  supports nor contradicts the single-flight conclusions.
- **G8 (CBO-039) and the hashing contract.** The native-build identity gap below concerns
  identity framing. G8 should see it if identity framing is reviewed there.
- **G7 (U2 and LOP-X006).** Supplier moves change bits. This page judges that acceptable under
  either answer to U2, provided the identity prerequisite holds.

**Shared prerequisites.** Fixing the native-build identity input must come before any change to
the vendored SUNDIALS numerics, whether the move to 7.9 or the removal of a local hunk. A
structure-derived bound check on real Jacobians (not run in this pass) must come before any change to the dense opaque factor
allowances.

**Reinforcing and competing choices.**

*Reinforcing:*
- The faer-LU helper (CBO-026) and the typed resource-failure latch for Diffsol (CBO-027) share
  one failure classification.
- The FERAL fork bound (CBO-001/026) and the KINSOL block preconditioner (CBO-026) share one
  owner.
- The SUNDIALS move (CBO-028) is the natural time to re-verify the SUNDIALS-derived workspace
  formulas (CBO-001).

*Competing:*
- Swapping faer LU for FERAL LU (rejected here) competes with the faer-LU helper.
- Adopting `quick_cache` competes with retaining the hand-written least-recently-used (LRU)
  session cache.

**Consequence-based priority** (separate from prerequisite order):
1. The native-build identity gap.
2. The SUNDIALS move to 7.9.0.
3. Typed failure classification for the faer and KLU owners in Diffsol.
4. The FERAL fork LU bound.
5. The quiescent gate helper, together with a decision on the HiGHS gate discipline.
6. Everything else: retain current, or retire the premise.

<a id="cbo-026"></a>

## CBO-026 — Sparse factorization wrappers split across faer, FERAL and KLU

**Responsibility and target intent.** The operation factors and solves sparse square or KKT
matrices, and answers numerical questions about them. The promised behaviour comes from three
places:
- §15.4: fresh numeric factors with shared immutable symbolic analysis for square response,
  `Par::Seq`, and refusal under strict memory when no complete fill bound exists;
- §18.8: charged opaque allowances;
- the preservation list: scratch charged before allocation, singular and fatal outcomes typed
  separately, observation names.

Each question has one estimator, and the questions differ:

| Question | Estimator | Source |
| --- | --- | --- |
| Diagnostic κ₁ (lower bound) | FERAL Hager–Higham, through `SparseLu::condition_estimate_1` and `Solver::estimate_condition_1norm` | `conditioning.rs` |
| Refactor acceptance | KLU `klu_l_rcond`, then `klu_l_condest` under the SUNDIALS 7.1.1 KLU setup policy | `dynamics/linear.rs`, `Klu::refactor` |
| Accuracy of an action | Normwise backward error | `square_response/actions.rs` `backward`; `kkt/path/arclength.rs` inline |
| Certified inertia | FERAL LDLᵀ only | `conditioning.rs`, `kkt/activity.rs` |

The audit's "one estimator per question" goal is therefore already met. Engineering accuracy
states explicitly that it reports backward error and "never forward error"
(`engineering_accuracy.rs`), so it has no consumer for a κ estimate.

**What actually repeats.** There are three faer LU owners, not two:
- `SparseFactor::prepare` in square response;
- `TangentWorkspace` in arclength;
- `FaerLu` in `dynamics/linear.rs` for Diffsol.

They repeat the same mechanics: symbolic analysis, scratch query, `MemBuffer::try_new`,
`factorize_numeric_lu`, mapping `LuError`/`FaerError`, and the finite check on the solution.
Two of them carry the same opaque allowance formula `n²·64 + 512n + 4096`
(`SparseFactor::allowance` and the `reservation` in `arclength::assemble`).

Three facts are decision-relevant:
- **Failure mapping is inconsistent.** Square response and arclength map `SymbolicSingular` to
  numerical, `OutOfMemory` to memory and `IndexOverflow` to contract. Diffsol `FaerLu` collapses
  every faer failure into `LinearSolverError::Other(String)`, which `integrator::native` reports
  as numerical. A faer allocation failure inside a Diffsol step is therefore retried by step
  reduction and then surfaces as a numerical failure, not a memory failure. The raw Diffsol `Klu`
  does the same with `KLU_OUT_OF_MEMORY`, which becomes `KluFailedToFactorize`.
- **Scratch admission is inconsistent.** Square response and arclength compare
  `StackReq::size_bytes()` with an allowance before `MemBuffer::try_new`. Diffsol `FaerLu` only
  attempts the fallible allocation.
- **Each FERAL use has a different contract.** Markowitz LU serves the one-shot diagnostic and
  the BTF-block preconditioner. LDLᵀ serves inertia. Only the LDLᵀ path charges a library
  estimate (`admit_factor`).

**Credible complete options.**
1. **Retain current.**
2. **One crate-local `SparseFactor` trait over faer, FERAL and KLU.** It would carry symbolic
   reuse, scratch admission, singularity classification, observation and solves.
3. **Swap faer LU for FERAL LU** in tangent and square response (`feral/unsymmetric-sparse-lu`).
4. **Run FERAL `HagerHighamOperator` over the faer and KLU factors**
   (`feral/hager-higham-estimator-over-any-factor`).
5. **A narrow faer-LU owner helper** for the three faer owners only, with per-library code
   elsewhere unchanged.

**Comparative argument.**
- **Option 2 erases real distinctions.** The owners differ in kind:
  - an `Arc`-shared immutable response factor, with a symbolic cache keyed by normalization;
  - a mutable per-attempt workspace;
  - Diffsol's `LinearSolver` deferred-error contract;
  - SUNDIALS-owned KLU under a local fixed-pattern integration;
  - inertia, which only LDLᵀ has.

  A common trait would either be lowest-common-denominator (and leaked through) or would carry
  every variant. Neither discharges more than option 5. It adds a shared abstraction with
  several unrelated consumers, which is the coupling LOP-09's countercondition warns against.
- **Option 3 regresses the prepay contract.** FERAL LU has no caller scratch and keeps
  thread-local solve buffers outside admission. Its Markowitz fill cannot be predicted. It has
  no multi-RHS solve, which square-response actions use. The gain, typed factor-time
  singularity, is already obtained by the finite-solution check. Rejected under the
  no-compromise rule.
- **Option 4 answers a question no consumer asks.** It would add up to about 11 extra solves per
  factor. Its only consumer would be a new forward-error report, which the accuracy contract
  deliberately does not make. Rejected as premise, not as capability: the type remains useful
  if a forward-error consumer appears.
- **Option 5 has the best change locality** at the cost of one small module, and it gives these
  owners one classification:
  - singular → numerical, except that faer's `SymbolicSingular` on a pattern §15.3 already
    validated is a structural or contract breach (PS-04), not a numerical failure; *(Revised after the numerical and solver methods challenge.)*
  - allocation → memory;
  - overflow → contract;
  - other → internal.

  It also gives one scratch-admission rule, one finite-solution rule and one allowance formula,
  while each owner keeps its own lifecycle.

**Revealing case.** A Diffsol BDF step where faer cannot grow `NumericLu` storage. With option 5
alone, Diffsol still shrinks the step and retries, because `set_linearisation` is infallible
and every `solve_in_place` error counts as a nonlinear failure. The classification does not
reach the outcome unless the solver also records a terminal typed cause in the integrator's
shared state. The callback boundary already records its terminal failures the same way
(§18.2: "only terminal failures latch"). `integrator::native` would then prefer that cause over
Diffsol's `TooManyNonlinearSolverFailures`. This is the CBO-027 residual and is recommended
there. A second case: arclength gives scratch half its byte limit (`bytes / 2`), while square
response gives scratch the whole request. The helper must therefore take the scratch budget as
an argument and must not impose a split.

**Disposition: Recommend (bounded, low priority).**
- **Enabling mechanisms.** A `pse-backend-native`-private module (the name is illustrative)
  built on faer 0.24.4:
  - `sparse::linalg::lu::factorize_symbolic_lu`;
  - `SymbolicLu::{factorize_numeric_lu_scratch, solve_in_place_scratch, factorize_numeric_lu}`;
  - `LuRef::solve_in_place_with_conj`;
  - `dyn_stack::{StackReq, MemBuffer::try_new, MemStack}`.

  It exposes:
  - an admitted scratch buffer, given a caller budget;
  - a classified factor result;
  - a checked solve;
  - one `opaque_allowance(n)`;
  - observation under a caller prefix, so `square_response.faer.*`, `arclength.faer.*` and
    `diffsol.faer.*` stay unchanged.
- **Responsibility boundaries.** Each owner keeps its ownership model, reuse key and evidence:
  - square response keeps the shared symbolic cache and `Arc` factors;
  - arclength keeps its family-keyed workspace;
  - Diffsol keeps deferred errors.

  The FERAL LU, FERAL LDLᵀ and KLU code stays where it is.
- **Domain rules retained.** `Par::Seq` or the admitted `Par`; backward-error limits. The first
  pass also listed "no FMA", which faer does not satisfy; see
  [faer kernels and the FMA-free premise](#faer-kernels-and-the-fma-free-premise). *(Revised after the numerical and solver methods challenge.)*
- **Intended behaviour changes.**
  - Diffsol `FaerLu` and `Klu` resource failures surface as memory or contract errors, not
    numerical ones (with the CBO-027 latch).
  - Diffsol `FaerLu` scratch is charged before allocation, against the allowance the session
    already charges for the factor (the existing `opaque_allowance(n)`), not a new and smaller
    budget. A tighter budget would start refusing dynamics that run today. *(Revised after the omission and preservation challenge.)*
- **Machinery removed.** Three error mappers and the duplicated allowance formula.
- **Machinery added.** One small module.
- **Transition.** Local and testable with the existing unit controls.

**Rule impacts.** None: this is a refactor within contracts. The Diffsol classification change
is a truthful-outcome correction under §18.6.

**Evidence.** The faer 0.24.4 APIs and error types are *Interface-checked*
(`src/sparse/linalg/lu.rs`). The three owners and `integrator::native` were read. The target is
*Proposed*.

<a id="cbo-027"></a>

## CBO-027 — Diffsol Newton linear solvers and adjoint checkpoints re-implemented around upstream limits

**Responsibility and target intent.** Newton linear solves in Diffsol's implicit schemes must
fail without panicking and shrink the step through Diffsol's own recovery. They must reuse KLU
storage, refuse nonfinite solutions and report work. Checkpointed adjoints must honour these
limits on the original clock:
- the checkpoint-population limit;
- cancellation;
- the accepted-step budgets (forward, replay and backward);
- sample jumps.

All of these are promised behaviour (ADR-0110 item 3; §18.8).

**Credible complete options.**
1. **Retain 0.16.2 with the local solvers and the driven adjoint loop.**
2. **Move to Diffsol 0.18.0 / `diffsol-la` 0.3.0.** This adds a required
   `solve_transpose_in_place`, adopts the library faer sparse LU's storage-reuse redesign and
   breaks several APIs (#373, #369).
3. **Use the library-driven checkpointing and backward pass**
   (`solve_with_checkpointing`, `solve_adjoint_backwards_pass`).
4. **Upstream contributions (U1).** A fallible `set_linearisation`, or a typed abort class that
   bypasses step shrinking. A step-observer or limit hook in `solve_with_checkpointing`, together
   with #404's step cap.

**Comparative argument.**
- **Option 3 loses promised behaviour.** It gives up the checkpoint limit, cancellation, the step
  budgets and sample emission. It also imports open defect #419: the algebraic adjoint is not
  updated after a cost jump. The local backward pass avoids #419 by re-initializing
  consistently after each sample jump (`adjoint.rs` module doc and backward loop).
- **Option 2 deletes nothing.** The library sparse LU still panics (`expect`), and its scratch
  still bypasses admission. KLU still has no refactor. The move adds a trait method to each local
  solver and API churn. It keeps the reasons for options 1 and 3 intact.
- **Option 4 would remove only the deferred-`Result` idiom.** Admission, nonfinite refusal,
  KLU refactor with its rcond/condest policy, observation and the loop's budgets all remain
  local. A typed abort class would add one more thing: resource failures would no longer be
  retried. The latch below gets the outcome right without that.

**What the review found locally.**
- The `adjoint.rs` module doc still gives the crate-private reset setter as the reason for
  recording checkpoints locally. The library assessment showed the real reasons are the limit,
  cancellation, budgets and samples. A root on the adjoint forward pass is already an error
  (`root on the adjoint forward pass`), so reset segments never arise. The doc should be
  corrected.
- Resource failures are reclassified as numerical (see CBO-026).

**Revealing case.** An allocation failure during the factorization in a DAE step. Today the step
shrinks until Diffsol's nonlinear-failure limit, then surfaces as numerical. PS-10 requires a
truthful outcome. The local latch delivers it without any upstream change.

**Disposition: Retain current**, with two local corrections:
- the module-doc reason;
- a terminal typed-cause latch for resource and contract failures in `FaerLu` and `Klu`,
  consumed by `integrator::native` (shared with CBO-026).

Revisit when a Diffsol release:
- makes `set_linearisation` fallible;
- adds a checkpointing observer or limit hook;
- fixes #419;
- or ships a numerical fix that the project needs.

The move to 0.18 alone is not worth it.

- **U1 = yes.** Optionally, open an upstream issue proposing a fallible linearisation with a
  "do not retry" error class, and an observer hook for `solve_with_checkpointing`. Low priority:
  even if accepted, the local solvers stay.
- **U1 = no.** As stated.

**Rule impacts.** None.

**Evidence.**
- *Interface-checked:* `dynamics/linear.rs`; `dynamics/integrator.rs` (`native`, the run
  dispatch); `dynamics/integrator/adjoint.rs` (root refusal, `StepBudget`, `max_checkpoints`,
  replay budget).
- Diffsol 0.18 and `main` facts are inherited from the [diffsol account](../libraries/diffsol/README.md),
  dated 2026-10-27.

<a id="cbo-028"></a>

## CBO-028 — Vendored supplier numerics whose premises may have aged

### SUNDIALS part

**Responsibility and target intent.** The bundled SUNDIALS (`sundials-sys` 0.6.2, native 7.1.1)
supplies IDAS, KINSOL and SUNLinSol_KLU. The local delta has three kinds of change:
- integration hooks: checkpoint population and budgets, the accepted-step budget, the KLU
  fixed pattern and work counters;
- build freshness;
- one genuine numerical correction: the quadrature history refreshed after `IDACalcIC`.

The record in [native-supplier-notes.md](../../../vendor/native-supplier-notes.md) and register
R-41 own the revisit.

**What changed upstream** (CHANGELOG re-read for this review):

| Release | Change | Relevance here |
| --- | --- | --- |
| 7.8.0 | IDAS quadrature predictor after `IDACalcIC` (commit `a7393b501`). | The trigger has fired. |
| 7.8.0 | CVODES, IDAS and KINSOL no longer leak memory after a failed `malloc`. | Relevant to honest storage under resource refusal. |
| 7.3.0 | KINSOL: "errors that occurred when computing Anderson acceleration were not captured" is fixed. | Truthful outcomes. At 7.1.1 the independent original-space assessment is the backstop. |
| 7.3.0 | KINSOL damping on the first Anderson iteration is fixed. The defect bites "unless `KINSetDamping` and `KINSetDampingAA` are both called with the same value". | `kinsol.rs` already sets both to `method.damping`, so pse-arrow avoids it. |
| 7.5.0 | Fixes a 7.3.0 regression: the Anderson depth was not reinitialized when one KINSOL instance solved repeatedly. | The session cache and `solve_with_restarts` reuse instances, so the target must be ≥ 7.5. |
| 7.6.0 | Linear-solver counters reset on reinitialization. `IDA_TOO_CLOSE` replaces some `IDA_ILL_INPUT` returns. Constraint failures are counted separately. | Observation and status-map changes. |
| 7.7.0 | CMake option names namespaced. The old names are deprecated. | The build-script options need checking. |
| 7.3.0 | "All work space functions" deprecated, for removal in 8.0.0. | See CBO-001. |

**Credible complete options.**
1. **Retain 7.1.1 with the local hunk.**
2. **Replace the bundled native tree with SUNDIALS 7.9.0** inside the vendored `sundials-sys`.
   Drop the quadrature hunk, rebase the integration hunks, and re-verify the version-coupled
   Rust sites. Eleven non-test sites cite 7.1.1 behaviour:
   - `dynamics/layout.rs` SPGMR/SPFGMR formulas;
   - `dynamics.rs` BBD bytes;
   - `kinsol.rs` counter boundaries;
   - `kinsol/restarts.rs` `kin_nni` reset;
   - `dynamics/idas/local_error.rs` payload sizes;
   - `sundials_errors.rs` handler stack;
   - `dynamics/idas.rs` statistics fields and history bound;
   - `dynamics/linear.rs` KLU setup policy.
3. **Wait for a new `sundials-sys` crate.** None has appeared since 2024-12-25.
4. **U1 route.** Contribute a `sundials-sys` release that bundles 7.9. The vendored delta would
   then reduce to the integration hunks.
5. **Go to 8.0 when it is released.** That removes the workspace queries (CBO-001).

**Comparative argument.**
- **Domain authority.** The supplier now owns the quadrature fix. Keeping the local hunk means
  owning a numerical correction to IDAS internals that upstream has made differently: it
  evaluates the quadrature right-hand side at the first `IDASolve`, not inside `IDACalcIC`.
- **Gains of option 2.** It retires the only numerical correction in the delta and picks up the
  two KINSOL/IDAS truthfulness and resource fixes.
- **Costs of option 2.** The rebase is not mechanical: 7.6 touched IDAS counters, returns and
  constraints, which sit next to the budget hunks. The version-coupled sites need
  re-verification. New return codes must be mapped. The changed counter-reset semantics must be
  checked against the session-reuse controls.
- **Option 5 is premature.** No 8.0 release has been inspected, and its workspace-query
  replacement is unknown.
- **Option 4 depends on U1** and is still "not yet available" until released.

**The bit-reproducibility question.** The move can change numerical results and counters;
it does not imply that every IDAS or KINSOL result changes bits.
Is that acceptable or remedy-open? It is **acceptable**, on these grounds:
- **The identity contract already absorbs supplier moves as declared identity changes.**
  - §16.5 frames the linked native build into `pse.solver.profile.v4` through
    `Table::build_identity` (`pse.native.build.v1`).
  - Recorded results keep their identities.
  - Replay distinguishes recorded-decision replay from tolerant reruns.
  - Export retains build premises, and a run missing them refuses.
  - §18.3 already disclaims bitwise equality across solvers.
- **It holds under either answer to U2.**
  - *Replay of recorded values:* unaffected.
  - *Bit-portable regeneration:* this applies to seeded sampling, not to solver reruns. A rerun
    under a new build is a new identity, and the old one refuses for its missing premise. Solver
    results already vary by CPU class, because faer selects SIMD kernels at runtime (see
    [numerical workflows](numerical-workflows.md#cbo-033) and LOP-X006 revision 3), so neither
    branch promises bit-identical solver results across hosts today. *(Revised after the omission and preservation challenge.)*

There is one condition. The identity must actually move, and today it does not always:

**Prerequisite: the native-build identity gap.** *Interface-checked*:
- `BackendExecution::build` defaults to `None` with the doc comment "`None` when the pinned crate
  fully determines the build" (`execution.rs`).
- The dynamics adapters (Diffsol and IDAS in `execution/dynamics.rs`) do not override it.
- KINSOL's fingerprint (`kinsol::build`) frames:
  - the SUNDIALS and KLU version macros;
  - compiler metadata;
  - `kinsol-source.txt`, which is only the dependency declaration text (`build.rs`
    `kinsol_profile`), not the vendored patch content.
- The supplier notes state that "Package versions stay unchanged" under local edits.

Two consequences follow:
- A change to the vendored SUNDIALS numerics at an unchanged version leaves every profile
  identity unchanged in every build. Removing or adding a hunk is such a change.
- A SUNDIALS version change is visible only in builds that link KINSOL.

**Remedy.** Frame the vendored supplier delta into native build identity. Options include
hashing `vendor/deltas/sundials-sys.patch` in a build script, as `pounce-source.txt` is hashed.
Equivalently, the dynamics adapters' `build()` could hash the SUNDIALS version together with
that delta digest. This is a correction within ADR-0108 item 14's meaning ("identity of the
linked native build beyond the pinned crate"). It changes every existing profile identity once.

**Revealing case.** A future agent removes the quadrature hunk while still at 7.1.1, for example
while testing. IDAS numerics change and no identity changes. The prerequisite closes that.

**Disposition (SUNDIALS): Recommend (medium priority; after the prerequisite).**
- **Target.** SUNDIALS 7.9.0 as the bundled tree of the vendored `sundials-sys` 0.6.2. The
  package version could optionally change to a local pre-release so that the declaration also
  moves.
  - Drop `IDAInitialQuadDerivatives` after `IDACalcIC` and the helper factoring in `idas.c`.
  - Rebase the integration hunks in `idaa.c`, `idaa_io.c`, `idas_impl.h`, `idas_io.c`,
    `idas.c`, `sunlinsol_klu.{h,c}` and `wrapper.h`.
  - Update `build.rs` to the namespaced CMake options if the old names stop working.
- **Consumer integration.**
  - Map `IDA_TOO_CLOSE` and any other new returns in the exhaustive status maps.
  - Re-derive the version-coupled formulas and counter boundaries listed above.
  - Run the three named quadrature controls, the KINSOL session-reuse and restart controls, and
    the KLU fixed-pattern controls.
- **Intended behaviour changes.**
  - Quadrature history is refreshed at the first `IDASolve`.
  - KINSOL Anderson-acceleration errors are captured.
  - Counters reset on reinitialization.
  - Profile identities move.
- **Machinery removed.** The only local numerical correction in the delta.
- **Machinery remaining.** Every integration hook.
- **U1 = yes.** Prefer contributing a `sundials-sys` release, and then follow it.
- **U1 = no.** Use the vendored replacement.

**Rule impacts.**
- Under the decisions table this is a "deferred trigger firing": a short ADR, with review at the
  maintainer's discretion.
- Upgrading a third-party dependency on its own needs no ADR.
- The identity prerequisite is arguably a bug fix within ADR-0108. If the maintainer reads it
  as a hashing-contract change, it needs an ADR and design review.

### Uno part

**Responsibility and target intent.** Uno provides filter trust-region SQP with an L-BFGS
operator, and SLP, over HiGHS subproblems. The patch carries four admitted numerical changes,
each with observable controls:
- tolerance forwarding;
- zero QP regularization;
- a current-subproblem feasibility seed;
- dense operator materialization.

It also carries integration hooks: scope, deadlines, memory latch and serial controls.
ADR-0157, ADR-0215 and §16.2 own the binding.

**Credible complete options.**
1. **Retain `805a27ec` plus the patch.**
2. **Rebase onto v2.9.2.** This brings the `qp_iteration_limit` safeguard, which adds a new
   `SubproblemStatus::ERROR` route, and nonbasic bound placement. Seven upstream commits touch
   the patched files.
3. **A BQPD QP engine**, which is matrix-free and would remove materialization. This is a new
   composition, not a version move.
4. **U1 route.** Contribute generic forwarding of subproblem-solver options to Uno.

**Comparative argument.**
- **v2.9.2's placement fix works on QP readback, not QP initialization.** The seed works around
  initialization: HiGHS 1.15.0 QP truncates small activities. On reading the code, v2.9.2
  probably does not remove the seed's premise; a QP-initialization check on the rebased Uno would settle it (not run in this pass).
- **The fix's gain is robustness only.** Off-bound nonbasic values could produce bound-violating
  trials. pse-arrow already treats domain violations as recoverable trials (§18.2), and
  original-space acceptance guards the final outcome independently.
- **The iteration safeguard is a new termination route.** It interacts with the forwarded
  deadlines and needs mapping and qualification.
- **The rebase is non-mechanical** and requires an image rebuild and Plan 49 re-qualification.
- **BQPD is a major composition change.** By the brief's rule it is out of scope here.

**Disposition (Uno): Retain current.** Revisit at the first of:
- the next solver-image rebuild;
- a check showing the seed is obsolete;
- an observed bound-violating-trial defect;
- an upstream forwarding feature (U1).

- **U1 = yes.** Recommend contributing a generic HiGHS-option forwarding hook covering
  tolerances, `qp_regularization_value`, `threads` and `parallel`. Three of the four numerical
  changes would then become configuration. Rebase onto the release that contains it.
- **U1 = no.** As stated.

**Rule impacts.** None, unless the composition changes (BQPD would need an ADR amendment of
ADR-0157).

**Evidence.**
- *Interface-checked:* `uno-pse.patch` (four files: `HiGHSSolver.{cpp,hpp}`,
  `HiGHSQuadraticProgram.cpp`, `NoRelaxation.cpp`); `pse_uno.cpp` convention calls;
  `Cargo.toml [workspace.metadata.pse.uno]`.
- Upstream facts at v2.9.2 are inherited from the [uno account](../libraries/uno/README.md).

<a id="cbo-029"></a>

## CBO-029 — Two NLP-oracle callback translators for three native NLP solvers

**Responsibility and target intent.** Present one `NlpOracle` to Ipopt, Uno and POUNCE with:
- panic containment;
- typed causes;
- no stale values;
- recovery to original coordinates (§18.2, §18.3).

**What the code actually does** (*Interface-checked*). There is one chain, run by "the one NLP
runner" (`execution/runner.rs`):

1. The oracle is wrapped in `tnlp::Adapter` (`tnlp.rs`). This adapter owns:
   - normalization;
   - bound transport and certification expansion;
   - starts and multiplier scaling;
   - derivative and linearity proofs;
   - presolve tapes (`ExpressionProvider`);
   - `CallbackState`.
2. The adapter feeds the pounce-presolve wrapper stack (`presolve/pipeline.rs`).
3. That stack is exposed again as one post-presolve `NlpOracle`: `pipeline::Transport`.
4. Each backend consumes the `Transport`:
   - **POUNCE** wraps it in the same `tnlp::Adapter` type with identity normalization
     (`pounce.rs`).
   - **Ipopt and Uno** use `foreign_nlp` (`uno/callback.rs`, compiled as `foreign_nlp`). This is
     the C-ABI dispatch: pointer and extent checks, `catch_unwind`, and status codes.
     - Ipopt has only the retained C++ TNLP sequence route; `ipopt.rs` re-exports
       `sequence::Sequence`.
     - Uno's conventions are already configuration: `UNO_ZERO_BASED_INDEXING` and
       `UNO_MULTIPLIER_POSITIVE` in `pse_uno.cpp`.

Both layers share `CallbackState` and `nlp_pattern::Pattern`.

**Credible complete options.**
1. **Retain current.**
2. **Generate both the Rust `TNLP` and the C callback set from one oracle semantics** (the
   pounce account's suggestion).
3. **Feed POUNCE the presolve TNLP directly**, skipping the `Transport` round trip.
4. **Feed all three through `pounce-cinterface`'s Ipopt C ABI.**

**Comparative argument.**
- **The premise of "two translators to unify" does not hold.** The two pieces sit at different
  layers: normalization and presolve semantics in one, C-ABI containment in the other. Any new
  derivative or normalization rule lands once, in the `Adapter`. The C dispatch has no
  normalization; it refuses an un-transported oracle (`oracle.normalization().is_some()` →
  internal error in `uno.rs` and `ipopt/sequence.rs`). What remains duplicated is a few
  `finite` and dimension helpers.
- **Option 3 would create a second post-presolve path.** It would bypass `Transport`, which is
  also where structural retention (`structural::retain_nlp`) and the separator attach.
- **Option 4 loses typed Rust failures, presolve tapes and rich finalize data** (pounce account).

**Disposition: Premise rejected; retain current.** The research question "can one TNLP feed the
C ABIs?" is answered: it already does, through `Transport` and `foreign_nlp`.

**Rule impacts.** None.

<a id="cbo-030"></a>

## CBO-030 — Native session lifecycles re-expressed per adapter

**Responsibility and target intent.**
- Construct, use and destroy native state on the owning thread.
- Keep population, resident storage and active compute as separate owners.
- Release compute while waiting on a native exclusion, and never wait for capacity while holding
  one: ADR-0176; §18.8, "a tentative native gate is released before blocking for compute
  capacity".
- Free exactly once, including on unwind.
- Respect the solver-global states: the HiGHS scheduler, the SCIP TPI pool, PETSc
  initialization and IBEX.

**What is already shared** (*Interface-checked*):
- `execution::Retained`: one owner of a `Box<dyn Any>` adapter session, with backend-specific
  neutral reset.
- `pause_compute`, `try_resume_compute` and `resume_compute`.
- `NativeWait`, the registration-before-inspection wait on `WakeSignal`, `Cancellation` and
  abandonment, used by IBEX, SCIP and PETSc.
- `compute_scoped`.

**What is still repeated.** `petsc.rs` `Admission::enter`, `scip.rs` (the concurrent owner) and
`root_isolation.rs` (the IBEX guard) each run the same loop:
1. pause;
2. loop over:
   1. register `NativeWait`;
   2. check;
   3. `try_lock`;
   4. `try_resume_compute`;
   5. if capacity is unavailable: drop the gate, notify, run a blocking `resume_compute`, pause
      again, and retry;
   6. otherwise wait to the deadline.

They differ only in:
- the health check (SCIP's terminal-failure flag);
- the poison policy (IBEX treats poison as "library unavailable", the others as internal);
- IBEX's algorithm checkpoint and its `catch_unwind`/resume-on-panic;
- the nesting flag.

**`highs_lifecycle.rs` follows a different discipline.** Its `read_at_transition` and `exclusive`
acquire the `RwLock` gate, then call blocking `resume_compute` while holding it. No deadlock
cycle was found: every waiter for the gate has paused its compute first. It is still a
divergence from the try-resume rule that §18.8 states for IBEX, SCIP and PETSc. §18.8 does not
list HiGHS.

**Credible complete options.**
1. **Retain current.**
2. **A local `QuiescentGate` helper** next to `NativeWait`, parameterized by:
   - the lock;
   - the change signal;
   - a health predicate;
   - the poison policy;
   - an optional checkpoint;
   - the nesting flag.

   It would serve PETSc, SCIP and IBEX.
3. **A generic native-session trait or RAII framework** covering acquire, quiesce, bytes and
   drop for every adapter.
4. **Replace the KINSOL per-thread LRU (`implicit.rs` `sessions`)** with `quick_cache` 0.6.24
   `unsync::Cache` and a `Weighter`.
5. **Generic pool crates.** Screened by the library account. None counts bytes or integrates
   cancellation.

**Comparative argument.**
- **Option 2 states the ADR-0176/ADR-0195 rule once in code.** A stricter rule then reaches every
  gate: for example quarantine health, or a deadline nuance.
- **Option 3 would erase genuine differences.** Neutral reset, TPI quarantine, retained C++
  applications and FERAL leases all differ. `Retained` already owns the common part.
- **Option 4 discharges only the eviction loop (about 10 lines) and changes semantics:**
  - `quick_cache` evicts during `insert`, so KINSOL sessions would be freed inside the
    thread-local `RefCell` borrow. The current code explicitly frees outside the borrow. Fixing
    that needs `Lifecycle` request-state plumbing.
  - The eviction policy becomes CLOCK-PRO, while the public `budget_sessions` doc promises
    "least recently used first".
  - A budget change requires rebuilding the cache.

  Retain the hand-written cache.

**Revealing case.** IBEX's guard must restore compute on panic, and it treats a poisoned lock as
"no library". A helper that hard-codes "poison means internal error" would change IBEX's
outcome class. The helper must take the poison policy and panic restoration as parameters. If it
cannot do that cleanly, IBEX stays separate and the helper serves PETSc and SCIP only.

**Disposition: Recommend (bounded, low-to-medium priority).**
- **Target.** A `pub(crate)` `QuiescentGate` acquisition helper in `execution.rs`, built on the
  existing `NativeWait`, `pause_compute`, `try_resume_compute` and `resume_compute`, used by
  PETSc, SCIP and IBEX.
- **Machinery removed.** Two or three copies of the acquisition loop.
- **Unchanged.** Solver-global state stays with each adapter: PETSc init/finalize, the SCIP
  quarantine, IBEX random state.

**Unresolved sub-question for the owner.** Should the HiGHS scheduler gate adopt the same
try-resume discipline, or is holding the read gate while resuming deliberate (writer preference
through `WAITERS`)? The answer should be recorded beside §18.8's gate paragraph. Settling
evidence: the owner's intent, plus a check that any change keeps Uno's writer preference.

**Rule impacts.** None for the helper. If HiGHS is brought under the rule, §18.8's gate
paragraph is amended through the normal design route.

**Evidence.** *Interface-checked:* `petsc.rs`, `scip.rs` and `root_isolation.rs` acquisition
loops; `highs_lifecycle.rs`; the `execution.rs` compute and `NativeWait` APIs; `implicit.rs`
`sessions`. For `quick_cache`, the `unsync` semantics are inherited from the account.

<a id="cbo-031"></a>

## CBO-031 — Convex and conic encodings hand-synchronized per solver

**Responsibility and target intent.**
- Solve LP, QP and cone programs.
- Certify infeasibility independently in original coordinates.
- Map reports back to coefficient rows.
- Keep row order and sign identity: `pse.cone.layout.v3` and `pse.cone.lowered-row.v1` feed
  hashes.

**What the code actually does** (*Interface-checked*).
- **The pse cone form is the hub.** It comprises `conic::{SparseMatrix, Cone}` and
  `ConicProblem`. Its conventions match Clarabel's: the PSD upper triangle is scaled, and
  `Cone::Power` maps directly to `PowerConeT`. Clarabel therefore needs only type mapping inside
  the adapter.
- **Coefficient programs routed to Clarabel are lowered once** (`ConicProblem::from_coefficients`).
- **POUNCE-convex has one forward table** (`pounce_convex::form`, `rows: Vec<Target>`, with
  `psd_rows` and the power-cone `[2, 0, 1]`). That one table is used for both the forward
  mapping and the multiplier and certificate mapping. It refuses the generalized power cone.
- **HiGHS keeps its row-bounds form.** Its rays are converted into the cone layout once
  (`certificate::from_highs_rays`, `farkas_from_rows`).
- **Verification runs once, on the hub** (`certificate::verify`).
- **The Clarabel refinement (ADR-0204) reuses the retained solver.**
  `Session::solve_once` calls `update_settings` and `set_termination_callback`. This answers
  the clarabel account's unsettled premise.

**Credible complete options.**
1. **Retain current.**
2. **A shared "standard conic form" for POUNCE and Clarabel.**
3. **Derive the per-solver tables from a declarative description.**
4. **Replace the refinement with Clarabel options** (`iterative_refinement_*`, equilibration).

**Comparative argument.**
- **Option 2 already exists:** it is the pse form.
- **Option 3 would add a description language for two small tables** that are already
  single-source in each direction.
- **Option 4 is ruled out.** Clarabel's options act on each KKT solve, not on the
  original-coordinate quality ratio (clarabel account).
- **Redundant routes are deliberate.** POUNCE-convex is explicit-only, and the differential tests
  `pounce_convex_qp_matches_highs` and `clarabel_lp_matches_highs` depend on them.

**Disposition: Premise rejected (hand-synchronization); retain current.** The generalized power
cone on POUNCE is a supplier capability gap. Adding it in the fork would be new functionality
and is not proposed.

**Rule impacts.** None.

<a id="cbo-032"></a>

## CBO-032 — KKT-point sensitivity built beside fuller library sensitivity surfaces

**Confirmation of retirement.** The premise is false, as the library assessment found and this
review confirms. The following are in use:
- `kkt/sensitivity.rs` and `kkt/advance.rs` use `SensApplication`, `IndexSchurData`,
  `SensOptions`, `compute_reduced_hessian` and `compute_reduced_hessian_eigen`;
- `kkt/path.rs` uses `boundcheck::step_along_path`, `BoundMultiplier` and
  `rowlimit::{RowLimitView, WatchedRow}`;
- `kkt/activity.rs` uses `SensBacksolver` and `BoundRow`;
- `kkt/path/qp.rs` uses `pounce_qp::ParametricActiveSetSolver`.

The remaining surfaces are `pounce-sensitivity` (engine-coupled), `activity_kernel` (IPM
iterates) and sIPOPT (not built, and same-solver). They violate backend independence.

**The one residual question**, whether `boundcheck::refine_step_onto_bounds` (sIPOPT fix-relax)
should serve the advanced step, is not a leverage question. `kkt/advance.rs` *refuses* a
prediction whose active set changes; that is its validity contract. Using fix-relax would
change which predictions are admitted. That is a functional change with accuracy tradeoffs.

**Disposition: Premise rejected; CBO-032 retired in this reassessment.** Its named library
facilities are already consumed; the codebase occurrence and obsolete applicability match were
removed, while the reusable library type and this rationale remain. The ID will not be reused. Route the fix-relax
question to a functional-extension review if the maintainer wants it.

<a id="faer-kernels-and-the-fma-free-premise"></a>

## faer kernels and the FMA-free premise

*Added after the numerical and solver methods challenge; Interface-checked by reading the installed
faer 0.24.4 and pulp 0.22.3 sources.*

The workspace forbids `target-cpu=native` so that LLVM cannot contract `a*b + c` into fused
multiply-adds. faer reaches FMA by a different route that this rule does not cover:
- faer 0.24.4 declares `pulp` with `features = ["x86-v3"]` unconditionally, so the workspace's
  `default-features = false` does not remove it.
- pulp selects its V3 path at run time (`Arch::new()`), and that path uses `_mm256_fmadd_pd`.
- This is not the only dispatch owner. With the workspace-enabled `std` feature on x86_64,
  faer `linalg/matmul/mod.rs` selects `private_gemm_x86::InstrSet::{Avx512, Avx256}` directly
  using CPU features; small products use `nano_gemm::planless`, with `gemm` on the remaining
  path. Those routes are not disabled by a pulp V3 patch. nalgebra can separately enter
  `matrixmultiply`.
- `pse.native.build.v1` frames compile-time facts only, so results from these kernels vary with
  the CPU class under an unchanged identity. The numerical-workflows page already relies on this
  for fit statistics; this page had wrongly listed "no FMA" as a retained rule. Source
  establishes the potential variation; no cross-CPU product difference was measured here.

**Disposition: Remedy-open; preserve the standing FMA-free requirement.** A supplier override
or maintained patch must cover the complete consumed factorization/kernel closure. No complete
no-FMA configuration was demonstrated, and a pulp-only patch is insufficient. A released
upstream override remains a U1 option; a local maintained patch remains another, with neither
its full coverage nor its throughput cost established.

ISA attribution or identity framing can explain which implementation produced a result, but
neither removes FMA nor satisfies the arithmetic requirement. U2 concerns design-input replay
versus regeneration; it does not waive solver arithmetic rules. If dispatch attribution is
selected, it must describe actual relevant kernels, build configuration and arithmetic policy,
not assume one pulp ISA label accounts for the whole closure. Keep supplier-build identity
repair as a separate prerequisite. A change to the arithmetic requirement itself belongs to its
decision owner, not this review.

<a id="cbo-001-native"></a>

## Native input to CBO-001 (for G4's joint judgment)

The native layer contains two different shapes that the audit grouped together.

**A. Mirrors of library internals** (LOP-X001 proper):

| Site | What it mirrors | Library route |
| --- | --- | --- |
| `kinsol/block.rs` `checked_extent` | FERAL 0.18 Markowitz internals: projections, CSR copies, lazy heaps, historical fill. Cubic in block size. | None in FERAL. The fork is maintainer-owned, and fork commit `a695be61` added `Solver::conservative_storage_bytes` for the bounded LDLᵀ profile ("source-derived and conservative", fork `source-extension.md`). The same remedy fits LU: a fork-resident `SparseLu` conservative owner bound beside the allocating code. |
| `dynamics/layout.rs` SPGMR/SPFGMR formulas; `kinsol.rs` `native_header_bytes`, `native_krylov_storage`, `anderson_storage`; `dynamics.rs` BBD bytes | SUNDIALS 7.1.1 allocation shapes (`SUNLinSolSpace` and others). | No prediction API through 7.9. The workspace queries are post-hoc, deprecated in 7.3 and removed in 8.0. Keep the formulas, but re-derive them at every SUNDIALS move (CBO-028). |
| Uno bridge `H(n,k)` and `D(d)` (`uno-binding.md`) | Uno and HiGHS representation storage. | None. Already source-documented in the binding and enforced in the patched C++. |

**B. Deliberately structure-blind complete bounds** (not mirrors):

| Site | Formula |
| --- | --- |
| Square-response `SparseFactor::allowance` and `arclength::assemble` | n²·64 + 512n + 4096 |
| IDAS KLU `PreparedLayout::direct_bytes` | n²·32 (also for adjoint and second order) |

These satisfy §15.4: "input-shape estimates never become that missing bound". Square response
refuses strict-memory requests outright. Their AP-07 consequence is growth: they refuse large
sparse problems whose actual factors are small. Their remedy is a *structure-derived* complete
bound, not a library reporter.

Any proposed BTF-derived replacement must include symbolic storage, off-diagonal `nzoff`,
growth/reallocation overlap and work arrays as well as block factors. A sum of block-square
terms alone is not an established complete bound. The real-Jacobian check remains unrun.

**The kinds of library size information** (LOP-X001 revision 2):

| Library | Kind | Notes |
| --- | --- | --- |
| faer | Symbolic-phase prediction, for scratch only | `factorize_numeric_lu_scratch` / `solve_in_place_scratch` → `StackReq`. Used with an allowance check by square response and arclength. Diffsol `FaerLu` does not compare it with any allowance. |
| faer | No factor-storage prediction | `NumericLu` hides capacity. Public lower-level APIs (`colamd::order`, `qr::column_counts_ata`, `SymbolicLu::col_perm`) let a caller compute a George–Ng structural bound on nnz(L+U) for partial-pivoting LU. That bound does not cover the supernodal path's relaxed-supernode padding or faer's `try_reserve` growth, so it is not a complete storage bound without mirroring faer internals. The owner-side remedy would be a faer API (U1). |
| FERAL LDLᵀ | Symbolic-phase prediction (kind 1) | `symbolic_factorize().work_estimate()`, used by `conditioning::admit_factor`. It excludes delayed-pivot growth, and the full envelope stays reserved. |
| FERAL LDLᵀ | Library conservative bound (kind 2) | `conservative_storage_bytes`, fork-added. |
| FERAL LU | None | `conditioning::jacobian_condition` runs Markowitz LU with no factor pre-admission, only the enclosing worker's opaque allowance. `kinsol/block.rs` uses the cubic mirror. |
| KLU | Reuse-time exactness (kind 5) | `klu_l_refactor` allocates nothing, so later refreshes can be charged exactly (Diffsol `Klu::refactor`; KINSOL through the local `SUNLinSol_KLUGetMemUsage`). |
| KLU | No upper bound for the first factor | Growth is `newlusize = memgrow·lusize + 2n + 1` through `realloc`, with old and new allocations transiently live and no dense cap (`klu_kernel.c`). `klu_l_symbolic` exposes `nblocks`, `R`, `maxblock` and `nzoff`. A "dense within each BTF diagonal block, plus caller-set `memgrow`/`initmem*` growth" bound is complete. Recycle flowsheets often form one large strongly connected block, where it degenerates toward n². |
| SUNDIALS | Post-hoc only (kind 6), deprecated | 7.3 deprecates "all work space functions" (`KINGetWorkSpace`, `KINGetLinWorkSpace`, `IDABBDPrecGetWorkSpace`), and 8.0 removes them. Whether `N_VSpace` and `SUNMatSpace` are included is not explicit in the changelog text read. They are used here for the *retained* session weight, after construction, which is legitimate post-hoc use. At 8.0 they need a local integration hook in the vendored tree. |

**Recommendation input for G4.**
1. **Move the FERAL Markowitz bound into the fork as a `SparseLu` conservative owner bound.**
   - This is not U1-dependent: the fork is maintainer-owned with established admission
     integrations. Whether this practice sits outside U1 is one of the [open questions](README.md#open-questions).
   - Its first consumer is the KINSOL block preconditioner.
   - Do **not** apply a cubic bound to `jacobian_condition` without checking diagnostic sizes.
     A cubic charge on a whole Jacobian would refuse diagnostics that run today. That would be a
     regression, so a diagnostic-size check or a tighter Gilbert–Peierls bound comes first.
2. **Keep the SUNDIALS-derived formulas.** Add them to the SUNDIALS move's re-verification list.
   Do not move them into the vendored C: that would grow the rebase burden CBO-028 is trying to
   shrink.
3. **Treat replacing the dense allowances with structure-derived bounds as Unresolved, pending
   a structure-derived bound check on real Jacobians.** This means BTF-block dense bounds for KLU, and George–Ng bounds for the faer
   simplicial path only.
4. **Charge Diffsol `FaerLu` scratch against an explicit budget** (CBO-026 helper).

## Pattern notes

- **LOP-X001.** Two refinements are recorded in [additional patterns](../assessment-patterns.md) revision 3:
  - when a supplier is already vendored or forked with integration practice, a predictor
    resident in the supplier is the owner-side remedy;
  - structure-blind complete bounds are distinct from mirrors.
- **LOP-X006.** A refinement is proposed: the identity that keys reproducibility must cover local
  patches at unchanged package versions.
- **LOP-02 / LOP-09.** A counter-cue, recorded as a base-framework note: check whether
  apparently parallel translators are successive layers of one chain, or spokes of one hub
  whose tables are single forward maps (CBO-029, CBO-031).
