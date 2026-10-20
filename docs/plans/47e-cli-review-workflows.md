---
title: "Plan47e: CLI review workflows"
status: complete
date: 2026-10-18
adrs: []
review_sources: [docs/design_review/reviews/design_review_pipeline-diagnostics-and-review-artifacts_2026-10-18.md]
doc_role: plan
doc_owner: docs/plans/47-pipeline-diagnostics-and-review-artifacts.md
doc_topics: [design, diagnostics, documentation]
---

# Plan47e: CLI review workflows

## Target and assessed foundation

Deliver review F03: an operator can discover work, inspect progress, select useful future recording, analyze/compare retained evidence, follow source context, export a selected pack and review it offline through the CLI. The [coordinator](47-pipeline-diagnostics-and-review-artifacts.md) owns finding/rule decisions; this companion owns E01–E05. RC03 is accepted; the selected implementation and acceptance are recorded in the Outcome.

**Interface-checked:** `python/pse/diagnostics.py::main` exposes `list`, `inspect`, `capture`, `capture-launch` and `read-detail`; `scripts/diagnostic_trace.py` supplies three optional bounded SQL queries. `Diagnostics.collect`, static source mapping, invocation explanation/comparison, Arrow projection and protected result/context export are separate APIs. Reuse these operations and their owners. Adding flags around current live reading cannot deliver frozen semantics, and duplicating Rust interpretation in Python would create another authority.

The current `just diagnostics` route remains the user entrypoint. Extend its parser and help with explicit live/frozen subjects, retained analysis/comparison, selected review export/readback and cooperating recording. Exact leaf-command names are routine implementation choices; every exposed command must identify whether it reads, records, exports or performs new numerical work. No mandatory runner, prescriptive agent hook, diagnostic service or arbitrary command execution inside a read API is introduced.

## Work packages

| Package | Delivered capability | Prerequisites | State |
| --- | --- | --- | --- |
| E01 | Explicit live/frozen discovery and file/pack inspection | Working A02/A05; current discovery | done |
| E02 | Cooperating preconstruction detail selection and honest capture launch | Working C03; existing command/capture owners | done |
| E03 | Shared source-linked analysis/comparison and explicit diagnostic work | A03/A05; C01/C02/C06; relevant B facts | done |
| E04 | Selected context/results/pack export and offline review | Working D01–D05; E01/E03 | done |
| E05 | Joined CLI journey, errors, help and consumer migration | Working E01–E04 | done |

<a id="e01"></a>
## E01 — Discover and inspect exact live or recorded subjects

Preserve current bounded process listing, pagination/cursor and exact operation lookup. Extend inspection to admit A05's retained document set or pack and explicitly choose A02's frozen semantics. The input kind and interpretation mode are visible in human-readable and machine-readable output; a saved file is not automatically a live target. Default existing live commands retain their meaning. The explicit recorded route uses the recorded anchor unless a source-clock interval is selected.

Surface current activity, cumulative/recent facts, source freshness, retained interval, independent omissions/loss and exact related identities. Follow only supplied pack members offline. No expired parent substitution, PID-name matching, native attachment, store startup or arbitrary filesystem search is allowed to make missing evidence appear complete.

Use the shared typed result, stable source references and exact integer codec. A missing optional tool does not block raw/context/result inspection. **Targeted acceptance:** CLI fixture tests cover explicit live/frozen modes, empty bounded listings, unsupported/missing documents, unknown clock, absent related members and relocated readback. Instrument tests to reject any offline live-neighbour/store/native access. Preserve existing live-window behavior.

<a id="e02"></a>
## E02 — Select recording before the work it claims to cover

Expose family/window/sampling-or-complete selection, finite source/managed/output allowances and activation acknowledgement through a cooperating execution boundary. Reuse C03 and the actual existing case/run construction routes: selection must be installed and acknowledged before those owners construct/execute work. The CLI's generic capture-launch can profile an owned argv, but arbitrary commands do not thereby acquire selected source recording. A cooperating entrypoint must explicitly consume the request; otherwise report that capability unavailable.

