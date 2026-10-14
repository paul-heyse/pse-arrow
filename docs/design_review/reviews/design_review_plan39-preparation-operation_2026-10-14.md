---
title: Plan39 finite strict preparation operation
date: 2026-10-14
tier: change
purpose: target
status: review
standard: Core 3.4
heuristics: Efficient Architecture 1.0
profile: Process Simulator 1.5
baseline: 42f9c506a
---

# Plan39 finite strict preparation operation

**Decision: Accept the target at Proposed strength.** One finite preparation operation is the appropriate owner for dependency setup, the selected executor-family refresh and compilation evidence shared by participating captures. It removes a demonstrated sequencing conflict without weakening scientific identity, source-qualified I/O review or original-path deployment freshness. This assessment accepts a proposed architecture; it does not qualify its implementation or an installed simulator.

The reviewer is the fresh independent `/root/review39_preparation` agent, separate from the coordinator authoring the amendment and subsequent implementation. The inspected baseline is main `42f9c506a` plus the proposed “One finite preparation operation” amendment in [ADR-0189](../../adr/0189-pure-macro-producer-channels.md). Current follow-up and final acceptance belong to [Plan39](../../plans/39-implementation-friction-and-codex-effectiveness.md), including its 39d qualification owner. This review is not a second disposition ledger.

## 1. Scope, target and evidence

The target is a development loop in which a responsibility can be compiled and tested with its actual dependencies, alongside truthful selective scientific producer qualification. Strict preparation must assemble runtime, worker and Python roles whose original consumed premises are still current. Ordinary Local admission remains a separate supported path. Numerical models, physical formulation, solver algorithms, study behavior and scientific qualification are outside this bounded change.

Apply Core 3.4, Efficient Architecture 1.0, Process Simulator 1.5 and the pse-arrow binding. Relevant drivers are AP-03/AP-04/AP-07, DP-09/DP-19 and H1/H12/H18–H20/H23–H25: reuse established facts at their actual validity lifetime, while retaining the independent checks that establish a different property.

**Interface-checked:** directly inspected `xtask/src/producer_identity.rs` selection, refresh, stream, capture, source/configuration consistency and unit-key paths; `scripts/producer_deployment.py` role sequencing and final verification; `scripts/producer_cargo.py` single-use actual Maturin child adapter; and `crates/pse-buildinfo/identity.rs` deployment association verification. Also read the current architecture's blueprint §5.3 contracts, ADR-0189 and its preceding guarded-macro review. No tests, builds, services or quantitative measurements were run for this review. Historical test claims were not independently rerun or adopted as new evidence.

## 2–5. Responsibility model and revealing scenarios

There are three distinct facts: a selected executable was compiled in an admitted exact context; its reviewed external input channels are complete; and the deployed role plus its original consumed inputs remain current. A successful build, a Fresh flag or protected historical bytes cannot establish all three. The proposed operation owns only the first fact's finite sharing scope and the ordering of effects that would destroy it. Existing source-review and deployment-verification owners retain the other decisions.

| Owner | Intended responsibility and consumed contract |
|---|---|
| Preparation coordinator | Perform dependency setup, enroll the finite family union, refresh once before captures, own operation lifetime and failed/completed outcomes. |
| Rust producer capture | Interpret the actual graph and Cargo messages; establish exact executor compilation contexts and complete consumed premises; admit same-operation reuse or refuse. |
| Actual installer adapter | Bind the real Maturin Cargo argv, working directory, SDK/configuration and environment; it cannot substitute earlier planned intent or manufacture qualification. |
| Historical evidence owner | Seal raw actual associations and protected bytes for later reconstruction; sealing grants no current-path validity. |
| Deployment verifier | Independently verify current artifact/input bytes, canonical paths, absences, namespaces and native environment. |

