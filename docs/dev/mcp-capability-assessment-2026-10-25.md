---
title: MCP capability acceptance assessment
date: 2026-10-25
status: complete
doc_role: evidence
doc_owner: docs/dev/capability-interface.md
doc_topics: [validation, diagnostics, environment]
doc_provenance: observed
doc_retention: while-dependent
---

# MCP capability acceptance assessment — 2026-10-25


## Correction outcome — 2026-10-25

**The selected correction scope passed. Tested:** all eight selected fresh Codex workflows
passed, the complete managed setup run passed **2,426 tests**, and fresh coverage capture
plus its canonical MCP reader passed. Combined original and correction evidence now gives
**94/94 callable capabilities at least one independently checked positive path**. This is
not a fresh exhaustive semantic rerun of all 94 bindings or whole-platform qualification.

**Implemented:** coverage argument composition, runtime currency classification, persistent
service telemetry selection, collector settlement checks, replay preflight ordering, async
setup discovery, descriptor-backed test logging, and the missing frames producer relationship
were corrected. The later instruction to convert historical runs and delete their worktree
superseded the earlier retention-only choice: all seven related report resources were
converted to current physical custody, and the sole historical worktree and branch were
removed. Original bytes, outcomes and provenance meanings were preserved.

The original assessment below remains historical evidence. Its failed attempts, 93/94 count,
unrun fallback and retained-worktree statements describe the state before these corrections.
Current results and disposition are recorded here rather than rewriting those observations.

### Corrections and disposition

| Finding or gap | Current result | Independent acceptance |
| --- | --- | --- |
| MCP-A01: coverage producer | Corrected. LLVM-only reporting options are excluded only from scope interpretation and retained in executed argv; unknown/malformed scope options still refuse. | Real fresh capture: one signed-zero test passed, 53 unselected. CLI-to-STDIO reader returned complete, fresh, observed data with that exact population. Git base HEAD has no changed-source regions; no changed-region coverage or assertion-adequacy claim follows. |
| MCP-A02: runtime currency | Corrected for the six named observation/output selectors through the existing shared identity owner. Unknown compiler/environment inputs and actual artifact witnesses remain conservative. | Joined native telemetry/MCP passed under all six changed selectors **without `--prepare`**, including source admission, the exact two-worker metric/trace joins and usable native producer outcome. |
| Continuous private telemetry | Corrected. Service setup records its selected telemetry path; child configuration and launch identity consume it. Legacy selection migration is offline, bounded and restricted to dedicated original ownership. | Continuous F02 and its finite comparison each passed two independent 11-span parent chains. A separate finite-only journey also passed. Shared-allocation refusal, dead-caller release and held-ledger expiry controls preserve unrelated work and original clocks. |
| Collector-down fallback | Qualified. The fixture waits for the authenticated original incarnation and cgroup to settle, accepting inactive or failed terminal unit state; unknown/replacement identities refuse. | Actual collector stop followed by MCP health/configuration reads passed: collector unavailable, configuration readable. |
| MCP-A03: replay/custody | Future known preflight refusals create no output, report resource or persistent consumer reference. Existing report disposal guards remain unchanged. Historical evidence moved through the owner-controlled converter. | Replay refusal/success regressions passed; 92 resource controls passed, including creation/publication interruption, read-only recovery, mixed-batch refusal, same-process timeout retry and protected old/current locators. Seven real reports were verified before and after old-worktree removal. |
| Async test discovery and XML runner | Corrected. One unittest discovery pass includes async-only cases once with requirements intact; real subprocess controls own descriptor-backed stderr. | Both formerly failing XML cases passed; final complete setup run: 2,426 passed, zero failures/errors/skips, 282.070 seconds. |
| Frames producer lookup | Corrected with the actual `pse_ids::Frame` type, Rust owner and markdown emitter in the existing relationship representation. | Generator unit passed; workspace-source module 16/16 passed; real STDIO and fresh Codex exact-path lookup returned the declared relationship. Regeneration and all six selected comparison leaves passed through the original owners. |

No dependency pins, scientific wire schemas or MCP catalogue counts changed: 11 bootstrap
tools, 14 domains, eight workflows and 141 entries remain. The implementation uses the
existing owners and mechanisms; the supplied [FastMCP reference](../libraries/fastmcp.md),
[OpenTelemetry reference](../libraries/OpenTelemetry_Rust.md) and
[Serde/Schemars reference](../libraries/Serde_serde_json_and_Schemars.md) informed the work.
The installed pins, rather than newer overview examples, governed implementation.

### Current verification and original attempts

Current local evidence is under `build/mcp-corrections/20261025T183742Z/`, abbreviated
`C/` here. `C/coverage-index.json` preserves the combined per-binding basis and failed host
attempt alongside successful outcomes. Exact commands and conditions remain in the lane
receipts and logs. Every check uses the zero failure target; overlapping populations are
not summed.

