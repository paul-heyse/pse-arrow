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
| J02 | in-progress | J01; working narrow A/B records/options/decode policy; narrow G02 inspector | Current report interpretation three seals original dependency commitments/omissions; source-generated one-way roots support explicit report, selection and full scopes. Independent binary readback and foreign/malformed/oversized/missing-import controls are narrowly tested. Full lifecycle/population delivery and retained raw-artifact protection remain. |
| J03 | in-progress | J02; native failed-test rerun and current listing | Native rerun consumes authenticated original input and report/inventory commitments. The representative controlled failure/export/readback/exact-rerun journey passed for the earlier selection family; interpretation-three full dependency delivery and its new journey are recorded below. Complete changed package/features/identity, absent-evidence and stale-binary controls, and every selected consumer/deletion. Historical outcomes never become present evidence. |
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

### Native one-way source and inventory commitment — 2026-11-04

**Implemented:** actual host diagnostic, action, outcome, invocation and test-identity
owners now declare their projection fields alongside their native definitions. The
complete `ErrorRecord` tree retains diagnostic children, ordinary causes, delegated
diagnostic sources, related occurrences, labels, parser context, recovery and typed
numerical/operand observations. Recovery's internal selection context remains private
and cannot be recreated by decoding its public observations. Existing diagnostic scalar
owners supply their source declarations; no diagnostic DTO or JSON bag replaces them.

`Report` and retained `CapturedEvidence` expose borrowed `ProjectionObservationRecord`
accessors with observation-only construction roles. Passive data shapes remain separately
decodable. The retained root includes complete original report/inventory bytes, original
terminal association, digest, report facts and exact selected native test identities.
No generic inverse constructs a report, captured-evidence owner, native test verdict or
physical-settlement capability.

Current native reports use interpretation 2; terminal protocol interpretation remains 1.
The report keeps the original inventory path once and commits `inventory_content`
(SHA-256 and byte extent). Discovery records the actual original private bytes; publication
reconciles those bytes before serializing the authoritative report. Live validation,
retained inventory reads and native rerun share that same observation-owned content check
and private-file/directory reader. A different same-shape inventory cannot inherit the
original report association. Missing current commitments or original inventory refuse.

Legacy interpretation 1 still supports original report/failure observation with its
original bytes and terminal. Its path-only inventory reference has no original content
commitment: inventory adoption and exact rerun explicitly report that dependency as
unavailable. A reader never substitutes its newly computed digest for a legacy commitment.
Current performers cannot publish legacy reports. This is a retained interpretation,
not a second production workflow or store.

**Tested, narrow native scope:** nine selected host unit controls passed against zero
failures using `scripts/pse test pse-host --lib` with the report-observation tests,
original-report custody validation, retained incomplete report, source-world construction
roles and private evidence reader selectors. The controls include complete recursive
failure preservation, original publication binding, same-length inventory replacement,
missing inventory, and legacy report readability without inventory eligibility. Compilation
completed without warnings. Generated transport publication/readback and the remaining
J02–J05 population obligations are still required; this checkpoint closes no package.

The subsequent retained-report root exposes original report bytes, terminal and passive
facts even when inventory adoption is unavailable. The full retained-evidence root
reports listing completeness from the actual native inventory owner; compiler actions
make no test-population promise. Both roots expose the original manager's recorded
settlement and optional native-lifetime observation separately from the helper's report
commitment. Legacy absence of the lifetime observation remains unavailable. Current
producers record `Some(observed)` and a completed native wait cannot promote original
settlement from an absent observation.

**Tested, additional narrow source scope:** three selected host controls passed for
retained-root construction, legacy lifetime absence and incomplete original report
reading. The prior source slice's existing pure-host workflow also passed, exercising
actual discovery, original inventory recording and report publication. The independent
bundle generator's focused render control passed with the pinned generators explicitly
prepared and the maintainer's unbounded environment activated before admission; this
checks candidate descriptors, validators, immutable source policy and encode-only root
source. Committed generated output, compiled independent readback and J04 retained raw
artifact protection remain separate obligations.

