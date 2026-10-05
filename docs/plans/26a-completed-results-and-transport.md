---
title: "26a: Completed results and transport"
status: done
date: 2026-10-05
adrs: [ADR-0092, ADR-0145]
review_sources: ["../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f08"]
scenario_sources: ["../design_review/reviews/design_review_testing-architecture_2026-10-05.md#revealing-changes"]
---

# 26a: Completed results and transport

## Purpose and foundation assessment

Seal completed trajectory meaning and make its transport reuse safe and production-owned.
The [coordinator](26-testing-architecture.md) owns F08 disposition, shared decisions and final
acceptance. This companion owns A1–A3 progress. The baseline and evidence limits are those
of the coordinator. This section records the **Proposed** target and foundation assessment
at authoring; the Outcome records the implementation and scoped acceptance.

At authoring, `ModelingTrajectory` exposed mutable identity, native report, preparation, checks,
reports, coverage, acceptance and validation-error fields beside a retained private completion.
Cloning deep-copies several of those allocations while sharing the native lease. Its
diagnostic and exported assessment can consequently disagree after a legal Rust mutation.
The constructor's initial agreement is insufficient as a lifetime contract.

At authoring, direct transport also derived qualification from satisfied checks while supervised completion
uses accepted permission. The two meanings must be reconciled at the completion owner, not
patched separately in Python and Arrow encoders. Physical feasibility, native termination,
coverage and permission remain distinct facts.

The generated `Collection` encoder and checked-batch leases are suitable foundations. Keep
their physical conversions, relation admission and escaped allocation ownership. Existing
`RunResult::tables` already encodes once and retains its result, including encoding failures;
do not add another cache at that boundary. Relevant authorities are blueprint §13.5/§13.6,
§19.2 and §21.

## Selected target and interfaces

### One immutable completed snapshot

Keep `ModelingTrajectory` as the public result handle, backed by one clone-shared immutable
snapshot. Private construction owns run identity, native report, preparation, checks, reports,
coverage, validation diagnostic, `numerics::Completed`, final computation header, allocation
lease and transport state. A clone shares this snapshot; it does not clone report/preparation
vectors or create a new permission projection.

Remove public writable fields and the stored `accepted` boolean. Provide read-only accessors
for identities and borrowed report/preparation/check/report slices. `accepted()` derives
`Completed::permits_use()`. `diagnostic()` consults that same permission and presents retained
typed causes; it cannot be suppressed by overwriting a projected flag. Do not expose a mutable
reference or an owned public `Arc<Report>` enabling changes beneath completed identity.

Changed evidence or report meaning requires a different completed product through the owned
operation. Do not add mutation/recompletion setters for a finished result. Existing callers
move directly to accessors; no compatibility fields or second result path survive.

### Completion owns the final header

Extract successful Simulation header projection into the existing completion owner. Construct
it once with the completed trajectory. `RunResult::capture_completion` consumes that product,
rather than deriving a second successful Simulation header. Rejected executions with no
trajectory still use the existing completion owner.

For Shooting, the private projection operation consumes the coherent joined shooting report,
its retained lease and the enclosing completed Shooting header. It retains the already
composed checks and permission; it does not independently accept arbitrary `SampleChecks`
plus a completion. The header keeps the optimizer backend/profile, native termination,
candidate facts and composed qualification. A dynamics header with its kind changed to
Shooting is not equivalent.

Bind final kind and header at private construction. Remove the independently selectable
`tables_for_kind` export. The current inconsistent-pieces shooting projection test moves to
a coherent refused joined report, retaining its diagnostic and lease expectations.

### Lazy successful whole-result materialization

Select a result-owned, synchronized optional shared map of checked batches. The first export
request serializes materialization, uses the existing whole-collection encoder and publishes
the map only after complete success. Subsequent reads, including reads through cloned result
handles, share those checked batches and buffers. Construction without export does no encoding.

The runtime `tables()` operation returns a shared immutable map; `table(name)` resolves the
relation and clones its checked batch. Unknown names and relations absent from this result
are typed errors. The Python native handle calls this production `table` operation, not
`tables()` followed by removal from a newly constructed map. The facade retains its current
table-returning role and ownership semantics.

Reserve metadata before growing retained maps. Native report memory stays under its retained
lease; Arrow buffers stay under their existing columnar ownership. Sharing maps/batches must
not recharge their buffers. Abandoned construction releases temporary reservations. Concurrency
and poison/failure handling use the existing typed workflow error conventions; no panic escapes
as a successful result or silently starts a numerical attempt.

A direct export error leaves the optional map empty and releases partial allocations. Later
direct access can retry encoding after budget pressure clears. This never reruns integration,
assessment or completion. Preserve `RunResult`'s existing outer sticky encoding-error contract:
direct retry does not promise recovery through a `RunResult` that already retained an error.

This choice reuses the existing encoder and blueprint §19.2 encode-once responsibility without
a second relation dispatcher. Its deliberate cost is that first access materializes every
trajectory relation, even if only one is requested. Complete-map budget refusal is a transport
error, not a scientific refusal. Reopen requested-relation encoding only if selective export
under smaller budgets becomes a required capability or representative measurements establish
consequential unused retention. Speed and memory improvements remain unmeasured.

## Packages and migration

