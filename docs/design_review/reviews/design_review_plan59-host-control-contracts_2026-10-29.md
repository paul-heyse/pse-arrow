---
title: "Plan59 host control, diagnostic and analysis contracts"
date: 2026-10-29
status: proposed
doc_role: review
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [design, environment, diagnostics, current-work]
standard: {core: "3.4", process-simulator: "1.5"}
---

# Plan59 host control, diagnostic and analysis contracts

**Decision: Accept the A07 target-contract amendment at Proposed evidence.** The amendment
supports a coherent native boundary: the host owns invocation transport and OS containment;
product owners retain scientific meaning; analysis providers retain configuration and source
revision meaning; original resource owners establish settlement. Successful acknowledgement or
observation is different from a completed correctness pass. Existing-work control must remain
possible when current source cannot build.

This decision accepts the declared semantic amendment, not the unimplemented generation-control,
ledger, service-entry or analyzer-session mechanisms. Those remain subject to their actual
A03/A05/B03/G implementation and qualification obligations. Formal ADR-0233 status remains
proposed. No runtime, product, scientific or whole-Plan59 acceptance follows.

## Scope and evidence — slots 1 and 10

This is a **change-tier, target-purpose, combined** review of A07's native host control,
diagnostic, product-delegation and analysis-context contract changes. It supplies the scoped
formal route required before dependent A03/A05/B03 implementation. It is an independent
delegated assessment; the coordinator authors the proposed ADR/architecture amendments and
retains integration accountability.

