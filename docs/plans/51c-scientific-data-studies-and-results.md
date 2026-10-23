---
title: "Plan 51c: Scientific data, studies and results"
status: draft
date: 2026-10-22
adrs: []
review_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
scenario_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/51-engineering-workspaces-and-evidence.md
doc_topics: [current-work, design, numerical-execution, validation, storage]
---

# Plan 51c: Scientific data, studies and results

## Scientific target and existing foundations

**Proposed:** navigate a complete, explicit chain from original measurements through calibration,
observation selection and a qualified fit to a newly authored parameter revision and downstream
study. Select study/result cohorts by their actual scientific context, inspect original protected
results lazily and follow progress with truthful reconnect behavior. O04/O05/O12/O21 disposition
and package state belong to the [coordinator](51-engineering-workspaces-and-evidence.md).

**Interface-checked:** measured data is already typed. The reference
`packages/reference/seed-data/models/vessel-measurements.pse` includes provenance, units and
standard uncertainty. Existing `fit_cases` declare parameter bounds/scales, experiments/routes,
bindings, observation identity, measured attributes, standard deviations, time basis, inclusion
and importance. `workflow/fitting/modeling.rs::FitDeclarations` and fitting execution already
check original experiments and candidate assessments. `FitReport` retains candidates, predictions,
residuals, trajectories, checks and qualified covariance/Wald/profile evidence. Runtime fit
parameter/observation/fitted-cell access retains run and source-unit context.

The missing capability is composed research ownership and an explicit fit-result-to-authored-
parameter-to-downstream-study bridge. Do not duplicate fitting, introduce a calibration solver,
silently copy floating-point values or make a successful database write a scientific qualification.
Library-owned mathematics and class-specific native solvers keep their current responsibilities.

Existing local studies are store-independent; `Runtime.record_study` explicitly records an exact
model revision/physical source and its points/variants. `StudyHandle` has status, wait, result,
cancel and review inventory/diagnostic selection. `StudyOutputDemand::{Full,Compact}` names exact
relations/member rows/bytes and refuses excess rather than silently truncating. Result access
uses original run/attempt/range and current protection. Extend these paths rather than invent a
workbench-specific execution engine or result payload backend.

## Research identities and data movement

Represent immutable dataset versions and the exact original/curated/derived data objects they
reference. Preserve source provenance, acquisition clock/time basis, physical quantity, unit,
basis/reference state, standard uncertainty and inclusion decisions. These meanings use existing
physical typing and authored source declarations. Unknown uncertainty or calibration is explicit;
it is not zero. A changed calibration or exclusion produces a new selected context, retaining
the original raw measurement and its ancestry.

Physical-property sources, reference cases and reference-campaign Parquet datasets use their
existing content/provenance owners in this catalogue. A selected surrogate-training or validation
dataset can be a derived research artifact with those same exact sources and exclusions; this
does not select a new ML framework, active-learning loop or automatic experiment scheduler.

Calibration is an explicitly declared, checked transformation or existing quantity conversion,
with versioned parameters, validity/effective interval and original-to-derived correspondence.
Use existing scientific mathematics/admission for the transformation. No bespoke Rust equation
evaluation, implicit interpolation outside validity or overwrite of raw observations is selected.
PJ05's effective-time and recorded-cut semantics select historical context; a recorded fit pins
the actual dataset, calibration and observation selection, not a floating "latest" query.

Transform uncertainty/weights explicitly too: use authored transformed uncertainty or an existing
declared supported propagation rule with its assumptions. Raw standard deviation cannot be carried
unchanged through a scale or nonlinear transformation. If supported uncertainty treatment is absent,
retain the calibrated values as such but refuse any fitting weighting/qualification that requires
it; do not infer zero uncertainty or manufacture a Jacobian-based guarantee.

