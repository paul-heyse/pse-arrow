---
title: Test assurance architecture target
status: review
date: 2026-11-08
tier: design
purpose: target
focus: holistic-unconstrained
doc_role: review
doc_topics: [design, validation, preparation, environment, numerical-execution]
standard: core-3.4
profile: process-simulator-1.5
baseline: dab55ccca2dd177792bea5469ce7798f93ad2deb
audit_publication: 0ecfec538f216290a1e642b0619f0068566e18b1
audit_source: dab55ccca2dd177792bea5469ce7798f93ad2deb
evidence: Interface-checked
decision: Revise
---

# Test assurance architecture target

**Revise toward owner-local tests composed through shared selection, effect and evidence contracts.**
The target preserves Cargo and nextest as the native test authorities, independent scientific
oracles, the split between database contract and physical service tests, and authentic result
custody. It fixes the joins where a test can be selected without the right prerequisite, where an
observation can accept the wrong failure, or where a narrow operation repeats broad work.

## Scope and baseline

The functional target is reliable, discriminating evidence for simulation, editing and re-solving,
studies, recycles, dynamics and extension, with focused feedback and honest assembled qualification.
Scope includes test bodies, doctests, fixtures, generators, independent references, and the routes
through build, execution, observation and physical settlement. Production owners enter the review
where their contracts govern assurance. Sources were inspected at `dab55ccca`. Concurrent edits to
the managed workflow and borrowed receiver were excluded. The [audit](../test-audit/README.md) was
refreshed for the committed selection and custody changes at publication `0ecfec538`. Other topic
evidence retains its November 7 source `3419aca2a` and limits. Before publication the excluded
workflow delta landed as `be095a207`. The review reconciled its release scope and the Plan69
acceptance and capture updates through `3799bd4ee`, without moving the rest of the source baseline.

Core 3.4, heuristics 1.0, test-assessment 1.2 and process-simulator 1.5 apply through
[standard.toml](../design_principles/standard.toml) and the repository binding. The review traces
representative cases and reconciles all 72 audit identities. It does not enumerate every compiler,
provider or model combination. Existing mechanisms and findings are **Interface-checked**.
The target, scenarios and benefits are **Proposed**. Named prior runs retain their original
conditions. The review ran no test campaign and establishes no quantitative improvement.
A fresh reviewer challenged the integrated proposal. The review tightened gate judgments,
kept settlement within the original deadline, and identified the full executable closure that
nextest can open before a test-name filter takes effect.

### Starting contracts worth preserving

- Plan69 records ordinary root eligibility, a distinct managed-primary profile and authenticated
  result custody after native drain. A passive candidate, an admitted compiler graph, the
  nextest-selected cases and their verdict are distinct facts. Plan69c records ordinary graph
  admission, scoped native controls, exact failed rerun, and settled interrupted-listing release.
  Exact case enumeration and image capture remain distinct from the delivered graph proof.
  [Selection](../../../crates/pse-host/src/selection.rs) ·
  [Plan69c](../../plans/69c-ordinary-testing-and-managed-results.md)
- Plan67c records scoped SurrealKit, Rust service, and prepared worker checks. A generated schema
  assertion, a typed caller, a physical reopen and a worker reaped by its owner establish
  different properties. Native borrowing and the in-process authored source catalog removed
  selected setup work. [Plan67c](../../plans/67c-surrealkit-and-rust-test-consolidation.md)
- The ordinary reference exchange separates original native outcome from independent IDAES
  observations. It validates finite, complete values in their physical context and challenges
  setup failure versus a named overheated trial rejection. Plan66l retains the configured
  scientific population and assembled acceptance. [Ordinary receiving controls](../../../crates/pse-runtime/src/workflow/modeling/ordinary_delivery_tests.rs) ·
  [Plan66l](../../plans/66l-required-population-and-assembled-acceptance.md)

## Target responsibilities and alternatives

| Decision | Authoritative owner | What another owner consumes |
|---|---|---|
| Eligible compiler roots, closure and explicit Arrow validation | Cargo graph and native correctness adapter | Admitted target, host/build and selected dev units or a typed refusal |
| Collected cases, filters and per-run scheduling | Pinned nextest for nextest routes, native Cargo/rustdoc or Miri for their distinct routes | Actual selected identities and effective runner scope |
| Scientific and fixture prerequisites, physical effects and settlement | The native fixture, provider or performing resource owner | Scope and authentic readiness, cancellation, result and release facts |
| Case meaning and independent expectation | Authored test, fixture specification or scientific reference owner | Identified case, typed inputs and bounds, independent evidence and their applicability |
| Managed result and retention | Original workflow and invocation custody owners | Complete selected verdict, raw exit, capture and settlement, each preserved separately |

