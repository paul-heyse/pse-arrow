---
title: Library-owned transformations and public native workflow
status: complete
date: 2026-09-24
adrs: [ADR-0082, ADR-0083, ADR-0084]
phase: 1
---

# M15–M16 execution packet

Implements the approved M15–M16 scope of Plan 14. Existing M00–M14 work is the
baseline, not a mechanism to recreate. User choices: typed declarations/builders
are primary; qualified automatic presolve is the default; Python has blocking and
asyncio access to the same cancellable native job. No legacy migration or fallback.

## Contracts and ordered packages

| Package | Implementation and acceptance obligation | State |
|---|---|---|
| M15.1 | Typed Off/Auto/required-pass policy; requested/eligible/applied/effective reports; distinct semantic, transformation, native reuse and warm identities | complete |
| M15.2 | Independent Symbolica affine-row and objective facts, preserved guards/aliases/parameter assumptions, pure Salsa dependencies | complete |
| M15.3 | Bounded native FbbtTape projection; unsupported operations opaque; genuine shared ExpressionProvider; complete/partial/unavailable coverage | complete |
| M15.4 | Shared original TNLP, PresolveTnlp and outer LinearEqElimTnlp for POUNCE/direct Ipopt; affine constant normalization; qualified tolerances and original-space finalization | complete |
| M15.5 | Positive source-indexed scales mapped into native library facilities; unit conversion distinct from numerical and factor scaling | complete |
| M15.6 | Owned original objective/constraint/bound observations, physical violations, fixed-value recovery, qualified duals and independent KKT checks | complete |
| M15.7 | Single library warm projection, full expression-sensitive invalidation, explicit reuse classes and required-reuse refusal | complete |
| M16.1 | Registry-authoritative typed builders/document admission into atomic immutable model revisions and existing compiler inputs | complete |
| M16.2 | Runtime/model/preparation/job/result APIs; native initialization, solve/sequences, cones and graph operations; actual linked capability discovery | complete |
| M16.4 | Generated physical result/provenance/diagnostic contracts, checked retained Arrow batches and stream ownership | complete |
| M16.3 | Thin PyO3 boundary, typed Python builders, blocking and asyncio completion on one executor, cancellation through native join | complete |
| M16.5 | Explicit control-last exact Delta publication and existing settlement recovery; immutable results, no solver replay | complete |
| M16.6 | Delete replaced boundary variants/callers; regenerate contracts/stubs; reconcile current-state documentation | complete |

### Transformation ownership

Continuous NLP adapters share pounce-presolve. HiGHS and Clarabel keep their native
pre/postsolve, and KINSOL keeps its square-system/scaling contract. All-fixed
problems evaluate directly. Auxiliary nonlinear elimination is explicit and only
the safe coupling policy is admitted; no automatic aggressive elimination or
objective-changing LICQ remedy. Required unsupported passes fail admission.

The library owns transformations and recovery. Project records describe effects,
not an independently executable transformation IR. Per-row coefficient facts must
work in mixed nonlinear models. FBBT covers the pinned FbbtOp vocabulary, with
Opaque for unsupported providers/control flow. It never replaces domain guards.
Normalize proved affine constants for the library's homogeneous linear-row
propagation interface. Record fixed upstream coefficient/equality tolerances;
never pretend unsupported tolerance settings were applied. Qualify reductions
against declared intervals and original coefficient support. Retain source-space
proof and diagnostic attribution; raw bound crossings are not infeasibility proofs.

The wrapper order is original adapter, row presolve, affine elimination, native
scaling. Native finalization traverses recovery once. Intermediate or placeholder
values from failed library reevaluation must not become physical observations.
Use the original worker to independently evaluate the candidate and preserve
native status even if validation fails. Physical feasibility, termination,
candidate presence and multiplier validity remain separate.

Canonical reported multipliers use L = sense*f + lambda*g - z_lower*x + z_upper*x.
Library-owned scaling and bound attribution are undone once; recovered multipliers
need original-space stationarity/complementarity checks. Unsupported/failed recovery
is typed missing data, never zeros or an unqualified sensitivity claim.

Warm seeds are in original coordinates; bases/working sets require exact native
layout compatibility. Library projection has one owner. Expression tapes, bounds,
values, provider contracts, maps and scales enter identities. A missing library
fingerprint forces rebuild; its fingerprint alone cannot see tape edits. Mutable
presolve/solver objects remain worker-owned, outside Salsa.

