---
title: "Plan69d: Routine product and data operations"
status: planned
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

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| D01 | planned | A01–A03, B02 prepared route, current product read owner | Bounded typed reads through one front door with authenticated target selection and passive discovery. |
| D02 | planned | D01 dispatch and target selection, original product effect owner | Original receipt and stable recovery through duplicate, stale and uncertain acknowledgement controls. |
| D03 | planned | D01 and each exact query or exchange owner, D02 for writes | Supported data operations migrate. Each absent owner capability has a recorded prerequisite or deferral. |