A **local patch** to each broken recipe, panic or broad read would make several improvements
quickly. It leaves repeated prerequisite interpretation and the same gaps on the next route.
A **common test framework** could centralize policy and setup, but would duplicate Cargo and
nextest decisions, couple unrelated fixture and oracle consumers, and require another lifecycle
and evidence authority. Prefer **composition through existing owners**, with shared mechanisms
only where consumers really share the contract. A new ordinary case should need a local case and
its real prerequisites. A changed solver executable should requalify its actual receiving
boundary. A different numerical expectation should not require a new runner.

Real integration, independent expectations and exact authentication cost work for a reason.
A physical private service is appropriate for migration/reopen, actor and process claims.
Pure policy should run without it. An immutable selected artifact can carry established validity
within an owned traversal. Launch and ownership handoff remain verification boundaries. Compare
complete focused and assembled routes, including recovery and the fate of retained artifacts.

## Recommended changes

### <a id="r01"></a>R01 — Establish actual route coverage and requirements before effects

Compose Cargo unit-graph admission, actual nextest identity, and fixture/qualification readiness
once per relevant stage. Derive case matching from the captured population. Keep effect requirements
with the owner that can refuse an unsafe launch when a rename or omitted association slips past a
filter. Use nextest for per-invocation scheduling. Apply exclusion through the actual shared
effect owner only where two runs contend for the same resource. Keep unrelated private and pure
cases independent and retain the configured ordinary host width.

Use the same version pin for generic and managed nextest. Verify the actual executable selected
for launch, preserving the captured-image authentication where managed nextest claims it.
Keep generic Cargo's fixture-free contract. The pinned nextest metadata model exposes
filter matches and case identity. It does not expose effective group in each case summary. Use
the pinned native group filter where it is needed, or eliminate the query where an existing
effect owner can settle the question directly. Parsing terminal configuration output or adding a
second scheduler would increase policy drift. The exact versions are nextest 0.9.146 and
`nextest-metadata` 0.15.0 at this baseline.
[Nextest pin](../../../.config/diagnostic-tools.toml) · [Discovery](../../../crates/pse-host/src/discovery.rs) ·
[Managed runner](../../../crates/pse-host/src/test_workflow/execute/runner.rs) · [Generic adapter](../../../crates/pse-host/src/correctness.rs)

Make specialist contracts executable. Bind the four parity interpreter selectors in the recipe
from the explicitly prepared locked environment. The managed input roles already recognize these
selectors as caller inputs. Keep ordinary reference-only unit controls distinct from
those that need source-produced native contracts, and give both named selections. Bring local and
CI Miri through their supported command with the required validation. Give the arbitrary AST case
a declared feature selection. Select qualification-module and long-hold cases through their
actual prerequisite, separately from production-module borrowing. Move ordinary compiler and
modeling consumers from the retired capsule role to the selected authored capture in R06.
Keep explicit manual capture for cases whose claim depends on receiving staged bytes.
Classify absent/stale preparation
as a nonpass before effects where it can be observed, and as an attributable setup refusal at the
target owner otherwise. Exclude the marker-gated park and foreground subprocess helpers from
assurance counts on the unmarked path. Their marked parent cases own the behavioral evidence.
Repair descriptions of retired commands.
[Recipes](../../../justfile) · [Native workflow](../../dev/native-workflow.md) ·
[Capsule reader](../../../tests/fixtures/scientific-inputs.rs) ·
[Fixture preparation](../../../crates/pse-host/src/fixtures/execution.rs) ·
[Operations declarations](../../../crates/pse-operations/Cargo.toml)

### <a id="r02"></a>R02 — Exercise real effect owners and compose settlement

Use the production connection and protected-result release contract in logical borrower tests.
Keep disposable namespace/database orchestration outside `CanonicalStore`. Replace the fixture
cleanup-executor substitution with an explicit retained production execution/connection owner
and the same release and drain operations used by the caller. Check production caller drop and
independent borrower drain before removing the special teardown path. Keep private services for
physical migration, reopen, actual wire and process transitions, and SurrealKit for schema and
actor behavior. Consolidate test-side `NativeServiceSpec` construction around the actual prepared
inputs, retaining named intentional policy variations.

Reserve a finite settlement window inside the existing outer deadline. Clamp child and startup
waits to the earlier work cutoff, preserve original PIDFD and service settlement, and dispose
only when the owner can establish drain. At this baseline SurrealKit allows four 30-second
and two 120-second child waits within a service whose original deadline is 300 seconds. The
sum of the allowed waits leaves shutdown with an expired clock in a supported slow path.
For SurrealKit's seed `THROW`, capture bounded attributable failure evidence in addition to
exit/no-report. Retain the independent named assertion and rollout drift controls.
Check the selected nextest profile's outer cutoff too. The default 360-second limit is shorter
than the target-owned repeated-analysis service's 600-second allowance where that profile selects it.
[Canonical protection](../../../crates/pse-operations/src/canonical.rs) ·
[Native service](../../../crates/pse-operations/src/native_service.rs) ·
[SurrealKit suite](../../../crates/pse-operations/tests/native_service/surrealkit.rs)

