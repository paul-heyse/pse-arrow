---
title: "Plan65h: Integrated acceptance and handoff"
status: in-progress
date: 2026-11-03
adrs: []
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md]
doc_role: plan
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [current-work, validation, performance]
---

# Integrated acceptance and handoff

## Target and evidence boundary

**Proposed:** establish that the combined [Plan65 target](65-proto-integrated-target.md)
works across declaration, generation, admission, execution, persistence and independent
inspection. This document owns assembled acceptance; companion tables own local delivery.
The coordinator records the current baseline and joint Proto-first ordering. Final acceptance
consumes the actual completed scientific population; no whole-Plan64 barrier blocks independent
contract implementation or targeted functional slices.
No current product/scientific/performance qualification is claimed by this document.

Architecture, component behavior, scientific adequacy and measured benefit are distinct
judgments. Use core 3.4, heuristics 1.0, process-simulator 1.5 and test-assessment through
the selected standard. A wire round trip using the same generated mapping on both sides
establishes correspondence under that mapping, not independent physical meaning.

## Complete journeys and discriminators

Consume early 65i/65j semantic agreement and P03's two complete working slices. H owns final
reconciliation across A–G/I/J, not the first definition of scientific product meaning or developer
evidence. A generated value is a mechanical assurance input; independent physical/format/history
expectations remain necessary. The [Buffa 0.9.2 generator](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/lib.rs)
`CodeGenConfig::generate_arbitrary` supplies bounded schema-driven construction under the
consumer's `arbitrary` feature. Retain bytes/seed, world/policy and original native failure
identity through J/G; qualify the actual custom/extern field feature closure. H must not derive
both expected scientific meaning and observed output from the same generated declaration.

| Journey | Success and revealing failure controls | Independent evidence |
| --- | --- | --- |
| Owner to independent inspector | New alternative/quantity/rule/effect yields consistent descriptors, native maps, validator, docs and client; unknown option/version and missing imports refuse rather than silently degrading | Separately specified expected identities/options and negative ingress; inspect generated artifact rather than regenerate expected output with same producer |
| Engineering operation to protected result | Inspect/edit exact-parent source, prepare, submit, observe/cancel, read selected products and close independent client | Actual local service, native operation identity, original report and protected canonical read; stale parent and changed-request duplicate controls |
| Scientific provider/portable artifact | Exact reconstructible configuration/context, values and derivative demand through direct and selected isolated route, repeated and concurrent contexts | Independent scientific source/identities and formulation checks; direct/proxy match alone cannot validate their common library |
| Diagnostics and retained history | Multi-cause/source failure retains complete evidence; bounded public detail visibly omits; old interpretation remains readable or explicitly incompatible after evolution | Independently constructed violation/omission/source cases, malformed known versus unknown details, expired protection/history controls |
| Database mapping/query/recovery | Full u64, IEEE bits, structured identities and absent/null semantics survive; graph occurrence/freshness match selected source | Native server-backed queries plus independently specified values; duplicate-object keys checked before normalization and projection rebuild failures exercised |
| Interrupted effect and selected replay | Always lose acknowledgement after commit and settle the original effect identity without duplication. Where C06 selects a continuity promise, also expire cursors and rebuild without false caught-up state | Actual effects/commit and separately faulted acknowledgement; selected replay additionally uses original cursors/ordinal and retention-gap reconstruction |
| Stream lifetime and native drain | Valid prefix followed by terminal error; stalled reader, absolute deadline, quiet numerical phase, cancel and worker death retain original outcomes and custody | Real process/thread/I/O settlement, inner transport errors and independent cleanup failure; handler drop/EOF is not evidence of drain |
| Extension locality | Add non-water capability/action, solver diagnostic and result product through their owners | Actual supported implementation/control and unknown/refusal case, source change footprint plus complete consumer checks; no required universal IR |
| Scientific operation and product use | Existing supported steady/dynamic/fitting routes retain formulation, inputs, provider demands, coverage and selected attempt; raw covariance or unselected samples remain inspectable without false scientific permission | Independently specified completion/product-use controls; spatial ControlVolume1D refuses unsupported dynamics, with an existing supported dynamic model as positive case |
| Developer evidence and exact rerun | List/check/rustdoc/test, zero match, expected-refusal assertion pass, setup/assertion failure and incomplete capture retain distinct original report/inventory meanings | Native waited exit/private report/extent/digest reconciliation; original input/report/inventory binding plus current listing; no new runner/store |
| Observation versus authority | Historical/foreign decoded observations remain inspectable without minting readiness, publication, scientific admission or native drain capability | Owner construction restrictions plus mismatched waited exit, digest, extent and historical-context attacks |
| Dependency and evidence lifetime | Diagnostic/redaction changes preserve unaffected science; datum/correlation changes invalidate affected science despite unchanged wire shape; retained reads lose dependencies truthfully | Independently varied scientific/artifact/handle dependencies; absent/expired/incomplete/incompatible/corrupt evidence; active readers and failed release |
| Descriptor semantics and ingress | Required custom options survive actual acquisition chains; wire-compatible semantic changes are detected; generated/mutated values have expected acceptance/preservation/refusal | Independently specified options and old/new worlds, legal wire reordering and unknowns; I10 simplification only after equivalent alias/trailing/extern/allocation controls |