| Command / lane | Result and scope | Evidence |
| --- | --- | --- |
| Coverage/parser/currency/archive unit selection | 78 focused tests passed. | `scope-currency-replay-unit-corrected.log` |
| Service/telemetry/host-admission unit selection | 368 tests passed before the final dedicated dead-caller refinement; three refined migration controls then passed. The complete final setup run re-exercised this current code. | `telemetry-service-unit-final.log`, `legacy-migration-guarded.log`, `setup-final/` |
| Custody resource unit module | Final 92/92 passed after recovery and static corrections. | `custody-static-correction-tests.log` |
| `just check-package pse-codegen`; selected generator unit; workspace sources | Compile and one generator unit passed; workspace sources 16/16 passed. | `codegen-check.log`, `codegen-frame-test.log`, `workspace-sources-final.log` |
| `just setup-test-report …/setup-final` | One clean 2,426-case run passed. Ordinary recipe, with the existing native dependency loader path; no extra enclosing native admission. | `setup-final.log`, `setup-final/setup-test.xml` |
| Real STDIO source/protocol census | 21 cases/420 tool calls passed; all 94 malformed-envelope cases reached the intended refusal. | `source-current/lane.json` |
| Fresh actual Codex workflows | All eight selected workflows passed, plus a second diagnose-run journey reading the independently known continuous trace/span. | `host/*/{case,outcome}.json`, `events.jsonl` |
| Four disposable native journeys | Continuous/finite parentage, independent finite capture, joined telemetry/MCP without preparation, and canonical project/result/LP/IPC/nested controls all passed, each exit zero. | `native/lane.json` and its original child receipts |
| `just hygiene` plus corrected failing leaves | Original full hygiene failed lint-py/typecheck in new code; all other leaves passed. Ruff and Pyrefly reruns passed, zero diagnostics with seven existing suppressions unchanged. This is a repaired composite result, not a second clean full hygiene run. | `hygiene/`, `lint-py-final.log`, `typecheck-final.log`; final fixture-only closeout reruns also passed |
| Governance and native contract/data lints | All passed. No feature powerset or performance campaign was added for this tooling/generator correction. | `governance.log`, `lint-native-contracts.log`, `lint-native-data.log` |
| Ordinary native restoration | Prepared/published native, validated dev extension after fixture teardown. The final artifact bytes match the successful prepared Codex result-reader artifact; the restored-model probe passed both controls. | `ordinary-native-restoration.json`, `native/restored-model/receipt.json` |
| Documentation and lifecycle validation | Both passed: 667 chapters published; 1,401 lifecycle scope items validated, no errors. | `docs-final.log`, `document-lifecycle-final.log` |

The first full setup attempt had an unnecessary enclosing native admission that expired;
its failures remain in `setup.log`. A normal attempt without the required existing native
loader could not import one test module and refused its failed-loader selection
(`setup-normal.log`). With that prerequisite supplied, the first complete run had one
old fixture missing its newly required telemetry selection; that fixture was corrected,
its 17-case module passed, and the final complete 2,426-case run passed. None of these
original attempts was replaced by the clean result.

One unprepared Codex native read correctly refused because that attachment omitted many
other, conservatively bound environment inputs. A fresh explicitly prepared attachment
then read the independently known nested `fresh` value. This prerequisite result does not
undo the separate no-preparation proof for the six corrected runtime selectors. Native
preparation occurred once in that successful host journey; no uncertain scientific action
was replayed. Current generation acceptance regenerated the implemented Frame relationship
and compared outputs; it did not repeat the original temporary source-marker experiment.

### Historical conversion and physical closeout

The user-authorized conversion moved the five retained runs and two associated cold replays
to `C/converted-runs/<resource-id>/`. Stable IDs and original logical identities remain;
current physical locations are separate custody metadata. Original receipt bytes and
pass/failure/incomplete/provenance flags are unchanged. The original empty failed replay
still has no invented receipt or producer manifest.

| Original retained run | Stable resource ID |
| --- | --- |
| Archive | `59aa931096c9406da23c7b44051392b9` |
| Generated contracts check | `54b9050c68f442a1b4154b25eddd0c1b` |
| Generated contracts | `a35cd98bf8104172a72c66757e720650` |
| Generator restore | `00f05894ba824ff9b3c630b45fb421a1` |
| Original failed host replay | `21aa6e3c979047f78ab51d83b39dc80e` |

The associated replay IDs are `017fdb71a1964cdb902cd95f589a8a86` and
`bdc314f36bdd44ba8a1a2c26e27676ec`. `C/historical-conversion.json` records the returned
locations; original snapshots, conversion verification and post-removal reader checks are
retained alongside it. Six existing roots remained byte-identical through protected old
and current locators. The native archive inspector passed at its returned current path
both before and after deletion. Ordinary native operations should use those returned
physical paths rather than assuming every historical absolute CLI path is compatible.

The registry inventory found exactly those seven authorities under the historical
checkout. After conversion, independent identity, clean-tree, ancestry and observed
cwd/open-file/mapped-file checks permitted normal Git worktree removal and merged-branch
deletion. `C/worktree-retirement.json` records success: only the main worktree remains.
There was no forced deletion, fabricated provenance or weakening of disposal eligibility.

All four new native journeys completed original context/kernel retirement and telemetry
stop/drain/unit/state cleanup; their held controllers were reaped. The ordinary native
extension was restored afterward. Existing pre-selection storage profiles still require
explicit stopped, quiesced readmission before current-code startup; no ordinary live
service was automatically migrated or restarted.

Historical lost-ACK drain remains unobserved. Coverage still does not prove assertion
adequacy, telemetry is not result/settlement authority, and no whole scientific/solver,
profiler, parity, performance or cross-platform qualification is claimed.


## Original verdict and practical consequences

**Full-contract acceptance is withheld.** The approved campaign is complete as an
assessment: every selected area was exercised or assigned an explicit failure or
qualification limit. **Tested:** all eight declared workflows have successful actual
Codex journeys; 93 of 94 callable capabilities have at least one independently checked
positive observation. The complete 141-entry catalogue was reconciled, and all 94
callables passed strict malformed-envelope checks and schema/effect comparisons.
These are coverage counts, not 93 complete semantic or cleanup qualifications.

The main product failure is the fresh coverage producer: its command construction is
rejected before tests start, leaving `coverage_correspondence_read` without positive
fresh-data evidence. A runtime-selector/artifact-currency discrepancy also needs
separate repair assessment. Continuous telemetry in the selected private service
fixture and collector-down fallback remain unqualified. Five original report resources
could not be retired through their owner, so the clean disposable worktree remains.

No production repair was made. The changes are acceptance drivers and this report.
The assessment does not change the supported contract or reopen historical Plan52/55
implementation status. Findings below are proposals for disposition by their existing
owners. No whole scientific platform, performance, parity, other operating system or
complete profiler/solver matrix qualification is claimed.

## Original baseline and conditions

The initial main checkout was clean at
`b5bdac9da763bf1a70f20b0599de0edad8a955f9`. The isolation checkout stayed at that product
revision. Assessment-only changes were committed as `0478f0990` and `3d884ec45`.
A concurrent independent design audit committed documentation as `c02f68670`; its edits
were preserved. Native preparation was rerun after an early source-cut change interrupted
preparation, before any selected Rust tests were dispatched.

