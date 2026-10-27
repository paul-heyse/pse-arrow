---
title: "Retained native reads and owned artifact publication"
date: 2026-10-26
tier: design
purpose: target
focus: principle-focused
standard:
  core: "3.4"
  heuristics: "1.0"
  process-simulator: "1.5"
  test-assessment: "1.2"
source_baseline: "1240ca074a2d2d2b265da23d08aa5e02b120e067 plus inspected concurrent work; operator-directed reassessment at 1e5a6e79a2404ddb2594a77a92543fc8048cff38"
disposition_owner: docs/plans/56-design-principle-alignment.md
doc_role: review
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, environment, preparation, diagnostics]
---

# Retained native reads and owned artifact publication

## Assessment and scope — slots 1 and 12

**Accept the revised bounded target at Proposed evidence.** Responsibility-owned operation projections, owner-controlled immutable artifact publication and foreground-owned retained read workers form a coherent correction for DA-015/043. Construction establishes the artifact's validity once. Consumers use the issued contract and its immutable generation; internal handoffs and successive pages carry that established property.

This revises the earlier judgment that qualified producing-input freshness and installed-image fencing were necessary conditions of the target. Those mechanisms re-established facts the construction and publication owner can secure directly. A smaller source scanner or a more strongly typed provenance record would retain the wrong responsibility in reads. AP-04 and AP-07 favor an explicit construction/publication lifecycle that removes that work, supported by heuristics H18–H20 and H28.

The target remains within ADR-0231 and Plan56f F02/F05/F06, subject to their corresponding revision through the owning design route. This review does not establish implementation completion, pass runtime acceptance or close DA-015/043. It changes the proposed correction and its acceptance obligations.

The functional outcome is proportionate interactive scientific inspection: successive exact pages reuse an admitted native generation and stable preparation while preserving original physical context, scientific permission, response completeness and native drain. Relevant changes include a new finite operation, successful or failed generation construction, explicit reload/replacement, changed physical/provider context, concurrent reads and endpoint loss.

The boundary includes Rust document/operation owners, generated host projections, PyO3/facade/MCP/CLI consumers, extension and executable publication, worker attachment and original protected native readers. Solver algorithms, new scientific models, new persistence authority, daemonization and a general provenance framework remain outside scope.

### Attribution and changed premises

**Original assessment:** independent delegated design reviewer, 2026-10-26, against the original source baseline below. Its exact text is preserved in Git at `1e5a6e79a2404ddb2594a77a92543fc8048cff38` under this document's path.

**Revision:** delegated design reviewer, 2026-10-26, following explicit operator direction during authorized implementation. The operator questioned refusals caused by artifact sensitivity, requested modeled contracts insensitive to linting, and clarified that controlled creation/editing plus construction contingencies should avoid double validation. This revision independently assesses the resulting target; it does not present the original judgment as unchanged.

No builds, tests, services or fault probes were performed for this revision. Its evidence is source/design reasoning and the supplied construction/publication contract. Implementation and qualification remain with the Plan56 owners.

## Baseline and audit applicability

