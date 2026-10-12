---
title: Plan 36 exact resource metadata and terminal freeze
review_date: 2026-10-11
tier: change
purpose: target
standard: core-3.4 + process-simulator-1.5
baseline: 2d6bf6ae7d03a8a9f6f109696bfe3d9cc8455466
disposition_owner: docs/plans/36e-build-test-and-development.md
---

# Plan 36 exact resource metadata and terminal freeze

## 1. Scope and decision

**Accept the corrected target design.** Behavioral adequacy and architectural fitness are satisfied at the **Proposed / Interface-checked** design level. This accepts ADR-0174, the associated blueprint §24.1 amendment, and Plan 36e's exact metadata/terminal contract after the cutover-order correction below. It does not establish an implemented or tested v2 registry.

The independent reviewer assessed the bounded change against Core 3.4, Heuristics for Efficient Architecture 1.0, process-simulator 1.5 and the pse-arrow binding selected by `standard.toml`. Baseline is HEAD `2d6bf6ae7` plus the current uncommitted proposal and implementation context inspected on 2026-10-11. Current policies were examined as design context and rule impacts, not judgment criteria. The disposition owner is Plan 36e P-E1/P-E2, under Plan 36's coordinator. R36-01/R36-02/R36-03, the complete Plan 36 series, scientific algorithms and build topology are excluded.

The functional target is exact ownership and bounded coordination for disposable test resources and retained reports, with authentic selected-invocation outcomes, crash-safe registration/freeze, resumable projection and preserved effect/drain obligations. Credible variation comprises another resource instance or runner composition, increasing cold history with fixed live work, interruption between publications, and replacement of metadata storage. No capacity or latency claim is accepted.

Inspected owners and consumers: `scripts/test_resources.py` registration, exact completed lookup, reports/references, finalization, eligibility and reclaim; `scripts/host_admission.py` metadata/lock/version handling; `scripts/test_run.py`; `scripts/python_tests.py`; selected native Python/Rust paths in `scripts/native_tests.py`; `scripts/surreal_server.py::require_borrowers_drained`; and `python/pse/tests/conftest.py::canonical_substrate`. Development investigation and the abstract freeze probe were read as leads, not accepted as production qualification.

## 2–5. Ownership, contracts and distinguishing scenarios

`test_resources` remains the semantic owner of resource identity, test association, retention/pins, borrowers, references, drain and disposal eligibility. The storage mechanism owns exact files, indexes, stable exclusion and durable publication. Runners supply authenticated selection and outcomes through one selected terminal composer; fixtures supply creation and drain facts. Service ownership still supplies generation, admission and physical effect checks. Neither storage indexes nor fixtures acquire authority to declare test success.

The target deliberately separates ResourceId, normalized logical identity, invocation nonce, service generation, test identity and frozen terminal identity. Hashes select index paths; stored full identities must agree. One `resources/<id>.json` remains writable authority through phase changes. Active markers and invocation/identity access paths identify authority rather than carrying independently editable retention policy. A cold exact read need not enumerate history.

Registration under invocation → identity → resource exclusion verifies Open and durably publishes Pending membership before resource authority, then publishes the identity index and Registered membership before returning a capability. This gives recovery an exact discovery path even after resource publication but before registration acknowledgment. A proven metadata-only never-publication can roll back; uncertainty cannot prove absence or drain. Freeze resolves pending membership, closes additions and commits one immutable authenticated terminal identity. Projection then releases invocation exclusion and updates exact resources idempotently against that identity. There is no consumed promise that all resource dispositions become visible simultaneously.

The accepted registration's creation/effect borrower survives a concurrent freeze. Terminal projection can record the test disposition while effects are unsettled, but cannot release the borrower, prove physical drain or authorize removal. Bare unassociated resources remain incomplete/pinned. This preserves the important legitimate case of creation already accepted before an interrupted runner freezes its result. A refusal after freeze precedes new resource effects; an effect already issued remains with its identified owner.

One composer binds catalog/selection, workload exit or signal, complete/partial authentic report evidence, per-test dispositions, reconciliation and owned drain. Missing reports cannot infer Pass. Nested observer children consume selected ownership; existing process owners retain signal forwarding and opaque work drain. Workload outcome, reconciliation/cleanup errors and owned drain are distinct. Report artifacts, membership and diagnostic evidence persist until projection and their reference/retention obligations release them.

All lock acquisition consumes the original absolute deadline. Resource-first operations never acquire invocation exclusion. No metadata lock spans database creation/removal, service work or physical drain. Stable lock files are not unlinked during ordinary cleanup, avoiding separate inodes for concurrent openers. Cleanup follows active/unresolved paths; exact operations do not scan unrelated live or historical authorities. Necessary invocation projection remains proportional to that invocation's members.