Support attach-after-start as its actual suffix/sampled interval, never retroactive complete coverage. Preserve existing owned CPU/timeline capture activation and command lifetime separation. Optional capture stop/failure does not cancel the launched simulation; command cancellation/drain uses its own registered route. Complete-source requirements remain unmet when source loss, unsupported family, missing activation or writer failure occurs, even if queue drain succeeds.

Render the acknowledged selection and actual status rather than asking operators to infer it from a path or a profile file. Do not silently enable deep tracing, grow the ordinary hub or start extra numerical diagnostics. **Targeted acceptance:** cooperating launch observes early preparation and requested native family; noncooperating launch refuses that claim; late attachment exposes missing prefix; unsupported/refused collection, failed exporter and failed command preserve independent results and exact owned cleanup. Ordinary CLI reads still require no store/native service.

<a id="e03"></a>
## E03 — Ask targeted questions over retained evidence

Expose C02's source-linked explanations and retained invocation comparison without requiring an in-process `RunResult`. Accept exact documents/pack selections and explicit interpretation policy; return facts, hypotheses, limitations, evidence locations and optional next observations through C06. Support useful selected event/field and context drill-down with C01/C04, retaining whole coherent event identity and the predecessor/reset information the analysis requires.

Questions cover preparation rebuild, evaluator demand/reuse/cost, root/recycle progress, integration/replay work, fitting/study readiness, factor recovery, waiting, comparability and physical attribution. Requested output selects a useful question; it does not invent exclusive CPU or a numerical winner. Source/context references remain followable to A03's exact available mappings. Discovery-only optional tools and absent physical pullbacks stay explicit.

Distinguish passive analysis from an explicit numerical diagnostic. A next observation is a suggestion, not an automatically executed action. When an operator explicitly selects a numerical diagnostic, call the existing diagnostic operation with its original source/attempt/failure attribution, finite caller deadline, admission and independent scientific product availability. Opening a report or requesting source text performs no residual/property/derivative work. An explicitly requested static source map may expose preparation under A03's separate contract.

**Targeted acceptance:** CLI comparison from two saved invocation documents; changed products/settings/starts and incomplete coverage; same-order cache hit versus actual evaluation; unknown norm/coordinate mapping; exact UInt64/nonfinite values; no automatic follow-up. Explicit diagnostic tests assert an actual distinct operation and truthful refusal/partial product semantics, rather than a changed scientific stopping threshold.

<a id="e04"></a>
## E04 — Export and review the selected offline pack

Expose D01's selection and D02–D04's publication using exact available operation/run/attempt subjects, requested raw/context/analysis/profile members and scientific result selection. An export from live/durable sources uses those explicit existing owners; opening the produced pack uses only A02/A05/D05. The CLI must make this difference visible rather than hiding store reads inside an apparently offline command.

Report destination, declared scope, membership/required-context completeness, independent scientific/capture availability and publication outcome. Missing optional evidence can be explained in a sealed pack; failed sealing is incomplete. Preserve readable failure paths and never print success merely because a directory exists. No automatic upload or third-party sharing link is created; the operator receives a local artifact they can share.

Read/render relocated packs through the same C06 interpretation, using ordinary documented files without PSE/native execution for the human report. Optional tool queries use their actual availability and bounded lifetime; failure leaves raw members readable. **Targeted acceptance:** complete failed-run pack, selected usable independent result, unavailable reconstruction capability, missing optional CPU profile, absent required context, existing destination, output exhaustion and interrupted export. Machine output distinguishes each state without conflating it with scientific success.

<a id="e05"></a>
## E05 — Complete the operator journey and remove replaced glue

Exercise a CLI-only short route: discover early work → install acknowledged relevant recording through a cooperating route → inspect selected recent progress → analyze/compare retained documents → follow model/source context → export selected results/evidence → open the relocated offline pack. Use existing small analytic/process fixtures and actual APIs. A successful helper-only test does not establish the CLI composition.

Add help/examples to the existing diagnostic guide as commands land, identifying input scope, collection coverage, optional dependencies and whether a command performs new work. Reuse existing environment/native capability declarations; do not add implicit package installation or full wheel builds. Migrate the recurring case-observation/manual review glue only where the new owned operation truly replaces it; keep distinct legitimate case-running and scientific APIs. Delete duplicate classifiers, live-clock frozen-reading helpers and fixtures solely for replaced mechanisms once callers move and targeted tests pass.