The consumed diagnostic account is the audit published at `f9b96810a84409d9d32ff2a1f62b029ac151ae2e`, whose inspected product source was `682fc3830f8a370f0995544af5adf821e4be9ab9`. Relevant anchors are [DA-015](../audit/boundaries-and-composition.md#da-015) and [DA-043](../audit/development-tooling.md#da-043).

The original review reconciled that account with source at `1240ca074a2d2d2b265da23d08aa5e02b120e067`, proposed ADR-0231, current Plan56 owners and concurrent F01/F04/F05 work. The revision addresses the exposed F05 responsibility mismatch without reopening the broad audit. The moving implementation baseline is not a qualified release baseline.

Intervening A–E work supplies original physical-context interpretation, governing/product permission, canonical effects and source protection, publication/recovery owners, responsibility-scoped documents and deployment/allocation contracts. These authorities remain dependencies. DA-015's earlier universal extension-prerequisite diagnosis has already narrowed: supplied-only evidence comparison uses its separately built executable without product-extension/store prerequisites. That executable can use the same owned publication lifecycle without acquiring unrelated product prerequisites.

The original F05 mechanism consumed Cargo unit/artifact/build-script observations, associated dep-info and source/environment witnesses to infer whether an installed artifact remained current. This conflated two questions: what construction used, and whether a successfully published immutable artifact remains valid for its issued contract. Rewritten build controls or later source edits do not change the latter. Native transport and retained foreground ownership remain relevant migration seams, but their admission need not reconstruct a producer proof.

No historical file counts, timings or prior test results are renewed here.

## Responsibilities and consumed contracts — slots 2, 3 and 5

| Responsibility | Owned decision and contract | Required boundary |
|---|---|---|
| Scientific operation owner | Request/result meaning, defaults, bounds, effects, prerequisites and read eligibility | Hosts consume projections without reconstructing scientific rules |
| Existing document/code generation | Finite projections, directional schemas and mechanical adapters | Unknown declaration constructs refuse; generation does not independently prove scientific correctness |
| Construction/publication owner | Successful build/install, immutable generation, issued contract and atomic publication | Only complete successful construction publishes; failure preserves the prior generation |
| Foreground read owner | Published generation attachment, admitted context, request correlation, retirement and settlement | The endpoint/workload owns lifetime; a cursor does not grant execution authority |
| Native reader | Original selection, physical interpretation, governing permission and protection | Retention preserves these meanings and their failure distinctions |
| Dedicated effect owner | Effectful or crash-isolated execution and physical drain | Read reuse does not absorb writes or weaken isolation |

The finite operation declaration should capture the public surface already consumed by hosts. Small typed receiver glue remains appropriate where invocation differs. Neither arbitrary runtime execution generation nor a general workflow language is needed.

Read eligibility is an owned property of the consumed operation contract. `EXACT_READS`, an operation name or an “inspection” label cannot independently establish it. A capture action, changing diagnostic subject or crash-isolated operation may retain dedicated execution.

### Smallest sufficient artifact model

Use the existing preparation record as the published contract. Its typed meaning identifies the native artifact kind, immutable generation path and the interface/build selection the owner actually constructed. A generation identifier distinguishes publications; it is not a new proof ledger. Construction observations such as Cargo stdout, unit records, dep-info and producing inputs may help construction or diagnose its failure. They do not become ongoing consumer obligations merely because they were observed.

The owner privately constructs and copies the completed extension or executable into a new immutable generation. It publishes the descriptor atomically only after those steps succeed. Build, install, copy or publication failure leaves the previous published generation available. An incomplete candidate cannot become the selected publication. The same owner controls replacement and reclamation; a referenced generation cannot be removed while an attached worker still needs it.

The consumer resolves that published contract and attaches its exact immutable path once. A mutable editable-install alias or Cargo target path is not the retained generation. No later digest, loaded-image fence, source scan, environment comparison, dep-info interpretation, producer rescan or `Witness.inputs` check is required to re-establish construction validity. Internal consumers receive the already admitted typed owner rather than independently decoding or revalidating its construction facts.

Compatibility remains a distinct question only when a caller requests an interface that may differ from the issued contract, such as a compiled catalogue expected by an independently supplied client. Check that boundary where it exists. Equality checks repeated between projections from the same admitted owner add no assurance. An incompatible requested interface requires a compatible construction or attachment; it does not make an older artifact invalid.

Changing source files, compiler controls or the environment used for a future build does not revoke an existing immutable generation. No claim is made that arbitrary source edits are semantically equivalent. They describe a future construction, while the published generation continues to describe the earlier successful one. Python serving-code changes use explicit drain/reload control rather than inferred import-closure freshness.

### Retained lifetime semantics

1. A worker attaches to one issued immutable generation and admitted context. A new publication does not silently rebind that worker or its cursors. An explicit replacement drains the old owner before attaching the new one; an existing generation may remain in use while its owner allows it.
2. Page position and output window are changing request inputs. They do not require reconstruction or revalidation of stable preparation. Changed original physical/provider context or a separately requested deployment/contract requires the corresponding owner transition; a worktree edit alone is not such a transition.
3. Each incoming request retains necessary bounded decoding, operation/request association and response correlation. Controlled native values do not lose validity at every internal handoff. The original one-shot handoff cannot simply be reused unchanged for different requests.
4. An admitted multi-page workload retains its original deadline. Queueing, page advance, cancellation or worker replacement cannot renew it. A separately admitted operation may have its own deadline without inheriting permission from an old cursor.
5. Cancellation and observer detachment remain distinct outcomes. Live native work retains resource charges and protections until physical settlement. Late output cannot become another request's response.
6. Existing admission covers retained preparation, active work, queued request bytes and outgoing responses together. Worker-count limits alone do not own accumulating idle contexts or queues.
7. Endpoint EOF, shutdown, explicit reload and replacement settle owned workers. Independent teardown continues after a sibling failure, retains individual failures and preserves cancellation. Process exit or transport completion alone is not proof of native settlement.
8. Retention holds no database transaction across pages, grants no fresh scientific permission, creates no daemon and implies no automatic replay.

These are operation, context and lifecycle guarantees. They do not justify repeating artifact construction validation.

### Physical and numerical scope

| Element | Physical meaning and authority | Preservation requirement |
|---|---|---|
| Model/physical inspection | Authored physical documents and admitted preparation | Bind reuse to the original physical context |
| Stored result inspection | Persisted context, issuer, relation/schema and selection | Preserve missing-context refusal and governing permission separately from readable bytes |
| Native execution context | Admitted provider/preparation/deployment and attached publication | Preserve explicit context ownership across requests |

`workflow/result_inspection.rs::inspect_result` and `workflow/results.rs` own original result selection, physical context and cursor semantics. These remain preservation seams. No formulation, derivative, scaling, structural-analysis or solving stage is added. Existing scientific preparation, solver outcome interpretation and Arrow correctness validation are unaffected by this artifact-lifecycle correction.

## Revealing scenarios — slot 4

| Scenario | Expected behavior and change boundary | Evidence |
|---|---|---|
| S01: add a public read using existing scientific concepts | Extend the owner declaration and necessary typed receiver glue; remove replaced host mirrors | Proposed; existing catalogue/generator supplies the foundation |
| S02: lint rewrites Cargo controls, or source/environment changes after publication | The issued immutable generation remains usable; a future construction may consume different inputs | Proposed; replaces the original change-triggered freshness refusal |
| S03: construction, copy or publication fails; a mutable installed alias changes | Failed construction preserves the old publication; alias changes cannot redirect an attached immutable generation | Proposed; owner construction/failure controls require functional verification |
| S04: explicitly publish and select a new generation or reload serving code | Drain/replace under the owning control; preserve old cursor/incarnation boundaries and referenced generations | Proposed; no source watcher or post-publication image check |
| S05: advance a page, then change original physical/provider context | Stable preparation survives page advance; actual context transitions retain their owned refusal/replacement semantics | Proposed; original native readers supply the meaning |
| S06: cancel work, close an endpoint or grow concurrent retained reads | Preserve original deadlines, exact response association, shared admission, protections and physical settlement | Proposed; assembled behavior remains untested by this review |

## Architectural foundations and gates — slot 6

These verdicts assess the revised target; they do not pass the moving implementation.

| Foundation | Verdict | Basis |
|---|---|---|
| AP-01 Separation | satisfied | Scientific meaning, artifact publication and native lifetime have separate owners |
| AP-02 Stable contracts | satisfied | Consumers depend on issued capabilities and exact generation rather than producer internals |
| AP-03 Composition | satisfied | Existing preparation records, generators and native readers compose without a new framework |
| AP-04 Domain model/authority | satisfied | Construction, publication, attachment, context, request and incarnation govern behavior explicitly |
| AP-05 Explicit structure | satisfied | Atomic publication, failure preservation, replacement and reclamation have owned transitions |
| AP-06 Local reasoning/testability | satisfied | Construction failure and worker settlement can be challenged independently; native settlement still needs integration |
| AP-07 Execution fit | satisfied | Established validity is carried through the useful lifetime; repeated reconstruction is removed |

| Gate | Target verdict | Reason |
|---|---|---|
| G1 Authority | pass, Proposed | Construction issues the artifact contract; consumers do not create another freshness authority |
| G2 Semantic fidelity | pass, Proposed | Operation meaning, original context and scientific permission survive retention |
| G3 Validity | pass, Proposed | Valid construction precedes publication; request decoding and genuinely distinct compatibility boundaries remain enforced |
| G4 Hidden behavior | pass, Proposed | Publication, attachment, explicit reload and effects are visible owner operations |
| G5 Consistency/recovery | pass, Proposed | Failed candidates preserve the old publication; referenced generations and native work retain owners |
| G6 Transformation/reuse | pass, Proposed | Immutable issued artifacts and admitted preparation retain established validity |
| G7 Truthful claims | pass, Proposed | Target reasoning, implementation and qualification remain separately stated |
| G8 Library leverage | pass, Proposed | Existing build/install and process capabilities are used without duplicating their guarantees |
| G9 Architectural fitness | pass, Proposed | All applicable foundations are satisfied by the revised bounded target |
| PS-G1 Physical consistency | pass for preservation, Proposed | Original physical interpretation and permission remain intact |
| PS-G2 Well-posedness | not applicable to new behavior | No formulation or solver-entry path is introduced |
| PS-G3 Numerical integrity | not applicable to new behavior | No numerical state or solver outcome interpretation is changed |

## Findings, preservation and superseded obligations — slot 7

The findings retain their identities and Plan56 disposition owner. The correction to DA-043 is materially revised:

| Finding | Revised scoped conclusion | Distinguishing acceptance |
|---|---|---|
| DA-015 | Finite operation meaning belongs to scientific owners and generated projections | Challenge defaults, effects, eligibility and prerequisites independently across actual hosts; delete replaced mirrors |
| DA-043 | Reconstructing source/build currency in reads is unnecessary under owned immutable publication | Establish validity during successful construction; consume the typed publication and retained owner; preserve explicit replacement and physical drain |

The original review accepted too much machinery as necessary assurance. It required a qualified input extractor and later revalidation without first asking whether controlled construction and immutable ownership already discharged the guarantee. This is a correction to the proposed design judgment, not a claim that a single excluded stdout path fixes the responsibility mismatch.

| Original obligation | Revised disposition and reason |
|---|---|
| Refuse an existing artifact after relevant source/environment/package changes | **Superseded by operator direction.** Such changes affect future construction, not validity of the issued immutable generation |
| Refuse/retire when mutable installed bytes or aliases change between reads | **Superseded by operator direction.** Attach the privately published immutable path; mutable aliases are outside that generation |
| Recheck producer records, dep-info, source trees or `Witness.inputs` to admit reads | **Superseded by operator direction.** Construction establishes validity once; observations are not consumer proof obligations |
| Fence or digest the loaded image repeatedly, or add another image check after publication | **Superseded by operator direction.** Exact immutable-path attachment and owner-controlled lifecycle remove that revalidation need |
| Retire a worker because Python serving source changed on disk | **Superseded by operator direction.** Serving-code changes use explicit drain/reload control |
| Preserve physical context, original cursors, request isolation, deadline and settlement | **Retained.** These are distinct scientific or lifecycle obligations, not producer reconstruction |

Implementation must therefore replace the old mechanism and its obsolete refusal tests rather than retain both paths. Any construction observation that has no remaining construction or diagnostic consumer should be deleted with its callers. No full provenance qualification campaign is required to remove read-time revalidation.

## Mechanism and library fit — slot 8

Responsibility-scoped Rust catalogues and `pse-codegen` already supply the finite projection route. Serde/Schemars describe owned directional documents; typed receiver glue handles genuine invocation differences. Generation removes repeated metadata without pretending to generate scientific behavior.

Cargo and the existing extension installer perform construction. The publication owner closes the remaining gap by copying the successful final extension/executable into a private immutable generation and atomically publishing the existing preparation descriptor. It owns failures and replacement. Cargo stdout, unit records and dep-info remain construction observations where useful; modeling them more precisely does not make them necessary on reads.

Existing foreground/process/native owners supply attachment, request association and drain. The target needs no new service framework, daemon, provenance ledger or dependency. Fixed executable consumers use the same lifecycle without product-extension prerequisites.

## Alternatives and tradeoffs — slot 9

| Alternative | Assessment |
|---|---|
| Per-call source/build/image revalidation | Reject for this controlled target: repeatedly reconstructs established construction facts and makes incidental build activity revoke useful artifacts |
| Cache or type the existing freshness scanner more precisely | Insufficient correction: improves the mechanism while retaining unnecessary consumer responsibility |
| Owned immutable publication with an explicitly admitted multi-page workload | Smallest conforming fallback when cross-call retention is unnecessary |
| Owned immutable publication with a foreground retained worker | Selected for eligible successive reads; adds only the needed request/lifetime ownership |
| General producer service, daemon or provenance ledger | Adds authority, persistence and recovery obligations without a supported need |

The immutable copy has a concrete purpose: it transfers the successfully constructed native artifact from mutable build/install output into owner-controlled lifetime custody. Retained generations incur storage/reclamation ownership; they do not require a second cataloguing framework. The owner must preserve referenced generations until settlement. Explicit reload makes update timing deliberate rather than treating arbitrary file changes as invalidation events.

No latency, throughput or capacity improvement is measured here. The structural claim is narrower: reads no longer reconstruct source/build/image validity already established by construction.

## Verification and evidence limits — slot 10

| Claim | Evidence and required settlement |
|---|---|
| Revised target separates concerns and removes unnecessary work | Proposed; independent scoped design reasoning in this revision |
| Publication is atomic and failures preserve the prior generation | Proposed; targeted owner tests must exercise failed build/copy/publication and incomplete candidates |
| Consumers use the issued immutable generation | Proposed; verify actual extension/executable attachment and that mutable aliases cannot redirect retained ownership |
| Lint/build-control/source/environment changes do not revoke an issued artifact | Proposed; targeted controls should demonstrate continued use without freshness scans, without claiming arbitrary edits are semantically equivalent |
| Distinct requested interface compatibility is preserved | Proposed; challenge actual incompatible requests where that boundary exists; do not duplicate same-owner checks |
| Retention preserves context and settlement | Proposed; actual page/cancel/EOF/explicit reload/replacement journeys through relevant routes and foreground CLI remain required |
| Replaced reconstruction is removed | Proposed; inspect the real call paths for absence of source/env/dep-info/producer rescans, `Witness.inputs`, repeated image fences and internal construction revalidation |
| Generated projections preserve meaning | Proposed; independent cross-host expectations must challenge the owner declaration |
| Performance improvement | Unmeasured; no quantitative claim |

Plan56g owns assembled acceptance after functional integration. This review supplies no new test pass count and does not upgrade prior source-only, mocked or focused evidence. Native settlement still requires its actual lifecycle journeys. Existing scientific correctness validation remains required in its own scope. Superseded freshness-refusal obligations must be revised honestly in their owning tests/docs rather than reported as satisfied by an exception.

## Rule impacts and disposition — slot 11

| ID | Current rule or prior proposed target | Recommended change | Dependency and fallback |
|---|---|---|---|
| RC01 | Blueprint §21.1 boundary projections | Responsibility-owned finite operation declarations and generated host projections, retaining typed receiver glue | DA-015/F02; otherwise retain the outstanding manual-projection obligation |
| RC02 | Blueprint §24.1 fresh-worker image admission and the original ADR-0231 currency proposal | Owner-controlled successful construction, immutable generation publication and foreground attachment under an issued contract; explicit reload and drain | DA-043/F05/F06 through the owning ADR/design amendment; single admitted multi-page workloads remain a conforming retention fallback |

These review-local identifiers do not create new operator decisions or replace Plan56's RC numbering. ADR-0231, the architecture owner and Plan56 must reconcile the changed proposal and superseded refusal criteria. No scientific permission change, new persistence authority or restart of withdrawn producer campaigns is recommended. Current disposition and acceptance evidence remain solely in Plan56 and its F/G companions.

## Decision

**Behavioral/semantic target:** adequate at Proposed evidence under the revised construction/publication contract.

**Architectural fitness:** adequate at Proposed evidence. Carrying owned validity removes the repeated assurance work that the original review accepted unnecessarily.

**Overall:** **Accept the revised bounded target.** Successful immutable publication, exact generation attachment, necessary request/context boundaries and composed native settlement remain realization obligations. Source-freshness and post-publication image-revalidation obligations are superseded as stated above. Implementation completion cannot be inferred from this review, ADR status or source-only agreement.
