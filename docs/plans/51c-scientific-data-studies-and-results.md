---
title: "Plan 51c: Scientific data, studies and results"
status: complete
date: 2026-10-22
adrs: [ADR-0218]
review_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
scenario_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/51-engineering-workspaces-and-evidence.md
doc_topics: [current-work, design, numerical-execution, validation, storage]
---

# Plan 51c: Scientific data, studies and results

## Scientific target and existing foundations

**Implemented:** navigate a complete, explicit chain from original measurements through calibration,
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

The extension composes research ownership and an explicit fit-result-to-authored-
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

**Implemented:** SC01–SC06 are complete at targeted acceptance on 2026-10-23. The
[coordinator](51-engineering-workspaces-and-evidence.md#package-state-and-execution-route)
owns package/opportunity state. SC06 selects explicit file/batch admission, without external
streaming acquisition or control effects. Revisit only a named producer with defined clock,
calibration, ordering, idempotence and loss semantics.

**Tested:** the following commands exercised the actual pinned Linux/MX1.0 store, original
scientific producers and selected protected readers. Correctness builds explicitly verified
Arrow force-validation; the failure baseline for every check is zero. These are affected
functional controls, not whole-series qualification.

The shared Rust acceptance command for 51c/51d is:

```bash
scripts/pse-env --native=solver -- .venv/bin/python -m scripts.arrow_validation nextest run --locked \
  -p pse-runtime -p pse-operations -p pse-backend-native -p xtask \
  --features pse-runtime/canonical-tests,pse-runtime/solver-ipopt,pse-operations/evidence-query,xtask/project-tools,xtask/evidence-tools,xtask/canonical-tests \
  -E 'test(research_) | test(workbench_) | test(study_observer_) | test(evidence_) | test(project_cli_runtime_) | test(bounded_progress_snapshot_after) | test(compact_projection) | test(review_context_ingress_keeps_large_integer_payload_and_rejects_unknown_version) | test(review_context_selection_refuses_unbounded_duplicate_and_oversized_facts) | test(plan38b_requested_seed_survives_cancellation_before_dispatch_and_portable_reconstruction) | test(fitting_residual_lm_public_result_reopens_actual_method_termination_and_profile)'
```

| Command / scope | Result and boundary |
| --- | --- |
| Shared Rust acceptance above | 70 passed, zero failed, 1,839 excluded by the explicit selection; actual native fit/publication/downstream, source/replay, cohort/observer, evidence and CLI controls across 10 binaries (Nextest `83280600-0939-478c-8157-c53fa09c321b`) |
| `scripts/pse-env -- just py-sync-correctness` | Passed actual force-validated dev extension build, native signature generation and annotation checks on Python 3.14.8 |
| `scripts/pse-env -- .venv/bin/python -m pytest -q python/pse/tests/test_research.py python/pse/tests/test_study_workbench.py --basetemp /tmp/pse51cd-py-final` | 2 passed, zero failed; public creation/discovery, typed member/selection/exclusion detail, physical refusal, immutable correction, protected reopen, actual two-point study browsing/cohort and passive observer authority/reconnect/exit |
| `scripts/pse-env -- just codegen` | Passed all five generator targets, including strict documents, registry/Arrow/native schemas, Python contracts, actual native stubs and Ipopt bindings |
| `scripts/pse-env -- just project-build` | Passed dedicated CLI build; `project_cli_runtime_import_fork_compare_and_passive_discovery` in the Rust selection exercises actual file/batch admission and the shared typed CLI handler |
| `scripts/pse-env -- just ready` | Passed configured environment/deployment checks; unrelated historical resources on unsupported deployment profiles remain untouched and cleanup deferred |
| `scripts/pse-env -- just family-check` | Passed one exact Arrow/Parquet/object_store/DataFusion type universe after the optional evidence-query dependency edge |

The scientific control `research_calibrated_fit_reopens_and_publishes_original_cells` admits an
authored affine calibration with non-unit slope and transformed sigma, verifies actual weighted
residuals, performs residual LM, drops live results and reopens original eligible fitted cells.
It retains usable estimates while actual covariance is withheld, checks wrong attempt/unit/bounds,
local-only preview, lost activation reply settlement, moved-head refusal, backdated correction,
actual native ancestry filtering and an ordinary downstream study reporting prediction 11 from
the published intercept 3 and slope 2. `research_parquet_versions_reopen_original_typed_data_and_reject_partial_input`
uses real Parquet and original data admission. Unknown sigma, explicit exclusion, partial selection
and invalid calibration-domain controls retain original scientific refusal behavior.

`workbench_public_cohort_retains_population_compares_value_variants_and_refuses_incompatible_quantity`
uses a complete success/failed/unattempted population, real selected simulation samples and an
unrelated deliberately opaque payload that must not be decoded. Native snapshot and original
protected readers govern membership and detail. The public observer and backend snapshot tests
exercise actual running-to-terminal progress, bounded later-sequence capture, full catch-up,
unavailable-prefix gap/current resync, issuer/context rejection and observation without a
scientific cancellation lease. Portable replay/review-context controls cover the changed envelopes.

**Not run:** whole-series integration, all solver/Python journeys, full hygiene, governance,
native/powerset checks, other platforms or performance campaigns. These remain Q01–Q04 in 51g.
**Measured:** no quantitative performance improvement is claimed. Existing selected-result
protection is reused; metadata inspection or a cohort document grants no retention right.

**Tested (mechanical publication):** `scripts/pse-env -- just turn-end` passed ADR indexing
and formatting; `scripts/pse-env -- just docs` published 616 chapters with scoped search. These
are not whole-series quality evidence. The final format-only Ruff `--exit-zero` invocation
printed 66 diagnostics against the zero target; it does not claim a clean lint result. The
new undefined native/forward type annotations and misplaced export import were corrected, then
the correctness extension/stub and two Python journeys passed again. Remaining style/type/lint
qualification and repair belong to Q03; no nonzero baseline is accepted. A formatter-only change
to an unrelated existing diagnostic fixture was excluded. Historical unsupported-profile resource
cleanup was deferred without retiring or deleting those resources.

## Outcome

### What was built

**Implemented:** immutable raw/curated/derived dataset versions and exact frozen research
selections compose existing typed measurements, authored calibration, physical admission and
project temporal context. Selected detail exposes typed original members and inclusion reasons.
Research attribution is retained by original direct/study execution inputs and portable replay;
compact study/run projections are derived from those owners, not attached after execution.
Identical repeated sources deduplicate; mixed inputs remain valid with unavailable single-source
projection when a compact row cannot represent them.

**Implemented:** explicit preview/publish/reopen composes original qualified fit exports, local
source admission, guarded source activation and atomic ancestry/receipt publication. There is no
copy-floats bypass, duplicate fit report or parallel result backend. Original result protection is
held through activation; replay settles the original intent before new mutable checks.

**Implemented:** native indexed browsing and one-snapshot frozen cohorts support selected protected
Arrow/DataFusion analysis across physically compatible source revisions, with honest missing and
failed populations. Rust/Python/CLI consumers expose dataset/selection detail, publication,
workbench/cohort/result analysis and passive reconnectable observation. Bounded per-attempt
capture selects before copying, distinguishes live/current/terminal sources and does not cancel
science on exit.

### A mistake made and corrected

**Tested:** functional acceptance exposed publication preparation pruning mapped parameters that
were unused by the current target. The preparation now explicitly demands mapped paths through
the original bindings. Preview initially used a durable edit path; the existing source guard
correctly refused it. Preview now uses original local admission, and only explicit publishing
performs the guarded durable effect. The downstream fixture now contains a real parameter
consumer and uses a feasible-point route, proving the published values affect its prediction.
Authentic physical documents and their canonical dimensionless storage unit replaced invalid
synthetic fixture assumptions. No physical checks were weakened.

Independent implementation review identified progress capture starving later source sequences and
loss of compact metadata for valid repeated/mixed execution inputs; the owning capture/projection
mechanisms were corrected. Python acceptance caught assumed output type names and a nested
standalone document-header mismatch. Ordinary nested types, synchronized generated catalogues
and actual compiled signatures repaired the boundary. A final generated batch annotation used an
unresolved qualified name; removing its redundant native string annotation reuses the original
Arrow output type hint, with a rebuilt extension, regenerated stub and passing Python rerun.
Public exports now live with the other imports, and the forward physical-context annotation
matches adjacent declarations. Unknown-version tests now reject a future
version instead of the newly current one. Earlier affected 64-pass/2-fail Rust and failing Python
runs did not establish acceptance; the final results above supersede those repaired failures.

### Deviations from the plan, deliberate

**Implemented:** selected bounded polling composes original progress/history rather than adopting
native live delivery as durable history. Initial KPI aggregation supports registered relations
with complete physical meaning, including simulation samples; unsupported relations refuse.
Native cohorts allow at most 64 studies/128 occurrences and project selection at most 64 branch
owners, requiring narrower selection for larger populations rather than truncated rankings.
Calibration uses explicitly authored transformed sigma, without introducing speculative nonlinear
uncertainty propagation. External acquisition remains the selected file/batch capability.
No solver, dependency pin, process-memory policy or storage topology was changed.

Enduring contracts are at blueprint §19.4/§20.6 with proposed ADR-0218; operational entrypoints,
limits and reference-filter meanings are in the [substrate guide](../dev/surreal-substrate.md#research-versions-study-workbench-and-selected-evidence).
All three implementation worktrees were merged, conflicts preserved the already-integrated and
repaired main implementations, and clean merged worktrees/branches were removed. The supplied
untracked library reference was preserved. Keep this completed companion while 51e–51g consume
its contracts; Q04 owns aggregate retirement, existing-plan reconciliation and final publication.
ADR acceptance and full Plan51 completion are not claimed.