The clients were fresh Codex processes with shell execution disabled, independent real
FastMCP STDIO clients, and the delivered capability CLI on this Linux workstation.
The source-final environment records Python 3.14.8, FastMCP 4.1.3 and jsonschema 4.26.0;
the pinned Rust toolchain was `nightly-2026-09-29`. Workload commands used the repository
environment and ordinary selected unbounded policy. Native actions retained their
original admission, preparation, result and drain owners. No dependency pins changed.
The failure baseline for every reported check is zero.

Current contracts remain in the [capability interface guide](capability-interface.md),
blueprint §21.1/§23/§24 and their native owners. Historical results are labelled as such;
a current reader consuming historical inputs does not make the original work current.

Original evidence is local under `build/mcp-assessment/20261025T172727Z/` (abbreviated
`E/` below). These ignored files are not website attachments. Exact commands, arguments,
responses, original managed results and failed attempts are retained. The final
`E/coverage-index.json` maps every entry to evidence and limits. Lane aliases are:

| Alias | Original evidence | Meaning |
| --- | --- | --- |
| S | `E/source-final/{lane,cases,catalogue,strict-inputs,protocol-resources}.json` | Independent STDIO source facts, discovery, guidance and input checks |
| I | `E/isolation/lane.json` and named child receipts | Isolated native providers, edit custody, CLI, generation and freshness |
| N | `E/native/lane.json` and named child receipts | Selected linked-native consumers, store, diagnostics and telemetry |
| R | `E/interruption/lane.json`, `reader-reassessment.json` | Lost acknowledgement, original readers, actions and cancellation |
| H | `E/host/reassessment.json`, each case's `events.jsonl` and `outcome.json` | Actual Codex, exact ordered requests and independent expected fields |
| C | `E/components/lane.json`, `setup-discovery.json` | Component regression population, including omitted async modules |
| A | `E/hygiene/`, final leaf logs, `turn-end-final/` | Scope-end static and formatting results |

### Assessment-area disposition

This table records the full array of work, including blocked conclusions. Successful
component tests support an area but do not substitute for its protocol or native case.

| Area | Outcome and concrete oracle | Evidence |
| --- | --- | --- |
| Catalogue and bootstrap | Passed: 11 bootstrap tools, 14 domains, eight workflows, 141 entries; all declared descriptions resolved | S catalogue and cases |
| Checkout identity and initial availability | Passed identity/handshake and passive diagnosis; live import compatibility observed; native currency initially UNKNOWN, not universally ready | `cli-diagnosis.json`, `readiness-live.json`, I `identity.json` |
| Selection and reveal | Passed hidden/reveal/reset behavior and two concurrent real STDIO selections without a union | S; `selection-race.json` |
| Guidance and workflow discovery | Passed 26 exact guide bodies/digests, package member bytes/size/digest and native MIME; 18 direct protocol resource observations | S protocol resources and cases |
| Input, schema and effect contracts | Passed all 94 strict invalid envelopes; real `list_tools` input/output/effects and generated CLI wire revision/lifetime/read-only comparisons | S strict inputs; `wire-contracts.json` |
| Source and knowledge reads | Passed 16 source readers against independently known source facts; supported corpus ownership respected | S source cases; N model/knowledge receipts |
| Actual agent workflows | All eight passed selected actual Codex journeys with guidance, prompt, exact schema, request sequence, reset and terminal checks | H and I actual-host receipts |
| Navigation and editing | All 24 editing bindings have positive selected-provider evidence; exact locations, rename/apply bytes, two-page action preview and custody checked | I independent oracles and call files |
| Native model/project/result/store | Passed selected source/admission/model/project, LP/result/IPC, pack and narrow database controls; explicit partial/empty/refused meanings preserved | N lane and receipts |
| Work, tests and assessment readers | Selected force-validated unit/compile checks, two assessment gates, authentic reader binding and no reader-triggered replay passed | R; H qualify-change; native Rust receipts |
| Telemetry and investigation | SDK counts, metrics, exemplars, watches/alerts, known traces, bounded custody and finite full parent chains passed; continuous/private and collector fallback gaps remain | N telemetry/investigation receipts; H diagnose-telemetry |
| Diagnostics and interruption | Real GDB collection and lost-ACK recovery passed; synthetic selected-profiler cancellation preserved unrelated subject; original lost-ACK native drain remains unobserved | R; N finite capture |
| Experiments | Archive/inspect/replay passed through CLI and aligned actual host; distinct originals and hash refusal checked; replay current-source qualification remains false | I archive/replay; H aligned case |
| Generation and Python stages | With a controlled fixture generator-owner change, actual host regenerate/compare passed; original bytes restored; dev/native preparation, stubs and admission passed | I generation and stage receipts |
| Coverage, regression and static checks | Fresh coverage producer failed before tests; component composite passed; full hygiene initially failed two leaves, both corrected and rerun successfully | Coverage logs; C; A |
| Freshness and retirement | Old host refused changed serving source; fresh host accepted restored source. Owned native fixtures settled; five report resources block worktree retirement | I freshness and retirement handoff; N/R cleanup |

## Original capability coverage matrix

**Interface-checked:** every entry below was described; all 94 tool entries additionally
had input/output schema and effect/lifetime correspondence checked against real wire
and generated CLI surfaces. All 94 strict invalid-envelope cases reached the intended
input-validation refusal, including typed native `malformed_invocation` responses.
A hidden-selection or stale-source refusal was not accepted as an input-validation pass.

**Tested:** `Positive` means at least one selected independent observation exists in the
listed lane, not that every field, provider, prerequisite, effect or lifecycle branch
passed. `Limited` highlights positive but partial or fixture-bounded meaning. `Blocked`
means no positive fresh reader evidence. The 46 native/guidance entries and one internal
entry are interface-checked declarations only; their presence does not qualify every
native command described by their guidance. Supporting native journeys are reported
separately. The component maps are literal navigation indexes, not execution proofs.

### Callable entries (94)