The first transport route exports original captured evidence through
`inspect developer-evidence --from-result … --output …`; report-only export also
supports legacy and pre-inventory failure reports. Destinations are exclusively
created. `inspect payload` consumes the generated descriptor and binary artifact,
using Buffa's maintained validator, the shared typed metadata checker and independently
generated native source coverage. The compiled reader additionally requires the complete
known descriptor content: surviving metadata cannot authorize decoding changed tags,
primitive kinds or nested shapes through a different compiled schema.

The reader reconciles the terminal's exact original report extent/digest, complete passive
report facts, and the original v2 inventory commitment and native selected population.
Finite Buffa `DecodeOptions` bound binary input, recursion and materialized elements;
an exceeded limit refuses without truncating original evidence. Manager facts remain
recorded-at-read source observations, separately attributed from the original helper
commitment. Content consistency of a portable artifact does not authenticate its producer,
grant exact rerun, reconstruct native verdicts or establish current physical settlement.
Native `byte_length` count names preserve the original JSON `bytes` spelling while
avoiding a collision with Buffa's owned-view raw-byte accessor. Strict adapter-wrapper
visibility is selected independently of unknown-field refusal.

The implementation uses Buffa 0.9.2 `Message`/`DecodeOptions`, generated
`protovalidate_buffa::Validate`, and the shared strict `support::decode_root` for passive
native children; sealed roots have only borrowed encoders. The [pinned Buffa guide](https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md)
characterizes decoding/reflection, while the [shared descriptor checker](../../crates/pse-contract-proto/src/check.rs),
[crossing implementation](../../crates/pse-contract-proto/src/support.rs) and
[native developer publisher](../../xtask/src/codegen/developer_evidence.rs) define the
actual selected policy and generated delivery. No hand-authored wire DTO or second
metadata interpretation is introduced.

**Tested, first compiled delivery:** twelve selected host controls passed against zero
failures, including the existing real selected-test workflow, source-owned report
reconciliation and retained legacy export. The real population control rejects changed
selected identities/completeness, missing inventory and a same-length inventory replacement
even with a newly calculated observer digest. A subsequent seven-control readback/export
selection passed with zero compiler warnings, covering unknown wire fields, malformed and
oversized input, missing imports, foreign families and known-metadata wire-shape drift.
The strict-visibility generator control and complete developer publisher control each
passed. Commands used the native `scripts/pse test` library/binary selectors, actual user
scopes, approved unbounded activation and cache-off mode. The first CLI journey exported
a real retained twelve-test invocation, then independently read the binary artifact through
`inspect payload --encoding binary`; generated validation and original report/inventory
association passed. This does not close J01–J05 or qualify unexercised lifecycle populations.

**Tested, representative failed-workflow journey:** the existing
`rerun_controlled_failure_probe` failed its single selected native test only under the
explicit qualification environment flag. Native failure reading retained its exact
compiler/test identity, original assertion detail and runner observation. Inventory-bound selection
export and independent binary inspection succeeded. Native rerun selection recovered the
original input, package/lib/features and exact identity, with admission still explicitly
unavailable in the intent-only preview. Fresh native execution with the deliberate failure
flag absent passed that one test, with capture and settlement complete. The original failed
report, original inventory and exported artifact digests remained unchanged. This establishes
selection/custody and a separately observed new outcome; it does not reproduce the original
environment or turn the historical failure into success. Aborted, interrupted and broader
retention/population qualification remain open.

