---
title: Test ownership and evidence retention
status: done
date: 2026-10-08
adrs: [ADR-0160]
review_sources: [docs/design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md]
scenario_sources: [docs/design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md#4-revealing-scenarios]
---

# 30c: Test ownership and evidence retention

## Responsibility and foundation

This companion implements WP03/WP04/WP06 and the prospective AE-25 follow-up.
[Plan 30](30-websocket-and-persistent-agent-environment.md#finding-dispositions) owns
their status. Test selection and pass/fail remain nextest/pytest and the existing
`native_tests`/`validation` terminal composer; scientific run retention remains the
canonical retention owner. Resource declarations associate those authorities rather
than replacing them.

**Interface-checked:** `FixtureLifetime::drop` can execute before a later Rust assertion;
`ManagedStudyFixture.__exit__` can execute before pytest's final outcome. Both currently
dispose after drain without a complete passing-test premise. Abrupt exit may leave
resources, while normal failed assertions erase them. Python's persistent state path
does not supply Rust's UUID-database isolation.

The existing runner captures actual selection, reconciles JUnit/terminal outcomes and
records immutable assessment outputs. `validation_receipts::reuse_checks` preserves
original artifact paths/digests. These are suitable suppliers for terminal disposition
and evidence reference protection. Their disagreement/missing evidence stays a failure,
not inferred success.

## C0 — One final outcome and explicit resource association

Register every disposable mutable fixture with an invocation nonce, the runner's exact
selected test identity, service generation, opaque database/control-directory identity,
receiver/cgroup owners and retention class before initialization can create effects.
Use existing Rust nextest identities and Python node IDs; do not infer them from a
database prefix, PID, filename or display name alone. Resource registration does not
add an acceptance manifest or select tests.

Extend existing runner integration to supply this context and finalize its resources
after terminal reconciliation. Assessment already selects `terminal-owner=assessment`;
preserve that single-owner arrangement. Native wrapper ownership applies only when it
is the selected terminal owner. Migrate ordinary unit/component recipes and supported
direct nextest/pytest wrapper routes so the rule is not limited to full assessment.

For direct bare tools without a validated invocation/test association, fixture resources
remain incomplete/pinned. Report their location and release route; do not invent a
passing result from process exit alone. Small pure tests need no database or resource
registration. C0 source-checks the actual pinned runner identity surfaces before using
an environment variable or event shape; present APIs are not assumed from documentation
for another runner release.

The finalizer consumes existing selected/terminal records plus resource/drain facts.
Test failure includes setup/call/teardown failure, abnormal runner exit, collection or
selection mismatch, interruption and missing terminal results. Successful drain is
necessary but not evidence of test success. A cleanup error remains a reported failure
and pinned resource even when the test body passed.

C2 can first deliver outcome reconciliation, drain and pinning. Automatic deletion
requires C3's working reference-eligibility slice (or the already implemented
equivalent owner guard); unknown reference eligibility retains the resource. C3
supplies that slice from C1's registry and the existing receipt graph before its
later release/reclaim API consumes C2 disposition. This is an early prerequisite,
not a whole-package cycle.

## Mutable context and public-boundary integration

The default database-backed Rust/Python test receives a unique database on the selected
service. Concurrent invocations and xdist workers cannot share mutable schema, problem
heads/history or runtime session selection accidentally. Tests intentionally checking
shared-state concurrency explicitly borrow one owned shared context.

Reuse immutable authored inputs and qualified products where their contracts permit.
Schema installation occurs once per new test database, using the current definite-
conflict retry and explicit create/open distinction. Avoid repeated server startup,
imports masquerading as clones or a cached successful result used as fixture authority.

Pass the explicit context through `CanonicalOptions`, test helpers and native Python
runtime construction rather than rewriting shared server configuration or process-wide
environment selection from concurrent tests. The native Rust/Python boundary validates
the selected context; Python does not mint unverified receiver, schema or scientific
capabilities. Existing scientific authoring/result APIs retain their meanings. If the
needed exposed boundary changes a governed contract, schedule its ADR/design amendment
before migration.

## Disposition, drain and reclamation

Resources begin **incomplete/pinned**. Fixture Drop/context exit releases borrowers,
closes channels and requests/drains owned execution as appropriate; it registers those
facts without removing stored evidence. A passing terminal test result arrives only
from C0's reconciler.

| Terminal/resource facts | Disposal behavior |
|---|---|
| Reconciled pass, disposable fixture, completed reader/native/cgroup drain, no protected reference | Automatically remove its owned database and disposable controls; retain the minimal outcome/provenance/reclamation receipt. |
| Failure, interruption, missing result, unverified drain or cleanup error | Pin resources and diagnostic locations until explicit release; independent native drain still proceeds. |
| Explicit scientific history/operator retention | Use canonical withdrawal/retirement and its reader/root protections; test success alone grants no disposal. |
| Successful assessment referenced by another retained receipt | Protect original reports/artifacts and transitive origin references; latest links and age do not authorize deletion. |
| Explicit release after failure/incomplete state | Remove the retention pin, then perform the same ownership/reference/drain checks; release is not authority to kill unrelated work or bypass scientific readers. |

Whole disposable databases have a simple owned removal path after the conditions above.
Retained domain results continue through study/analysis withdrawal, run retirement and
bounded reclamation. Do not rebuild their tombstone/root/reader semantics in filesystem
cleanup. Source reclamation remains a separate canonical owner.

Preserve compact immutable test outcomes and evidence necessary to interpret them.
Disposable bulky payloads can be removed automatically after success; retention of
reports required by reuse is not waived. Unknown historical reports, directories and
current failure materials start protected. Any assessment-output cleanup must consume
the existing origin/artifact reference graph; do not sweep `build/assessment` by age.

Reference acquisition/publication and disposal coordinate through the same ownership
boundary. A reuse reader registers its borrow before accessing an origin; publication
of the new receipt retains the original reference before releasing that borrow.
Reclamation cannot race a new reference into a deleted origin. Unregistered historical
origins remain protected; corrupt or missing reference information refuses disposal.

Add list/status, explicit pin/release and resumable reclaim operations to the existing
agent-facing boundary. Status explains owner, final disposition, surviving work,
protected references and next eligible action without exposing credentials. Release
selects exact owned identities. Reclamation rechecks eligibility before each bounded
page/removal; crash mid-cleanup resumes honestly and never converts a missing file into
proof of a passed test. Large failure payloads stay pinned until operator release,
even under disk pressure. Disk admission may refuse new work rather than delete them.

## Recoverable exclusion

Replace the empty qualification `O_EXCL` file with short kernel-held exclusion and
records bound to nonce, PID/start generation and actual unit/cgroup ownership. Use
the existing supervisor's stronger patterns. Never hold a metadata lock across a
native solve, remote query or descendant drain.

Recover allocation/exclusion only after confirming that the previous owned descendants
are drained or still accounted for. A dead parent, unlocked descriptor or reused PID
does not prove the native child ended. Recovery leaves failed/incomplete evidence
pinned. No age-based lock stealing or automatic stale-failure deletion is allowed.
The reference fixture retains genuine exclusive lane ownership; functional contexts
need only their own context/receiver exclusion.

## Packages and acceptance

| Package | Inputs and delivered behavior | Migration/deletion and focused acceptance | Status |
|---|---|---|---|
| C0 — Runner/resource contract | R0 policy route; existing selection/terminal owners and actual pinned runner IDs. Design registration/finalization and unassociated-tool behavior. | Assertion after fixture Drop, pytest setup/call/teardown errors, duplicate names, missing/contradictory results and terminal-owner selection. No new test selection or pass/fail ledger. | Implemented; focused controls and selected D3 handoff Tested |
| C1 — Isolated registered contexts | C0; B0/B1 service selection; A1 bound connection; D1 admission. Register before creation and pass context explicitly to Rust/Python. | Migrate canonical fixture helpers, Python conftest/native runtime, managed fixtures and unit/component runners. Test concurrent invocations/xdist with identical problem names, intentional sharing and initialization failure. Delete ambient shared mutable fixture selection. | Implemented; focused controls and selected D3 handoff Tested |
| C2 — Final disposition and ownership recovery | C1; existing result-reader/native/cgroup drain and terminal composer; C3 early reference guard before any disposal. Finalize successful disposal and retain failure pins; recover abandoned exclusion. | Test normal failure after successful science, failure after Drop, abrupt exit, surviving descendants/PID reuse, collection failure, unknown reference eligibility and cleanup interruption. Remove destructor-success inference, unconditional fixture-directory deletion and empty-lock qualification. | Implemented; focused controls and selected D3 handoff Tested |
| C3 — Reference-protected release/reclaim | C1 registry plus canonical retention and assessment origin/artifact references supply the early disposal guard; C2 disposition enables later exact status/release and bounded resumable cleanup. | Test referenced successful evidence, new-reference/reclaim races, retained analysis/live reader, explicit failed release, corrupt/missing reference and interrupted reclamation. Remove only the replaced fixture cleanup mechanisms; preserve existing materials. D3 owns assembled acceptance. | Implemented; focused controls and selected D3 handoff Tested |

Use owner-local runner/receipt/supervisor tests and targeted `pse-operations` lifetime
tests, plus scoped pytest fixture controls. Canonical controls use actual owned
databases; filesystem tests use disposable inputs with known contents. A mocked
terminal pass is insufficient to qualify the real xdist/nextest finalization path.

## Verification and checkpoint

**Proposed:** after C1–C3, a normal failed test retains the same relevant state/logs as
an interrupted test; a successful disposable test reclaims only its owned unreferenced
payload after drain. Reused reports remain valid at their original paths/digests.
Parallel tests interfere only where sharing is deliberately declared.

D3 completed the selected new-scope composed verification and
scope-end checks; the old Python failure-only continuation remains at 28e and is not
replaced by this fixture plan. The focused checks below establish their named scope,
not every package acceptance journey.

**Tested:** `scripts/pse-env --resource-class light -- .venv/bin/python -m unittest
scripts.tests.test_test_resources scripts.tests.test_plan30_routing` passed 45 controls
(33 resource and 12 routing controls), zero failures against the zero baseline.
Resource controls cover terminal disposition, manual pins, references, report path
ownership, fair bounded cleanup selection, service admission and stale configuration.
Registration and cleanup claims refuse live lifecycle reservations; a drained fixture's
live cleaner still prevents destructive service lifecycle work after claim locks release.

**Tested:** the same light wrapper with `PSE_PLAN30_C_ACTUAL` selecting the newly owned
`build/plan30-c-actual-20261009-collection-6` directory exercised
`scripts.tests.test_plan30_resources_actual.ActualResourceControls.test_actual_runner_ownership_and_retention`
under the actual-control module command. This control passed, zero failures against the
zero baseline. Two concurrent pytest invocations with
two xdist workers each registered four isolated contexts for identical node IDs. Passing
disposable controls were removed; assertion after context exit retained a failed/drained
resource, and abrupt exit retained an incomplete/undrained resource. A real reference
borrow protected evidence; exact release followed by interrupted-cleaner recovery
compacted its owned payload. Service configuration was a filesystem-control stand-in;
this journey did not execute a canonical database or native science.

**Tested:** the light wrapper with `PSE_PLAN30_C_COLLECTION_ACTUAL` selecting
`build/plan30-c-nested-collection-20261009-2` and `-m unittest
scripts.tests.test_plan30_resources_actual.ActualResourceControls.test_nested_collection_preserves_parent_catalog_and_later_fixture_association`
passed one actual subprocess control, zero failures against the zero baseline.
Three parent pytest tests and one nested subset test passed; the parent catalog and
enumeration bytes remained unchanged, its actual collector identity remained the owner,
and four resources in that invocation received passing disposition and cleanup, including
the later parent fixtures. The preceding combined actual-control command had one passing
xdist control and one nested-driver receipt lookup error; that lookup was repaired and
only the nested control was rerun. Failed materials remain preserved. This control uses
filesystem fixtures and exercises no native science.

**Tested:** `scripts.tests.plan30_context_overlap_run prepare/run`, using a compile
preparation followed by the admitted `exclusive-observer` route, passed the actual
Rust and Python canonical-fixture controls: two passes, zero failures against the zero
baseline. `build/plan30-c1-overlap-20261009-2/result.json` records two distinct registered
databases on the selected functional service, overlapping fixture lifetimes, isolated
mutations of the same authored table/record name and both databases removed after
reconciled pass and drain. The prepared native Python path/hash matched the extension
actually imported by the Python child. This was fixture/schema and codec isolation,
without scientific execution.

**Tested:** `scripts/pse-env --resource-class light -- .venv/bin/python -m unittest
scripts.tests.test_plan30_routing` passed 12 controls, zero failures against the zero
baseline. Prebuilt observer routing retains caller selection/features and terminal
ownership; managed Rust and both managed Python launchers bind their selected worker
before reference-state admission and restore ambient binding on exit. The test-only
Rust thread stack defaults to the capacity policy's 16 MiB; explicit positive finite
byte overrides survive nested execution, and provenance records the effective setting.
Production native-job stack policy is unchanged. These launcher
controls do not qualify the native scientific journeys launched by D3.

## Implementation checkpoint — 2026-10-09

Runner invocation/test identities now register opaque database/control resources before effects. Fixture exit records drain; existing terminal reconciliation governs disposal. Failed and incomplete resources, manual pins and protected evidence references survive. New report compaction preserves compact receipts and waits for registered groups to drain; exact reference release is explicit.

Registration and database/control cleanup claims now share the service's short
context-admission gate. A lifecycle reservation cannot miss a newly published native-free
borrower or an active cleaner of a drained fixture. Removal and readiness IPC remain
outside metadata locks. Focused controls and the actual pytest/xdist filesystem journey
exercise this composition. Actual Rust/Python canonical-fixture overlap now verifies
separate registered databases, same-name probe isolation, terminal cleanup and the
Python child's native artifact association.

Python collection now publishes the owning invocation's catalog once under short
kernel-held exclusion, bound to the actual collector PID/start generation and boot.
A nested collector verifies its subset without rewriting the parent catalog or
enumeration; the owning collector and xdist controller must retain exact agreement.
Xdist workers retain JUnit node-ID properties while the controller owns publication.
Actual nested collection and the existing xdist journey exercise later fixture
association and terminal cleanup with this authority boundary.

D3 completed the selected native interruption, retained-read consumer journeys and scope-end checks. The parent Outcome records their conditions and exclusions; the focused controls alone are not the assembled acceptance claim.

## Outcome (recorded after implementation)

**Implemented:** this packet's target mechanisms and required consumer migration are
complete. **Tested:** the focused controls above and the selected assembled D3 journeys
passed against the zero failure baseline under their recorded conditions. The
[series Outcome](30-websocket-and-persistent-agent-environment.md#outcome-recorded-after-implementation)
owns the repaired composite results, commands, current storage restart proof and
qualification exclusions. Earlier failed receipts retain their original outcome;
there was no restart of the former full Python suite or broader Plan 28 campaign.

A mistake made and corrected, and deliberate deviations, are recorded at that same
series Outcome with their owning repair. Enduring contracts and operation guidance
live in blueprint §20.6/§24.1 and the substrate/environment/validation guides. This
completed handoff remains while retained Plan 28/29 readers depend on it; it is not an
active implementation backlog. Benefit measurement and the conditional topology
investigation retain their explicit authorization and observable triggers.