**S01 — Overlapping role dependencies.** At baseline, `run_selected` can perform `cargo clean` for the selected executor package-name families for each role. The implementation correctly recognizes that Cargo cleaning acts on name families rather than exact version/source IDs. Later role cleaning can therefore remove earlier `.d`, generated or native output paths. `capture` then verifies all role associations against current paths. Sealed history preserves historical evidence but cannot repair current validity. Move this cleanup before any role capture, without removing the final verification.

**S02 — Actual Python context differs from planned intent.** Maturin's observed child can add a feature, target, SDK or effective input difference. The existing adapter validates its actual argv, rejects a different Cargo selection and seals that actual realization before installation continues. Keep that boundary. Late enrollment may introduce a new exact compilation context within an already refreshed family. It must obtain its own first non-Fresh compilation evidence; it cannot inherit a package-wide grant. A newly required family that would require cleanup ends this operation before build, rather than cleaning beneath completed captures.

**S03 — Repeated exact context.** Cargo may report an executor Fresh in a later role. Admit it only against a preceding successful non-Fresh compilation in this operation with the same complete actual context and unchanged consumed premises and outputs. `unit_keys` incorporates selected dependency contexts but does not include all raw source, effective environment, native or absence facts. It is useful association evidence, not a sufficient permission key. Likewise, package ID plus `fresh:false` does not by itself establish which selected executor context was compiled.

**S04 — Failure, interruption or concurrent mutation.** A child that fails, emits incomplete/ambiguous associations or leaves unsettled writers cannot enroll completed compilation evidence. The operation is terminal after failure and a retry starts a new private operation; protected earlier evidence may remain for diagnosis. Hold operation ownership through its actual writers and final reconciliation. Preserve existing coordinated exclusion and before/after checks; a private marker does not make arbitrary concurrent Cargo tools safe. This change makes no adversarial same-user isolation claim.

The baseline's current verifier is an important preservation constraint. It checks original built and installed artifacts, exact file observations, missing entries, namespace digests and native environment under an operation checkpoint. Protected copies, a matching scientific key or a completed operation marker must not replace these checks. If a later legitimate build changes an earlier consumed namespace or output, final current verification still refuses; this review does not authorize namespace pruning or receipt rewriting to avoid that result.

## 6. Architectural and gate assessment

These verdicts assess the proposed target including the operation and reuse constraints above, rather than treating the unmodified baseline as accepted implementation.

| Foundation | Target verdict and rationale |
|---|---|
| AP-01 | Satisfied: sequencing and bounded compilation reuse belong to preparation; channel review and current verification remain with their existing owners. |
| AP-02 | Satisfied: roles consume admitted exact compilation evidence, without interpreting package cleaning or installer planning as qualification. |
| AP-03 | Satisfied: role captures compose under one effect lifetime instead of independently invalidating shared prerequisites. |
| AP-04 | Satisfied: operation identity, family refresh, exact compilation context, review completeness, historical evidence and current deployment are distinct governing meanings. |
| AP-05 | Satisfied: enrollment, first compilation, permitted reuse and terminal failure are explicit constraints, not unchecked script order. |
| AP-06 | Satisfied: tiny real compiler fixtures can attack this mechanism without a store, scientific model or full native deployment. |
| AP-07 | Satisfied: one required fresh baseline supports repeated exact contexts within the operation; role boundaries no longer force repeated destructive refresh. No speedup is measured. |

G1–G4 pass at Proposed strength: authority and actual context are explicit, unsupported context is refused, and preparation effects remain visible. G5 passes with owned terminal outcomes and final reconciliation. G6 passes only for dependency-complete same-operation reuse; no persistent cache permission is introduced. G7 passes because compilation, complete external-input assurance, historical reconstruction and actual qualified installation remain separate claims. G8 passes: Cargo owns compilation, graph selection and package cleaning; Maturin owns installation; no compiler or installer is reimplemented. G9 follows the satisfied foundations at target strength. PS-G1–PS-G3 are not applicable because this change does not alter or certify physical formulation, well-posedness or numerical outcomes.