Tests include opt-out/disabled/refused collection, missing profile/query tool, no native handle, failed pre-handle work, managed child loss, partial history and explicit cancellation/drain. Human-readable wording may evolve; assert typed meaning and actual source linkage, not a fixed command count or exact prose. Baseline failures remain zero.

## Verification and handoff

**Tested (2026-10-18), zero failure target:** fast editable native dev correctness build and
actual-API stub generation passed. `scripts.python_tests --unit-only
python/pse/tests/test_review_workflows.py python/pse/tests/test_frozen_review.py -n 0 -q`
passed 14 tests; the additional persistent
`test_query_table_cli_writes_registry_typed_ipc_stream` passed its focused rerun (1 test).
The latter checks real PyArrow IPC readback and exact UInt64 above 2^53. Three selected
existing diagnostic controls passed for opt-out, complete/refused source detail and passive
reads; see 47c. `python -m unittest scripts.tests.test_case_audit -q` passed 29 tests before
the final selector correction, whose focused `SourceRecordingProtocolTests` then passed 3.
All executed unit selections have passing current affected acceptance, not a broad suite claim.

**Tested:** a temporary bounded harness exercised literal CLI `record-launch` against the
force-validated native benchmark in once-only `--test` mode, followed by CLI recorded
inspection, ten-question analysis, root saved-invocation self-comparison, registry detail IPC
query, selected export, relocation, offline review and identical reanalysis. Its actual final
cases were:

| Case / explicit selection | Actual selected source | Result |
| --- | --- | --- |
| `compact-idas-dae-5`, ordinary 0.001 scientific policy | `dynamic_segment_summary.v1` 6 events; `dynamic_output.v1` 4 events | Science usable; all 10 events acknowledged, zero source/metadata loss; 290 IPC rows; sealed selected pack; relocated analysis identical |
| `compact-root-diagonal-4`, same ordinary policy, explicit 128 MiB source memory / 16 MiB output | `root_evaluation.v1` 274; `native_owner_return.v1` 4 | Science usable; all 278 events acknowledged, zero loss; two raw pages; selected first-page projection 2,042 rows; saved-invocation self-comparison and relocated analysis invariant |

The original root 16 MiB memory capture remains incomplete with 103 acknowledged events,
175 dropped events and 145 metadata losses. Its entire offline journey also passed with that
incompleteness visible. An initial stale-family IDAS selection remained unobserved rather than
becoming false zero work. Original artifacts and the 16 MiB projection refusal are retained;
the successful passive projection used 64 MiB and did not rerun science. All command groups,
fixture cleanup and native joins drained. Each final scientific control ran one iteration;
no elapsed speedup, precise performance target or long campaign is claimed.

These native packs select diagnostic facts/raw prefixes. The benchmark did not retain a
ReviewContextDocument, and the dynamic benchmark did not retain InvocationEvidenceDocument;
those omissions are explicit. These journeys do not establish full original model/context or
scientific-result export for every context. Protected result/context contracts are exercised by
47a/47d's selected controls; broader assembled coverage remains 47f.

<a id="outcome"></a>
## Outcome (recorded after implementation)

