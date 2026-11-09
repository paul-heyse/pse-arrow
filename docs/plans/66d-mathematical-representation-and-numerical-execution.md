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

This companion owns I03/I06 and S02–S04/S07 from the
[November 6 combined review](../plan_improvement_reviews/plan64-65-integrated-target-and-library-leverage-2026-11-06.md)
and its supporting analysis. The [November 9 Symbolica dispositions](66-integrated-simulation-target.md#symbolica-improvement-dispositions)
identify the new work by source. Blueprint §7/§14–§18 and
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

Use maintained Symbolica 3.0.1 with workspace defaults off and `integer-gmp`/`float-mpfr`
selected. Keep `pse-math/compiled-evaluator` an explicit native code-generation profile.
Reuse one guarded arithmetic region across [selected products](#symbolica-shared-products)
without widening the demand. The
[`export_instructions`](../../vendor/symbolica/src/evaluate/instruction.rs) API exposes
an evaluator's numerical program for selected downstream uses; it neither imports a
symbolic body nor admits a receiver. [Precision selection](#symbolica-precision) and
[exact coefficient work](#symbolica-exact-objectives) retain their own contracts. The
[Symbolica overview](../libraries/scientific_calcs/symbolica.md) supplies orientation;
the maintained sources linked at the owners define the consumed behavior.

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
or a specifically approved revised contract replaces them. The existing provider-free
Value/First handoff restores a prepared stage's interpreted f64 evaluator, not a symbolic
body or native/JIT code. It cannot close canonical reconstruction.
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
The private Value/First root factory had no supported caller and was retired under
[ADR-0259](../adr/0259-retire-private-root-lane.md). Current NLP and response work prepare
Second and actions through [in-process assembly](../../crates/pse-runtime/src/math/products.rs)
and selected scientific-worker sessions. Higher-order live transfer requires a concrete
receiving consumer with an original product, seed and cotangent contract.

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
or the Value/First numerical product in place of its admitted symbolic body.

**D02 decision work (Symbolica review [I01](../plan_improvement_reviews/plan66-symbolica-leverage-2026-11-09.md#I01)).**
Qualify restoration of one producer-sealed, already-normalized guarded body first. The
maintained [`inspect_packing`](../../vendor/symbolica/src/atom/packing.rs) walks bytes with
caller-owned `PackingFrame` scratch and reports symbol occurrences and coefficient extents
without creating atoms, parsing native coefficients or registering symbols. It supports a
restricted packed profile with a declared receiving Float environment and rejects attributed
symbols and unsupported flags or coefficient forms. Layout inspection does not establish
canonical order, producer authority or destination compatibility. Do not turn a normalized
flag or equal digest into that proof. Use the actual registered pool and noncreating
[`Symbol::get_symbol`](../../vendor/symbolica/src/atom.rs) to check names and roles; require
the original identities, attributes and order for a copy-only slice. Refuse incompatibility
before growth; do not enter general `Atom::import_with_map` remapping and normalization.

Use a supplier-owned checked restore patterned on `F64ProgramView::inspect`/`restore_with`:
inspect sizes, work and depth using admitted frames, then reserve stage backing, scratch and
coefficient materialization before allocation. Admission callbacks must not publish or register
while inspecting. Keep authored recipe and proof meanings with their existing owners. Bind the
restored stage and slot order, dependencies, provider bindings, guards and source tokens to the
original qualified request. Establish equivalent membership and full receipt consumption without
ordinary reconstruction or a second authority. Keep scope cancellation and custody with C04;
budget new partials and proofs separately from restoration. A later profile that needs
registration or renormalization needs its own pre-allocation bound.

Start with `canonical_portable_body_large_scientific_recipe_persists_and_reconnects` at the
[canonical receiver](../../crates/pse-runtime/src/math/portable.rs) after producer retirement,
then a guarded body that actually serves a symbolic partial or class consumer. Check exact
coefficients and correspondence plus an independent original result. Include one-unit-short
allowance, truncated nested data, reordered or foreign symbols, large exact coefficients,
unsupported flags, missing/extra/changed receipts and a crossed domain guard. Refuse before
allocation or registration where the failed premise is knowable and return no partial body.
The [supplier packing controls](../../vendor/symbolica/tests/packing.rs) complement these
owner-level tests; they do not replace them.

If the sealed-body route cannot preserve proof custody or costs more across the supported
operation, compare bounded typed replay with supplier admission before normalization, coefficient
and partial growth, and any pool expansion. This is a design alternative, not a runtime
fallback. If neither route qualifies, keep binary activation closed and JSON v3 current.
If one qualifies, C04 owns a single cutover and B03 owns any changed projection; no dual reader.

The bounded-copy experiment uses the existing supplier packing inspection and raw copy
operations on the exact fixed stages of one producer-qualified body. Its fresh receiver checks
already registered formal identities and the requested packed backing before copying. This is
not canonical receipt admission: the compiler still derives source-to-stage correspondence
and ordered physical and piecewise proof requests together in `Request::admit_inner`.
`RecordAuthority` establishes membership, not that correspondence; the compiler's physical-only
pass suppresses proof requests. Bounded typed replay is also unqualified: arithmetic normalizes
before the occurrence tick, shared partials differentiate before their count check, and resolving
a new slot can allocate/register an entire formal chunk. Keep JSON v3 current.

The [selected compiler control](../../crates/pse-compiler/src/typed_math_portable.rs) now
drives the same typed traversal in an atom-free mode. It matches the original guarded producer's
ordered stages, slots, effects and physical/math receipt requests, including an exact integral
power. Missing, extra, changed or reordered receipts and cancellation refuse. Shapes whose
schedule depends on normalization, provider equality or authored partials remain unavailable
in this profile. This proves correspondence for the selected producer only: expression/dependency
custody, bounded source framing, and the large cold canonical symbolic consumer remain open.
There is no new production receiver or supplier feature.

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
| D01 | in-progress | Actual remaining G/H/I/J blocks; current source-owned evaluator/accuracy contracts | Selected affine construction, allocation accounting, request-local objective reuse and strict-domain export are integrated. H03 separates row floors, numerical requirements and the scheme/domain conflict. Remaining family demands and assembled evidence stay with their owners. |
| D02 | in-progress | D01 selected transforms; A semantic dependencies and C preparation | Selected affine, shared derivative and adapter demand work is implemented. The active recipe checks cancellation before decoding and symbolic construction. The bounded-copy test does not qualify canonical receipt admission; C04 keeps JSON v3 as the only production reader. |
| D03 | complete | Current direct factor and SVD routes and their selected consumers | Retain factors and SVD for the traced workloads. Reopen for a named large repeated-response workload with an admitted operator and preconditioner candidate. |
| D04 | in-progress | Changed maintained arithmetic/provider consumers; confirmed strict-domain rule through the decision route for that slice | Direct and joined modeling results carry the opt-in invocation observation in `runtime.provider_selections`. BT passed for liquid, two-phase and vapor with original ports and member identity; the canonical two-phase read retained the original attempt. Other provider and solver populations remain open, including strict-domain proof-path and affected demand/precision controls. |
| D05 | in-progress | Selected changed D02/D04 deliveries; actual family consumers | Selected polynomial actions use staged products and no longer flatten the shared body. Retain bounded symbolic cancellation and the actual entry products. Complete remaining caller-specific retirements and hand off the enduring contracts and independent evidence to L. Candidate composition is not required. |
| D06 | complete | Verified published 3.0.1 archive and current tracked supplier tree | The refreshed delta and receipt reproduce all 159 tracked paths, bytes and executable modes, including the build-provenance fix. No dependency change. |

The canonical two-phase BT control reached the original protected read and found
that persisted visit rows omitted the actual attempt ID. The writer projected before
receiving its durable receipt. The run now retains the issued attempt at completion
and uses that same identity for both early publication and later local projection;
an ephemeral run still has no durable attempt. **Tested, selected:** the corrected
canonical two-phase control passed in the default-profile, force-validated runtime
invocation `01a1bf81-177a-7550-9526-7aeffd6cfb4b`. The protected reader retained the
exact issued attempt and all original visit rows after producer retirement. Its
regime observation remains distinct from scientific permission.

D03 is a decision obligation, not a mandatory second backend. Likewise expression streaming,
new tensor machinery or another property/DAE library requires a demonstrated remaining consumer;
record the concrete gap before adopting it. Existing native mathematical meaning stays authoritative.

### D01/D02 — Shared admission and exact native demand

Compose preparation once from immutable formulation facts, explicit fresh or retained
witness ownership and the admitted resource context. Reuse the existing
`routing::Requirements::decision` and `execution::Capability` under the pinned native
solver profiles. The router selects the eligible class adapter. The adapter's
`callback_demand` supplies exact preparation and evaluation requirements, including the
first missing product, from the same capability decision. Carry that decision and the reconciled
witness reservation to [C02/C04](66c-preparation-execution-and-service-lifecycles.md#c02--complete-actual-local-operationservice-consumers)
instead of independently recomputing them in conditional artifact preparation,
initialization, fitting and shooting. Reuse `OracleContract` for shared structural facts
and ordered identities. Bind reuse to the same selected adapter, route, profile, controls,
support and witness; changed provider or initialization settings can require new demand.
Formulation, derivative source and resource context stay with their native owners.
[Native router](../../crates/pse-backend-native/src/routing.rs) ·
[Adapter contract](../../crates/pse-backend-native/src/execution.rs) ·
[Oracle contract](../../crates/pse-backend-native/src/lib.rs)

Adapter assessment now selects and retains the callback demand for the actual candidate.
Serial and ready conditional initialization use it to prepare the selected callback product,
including a distinct KINSOL Jacobian action and assembled preconditioner where required.
A derived path with a newly frozen layout reassesses that layout; the original source
eligibility check and the derived decision are for different premises.

Preserve KINSOL directional Jacobian action versus an assembled preconditioner, constrained
optimization curvature and bounds, residual fitting eligibility, and dynamic event and
partial trajectory semantics. Reuse preparation only across unchanged consumed premises.
Keep backend termination, independent original-coordinate screening and scientific-use
permission distinct. Exercise fresh and retained witnesses, changed value and structural
premises, and unsupported artifact or derivative demand before dispatch. Reuse
`preparation_and_arclength_suitability_are_adapter_facts`; distinguish JVP-only SPGMR
from preconditioned/assembled work, exact Hessian from limited-memory or gradient-only
work, and equal-width reordered layouts from a First-only provider asked for Second.
[KINSOL demand selection](../../crates/pse-backend-native/src/execution/kinsol.rs)

D04 receives the revised B03/C03 worker crossing where isolated placement is selected,
with independently specified output order and derivative expectations. Confirm supported
demand against the native provider and refuse an unsupported action without substituting
another derivative product. Keep that evidence separate from wire structure or readiness.

<a id="symbolica-shared-products"></a>
### D01/D02 — Preserve shared definitions in demanded derivatives

Compare the current direct [`flat_expressions`](../../crates/pse-math/src/products.rs)
substitutions and weighted differentiation against existing staged forward/reverse products
for one real provider-free weighted action. Prefer the staged route when it retains the
original product and avoids expansion. Weights, directions and cotangents remain ordered
runtime inputs. A directional product must not materialize a dense Hessian; actual entry
consumers retain the original support and ordering. For an actual symbolic partial, compare
scoped definitions and Symbolica differentiation against the current
[shared-partial path](../../crates/pse-math/src/typed_partial.rs). Keep native provider
partials and branch-local regularity at their existing guarded owners.

The maintained [evaluator builder](../../crates/pse-math/src/library.rs) already uses
`FunctionMap::add_aliases`, `Atom::evaluator_multiple` and `direct_translation(true)` to carry
shared definitions to one instruction DAG. Preserve that sharing before compilation rather
than expanding and finding it again. [`AliasedAtom`](../../vendor/symbolica/src/atom/alias.rs)
is a candidate for compact pure definitions; opaque aliases do not establish their derivatives.
Joint multi-output preparation is the default when demand is known together. Consider
[`ExpressionEvaluator::merge`](../../vendor/symbolica/src/evaluate/optimize.rs) only for
already-built exact products that would otherwise be rebuilt: it checks parameter count,
not ordered identity, and its `Eq + Hash` contract excludes primitive `f64`. The application
owns layout, coefficients, output order and guard/effect membership.
[`Dualizer` and `Vectorize`](../../vendor/symbolica/src/evaluate/dual.rs) are candidates for
selected Taylor axes/lanes; retained sub-evaluators can make vectorization unavailable.
[`optimize_stack`](../../vendor/symbolica/src/evaluate/optimize.rs) and optimizer limits manage
optimization and temporary work, not the pre-allocation envelope.

Use repeated intermediate chains/diamonds, reordered outputs and independent analytic `Jv`,
`Jᵀw` or `Hv` alongside the [guarded square-root controls](../../crates/pse-math/src/product_guard_tests.rs)
and one actual repeated fit or solver callback. Retain unsupported-order and provider/guard
refusals. D05 deletes flattening only for migrated products with a proven replacement;
do not add a second DAG/cache, write application differentiation rules or fall back after a
selected arithmetic route refuses construction.

Selected provider-free polynomial actions use the staged product over the original
ordered axes. The guarded symbolic and requested entry paths retain the bounded direct
calculation: a valid square-root weighted product can have a finite final answer even
though a staged intermediate second derivative would overflow. Shared symbolic partials
use Symbolica `replace_map` once to lift earlier bindings into unary calls, Symbolica's
derivative, and one `replace_map` to rebind results and partials to the original slots.
The implementation preserves joint output publication within each block.

<a id="symbolica-exact-objectives"></a>
### D01/D02 — Carry exact objective analysis into coefficient extraction

Reuse one bounded, request-local exact objective between
[`shape_facts` / `aggregate_objectives`](../../crates/pse-math/src/class_evidence.rs) and
[`coefficient_projection`](../../crates/pse-math/src/coefficients.rs). Class-only demand keeps
the needed degree evidence without expanding a polynomial for classification. For an actual
LP/QP coefficient consumer, use Symbolica [`to_polynomial_in_vars`](../../vendor/symbolica/src/atom/core.rs)
with the original selected variables, exact domain and coefficient lookup. Compare a single
coefficient-domain extraction supplying both products where the request needs both; carry
degree facts, allocations and the actual fixed-value premises through one work owner.
Keep existing selected-affine-row reuse, independent product admission and the
[factorable projection](../../crates/pse-math/src/factorable.rs) with its own fidelity.
Do not retain arbitrary objective atoms in durable facts or add a parallel cache.

A joint class and coefficient request now hands the exact aggregate objective to the
coefficient stage in the same work allowance. Class-only demand still uses degree evidence;
a separate request for coefficients may aggregate the objective under its own allowance
while reusing the proved affine rows.

Selective exact elimination remains a candidate for a named small block. Compare
[`Atom::system_to_matrix`](../../vendor/symbolica/src/atom/core.rs) and
[`SolveBuilder`/`SolutionSet`](../../vendor/symbolica/src/solve/solution_set.rs)
with the retained native formulation, preserving pivot/nonzero conditions, free coordinates,
solution coverage and recovered original variables. Test `a·x=b` at `a=b=0` and a cancelled
rational factor at its excluded input. Numerical `zero_test` does not prove exact identity;
even exact nonidentity does not discharge a pointwise nonzero guard. Do not expand exact
algebra globally or replace the bounded cubic route with generic enumeration.

Start from `coefficient_objective_projection_reuses_proved_rows_under_one_work_ledger` and
the [selected-row controls](../../crates/pse-math/src/assembly_tests.rs). Compare independent
coefficients and original residuals for changed fixed inputs, weights, ordered columns/pairs,
and singular or out-of-domain cases. D05 removes repeat aggregation only after the actual
class and coefficient consumers use the qualified request-local result.

<a id="symbolica-precision"></a>
### D02/D04 — Apply precision only to the demanded claim

Use [`map_coeff_with_prec`](../../vendor/symbolica/src/evaluate/evaluator.rs) only for an
admitted product that needs a different numerical domain. In maintained 3.0.1 the caller maps
literal stack values; the binary precision applies to registered constants, whose evaluation
is unwrapped by the supplier. Require an eligible domain and constants; this API neither
corrects Numerica literal rounding nor certifies enclosure or provider behavior. Keep routine
binary64 and the existing [bounded cubic/directed MPFR proof](../../vendor/symbolica-bounded-roots.md).
Separate exact identity, rounding, conditioning/backward error, validated enclosure and physical
agreement. Inherited ball transcendentals do not establish a general enclosure contract.

Before changing a realization, choose a real G–K consumer and a revealing original-coordinate
check, a precision-sensitive literal and the exact guard control
`exact_nonzero_comparison_preserves_rationals_beyond_binary64_extent`. Consolidate duplicated
rounding/escalation policy at the math owner only if actual consumers share it. Do not reopen
D03 for this work: direct factors and weighted-response SVD remain selected, and an adjoint
objective gradient does not supply response Jacobian or covariance evidence.

No traced A–F consumer needs a new evaluation domain. Exact rational guard identity,
directed MPFR cubic proof, and routine binary64 evaluator results keep their distinct
contracts. The next precision change requires a named consumer with a discriminating
original-coordinate check.

<a id="strict-domain-fidelity"></a>
### D01/D04 — Preserve strict-domain fidelity at native export

The [SCIP export owner](../../crates/pse-backend-native/src/execution/factorable.rs) implements
[ADR-0258](../adr/0258-preserve-strict-domain-export-fidelity.md). The former inward-margin
rule is deleted. Strict ranges now keep the original endpoint as `Relaxed`; omitted
conditional guards and point exclusions also relax fidelity. Exact-only demand refuses a
relaxed graph. Before native admission,
[POUNCE's outward-rounded `fbbt::Interval`](https://github.com/paul-heyse/pounce/blob/141a9d00e1e1d3eb3aa716f7640390ecdfcbdfbe/crates/pounce-presolve/src/fbbt/interval.rs)
checks reachable log and partial power operands on the selected closed box. The check uses
the same minimum distance that SCIP
10.0.2's [log](https://github.com/scipopt/scip/blob/v10.0.2/src/scip/expr_log.c)
and [power](https://github.com/scipopt/scip/blob/v10.0.2/src/scip/expr_pow.c)
handlers enforce; the adapter reserves and reads back both settings. This check applies
even if a mathematically positive box already discharged the original guard. A box the
pass cannot establish as safe refuses. The original point checks still own eligibility.

Replace the exact-margin assertion with a projection/quality control for a valid point below
the margin and `min x` on `0 < x ≤ 1` (infimum zero, unattained), and exercise refusal at an
unsafe closure endpoint. Separately investigate whether a branch-local guard is lost when
equal arithmetic arms fold (`if x ≥ 0 then x/x else 1` still excludes zero). The authored
control now verifies the reached guard and original zero refusal. The current projection of
that guarded division is unavailable and exact-only admission refuses it. This is separate
from the tested relaxed closure of a representable strict endpoint. Require the selected native
proof-path control before handing J04 and L an accepted export. This work does not depend on I01.

**Tested, selected.** `scripts/pse test pse-compiler pse-math pse-backend-native --lib`
with the focused guarded-receipt, strict-export, shared-partial and ethane filters passed nine
controls in default mode, with force validation in the Arrow closure, zero failures/warnings
and complete settlement (`01a1c12d-b299-73f5-82d6-b881a7f8a58f`). This is not a native SCIP
proof-path or binary receiving qualification.

<a id="symbolica-composition-candidates"></a>
### D02/D05 — Candidates for complete mathematical operations

A named, independently reusable provider-free chain triggers a comparison of current joint
preparation, separate stages and released Symbolica 3.0.2 (`afbf7aa4…`, Numerica `^3.0.2`)
[`EvaluatorComposer`](https://github.com/symbolica-dev/symbolica/blob/v3.0.2/src/evaluate/compose.rs).
It binds earlier results/constants to later inputs, prunes unused work and reoptimizes without
symbolic Horner search; `finish` requires `Eq + Hash` and reoptimization may change binary64
realization. Compare actual input/order/invalidation and original output evidence, with no
elision of a fallible provider. [`from_instructions`](https://github.com/symbolica-dev/symbolica/blob/v3.0.2/src/evaluate/instruction/import.rs)
checks structured numerical control flow but allocates validation and recursive state and
builds a fresh layout. [Atom Serde](https://github.com/symbolica-dev/symbolica/blob/v3.0.2/tests/serde_atoms.rs)
allocates and remaps global state. [Native equality tests](https://github.com/symbolica-dev/symbolica/blob/v3.0.2/src/atom/zero_test.rs)
can run exact fallback beyond sample limits and do not prove pointwise nonzero. None supplies
bounded symbolic restore. A release change would require equivalent maintained packed
inspection, f64 transfer, bounded real roots and build provenance, or a scoped rebase with
the actual selected feature closure and consumer checks. No pin change is selected.

Keep the guide's other candidates tied to actual gaps. A large additive workload with measured
expression growth can justify comparing [`TermStreamer`](../../vendor/symbolica/src/streaming.rs)
with coefficient-domain or in-memory work; `max_mem_bytes` sets a spill threshold, not a
per-term or global-merge bound. A named exact block can justify specialized polynomial/modular
work; a proved structural symmetry or physical tensor contract can justify an ecosystem
comparison. A real long-lived or vector workload can justify JIT, SIMD or native codegen
against the prepared interpreter under the same precision and platform contract. No candidate
adds a family-wide requirement, an alternate production path or a qualification campaign.

<a id="symbolica-supplier-provenance"></a>
### D06 — Refresh the maintained Symbolica source evidence

The complete delta and source receipt now include the `build.rs` change in `f903f0b05`.
The [supplier archive procedure](../../vendor/deltas/readme.md) verified the published 3.0.1
archive (`e9a0d354…`), derived the complete binary delta and independently reapplied it to
a fresh extraction. All 159 tracked paths, bytes and executable modes matched. Commit
`8401078ea` publishes the delta and receipt together with no change to the pin. This is
source provenance evidence, independent of the 3.0.2 composition candidate.

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