| Scenario | Expected target behavior | Design assessment |
|---|---|---|
| S01: duplicate logical registration and hash collision | Exact exclusion and full identity comparison reject conflicting ownership | Sufficient; hash equality never supplies ownership |
| S02: registration races freeze in both orders; interruption after each publication | Pending membership makes accepted/uncertain resources discoverable; frozen phase refuses new registration | Sufficient ordering and fail-closed recovery contract; actual-file controls outstanding |
| S03: accepted creation completes after freeze | Retain effect borrower/charge; terminal disposition does not imply drain | Sufficient distinction and preserved ownership |
| S04: interrupted projection or nested runner interruption | Resume the same frozen terminal identity; one composer; no newest-report/PGID substitution | Sufficient contract; actual interruption controls outstanding |
| S05: fixed live work, growing cold history | Exact lookup and active/unresolved enumeration avoid historical scans | Structurally credible; no measured latency claim |
| S06: old writer waits across cutover; crash between publications | Durable refusal fence precedes activation; fenced/unactivated recovery resumes staged imports and never serves writable v1 | Corrected during review; see R36M-F01 |
| S07: borrower/reference acquisition races reclaim | Resource exclusion settles the race; service generation/admission and actual drain still govern effects | Policy preserved; all consumers must migrate together |

The process-simulator profile is loaded, but no physical formulation, property model, well-posedness analysis, numerical stage or solver outcome mapping changes in this boundary. Physical-semantics tables, numerical stage columns and model conformance suites are therefore not applicable. Relevant neighboring simulator behavior is interruption/drain and honest retention of qualification evidence, not scientific acceptance.

## 6. Architectural assessment and gates

These verdicts assess the corrected target, not the current v1 implementation.

| Foundation | Verdict | Grounding |
|---|---|---|
| AP-01 Separation of concerns | satisfied | Resource policy stays with one owner; storage and runner composition have distinct responsibilities |
| AP-02 Stable contracts | satisfied | Exact operation contracts hide backend files; explicit versioned cutover preserves consumers and uncertainty |
| AP-03 Composition | satisfied | Fixtures, reports, runner terminal evidence and service drain compose without independently finalizing the same invocation |
| AP-04 Domain model/authority | satisfied | Pending/Registered/Frozen, terminal disposition, effect borrow and drain encode consequential differences and govern target operations |
| AP-05 Explicit structure | satisfied after correction | Ordered publications, lock/deadline rules, stable lock lifetime and fenced/unactivated recovery state are explicit |
| AP-06 Local reasoning/testability | satisfied | Temporary registries and controlled service observations exercise metadata responsibility without a running scientific system |
| AP-07 Execution fit | satisfied | Identified dependencies replace whole-ledger work; short exclusion and resumable per-member projection fit independent resource operations |

| Gate | Verdict | Evidence/scope reason |
|---|---|---|
| G1 Authority | pass | One resource authority and one immutable invocation terminal; indexes remain access paths |
| G2 Semantic fidelity | pass | Migration preserves association, identities, references, pins, manifests and uncertainty |
| G3 Validity | pass at design level | Conflict/corruption/unknown version refuse; Pending recovery cannot infer successful absence |
| G4 Hidden behavior | pass | Explicit registration/publication/effect boundaries; inspection does not become a terminal owner |
| G5 Consistency/recovery | pass after correction | Pending-before-authority, exact freeze/projection, effect ownership and refusal-fence-first cutover |
| G6 Transformation/reuse | pass | Projection binds the same terminal identity; cold promotion and migration do not redefine resource meaning |
| G7 Truthful claims | pass | Proposed design is separated from implementation controls; incomplete evidence remains incomplete |
| G8 Library leverage | pass | Built-in filesystem/locking primitives are composed for domain transitions; SQLite is explicitly compared rather than excluded |
| G9 Architectural fitness | pass | Applicable foundations satisfied for the corrected bounded target |
| PS-G1 / PS-G2 / PS-G3 | not applicable | No physical semantics, problem formulation or numerical result interpretation changes |

## 7. Finding and correction

<a id="r36m-f01"></a>
### R36M-F01 — Legacy refusal must be durable before v2 activation

**Priority: high. Principles/gates: AP-05, DP-19, DP-24, G5/G9. Scenario: S06.**

**Interface-checked diagnosis:** The initially inspected ADR-0174 and Plan 36e said to publish activation plus an unsupported-version sentinel after imports agreed, without ordering those durable publications or defining recovery between them. `host_admission.metadata` takes the legacy `.lock`, reads `allocations.json`, and accepts version 1 through `validate_ledger`. Exclusion ends when a crashed migrator's descriptor closes. If activation became durable before the sentinel, a waiting legacy writer could then read and mutate the still-valid v1 ledger while new consumers used v2. A lock alone cannot make two durable writes atomic across a crash.

