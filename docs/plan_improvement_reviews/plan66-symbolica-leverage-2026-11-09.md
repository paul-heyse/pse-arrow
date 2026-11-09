# Plan 66 improvement review — Symbolica and shared mathematical execution

**2026-11-09 · Inspected `b4aa4a8a8` on `main`, initially clean.** This review follows the
[Plan 66 coordinator](../plans/66-integrated-simulation-target.md), its November 9 checkpoint
and the A–L companions. It applies [core 3.4, efficient architecture heuristics 1.0,
process-simulator 1.5 and the repository binding](../design_review/design_principles/standard.toml),
including [test assessment 1.2](../design_review/design_principles/core/test-assessment.md).
The [prior scientific leverage review](plan64-65-scientific-extensibility-analysis-2026-11-06.md#S02)
already selected representation-first, demand-shaped mathematics. This assessment adds to that
work; the coordinator still owns its disposition. The recommendations below are **Proposed**;
observations of the inspected implementation and supplier interfaces are **Interface-checked**.

## Combined judgment

Give Symbolica the complete arithmetic operation *within a qualified mathematical region*:
normalize and transform expressions, preserve reusable definitions, differentiate the product
actually needed, and prepare its evaluator together. Use exact domains for bounded exact
questions and choose numerical representation at the actual consumer. The strongest opportunity
is keeping the same sharing through symbolic work and numerical preparation. It is not a new
universal expression representation or a replacement for the simulator's execution graph.

The current architecture already gets much of this right. The [guarded stage owner](../../crates/pse-math/src/guarded.rs)
groups arithmetic outputs and separately represents pre-normalization guards, lazy branches,
applicability and fallible multi-output providers. The [Symbolica builder](../../crates/pse-math/src/library.rs)
already uses `FunctionMap::add_aliases`, `Atom::evaluator_multiple` and `direct_translation(true)`
to compile a shared instruction DAG. D03's existing factors and weighted-response SVD have
qualified consumers; a change in symbolic representation does not create a new case for a
different numerical method. What remains is to stop losing sharing in earlier transformations,
stop rebuilding work that the producer can safely transfer, and make the same small set of
mathematical products available to real consumers without making each consumer a compiler.

The shared pipeline proposed here retains the existing authoritative boundaries: authored
relations and physical facts → one typed admission → guarded Symbolica regions → selected
symbolic evidence or numerical products → class-specific execution → original-coordinate
assessment and protected delivery. A narrow substitution is better where the general shape
would widen derivative or evidence demand. Retain current routes where the broader mechanism
has no qualified consumer or cannot preserve the original meaning. This follows AP-04/AP-07,
DP-01/08/10/13/15/16, PS-06–PS-12 and heuristics H2–H8/H12. It does not reopen formal reviews
or change accepted scope.

## D02/C04: reconstruct the admitted body, not just an executable program

**Interface-checked.** The [passive recipe](../../crates/pse-compiler/src/typed_math_portable.rs)
contains resolved syntax, exact support and provider joins, source and member tokens, physical
and mathematical receipts, and `BodySpec`. `reconstruct_recipe` restores those records and
calls `Request::admit_inner` inside both strict receipt scopes; replay consumes the existing
proof backing and checks each request, and fails if any receipt remains unconsumed. The
[test-only binary receiver](../../crates/pse-compiler/src/typed_math_portable_generated.rs)
indexes the bounded records and then calls that **same** reconstruction. In the
[canonical production receiver](../../crates/pse-runtime/src/math/portable.rs), a present
eligible record receives an outer byte/scratch lease and producer/request/interpretation
qualification before replay. A corrupt qualified hit fails; it is not a cache miss or permission
to run ordinary admission. [ADR-0164](../adr/0164-unify-simulation-substrate.md)
owns the trust boundary. The outer reservation and the `BodyLimits` occurrence/slot counts
do not reserve all internal symbolic work before it occurs.

| Alternative | What it supplies | Consequence for 66d/66c |
| --- | --- | --- |
| Bound existing typed recipe replay | Preserves the strict replay logic and needs no second stored mathematical product. Can share the new supplier limits with live construction. | Requires source-derived admission at the actual normalization, exact coefficient, shared partial and symbol-pool growth sites, including high-water overlap and interruption. A byte multiplier, a cap on outputs, or a child process does not establish this. Choose it if the stage-artifact route below cannot preserve proof custody or costs more across the supported operation. |
| Restore a producer-sealed, already-normalized guarded body | Reuses exact symbolic work and admits new backing by inspected extent. Keep the recipe and authoring as semantic authority; the stage product is rebuildable, supplier-specific derived state. | Preferred first investigation. The typed owner must prove equivalent validation of consumed receipts, guard/effect order and meaning without rerunning construction. The supplier must prove destination symbol/ordering compatibility and safe materialization before allocating. The wire encoding or a matching hash alone supplies none of these. |
| Transfer an already prepared numerical product | The maintained `F64ProgramView::inspect`/`restore_with` path checks supported instruction extents and admits backing before validation and materialization; it avoids all symbolic registration. | Keep the current provider-free Value/First scope. Evaluate an extension only for a named receiving product and its exact profile. It cannot return an `AdmittedBody` for symbolic partials, class evidence or factorable work. Do not run it as a recovery path for a failed symbolic admission. |

<a id="I01"></a>
### I01 — Qualify bounded restore of one real guarded body before binary activation

**Proposed amendment · D02 owns mathematical restore; C04 owns receiving and settlement;
B03 owns the mechanical projection if it changes.** Build on the existing maintained
[`inspect_packing`](../../vendor/symbolica/src/atom/packing.rs), with
[supplier controls](../../vendor/symbolica/tests/packing.rs), and the
[`F64ProgramView` transfer pattern](../../vendor/symbolica/src/evaluate/f64_transfer.rs).
`inspect_packing` already walks one packed atom with caller-owned frames, reports counts and
encoded coefficient extents, checks the declared finite Float environment, and provisionally
reports plain symbol occurrences. Its profile rejects unnormalized flags, attributed symbols,
special, field and polynomial coefficients. Its layout check does **not** establish reduced
coefficients, normalized order, origin, destination compatibility or authority to publish an
`Atom`. It is not currently a production caller. The public
[`AtomView::from` and `Atom::import_with_map`](../../vendor/symbolica/src/atom/representation.rs)
do not close that gap: the former presupposes valid bytes/current state; the latter resizes
from the serialized length before reading, then remaps and normalizes. Full
[`State::import`](https://raw.githubusercontent.com/symbolica-dev/symbolica/v3.0.1/src/state.rs)
can also mutate global state. Partial state export is packaging convenience, not isolated
registration custody.

Use a supplier-owned checked restore of an inspected packed profile under a typed stage owner.
First qualify producer, selected request and exact interpretation through the existing canonical
selection. Match each artifact stage/slot/output to the original admitted graph and receipt
context. Inspect bounded frames, total bytes, work, depth, coefficient extent and exact
formal/function membership; keep inspection callbacks free of registration and publication.
For a copy-only slice, require the destination's already-registered pool, symbol roles,
attributes and ordering to match; [`Symbol::get_symbol`](../../vendor/symbolica/src/atom.rs)
can look up an existing namespaced symbol. If identities/order differ, refuse that profile;
general remap with normalization would need its own bound. Never interpret a producer-set flag
as a proof of normalized order. Admit stage backing, supplier scratch and any coefficient
materialization before those operations; charge genuine pool growth separately to its owner if
a later profile needs it. Keep the original cancellation/deadline and lease until work settles.
Restoring staged expressions does not budget new derivative or proof work requested afterward.

**Decision evidence.** Start with a producer-retired cold receiver using the existing large
scientific recipe reconnect case, then the smallest guarded case with a downstream symbolic
partial or class-evidence consumer. Check exact stage and source correspondence, permitted
outputs, receiver-native evidence, an independently expected residual or derivative, and
failure before allocation/registration when the budget is one unit short. Include truncated
nested frames, reordered/foreign formal symbols or proof requests, missing/extra receipts,
large exact coefficients, unsupported flags, and a crossed domain boundary; assert no partial
body or changed symbol pool. Keep existing canonical and compiler negatives even when a new
supplier test covers packed bytes. If producer-sealed stages cannot establish the same proof
custody without preserving duplicate authority or excessive durable backing, qualify bounded
replay instead. A durable product change needs the existing interpretation/decision route.
The one selected route must move its real canonical producers and consumers before replacing
JSON v3 and its old-wire fixtures; do not add a dual reader or fallback. Until then, binary
activation stays closed. This is operation admission, not a new host resource policy.

<a id="I02"></a>
### I02 — Keep shared expressions through demanded derivative construction

**Proposed amendment · D01/D02/D05 with A02, H05 and J02/J06.** The bounded direct path in
[`products.rs`](../../crates/pse-math/src/products.rs) uses `flat_expressions` to substitute
stage outputs into later expressions, then builds a weighted expression and differentiates it.
Repeated intermediates may be expanded before `evaluator_multiple` can share them again.
[`typed_partial`](../../crates/pse-math/src/typed_partial.rs) already represents prior bindings
as functions of the active differentiation coordinate; it uses Symbolica derivatives and
rebinds the partials, but currently scans and substitutes bindings repeatedly. These are two
uses of the same sharing principle with different claims: authored symbolic partials remain
symbolic; an operator action need only apply the demanded derivative in its original axes.

First use the already-supported staged forward/reverse products as the comparison for a
provider-free weighted gradient/action. Preserve the existing shared region and pass runtime
weights and directional seeds as data. Where an exact symbolic product really is consumed,
compare explicit shared function definitions and Symbolica differentiation with the current
bounded expansion; carry definitions into the existing `FunctionMap` and direct evaluator.
[`AliasedAtom`](../../vendor/symbolica/src/atom/alias.rs) and `AtomCore::alias_subexpressions`
offer a source for compact pure definitions, but opaque aliases do not prove the required
derivative relationship, and scanning a fully expanded atom first cannot avoid that expansion.
Do not add another public DAG, parallel product cache, hand-written chain-rule engine or
one generic `Transformer` for physical rules. Extend a shared preparation owner only if an
actual consumer is simpler after migration.

**Decision evidence.** Compare a chain/diamond of repeatedly consumed intermediates, two
output orders and independent seeds/cotangents, and the existing
[guarded square-root products](../../crates/pse-math/src/product_guard_tests.rs). Check an
independent analytic `Jv`, `Jᵀw` or `Hv`, retained value and unsupported-order refusal. A
directional-only demand must not materialize a full Hessian; an entry consumer must still
receive the actual ordered support. Check one real repeated fit or solver callback before
deleting the flattening route for that product and the tests that only assert its mechanics.
Keep branches, provider cuts, source diagnostics and product-specific failure tests. No
different arithmetic path after a selected mechanism fails its construction admission.

<a id="I03"></a>
### I03 — Carry one exact objective analysis into its coefficient consumer

**Proposed amendment · D01/D02/D05; B's mechanical projections and J04/J06 only where
affected.** In [`class_evidence`](../../crates/pse-math/src/class_evidence.rs), `shape_facts`
aggregates objectives and differentiates selected columns/pairs to prove degree. On the
coefficient path, [`coefficient_projection`](../../crates/pse-math/src/coefficients.rs) calls
`aggregate_objectives` and expands/converts those expressions to polynomials. Compare carrying
the already aggregated exact objective and its degree facts inside the same bounded request
against a single coefficient-domain extraction that supplies both the needed facts and the
coefficient population. Select by actual demand: a class-only request should not expand a
polynomial just to classify it; an exact LP/QP consumer should not reconstruct the same objective.
Retain the existing selected-affine-row reuse, value binding and independent admission of each
product. Do not retain arbitrary objective atoms in durable class facts or make a second cache.

Use Symbolica's explicit variable and coefficient domains (`to_polynomial_in_vars`, coefficient
lookup and, where demanded, exact polynomial/rational operations). For a small isolated algebraic
block, selective elimination and [`SolveBuilder`/`SolutionSet`](../../vendor/symbolica/src/solve/solution_set.rs)
are useful candidates: compare the complete solution and coverage conditions against leaving the
block to the existing native formulation. Carry pivot/nonzero assumptions, guards and recovered
original coordinates. `a·x=b` at `a=b=0` has a free direction, and cancelling
`(x²−1)/(x−1)` must not admit `x=1`. Neither `zero_test`'s sampling nor an exact identity test
establishes pointwise domain validity. Gröbner elimination, modular reconstruction and opaque
coefficient rings can avoid work on specific exact blocks; there is no basis to run them on all
rows. The maintained bounded cubic supplier already serves its original root-selection contract;
reuse it rather than substituting unbounded generic root enumeration.

**Decision evidence.** Start with `coefficient_objective_projection_reuses_proved_rows_under_one_work_ledger`
and the selected-affine-row controls in [assembly tests](../../crates/pse-math/src/assembly_tests.rs).
Compare constants, weights, ordered columns/pairs, changed fixed inputs and rank-deficient or
out-of-domain negatives against independent coefficients and original residuals. Prove before
deletion that the single request-local result serves both real consumers; remove only the
duplicate objective reconstruction and its mechanics tests. Keep the library-neutral
[factorable projection](../../crates/pse-math/src/factorable.rs): it carries exactness and
obligations into external native engines, a different claim from degree or an f64 value.

<a id="I04"></a>
### I04 — Prepare the adapter's exact mathematical demand once

**Proposed amendment · D01/D02/D04, C02/C04, A02/A04 and the actual G–J consumers.**
[`assess_static` and native admission](../../crates/pse-backend-native/src/routing.rs) ask the
adapter for callback demand; [conditional solve](../../crates/pse-runtime/src/math/solves.rs)
and [initialization](../../crates/pse-runtime/src/math/initialization.rs) ask again. Reuse one
checked demand under the existing selected profile/adapter, route, witness and exact support
owner when those premises are the same. Recompute when initialization or provider/solver
settings change the contract. This removes repeated projection and makes the intended numerical
product visible before allocation. It does not equate a solver's selected action with a dense
derivative or with permission to publish one.

Reuse the existing grouped `PreparedSupport` and scalar sharing for unchanged regions. Prefer
joint `evaluator_multiple` and scoped definitions when outputs are known together; evaluate
[`ExpressionEvaluator::merge`](../../vendor/symbolica/src/evaluate/optimize.rs) only for
already-built, compatible exact evaluators whose real consumer would otherwise rebuild them.
The maintained `merge` requires matching **ordered** parameters but checks only their count;
the owner must verify layout, coefficient domain, branch/guard/effect membership and output
order. It requires `Eq + Hash`; primitive `f64` is not that domain. `Dualizer` and generic
`Vectorize` are useful for selected directions/lanes, not a reason to widen to maximum order;
retained non-inlined sub-evaluators can refuse vectorization. Optimizer controls and
`optimize_stack` manage optimization/temporary work, not preconstruction admission.

**Decision evidence.** Use the existing `preparation_and_arclength_suitability_are_adapter_facts`
control, then a JVP-only SPGMR route versus a preconditioned or assembled route, and an exact
Hessian NLP versus limited-memory or gradient-only fitting. Include two equal-width but
reordered parameter layouts and a provider that supports First but not Second. Preserve
original units, seeds, weighting, sparse shape, failure/settlement and the exact closure of
retained work. Remove repeat demand queries only where the caller consumes the same checked
request. Keep external provider derivatives and typed piecewise regularity explicit:
[`typed_external`](../../crates/pse-math/src/typed_external.rs) already uses Symbolica for the
chain rule and provider-native derivative operations for opaque calls; the corresponding
[`typed_piecewise`](../../crates/pse-math/src/typed_piecewise.rs) obligations are physical policy.

<a id="I05"></a>
### I05 — Apply precision where the product demands it

**Proposed amendment · D02/D04 with scientific consumers in G–K.** Use
[`map_coeff_with_prec`](../../vendor/symbolica/src/evaluate/evaluator.rs) on a suitably admitted
evaluator when a real product needs a different numerical domain. Its precision argument
applies to registered constants; literal mapping is the caller's function, and the maintained
implementation unwraps registered-constant evaluation. It does not correct Numerica literal
rounding, make a foreign callback eligible, or turn arbitrary-precision arithmetic into
certification. Retain the current general binary64 realization until a product needs certified
conversion, and reuse the existing [directed MPFR/cubic contract](../../vendor/symbolica-bounded-roots.md)
for its specific proved scope. Distinguish exact equality, correctly rounded conversion,
conditioning/backward error, interval enclosure, and independent physical agreement. Numerica
ball arithmetic's inherited transcendental methods are not a general enclosure contract.

Use an exact-rational/approved precision path at a real consumer only after a revealing control:
the existing `exact_nonzero_comparison_preserves_rationals_beyond_binary64_extent` uses
differences around `2^±1200`; compare an exact/near-boundary case, a precision-sensitive literal
and an original-coordinate check before changing numerical realization. If multiple consumers
would otherwise duplicate escalation and rounding policy, move just that common selection to
the existing math owner and delete local copies. No repository-wide precision switch. Retain
the current direct weighted-response SVD; an adjoint objective gradient supplies neither the
response Jacobian nor covariance/rank evidence.

<a id="I06"></a>
### I06 — Stop labeling a narrowed strict-domain export exact

**Observed correctness gap; proposed decision amendment · D01/D04, J04 and L.** In the
[native factorable plan](../../crates/pse-backend-native/src/execution/factorable.rs), an
ordinary unconditional strict range passes through `close`, which offsets a finite bound by
`1e-9 × max(1, |bound|)`. The existing `closed_export_preserves_mixed_predicate_order_and_existing_range_policy`
control asserts `Exact` after a separate `Nonzero` loss is removed. Fidelity can then support
`ExactExport` for a bound. For `min x` with `0 < x ≤ 1`, a solve on `x ≥ 10⁻⁹` can return a
point satisfying the original predicate while excluding valid points and changing the infimum
from zero to `10⁻⁹`. Original-point checking cannot repair an invalid global claim. This is
distinct from Symbolica's ability to simplify or represent the original expression.

Use the existing `selected_closure` pattern where a sound closed **superset** at the original
endpoint is safe to evaluate; label it relaxed and retain original candidate qualification.
Where closure would make the native expression unsafe or unavailable, refuse the certified
export. A positive margin can serve an explicitly candidate-only strategy, never a proof,
dual bound or original-space infeasibility claim. Update blueprint §18.10.1 through the
applicable decision route, tracing [ADR-0249](../adr/0249-exact-conjunctive-domain-predicates.md)
and its limited change of the previous policy. Replace the exact-margin assertion with a
pure projection/quality control around a valid point below the margin and unattained infimum;
require a selected real native proof-path check before adoption. Preserve independent
evaluation, original bounds and existing Nonzero semantics. This correction has higher
consequence than the optional reuse improvements and need not wait for I01.

Also check the adjacent conditional case before changing the fidelity policy: if the arithmetic
arms of `if x ≥ 0 then x/x else 1` reduce to the same node, the branch-local exclusion at zero
still matters. The [factorable branch builder](../../crates/pse-math/src/factorable.rs) can merge
equal arithmetic arms and mark guards conditional; the export skips conditional obligations.
This is a source-based test candidate, not a reproduced defect for that exact input. A missing
guard may be a sound relaxation; the inward strict margin is a restriction. Do not give both
the same evidence classification merely because each changes the admitted set.

## Cross-boundary and adjacent improvements

<a id="I07"></a>
### I07 — Retire wire operations that only carry intermediate computation

**Proposed amendment · C02/C04 and B03 with the actual performing and receiving consumers.**
Select the operation before selecting what crosses a boundary. Internal normalization, expression
lookup, proof staging, exact transformations and evaluator assembly do not need separate public
endpoints. The desired unit is a prepared region or complete admitted scientific operation,
selected for the actual consumer. Transfer inputs and requested products at effect/placement
boundaries; keep ordered guards, native providers, attempt state and settlement with their
performing owner. A library format cannot supply those responsibilities (AP-01/AP-07,
DP-10/13/16/19, heuristics H2–H4/H8).

| Observed surface | Recommendation and evidence for deletion |
| --- | --- |
| Private provider-free root `pse.private-native-root.v4`, `PortableRootSource`, `solve_portable_root` and the `pse-native-lane` launcher | The [live-root implementation](https://github.com/paul-heyse/pse-arrow/blob/9de6d7d94e22890bf2c8ba73c2d0b47e0d700562/crates/pse-runtime/src/math/native_child.rs) has an actual executable and isolation controls, but a repository call-site trace reaches only its [acceptance callers](https://github.com/paul-heyse/pse-arrow/blob/9de6d7d94e22890bf2c8ba73c2d0b47e0d700562/crates/pse-runtime/src/math/native_child_tests.rs); production solve paths already have other owners. This is the strongest immediate retirement candidate. Confirm deployment/public Rust consumers before removing the exported route, launcher, protocol fixtures and tests whose sole claim is that obsolete path. Retain useful supplier-transfer and settlement controls at their real owners. If an actual isolated root consumer exists, compare a single complete root operation on the existing performing owner with retaining the lane; do not extend the lane merely because f64 transfer works. |
| Managed scientific-worker `Prepare`, `Evaluate`, `EvaluateBatch` | [`Proxy::fill` and `fill_batch`](../../crates/pse-runtime/src/scientific_worker/proxy.rs) implement actual calls for an explicitly selected isolated provider. The traced first-party `LocalWaterFactory` selections are controls; the [ordinary engineering binary](../../crates/pse-runtime/src/bin/pse-engineering.rs) selects in-process CoolProp factories. Keep the worker out of the ordinary mathematical path. When isolated placement is actually needed, pure neighboring arithmetic may move into one prepared worker region or complete solve, eliminating per-sample crossings. Compare against the existing grouped operation; wrapping scalar calls in a vector does not remove them. Keep explicit batch capability, ordered point IDs, per-point refusal, admission, stale session refusal, original stop and finish. Common shape/identity checking can have one owner; collapse variants only if eligible singletons and cohorts retain their semantics and narrow ordinary cost. |
| Engineering `PrepareCase` → `SubmitRun` | [Source declarations](../../crates/pse-model/src/engineering.rs) and [service session](../../crates/pse-runtime/src/host_operations/engineering/session.rs) show that the prepared identity is currently required by submit; the [separately driven client fixture](../../crates/pse-runtime/tests/engineering_service.rs) proves this two-step journey. Compare a complete authored-case submission that prepares internally under the existing owner for consumers that only want to run. Keep an independent Prepare operation only if a supported caller consumes its preparation inspection/reuse/authorization semantics. Move actual clients before deleting the mandatory preflight leg and its generated contract; never discard stale-revision checks, idempotency, original preparation provenance or independent failure/recovery behavior. This is a public-contract candidate, not a finding that external callers are absent. |
| Generated binary recipe and protected result/Study surfaces | A qualified recipe route would replace one current encoding after I01, not add a public Atom API or a parallel supplier RPC. Keep Study input capabilities with distinct meaning, operation admission, result observation/cancellation, and protected reads. Their authorities, failures and lifetimes remain even when intermediate mathematical steps disappear. Leave Arrow, DataFusion and Protobuf at the consumed data/contract boundaries. |

**Decision evidence.** Compare the complete path with the same request, preparation reuse,
callback shape, result/order, attempt, cancellation and evidence. Demonstrate actual deletion of
producer/receiver hops and generated reach, or cheaper ownership with a real worker/client
case, before changing a live protocol. Reuse the original-connection, abandoned Prepare and
failed retirement controls, and a foreign session/changed repeat case. In particular, one
combined request must not turn an unsettled attempt into a successful result. Do not resurrect
the mirrors already replaced in the [shared-contract review](../design_review/reviews/design_review_shared-engineering-and-solver-contracts_2026-11-08.md).

<a id="I08"></a>
### I08 — Compare released evaluator composition on a real multi-region operation

**Candidate for D02/D05; supplier provenance repair is an independent codebase improvement.**
The workspace uses maintained **3.0.1**, published revision `e9a0d354…`, with defaults off,
GMP integers and MPFR floats; native code generation is explicit via `pse-math/compiled-evaluator`
(locked SymJIT 2.26.0). The checkout's [complete supplier delta](../../vendor/deltas/symbolica.patch)
and [source receipt](../../vendor/deltas/symbolica.source.json) predate the tracked `build.rs`
fix in `f903f0b05`; the tracked file's checksum differs. Refresh those artifacts from the
verified archive and actual tracked tree in a separate supplier maintenance change, then use the
documented byte/mode comparison. Do not remove the build-provenance fix or change pins merely
to make the retained receipt pass. This issue is independent of Symbolica adoption.

Both published archives were checked against the [crate registry](https://index.crates.io/sy/mb/symbolica).
Released **3.0.2** (`afbf7aa4…`, Numerica `^3.0.2`) adds three meaningful alternatives:

- [`EvaluatorComposer`](https://github.com/symbolica-dev/symbolica/blob/v3.0.2/src/evaluate/compose.rs)
  binds prior results/constants to later inputs, selects outputs, and can prune and optimize
  composed programs without repeating symbolic Horner search. `finish` requires `Eq + Hash`;
  reoptimization can change binary64 realization. Evaluate on a named provider-free chain
  whose parts are independently reusable, comparing current joint preparation, separate stages,
  and composition. Keep original ordered coordinates, guards and invalidation; an eliminated
  pure computation and a skipped fallible provider are different effects.
- [`ExpressionEvaluator::from_instructions`](https://github.com/symbolica-dev/symbolica/blob/v3.0.2/src/evaluate/instruction/import.rs)
  validates forward control flow and definition-before-use, including nested programs. It takes
  already allocated structured instructions, allocates validation/recursive state, and builds a
  new stack/result layout. It can simplify a trusted numerical-program boundary but does not
  provide the maintained passive-byte budget or original-layout transfer. It cannot restore
  a symbolic body. The release's [Atom Serde tests](https://github.com/symbolica-dev/symbolica/blob/v3.0.2/tests/serde_atoms.rs)
  demonstrate remapping across processes, not a bounded preflight; allocation and global-state
  import remain in the path.
- [Native zero/equality tests](https://github.com/symbolica-dev/symbolica/blob/v3.0.2/src/atom/zero_test.rs)
  add exact fallback, but a sample limit is not its resource bound and a nonidentity result is
  not a pointwise nonzero guarantee. Use them only for an identified evidence contract.

The candidate archive lacks the maintained packed inspector, f64 transfer and bounded real-root
overloads. Its published `build.rs` is unchanged from published 3.0.1 and does not include the
local provenance fix. Before any pin change, require equivalent consumed contracts or a scoped
rebase, one resolved exact supplier family, the actual feature closure, and discriminating
compiler/native tests. Compare the whole operation under identical evidence conditions; the
candidate also changes optimizer behavior. No upgrade recommendation or claimed speedup here.

## Scientific acceptance and sequencing

| Representative operation | Distinction any shared implementation must preserve |
| --- | --- |
| [G02/G04: Water, wet turbine and ordinary BT](../plans/66g-properties-equipment-and-steady-workflows.md#packages) | CoolProp saturation endpoint jets are derivatives along pressure at fixed quality endpoint; homogeneous Water uses temperature and molar density. Compose the selected jets with authored arithmetic; do not substitute one set of derivatives or collapse phase identity. Ordinary BT retains selected charts, regime-crossing refusal, ordered provider ports and the original attempt. The later protected BT read confirms provenance, not closure of historical derivative discrepancies. Keep independent endpoint/Clapeyron, original energy, entropy and component checks, and the outstanding protected wet receiver. G03 pure-ethane viscosity supplies a contrasting provider-free authored law. |
| [H03–H06: aqueous cascade and feed/wash/feed](../plans/66h-inventory-distributed-and-cyclic-physics.md#packages) | Reuse the arithmetic stencil and its demanded derivatives; keep independent initial stocks, conservation, discontinuity handling and stock/time custody. The H03 strict-zero Radau control exposes negative internal stages that higher arithmetic precision does not fix; preserve the positive cohort's independent refinement and original-clock trajectories. Fixed holds do not prove periodic or state-triggered behavior. Start at zero, with `x′=p>0` until `x=1`, reset to zero and continue with `x′=2`: the post-event sensitivity at fixed `T>1/p` is `2/p²`. Differentiating only the local post-event RHS misses it. Preserve the native event/consistency owner and typed refusals. |
| [J02/J06: mixed inference and predictive control](../plans/66j-inference-uncertainty-design-and-control.md#packages) | The [existing mixed-fit control](../../crates/pse-runtime/src/workflow/fitting/p09_tests.rs) already combines one steady and two differently scheduled transient experiments at nonzero physical time with independently expected Jacobian rows. Reuse it, then close the configured physical/reconciliation/receiving population. Compare preparation reuse and derivative products without replacing original interval/axis admission, IDAS/Diffsol execution, active-set/hold semantics or weighted-response evidence. A selected gradient must not stand in for independent response or covariance. |
| [I/K: specialized processes and downstream assessments](../plans/66i-specialized-process-families.md#packages) | Beds, capture/separation, electrochemistry, power and networks share arithmetic only where their domains, layout and requested products agree. Do not fuse away transport direction, ragged membership or constitutive limits. K's [costed exchanger](../plans/66k-engineering-assessments-and-native-views.md#packages) is an actual read/view/export receiver: keep unknown area support, pressure extrapolation and cost-index meaning. Views consume original products without recalculating science or strengthening permission. |

Address I06 through its decision route independently of the reconstruction question. Run the
I01 feasibility investigation before freezing any binary interpretation or widening private
numerical transfer. I02–I05 can proceed independently where their consumers already exist:
align the demanded product and guard ownership first, then remove duplicate construction.
I07's private-lane consumer audit can start immediately; live boundary changes follow their
actual C/B/F consumers and the chosen operations. I08 is optional and follows identification
of an independently reusable provider-free chain. The supplier receipt repair can proceed
independently. Use [L01–L05](../plans/66l-required-population-and-assembled-acceptance.md#packages)
for exact configured population, required inputs, original failures and withheld evidence,
complete-operation measurements when a material benefit is claimed, and independent acceptance.
Keep numerical equivalence, physical evidence, protocol behavior and performance as separate
claims. A small pure control can discriminate an arithmetic or packing rule; it does not
replace real settlement, reference, protected-reader or client journeys.

## Coverage and limits

The [Symbolica reference](../libraries/scientific_calcs/symbolica.md) was read in full (Git blob
`26dc8e44a6ae5e11215b3e0805793661aa4dfa80`). The two Context7 indexes and queries helped locate
sources, but their unversioned Rust/Python results do not establish a pinned Rust contract.
Decisive comparisons used maintained source and tests, checksum-verified published archives,
versioned Rust APIs and official source. Later or companion features were screened for a real
consumer and independently verified where material. This was not an exhaustive upstream API
absence claim, dependency powerset, completion audit or performance campaign.

| Guide coverage | Treatment |
| --- | --- |
| Base §§1–3; Rust addendum §§1.1–1.2 and 2 | Borrowed views, structural traversal, symbols/attributes, scoped rewriting and mathematical properties inform I01–I03. Preserve Unknown, normalize/match semantics and source guards. Rule pipelines or callbacks can replace demonstrably duplicated algebra, not typed model admission. Associative search and repeat transforms need a termination/work contract. |
| Base §§4–5; addendum §§1.3, 3 and 5.1–5.3 | Exact domains, coefficient rings, polynomial arithmetic, rational forms, algebraic extensions and root coverage inform I03/I06. Resultants, modular reconstruction, Hensel and sparse GCD are tools for an identified exact block; neither failed reconstruction nor numerical Newton supplies global evidence. Rationalization changes the value being claimed. Keep embedding/root identity and original assumptions. |
| Base §§6–8; addendum §§4 and 6 | Symbolic derivatives, selected Taylor/dual propagation, multi-output and nested evaluators, merge, instructions, precision and lifecycle inform I02/I04/I05/I08. Series-to-expression drops the unknown remainder; no current Plan 66 certified error product can infer a bound from it. Adaptive integration is a different operation from temporal DAE/ODE solve or campaign uncertainty. JIT, SIMD, C++, assembly and CUDA remain candidates only for a concrete workload and qualified feature/target profile; compiled loading supplies numerical work, not symbolic replay. |
| Base §9; addendum §5.5 | Packing, state handling, aliasing and memory/growth inform I01/I02. [`TermStreamer`](https://raw.githubusercontent.com/symbolica-dev/symbolica/v3.0.1/src/streaming.rs) spills terms after buffering them; `max_mem_bytes` is not admission for an individual term, exact limbs, derivative expansion or global merge. Consider it only for a genuinely large additive workload, with owned disk, cancellation and settlement. |
| Base §§10–11; addendum §§5.4/5.6 | Numerica supports exact/numeric domains; Graphica canonicalization is a candidate for an actual structural symmetry, not identity of physical occurrences. Matrices/tensors and special functions are a fit only with a declared physical/precision contract. [Spenso 0.6.0](https://index.crates.io/sp/en/spenso) declares an optional Symbolica `^1.2` dependency; a same-Atom 3.x route needs a qualified revision or deliberate bridge. [`symbolica-integrate` 2.0.1](https://index.crates.io/sy/mb/symbolica-integrate) accepts `^3` and its defaults can widen the workspace feature closure. Neither establishes a current requirement. Community/HEP, Typst and experimental ODE integrations supply no applicable native replacement on the traced footprint. |
| Base §§12–16 and conclusion; addendum scope, §7 and conclusion | Apply exact-release Rust and feature boundaries, record the deployment implications of licensing without using it to reject technically suitable candidates, and use the referenced primary sources. Python methods, Wasm and C bindings do not establish Rust support. Manifests for published 3.0.1/3.0.2 say Rust 1.96; the guide's older minimum is not the workspace requirement. I08 records relevant additions and corrections. |

| Plan companions | Review disposition |
| --- | --- |
| [A](../plans/66a-scientific-operations-and-product-semantics.md), [B](../plans/66b-contracts-generation-and-interpretation.md), [C](../plans/66c-preparation-execution-and-service-lifecycles.md), [D](../plans/66d-mathematical-representation-and-numerical-execution.md) | A retains product, coordinate and permission meaning; B projects only consumed contracts. C and D jointly own reconstruction, preparation and placement; apply I01–I08 at those owners where selected. |
| [E](../plans/66e-protected-data-and-result-delivery.md), [F](../plans/66f-developer-operations-clients-and-evidence.md) | Preserve exact original protected results, readback and attempt lineage; no Symbolica replacement for result custody. Migrate actual independent clients and original evidence acquisition/rerun if I07 changes a performing boundary. |
| [G](../plans/66g-properties-equipment-and-steady-workflows.md), [H](../plans/66h-inventory-distributed-and-cyclic-physics.md), [I](../plans/66i-specialized-process-families.md) | G/H representative implementation was traced; I's family prerequisites and receiving footprint were reviewed. Extend only through the compatible regions and products, with independently checked science. No claim of full family equivalence or completed prerequisites. |
| [J](../plans/66j-inference-uncertainty-design-and-control.md), [K](../plans/66k-engineering-assessments-and-native-views.md), [L](../plans/66l-required-population-and-assembled-acceptance.md) | Trace preparation and numerical demand into fitting, response, control, surrogate and actual assessment consumers. Keep inference/estimation meaning, original reporting and exact acceptance denominator with their current owners. Test the selected products, not a generic library sweep. |

No product build, test or simulation was run for this review. Referenced test names identify
existing controls or proposed discriminators; prior results retain their original conditions.
No speedup, peak-memory reduction, production activation or change to plan acceptance is claimed.