Parameter publication is a deliberate command. It selects original fitted cells from one exact
run/attempt and checks actual fit outcome, parameter eligibility, finite values and physical
admission. Independent uncertainty/covariance validity remains independent: absence of qualified
covariance does not by itself make a usable finite point estimate invalid, and a point estimate
does not confer uncertainty validity. Produce an ordinary authored source delta, admit/publish
through the source owner and record a receipt referencing the original fitted cells and actual
new source revisions. Never make the receipt a competing copy of the fitted truth. Downstream
study creation records the consumed source and research ancestry so lineage survives beyond a UI.

Metadata selection remains in indexed native queries: project/branch/source/physical context,
dataset/calibration/fit provenance, study occurrence, run/attempt and truthful outcome. Freeze
cohort membership and completeness before analysis. Select the required protected result relations
and process Arrow batches/DataFusion for cross-run summaries, residual populations or rankings.
Do not fetch every scalar separately or load all trajectories to display study headers. KPI
comparisons require compatible physical meaning and explicit missing/failed members; a small
display limit does not establish bounded upstream work or a complete ranking.
Use seek/continuation for neighboring-case navigation when its native ordering matches the
predicate. A bounded sample cannot establish a global optimum. Explicitly retain unattempted,
cancelled, failed, candidate and usable point distinctions in selection and display.

## Observation and external input

`RunHandle.progress` is bounded with drop count; `current_progress` is a current typed checkpoint;
`Runtime.progress` retains available terminal history. Live simulation cursors have a distinct
lifetime from cancellation. Compose these existing owners for a selected study and exact point
occurrences, retaining parent and child run/attempt identities. An observer cursor identifies its
issuing selection, source sequences and catch-up position. Reconnect either catches up from
available history or reports an explicit gap and resynchronizes the current snapshot. Never
invent missed samples, infer continuity from timestamps or reorder integration/model time.

The selected gRPC transport already exposes `query_stream` and `subscribe_live`; native delivery
can reduce polling for refresh hints. In the pinned client, `subscribe_live` sends
`resume_from: None`. Therefore durable replay remains an application-history contract; a live
notification alone is neither a recorded scientific sample nor proof of complete change coverage.
Query rows remain provisional until successful statement completion. Exiting/cancelling an
observer releases its resources but does not cancel or drain the study.

External acquisition is a separate bounded choice. A timestamped external observation producer
could reuse dataset/calibration admission and the observer, but a database integration does not
establish clock accuracy, calibration, delivery loss policy or real-time device control.

## Work packages

<a id="SC01"></a>
### SC01 — Dataset versions and source admission

Consume ST01's research/source declaration slice and PJ01's exact project membership. Add explicit
create/select/version operations for measurement objects, their provenance and raw/curated/derived
relationship. Connect selected data to existing typed source admission and fit declarations;
add Rust/Python/CLI discovery and selected-detail consumers. Bulk data stays in existing sealed
owned artifacts; native queries select metadata without scanning all measurements.

Acceptance: independently authored unit/basis/uncertainty fixtures round-trip and reject incompatible
physical context; correcting a dataset retains its original version and original raw values.
Selecting metadata cannot grant payload lifetime. Remove any prototype parallel dataset schema
or direct table-to-fit shortcut as consumers migrate.

<a id="SC02"></a>
### SC02 — Calibration and observation selection

Use SC01 and PJ05's exact temporal context. Admit declared calibration and an explicit observation
selection into the original fitting inputs, preserving exclusions and uncertainty interpretation.
Expose why an observation was selected, transformed or excluded through original source references.
Keep the original experiment validation and qualified mathematical path active.

Acceptance: a known calibrated dataset agrees with an independently specified transformation.
A non-unit-slope fixture checks transformed values, standard uncertainty and actual fit weights,
including refusal of unsupported uncertainty propagation. Wrong units, invalid domain and
conflicting effective context fail. A backdated correction changes
a new selection but not the observations or receipt of an old fit. A missing observation is not
silently filled or treated as a zero residual.

