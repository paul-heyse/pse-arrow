---
title: "Plan66d: Mathematical representation and numerical execution"
status: in-progress
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/66-integrated-simulation-target.md
doc_topics: [design, numerical-execution, performance, validation]
---

# Mathematical representation and numerical execution

## Remaining target and existing strengths

This companion actions review I03/I06 and S02–S04/S07 under the
[Plan66 coordinator](66-integrated-simulation-target.md). Blueprint §7/§14–§18 and
PS-05–PS-10 supply mathematical context. Main `66341514d` already contains sparse
symbolic/numeric separation, implicit response actions, shared multi-output evaluators,
selective accuracy arithmetic, SVD fitting covariance and class-specific native runners.
Do not recreate those capabilities. Remaining work selects stronger representations at
actual demanded blocks, completes changed consumers and verifies the current maintained
arithmetic closure beyond its focused tests.

## Selected design and critical library functionality

Preserve original coordinates, exact constants, source correspondence, domain guards and
scientific acceptance through symbolic simplification and numerical execution. Start from
the requested product and mathematical class; choose symbolic/coefficient/factorable/sparse
or action-based realization before introducing caches or pools. A bounded/constrained solve
cannot become plain Newton iteration merely because the latter is easy to call.

Maintained Symbolica3.0.1
[`export_instructions`](../../vendor/symbolica/src/evaluate/instruction.rs) permits
library-owned evaluation programs to cross an admitted mathematical boundary;
[`map_coeff_with_prec`](../../vendor/symbolica/src/evaluate/evaluator.rs) changes coefficient
representation with explicit precision. Use these with existing multi-output preparation
to avoid repeated expression conversion and to retain high-precision constants where demanded.
Compare selective affine/rational elimination against retained numerical blocks on actual
consumers. Carry pivot/nonzero assumptions and recovered original variables; do not expand
exact algebra globally or discard guards. The
[Symbolica overview](../libraries/scientific_calcs/symbolica.md) supplies broader leads;
maintained source defines the consumed contract.

Keep precision/error claims explicit: repeated high precision, backward error, conditioning,
validated enclosure and independent physical agreement answer different questions. Basic ball
arithmetic is not a general transcendental enclosure, and series-to-expression conversion
does not retain a remainder automatically. Review pending accuracy opportunities by demanded
claim; do not prescribe one arithmetic mechanism to every solver. Physical budgets or comparison
allowances may change with dimensional/scientific rationale and revealing countercontrols.

For a selected large response, investigate maintained faer0.24.4
[`LinOp`/`BiLinOp`](../../vendor/faer/src/operator/mod.rs) and
[`lsmr`/`lsmr_scratch`](../../vendor/faer/src/operator/lsmr.rs) under the existing native
response-action owner. These provide multi-RHS actions and explicit scratch/parallelism.
Select an action-based route only after comparing the actual consumer with retained factors:
repeated RHS often favor factors; a large sparse problem may favor preconditioned actions.
The [maintained arithmetic contract](../../vendor/faer-arithmetic.md) qualifies an exercised
binary64 closure, not every generic scalar or iterative method. Preserve the current
[direct weighted-response SVD](../../crates/pse-runtime/src/workflow/fitting/covariance.rs)
and its rank/withholding rules; do not form normal equations to simplify a large-response path.

