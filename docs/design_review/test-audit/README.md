# Test code design principle audit

This is the living diagnostic account for test architecture, evidence and shared support.
Start with the [integrated architectural account](architecture-and-assurance.md): it explains
the system, the interactions a local inspection can miss, the contrary evidence and the remaining
improvement opportunities. The topic pages contain the primary findings. This account does not
select remedies, authorize implementation or qualify the test suite.

The [skill](../../../.codex/skills/design-principle-audit-tests/SKILL.md) owns the method;
the [binding](../design_principles/binding/pse-arrow.md#test-code-design-principle-audit) owns
invocation, finding identity and follow-up routes. The test companion interprets the full standard;
it does not bound the assessment.

## Baseline and method

- **Audited source:** `05e6a2b8a49f08de4e33e2e9ad64a753cbea2287`.
- **Date:** 2026-10-23. Publication is a subsequent commit, recorded separately by Git.
- **Included uncommitted changes:** none. Dirty `history.txt` and concurrent product changes,
  including runtime/Python boundary/reduction edits that appeared during this audit, are excluded.
  Decisive reads were made before those edits or against the fixed source revision.
- **Selected standard:** Core 3.4; efficiency heuristics 1.0; test-assessment companion 1.2;
  process-simulator profile 1.5; pse-arrow binding through
  [standard.toml](../design_principles/standard.toml).
- **Lens:** holistic and integrative, with particular attention to efficiency-oriented design
  principles through static architectural review. Every core foundation remains applicable.

The first iteration comprised three passes: companion-directed findings DTA-001–019,
principle-directed findings DTA-020–030, then efficiency findings DTA-031–048. Its latest source
baseline was `7902a1f5be1e46b65f17bb3330b4d67a359bbc73`; earlier passes used `cbda2efc6`.
The corresponding publication is retained in Git, including earlier numerical figures and
judgments that this follow-up corrects. Historical measurements were not refreshed or promoted.

This follow-up first formed an independent architecture picture from contracts, source and
consumers, before comparing it with the finding catalog. Complementary read-only reviews
revalidated execution/evidence and efficiency findings; the coordinator revalidated oracle,
scenario and coupling findings and reconciled all 48. All new judgments are **Interface-checked**.
No tests, mutation probes, benchmarks or profiling were executed.

The source diff since the earlier baseline includes developer-knowledge tooling, its tests and
generated relationships. Core cited test/fixture/runner paths were substantially unchanged.
Changed tooling received attention at consumed input/evidence boundaries, rather than starting
another broad tooling or production audit. Architecture and current policy define intended
obligations; code establishes their realization. Documentation of manual routes is valid
counterevidence, while historical commands/results confer neither current qualification nor
authority to revive retired work.

## Integrated synthesis

The test system has meaningful shared checks and real cross-layer witnesses. It is not a
collection of disconnected unit tests, and using a production report, test-only seam or shared
policy is not inherently circular. Deliberately wrong expectations, missing closure evidence,
contextual routing, actual canonical reopening and independent downstream covariance provide
important complementary challenges.

The additional holistic conclusions concern interactions rather than another list of small defects:

1. **Scientific change reach and preparation use different dependency models.** Package-based
   affected preview, embedded reference lists and broad runtime package loading disagree about
   the scope of a scientific edit. Fixing one list does not repair that whole feedback boundary
   (DTA-027/030/035/036).
2. **Broad preparation amplifies failure before the intended assertion.** A durable fixture can
   exist before unrelated source admission prevents a selected scientific assertion. Setup/product
   diagnostic collapse and coarse failure retention then widen the consequences. The capability-only
   Python case demonstrates the accidental fixture dependence directly (DTA-018/025/030/031/042).
3. **Assurance requirements are fragmented across the composed routes.** Markers, features,
   scheduler terms, preparation tables, qualification scopes and terminal/exit interfaces do not
   have complete correspondence merely because each owner has local tests
   (DTA-012/015/016/017/022/023).
4. **Evidence and resource lifetimes have dual consumers.** Recovery protects uncertain borrowers;
   sealing authenticates reuse. Those guarantees survive even when disposal is unavailable. The
   demonstrated opportunities are repeated immutable payload publication, broad-before-narrow
   source capture and accumulating checkpoint serialization, not wholesale removal of protection
   (DTA-038/039/042/047/048).

The [architectural account](architecture-and-assurance.md#additional-integrative-shortfalls-and-improvement-opportunities)
grounds these interactions and walks model/backend changes, test renames, focused reruns,
setup failures and retained readers. Its portable-reconstruction section records an additional
evidence boundary after an intentional preparation change, rather than inventing a missing
automatic-reuse requirement.

No distinct new structural cause met the threshold for another ID in this representative pass.
The integrative conclusions widen the useful diagnosis of existing causes without duplicating them
or claiming that all possible shortfalls have been found. Improvement properties remain proposals
for a later solution review, not accepted implementation scope.

## Material corrections to the previous account

- **DTA-008 is withdrawn:** explicitly scoped static routing tests and production contextual
  tests protect complementary responsibilities. A test-only readiness seam alone did not
  establish a defective production composition.
- **DTA-041 is withdrawn:** failed/unknown retention, uncertain publication and shared-service
  borrowers give fixture recovery concrete obligations. Disposability and code size did not
  establish that the recovery protocol was unjustified.
- **DTA-001 is unresolved and DTA-002 reframed:** maintainer-selected engineering and historical
  allowances are legitimate acceptance contracts. More precise references do not automatically
  impose tighter criteria. A projection defect was not established against complementary
  controls. Claim-dependent projection and phase discrimination remain distinct inquiries.
- **DTA-013/014/023 correct route claims:** manual capability/signal/context routes exist,
  supervised unit-package prepares special fixtures, and optional CLI work does not make the
  enclosing API tests vacuous. Inactive bodies, optional controls and required campaign omissions
  must not be conflated.
- **DTA-018/031/034/039 distinguish actual preparation scopes:** database-local guarantees,
  durable source recording, ordinary retained Cargo inventory and mutable artifact boundaries
  contradict blanket unnecessary-work claims.
- **DTA-040 preserves aggregate qualification semantics:** later edits can invalidate a final
  snapshot claim without erasing historical executions or forcing every check to rerun.
  Exact historical input attribution requires evidence beyond aggregate snapshots.
- **DTA-044 is unresolved:** separate databases do not alone establish receiver/capacity
  independence. DTA-045 retains concrete pure-control overgrouping; DTA-046 corrects envelope
  arithmetic and distinguishes ceilings from simultaneous demand.
- **DTA-047/048 preserve consumed evidence meaning:** sealed origins authenticate reuse, and
  Cargo graphs are parsed to validate suite/root/executable correspondence, not merely compared
  as opaque equality strings.

Withdrawal and narrowing reflect corrected diagnosis, not code repairs or successful new tests.

## How the standard informed this follow-up

**AP-01 — responsibility and change locality.** The scientific-dependency scenario joins
source assembly, compiled inputs, selection and fixtures. The support problem is the dependency
knowledge and cross-owner changes it requires, not file size or failure to mirror production modules.

**AP-02 — consumed contracts.** Adapter static policy and contextual readiness are legitimate
different contracts. Local helper robustness, private query-name observations and behavior-altering
fixture lifetime paths remain narrower concerns. Actual public-boundary journeys provide contrary
evidence to a blanket untested production shape.

**AP-03 — composition.** The account follows how local scientific tests, shared conformance,
native execution, canonical publication and Python readers join. Its strongest execution defects
are disagreements at those joins, not an arbitrary number of roots or helper abstractions.

**AP-04 — semantic model and authority.** Numerical policy owns its declared criterion;
independent expectations challenge its realization where that is the claim. Test scope, selected
population, executed outcome, aggregate snapshot qualification and reusable historical evidence
have different meanings. Collapsing them produces both false assurance and false audit findings.

**AP-05 — explicit requirements.** Name-keyed scheduling/preparation, differing option readers
and inactive bodies are examined against real consumers. Honest per-definition inventory and
explicit dynamic exclusions are retained as such, rather than relabelled dishonest completeness.

**AP-06 — local reasoning and independence.** Capability-only fixture activation and setup
failure amplification show non-local dependencies. Analytic/exhaustive controls, wrong-expectation
controls, missing-closure controls and reopened scientific results demonstrate useful independence
without requiring every local test to duplicate the full product.

**AP-07 — execution fit.** Relevant DP-09/10/16/19/20 and heuristics about necessary inputs,
physical units, reused validity, live state, coordination and recovery inform the structural
judgments. Source establishes repeated selection-bearing publication, broad preparation and
pure-control serialization. It does not identify the dominant runtime bottleneck, prove a
cache/share alternative equivalent, or establish an OOM from configured ceilings.

**Operational and profile obligations.** DP-01/02/03/05/07/09–24 are applied where semantic
authority, relationships, enforcement, reuse, library integration, effects, diagnostics and
evidence actually enter these scenarios. PS-01–13 remain the scientific criteria for relevant
test responsibilities; this follow-up particularly reassesses derivative/derived-result
discrimination, truthful outcomes and shared/reference checks (PS-07/10/12/13). The earlier
per-principle inventory is not promoted to comprehensive renewed scientific qualification.

## Revalidation outcomes

Each primary section supplies the decisive source, consequence, counterevidence and limit.
Confirmed means the bounded diagnosis stands; it does not endorse every old count or sentence.
Reframed findings may now be evidence boundaries or unresolved tradeoffs rather than established
misalignment. No implementation priority is implied.

| Finding | Follow-up outcome | Current scope |
|---|---|---|
| [DTA-001](oracles-and-acceptance.md#dta-001) | Unresolved | Complementary projection coverage remains unsettled; no route-level defect was established |
| [DTA-002](oracles-and-acceptance.md#dta-002) | Reframed | Historical phase-reference comparisons do not distinguish every phase regime |
| [DTA-003](oracles-and-acceptance.md#dta-003) | Confirmed | Three computed references lack a committed regeneration path |
| [DTA-004](oracles-and-acceptance.md#dta-004) | Confirmed | The HiGHS mapping test does not detect loss of a declared status arm |
| [DTA-005](oracles-and-acceptance.md#dta-005) | Narrowed | The integrated PR jet check shares primitive derivatives with the path it checks |
| [DTA-006](oracles-and-acceptance.md#dta-006) | Narrowed | Opaque-input rejection checks can accept the wrong refusal |
| [DTA-007](oracles-and-acceptance.md#dta-007) | Narrowed | Some numerical helpers lack shape and non-finite discrimination |
| [DTA-008](doubles-and-coupling.md#dta-008) | Withdrawn | Static routing policy tests are not evidence of a defective test-only composition |
| [DTA-009](doubles-and-coupling.md#dta-009) | Confirmed | Reuse observations depend on private query names and some lack positive controls |
| [DTA-010](doubles-and-coupling.md#dta-010) | Narrowed | Implicit fixture construction duplicates preparation mechanics across crates |
| [DTA-011](doubles-and-coupling.md#dta-011) | Narrowed | Fixture lifetime ownership changes the canonical-store path exercised by some tests |
| [DTA-012](evidence-routes-and-execution.md#dta-012) | Narrowed | Managed command exit and reconciled assurance verdict can disagree |
| [DTA-013](evidence-routes-and-execution.md#dta-013) | Reframed | Some controls are inactive or outside declared campaigns; manual routes are not absent |
| [DTA-014](evidence-routes-and-execution.md#dta-014) | Narrowed | Optional CLI qualification has no separate executed outcome in enclosing Rust API tests |
| [DTA-015](evidence-routes-and-execution.md#dta-015) | Confirmed | Sanctioned routes compose different test-specific lifecycle and scheduling guarantees |
| [DTA-016](evidence-routes-and-execution.md#dta-016) | Confirmed | Scheduling requirements depend on names without complete correspondence checks |
| [DTA-017](evidence-routes-and-execution.md#dta-017) | Confirmed | A stale validation declaration is checked while some routes bypass actual leaf verification |
| [DTA-018](execution-efficiency.md#dta-018) | Reframed | A canonical fixture reconstructs a complete integration lifecycle per test |
| [DTA-019](evidence-routes-and-execution.md#dta-019) | Confirmed | Some shared test doubles and declared dev dependencies have no external consumer |
| [DTA-020](oracles-and-acceptance.md#dta-020) | Narrowed | Published KKT-derived results lack a nonlinear-constraint reference case in the inspected portfolio |
| [DTA-021](oracles-and-acceptance.md#dta-021) | Narrowed | Some default model variants lack discriminating reference scenarios |
| [DTA-022](test-execution-system.md#dta-022) | Confirmed | Runner consumers independently interpret selection, profiles and fixture identity |
| [DTA-023](test-execution-system.md#dta-023) | Narrowed | Preparation and qualification selections require name-keyed maintenance |
| [DTA-024](test-execution-system.md#dta-024) | Confirmed | Reuse identities omit consequential generator inputs and test-tool identity |
| [DTA-025](evidence-routes-and-execution.md#dta-025) | Narrowed | Fixture infrastructure failure is reported through Rust test failure and can fan out |
| [DTA-026](test-execution-system.md#dta-026) | Reframed | Reciprocal lifecycle dependencies and several roots obstruct local test-system reasoning |
| [DTA-027](doubles-and-coupling.md#dta-027) | Narrowed | Reference-source dependencies are maintained in several test assembly lists |
| [DTA-028](doubles-and-coupling.md#dta-028) | Narrowed | Free-text modeling refusal contracts couple some negative tests to wording |
| [DTA-029](execution-efficiency.md#dta-029) | Narrowed | Store-test selection is coupled to a feature-dependent production-library variant |
| [DTA-030](execution-efficiency.md#dta-030) | Narrowed | Shared reference preparation repeats a union closure for differently scoped consumers |
| [DTA-031](execution-efficiency.md#dta-031) | Narrowed | Capability-only Python tests acquire a durable fixture through a broad runtime dependency |
| [DTA-032](execution-efficiency.md#dta-032) | Narrowed | Overlapping testing/tooling routes prepare different feature closures |
| [DTA-033](execution-efficiency.md#dta-033) | Narrowed | Focused Python routes unconditionally invoke a broad extension-preparation pipeline |
| [DTA-034](execution-efficiency.md#dta-034) | Narrowed | The unprepared fallback repeats Rust resolution and enumeration |
| [DTA-035](execution-efficiency.md#dta-035) | Narrowed | Embedding shared reference documents widens compile-time change propagation |
| [DTA-036](execution-efficiency.md#dta-036) | Confirmed | Package-based affected-test preview does not represent scientific fixture dependencies |
| [DTA-037](execution-efficiency.md#dta-037) | Confirmed | The grouped codegen comparisons repeat overlapping generation work |
| [DTA-038](execution-efficiency.md#dta-038) | Confirmed | Fixture registration rewrites selection-bearing invocation state under shared admission |
| [DTA-039](execution-efficiency.md#dta-039) | Narrowed | Broad source capture and repeated artifact verification have different validity scopes |
| [DTA-040](execution-efficiency.md#dta-040) | Reframed | Aggregate snapshot qualification has a wider invalidation window than an individual observation |
| [DTA-041](execution-efficiency.md#dta-041) | Withdrawn | Disposability alone does not establish that fixture recovery is overengineered |
| [DTA-042](execution-efficiency.md#dta-042) | Narrowed | Coarse failure retention and cleanup joins amplify later lifecycle work |
| [DTA-043](execution-efficiency.md#dta-043) | Narrowed | Per-run test exclusion does not by itself exclude another run sharing the service |
| [DTA-044](execution-efficiency.md#dta-044) | Unresolved | The necessity of the Python canonical-owner barrier remains unestablished |
| [DTA-045](execution-efficiency.md#dta-045) | Narrowed | A broad scientific-group regex serializes demonstrably pure controls |
| [DTA-046](execution-efficiency.md#dta-046) | Narrowed | Per-runtime ceilings do not establish a safe aggregate concurrent-demand envelope |
| [DTA-047](execution-efficiency.md#dta-047) | Reframed | Current nondisposable reports still traverse generic reclamation machinery |
| [DTA-048](execution-efficiency.md#dta-048) | Narrowed | Checkpoint serialization repeats growing evidence maps and failure payloads |

## Scope, strengths and limits

Inspection followed representative responsibilities across crate tests, tests/support, reference
source/data, conformance, lifecycle, installed Python tests/parity, configuration and the runner/
receipt/resource/server compositions. Source/history searches challenged selected negative claims.
It was not an individual inspection of every Rust/Python test, solver double or bit-exact comparison.

Strengths to preserve include independent analytic/external references, exhaustive small structural
oracles, typed physical refusals and admitted controls, deliberate wrong-result controls, shared
conformance checks, correct separation of original state from solver status, actual changed/reopened
histories, reader/drain ownership, strict marker/parity behavior, explicit validation features,
bounded clocks and truthful incomplete outcomes. Some currently unused test doubles have strong
self-tests without current external adoption; neither that fact nor their existence determines
the necessary product fault-coverage strategy.

Material unresolved questions include smaller sufficient reference closures; case-specific
projection independence; necessity of the canonical-owner barrier; genuinely simultaneous
observer/receiver demand; equivalence of cheaper mutable-path verification; and proportionality
of aggregate snapshot/reclaim scope. No elapsed time, speedup, queue delay, resource-pressure
attribution or fresh local inventory was measured. Existing recorded results retain their original
conditions and are not this follow-up's test results.

Runtime suites, benchmark/profiling campaigns, mutation probes and repository-wide health checks
were **not_run**. Publication uses focused link/anchor/identity consistency and scoped diff checks.
This account establishes no current scientific/product qualification and does not certify complete
alignment with every principle.

## Reading and ownership

- [Integrated architecture and assurance](architecture-and-assurance.md): whole-system picture,
  cross-finding interactions, contrasting controls, scenarios and the portable evidence boundary.
- [Oracles and acceptance](oracles-and-acceptance.md): independent expectations, allowances,
  negative cases, second-order evidence and variants.
- [Doubles and coupling](doubles-and-coupling.md): seams, lifetimes, source construction and refusals.
- [Evidence routes](evidence-routes-and-execution.md): selection, outcomes, lifecycle and setup.
- [Execution system](test-execution-system.md): requirement interpretation and composition.
- [Efficiency principles](execution-efficiency.md): preparation, representations, assurance,
  recovery, scheduling and evidence lifetimes.

DTA-001–048 remain allocated; DTA-008 and DTA-041 are withdrawn and never reused. The next
available ID is DTA-049. DTA-018's former page retains its move pointer. A shared cause already
in the broad account keeps that identity and primary home; links here do not refresh its baseline.
The broad account was consulted for identity/overlap, not adopted as this audit's evidence.

The account owns diagnosis at its stated source. Reviews own remedy alternatives and judgment;
implementation plans own scheduled disposition and closure. Git preserves prior publication,
old evidence conditions and diagnostic lineage. No parallel status ledger, recurring gate or
automatically authorized remediation follows from this refresh.
