---
title: "Plan69d: Routine product and data operations"
status: in-progress
date: 2026-11-07
adrs: []
doc_role: plan
doc_owner: docs/plans/69d-routine-product-and-data-operations.md
doc_topics: [current-work, design, storage]
---

# Routine product and data operations

This companion owns [O05](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O05).
The project operator already supports bounded metadata, project creation, typed navigation and
offline pack inspection. It distinguishes an offline pack from a configured canonical database.
Build the CLI front door over existing product authority. Plan66 owns product and scientific
semantics. The [coordinator](69-intent-first-cli-and-codex-workflows.md) owns assembled acceptance.
[Current project operator](../../crates/pse-runtime/src/bin/project/tools.rs) ·
[Blueprint §21.7](../authoritative_design/sections/native-host-and-engineering-tools.md#section-21-7)

## D01 — Bounded discovery and project reads

Start with project discovery and metadata, branch and draft inspection, cursor-bounded listing,
and offline pack inspection. Record the typed request, result, size or page bounds and exact
authority for each existing operation. Compare a prepared `pse-project` dispatch with its
existing typed transport. Prefer prepared dispatch if it carries typed requests, original
receipts and diagnostics without caller-managed files or duplicated startup. Use the existing
transport if it preserves those contracts with less repeated setup. Keep `pse-host` independent
of product and scientific dependencies. If neither path can preserve the acknowledgement,
schedule the narrow owner handoff before exposing the affected operation.

Use A02 to resolve an authenticated project or store. Offline pack inspection needs no service.
Ambiguous online selection returns valid authorized choices before the operation. Online read
uses a prepared connection-only client. Discovery, help and read cannot start the configured
database. Preserve generation-fenced cursors and refusals. Test offline operation, stale cursor,
unique target and ambiguous target, including an assertion that neither read path starts a service.

Where selection crosses a process boundary, receive A02's validated target and
its identity or revalidate the same premise at the owner. Use A01 typed syntax
for arguments and local alternatives. The owning transport retains target authority.
Check that dispatch preserves the resolved selection and its original acknowledgement.
[Catalog and resolved context](69a-cli-contracts-configuration-and-presentation.md)

## D02 — Typed writes and original acknowledgement

Expose project creation and supported navigation or branch/draft effects through the typed
owner. Accept concise typed arguments, stdin or an explicitly authored intent document where
it carries required meaning. Absorb any implementation-specific request staging into the
performing owner and remove the former manual handoff from routine guidance. Use A03 for
receipts, stale premises, changed duplicate inputs and unresolved effects. Preserve exact
target and permission checks before the first write. Report authoritative references on success.

Where the owner supports it, allocate a stable operation identity before submission, expose
it for recovery, and accept a caller-supplied external key when meaningful. Reconcile uncertain
acknowledgement against that same ID. Repeated recovery observes the original effect or an
explicit unresolved outcome. It cannot silently allocate a new ID or retry under changed
inputs. If an owner requires a supplied identity to guarantee recovery, keep that required
until the handoff can guarantee the same behavior. Exercise ambiguous selection with zero
writes, wrong identity, a duplicate with changed input, and interruption after submission.
Assert that the receiving A03 envelope retains owner code, typed target and operation
facts, settlement, and an exact safe retry or reference. Distinguish an unresolved
acknowledgement from refusal before mutation.

### D02 owner checkpoint

The canonical owner binds `create_project`, `project_navigate` and
`project_acknowledgement` to the exact request, store identity and original operation key.
The captured binary offers the separate bounded `perform` protocol. The CLI checks that
the capture declares that protocol before submission. The owner verifies the expected
issuer, projects typed original failures, and omits the opaque request bytes from the
receipt. An absent acknowledgement leaves settlement unproved. The recovery command
observes the same key. Navigation can read the original project descriptor and typed action
from an authored input or stdin. The passive observation channel retains its separate contract.
[Canonical project owner](../../crates/pse-operations/src/canonical_projects.rs) ·
[Existing project operator](../../crates/pse-runtime/src/bin/project/tools.rs) ·
[Captured transport](../../crates/pse-runtime/src/tooling/reader.rs)

## D03 — Supported queries and exchange

Inspect the existing query, pack and product owners for bounded data queries, typed export,
and typed import. For each candidate, record supported data scope, bounds, explicit authored
inputs, original acknowledgement and transaction or checkpoint unit. Expose supported queries
and inspection first. Adopt an import or export only when the owning action and required
admission exist. Name any unavailable capability and the missing owner contract in the plan
checkpoint. A pack inspector alone supplies no import effect.

Pass authoritative IDs to consumers. Export to stdout or a deliberate destination when the
caller requests a deliverable. For bulk writes expose the owner’s actual atomic or recoverable
checkpoint boundary, including completed steps and the exact safe continuation. A generic SQL
or shell escape cannot establish those guarantees for these actions. Compare the receiving
CLI with the original product action on bounded data, no-effect refusal and partial completion.
Replace recurring helper-file and wrapper sequences once the receiving controls pass.

If an export gains a useful progress display, receive A04's policy. Consider
indicatif 0.18.6 `ProgressBar::wrap_write` only for a real writer boundary where it
replaces a counter. Report its unit as accepted bytes. Flush, publication and
settlement remain owner facts. Verify that a full count can coexist with an
incomplete or failed publication.
[Presentation and adapter policy](69g-output-policy-and-owner-driven-progress.md) ·
[Pinned writer adapter source](https://docs.rs/indicatif/0.18.6/src/indicatif/iter.rs.html)

### D03 owner checkpoint

- **Bounded queries.** D01's prepared reader already returns typed project headers and pages
  of up to 64 metadata or branch rows with the issuing store and original continuation. Its
  offline pack inspection verifies the manifest and member extents and digests and returns the
  manifest and its checksum. The same reader already supports exact study and attempt observation
  with native cursors and up to 512 events per page. Additional passive routes can proceed
  independently of D02. `pse project study-workbench` delegates to
  `control_study_workbench_page` through the same captured reader and target selection. It
  defaults to 64 compact live headers, takes optional exact filters and accepts the original
  issuer- and predicate-bound cursor. The host parser and reader fixture cover the new route.
  `pse project research` reads up to 128 dataset headers per project/key seek page or
  one exact dataset or frozen selection through the authenticated connection-only reader.
  [Project reader](../../crates/pse-runtime/src/tooling/reader/canonical/project.rs) ·
  [Prepared reader](../../crates/pse-runtime/src/tooling/reader.rs) ·
  [Study browse](../../crates/pse-operations/src/canonical_control.rs) ·
  [Research read](../../crates/pse-runtime/src/workflow/research.rs)
- **Closure and export.** `ProjectClosure` and `ProjectExport` take an exact `ProjectSelection`,
  explicit member keys, original run/attempt and result row predicates, omissions, and document
  and copy allowances. The closure owner caps each primary key list at 1,024 and documents at
  64 MiB. `pse project describe-closure` returns authenticated metadata for an already-issued
  selection through the prepared connection-only adapter. Issuing that selection uses
  source admission. Executable closure and export require authored
  physical inputs that admit the same physical identity. Both exports require an intentional
  fresh destination and return the original manifest after verifying copied members and publishing
  `project.json` last. A partial directory may remain before publication. The D02/A03 effect
  handoff must preserve that distinction. Current packs bind integrity of the manifest and
  copied members. They carry no original operation key or digest of the complete authored
  request, so manifest readback alone cannot prove settlement of a particular submission.
  Before exposing a routine export route, bind the request and destination at the publication
  owner and make that association available on readback.
  [Source selection](../../crates/pse-runtime/src/workflow/projects.rs) ·
  [Closure](../../crates/pse-runtime/src/workflow/project_closure.rs) ·
  [Pack owner](../../crates/pse-runtime/src/workflow/project_portability.rs) ·
  [Manifest publication](../../crates/pse-runtime/src/workflow/portable_payload.rs)
- **Import.** `ProjectImportIssue` verifies an executable pack and returns the original immutable
  intent bound to the exact manifest digest, receiving store and fresh operation and destination
  IDs. Carry that intent to `ProjectImportStatus` and `ProjectImport`; the issue/apply document
  allowance defaults to 16 MiB. Apply rechecks the pack, admits the source and binds the exact
  evidence artifact before copying it. The original `ProjectImportReport` distinguishes acknowledged
  source from completed project membership. `ProjectImportRecover` takes the same intent and
  original artifact header and returns exact pending, collected or completed state. `ProjectImportOpen`
  requires completed membership and reopens admitted source. D02/A03 must preserve those original
  identities, diagnostics and safe continuations. The prepared receiver exposes
  `import-issue`, `import`, `import-status` and `import-recover`. It retains the exact issued
  intent in private checkout custody and uses it by default. `import-forget` releases it
  after exact completion is observed. `ProjectImportOpen` still needs a distinct protected
  physical-admission route.
  [Original action types](../../crates/pse-runtime/src/host_operations/project.rs) ·
  [Action dispatch](../../crates/pse-runtime/src/host_operations/project/handler.rs) ·
  [Import checkpoints](../../crates/pse-operations/src/canonical_project_imports.rs) ·
  [One-shot operator](../../crates/pse-runtime/src/bin/project/runtime.rs)
- **Study deliverables.** `StudyCohort`, `StudyResult` and `StudyAnalyze` share a protected
  frozen cut of at most 64 studies and 10,000 members. Cohort pages and selected result IPC
  use explicit byte allowances and a staged destination. Analysis uses an exact registered
  output and row interval and a 4 MiB IPC encoder bound. Preserve the original capture receipt,
  output publication facts and capture-owner lifetime in an explicit effect adapter. The receipt
  identifies the cut for attribution and grants zero ability to reopen its protected payload.
  [Cohort and analysis owner](../../crates/pse-runtime/src/workflow/study_workbench.rs) ·
  [Selected result export](../../crates/pse-runtime/src/workflow/connected_results.rs)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| D01 | in progress | A01–A03, B02 prepared route, current product read owner | Connection-only reads and the target identity query are wired. Named selections carry expected issuer and database, and ambiguous targets return authenticated choices. The assembled receiving journey remains. |
| D02 | in progress | D01 dispatch and target selection, original product effect owner | Typed create, navigation and exact acknowledgement routes are implemented. The live owner control passed for wrong issuer, duplicates, stale navigation and receipt recovery. The assembled captured CLI journey remains. |
| D03 | in progress | D01 and each exact query or exchange owner, D02 for writes | Live project-owner controls passed for bounded research, descriptive closure and original import recovery. Assembled CLI import verification, protected import open and original publication receipts for export and study deliverables remain. |
