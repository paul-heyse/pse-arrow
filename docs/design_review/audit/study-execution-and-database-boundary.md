# Study execution and database boundary

**Interface-checked. Refreshed 2026-10-29 at source `f83bac039` plus the working tree.**

This page began as a scoped supplement (2026-10-28, source `64a217de2`). It is now part of the
whole-system account and keeps the primary homes of DA-046 and DA-047. Two of its scoped premise
corrections are carried into the primary findings:
- [DA-033](boundaries-and-composition.md#da-033) is resolved: exact settlement is shared;
- [DA-035](execution-fit.md#da-035) and [DA-034](execution-fit.md#da-034) are narrowed.

Per-point execution of durable studies is [DA-055](execution-fit.md#da-055). The full 2026-10-28
text, including its research basis, is at publication `5e8aea0cc`.

<a id="da-046"></a>

## DA-046 — Metadata access requires the scientific runtime composition root

**Principles:** AP-01, AP-02, AP-06, AP-07, DP-10, DP-16, DP-17; H3, H6, H17, H27.

**Status:** inherited and corrected by the native replacement. The old path is retiring.
- The native project CLI answers the study-workbench page before building any runtime. It
  allocates a bounded response consumer and calls `control_study_workbench_page` directly, then
  returns (`crates/pse-runtime/src/bin/project/runtime.rs:213-224`). `SharedRuntime::build`
  follows only for other actions (`:225-226`).
- The Python route still constructs the scientific root first: `pse-py/src/workflow.rs:926`
  calls `runtime::acquire`. It retires with `pse-py` under 58g G03.
- Once G03 lands, this identity can be resolved on removal evidence.

**Residual.** Other native project actions build the runtime and digest the executable before
dispatch, whether or not they need scientific construction ([DA-043](development-tooling.md#da-043)).

<a id="da-047"></a>

## DA-047 — Poll and child-destruction attribution have different contracts

**Principles:** AP-02, AP-05, AP-06, DP-08, DP-21; H1, H4, H18.

**Status:** retained. Only formatting and typing commits touched the decisive file after
`64a217de2`, and no Plan58 package names it. 58c governs destruction joins, not attribution
scope.

**Structural cause.**
- `ExecutionContext::instrument_with` attaches its OpenTelemetry carrier inside a `poll_fn`
  closure only (`crates/pse-runtime/src/execution_context.rs:91-119`).
- `pse_diagnostics::observation::Scoped` installs observation only during poll, and its child
  undergoes ordinary field destruction.
- `Instrumented` enters its span on poll and on pinned destruction. `WithDispatch` installs the
  subscriber on poll without corresponding destruction scoping.
- By contrast, `canonical_client/owned.rs::ObservedBody` explicitly installs dispatch, span and
  OTel context around both polling and inner-body destruction.

**Consequence.**
- A child may execute under captured attribution but emit cleanup diagnostics under the dropping
  caller's observation, subscriber and OTel context.
- To add such a diagnostic safely, authors must understand how the wrappers compose.
- This is an observational lifecycle inconsistency. It is not an established permission,
  premature-release or drain failure.

**Counterevidence and uncertainty.**
- Synchronous `ExecutionContext::enter` restores carriers lexically.
- Request-body destruction is explicitly scoped.
- No misattributed event was reproduced.
- Whether a particular async boundary intends polling-only attribution is a review contract
  question.