Replace time-only phase inference with original entry, commit, lease, or process witnesses
where available. Exercise the actual preparation owner after it acquires the recipe lock, using
a controlled cancellable external build stage and asserting no pointer publication and successful
retry. Retain the synthetic lock control at its proper scope. Use a private journal for tests of
admission logic and the real manager for claims about cgroups, ownership and SIGKILL recovery.
At retention, consume actual drain and release authority for target-owned effects. Preserve the
original result pair and Plan69's authenticated expiry and hold checks.
The later unpublished-inventory cleanup releases a settled cancelled listing. It supplies no
test-owner settlement witness for a completed or failed fixture execution.
[Preparation tests](../../../crates/pse-host/tests/native.rs) ·
[Foreground controls](../../../crates/pse-host/tests/foreground.rs) ·
[Custody](../../../crates/pse-host/src/invocation/compact.rs)

### <a id="r03"></a>R03 — Localize helpers and make assertions discriminate

Keep case meaning and required dependency identity visible in test inputs. Use typed authored
construction or an explicit checked overlay for package and lineage changes. Replace unchecked
string rewrites with a failure when the intended dependency or subject is absent. Share the
identical mechanics of residual/source test-body construction where a shared contract can be
stated. Preserve separate equations and deliberately distinct controls. For memo reuse, require
a positive observation of the relevant query and stable observable identity at the supported
boundary before asserting no extra work. Counting two matching zeros cannot establish reuse.

Prefer typed owner outcomes and evidence of entry at the expected boundary to free-text details
or generic errors. Remove fixture-only public seams when normal ownership supports their
consumers. Keep private test-only seams where they provide uniquely discriminating access to a
real stage. A scoped export gate compiled only in tests has a different cost from a public
fixture lifetime API. Move pure refusal and small mechanism controls out of unnecessarily broad
runtime/service construction. [Memo controls](../../../crates/pse-compiler/src/workspace/modeling/granular_tests.rs) ·
[Reference construction](../../../crates/pse-runtime/tests/reference_support/mod.rs) ·
[Support audit](../test-audit/doubles-and-coupling.md)

### <a id="r04"></a>R04 — Separate reference agreement from candidate policy

Keep expected physical quantities, units, scope and precision under the independent case or
oracle. Assert the candidate's own tolerance policy separately. Legacy covariance and sensitivity
must reject a wrong value paired with an inflated native allowance. An independent analytic
or sIPOPT check must also use its own allowance. Keep the ordinary Proto comparator's independent
values and validate its separate native engineering context and units. Bind shared input facts
to the experiment identity, release, conditions and producer. Regeneration and reference agreement
have distinct roles. In particular, attribute the separate 358 K and 378 K cases to their actual
inputs or independently derived bounds rather than a shared 368 K case.

Converge the reference admission and provenance policy while retaining case-specific capabilities.
Verify the locked interpreter and packages and the exact solver used, including an indirect
initializer when it determines the verdict. A second PATH probe cannot attest the actual
instance. Keep independent Python oracles and their distinct sound wire contracts. Do not
require a universal comparator or interchange model. Preserve the pinned IDAES release.
[Covariance exchange](../../../crates/pse-runtime/tests/reference_covariance_exchange.rs) ·
[Sensitivity oracle](../../../tests/reference/oracles/sensitivity.py) ·
[Reference tools](../../../scripts/scientific_reference.py)

Bound oracle diagnostics during execution using the original child owner, with an explicit
overflow outcome and continued drain or an owned cancellation. Bound legacy response production
and check input extent before decoding. Preserve the ordinary wire encoder/readback bounds.
The current `StderrCapture` is stderr-only and its overflow and Park contract do not cover
the finite two-stream requirement. Extend or compose the existing supervision boundary only
after confirming the combined contract. Distinguish a bounded channel from a process-wide
filesystem guarantee. [Ordinary reference capture](../../../crates/pse-runtime/src/workflow/modeling/ordinary_reference.rs) ·
[Supervisor](../../../crates/pse-diagnostics/src/supervised_child.rs)

### <a id="r05"></a>R05 — Challenge the cases that can conceal correlated failures