These are required scenario families; their illustrative examples do not bound migration to
one operation. H01 resolves the actual selected producer/consumer population from A–G/I/J and
preserves all important supported distinctions. Do not restore a retired harness merely to
reuse an old command: use current Rust fixture/reference owners and their actual routes.

## Measurement and operational fit

Measure the matched complete path where a quantitative benefit is claimed: cold preparation,
warm repeated use, production/selection, conversion/encoding, transfer/decode, useful native
work, retained memory and drain. Compare identical science/outputs/physical policy and declared
concurrency, including empty/narrow/full selection, larger/skewed inputs and interrupted work.
Record external versus accounted memory distinctions and instrumentation overhead. Codec-only
measurements cannot establish whole-solve or retained-result benefit.

No speedup threshold is invented. Correct behavior with a regression requires an explicit
explanation of the purchased capability and reconsideration of the physical route. Structural
amplification can require redesign without quantitative timing evidence. Do not introduce
host caps, serial lanes or a new permanent accounting system to make a benchmark convenient.

## Package state

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| H01 | planned | Working selected A–G/I/J consumer contracts, P03 slices and actual post64 population; no dependency on I06/J05 final handoff evidence | Reconcile final acceptance population against early I01/J01 design inputs and all selected migrations; select exact native test/fixture/reference routes and independent controls without a second status ledger |
| H02 | planned | All functional A–G/I/J scope integrated; P04; H01 acceptance population | Run required service/database/native/reference, developer evidence/rerun, scientific permission/invalidation and fault/recovery journeys; retain actual failures and corrected reruns; no listing/check/zero-match/expected scientific refusal mislabeled as successful scientific execution |
| H03 | planned | H02 correctness and matched baseline/configuration | Measure selected whole-lifetime benefits and regressions; distinguish qualitative removals from Measured outcomes and explicitly defer only nonrequired measurements with triggers |
| H04 | planned | H02/H03 plus complete generated replacement population | Run scope-end hygiene and selected governance/docs/native/powerset checks; fix failures; reconcile zero baseline, enduring contracts/operator guidance and deletion/retirement obligations |

During functional implementation, compile touched packages using `scripts/pse check` and
run targeted tests through `scripts/pse test` with selected features and explicit validation
where Arrow is consumed. Run `just codegen` as part of declaration changes. Root owns
`just turn-end`; no per-package broad integration/hygiene campaign. At H02–H04 select exact
current commands and modes based on the actual changed closure, then report command, scope,
zero failure baseline, actual results and exclusions. No product tests are due for authoring.

## Consolidation qualification — 2026-11-04

The operator requested completion of current tasks, integration and removal of merged
worktrees, all Clippy findings repaired, targeted functional checks, and reconciled status.
All required worktree content is now integrated on main; only main remains. Original
invocation evidence outside removed checkouts is retained. This is a bounded qualification
checkpoint during ongoing functional execution, not completion of H01–H04 or either series.

Earlier failed Clippy attempts exposed authored-code issues, repeated generated recursive
flags/redundant fallible wrappers, and then 677 repeated borrow/dereference diagnostics in
borrowed Recipe views plus downstream authored findings per workspace mode. Repairs moved
to declaration/generator/native owners and their consumers; regenerated outputs preserve
validation, field correspondence and original failure categories. The original failed attempts
remain evidence. Fixture expectations, scientific tolerances, native deadlines and the zero
failure baseline remain unchanged.

**Tested, updated 2026-11-05:** receiving regeneration passed all fourteen targets. Receiving
`just clippy` passed both workspace/all-target modes (default and no-default), with
`-D warnings`, zero failures/warnings against the zero baseline, and complete native
settlement. Original bundle: `01a1aa4c-d24c-756b-8f22-363188021880`.
The receiving workspace/all-target changed-feature Clippy also passed with `-D warnings`:
`xtask/full-codegen` and runtime `engineering-service,native-study-tests,product-operators,
property-coolprop,solver-kinsol,solver-ipopt,solver-diffsol,solver-root-isolation,solver-highs`.
The linked native workspace/conformance/benchmark `just lint-solver-contracts` passed in its
declared native profile before the last downstream fixture repairs. Both workspace modes and
the changed-feature run precede the final test-only parsed-field assertion adjustment
(`02676ee6b`). These are actual passing scopes, not a final-source static qualification;
the operator stopped further verification before another static run. No optional-provider
powerset claim follows from these selections.

