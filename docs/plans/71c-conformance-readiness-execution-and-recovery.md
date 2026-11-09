---
title: "Plan71c: Conformance readiness, execution, and recovery"
status: draft
date: 2026-11-09
adrs: []
doc_role: plan
doc_owner: docs/plans/71-workflow-and-simulation-test-feedback.md
doc_topics: [current-work, diagnostics, preparation, validation]
review_sources: [docs/functional_reviews/workflow-simulation-test-execution-and-diagnostics-2026-11-09.md]
---

# Conformance readiness, execution, and recovery

The [coordinator](71-workflow-and-simulation-test-feedback.md#opportunity-dispositions)
owns O02 and O03 disposition. This companion changes the standalone developer campaign,
not the required scientific population. The [current driver](../../xtask/src/main.rs)
combines a Cargo-built native tool, campaign selection, runtime construction, completion
and optional report export. [Campaign](../../xtask/src/development/conformance.rs)
already owns admitted manifest/run selection, authored roots and provider checks; the
runtime already supports bounded case scheduling and truthful partial results after
cancellation. Extend those owners rather than introducing a new campaign runner.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| [C01](#c01) | Planned | Existing campaign and prepared-tool admission | One manifest owner backs passive readiness, explicit preparation and current or historical execution. |
| [C02](#c02) | Planned | A01; admitted selection from C01 | Original case progress and findings are bounded, recoverable after published checkpoints and readable through the CLI. |
| [C03](#c03) | Planned | C02 producer and CLI route | Cooperative interruption, independent settlement and deliberate export are usable without obscuring the original outcome. |

<a id="c01"></a>
## C01 — Assess the selected campaign before constructing its runtime

Route the target `pse conformance readiness`, `prepare` and `run` actions to the
same admitted campaign interpretation. Readiness takes the original manifest,
optional run, selected case identities and provider profile. Expose declared
requirements, missing roots, selection refusal, missing or mismatched prepared
capabilities, and what still needs runtime admission. Assess source-visible facts
without executing Cargo, starting providers, constructing `SharedRuntime` or
creating a report directory. If exact assessment requires an unprepared owner,
return that state with the explicit preparation action; no second manifest parser.
The execution owner checks that the original selection is still valid before effects.

The current [recipe](../../justfile) couples `cargo-pse-correctness run` to
`xtask conformance`. [Its features](../../xtask/Cargo.toml) distinguish
`native-solvers,force-validate` from `property-coolprop`; the runtime selector
`--provider coolprop8-water` is not itself a build capability. Preserve automatic
class-specific solver selection and report provider or physical envelope refusal
without broadening what an existing provider claims. Reuse the native prepared-tool
and image owner for explicit preparation and execution. Bind executable, feature,
toolchain, generated and build-script inputs, native ABI/loader and required
helper context; keep the force-validation claim. Model-only edits may reuse a still
valid image when the model is read at execution. Re-admit the changed authored
bytes and perform fresh science. Rust, feature, provider linkage and other build
inputs must re-enter the relevant freshness or preparation decision. A historical
image can be selected explicitly as historical evidence; a byte digest alone
cannot certify current source.

Migrate the managed recipe and direct developer guidance to the shared admitted
route; recipe shortcuts can delegate to it. Retire the replaced orchestration and
automatic fixed scratch output where the new route owns them. Exercise unknown run,
missing authored input and unprepared provider without effects, model-only reuse with
fresh scientific execution, and a build-affecting input that must invalidate reuse.

<a id="c02"></a>
## C02 — Read original case progress before export

Use the existing [runtime observation boundary](../dev/simulation-diagnostics.md)
and the admitted campaign identity. Publish selected run and case identities,
last useful phase, first actionable finding, pending or unresolved effect, and
coverage. Distinguish held-required input, refusal, unattempted, interrupted,
completed and unobserved; do not infer a required population from the rows that
happened to run. Bind run IDs and operation references to the original invocation,
as in [A01](71a-cli-inspection-and-evidence-lifecycle.md#a01). Carry timestamps,
staleness and omitted records. Use bounded coalesced checkpoint publication,
not a complete event log, repeated full table materialization, a product database
or a test-only phase probe. Publish the admitted selection before execution and
meaningful progress from actual owners while it proceeds.

Use the compact conformance budget and owner expiry in
[A03](71a-cli-inspection-and-evidence-lifecycle.md#a03). Separate a checkpoint
from an acknowledged terminal and protected scientific evidence. If the child
dies before publishing a checkpoint, expose only the observed original exit and
available selection. If it dies after a checkpoint, retain that last state without
inventing further progress, rollback, cleanup or a result. Read either path via
`pse inspect invocation ID` and its paged case view. Verify an abrupt exit before
export both with and without a published checkpoint; also verify omission at the
cap and that the optional diagnostic cap does not change the scientific result.

<a id="c03"></a>
## C03 — Cancel cooperatively, settle issued work and export deliberately

Reuse the native [signal-to-cancel pattern](../../crates/pse-runtime/src/bin/support/signals.rs)
to connect SIGINT and SIGTERM to the existing runtime cancellation source.
Stop new admission, let issued work settle, and expose the returned incomplete
report with a nonzero run result. The managed supervisor owns process exit; the
native runtime and fixture own drain. Preserve an original finding even when
cleanup also fails. Expose separate scientific result, capture coverage,
export disposition and settlement; inspect or cancel does not reissue an
uncertain staged operation. Durable resume and inferred rollback are outside scope.

Keep `RunReport::export` as the deliberate full scientific IPC route with
fresh no-clobber destinations. It can publish each complete relation and still
fail before the whole set is available; report precisely which output exists
under its owner. Default inspection must not emit the six Arrow files or read
them back just to summarize the run. The compact projection stays available if
cleanup or export fails. An explicit caller-owned export has a receipt and is
outside managed expiry. Validate a signal during active native work, retained
cancelled and unattempted statuses, failed cleanup after a finding, and an
export collision preserving prior bytes. Use selected deterministic controls;
do not gate the new behavior on a whole reference campaign.