At ndarray0.16.1 use `ArrayView`, `ArrayViewMut`, `Zip::and_broadcast`/`for_each`; at faer use
`MatRef`/`MatMut` and explicit scratch. These admit borrowed/strided data and fused traversal.
See [S04](../plan_improvement_reviews/plan64-65-scientific-extensibility-analysis-2026-11-06.md#S04)
and [ndarray reference](../libraries/scientific_calcs/ndarray.md). Preserve species/phase,
units/datum and ordered coordinate correspondence at the semantic adapter. Compare packing
with strided views on a real batch; zero-copy alone is not a throughput result. Charge retained
coefficients, factors and scratch to their actual native owner.

Native solver selection remains by full problem class and supported derivatives/domain:
KINSOL square systems, Ipopt/POUNCE smooth NLP, HiGHS linear/mixed-linear, Clarabel admitted
cones, SCIP factorable mixed nonlinear, and Diffsol/IDAS admitted dynamics. Use the selected
[native solver reference](../../.codex/skills/native-solver-libraries/SKILL.md) and pinned
source contracts for the route actually changed. No new fallback chain or solver family is
selected by this plan. Higher-index/hybrid derivative extensions need a concrete H/I/J consumer,
exact consistent-initialization/event contract and its applicable decision before activation.

### Canonical reconstruction activation

C04's canonical binary Recipe receiving requires a justified pre-allocation envelope for
Symbolica normalization, coefficient and derivative/partial growth, and global symbol
registration. A bounded decoder, post-construction node/slot limit, payload multiplier or
owned child process does not establish that bound. D02 supplies the mathematical construction
contract and C04 the receiving admission/receipt/custody contract. Retain current JSON v3
receiving until bounded reconstruction or a qualified alternative satisfies those obligations,
or a specifically approved revised contract replaces them. The already implemented provider-free
compiled f64 Value/First live-child handoff is distinct and cannot close canonical reconstruction.
This is an operation admission contract, not a new host-wide memory cap.

Source investigation of maintained Symbolica3.0.1 confirms a useful narrower mechanism:
`ExpressionEvaluator<f64>::f64_transfer`, allocation-free `F64ProgramView::inspect`, and
`F64ProgramView::restore_with(admit)` inspect original instruction/constant/result/stack extents
and admit requested backing before validation scratch or materialization. They restore the
supplier's interpreter without atom import or global symbol registration; the stated envelope
excludes allocator metadata/RSS, borrowed input and outer owners. The supplier format has no
derivative-order concept: already prepared derivative programs could use it, but the current
math wrapper refuses Second, directional/direct/reverse products and providers. D02 must qualify
the exact native product/layout/stage/worker/provider closure before extending that support;
removing a rejection alone is insufficient. Sources: [maintained supplier contract](../../vendor/symbolica/src/evaluate/f64_transfer.rs)
and [native math transfer owner](../../crates/pse-math/src/execution/transfer.rs).
The traced first-party live-root callers are acceptance tests of the closed Value/First factory;
current NLP/response work prepares its Second/actions through in-process assembly and original
scientific-worker sessions. Higher-order live transfer therefore stays a capability candidate,
not an extra implementation added to claim reconstruction progress. Require a concrete receiving
consumer and its original product/seed/cotangent contract first; see
[current live-root factory](../../crates/pse-runtime/src/math/native_child.rs) and
[current demanded-product preparation](../../crates/pse-runtime/src/math/products.rs).

That numerical product cannot replace canonical reconstruction's `AdmittedBody`, whose actual
consumers require subsequent partial compilation, symbolic affine/quadratic evidence and exact
factorable work. Ordinary evaluator Serde/bincode decoding allocates vectors/external state;
`Atom::import`, workspace recycling, evaluator cache limits, `map_coeff_with_prec`, instruction
export and native library loading do not establish the missing general construction envelope.
Selected GMP/MPFR growth and process-global registration remain part of that obligation. A
qualified numerical receiving slice must preserve the original binary64 realization and withhold
unavailable symbolic/higher-precision products. Any broader supplier extension needs source-backed
preconstruction admission and registration custody, following the scope discipline in the existing
[bounded-root maintenance contract](../../vendor/symbolica-bounded-roots.md).

The binary recipe receiver is exercised only in tests and invokes the same
`reconstruct_recipe` as JSON v3. The native [typed admission](../../crates/pse-compiler/src/typed_math/direct.rs)
uses Symbolica normalization during body construction and shared partials. The
[formal pool](../../crates/pse-math/src/library.rs) can register a full symbol chunk
while resolving a slot. Restricting a recipe to already registered slots would
leave coefficient, normalization and derivative construction without enforceable
preconstruction admission. No current canonical consumer can use an atom-only restriction
or the compiled Value/First product in place of its admitted symbolic body. Keep
canonical binary activation open and retain JSON v3.

### D03 response selection for current consumers

**Interface-checked.** Keep retained factors for the existing root predictor and
the direct weighted-response SVD for fitting. The four-cell 31/61-point H03
controls exercise refinement and accuracy without requesting a response. The
known-area and confounded exchanger fits exercise rank and withholding over one or
two fit columns. The existing sparse predictor is a small authored root control.
None supplies a large repeated-right-hand-side workload for an LSMR comparison.
Revisit when a named large sparse response has an admitted original-coordinate
forward and transpose operator, a preconditioner candidate and a useful repeated
solve workload. [Root predictor control](../../crates/pse-runtime/src/workflow/modeling/root_response_tests.rs) ·
[Exchanger fitting controls](../../crates/pse-runtime/src/workflow/fitting/scientific_equipment_tests.rs)

## First demanded mathematical correction — November6

H03's single aqueous cell reached a 610-coordinate simultaneous formulation and passed its
2310 derivative comparisons, then refused affine preparation before solving. This is not a
physical acceptance result. At that failure, `pse-compiler`'s `ExecutableCase::class_allocation_bound`
reused the full value-rebind/factorable bound. In `pse-math::factorable`, that bound includes
dense quadratic curvature storage proportional to `(columns + 1)^2 * (nodes + 1)`.
`ExecutableCase::prepare_affine_rows` instead calls the existing selected-row
`CasePlan::affine_row_facts`: it borrows original contribution indexes, resolves only actual
formal candidates and uses Symbolica first partials to establish original affine rows. It
does not build that dense quadratic memo. Do not interpret the rebind estimate as measured
affine memory demand or widen the worker solely to clear it.

D01/D02's first correction supplies a separate affine-row construction estimate: selected
original contribution/slot inventories, source-symbol discovery, coefficient maps, base/fresh
facts and retained expression allowance. Full coefficient-class/rebind demands remain separate.
The runtime now consumes this operation-specific estimate without changing proof, cancellation,
stale-assumption or native job ownership. Prepared/rebound publication also counts owned
`ProblemFacts` vector payload. The existing foreign allocation allowance remains distinct:
normalization/differentiation still allocates inside Symbolica before its post-result charge,
so this known Rust/retained-result estimate does not qualify transient symbolic preallocation
or C04's canonical reconstruction gate. No opaque fallback or worker increase was introduced.
Sources: [compiler class/affine owners](../../crates/pse-compiler/src/workspace.rs),
[current rebind bound](../../crates/pse-math/src/factorable.rs), and
[selected-row Symbolica proof](../../crates/pse-math/src/class_evidence.rs).
H independently assesses the physical temporal layout and the retained compiler frontier;
its simultaneous refinement cases remain required even if the separate integrated route passes.

The resulting four-cell/61-point frontier census identified repeated charges for identical live
allocation owners in planning and projection. Main now counts shared model, resolved arena,
symbol-table and physical-premise snapshot allocations once by their actual live owners.
Every distinct support copy's maps, captured reads, view roots and occurrence/slot/contribution
attribution stays charged; equal independently allocated snapshots stay separate. This is an
accounting correction, with no semantic interning, changed identity, producer sharing or retention
limit increase. Six targeted compiler controls passed in the owning checkout, zero failures/build
warnings with complete capture/settlement, including independent-equal versus shared-owner and
unchanged-limit refusal controls. See [frontier accounting](../../crates/pse-compiler/src/workspace/modeling/executable.rs)
and [resolved allocation components](../../crates/pse-modeling/src/resolved.rs). The recorded
2,415,722,790-byte raw census and its repeated snapshot charges describe known-owned accounting,
not measured resident memory or physical acceptance.

**Tested, targeted:** `scripts/pse test pse-compiler --lib -E
'test(affine_row_construction_uses_selected) | test(affine_row_rebind_keeps)'` passed two controls,
zero failures/pending/build warnings (invocation `01a1b283-3b51-7035-95d1-01c5b4529596`).
The 128-coordinate sparse case establishes original coefficients/constants without charging
the unrelated quadratic curvature population; retained product fits its estimate, original
row/domain/artifact correspondence survives, and work exhaustion/cancellation still refuse.
The initial control failed because it requested path observations without declaring those
outputs; explicit source demand corrected the test. Runtime all-target compilation subsequently
passed with `engineering-service,native-study-tests`, zero build warnings. The targeted
`latest_only_changed_rebinds_plateau_and_escaped_fields_keep_only_their_components` control
also passed with that closure, zero failures/pending/build warnings. Retained accounting now
counts both owned `ProblemFacts` vector copies: the current published value and its independent
payload anchor. Shared certificate allocation remains separately owned. These checks establish
the selected construction/retention contract, not H's physical receiving or symbolic transient
allocation bounds.

### Numerical demand exposed by temporal refinement

The first H03 discriminator compared the zero-initial four-cell physical problem at
31/61 temporal points. Ordinary physical allowances pass, but their engineering row floors
mask the expected temporal signal: the initial common-time maximum errors were approximately
0.187/0.177 mol/m³. Tightening scalar solver fractions alone left those observations unchanged.
This is not convergence evidence or proof that the native numerical request was tightened.

Use existing Analysis `NumericalPolicy.requirements`, resolved by
[`pse_math::numerics`](../../crates/pse-math/src/numerics.rs), for an actual target-keyed demand.
The selected owner is `PreparedCase.plan.structure().rows()`, not the whole admitted modeling
population. Preserve original equation/instance lineage, canonical unit, scope and frozen
conditioning; assert the resolved Analysis provenance and native physical tolerance inventory.
[`Normalization::with_row_budgets`](../../crates/pse-math/src/normalization.rs) transports these
demands to the native representation; original-coordinate
[`quality`](../../crates/pse-backend-native/src/quality.rs) independently checks the returned
candidate. A numerical request is distinct from the authored physical acceptance allowance.

The first explicit request, capped at `1e-10 * original nominal` per actual row, reached all
1240 selected coarse rows with matching units/provenance/budgets before execution and in the
retained result. That coarse solve returned an infeasible/unqualified diagnostic candidate;
the fine solve was not reached. Its focused control failed 1/1 with zero warnings and complete
settlement against the zero baseline. Native options and original violation lineage locate the
conflict in the original `Cell.conserve material` row, not a lost precision request. The exact
three-stage right-Radau first element requires negative intermediate concentrations for this
zero-initial step: approximately -0.228 mol/m³ at coarse spacing and -0.0199/-0.0139 mol/m³ at
fine spacing, conflicting with the original nonnegative concentration domain. An independently
authored Runge–Kutta tableau and faer 0.24.4 `Mat::from_fn`, `partial_piv_lu()` and
[`Solve::solve`](../../vendor/faer/src/linalg/solvers.rs) check the linear stage equations and
their residuals; they do not consume the production stencil. Production
[`collocation::element`](../../crates/pse-math/src/collocation.rs) continues to use Symbolica's
exact polynomial construction, certified real-root refinement and derivative/integration
weights. No alternative production discretizer or solver-option relaxation was introduced.

Retain that original strict case as an explicit infeasible/unqualified native diagnostic
control. A distinct, interior-initialized 10-to-100 mol/m³ cohort has independently positive
first-stage values and retains the same geometry, clock, original complete 31/61-point
populations and physical allowances. Both resolutions use the actual Analysis row demands;
the unchanged eightfold common-time error-improvement discriminator passes. **Tested,
targeted:** native `scripts/pse test pse-runtime --lib --features
solver-ipopt,solver-kinsol,solver-root-isolation,solver-diffsol` selecting the refinement and
strict-zero refusal controls passed 2/2, zero failures/build warnings, complete original capture
and settlement (`01a1b345-1b34-75e3-8159-484ec11dfd0f`). This establishes the selected
scheme/domain distinction and refinement comparison, not general positivity, convergence order,
other physical families or a completed D package. No physical bound, acceptance allowance or
native quality check was weakened. The source controls are
[`aqueous_inventory_tests`](../../crates/pse-runtime/src/workflow/modeling/aqueous_inventory_tests.rs).

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| D01 | in-progress | Actual remaining G/H/I/J blocks; current source-owned evaluator/accuracy contracts | Selected affine construction and actual shared-allocation accounting corrections are integrated. H03 now separates physical row floors, actual numerical requirements and a discrete scheme/domain conflict through an independently checked refusal plus passing interior-cohort refinement. Remaining family-specific mathematical demands stay open. |
| D02 | in-progress | D01 selected transforms; A semantic dependencies and C preparation | Selected affine producer/retention and H03 precision/domain receiving are implemented. The active recipe checks cancellation before decoding and before symbolic construction. The preallocation gate remains open for normalization, coefficients, partials and process-wide symbol registration. Continue the other demanded evaluator, batch and layout consumers. |
| D03 | complete | Current direct factor and SVD routes and their selected consumers | Retain factors and SVD for the traced workloads. Reopen for a named large repeated-response workload with an admitted operator and preconditioner candidate. |
| D04 | in-progress | Changed maintained arithmetic/provider consumers | Direct and joined modeling results carry the opt-in invocation observation in `runtime.provider_selections`. Run the actual BT liquid, two-phase and vapor control with original ordered provider ports and typed component membership. Complete the persisted independent read, other affected providers and solver populations. |
| D05 | planned | D02–D04 selected deliveries; family consumers | Retire replaced numeric paths after targeted controls and caller migration; supply complete-operation comparisons and enduring mathematical contracts to L. |

D03 is a decision obligation, not a mandatory second backend. Likewise expression streaming,
new tensor machinery or another property/DAE library requires a demonstrated remaining consumer;
record the concrete gap before adopting it. Existing native mathematical meaning stays authoritative.

## Verification and outcome

**Tested, partial.** The default-profile, force-validated native runtime selection
`01a1bdf2-e013-7126-9634-28bb9baf13d8` passed three of five library controls and all sixteen
coordinator controls with complete settlement. The BT liquid case accepted the original solve,
closure and physical expectations, then reported eight suspicious Jacobian differences in the
independent derivative comparison. This matches the existing liquid fixture discrepancy recorded
in [Plan 28e](28e-rebuild-retirement-and-qualification.md). The original derivative requirement
remains in the BT receiving control, so two-phase and vapor invocation readback remains open.
The selected implicit-regime owner now uses the current authored start when it lies inside
the retained certified uniqueness chart. It uses the previous root as a fallback for starts
outside the chart. This avoids replacing a valid current start with an older point that the
inner solver could accept within its residual tolerance. The original proof and residual gates
still apply. The maintainer stopped the rerun before it established derivative results.
The failed-block control retained the original mathematical cause and a stable repeated-call
allocation plateau, then failed its final pool balance. After joining native service owners
explicitly, the same default-profile control passed its original cause, allocation plateau and
zero-balance assertions with complete settlement (`01a1be0e-2bc5-744e-b07a-16b0110a5bcc`).

The new observation path seals the original winning provider invocation only when the
consumer requests the fact. Its native and kernel controls await verification. JSON v3
remains the current canonical recipe implementation pending the reconstruction gate.
Binary activation includes deletion of the replaced reader.

**Proposed verification:** changed coefficients with unchanged structure; changed sparsity,
phase regime and precision; ill-conditioned/rank-deficient cases; incompatible strided/ragged
layouts; rejected original domain; multiple RHS; cancellation and escaped scratch/outputs.
Compare original-coordinate residual/actions with a retained direct route and independent
physical/reference expectations. Structural equivalence and a library convergence flag do not
prove physical validity. Run targeted controls as implementation lands, and receiving native/
scientific campaigns through L at functional scope end. Performance benefits remain unmeasured.

**Outcome:** the first demanded affine construction correction is implemented with bounded
compiler and runtime controls above. H receiving and the remaining mathematical/physical population
stay open; no whole-plan, canonical reconstruction or physical acceptance is inferred.