The [completion audit](#completion-audit--2026-10-18) qualifies the implementation claims
below. Earlier commands/results retain their original tested scope; they do not establish
complete delivery of the reopened packages. The Work packages table owns current progress.

### What was built

**Implemented:** the existing diagnostics CLI exposes recorded inspection, bounded shared
analysis/comparison, selected context/inventory, contextual trace query, typed registry table
query, cooperating record-launch, selected pack export and offline review. Typed versioned
requests/results are the machine contract; human output projects the same Rust interpretation.
Offline subjects resolve indexed supplied members only. Suggestions never start science,
install a tool, enable tracing or discover missing model sources.

**Implemented:** explicit `diagnose` / `Runtime.diagnose_documents` is separate original-source
work. One clock begins before physical/model admission and preparation, with existing solve
settings, finite source extent and diagnostic controls, independent operation identity and
original failure/run attribution. Success, pre-handle refusal and deadline expiry were tested;
unknown physical mappings remain unknown. Caller activity/native supervisors retain admitted
work through actual drain after cancellation; passive review does not reconstruct or evaluate.

**Implemented:** record-launch owns literal argv, separate ACK/command clocks, finite log
prefixes and exact group cleanup. An exclusive per-ACK claim prevents two commands consuming
one selection. Malformed/missing source ACK does not stop science. Logger/ACK failures remain
separate and prefix counts reflect bytes actually retained. Declared query subprocesses share
a single finite clock across actual-version discovery and execution, terminate owned descendants,
and preserve failure output. Explicit trace-query uses the declared interpreter/native route.

**Implemented:** protected `StoredResult.export_review_pack` composes the shared publisher and
actual selected reader under signal-aware owned export/join. Shared schemas/contracts/stubs were
generated from their declarations. The diagnostic guide documents scopes, formats, limits and
new-work boundaries. The old bare trace-query/collector glue is removed; its native-dependent
controls moved into the Python suite, preserving isolation of setup controls.

### A mistake made and corrected

Independent source review caught the native Arrow RecordBatch wrapper being passed directly to a
PyArrow IPC writer. The boundary now uses `pa.record_batch`, matching existing projections;
the persistent IPC regression and actual native detail journeys pass. Launcher review also
caught claim cleanup starting after a fallible write, buffered log counts exceeding retained
bytes, and descendants surviving a normally exited leader. Cleanup ownership, final extents
and group drain now cover those paths; injected setup/flush failures are tested.

The first native benchmark compile exposed a stale implicit residual-body caller. It now
provides the current value-only zero-coordinate source and first-order unknown-axis product,
without an eager full second-order parameter product. No scientific formula/default changed.

### Deviations from the plan, deliberate

Execution remains short and question-led. One explicit larger finite recording/projection
allowance settled complete-source transport; ordinary defaults were not enlarged and original
failures were retained. No full wheel install, profiler repair, new platform, accuracy tightening
or environment/provenance proof was introduced. Plan47f still owns all-context assembled
qualification, observer/export fit, final hygiene/Clippy/manual checks and overall closure.

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
| E01 | Aligned | Bounded live discovery remains separate from supplied file/pack recorded inspection. |
| E02 | Aligned | Literal cooperating launch, preconstruction request/ACK, independent command/capture outcomes and owned cleanup have actual paths. |
| E03 | Integration incomplete | CLI delegates to shared analysis/comparison and explicit original-source diagnostic work. CA04–CA07 block promised composed answers/suppliers; the local delegation and separate-work boundary need no replacement. |
| E04 | Partial | CA08/CA12 plus D01–D05 prerequisites: generic frozen export exists, but original live/durable context and protected selected scientific export are not composed into CLI review. |
| E05 | Partial | CA08/CA12: help and commands exist, but the joined CLI-only journey lacks original context acquisition, selected scientific rows and the promised usable local report. |

Next execution closes the cited supplying gaps and focused acceptance before composed completion
is claimed. Existing aligned mechanisms and the distinct live/scientific boundaries are retained;
missing broad qualification is not itself an implementation gap.


## Completeness repair — 2026-10-18

**Implemented:** Recorded input remains the default. Explicit `--source live` reads the selected checkpoint;
`--source handoff` reads an actual retained receipt. `--handoff` joins original descriptive facts
without replacing the selected primary snapshot. `--memory-bytes` selects finite analysis
admission. Protected scientific selectors require an explicit existing `ReviewExportDeployment`;
supplied-only export opens no runtime/deployment. Both routes call the shared publication owner,
including multiple original field/range/output selections and typed diagnostic tables.

Public tests exercise optional joined/pre-handle handoff, relocation, unchanged invocation and
scientific result, entered collection preservation, explicit live-plus-handoff selection, shared
analysis equivalence and protected multi-selection through CLI export. The coordinator's
corrective Outcome owns actual results and the boundary before 47f's all-context assembled
qualification. No full wheel campaign, new profiler platform, accuracy change or environment
proof work is introduced.
