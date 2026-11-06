---
title: "Plan66j: Inference, uncertainty, design and control"
status: planned
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/66-integrated-simulation-target.md
doc_topics: [current-work, design, validation]
---

# Inference, uncertainty, design and control

## Remaining outcome and current foundation

This companion of the sole go-forward [Plan66](66-integrated-simulation-target.md)
owns remaining joint inference/reconciliation, uncertainty, surrogate/discrete design,
adaptive studies and control/multiperiod application delivery. It preserves the exact
required scientific population at [66l](66l-required-population-and-assembled-acceptance.md),
including held external trainers, data, inherited callable cases and failed rows.
Assessment and native engineering views belong to [66k](66k-engineering-assessments-and-native-views.md).

**Interface-checked baseline:** main `66341514d` already implements native fitting,
[qualified/withheld covariance](../../crates/pse-runtime/src/workflow/fitting/covariance.rs),
[library sampling](../../crates/pse-runtime/src/math/sampling.rs),
[stepped surrogate execution](../../crates/pse-runtime/src/math/surrogate/library.rs),
[horizon orchestration](../../crates/pse-runtime/src/workflow/horizon.rs) and local/result
inspection. Horizon includes MHE/NMPC, sequential physical plant application, previous
accepted seeds, KKT advanced-step prediction and fallback. These mechanisms are not
missing implementation packages. Remaining work is new scientific contracts exposed by
actual experiments, finite application integrations, receiving consumers and missing
independent acceptance. Existing task/step/point permission must not be mistaken for
complete experiment/member permission or a qualified statistical product.

The target adopts review I01/I03/I09/I10 and scientific S02–S07. One physical model
serves multiple analyses through explicit scientific interpretation, not copied model
graphs or a universal campaign engine. Product availability is demand-shaped and scoped
through [66a](66a-scientific-operations-and-product-semantics.md); mathematical realization
and derivative eligibility belong to [66d](66d-mathematical-representation-and-numerical-execution.md).

## Scientific population and interpretation

| Outcome | Required configured scope, inputs and distinctions |
|---|---|
| Joint inference/reconciliation | Shared-parameter exchanger fitting, selected NRTL/PR experiments and required reaction/property cases; observation correspondence, bounds, weights/noise, shared versus experiment-local parameters, initial state and schedule target roles; source-selected reconciliation retains reconciled versus raw/input/outcome roles |
| Mixed experiments | A steady experiment plus two transient experiments with different schedules; consistent initial conditions and actual physical time; joint residuals and sensitivities preserve original experiment/attempt/member identity and selected response demand |
| Uncertainty/robustness | Declared input measures/dependence and propagation/estimator roles; realized designs, all selected outputs and failure interpretation; the six fixed 200-row robustness populations retain all 1,200 rows, original nominal/cloning policies and attributed outcomes |
| Surrogate pipelines | Required ALAMO/PySMO training/evaluation, polynomial/RBF/kriging and adaptation/CV choices, analytic/training rows and six sampling families; selected reformer/sCO2 and modern surrogate helpers; ONNX/OMLT-style scaling/evaluation/embedding/optimization behavior and exact artifact/trainer prerequisites |
| Adaptive experimentation/design | Typed proposals versus pending/observed/failed evaluations; bounded physical operating envelopes and selected mixed/discrete choices; actual observation and restart provenance, complete native evaluation and scientific acceptance |
| Control/state estimation | Steam/SOC PID and selected CSTR NMPC cases; steady targets, plant/controller horizons, time/hold/observation/state custody, input/rate bounds and failure policy; native MHE remains a native extension rather than released-reference equivalence |
| Multiperiod/grid | Period linking/ramping, dispatch/design-operation and price-taker commitment/clustering; thermal-generator bidder/tracker/coordinator/forecaster/backcaster roles where selected; prices/scenarios and exact ordinary CSV/helper/plugin inputs; market periods do not imply physical integration |

Immutable input-role provenance is
`git:66341514d:docs/plans/64h-required-process-population.md`, transferred to 66l.
The six robustness populations' historical 1,071 successes and 129 failures are
reference outcome evidence, not a failure budget or permission to omit rows. Resolve
admissible physical envelopes and expected infeasibility/domain failures explicitly.
Unavailable required external trainers, TensorFlow/helper imports, model weights,
scaling metadata and Prescient integrations remain prerequisites of their selected
cases, not silently excluded capabilities. Archived reconciliation adoption remains
66l's explicit supplemental decision; mandatory scientific reconciliation persists.

Joint inference declares which quantities are fitted, observed, prescribed or nuisance
variables, and which are shared between experiments. An observation unit/datum/time
map is part of the model. Confounding must remain inconclusive; additional samples or
a GP variance cannot repair structural non-identifiability. Distinguish parameter
uncertainty, measurement noise, numerical error, physical model discrepancy and
predictive uncertainty. Do not promote an ordinary result matrix to a covariance claim.

## Concrete native/library realization

Keep direct weighted-response SVD in the existing covariance owner; it avoids squaring
conditioning through normal equations. faer 0.24.4 `LinOp`/`BiLinOp`, `lsmr` and explicit
scratch are conditional large-response mechanisms, with actual original-coordinate
action/rank/error checks before they can support a product. See maintained
[operator source](../../vendor/faer/src/operator/mod.rs) and
[LSMR](../../vendor/faer/src/operator/lsmr.rs). Many repeated right-hand sides may favor
existing sparse factors. Do not weaken covariance withholding to make an iterative
response route appear interchangeable. ndarray 0.16.1 borrowed views and `Zip` support
batch/observation layouts without newer `ArrayRef` APIs or Arrow mathematical kernels.