**Implemented:** the final source removes three mechanical engineering response maps in
favor of existing generated adapters, shares original native operation owners, deletes the
water-preparation wrapper and duplicate scientific fixture inclusion, and uses existing
domain selections rather than argument/tuple mirrors. Reader and scientific fixture helpers
propagate typed errors and preserve independent release/drain failures. Buffa oneof names
retain their schema vocabulary under a conditional, enum-local style exception; a new
owned/view naming regression control covers that policy. The standalone scientific child's
existing hard-exit policy is now confined to its process owner with an explicit reason;
it cannot be replaced by unwinding while an indivisible foreign call remains active. These
are narrow naming/containment exceptions, not blanket lint suppression.

**Tested, 2026-11-05:** these native correctness runs used `--profile local`, cache off,
`CARGO_INCREMENTAL=0`, the pinned toolchain, and explicit Arrow validation selected by the
native correctness route. Each passed against the zero baseline with zero test failures,
retries, build warnings or pending tests, complete original capture and settled native owners:

| Command and selected scope | Result | Original invocation |
| --- | --- | --- |
| `scripts/pse test pse-codegen pse-contract pse-contract-proto pse-ids pse-model pse-schema pse-host pse-kernels pse-backend-native pse-compiler --lib --profile local -E <selected families>`: contract/ID, generator projection, descriptor-world, RPC/report, provider, portable and numerical-policy controls; the original inventory retains the exact filter and identities | 244/244 passed | `01a1aa56-9056-700c-8aef-41caa1ce2986` |
| `scripts/pse test pse-codegen:buffa_allocation --profile local`: actual generated allocation-accounted decoders and owned/view schema naming | 2/2 passed | `01a1aa57-07ef-7668-9475-95011695a8de` |

The first component run completed 244 tests with one failed token-spelling assertion. The
generator correctly emitted a raw Rust identifier; the corrected test parses the emitted
borrowed `self` field accesses and still requires both original fields, local rules, recursive
guarding and failure propagation. The full 244-test selection then passed. Production
validation was not weakened.

**Interface-checked / not executed:** the affected runtime library built and discovered 87
selected tests using the runtime feature selection above. Execution refused first for absent
`PSE_SCIENTIFIC_INPUTS`, then for absent `PSE_NATIVE_LANE_EXECUTABLE`; neither attempt ran
tests. Scientific inputs were subsequently prepared with the existing `xtask scientific-inputs`
owner and `tests/fixtures/scientific-inputs.toml`. The required native lane belongs to `xtask`,
with `solver-kinsol`; its normal build/capture was in progress when the operator requested
cancellation on 2026-11-05. The native boundary received cancellation, returned exit 130 and
its compiler processes exited. No successful lane capture or runtime functional pass follows
from that interrupted build. Original logs and partial capture remain under
`/tmp/pse-plan65-final-qualification-mpayuhBq`; use a fresh completed lane capture on resumption.

**Next work when resumed:** prepare the original native-lane executable, supply it and the
prepared scientific-input capsule, and run the same 87-test runtime selection. It covers
engineering session/RPC ownership, result metadata and selected delivery, original error
causes, scientific worker process controls, steam and exchanger fitting. Then run the affected
reader/worker integration targets, predecessor upgrades, keeper/coordinator controls and F04
receiving controls; capture final-source service/worker images and run the actual four-phase
Study composition. Repeat static checks for changed source and refresh native readiness.
No new qualification is started at this operator-requested stopping point. New scientific
fixture failure/unwind settlement paths remain source-reviewed rather than fault-exercised.

Earlier receiving evidence retains its original scope: F04's 26 columnar/relation/runtime
controls and A03's 17 generator/host controls passed on main; 64j owns the three-phase actual
Study pass and fourth-phase source controls. Fresh final-source four-phase composition,
quantitative benefits, broader scientific/reference/performance campaigns, WASI, the optional
provider powerset, H01–H04 completion and both series' completion remain unestablished.

## Completion and enduring meaning

Series completion requires every selected actual consumer, all required failure controls,
explicit historical read/migration disposition, no replaced production path left as a shim,
and supported independent clients. It does not require adopting every library capability or
delivering remote/browser execution. Any unresolved required scientific or lifecycle guarantee
blocks that claimed support; an optional feature stays explicitly unsupported.

Move enduring contracts into their architecture/API/operator owners through accepted decisions;
preserve original scientific and measurement limits. Plan publication itself changes no ADR
status. Before retiring series documents, reconcile links and necessary evidence; Git is the
archive. Report Implemented, Tested and Measured claims separately in the coordinator Outcome,
including mistakes corrected and deliberate deviations. Do not promote focused controls to
whole-simulation qualification.