The inspected source is `e1de22a03`, plus concurrent wording fixes in proposed ADR-0233,
blueprint §21.7 and §24.1. These fixes are included explicitly rather than attributed to the
commit. The changes are compared with the prior governing text at `0681f48c7` and the selected
Plan59 A07, R3/R6 and RC01–RC05 scope. The current audit prerequisite is the
[host-contract refresh](../audit/plan59-host-contracts.md), publication revision `3afeae07a`.
Its [DA-039 host revalidation](../audit/plan59-host-contracts.md#host-projection-still-collapses-distinct-failures)
is the relevant diagnostic input; the wider finding retains its
[primary home](../audit/structure-and-invariants.md#da-039).

The applied standard is core 3.4, efficiency heuristics 1.0, process-simulator profile 1.5
and the pse-arrow binding selected by [standard.toml](../design_principles/standard.toml).
The scoped drivers are truthful automation, control despite source breakage, independent
failure/settlement facts, locally testable owners and reuse without another coordination
platform. No new numerical formulation, solver, physical model, database transition or
scientific acceptance predicate is proposed here. Those scientific contracts remain owned by
their native components.

**Interface-checked:** the current host errors, foreground settlement, manager capability
interfaces, main dispatch, explicit bootstrap and shared typed diagnostics were inspected.
**Proposed:** the amended contract and the future mechanisms identified below. No build, test,
service, fault injection, benchmark or scientific conformance campaign ran for this review.
Current code does not establish the pending managed envelope, control ledger or analysis-session
implementation. The review does not inspect their future algorithms or preapprove a new crate
or resident service.

**Publication reconciliation:** `3afeae07a` publishes the inspected wording fixes and audit.
The subsequent `5ee466c45` ports native command dispatch to clap; that later implementation
does not supply evidence for the envelope/control mechanisms considered here. Source statements
about the old main dispatch below refer to the expressly identified inspected baseline, not a
claim that the command-tree migration is still absent at publication.

## Responsibilities and governing distinctions — slots 2 and 3

The target's principal improvement is separating operations that share a CLI process but do
not share authority. The following is the assessed decomposition, not a second schema owner.

| Responsibility | Authority and consumed contract | Consequential distinction |
|---|---|---|
| Native host invocation | Host outer envelope, environment, prepared tool selection and actual OS-manager containment | Invocation identity is not a scientific run, process or observation identity. Supplied associations do not grant stop permission. |
| Original execution and settlement | Original child/resource/fixture owners and actual manager/process facts | Child exit, original physical drain, fixture disposal and scientific acceptance are independently observable. Ledger absence is not drain. |
| Product request and result interpretation | Explicitly prepared product owner, its grammar/schema and scientific rules | The host forwards selected bytes and agreed transport metadata; it does not acquire an Arrow/database/scientific decoder. |
| Existing-work control | Compatible prepared control generation plus authenticated original work identity | Read/wait/cancel may not require rebuilding broken current source. A recorded arbitrary path is not executable authority. |
| New-work construction | Explicit bootstrap and selected source/tool identity | A stale new-work route rebuilds and refuses if rebuilding fails. A successful unrelated old generation cannot hide that failure. |
| Source analysis | Selected provider and configuration/source generation | Query cancellation is not session termination; provider revision cancellation is not workload failure; watcher events are not an atomic snapshot. |

This model is adequate for the amendment because it governs verbs and failure interpretation,
not merely output field names. A launch can successfully acknowledge an operation that remains
running. An observation wait can successfully expire without changing its subject. Conversely,
a command asserting a terminal correctness result must obtain the selected terminal composer's
complete reconciled Pass. Missing required records cannot be promoted by exit zero, a valid
JSON schema or successful cleanup. Exec adapters continue to forward original child exits and
signals under their separate boundary contract.

The amended §24.1 now qualifies its earlier universal zero-exit claim as a **terminal
correctness-result claim**. R6 is therefore a refinement with a different success subject,
not an escape from terminal evidence requirements. Workload timeout, observation timeout,
cancellation and unknown settlement remain distinguishable. Optional observation loss is
reported; required verdict-evidence loss prevents a completed pass.

Diagnostic projection retains native typed causes, source facts and locations, including
independent cleanup failures. Neither the host nor an analysis adapter may infer semantic
category by parsing formatted messages. Product outcomes and scientific permission remain
under the product owner even when the host serializes their transport.

## Representative changes and failure scenarios — slots 4 and 5

1. **Cancel after a source-breaking edit.** A later CLI process authenticates the original
   invocation and a compatible prepared control generation; it controls only the original
   work. It does not compile current source or execute an arbitrary ledger path. The original
   resource owner still proves drain. This preserves an essential control path without making
   a stale binary the owner of new-work source selection.
2. **Successful launch or expired wait.** The command reports acknowledgement/observation success
   with the subject explicitly launched/running. A subsequent completed-result request with
   missing selected records cannot report Pass. The distinction remains present in machine
   output and exit selection rather than being recoverable only from prose.
3. **Child and cleanup fail independently.** Both original failures and the original child
   status remain available. Cleanup cannot overwrite a frozen assurance verdict or convert a
   failed resource release to a successful one. Conversely, a host cleanup failure must not
   be rewritten as a scientific solver outcome.
4. **A product grammar changes.** Its prepared product owner changes parsing and response
   interpretation. The host need not import generated product schemas or change scientific
   decoding. Transport failures remain host facts; invalid product requests remain product
   facts. Preparation and protocol incompatibility refuse before effects.
5. **Two queries share an analyzer session.** Canceling one query releases that query's interest
   without killing the session or another query. A source/configuration revision may invalidate
   an older provider snapshot explicitly. Retained tool generations and original children stay
   protected until actual session settlement, independently of client disconnection.
6. **A source query requests an effect.** Passive reads do not trigger compiler-private setup,
   generation or edit application. Explicit probes/preparation/application have their own effect
   and evidence boundaries; a preview is not proof of atomic multi-file publication.

These scenarios justify separate contracts without requiring a general operation catalogue,
second database, permanent admission daemon or per-query compiler construction. Shared prepared
generations and reusable provider sessions avoid repeated preparation when premises hold.
Generation pruning must be based on actual use protection and settlement, not an age count or
the disappearance of a CLI row. This qualitative execution judgment claims no measured latency,
throughput or memory benefit and introduces no new host limits.

## Current source, finding and alternatives — slots 7 through 9

### F01 — Preserve failure structure when adding the envelope

**Interface-checked; implementation obligation, existing DA-039.** In
`crates/pse-host/src/lib.rs`, every `HostError` variant projects to
`RuntimeInfrastructure`, with no forwarded diagnostic children. `Settlement` nevertheless
retains the original operation and cleanup error fields. In `foreground.rs::settle_result`,
successful child status plus failed cleanup is formatted into an `Ownership` string.
`main.rs` prints display text and exits 125. A03 must not carry that collapse into a new
machine envelope. The trigger is a deadline/refusal/uncertain launch or independently failed
cleanup; the consequence is lost actionable meaning despite structurally valid output
(AP-04/AP-05, G2/G7).

The correction is source-owned typed projection with preserved facts and independent outcome
dimensions, using `pse-diagnostics::TypedDiagnostic::{diagnostic_facts, diagnostic_children}`
and its explicit child traversal where applicable. The original fields remain the source of
meaning. Tests must distinguish categories and retain both operation and cleanup failures,
including successful child status with failed cleanup; display equivalence is insufficient.
A07 specifies that correction but does not implement or close DA-039.

### Existing mechanisms and library fit

`manager.rs` provides owned/borrowed capabilities and actual stop-and-drain behavior;
`foreground.rs` preserves original code/signal handling and dual failures. Explicit bootstrap
already constructs a private generation, probes tools and atomically changes `current`.
These are concrete reusable mechanisms. The shared typed diagnostic interface separates
semantic facts from miette presentation and gives the host a lower-owner integration seam.
No new library-specific capability or API behavior is assumed by this amendment.

There are equally concrete limits. `foreground_placement` currently requires an application
`.scope`, and `require_reference` checks the current bus client's unit reference. They are
not evidence of detached service entry or later-client stop authority. A05/B03 must implement
and test the selected service/control path rather than treating the old capability as already
substitutable. Likewise, an analyzer session needs its actual provider snapshot and original
child/tool lifetime; naming a session record does not establish those guarantees.

| Alternative | Judgment for the scoped target |
|---|---|
| Rebuild for every command, including existing-work control | Simpler dispatch, but source breakage prevents stopping or observing previously launched work. Reject for the declared workflow. |
| Execute any recorded old tool path | Avoids rebuilding but gives mutable metadata executable authority. Reject; prepared generation compatibility and original identity remain required. |
| Host parses every product and analyzer result | Centralizes entrypoints while duplicating semantic ownership and widening the lean host's dependency/test closure. Reject. |
| One success bit or schema-valid envelope | Cannot represent acknowledgement, child exit, verdict and settlement independently. Reject for semantic loss. |
| Separate coarse transport, original resource ownership and prepared product/provider interpretation | Select the amendment. It preserves useful native mechanisms and confines independent changes to their owners. |

Detailed control-authentication, pin/disposal, service-entry and provider-selection algorithms
are intentionally not accepted here. Their realization is scheduled work rather than an
unbacked claim of an available capability.

## Foundation and gate judgments — slot 6

These verdicts assess the declared **Proposed semantic amendment**. They do not assert that
current source passes the same implementation checks; F01 describes a current source gap.

| Foundation | Verdict and reason within this scope |
|---|---|
| AP-01 | **Satisfied:** scientific interpretation, transport, physical settlement and analysis revision have different owners and reasons for change. |
| AP-02 | **Satisfied:** consumers require explicit protocol/context compatibility rather than incidental current executable or product types. |
| AP-03 | **Satisfied:** composition connects native owners; invocation identity or a CLI entrypoint does not confer another owner's authority. |
| AP-04 | **Satisfied:** acknowledgement, observation, child outcome, assurance and settlement are consequential modeled distinctions governing verbs; product/provider interpretation remains authoritative. |
| AP-05 | **Satisfied:** identity associations, required evidence, incompatible controls and effects have explicit meanings and refusal obligations rather than success sentinels. |
| AP-06 | **Satisfied:** transport/diagnostic cases need no scientific runtime or database; product grammar and provider revision behavior can be checked at their owners. Actual OS ownership still needs OS tests. |
| AP-07 | **Satisfied:** existing-work control avoids unrelated reconstruction; reuse protects actual generation/session users, with no added global coordinator or repeated product parsing. No performance claim is made. |

| Gate | Verdict and decisive contract |
|---|---|
| G1 Authority | **Pass:** host, product, provider and original resource owners remain distinct; projections confer no new authority. |
| G2 Semantic fidelity | **Pass:** independent identities/outcomes and original diagnostic structure survive projection; optional loss is explicit and required evidence cannot disappear into success. |
| G3 Validity | **Pass:** incompatible controls, forged paths and missing required verdict evidence have explicit refusal/incomplete outcomes before their dependent operation. |
| G4 Hidden behaviour | **Pass:** current-source construction is confined to selected new work; source reading, existing-work control, probes and edit application have distinct effect contracts. |
| G5 Consistency and recovery | **Pass:** partial/unknown settlement is not committed completion; immutable prepared controls and actual-use protection are required, with no blind relaunch authority. |
| G6 Transformation and reuse | **Pass:** product projection and analysis reuse retain their original semantic/context scope; watchers do not imply atomic snapshots. |
| G7 Truthful capability claims | **Pass:** acknowledgement is not terminal Pass; source candidates are not actual tests; static calls are not runtime causality; pending mechanisms remain Proposed. |
| G8 Library leverage | **Pass:** reuse shared typed diagnostics and existing native manager/child boundaries; the amendment introduces no competing generic implementation. |
| G9 Architectural fitness | **Pass:** all applicable foundations are satisfied for the six assessed scenarios; the scoped change adds no avoidable cross-owner scientific or database coordination. |

PS-G1/PS-G2 are **not applicable**: this amendment constructs no physical quantity,
formulation or solver input. PS-G3 is **not applicable to numerical integrity qualification**:
no solve is performed or independently verified here. Its PS-10 boundary obligation is retained:
host acknowledgement, exit or cleanup cannot confer scientific acceptance or reinterpret a
solver outcome. Existing scientific verification remains required at its actual owner.

## Required follow-through and rule impacts — slots 10 and 11

[Plan59](../../plans/59-agent-cli.md) owns disposition and qualification, with no duplicate
progress ledger here. The following are implementation checks still required, **not_run** in
this review:

- A03: acknowledgement/running/terminal/incomplete outputs; selected missing reports;
  timeout/cancellation distinctions; non-ASCII source locations; typed facts and independent
  operation/cleanup failures; original exec child status forwarding.
- A05/B03: a later client controls original work after current source is broken; incompatible
  or forged generation/unit identity refuses; real service-entry nested execution and original
  child drain; generation retention until actual adapter/supervisor/finalizer/fixture settlement.
- F01 and analysis consumers: actual prepared product delegation; independent passive source
  reads; provider/configuration/source scope; query cancel versus session stop; revision
  cancellation and retained tools/children; explicit preparation/probes/application.

The root corrected three contradictory readings before this judgment: §24.1's initial
terminal-pass rule now qualifies its subject; ADR-0233's rebuild clause names new-work verbs;
§21.7 distinguishes optional native product interfaces from Plan59's selected CLI. They need
to remain in the published amendment together. ADR-0233's Plan58-oriented verification and
scenario pointers should also be extended to the owning Plan59 A07/A03/A05/B03/G obligations;
that is a publication/navigation correction, not another substantive architecture choice.

The relevant rule impacts are precisely the operator-confirmed R3/R6 and RC01–RC05 amendment
to proposed ADR-0233 and blueprint §21/§23/§24. This review proposes no additional rule change,
SHOULD deviation, accepted-ADR status change, new crate or service. Later investigations that
propose such changes retain their own scoped decision route. Review acceptance does not itself
edit or supersede authority.

## Decision — slot 12

**Behavioral/semantic adequacy: Accept at Proposed evidence for A07's contract amendment.**
All applicable core gates pass for this declared scope; no unresolved MUST or silent semantic
loss remains in the corrected target wording.

**Architectural fitness: Accept at Proposed evidence for the same scope.** All applicable
foundations are satisfied. The target preserves original resource ownership and product/provider
semantic authorities without requiring a second runtime coordination system.

**Overall: Accept, limited to the declared A07 target contracts.** This is not the template's
SHOULD-exception decision: no deviation is taken. Publication of the corrections and this
review can satisfy A07's scoped review prerequisite. Actual mechanism readiness, DA-039
resolution, runtime qualification and whole-plan completion remain unresolved implementation
obligations under the named Plan59 owners. ADR-0233 remains proposed.
