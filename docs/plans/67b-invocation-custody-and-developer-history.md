---
title: "Plan67b: Invocation custody and fresh rerun"
status: in-progress
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/67-storage-lifecycle-and-build-reuse.md
doc_topics: [design, diagnostics, storage, validation, current-work]
review_sources: [docs/functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md]
---

# Invocation custody and fresh rerun

## Target and foundation

Deliver [O02](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#O02).
An ordinary invocation keeps its working inputs through actual settlement. A developer can
rerun a selected failure from its working report after the old executable is released.
Once the report is released, run the named current test or campaign again. The maintainer
deferred [O09](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#O09)
on November 7 in favor of fresh evidence.

**Interface-checked at `c2126df45`.** The native capture owner authenticates an opened original
and refuses if it changes during capture. Its private image and `.capture.json` tie bytes to the
original source and executing path. The performing report carries normalized selection, original
observations, required-capture coverage, and exact inventory commitments. Current failed rerun
uses authenticated report and inventory facts to form a new selection. It maps package, target,
features, binary ID and test name against the fresh native listing, and authenticates the new
executable. There is already a test control for rerun after old executable disposal. Build on
those responsibilities and preserve the distinction between an original native observation and
current authority. [Capture owner](../../crates/pse-host/src/discovery/capture.rs) ·
[Performing report](../../crates/pse-host/src/test_workflow/execute/report.rs) ·
[Current rerun](../../crates/pse-host/src/test_workflow/source.rs) ·
[Rerun controls](../../crates/pse-host/src/test_workflow/execute/runner/rerun_tests.rs)

[Plan66c](66c-preparation-execution-and-service-lifecycles.md) supplies real settlement.
[Plan66e](66e-protected-data-and-result-delivery.md) remains the authority for protected
scientific data, result delivery, and selected raw custody. [Plan66f](66f-developer-operations-clients-and-evidence.md)
owns native developer verdicts, report interpretation, and rerun selection. Reuse its verified
passive observations and compact delivery. [ADR-0252](../adr/0252-ephemeral-developer-evidence-relations.md)
defines finite queries over explicitly acquired developer evidence. Keep that interface, current
physical state and protected scientific results with their respective owners.

## Active executable custody

Reuse a selected installed tool under its authenticated generation, where its real launch and
loader contract fit. The same native invocation should borrow one verified built image across
enumeration and run if the performing owner can establish stable access for that whole interval.
Acquire ownership and seal the image identity before it becomes discoverable to consumers.
Where this cannot hold for a mutable Cargo output, capture one private image for the active
operation and release it after the last borrower and actual descendants drain. Retain the original
mode, dependency closure, source association, refusal behavior, and exact handling of cancellation
and uncertainty. Track file lifetime separately from current process and database settlement.

Compare protected digest sharing only for identical bytes across multiple live owners that still
need private images. Require immutable source ownership, independent mode and identity checks,
no mutation or unsafe path alias, and a release protocol through the actual custody owner.
The ext4 probe refused reflinks. Use the existing verified streaming path for cross-device
export. Prefer the native list and test flow for ordinary rerun. Consider
`cargo nextest archive` and `run --archive-file` only if a named transport consumer needs
relocation, and verify the pinned Nextest release and completeness of the associated resources.
An archive brings its own retained bytes and cannot establish fixture or runtime admission.
[Nextest tool pin](../../.config/diagnostic-tools.toml) ·
[Nextest archive contract](https://nexte.st/docs/ci-features/archiving/) ·
[Original-file delivery](../../crates/pse-host/src/developer_evidence.rs)

## Fresh evidence and retirement

Use the original native report and its associated inventory while the invocation owns them.
The failed rerun route selects the original failed identity against a newly admitted inventory.
An unavailable or changed identity stays unavailable. After cleanup, select the case or campaign
from its maintained source and run it again. Record the new revision, tool and fixture selection,
coverage, observed outcome and settlement. Compare separately produced current runs through the
existing finite reader when the investigation needs the comparison.

Release settled incident working results and any interim exports, including cited results, under
[67a](67a-plan66-consolidation-and-incident-cleanup.md). A citation identifies a test or campaign
to regenerate. Actual resource use, unresolved settlement and separately owned published data keep
their original custody. The November 7 decision withdraws the unpublished structured history
pilot. Existing scoped SurrealDB fixtures keep their native lifecycle and current evidence roles.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| B01 | in-progress | Current capture, report and Plan66 C/E/F owners | Choose the active custody path per tool and consumer. Specify the working report, rerun and release boundary. |
| B02 | in-progress | B01 active custody and working settlement interface | Migrate ordinary inventory/launch borrowers and remove their repeat retained copies. Release private images after drain and preserve explicit holds. Verify distinct compiler contexts, mutation/alias/load failures, interrupted publication, and later observation after a target clean. |
| B03 | deferred | A concrete future consumer and separate decision | Structured history pilot withdrawn under the November 7 regeneration decision. |
| B04 | deferred | A concrete future consumer and separate decision | Persistent history triage and expiry withdrawn. Current-inventory rerun remains in B02. |

Repeated ordinary test runs must add zero retained executable copies after settlement. Scope
working reports to current use and retire them with their actual owner.

## Execution checkpoint — November 7

The private tool controls have one stable image per active operation. The original report and
inventory commitments are sufficient for historical selection and a new inventory lookup.
The first release slice uses the foreground owner's published result and complete physical
settlement, including the absence of an enclosing borrower. It limits release to the two fixed
inventory controls or the fixed compiler adapter. The settled foreground test path also
releases compiler-captured executables identified by the authenticated original inventory.
A native SurrealKit test left the original inventory after both control copies and sidecars
were released. Its selected evidence passed independent descriptor readback. The targeted host
batch retained 6.7 MiB in its inventory directory after image release. It exercised refusal of
changed bytes, interrupted-release reconciliation, and failed rerun from a fresh inventory
after old executable disposal. A separate native rerun resolved one real original failure
against the current build and passed with complete settlement. Detached work and borrowed
scopes require their own integration.

The isolated SurrealDB pilot was exercised on the pinned server. The November 7 maintainer
decision replaced that path with regeneration. The prototype was never deployed and its
source was withdrawn from the active checkout.
