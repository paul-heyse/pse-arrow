# Study execution and database boundary

**Interface-checked, 2026-11-01, source `19e7b49a67df24c1bf73005a6186e145e930d53a`.**
The [collection baseline and limits](README.md) apply throughout.

This page retains the primary homes of DA-046 and DA-047 from the earlier scoped inquiry.
Current durable execution-fit findings are
[DA-055](execution-fit.md#da-055) and [DA-063](execution-fit.md#da-063).
Exact settlement remains historically resolved under
[DA-033](boundaries-and-composition.md#da-033).

## <a id="da-046"></a>DA-046 — The scientific-root requirement for workbench metadata access has retired

**Prior diagnosis resolved for its known path.**

The native project CLI handles study-workbench paging before `SharedRuntime::build` and calls
the control reader directly (`pse-runtime/src/bin/project/runtime.rs`). The former Python
product path has been removed. Its composition-root requirement is therefore no longer a
current manifestation.

Other native actions can still build the root and digest the executable before dispatch.
Their fixed-work consequence belongs to [DA-043](development-tooling.md#da-043), not an inherited
claim that all metadata access constructs scientific execution.

## <a id="da-047"></a>DA-047 — Destruction attribution remains a bounded contract question

**AP-02, AP-05, AP-06, DP-08, DP-21; H1, H4, H18. Partially revalidated; diagnosis conditional.**

`pse-runtime/src/execution_context.rs::instrument_with` attaches the captured OpenTelemetry
context inside `poll_fn` while polling. The inner future undergoes ordinary destruction outside
that attachment. Synchronous `ExecutionContext::enter` restores carriers lexically.
The canonical client's `ObservedBody` explicitly scopes both poll and inner-body destruction.

These paths have different attribution contracts. If asynchronous cleanup is intended to retain
the original context, ordinary destruction can emit under the dropping caller's context.
Whether the inspected generic wrapper promises that lifecycle contract remains unresolved;
no misattributed cleanup event was reproduced.

The broader earlier statements about every subscriber/observation wrapper were not revalidated
and do not inherit this source date. This observation establishes neither premature release,
permission failure nor failed drain.

## Useful current boundary evidence

Canonical analysis publication seals the full request and closure, stages bounded products and
activates only a complete publication (`pse-operations/src/canonical_analyses.rs`).
Immutable columnar proofs retain their field premises; input replacement drops a witness unless
the input is unchanged (`pse-relations/src/columnar.rs` and `pse-engine/src/session/input.rs`).
Store recovery distinguishes exact lost-acknowledgement settlement from reissuing a different
operation. Those source-level strengths constrain any later proposed correction.

The original 2026-10-28 supplement is recoverable at publication `5e8aea0cc`.
Its later account is present at source `19e7b49a6`; neither historical baseline is silently
promoted into current runtime qualification.
