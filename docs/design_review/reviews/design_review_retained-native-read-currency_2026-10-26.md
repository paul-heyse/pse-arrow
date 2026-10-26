---
title: "Retained native reads and producing artifact currency"
date: 2026-10-26
tier: design
purpose: target
focus: principle-focused
standard:
  core: "3.4"
  heuristics: "1.0"
  process-simulator: "1.5"
  test-assessment: "1.2"
source_baseline: "1240ca074a2d2d2b265da23d08aa5e02b120e067 plus inspected concurrent work"
disposition_owner: docs/plans/56-design-principle-alignment.md
doc_role: review
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, environment, preparation, diagnostics]
---

# Retained native reads and producing artifact currency

## Assessment and scope — slots 1 and 12

**Accept the bounded target at Proposed evidence.** Finite responsibility-owned operation projections, currency associated with the artifact actually produced and installed, and foreground-owned retained read workers form a coherent correction for DA-015/043. Their responsibilities should remain distinct: operation declarations determine meaning and eligibility; preparation owners establish currency; a foreground lifetime owner carries admitted native execution, protection and settlement.

This accepts the design direction in ADR-0231 and Plan56f F02/F05/F06. It does **not** accept the current F05 extractor as complete, authorize removal of existing currency guarantees before its replacement qualifies, establish F06 implementation, or close DA-015/043. Current implementation adequacy remains unestablished.

The review is an independent, focused continuation of Plan56's principle-alignment engagement under P00/RC06. Its principal focus is AP-01/AP-04/AP-07 and DP-01/09/10/18/19/20/22. Other applicable foundations constrain the correction. Library evidence supports feasibility; this is not a library-selection engagement.

The functional outcome is proportionate interactive scientific inspection: successive exact pages reuse stable preparation without confusing source currency, loaded image, physical context, original scientific permission, response completeness or native drain. Relevant variation includes another finite public operation, changed consumed build inputs, changed physical/provider context, concurrent reads, worker replacement and endpoint loss.

The boundary includes Rust document/operation owners, generated host projections, PyO3/facade/MCP/CLI consumers, extension and executable preparation, helper admission, worker transport and original protected native readers. It excludes solver algorithm changes, new scientific models, new persistence authority, daemonization, persistent compiler products, withdrawn all-role producer proofs and unrelated qualification campaigns.

**Reviewer:** independent delegated design reviewer, 2026-10-26. No repository edits, builds, tests, services or fault probes were performed.

## Baseline and audit applicability