## 7. Finding

<a id="f01"></a>
### F01 — Role-local refresh destroys the assembled operation's current premises

**Diagnosis — Interface-checked.** `run_selected` performs family cleanup inside each selected role capture when `refresh_executors` is selected. `producer_deployment.capture` invokes roles sequentially and only afterward independently verifies every association. Shared family cleanup can remove an earlier role's consumed original paths. This conflicts with AP-03/AP-07, DP-19 and S01: the semantic role boundary is the wrong physical lifetime for this destructive prerequisite.

**Consequence.** An individually completed role and its sealed history can become unusable for the required assembled current deployment because another legitimate role refreshes overlapping package families. Repeating captures relocates the conflict rather than fixing its lifecycle.

**Correction — Proposed.** Enroll and refresh the operation's selected family union once after dependency setup and before capture. The Rust owner admits exact first compilation and subsequent same-operation reuse under complete actual premises. Keep final current verification, actual Python child authority and original evidence interpretation. Priority is correctness and feasibility of strict assembly; exact-context/lifecycle contracts are prerequisites to enabling reuse.

**Verification.** A small real Cargo fixture with overlapping role families must preserve earlier current inputs across later capture. Distinct feature/platform/dependency/SDK contexts require distinct first compilations. A Fresh artifact from an earlier operation, unknown family, failed child, changed source/environment/native/absence premise, altered output or incomplete graph/message association refuses. Build-script output-message association is tested separately from executor compilation; a replayed `build-script-executed` message alone cannot establish execution in that child. An operation failure cannot be resumed by presenting its leftover marker. These controls establish the mechanism only; actual PSE all-role capture and import remain final validation.

## 8–10. Alternatives and support limits

Keeping role-local clean is simpler locally but cannot support the assembled current-path contract reliably. Removing refresh or accepting any Cargo Fresh executor would weaken the selected assurance. A persistent qualified compiler cache would require a wider retained-evidence, invalidation and recovery scope that this target does not need. Copying original paths to protected storage and rewriting receipts would change current deployment meaning and is rejected.

The finite operation is the simplest conforming choice: retain existing Cargo/Maturin capabilities and existing evidence, add only an operation-scoped owner and the exact-context admission needed for its concrete reuse. An internal protocol between Python orchestration and Rust capture is justified; it must not become a second implementation of capture qualification. Source/tool upgrades can change supported contexts and require review; unknown channels stay conservative. No numerical performance conclusion, universal Cargo support or successful native qualification follows from this assessment.

## 11. Rule impact and disposition

<a id="rc01"></a>
**RC01 — Move fresh-executor assurance from each role's destructive refresh to one finite preparation operation.** The affected current realization is Plan39b B4/39d strict preparation and blueprint §5.3, recorded by the proposed ADR-0189 amendment. Preserve the requirement for first actual fresh compilation, now per exact context within the operation; permit later Fresh use only from preceding operation-owned compilation with unchanged complete premises. F01 and the target depend on this change. If the role-local rule is kept, no safe multi-role reuse is granted and the current destructive sequencing conflict remains to be corrected by another design. The operator authorized this bounded execution target; formal ADR status and final implementation acceptance remain separate.

The original guarded macro-channel and source-pin decisions are preserved, not reopened. No additional rule impact is proposed. Plan39 owns implementation status and subsequent acceptance; this document records the independent bounded judgment.

## 12. Decision and coverage limits

Behavioral adequacy and architectural fitness **pass at Proposed strength** for the finite operation target with the explicit admission and failure constraints stated here. Overall **Accept the target**; implement F01 through its existing plan owner. No source-inspected design premise requires weakening native completeness or original-path verification.

Not examined: actual PSE all-role strict captures, successful qualified Maturin installation/import, admitted ELF transformation and imported-byte association, native closure completeness, service or cancellation campaigns, full hygiene, scientific journeys or measurements. Those remain with Plan39d. No test or measurement claim is made by this review.
