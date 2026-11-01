# Execution fit

**Interface-checked, 2026-11-01, source `19e7b49a67df24c1bf73005a6186e145e930d53a`.**
The [collection baseline and limits](README.md) apply throughout.
Cost, contention and failure consequences below are source reasoning, not measurements.

## <a id="da-062"></a>DA-062 — Numerical modes use different acceptance rules for the same variable bounds

**AP-04, DP-01, PS-07, PS-11. New source diagnosis.**

`pse-backend-native/src/assembled.rs` builds the ordinary variable box directly from case-plan
lower and upper bounds. `pse-backend-native/src/quality.rs` evaluates physical interval
violations against frozen per-variable tolerances; `Quality::feasible` admits a normalized
maximum no greater than one.

By contrast, the original observer in `workflow/strategies/conditional.rs` rejects any
coordinate outside the raw box. `math/initialization.rs` repeats that strict interval check
before committing a candidate. The conditional observer immediately preceding it uses
tolerances for row checks. `math/solves/causal.rs` later computes original bound quality using
tolerances, but the strict observer can refuse before that stage.

For a free continuous variable with an ordinary finite upper bound and a positive variable
budget, a finite representable `x > upper` whose positive deviation is no greater than that
budget can satisfy ordinary bound-quality acceptance while failing the strategy observer,
assuming the other rows and domains remain valid. This is a source
counterexample to one shared numerical acceptance meaning, not an executed solver failure.
A candidate's usability can depend on execution mode rather than the original tolerance.

Hard domain/sign guards and fixed specifications may correctly require strict checks.
They are separate admitted facts; this finding does not recommend relaxing them.
The known discrepancy concerns the inspected ordinary interval obligations. No numerical
fixture, convergence change or platform threshold was tested.

## <a id="da-063"></a>DA-063 — Pending study capture can block unrelated run renewal and cancellation

**AP-01, AP-07, DP-20; H23–H25. New source diagnosis.**

Authored `pse-codegen/src/codegen/surreal_study_workbench.rs::publication` consults the singleton
`canonical_study_capture_gate:current` and refuses publication whenever a capture is pending.
It has no selector-specific exception or writer-side expiry bypass.
`pse-operations/src/canonical_study_workbench.rs` opens and commits capture through separate
RPCs, so the pending gate persists between them.

That publication guard is also reached by execution `available → writer → renew` in
`surreal_execution.rs` and public study cancellation in `surreal_control.rs`.
A stalled capture for problem A can therefore delay renewal or cancellation for an unrelated
run B until capture commits or authenticated recovery clears the gate. A prolonged renewal
failure can reach `workflow/durable.rs`'s `on_loss` policy. This crosses the run-specific
separation described in blueprint §20.6.11.

There are important protections and reasons for the gate. Complete selector capture includes
future matching membership and metadata (blueprint §20.6.3); merely freezing currently selected
rows would not preserve that contract. Busy responses are retried. Commit clears the gate
before user paging. Recovery fences the capture incarnation, and a different custodian must
establish that the original native owner drained.

Those protections do not isolate unrelated liveness operations during the pending interval.
No claim is made that a normal short capture kills a run, that all writers serialize, or that
this caused an observed outage. Capture durations, retry exhaustion and failed-process
interleavings were not exercised.

## <a id="da-055"></a>DA-055 — Each durable study point repeats the generic lifecycle against shared problem keys

**AP-07, DP-10, DP-16, DP-20; H12, H14, H23. Revalidated, narrowed.**

`workflow/worker.rs` dispatches each study action with no retained continuation owner.
Authored `surreal_execution.rs` updates shared `source-creators:<problem>` protection while
performing its guarded lifecycle. Study population growth therefore repeats lifecycle
preparation and touches a shared per-problem protection key, rather than varying only the
point-specific scientific work.

Durability, independent recovery and fencing are real obligations. Separate points may require
separate settlement. The diagnosis identifies repeated fixed protocol and shared-key exposure
for later execution-fit assessment; it does not establish a throughput bottleneck or select
a batched protocol. Historical approximate transaction counts are not current evidence.
The older claim of eleven always-on result relations is corrected:
`workflow/results.rs` filters to actual table IDs.

## <a id="da-056"></a>DA-056 — Multiple retention families do not by themselves establish duplicated authority

**AP-01, AP-07, DP-01, DP-20. Prior diagnosis downgraded to an unresolved inquiry.**

`canonical/lifecycle.rs` deliberately composes distinct release effects.
`canonical_files.rs` has bounded file readers, authenticated process identity and cleanup
protections. Counting these families does not demonstrate that they re-decide the same
obligation. Lost acknowledgements and different retained resources genuinely require different
settlement facts.

This refresh did not establish a duplicated retention meaning that justifies the earlier broad
diagnosis. The question of whether a complete operation unnecessarily reconstructs protection
remains open; it needs a concrete duplicated obligation, not another family count.

## <a id="da-034"></a>DA-034 — Relation-sized file publication is a bounded execution-fit observation

The relation-change seal and above-inline reserve/seal remain in
`workflow/result_projection.rs`. Different IPC schemas can require distinct file boundaries.
The behavior is revalidated; an architectural violation or unfit supported workload is not.
This is no longer presented as an established defect solely because it follows a relation
boundary.

## <a id="da-035"></a>DA-035 — Reopen paging remains small, but its misfit is unestablished

`canonical_studies.rs` retains `LIMIT 4` full-descriptor paging for reopen and admission
reconstruction in `workflow/study.rs`. Those operations may need the descriptors.
Creation and per-point execution must not be conflated with this path.
No workload showing unfit paging, needless reconstruction or measured slowdown was established.
The remaining observation is an investigation lead.

## <a id="da-025"></a>DA-025 — Physical collection remains an explicit scope boundary

Inherited scope record, not an active violation. The 2026-10-29 account recorded logical
retirement without automatic physical collection. This refresh did not reassess that policy
or qualify its collector. Current operating scope belongs to the architecture.

## Earlier decisions and resolved identities

These dispositions are inherited, not reopened by the presence of parallel caches or protocols.
Their earlier explanations are recoverable at source `19e7b49a6` and its Git history.

| Identity | Retained lineage |
|---|---|
| <a id="da-022"></a>DA-022 | Closed by decision I16; no new unsupported-allocation evidence established |
| <a id="da-030"></a>DA-030 | Closed by decision I08–I10; parallel preparation/reuse mechanisms alone do not reopen it |
| <a id="da-042"></a>DA-042 | Previously resolved: explicit telemetry export filter and SDK loss layer |
| <a id="da-044"></a>DA-044 | Previously resolved for bounded recovery; old Python mechanism is now retired |
| <a id="da-045"></a>DA-045 | Previously resolved: closure does not retain an aggregate document vector |
| <a id="da-026"></a>DA-026 | Resolved 2026-10-25: per-row locators removed |
| <a id="da-027"></a>DA-027 | Resolved 2026-10-25: group/re-expand cycle removed |
| <a id="da-028"></a>DA-028 | Structurally resolved 2026-10-25: guarded multi-output regions |
| <a id="da-029"></a>DA-029 | Previously resolved in compiler and runtime; later formulation concerns retain DA-006 |