| Domain | Capability | Lifetime | Positive observation and boundary | Evidence |
| --- | --- | --- | --- | --- |
| diagnostics | `diagnostic_action` | single | Limited: Real GDB/finite capture; selected cancellation uses synthetic profiler | N, R |
| diagnostics | `diagnostic_action_read` | single | Positive: Selected native/provider fact or original value; lane limits apply | R |
| diagnostics | `native_diagnostic_checkpoint` | single | Limited: Bounded codec fixture; separate live joined checkpoint | N |
| diagnostics | `native_diagnostic_inspect` | single | Limited: Bounded checkpoint process/operation equality | N |
| diagnostics | `native_diagnostic_processes` | single | Limited: Bounded codec and live two-worker discovery | N |
| diagnostics | `native_evidence_compare` | single | Limited: Current reader of supplied historical comparison inputs | N |
| diagnostics | `native_evidence_inspect` | single | Limited: Bounded frozen fixture | N |
| diagnostics | `native_invocation_read` | single | Limited: Exact original invocation JSON equality | N |
| diagnostics | `telemetry_configuration` | single | Limited: Selected original incomplete configuration preserved | N |
| diagnostics | `telemetry_health` | single | Limited: Selected original incomplete/uncertain health preserved | N |
| diagnostics | `telemetry_investigation_export` | single | Limited: Constructed selection custody/hash fixture | N |
| diagnostics | `telemetry_profile_capture` | single | Limited: Real retained Timeline; live CPU capture unqualified | N |
| diagnostics | `telemetry_query` | single | Positive: Selected native/provider fact or original value; lane limits apply | H, N |
| diagnostics | `telemetry_schema` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| diagnostics | `telemetry_selection_read` | single | Limited: Constructed rows0,1 direct/resource/projection equality | N |
| diagnostics | `telemetry_trace_lookup` | single | Positive: Selected native/provider fact or original value; lane limits apply | H, N |
| diagnostics | `telemetry_watch_configure` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| diagnostics | `telemetry_watch_read` | single | Limited: Selected original incompleteness preserved; alerts independently checked | N |
| editing | `edit_preview_apply` | endpoint | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `edit_preview_read` | endpoint | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `edit_preview_reconcile` | endpoint | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `edit_preview_release` | endpoint | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `edit_preview_status` | endpoint | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_action_preview` | endpoint | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_call_hierarchy` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_code_actions` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_completions` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_declaration` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_definition` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_diagnostics` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_hover` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_implementations` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_incoming_calls` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_outgoing_calls` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_references` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_rename_preview` | endpoint | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_signature` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_symbols` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_tool_support` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_type_definition` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `navigation_workspace_symbols` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| editing | `structural_json_source_bytes_preview` | endpoint | Positive: Selected native/provider fact or original value; lane limits apply | I |
| environment | `capability_explain` | single | Positive: Selected native/provider fact or original value; lane limits apply | N, S |
| environment | `capability_readiness` | single | Limited: Passive declaration only; not live native readiness | S |
| environment | `capability_readiness_check` | single | Limited: Live import compatible; native currency UNKNOWN; operation unexercised | readiness-live |
| environment | `compilation_check` | single | Positive: Selected native/provider fact or original value; lane limits apply | R |
| environment | `environment_explain` | single | Positive: Selected native/provider fact or original value; lane limits apply | I, S |
| environment | `python_extension_dev_stage` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| environment | `python_extension_native_stage` | single | Positive: Selected native/provider fact or original value; lane limits apply | H, I, N |
| environment | `storage_observe` | single | Positive: Selected native/provider fact or original value; lane limits apply | S |
| execution | `product_run_read` | single | Limited: Authentic original results; lost-ACK native drain unobserved | H, R |
| execution | `rust_test_reference_read` | single | Limited: Original lost-ACK test bound; native drain unobserved | R |
| execution | `workload_watch` | single | Limited: Original owned work selection; not universal settlement | R |
| experiments | `experiment_archive` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| experiments | `experiment_inspect` | single | Positive: Selected native/provider fact or original value; lane limits apply | H, I |
| experiments | `experiment_replay` | single | Limited: Replay passes; current_source_qualified false | H, I |
| generation | `contracts_check` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| generation | `contracts_generate` | single | Positive: Selected native/provider fact or original value; lane limits apply | I |
| generation | `native_contract_schema` | single | Positive: Selected native/provider fact or original value; lane limits apply | S |
| governance | `document_scope_query` | single | Positive: Selected native/provider fact or original value; lane limits apply | S |
| governance | `documentation_build` | single | Positive: Selected native/provider fact or original value; lane limits apply | documentation-call |
| integration | `database_info` | endpoint | Limited: Narrow info succeeds; broad response partial | N |
| integration | `database_list` | endpoint | Limited: Narrow scope succeeds; broad response partial | N |
| integration | `database_query` | endpoint | Positive: Selected native/provider fact or original value; lane limits apply | N |
| integration | `database_select` | endpoint | Positive: Selected native/provider fact or original value; lane limits apply | N |
| knowledge | `knowledge_explain` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| knowledge | `knowledge_query` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| knowledge | `knowledge_source_corpus` | single | Positive: Selected native/provider fact or original value; lane limits apply | H, S |
| knowledge | `library_source_packet` | single | Positive: Selected native/provider fact or original value; lane limits apply | S |
| projects | `native_model_compare` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| projects | `native_model_declarations` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| projects | `native_model_inspect` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| projects | `native_preparation_discover` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| projects | `native_project_bundle_inspect` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| projects | `native_project_metadata` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| projects | `native_project_select` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| projects | `native_research_read` | single | Limited: Known empty research population | N |
| projects | `native_result_inspect` | single | Limited: Known LP/nested fresh value; readable completeness remains false | H, N |
| projects | `native_result_read` | single | Limited: Protected result/IPC; explicit field and population bounds | N |
| projects | `native_revision_preparation_discover` | single | Limited: Known empty preparation population | N |
| projects | `native_revision_source` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| projects | `native_source_admit` | single | Positive: Selected native/provider fact or original value; lane limits apply | N |
| validation | `assessment_read` | single | Limited: Exact original two tooling gates; input_coverage false | R |
| validation | `assessment_run` | single | Limited: Two tooling gates pass; input_coverage false | R |
| validation | `assessment_select` | single | Positive: Selected native/provider fact or original value; lane limits apply | S |
| validation | `coverage_correspondence_read` | single | Blocked: Producer fails before tests; reader correctly unavailable | coverage logs |
| validation | `rust_test_describe` | single | Positive: Selected native/provider fact or original value; lane limits apply | S |
| validation | `rust_test_run` | single | Positive: Selected native/provider fact or original value; lane limits apply | H, R |
| workspace | `workspace_concepts_query` | single | Positive: Selected native/provider fact or original value; lane limits apply | H, S |
| workspace | `workspace_integration_view` | single | Positive: Selected native/provider fact or original value; lane limits apply | S |
| workspace | `workspace_inventory` | single | Positive: Selected native/provider fact or original value; lane limits apply | S |
| workspace | `workspace_producer_relationships` | single | Positive: Selected native/provider fact or original value; lane limits apply | I, S |
| workspace | `workspace_recipe_describe` | single | Positive: Selected native/provider fact or original value; lane limits apply | S |
| workspace | `workspace_task_context` | single | Positive: Selected native/provider fact or original value; lane limits apply | H, S |

### Native/guidance and internal entries (47)

All rows here are **Interface-checked**, without a claim that their complete native
command family was executed. Native implementation tests above cover selected functions.

| Domain | Entry | Exposure | Declared source/route |
| --- | --- | --- | --- |
| diagnostics | `active-diagnostics` | native | `scripts/active_diagnostics.py` |
| diagnostics | `scientific-diagnostics` | native | `python/pse/diagnostics.py` |
| diagnostics | `telemetry-observation` | native | `scripts/telemetry.py` |
| editing | `controlled-edits` | native | `scripts/semantic_edits.py` |
| editing | `language-navigation` | native | `scripts/semantic_navigation.py` |
| environment | `build-cache` | native | `scripts/build_environment.py` |
| environment | `dependency-maintenance` | native | `Cargo.toml` |
| environment | `environment-preparation` | native | `scripts/pse_env.py` |
| environment | `host-implementation` | internal | `scripts/host_admission.py` |
| environment | `python-extension` | native | `scripts/python_extension.py` |
| evidence | `evidence-inspection` | native | `xtask/src/evidence_tools.rs` |
| execution | `producer-workers` | native | `scripts/producer_deployment.py` |
| execution | `workload-observation` | native | `scripts/activity.py` |
| experiments | `experiments-recovery` | native | `scripts/experiments.py` |
| experiments | `scientific-references` | native | `scripts/scientific_reference.py` |
| generation | `generated-contracts` | native | `xtask/src/codegen.rs` |
| governance | `decisions-register` | native | `scripts/adr.py` |
| governance | `documentation-lifecycle` | native | `scripts/document_lifecycle.py` |
| governance | `skill-adr` | native | `.codex/skills/adr/SKILL.md` |
| governance | `skill-create-plan` | native | `.codex/skills/create-plan/SKILL.md` |
| governance | `skill-design-principle-audit` | native | `.codex/skills/design-principle-audit/SKILL.md` |
| governance | `skill-design-principle-audit-tests` | native | `.codex/skills/design-principle-audit-tests/SKILL.md` |
| governance | `skill-design-review` | native | `.codex/skills/design-review/SKILL.md` |
| governance | `skill-design-review-audit` | native | `.codex/skills/design-review-audit/SKILL.md` |
| governance | `skill-design-review-process-simulator` | native | `.codex/skills/design-review-process-simulator/SKILL.md` |
| governance | `skill-execute-plan` | native | `.codex/skills/execute-plan/SKILL.md` |
| governance | `skill-functional-extension-review` | native | `.codex/skills/functional-extension-review/SKILL.md` |
| governance | `skill-plan-completion-audit` | native | `.codex/skills/plan-completion-audit/SKILL.md` |
| governance | `skill-plan-creation` | native | `.codex/skills/plan-creation/SKILL.md` |
| governance | `skill-plan-design-review` | native | `.codex/skills/plan-design-review/SKILL.md` |
| governance | `skill-plan-execution` | native | `.codex/skills/plan-execution/SKILL.md` |
| governance | `skill-plan-functional-extension-review` | native | `.codex/skills/plan-functional-extension-review/SKILL.md` |
| governance | `skill-plan-improvement-review` | native | `.codex/skills/plan-improvement-review/SKILL.md` |
| integration | `git-integration` | native | `scripts/worktree.py` |
| integration | `session-specialists` | native | `scripts/codex_session.py` |
| knowledge | `knowledge-recall` | native | `scripts/knowledge_store.py` |
| knowledge | `library-research` | native | `scripts/library_skills.py` |
| knowledge | `skill-native-solver-libraries` | native | `.codex/skills/native-solver-libraries/SKILL.md` |
| projects | `projects-revisions` | native | `xtask/src/project_tools.rs` |
| projects | `scientific-inputs` | native | `scripts/scientific_inputs.py` |
| projects | `scientific-python-api` | native | `python/pse/__init__.py` |
| storage | `build-storage` | native | `scripts/build_storage.py` |
| storage | `database-tooling` | native | `scripts/surreal_server.py` |
| validation | `qualification-assessment` | native | `scripts/validation_scope.py` |
| validation | `validation-selection` | native | `scripts/select.py` |
| validation | `verification-techniques` | native | `scripts/assertion_sensitivity.py` |
| workspace | `source-context` | native | `scripts/workspace_tasks.py` |

### What the positive observations establish

Source reads asserted original repository facts rather than merely accepting valid JSON.
Navigation exercised real Python and Rust providers: definitions/locations, references,
completions, diagnostics, symbols and call hierarchy had selected checks. Rust supplied a
known nonempty `const fn is_nan` signature; a Python function-name signature position
legitimately returned empty. Rename/apply and structural edits were compared with exact
expected final bytes. The Rust visibility action preview was independently reconstructed
across both pages. This does not qualify every LSP provider branch.

Native model controls covered admitted source, preparation, declarations, inspection and
changed-source comparison. Canonical project/revision, exported packs, protected result
access and nested IPC reads passed. A bounded LP independently required `x = 0.5` within
`0..2`; nested reads required source row zero and `start.kind = fresh`. Original
`readable_complete=false` and context incompleteness were retained; no general scientific
usability was inferred. Research/preparation discovery had known empty populations.
Unknown nested fields returned an observed document whose kind was `refused`, not a value.

Database controls used narrow table/field/schema scopes, a parameterized `RETURN 7`, a
wrong-fence refusal and a native VIEWER write attempt. That write returned status OK with
an empty no-op response; a privileged before/after read independently proved no rows were
created. An exception was not required as the oracle. Broad bounded info/list responses
remain partial.

The six offline native diagnostic/evidence readers used the existing public adapter's
codec-created bounded checkpoint, exact process/operation/frozen/invocation comparisons
and literal expected comparison counts. The comparison executable ran now over supplied
historical baseline/candidate inputs. A separate joined live checkpoint observed the
known Python executor with two workers and workflow polls. The bounded codec fixture
alone does not establish arbitrary live checkpoint coverage; larger captures may refuse
whole at their documented bounds.

Telemetry controls used disposable owned pipelines: three known SDK events, sampler-on
and sampler-off resets, independent counter/rate/bucket/exemplar expectations, watch
configure/read/disable, and scheduled alerts (six deliveries/eight history observations).
Actual MCP `telemetry_query` checked all four streams plus rate/exemplars; these were not
only direct backend reads. Actual Codex queried a known compiler-message counter of three
and the exact emitted trace/span.

Investigation export/selection compared manifest/custody hashes and fixture rows zero/one
across direct and resource projections. The original investigation subdriver supplied
constructed query answers; these prove custody/projection, not live population discovery.
Timeline capture produced a nonempty original retained operation; finished CPU and wrong
identity requests refused without output. Live CPU profiling is not qualified.

Finite F02 capture passed two independent sibling 11-span parent chains spanning the
canonical request/helper/native SDK/direct receiver/executor/module. Original capture
contained 1,885 same-trace server spans, including 1,462 executor and 409 receiver-RPC
spans, and 2,294 spans over all traces; workload exit zero, no omitted spans or capture
errors, restoration complete. Its pass does not replace the failed continuous case.

## Original workflow outcomes

The final host assessor requires workflow selection, owned guidance read before the
operation, `get_prompt`, exact schema description before invocation, exact ordered
requests, independently expected fields/paths, no extra or replayed action, final reset
and terminal completion. Dynamic action-to-reader fields bind to the original returned
reference. Retained transcripts were reassessed with the final assessor without replay.
All host processes below were reaped. These are selected journeys within each workflow,
not exhaustive qualification of every optional workflow step.

| Workflow | Successful actual-host journey | Calls and evidence |
| --- | --- | --- |
| `inspect-plan` | Read the exact selected `server.py` source context and compare its digest; source-only journey | 10; H `inspect-plan` |
| `plan-functional-review` | Read planning guidance and query the known `docs/README.md` document identity; no review executed | 9; H `plan-functional-review` |
| `investigate-evidence` | Read the exact supported source document and expected corpus item | 7; H `investigate-evidence` |
| `qualify-change` | Run one signed-zero Rust unit with force-validation, then read its authentic product result | 12; H `qualify-change` |
| `diagnose-run` | Recover original lost-ACK product reference without replay; separately query known telemetry and trace | 7 plus 9; H `diagnose-run`, `diagnose-telemetry` |
| `manage-project` | Explicit native endpoint prepares its exact environment, then reads the expected nested fresh value | 12; H `manage-project-native-prepared` |
| `run-experiment` | Inspect existing archive, align original runtime environment, replay once into new output | 12; H `run-experiment-aligned`; current-source qualification false |
| `change-generated-contract` | After a controlled fixture edit of the isolated generator owner, query producer, regenerate and compare; exact source/output restoration follows | 17, four operations; I `generation-host/actual-host` |

The generation fixture changed only a temporary marker in the markdown generator header,
never hand-edited generated output. Generated frames showed that marker while scientific
`frame.rs` stayed byte-identical. Generation passed its one selected operation; comparison
passed all six selected leaves with complete/covered results. Restoring the generator and
regenerating restored generator, frames and scientific declaration hashes exactly.

A separate existing core Codex driver passed nine stages/22 calls, including one real
`float::tests::f32_nan_variants_collapse_too` unit and complete owned drain. Its original
result is under `build/capabilities-52k/mcp-assessment-20261025T172727Z/` (invocation
`34eca8fa3bed4b6e89dc9c654ca86934`). Four existing actual-host workflow probes also passed
stale/partial/planning/generated-recovery controls; they intentionally reopened historical
retained references, rather than executing new scientific work.

Freshness used an old actual host, a controlled comment change in
`environment_queries.py`, then a second description/read: description reported
`restart_required=true` and the same host returned the typed restart refusal. After exact
source restoration and old-host teardown, a new actual host passed with
`restart_required=false`. The old host had ten calls; the fresh host had ten. Codex marked
the refused item as failed while preserving typed content, which the final assessor reads.

## Original failures, refusals and uncertainty

### Product and owner gaps requiring separate disposition

| ID | Observed gap and consequence | Original evidence and proposed next decision |
| --- | --- | --- |
| MCP-A01 | Fresh coverage capture fails before tests: `unsupported scope option --json; cannot infer selected work`. The producer constructs `cargo llvm-cov nextest --json`, which the scope parser rejects. The subsequent reader correctly returns `pse.workspace.source_unavailable`; it has no positive fresh-data coverage. | `E/coverage-command.log`, `coverage-reader.json`; propose repairing producer/parser composition, then rerun this exact bounded producer and reader. |
| MCP-A02 | Changing only runtime observation/output selectors caused `artifact_not_current`, despite the current capability interface guide retaining endpoint selectors and explicit telemetry carriers as runtime inputs. Exact-environment native preparation allowed the later operations. | N `telemetry-mcp-observed/observed-calls.json`; keys: `OTEL_BSP_EXPORT_TIMEOUT`, `OTEL_BSP_SCHEDULE_DELAY`, `OTEL_EXPORTER_OTLP_TIMEOUT`, `OTEL_METRIC_EXPORT_INTERVAL`, `PSE_DIAGNOSTICS_DIR`, `PSE_TELEMETRY_STATE`. Propose reconciling artifact identity with its current owner contract. |
| MCP-A03 | Original report custody refuses retirement: three generation reports have unverified provenance, one failed preflight replay lacks a producer manifest, and its archive retains a protected consumer reference. Safety refusals worked; the retirement journey remains incomplete. | I `retirement-final-handoff.json`, original `test_resources.eligible` guards; propose deciding source-owned sealing/provenance/failed-consumer retirement semantics, without bypassing them. |

These are assessment-local identifiers, not new plan status authorities. A01 is a
producer defect, not evidence that a functioning coverage reader returned wrong data.
A02 is an observed currency/contract discrepancy; this campaign did not diagnose or
repair every fingerprint path. A03 does not establish a general data leak.

### Qualification limits and failed fixture controls

Continuous F02 failed twice with `f02.complete_module_parent_chain_not_observed` in
`N/telemetry-f02-explicit/f02/acceptance.json` and the selected retry. Native siblings
passed and the fixture had 1,343 trace rows, 1,222 SDK-request spans and three known producer
incarnations, but no complete receiver/module chain. Source inspection showed the persistent
service unit does not persist the caller's private `PSE_TELEMETRY_STATE`, while
`server_environment` resolves service/default producer configuration. Selecting a private
pipeline for this caller did not configure the owned continuous service. This fixture
therefore did not qualify continuous receiver/module visibility; it does not prove a general
receiver defect or absent native execution. The supported finite capture passed separately.
The ordinary service/pipeline was not modified to force this test through.

The prepared joined telemetry journey passed genuine checkpoint/process identity,
two-worker executor/poll observations, usable producer exit zero, exact-incarnation backend
join, source admission and known traces. It later failed `fixture.unit_not_stopped` at the
collector-stop control. **Collector-down fallback was not run.** The later authenticated
cleanup stopped/drained the unit and removed its files; that cleanup does not turn the
original journey into a pass (`N/telemetry-mcp-prepared/joined/acceptance.json`).

The exact `docs/generated/frames.md` producer lookup returned an empty relationship.
The declared `docs/generated/relations/authored.md` relationship supported the generation
journey. This is an observed sidecar coverage limit, not a mapping repair.

`capability_readiness_check` observed live import compatibility, native currency UNKNOWN
with `check-unavailable`, and `operation_exercised=null`. Passive readiness is not evidence
of executed native readiness. Some telemetry health/configuration/watch responses were
incomplete by design; selected fields and uncertainty were preserved. Their positive
matrix entries do not mean complete pipeline health.

### Preserved failed attempts and assessor corrections

Original attempts remain alongside corrected drivers. No failure was overwritten and
no uncertain action was automatically replayed to obtain a convenient success.

| Original attempt | Correction and final interpretation |
| --- | --- |
| Source corpus query used an arbitrary unsupported `server.py` owner | Use the exact supported document; original semantic refusal was correct. |
| Strict native malformed input was expected to be outer protocol failure | Accept only the original typed malformed-envelope refusal at the intended validation stage, even when protocol delivery succeeds. |
| Initial editing receipt targeted protected `build/`; rename handle/page comparisons were wrong | Use a fresh valid fixture and exact nested handle/two-page expected bytes. The original unsettled edit response alone establishes neither rollback nor no edit. |
| Bare actual-host native reads lacked providers; attached native read then differed on PATH currency | Use the documented explicit native endpoint and prepare within that exact host environment. Initial prerequisites remain failed observations. |
| Initial host archive replay had a runtime-environment mismatch before execution | Forward original environment variable names and use a fresh output; no uncertain executed action was replayed. |
| Root commit interrupted first native Rust preparation | No tests had dispatched. Repeat on stable source; 17 selected tests then passed. |
| Unknown nested field expected outer native refusal; VIEWER write expected exception | Check typed refused document and independently prove no database mutation. |
| Mutable top-level xtask lacked canonical-tools | Select the exact immutable feature artifact, verify help/hash and owned copy. |
| Lost-ACK assessor confused raw versus managed terminal state | Reassess the same original reference, without replay; native drain remains unobserved. |
| Investigation wrapper expected the wrong metric family and nested `asyncio.run` | Retain successful original native evidence; run the bounded helper in its supported thread context. |
| Two async tests errored under XML runner because redirected stderr lacks `fileno` | Run those two unchanged through stdlib unittest with real file streams; preserve original runner errors. |
| New acceptance scripts failed initial lint/type checking | Structural narrowing and checked subprocess handles fixed the owned scripts; both failing leaves were rerun successfully. No ignores or new suppressions. |

## Original recovery and physical cleanup

Lost-ACK injection dropped the response after exactly one original Rust unit completed.
The caller timed out; a new endpoint recovered the original test/product/watch references
without replay. Original invocation `391ae2642c0a4cbb866d697f3d328834` still has native
`owned_drain=unobserved`. The dropped-response endpoint separately exited zero and was
reaped; a reader pass does not upgrade the native drain observation.

Real owned GDB capture had target exit zero and complete helper drain. The cancellation
control selected a synthetic profiler but used real helper admission/cancellation/drain;
the unrelated owned subject survived cancellation and was finally terminated/waited by
its owner. This does not qualify cancellation behavior of every installed profiler.

Native canonical contexts/kernels and finite capture resources were drained/retired by
their owners. Disposable telemetry units were stopped/drained, unit files and selected
state removed, and failed data retained stopped for evidence. See N `cleanup_final`,
`canonical-explicit/lifecycle/passed.json`, `telemetry-f02-finite/f02/retirement/passed.json`
and the respective `telemetry-cleanup.json` files. No live owned native work remained in
those final observations. The ordinary main native Python extension was prepared through
its original owner after private telemetry runs (`E/ordinary-native-restoration.json`):
prepared/published, mode `native`, profile `dev`, `validated=true`, exit zero. No historical binary bytes were restored.

The isolated checkout has clean tracked state and no observed process with that worktree
as cwd. All seven report trees had owner/path identity checked and were exported byte-for-byte
before source-owned release/reclaim attempts. Report provenance remains unverified where
listed below. Exports preserve evidence; they do not discharge original custody.
Two successful replay resources became cold/compacted. Five active resources remain,
all drained, with no active borrows or pins:

| Original resource | Retained ID | Owner refusal |
| --- | --- | --- |
| Archive | `59aa931096c9406da23c7b44051392b9` | Protected reference to failed replay checks |
| Generated contracts check | `54b9050c68f442a1b4154b25eddd0c1b` | Unverified report provenance |
| Generated contracts | `a35cd98bf8104172a72c66757e720650` | Unverified report provenance |
| Generator restore | `00f05894ba824ff9b3c630b45fb421a1` | Unverified report provenance |
| Original failed host replay | `21aa6e3c979047f78ab51d83b39dc80e` | Missing producer artifact manifest; disposition remains incomplete |

Cold replay IDs are `017fdb71a1964cdb902cd95f589a8a86` and
`bdc314f36bdd44ba8a1a2c26e27676ec`. Final identity/custody is recorded in
I `retirement-final-handoff.json` and exports in `retirement-exports.json`.
Drain was recorded only after independent child/creator/unit/borrow checks; no manifest
was fabricated and incomplete producer status was preserved.

**Worktree and branch deliberately retained:**
`/home/paul/pse-arrow-wt/mcp-acceptance-20261025T172727Z`, branch
`mcp-acceptance-20261025T172727Z`. No force deletion or custody bypass occurred.
The original enclosing navigation attempt's exit 125 and transient allocation warning
remain recorded even though the allocation was absent on later observation.

## Original commands, counts and evidence

All rows use the zero failure target. Overlapping command populations are not summed.
Exact invocations and modes are retained in the corresponding lane JSON and logs.

| Command / mode | Actual result and scope | Evidence |
| --- | --- | --- |
| `scripts/pse-env -- .venv/bin/python -m scripts.tests.mcp_capability_assessment` with source-final output | Passed 21 cases, 420 tool calls; 18 additional direct resource observations. Source/protocol only. | S lane and environment |
| `just setup-test-report …/components/full-setup` | 2,277 passed, zero failures/errors/skips, 381.752 seconds; original invocation `dc02d66826484ac4bf5ecd0ca9399efd`. | C original XML/selection/log |
| Async-only supplement and cross-owner unittest commands | 106 additional unique cases across 16 modules. Initial 86-case XML run: 84 pass/two runner errors; unchanged two-case stdlib rerun and remaining 20-case supplement pass. | C attempts and original results |
| Composed component population | 2,383 unique latest case outcomes passed, zero current failures/errors/skips. **Not one clean 2,383-case run.** Setup discovery's literal `unittest.TestCase` scan omitted async-only modules. | C latest-unique counts and setup discovery |
| Native Python public-adapter selection | 52 passed, zero failures/errors/skips, 105.06 seconds; invocation `6dfa13c32cfe4f629593012d9a96aca6`. | N `public-adapters/native-python.xml` |
| Linked-native selected Rust controls | 17 passed, zero failures, 1,573 unselected; original invocation `fdbd89a3919643adaff25d6a66e239f8`, nextest `ad626b18-6152-4c03-921c-7b170929d74b`. Includes four genuine canonical nested/IPC/multipage/cancel controls. | N selected native Rust receipts |
| `scripts.tests.mcp_host_acceptance` / isolated fresh Codex | Eight workflows passed selected journeys; initial failed prerequisite cases retained; final assessor rereads original transcripts without replay. | H, I host events/outcomes |
| `python -m unittest scripts.tests.test_mcp_host_acceptance -v` through `scripts/pse-env` | Eight synthetic assessor-fault tests passed; validates the assessor, not the product. | `host-assessor-unit-final.log` |
| `scripts.tests.mcp_interruption_acceptance` | Lost-ACK, bounded actions, real GDB and selected cancellation composed pass, with original failed assessor attempt retained. | R commands and receipts |
| `compilation_check` selected `pse-ids` library | Passed, force-validation active, complete/covered and input coverage true. | R compilation original checks |
| `assessment_run` selected fmt-check | Both `fmt-rust-check` and `lint-toml` passed; complete/covered, **input coverage false**, narrow tooling scope. | R original assessment and reader reassessment |
| `coverage_correspondence capture … -- -p pse-ids --lib -E 'test(=float::tests::signed_zero_is_preserved_and_distinguished)'` through `scripts/pse-env` | Exit one before any tests; fresh correspondence reader correctly unavailable. | Coverage command/reader logs |
| `just hygiene --output …/hygiene` | Original full run failed two leaves: lint-py (30 errors) and typecheck (176 diagnostics) in new assessment scripts. Other leaves passed, including codegen, generated capabilities/CLI, Python contracts/stubs, Clippy modes, rustdoc and family checks. | `hygiene.log` and original checks |
| `just lint-py`; `just typecheck` | Corrected failing leaves both exit zero: all Ruff checks pass, zero Pyrefly diagnostics (seven existing configured suppressions unchanged). This is a composed scope-end result, not a second clean full hygiene run. | `lint-py-final.log`, `typecheck-final.log` |
| `just turn-end --output …/turn-end-final` | ADR index and formatting passed, exit zero. | A final checks |
| `documentation_build` original capability | Passed, complete/covered; source unchanged during the operation. | `documentation-call.json` |
| Final `just docs`; document lifecycle validation | Both passed, exit zero: 667 chapters published; 1,401 lifecycle scope items validated, no errors. | `docs-final.log`, `document-lifecycle-final.log` |

The regression result readers were reopened, but their original unobserved native drain
was not rewritten as global cleanup success. Component mention maps support navigation
only. Finite telemetry and diagnostic fixture counts are observations under the stated
conditions, not measured performance claims.