The consumed diagnostic account is the audit published at `f9b96810a84409d9d32ff2a1f62b029ac151ae2e`, whose inspected product source was `682fc3830f8a370f0995544af5adf821e4be9ab9`. The relevant finding anchors are [DA-015](../audit/boundaries-and-composition.md#da-015) and [DA-043](../audit/development-tooling.md#da-043).

This review reconciled that account with source at `1240ca074a2d2d2b265da23d08aa5e02b120e067`, the proposed ADR-0231, current Plan56 owners and concurrent F01/F04/F05 work inspected during review. The working tree was changing; this is ordinary source capture, not a frozen or qualified implementation baseline.

Intervening A–E work changes the premises:

- A supplies original physical-context interpretation and cursor binding; B supplies governing/product permission and owned scientific operation facts.
- C supplies canonical meaning, exact effects and source protection. D supplies changed population/cohort/publication and recovery owners.
- E supplies responsibility-scoped document publication, product/tooling separation, deployment/allocation and prepared-input contracts. These are dependencies to preserve, not reasons to restart their reviews.
- DA-015's earlier universal extension-prerequisite observation no longer applies literally. `native_contracts.Binding.requirements` routes `evidence_compare` to the independently built evidence executable; `evidence_worker.execute` avoids product-extension/store construction. Its executable currency obligation remains.
- Current native transport still launches one-shot children. `native_worker.execute` still performs currency admission, rejects an already-loaded `pse._native`, imports the extension and fences the installed artifact. `execution_basis.Handoff` binds one exact request. Those are decisive retention migration seams.
- The concurrent producing-input extractor and `producing_currency.Witness` provide a plausible bounded mechanism. They were inspected as work in progress. Their existence establishes neither complete input coverage nor installed-image association.

Thus the relevant diagnoses remain applicable, with these narrowings. No broad audit was refreshed and no historical file counts, timing observations or prior test results were renewed.

## Responsibilities and consumed contracts — slots 2, 3 and 5

| Responsibility | Owned decision and contract | Required boundary |
|---|---|---|
| Scientific operation owner | Request/result meaning, defaults, bounds, effects, prerequisites and read-lifetime eligibility | Hosts consume projections; they do not reconstruct scientific rules |
| Existing document/code generation | Faithful finite projections, directional schemas and mechanical adapters | Unknown constructs refuse; generated agreement does not independently prove declaration correctness |
| Preparation/currency owner | Supported producing inputs, configuration and actual installed artifact association | Currency is distinct from schema compatibility and scientific identity |
| Foreground read owner | Admitted worker/context, request correlation, retained resources, retirement and settlement | Endpoint/workload owns lifetime; callers do not gain native execution authority from a cursor |
| Native reader | Original selection, physical interpretation, governing permission and protection | Retention preserves these authorities and their failure distinctions |
| Dedicated effect owner | Effectful or crash-isolated execution and physical drain | Read reuse does not absorb writes or weaken their isolation |

The finite operation declaration should capture the public surface already consumed by the hosts. Small typed receiver glue remains appropriate where invocation differs. Generating arbitrary runtime execution or introducing a general workflow language would add a responsibility the target does not need.

Read eligibility must be an owned property of the operation's consumed contract. `EXACT_READS`, whose present purpose includes preservation of whole documents and an explicit capture action, is not by itself a sufficient retention policy. Nor are an operation name, request shape or “inspection” label. An operation that writes a capture, depends on a changing diagnostic subject or requires crash isolation can legitimately retain dedicated execution.

Currency and read-context identity serve different purposes. A producing receipt describes why an installed executable is current. A read context additionally binds the admitted source/physical/provider/preparation/deployment premises and the image loaded in its worker. Neither a schema fingerprint nor equality of wire documents supplies that second guarantee.

The retained owner may reuse immutable preparation and established validity. Each incoming request still requires independent bounded decoding, exact admitted operation/request association and correlation with its response. It must not carry the original one-shot environment handoff unchanged across different requests.

### Retained lifetime semantics

The accepted target includes the following preservation requirements:

1. A worker incarnation establishes its loaded-image witness and admitted context before receiver entry. Later reuse checks the premises that can change. Changed premises retire that incarnation; old cursors cannot silently attach to a replacement.
2. Page position and selected output window are changing request inputs. They do not alone require rebuilding stable preparation. Changed source, original physical context, provider, issuer, preparation or deployment cannot be treated as an ordinary page advance.
3. An admitted multi-page workload retains its original deadline. Queueing, page advance, cancellation or worker replacement cannot renew it. A separately admitted operation may have its own deadline without inheriting permission from an old cursor.
4. Cancellation and observer detachment remain distinct outcomes. Whichever policy applies, live native work retains its resource charge and protections until physical settlement. Late output cannot become another request's response.
5. Existing admission covers retained input/preparation state, active work, queued request bytes and outgoing responses together. Limiting worker count alone is insufficient if idle contexts or queued requests accumulate without ownership.
6. Endpoint EOF, shutdown and replacement settle owned workers. Independent teardown must continue after a sibling failure, retaining failures and cancellation. Process exit or transport completion alone cannot be promoted to proof of native settlement.
7. Retention does not hold a database transaction across pages, grant fresh scientific permission, create a daemon or imply automatic replay.

These requirements make retention a lifetime change rather than a bypass around admission.

### Physical and numerical scope

| Element | Physical meaning and authority | Preservation requirement |
|---|---|---|
| Model/physical inspection | Authored physical documents and admitted preparation | Bind reuse to the exact original physical context |
| Stored result inspection | Original persisted context, issuer, relation/schema and selection | Preserve missing-context refusal and governing permission separately from readable bytes |
| Native execution context | Admitted provider/preparation/deployment and loaded image | Context equality must not be inferred from matching schemas |

`workflow/result_inspection.rs::inspect_result` already checks issuer, basis and schema cursor premises and validates original selection/physical context. `workflow/results.rs` retains native result-cursor ownership. These are preservation seams, not disposable per-page overhead.

No formulation, derivative, scaling, structural-analysis or solving stage is added here. Well-posedness remains with existing preparation/execution owners; a retained read cannot bypass it or reinterpret solver outcomes. Numerical-stage columns and new unit/property conformance suites are therefore not applicable to this change.

## Revealing scenarios — slot 4

| Scenario | Expected behavior and change boundary | Evidence |
|---|---|---|
| S01: add a public read using existing scientific concepts | Owner declaration and necessary typed receiver glue extend the surface; independent host default/prerequisite lists disappear | Proposed; existing Rust catalogue and generator provide a source-supported foundation |
| S02: edit a consumed input, an unrelated workspace leaf, or producing-package membership | Relevant changes invalidate currency; unrelated changes do not; uncertain opaque inputs stay conservative | Proposed; extractor/Witness source is incomplete qualification evidence |
| S03: replace installed bytes during admission/import or between reads | Refuse or retire before receiver entry under mismatched premises; distinguish loaded image from current path contents | Proposed; existing pre/post-import fencing is a preservation requirement |
| S04: advance a page, then change physical/provider/source context | Stable preparation survives page advance; changed context retires the worker and stale cursors refuse | Proposed; original cursor/physical readers supply consumed semantics |
| S05: cancel a queued/active request, close the endpoint, or replace a worker | Original deadline, request association, charges and protections survive until exact settlement | Proposed; existing attached and detached transport policies must be migrated explicitly |
| S06: grow concurrent reads and retained contexts | Existing shared admission bounds admitted and retained work, with backpressure; no new workstation partition | Proposed; assembled behavior remains untested |

S01 is an operation extension; S02/S03 are producer/mechanism changes; S04–S06 exercise composition and lifecycle. Adding new physics is outside this boundary and remains with its scientific owner.

## Architectural foundations and gates — slot 6

These verdicts concern the **proposed target**, not the unfinished implementation.

| Foundation | Verdict | Basis |
|---|---|---|
| AP-01 Separation | satisfied | Operation meaning, currency and lifetime have distinct owners and independent reasons to change |
| AP-02 Stable contracts | satisfied | Typed requests/results and explicit prerequisite/effect/context contracts precede host mechanics |
| AP-03 Composition | satisfied | Existing generators, preparation and native readers compose without another workflow interpreter |
| AP-04 Domain model/authority | satisfied | Operation, producing artifact, loaded image, context, request, cursor and worker incarnation remain consequential distinct concepts |
| AP-05 Explicit structure | satisfied | Eligibility, request admission, retirement and drain have explicit boundaries |
| AP-06 Local reasoning/testability | satisfied | Declaration projection, currency interpretation and transport lifecycle can be challenged separately; actual native settlement still requires integration |
| AP-07 Execution fit | satisfied | Stable preparation/import work moves to its useful lifetime; retained state and interruption obligations remain admitted |

| Gate | Target verdict | Reason |
|---|---|---|
| G1 Authority | pass, Proposed | Owned operation facts and existing preparation receipts; no second freshness authority |
| G2 Semantic fidelity | pass, Proposed | Directional contracts and original physical/permission interpretation are preserved |
| G3 Validity | pass, Proposed | Unknown inputs, malformed requests and stale context refuse before receiver use |
| G4 Hidden behavior | pass, Proposed | Preparation, process retention and effects remain explicit |
| G5 Consistency/recovery | pass, Proposed | Request/incarnation isolation, original deadlines and physical drain remain required |
| G6 Transformation/reuse | pass, Proposed | Complete scoped premises govern reuse; conservative fallback remains explicit |
| G7 Truthful claims | pass, Proposed | Unsupported modes and implementation qualification remain separately stated |
| G8 Library leverage | pass, Proposed | Existing Cargo evidence, code generation and native/process owners are reused; no generic compiler or solver is recreated |
| G9 Architectural fitness | pass, Proposed | All applicable foundations are satisfied by the bounded target |
| PS-G1 Physical consistency | pass for preservation, Proposed | Original physical context and its refusal semantics survive retention |
| PS-G2 Well-posedness | not applicable to new behavior | No formulation or solver-entry path is introduced |
| PS-G3 Numerical integrity | not applicable to new behavior | No numerical state or solver outcome interpretation is changed |

Current F05/F06 implementation gates are **unresolved**, rather than passed by this table.

## Findings, preservation and blockers — slot 7

No additional target-level MUST defect was established. The existing findings retain their identities and Plan56 disposition owner:

| Finding | Current scoped conclusion | Correction and distinguishing acceptance |
|---|---|---|
| DA-015 | Host operation meaning still has manual projections; supplied-only comparison prerequisites have already narrowed | Generate the finite surface from responsible owners and remove replaced mirrors. Independently challenge defaults, effects, eligibility and prerequisite populations across actual hosts |
| DA-043 | One-shot currency/import/lifetime work remains; proposed extraction is not qualified | Associate actual producing inputs with installed bytes, then retain an exact admitted read lifetime. Demonstrate relevant/unrelated change discrimination, image races, context changes and physical drain |

Three implementation blockers precede removal of the old guarantees:

- **Producing evidence:** qualify ordinary editable dev/native and fixed executable routes, including selected features/targets, rustc environment, build-script behavior, configuration/tool/interpreter/native inputs and transformed installed bytes. A final depfile or artifact message alone is insufficient.
- **Session admission:** replace the current one-request handoff and “module already loaded” refusal with an explicit admitted session plus independently bound requests. Merely looping `native_worker.execute` is neither functional nor adequate.
- **Composed settlement:** adopt the lifetime through both dispatch routes and foreground CLI, including retained state, deadlines, stale cursors, endpoint EOF, cancellation and drain.

These blockers are realization obligations already implied by the target, not grounds to reopen the whole-system design.

## Mechanism and library fit — slot 8

The finite projection extends existing responsibility-scoped Rust catalogues and `pse-codegen` document generation. Schemars/serde describe owned documents; generated Python and MCP/CLI projections consume them. Typed invocation glue retains responsibility for genuine receiver differences. Generation removes repeated metadata without pretending to generate scientific behavior.

For currency, the inspected adapter reuses Cargo's selected unit graph, `compiler-artifact` and `build-script-executed` observations, associated dep-info and declared environment inputs. Conservative producing-package membership covers supported opaque cases. `producing_currency.Witness` retains file/directory observations so unchanged bytes need not be rehashed. This is a source-supported candidate mechanism, **not qualified evidence of completeness**.

Cargo does not own Maturin's final installed transformation, the Python image actually imported, or native installation custody. Those remain integration-owner obligations. The same producing-currency contract should serve fixed executable consumers without requiring them to acquire product-extension prerequisites.

Existing process/native owners supply execution and drain. The foreground protocol supplies the remaining session/request gap; a new service framework, daemon or persistent evidence ledger is unnecessary.

No dependency change or newly selected library is required by this judgment.

## Alternatives and tradeoffs — slot 9

| Alternative | Assessment |
|---|---|
| Fresh child and current broad currency per call | Preserves simple isolation, but repeatedly reconstructs stable preparation and imports. Suitable for effects/crash isolation; poor default for eligible successive reads |
| Qualified currency plus one explicitly admitted multi-page workload | Simplest conforming fallback; amortizes within one workload but retains setup across separate eligible calls |
| Qualified currency plus foreground retained worker | Selected. Removes repeated setup while keeping custody local; adds request/session correlation, retained-state ownership and explicit retirement |
| Cargo freshness alone | Useful evidence source, but does not cover transformed installation or the loaded worker image |
| Daemon, persistent compiler cache or universal producer service | Adds persistence, recovery and authority obligations unnecessary for this foreground outcome |

The selected design trades process-level freshness for an explicit lifetime contract. That trade is justified for eligible reads because the stable admitted state is reused directly. It is not justified for every native operation merely because retention is available.

Conservative package closure may still inspect more than the exact consumed source set. That is acceptable where dependency evidence is opaque; it does not justify returning to unrelated whole-workspace hashing. No latency, throughput or capacity improvement is measured here.

## Verification and evidence limits — slot 10

| Claim | Evidence and required settlement |
|---|---|
| Target separates concerns coherently | Proposed; independent source/design reasoning in this review |
| Current one-shot seams require replacement | Implemented source inspected in `native.py`, `process.py`, `native_worker.py` and `execution_basis.py`; no runtime probe |
| Producing extraction supplies complete supported currency | Unestablished; F05's actual producer/installer controls remain required |
| Generated operation projections preserve meaning | Proposed; actual cross-host expectations must challenge the declaration rather than derive every expected value from it |
| Retention preserves exact context and settlement | Proposed; actual endpoint/page/cancel/EOF/replacement journeys through both routes and foreground CLI remain required |
| Repeated work is removed | Proposed; verify actual paths no longer repeat unchanged tree/image hashing, secondary import or per-page admission |
| Performance improvement | Unmeasured; no quantitative claim |

Plan56g owns assembled acceptance after functional integration. This review ran no product checks and supplies no new pass count. Prior A–E evidence retains its recorded source, commands and conditions. F05 controls cannot be replaced by synthetic receipt agreement, and F06 drain cannot be established solely by mocked process exit.

## Rule impacts and disposition — slot 11

| ID | Current rule | Recommended change | Dependency and fallback |
|---|---|---|---|
| RC01 | Blueprint §21.1's current boundary projection description | Record responsibility-owned finite operation declarations and generated host projections, retaining typed receiver glue | DA-015/F02. If unchanged, retain manual host wiring and its outstanding alignment obligation |
| RC02 | Blueprint §24.1's fresh-worker image-admission wording | Permit an actually witnessed foreground endpoint/workload worker to remain admitted under unchanged premises, with request isolation and drain-before-replacement | DA-043/F06; this is the selected Plan56 RC06/ADR-0231 route. If retained unchanged, use fresh children plus qualified currency and explicit multi-page workloads |

These review-local identifiers do not create additional operator decisions or override Plan56's existing RC numbering. ADR-0231 and the owning design route carry the amendment. No SHOULD exception, scientific permission change, new persistence authority or restart of withdrawn producer campaigns is recommended.

Current disposition and acceptance evidence remain solely in Plan56 and its F/G companions.

## Decision

**Behavioral/semantic target:** adequate at Proposed evidence.

**Architectural fitness:** adequate at Proposed evidence.

**Overall:** **Accept the bounded target.** No further target correction is required before proceeding with its authorized realization. F05's qualified producing/installed association, F06's actual session admission and composed native settlement remain mandatory implementation obligations. Their completion cannot be inferred from this review, ADR status or source-only agreement.