Use immutable original producer bytes for predecessor receipt and request fixtures where
available. Otherwise use an independently authored, identified specification witness and
state what it proves. Verify version, fingerprint, issuer and original applicability
independently. Keep real data-bearing migration and restart separate from old read-only
receipt acceptance. Exercise final apply after a committed lost acknowledgement, definite
absence, and unresolved readback. A narrow owner-local fault seam can discriminate branches.
Use an original physical commit and interruption witness for the integration claim.

Use bounded generated cases for identities, nullable range/negation and data shapes where
curated examples leave a hole. Exercise canonical pruning against an equivalent unpruned
source with multiplicity and limit ordering checked. Preserve deterministic named controls.
Use exact diagnostic codes at the owners, including delegated errors, and compile-fail examples
with a named compiler failure and a valid neighboring form. Calibrate governance claims to what
their checks discriminate.
[Migration](../../../crates/pse-operations/src/canonical_migration.rs) ·
[Pruning controls](../../../crates/pse-runtime/src/workflow/result_pruning_tests.rs) ·
[Unpruned comparison](../../../tests/engine/tests/pushdown_vs_unpruned.rs) ·
[Taxonomy governance](../../../tests/governance/tests/error_taxonomy.rs)

### <a id="r06"></a>R06 — Match preparation and authentication to the selection

Demand-select the scientific family and its transitive source, data and provider closure
through the authored catalog before reading unrelated bodies. Keep required discovery for
ambiguity, version and missing-input checks. Use the same known consumer associations for
affected preview, with conservative widening where coverage is unknown. Share immutable
preparation only while that closure and its versioned identity hold. Preserve real build
differences for native capability, validation and maintenance. The old scenario-only variants
have retired. For a focused control whose crate-wide build still includes unrelated capabilities,
first locate the actual smaller owner. Move the control only if it continues exercising the same
contract and the admitted graph becomes narrower. A library filter alone cannot deliver that.

Use the existing `verified_tests` traversal for an inventory page and authenticate each selected
execution group over the controls, metadata, runtime dependencies and images it may open.
Nextest can enumerate a binary before discovering that no case in it matches the test-name
filter. Narrow within a group only with trustworthy binary-level membership, including auxiliary
images. A receiving owner independently authenticates what it consumes. Prefer a group-scoped
handoff where an independently verifiable group can preserve selection and launch. Compare it
with borrowing the original immutable artifact under its owner if that preserves the trust
boundary. A smaller hash pass alone leaves whole-inventory parsing at every receiver.
Whole-inventory authentication remains appropriate for a whole-inventory operation. Plan69c
separately owns binary membership and copying/custody during enumeration. Its proposed narrowing
cannot by itself remove repeated authentication after capture.
For membership governance, use pinned `cargo_metadata` 0.23.1's
`MetadataCommand::no_deps` with the real checkout. Reuse one member set for comparisons and
give the stray-directory algorithm a local negative control. Keep dependency floors and
resolved ownership checks on the full graph. These changes reduce structurally repeated work.
They carry no timing or capacity estimate.
[Authored capture](../../../tests/fixtures/scientific-input-capture.rs) ·
[Inventory verification](../../../crates/pse-host/src/discovery.rs) ·
[Membership cases](../../../tests/governance/tests/every_crate_registered.rs)

## Assessment and discriminating verification

The scientific traces keep the following semantics explicit. They establish assurance
requirements for the sampled paths. The model and solver owners retain their formulation
and well-posedness contracts.

| Receiving case | Required meaning and evidence |
|---|---|
| Ordinary reference exchange | State coordinates, physical unit and caloric datum precede numerical comparison. A reported failed trial must identify the intended stage, with a successful setup witness. Native quality and independent agreement remain separate observations. |
| Covariance and sensitivity | Parameter identity, units, perturbation or information basis, and the meaning of each numerical allowance must match the case. Native solve or derivative quality cannot supply the acceptance limit for the independent comparator. |
| Phase and configured population | Retain the admitted material, valid phase regime and active constraints with the named closure or well-posedness witness. Report configured and unavailable populations at Plan66 rather than infer model coverage from fixture discovery. |

| Foundation | Current judgment and condition for the proposed target |
|---|---|
| AP-01 Separation | **Fails in the named seams.** Fixture lifecycle leaks into protection and scenario construction restates dependencies. R02/R03 restore change locality while retaining live lifecycle claims. |
| AP-02 Contracts | **Fails in specialist prerequisites and wrong-reason controls.** R01–R05 give the actual receiving boundary stable identity and refusal semantics. |
| AP-03 Composition | **Fails where child caps and parent deadlines compose independently.** R01/R02/R06 use selected requirements and owner lifetimes without duplicating native runner policy. |
| AP-04 Domain model and authority | **Fails for candidate-owned comparison bounds and historical inputs made by the current producer.** R04/R05 separate case intent, input identity, actual native observation and independent expectation. |
| AP-05 Explicit constraints | **Fails at unchecked requirements, generic refusal and live output bounds.** R01–R05 make missing inputs, overflow and unresolved settlement observable. |
| AP-06 Local reasoning and testability | **Fails for pure claims behind broad fixtures and ambiguous setup verdicts.** R02/R03 retain narrow observable controls and explicit integration where effects matter. |
| AP-07 Execution fit | **Fails for repeated broad acquisition and independently renewed deadlines.** R02/R04/R06 preserve necessary validation and independent checks while narrowing work, live state and recovery to relevant scopes. |

