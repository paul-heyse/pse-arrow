---
title: "Plan65h: Integrated acceptance and handoff"
status: proposed
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
Source baseline and the assumed completed-Plan64 prerequisite are recorded in the coordinator.
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