**Required correction:** After imports validate under legacy exclusion, durably publish the legacy refusal sentinel before v2 activation. The fence-published/activation-absent state must keep old writers refusing and let only the new owner resume/validate the same staged imports under exclusion before activation. Recovery must preserve ambiguous records and evidence and must not restore writable v1. The affected surface is ADR-0174, its §24.1 target contract and Plan 36e cutover; this does not require SQLite or a general journal.

**Readback:** The coordinator amended all three target owners during this review. The reviewer read the revised paragraphs: they explicitly require durable sentinel-before-activation, fenced-but-unactivated recovery of staged imports, legacy exclusion and no writable v1 recovery. This closes the target ambiguity. The original uncorrected target was Not Accept because G5/AP-05 were unresolved; the corrected target is Accept. Current disposition and implementation status belong only to Plan 36e.

**Implementation verification remains required:** Inject failures before sentinel publication, after durable sentinel but before activation, and after activation; restart both the new owner and a legacy writer waiting on `.lock`. Assert refusal of every legacy write after fencing, preserved imports/evidence, and exact resumed activation. Include fsync/replace failures and directory durability, not only exceptions after a successful write.

No additional consequential target finding was established in the inspected boundary. This is not an absence claim about the whole subsystem or series.

## 8–9. Alternatives and library fit

The whole-ledger baseline unnecessarily serializes independent exact operations and still lacks a registration/freeze fence. The selected per-record design is the simplest viable target among the examined options when independent dispositions need not commit together. It adds explicit pending/index/recovery ordering, but retains exact independent lock domains and avoids inventing simultaneous bundle visibility.

SQLite is a credible library-owned alternative, with transactional atomic commit and indexed lookup. SQLite permits only one writer at a time; WAL allows readers alongside a writer rather than independent simultaneous writers. These capabilities were checked using Context7 `/websites/sqlite_docs`, with official [atomic commit](https://www.sqlite.org/atomiccommit.html) and [isolation](https://www.sqlite.org/isolation.html) documentation. The comparison is about contracts and integration, not license or dependency eligibility.

No inspected consumer needs simultaneous disposition visibility over all resources. `reference_report` protects one origin with a borrower, verifies it, retains a reference on that origin and releases the borrower; it is not an atomic two-resource transfer. SQLite would own generic metadata atomicity but still require domain terminal authentication, effect reconciliation, external service drain and legacy-writer fencing. Per-record files therefore remain reasonable at this scope. Reopen the choice before migration if actual controls reveal inseparable cross-resource ownership or an exact recovery invariant that the selected protocol cannot honor. No database performance advantage or per-record latency improvement has been measured here.

## 10. Evidence and limits

**Interface-checked:** Target documents, source contracts and the distinguishing crash/race paths above. **Proposed:** v2 implementation and its filesystem guarantees. Reviewer execution commands: **not_run**; this was a read-only target review. No tests, benchmarks, production cutover, removal, formatting or repository edits were performed by the reviewer.

The development evidence records 50 existing policy controls and ten abstract crash/order cases against a zero-failure target. The reviewer inspected that evidence and abstract model but did not rerun or promote it: it neither tests v2 files nor covers interrupted legacy fencing. The remaining implementation acceptance must exercise real registration/freeze orders, fsync/replace failure prefixes, exact collision refusal, stable lock lifetime/deadlines, projection retries, authentic partial reports, signals/exceptions, accepted late effects, reference/reclaim races, exact cold lookup and bounded active access as cold history grows. No native scientific, parity, whole-product or performance qualification follows from this decision.

## 11. Rule impacts, exceptions and disposition

- **resource-metadata#RC01:** Blueprint §24.1's invocation-resource/terminal metadata convention gains exact resource authority, durable Pending membership, terminal freeze and resumable per-resource projection, with the corrected cutover fence. ADR-0174 explains the rationale; Plan 36e P-E1/P-E2 owns execution and R36M-F01's disposition. Keeping the current convention would leave the late-registration/freeze gap and whole-ledger interference unresolved; removal of the unrelated sweep alone cannot provide the target guarantee.
- No retention/disposal, service admission, native drain, test selection, toolchain or scientific authority rule is relaxed or retired. No SHOULD exception is required. The cutover-order clarification is part of the selected target, not permission for production migration or deletion.

## 12. Decision and handoff

Behavioral/semantic adequacy: **Accept at design level**. Architectural fitness: **Accept at design level**. Overall: **Accept the corrected ADR-0174 / §24.1 / Plan 36e exact metadata and terminal target**.

The decisive evidence is the explicit operation and failure contract, inspected resource/runner/service consumers, and corrected durable legacy-fence ordering. Actual-file and runner acceptance remains open with Plan 36e; accepting this design does not mark P-E1/P-E2 implemented or tested.