### Behavioral and scientific judgment

| Gate | Verdict for the inspected assurance scope |
|---|---|
| G1 Authority | **Fail.** The reference routes independently enforce different admission meanings for the declared parity environment. R04 separates the shared environment contract from legitimate case-specific capabilities. |
| G2 Semantic fidelity | **Fail.** SurrealKit exit 1 without a report carries multiple possible causes and cannot establish the named seed failure. R02 requires an attributable witness. |
| G3 Validity | **Unresolved.** Production-only fixture readiness does not prove the required qualification input. Check its exact admission point and effect order under R01. |
| G4 Hidden behavior | **Fail.** Logical fixture custody substitutes the cleanup executor beneath the production protection path the case claims to exercise. R02 restores the ordinary contract. |
| G5 Consistency and recovery | **Unresolved.** The existing fail-closed settlement and drain protect named routes. Final-apply recovery after failed readback and the nested deadline's teardown behavior lack the required discriminating evidence. R02/R05. |
| G6 Transformation and reuse | **Unresolved.** Current serialization can agree with itself in a predecessor test. A true predecessor witness and selective authentication controls must establish the supported transformations. R05/R06. |
| G7 Truthful capability claims | **Fail.** Advertised parity and reference-unit scope exceed what the recipes alone execute. R01 checks actual reach and required inputs. |
| G8 Library leverage | **Pass for inspected scheduling, selection and database assertions.** The current design delegates them to Cargo, nextest and SurrealKit. The proposed bounded supervision composition needs its exact API checked under R04. |
| G9 Architectural fitness | **Fail.** The scenario and foundation judgments above independently require revision. |
| PS-G1 Physical consistency | **Unresolved at the broader claimed reference scope.** Selected phase, unit and closure controls support their cases. R04 and Plan66 must tie the physical basis, datum, conditions and release to each required comparison. |
| PS-G2 Well-posedness | **Unresolved at the broader scientific scope.** Named structural and admission witnesses remain valid. Configured population adequacy stays with Plan66. |
| PS-G3 Numerical integrity | **Unresolved for the scientific outcomes at issue.** Candidate-owned allowances make the selected independent agreement claim unsound. They do not establish that any named solver computed a wrong result. R04 must distinguish native stopping and post-solve quality from independent acceptance. |

| Scenario | What would discriminate the target |
|---|---|
| Add or rename an ordinary or specialist case | The actual inventory and declared effect agree. An unprepared qualification case is an attributable nonpass and an unrelated pure case has no provider effect. An Arrow-consuming doctest without the required leaf fails admission. |
| Two overlapping invocations and an interrupted effect | Only a genuinely shared authority excludes the second caller. Original worker/fixture drain or unresolved ownership remains explicit. Independent private work proceeds. A bad seed and an unrelated setup failure produce distinguishable evidence. |
| Wrong scientific answer and historical mutation | A wrong value with an inflated allowance fails. An equivalent valid unit case passes. Old independently identified bytes are accepted only at their original scope. A retargeted receipt and a lost acknowledgement without authentic readback cannot promote state. |
| Focused test, retained page and noisy child | A small selection reads only its source/verification closure. Changed selected bytes refuse while unrelated group bytes do not expand the read. An output flood followed by exit zero yields nonpass with settled custody. An oversized/truncated response refuses before decoding. |
| Governance and diagnostic controls | Stray membership and an omitted stable diagnostic code are caught by their own controls. Dependency graph checks still observe the resolved graph. A compile-fail example rejects for its intended error. |

These are verification obligations for the target design, alongside the actual receiving controls
in Plan69 and the configured scientific population in Plan66. Before selecting the exact
two-stream implementation, confirm overflow observation, resource bounds and Refuse-mode drain
together. For shared exclusion, inspect the actual contention boundary before adopting narrower
or wider scope. For predecessor bytes, confirm provenance before assigning historical execution
credit. None of these questions warrants changing supported scope to hide a gap.

## Audit finding correspondence

This table records proposed remedy judgment. Existing evidence remains bounded to its named
scope. A correction is an open design recommendation. Confirmation is required where exact
configured execution remains with an active owner. Withdrawal and retirement preserve lineage.

