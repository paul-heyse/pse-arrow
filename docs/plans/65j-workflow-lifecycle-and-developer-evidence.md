---
title: "Plan65j: Workflow lifecycle and developer evidence"
status: proposed
date: 2026-11-03
adrs: []
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md]
doc_role: plan
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [design, diagnostics, preparation, validation]
---

# Workflow lifecycle and developer evidence

## Target and baseline

**Proposed:** make developer workflow evidence an explicit semantic population of Plan65,
with independently readable projections and complete original-evidence relationships.
Retain native correctness, Nextest, fixture, invocation and capture owners. This target
introduces no workflow runner, second verdict store or universal lifecycle enum.

The [coordinator](65-proto-integrated-target.md) owns the combined target. This companion
owns J01–J05 and integrates review [I04–I06 and I11](../plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md#I04).
Implementation starts alongside unfinished Plan64 under the operator's 2026-11-04 instruction,
using actual working prerequisites and allowing replacement of overlapping mechanisms. Inspection
at `0d9e921cf0c87b8a688a4cd96b68ca1f7f0001d5` supplies source observations, not
completed-Plan64 acceptance. Core 3.4, heuristics 1.0, process-simulator 1.5 and test-assessment
1.2 govern this design. All deliveries and benefits below remain **Proposed**.

## Existing semantic owners and foundation assessment

`pse-host::test_workflow::execute::report::{Action, Outcome, Report, GroupObservation}`
distinguishes list, test, check and rustdoc; zero match, pending and incomplete results;
admitted selection, immutable inventory, fixture requirements, phase exits and captures.
The runner owns execution reconciliation. Nextest supplies test selection/execution/reporting;
preserve its useful native reporting rather than adding an equivalent runner. Compiler
observations are distinct from test verdicts. Exact selected Nextest output/configuration
is reconciled at P01 against the pinned tool and existing native adapter; no new CLI contract
is presumed here.

`Report::publish` writes the complete private report before returning a digest-bound terminal
reference. `managed::workflow::validate_report` reconciles the terminal with actual waited
helper status and verifies original private-file identity, extent and digest. `ManagedSummary`
retains raw capture and settlement observations; `TerminalResult<T>` retains private custody.
`test_workflow::source::Scope::RerunFailed` uses original input/report/inventory, checking
failed identities against current listing. Fixture declarations distinguish target-owned,
runner-borrowed and preparation-only inputs. Readiness grants no service-control authority.

These foundations already supply useful semantics. Generate their mechanical observation
projections and consolidate duplicate readers while retaining performing-owner reconciliation.
Native JSON remains eligible as the simplest complete private report persistence format.
Uniform Proto storage would otherwise add conversions without improving attribution or rerun.

## Evidence and construction contracts

| Meaning | Required distinction |
| --- | --- |
| Intent and selection | Original invocation/action, source basis, admitted compiler context and intended selection |
| Population | Listed inventory, actual selected/executed tests and unavailable or incomplete membership |
| Prerequisites | Fixture requirements, observed readiness, performing owner and setup failure |
| Assertion result | Native test result; passing an assertion expecting scientific refusal is not successful science |
| Observation | Phase exits, raw capture completeness, missing evidence and bounded-view omissions |
| Settlement | Native/fixture/capture cleanup observations, separate from a readable report |
| Rerun basis | Original input/report/inventory identities and present-listing reconciliation |

Consume construction roles from [65a](65a-contract-foundations-and-generation.md) and
[65b](65b-representation-validation-and-interpretation.md): requests enter trusted contextual
construction; observations decode for inspection; sealed verdict, readiness, control,
publication and settlement capabilities have no generic inverse wire constructor. Historical
evidence reports its original owner's recorded facts without recreating current authority.

Generate an observational shape when a sealed native type has useful public facts. Declare
direction at the owner, without type-name heuristics or blanket deserialization of private
verdict constructors. Rerun is an explicit contextual request to the native workflow, not
replay of arbitrary commands embedded in a report. Shared machinery can expose related facts
across development and production; scientific product-use policy stays in
[65i](65i-scientific-operations-and-product-semantics.md).

## Concrete library composition

65a owner annotations generate Buffa records, descriptors, validators and one-way observation
projections. Use Buffa 0.9.2 `DecodeOptions` to select message, element-memory, recursion and
unknown-field bounds for uncontrolled binary input; the
[versioned runtime source](https://github.com/anthropics/buffa/tree/v0.9.2/buffa/src)
and [guide](https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md) characterize decoder
policy and feature requirements. These bounds govern decoding, not original-report attribution,
selection completeness or settlement. Preserve private-file/extent/digest reconciliation
before attributing evidence to a performer. Carry native JSON where it remains useful.

The existing schema/payload engine consumes Buffa `DescriptorPool`, generated vtable
`ReflectMessage::for_each_set` and `DynamicMessage::from_options` to inspect approved worlds
and typed construction-role metadata. The [reflection implementation](https://github.com/anthropics/buffa/tree/v0.9.2/buffa-descriptor/src/reflect)
characterizes pool/dynamic operations; [generated reflection](https://github.com/anthropics/buffa/tree/v0.9.2/buffa/src)
characterizes vtable access. Reflective readability does not grant native adoption. Reuse
the shared option checker and local validators rather than another field walker or dynamic
validation fallback. Inspect only the selected family; no full-corpus initialization per read.

Where a local-client boundary exists, use 65f's `ErrorDetail::from_message` and
`ConnectError::with_detail` for structured presentation, retaining `with_source` for local
Rust causes. The [ConnectRPC 0.9.1 error source](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/error.rs)
characterizes that composition. Bounded wire details disclose omissions and reference original
evidence; they do not replace complete reports. Offline reading and the native CLI suffice
for the first independent consumer. A new service needs its own concrete access benefit.

## Retained closure, recovery and disposal

Each promised developer read/rerun retains original selection/context, report, inventory,
failure/raw artifacts and required interpretation. Preserve original bytes where addressed
by digest. Bind schema/policy where independent projection needs it; a wire digest does not
replace semantic identity. Readers distinguish absent, expired, incomplete, incompatible and
corrupt evidence. A readable summary cannot imply a complete original account.

Retained evidence does not keep fixtures ready or native work alive. Active readers retain
the required artifact protection while unrelated prepared tools/workers may retire. Failed
publication, missing inventory, unresolved raw I/O and independent failed release remain
visible. Scientific completion/manifest/protection remains with its native owners and
[65c](65c-storage-projections-and-history.md), without moving host-private reports into a
second canonical store. Forwarding, history, rerun, uncertain-effect recovery and scientific
admission remain separate operations. Retire readers only after promised interpretation is
migrated, retained, or explicitly reported incompatible.

## Packages and acceptance

| ID | State | Prerequisite | Delivery and targeted acceptance |
| --- | --- | --- | --- |
| J01 | in-progress | P01; current test and scientific evidence owners | Shared passive report-observation contract and original terminal matching are implemented in source; remaining population/construction/closure agreement and working generated delivery remain. Compare list/check/test/rustdoc, expected refusal, setup failure, zero match and incomplete capture before A/B interfaces freeze. |
| J02 | planned | J01; working narrow A/B records/options/decode policy; narrow G02 inspector | Deliver complete native report/inventory publication and independent readback using Buffa pool/dynamic inspection and one-way generated observations. Preserve original file/digest checks. Test foreign/malformed/oversized/missing-dependency evidence; delete replaced observation mappers/readers. |
| J03 | planned | J02; native failed-test rerun and current listing | Bind exact rerun to original input/report/inventory and reconcile present listing. Test changed package/features/identity, absent evidence and stale binaries. Delete client-side command reconstruction; historical success never becomes present evidence. |
| J04 | planned | J02/J03; existing invocation/raw/fixture custody and interpretation policy | Integrate family retention, interruption and cleanup. Test report without inventory, unavailable old policy, corrupt raw capture, active reader and failed release. Preserve fixture lifetime, capture completeness and native settlement separately; delete replaced reference/cleanup translations. |
| J05 | planned | J02–J04 full population; narrow F02 presentation and G02 reading | Complete list/check/rustdoc/test and exact rerun across assertion failure, passing expected-refusal test, zero match, setup failure, incomplete capture and interruption. Migrate every selected reader and remove replaced maps/fixtures; supply actual limits and population to H. |

J01 supplies early design agreement. Narrow A/B implementation enables J02; G02's inspector
slice need not await all G01 host families. F01 consumes J01 design, while J05 requires only
working F02 presentation. Developer readback does not wait for all C storage or D services.
This separates design readiness from working prerequisites without a circular plan barrier.

## Verification, extension and completion

**Proposed verification:** independent expected observations challenge generated correspondence.
Owner construction controls prevent decoded observations from becoming pass/readiness/publication/
drain capabilities. Include a complete failing test population with successful cleanup, expected
scientific refusal, zero-exit helper missing required outcomes, and interrupted work whose report
is readable before settlement. Extend to a new test action or scientific product by adding its
native owner semantics and generated observation, reusing inspection/reference machinery without
merging their distinct verdicts. An added action must define its coverage and retention promises.

Compile touched packages and run targeted controls through pinned native routes during
implementation, with explicit Arrow validation where consumed. Generator changes run
`just codegen`; assembled integration/hygiene remains at functional scope end in 65h.
No tests or product campaigns were executed by this document. Completion requires every
selected producer/reader, promised retained interpretation, original rerun attribution and
deletion of replaced paths, with all required qualification reported separately.

## Native sources

### Initial native evidence delivery — 2026-11-04

**Implemented:** `execute::ReportObservation` provides independently
decodable version, original invocation/action/outcome, inventory association, captured count,
capture completeness and diagnostic facts. It contains no native `GroupObservation` or
`TerminalResult` capability. Its `validate_against` operation checks supported interpretation
and exact original terminal association while explicitly leaving byte custody, waited status
and current execution eligibility to their native owners.

Live performing-report validation and retained `test_inventory::captured_report` use that same
contract. Retained reading now rejects an internally inconsistent report even when its bytes
match the referenced digest. Inventory association and failed-test rerun reuse these facts;
their duplicate private address/version/action headers were removed. Native `Report` remains
serialize-only, and this change does not issue pass, readiness or settlement capabilities.

**Tested, narrow native scope:** four `pse-host --lib` controls passed against the zero-failure
baseline: `recorded_report_facts_are_decodable_without_native_verdict_construction`,
`retained_report_facts_require_original_identity_interpretation_and_capture`,
`original_report_requires_private_complete_unchanged_bytes_and_waited_status`, and
`authenticated_incomplete_original_report_stays_incomplete_and_changed_bytes_refuse`.
The native source-built test route reconciled four selected/completed tests, zero failures and
complete original capture and settlement. Earlier launch refusals executed no tests; the passing
route kept its actual user-systemd scope anchor alive through child settlement. This establishes
the report-fact slice, not complete developer workflow qualification. Working A/B observation
projection, independent descriptor reading, explicit retained-dependency outcomes and complete
J01–J05 population integration remain required. This checkpoint closes no package.

- `crates/pse-host/src/test_workflow/{mod.rs,source.rs,execute/report.rs,execute/runner.rs}`;
  `selection.rs`, `test_inventory.rs`, `fixtures.rs`, `fixtures/execution.rs` and `invocation.rs`.
- [Report reconciliation](../../crates/pse-host/src/managed/workflow.rs) and
  [sealed terminal custody](../../crates/pse-host/src/managed/result.rs).
- Runtime `workflow/{completion.rs,run_outcome.rs,durable.rs}` and
  `pse-operations::{canonical_execution,canonical_results}` retain scientific counterparts.