### Public workflow and durable meaning

Runtime, ModelBuilder/CaseBuilder, ModelRevision, PreparedCase, RunHandle,
RunResult and existing Publication form the public surface. Builders and package
documents enter the same typed declaration admission. Registry DTOs are authority;
compiler objects are derived views. Existing DSL expressions remain source inputs,
without YAML round trips or a Python mathematical engine. Unsupported source
constructs fail explicitly. Invalid edits do not replace the previous revision.

Native requests retain class-specific representations and backend options.
Linked availability differs from model eligibility. Providers select admitted
native factories, not Python callbacks. Failed/limited/cancelled outcomes and
unattempted sequence steps are inspectable data, not successful solutions.

Pin pyo3-async-runtimes 0.29.0 with tokio-runtime and borrow the existing process
Tokio executor using init_with_runtime. Async cancellation drops a waiter and
requests native cancellation; it never releases execution ownership early. Blocking
waits detach Python and check signals. Shared terminal state preserves reports
after waiter cancellation; completion witnesses native destruction and join.

Results contain original variables, values, bounds, evaluated constraints,
equality residuals, side-specific violations, physical tolerance/quantity metadata,
applicable duals, authored objective/sense, typed native metrics/certificates and
source/provider/profile/build/transformation provenance. Null/unsupported differs
from zero and from an absent bound. Arrow buffers retain allocation ownership
through the final C-stream/array reader.

Explicit publication consumes immutable results and reuses existing native member
writes, control-last commit, exact versions, conditional parent and settlement.
Failed attempts may publish audit results. A marker alone does not deduplicate a
retry; unresolved effects require native log inspection. Reopen exact versions.
Persist declarations needed to rebuild, never CAS display text, Salsa IDs, evaluator
handles or solver factors. Refuse incompatible old compiled artifacts.

## Targeted verification

Baseline is zero failures, with explicit pse-relations/force-validate. Extend
`just unit-native-contracts`; add a focused public-contract recipe. Compile through
`just check-solver-contracts` and affected-package recipes. Generate boundary
contracts through their generators. Refresh the stale Python environment/extension
before Python unit controls. Preserve the installed native build profile explicitly.

Controls: mixed affine/nonlinear rows, near-zero coefficients, affine constants,
narrow intervals, tape-only edits, opaque operations, all-eliminated models,
scaled/maximization duals, transferred bounds, original KKT/feasibility, stale starts,
failed final evaluation, builder/document equivalence, indexed/ragged identities,
atomic edits, cancel/wait races, retained buffers, publication settlement and
incompatible-artifact refusal. No test substitutes raw g(x) for a residual.

Full native convergence, installed compile/solve/publish/reopen, storage fault
journeys, resource stress, numerical comparisons and performance remain M22.
Do not report targeted packet closure as those gates passing.

## Verification and completion boundary

**Implemented:** the packages above close at the targeted-contract boundary.
`pse-math::presolve` supplies immutable Symbolica proofs and genuine native tapes;
the compiler tracks them through Salsa. `pse-backend-native::presolve` and `tnlp`
share library transformations across direct Ipopt and POUNCE. The generated
`authored.computation_models` declaration feeds `pse-runtime::workflow`, with thin
PyO3 handles and Python builders. Generated result rows use the existing native
Arrow ownership and exact Delta publication contracts.

**Tested:** `just unit-native-contracts` runs 105 selected Rust units with zero
failures and 58 tests excluded by the selection. The profile is Rust 1.98.1,
full native solver features, pinned native runtime and explicit
`pse-relations/force-validate`, with the locally provisioned Symbolica license.
The baseline is zero failures. This includes seven native presolve controls,
the Symbolica alias/assumption projection control, and both public workflow units.
`just unit-public-contracts` independently selected the two workflow units and
passed both under the same native/force-validation profile.

**Tested:** `just py-native-contracts` passes 25 Python unit tests, zero failures,
using CPython 3.14.7 and the editable native-solver/force-validation extension,
with the pinned Ipopt/MUMPS runtime path. The baseline is zero failures. This
includes cancelled-waiter rejoin, final Arrow reader ownership and refusal of a
package with no native declarations. `just unit-package pse-model
'test(artifact::durability_unit::)'` passes the existing descriptor compatibility
unit, zero failures and 32 excluded, with explicit force-validation; changed
validity and old/forged formats are rejected. No publication write is exercised.