| Finding | Judgment | Reason or proposed change |
|---|---|---|
| [DTA-001](../test-audit/oracles-and-acceptance.md#dta-001) | Preserve | Withdrawn in its original scope. R04 addresses the different candidate allowance finding, DTA-061. |
| [DTA-002](../test-audit/oracles-and-acceptance.md#dta-002) | Preserve | Keep the selected independent phase brackets and wrong-interior or missing-slack controls. |
| [DTA-003](../test-audit/oracles-and-acceptance.md#dta-003) | Confirm coverage | Keep the retained PFR, decay and controller regeneration evidence. Match the exact simultaneous-controller case and configuration to Plan66l L01/L03 before attributing executed acceptance. |
| [DTA-004](../test-audit/oracles-and-acceptance.md#dta-004) | Preserve | Keep the independent HiGHS full-status mapping control. |
| [DTA-005](../test-audit/oracles-and-acceptance.md#dta-005) | Preserve | Keep the independent PR jet check and its named derivative scope. |
| [DTA-006](../test-audit/oracles-and-acceptance.md#dta-006) | Preserve | Keep the real mixture input mutation and typed refusal controls. |
| [DTA-007](../test-audit/oracles-and-acceptance.md#dta-007) | Preserve | Keep the scoped finite-value, shape and time/extent checks. |
| [DTA-008](../test-audit/doubles-and-coupling.md#dta-008) | Retire | Withdrawn. Static scheduling controls and actual contextual readiness serve different claims. |
| [DTA-009](../test-audit/doubles-and-coupling.md#dta-009) | Correct | [R03](#r03) makes memo observation positive and stable rather than counting debug labels. |
| [DTA-010](../test-audit/doubles-and-coupling.md#dta-010) | Correct | [R03](#r03) narrows residual/source setup reuse to explicit common mechanics. |
| [DTA-011](../test-audit/doubles-and-coupling.md#dta-011) | Correct | [R02](#r02) makes borrowed protection cleanup representative of the actual product owner. |
| [DTA-012](../test-audit/evidence-routes-and-execution.md#dta-012) | Retire | Original Python verdict path retired. Preserve managed nonpass and original drain/result separation through R01 and Plan69c. |
| [DTA-013](../test-audit/evidence-routes-and-execution.md#dta-013) | Retire | Original inactive Python control population retired. R01 checks current route correspondence under DTA-050. |
| [DTA-014](../test-audit/evidence-routes-and-execution.md#dta-014) | Retire | Original optional CLI outcome retired. Retain current declared native operator-evidence cases at their actual scope. |
| [DTA-015](../test-audit/evidence-routes-and-execution.md#dta-015) | Correct | [R01](#r01) aligns effect and requirement guarantees across their actual routes, retaining distinct Cargo and managed roles. |
| [DTA-016](../test-audit/evidence-routes-and-execution.md#dta-016) | Correct | [R01](#r01) checks name and selection correspondence against actual case identity and effect admission. |
| [DTA-017](../test-audit/evidence-routes-and-execution.md#dta-017) | Preserve and verify | Old bare Python capture retired. [R01](#r01) preserves actual force-validation and adds the Arrow-consuming doctest admission witness. |
| [DTA-018](../test-audit/execution-efficiency.md#dta-018) | Correct | [R02](#r02) and [R03](#r03) reduce broad preparation for narrow controls and preserve physical lifecycle claims. |
| [DTA-019](../test-audit/evidence-routes-and-execution.md#dta-019) | Preserve | Keep the scoped unused-helper closure and current dev-only testkit boundary. |
| [DTA-020](../test-audit/oracles-and-acceptance.md#dta-020) | Preserve | Keep the named nonlinear curvature and reopened covariance cases in their native fixture scopes. |
| [DTA-021](../test-audit/oracles-and-acceptance.md#dta-021) | Preserve | Keep the named countercurrent and unequal-pressure physical controls. |
| [DTA-022](../test-audit/test-execution-system.md#dta-022) | Retire | Independent Python runner interpretation retired. Current native contract divergence remains DTA-063. |
| [DTA-023](../test-audit/test-execution-system.md#dta-023) | Correct | [R01](#r01) replaces references to obsolete routes and checks that remaining specialist routes select the intended work. |
| [DTA-024](../test-audit/test-execution-system.md#dta-024) | Correct | [R01](#r01) extends exact nextest identity to generic nextest without changing Cargo's role. |
| [DTA-025](../test-audit/evidence-routes-and-execution.md#dta-025) | Correct | [R01](#r01) and [R03](#r03) separate prerequisite/setup nonpass from a failed assertion and retain settlement independently. |
| [DTA-026](../test-audit/test-execution-system.md#dta-026) | Retire | The Python reciprocal tooling cause retired. Preserve lineage in the broad audit's DA-037 redirect. |
| [DTA-027](../test-audit/doubles-and-coupling.md#dta-027) | Correct | [R03](#r03) checks authored dependency/lineage edits and [R06](#r06) converges case source closure across consumers. |
| [DTA-028](../test-audit/doubles-and-coupling.md#dta-028) | Correct | [R03](#r03) uses typed consumed refusal reasons and separate display assertions. |
| [DTA-029](../test-audit/execution-efficiency.md#dta-029) | Preserve | Scenario feature variants retired. Keep actual service and maintenance capability distinctions. |
| [DTA-030](../test-audit/execution-efficiency.md#dta-030) | Correct | [R06](#r06) materializes the selected family closure and avoids repeat union capture. |
| [DTA-031](../test-audit/execution-efficiency.md#dta-031) | Retire | The Python runtime fixture dependency for capability-only checks retired. |
| [DTA-032](../test-audit/execution-efficiency.md#dta-032) | Preserve | Retain compatible compiler-graph reuse at its Plan67d scope. Remaining variants are DTA-053. |
| [DTA-033](../test-audit/execution-efficiency.md#dta-033) | Retire | The Python extension preparation path retired. |
| [DTA-034](../test-audit/execution-efficiency.md#dta-034) | Preserve | Keep one managed source build/list per context. DTA-039 covers repeated verification of the resulting capture. |
| [DTA-035](../test-audit/execution-efficiency.md#dta-035) | Preserve | Keep the sampled removal of scientific package embedding. Generated descriptors establish a different contract. |
| [DTA-036](../test-audit/execution-efficiency.md#dta-036) | Correct | [R06](#r06) reuses known case/fixture associations and preserves conservative widening. |
| [DTA-037](../test-audit/execution-efficiency.md#dta-037) | Preserve | Keep one aggregate codegen comparison with separate focused routes. |
| [DTA-038](../test-audit/execution-efficiency.md#dta-038) | Retire | The Python invocation-state rewrite path retired. |
| [DTA-039](../test-audit/execution-efficiency.md#dta-039) | Correct | [R06](#r06) scopes verification to each owned traversal and consuming execution group. |
| [DTA-040](../test-audit/execution-efficiency.md#dta-040) | Retire | Preserve the earlier rejected blanket aggregate-reuse claim. |
| [DTA-041](../test-audit/execution-efficiency.md#dta-041) | Retire | Preserve the withdrawal of the generic recovery machinery diagnosis. |
| [DTA-042](../test-audit/execution-efficiency.md#dta-042) | Retire | Earlier Python cleanup joins retired. Target-owned settlement remains DTA-060. |
| [DTA-043](../test-audit/execution-efficiency.md#dta-043) | Retire | Earlier shared Python service premise retired. Keep isolated logical databases within the native borrowed endpoint. |
| [DTA-044](../test-audit/execution-efficiency.md#dta-044) | Retire | Former Python canonical barrier retired. Preserve actual effect exclusion under DTA-046. |
| [DTA-045](../test-audit/execution-efficiency.md#dta-045) | Preserve | Earlier broad serialization was narrowed. Keep pure controls outside protection for real effects under R01. |
| [DTA-046](../test-audit/execution-efficiency.md#dta-046) | Correct | [R01](#r01) scopes same-resource contention to the actual owner. Validate overlap across independent invocations before claiming coverage. |
| [DTA-047](../test-audit/execution-efficiency.md#dta-047) | Retire | Earlier generic reclamation path retired. |
| [DTA-048](../test-audit/execution-efficiency.md#dta-048) | Retire | Earlier growing Python checkpoint maps retired. |
| [DTA-049](../test-audit/oracles-and-acceptance.md#dta-049) | Confirm coverage | The native seed manifest route exists. Resolve full configured population and assembled evidence at Plan66l L01/L03. R01/R04 address distinct route and evidence quality. |
| [DTA-050](../test-audit/evidence-routes-and-execution.md#dta-050) | Correct | [R01](#r01) supplies the named parity, reference-unit, Miri, arbitrary, helper, capsule and qualification routes with actual prerequisites. |
| [DTA-051](../test-audit/doubles-and-coupling.md#dta-051) | Retire | The demonstrated Python orphan population retired. Retain currently used Plan14 controls. |
| [DTA-052](../test-audit/execution-efficiency.md#dta-052) | Correct | [R02](#r02) replaces target-owned release inference with original settlement. Retain Plan67/69 scoped custody and the retirement of the old accumulators. |
| [DTA-053](../test-audit/execution-efficiency.md#dta-053) | Narrow and verify | Old scenario-only variants have retired. [R06](#r06) preserves actual capability distinctions and requires a narrower owning graph before claiming reduced focused build work. |
| [DTA-054](../test-audit/oracles-and-acceptance.md#dta-054) | Correct | [R04](#r04) binds reference inputs, release, experiment and claimed result scope. |
| [DTA-055](../test-audit/doubles-and-coupling.md#dta-055) | Correct | [R02](#r02) and [R03](#r03) remove unnecessary fixture-only product seams, preserving private discriminating controls. |
| [DTA-056](../test-audit/oracles-and-acceptance.md#dta-056) | Correct | [R05](#r05) uses revealing generated identities and differential pruning cases while retaining named controls. |
| [DTA-057](../test-audit/evidence-routes-and-execution.md#dta-057) | Correct | [R02](#r02) requires original reached-phase witnesses for cancellation and timing-sensitive lifecycle cases. |
| [DTA-058](../test-audit/oracles-and-acceptance.md#dta-058) | Correct | [R05](#r05) calibrates assurance wording and verifies stable error-code and compile-fail claims. Parity routing remains DTA-050. |
| [DTA-059](../test-audit/doubles-and-coupling.md#dta-059) | Correct | [R02](#r02) and [R03](#r03) unify test-side service inputs and make intended package variation explicit. |
| [DTA-060](../test-audit/evidence-routes-and-execution.md#dta-060) | Correct | [R02](#r02) composes original work and settlement budgets and keeps authenticated disposal. |
| [DTA-061](../test-audit/oracles-and-acceptance.md#dta-061) | Correct | [R04](#r04) moves reference acceptance allowances under independent case authority. |
| [DTA-062](../test-audit/oracles-and-acceptance.md#dta-062) | Correct | [R04](#r04) binds case-specific capability and provenance to the actual interpreter and solver. |
| [DTA-063](../test-audit/test-execution-system.md#dta-063) | Correct | [R01](#r01) shares runner verification while preserving explicit fixture-free and managed contracts. |
| [DTA-064](../test-audit/test-execution-system.md#dta-064) | Retire | Old surviving-route dependency on Python launch tooling retired. Verify current route adequacy under DTA-050. |
| [DTA-065](../test-audit/oracles-and-acceptance.md#dta-065) | Correct | [R05](#r05) uses original predecessor bytes or independently authored specifications with explicit evidence scope. |
| [DTA-066](../test-audit/oracles-and-acceptance.md#dta-066) | Correct | [R05](#r05) discriminates final apply loss/absence/readback and retains a distinct actual commit witness. |
| [DTA-067](../test-audit/oracles-and-acceptance.md#dta-067) | Correct | [R02](#r02) and [R03](#r03) check intended refusal and reached boundary, including SurrealKit setup. |
| [DTA-068](../test-audit/doubles-and-coupling.md#dta-068) | Correct | [R02](#r02) tests actual preparation cancellation and gives marker restoration an affirmative witness. |
| [DTA-069](../test-audit/doubles-and-coupling.md#dta-069) | Correct | [R02](#r02) isolates journal policy controls and preserves the named physical manager journey. |
| [DTA-070](../test-audit/doubles-and-coupling.md#dta-070) | Retire | Python orchestration and supervisor test mechanisms retired. Retained reference tools serve distinct claims. |
| [DTA-071](../test-audit/execution-efficiency.md#dta-071) | Correct | [R06](#r06) narrows Cargo member discovery and keeps resolved-graph checks at their actual scope. |
| [DTA-072](../test-audit/execution-efficiency.md#dta-072) | Correct | [R04](#r04) bounds live child diagnostics and legacy responses with original process settlement. |

## Rule impacts and decision

**Rule impacts: none identified for the recommended scope.** The proposal preserves the native
correctness/force-validation contract, configured ordinary placement, genuine scientific resource
exclusion, independent pinned IDAES parity, and authentic managed exit and release semantics.
Changing the named test, profile and command declarations to realize existing obligations is
within those rules. A later change to any governing contract must name its impact and use the
normal maintainer and decision route.

**Revise.** Current assurance fails for the named routes, oracle and lifecycle boundaries.
Architectural fitness independently fails at G9. The owner-local composition is the preferred
target at **Proposed** strength. The main remaining evidence questions concern the exact
two-stream supervision realization, the actual shared-resource boundary between invocations,
and the availability of attributable historical bytes and a faithful final-edge ACK-loss
control. The scenarios above specify what would settle them.

Plan66l remains responsible for required configured science and assembled acceptance. Plan69c
remains responsible for ordinary admission, exact failed rerun, bounded results and release.
Their relevant accepted evidence should be reused with its original conditions. Plan53 is
historical. If v4 receiving is activated later, carry the same SurrealKit discrimination,
settlement and artifact-identity obligations into its receiving scope. This review supplies one
target and its correspondence to the audit for a future decision about adopting and sequencing
the corrections.