Generic payload inspection now preserves raw facts and a typed native interpretation or
integrity refusal independently. An unavailable metadata policy remains attributable to its
original acquired pool; structurally readable current-family data with a corrupt original
report also remains inspectable. Both cases have no adopted native readback. The direct
developer-evidence integrity API still refuses, and malformed wire/input-limit failures do
not fall through to an unbounded decoder. The known-family file reader bounds materialization
before decoding through `Read::take`; Buffa applies the same selected family extent during
binary decoding. Nine focused native controls passed with zero failures and compiler warnings,
including original-pool metadata-version unavailability, corrupt original bytes and a sparse
file beyond the input limit. This is a read-only unavailable-interpretation route, not an old
production writer or a compatibility adoption path. A previously exported Proto family may
therefore require raw inspection after the compiled metadata world changes; native original
JSON report-version support is a separate interpretation promise.

The retained rerun reader now authenticates original workflow-input bytes through the
same original metadata commitment as the performing helper, replacing its weaker generic
JSON read. Parsing a same-length replacement with the same invocation/action is insufficient.
Report and portable artifact publication also share the native same-directory temporary
file, file-sync, `rustix::fs::renameat_with(RenameFlags::NOREPLACE)` and directory-sync
mechanism. An existing file or symlink is preserved, a failed writer exposes no destination
prefix, and portable output is refused above its reader's bound before writing. The
[pinned rustix API](https://docs.rs/rustix/1.1.5/rustix/fs/fn.renameat_with.html) characterizes
the no-replacement operation; the [native publication owner](../../crates/pse-host/src/invocation.rs)
composes staging and durability. Five selected native host controls passed with zero
failures and compiler warnings for original-input tampering and these publication invariants.

The subsequent J03/F03 source cutover makes report interpretation three the sole current
writer; terminal protocol one remains separate. Exclusive `Report::publish(&mut self)`
seals the actual source-enumerated dependencies once after the phase/capture cutoff and
revalidates those exact promises on publication retries. Immutable original omissions retain
their typed diagnostic or unsettled disposition. A missing, changed or expired committed
file later refuses full acquisition; it never changes the original account. The performing
owner retains the exact authenticated startup file identity before execution, including the
native publication newline, rather than re-reading and hashing input after execution.
`InputBinding` authenticates the JSON value; surrounding whitespace cannot redefine the
already accepted full-file commitment. This excludes
metadata/control requests, environment bags, executable/provider and live fixture custody.

`--scope report|selection|full` defaults to full. The existing borrowed report root carries
report or full scope; the existing inventory root carries selection scope. Full includes
exact input, inventory, raw streams, native JUnit assertion output, reached compiler/discovery
context, runner configuration and passive fixture outcomes when originally promised. The
reader checks a bijection between original manifest ordinals and included bytes, complete
extent/digest identities, original inventory population, and original input interpretation.
Original omissions remain explicit even if files appear later. Legacy reports one/two remain
readable in their supported report/selection scopes; neither acquires a historical full
commitment. The generated decoder still requires the exact compiled descriptor world and
typed source metadata; acquired bytes construct no native verdict or current readiness.

Nextest configuration producers use the existing atomic 0400 publication owner. Repeated
construction reuses only the same protected immutable content, checked through the existing
descriptor reader's whole-file digest/extent and device/inode association; changed or
unprotected content refuses without replacement or repair. After actual native Nextest exit
and required capture settlement, the producer finalizes only its declared JUnit regular file:
NOFOLLOW open, native uid/single-link/path-to-FD identity checks, FD permissions 0400 and sync,
then hashing and identity recheck. Retained readers never chmod files. These mechanisms use
the existing `artifacts::read_descriptor` and native publication owners rather than a new
storage path; the pinned rustix source above describes exclusive publication. Buffa generated
messages, `MessageField::into_option`, maintained validation and bounded `DecodeOptions`
continue to provide the wire mechanics described under Concrete library composition.

Discovery failures now retain a source-owned attempt account containing every actual
version/build/list phase reached, including a failed child or absent capture observations.
The same original attempt supplies the successful inventory's complete phase data; there
is no later directory scan or second runner. On failure, the performing report keeps the
attempt's original relative addresses, byte/capture facts and typed cause without inventing
inventory, selected population or rerun eligibility. A known nonzero child with complete
raw capture remains distinct from incomplete capture. Three focused native controls passed
with zero failures and compiler warnings for original successful-plus-failed capture,
missing child observations and report adoption of the original failed account. This closes
the lost-prior-phases source gap. The interpretation-three manifest now consumes those
same actual reached phases, retaining failed/absent child observations separately from
inventory/population availability.

**Tested, narrow interpretation-three scope:** `scripts/pse check pse-host --all-targets`
passed with zero errors/warnings. Nineteen selected host controls passed for original
commitment sealing/retry immutability, authenticated input, exact portable-file bijection,
original omissions/later file appearance, missing/mutated bytes, legacy/full-scope refusal,
FD finalization/no read-time repair and hardlink/path replacement refusal. Three additional
configuration/legacy controls passed for same-inode immutable reuse, changed/unprotected
configuration refusal and unchanged original incomplete-report meaning. Initial fixture
failures exposed nonprivate temporary directories, two stale native fixture fields, and
repeated immutable configuration construction; the fixtures/producers were corrected rather
than weakening custody guards. Full lifecycle qualification, active-reader retention and
failed-release behavior remain J04/J05 obligations; no package closes at this checkpoint.

**Tested, interpretation-three developer journey:** the existing flag-gated
`rerun_controlled_failure_probe` produced one actual failed test with original capture and
settlement complete. Default-full export included all eighteen originally committed files
(6,964,203 bytes), including native JUnit assertion evidence, with zero original omissions.
Independent binary `inspect payload` validated the complete generated family and original
report/input/inventory associations; it preserved the failed original and its complete
selected population. Native `inspect test-selection --rerun-failed` retained the exact
original package/lib/features/test identity while reporting admission/listing unavailable
at preview. Actual `test --rerun-failed`, with the failure flag absent, rebuilt/listed through
the existing owner and passed that one identity. All eighteen original commitments and
report, inventory, JUnit and portable payload digests remained unchanged. This demonstrates
the bounded failed-original/full-inspection/new-outcome journey; it does not reproduce the
original runtime environment or qualify interruption, expiry or every J05 population case.

**Tested, targeted on main:** `scripts/pse test pse-host --lib -E
'test(retained_input_requires_original_binding_before_rerun_selection) |
test(input_dependency_uses_authenticated_snapshot_instead_of_rehashing_replacement_path)'`
passed 2/2, zero failures/pending/build warnings, configured unbounded mode and cache off;
original capture and native settlement completed. Independent review found that the former
post-execution reread could accept whitespace replacement under the JSON-value binding while
misattributing full-file bytes. Startup now retains the accepted full-file identity, including
the producer newline. Both publication and retained selection refuse whitespace-only
replacement against that original identity; exact restoration preserves the sealed promise.
The reviewer resolved the defect in source; this targeted pass supplies execution evidence.

Worktree consolidation preserves the original invocation stores and full portable export
outside the retired checkouts. Checkout-local tools/build outputs are retired; original
outcomes and incomplete/unavailable custody observations are unchanged. This administrative
cleanup does not qualify J04 expiry, active-reader retention, release or interruption behavior.

- `crates/pse-host/src/test_workflow/{mod.rs,source.rs,execute/report.rs,execute/runner.rs}`;
  `selection.rs`, `test_inventory.rs`, `fixtures.rs`, `fixtures/execution.rs` and `invocation.rs`.
- [Report reconciliation](../../crates/pse-host/src/managed/workflow.rs) and
  [sealed terminal custody](../../crates/pse-host/src/managed/result.rs).
- Runtime `workflow/{completion.rs,run_outcome.rs,durable.rs}` and
  `pse-operations::{canonical_execution,canonical_results}` retain scientific counterparts.