**Interface-checked:** `just check-native-python` passes the full native Python
profile and `just check-package pse-runtime` passes default all-target compilation.
`just codegen-contracts` generated the declaration/result Rust, Python and docs
from the registry. `just py-sync-native` installs the native editable extension
and generates stubs from its actual compiled API. The transitive
`proc-macro-error2` future-compatibility notice is still present.
`just python-contracts-check` and `just family-check` pass. Focused Ruff checks of
the edited Python boundary/tests and Pyrefly checks of `_workflow.py`, `modeling.py`
and `test_native_workflow.py` report zero errors. These focused checks do not close
the whole-plan quality gate.

The contract controls cover affine constants and mixed nonlinear rows, tiny
coefficients and narrow intervals, expression identity, explicit native options,
required-pass refusal, maximization/scaling conventions, warm projection,
failed original evaluation, the library's all-eliminated stand-down, immutable
edits, document/builder equivalence, repeatable completion, original residuals,
pure publication preparation and final-buffer ownership. Python controls add
blocking/async access, cancellation and rejoin, typed profile refusal and public
Arrow readers. Actual native convergence and storage effects are not invoked by
these packet controls.

**Open qualification:** the exploratory `just lint-solver-contracts` run failed
in `pse-math`: it reported 79 library diagnostics and 107 library-test diagnostics
(the latter includes the shared diagnostics), against a zero-diagnostic target.
Task-local projection findings were subsequently corrected; those earlier counts
are observations, not a new baseline or a final clean check. Remaining full lint,
format, generation-equivalence, governance and formal ADR/blueprint reconciliation
belong to M22. Full compile/solve/publish/reopen, interrupted publication and native
settlement recovery, scientific references, native cancellation under long calls,
all feature combinations and performance/resource campaigns remain unexecuted.

## Outcome

**Implemented:** automatic presolve is qualified against native numerical limits;
explicit policies use the library option registry and required passes cannot
silently disappear. Library wrappers own elimination, derivative transport, warm
projection and final recovery. Fresh original-model observations preserve raw
native termination when validation fails. Physical feasibility, KKT observations,
primal availability and multiplier qualification remain distinct.

The public native workflow has one process budget/executor, source-backed physical
contexts, generated declarations, immutable model revisions, a shared preparation
boundary, cancellable jobs and immutable results. POUNCE's serde statistics and
HiGHS diagnostic products remain inspectable. Results retain original fixed and
parameter values, raw constraints, equality residuals, side violations, units,
source/provider/profile/build identities and transformation effects. Arrow readers
retain their final buffer leases. Publication is an explicit one-use command over
existing member writes and control-last settlement, with exact-version reopen.

**Mistakes corrected:** the first boundary draft exposed a fixture physical catalog
as a default; production now requires admitted source rows. Native presolve's option
registry does not register the solver's `tol`, so the adapter supplies the library's
actual certification default instead of accepting a spurious zero. The library
stands down when affine elimination would remove every column; tests and reports
now describe that behavior instead of claiming a synthetic solve. Python builds
now install the native editable profile and regenerate actual annotations after
custom scalar extraction; transitive MUMPS lookup is explicit in the recipe.

**Deliberate boundaries:** authored all-fixed cases evaluate directly; mathematically
fully eliminated native cases follow the pinned library's stand-down. Python's
frontdoor covers algebraic optimization/root/initialization intent, feasibility and
finite sequences. Rust `Runtime::native()` retains the existing explicit cone,
conditional initialization, graph/tear and declared-map services. Native provider
factories are bound in Rust; Python callbacks and a new provider-factory/configuration
language are not introduced. Python cone/flowsheet/provider constructors are not
claimed by this packet. Native temporary memory uses admitted allowances, not a
universal allocator/RSS guarantee. Published declarations are inputs for explicit
rebuild; the descriptor does not promise automatic provider/evaluator restoration.

**Next:** M17 native dynamics, M18 fitting/sensitivity, M19 cleanup, M20 fresh
acceptance inventory, M21 closure and M22 final qualification. The public workflow
and development profile are documented in [native workflow](../dev/native-workflow.md).


**Successor checkpoint:** M17–M18 is now implemented; its
[packet](14-m17-m18-execution.md) owns the new dynamic/fitting/provider construction
boundary and current evidence. The next implementation scope is M19–M21, followed
by M22 full qualification. Earlier boundary statements above describe the M16 checkpoint.