<a id="SC03"></a>
### SC03 — Fit-to-parameter-source publication

Connect SC02, the existing fit result owner and PJ04's source-publication operation. Provide an
explicit preview/publish operation selecting eligible fitted cells and target authored parameter
objects. Preserve bounds, physical typing and fit/uncertainty qualifiers. Publish through ordinary
source CAS; settle unknown effects by the original operation. Store only the publication receipt
and ancestry references, with source values owned by the resulting admitted source.

Acceptance: fit a representative declared experiment, publish eligible parameters and execute a
downstream study whose ancestry reaches the original observations. Reject wrong run/attempt,
ineligible/nonfinite cells, incompatible units and moved heads. Verify independent covariance
failure does not become fabricated uncertainty or automatically erase eligible point estimates.
Delete any copy-floats convenience path that bypasses source admission/provenance.

<a id="SC04"></a>
### SC04 — Study/cohort query and lazy result workbench

Consume available PJ01/SC01 references and ST02/ST04 selection contracts; calibration publication
is needed only for queries promising that lineage. Add indexed filters over real study/run/outcome
and research context, freeze exact cohort membership and expose compact headers followed by
explicit protected detail. Integrate Rust/Python/CLI cohort comparison with existing output demand,
result selection and selected Arrow/DataFusion analytics. Preserve unsupported/missing members.

Acceptance: a cohort with successful, failed and absent results retains an honest population;
unit/context-incompatible KPIs cannot be ranked together without an explicit admitted transform.
Selecting one result avoids decoding unrelated payloads; released or altered payloads fail at
their original protection reader. Exercise the public selector and analysis, not only a SQL helper.

<a id="SC05"></a>
### SC05 — Study observer and reconnect

Compose existing parent/point run progress, live snapshots and terminal history into a read-only
study observer using ST04's cursor/lifetime conventions. Public Rust/Python/CLI consumers can open,
poll/follow, disconnect and reconnect with an explicit selected-context cursor. Use notification
hints only when useful; bounded polling of selected owners is a valid initial realization, never
whole-store polling. Keep capture limits, output limits and scientific execution allowance distinct.

Acceptance: disconnect during point execution, reconnect with full history and with an intentionally
unavailable prefix, and show exact catch-up or gap/current resync. Wrong selection/authority cursors
fail. Observer exit does not cancel science; a separately issued cancel retains its original
cancellation/drain behavior. No full upstream streaming campaign is required.

<a id="SC06"></a>
### SC06 — External acquisition decision

Resolve inquiry I09 against a concrete external observation producer and workflow. Assess clock
meaning, source identity, calibration validity, idempotence, duplicate/loss handling, admission and
the boundary between captured data and control effects. Compare explicit file/batch admission
with incremental producer integration using the same SC01/SC02 semantics.

If acquisition is selected, admit bulk observation batches with source sequence, clock, units,
calibration and coverage. Do not route every scalar through a solver callback. Incoming data
cannot silently replace an executing case's frozen inputs: observation, a new input revision and
external control are different owned actions.

Completion is either a no-adopt decision with supported batch alternative and observable revisit
trigger, or an explicitly selected integration with its actual producer, supported delivery and
targeted acceptance. Any new platform/effect/rule guarantee receives the maintainer's selection
before implementation. Do not add a speculative acquisition framework or real-time control system.

## Verification and handoff

**Proposed:** targeted scientific/physical and operation tests accompany the changed paths;
51g owns assembled study, fitting, Python and scope-end qualification. A database-only test cannot
qualify numerical behavior. Parameter publication requires the existing scientific outcome checks,
not new assertions about a solver based on metadata success.

Current checkpoint: source assessment and proposed integration only. SC01's source/data slice
can progress with PJ01, SC04 can use existing study records before all research lineage exists,
and SC05 is independent of SC03. No new scientific capability has been tested or measured here.