For adaptive native studies compare the existing `EgorSolver` stepped owner with an
ask/tell proposal adapter using ego 0.38.1
[`EgorServiceApi::suggest`](https://docs.rs/crate/egobox-ego/0.38.1/source/src/solver/egor_service.rs).
Its factory supports mixed-integer space; asynchronous/batch/constraint semantics still
need the selected consumer contract. gp 0.36.4
[`GaussianProcess::update`](https://docs.rs/crate/egobox-gp/0.36.4/source/src/algorithm.rs)
uses a block Cholesky update with fixed theta/normalization; it is not a hyperparameter
refit. Select update/refit policy from observed scientific scale/regime changes.
Reuse current native admission, cancellation, work accounting and panic settlement;
do not add a second optimizer/workflow scheduler. Failure rejection/imputation/viability
must preserve real failure observations; imputed responses cannot become measured truth.

Existing native DOE records library/version and normalized physical map. Extend the
actual campaign replay contract using rand_xoshiro 0.6.0 `jump`/`long_jump` or retained
generated designs when needed. Bind streams to semantic work units rather than worker
arrival. Checkpoint all sampler/adaptive state and coordinate transforms; a seed alone
is insufficient. Separated streams do not prove independence, and a uniform normalized
sample need not have a uniform transformed physical measure. See
[exact 0.6.0 source](https://docs.rs/crate/rand_xoshiro/0.6.0/source/src/xoshiro256plus.rs).
No newer `state()` API or unselected serde feature is assumed.

Preserve mathematical class in 66d: smooth local optimization, discrete/mixed decisions,
surrogate evaluation and optimization embedding have different guarantees. Selected
native alternatives can satisfy an explicitly matched outcome; they do not reproduce
an unavailable trainer or establish global optimality automatically. Library mechanisms
above are additive to complete application delivery, not substitutes for its population.

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| J01 | planned | 66l finite scientific/input selections; existing fit/Study/horizon owners | Remaining application, statistical interpretation, experiment/member product and receiving-consumer gaps are resolved without recreating delivered engines |
| J02 | planned | Selected working 66g/66h model; J01 observation/parameter roles; 66a/66d changed product slice | Joint steady/transient inference and reconciliation deliver attributable residuals/parameters and honest identifiability/uncertainty through native readers |
| J03 | planned | Accepted deterministic evaluation; selected measures/dependence and realized inputs | Required uncertainty/robustness populations retain every member/failure and produce statistically appropriate original-coordinate products and replay |
| J04 | planned | Selected training/artifact/helper closure and 66d representation/class | Required surrogate/discrete stages deliver actual evaluation/training/embedding/design behavior or retain explicit held prerequisites; unsupported demands remain refused |
| J05 | planned | Selected J02/J03/J04 expensive evaluation and native study lifecycle | Adaptive proposal/update/refit/replay integrates under existing owners with pending/observed/failed distinction and scientifically attributable downstream decisions |
| J06 | planned | Selected 66h physical-time/model products or accepted steady period model | PID/NMPC/MHE and multiperiod/grid consumers deliver original state/time/coupling, feasible/infeasible behavior and required initialization/restart correspondence |
| J07 | planned | Selected J02–J06 producers plus 66b/66e/66f receiving paths | Typed clients, retained/exported products, assessment/view consumers and changed bindings migrate; replaced proposal/aggregation/interpretation paths and fixtures are deleted |
| J08 | planned | Selected J02–J07 deliveries and 66l expectations | Complete finite application evidence and benefit/extension controls reach 66l, with unsupported or inconclusive products preserved and enduring meaning published |

J01 operates per ready application. A shared exchanger experiment can proceed from
accepted steady physics while transient fitting waits only for its actual temporal
products. J06 reuses the implemented horizon engine: it closes the finite physical
controller/grid cases and missing receiving behavior. J05 becomes concrete through a
selected required adaptive/design consumer, not an unconditional new framework. A family
with held training data does not stop independent deterministic fitting or control.

## Migration and acceptance

Requests preserve original scientific model/input/experiment identity and demand;
results preserve executed attempt, member and supported product conditions. Compatible
numeric overlays remain legitimate; immutable authored originals, caller preparation
overlays and applied solver inputs have separate attribution. Use
[66b](66b-contracts-generation-and-interpretation.md) for declaration/generation/adoption,
[66c](66c-preparation-execution-and-service-lifecycles.md) for actual performing ownership,
and [66e](66e-protected-data-and-result-delivery.md)/[66f](66f-developer-operations-clients-and-evidence.md)
for protected/read/export/client delivery. Retire duplicate loops and old mappings only
after complete selected callers migrate, not merely after a library proposal compiles.

**Planned verification:** force-validating native controls challenge deliberately wrong
experiment/member associations; reordered observations; changed initial/schedule targets;
steady plus two different transient experiments; confounding, rank loss and withheld
covariance; failed and missing samples; incompatible physical measures; restart under
different worker ordering; trainer/evaluator/scaling mismatch; infeasible controller
horizons, active-set prediction fallback, period-link/ramping violation and failed
siblings with independently available products. Compare independent residual/closure,
known-parameter and distributional expectations appropriate to the claim.

Preserve zero-state structural analysis and diagnostics separately from current point
products. No label change or historical decoded result can strengthen permission.
Selected controls/readback prove their scope; 66l owns assembled populations and isolated
IDAES 2.13.0 references where comparable, plus native-extension evidence. No tests or
measurements ran for authoring, and no new numerical/statistical benefit is claimed.