| Package | Prerequisite and delivered behavior | Retirement and acceptance | Status |
|---|---|---|---|
| <a id="a1"></a>A1 Immutable completion | Existing completion/report owners; coordinator P0 if a governed boundary changes. Seal snapshot, derive accessors, capture final Simulation header/coherent Shooting projection and migrate every affected field/header consumer | Remove writable fields/duplicate acceptance. Accepted, refused, partial and terminal-prefix projections agree; clone shares snapshot/lease | implemented; assembled acceptance complete |
| <a id="a2"></a>A2 Bounded shared transport | A1 working immutable products and kind-correct headers. Implement success-only synchronized lazy map/runtime `table` and migrate Rust/Python export callers | Remove caller-selected kind and repeated whole-map construction. Concurrent first reads encode once; retry follows the specified owner contract | implemented; assembled acceptance complete |
| A3 Migration audit and boundary acceptance | A1/A2 integrated interfaces and callers. Confirm complete migration and retained independent journeys | No replaced API/helper remains; boundary/lifetime/scientific controls exercise all migrated meanings | implemented; assembled acceptance complete |

Production editing owners are `workflow/modeling/dynamics.rs`, its trajectory encoder and
the existing completion/numerics/simulation-result owners in `pse-runtime`; Python native
adaptation lives in `pse-py/src/workflow/modeling.rs`. Migration includes both trajectory
constructors, `run()` cloning, supervised Simulation/Shooting encoding, numerical completion,
durable termination projection, modeling conformance, conditional integrated journeys, study
consumers and dynamic optimization acceptance. Follow actual field users across the tree;
the five-table Python journey is representative, not the migration boundary.

Do not alter numerical iteration, tolerances, trajectories, assessment applicability or
partial-outcome policy to simplify transport. Keep analytic trajectory/integral, event-prefix,
original closure and interrupted-DAE evidence. No second solving or validation path is added.

## Verification

The checks below were **Proposed** at authoring; executed evidence is recorded in the Outcome. Compile with `just check-package pse-runtime` and
`just check-package pse-py` as applicable. Isolated completion/materialization controls use
`just unit-package pse-runtime <filter>` with explicit correctness force-validation supplied
by the recipe. Actual solver controls use `just unit-native-package` and the relevant explicit
feature/filter arguments; linked Python uses the existing capped native recipes.

Add focused owner tests for:

- Read-only completion and agreement among accepted, diagnostic, header and candidate assessment
  for acceptance, refusal, incomplete evidence, interruption and declared terminal prefix.
- Clone-shared snapshot and successful materialization; concurrent requests publish one map,
  later accesses share storage, and distinct attempts never share even when supplied IDs match.
- Actual Simulation/Shooting header facts, including optimizer profile and refused completion;
  no independent qualification is calculated by the encoder.
- Direct budget refusal, temporary-allocation release and later export retry without another
  numerical operation; retained outer `RunResult` errors preserve their existing behavior.
- Returned batches staying valid and charged after source, trajectory and runtime handles drop;
  last ownership releases reservations.

After `just py-sync-native`, run the existing native Python event/check/terminal-report table
journey in `test_modeling_kernel.py` and its substantive lifetime assertions. Fix its current
category to reflect actual effects through 26c, rather than treating its `unit` label as truth.
Keep the shooting projection diagnostic/lease control, migrated to a coherent completed report.
Final assembled/native/Python and static evidence belongs to coordinator Q1, not another
full campaign in this companion.

## Outcome

### What was built

**Implemented**, 2026-10-05: trajectories retain one private immutable, clone-shared completion
snapshot, final header and assessment. Simulation and Shooting exports consume those decisions.
A synchronized, success-only table map retains bounded allocations; concurrent callers share
its storage. Direct budget refusal can retry without solving again, while an enclosing
`RunResult` preserves its existing sticky transport error. Checked Arrow export retains the
source, container and wrapper leases through the last escaped array without copying payloads.
All affected Rust and Python callers use the completed result and fallible export interfaces.

**Tested**, zero failure baseline: `just check-package pse-runtime` and
`just check-package pse-py` compile the migrated consumers. Coordinator Q1's linked
`just native-test --profile ci -E <Plan 26 owner selection>` passes the completion,
Simulation/Shooting, concurrent map, same-ID distinct-attempt, assessment/header agreement,
budget retry, sticky error and escaped-owner controls. That selection also retains analytic
terminal-integral, interrupted-DAE and Shooting/simultaneous optimization comparisons.
The complete assembled receipt, including its unrelated registry repair, belongs to Q1.
Assembled acceptance is complete. Plan 26's independent implementation conformance review
and accepted remedy follow-up retain the transport judgment; the coordinator Outcome owns
the final scoped evidence and its limits.

### A mistake made and corrected

An early transport implementation left final Shooting endpoint assessment available for
recalculation during projection. The completed Shooting report now retains that assessment;
the encoder borrows it, and numeric retention accounting includes missing-observation storage.

### Deviations from the plan, deliberate

The first successful access still materializes the complete trajectory table map, as selected
in the design. No selective-encoding hierarchy or measured speed/memory claim is introduced.
Wholly bufferless Arrow exports are explicitly refused because no payload buffer could retain
the owner; current registry result relations have retainable buffers.
