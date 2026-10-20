---
title: "Plan47d: Offline review-pack publication"
status: complete
date: 2026-10-18
adrs: []
review_sources: [docs/design_review/reviews/design_review_pipeline-diagnostics-and-review-artifacts_2026-10-18.md]
doc_role: plan
doc_owner: docs/plans/47-pipeline-diagnostics-and-review-artifacts.md
doc_topics: [design, diagnostics, storage, documentation]
---

# Plan47d: Offline review-pack publication

**Implemented, selected export correction — 2026-10-19:** 47f's growth/skew assessment
removed whole-set JSON trees from table-source comparison. The publisher first selects the
requested typed source and compares it with the finalized original source; unavailable indices
and changed selected contents still refuse. Original finalized typed decoding retains its
independent finite document allowance. The table projection pool is separate and neither
allowance establishes all private peak memory. The [47f Outcome](47f-qualification-and-design-publication.md#outcome)
owns all-five-kind refusal/readback controls and actual growth measurements.

## Target and assessed foundation

Own review F01's selected offline artifact closure and independent publication, consuming 47a's frozen context/membership and 47c's shared analysis. The [coordinator](47-pipeline-diagnostics-and-review-artifacts.md) owns finding/rule/investigation decisions; this companion owns D01–D05 progress. RC01/RC02 are accepted. Earlier selected delivery and the completion-audit corrections are recorded below.

**Interface-checked:** diagnostic capture/detail already records raw files, coverage and readable prefixes. Canonical result export in `pse-runtime/src/workflow/connected_results.rs::export_ipc` preserves exact lineage, uses an Arrow **StreamWriter**, finishes/flushes/synchronizes and creates the final hard link only after success, refusing an existing destination. That is a useful protected export boundary, not an IPC FileWriter/footer or atomic multi-file pack. ReplayDocument's stronger reconstruction admission remains a distinct product. Individual scientific artifact retain/release/export operations do not themselves define the selected review closure.

The initial container is a local directory of ordinary documented files. The application owns membership/meaning; existing codecs, Arrow/Parquet and filesystem/archive libraries own generic mechanisms. No new database, object-store transaction system, permanent diagnostic archive, cryptographic attestation or runnable environment is required. Generic object_store single-object atomic writes do not imply atomic publication of all pack members.

## Work packages

| Package | Delivered capability | Prerequisites | State |
| --- | --- | --- | --- |
| D01 | Declared review selection, required context and membership completeness | Working A03/A04; RC01/RC02 contract | done |
| D02 | Streamed immutable selected raw/context/result exports | D01; existing protected exporters; A05 file documents | done |
| D03 | Readable offline analysis and useful typed selected tables | D02 source references; C01/C02/C06 analysis | done |
| D04 | Independently sealed publication and truthful interrupted output | D01–D03 completion contracts | done |
| D05 | Offline-only validation/readback of relocated packs | A02/A05; working D02–D04 | done |

<a id="d01"></a>
## D01 — Define selected scope and explanatory dependencies

Add a review-export operation at the existing runtime/workflow artifact composition boundary. It consumes exact operation/invocation/run/attempt selections, recorded window or prefix, selected analysis questions, required/optional diagnostic families/profiles, descriptive context and selected scientific result relations/fields/ranges. Both retained file input and explicit live/durable export suppliers are supported through their actual available contracts; an offline reader has no store dependency.

Emit one authoritative versioned index containing member roles/identities, relative locations, source references, recorded time/clock scope, schema/interpretation version, selection, independent availability/coverage and publication outcome. The index derives the readable report; there are not two editable definitions. Original IDs remain original IDs. Rewritten local paths are index translations, never edits to original raw observations.

Determine required explanatory closure from the selected question and products. A numerical comparison may require original inputs, effective settings, starts, requested products, norms and qualification, whereas a failed source-preparation review has different dependencies. Follow referenced source/model/property definitions far enough to preserve interpretation; an output subset does not justify omitting necessary context. Required context missing from retention stays explicitly unavailable; the exporter must not claim the selected question is self-contained merely because copying succeeded.

Keep separate: export/seal completion; membership completeness against declared selection; raw capture sampling/loss; scientific outcome; and each selected result's usability. An intentionally partial evidence selection can be fully exported and explained. A failed scientific run can have a sealed complete review of its recorded failure. An unavailable optional CPU profile can coexist with complete required context. No label implies unrecorded facts or executable reconstruction.

For I07, decide current versus retained historical attempts and child paging using A04's actual capability. Preserve known omissions and finite bounds; never silently advertise an all-study pack from the first current-attempt page. **Targeted acceptance:** complete and partial scopes, an unrequested versus failed optional member, absent required physical context, retry selection and duplicate/conflicting membership. Pure selection tests use supplied fixtures rather than starting a database.

<a id="d02"></a>
## D02 — Export selected immutable material through existing owners

When selected, export the existing engine observation reports/snapshots and applicable trace
metrics through C01's copied projection: bounded logical/physical plan and rule text, actual
operator/node/partition metrics, collection bounds/drop counts and completion/abandonment scope.
Off/Contract/Diagnostic distinctions remain explicit. Exporting these facts does not enable a
different policy, execute a query again or invent missing plan text and exclusive CPU attribution.

Copy/stream selected context, invocation/checkpoint documents, source declarations/maps, detail metadata and acknowledged readable JSONL prefixes, raw trace/perf/profile data with logs, selected analysis policy/query inputs, and exact protected scientific result exports. Keep the original evidence plus its partiality, not just a derived chart. Each member has its original source/version/selection; capture/tool outputs cannot stand in for physical/model/run context.

Capture an immutable read boundary before copying each growing artifact. Use existing acknowledged prefixes, protected result selection/fences and supplied document snapshots. Do not copy arbitrary bytes beyond a declared readable prefix or turn a moving file length into a completeness claim. Snapshot/copy failures produce an unavailable/failed member or incomplete export according to its declared role. A later live publication cannot silently alter an already selected member. No global pause of numerical work or cross-process atomicity is claimed.

Apply result field/range/member selection before avoidable decoding and retain original run/attempt/schema/terminal lineage. Stream existing IPC output or selected batches into the appropriate established exporter; do not requery each rendered table independently or hold entire Replay plus all histories/profiles/results in memory. Membership metadata may require a global pass, but duplicated per-consumer scans are unnecessary. I07 evaluates large/skewed selections at this boundary.

Treat scientific protection, source snapshots and private export file lifetimes separately. Retain only actual leases needed to finish an export; release after drain/finalization and avoid keeping native handles alive. Copy failure never alters the original science, database, capture or retained evidence. Existing result `.incomplete` behavior stays meaningful for the member; the enclosing pack has its own completion.

**Targeted acceptance:** selected results from success/failed independent steps; a readable detail prefix with an unacknowledged tail; active writer replacement; missing profile/logs; output exhaustion; cancellation; exact field/range ordering and schema/lineage; and bounded paging/streaming under more members than one page. Do not weaken current protection or ACK contracts to export faster.

<a id="d03"></a>
## D03 — Publish useful offline analysis without executing PSE

Produce a readable Markdown index/report and fully local HTML containing the selected analysis, bounded charts/tables, scope, model/run/settings orientation, source-linked facts/hypotheses/limitations and next observations. All required assets/data are local or embedded and linked relatively. A recipient must be able to understand the review without installing PSE, contacting a store, loading native solver libraries, using a CDN/symbol server or running a hosted viewer. Optional raw trace/profile tools remain useful independent choices.

Use C06's shared typed analysis result and C01's selected projection, not another renderer-owned classifier. Include selected raw observations and enough typed context to independently examine conclusions. Keep original JSON/JSONL/trace/profile authority. Default to the existing typed IPC boundary for selected tables; use Parquet where a concrete large/compressed selective-read purpose warrants it. Do not export every format/table by default. Optional CSV carries schema/unit/category interpretation and cannot be the only authority for UInt64, unavailable or nonfinite values.

Record whether an IPC member is file or stream format and whether it finalized. A partial file format is not assumed readable because stream IPC supports different prefix behavior. Keep source field/event references, analysis/query/policy versions and original endpoints alongside clipped/aggregated values. Browser rendering represents large integer identities exactly, without a lossy JavaScript Number round trip. Distinguish inclusive sums, union coverage, exclusive time, sampled CPU and unsupported critical-path claims.

Profiles need meaningful readable summaries even when the external recipient lacks optional viewers. Preserve raw data and known symbol/unwind/loss limitations; successful profile conversion is not complete offline symbolication. C05/I06 owns whether a selected symbol/trace derivative improves portability. A static report is a review product, not a promise to reproduce every interactive profiler operation offline.

**Targeted acceptance:** read report/assets with network disabled; follow a statement to original selected data/model/source references; preserve exact large integers, explicit unavailable/nonfinite and empty-event rows; display incomplete captures without a false success badge; and explain selected outcomes without executing any science.

<a id="d04"></a>
## D04 — Seal only the completed selected export

Stage each owned member privately at the requested local destination, finalize writers and wait for owned export/drain before publishing final membership and a complete seal. Use a simple local no-overwrite publication boundary. Refuse an existing complete destination rather than replacing another export; path handling must keep generated relative members within the selected pack. Native/external absolute paths remain descriptive, never executable resolution directives.

If preparation fails, cancellation arrives or an output allowance is exhausted, preserve an explicitly incomplete directory with known readable members/prefixes and failure/omission state. It must never acquire a complete seal. An analysis or optional-member failure may yield a sealed pack only if D01's declared selection contract permits that member to be explicitly unavailable; no finalizer can relabel earlier stopped capture as complete evidence. Required-context availability remains separate from filesystem publication success.

Match retry to the export operation. A new export can consume already immutable/protected source members; it must not rerun a scientific solve, restart another owner's capture or infer cleanup authority over originals. No distributed commit or crash-proof durability promise is added. Use ordinary local filesystem semantics and state their actual guarantee; add stronger durability only for a concrete later requirement. Optional archive encoding follows an already sealed directory and is not a second required pack path.

Remove any replaced ad hoc pack finalization/manual file-list logic only after its actual callers use this owner and targeted tests pass. **Targeted acceptance:** failure at member write, metadata/report generation and final publication; cancelled writer with readable prefix; existing destination; interrupted result member; optional member failure; and finalization racing a source update. Assert independent scientific outcomes and ownership are untouched.

<a id="d05"></a>
## D05 — Validate and read only supplied pack members

The pack reader loads its versioned index, validates selected identity/reference/format consistency and delegates to A02/A05/C01/C06. Supported incomplete output is readable only at its declared available scope. Unsupported versions, conflicting identities, unsafe relative paths or a missing required dependency produce explicit errors/limitations. A read must never execute a suggested next observation, invoke a factory, consult original source paths or fall back to live services.

Relocation testing uses copies and isolated fixtures: make the original fixture paths unreachable to that reader without deleting other owners' files or stopping services. Read the same selection at different local times and a different directory; assert invariant selected rows, coverage and references. Supply a tempting same-name live sibling and verify it is ignored. Unknown cross-clock alignment stays unknown on the external machine.

Test packs for failed/pre-handle work, independent usable results alongside failed products, missing optional profiler, current versus selected historical retries, managed/native children and partial hard-death prefixes. Compare derived conclusions against independently enumerated fixture events/lineage rather than using the exporter itself as its only oracle. Larger I07 controls establish that selection and streaming avoid repeated materialization; no exact benchmark threshold or environment proof is required.

## Verification and handoff

**Tested (2026-10-18):** the selected runtime/export/reader controls use explicit Arrow
force-validation and an isolated current-v6 registered canonical store. Against the zero
failure target, 14 `review_pack::tests` plus
`canonical_connected_results_reopens_declared_arrow_exact_range_and_cancellation` passed
in the selected 16-test run; the corrected
`canonical_output_indexes_read_original_rows_and_skip_unrelated_blocks` then passed its
focused rerun. All 16 selected controls are passing. Commands used
`scripts/select.py --features canonical-tests unit pse-runtime` with those explicit filters
under the owned native/exclusive route. Their actual scope includes protected original row
ranges and selected predicates, IPC schema/lineage, late failure/readable prefixes,
cancellation, borrower release, publication failure, contained-path rejection and relocation.
The ordinary shared canonical store was not migrated or modified for these checks.

The Python passive CLI-to-relocated-pack fixture passed in
`scripts.python_tests --unit-only python/pse/tests/test_review_workflows.py
python/pse/tests/test_frozen_review.py -n 0 -q` (14 selected tests, zero failures at that
checkpoint). Additional actual native journeys belong to 47e's Outcome. These results do
not establish all-context propagation, large/skewed export cost or Plan47f qualification.

<a id="outcome"></a>
## Outcome (recorded after implementation)

The [completion audit](#completion-audit--2026-10-18) qualifies the implementation claims
below. Earlier commands/results retain their original tested scope; they do not establish
complete delivery of the reopened packages. The Work packages table owns current progress.

### What was built

**Implemented:** `review_pack` owns explicit versioned selection/index/export requests,
independent publication/membership/context/capture/scientific states, bounded source-prefix
copies, protected canonical and filtered result streams, and index-last local publication.
Required-member failures, cancellation and output exhaustion preserve an incomplete directory;
optional unavailable members remain explicit. Scientific readers drain and release before
sealing. Existing destinations are refused. The reader holds contained file descriptors,
validates indexed versions/formats/identities/references and reads only declared prefixes.
Original paths and live neighbours grant no resolution authority.

**Implemented:** shared typed analysis produces local Markdown/HTML with relative source
links and exact integer text. JSON/JSONL/raw traces/profiles and registry IPC streams remain
original authorities. Human reports require neither PSE nor network access. Existing protected
result IPC export is reused rather than requerying or materializing a whole result for every
consumer. Python exposes the same publisher and a protected `StoredResult.export_review_pack`
adapter. Its relation usability stays unknown when no selected-product assessment is supplied;
whole-attempt success is not promoted to selected-product permission.

**Tested:** the controls above establish the selected publication/readback contracts. I07's
larger assembled work/live-state obligation and finding F01 remain with the coordinator/47f.

### A mistake made and corrected

A protected selected-output fixture initially applied the strict generated relation view to
portable IPC containing intentional transport metadata. It now preserves that metadata and
checks the actual Arrow coordinates, values and original source bytes. Another fixture initially
asked for whole-client drain while its original legitimate reader remained alive; pack-specific
borrower release is checked immediately, and whole-client drain follows the original reader's
natural end. Neither correction weakens production protection.

### Deviations from the plan, deliberate

The selected format is directory publication plus registry IPC streams and static reports.
No archive, additional columnar format, diagnostic database or profiler platform was needed.
Full large/skewed and all-context qualification remains the explicitly separate 47f scope.
The isolated test deployment and its data are retained quiesced and drained-stopped; no
cleanup authority over another owner's service or data was inferred.

## Completion audit — 2026-10-18

Historical assessment at audit time. Current package states and the
[corrective Outcome](47-pipeline-diagnostics-and-review-artifacts.md#corrective-outcome--2026-10-18)
supersede the remaining-work statements in this section.


**Interface-checked:** independent production-source inspection and coordinator integration
review against the package targets. The [coordinator's completion audit](47-pipeline-diagnostics-and-review-artifacts.md#implementation-completeness-audit--2026-10-18)
owns CA01–CA12, their source basis, priority and closure conditions. This assessment explains
package-table corrections; it is not another finding-disposition ledger. No product repair or
47f qualification was performed. Historical focused passes above remain valid under their
stated conditions; a passing local fixture is not complete consumer integration.

| Package | Audit assessment | Basis / remaining boundary |
| --- | --- | --- |
| D01 | Partial | CA08–CA09/CA11: public selected export, question-required explanatory closure and non-frozen reference consistency are incomplete. |
| D02 | Partial | CA07–CA08/CA10: protected exporters and prefix copying exist, but actual engine/context/result supplier composition and diagnostic IPC admission remain incomplete. |
| D03 | Partial | CA10/CA12: diagnostic table membership, meaningful retained context/table/profile orientation and clickable local HTML source links are incomplete. |
| D04 | Aligned locally | Private staging, no-overwrite, finite allowances, cancellation, readable failure prefixes and index-last sealing exist. CA09/CA11 correct the selection premises checked by this mechanism. |
| D05 | Partial | CA09–CA11: offline containment/version checks exist, but required closure, diagnostic IPC consistency and non-frozen pointer resolution remain incomplete. |

Next execution closes the cited supplying gaps and focused acceptance before composed completion
is claimed. Existing aligned mechanisms and the distinct live/scientific boundaries are retained;
missing broad qualification is not itself an implementation gap.


## Completeness repair — 2026-10-18

**Implemented:** A shared publisher composes supplied facts and explicit protected scientific selections through
actual result owners, one reader at a time. The same composition serves Runtime, StoredResult
and CLI export. Pack request/selection/index and analysis envelopes are v2. Registry diagnostic
IPC retains its own schema/source contract, independently from canonical scientific lineage.
Scientific outcomes/usability are not upgraded by copying, and failures retain incomplete output.

One question/product prerequisite owner now serves analysis and readback. It consumes actual
native, inventory, engine and typed dynamic input/settings facts, while diagnostic parents do not
acquire invented scientific runs. Root/baseline premises and actual scientific child scopes stay
independent. Both stored and recomputed prerequisite references and statement references resolve
against actual supplied supported members. Unsupported coordinates and missing required facts
stay incomplete; unrelated optional omissions do not block narrow explanations.

Static HTML and Markdown present bounded original model/input/settings, selected table values
and retained profile names/limits, with local source links. Missing optional members have no
fabricated file link. The coordinated runtime selection exercised normal/refused protected
exports, required closure, dangling pointers, typed IPC, exact UInt64 cells and relocated readback;
public CLI composition is recorded in the coordinator's [corrective Outcome](47-pipeline-diagnostics-and-review-artifacts.md#corrective-outcome--2026-10-18). 47f retains larger
assembled/export-cost qualification.
