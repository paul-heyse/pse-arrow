# Local reasoning and assurance

**Interface-checked, 2026-11-01, source `19e7b49a67df24c1bf73005a6186e145e930d53a`.**
The [collection baseline and limits](README.md) apply throughout.

## <a id="da-021"></a>DA-021 — Mathematical and workflow owners remain cyclically coupled

**AP-01, AP-02, AP-06, AP-07, DP-08, DP-17. Revalidated, narrowed.**

`pse-runtime/src/math/staged.rs::StepRetention` embeds
`workflow::numerics::CandidateDecision`. Conversely `workflow/staged.rs` composes
`math::NativeSession`, `SessionDisposition` and `StepRetention`. This is a concrete policy
dependency cycle across the intended numerical/workflow owner boundary, not a diagnosis from
file size. `workflow/artifacts.rs` handles still retain the complete `Runtime`, whose major
owners are crate-visible in `workflow/mod.rs`.

Changes to scientific retention and execution can therefore require reasoning across both
owners; a narrow handle can also retain a broader capability population than its apparent
responsibility suggests. Telemetry and host dependencies remain unconditional in the runtime
manifest. Product-to-tooling dependency reach has its primary home in
[DA-020](boundaries-and-composition.md#da-020).

There is significant contrary evidence. `RetainedDataServices::new(store, BoundModel)` in
`workflow/services.rs` independently constructs a read caller with no math service,
preparation cache, source loader, execution policy or publication capability. The previous
blanket impression that retained access must construct scientific runtime state is false.
No historical line/import counts or compilation-cost claims are renewed; this finding does
not mandate a crate split.

## <a id="da-023"></a>DA-023 — Earlier architecture drift has been substantially reconciled

**Prior diagnosis retired for its inspected manifestations.**

The current architecture overview and blueprint §21.7 describe the native host and current
responsibility layout. The former Python product boundary is absent, and repository instructions
now describe generated-path protection and advisory architecture/ADR workflow context accurately.
The earlier diagnosis of pre-cutover structure presented as current is not supported by these
current owners.

This is a source/document reconciliation of the named manifestations, not verification of every
architecture sentence. The 2026-10-29 explanation remains in Git at source `19e7b49a6`.

## <a id="da-024"></a>DA-024 — The retiring Python producer premise no longer applies

**Prior diagnosis retired; optional strict qualification remains a coverage limit.**

`math/portable.rs::ExpectedProducerTarget` now declares the worker target.
The old Python consumer is gone. Native project and worker roots still pass no producer
qualification to `CanonicalDeployment::new`, but `workflow/canonical.rs` expressly permits
ordinary admission without persisted reuse for an incomplete producer. This is not independently
a violated requirement.

Strict producer/local-observation mechanisms remain library capabilities whose complete
first-party integration and platform qualification were not established by this audit.
That limit must not be turned into a fresh diagnosis that native execution lacks required
admission. The earlier Python-only reachability claim is retired.

## Assurance inspected, with its limits

Completion tests in `workflow/completion.rs` independently vary report success, coverage and
assessment permission; an injected validation failure remains unqualified despite native
success. Worker-loss assertions in `workflow/tests/worker.rs` withhold completion and reusable
seeds. Canonical fixture setup in `pse-operations/src/native_fixture.rs` and `testing.rs`
authenticates private state identity and database isolation before remote effects.
These are useful discriminating assertions for their stated boundaries.

They were read, not run. No new whole-test-account conclusion follows. Shared constructors or
setup are not automatically dependent expected results; independence depends on the particular
claim and where its expected value comes from. Test-only findings keep their primary home in
the [separate test account](../test-audit/README.md), whose older baseline is unchanged.
Full solver status mapping, derivative correctness, test-population completeness and platform
behavior remain unqualified here.

## Earlier identity

<a id="da-041"></a>**DA-041** remains historically resolved: vendored deltas have a reconstructable
patch record. This refresh did not rerun patch reproduction. Its previous evidence is recoverable
at source `19e7b49a6` and earlier source `682fc3830`.
