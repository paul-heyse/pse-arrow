---
title: "Plan59 native host contract audit refresh"
date: 2026-10-29
doc_role: review
doc_owner: docs/design_review/audit/README.md
doc_topics: [design, environment, diagnostics]
---

# Plan59 native host contract refresh

This is the diagnostic prerequisite for the scoped A07 review of invocation control,
diagnostic projection, product delegation and reusable analysis context. It does not refresh
the whole-system account or reopen scientific design work.

## Baseline and coverage

**Interface-checked:** source `e1de22a03` on 2026-10-29, plus the concurrent wording corrections
in proposed ADR-0233, blueprint §21.7 and §24.1. Those corrections distinguish terminal
correctness-result success from acknowledgement/observation success, restrict automatic
rebuilding to new-work verbs, and distinguish optional native product interfaces from the
selected Plan59 agent CLI. The audit publication revision is separate from this source state;
Git records its publication.

The selected standard is core 3.4, efficiency heuristics 1.0, process-simulator profile 1.5
and the pse-arrow binding, through [standard.toml](../design_principles/standard.toml).
The profile's truthful outcome requirements inform the boundary; no physical model,
formulation, solver or scientific result is audited here.

Inspected owners are `pse-host`'s library error, main dispatch, foreground settlement and
manager capability interfaces; `pse-diagnostics`' typed facts/children; explicit host
bootstrap publication; the A07 contract amendments; and Plan59's R3/R6, RC01–RC05 and
A03/A05/B03 acceptance boundaries. The independent source assessment follows the current
audit's [DA-039](structure-and-invariants.md#da-039) into its host footprint. Older runtime,
canonical-store and service manifestations of that finding retain their original baseline.
No build, test, service, fault injection or benchmark ran. Current source is not evidence that
the pending CLI ledger, generation authentication or analyzer-session mechanisms exist.

## Revalidated diagnostic cause

### Host projection still collapses distinct failures

**Interface-checked; existing [DA-039](structure-and-invariants.md#da-039), host footprint only.**
`crates/pse-host/src/lib.rs` retains distinct `HostError::Ownership`, `Deadline`,
`UnknownLaunch` and `Settlement` variants, but its `impl_diagnostic!` returns
`RuntimeInfrastructure` for every variant and forwards no diagnostic children or facts.
`Settlement` keeps the original operation and cleanup errors in separate fields, so their
existence is preserved internally; its public typed projection does not expose that structure.
The inspected `main.rs` prints the display error and exits 125 rather than producing the
planned managed envelope.

The distinction matters when an ownership refusal, observation deadline or uncertain launch
reaches a machine consumer: display text cannot safely supply category, original identity or
independent cleanup structure. The original host outcome must not be replaced by a generic
success/failure field. `foreground.rs::settle_result` preserves two actual errors in
`Settlement`; its success-plus-cleanup-failure branch instead puts the child status in an
`Ownership` string. A03 must preserve the independent child and settlement dimensions when it
adds the outer projection. Publication of A07 is not resolution of this source finding.

Contrary evidence matters: `pse-diagnostics::TypedDiagnostic` already exposes
`diagnostic_facts` and `diagnostic_children`, and `diagnostic_leaves` walks explicit child
occurrences. The shared owner supplies the structured integration seam; a second string
classifier or recursively invented host vocabulary is not required by this gap. Plan59 A03
owns the scheduled correction and its targeted verification.

## Strengths and changed contract significance

- **Interface-checked:** `manager.rs` separates owned stop authority from borrowed observation,
  checks native unit identity and keeps physical `stop_and_drain` distinct from manager replies.
  These are useful existing boundaries rather than a reason for a new global coordinator.
- **Interface-checked:** `foreground.rs` keeps original code/signal handling and combines
  operation and cleanup failures. The new command/subject distinction must retain that behavior.
- **Interface-checked:** `scripts/bootstrap-host` uses explicit construction, a private
  generation, tool probes and atomic `current` publication. This supports immutable prepared
  generation selection, but establishes neither the pending per-use pins nor cross-process
  recorded-generation authentication.
- **Proposed:** A07 distinguishes invocation, scientific, process and observation identities;
  a successful launch acknowledgement or expired bounded observation wait is not a completed
  correctness pass. This removes the contradictory universal terminal-pass reading without
  weakening requirements on a command that asserts terminal correctness.
- **Proposed:** product owners keep request/schema/scientific interpretation. Analysis providers
  keep revision and configuration meaning. The host retains containment and transport. These
  are materially different responsibilities despite sharing one CLI entrypoint.

## Implementation uncertainties retained

`manager.rs::foreground_placement` currently admits a `.scope`, while `require_reference`
checks the current bus client's reference. These existing mechanisms do not by themselves
establish detached service entry or a later client's authenticated right to control recorded
work. A05/B03 must establish the compatible control generation and original unit authority
without trusting an arbitrary recorded executable path or rebuilding broken current source.
No claim is made that these pending capabilities are defective merely because they have not
yet been implemented.

Analysis sessions additionally require real retained tool/child ownership, source and
configuration context, and distinct query cancellation/session termination. Contract publication
alone cannot establish their lifetime, snapshot or provider behavior. The selected G work and
its A05/B03 slices retain those obligations.

No new independent diagnostic finding is added. DA-039 remains at its existing primary home;
this page supplies the current host revalidation and the bounded review baseline. DA-043,
DA-057 and the rest of the whole-system account were not revalidated or resolved here. Current
work and completion evidence remain in [Plan59](../../plans/59-agent-cli.md), not this audit.
